//! Bounded metadata catalog and CAS updates for explicit copy retention.
//! No file contents, grants, model tools or inbox deletion live in this module.
use super::*;
#[path="purge_quarantine.rs"]
pub(super) mod quarantine;
#[path="copy_retention.rs"]
pub(in crate::pet_documents) mod native;
use std::{fs::OpenOptions,os::unix::fs::{MetadataExt,OpenOptionsExt},time::{Duration,Instant}};
const LIMIT:&str="retentionCatalogLimit";
const CHANGED:&str="retentionChanged";
const MAX_FILES:usize=4096;
const MAX_CATALOG_BYTES:u64=128*1024*1024;
const MAX_ROWS:usize=131072;
pub(super) const CONTROL_ROOT:&str=".coucou-purge-v1";
fn reserved(name:&str)->bool{name==CONTROL_ROOT||name.starts_with(".coucou-purge-setup-")}
fn hash(value:&impl Serialize)->Result<String,&'static str>{
    Ok(ring::digest::digest(&ring::digest::SHA256,&serde_json::to_vec(value).map_err(|_|CHANGED)?).as_ref().iter().map(|b|format!("{b:02x}")).collect())
}
fn hidden_internal(name:&str)->bool{
    (name.starts_with(".attachments-")&&name.ends_with(".tmp"))||
    (name.starts_with('.')&&name.ends_with(".json.lock")&&name.len()==75&&name.as_bytes()[1..65].iter().all(|b|b.is_ascii_hexdigit()))
}
fn scan_at(directory:&Path)->Result<Vec<Registry>,&'static str>{
    let file=match OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(directory){
        Ok(f)=>f,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok(vec![]),Err(_)=>return Err(CHANGED),
    };
    let before=file.metadata().map_err(|_|CHANGED)?;
    let private=|m:&std::fs::Metadata|m.is_dir()&&m.uid()==unsafe{libc::geteuid()}&&m.mode()&0o077==0;
    if !private(&before){return Err(CHANGED);}
    let deadline=Instant::now()+Duration::from_secs(8);let mut bytes=0u64;let mut rows=0usize;let mut count=0usize;let mut all=vec![];
    for entry in std::fs::read_dir(directory).map_err(|_|CHANGED)?{
        count+=1;if count>MAX_FILES||Instant::now()>=deadline{return Err(LIMIT);}
        let entry=entry.map_err(|_|CHANGED)?;let name=entry.file_name();let name=name.to_str().ok_or(CHANGED)?;
        if hidden_internal(name){continue;}
        // Unknown visible and hidden metadata is not silently ignored.
        if name.len()!=69||!name.ends_with(".json")||!name.as_bytes()[..64].iter().all(|b|b.is_ascii_hexdigit()){return Err(CHANGED);}
        let path=directory.join(name);let meta=std::fs::symlink_metadata(&path).map_err(|_|CHANGED)?;
        bytes=bytes.checked_add(meta.len()).filter(|n|*n<=MAX_CATALOG_BYTES).ok_or(LIMIT)?;
        let snapshot=io::load_at(&path)?;let registry=snapshot.registry;
        if path_at(directory,&registry.provider,&registry.conversation_id)?!=path{return Err(CHANGED);}
        rows=rows.checked_add(registry.records.len()).filter(|n|*n<=MAX_ROWS).ok_or(LIMIT)?;all.push(registry);
    }
    let after=file.metadata().map_err(|_|CHANGED)?;let current=std::fs::symlink_metadata(directory).map_err(|_|CHANGED)?;
    if !private(&current)||current.dev()!=before.dev()||current.ino()!=before.ino()||
        (after.mtime(),after.mtime_nsec(),after.ctime(),after.ctime_nsec())!=(before.mtime(),before.mtime_nsec(),before.ctime(),before.ctime_nsec()){
        return Err(CHANGED);
    }
    all.sort_by(|a,b|(&a.provider,&a.conversation_id).cmp(&(&b.provider,&b.conversation_id)));Ok(all)
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct Item{pub id:String,pub name:String,pub kind:String,pub size:u64,pub entries:usize,pub selected:bool,pub purging:bool}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct Owned{pub provider:String,pub conversation_id:String,pub revision:String,pub items:Vec<Item>}
pub(super) fn catalog()->Result<Vec<Owned>,String>{catalog_at(&directory()).map_err(message)}
fn catalog_at(directory:&Path)->Result<Vec<Owned>,&'static str>{
    scan_at(directory)?.into_iter().map(|r|{
        let snapshot=io::load_at(&path_at(directory,&r.provider,&r.conversation_id)?)?;if snapshot.registry!=r{return Err(CHANGED);}
        Ok(Owned{provider:r.provider,conversation_id:r.conversation_id,revision:snapshot.revision,items:r.records.into_iter().map(|v|Item{
            id:v.id,name:v.name,kind:v.kind,size:v.size,entries:v.entries,selected:v.selected,purging:v.purge_transaction.is_some(),
        }).collect()})
    }).collect()
}
pub(super) fn resume_marker_at(directory:&Path,provider:&str,cid:&str,tx:&str,records:&[Record],sources:&[Attachment])->Result<Pending,String>{
    if !crate::assistant_memory::valid_id(tx)||!valid_records(records){return Err(message(CHANGED));}
    let names=sources.iter().map(|s|Path::new(&s.file.path).file_name().and_then(|n|n.to_str()).ok_or(CHANGED)).collect::<Result<Vec<_>,_>>().map_err(message)?;
    for owner in scan_at(directory).map_err(message)?{for r in owner.records{
        if records.iter().any(|old|old.id==r.id)||(r.purge_transaction.is_none()&&names.contains(&r.copy_name.as_str())){
            let own=owner.provider==provider&&owner.conversation_id==cid&&records.iter().any(|old|{let mut marked=old.clone();marked.purge_transaction=Some(tx.into());r==*old||r==marked});
            if !own{return Err(message(CHANGED));}
        }
    }}
    io::prepare(&path_at(directory,provider,cid).map_err(message)?,None,|r|{
        r.bind(provider,cid)?;for old in records{if let Some(row)=r.records.iter_mut().find(|r|r.id==old.id){let mut marked=old.clone();marked.purge_transaction=Some(tx.into());if *row!=*old&&*row!=marked{return Err(CHANGED);}*row=marked;}
            else if sources.iter().any(|s|s.id==old.id){return Err(CHANGED);}
        }Ok(())
    }).map(Pending).map_err(message)
}
pub(super) fn claimed_names_at(directory:&Path,provider:&str,cid:&str,tx:&str,records:&[Record])->Result<BTreeSet<String>,String>{
    let mut names=BTreeSet::new();for owner in scan_at(directory).map_err(message)?{for r in owner.records{
        if r.purge_transaction.is_some(){continue;}
        let own=owner.provider==provider&&owner.conversation_id==cid&&records.iter().any(|old|old.id==r.id&&old.name==r.name&&old.kind==r.kind&&old.size==r.size&&old.entries==r.entries&&!r.selected);
        if !own{names.insert(r.copy_name);}
    }}let _=tx;Ok(names)
}
// Bind only selected IDs and live basename claims. Work in unrelated CIDs may
// continue without invalidating the review. Pending rows retain ID ownership,
// while their old source basename may already have been reused by a new drop.
fn claims(directory:&Path,provider:&str,cid:&str,selected:&[Record])->Result<String,&'static str>{
    let ids=selected.iter().map(|r|r.id.as_str()).collect::<BTreeSet<_>>();let names=selected.iter().map(|r|r.copy_name.as_str()).collect::<BTreeSet<_>>();
    let mut matches=vec![];
    for owner in scan_at(directory)?{for r in owner.records{
        if ids.contains(r.id.as_str())||(r.purge_transaction.is_none()&&names.contains(r.copy_name.as_str())){
            matches.push((owner.provider.clone(),owner.conversation_id.clone(),r));
        }
    }}
    if matches.len()!=selected.len()||matches.iter().any(|(p,c,r)|p!=provider||c!=cid||!selected.contains(r)){return Err(CHANGED);}
    matches.sort_by(|a,b|(&a.0,&a.1,&a.2.id).cmp(&(&b.0,&b.1,&b.2.id)));hash(&matches)
}
pub(super) struct Review{directory:PathBuf,path:PathBuf,snapshot:io::Snapshot,pub records:Vec<Record>,claims:String}
impl Review{
    pub(super) fn prepare(provider:&str,cid:&str,ids:&[String],revision:&str)->Result<Self,String>{
        Self::at(&directory(),provider,cid,ids,revision).map_err(message)
    }
    fn at(directory:&Path,provider:&str,cid:&str,ids:&[String],revision:&str)->Result<Self,&'static str>{
        let unique=ids.iter().collect::<BTreeSet<_>>();if ids.is_empty()||ids.len()>MAX_CONVERSATION_ATTACHMENTS||unique.len()!=ids.len(){return Err(CHANGED);}
        let path=path_at(directory,provider,cid)?;let snapshot=io::load_at(&path)?;
        if snapshot.revision!=revision||!snapshot.unchanged()||snapshot.registry.records.iter().any(|r|r.purge_transaction.is_some()){return Err(CHANGED);}
        let records=ids.iter().map(|id|snapshot.registry.records.iter().find(|r|r.id==*id).filter(|r|!r.selected&&!reserved(&r.copy_name)).cloned().ok_or(CHANGED)).collect::<Result<Vec<_>,_>>()?;
        let all=scan_at(directory)?;
        for selected in &records{
            let owners=all.iter().flat_map(|o|o.records.iter().filter(move|r|r.id==selected.id||(r.purge_transaction.is_none()&&r.copy_name==selected.copy_name)).map(move|r|(&o.provider,&o.conversation_id,r)) ).collect::<Vec<_>>();
            if owners.len()!=1||owners[0].0!=provider||owners[0].1!=cid||owners[0].2!=selected{return Err(CHANGED);}
        }
        let claims=claims(directory,provider,cid,&records)?;if !snapshot.unchanged(){return Err(CHANGED);}
        Ok(Self{directory:directory.into(),path,snapshot,records,claims})
    }
    pub(super) fn attachments(&self,inbox:&Path)->Vec<Attachment>{
        self.records.iter().map(|r|attachment(&self.snapshot.registry.provider,&self.snapshot.registry.conversation_id,r,inbox,&r.copy_name)).collect()
    }
    pub(super) fn mark(&self,tx:&str)->Result<Pending,String>{
        if !crate::assistant_memory::valid_id(tx)||!self.snapshot.unchanged()||claims(&self.directory,&self.snapshot.registry.provider,&self.snapshot.registry.conversation_id,&self.records).map_err(message)?!=self.claims{return Err(message(CHANGED));}
        io::prepare(&self.path,Some(&self.snapshot.revision),|r|{
            for old in &self.records{let item=r.records.iter_mut().find(|v|v.id==old.id).ok_or(CHANGED)?;if item!=old{return Err(CHANGED);}item.purge_transaction=Some(tx.into());}Ok(())
        }).map(Pending).map_err(message)
    }
}
pub(super) fn attachment(provider:&str,cid:&str,r:&Record,inbox:&Path,basename:&str)->Attachment{
    Attachment{id:r.id.clone(),provider:provider.into(),conversation_id:cid.into(),sequence:0,file:crate::files::DroppedFile{
        name:r.name.clone(),path:inbox.join(basename).to_string_lossy().into_owned(),kind:r.kind.clone(),size:r.size,entries:r.entries,
    }}
}
pub(super) fn valid_records(records:&[Record])->bool{
    !records.is_empty()&&records.len()<=MAX_CONVERSATION_ATTACHMENTS&&records.iter().all(|r|!r.selected&&r.purge_transaction.is_none()&&!reserved(&r.copy_name))&&{
        Registry{version:1,generation:1,provider:"ollama".into(),conversation_id:"retention-check".into(),records:records.to_vec()}.validate().is_ok()
    }
}
pub(super) fn finish_at(directory:&Path,provider:&str,cid:&str,tx:&str,records:&[Record])->Result<Pending,String>{
    if !crate::assistant_memory::valid_id(tx)||!valid_records(records){return Err(message(CHANGED));}
    io::prepare(&path_at(directory,provider,cid).map_err(message)?,None,|r|{
        r.bind(provider,cid)?;
        for old in records{if let Some(current)=r.records.iter().find(|r|r.id==old.id){let mut expected=old.clone();expected.purge_transaction=Some(tx.into());if current!=&expected{return Err(CHANGED);}}}
        r.records.retain(|r|!records.iter().any(|old|old.id==r.id));Ok(())
    }).map(Pending).map_err(message)
}
pub(super) fn restore_at(directory:&Path,provider:&str,cid:&str,tx:&str,records:&[Record],names:&BTreeMap<String,String>)->Result<Pending,String>{
    if !crate::assistant_memory::valid_id(tx)||!valid_records(records)||records.len()!=names.len()||names.values().any(|n|!component(n)||reserved(n)){return Err(message(CHANGED));}
    // An absent physical path is insufficient: another manifest may still own
    // that basename. Restores never alias an existing record's future copy.
    for owner in scan_at(directory).map_err(message)?{for r in &owner.records{
        let own=owner.provider==provider&&owner.conversation_id==cid&&records.iter().any(|old|{let mut restored=old.clone();restored.copy_name=names.get(&old.id).cloned().unwrap_or_default();old.id==r.id&&(r.purge_transaction.as_deref()==Some(tx)||r==&restored)});
        if !own&&(records.iter().any(|old|old.id==r.id)||(r.purge_transaction.is_none()&&names.values().any(|n|n==&r.copy_name))){return Err(message(CHANGED));}
    }}
    io::prepare(&path_at(directory,provider,cid).map_err(message)?,None,|r|{
        r.bind(provider,cid)?;
        for old in records{
            if let Some(current)=r.records.iter().find(|r|r.id==old.id){let mut expected=old.clone();expected.purge_transaction=Some(tx.into());let mut restored=old.clone();restored.copy_name=names.get(&old.id).ok_or(CHANGED)?.clone();if current!=&expected&&current!=&restored{return Err(CHANGED);}}
            r.records.retain(|r|r.id!=old.id);let mut restored=old.clone();restored.copy_name=names.get(&old.id).ok_or(CHANGED)?.clone();r.records.push(restored);
        }Ok(())
    }).map(Pending).map_err(message)
}
#[cfg(test)]
#[path="manifest_purge_tests.rs"]
mod tests;
