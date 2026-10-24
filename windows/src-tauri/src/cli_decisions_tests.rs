use super::*;
use std::thread;
type Requests=Arc<Mutex<HashMap<String,PendingRequest>>>;
fn fixture(provider:&str,external:bool)->(Chat,Value,Arc<AtomicBool>){
    let chat=Chat::default();let cancel=Arc::new(AtomicBool::new(false));
    let session=if external{"019a734b-0000-7000-8000-000000000001".into()}else{chat.begin_cli(provider,cancel.clone()).unwrap();chat.conversation_id(provider).unwrap()};
    let mut t=json!({"petId":crate::pet_config::pet_id(provider).unwrap(),"provider":provider,"origin":if external{"external"}else{"mochi"},"sessionId":session});if !external{t["conversationId"]=json!(session);}
    (chat,t,cancel)
}
fn insert(requests:&Requests,id:&str,t:&Value,cancel:Arc<AtomicBool>,expires:Instant,questions:Option<Value>)->mpsc::Receiver<Value>{
    let(tx,rx)=mpsc::channel();let mut request=json!({"id":id,"target":t,"kind":if questions.is_some(){"question"}else{"permission"},"approvalScope":"once"});if let Some(q)=questions{request["questions"]=q;}
    register(requests,id.into(),PendingRequest{native_form:None,choices:None,reply:tx,request,policy:RequestPolicy::Native{cancel,source:None},expires_at:expires}).unwrap();rx
}
fn until()->Instant{Instant::now()+Duration::from_secs(30)}
#[test]
fn decisions_for_all_native_adapters_and_service_owners_require_the_exact_original_target(){
    for provider in ["codex","copilot","antigravity","integration_spotify","integration_notion"]{
        let(chat,t,cancel)=fixture(provider,false);let requests=Arc::default();let rx=insert(&requests,"request",&t,cancel,until(),None);
        assert!(answer_in(&chat,&requests,"request",json!({"allow":true}),None).is_err());
        for key in ["petId","provider","sessionId","conversationId","origin"]{let mut wrong=t.clone();wrong[key]=json!("other");assert!(answer_in(&chat,&requests,"request",json!({"allow":true}),Some(wrong)).is_err());}
        assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));assert!(requests.lock().unwrap().contains_key("request"));
        answer_in(&chat,&requests,"request",json!({"allow":false,"answers":{}}),Some(t)).unwrap();assert_eq!(rx.recv().unwrap()["allow"],false);assert!(requests.lock().unwrap().is_empty());
    }
}
#[test]
fn terminal_resolution_or_source_disconnect_invalidates_the_other_surfaces_before_cleanup(){
    for provider in ["codex","copilot","antigravity"]{let(chat,t,cancel)=fixture(provider,true);let requests=Arc::default();let rx=insert(&requests,"original",&t,cancel.clone(),until(),None);cancel.store(true,Ordering::Release);
        assert!(answer_in(&chat,&requests,"original",json!({"allow":true}),Some(t)).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));assert!(!live(requests.lock().unwrap().get("original").unwrap()));
    }
}
#[test]
fn stop_and_a_new_turn_in_the_same_conversation_cannot_adopt_the_old_request(){
    let chat=Chat::default();let cancel=Arc::new(AtomicBool::new(false));let turn=chat.begin_cli("codex",cancel.clone()).unwrap();let lease=chat.lease(&turn);let cid=chat.conversation_id("codex").unwrap();let t=json!({"petId":"agent_codex","provider":"codex","origin":"mochi","sessionId":cid,"conversationId":cid});let requests=Arc::default();let rx=insert(&requests,"old-turn",&t,cancel.clone(),until(),None);chat.cancel_provider("codex");assert!(cancel.load(Ordering::Acquire));drop(lease);
    let new_cancel=Arc::new(AtomicBool::new(false));chat.begin_cli("codex",new_cancel).unwrap();assert_eq!(chat.conversation_id("codex").unwrap(),t["sessionId"].as_str().unwrap());
    assert!(answer_in(&chat,&requests,"old-turn",json!({"allow":true}),Some(t)).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));assert!(requests.lock().unwrap().contains_key("old-turn"));
}
#[test]
fn an_ended_mochi_conversation_refuses_an_answer_even_if_the_native_flag_is_not_yet_cancelled(){
    let(chat,t,_)=fixture("copilot",false);let independent=Arc::new(AtomicBool::new(false));let requests=Arc::default();let rx=insert(&requests,"ended",&t,independent.clone(),until(),None);chat.cancel_provider("copilot");assert!(!independent.load(Ordering::Acquire));
    assert!(answer_in(&chat,&requests,"ended",json!({"allow":true}),Some(t)).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));
}
#[test]
fn expiry_is_checked_at_consumption_not_only_by_the_waiting_native_callback(){
    let(chat,t,cancel)=fixture("antigravity",true);let requests=Arc::default();let rx=insert(&requests,"expired",&t,cancel,until(),None);requests.lock().unwrap().get_mut("expired").unwrap().expires_at=Instant::now()-Duration::from_millis(1);
    assert!(answer_in(&chat,&requests,"expired",json!({"allow":true}),Some(t)).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));assert!(requests.lock().unwrap().contains_key("expired"));
}
#[test]
fn concurrent_human_surfaces_produce_at_most_one_native_answer(){
    let(chat,t,cancel)=fixture("copilot",true);let requests=Arc::default();let rx=insert(&requests,"single",&t,cancel,until(),None);let barrier=Arc::new(std::sync::Barrier::new(3));let mut workers=vec![];
    for allow in [false,true]{let(chat,t,requests,barrier)=(chat.clone(),t.clone(),requests.clone(),barrier.clone());workers.push(thread::spawn(move||{barrier.wait();answer_in(&chat,&requests,"single",json!({"allow":allow,"answers":{}}),Some(t))}));}barrier.wait();
    let successful=workers.into_iter().map(|worker|worker.join().unwrap().is_ok()as usize).sum::<usize>();assert_eq!(successful,1);assert!(rx.recv().unwrap()["allow"].is_boolean());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Disconnected)));assert!(requests.lock().unwrap().is_empty());
}
#[test]
fn native_permission_payloads_cannot_supply_broader_scope_or_unrelated_answers(){
    let(chat,t,cancel)=fixture("codex",true);let requests=Arc::default();let rx=insert(&requests,"native",&t,cancel,until(),None);
    for wrong in [json!({"allow":"true"}),json!({"decision":"acceptForSession"}),json!({"allow":true,"scope":"session"}),json!({"allow":true,"answers":{"unrelated":["always"]}}),json!({"allow":true,"answers":[]})]{assert!(answer_in(&chat,&requests,"native",wrong,Some(t.clone())).is_err());}
    assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));answer_in(&chat,&requests,"native",json!({"allow":true,"answers":{}}),Some(t)).unwrap();assert_eq!(rx.recv().unwrap(),json!({"allow":true,"answers":{}}));
}
#[test]
fn native_questions_preserve_text_but_reject_cross_request_non_text_and_false_positive_denials(){
    let(chat,t,cancel)=fixture("codex",true);let requests=Arc::default();let rx=insert(&requests,"question",&t,cancel,until(),Some(json!([{"id":"destination","question":"Where?"}])));
    for wrong in [json!({"allow":true,"answers":{"other":["value"]}}),json!({"allow":true,"answers":{"destination":[true]}}),json!({"allow":false,"answers":{"destination":["must not forward after deny"]}}),json!({"allow":true,"answers":{"destination":"value"}}),json!({"allow":true,"answers":{"destination":["x".repeat(8193)]}})]{assert!(answer_in(&chat,&requests,"question",wrong,Some(t.clone())).is_err());}
    let answer=json!({"allow":true,"answers":{"destination":["/fixture/project\nSecond line"]}});answer_in(&chat,&requests,"question",answer.clone(),Some(t)).unwrap();assert_eq!(rx.recv().unwrap(),answer);
}
#[test]
fn duplicate_native_question_ids_are_refused_before_publication(){
    let(_chat,t,cancel)=fixture("codex",true);let requests=Mutex::new(HashMap::new());let(tx,_)=mpsc::channel();let request=json!({"id":"duplicate","target":t,"kind":"question","questions":[{"id":"q"},{"id":"q"}]});assert!(register(&requests,"duplicate".into(),PendingRequest{native_form:None,choices:None,reply:tx,request,policy:RequestPolicy::Native{cancel,source:None},expires_at:until()}).is_err());assert!(requests.lock().unwrap().is_empty());
}
#[test]
fn boot_identity_prevents_restart_collisions_and_counter_exhaustion_never_wraps(){
    let a=AtomicU64::new(1);let b=AtomicU64::new(1);let first=id_from_boot(&"a".repeat(64),false,&a).unwrap();let second=id_from_boot(&"b".repeat(64),false,&b).unwrap();assert_ne!(first,second);assert_ne!(first,id_from_boot(&"a".repeat(64),true,&AtomicU64::new(1)).unwrap());
    let exhausted=AtomicU64::new(u64::MAX);assert!(id_from_boot(&"a".repeat(64),false,&exhausted).is_none());assert!(id_from_boot(&"a".repeat(64),false,&exhausted).is_none());assert_eq!(exhausted.load(Ordering::Acquire),u64::MAX);
}
#[test]
fn contention_preserves_the_original_request_for_a_later_explicit_answer(){
    let(chat,t,cancel)=fixture("codex",false);let requests=Arc::default();let rx=insert(&requests,"contended",&t,cancel,until(),None);let lock=requests.lock().unwrap();assert!(answer_in(&chat,&requests,"contended",json!({"allow":true}),Some(t.clone())).is_err());drop(lock);
    let result=chat.with_live_conversation("codex",t["sessionId"].as_str().unwrap(),||answer_in(&chat,&requests,"contended",json!({"allow":true}),Some(t.clone()))).unwrap().unwrap();assert!(result.is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));
    answer_in(&chat,&requests,"contended",json!({"allow":false}),Some(t)).unwrap();assert_eq!(rx.recv().unwrap()["allow"],false);
}
#[test]
fn native_pending_capacity_does_not_replace_earlier_requests(){
    let(_chat,t,cancel)=fixture("codex",true);let requests=Arc::default();let mut receivers=vec![];for n in 0..MAX_PENDING{receivers.push(insert(&requests,&format!("pending-{n}"),&t,cancel.clone(),until(),None));}
    let(tx,_)=mpsc::channel();let request=json!({"id":"overflow","target":t,"kind":"permission"});assert!(register(&requests,"overflow".into(),PendingRequest{native_form:None,choices:None,reply:tx,request,policy:RequestPolicy::Native{cancel,source:None},expires_at:until()}).is_err());assert_eq!(requests.lock().unwrap().len(),MAX_PENDING);assert!(receivers.iter().all(|rx|matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty))));
}

