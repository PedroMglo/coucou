//! Real private pipes/child, synthetic protocol, no native CLI/account/inference.
use super::*;
use std::path::PathBuf;
const SERVER:&str=r#"import json,os,pathlib,sys,time
root=pathlib.Path('.')
mode=(root/'request-mode').read_text()
(root/'request-pid').write_text(str(os.getpid()))
def send(v): print(json.dumps(v),flush=True)
def approval(i=77,turn='turn-1'):
    if mode.startswith('legacy-'):
        method='execCommandApproval'
        params={'conversationId':'foreign-thread' if mode=='legacy-foreign' and i==76 else 'owned-thread','callId':'call-is-not-a-turn','command':['fixture','--exact'],'cwd':'/fixture','parsedCmd':[]}
        if mode=='legacy-root':
            method='applyPatchApproval';params={'conversationId':'owned-thread','callId':'patch-call','fileChanges':{},'grantRoot':'/fixture/root'}
        send({'id':i,'method':method,'params':params});return
    if mode.startswith('mcp-'):
        params={'threadId':'owned-thread','serverName':'fixture-server','mode':'form','message':'Private fixture question','requestedSchema':{'type':'object','properties':{'answer':{'type':'string'}}}}
        if mode.startswith('mcp-typed'):
            params['mode']='openai/form'
            params['requestedSchema']={'type':'object','properties':{'count':{'type':'integer','minimum':1,'maximum':3},'flag':{'type':'boolean','default':True},'pick':{'type':'string','oneOf':[{'const':'wire-value','title':'Friendly'}]},'tags':{'type':'array','items':{'anyOf':[{'const':'a','title':'First'},{'const':'b','title':'Second'}]},'minItems':1,'maxItems':2},'secret':{'type':'string','format':'password','default':'PRIVATE_DEFAULT'}},'required':['count','flag','pick','tags']}
        if mode!='mcp-missing': params['turnId']=turn if mode=='mcp-wrong-turn' else None
        send({'id':i,'method':'mcpServer/elicitation/request','params':params});return
    if mode=='decision-session':
        send({'id':i,'method':'item/commandExecution/requestApproval','params':{'threadId':'owned-thread','turnId':turn,'command':'fixture command','availableDecisions':['acceptForSession','cancel']}});return
    if mode=='decision-policy':
        send({'id':i,'method':'item/commandExecution/requestApproval','params':{'threadId':'owned-thread','turnId':turn,'command':'fixture command','availableDecisions':[{'acceptWithExecpolicyAmendment':{'execpolicy_amendment':['fixture','--exact']}}]}});return
    if mode=='partial-response':
        send({'id':i,'method':'item/permissions/requestApproval','params':{'threadId':'owned-thread','turnId':turn,'permissions':{'fileSystem':{'read':['/fixture/'+('x'*2048)+str(n) for n in range(100)]}}}});return
    if mode=='decision-permissions':
        send({'id':i,'method':'item/permissions/requestApproval','params':{'threadId':'owned-thread','turnId':turn,'permissions':{'network':{'enabled':True}}}});return
    send({'id':i,'method':'item/commandExecution/requestApproval','params':{'threadId':'owned-thread','turnId':turn,'itemId':'item-1','startedAtMs':1000,'command':'fixture command','availableDecisions':['accept','decline']}})
def completed(text):
    send({'method':'item/completed','params':{'threadId':'owned-thread','turnId':'turn-1','item':{'id':'final','type':'agentMessage','text':text}}})
    send({'method':'turn/completed','params':{'threadId':'owned-thread','turn':{'id':'turn-1','status':'completed'}}})
