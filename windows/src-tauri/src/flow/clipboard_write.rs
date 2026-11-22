//! Fixed same-image clipboard driver. A live owner serves already authorized
//! data, but each new write requires a fresh bounded request and Current.
use std::{io::{Read,Write},sync::{Mutex,TryLockError,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use serde::{Serialize,Deserialize};
use super::{clipboard_read::{Route,MAX_TEXT,sha,hex,constrain_resident,authenticated_parent},paste::Mutation};
use crate::pet_helper::{Resident,ExchangeError,Lane};
const FLAG:&str="--coucou-clipboard-write-v1";
const MAX_HEADER:usize=1024;
const TIMEOUT:Duration=Duration::from_secs(3);
static DRIVER:Mutex<Option<Driver>>=Mutex::new(None);
static STOPPED:AtomicBool=AtomicBool::new(false);
struct Driver{route:Route,owner:Resident,next:u64}
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
enum Phase{Prepare,Commit,Abort}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Header{version:u32,phase:Phase,sequence:u64,request_id:String,route_hash:String,deadline_ms:u64,bytes:usize,sha256:String}
#[derive(Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
enum Status{Prepared,Accepted,Aborted,Retired,Uncertain}
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Ack{version:u32,phase:Phase,sequence:u64,request_id:String,route_hash:String,bytes:usize,sha256:String,status:Status}
fn monotonic_ms()->Option<u64>{
    let mut t=libc::timespec{tv_sec:0,tv_nsec:0};
    if unsafe{libc::clock_gettime(libc::CLOCK_MONOTONIC,&mut t)}!=0||t.tv_sec<0||t.tv_nsec<0{return None;}
    (t.tv_sec as u64).checked_mul(1000)?.checked_add(t.tv_nsec as u64/1_000_000)
}
fn fresh(h:&Header,now:u64)->bool{h.deadline_ms>now&&h.deadline_ms-now<=3000}
fn valid(h:&Header,route:&str,expected:u64)->bool{
    h.version==1&&h.sequence==expected&&expected<u64::MAX&&hex(&h.request_id,32)&&h.route_hash==route&&hex(&h.route_hash,64)&&h.bytes<=MAX_TEXT&&hex(&h.sha256,64)
}
fn frame(h:&Header,text:&str)->Result<Vec<u8>,()>{
    if h.phase!=Phase::Prepare||text.len()!=h.bytes||text.len()>MAX_TEXT||sha(text.as_bytes())!=h.sha256{return Err(());}
    let mut bytes=serde_json::to_vec(h).map_err(|_|())?;if bytes.len()>MAX_HEADER{return Err(());}bytes.push(b'\n');bytes.extend_from_slice(text.as_bytes());Ok(bytes)
}
fn ack(bytes:&[u8],h:&Header)->Result<Status,()>{
    if bytes.len()>MAX_HEADER+1||bytes.last()!=Some(&b'\n')||bytes[..bytes.len()-1].contains(&b'\n'){return Err(());}
    let a:Ack=serde_json::from_slice(&bytes[..bytes.len()-1]).map_err(|_|())?;
    if a.version!=1||a.phase!=h.phase||a.sequence!=h.sequence||a.request_id!=h.request_id||a.route_hash!=h.route_hash||a.bytes!=h.bytes||a.sha256!=h.sha256{return Err(());}Ok(a.status)
}
fn control(h:&Header,phase:Phase)->Result<(Header,Vec<u8>),()>{
    if phase==Phase::Prepare{return Err(());}let mut h=h.clone();h.phase=phase;let mut bytes=serde_json::to_vec(&h).map_err(|_|())?;if bytes.len()>MAX_HEADER{return Err(());}bytes.push(b'\n');Ok((h,bytes))
}
enum Disposition{Idle,Advance,Retire}
fn transact(h:&Header,text:&str,end:Instant,current:&dyn Fn()->bool,mut exchange:impl FnMut(&[u8],Instant,&dyn Fn()->bool)->Result<Vec<u8>,ExchangeError>)->(Mutation,Disposition){
    let Ok(input)=frame(h,text)else{return(Mutation::Skipped,Disposition::Idle);};
    match exchange(&input,end,current){
        Err(ExchangeError::NotStarted)=>return(Mutation::Skipped,Disposition::Idle),
        Err(ExchangeError::Uncertain)=>return(Mutation::Uncertain,Disposition::Retire),
        Ok(bytes)=>match ack(&bytes,h){
            Ok(Status::Prepared)=>{},Ok(Status::Retired)=>return(Mutation::Skipped,Disposition::Advance),
            _=>return(Mutation::Uncertain,Disposition::Retire),
        },
    }
    // Native initialization has finished. Recheck authority at commit, never
    // granting a delayed initialization permission to write by itself.
    if current()&&Instant::now()<end{
        let Ok((commit,input))=control(h,Phase::Commit)else{return(Mutation::Uncertain,Disposition::Retire);};
        match exchange(&input,end,current){
            Err(ExchangeError::NotStarted)=>{}, // Still prepared: cleanup only.
            Err(ExchangeError::Uncertain)=>return(Mutation::Uncertain,Disposition::Retire),
            Ok(bytes)=>return match ack(&bytes,&commit){
                Ok(Status::Accepted)if current()&&Instant::now()<end=>(Mutation::Done,Disposition::Advance),
                Ok(Status::Retired)=>(Mutation::Skipped,Disposition::Advance),
                _=>(Mutation::Uncertain,Disposition::Retire),
            },
        }
    }
    // This fixed Abort can only discard the prepared bytes. It cannot write,
    // renew the original deadline, or be substituted for Commit after Stop.
    let Ok((abort,input))=control(h,Phase::Abort)else{return(Mutation::Uncertain,Disposition::Retire);};
    match exchange(&input,Instant::now()+Duration::from_millis(250),&||true){
        Ok(bytes)if matches!(ack(&bytes,&abort),Ok(Status::Aborted))=>(Mutation::Skipped,Disposition::Advance),
        _=>(Mutation::Uncertain,Disposition::Retire),
    }
}
fn nonce()->Option<String>{let mut bytes=[0;16];use ring::rand::SecureRandom;ring::rand::SystemRandom::new().fill(&mut bytes).ok()?;Some(bytes.iter().map(|b|format!("{b:02x}")).collect())}
pub(super)fn write(text:&str,current:&dyn Fn()->bool)->Mutation{
    let end=Instant::now()+TIMEOUT;
    if text.len()>MAX_TEXT||STOPPED.load(Ordering::Acquire)||!current(){return Mutation::Skipped;}
    let(Some(route),Some(request_id),Some(deadline_ms))=(Route::capture(),nonce(),monotonic_ms().and_then(|n|n.checked_add(3000)))else{return Mutation::Skipped;};
    let route_hash=route.hash();let digest=sha(text.as_bytes());
    let allowed=||!STOPPED.load(Ordering::Acquire)&&Instant::now()<end&&current()&&route.current();
    let mut slot=loop{
        if !allowed(){return Mutation::Skipped;}
        match DRIVER.try_lock(){Ok(slot)=>break slot,Err(TryLockError::WouldBlock)=>std::thread::sleep(Duration::from_millis(2)),Err(_)=>return Mutation::Skipped}
    };
    if !allowed(){return Mutation::Skipped;}
    // Never retarget or withdraw an existing owner on another logical route.
    if slot.as_ref().is_some_and(|d|d.route!=route){return Mutation::Skipped;}
    if slot.as_ref().is_some_and(|d|!d.owner.alive()){drop(slot.take());}
    if slot.is_none(){
        let mut command=crate::pet_helper::command("/proc/self/exe",&[FLAG.into()]);route.apply(&mut command);constrain_resident(&mut command);
        let Ok(owner)=Resident::start(Lane::ClipboardWrite,command,end,&allowed)else{return Mutation::Skipped;};
        *slot=Some(Driver{route:route.clone(),owner,next:1});
    }
    let driver=slot.as_mut().unwrap();let sequence=driver.next;
    let Some(next)=sequence.checked_add(1)else{return Mutation::Skipped;};
    let header=Header{version:1,phase:Phase::Prepare,sequence,request_id,route_hash,deadline_ms,bytes:text.len(),sha256:digest};
    let(result,disposition)=transact(&header,text,end,&allowed,|input,end,current|driver.owner.exchange_line(input,MAX_HEADER+1,end,current));
    match disposition{Disposition::Idle=>{},Disposition::Advance=>driver.next=next,Disposition::Retire=>drop(slot.take())}result
}
fn header(input:&mut impl Read)->Result<Option<Header>,()>{
    let mut bytes=Vec::new();let mut byte=[0];
    loop{match input.read(&mut byte){
        Ok(0)if bytes.is_empty()=>return Ok(None),Ok(0)=>return Err(()),
        Ok(_)=>{if byte[0]==b'\n'{break;}bytes.push(byte[0]);if bytes.len()>MAX_HEADER{return Err(());}},
        Err(e)if e.kind()==std::io::ErrorKind::Interrupted=>continue,Err(_)=>return Err(()),
    }}
    serde_json::from_slice(&bytes).map(Some).map_err(|_|())
}
fn respond(output:&mut impl Write,h:&Header,status:Status)->Result<(),()>{
    let a=Ack{version:1,phase:h.phase,sequence:h.sequence,request_id:h.request_id.clone(),route_hash:h.route_hash.clone(),bytes:h.bytes,sha256:h.sha256.clone(),status};
    let mut bytes=serde_json::to_vec(&a).map_err(|_|())?;if bytes.len()>MAX_HEADER{return Err(());}bytes.push(b'\n');output.write_all(&bytes).and_then(|_|output.flush()).map_err(|_|())
}
fn same(h:&Header,control:&Header)->bool{h.version==control.version&&h.sequence==control.sequence&&h.request_id==control.request_id&&h.route_hash==control.route_hash&&h.deadline_ms==control.deadline_ms&&h.bytes==control.bytes&&h.sha256==control.sha256}
fn serve(mut input:impl Read,mut output:impl Write,route:&str,now:impl Fn()->Option<u64>,mut prepare:impl FnMut(&dyn Fn()->bool)->Status,mut put:impl FnMut(&str,&dyn Fn()->bool)->Status)->Result<(),()>{
    let mut expected=1;
    while let Some(h)=header(&mut input)?{
        if h.phase!=Phase::Prepare||!valid(&h,route,expected){return Err(());}
        let mut body=vec![0;h.bytes];input.read_exact(&mut body).map_err(|_|())?;
        if sha(&body)!=h.sha256{return Err(());}let text=String::from_utf8(body).map_err(|_|())?;
        let current=||now().is_some_and(|n|fresh(&h,n));
        let status=if !current(){Status::Retired}else{prepare(&current)};
        respond(&mut output,&h,status)?;
        if status==Status::Prepared{
            let c=header(&mut input)?.ok_or(())?;
            if !same(&h,&c)||c.phase==Phase::Prepare{return Err(());}
            let result=if c.phase==Phase::Abort{Status::Aborted}else if !current(){Status::Retired}else{put(&text,&current)};
            respond(&mut output,&c,result)?;
        }else if status!=Status::Retired&&status!=Status::Uncertain{return Err(());}
        expected=expected.checked_add(1).ok_or(())?;
    }Ok(())
}
fn prepare_with<T>(slot:&mut Option<T>,current:&dyn Fn()->bool,init:impl FnOnce()->Result<T,()>)->Status{
    if !current(){return Status::Retired;}
    if slot.is_none(){match init(){Ok(c)=>*slot=Some(c),Err(_)=>return Status::Uncertain}}
    if current(){Status::Prepared}else{Status::Retired}
}
fn put_with<T>(slot:&mut Option<T>,text:&str,current:&dyn Fn()->bool,init:impl FnOnce()->Result<T,()>,set:impl FnOnce(&mut T,&str)->Result<(),()>)->Status{
    if !current(){return Status::Retired;}
    if slot.is_none(){match init(){Ok(c)=>*slot=Some(c),Err(_)=>return Status::Uncertain}}
    if !current(){return Status::Retired;}
    match set(slot.as_mut().unwrap(),text){Ok(())if current()=>Status::Accepted,_=>Status::Uncertain}
}
fn process_fd(pid:libc::pid_t)->Result<std::os::fd::OwnedFd,()>{
    use std::os::fd::FromRawFd;
    let fd=unsafe{libc::syscall(libc::SYS_pidfd_open,pid,0u32)};
    if fd<0{return Err(());}Ok(unsafe{std::os::fd::OwnedFd::from_raw_fd(fd as i32)})
}
fn process_ended(fd:&std::os::fd::OwnedFd,timeout:i32)->Result<bool,()>{
    use std::os::fd::AsRawFd;
    let mut poll=libc::pollfd{fd:fd.as_raw_fd(),events:libc::POLLIN,revents:0};
    loop{let rc=unsafe{libc::poll(&mut poll,1,timeout)};if rc>=0{return Ok(rc>0);}if std::io::Error::last_os_error().kind()!=std::io::ErrorKind::Interrupted{return Err(());}}
}
fn watch_parent()->Result<std::thread::JoinHandle<()>,()>{
    let parent=unsafe{libc::getppid()};let me=unsafe{libc::getpid()};
    if parent<=1||unsafe{libc::getpgrp()}!=me||!authenticated_parent(){return Err(());}
    let fd=process_fd(parent)?;
    if unsafe{libc::getppid()}!=parent||!authenticated_parent(){return Err(());}
    // PIDFD_THREAD is deliberately absent: a Tokio worker may finish while
    // Coucou is still alive. This monitor belongs to the supervised child,
    // and only ever terminates its own dedicated process group.
    std::thread::Builder::new().name("clipboard-parent".into()).spawn(move||{
        loop{match process_ended(&fd,-1){Ok(false)=>continue,_=>{unsafe{libc::kill(-me,libc::SIGKILL);}std::process::exit(70);}}}
    }).map_err(|_|())
}
pub(crate)fn entry()->bool{
    let args:Vec<_>=std::env::args_os().collect();if args.get(1).is_none_or(|a|a!=FLAG){return false;}
    if args.len()!=2||!authenticated_parent(){std::process::exit(70);}
    let Ok(_parent_watch)=watch_parent()else{std::process::exit(65);};
    let Some(route)=Route::capture()else{std::process::exit(65);};let clipboard=std::cell::RefCell::new(None);
    let result=serve(std::io::stdin().lock(),std::io::stdout().lock(),&route.hash(),monotonic_ms,|current|{
        prepare_with(&mut clipboard.borrow_mut(),&||authenticated_parent()&&route.current()&&current(),||arboard::Clipboard::new().map_err(|_|()))
    },|text,current|{
        put_with(&mut clipboard.borrow_mut(),text,&||authenticated_parent()&&route.current()&&current(),||Err(()),|c,text|{
            use arboard::SetExtLinux;c.set().exclude_from_history().text(text.to_owned()).map_err(|_|())
        })
    });
    // Avoid a potentially blocking arboard Drop. The parent owns/reaps this
    // entire process group; closing the connection withdraws any owned data.
    std::process::exit(if result.is_ok(){0}else{65})
}
pub(crate)fn shutdown(app:&tauri::AppHandle){
    STOPPED.store(true,Ordering::Release);
    use tauri::Manager;
    if let Some(flow)=app.try_state::<std::sync::Arc<super::Flow>>(){super::capture::cancel_processing(&flow);}
    if let Ok(mut slot)=DRIVER.lock(){drop(slot.take());}
}
#[cfg(test)]#[path="clipboard_write_tests.rs"]mod tests;