#[test]
fn a_resolved_native_request_cannot_be_answered_while_its_mochi_turn_is_still_active(){
    let(chat,t,cancel)=fixture("codex",false);let(source,native_cancel)=crate::pet_native_decisions::Ticket::fixture();let(tx,rx)=mpsc::channel();let request=PendingRequest{native_form:None,choices:None,reply:tx,request:json!({"id":"bound","kind":"permission","target":t}),policy:RequestPolicy::Native{cancel,source:Some(source)},expires_at:until()};let requests=Mutex::new(HashMap::new());register(&requests,"bound".into(),request).unwrap();native_cancel.store(true,Ordering::Release);
    assert!(answer_in(&chat,&requests,"bound",json!({"allow":true}),Some(t)).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));assert!(requests.lock().unwrap().contains_key("bound"));
}

fn choice_request(id:&str,t:&Value,cancel:Arc<AtomicBool>,source:crate::pet_native_decisions::Ticket,params:&Value)->(PendingRequest,mpsc::Receiver<Value>){
 let choices=crate::pet_native_choices::Choices::capture(&source,id,params).unwrap().unwrap();let(tx,rx)=mpsc::channel();let request=json!({"id":id,"kind":"permission","target":t,"nativeChoices":choices.public()});(PendingRequest{native_form:None,choices:Some(choices),reply:tx,request,policy:RequestPolicy::Native{cancel,source:Some(source)},expires_at:until()},rx)
}

