//! Isolated durable dialogue preparation. No model, tool or account calls.
use super::{model::*,store::{Dir,Guard,nonce,protocol_directory}};
use crate::chat::{Chat,ContextFence,SavedConversation};
use super::model::{ContextBinding as Binding,PreparationMode as Mode};
use std::{path::Path,sync::Arc,os::fd::AsRawFd};
use crate::pet_memory::{context::Capture,context_model::{Selection,Selected,same_selection}};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-goal-contexts\"}\n";
const HEADER:&str=".context.json";const LOCK:&str=".context.lock";const JOURNAL:&str="conversation.json";

struct Fence{dir:Dir,_lock:Guard,binding:Binding,header:Vec<u8>,memory:Option<Arc<Capture>>}
impl ContextFence for Fence{
    fn current(&self)->bool{
        if self.memory.as_ref().is_some_and(|m|m.verify(crate::assistant_memory::now()).is_err()){return false;}
        if self.dir.check().is_err()||self.dir.bytes(HEADER,16384).ok().flatten().is_none_or(|(b,_)|b!=self.header){return false;}
        let Some(current)=self.dir.open_file(LOCK,libc::O_RDONLY).ok().flatten()else{return false;};
        use std::os::unix::fs::MetadataExt;
        let(Ok(a),Ok(b))=(current.metadata(),self._lock.0.metadata())else{return false;};
        a.dev()==b.dev()&&a.ino()==b.ino()&&self.dir.open_file(JOURNAL,libc::O_RDONLY).ok().flatten().is_some_and(|f|f.metadata().is_ok_and(|m|m.len()<=crate::assistant_memory::MAX_BYTES))
    }
    fn goal_id(&self)->&str{&self.binding.goal_id}
    fn selected_memory_message(&self)->std::result::Result<Option<serde_json::Value>,String>{
        if !self.current(){return Err("O contexto ou as recordações do objetivo mudaram.".into());}
        match(&self.binding.memory,&self.memory){
            (None,None)=>Ok(None),
            (Some(_),Some(capture))=>capture.goal_message(&self.binding.goal_id,self.binding.source_conversation_id.as_deref(),crate::assistant_memory::now()).map(Some).map_err(|_|"As recordações do objetivo mudaram ou expiraram.".into()),
            _=>Err("A origem das recordações não corresponde ao objetivo.".into()),
        }
    }
}
fn root_lock(root:&Dir)->Result<Guard>{
    let file=root.open_file(".catalog.lock",libc::O_RDWR)?.ok_or(Error::UnsafePath)?;
    if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Error::Busy);}
    let guard=Guard(file);let current=root.open_file(".catalog.lock",libc::O_RDONLY)?.ok_or(Error::UnsafePath)?;
    use std::os::unix::fs::MetadataExt;
    let a=guard.0.metadata().map_err(|_|Error::UnsafePath)?;let b=current.metadata().map_err(|_|Error::UnsafePath)?;
    if a.dev()!=b.dev()||a.ino()!=b.ino(){return Err(Error::UnsafePath);}Ok(guard)
}
fn memory_at(binding:&Binding,path:&Path)->Result<Option<Arc<Capture>>>{
    let Some(selection)=&binding.memory else{return Ok(None);};
    let source=binding.source_conversation_id.as_deref().unwrap_or(&binding.conversation_id);
    Capture::at_references(path,&binding.pet_id,source,&selection.references,crate::assistant_memory::now()).map(|c|Some(Arc::new(c))).map_err(|_|Error::Conflict)
}
pub(super)fn validate_memory(binding:&Binding)->Result<()>{memory_at(binding,&Capture::catalog_path()).map(|_|())}
fn open_directory(root:&Dir,goal:&Record,main:&Chat,memory_path:&Path)->Result<(Binding,Chat)>{
    let dir=Dir::open(&root.path.join(&goal.id),true)?.ok_or(Error::NotFound)?;
    let file=dir.open_file(LOCK,libc::O_RDWR)?.ok_or(Error::UnsafePath)?;
    if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Error::Busy);}let lock=Guard(file);
    let(header,_)=dir.bytes(HEADER,16384)?.ok_or(Error::InvalidRecord)?;
    let binding:Binding=serde_json::from_slice(&header).map_err(|_|Error::InvalidRecord)?;
    if !binding.valid(goal){return Err(Error::Conflict);}
    let journal=dir.open_file(JOURNAL,libc::O_RDONLY)?.ok_or(Error::InvalidRecord)?;
    if journal.metadata().map_err(|_|Error::UnsafePath)?.len()>crate::assistant_memory::MAX_BYTES{return Err(Error::LimitReached);}
    let path=dir.path.join(JOURNAL);let memory=memory_at(&binding,memory_path)?;let fence=Arc::new(Fence{dir,_lock:lock,binding:binding.clone(),header,memory});
    if !fence.current(){return Err(Error::Conflict);}
    let chat=main.isolated_goal(path,&binding.provider,&binding.conversation_id,fence).map_err(|_|Error::InvalidRecord)?;
    Ok((binding,chat))
}
pub(super)fn prepare(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str)->Result<(Binding,Chat)>{
    prepare_with_memory(path,goal,main,mode,request,None)
}
pub(super)fn prepare_with_memory(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>)->Result<(Binding,Chat)>{
    prepare_at(path,goal,main,mode,request,selected,&Capture::catalog_path())
}
fn prepare_at(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>,memory_path:&Path)->Result<(Binding,Chat)>{
    goal.validate()?;if goal.state!=State::Draft||!key(request){return Err(Error::Conflict);}
    let root=protocol_directory(path,PROTOCOL,true)?.ok_or(Error::StorageUnavailable)?;let _root_guard=root_lock(&root)?;
    let target=root.path.join(&goal.id);
    if target.exists()||std::fs::symlink_metadata(&target).is_ok(){
        let(binding,chat)=open_directory(&root,goal,main,memory_path)?;
        if binding.request_id!=request||binding.mode!=mode||!same_selection(binding.memory.as_ref(),selected){return Err(Error::Conflict);}
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
    let memory=if let Some(selected)=selected{
        let source_cid=source.as_deref().map(str::to_string).unwrap_or_else(||cid(goal));
        let capture=Capture::at(memory_path,&goal.spec.owner.pet_id,&source_cid,selected,crate::assistant_memory::now()).map_err(|_|Error::Conflict)?;
        let selection=Selection{schema_version:1,references:capture.references().to_vec()};
        if !selection.valid(&goal.spec.owner.pet_id,source.as_deref()){return Err(Error::Conflict);}Some(selection)
    }else{None};
    let binding=Binding{schema_version:if memory.is_some(){2}else{1},memory,goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),conversation_id:cid(goal),request_id:request.into(),mode,source_conversation_id:source,source_messages,copied_messages:messages.len(),seed_hash:fingerprint(&messages)?,prepared_at:crate::assistant_memory::now()};
    if !binding.valid(goal){return Err(Error::InvalidRecord);}
    let setup=format!(".context-setup-{}",nonce()?);let c=std::ffi::CString::new(setup.clone()).map_err(|_|Error::UnsafePath)?;
    if unsafe{libc::mkdirat(root.file.as_raw_fd(),c.as_ptr(),0o700)}!=0{return Err(Error::StorageUnavailable);}
    let staged=Dir::open(&root.path.join(&setup),true)?.ok_or(Error::StorageUnavailable)?;
    staged.write_new(HEADER,&serde_json::to_vec(&binding).map_err(|_|Error::InvalidRecord)?)?;staged.write_new(LOCK,b"")?;
    let saved=SavedConversation{id:binding.conversation_id.clone(),created:binding.prepared_at,updated:binding.prepared_at,messages,..Default::default()};
    let library=crate::assistant_memory::Library{pets:std::collections::HashMap::from([(binding.provider.clone(),saved)]),..Default::default()};
    crate::assistant_memory::save(&staged.path.join(JOURNAL),&library).map_err(|_|Error::StorageUnavailable)?;
    staged.sync()?;root.rename(&setup,&root,&goal.id,true)?;root.durable()?;
    open_directory(&root,goal,main,memory_path)
}
pub(super)fn open(path:&Path,goal:&Record,main:&Chat)->Result<(Binding,Chat)>{
    open_at(path,goal,main,&Capture::catalog_path())
}
#[cfg(test)]
pub(super)fn open_memory_fixture(path:&Path,goal:&Record,main:&Chat,memory_path:&Path)->Result<(Binding,Chat)>{open_at(path,goal,main,memory_path)}
fn open_at(path:&Path,goal:&Record,main:&Chat,memory_path:&Path)->Result<(Binding,Chat)>{
    let expected=goal.execution_context.as_ref().ok_or(Error::NotFound)?;
    let root=protocol_directory(path,PROTOCOL,false)?.ok_or(Error::NotFound)?;
    let(binding,chat)=open_directory(&root,goal,main,memory_path)?;
    if &binding!=expected{return Err(Error::Conflict);}Ok((binding,chat))
}

#[cfg(test)]#[path="context_memory_tests.rs"]mod memory_tests;