for line in sys.stdin:
    p=json.loads(line)
    with (root/'native-requests.jsonl').open('a') as f: f.write(json.dumps(p)+'\n')
    m=p.get('method')
    if m=='initialized': continue
    if m is None:
        if p.get('id')==76: approval()
        if p.get('id')==77: completed('denied')
        continue
    result={}
    if m=='account/read': result={'account':{'type':'chatgpt'}}
    if m=='config/read': result={'config':{}}
    if m in ['thread/start','thread/resume']: result={'thread':{'id':'owned-thread'}}
    if m=='turn/start': result={'turn':{'id':'turn-1'}}
    if m=='turn/start' and mode in ['legacy-before-turn','mcp-typed-before-turn']: approval()
    if 'id' in p: send({'id':p['id'],'result':result})
    if m=='turn/start':
        if mode in ['wrong-turn','mcp-wrong-turn']: approval(76,'other-turn')
        elif mode=='legacy-foreign': approval(76)
        elif mode not in ['legacy-before-turn','mcp-typed-before-turn']: approval()
        if mode=='partial-response':
            prefix=os.read(sys.stdin.fileno(),1)
            (root/'partial-response-prefix').write_bytes(prefix)
            send({'method':'serverRequest/resolved','params':{'threadId':'owned-thread','requestId':77}})
            completed('resolved while writing')
            time.sleep(4)
        if mode in ['resolved','disconnect','legacy-resolved','legacy-disconnect','mcp-resolved','mcp-disconnect','mcp-typed-resolved','mcp-typed-disconnect']:
            until=time.monotonic()+4
            while not (root/'human-callback-entered').exists():
                if time.monotonic()>until: sys.exit(4)
                time.sleep(.002)
            if mode.endswith('disconnect'): sys.exit(0)
            for i in range(48): send({'method':'item/agentMessage/delta','params':{'threadId':'owned-thread','turnId':'turn-1','itemId':'progress','delta':'progress '}})
            send({'method':'serverRequest/resolved','params':{'threadId':'owned-thread','requestId':77}})
            completed('resolved elsewhere')
"#;
struct Fixture{cwd:PathBuf,exe:PathBuf}
impl Fixture{
    fn new(mode:&str)->Self{let(cwd,exe)=super::tests::mock();std::fs::write(cwd.join("app-server"),SERVER).unwrap();std::fs::write(cwd.join("request-mode"),mode).unwrap();Self{cwd,exe}}
    fn received(&self)->Vec<Value>{std::fs::read_to_string(self.cwd.join("native-requests.jsonl")).unwrap().lines().map(|l|serde_json::from_str(l).unwrap()).collect()}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.cwd);}}