fn form_request(id:&str,t:&Value,cancel:Arc<AtomicBool>)->(crate::pet_native_decisions::Endpoint,PendingRequest,mpsc::Receiver<Value>){
 use crate::{pet_config::Adapter,pet_native_decisions::{Capture,Endpoint,PolicyBinding,Transport}};
 let p=json!({"threadId":"native-thread","turnId":null,"serverName":"fixture","mode":"openai/form","message":"Synthetic form","requestedSchema":{"type":"object","properties":{"count":{"type":"integer","minimum":1,"maximum":3},"secret":{"type":"string","format":"password","default":"PRIVATE_DEFAULT"}},"required":["count"]}});
 let endpoint=Endpoint::new(Adapter::Codex,Transport::ManagedPipe).unwrap();let source=endpoint.capture(Capture{request_id:Some(&json!(7)),session_id:"native-thread",turn_id:None,prompt_id:None,method:"mcpServer/elicitation/request",arguments:&p,policy:PolicyBinding::unknown(),cancel:None}).unwrap();
 let form=crate::pet_mcp_form::Form::capture(&source,id,&p).unwrap();let request=json!({"id":id,"kind":"question","target":t,"nativeForm":"mcp","questions":form.questions(),"detail":form.detail()});let(tx,rx)=mpsc::channel();
 (endpoint,PendingRequest{native_form:Some(form),choices:None,reply:tx,request,policy:RequestPolicy::Native{cancel,source:Some(source)},expires_at:until()},rx)
}
#[test]fn typed_form_validation_precedes_consumption_and_native_content_is_materialized_once(){
 let(chat,t,cancel)=fixture("codex",false);let(_endpoint,request,rx)=form_request("mcp-review",&t,cancel);let count=request.request["questions"][0]["id"].as_str().unwrap().to_string();let secret=request.request["questions"][1]["id"].as_str().unwrap().to_string();assert!(!request.request.to_string().contains("PRIVATE_DEFAULT"));let requests=Mutex::new(HashMap::new());register(&requests,"mcp-review".into(),request).unwrap();
 for bad in[json!({"allow":true,"answers":{}}),json!({"allow":true,"answers":{(count.clone()):["true"]}}),json!({"allow":true,"answers":{(count.clone()):["4"]}}),json!({"allow":true,"nativeResponse":{"action":"accept","content":{"count":2}}})]{assert!(answer_in(&chat,&requests,"mcp-review",bad,Some(t.clone())).is_err());assert!(requests.lock().unwrap().contains_key("mcp-review"));assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));}
 let good=json!({"allow":true,"answers":{(count.clone()):["2"],(secret.clone()):["user secret"]}});answer_in(&chat,&requests,"mcp-review",good.clone(),Some(t.clone())).unwrap();let got=rx.recv().unwrap();assert_eq!(got["nativeResponse"],json!({"action":"accept","content":{"count":2,"secret":"user secret"},"_meta":null}));assert!(answer_in(&chat,&requests,"mcp-review",good,Some(t)).is_err());
}
#[test]fn typed_form_denial_sends_no_content_and_optional_omission_does_not_apply_defaults(){
 for allow in[true,false]{let(chat,t,cancel)=fixture("codex",false);let(_endpoint,request,rx)=form_request("mcp-decision",&t,cancel);let id=request.request["questions"][0]["id"].as_str().unwrap().to_string();let requests=Mutex::new(HashMap::new());register(&requests,"mcp-decision".into(),request).unwrap();let answer=if allow{json!({"allow":true,"answers":{id:["1"]}})}else{json!({"allow":false,"answers":{}})};answer_in(&chat,&requests,"mcp-decision",answer,Some(t)).unwrap();let response=rx.recv().unwrap()["nativeResponse"].clone();assert_eq!(response["content"],if allow{json!({"count":1})}else{Value::Null});}
}
#[test]fn typed_form_rejects_wrong_target_or_resolved_source_without_consuming(){
 let(chat,t,cancel)=fixture("codex",true);let(_endpoint,request,rx)=form_request("mcp-owner",&t,cancel);let source=match &request.policy{RequestPolicy::Native{source:Some(s),..}=>s.clone(),_=>unreachable!()};let requests=Mutex::new(HashMap::new());register(&requests,"mcp-owner".into(),request).unwrap();let mut wrong=t.clone();wrong["sessionId"]=json!("019a734b-0000-7000-8000-000000000002");assert!(answer_in(&chat,&requests,"mcp-owner",json!({"allow":false}),Some(wrong)).is_err());source.cancel();assert!(answer_in(&chat,&requests,"mcp-owner",json!({"allow":false}),Some(t)).is_err());assert!(requests.lock().unwrap().contains_key("mcp-owner"));assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));
}
#[test]fn typed_form_registration_requires_its_exact_source_and_public_presentation(){
 for tamper in["source","questions","detail","nativeForm"]{let(_chat,t,cancel)=fixture("codex",true);let(_endpoint,mut request,_rx)=form_request("mcp-public",&t,cancel.clone());let(_other,other,_)=form_request("other",&t,cancel);
  match tamper{"source"=>request.policy=other.policy,"questions"=>request.request["questions"][0]["required"]=json!(false),"detail"=>request.request["detail"]=json!("substituted"),_=>request.request["nativeForm"]=json!("other")};assert!(register(&Mutex::new(HashMap::new()),"mcp-public".into(),request).is_err());
 }
}
#[test]fn an_explicit_native_choice_is_materialized_by_rust_and_consumed_once_for_the_original_owner(){
 for provider in["codex","integration_spotify"]{let(chat,t,cancel)=fixture(provider,false);let(source,_)=crate::pet_native_decisions::Ticket::fixture();let(request,rx)=choice_request("bound-choice",&t,cancel,source,&json!({}));let id=request.request["nativeChoices"].as_array().unwrap().iter().find(|v|v["scope"]=="session").unwrap()["id"].clone();let requests=Mutex::new(HashMap::new());register(&requests,"bound-choice".into(),request).unwrap();let answer=json!({"allow":true,"nativeChoice":id});answer_in(&chat,&requests,"bound-choice",answer.clone(),Some(t.clone())).unwrap();assert_eq!(rx.recv().unwrap()["nativeResponse"],json!({"decision":"acceptForSession"}));assert!(answer_in(&chat,&requests,"bound-choice",answer,Some(t)).is_err());}
}
#[test]fn forged_native_values_foreign_choice_ids_and_mismatched_allow_do_not_consume_a_review(){
 let(chat,t,cancel)=fixture("codex",false);let(source,_)=crate::pet_native_decisions::Ticket::fixture();let(request,rx)=choice_request("bound-choice",&t,cancel,source,&json!({}));let public=request.request["nativeChoices"].clone();let session=public.as_array().unwrap().iter().find(|v|v["scope"]=="session").unwrap()["id"].clone();let requests=Mutex::new(HashMap::new());register(&requests,"bound-choice".into(),request).unwrap();
 for wrong in[json!({"allow":true,"nativeResponse":{"decision":"acceptForSession"}}),json!({"allow":true,"nativeChoice":"other-request-choice-1"}),json!({"allow":false,"nativeChoice":session}),json!({"allow":true,"nativeChoice":session,"scope":"session"})]{assert!(answer_in(&chat,&requests,"bound-choice",wrong,Some(t.clone())).is_err());assert!(requests.lock().unwrap().contains_key("bound-choice"));assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));}
 answer_in(&chat,&requests,"bound-choice",json!({"allow":true,"nativeChoice":session}),Some(t)).unwrap();assert_eq!(rx.recv().unwrap()["nativeResponse"]["decision"],"acceptForSession");
}
#[test]fn a_boolean_cannot_consume_a_request_that_only_offers_extended_native_choices(){
 let(chat,t,cancel)=fixture("codex",false);let(source,_)=crate::pet_native_decisions::Ticket::fixture();let(request,rx)=choice_request("extended",&t,cancel,source,&json!({"availableDecisions":["acceptForSession","cancel"]}));let requests=Mutex::new(HashMap::new());register(&requests,"extended".into(),request).unwrap();for allow in[true,false]{assert!(answer_in(&chat,&requests,"extended",json!({"allow":allow}),Some(t.clone())).is_err());}assert!(requests.lock().unwrap().contains_key("extended"));assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));
}
#[test]fn a_choice_snapshot_from_another_native_capture_is_refused_before_publication(){
 let(_chat,t,cancel)=fixture("codex",true);let(a,_)=crate::pet_native_decisions::Ticket::fixture();let(b,_)=crate::pet_native_decisions::Ticket::fixture();let(mut request,_)=choice_request("mixed",&t,cancel.clone(),a,&json!({}));request.policy=RequestPolicy::Native{cancel,source:Some(b)};let requests=Mutex::new(HashMap::new());assert!(register(&requests,"mixed".into(),request).is_err());assert!(requests.lock().unwrap().is_empty());
}
#[test]fn policy_revocation_leaves_other_native_reviews_and_local_once_authority_independent(){
 let(chat,t,cancel)=fixture("codex",false);let(source,_)=crate::pet_native_decisions::Ticket::fixture();let(request,rx)=choice_request("revoked",&t,cancel,source.clone(),&json!({}));let id=request.request["nativeChoices"][1]["id"].clone();let requests=Mutex::new(HashMap::new());register(&requests,"revoked".into(),request).unwrap();source.invalidate_policy();assert!(answer_in(&chat,&requests,"revoked",json!({"allow":true,"nativeChoice":id}),Some(t.clone())).is_err());assert!(matches!(rx.try_recv(),Err(mpsc::TryRecvError::Empty)));
 let(tx,_)=mpsc::channel();let local=PendingRequest{native_form:None,choices:None,reply:tx,request:json!({"kind":"permission","target":t}),policy:RequestPolicy::LocalOnce{deadline:until(),cancel:Arc::default()},expires_at:until()};assert!(validate_decision(&local,&json!({"allow":true,"nativeChoice":"revoked-choice-1"}),Some(&t)).is_err());
}
