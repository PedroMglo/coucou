use super::*;
use crate::pet_runtime::plans;
pub(super) fn setup_plan()->(Fixture,Authority,Record,Pending,Owned,Chat){
    setup_plan_custom(vec![StepInput{description:"Propose pause".into(),postcondition:"Playback independently observed paused".into(),operation:Some(plans::OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:json!({"action":"pause"})})}],2,Arc::new(|_,_,_|Validity::Current))
}
pub(super) fn setup_plan_custom(steps:Vec<StepInput>,tools:u16,fence:Fence)->(Fixture,Authority,Record,Pending,Owned,Chat){
    setup_plan_criteria(steps,tools,fence,vec!["Verified original result".into()])
}
pub(super) fn setup_plan_criteria(steps:Vec<StepInput>,tools:u16,fence:Fence,criteria:Vec<String>)->(Fixture,Authority,Record,Pending,Owned,Chat){
    let f=Fixture::new();let a=Authority::ensure(&root(&f)).unwrap();let mut g=f.goal_named_criteria("ollama","goal-ollama",criteria);let now=crate::assistant_memory::now()-100;
    g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"task".into(),Action::ProposeTask{description:"Original task".into()},now).unwrap().record;
    g=f.store().mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"plan".into(),Action::ProposePlan{task_id:g.tasks[0].id.clone(),steps},now+1).unwrap().record;
    let mut limits=budget();limits.deadline_at=now+60_000;limits.max_tool_calls=tools;
    let approved=Review::prepare(&g,evidence(&g,now+2),limits,"queue".into(),nonce().unwrap(),now+2).unwrap().approve(&g,evidence(&g,now+3),now+3).unwrap();
    let cert=a.certify(&approved,&g,now+4).unwrap();let queued=f.store().enqueue_certified(&cert,now+5).unwrap().record;
    let main=Chat::persistent(f.path.join("live-main.json"));let c=Controller::new().unwrap();let p=c.prepare(&queued,&root(&f),&f.path.join("contexts"),&main,&evidence(&queued,now+6),"claim".into(),now+6,fence).unwrap();
    let(receipt,o)=p.claim(&f.store(),&a,&evidence(&queued,now+7),now+7).unwrap();(f,a,receipt.record,p,o.unwrap(),main)
}
#[test]
fn native_text_request_uses_reviewed_work_without_exposing_the_envelope_in_the_visible_question(){
    let(f,_a,g,_p,o,_main)=setup_plan();let(server,worker)=fixture(vec![("application/json",local_metadata().to_string()),("text/event-stream",stream("Text only",true))]);
    assert_eq!(run(o.respond_once(None,&g,&server,"answer",&crate::pet_config::defaults())).unwrap(),"Text only");
    let requests=worker.join().unwrap();assert_eq!(requests.len(),2);let body=&requests[1].1;assert!(body.get("tools").is_none());
    let user=body["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap();let data:Value=serde_json::from_str(user).unwrap();
    assert_eq!(data["intent"],g.spec.intent);assert_eq!(data["criteria"],json!(g.spec.criteria));assert_eq!(data["proposals"]["plans"][0]["steps"][0]["operation"]["arguments"]["action"],"pause");assert_eq!(data["toolExecutionAuthorized"],false);
    let visible=serde_json::to_string(&o.live.chat.conversation("ollama").unwrap().messages).unwrap();assert!(visible.contains(&g.spec.intent));assert!(!visible.contains("toolExecutionAuthorized"));assert!(!visible.contains("operationId"));
    let stored=f.store().read(&g.id,"ai_ollama").unwrap();assert_eq!(stored.schema_version,6);assert_eq!(stored.plans[0].steps[0].state,WorkState::Proposed);
    assert_eq!(ledger(&f,&g)["attempts"].as_array().unwrap().len(),1);assert_eq!(ledger(&f,&g)["attempts"][0]["kind"],"model");
}
#[test]
fn changed_reviewed_work_is_refused_before_metadata_inference_or_budget_debit(){
    let(f,_a,g,_p,o,_main)=setup_plan();let mut changed=g.clone();changed.plans[0].steps[0].operation.as_mut().unwrap().arguments["action"]=json!("play");
    let server=crate::local_chat::Server{id:"ollama",name:"No IO",url:"http://127.0.0.1:1".into(),key:None};
    assert!(run(o.respond_once(None,&changed,&server,"answer",&crate::pet_config::defaults())).unwrap_err().contains("objetivo original"));assert!(ledger(&f,&g).is_null());assert_eq!(o.workers(),0);
}