fn options()->TurnOptions<'static>{TurnOptions{model:"fixture-free",system:"Private synthetic fixture",history:&[],query:"Original request",tools:&[],session_id:None,full_access:false,effort:None,name:None}}
#[test]
fn actual_native_request_reaches_the_callback_with_original_identity_and_is_retired_after_one_reply(){
    let f=Fixture::new("normal");let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let mut seen=vec![];
    let result=client.answer_native(options(),|_,_|panic!("no tool"),|method,params,ticket|{let ticket=ticket.unwrap();assert!(ticket.current());assert_eq!(method,ticket.identity().method);assert_eq!(params["threadId"],ticket.identity().session_id);assert_eq!(params["turnId"].as_str(),ticket.identity().turn_id.as_deref());assert_eq!(ticket.identity().request_id,json!(77));assert_eq!(ticket.identity().endpoint.len(),64);seen.push(ticket.clone());Some(json!({"decision":"decline"}))},|_|{},|_|{}).unwrap();
    assert_eq!(result.text,"denied");assert_eq!(seen.len(),1);assert!(!seen[0].current());drop(client);let replies:Vec<_>=f.received().into_iter().filter(|v|v["id"]==77&&v.get("method").is_none()).collect();assert_eq!(replies.len(),1);assert_eq!(replies[0]["result"]["decision"],"decline");
}
#[test]
fn another_turns_native_request_never_reaches_the_human_callback(){
    let f=Fixture::new("wrong-turn");let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let mut count=0;
    let result=client.answer_native(options(),|_,_|panic!("no tool"),|_,params,ticket|{assert_eq!(params["turnId"],"turn-1");assert_eq!(ticket.unwrap().identity().request_id,json!(77));count+=1;Some(json!({"decision":"decline"}))},|_|{},|_|{}).unwrap();assert_eq!(count,1);assert_eq!(result.text,"denied");drop(client);let got=f.received();let rejected=got.iter().find(|v|v["id"]==76&&v.get("method").is_none()).unwrap();assert!(rejected.get("error").is_some());
}
#[test]
fn terminal_resolution_behind_more_than_32_events_invalidates_a_waiting_human_callback_without_a_second_reply(){
    let f=Fixture::new("resolved");let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let marker=f.cwd.join("human-callback-entered");let mut count=0;
    let result=client.answer_native(options(),|_,_|panic!("no tool"),|_,_,ticket|{let ticket=ticket.unwrap();assert!(ticket.current());count+=1;std::fs::write(&marker,"ready").unwrap();let until=Instant::now()+Duration::from_secs(3);while ticket.current(){assert!(Instant::now()<until,"native resolution did not invalidate the original request");std::thread::sleep(Duration::from_millis(2));}Some(json!({"decision":"accept"}))},|_|{},|_|{}).unwrap();assert_eq!(count,1);assert_eq!(result.text,"resolved elsewhere");drop(client);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("method").is_none()));
}
#[test]
fn disconnect_during_a_human_callback_invalidates_the_ticket_and_never_sends_its_answer(){
    let f=Fixture::new("disconnect");let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let marker=f.cwd.join("human-callback-entered");
    let result=client.answer_native(options(),|_,_|panic!("no tool"),|_,_,ticket|{let ticket=ticket.unwrap();std::fs::write(&marker,"ready").unwrap();let until=Instant::now()+Duration::from_secs(3);while ticket.current(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(2));}Some(json!({"decision":"accept"}))},|_|{},|_|{});assert!(result.is_err());drop(client);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("method").is_none()));
}

#[test]fn actual_native_pipes_preserve_explicit_session_policy_and_permission_scope_choices(){
 for(mode,scope,expected)in[("decision-session","session",json!({"decision":"acceptForSession"})),("decision-policy","policy",json!({"decision":{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["fixture","--exact"]}}})),("decision-permissions","session",json!({"permissions":{"network":{"enabled":true}},"scope":"session"}))]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();client.answer_native(options(),|_,_|panic!("no tool"),|_,params,source|{let source=source.unwrap();let choices=crate::pet_native_choices::Choices::capture(source,"native-review",params).unwrap().unwrap();let public=choices.public();let id=public.as_array().unwrap().iter().find(|v|v["scope"]==scope).unwrap()["id"].clone();let mut human=json!({"allow":true,"nativeChoice":id});human["nativeResponse"]=choices.resolve(&human).unwrap();crate::pet_native_choices::codex_response(&source.identity().method,params,Some(&human))},|_|{},|_|{}).unwrap();drop(client);let received=f.received();let replies=received.iter().filter(|v|v["id"]==77&&v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["result"],expected);
 }
}

#[test]fn a_native_response_resolved_during_real_pipe_backpressure_is_never_completed_or_retried(){
 let f=Fixture::new("partial-response");let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let result=client.answer_native(options(),|_,_|panic!("no tool"),|_,params,source|{let source=source.unwrap();let choices=crate::pet_native_choices::Choices::capture(source,"large-native-response",params).unwrap().unwrap();let public=choices.public();let id=public.as_array().unwrap().iter().find(|v|v["scope"]=="session").unwrap()["id"].clone();choices.resolve(&json!({"allow":true,"nativeChoice":id})).ok()},|_|{},|_|{});
 assert!(result.is_err());assert!(client.write_incomplete);assert!(client.input.is_none());assert_eq!(std::fs::read(f.cwd.join("partial-response-prefix")).unwrap(),b"{");drop(client);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("result").is_some()));
}
#[test]fn a_guarded_response_cannot_change_its_native_request_id_or_replay_after_resolution(){
 let f=Fixture::new("normal");let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let(source,_)=crate::pet_native_decisions::Ticket::fixture();assert!(client.write_frame_guarded(json!({"id":78,"result":{"decision":"accept"}}),Instant::now()+Duration::from_secs(1),false,Some(&source),crate::pet_native_write::Use::Decision).is_err());source.cancel();let outcome=client.write_frame_guarded(json!({"id":77,"result":{"decision":"accept"}}),Instant::now()+Duration::from_secs(1),false,Some(&source),crate::pet_native_write::Use::Decision).unwrap();assert_eq!(outcome,crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::SourceEnded));drop(client);assert!(!f.received().iter().any(|v|v["id"]==77||v["id"]==78));
}

