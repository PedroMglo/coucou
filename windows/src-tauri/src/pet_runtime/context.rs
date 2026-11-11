//! Isolated durable dialogue preparation. No model, tool or account calls.
use super::{model::*,store::{Dir,Guard,nonce,protocol_directory}};
use crate::chat::{Chat,ContextFence,SavedConversation};
use super::model::{ContextBinding as Binding,PreparationMode as Mode};
use std::{path::Path,sync::Arc,os::fd::AsRawFd};
use crate::pet_memory::{context::Capture,context_model::{Selection,Selected,same_selection}};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-goal-contexts\"}\n";
const MAX_CALENDAR_HEADER:usize=128*1024;
const HEADER:&str=".context.json";const LOCK:&str=".context.lock";const JOURNAL:&str="conversation.json";

struct Fence{dir:Dir,_lock:Guard,binding:Binding,header:Vec<u8>,header_stamp:super::store::Stamp,calendars:Option<Arc<crate::pet_calendar::goal::Captured>>,memory:Option<Arc<Capture>>,received:Option<Arc<super::received_context::Captured>>,source:Option<Arc<super::received_result_data::OriginDependencies>>}
impl ContextFence for Fence{
    fn current(&self)->bool{
        if self.calendars.as_ref().is_some_and(|c|c.current().is_err()){return false;}
        if self.source.as_ref().is_some_and(|s|!s.current(crate::assistant_memory::now())){return false;}
        if self.memory.as_ref().is_some_and(|m|m.verify(crate::assistant_memory::now()).is_err())||self.received.as_ref().is_some_and(|m|m.verify(crate::assistant_memory::now()).is_err()){return false;}
        if self.dir.check().is_err()||(if self.calendars.is_some(){!self.dir.has_stamp(HEADER,&self.header_stamp).unwrap_or(false)}else{self.dir.bytes(HEADER,16384).ok().flatten().is_none_or(|(b,_)|b!=self.header)}){return false;}
        let Some(current)=self.dir.open_file(LOCK,libc::O_RDONLY).ok().flatten()else{return false;};
        use std::os::unix::fs::MetadataExt;
        let(Ok(a),Ok(b))=(current.metadata(),self._lock.0.metadata())else{return false;};
        a.dev()==b.dev()&&a.ino()==b.ino()&&self.dir.open_file(JOURNAL,libc::O_RDONLY).ok().flatten().is_some_and(|f|f.metadata().is_ok_and(|m|m.len()<=crate::assistant_memory::MAX_BYTES))
    }
    fn goal_id(&self)->&str{&self.binding.goal_id}
    fn selected_memory_message(&self)->std::result::Result<Option<serde_json::Value>,String>{
        if !self.current(){return Err("O contexto ou as recordações do objetivo mudaram.".into());}
        if self.binding.memory.is_some()!=self.memory.is_some()||self.binding.received_request.is_some()!=self.received.is_some()||self.binding.calendars.is_some()!=self.calendars.is_some(){return Err("A seleção de dados do objetivo mudou.".into());}
        let notes=self.memory.as_ref().map(|c|c.goal_message(&self.binding.goal_id,self.binding.source_conversation_id.as_deref(),crate::assistant_memory::now()).map_err(str::to_string)).transpose()?;
        let received=self.received.as_ref().map(|c|c.message(&self.binding.goal_id,crate::assistant_memory::now()).map_err(|_|"O pedido recebido mudou ou expirou.".to_string())).transpose()?;
        let existing=super::goal_data_message::compose(&self.binding.goal_id,notes,received).map_err(|_|"Os dados do objetivo excedem o limite ou mudaram.".to_string())?;
        let calendars=self.calendars.as_ref().map(|c|c.data().cloned().map_err(str::to_string)).transpose()?;
        super::goal_data_message::with_calendars(&self.binding.goal_id,existing,calendars).map_err(|_|"Os calendários e dados selecionados excedem o limite do objetivo.".to_string())
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
fn calendars_at(binding:&Binding,goal:&Record)->Result<Option<Arc<crate::pet_calendar::goal::Captured>>>{
 match &binding.calendars{None=>Ok(None),Some(c)=>crate::pet_calendar::goal::Captured::reopen(goal,&c.selection,c.data.clone(),Arc::new(||true)).map(|c|Some(Arc::new(c))).map_err(|_|Error::Conflict)}
}
pub(super)fn same_calendars(binding:&Binding,request:Option<&crate::pet_calendar::goal::Request>)->bool{match(&binding.calendars,request){(None,None)=>true,(Some(c),Some(r))=>c.matches(r),_=>false}}
fn received_at(binding:&Binding,goal:&Record,authority:Option<&Path>,memory:&Path)->Result<Option<Arc<super::received_context::Captured>>>{
 match &binding.received_request{None=>Ok(None),Some(selected)=>super::received_context::Captured::at_with_memory(authority.ok_or(Error::Conflict)?,memory,goal,selected,crate::assistant_memory::now()).map(|c|Some(Arc::new(c)))}
}
fn open_directory(root:&Dir,goal:&Record,main:&Chat,memory_path:&Path,authority:Option<&Path>)->Result<(Binding,Chat)>{
    let dir=Dir::open(&root.path.join(&goal.id),true)?.ok_or(Error::NotFound)?;
    let file=dir.open_file(LOCK,libc::O_RDWR)?.ok_or(Error::UnsafePath)?;
    if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(Error::Busy);}let lock=Guard(file);
    let(header,header_stamp)=dir.bytes(HEADER,MAX_CALENDAR_HEADER)?.ok_or(Error::InvalidRecord)?;
    let binding:Binding=serde_json::from_slice(&header).map_err(|_|Error::InvalidRecord)?;
    if !binding.valid(goal)||header.len()>if binding.calendars.is_some(){MAX_CALENDAR_HEADER}else{16384}{return Err(Error::Conflict);}
    let journal=dir.open_file(JOURNAL,libc::O_RDONLY)?.ok_or(Error::InvalidRecord)?;
    if journal.metadata().map_err(|_|Error::UnsafePath)?.len()>crate::assistant_memory::MAX_BYTES{return Err(Error::LimitReached);}
    let path=dir.path.join(JOURNAL);let memory=memory_at(&binding,memory_path)?;let received=received_at(&binding,goal,authority,memory_path)?;let source=super::received_result_data::OriginDependencies::capture(authority,memory_path,goal,crate::assistant_memory::now())?;let fence=Arc::new(Fence{dir,_lock:lock,binding:binding.clone(),header,header_stamp,calendars:calendars_at(&binding,goal)?,memory,received,source});
    if !fence.current(){return Err(Error::Conflict);}
    let chat=main.isolated_goal(path,&binding.provider,&binding.conversation_id,fence).map_err(|_|Error::InvalidRecord)?;
    Ok((binding,chat))
}
pub(super)fn prepare(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str)->Result<(Binding,Chat)>{
    prepare_with_memory(path,goal,main,mode,request,None)
}
pub(super)fn prepare_with_memory(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>)->Result<(Binding,Chat)>{
    prepare_at_data(path,goal,main,mode,request,selected,&Capture::catalog_path(),None,false)
}
#[cfg(test)]fn prepare_at(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>,memory_path:&Path)->Result<(Binding,Chat)>{prepare_at_data(path,goal,main,mode,request,selected,memory_path,None,false)}
pub(super)fn prepare_with_data(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>,authority:&Path,include_received:bool)->Result<(Binding,Chat)>{prepare_at_data(path,goal,main,mode,request,selected,&Capture::catalog_path(),Some(authority),include_received)}
#[cfg(test)]pub(super)fn prepare_selected_data_fixture(path:&Path,goal:&Record,main:&Chat,selected:Option<&[Selected]>,memory:&Path,authority:&Path,include_received:bool)->Result<(Binding,Chat)>{prepare_at_data(path,goal,main,Mode::Dedicated,"funded-model-context",selected,memory,Some(authority),include_received)}
fn prepare_at_data(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>,memory_path:&Path,authority:Option<&Path>,include_received:bool)->Result<(Binding,Chat)>{
 prepare_at_data_calendars(path,goal,main,mode,request,selected,memory_path,authority,include_received,None,Arc::new(||true))
}
pub(super)fn prepare_with_calendar_data(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>,authority:&Path,calendars:Option<&crate::pet_calendar::goal::Request>,current:Arc<dyn Fn()->bool+Send+Sync>)->Result<(Binding,Chat)>{
 prepare_at_data_calendars(path,goal,main,mode,request,selected,&Capture::catalog_path(),Some(authority),false,calendars,current)
}
fn prepare_at_data_calendars(path:&Path,goal:&Record,main:&Chat,mode:Mode,request:&str,selected:Option<&[Selected]>,memory_path:&Path,authority:Option<&Path>,include_received:bool,selected_calendars:Option<&crate::pet_calendar::goal::Request>,calendar_current:Arc<dyn Fn()->bool+Send+Sync>)->Result<(Binding,Chat)>{
    goal.validate()?;if selected_calendars.is_some()&&(include_received||mode!=Mode::Dedicated){return Err(Error::InvalidInput);}if include_received&&selected.is_some_and(|s|s.len()>=crate::pet_memory::context_model::MAX_SELECTED){return Err(Error::LimitReached);}if include_received&&!goal.spec.owner.local_engine(){return Err(Error::InvalidInput);}if goal.state!=State::Draft||!key(request){return Err(Error::Conflict);}
    let root=protocol_directory(path,PROTOCOL,true)?.ok_or(Error::StorageUnavailable)?;let _root_guard=root_lock(&root)?;
    let target=root.path.join(&goal.id);
    if target.exists()||std::fs::symlink_metadata(&target).is_ok(){
        let(binding,chat)=open_directory(&root,goal,main,memory_path,authority)?;
        if binding.request_id!=request||binding.mode!=mode||binding.received_request.is_some()!=include_received||!same_selection(binding.memory.as_ref(),selected)||!same_calendars(&binding,selected_calendars){return Err(Error::Conflict);}
        // Reconcile a previous uncertain publication before linking the goal.
        root.durable()?;std::fs::File::open(&target).and_then(|f|f.sync_all()).map_err(|_|Error::StorageUnavailable)?;
        return Ok((binding,chat));
    }
    if root.names()?.iter().filter(|n|goal_id(n)).count()>=MAX_GOALS{return Err(Error::LimitReached);}
    let(source,source_messages,messages)=match (&goal.spec.context,mode){
        (Context::Unbound,Mode::Dedicated)=>(None,0,vec![]),
        (Context::Mochi{conversation_id},Mode::Dedicated)if selected_calendars.is_some()=>{
            if !main.contains_public_conversation(&goal.spec.owner.provider,conversation_id).map_err(|_|Error::Conflict)?{return Err(Error::Conflict);}
            (Some(conversation_id.clone()),0,vec![])
        },
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
    let received_request=if include_received{let selected=super::received_context::Selection::from_goal(goal)?;super::received_context::Captured::at_with_memory(authority.ok_or(Error::Conflict)?,memory_path,goal,&selected,crate::assistant_memory::now())?;Some(selected)}else{None};
    let calendars=selected_calendars.map(|request|crate::pet_calendar::goal::Captured::native(goal,request.clone(),calendar_current).and_then(|c|c.context()).map_err(|_|Error::Conflict)).transpose()?;
    let binding=Binding{calendars:calendars.clone(),schema_version:if calendars.as_ref().is_some_and(|c|c.selection.pet_scope()){11}else if calendars.is_some(){10}else if received_request.as_ref().is_some_and(|r|r.schema_version==7){9}else if received_request.as_ref().is_some_and(|r|r.schema_version==6){8}else if received_request.as_ref().is_some_and(|r|r.schema_version==5){7}else if received_request.as_ref().is_some_and(|r|r.schema_version==4){6}else if received_request.as_ref().is_some_and(|r|r.schema_version==3){5}else if received_request.as_ref().is_some_and(|r|r.schema_version==2){4}else if received_request.is_some(){3}else if memory.is_some(){2}else{1},received_request,memory,goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),conversation_id:cid(goal),request_id:request.into(),mode,source_conversation_id:source,source_messages,copied_messages:messages.len(),seed_hash:fingerprint(&messages)?,prepared_at:crate::assistant_memory::now()};
    if !binding.valid(goal){return Err(Error::InvalidRecord);}
    let header=serde_json::to_vec(&binding).map_err(|_|Error::InvalidRecord)?;if header.len()>if binding.calendars.is_some(){MAX_CALENDAR_HEADER}else{16384}{return Err(Error::LimitReached);}
    let setup=format!(".context-setup-{}",nonce()?);let c=std::ffi::CString::new(setup.clone()).map_err(|_|Error::UnsafePath)?;
    if unsafe{libc::mkdirat(root.file.as_raw_fd(),c.as_ptr(),0o700)}!=0{return Err(Error::StorageUnavailable);}
    let staged=Dir::open(&root.path.join(&setup),true)?.ok_or(Error::StorageUnavailable)?;
    staged.write_new(HEADER,&header)?;staged.write_new(LOCK,b"")?;
    let saved=SavedConversation{id:binding.conversation_id.clone(),created:binding.prepared_at,updated:binding.prepared_at,messages,..Default::default()};
    let library=crate::assistant_memory::Library{pets:std::collections::HashMap::from([(binding.provider.clone(),saved)]),..Default::default()};
    crate::assistant_memory::save(&staged.path.join(JOURNAL),&library).map_err(|_|Error::StorageUnavailable)?;
    staged.sync()?;root.rename(&setup,&root,&goal.id,true)?;root.durable()?;
    open_directory(&root,goal,main,memory_path,authority)
}
pub(super)fn open(path:&Path,goal:&Record,main:&Chat)->Result<(Binding,Chat)>{
    open_at_data(path,goal,main,&Capture::catalog_path(),None)
}
#[cfg(test)]
pub(super)fn open_memory_fixture(path:&Path,goal:&Record,main:&Chat,memory_path:&Path)->Result<(Binding,Chat)>{open_at(path,goal,main,memory_path)}
#[cfg(test)]fn open_at(path:&Path,goal:&Record,main:&Chat,memory_path:&Path)->Result<(Binding,Chat)>{open_at_data(path,goal,main,memory_path,None)}
pub(super)fn open_with_authority(path:&Path,goal:&Record,main:&Chat,authority:&Path)->Result<(Binding,Chat)>{open_at_data(path,goal,main,&Capture::catalog_path(),Some(authority))}
/// Native-user observation of already persisted public dialogue. No Chat,
/// selected-data capture, authority grant or new model use can escape this path.
pub(super)fn read_snapshot(path:&Path,goal:&Record)->Result<(Binding,crate::chat::PublicSnapshot)>{
    goal.validate()?;let expected=goal.execution_context.as_ref().ok_or(Error::NotFound)?;
    let root=protocol_directory(path,PROTOCOL,false)?.ok_or(Error::NotFound)?;
    let dir=Dir::open(&root.path.join(&goal.id),true)?.ok_or(Error::NotFound)?;
    let file=dir.open_file(LOCK,libc::O_RDONLY)?.ok_or(Error::UnsafePath)?;
    if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_SH|libc::LOCK_NB)}!=0{return Err(Error::Busy);}let lock=Guard(file);
    let(header,stamp)=dir.bytes(HEADER,MAX_CALENDAR_HEADER)?.ok_or(Error::InvalidRecord)?;
    let binding:Binding=serde_json::from_slice(&header).map_err(|_|Error::InvalidRecord)?;
    if !binding.valid(goal)||&binding!=expected||header.len()>if binding.calendars.is_some(){MAX_CALENDAR_HEADER}else{16384}{return Err(Error::Conflict);}
    let(bytes,journal_stamp)=dir.bytes(JOURNAL,crate::assistant_memory::MAX_BYTES as usize)?.ok_or(Error::InvalidRecord)?;
    let library=crate::assistant_memory::inspect_goal_bytes(&bytes,&binding.provider,&binding.conversation_id).map_err(|_|Error::InvalidRecord)?;
    let saved=library.pets.get(&binding.provider).ok_or(Error::InvalidRecord)?;
    let snapshot=Chat::public_snapshot(&saved.messages).map_err(|_|Error::InvalidRecord)?;
    let current=dir.open_file(LOCK,libc::O_RDONLY)?.ok_or(Error::UnsafePath)?;
    use std::os::unix::fs::MetadataExt;
    let a=lock.0.metadata().map_err(|_|Error::UnsafePath)?;let b=current.metadata().map_err(|_|Error::UnsafePath)?;
    if a.dev()!=b.dev()||a.ino()!=b.ino()||!dir.has_stamp(HEADER,&stamp)?||!dir.has_stamp(JOURNAL,&journal_stamp)?{return Err(Error::Conflict);}
    Ok((binding,snapshot))
}
#[cfg(test)]pub(super)fn open_data_fixture(path:&Path,goal:&Record,main:&Chat,memory_path:&Path,authority:&Path)->Result<(Binding,Chat)>{open_at_data(path,goal,main,memory_path,Some(authority))}
fn open_at_data(path:&Path,goal:&Record,main:&Chat,memory_path:&Path,authority:Option<&Path>)->Result<(Binding,Chat)>{
    let expected=goal.execution_context.as_ref().ok_or(Error::NotFound)?;
    let root=protocol_directory(path,PROTOCOL,false)?.ok_or(Error::NotFound)?;
    let(binding,chat)=open_directory(&root,goal,main,memory_path,authority)?;
    if &binding!=expected{return Err(Error::Conflict);}Ok((binding,chat))
}

#[cfg(test)]#[path="context_memory_tests.rs"]mod memory_tests;
