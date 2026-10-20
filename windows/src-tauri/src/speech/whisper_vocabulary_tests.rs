use super::*;
use serde_json::{json,Value};
fn header(count:usize,hints:&[&str])->Option<Value>{let terms=hints.iter().map(|s|s.to_string()).collect::<Vec<_>>();serde_json::from_slice(&request_header(count,&terms)?).ok()}
#[test]fn neutral_compatibility_header_has_no_hints(){assert_eq!(header(12,&[]),Some(json!({"samples":12})));assert_eq!(header(0,&[]),Some(json!({"samples":0})));}
#[test]fn utf8_and_ascii_dedup_remain_request_data(){assert_eq!(header(12,&[" Copilot ","copilot","Árvore","Árvore","Docker Compose"]),Some(json!({"samples":12,"vocabulary":["Copilot","Árvore","Docker Compose"]})));}
#[test]fn invalid_vocabulary_count_size_controls_and_pcm_length_reject(){
 for hints in[vec![""],vec![" "],vec!["x\ny"],vec!["x\0y"],vec!["x\u{7f}"]]{assert!(header(1,&hints).is_none());}
 assert!(request_header(1,&vec!["x".repeat(161)]).is_none());assert!(request_header(1,&vec!["á".repeat(81)]).is_none());assert!(request_header(1,&vec!["x".into();33]).is_none());assert!(header(960001,&[]).is_none());
 let many=(0..32).map(|n|format!("{n:02}{}","x".repeat(158))).collect::<Vec<_>>();assert!(request_header(1,&many).is_none());
}
#[test]fn worst_escaping_stays_bounded_and_utf8_valid(){
 let names=(0..25).map(|n|format!("{n:02}{}","\\\"".repeat(19))).collect::<Vec<_>>();let bytes=request_header(960000,&names).unwrap();assert!(bytes.len()<=MAX_HEADER_BYTES);assert!(std::str::from_utf8(&bytes).is_ok());assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap()["vocabulary"].as_array().unwrap().len(),25);
}
#[cfg(target_os="linux")]
struct Fixture {root:std::path::PathBuf}
#[cfg(target_os="linux")]
impl Fixture{
 fn new()->Self{static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);let root=std::env::temp_dir().join(format!("voice-vocabulary-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&root).unwrap();std::fs::write(root.join("model.bin"),b"fake fixture; not a model").unwrap();Self{root}}
 fn call(&self,hints:&[&str],helper:&str)->Option<super::super::Transcript>{let hints=hints.iter().map(|s|s.to_string()).collect::<Vec<_>>();transcribe_request(Path::new("/usr/bin/python3"),&self.root,&[0.25;4],None,&hints,helper)}
}
#[cfg(target_os="linux")]impl Drop for Fixture{fn drop(&mut self){release_if_idle();let _=std::fs::remove_dir_all(&self.root);}}
const ECHO:&str=r#"import json,sys,os
while True:
 line=sys.stdin.buffer.readline()
 if not line:break
 request=json.loads(line)
 request['workerPid']=os.getpid()
 remaining=request['samples']*4
 while remaining:
  data=sys.stdin.buffer.read(remaining)
  if not data:raise SystemExit(1)
  remaining-=len(data)
 print(json.dumps({'text':json.dumps(request,ensure_ascii=False),'device':'fake'},ensure_ascii=False),flush=True)
"#;
#[test]#[cfg(target_os="linux")]fn fake_worker_requests_do_not_keep_prior_vocabulary(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();
 let f=Fixture::new();let a=f.call(&["Cody","unit tests"],ECHO).unwrap();let b=f.call(&[],ECHO).unwrap();let c=f.call(&["Novo nome"],ECHO).unwrap();let a:Value=serde_json::from_str(&a.text).unwrap();let b:Value=serde_json::from_str(&b.text).unwrap();let c:Value=serde_json::from_str(&c.text).unwrap();assert_eq!(a["workerPid"],b["workerPid"]);assert_eq!(b["workerPid"],c["workerPid"]);assert_eq!(a["vocabulary"],json!(["Cody","unit tests"]));assert!(b.get("vocabulary").is_none());assert_eq!(c["vocabulary"],json!(["Novo nome"]));
}
#[test]#[cfg(target_os="linux")]fn malformed_input_never_starts_fake_child(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();
 let f=Fixture::new();let marker=f.root.join("should-not-run");let helper=format!("from pathlib import Path\nPath({:?}).write_text('started')",marker.to_str().unwrap());assert!(f.call(&["bad\nterm"],&helper).is_none());assert!(!marker.exists());
}
#[test]#[cfg(target_os="linux")]fn greeting_and_control_words_are_returned_without_post_rewrite_or_action(){
 let _serial=TEST_WORKER_SERIAL.lock().unwrap();
 let f=Fixture::new();let helper=ECHO.replace("json.dumps(request,ensure_ascii=False)","'Olá, podes ajudar-me? parar conversa'");let text=f.call(&["Ollama","Docker Compose"],&helper).unwrap().text;assert_eq!(text,"Olá, podes ajudar-me? parar conversa");assert!(f.root.read_dir().unwrap().all(|p|p.unwrap().file_name()=="model.bin"));
}
