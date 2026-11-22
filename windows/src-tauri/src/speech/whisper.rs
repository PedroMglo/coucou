//! Project-owned Whisper worker. Request-bound, bounded PCM; no transcript logs.
use std::io::{BufRead,BufReader,Read,Write};
use std::path::{Path,PathBuf};
use std::process::{ChildStdin,Command,Stdio};
use std::sync::{Mutex,OnceLock,mpsc,atomic::{AtomicBool,Ordering}};
use std::time::{Duration,Instant};
#[cfg(target_os="linux")]use std::os::{fd::AsRawFd,unix::process::CommandExt};
#[cfg(target_os="linux")]type NativeChild=crate::pet_process_group::OwnedChild;
#[cfg(not(target_os="linux"))]type NativeChild=std::process::Child;
const HELPER:&str=include_str!("whisper_helper.py");
const WIRE_VERSION:u64=1;
const MAX_RESPONSE_BYTES:u64=128*1024;
#[cfg(test)]static TEST_WORKER_SERIAL:Mutex<()>=Mutex::new(());
#[derive(PartialEq,Eq)]struct Key{python:PathBuf,model:PathBuf,helper:[u8;32]}
impl Key{fn new(python:&Path,model:&Path,helper:&str)->Self{let digest=ring::digest::digest(&ring::digest::SHA256,helper.as_bytes());Self{python:python.into(),model:model.into(),helper:digest.as_ref().try_into().unwrap()}}}
struct Worker{child:NativeChild,input:ChildStdin,output:Option<mpsc::Receiver<serde_json::Value>>,reader:Option<std::thread::JoinHandle<()>>,key:Key,next_id:u64}
impl Worker{
    fn exited(&mut self)->bool{
        #[cfg(target_os="linux")]{self.child.exited().unwrap_or(true)}
        #[cfg(not(target_os="linux"))]{self.child.try_wait().ok().flatten().is_some()}
    }
}
impl Drop for Worker{fn drop(&mut self){
    // Unblock a reader waiting to send into a full channel before joining it.
    drop(self.output.take());
    #[cfg(target_os="linux")]{
        while self.child.stop_and_reap().is_err(){std::thread::sleep(Duration::from_millis(10));}
        if let Some(reader)=self.reader.take(){let _=reader.join();}
    }
    #[cfg(not(target_os="linux"))]{
        // Preserve the legacy platform route; no Linux-family containment or
        // nonblocking-stdin guarantee is claimed for this branch.
        let _=self.child.kill();let _=self.child.wait();drop(self.reader.take());
    }
}}
// The existing cache mutex also owns cleanup on early return or unwind.
struct Lease<'a>{inner:std::sync::MutexGuard<'a,Option<Worker>>,reuse:bool}
impl std::ops::Deref for Lease<'_>{type Target=Option<Worker>;fn deref(&self)->&Self::Target{&self.inner}}
impl std::ops::DerefMut for Lease<'_>{fn deref_mut(&mut self)->&mut Self::Target{&mut self.inner}}
impl Drop for Lease<'_>{fn drop(&mut self){if !self.reuse{*self.inner=None;}}}
fn slot()->&'static Mutex<Option<Worker>>{static SLOT:OnceLock<Mutex<Option<Worker>>>=OnceLock::new();SLOT.get_or_init(||Mutex::new(None))}
fn start(python:&Path,model:&Path,key:Key,helper:&str,current:&dyn Fn()->bool)->Option<Worker>{
    if !current()||!python.is_file()||!model.join("model.bin").is_file(){return None;}
    let mut command=Command::new(python);
    #[cfg(target_os="linux")]{
        command.process_group(0);
        let site=python.parent()?.parent()?.join("lib/python3.11/site-packages/nvidia");
        let libraries=[site.join("cublas/lib"),site.join("cudnn/lib")];
        if libraries.iter().all(|p|p.is_dir()){
            let mut paths=libraries.to_vec();if let Some(old)=std::env::var_os("LD_LIBRARY_PATH"){paths.extend(std::env::split_paths(&old));}
            if let Ok(path)=std::env::join_paths(paths){command.env("LD_LIBRARY_PATH",path).env("COUCOU_WHISPER_CUDA","1");}
        }
    }
    command.args(["-I","-u","-c",helper]).arg(model).env("HF_HUB_OFFLINE","1").env("HF_HUB_DISABLE_IMPLICIT_TOKEN","1")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    if !current(){return None;}
    let child=command.spawn().ok()?;
    #[cfg(target_os="linux")]let mut child=NativeChild::new(child);
    #[cfg(not(target_os="linux"))]let mut child=child;
    #[cfg(target_os="linux")]let raw=&mut child.child;
    #[cfg(not(target_os="linux"))]let raw=&mut child;
    let input=raw.stdin.take()?;let out=raw.stdout.take()?;
    #[cfg(target_os="linux")]crate::pet_helper::nonblocking(input.as_raw_fd()).ok()?;
    let(tx,rx)=mpsc::sync_channel(2);
    let reader=std::thread::spawn(move||{
        let mut reader=BufReader::new(out);
        loop{
            let mut line=String::new();
            let Ok(n)=reader.by_ref().take(MAX_RESPONSE_BYTES+1).read_line(&mut line)else{break;};
            if n==0||n as u64>MAX_RESPONSE_BYTES||!line.ends_with('\n'){break;}
            let Ok(value)=serde_json::from_str(&line)else{break;};
            if tx.send(value).is_err(){break;}
        }
    });
    Some(Worker{child,input,output:Some(rx),reader:Some(reader),key,next_id:1})
}
pub const MAX_VOCABULARY_TERMS:usize=32;
pub const MAX_VOCABULARY_TERM_BYTES:usize=160;
pub const MAX_VOCABULARY_BYTES:usize=4096;
pub const MAX_HEADER_BYTES:usize=12288;
fn vocabulary_key(term:&str)->String{term.chars().map(|c|if c.is_ascii(){c.to_ascii_lowercase()}else{c}).collect()}
fn request_header(count:usize,vocabulary:&[String])->Option<Vec<u8>>{
    if count>16000*60||vocabulary.len()>MAX_VOCABULARY_TERMS{return None;}
    let mut terms=Vec::new();let mut seen=std::collections::BTreeSet::new();let mut total=0usize;
    for raw in vocabulary{
        let term=raw.trim();
        if term.is_empty()||term.len()>MAX_VOCABULARY_TERM_BYTES||raw.chars().any(char::is_control){return None;}
        let key=vocabulary_key(term);if !seen.insert(key){continue;}
        total=total.checked_add(term.len())?;if total>MAX_VOCABULARY_BYTES{return None;}
        terms.push(term);
    }
    let value=if terms.is_empty(){serde_json::json!({"samples":count})}else{serde_json::json!({"samples":count,"vocabulary":terms})};
    let mut bytes=serde_json::to_vec(&value).ok()?;bytes.push(b'\n');
    (bytes.len()<=MAX_HEADER_BYTES).then_some(bytes)
}
/// Compatibility API: neutral ASR hints; never installs a global prompt.
pub fn transcribe(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>)->Option<super::Transcript>{transcribe_with_vocabulary(python,model,samples,stop,&[])}
pub(crate)fn transcribe_with_vocabulary(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String])->Option<super::Transcript>{
    transcribe_with_current(python,model,samples,stop,vocabulary,&||true)
}
pub(crate)fn transcribe_with_current(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String],current:&dyn Fn()->bool)->Option<super::Transcript>{
    transcribe_request_guarded(python,model,samples,stop,vocabulary,HELPER,current)
}
#[cfg(test)]
fn transcribe_request(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String],helper:&str)->Option<super::Transcript>{
    transcribe_request_guarded(python,model,samples,stop,vocabulary,helper,&||true)
}
fn wire_header(neutral:&[u8],id:u64)->Option<Vec<u8>>{
    let mut value:serde_json::Value=serde_json::from_slice(neutral).ok()?;let object=value.as_object_mut()?;
    object.insert("protocolVersion".into(),WIRE_VERSION.into());object.insert("requestId".into(),id.into());
    let mut bytes=serde_json::to_vec(&value).ok()?;bytes.push(b'\n');(bytes.len()<=MAX_HEADER_BYTES).then_some(bytes)
}
fn write_request(worker:&mut Worker,header:&[u8],samples:&[f32],stopped:&dyn Fn()->bool)->Option<()>{
    let mut write=|bytes:&[u8]|->Option<()>{let mut at=0;while at<bytes.len(){
        if stopped(){return None;}
        // No reply is valid before the complete selected request was written.
        match worker.output.as_ref()?.try_recv(){Err(mpsc::TryRecvError::Empty)=>{},_=>return None}
        match worker.input.write(&bytes[at..]){
            Ok(0)=>return None,Ok(n)=>at+=n,
            Err(e)if e.kind()==std::io::ErrorKind::Interrupted=>{},
            Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>std::thread::sleep(Duration::from_millis(5)),
            Err(_)=>return None,
        }
    }Some(())};
    write(header)?;
    for chunk in samples.chunks(4096){let bytes=chunk.iter().flat_map(|s|s.to_le_bytes()).collect::<Vec<_>>();write(&bytes)?;}
    if stopped(){return None;}worker.input.flush().ok()
}
fn transcribe_request_guarded(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String],helper:&str,current:&dyn Fn()->bool)->Option<super::Transcript>{
    transcribe_request_until(python,model,samples,stop,vocabulary,helper,current,Duration::from_secs(60))
}
fn transcribe_request_until(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String],helper:&str,current:&dyn Fn()->bool,timeout:Duration)->Option<super::Transcript>{
    let timeout=timeout.min(Duration::from_secs(60));
    let header=request_header(samples.len(),vocabulary)?;
    let began=Instant::now();let stopped=||stop.is_some_and(|s|s.load(Ordering::Acquire))||began.elapsed()>timeout||!current();
    let guard=loop{
        if stopped(){return None;}
        match slot().try_lock(){Ok(guard)=>break guard,Err(std::sync::TryLockError::WouldBlock)=>std::thread::sleep(Duration::from_millis(5)),Err(_)=>return None}
    };
    let mut guard=Lease{inner:guard,reuse:false};
    if stopped(){return None;}
    let key=Key::new(python,model,helper);
    if guard.as_mut().is_some_and(|w|w.key!=key||w.exited()){*guard=None;}
    if stopped(){return None;}
    if guard.is_none(){*guard=start(python,model,key,helper,&||!stopped());}
    let result=(||{
        let worker=guard.as_mut()?;if stopped(){return None;}
        let id=worker.next_id;worker.next_id=id.checked_add(1)?;let header=wire_header(&header,id)?;
        write_request(worker,&header,samples,&stopped)?;
        loop{
            if stopped(){return None;}
            match worker.output.as_ref()?.recv_timeout(Duration::from_millis(10)){
                Ok(value)=>{
                    if value["protocolVersion"].as_u64()!=Some(WIRE_VERSION)||value["requestId"].as_u64()!=Some(id)||stopped(){return None;}
                    let text=value["text"].as_str()?.to_owned();
                    let device=match value["device"].as_str(){Some("cpu")=>"cpu",Some("cuda")=>"cuda",_=>"unknown"};crate::log::line(format!("voice: Whisper backend {device}"));
                    return Some(super::Transcript{text,words:Vec::new()});
                },Err(mpsc::RecvTimeoutError::Timeout)=>{},Err(_)=>return None,
            }
        }
    })();
    // Every failure after possible dispatch is terminal for this attempt. The
    // same physical slot remains held through family/reader cleanup; no retry.
    let result=if stopped(){None}else{result};
    if result.is_none(){*guard=None;}else{guard.reuse=true;}
    crate::log::line(format!("voice: local Whisper transcription completed={} in {} ms",result.is_some(),began.elapsed().as_millis()));result
}
/// Never block microphone controls behind an ASR request. A busy worker observes
/// its cancelled voice token; otherwise its existing 120 s idle timeout applies.
pub fn release_if_idle() {
    if let Ok(mut worker)=slot().try_lock() {*worker=None;}
}

#[cfg(test)]
mod tests {
    #[test]
    fn microphone_cleanup_never_waits_for_a_busy_asr_worker() {
        let _serial=super::TEST_WORKER_SERIAL.lock().unwrap();
        let worker=super::slot().lock().unwrap();let (tx,rx)=std::sync::mpsc::channel();
        let release=std::thread::spawn(move||{super::release_if_idle();tx.send(()).unwrap();});
        let ready=rx.recv_timeout(std::time::Duration::from_secs(1)).is_ok();
        drop(worker);release.join().unwrap();assert!(ready);
    }
}

#[cfg(test)]
#[path="whisper_vocabulary_tests.rs"]
mod vocabulary_tests;

#[cfg(all(test,target_os="linux"))]
#[path="whisper_io_tests.rs"]mod io_tests;
