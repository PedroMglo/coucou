//! Local conversation removal and the authoritative attachment-reference fence.
//! This never deletes a provider's native sessions or any inbox copy.
use super::*;
use std::{collections::{BTreeMap,BTreeSet},path::Path,time::Instant};
#[path="chat_retention_recovery.rs"]
mod recovery;
pub(crate) use recovery::Summary as RecoverySummary;
const CHANGED:&str="A conversa ou as referências mudaram. Volta a rever a eliminação.";
fn digest(value:&impl Serialize)->Result<String,String>{
    let bytes=serde_json::to_vec(value).map_err(|_|CHANGED.to_string())?;
    Ok(ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect())
}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Owner{pub provider:String,pub conversation_id:String}
#[derive(Clone)]
pub(crate) struct References{pub revision:String,pub owners:BTreeMap<String,BTreeSet<Owner>>,busy:BTreeSet<Owner>,complete:bool}
impl References{
    fn capture(all:&Conversations)->Result<Self,String>{
        let mut owners=BTreeMap::<String,BTreeSet<Owner>>::new();let mut busy=BTreeSet::new();
        let mut add=|provider:&str,cid:&str,history:&[crate::attachment_history::Record]|{
            for record in history{for file in &record.attachments{owners.entry(file.id.clone()).or_default().insert(Owner{provider:provider.into(),conversation_id:cid.into()});}}
        };
        for(provider,c)in &all.pets{
            add(provider,&c.id,&c.attachment_history);
            if c.busy(){busy.insert(Owner{provider:provider.clone(),conversation_id:c.id.clone()});}
            for task in &c.tasks{if !task.terminal(){if let Some(cid)=&task.conversation_id{busy.insert(Owner{provider:task.provider.clone(),conversation_id:cid.clone()});}}}
        }
        for(provider,rows)in &all.archives{for c in rows{add(provider,&c.id,&c.attachment_history);}}
        let complete=all.retained.is_empty();
        Ok(Self{revision:digest(&(&owners,complete))?,owners,busy,complete})
    }
    pub(crate) fn unreferenced(&self,ids:&[String])->bool{self.complete&&ids.iter().all(|id|!self.owners.contains_key(id))}
    pub(crate) fn projection(&self,ids:&[String])->Result<String,String>{
        let selected=ids.iter().map(|id|(id,self.owners.get(id))).collect::<BTreeMap<_,_>>();digest(&(selected,self.complete))
    }
    pub(crate) fn idle(&self,provider:&str,cid:&str)->bool{!self.busy.contains(&Owner{provider:provider.into(),conversation_id:cid.into()})}
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Target{
    pub provider:String,pub conversation_id:String,pub revision:String,pub current:bool,pub title:String,
    pub message_count:usize,pub task_count:usize,pub attachment_count:usize,
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Receipt{
    pub provider:String,pub removed_conversation_id:String,pub current:Option<ConversationView>,
    pub recovery_id:Option<String>,pub durable:bool,
}
fn saved_target(all:&Conversations,provider:&str,cid:&str)->Result<(SavedConversation,bool),String>{
    let(mut saved,current)=if let Some(c)=all.pets.get(provider).filter(|c|c.id==cid){(c.saved(),true)}else{
        (all.archives.get(provider).and_then(|rows|rows.iter().find(|c|c.id==cid)).cloned().ok_or("A conversa já não existe neste pet.")?,false)
    };saved.tasks.clear();Ok((saved,current))
}
fn target(all:&Conversations,provider:&str,cid:&str)->Result<Target,String>{
    if !crate::assistant_memory::supported(provider)||!crate::assistant_memory::valid_id(cid){return Err(CHANGED.into());}
    let references=References::capture(all)?;if !references.idle(provider,cid){return Err(BUSY_ERROR.into());}
    let(saved,current)=saved_target(all,provider,cid)?;
    let mut tasks=all.pets.values().flat_map(|c|&c.tasks).filter(|t|t.provider==provider&&t.conversation_id.as_deref()==Some(cid)).collect::<Vec<_>>();tasks.sort_by(|a,b|a.id.cmp(&b.id));
    let revision=digest(&(provider,cid,current,&saved,&tasks))?;let title=summary(provider,&saved,current).title;
    let attachment_count=saved.attachment_history.iter().flat_map(|r|r.attachments.iter().map(|a|a.id.clone())).collect::<BTreeSet<_>>().len();
    Ok(Target{provider:provider.into(),conversation_id:cid.into(),revision,current,title,message_count:saved.messages.len(),task_count:tasks.len(),attachment_count})
}
fn library(all:&Conversations)->crate::assistant_memory::Library{
    crate::assistant_memory::Library{pets:all.pets.iter().filter(|(p,_)|crate::assistant_memory::supported(p)).map(|(p,c)|(p.clone(),c.saved())).collect(),archives:all.archives.clone(),retained:all.retained.clone()}
}
impl Chat{
    /// Passive snapshot: never identify/create a conversation, connect a native
    /// client, or read a transcript just to describe capabilities.
    pub(crate) fn capability_health(&self)->(bool,BTreeSet<String>){
        let all=self.inner.lock().unwrap();let healthy=!self.disk_error&&!all.pets.values().any(|c|c.save_error.is_some());
        let mut busy:BTreeSet<_>=all.pets.iter().filter(|(_,c)|c.busy()||c.tasks.iter().any(|t|!t.terminal())).map(|(p,_)|p.clone()).collect();drop(all);match self.resources.busy(){Ok(held)=>busy.extend(held),Err(_)=>return(false,busy)};(healthy,busy)
    }
    pub(crate) fn retention_references(&self)->Result<References,String>{
        if self.disk_error{return Err("A biblioteca está indisponível; as cópias foram preservadas.".into());}
        References::capture(&self.inner.lock().unwrap())
    }
    pub(crate) fn with_copy_purge_fence<T>(&self,provider:&str,cid:&str,revision:&str,ids:&[String],work:impl FnOnce()->Result<T,String>)->Result<T,String>{
        let mut unique=BTreeSet::new();
        if self.disk_error||crate::pet_config::integration_pet(provider).is_none()||!crate::assistant_memory::valid_id(cid)||ids.is_empty()||ids.len()>32
            ||ids.iter().any(|id|!crate::assistant_memory::valid_id(id)||!unique.insert(id))||revision.len()!=64{return Err(CHANGED.into());}let all=self.inner.lock().unwrap();
        if all.pets.values().any(|c|c.save_error.is_some()){return Err("A biblioteca tem uma gravação pendente. Guarda ou recupera o histórico antes de eliminar cópias.".into());}
        if let Some(path)=&self.disk{if recovery::pending(path)?{return Err("Existe uma eliminação do histórico por recuperar. Resolve-a antes de eliminar cópias.".into());}}
        let refs=References::capture(&all)?;if !refs.complete{return Err("Há histórico inativo com referências ainda não interpretadas. As cópias dos anexos foram conservadas.".into());}if (refs.revision!=revision&&refs.projection(ids)?!=revision)||!refs.unreferenced(ids){return Err(CHANGED.into());}if !refs.idle(provider,cid){return Err(BUSY_ERROR.into());}work()
    }
    pub(crate) fn with_copy_recovery_fence<T>(&self,provider:&str,cid:&str,work:impl FnOnce()->Result<T,String>)->Result<T,String>{
        if self.disk_error||crate::pet_config::integration_pet(provider).is_none()||!crate::assistant_memory::valid_id(cid){return Err(CHANGED.into());}
        let all=self.inner.lock().unwrap();if all.pets.values().any(|c|c.save_error.is_some()){return Err(CHANGED.into());}
        if !References::capture(&all)?.idle(provider,cid){return Err(BUSY_ERROR.into());}work()
    }
    pub(crate) fn purge_target(&self,provider:&str,cid:&str)->Result<Target,String>{
        if self.disk_error{return Err(CHANGED.into());}target(&self.inner.lock().unwrap(),provider,cid)
    }
    pub(crate) fn purge_local_conversation(&self,reviewed:&Target,deadline:Instant)->Result<Receipt,String>{
        self.purge_using(reviewed,deadline,|path,library,current|crate::assistant_memory::save_checked(path,library,current))
    }
    fn purge_using(&self,reviewed:&Target,deadline:Instant,persist:impl FnOnce(&Path,&crate::assistant_memory::Library,&dyn Fn()->bool)->std::io::Result<crate::assistant_memory::SaveReceipt>)->Result<Receipt,String>{
        if self.disk_error||Instant::now()>=deadline{return Err(CHANGED.into());}
        let mut all=self.inner.lock().unwrap();let current=target(&all,&reviewed.provider,&reviewed.conversation_id)?;
        if current.revision!=reviewed.revision||Instant::now()>=deadline{return Err(CHANGED.into());}
        let(saved,_)=saved_target(&all,&reviewed.provider,&reviewed.conversation_id)?;let mut next=all.clone();
        let mut removed_tasks=Vec::new();
        for c in next.pets.values_mut(){c.tasks.retain(|task|{let remove=task.provider==reviewed.provider&&task.conversation_id.as_deref()==Some(reviewed.conversation_id.as_str());if remove{removed_tasks.push(task.clone());}!remove});}
        let new_current=if current.current{
            let tasks=next.pets.get_mut(&reviewed.provider).map(|c|std::mem::take(&mut c.tasks)).unwrap_or_default();
            next.next=next.next.checked_add(1).ok_or(CHANGED)?;let mut fresh=Conversation{epoch:next.next,tasks,..Default::default()};fresh.identify();let view=fresh.view(&reviewed.provider);next.pets.insert(reviewed.provider.clone(),fresh);Some(view)
        }else{
            next.archives.get_mut(&reviewed.provider).ok_or(CHANGED)?.retain(|c|c.id!=reviewed.conversation_id);None
        };
        removed_tasks.sort_by(|a,b|a.id.cmp(&b.id));
        let staged=if let Some(path)=&self.disk{Some(recovery::Backup::stage(path,reviewed,saved,removed_tasks)?)}else{None};
        let receipt=if let Some(path)=&self.disk{persist(path,&library(&next),&||Instant::now()<deadline).map_err(|_|"Não foi possível publicar a eliminação. O histórico e a cópia de recuperação foram preservados.".to_string())?}else{crate::assistant_memory::SaveReceipt{durable:true}};
        // Publication already happened if save_checked returned a receipt.
        // Adopt it even when directory fsync could not confirm durability.
        for c in next.pets.values_mut(){c.save_error=None;}
        if !receipt.durable{if let Some(c)=next.pets.get_mut(&reviewed.provider){c.save_error=Some("A eliminação foi publicada, mas falta confirmar a gravação. A recuperação foi conservada.".into());}}
        *all=next;
        let recovery_id=staged.and_then(|backup|if receipt.durable&&backup.remove().is_ok(){None}else{Some(backup.id)});
        Ok(Receipt{provider:reviewed.provider.clone(),removed_conversation_id:reviewed.conversation_id.clone(),current:new_current,recovery_id,durable:receipt.durable})
    }
    pub(crate) fn purge_recovery_inventory(&self)->Result<Vec<recovery::Summary>,String>{
        let Some(path)=&self.disk else{return Ok(vec![])};recovery::inventory(path)
    }
    pub(crate) fn purge_recovery_target(&self,id:&str)->Result<recovery::Summary,String>{
        let path=self.disk.as_ref().ok_or(CHANGED)?;let backup=recovery::Backup::open(path,id)?;Ok(backup.summary)
    }
    pub(crate) fn finish_purge_recovery(&self,reviewed:&recovery::Summary,deadline:Instant)->Result<bool,String>{
        if self.disk_error||Instant::now()>=deadline{return Err(CHANGED.into());}
        let path=self.disk.as_ref().ok_or(CHANGED)?;let mut all=self.inner.lock().unwrap();let backup=recovery::Backup::open(path,&reviewed.id)?;
        if backup.summary.revision!=reviewed.revision{return Err(CHANGED.into());}
        // Publish the complete current RAM library durably before removing
        // recovery material. This also resolves an uncertain prior rename.
        let receipt=crate::assistant_memory::save_checked(path,&library(&all),||Instant::now()<deadline).map_err(|_|CHANGED)?;
        if !receipt.durable{return Err("A gravação ainda não foi confirmada. A recuperação foi conservada.".into());}
        for c in all.pets.values_mut(){c.save_error=None;}
        Ok(backup.backup.remove().is_ok())
    }
    pub(crate) fn restore_purge_recovery(&self,reviewed:&RecoverySummary,deadline:Instant)->Result<bool,String>{
        if self.disk_error||Instant::now()>=deadline{return Err(CHANGED.into());}
        let path=self.disk.as_ref().ok_or(CHANGED)?;let mut all=self.inner.lock().unwrap();let backup=recovery::Backup::open(path,&reviewed.id)?;
        if backup.summary.revision!=reviewed.revision{return Err(CHANGED.into());}
        let provider=&backup.summary.target.provider;let cid=&backup.summary.target.conversation_id;let refs=References::capture(&all)?;if !refs.idle(provider,cid){return Err(BUSY_ERROR.into());}
        let mut next=all.clone();
        match saved_target(&next,provider,cid){
            Ok((saved,_))=>if digest(&saved)?!=digest(&backup.conversation)?{return Err("A conversa já existe com alterações. A recuperação foi conservada, sem substituir dados.".into());},
            Err(_)=>{let rows=next.archives.entry(provider.clone()).or_default();if rows.len()>=crate::assistant_memory::MAX_ARCHIVES{return Err("O arquivo atingiu o limite. A recuperação foi conservada.".into());}rows.push(backup.conversation.clone());}
        }
        let owner=next.pets.entry(provider.clone()).or_default();owner.identify();
        for task in &backup.tasks{
            if let Some(existing)=owner.tasks.iter().find(|t|t.id==task.id){if digest(existing)?!=digest(task)?{return Err("Uma tarefa já existe com alterações. A recuperação foi conservada.".into());}}
            else{if owner.tasks.len()>=10{return Err("A lista de tarefas atingiu o limite. A recuperação foi conservada.".into());}owner.tasks.push(task.clone());}
        }
        let receipt=crate::assistant_memory::save_checked(path,&library(&next),||Instant::now()<deadline).map_err(|_|CHANGED)?;
        for c in next.pets.values_mut(){c.save_error=None;}
        if !receipt.durable{if let Some(c)=next.pets.get_mut(provider){c.save_error=Some("A recuperação foi publicada; falta confirmar a gravação.".into());}}
        *all=next;Ok(receipt.durable&&backup.backup.remove().is_ok())
    }
}
#[cfg(test)]
#[path="chat_retention_tests.rs"]
mod tests;
