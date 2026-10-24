//! Native, owner/window-bound explicit copy retention. Never a model tool.
use super::*;
use super::quarantine::{Transaction,Terminal,Phase,Summary};
use crate::{chat::Chat,pet_documents::{copy_review::{Budget,Copy},Store,store}};
const INVALID:&str="A revisão das cópias expirou ou já não corresponde à janela, conversa ou registos. Volta a rever.";
const INCOMPLETE:&str="A operação ficou incompleta. O registo de recuperação foi conservado; revê-o antes de repetir. Uma cópia parcialmente eliminada não pode ser recuperada na íntegra.";
type Fence=(u64,u64,u64);
struct Initial{manifest:Review,copies:Vec<Copy>,files:Vec<Attachment>}
enum Payload{Copies(Initial),Recovery(Transaction),Terminal(Terminal),Consumed}
struct NativeReview{window:String,provider:String,cid:String,deadline:Instant,fence:Fence,references:String,ids:Vec<String>,payload:Payload}
#[derive(Default)]
struct Reviews(BTreeMap<String,NativeReview>);
impl Reviews{
    fn expire(&mut self,now:Instant){self.0.retain(|_,r|r.deadline>now);}
    fn insert(&mut self,r:NativeReview,now:Instant)->Result<Preview,String>{
        self.expire(now);if now>=r.deadline||self.0.len()>=32||self.0.values().map(|r|r.ids.len()).sum::<usize>()+r.ids.len()>128{return Err(INVALID.into());}
        let mut bytes=[0u8;32];SystemRandom::new().fill(&mut bytes).map_err(|_|INVALID)?;let id=bytes.iter().map(|b|format!("{b:02x}")).collect::<String>();
        let(items,recovery)=match &r.payload{
            Payload::Copies(v)=>(v.files.iter().map(|f|quarantine::SummaryItem{id:f.id.clone(),name:f.file.name.clone(),kind:f.file.kind.clone(),size:f.file.size,entries:f.file.entries}).collect(),None),
            Payload::Recovery(tx)=>{let s=tx.summary().map_err(|_|INVALID)?;(s.items.clone(),Some(s))},
            Payload::Terminal(tx)=>{let s=tx.summary().map_err(|_|INVALID)?;(s.items.clone(),Some(s))},
            Payload::Consumed=>return Err(INVALID.into()),
        };
        let preview=Preview{review_id:id.clone(),provider:r.provider.clone(),conversation_id:r.cid.clone(),items,recovery,expires_in_seconds:r.deadline.saturating_duration_since(now).as_secs()};self.0.insert(id,r);Ok(preview)
    }
    fn take(&mut self,id:&str,window:&str,provider:&str,cid:&str,now:Instant)->Result<NativeReview,String>{
        self.expire(now);let r=self.0.get(id).ok_or(INVALID)?;if r.window!=window||r.provider!=provider||r.cid!=cid{return Err(INVALID.into());}self.0.remove(id).ok_or_else(||INVALID.into())
    }
    fn cancel(&mut self,id:&str,window:&str,provider:&str,cid:&str){self.expire(Instant::now());if self.0.get(id).is_some_and(|r|r.window==window&&r.provider==provider&&r.cid==cid){self.0.remove(id);}}
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(Mutex::default)}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(in crate::pet_documents) struct Preview{review_id:String,provider:String,conversation_id:String,items:Vec<quarantine::SummaryItem>,recovery:Option<Summary>,expires_in_seconds:u64}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(in crate::pet_documents) struct Receipt{provider:String,conversation_id:String,operation:String,complete:bool,recovery_id:Option<String>,error:Option<String>}
/// Only closed native reason codes become specific public messages. Unknown
/// internal errors keep the generic recovery notice, without paths/details.
fn recovery_failure(reason:&str)->String{match reason{
    quarantine::RESTORE_UNAVAILABLE=>"Não foi possível verificar todas as cópias completas para o restauro. Uma cópia pode estar incompleta, alterada ou indisponível; os ficheiros existentes foram conservados.".into(),
    quarantine::RESTORE_CLAIMED=>"O destino da cópia está referenciado por outro registo ou conversa. O restauro foi recusado sem mover essa cópia nem alterar os registos existentes.".into(),
    quarantine::CLEANUP_UNVERIFIED=>"A árvore privada de recuperação não pôde ser validada antes da limpeza. Os ficheiros e pastas existentes foram conservados; revê a recuperação antes de continuar.".into(),
    quarantine::CLEANUP_INTERRUPTED=>"A limpeza foi interrompida depois de começar. A recuperação foi conservada; não é possível afirmar que todas as cópias ainda podem ser repostas na íntegra.".into(),
    _=>INCOMPLETE.into(),
}}
fn receipt(provider:&str,cid:&str,operation:&str,tx:&mut Transaction,result:Result<(),String>)->Receipt{
    let complete=result.is_ok();let error=result.as_ref().err().map(|e|recovery_failure(e));let cleared=complete&&tx.remove_metadata().is_ok();
    Receipt{provider:provider.into(),conversation_id:cid.into(),operation:operation.into(),complete,recovery_id:(!cleared).then(||tx.id().to_owned()),error}
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct OwnerRow{provider:String,conversation_id:String,revision:String,busy:bool,items:Vec<InventoryItem>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct InventoryItem{id:String,name:String,kind:String,size:u64,entries:usize,selected:bool,purging:bool,history_references:usize}
#[derive(Serialize)]
pub(in crate::pet_documents) struct Inventory{owners:Vec<OwnerRow>,recoveries:quarantine::Inventory}
pub(in crate::pet_documents) fn inventory(chat:&Chat)->Result<Inventory,String>{inventory_at(chat,&directory(),&crate::files::inbox_dir())}
fn inventory_at(chat:&Chat,directory:&Path,inbox:&Path)->Result<Inventory,String>{
    reviews().lock().unwrap().expire(Instant::now());let refs=chat.retention_references()?;
    let owners=catalog_at(directory).map_err(message)?.into_iter().map(|o|OwnerRow{
        busy:!refs.idle(&o.provider,&o.conversation_id),provider:o.provider,conversation_id:o.conversation_id,revision:o.revision,
        items:o.items.into_iter().map(|i|InventoryItem{history_references:refs.owners.get(&i.id).map_or(0,|r|r.len()),id:i.id,name:i.name,kind:i.kind,size:i.size,entries:i.entries,selected:i.selected,purging:i.purging}).collect(),
    }).collect();let recoveries=quarantine::inventory(inbox).map_err(|_|INCOMPLETE)?;Ok(Inventory{owners,recoveries})
}
fn initial_at(chat:&Chat,state:&Mutex<Store>,directory:&Path,inbox:&Path,window:String,provider:String,cid:String,ids:Vec<String>,expected:String,deadline:Instant)->Result<NativeReview,String>{
    if Instant::now()>=deadline{return Err(INVALID.into());}let manifest=Review::at(directory,&provider,&cid,&ids,&expected).map_err(message)?;
    let files=manifest.attachments(inbox);let fence={let mut s=state.lock().unwrap();s.copy_purge_available(&provider,&cid,&ids,&files)?;s.review_fence(&provider,&cid)?};
    let refs=chat.retention_references()?;let references=refs.projection(&ids)?;
    chat.with_copy_purge_fence(&provider,&cid,&references,&ids,||Ok(()))?;
    let mut budget=Budget::new();let copies=files.iter().map(|a|Copy::capture(inbox,a,&mut budget)).collect::<Result<Vec<_>,_>>()?;
    if Instant::now()>=deadline{return Err(INVALID.into());}Ok(NativeReview{window,provider,cid,deadline,fence,references,ids,payload:Payload::Copies(Initial{manifest,copies,files})})
}
pub(in crate::pet_documents) fn prepare(chat:&Chat,window:String,provider:String,cid:String,ids:Vec<String>,expected:String,deadline:Instant)->Result<Preview,String>{
    let review=initial_at(chat,store(),&directory(),&crate::files::inbox_dir(),window,provider,cid,ids,expected,deadline)?;reviews().lock().unwrap().insert(review,Instant::now())
}
fn recovery_at(chat:&Chat,state:&Mutex<Store>,inbox:&Path,window:String,id:&str,deadline:Instant)->Result<NativeReview,String>{
    if Instant::now()>=deadline{return Err(INVALID.into());}
    let(summary,payload)=match Terminal::open(inbox,id).map_err(|_|INCOMPLETE)?{
        Some(tx)=>(tx.summary().map_err(|_|INVALID)?,Payload::Terminal(tx)),
        None=>{let tx=Transaction::open(inbox,id).map_err(|_|INCOMPLETE)?;(tx.summary().map_err(|_|INVALID)?,Payload::Recovery(tx))},
    };
    let ids=summary.items.iter().map(|i|i.id.clone()).collect::<Vec<_>>();let refs=chat.retention_references()?;let references=refs.projection(&ids)?;let fence=state.lock().unwrap().review_fence(&summary.provider,&summary.conversation_id)?;
    Ok(NativeReview{window,provider:summary.provider,cid:summary.conversation_id,deadline,fence,references,ids,payload})
}
pub(in crate::pet_documents) fn recovery_review(chat:&Chat,window:String,id:&str,deadline:Instant)->Result<Preview,String>{
    reviews().lock().unwrap().expire(Instant::now());let review=recovery_at(chat,store(),&crate::files::inbox_dir(),window,id,deadline)?;reviews().lock().unwrap().insert(review,Instant::now())
}
fn validate_state(state:&mut Store,r:&NativeReview,files:&[Attachment])->Result<(),String>{
    if Instant::now()>=r.deadline||!state.review_current(&r.provider,&r.cid,r.fence){return Err(INVALID.into());}state.copy_purge_available(&r.provider,&r.cid,&r.ids,files)
}
fn publish(mut p:Pending)->Result<(),String>{p.publish()?;p.confirm()}
fn finish_initial(r:&NativeReview,initial:&Initial,chat:&Chat,state:&Mutex<Store>,inbox:&Path)->Result<Receipt,String>{
    finish_initial_using(r,initial,chat,state,inbox,||Transaction::create(inbox,&r.provider,&r.cid,&initial.manifest.records,&initial.copies).map_err(|_|INCOMPLETE.to_owned()),|_|Ok(()))
}
fn finish_initial_using(r:&NativeReview,initial:&Initial,chat:&Chat,state:&Mutex<Store>,_inbox:&Path,create:impl FnOnce()->Result<Transaction,String>,mut checkpoint:impl FnMut(&str)->Result<(),String>)->Result<Receipt,String>{
    let mut budget=Budget::new();if !initial.copies.iter().all(|c|c.current(&mut budget)){return Err(INVALID.into());}
    chat.with_copy_purge_fence(&r.provider,&r.cid,&r.references,&r.ids,||validate_state(&mut state.lock().unwrap(),r,&initial.files))?;
    let mut tx=create()?;
    let result=(||{
        checkpoint("prepared")?;
        let pending=initial.manifest.mark(tx.id())?;
        chat.with_copy_purge_fence(&r.provider,&r.cid,&r.references,&r.ids,||{
            {let mut s=state.lock().unwrap();validate_state(&mut s,r,&initial.files)?;if !initial.copies.iter().all(Copy::root_current){return Err(INVALID.into());}
                crate::pet_files::revoke_copy_grants(&r.provider,&initial.files.iter().map(|f|f.file.path.clone()).collect::<Vec<_>>())?;
                publish(pending)?;checkpoint("markedManifest")?;s.touch(&r.provider,&r.cid);
            }
            tx.marked().map_err(|_|INCOMPLETE)?;tx.move_all().map_err(|_|INCOMPLETE)?;
            publish(finish_at(&initial.manifest.directory,&r.provider,&r.cid,tx.id(),&tx.records())?)?;checkpoint("removedManifest")?;tx.committed().map_err(|_|INCOMPLETE.to_owned())
        })?;
        // No indexed history/recovery contains these IDs, their grants were
        // revoked and their records/selection were removed under the fence.
        // Recursive cleanup uses only the locked private quarantine payload.
        tx.cleanup().map_err(str::to_owned)
    })();Ok(receipt(&r.provider,&r.cid,"deleteOwnedCopies",&mut tx,result))
}
fn finish_recovery(r:&NativeReview,tx:&mut Transaction,chat:&Chat,state:&Mutex<Store>,directory:&Path,restore:bool)->Result<Receipt,String>{
    tx.current().map_err(|_|INCOMPLETE)?;let sources=tx.source_files().map_err(|_|INCOMPLETE)?;
    let result=if restore{
        if *tx.phase()==Phase::Cleaned{return Err("As cópias desta operação já foram eliminadas. Só o registo da recuperação pode ser removido.".into());}
        chat.with_copy_recovery_fence(&r.provider,&r.cid,||{
            {let mut s=state.lock().unwrap();validate_state(&mut s,r,&sources)?;
                crate::pet_files::revoke_copy_grants(&r.provider,&sources.iter().map(|f|f.file.path.clone()).collect::<Vec<_>>())?;s.touch(&r.provider,&r.cid);
            }
            let names=claimed_names_at(directory,&r.provider,&r.cid,tx.id(),&tx.records())?;
            let names=tx.restore_moves(&names).map_err(str::to_owned)?;publish(restore_at(directory,&r.provider,&r.cid,tx.id(),&tx.records(),&names)?)?;tx.restored().map_err(|_|INCOMPLETE.to_owned())
        })
    }else{
        if matches!(tx.phase(),Phase::Restoring|Phase::Restored){return Err("Esta recuperação já começou a repor cópias. Continua a recuperação em vez de as eliminar.".into());}
        chat.with_copy_purge_fence(&r.provider,&r.cid,&r.references,&r.ids,||{
            {let mut s=state.lock().unwrap();validate_state(&mut s,r,&sources)?;
                crate::pet_files::revoke_copy_grants(&r.provider,&sources.iter().map(|f|f.file.path.clone()).collect::<Vec<_>>())?;
                if matches!(tx.phase(),Phase::Prepared|Phase::Marked|Phase::Moved){publish(resume_marker_at(directory,&r.provider,&r.cid,tx.id(),&tx.records(),&sources)?)?;}
                s.touch(&r.provider,&r.cid);
            }
            if *tx.phase()==Phase::Prepared{tx.marked().map_err(|_|INCOMPLETE)?;}
            if matches!(tx.phase(),Phase::Marked|Phase::Moved){tx.move_all().map_err(|_|INCOMPLETE)?;publish(finish_at(directory,&r.provider,&r.cid,tx.id(),&tx.records())?)?;tx.committed().map_err(|_|INCOMPLETE)?;}Ok(())
        }).and_then(|_|if *tx.phase()==Phase::Cleaned{Ok(())}else{tx.cleanup().map_err(str::to_owned)})
    };Ok(receipt(&r.provider,&r.cid,if restore{"restoreOwnedCopies"}else{"finishCopyPurge"},tx,result))
}
fn execute(mut r:NativeReview,chat:&Chat,state:&Mutex<Store>,inbox:&Path,directory:&Path,consent:&str)->Result<Receipt,String>{
    if Instant::now()>=r.deadline{return Err(INVALID.into());}
    // Move the payload out while retaining the original immutable binding.
    let payload=std::mem::replace(&mut r.payload,Payload::Consumed);
    match payload{
        Payload::Copies(v)if consent=="deleteOwnedCopies"=>finish_initial(&r,&v,chat,state,inbox),
        Payload::Recovery(mut tx)if consent=="finishCopyPurge"||consent=="restoreOwnedCopies"=>finish_recovery(&r,&mut tx,chat,state,directory,consent=="restoreOwnedCopies"),
        Payload::Terminal(mut tx)if consent=="clearCopyRecovery"=>{
            let id=tx.id().to_owned();let result=tx.remove_metadata();
            Ok(Receipt{provider:r.provider,conversation_id:r.cid,operation:"clearCopyRecovery".into(),complete:result.is_ok(),recovery_id:result.is_err().then_some(id),error:result.err().map(|_|INCOMPLETE.into())})
        },
        Payload::Terminal(_)=>Err("Esta operação terminou. Só é possível limpar os metadados de recuperação.".into()),
        Payload::Recovery(mut tx)if consent=="clearCopyRecovery"=>{
            if *tx.phase()==Phase::Prepared{
                if scan_at(directory).map_err(message)?.iter().any(|o|o.records.iter().any(|record|record.purge_transaction.as_deref()==Some(tx.id()))){return Err(INCOMPLETE.into());}
                tx.abandon_prepared().map_err(|_|INCOMPLETE)?;
            }
            if !matches!(tx.phase(),Phase::Cleaned|Phase::Restored){return Err("Esta operação ainda contém cópias por resolver. Termina a eliminação ou recupera as cópias antes de remover o registo.".into());}
            let result=tx.remove_metadata();Ok(Receipt{provider:r.provider,conversation_id:r.cid,operation:"clearCopyRecovery".into(),complete:result.is_ok(),recovery_id:result.is_err().then(||tx.id().to_owned()),error:result.err().map(|_|INCOMPLETE.into())})
        },
        _=>Err("Confirma explicitamente a operação apresentada na revisão das cópias.".into()),
    }
}
pub(in crate::pet_documents) fn commit(chat:&Chat,window:&str,provider:&str,cid:&str,id:&str,consent:&str)->Result<Receipt,String>{
    let review=reviews().lock().unwrap().take(id,window,provider,cid,Instant::now())?;execute(review,chat,store(),&crate::files::inbox_dir(),&directory(),consent)
}
pub(in crate::pet_documents) fn cancel(window:&str,provider:&str,cid:&str,id:&str){reviews().lock().unwrap().cancel(id,window,provider,cid);}
#[cfg(test)]
#[path="copy_retention_tests.rs"]
mod tests;
