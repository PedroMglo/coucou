use super::*;
use serde_json::json;
const A:&str="11111111-1111-1111-1111-111111111111";
const B:&str="22222222-2222-2222-2222-222222222222";
fn update(id:&str)->Value{json!({"method":"thread/settings/updated","params":{"threadId":id,"threadSettings":{"approvalPolicy":"on-request","approvalsReviewer":"user","sandboxPolicy":{"type":"workspaceWrite","writableRoots":["/fixture/repo"]},"cwd":"/fixture/repo","model":"not-exported","collaborationMode":{"settings":{"developer_instructions":"PRIVATE INSTRUCTIONS"}}}}})}
#[test]fn missing_or_invalid_identity_is_unknown_instead_of_default_permission(){
 let c=Cache::default();let s=c.snapshot(A,100,30);assert_eq!(s.reason,Some(Reason::NotObserved));assert!(s.stale);assert!(s.last_observed.is_none());
 for id in ["","latest","00000000-0000-0000-0000-000000000000","../../session"]{assert_eq!(c.snapshot(id,100,30).reason,Some(Reason::InvalidThread));}
}
#[test]fn valid_notification_retains_only_permission_projection_and_a_bounded_marker(){
 let mut c=Cache::default();let marker=c.event(&update(A),100).unwrap();let s=c.snapshot(A,100,30);
 assert!(!s.stale);assert_eq!(s.reason,None);assert_eq!(s.observed_at,Some(100));assert_eq!(s.sequence,Some(1));let settings=s.last_observed.unwrap();assert_eq!(settings.approval_policy,ApprovalPolicy::OnRequest);assert_eq!(settings.cwd,"/fixture/repo");
 let public=marker.to_string();assert_eq!(marker["method"],"mochi/threadSettingsObserved");assert_eq!(marker["params"]["threadId"],A);assert_eq!(marker["params"]["observationOnly"],true);
 assert!(!public.contains("PRIVATE"));assert!(!public.contains("approvalPolicy"));assert!(marker["params"].get("threadSettings").is_none());assert!(!format!("{settings:?}").contains("PRIVATE"));
}
#[test]fn all_reviewed_approval_modes_and_granular_defaults_are_exact(){
 for (raw,want) in [(json!("untrusted"),ApprovalPolicy::Untrusted),(json!("on-request"),ApprovalPolicy::OnRequest),(json!("never"),ApprovalPolicy::Never),(json!({"granular":{"sandbox_approval":true,"rules":false,"mcp_elicitations":true}}),ApprovalPolicy::Granular{sandbox_approval:true,rules:false,mcp_elicitations:true,request_permissions:false,skill_approval:false})]{assert_eq!(approval(&raw),Some(want));}
 for raw in [json!("always"),json!(true),json!({"granular":{"sandbox_approval":true,"rules":true}}),json!({"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true,"unknown":true}}),json!({"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true},"extra":true})]{assert!(approval(&raw).is_none());}
}
#[test]fn sandbox_modes_keep_scoped_native_values_without_inventing_full_access(){
 assert_eq!(sandbox(&json!({"type":"dangerFullAccess"})),Some(SandboxPolicy::DangerFullAccess));
 assert_eq!(sandbox(&json!({"type":"readOnly"})),Some(SandboxPolicy::ReadOnly{network_access:false}));
 assert_eq!(sandbox(&json!({"type":"externalSandbox","networkAccess":"enabled"})),Some(SandboxPolicy::ExternalSandbox{network_access:"enabled".into()}));
 assert_eq!(sandbox(&json!({"type":"externalSandbox"})),Some(SandboxPolicy::ExternalSandbox{network_access:"restricted".into()}));
 assert!(sandbox(&json!({"type":"workspaceWrite","writableRoots":["relative"]})).is_none());
 assert!(sandbox(&json!({"type":"dangerFullAccess","unreviewed":true})).is_none());
 assert!(sandbox(&json!({"type":"readOnly","networkAccess":"true"})).is_none());
 assert!(sandbox(&json!({"type":"externalSandbox","networkAccess":"yes"})).is_none());
}
#[test]fn root_lists_profile_names_and_paths_are_bounded(){
 assert!(sandbox(&json!({"type":"workspaceWrite","writableRoots":vec!["/fixture";33]})).is_none());
 assert!(sandbox(&json!({"type":"workspaceWrite","writableRoots":vec![format!("/{}","a".repeat(2048));9]})).is_none());
 assert!(path(&json!("/fixture/../other")).is_none());assert!(path(&json!("/fixture\nother")).is_none());
 let mut event=update(A);event["params"]["threadSettings"]["activePermissionProfile"]=json!({"id":":workspace","extends":"parent"});let parsed=settings(&event["params"]["threadSettings"]).unwrap();assert_eq!(parsed.profile,Some(Profile{id:":workspace".into(),extends:Some("parent".into())}));
 event["params"]["threadSettings"]["activePermissionProfile"]["id"]=json!("x".repeat(257));assert!(settings(&event["params"]["threadSettings"]).is_none());
}
#[test]fn invalid_targeted_refresh_preserves_last_evidence_time_and_marks_unknown(){
 let mut c=Cache::default();c.event(&update(A),100);c.event(&update(B),100);let mut bad=update(A);bad["params"]["threadSettings"]["approvalPolicy"]=json!("future-mode");c.event(&bad,105);
 let a=c.snapshot(A,105,30);assert!(a.stale);assert_eq!(a.reason,Some(Reason::InvalidNotification));assert_eq!(a.observed_at,Some(100));assert_eq!(a.sequence,Some(1));assert_eq!(a.last_observed.unwrap().approval_policy,ApprovalPolicy::OnRequest);assert!(!c.snapshot(B,105,30).stale);
 c.event(&update(A),106);assert!(!c.snapshot(A,106,30).stale);
}
#[test]fn unaddressable_settings_event_invalidates_known_rows_instead_of_selecting_latest(){
 let mut c=Cache::default();c.event(&update(A),100);c.event(&update(B),100);let marker=c.event(&update("invalid"),101).unwrap();assert_eq!(marker["params"]["unknown"],true);
 for id in [A,B]{assert_eq!(c.snapshot(id,101,30).reason,Some(Reason::InvalidNotification));}
}
#[test]fn disconnect_is_terminal_for_this_client_and_a_new_client_starts_empty(){
 let mut old=Cache::default();old.event(&update(A),100);old.disconnect();assert!(old.event(&update(A),101).is_none());let s=old.snapshot(A,101,30);assert_eq!(s.reason,Some(Reason::Disconnected));assert!(s.last_observed.is_some());assert_eq!(s.observed_at,Some(100));
 let new=Cache::default();assert_eq!(new.snapshot(A,101,30).reason,Some(Reason::NotObserved));assert!(new.snapshot(A,101,30).last_observed.is_none());
}
#[test]fn different_client_caches_and_threads_never_share_policy_observations(){
 let mut a=Cache::default();let mut b=Cache::default();a.event(&update(A),100);let mut never=update(A);never["params"]["threadSettings"]["approvalPolicy"]=json!("never");b.event(&never,100);
 assert_eq!(a.snapshot(A,100,30).last_observed.unwrap().approval_policy,ApprovalPolicy::OnRequest);assert_eq!(b.snapshot(A,100,30).last_observed.unwrap().approval_policy,ApprovalPolicy::Never);assert!(a.snapshot(B,100,30).last_observed.is_none());
}
#[test]fn stale_and_future_clock_readings_never_renew_measurement_time(){
 let mut c=Cache::default();c.event(&update(A),100);assert!(!c.snapshot(A,130,30).stale);for now in [99,131]{let s=c.snapshot(A,now,30);assert_eq!(s.reason,Some(Reason::Expired));assert_eq!(s.observed_at,Some(100));}
 assert_eq!(c.snapshot(A,100,30).observed_at,Some(100));
}
#[test]fn closed_thread_cannot_be_revived_by_a_late_settings_event_without_native_restart(){
 let mut c=Cache::default();c.event(&update(A),100);c.event(&json!({"method":"thread/closed","params":{"threadId":A}}),101);c.event(&update(A),102);assert_eq!(c.snapshot(A,102,30).reason,Some(Reason::ThreadClosed));assert_eq!(c.snapshot(A,102,30).observed_at,Some(100));
 c.event(&json!({"method":"thread/started","params":{"thread":{"id":A}}}),103);assert_eq!(c.snapshot(A,103,30).reason,Some(Reason::NotObserved));assert!(c.snapshot(A,103,30).last_observed.is_none());c.event(&update(A),104);assert!(!c.snapshot(A,104,30).stale);
}
#[test]fn registry_capacity_is_explicit_and_never_evicts_another_thread_as_a_default(){
 let mut c=Cache::default();for n in 1..=MAX_THREADS{let id=format!("00000000-0000-0000-0000-{n:012x}");c.event(&update(&id),100);}
 assert_eq!(c.entries.len(),MAX_THREADS);c.event(&update(A),101);assert_eq!(c.entries.len(),MAX_THREADS);assert_eq!(c.snapshot(A,101,30).reason,Some(Reason::Capacity));assert!(c.snapshot(A,101,30).last_observed.is_none());
}
#[test]fn rpc_requests_responses_and_unrelated_events_are_not_permission_observations(){
 let mut c=Cache::default();let mut request=update(A);request["id"]=json!("native-request");for event in [request,json!({"id":"reply","result":{}}),json!({"method":"item/commandExecution/requestApproval","params":{"threadId":A}})]{assert!(c.event(&event,100).is_none());}assert!(c.snapshot(A,100,30).last_observed.is_none());
}
#[test]fn malformed_policy_projection_never_becomes_a_default_or_raw_export(){
 let mut c=Cache::default();for key in ["approvalPolicy","approvalsReviewer","sandboxPolicy","cwd"]{let mut v=update(A);v["params"]["threadSettings"].as_object_mut().unwrap().remove(key);c.event(&v,100);assert_eq!(c.snapshot(A,100,30).reason,Some(Reason::InvalidNotification));assert!(c.snapshot(A,100,30).last_observed.is_none());}
}
#[test]fn receive_sequence_is_local_order_and_never_a_policy_update_ack(){
 let mut c=Cache::default();for now in [100,101,102]{c.event(&update(A),now);}let s=c.snapshot(A,102,30);assert_eq!(s.sequence,Some(3));assert_eq!(s.last_observed.unwrap().approval_policy,ApprovalPolicy::OnRequest);assert_eq!(s.observed_at,Some(102));
}

#[test]fn receive_sequence_exhaustion_retires_observation_without_wrap_or_fake_receipt(){
 let mut c=Cache::default();c.event(&update(A),100);c.sequence=u64::MAX;assert!(c.event(&update(A),101).is_none());let s=c.snapshot(A,101,30);assert_eq!(s.reason,Some(Reason::SequenceExhausted));assert_eq!(s.sequence,Some(1));assert!(s.stale);c.disconnect();assert_eq!(c.snapshot(A,102,30).reason,Some(Reason::SequenceExhausted));
}
