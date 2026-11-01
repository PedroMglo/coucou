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
#[test]fn native_request_cancellation_is_exact_typed_endpoint_bound_and_never_a_policy_grant(){
    let endpoint=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let requests=RequestBook::new(endpoint.clone());
    let capture=|id:&Value|requests.register(Capture{request_id:Some(id),session_id:"native-session",turn_id:None,prompt_id:Some(&json!(9)),method:"session/request_permission",arguments:&json!({}),policy:PolicyBinding::unknown(),cancel:None}).unwrap();let numeric=capture(&json!(1));let text=capture(&json!("1"));
    observe_request_cancellation(&requests,&json!({"method":"$/cancel_request","params":{"requestId":1}}));assert!(!numeric.current());assert!(numeric.withdrawal_pending());assert!(!numeric.policy_invalidated());assert!(text.current());
    observe_request_cancellation(&requests,&json!({"id":99,"method":"$/cancel_request","params":{"requestId":"1"}}));assert!(text.current());requests.finish(&numeric);assert!(!numeric.withdrawal_pending());assert!(text.current());endpoint.close();assert!(!text.withdrawal_pending());
}

#[test]fn catalog_observations_follow_receive_order_without_claiming_effective_permission_rules(){
    let source=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let mut state=NativeMonitor::default();state.config_requests.insert(1,ConfigRequest::New);
    state.observe(&json!({"id":1,"result":{"sessionId":"native-session","configOptions":[{"id":"model","category":"model","currentValue":"first","options":[{"value":"first"}]}]}}),&source).unwrap();
    state.config_requests.insert(2,ConfigRequest::Existing("native-session".into()));let response=json!({"id":2,"result":{"configOptions":[{"id":"model","category":"model","currentValue":"response-model","options":[{"value":"response-model"}]}]}});state.observe(&response,&source).unwrap();
    state.observe(&json!({"method":"session/update","params":{"sessionId":"native-session","update":{"sessionUpdate":"config_option_update","configOptions":[{"id":"model","category":"model","currentValue":"later-model","options":[{"value":"later-model"}]}]}}}),&source).unwrap();assert_eq!(state.catalog.as_ref().unwrap().current_model,"later-model");assert_eq!(Catalog::from_result(&response["result"]).current_model,"response-model");let seq=state.config_sequence;
    state.observe(&json!({"method":"session/update","params":{"sessionId":"foreign-session","update":{"sessionUpdate":"config_option_update","configOptions":[]}}}),&source).unwrap();assert_eq!(state.config_sequence,seq);assert_eq!(state.catalog.as_ref().unwrap().current_model,"later-model");
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
if behavior=='native-policy':configs[0]['currentValue']='autopilot';configs[3]['currentValue']='on'
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
  result=snapshot()
  if behavior!='load-no-echo' or method!='session/load':result['sessionId']='foreign-session' if behavior=='load-wrong-echo' and method=='session/load' else 'owned-session'
  out({'id':m['id'],'result':result})
 elif method=='session/set_config_option':
  p=m['params']
  for c in configs:
   if c['id']==p['configId'] and behavior!='reject-config':c['currentValue']=p['value']
  out({'id':m['id'],'result':snapshot()})
  if behavior=='ordered-config' and p['configId']=='model':
   for c in configs:
    if c['id']=='model':c['currentValue']='model-a'
   update('config_option_update',configOptions=configs)
  if behavior=='stall-write' and p['configId']=='reasoning_effort':time.sleep(20)
 elif method=='session/prompt':
  prompt=m['id']
  if behavior=='stall-prompt':continue
  if behavior=='orphan':
   child=subprocess.Popen([sys.executable,'-c','import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);time.sleep(30)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   (root/'orphan.pid').write_text(str(child.pid));time.sleep(.04)
   update('agent_message_chunk',content={'type':'text','text':'owned child started'})
   out({'id':prompt,'result':{'stopReason':'end_turn'}});continue
  if behavior.startswith('source-'):
   out({'id':'own-permission','method':'session/request_permission','params':{'sessionId':'owned-session','toolCall':{'toolCallId':'t','title':'Run synthetic tool'},'options':[{'kind':'allow_once','optionId':'once'},{'kind':'reject_once','optionId':'deny'}]}})
   until=time.monotonic()+3
   while not (root/'review-open').exists() and time.monotonic()<until:time.sleep(.005)
   if not (root/'review-open').exists():sys.exit(8)
   for i in range(48):update('agent_thought_chunk',content={'type':'text','text':'private reasoning'})
   if behavior in ['source-complete','source-complete-model']:
    if behavior=='source-complete-model':
     for c in configs:
      if c['id']=='model':c['currentValue']='model-a'
     update('config_option_update',configOptions=configs)
    update('agent_message_chunk',content={'type':'text','text':'Native prompt already finished'})
    out({'id':prompt,'result':{'stopReason':'end_turn'}})
   elif behavior=='source-eof':sys.exit(0)
   elif behavior in ['source-config','source-model']:
    for c in configs:
     if c['id']==('model' if behavior=='source-model' else 'allow_all'):c['currentValue']='model-a' if behavior=='source-model' else 'on'
    update('config_option_update',configOptions=configs)
   elif behavior=='source-request-cancel':
    for i in range(2):out({'method':'$/cancel_request','params':{'requestId':'own-permission'}})
   elif behavior=='source-duplicate':
    out({'id':'own-permission','method':'session/request_permission','params':{'sessionId':'owned-session','options':[{'kind':'allow_once','optionId':'different'}]}})
   continue
  if behavior=='fallback':
   for c in configs:
    if c['id']=='model':c['currentValue']='model-b' if c['currentValue']=='model-a' else 'model-a'
   update('config_option_update',configOptions=configs)
  out({'method':'session/update','params':{'sessionId':'foreign-session','update':{'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':'foreign private message'}}}})
  update('agent_thought_chunk',content={'type':'text','text':'private reasoning'})
  update('tool_call',toolCallId='t',status='pending',rawInput={'command':'private command arguments'})
  out({'id':'foreign-permission','method':'session/request_permission','params':{'sessionId':'foreign-session','options':[{'kind':'allow_once','optionId':'wrong-session'}]}})
 elif m.get('id')=='foreign-permission':
  permission_options=[{'kind':'allow_always','optionId':'always'},{'kind':'allow_once','optionId':'once'},{'kind':'reject_once','optionId':'deny'}]
  if behavior=='remembered-only':permission_options=[{'kind':'allow_always','optionId':'opaque-native-remember','name':'Allow once'},{'kind':'reject_always','optionId':'opaque-native-reject'}]
  out({'id':'own-permission','method':'session/request_permission','params':{'sessionId':'owned-session','toolCall':{'toolCallId':'t','title':'Run synthetic tool','rawInput':{'command':'synthetic command'}},'options':permission_options}})
 elif m.get('id')=='own-permission':
  if behavior=='cancel-await':continue
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

    #[cfg(target_os="linux")]
    #[test]fn final_dispatch_copilot_rechecks_after_human_callback_without_blocking_exact_denial(){
        for approve in [true,false]{let io=crate::settings::EffectFixture::new();let f=Fixture::new("normal");let mut client=f.client();client.io_probe=Some(io.probe.clone());
            let reply=client.answer_native(options(None),|_|{},|_|{},|_|{},|_,_|{io.external();Some(json!({"allow":approve}))}).unwrap();assert_eq!(reply.text,"Olá equipa");drop(client);
            let wire=f.wire();let responses=wire.iter().filter(|v|v["id"]=="own-permission"&&v.get("method").is_none()).collect::<Vec<_>>();assert_eq!(responses.len(),1);
            assert_eq!(responses[0]["result"],if approve{cancelled_outcome()}else{json!({"outcome":{"outcome":"selected","optionId":"deny"}})});
        }
    }
    #[cfg(target_os="linux")]
    #[test]fn final_dispatch_copilot_benign_commit_keeps_original_positive_choice(){
        let io=crate::settings::EffectFixture::new();let f=Fixture::new("normal");let mut client=f.client();client.io_probe=Some(io.probe.clone());
        client.answer_native(options(None),|_|{},|_|{},|_|{},|_,_|{io.save();Some(json!({"allow":true}))}).unwrap();drop(client);
        let wire=f.wire();let responses=wire.iter().filter(|v|v["id"]=="own-permission"&&v.get("method").is_none()).collect::<Vec<_>>();assert_eq!(responses.len(),1);assert_eq!(responses[0]["result"],json!({"outcome":{"outcome":"selected","optionId":"once"}}));
    }

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
    fn native_review_carries_original_acp_request_and_prompt_without_inventing_policy_or_turn(){
        let f=Fixture::new("normal");let mut c=f.client();let mut captured=None;
        let reply=c.answer_native(options(None),|_|{},|_|{},|_|{},|params,ticket|{
            assert!(ticket.current());let identity=ticket.identity();
            assert_eq!(identity.adapter,Adapter::Copilot.key());assert_eq!(identity.request_id,json!("own-permission"));
            assert_eq!(identity.session_id,"owned-session");assert_eq!(identity.method,"session/request_permission");
            assert!(identity.turn_id.is_none());assert_eq!(identity.correlation,"nativeRequest");
            assert_eq!(identity.native_policy.coverage,"notObserved");assert!(identity.native_policy.hash.is_none());
            let hash=ring::digest::digest(&ring::digest::SHA256,&serde_json::to_vec(params).unwrap()).as_ref().iter().map(|v|format!("{v:02x}")).collect::<String>();
            assert_eq!(identity.arguments_hash,hash);assert_eq!(identity.endpoint.len(),64);
            captured=Some(ticket.clone());Some(json!({"allow":true}))
        }).unwrap();assert_eq!(reply.text,"Olá equipa");
        let ticket=captured.unwrap();assert!(!ticket.current());
        let prompt=f.wire().into_iter().find(|v|v["method"]=="session/prompt").unwrap();assert_eq!(ticket.identity().prompt_id.as_ref(),Some(&prompt["id"]));
    }

    #[test]fn actual_acp_remembered_only_options_require_an_explicit_native_choice_and_echo_the_original_id(){
        for allow in[true,false]{let f=Fixture::new("remembered-only");let mut c=f.client();let mut calls=0;
            c.answer_native(options(None),|_|{},|_|{},|_|{},|params,source|{
                calls+=1;let choices=crate::pet_native_choices::Choices::capture(source,"review-acp",params).unwrap().unwrap();assert!(choices.resolve(&json!({"allow":allow})).is_err());let public=choices.public();let id=public.as_array().unwrap().iter().find(|v|v["allow"]==allow).unwrap()["id"].clone();let mut human=json!({"allow":allow,"nativeChoice":id});human["nativeResponse"]=choices.resolve(&human).unwrap();Some(human)
            }).unwrap();assert_eq!(calls,1);drop(c);let wire=f.wire();let answer=wire.iter().find(|v|v["id"]=="own-permission").unwrap();assert_eq!(answer["result"]["outcome"]["optionId"],if allow{"opaque-native-remember"}else{"opaque-native-reject"});assert!(!wire.iter().any(|v|v["params"]["value"]=="on"||v["params"]["value"]=="autopilot"));
        }
    }

    fn late_review(behavior:&str){
        let f=Fixture::new(behavior);let mut c=f.client();let monitor=c.monitor.clone();let original_stop=Arc::new(AtomicBool::new(false));c.cancel_with(original_stop.clone());let mut calls=0;
        let result=c.answer_native(options(None),|_|{},|_|{},|_|{},|_,ticket|{
            calls+=1;assert!(ticket.current());fs::write(f.dir.join("review-open"),b"ready").unwrap();
            let deadline=Instant::now()+Duration::from_secs(3);
            while ticket.current(){assert!(Instant::now()<deadline,"reader failed to withdraw a native review while its callback waited");std::thread::sleep(Duration::from_millis(5));}
            assert_eq!(ticket.policy_invalidated(),matches!(behavior,"source-config"|"source-model"));if behavior=="source-model"{assert_eq!(monitor.lock().unwrap().catalog.as_ref().unwrap().current_model,"model-a");}if behavior=="source-request-cancel"{assert!(ticket.withdrawal_pending());}Some(json!({"allow":true}))
        });
        assert_eq!(calls,1);
        if behavior=="source-complete"{assert_eq!(result.unwrap().text,"Native prompt already finished");}else if matches!(behavior,"source-config"|"source-request-cancel"){assert_eq!(result.unwrap().text,"Olá equipa");}else{assert!(result.is_err());}assert!(!original_stop.load(Ordering::Acquire));
        drop(c);let wire=f.wire();
        assert!(!wire.iter().any(|v|v["id"]=="own-permission"&&v["result"]["outcome"]["outcome"]=="selected"),"late review must never select an option");
        if matches!(behavior,"source-config"|"source-model"|"source-request-cancel"){assert_eq!(wire.iter().filter(|v|v["id"]=="own-permission"&&v["result"]==cancelled_outcome()).count(),1,"still-pending native request needs one cancellation outcome");}
        else{assert!(!wire.iter().any(|v|v["id"]=="own-permission"),"finished/disconnected/ambiguous source must not receive a second response");}
        if behavior=="source-request-cancel"{assert!(!wire.iter().any(|v|v["method"]=="session/cancel"));}
    }
    #[test]fn prompt_completion_withdraws_a_review_even_behind_more_than_32_events(){late_review("source-complete");}
    #[test]fn native_configuration_update_withdraws_the_review_before_answer_delivery(){late_review("source-config");}
    #[test]fn model_updates_are_observed_while_the_human_waits_and_explicit_selection_is_preserved(){late_review("source-model");}
    #[test]fn agent_request_cancellation_withdraws_the_review_once_and_keeps_the_prompt_running(){late_review("source-request-cancel");}
    #[test]fn native_eof_withdraws_the_review_while_the_human_callback_is_waiting(){late_review("source-eof");}
    #[test]fn repeated_pending_native_id_aborts_the_source_without_answering_the_original(){late_review("source-duplicate");}

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
    fn native_policy_is_preserved_during_catalog_listing_creation_and_resume() {
        let f=Fixture::new("native-policy");let mut c=f.client();assert_eq!(c.models().unwrap().len(),2);assert_eq!(c.catalog.config("mode","mode").unwrap()["currentValue"],"autopilot");assert_eq!(c.catalog.config("permissions","allow_all").unwrap()["currentValue"],"on");drop(c);
        assert!(!f.wire().iter().any(|v|v["method"]=="session/set_config_option"||v["method"]=="session/prompt"));
        for session in[None,Some("owned-session")]{let mut c=f.client();let reply=c.answer(options(session),|_|{},|_|{},|_|{},|_|Some(false)).unwrap();assert_eq!(reply.model,"model-b");assert_eq!(c.catalog.config("mode","mode").unwrap()["currentValue"],"autopilot");assert_eq!(c.catalog.config("permissions","allow_all").unwrap()["currentValue"],"on");drop(c);}
        assert!(!f.wire().iter().any(|v|v["method"]=="session/set_config_option"&&matches!(v["params"]["configId"].as_str(),Some("mode"|"allow_all"))));
    }
    #[test]fn unconfirmed_model_configuration_still_prevents_inference(){
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
        let f=Fixture::new("cancel-await");let mut c=f.client();let cancel=Arc::new(AtomicBool::new(false));c.cancel_with(cancel.clone());
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

    #[test]fn queued_native_packet_retains_source_and_caller_until_the_writer_proves_no_bytes_were_sent(){
        let f=Fixture::new("normal");let mut c=f.client();let(ticket,source)= {let e=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let t=e.capture(Capture{request_id:Some(&json!(91)),session_id:"owned-session",turn_id:None,prompt_id:Some(&json!(2)),method:"session/request_permission",arguments:&json!({}),policy:PolicyBinding::unknown(),cancel:None}).unwrap();(t,e)};
        let(tx,rx)=mpsc::sync_channel(1);let _original=c.input.replace(tx).unwrap();let t=ticket.clone();
        let worker=std::thread::spawn(move||{let result=c.respond_native(json!({"outcome":{"outcome":"selected","optionId":"synthetic"}}),&t,&json!({}));(c,result)});
        let packet=rx.recv_timeout(Duration::from_secs(2)).unwrap();assert!(packet.guard.source.as_ref().unwrap().0.same_capture(&ticket));assert!(packet.guard.caller.as_ref().unwrap().load(Ordering::Acquire));assert_eq!(serde_json::from_slice::<Value>(&packet.bytes).unwrap()["id"],91);ticket.cancel();
        let mut out=Vec::new();let outcome=crate::pet_native_write::frame(&mut out,&packet.bytes,&packet.guard).unwrap();assert_eq!(outcome,crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::SourceEnded));assert!(out.is_empty());let caller=packet.guard.caller.as_ref().unwrap().clone();packet.ack.send(Ok(outcome)).unwrap();let(c,result)=worker.join().unwrap();assert_eq!(result.unwrap(),outcome);assert!(!caller.load(Ordering::Acquire));drop(c);drop(source);
    }
    #[cfg(target_os="linux")]
    #[test]fn final_dispatch_copilot_queued_packet_rechecks_current_io_at_actual_writer(){
        let io=crate::settings::EffectFixture::new();let f=Fixture::new("normal");let mut c=f.client();c.io_probe=Some(io.probe.clone());let endpoint=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let ticket=endpoint.capture(Capture{request_id:Some(&json!(91)),session_id:"owned-session",turn_id:None,prompt_id:None,method:"session/request_permission",arguments:&json!({}),policy:PolicyBinding::unknown(),cancel:None}).unwrap();let(tx,rx)=mpsc::sync_channel(1);let _original=c.input.replace(tx).unwrap();
        let worker=std::thread::spawn(move||{let result=c.respond_native(json!({"outcome":{"outcome":"selected","optionId":"positive"}}),&ticket,&json!({}));(c,result)});
        let packet=rx.recv_timeout(Duration::from_secs(2)).unwrap();io.external();let mut out=Vec::new();let outcome=crate::pet_native_write::frame(&mut out,&packet.bytes,&packet.guard).unwrap();assert_eq!(outcome,crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::PreferencesUnconfirmed));assert!(out.is_empty());packet.ack.send(Ok(outcome)).unwrap();let(c,result)=worker.join().unwrap();assert_eq!(result.unwrap(),outcome);drop(c);drop(endpoint);
    }
    #[cfg(target_os="linux")]
    #[test]fn final_dispatch_copilot_partial_io_loss_retires_owned_writer_without_completing_frame(){
        struct Partial{io:Arc<crate::settings::EffectFixture>,bytes:Arc<Mutex<Vec<u8>>>,first:bool}
        impl Write for Partial{fn write(&mut self,b:&[u8])->std::io::Result<usize>{if self.first{self.first=false;self.bytes.lock().unwrap().extend_from_slice(&b[..2]);self.io.external();Ok(2)}else{panic!("must retire partial IO frame")}}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
        let io=Arc::new(crate::settings::EffectFixture::new());let source=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let bytes=Arc::new(Mutex::new(Vec::new()));let(tx,rx)=mpsc::sync_channel(1);let(ack,answer)=mpsc::sync_channel(1);let guard=crate::pet_native_write::Guard{dispatch:crate::pet_native_write::Dispatch::ProbedEffect(io.probe.clone()),deadline:Instant::now()+Duration::from_secs(2),cancel:None,caller:None,source:None};tx.send(Outbound{bytes:b"grant\n".to_vec(),guard,ack}).unwrap_or_else(|_|panic!("queue unavailable"));run_writer(Partial{io:io.clone(),bytes:bytes.clone(),first:true},rx,source.clone());let error=answer.recv().unwrap().unwrap_err();assert!(error.partial);assert_eq!(error.reason,crate::pet_native_write::Reason::PreferencesUnconfirmed);assert_eq!(*bytes.lock().unwrap(),b"gr");assert!(!source.is_open());
    }
    #[test]fn a_native_writer_partial_fault_closes_its_source_and_never_accepts_a_following_frame(){
        struct Partial{source:Endpoint,bytes:Arc<Mutex<Vec<u8>>>,first:bool}
        impl Write for Partial{fn write(&mut self,b:&[u8])->std::io::Result<usize>{if self.first{self.first=false;self.bytes.lock().unwrap().extend_from_slice(&b[..2]);self.source.close();Ok(2)}else{panic!("must not complete a partial native response")}}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
        let source=Endpoint::new(Adapter::Copilot,Transport::ManagedPipe).unwrap();let ticket=source.capture(Capture{request_id:Some(&json!(91)),session_id:"owned-session",turn_id:None,prompt_id:None,method:"session/request_permission",arguments:&json!({}),policy:PolicyBinding::unknown(),cancel:None}).unwrap();let bytes=Arc::new(Mutex::new(Vec::new()));let(tx,rx)=mpsc::sync_channel(2);let(ack,answer)=mpsc::sync_channel(1);
        assert!(tx.send(Outbound{bytes:b"grant\n".to_vec(),ack,guard:crate::pet_native_write::Guard{dispatch:crate::pet_native_write::Dispatch::NewEffect,deadline:Instant::now()+Duration::from_secs(2),cancel:None,caller:None,source:Some((ticket,crate::pet_native_write::Use::Decision))}}).is_ok());run_writer(Partial{source:source.clone(),bytes:bytes.clone(),first:true},rx,source.clone());let fault=answer.recv().unwrap().unwrap_err();assert!(fault.partial);assert_eq!(*bytes.lock().unwrap(),b"gr");assert!(!source.is_open());let(ack,_)=mpsc::sync_channel(1);assert!(tx.send(Outbound{bytes:b"again\n".to_vec(),ack,guard:crate::pet_native_write::Guard{dispatch:crate::pet_native_write::Dispatch::NewEffect,deadline:Instant::now()+Duration::from_secs(1),cancel:None,caller:None,source:None}}).is_err());
    }

    #[test]fn a_saved_session_accepts_no_echo_but_rejects_a_conflicting_echo_without_replacement(){
        let f=Fixture::new("load-no-echo");let mut c=f.client();let reply=c.answer(options(Some("owned-session")),|_|{},|_|{},|_|{},|_|Some(false)).unwrap();assert_eq!(reply.session_id,"owned-session");drop(c);assert!(!f.wire().iter().any(|v|v["method"]=="session/new"));
        let g=Fixture::new("load-wrong-echo");let mut c=g.client();let mut sessions=0;assert!(c.answer(options(Some("owned-session")),|_|sessions+=1,|_|{},|_|{},|_|None).is_err());assert_eq!(sessions,0);drop(c);assert!(!g.wire().iter().any(|v|v["method"]=="session/new"||v["method"]=="session/prompt"));
    }
    #[test]fn a_later_wire_config_notification_is_not_replaced_by_an_older_unconsumed_rpc_response(){
        let f=Fixture::new("ordered-config");let mut c=f.client();c.open(None).unwrap();let before=c.monitor.lock().unwrap().config_sequence;
        let id=c.begin_request("session/set_config_option",json!({"sessionId":"owned-session","configId":"model","value":"model-b"})).unwrap();let until=Instant::now()+Duration::from_secs(2);
        while c.monitor.lock().unwrap().config_sequence<before+2{assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(2));}
        let response=c.read().unwrap();assert_eq!(response["id"],id);assert_eq!(Catalog::from_result(&response["result"]).current_model,"model-b");c.sync_catalog().unwrap();assert_eq!(c.catalog.current_model,"model-a");assert!(c.validate_selection("model-b","").is_err());drop(c);assert!(!f.wire().iter().any(|v|v["method"]=="session/prompt"));
    }
    #[test]fn an_already_finished_prompt_is_not_cancelled_when_a_late_model_mismatch_is_reported(){
        let f=Fixture::new("source-complete-model");let mut c=f.client();let source=c.source.clone();let mut reviews=0;let result=c.answer_native(options(None),|_|{},|_|{},|_|{},|_,ticket|{reviews+=1;fs::write(f.dir.join("review-open"),b"ready").unwrap();let until=Instant::now()+Duration::from_secs(3);while source.is_open(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(2));}assert!(!ticket.current());Some(json!({"allow":true}))});assert_eq!(reviews,1);assert!(result.is_err());assert!(!c.source.is_open());drop(c);assert!(!f.wire().iter().any(|v|v["method"]=="session/cancel"||v["id"]=="own-permission"));
    }
    #[test]fn native_permission_environment_and_mode_are_not_overridden_by_the_launcher(){
        let cmd=command(Path::new("synthetic-copilot"),Path::new("/fixture"));let args=cmd.get_args().map(|v|v.to_string_lossy().to_string()).collect::<Vec<_>>();assert!(!args.iter().any(|v|v=="--mode"||v=="--allow-all"||v=="--deny-all"));
        let changed=cmd.get_envs().map(|(key,_)|key.to_string_lossy().to_string()).collect::<Vec<_>>();assert!(!changed.iter().any(|k|k=="COPILOT_ALLOW_ALL"||k=="COPILOT_ASSISTED_APPROVAL"));
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
