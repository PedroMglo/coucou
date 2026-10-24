//! Isolated durable dialogue preparation. No model, tool or account calls.
use super::{model::*,store::{Dir,Guard,nonce,protocol_directory}};
use crate::chat::{Chat,ContextFence,SavedConversation};
use super::model::{ContextBinding as Binding,PreparationMode as Mode};
use std::{path::Path,sync::Arc,os::fd::AsRawFd};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-goal-contexts\"}\n";
const HEADER:&str=".context.json";const LOCK:&str=".context.lock";const JOURNAL:&str="conversation.json";

struct Fence{dir:Dir,_lock:Guard,binding:Binding,header:Vec<u8>}
impl ContextFence for Fence{
    fn current(&self)->bool{
        if self.dir.check().is_err()||self.dir.bytes(HEADER,16384).ok().flatten().is_none_or(|(b,_)|b!=self.header){return false;}
        let Some(current)=self.dir.open_file(LOCK,libc::O_RDONLY).ok().flatten()else{return false;};
        use std::os::unix::fs::MetadataExt;
        let(Ok(a),Ok(b))=(current.metadata(),self._lock.0.metadata())else{return false;};
        a.dev()==b.dev()&&a.ino()==b.ino()&&self.dir.open_file(JOURNAL,libc::O_RDONLY).ok().flatten().is_some_and(|f|f.metadata().is_ok_and(|m|m.len()<=crate::assistant_memory::MAX_BYTES))
    }
    fn goal_id(&self)->&str{&self.binding.goal_id}
}
fn root_lock(root:&Dir)->Result<Guard>{
    let file=root.open_file(".catalog.lock",libc::O_RDWR)?.ok_or(Error::UnsafePath)?;
    if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Error::Busy);}
    let guard=Guard(file);let current=root.open_file(".catalog.lock",libc::O_RDONLY)?.ok_or(Error::UnsafePath)?;
    use std::os::unix::fs::MetadataExt;
    let a=guard.0.metadata().map_err(|_|Error::UnsafePath)?;let b=current.metadata().map_err(|_|Error::UnsafePath)?;
    if a.dev()!=b.dev()||a.ino()!=b.ino(){return Err(Error::UnsafePath);}Ok(guard)
}
fn open_directory(root:&Dir,goal:&Record,main:&Chat)->Result<(Binding,Chat)>{
    let dir=Dir::open(&root.path.join(&goal.id),true)?.ok_or(Error::NotFound)?;
    let file=dir.open_file(LOCK,libc::O_RDWR)?.ok_or(Error::UnsafePath)?;
    if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Error::Busy);}let lock=Guard(file);
    let(header,_)=dir.bytes(HEADER,16384)?.ok_or(Error::InvalidRecord)?;
    let binding:Binding=serde_json::from_slice(&header).map_err(|_|Error::InvalidRecord)?;
    if !binding.valid(goal){return Err(Error::Conflict);}
    let journal=dir.open_file(JOURNAL,libc::O_RDONLY)?.ok_or(Error::InvalidRecord)?;
    if journal.metadata().map_err(|_|Error::UnsafePath)?.len()>crate::assistant_memory::MAX_BYTES{return Err(Error::LimitReached);}
    let path=dir.path.join(JOURNAL);let fence=Arc::new(Fence{dir,_lock:lock,binding:binding.clone(),header});
    if !fence.current(){return Err(Error::Conflict);}
    let chat=main.isolated_goal(path,&binding.provider,&binding.conversation_id,fence).map_err(|_|Error::InvalidRecord)?;
    Ok((binding,chat))
}
pub(super)fn prepare(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str)->Result<(Binding,Chat)>{
    goal.validate()?;if goal.state!=State::Draft||!key(request){return Err(Error::Conflict);}
    let root=protocol_directory(path,PROTOCOL,true)?.ok_or(Error::StorageUnavailable)?;let _root_guard=root_lock(&root)?;
    let target=root.path.join(&goal.id);
    if target.exists()||std::fs::symlink_metadata(&target).is_ok(){
        let(binding,chat)=open_directory(&root,goal,main)?;
        if binding.request_id!=request||binding.mode!=mode{return Err(Error::Conflict);}
        // Reconcile a previous uncertain publication before linking the goal.
        root.durable()?;std::fs::File::open(&target).and_then(|f|f.sync_all()).map_err(|_|Error::StorageUnavailable)?;
        return Ok((binding,chat));
    }
    if root.names()?.iter().filter(|n|goal_id(n)).count()>=MAX_GOALS{return Err(Error::LimitReached);}
    let(source,source_messages,messages)=match (&goal.spec.context,mode){
        (Context::Unbound,Mode::Dedicated)=>(None,0,vec![]),
        (Context::Mochi{conversation_id},Mode::Snapshot)=>{
            let snapshot=main.snapshot_public_context(&goal.spec.owner.provider,conversation_id).map_err(|_|Error::Conflict)?;
            (Some(conversation_id.clone()),snapshot.source_messages,snapshot.messages)
        },_=>return Err(Error::Conflict),
    };
    let binding=Binding{schema_version:1,goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),conversation_id:cid(goal),request_id:request.into(),mode,source_conversation_id:source,source_messages,copied_messages:messages.len(),seed_hash:fingerprint(&messages)?,prepared_at:crate::assistant_memory::now()};
    if !binding.valid(goal){return Err(Error::InvalidRecord);}
    let setup=format!(".context-setup-{}",nonce()?);let c=std::ffi::CString::new(setup.clone()).map_err(|_|Error::UnsafePath)?;
    if unsafe{libc::mkdirat(root.file.as_raw_fd(),c.as_ptr(),0o700)}!=0{return Err(Error::StorageUnavailable);}
    let staged=Dir::open(&root.path.join(&setup),true)?.ok_or(Error::StorageUnavailable)?;
    staged.write_new(HEADER,&serde_json::to_vec(&binding).map_err(|_|Error::InvalidRecord)?)?;staged.write_new(LOCK,b"")?;
    let saved=SavedConversation{id:binding.conversation_id.clone(),created:binding.prepared_at,updated:binding.prepared_at,messages,..Default::default()};
    let library=crate::assistant_memory::Library{pets:std::collections::HashMap::from([(binding.provider.clone(),saved)]),..Default::default()};
    crate::assistant_memory::save(&staged.path.join(JOURNAL),&library).map_err(|_|Error::StorageUnavailable)?;
    staged.sync()?;root.rename(&setup,&root,&goal.id,true)?;root.durable()?;
    open_directory(&root,goal,main)
}
pub(super)fn open(path:&Path,goal:&Record,main:&Chat)->Result<(Binding,Chat)>{
    let expected=goal.execution_context.as_ref().ok_or(Error::NotFound)?;
    let root=protocol_directory(path,PROTOCOL,false)?.ok_or(Error::NotFound)?;
    let(binding,chat)=open_directory(&root,goal,main)?;
    if &binding!=expected{return Err(Error::Conflict);}Ok((binding,chat))
}
