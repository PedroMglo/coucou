use super::*;
use std::io::Cursor;

#[test]
fn permission_choices_never_promote_allow_once_to_always_or_interpret_labels_as_grants() {
    let p = json!({"options":[{"kind":"allow_always","optionId":"always","name":"Allow once"},{"kind":"allow_once","optionId":"opaque-once"},{"kind":"reject_once","optionId":"opaque-deny"}]});
    assert_eq!(permission_outcome(&p, Some(true)), json!({"outcome":{"outcome":"selected","optionId":"opaque-once"}}));
    assert_eq!(permission_outcome(&p, Some(false)), json!({"outcome":{"outcome":"selected","optionId":"opaque-deny"}}));
    assert_eq!(permission_outcome(&p, None), cancelled_outcome());
    assert_eq!(permission_outcome(&json!({"options":[{"kind":"allow_always","optionId":"always"}]}), Some(true)), cancelled_outcome());
    assert_eq!(permission_outcome(&json!({"options":[{"kind":"allow_once","optionId":"bad\nid"}]}), Some(true)), cancelled_outcome());
}

#[test]
fn catalog_uses_native_ids_deduplicates_and_excludes_disabled_models() {
    let c = Catalog::from_result(&json!({"models":{"currentModelId":"m","availableModels":[{"modelId":"m","name":"Model"},{"modelId":"m","name":"Duplicate"},{"modelId":"secret","name":"Disabled","_meta":{"copilotEnablement":"disabled"}}]}}));
    assert_eq!(c.models, vec![Model{id:"m".into(),label:"Model".into()}]);
    let c = Catalog::from_result(&json!({"configOptions":[{"id":"model","category":"model","currentValue":"nested","options":[{"group":"group","options":[{"value":"nested","name":"Nested"}]}]}]}));
    assert_eq!(c.models[0].id,"nested");assert_eq!(c.current_model,"nested");
}

#[test]
fn invalid_oversized_or_non_protocol_lines_are_rejected_without_echoing_payloads() {
    for bytes in [b"secret malformed\n".to_vec(), b"{\"id\":1}\n".to_vec(), vec![b'x';MAX_LINE+2]] {
        let error=read_message(&mut Cursor::new(bytes),&mut 0).unwrap_err();assert!(!error.contains("secret"));
    }
    let mut total=MAX_TOTAL;assert!(read_message(&mut Cursor::new(b"{\"jsonrpc\":\"2.0\"}\n"),&mut total).is_err());
}

#[test]
fn replay_includes_only_bounded_public_messages_and_never_becomes_a_slash_command() {
    let history=vec![json!({"role":"tool","content":"private tool output"}),json!({"role":"assistant","content":[{"type":"thinking","text":"private reasoning"}]}),json!({"role":"user","content":"earlier question"})];
    let mut options=TurnOptions{model:"",effort:"",system:"Follow AGENTS.md",history:&history,query:"/permissions allow-all",session_id:None};
    let text=prompt_text(&options);assert!(!text.starts_with('/'));assert!(!text.contains("private tool output"));assert!(!text.contains("private reasoning"));assert!(text.contains("earlier question"));
    options.session_id=Some("owned");assert!(!prompt_text(&options).contains("earlier question"));
}

#[cfg(unix)]
mod process {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    static SPAWN:std::sync::Mutex<()>=std::sync::Mutex::new(());

