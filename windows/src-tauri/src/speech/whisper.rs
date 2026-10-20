//! Project-owned Whisper worker. PCM lives in memory; the process expires idle.
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, OnceLock, mpsc, atomic::{AtomicBool,Ordering}};
use std::time::{Duration, Instant};

const HELPER: &str=include_str!("whisper_helper.py");
#[cfg(test)]static TEST_WORKER_SERIAL:Mutex<()>=Mutex::new(());
struct Worker { child: Child, input: ChildStdin, output: mpsc::Receiver<serde_json::Value>, key: String }
impl Drop for Worker { fn drop(&mut self) { let _=self.child.kill(); let _=self.child.wait(); } }
fn slot() -> &'static Mutex<Option<Worker>> { static SLOT: OnceLock<Mutex<Option<Worker>>>=OnceLock::new(); SLOT.get_or_init(||Mutex::new(None)) }
fn start(python: &Path, model: &Path, key: String, helper:&str) -> Option<Worker> {
    if !python.is_file() || !model.join("model.bin").is_file() { return None; }
    let mut command=Command::new(python);
    #[cfg(target_os = "linux")] {
        let site=python.parent()?.parent()?.join("lib/python3.11/site-packages/nvidia");
        let libraries=[site.join("cublas/lib"),site.join("cudnn/lib")];
        if libraries.iter().all(|p|p.is_dir()) {
            let mut paths=libraries.to_vec();
            if let Some(old)=std::env::var_os("LD_LIBRARY_PATH") { paths.extend(std::env::split_paths(&old)); }
            if let Ok(path)=std::env::join_paths(paths) { command.env("LD_LIBRARY_PATH",path).env("COUCOU_WHISPER_CUDA","1"); }
        }
    }
    let mut child=command.args(["-I","-u","-c",helper]).arg(model)
        .env("HF_HUB_OFFLINE","1").env("HF_HUB_DISABLE_IMPLICIT_TOKEN","1")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let input=child.stdin.take()?; let out=child.stdout.take()?;
    let (tx,rx)=mpsc::sync_channel(2);
    std::thread::spawn(move || {
        let mut reader=BufReader::new(out);
        loop {
            let mut line=String::new();
            use std::io::Read;
            if reader.by_ref().take(128*1024).read_line(&mut line).ok().filter(|n|*n>0).is_none() { break; }
            let Ok(value)=serde_json::from_str(&line) else { break; };
            if tx.send(value).is_err() { break; }
        }
    });
    Some(Worker { child,input,output:rx,key })
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
/// Compatibility API: neutral ASR, no vocabulary or prompt is installed.
pub fn transcribe(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>)->Option<super::Transcript>{
    transcribe_with_vocabulary(python,model,samples,stop,&[])
}
/// Hints affect this request only; never normalize or rewrite recognized text.
pub(crate) fn transcribe_with_vocabulary(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String])->Option<super::Transcript>{
    transcribe_request(python,model,samples,stop,vocabulary,HELPER)
}
fn transcribe_request(python:&Path,model:&Path,samples:&[f32],stop:Option<&AtomicBool>,vocabulary:&[String],helper:&str)->Option<super::Transcript>{
    // Reject before touching the process/cache, even when a worker already exists.
    let header=request_header(samples.len(),vocabulary)?;
    if samples.len()>16000*60 { return None; }
    let stopped=||stop.is_some_and(|s|s.load(Ordering::Relaxed));
    let began=Instant::now();
    let mut guard=loop {
        if stopped() { return None; }
        if let Ok(guard)=slot().try_lock() { break guard; }
        if began.elapsed()>Duration::from_secs(60) { return None; }
        std::thread::sleep(Duration::from_millis(30));
    };
    let key=format!("{}:{}",python.display(),model.display());
    if guard.as_mut().is_some_and(|w|w.key!=key || w.child.try_wait().ok().flatten().is_some()) { *guard=None; }
    if guard.is_none() { *guard=start(python,model,key,helper); }
    let worker=guard.as_mut()?;
    let result=(|| {
        worker.input.write_all(&header).ok()?;
        for chunk in samples.chunks(4096) {
            let bytes=chunk.iter().flat_map(|s|s.to_le_bytes()).collect::<Vec<_>>();
            worker.input.write_all(&bytes).ok()?;
        }
        worker.input.flush().ok()?;
        loop {
            if stopped() || began.elapsed()>Duration::from_secs(60) { return None; }
            match worker.output.recv_timeout(Duration::from_millis(60)) {
                Ok(value)=>{ crate::log::line(format!("voice: Whisper backend {}",value["device"].as_str().unwrap_or("cpu"))); return Some(super::Transcript { text:value["text"].as_str()?.to_string(), words:Vec::new() }); },
                Err(mpsc::RecvTimeoutError::Timeout)=>{}, Err(_)=>return None,
            }
        }
    })();
    if result.is_none() { *guard=None; }
    crate::log::line(format!("voice: local Whisper multilingual transcription completed={} in {} ms",result.is_some(),began.elapsed().as_millis()));
    result
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
