//! Real private pipes and the production operation broker, without providers,
//! user devices, account data or an app window.
use super::*;
use std::{cell::Cell,path::PathBuf};
const A:&str="11111111-1111-1111-1111-111111111111";
const SERVER:&str=r#"import json,os,pathlib,sys,time
root=pathlib.Path('.')
mode=(root/'tool-mode').read_text()
A='11111111-1111-1111-1111-111111111111'
replies=0
def send(v):print(json.dumps(v),flush=True)
def tool(i=77,call='native-call',turn='turn-1',thread=A,namespace=None):
 p={'threadId':thread,'turnId':turn,'callId':call,'tool':'music','arguments':{'action':'play'}}
 if namespace is not None:p['namespace']=namespace
 v={'id':i,'method':'item/tool/call','params':p}
 if mode=='missing-call' and i==76:del p['callId']
 if mode=='no-id' and i==76:del v['id']
 send(v)
def done():
 send({'method':'item/completed','params':{'threadId':A,'turnId':'turn-1','item':{'id':'final','type':'agentMessage','text':'completed'}}})
 send({'method':'turn/completed','params':{'threadId':A,'turn':{'id':'turn-1','status':'completed'}}})
def entered():
 until=time.monotonic()+4
 while not (root/'tool-entered').exists():
  if time.monotonic()>until:sys.exit(9)
  time.sleep(.002)
for line in sys.stdin:
 p=json.loads(line)
 with (root/'tool-requests.jsonl').open('a')as f:f.write(json.dumps(p)+'\n')
 m=p.get('method')
 if m=='initialized':continue
 if m is None:
  replies+=1
  if mode in ['foreign-turn','foreign-thread','missing-call','bad-id','namespace'] and replies==1:tool();continue
  if mode in ['replay','reused-wire'] and replies==1:tool(78 if mode=='replay' else 77);continue
  if mode=='budget' and replies<101:tool(77+replies,'budget-'+str(replies+1));continue
  done();continue
 result={}
 if m=='account/read':result={'account':{'type':'chatgpt'}}
 if m=='config/read':result={'config':{}}
 if m in ['thread/start','thread/resume']:result={'thread':{'id':A}}
 if m=='turn/start':
  result={'turn':{'id':'turn-1'}}
  if mode=='before-ack':tool()
 if 'id'in p:send({'id':p['id'],'result':result})
 if m=='turn/start':
  if mode=='foreign-turn':tool(76,'bad-call',turn='foreign-turn')
  elif mode=='foreign-thread':tool(76,'bad-call',thread='22222222-2222-2222-2222-222222222222')
  elif mode=='missing-call':tool(76,'bad-call')
  elif mode=='bad-id':tool({'invalid':True},'bad-call')
  elif mode=='namespace':tool(76,'bad-call',namespace='other-namespace')
  elif mode=='no-id':tool(76,'bad-call');tool()
  elif mode=='budget':tool(77,'budget-1')
  elif mode=='declared-namespace':tool(namespace='declared')
  elif mode!='before-ack':tool()
  if mode in ['resolved','effect-resolved','disconnect','policy','duplicate-wire']:
   entered()
   if mode=='disconnect':sys.exit(0)
   if mode=='duplicate-wire':tool()
   elif mode=='policy':send({'method':'thread/settings/updated','params':{'threadId':A,'threadSettings':{'approvalPolicy':'never','sandboxPolicy':{'type':'dangerFullAccess'}}}})
   else:send({'method':'serverRequest/resolved','params':{'threadId':A,'requestId':77}});done()
  if mode=='partial':
   (root/'tool-partial-prefix').write_bytes(os.read(sys.stdin.fileno(),1))
   send({'method':'serverRequest/resolved','params':{'threadId':A,'requestId':77}});done();time.sleep(4)
"#;
struct Fixture{cwd:PathBuf,exe:PathBuf}
impl Fixture{
 fn new(mode:&str)->Self{let(cwd,exe)=super::tests::mock();std::fs::write(cwd.join("app-server"),SERVER).unwrap();std::fs::write(cwd.join("tool-mode"),mode).unwrap();Self{cwd,exe}}
 fn received(&self)->Vec<Value>{std::fs::read_to_string(self.cwd.join("tool-requests.jsonl")).unwrap().lines().map(|l|serde_json::from_str(l).unwrap()).collect()}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.cwd);}}