    const PEER:&str=r#"#!/usr/bin/env python3
import json,sys,time,os,subprocess
from pathlib import Path
root=Path.cwd()
behavior=(root/'behavior').read_text()
configs=[{'id':'mode','category':'mode','type':'select','currentValue':'agent','options':[{'value':'agent'},{'value':'autopilot'}]},
 {'id':'model','category':'model','type':'select','currentValue':'model-a','options':[{'value':'model-a','name':'Model A'},{'value':'model-b','name':'Model B'}]},
 {'id':'reasoning_effort','category':'thought_level','type':'select','currentValue':'low','options':[{'value':'low'},{'value':'high'}]},
 {'id':'allow_all','category':'permissions','type':'select','currentValue':'off','options':[{'value':'on'},{'value':'off'}]}]
if behavior=='unsafe-mode':configs[0]['currentValue']='autopilot';configs[3]['currentValue']='on'
def out(v):print(json.dumps({'jsonrpc':'2.0',**v}),flush=True)
def update(kind,**values):out({'method':'session/update','params':{'sessionId':'owned-session','update':{'sessionUpdate':kind,**values}}})
def snapshot():return {'configOptions':configs,'models':{'currentModelId':'model-a','availableModels':[{'modelId':'model-a','name':'Model A'},{'modelId':'model-b','name':'Model B'}]}}
prompt=None
for line in sys.stdin:
 m=json.loads(line)
 with (root/'wire.jsonl').open('a') as f:f.write(json.dumps(m)+'\n')
 method=m.get('method')
 if method=='initialize':
  if behavior=='stall-init':time.sleep(20)
  out({'id':m['id'],'result':{'protocolVersion':1,'agentCapabilities':{'loadSession':behavior!='no-load'}}})
 elif method in ['session/new','session/load']:
  if method=='session/load':update('agent_message_chunk',content={'type':'text','text':'old replay must not be published'})
  out({'id':m['id'],'result':{'sessionId':'owned-session',**snapshot()}})
 elif method=='session/set_config_option':
  p=m['params']
  for c in configs:
   if c['id']==p['configId'] and behavior!='reject-config':c['currentValue']=p['value']
  out({'id':m['id'],'result':snapshot()})
  if behavior=='stall-write' and p['configId']=='reasoning_effort':time.sleep(20)
 elif method=='session/prompt':
  prompt=m['id']
  if behavior=='stall-prompt':continue
  if behavior=='orphan':
   child=subprocess.Popen([sys.executable,'-c','import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);time.sleep(30)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   (root/'orphan.pid').write_text(str(child.pid));time.sleep(.04)
   update('agent_message_chunk',content={'type':'text','text':'owned child started'})
   out({'id':prompt,'result':{'stopReason':'end_turn'}});continue
  if behavior=='fallback':
   for c in configs:
    if c['id']=='model':c['currentValue']='model-b' if c['currentValue']=='model-a' else 'model-a'
   update('config_option_update',configOptions=configs)
  out({'method':'session/update','params':{'sessionId':'foreign-session','update':{'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':'foreign private message'}}}})
  update('agent_thought_chunk',content={'type':'text','text':'private reasoning'})
  update('tool_call',toolCallId='t',status='pending',rawInput={'command':'private command arguments'})
  out({'id':'foreign-permission','method':'session/request_permission','params':{'sessionId':'foreign-session','options':[{'kind':'allow_once','optionId':'wrong-session'}]}})
 elif m.get('id')=='foreign-permission':
  out({'id':'own-permission','method':'session/request_permission','params':{'sessionId':'owned-session','toolCall':{'toolCallId':'t','title':'Run synthetic tool','rawInput':{'command':'synthetic command'}},'options':[{'kind':'allow_always','optionId':'always'},{'kind':'allow_once','optionId':'once'},{'kind':'reject_once','optionId':'deny'}]}})
 elif m.get('id')=='own-permission':
  update('tool_call_update',toolCallId='t',status='completed',rawOutput={'reasoning':'private tool result'})
  update('agent_message_chunk',content={'type':'text','text':'Olá '})
  update('agent_message_chunk',content={'type':'text','text':'equipa'})
  out({'id':prompt,'result':{'stopReason':'end_turn'}})
 elif method=='session/cancel':break
 elif method:out({'id':m.get('id'),'error':{'code':-32601,'message':'unknown'}})
"#;

    struct Fixture {dir:PathBuf,exe:PathBuf}
    impl Fixture {
        fn new(behavior:&str)->Self {
            let _spawn=SPAWN.lock().unwrap();
            static NEXT:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
            let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let dir=std::env::temp_dir().join(format!("mochi-acp-test-{}-{nonce}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            fs::create_dir_all(&dir).unwrap();fs::set_permissions(&dir,fs::Permissions::from_mode(0o700)).unwrap();
            let exe=dir.join("peer.py");fs::write(&exe,PEER).unwrap();fs::set_permissions(&exe,fs::Permissions::from_mode(0o700)).unwrap();fs::write(dir.join("behavior"),behavior).unwrap();Self{dir,exe}
        }
        fn client(&self)->Client {let _spawn=SPAWN.lock().unwrap();Client::start(&self.exe,&self.dir,Duration::from_secs(5)).unwrap()}
        fn wire(&self)->Vec<Value>{fs::read_to_string(self.dir.join("wire.jsonl")).unwrap().lines().map(|s|serde_json::from_str(s).unwrap()).collect()}
    }
    impl Drop for Fixture {fn drop(&mut self){let _=fs::remove_dir_all(&self.dir);}}
    fn options(session:Option<&str>)->TurnOptions<'_>{TurnOptions{model:"model-b",effort:"high",system:"Follow AGENTS.md and native permissions",history:&[],query:"Synthetic task",session_id:session}}

    #[test]
    fn native_turn_selects_the_advertised_model_and_once_permission_without_exposing_other_channels() {
        let f=Fixture::new("normal");let mut c=f.client();let mut sessions=vec![];let mut deltas=vec![];let mut phases=vec![];let mut permissions=0;
        let reply=c.answer(options(None),|s|sessions.push(s.to_string()),|s|deltas.push(s),|p|phases.push(p),|p|{assert_eq!(p["sessionId"],"owned-session");permissions+=1;Some(true)}).unwrap();
        assert_eq!(reply.text,"Olá equipa");assert_eq!(reply.model,"model-b");assert!(reply.acted);assert_eq!(reply.session_id,"owned-session");assert_eq!(reply.stop_reason,"end_turn");
        assert_eq!(sessions,vec!["owned-session"]);assert_eq!(permissions,1);assert_eq!(deltas.last().unwrap(),"Olá equipa");assert!(phases.contains(&Phase::Approval));
        let wire=f.wire();assert!(wire.iter().any(|v|v["id"]=="own-permission" && v["result"]["outcome"]["optionId"]=="once"));
        assert!(wire.iter().any(|v|v["id"]=="foreign-permission" && v["result"]==cancelled_outcome()));
        assert!(!format!("{deltas:?}").contains("private"));
        assert!(!wire.iter().any(|v|v["params"]["value"]=="on" || v["params"]["value"]=="autopilot"));
    }

    #[test]
    fn saved_session_is_loaded_without_republishing_history_and_unsupported_load_never_creates_a_replacement() {
        let f=Fixture::new("normal");let mut c=f.client();let mut deltas=vec![];
        c.answer(options(Some("owned-session")),|_|{},|s|deltas.push(s),|_|{},|_|Some(false)).unwrap();
        assert!(!deltas.iter().any(|s|s.contains("replay")));assert!(!f.wire().iter().any(|v|v["method"]=="session/new"));
        let g=Fixture::new("no-load");let mut c=g.client();
        assert!(c.answer(options(Some("owned-session")),|_|{},|_|{},|_|{},|_|None).is_err());
        assert!(!g.wire().iter().any(|v|v["method"]=="session/new" || v["method"]=="session/prompt"));
    }

    #[test]
    fn unsafe_initial_mode_is_reset_and_unconfirmed_configuration_prevents_inference() {
        let f=Fixture::new("unsafe-mode");let mut c=f.client();assert_eq!(c.models().unwrap().len(),2);
        let wire=f.wire();assert!(wire.iter().any(|v|v["params"]["configId"]=="allow_all" && v["params"]["value"]=="off"));
        assert!(wire.iter().any(|v|v["params"]["configId"]=="mode" && v["params"]["value"]=="agent"));
        let g=Fixture::new("reject-config");let mut c=g.client();assert!(c.answer(options(None),|_|{},|_|{},|_|{},|_|None).is_err());
        assert!(!g.wire().iter().any(|v|v["method"]=="session/prompt"));
    }

    #[test]
    fn cancellation_targets_only_the_owned_session_and_reaps_the_child() {
        let f=Fixture::new("stall-prompt");let mut c=f.client();let cancel=Arc::new(AtomicBool::new(false));c.cancel_with(cancel.clone());
        let flag=cancel.clone();let t=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(100));flag.store(true,Ordering::Relaxed);});
        let start=Instant::now();assert_eq!(c.answer(options(None),|_|{},|_|{},|_|{},|_|None).unwrap_err(),CANCELLED);t.join().unwrap();
        let child=c.child.id();drop(c);assert!(start.elapsed()<Duration::from_secs(2));
        assert!(f.wire().iter().any(|v|v["method"]=="session/cancel" && v["params"]["sessionId"]=="owned-session"));
        assert_ne!(unsafe{libc::kill(child as i32,0)},0);
    }

