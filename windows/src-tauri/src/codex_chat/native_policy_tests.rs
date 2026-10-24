use super::*;
use serde_json::json;
use crate::session_chat::rpc::native_settings::{Cache,ApprovalPolicy,SandboxPolicy};
const A:&str="019a734b-0000-7000-8000-000000000001";
const B:&str="019a734b-0000-7000-8000-000000000002";
fn result(thread:&str)->Value{json!({"thread":{"id":thread},"approvalPolicy":"never","approvalsReviewer":"user","sandbox":{"type":"readOnly","networkAccess":false},"cwd":"/fixture/repo","activePermissionProfile":{"id":"native-safe","extends":null},"model":"PRIVATE MODEL","developerInstructions":"PRIVATE INSTRUCTIONS"})}
fn event(thread:&str)->Value{let mut settings=result(thread);settings["sandboxPolicy"]=settings["sandbox"].take();json!({"method":"thread/settings/updated","params":{"threadId":thread,"threadSettings":settings}})}
fn seed(r:&Requests,rpc:i64,thread:&str){r.expect_handshake(&json!(rpc),"thread/start",&json!({})).unwrap();r.observe(json!({"id":rpc,"result":result(thread)}));}
fn approval(r:&Requests,id:i64,thread:&str)->Ticket{r.observe(json!({"id":id,"method":"item/commandExecution/requestApproval","params":{"threadId":thread,"turnId":"turn-1","command":"PRIVATE COMMAND"}})).ticket.unwrap()}
#[test]
fn handshake_policy_is_observed_from_native_fields_without_equating_never_with_full_access(){
    let mut cache=Cache::default();assert_eq!(cache.response(None,&result(A),100),Some(A.into()));let o=cache.snapshot(A,100,30);assert!(!o.stale);let s=o.last_observed.as_ref().unwrap();assert_eq!(s.approval_policy,ApprovalPolicy::Never);assert_eq!(s.sandbox_policy,SandboxPolicy::ReadOnly{network_access:false});assert_eq!(s.profile.as_ref().unwrap().id,"native-safe");
    let raw=format!("{o:?}");assert!(!raw.contains("PRIVATE MODEL")&&!raw.contains("PRIVATE INSTRUCTIONS"));
}
#[test]
fn uncorrelated_or_string_instead_of_numeric_rpc_response_cannot_publish_native_policy(){
    let r=Requests::with_endpoint("one".into());r.observe(json!({"id":1,"result":result(A)}));assert!(r.policy_snapshot(A).hash.is_none());r.expect_handshake(&json!(1),"thread/start",&json!({})).unwrap();r.observe(json!({"id":"1","result":result(A)}));assert!(r.policy_snapshot(A).hash.is_none());r.observe(json!({"id":1,"result":result(A)}));assert!(r.policy_snapshot(A).hash.is_some());assert_eq!(r.policy_snapshot(A).source,Some("threadResponse"));
}
#[test]
fn native_response_then_notification_keeps_the_newest_received_policy_not_a_delayed_consumer_copy(){
    let r=Requests::with_endpoint("one".into());seed(&r,1,A);let old=r.policy_snapshot(A);let mut updated=event(A);updated["params"]["threadSettings"]["activePermissionProfile"]["id"]=json!("new-profile");r.observe(updated);let new=r.policy_snapshot(A);assert_ne!(new.hash,old.hash);assert!(new.sequence>old.sequence);assert_eq!(new.source,Some("threadNotification"));let ticket=approval(&r,77,A);assert_eq!(ticket.identity().native_policy.hash,new.hash);let raw=serde_json::to_string(ticket.identity()).unwrap();assert!(!raw.contains("PRIVATE")&&!raw.contains("/fixture/repo"));
}
#[test]
fn changed_native_policy_invalidates_only_its_own_thread_and_terminal_resolution_wins_over_local_refusal(){
    let r=Requests::with_endpoint("one".into());seed(&r,1,A);seed(&r,2,B);let a=approval(&r,77,A);let b=approval(&r,78,B);let mut changed=event(A);changed["params"]["threadSettings"]["sandboxPolicy"]["networkAccess"]=json!(true);r.observe(changed);assert!(!a.current()&&a.policy_invalidated());assert!(b.current());r.observe(json!({"method":"serverRequest/resolved","params":{"threadId":A,"requestId":77}}));assert!(!a.current()&&!a.policy_invalidated());
}
#[test]
fn repeated_policy_and_private_model_or_instruction_changes_do_not_revoke_the_reviewed_permissions(){
    let r=Requests::with_endpoint("one".into());seed(&r,1,A);let a=approval(&r,77,A);let mut unchanged=event(A);unchanged["params"]["threadSettings"]["model"]=json!("DIFFERENT PRIVATE MODEL");unchanged["params"]["threadSettings"]["developerInstructions"]=json!("DIFFERENT PRIVATE INSTRUCTIONS");r.observe(unchanged);assert!(a.current());assert_eq!(a.identity().native_policy.hash,r.policy_snapshot(A).hash);
}
#[test]
fn invalid_or_unknown_native_policy_is_not_replaced_by_the_last_successful_projection(){
    let r=Requests::with_endpoint("one".into());seed(&r,1,A);let a=approval(&r,77,A);let mut invalid=event(A);invalid["params"]["threadSettings"]["approvalsReviewer"]=json!("future-reviewer");r.observe(invalid);assert!(!a.current()&&a.policy_invalidated());assert_eq!(r.policy_snapshot(A).coverage,"notObserved");assert!(r.policy_snapshot(A).hash.is_none());
}
#[test]
fn closed_thread_retires_its_review_without_sending_a_new_answer_to_a_closed_native_request(){
    let r=Requests::with_endpoint("one".into());seed(&r,1,A);let a=approval(&r,77,A);r.observe(json!({"method":"thread/closed","params":{"threadId":A}}));assert!(!a.current()&&!a.policy_invalidated());assert!(r.policy_snapshot(A).hash.is_none());
}
#[test]
fn resume_to_another_thread_or_missing_native_fields_stays_unknown_without_manufacturing_a_policy(){
    let mut c=Cache::default();c.response(None,&result(A),100);assert!(c.response(Some(A),&result(B),101).is_none());assert!(c.snapshot(A,101,30).stale);
    let r=Requests::with_endpoint("one".into());r.expect_handshake(&json!(1),"thread/start",&json!({})).unwrap();r.observe(json!({"id":1,"result":{"thread":{"id":A}}}));let a=approval(&r,77,A);assert!(a.current());assert_eq!(a.identity().native_policy.coverage,"notObserved");assert!(a.identity().native_policy.hash.is_none());
}
#[test]
fn fingerprints_exclude_private_fields_but_keep_native_approval_and_sandbox_distinct(){
    let mut a=Cache::default();let mut b=Cache::default();a.response(None,&result(A),100);let mut changed=result(A);changed["model"]=json!("DIFFERENT MODEL");changed["developerInstructions"]=json!("DIFFERENT INSTRUCTIONS");b.response(None,&changed,100);let hash=|c:&Cache|c.snapshot(A,100,30).last_observed.unwrap().fingerprint().unwrap();assert_eq!(hash(&a),hash(&b));changed["sandbox"]=json!({"type":"dangerFullAccess"});b.response(None,&changed,100);assert_ne!(hash(&a),hash(&b));
}