fn options(tools:&[Value])->TurnOptions<'_>{TurnOptions{model:"fixture-free",system:"Private native effect fixture",history:&[],query:"Original request",tools,session_id:None,full_access:false,effort:None,name:None}}
fn tools()->Vec<Value>{vec![json!({"type":"function","name":"music","description":"Synthetic only","inputSchema":{"type":"object"}})]}
fn config()->crate::pet_config::Configuration{let mut c=crate::pet_config::defaults();c.pets.get_mut("agent_codex").unwrap().permissions=Some(crate::pet_config::Permissions{documents:false,email_drafts:false,music:true,open_apps:false});c}
fn wait_ended(fence:&crate::pet_native_invocation::Fence){let until=Instant::now()+Duration::from_secs(3);while fence.current(){assert!(Instant::now()<until,"native source did not end");std::thread::sleep(Duration::from_millis(2));}}

#[test]fn declared_namespace_members_dispatch_without_aliasing_flat_or_other_group_names(){
 let member=tools().remove(0);let declared=vec![json!({"type":"namespace","name":"declared","description":"Synthetic group","tools":[member]})];assert!(tool_available(&declared,"music",&json!("declared")));assert!(!tool_available(&declared,"music",&Value::Null));assert!(!tool_available(&declared,"declared",&Value::Null));assert!(!tool_available(&declared,"music",&json!("other")));let mut duplicate=declared.clone();duplicate.push(declared[0].clone());assert!(!tool_available(&duplicate,"music",&json!("declared")));
 let f=Fixture::new("declared-namespace");let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let count=Cell::new(0);c.answer_native_tools(options(&declared),|name,_,native|{assert_eq!(name,"music");assert!(native.current());count.set(1);("synthetic".into(),false)},|_,_,_|panic!("no human"),|_|{},|_|{}).unwrap();assert_eq!(count.get(),1);drop(c);
}

