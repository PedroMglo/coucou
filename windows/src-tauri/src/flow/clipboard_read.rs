//! Fixed read-only child of this Coucou image. Not a model tool or a grant.
//! The route is logical, not yet a native seat/selection revision receipt.
use std::{ffi::OsString,io::{Read,Write},os::unix::{ffi::OsStrExt,fs::MetadataExt,process::CommandExt},process::Command,time::Duration};
use serde::{Deserialize,Serialize};
const FLAG:&str="--coucou-clipboard-read-v1";
const MAX_REQUEST:usize=1024;
const MAX_HEADER:usize=1024;
pub(super)const MAX_TEXT:usize=8*1024*1024;
const TIMEOUT:Duration=Duration::from_secs(3);
const MAX_MEMORY:u64=768*1024*1024;
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="lowercase")]
enum Backend{X11,Wayland}
#[derive(Clone,PartialEq,Eq)]
pub(super)struct Route{backend:Backend,display:OsString,runtime:Option<OsString>,auth:Option<OsString>,home:Option<OsString>}
impl Route{
 pub(super)fn capture()->Option<Self>{Self::from_env(|name|std::env::var_os(name))}
 fn from_env(mut env:impl FnMut(&str)->Option<OsString>)->Option<Self>{
  let valid=|v:&OsString|!v.is_empty()&&v.as_bytes().len()<=4096&&!v.as_bytes().iter().any(|b|b.is_ascii_control());
  if env("WAYLAND_SOCKET").is_some(){return None;}
  if let Some(display)=env("WAYLAND_DISPLAY"){
   if !valid(&display){return None;}let runtime=env("XDG_RUNTIME_DIR");
   if !std::path::Path::new(&display).is_absolute()&&runtime.as_ref().is_none_or(|v|!valid(v)||!std::path::Path::new(v).is_absolute()){return None;}
   return Some(Self{backend:Backend::Wayland,display,runtime,auth:None,home:None});
  }
  let display=env("DISPLAY")?;if !valid(&display){return None;}
  let auth=env("XAUTHORITY");let home=env("HOME");
  if auth.as_ref().is_some_and(|v|!valid(v))||home.as_ref().is_some_and(|v|!valid(v)){return None;}
  Some(Self{backend:Backend::X11,display,runtime:None,auth,home})
 }
 pub(super)fn current(&self)->bool{Self::capture().as_ref()==Some(self)}
 pub(super)fn hash(&self)->String{
  let mut bytes=vec![match self.backend{Backend::X11=>1,Backend::Wayland=>2}];
  for value in [Some(&self.display),self.runtime.as_ref(),self.auth.as_ref(),self.home.as_ref()]{
   match value{Some(v)=>{bytes.push(1);bytes.extend_from_slice(&(v.as_bytes().len()as u64).to_le_bytes());bytes.extend_from_slice(v.as_bytes());},None=>bytes.push(0)}
  }sha(&bytes)
 }
 pub(super)fn apply(&self,command:&mut Command){
  for name in ["DISPLAY","WAYLAND_DISPLAY","WAYLAND_SOCKET","XDG_RUNTIME_DIR","XAUTHORITY"]{command.env_remove(name);}
  match self.backend{
   Backend::Wayland=>{command.env("WAYLAND_DISPLAY",&self.display);if let Some(runtime)=&self.runtime{command.env("XDG_RUNTIME_DIR",runtime);}},
   Backend::X11=>{command.env("DISPLAY",&self.display);if let Some(auth)=&self.auth{command.env("XAUTHORITY",auth);}match &self.home{Some(home)=>{command.env("HOME",home);},None=>{command.env_remove("HOME");}}},
  }
 }
}
pub(super)fn sha(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
pub(super)fn hex(s:&str,n:usize)->bool{s.len()==n&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Request{version:u32,request_id:String,backend:Backend,route_hash:String}
impl Request{
 fn new(route:&Route)->Result<Self,Failure>{let mut nonce=[0;16];use ring::rand::SecureRandom;ring::rand::SystemRandom::new().fill(&mut nonce).map_err(|_|Failure::Unavailable)?;Ok(Self{version:1,request_id:nonce.iter().map(|b|format!("{b:02x}")).collect(),backend:route.backend,route_hash:route.hash()})}
 fn valid(&self)->bool{self.version==1&&hex(&self.request_id,32)&&hex(&self.route_hash,64)}
 fn bytes(&self)->Result<Vec<u8>,Failure>{let mut bytes=serde_json::to_vec(self).map_err(|_|Failure::Protocol)?;bytes.push(b'\n');if bytes.len()>MAX_REQUEST{return Err(Failure::Protocol);}Ok(bytes)}
}
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
enum Status{Text,Empty,Vacant,Unavailable,TooLarge}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Header{version:u32,request_id:String,route_hash:String,status:Status,bytes:usize,sha256:String}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(super)enum Failure{Retired,Unavailable,Busy,Transport,Protocol}
// Deliberately no Debug: a transcript/clipboard must never enter an error log.
pub(super)enum Reply{Text(String),Empty,Vacant,Unavailable,TooLarge}
fn decode(bytes:Vec<u8>,request:&Request)->Result<Reply,Failure>{
 let at=bytes.iter().take(MAX_HEADER+1).position(|b|*b==b'\n').ok_or(Failure::Protocol)?;
 if at>MAX_HEADER{return Err(Failure::Protocol);}
 let header:Header=serde_json::from_slice(&bytes[..at]).map_err(|_|Failure::Protocol)?;let body=&bytes[at+1..];
 if header.version!=1||header.request_id!=request.request_id||header.route_hash!=request.route_hash||header.bytes>MAX_TEXT||body.len()!=header.bytes||header.sha256!=sha(body){return Err(Failure::Protocol);}
 match header.status{
  Status::Text=>String::from_utf8(body.to_vec()).map(Reply::Text).map_err(|_|Failure::Protocol),
  status=>{if !body.is_empty(){return Err(Failure::Protocol);}Ok(match status{Status::Empty=>Reply::Empty,Status::Vacant=>Reply::Vacant,Status::TooLarge=>Reply::TooLarge,_=>Reply::Unavailable})}
 }
}
pub(super)fn constrain(command:&mut Command){
 constrain_mode(command,true)
}
pub(super)fn constrain_resident(command:&mut Command){
 constrain_mode(command,false)
}
fn constrain_mode(command:&mut Command,parent_thread:bool){
 let parent=std::process::id()as libc::pid_t;
 unsafe{command.pre_exec(move||{
  let memory=libc::rlimit{rlim_cur:MAX_MEMORY as libc::rlim_t,rlim_max:MAX_MEMORY as libc::rlim_t};
  let cpu=libc::rlimit{rlim_cur:2,rlim_max:2};
  if libc::setrlimit(libc::RLIMIT_AS,&memory)!=0||libc::setrlimit(libc::RLIMIT_CPU,&cpu)!=0||libc::prctl(libc::PR_SET_PDEATHSIG,if parent_thread{libc::SIGKILL}else{0})!=0{return Err(std::io::Error::last_os_error());}
  if libc::getppid()!=parent{return Err(std::io::Error::from_raw_os_error(libc::ECANCELED));}Ok(())
 });}
}
fn execute(route:&Route,request:&Request,command:Command,current:&dyn Fn()->bool)->Result<Reply,Failure>{
 execute_with(route,request,command,current,&||route.current())
}
fn execute_with(route:&Route,request:&Request,mut command:Command,current:&dyn Fn()->bool,route_current:&dyn Fn()->bool)->Result<Reply,Failure>{
 if !current(){return Err(Failure::Retired);}
 route.apply(&mut command);constrain(&mut command);
 let session=crate::pet_helper::Session::acquire(crate::pet_helper::Lane::ClipboardRead,TIMEOUT).map_err(|_|Failure::Busy)?;
 let bytes=session.run_sync_input(command,Some(request.bytes()?),MAX_TEXT+MAX_HEADER+1,TIMEOUT,||if current()&&route_current(){Ok(())}else{Err("O contexto do clipboard mudou antes da leitura.".into())},&||current()&&route_current()).map_err(|_|Failure::Transport)?;
 if !current()||!route_current(){return Err(Failure::Retired);}let result=decode(bytes,request)?;
 if !current()||!route_current(){return Err(Failure::Retired);}Ok(result)
}
pub(super)fn read(current:&dyn Fn()->bool)->Result<Reply,Failure>{
 if !current(){return Err(Failure::Retired);}let route=Route::capture().ok_or(Failure::Unavailable)?;let request=Request::new(&route)?;
 // Linux resolves this to the executing image, including after an on-disk
 // replacement. The child cannot drift to a different installed binary.
 let command=crate::pet_helper::command("/proc/self/exe",&[FLAG.into()]);execute(&route,&request,command,current)
}
pub(super)fn authenticated_parent()->bool{
 let parent=unsafe{libc::getppid()};if parent<=1||unsafe{libc::getuid()}!=unsafe{libc::geteuid()}{return false;}
 let Ok(me)=std::fs::metadata("/proc/self/exe")else{return false;};let Ok(other)=std::fs::metadata(format!("/proc/{parent}/exe"))else{return false;};let Ok(process)=std::fs::metadata(format!("/proc/{parent}"))else{return false;};
 process.uid()==unsafe{libc::geteuid()}&&(me.dev(),me.ino())==(other.dev(),other.ino())
}
fn serve(mut input:impl Read,mut output:impl Write,route:&Route,read:impl FnOnce()->Reply)->Result<(),()>{
 let mut bytes=Vec::new();input.by_ref().take((MAX_REQUEST+1)as u64).read_to_end(&mut bytes).map_err(|_|())?;
 if bytes.len()>MAX_REQUEST||bytes.last()!=Some(&b'\n')||bytes[..bytes.len()-1].contains(&b'\n'){return Err(());}
 let request:Request=serde_json::from_slice(&bytes[..bytes.len()-1]).map_err(|_|())?;
 if !request.valid()||request.backend!=route.backend||request.route_hash!=route.hash(){return Err(());}
 let reply=read();let(status,body)=match &reply{Reply::Text(text)if text.len()<=MAX_TEXT=>(Status::Text,text.as_bytes()),Reply::Text(_)|Reply::TooLarge=>(Status::TooLarge,&[][..]),Reply::Empty=>(Status::Empty,&[][..]),Reply::Vacant=>(Status::Vacant,&[][..]),Reply::Unavailable=>(Status::Unavailable,&[][..])};
 let header=Header{version:1,request_id:request.request_id,route_hash:request.route_hash,status,bytes:body.len(),sha256:sha(body)};let mut bytes=serde_json::to_vec(&header).map_err(|_|())?;if bytes.len()>MAX_HEADER{return Err(());}bytes.push(b'\n');output.write_all(&bytes).and_then(|_|output.write_all(body)).and_then(|_|output.flush()).map_err(|_|())
}
/// Only this exact internal mode is handled, before any Tauri/profile startup.
/// A shell/model process cannot turn it into an ungoverned clipboard endpoint.
pub(crate)fn entry()->bool{
 let args:Vec<_>=std::env::args_os().collect();if args.get(1).is_none_or(|a|a!=FLAG){return false;}
 if args.len()!=2||!authenticated_parent(){std::process::exit(70);}
 let Some(route)=Route::capture()else{std::process::exit(65);};
 let result=serve(std::io::stdin().lock(),std::io::stdout().lock(),&route,||{
  if route.backend==Backend::X11{
   use x11rb::protocol::xproto::ConnectionExt;
   let vacant=(||{let(conn,_)=x11rb::connect(None).map_err(|_|())?;let selection=conn.intern_atom(false,b"CLIPBOARD").map_err(|_|())?.reply().map_err(|_|())?.atom;let owner=conn.get_selection_owner(selection).map_err(|_|())?.reply().map_err(|_|())?.owner;Ok::<_,()>(owner==x11rb::NONE)})();
   match vacant{Ok(true)=>return Reply::Vacant,Err(_)=>return Reply::Unavailable,_=>{}}
  }
  match arboard::Clipboard::new().and_then(|mut clipboard|clipboard.get_text()){
   Ok(text)=>Reply::Text(text),Err(arboard::Error::ContentNotAvailable)=>Reply::Empty,Err(_)=>Reply::Unavailable,
  }
 });
 std::process::exit(if result.is_ok(){0}else{65})
}
#[cfg(test)]#[path="clipboard_read_tests.rs"]mod tests;