#[test]fn legacy_pipe_choices_preserve_exact_denial_session_abort_and_requested_root(){
 for(mode,index,expected)in[
  ("legacy-command",2,json!({"decision":{"denied":{"rejection":"The request was not approved."}}})),
  ("legacy-session",1,json!({"decision":"approved_for_session"})),
  ("legacy-abort",3,json!({"decision":"abort"})),
  ("legacy-root",1,json!({"decision":"approved_for_session"}))]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let mut seen=None;
  client.answer_native(options(),|_,_|panic!("no tool"),|method,params,ticket|{
   let ticket=ticket.unwrap();assert_eq!(ticket.identity().session_id,"owned-thread");assert!(ticket.identity().turn_id.is_none());assert_eq!(ticket.identity().request_id,json!(77));assert_ne!(params["callId"],ticket.identity().request_id);
   let choices=crate::pet_native_choices::Choices::capture(ticket,"legacy-card",params).unwrap().unwrap();let rows=choices.public();if mode=="legacy-root"{assert!(choices.resolve(&json!({"allow":true})).is_err());assert_eq!(rows[index]["scope"],"session");}
   let mut answer=json!({"allow":rows[index]["allow"],"nativeChoice":rows[index]["id"]});answer["nativeResponse"]=choices.resolve(&answer).unwrap();seen=Some(ticket.clone());crate::pet_native_choices::codex_response(method,params,Some(&answer))
  },|_|{},|_|{}).unwrap();assert!(!seen.unwrap().current());drop(client);let got=f.received();let replies=got.iter().filter(|v|v["id"]==77&&v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["result"],expected);
 }
}
#[test]fn legacy_before_turn_ack_is_deferred_and_another_conversation_never_reaches_the_broker(){
 for mode in["legacy-before-turn","legacy-foreign"]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let mut count=0;
  client.answer_native(options(),|_,_|panic!("no tool"),|method,params,ticket|{count+=1;assert_eq!(params["conversationId"],"owned-thread");assert!(ticket.unwrap().identity().turn_id.is_none());crate::pet_native_choices::codex_response(method,params,None)},|_|{},|_|{}).unwrap();assert_eq!(count,1);drop(client);
  if mode=="legacy-foreign"{let got=f.received();assert!(got.iter().any(|v|v["id"]==76&&v.get("error").is_some()));}
 }
}
#[test]fn mcp_nullable_or_absent_turn_is_captured_without_accepting_an_explicit_foreign_turn(){
 for mode in["mcp-null","mcp-missing","mcp-wrong-turn"]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let mut count=0;
  client.answer_native(options(),|_,_|panic!("no tool"),|method,_,ticket|{count+=1;assert_eq!(method,"mcpServer/elicitation/request");let ticket=ticket.unwrap();assert_eq!(ticket.identity().session_id,"owned-thread");assert_eq!(ticket.identity().turn_id.as_deref(),if mode=="mcp-wrong-turn"{Some("turn-1")}else{None});Some(json!({"action":"decline","content":null}))},|_|{},|_|{}).unwrap();assert_eq!(count,1);drop(client);
  let got=f.received();assert_eq!(got.iter().filter(|v|v["id"]==77&&v.get("result").is_some()).count(),1);if mode=="mcp-wrong-turn"{assert!(got.iter().any(|v|v["id"]==76&&v.get("error").is_some()));}
 }
}
#[test]fn legacy_and_nullable_mcp_resolution_or_eof_prevent_late_callback_replies(){
 for mode in["legacy-resolved","legacy-disconnect","mcp-resolved","mcp-disconnect"]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let marker=f.cwd.join("human-callback-entered");let mut count=0;
  let result=client.answer_native(options(),|_,_|panic!("no tool"),|_,_,ticket|{count+=1;let ticket=ticket.unwrap();assert!(ticket.identity().turn_id.is_none());std::fs::write(&marker,"ready").unwrap();let until=Instant::now()+Duration::from_secs(3);while ticket.current(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(2));}Some(json!({"delayed":"must never be sent"}))},|_|{},|_|{});
  assert_eq!(count,1);assert_eq!(result.is_ok(),mode.ends_with("resolved"));drop(client);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("method").is_none()));
 }
}