#[test]fn original_native_tool_identity_reaches_the_common_broker_once_even_before_turn_ack(){
 for mode in["normal","before-ack"]{
  let f=Fixture::new(mode);let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let effects=Cell::new(0);let callbacks=Cell::new(0);let mut retained=None;
  c.answer_native_tools(options(&tools()),|name,args,native|{callbacks.set(callbacks.get()+1);assert_eq!(native.source().identity().request_id,json!(77));assert_eq!(native.source().identity().session_id,A);assert_eq!(native.source().identity().turn_id.as_deref(),Some("turn-1"));retained=Some(native.clone());crate::pet_operations::run_native(native,"agent_codex",name,args,||true,||Ok(config()),|_|{effects.set(effects.get()+1);crate::pet_operations::NativeOutcome::new(crate::pet_operations::Delivery::Accepted,"synthetic ACK")}).legacy()},|_,_,_|panic!("no human request"),|_|{},|_|{}).unwrap();assert_eq!(callbacks.get(),1);assert_eq!(effects.get(),1);assert!(!retained.unwrap().current());drop(c);let received=f.received();let replies=received.iter().filter(|v|v["id"]==77&&v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["result"]["success"],true);
 }
}
#[test]fn foreign_scope_malformed_ids_notifications_and_other_namespaces_cannot_dispatch_an_operation(){
 for mode in["foreign-turn","foreign-thread","missing-call","bad-id","namespace","no-id"]{
  let f=Fixture::new(mode);let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let count=Cell::new(0);
  c.answer_native_tools(options(&tools()),|_,_,native|{assert_eq!(native.source().identity().request_id,json!(77));count.set(count.get()+1);("valid callback".into(),false)},|_,_,_|panic!("unbound tool escaped"),|_|{},|_|{}).unwrap();assert_eq!(count.get(),1,"{mode}");drop(c);
 }
}
#[test]fn duplicate_native_call_ids_never_repeat_an_effect_even_with_a_new_or_reused_rpc_id(){
 for mode in["replay","reused-wire"]{let f=Fixture::new(mode);let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let count=Cell::new(0);
  c.answer_native_tools(options(&tools()),|_,_,_|{count.set(count.get()+1);("synthetic accepted".into(),true)},|_,_,_|panic!("no human"),|_|{},|_|{}).unwrap();assert_eq!(count.get(),1);drop(c);let replies=f.received().into_iter().filter(|v|v.get("method").is_none()&&v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.iter().filter(|v|v["result"]["success"]==true).count(),1);
 }
}
#[test]fn resolution_eof_policy_or_duplicate_wire_id_before_dispatch_prevents_the_effect_without_stopping_the_turn(){
 for mode in["resolved","disconnect","policy","duplicate-wire"]{
  let f=Fixture::new(mode);let stop=Arc::new(AtomicBool::new(false));let mut c=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(6),false,stop.clone()).unwrap();let count=Cell::new(0);let mut seen=0;
  let result=c.answer_native_tools(options(&tools()),|name,args,native|{seen+=1;std::fs::write(f.cwd.join("tool-entered"),"ready").unwrap();wait_ended(native);crate::pet_operations::run_native(native,"agent_codex",name,args,||true,||Ok(config()),|_|{count.set(1);crate::pet_operations::NativeOutcome::new(crate::pet_operations::Delivery::Accepted,"")}).legacy()},|_,_,_|panic!("no human"),|_|{},|_|{});
  assert_eq!(seen,1);assert_eq!(count.get(),0);assert!(!stop.load(Ordering::Acquire));assert_eq!(result.is_ok(),mode!="disconnect");drop(c);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("result").is_some()));
 }
}
#[test]fn resolution_after_a_real_broker_dispatch_preserves_uncertain_scope_and_never_replays_or_sends_a_late_receipt(){
 let f=Fixture::new("effect-resolved");let stop=Arc::new(AtomicBool::new(false));let mut c=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(6),false,stop.clone()).unwrap();let count=Cell::new(0);let mut receipt=None;
 let result=c.answer_native_tools(options(&tools()),|name,args,native|{let r=crate::pet_operations::run_native(native,"agent_codex",name,args,||true,||Ok(config()),|_|{count.set(count.get()+1);std::fs::write(f.cwd.join("tool-entered"),"effect started").unwrap();wait_ended(native);crate::pet_operations::NativeOutcome::new(crate::pet_operations::Delivery::Accepted,"native ACK")});assert_eq!(r.delivery,crate::pet_operations::Delivery::Accepted);assert!(!r.scope_current);let result=r.legacy();receipt=Some(serde_json::from_str::<Value>(&result.0).unwrap());result},|_,_,_|panic!("no human"),|_|{},|_|{}).unwrap();assert_eq!(count.get(),1);assert!(!result.acted);assert_eq!(receipt.unwrap()["effectVerified"],false);assert!(!stop.load(Ordering::Acquire));drop(c);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("result").is_some()));
}
#[test]fn native_callback_limit_is_still_100_and_oversized_results_do_not_trigger_another_callback(){
 for mode in["budget","oversized"]{let f=Fixture::new(mode);let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(8),false).unwrap();let count=Cell::new(0);c.answer_native_tools(options(&tools()),|_,_,_|{count.set(count.get()+1);(if mode=="oversized"{"x".repeat(300_000)}else{"synthetic".into()},false)},|_,_,_|panic!("no human"),|_|{},|_|{}).unwrap();assert_eq!(count.get(),if mode=="budget"{100}else{1});drop(c);let got=f.received();let last=got.iter().filter(|v|v.get("method").is_none()&&v.get("result").is_some()).last().unwrap();assert_eq!(last["result"]["success"],false);assert!(serde_json::to_vec(last).unwrap().len()<256*1024);}
}
#[test]fn a_partially_written_native_tool_result_is_retired_without_repeating_the_callback(){
 let f=Fixture::new("partial");let mut c=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let count=Cell::new(0);let result=c.answer_native_tools(options(&tools()),|_,_,_|{count.set(count.get()+1);("x".repeat(128*1024),true)},|_,_,_|panic!("no human"),|_|{},|_|{});assert!(result.is_err());assert!(c.write_incomplete);assert!(c.input.is_none());assert_eq!(count.get(),1);assert_eq!(std::fs::read(f.cwd.join("tool-partial-prefix")).unwrap(),b"{");drop(c);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("result").is_some()));
}