    #[test]
    fn startup_timeout_reaps_a_stalled_child_and_launch_never_contains_permission_bypass_switches() {
        let f=Fixture::new("stall-init");let start=Instant::now();let _spawn=SPAWN.lock().unwrap();assert!(Client::start(&f.exe,&f.dir,Duration::from_millis(70)).is_err());assert!(start.elapsed()<Duration::from_secs(2));
        let cmd=command(&f.exe,&f.dir);let args=cmd.get_args().map(|v|v.to_string_lossy().to_string()).collect::<Vec<_>>();
        assert!(args.contains(&"--acp".into()));assert!(args.contains(&"--no-auto-update".into()));
        assert!(!args.iter().any(|v|matches!(v.as_str(),"--allow-all"|"--allow-all-tools"|"--yolo"|"--no-custom-instructions"|"--no-ask-user")));
    }

    #[test]
    fn blocked_stdin_obeys_cancellation_and_a_finished_client_never_reuses_its_old_session() {
        let f=Fixture::new("stall-write");let mut c=f.client();let cancel=Arc::new(AtomicBool::new(false));c.cancel_with(cancel.clone());
        let t=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(120));cancel.store(true,Ordering::Relaxed);});
        let text="x".repeat(128*1024);let mut opts=options(None);opts.query=&text;
        let start=Instant::now();assert_eq!(c.answer(opts,|_|{},|_|{},|_|{},|_|None).unwrap_err(),CANCELLED);t.join().unwrap();
        c.cancel_with(Arc::default());assert!(c.answer(options(None),|_|{},|_|{},|_|{},|_|Some(true)).is_err());
        drop(c);assert!(start.elapsed()<Duration::from_secs(2));
    }

    #[test]
    fn cancellation_during_approval_returns_the_cancelled_outcome_before_cancelling_the_owned_turn() {
        let f=Fixture::new("normal");let mut c=f.client();let cancel=Arc::new(AtomicBool::new(false));c.cancel_with(cancel.clone());
        let result=c.answer(options(None),|_|{},|_|{},|_|{},|_|{cancel.store(true,Ordering::Relaxed);Some(true)});
        assert_eq!(result.unwrap_err(),CANCELLED);drop(c);
        let wire=f.wire();assert!(wire.iter().any(|v|v["id"]=="own-permission" && v["result"]==cancelled_outcome()));
        assert!(wire.iter().any(|v|v["method"]=="session/cancel" && v["params"]["sessionId"]=="owned-session"));
    }

    #[test]
    fn final_selection_is_verified_and_agent_fallback_is_reported_instead_of_claiming_the_old_model() {
        let f=Fixture::new("fallback");let mut c=f.client();assert!(c.answer(options(None),|_|{},|_|{},|_|{},|_|Some(true)).is_err());
        assert!(c.answer(options(None),|_|{},|_|{},|_|{},|_|None).is_err());drop(c);
        assert_eq!(f.wire().iter().filter(|v|v["method"]=="session/prompt").count(),1);
        let g=Fixture::new("fallback");let mut c=g.client();let mut opts=options(None);opts.model="";opts.effort="";
        let reply=c.answer(opts,|_|{},|_|{},|_|{},|_|Some(false)).unwrap();assert_eq!(reply.model,"model-b");assert_eq!(reply.effort,"low");
    }

    #[test]
    fn exiting_the_leader_does_not_leave_a_tool_descendant_running() {
        let f=Fixture::new("orphan");let mut c=f.client();c.answer(options(None),|_|{},|_|{},|_|{},|_|None).unwrap();
        let pid=fs::read_to_string(f.dir.join("orphan.pid")).unwrap();drop(c);
        let deadline=Instant::now()+Duration::from_millis(200);
        loop {
            let status=fs::read_to_string(format!("/proc/{pid}/stat"));
            if status.as_ref().map_or(true,|s|s.split_once(") ").is_some_and(|(_,fields)|fields.starts_with('Z'))){break;}
            assert!(Instant::now()<deadline,"owned tool descendant still running");std::thread::sleep(Duration::from_millis(10));
        }
    }
}