#[test]fn typed_mcp_form_over_real_pipe_preserves_types_values_and_pre_ack_queueing(){
 for mode in["mcp-typed","mcp-typed-before-turn"]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(5),false).unwrap();let mut count=0;
  client.answer_native(options(),|_,_|panic!("no tool"),|_,params,source|{
   count+=1;let form=crate::pet_mcp_form::Form::capture(source.unwrap(),"pipe-form",params).unwrap();let questions=form.questions();assert!(!questions.to_string().contains("PRIVATE_DEFAULT"));let mut answers=serde_json::Map::new();
   for q in questions.as_array().unwrap(){let value=match q["question"].as_str().unwrap(){"count"=>json!(["2"]),"flag"=>json!([q["options"][1]["value"]]),"pick"=>json!([q["options"][0]["value"]]),"tags"=>json!([q["options"][1]["value"],q["options"][0]["value"]]),"secret"=>json!(["supplied secret"]),_=>unreachable!()};answers.insert(q["id"].as_str().unwrap().into(),value);}
   let mut human=json!({"allow":true,"answers":answers});let field=questions[0]["id"].as_str().unwrap();human["answers"][field]=json!(["true"]);assert!(form.resolve(&human).is_err());assert!(source.unwrap().current());human["answers"][field]=json!(["2"]);Some(form.resolve(&human).unwrap())
  },|_|{},|_|{}).unwrap();assert_eq!(count,1);drop(client);let got=f.received();let replies=got.iter().filter(|v|v["id"]==77&&v.get("result").is_some()).collect::<Vec<_>>();assert_eq!(replies.len(),1);assert_eq!(replies[0]["result"],json!({"action":"accept","content":{"count":2,"flag":false,"pick":"wire-value","tags":["b","a"],"secret":"supplied secret"},"_meta":null}));
 }
}
#[test]fn a_resolved_or_disconnected_typed_form_never_sends_its_late_native_payload(){
 for mode in["mcp-typed-resolved","mcp-typed-disconnect"]{
  let f=Fixture::new(mode);let mut client=Client::start(&f.exe,&f.cwd,Duration::from_secs(6),false).unwrap();let marker=f.cwd.join("human-callback-entered");let mut seen=0;
  let result=client.answer_native(options(),|_,_|panic!("no tool"),|_,params,source|{seen+=1;let source=source.unwrap();let form=crate::pet_mcp_form::Form::capture(source,"pending-form",params).unwrap();std::fs::write(&marker,"ready").unwrap();let until=Instant::now()+Duration::from_secs(3);while source.current(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(2));}assert!(form.resolve(&json!({"allow":false})).is_err());Some(json!({"action":"accept","content":{"must":"never send"}}))},|_|{},|_|{});assert_eq!(seen,1);assert_eq!(result.is_ok(),mode.ends_with("resolved"));drop(client);assert!(!f.received().iter().any(|v|v["id"]==77&&v.get("method").is_none()));
 }
}
