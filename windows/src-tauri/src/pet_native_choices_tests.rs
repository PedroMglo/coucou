use super::*;

#[test]fn final_dispatch_false_decision_that_persists_policy_is_not_control(){
 let method="item/commandExecution/requestApproval";let rule=json!({"applyNetworkPolicyAmendment":{"network_policy_amendment":{"host":"fixture.invalid","action":"deny"}}});let p=json!({"availableDecisions":[rule]});let result=json!({"decision":rule});
 assert!(!pure_refusal("codex",method,&p,&result));assert!(!codex_fixed_refusal(method,&result));
 for kind in ["reject_always","allow_once","allow_always"]{let p=json!({"options":[{"optionId":"opaque","kind":kind}]});assert!(!pure_refusal("copilot","session/request_permission",&p,&json!({"outcome":{"outcome":"selected","optionId":"opaque"}})));}
 let p=json!({"options":[{"optionId":"once","kind":"reject_once"}]});assert!(pure_refusal("copilot","session/request_permission",&p,&json!({"outcome":{"outcome":"selected","optionId":"once"}})));assert!(!pure_refusal("copilot","session/request_permission",&p,&json!({"outcome":{"outcome":"selected","optionId":"foreign"}})));
}
#[test]fn final_dispatch_fixed_refusals_never_carry_tool_data_or_amended_policy(){
 for(method,result)in [("execCommandApproval",json!({"decision":"abort"})),("item/fileChange/requestApproval",json!({"decision":"decline"})),("item/permissions/requestApproval",json!({"permissions":{},"scope":"turn"})),("mcpServer/elicitation/request",json!({"action":"decline","content":null,"_meta":null})),("item/tool/requestUserInput",json!({"answers":{"q":{"answers":[]}}}))]{
  assert!(codex_fixed_refusal(method,&result));let mut added=result.clone();added["private"]=json!("must not leave");assert!(!codex_fixed_refusal(method,&added));
 }
 for result in [json!({"answers":{"q":{"answers":["data"]}}}),json!({"answers":{"q":{"answers":[],"extra":true}}}),json!({"action":"decline","content":"data","_meta":null}),json!({"success":false,"contentItems":[{"text":"data"}]})]{assert!(!codex_fixed_refusal("item/tool/requestUserInput",&result));assert!(!codex_fixed_refusal("mcpServer/elicitation/request",&result));}
}
use crate::{pet_config::Adapter,pet_native_decisions::{Endpoint,Transport,Capture,PolicyBinding}};
fn native(adapter:Adapter,method:&str,params:&Value)->(Endpoint,Ticket){let e=Endpoint::new(adapter,Transport::ManagedPipe).unwrap();let t=e.capture(Capture{request_id:Some(&json!(77)),session_id:"native-session",turn_id:Some("native-turn"),prompt_id:None,method,arguments:params,policy:PolicyBinding::unknown(),cancel:None}).unwrap();(e,t)}
const COMMAND:&str="item/commandExecution/requestApproval";
fn selected(c:&Choices,scope:&str,allow:bool)->Value{let public=c.public();let id=public.as_array().unwrap().iter().find(|v|v["scope"]==scope&&v["allow"]==allow).unwrap()["id"].clone();json!({"allow":allow,"nativeChoice":id})}
#[test]fn advertised_order_and_exact_session_or_cancel_choices_are_preserved(){
 let p=json!({"availableDecisions":["cancel","acceptForSession","accept","decline","accept"]});let(_e,t)=native(Adapter::Codex,COMMAND,&p);let c=Choices::capture(&t,"review-a",&p).unwrap().unwrap();assert_eq!(c.provenance(),"nativeAdvertised");assert_eq!(c.public().as_array().unwrap().len(),4);assert_eq!(c.public()[0]["scope"],"turnCancellation");
 assert_eq!(c.resolve(&selected(&c,"session",true)).unwrap(),json!({"decision":"acceptForSession"}));assert_eq!(c.resolve(&selected(&c,"turnCancellation",false)).unwrap(),json!({"decision":"cancel"}));assert_eq!(c.resolve(&json!({"allow":true})).unwrap(),json!({"decision":"accept"}));assert_eq!(c.resolve(&json!({"allow":false})).unwrap(),json!({"decision":"decline"}));
}
#[test]fn missing_once_never_promotes_a_boolean_to_session_or_turn_cancellation(){
 let p=json!({"availableDecisions":["acceptForSession","cancel"]});let(_e,t)=native(Adapter::Codex,COMMAND,&p);let c=Choices::capture(&t,"review-a",&p).unwrap().unwrap();for allow in[true,false]{assert!(c.resolve(&json!({"allow":allow})).is_err());assert!(codex_response(COMMAND,&p,Some(&json!({"allow":allow}))).is_none());}assert!(c.resolve(&selected(&c,"session",true)).is_ok());
}
#[test]fn protocol_fallback_is_identified_and_never_guesses_rules_from_command_text(){
 let p=json!({"command":"arbitrary shell syntax"});let(_e,t)=native(Adapter::Codex,COMMAND,&p);let c=Choices::capture(&t,"review-a",&p).unwrap().unwrap();assert_eq!(c.provenance(),"protocolFallback");assert_eq!(c.public().as_array().unwrap().len(),4);assert!(!c.public().as_array().unwrap().iter().any(|v|v["scope"]=="policy"));
 let p=json!({"command":"unrelated text","proposedExecpolicyAmendment":["exact","native-prefix"]});let(_e,t)=native(Adapter::Codex,COMMAND,&p);let c=Choices::capture(&t,"review-b",&p).unwrap().unwrap();assert_eq!(c.resolve(&selected(&c,"policy",true)).unwrap(),json!({"decision":{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["exact","native-prefix"]}}}));
}
#[test]fn policy_amendments_preserve_exact_proposals_and_explicit_deny_effects(){
 let rule=json!({"applyNetworkPolicyAmendment":{"network_policy_amendment":{"host":"fixture.invalid","action":"deny"}}});let exec=json!({"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["tool","--exact"]}});let p=json!({"availableDecisions":[rule,exec]});let(_e,t)=native(Adapter::Codex,COMMAND,&p);let c=Choices::capture(&t,"review-a",&p).unwrap().unwrap();assert_eq!(c.resolve(&selected(&c,"policy",false)).unwrap(),json!({"decision":rule}));assert_eq!(c.resolve(&selected(&c,"policy",true)).unwrap(),json!({"decision":exec}));assert!(c.public()[0]["detail"].as_str().unwrap().contains("fixture.invalid"));
 let fabricated=json!({"allow":true,"nativeResponse":{"decision":{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["different"]}}}});assert!(codex_response(COMMAND,&p,Some(&fabricated)).is_none());
}
#[test]fn unknown_malformed_or_oversized_choices_do_not_create_an_approval(){
 for p in[json!({"availableDecisions":[]}),json!({"availableDecisions":true}),json!({"availableDecisions":["future_mode"]}),json!({"availableDecisions":[{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":"wrong"}}]}),json!({"availableDecisions":vec!["accept";33]}),json!({"private":"x".repeat(256*1024)})]{let(_e,t)=native(Adapter::Codex,COMMAND,&json!({}));assert!(Choices::capture(&t,"review",&p).is_err());}
}
#[test]fn file_decisions_preserve_the_known_native_set_without_policy_amendments(){
 let p=json!({});let(_e,t)=native(Adapter::Codex,"item/fileChange/requestApproval",&p);let c=Choices::capture(&t,"files",&p).unwrap().unwrap();assert_eq!(c.provenance(),"protocolFallback");assert_eq!(c.resolve(&selected(&c,"session",true)).unwrap(),json!({"decision":"acceptForSession"}));assert!(!c.public().as_array().unwrap().iter().any(|v|v["scope"]=="policy"));
}
#[test]fn additional_permissions_bind_exact_original_permissions_and_explicit_scope(){
 let permissions=json!({"network":{"enabled":true},"fileSystem":{"read":["/fixture/one"]}});let p=json!({"permissions":permissions});let method="item/permissions/requestApproval";let(_e,t)=native(Adapter::Codex,method,&p);let c=Choices::capture(&t,"permissions",&p).unwrap().unwrap();assert_eq!(c.resolve(&json!({"allow":true})).unwrap(),json!({"permissions":permissions,"scope":"turn"}));assert_eq!(c.resolve(&selected(&c,"session",true)).unwrap(),json!({"permissions":permissions,"scope":"session"}));assert_eq!(c.resolve(&json!({"allow":false})).unwrap(),json!({"permissions":{},"scope":"turn"}));assert!(codex_response(method,&p,Some(&json!({"allow":true,"nativeResponse":{"permissions":{"network":{"enabled":false}},"scope":"session"}}))).is_none());
}
#[test]fn copilot_uses_original_option_ids_and_never_the_label_as_permission_authority(){
 let p=json!({"options":[{"kind":"allow_always","optionId":"opaque-remember","name":"Allow once"},{"kind":"reject_always","optionId":"opaque-reject"}]});let(_e,t)=native(Adapter::Copilot,"session/request_permission",&p);let c=Choices::capture(&t,"acp-review",&p).unwrap().unwrap();assert!(!c.public().to_string().contains("opaque-remember"));assert_eq!(c.public()[0]["scope"],"nativeRemembered");assert!(c.public()[0]["label"].as_str().unwrap().contains("memorizar"));assert!(c.resolve(&json!({"allow":true})).is_err());
 let mut answer=selected(&c,"nativeRemembered",true);answer["nativeResponse"]=c.resolve(&answer).unwrap();assert_eq!(copilot_response(&p,Some(&answer)),json!({"outcome":{"outcome":"selected","optionId":"opaque-remember"}}));assert_eq!(copilot_response(&p,Some(&json!({"allow":true}))),json!({"outcome":{"outcome":"cancelled"}}));
}
#[test]fn ambiguous_copilot_ids_and_unknown_only_options_never_authorize_a_choice(){
 for p in[json!({"options":[{"kind":"allow_once","optionId":"duplicate"},{"kind":"reject_once","optionId":"duplicate"}]}),json!({"options":[{"kind":"allow_once","optionId":"bad\nid"}]}),json!({"options":[{"kind":"_future","optionId":"future"}]})]{let(_e,t)=native(Adapter::Copilot,"session/request_permission",&p);assert!(Choices::capture(&t,"review",&p).is_err());assert_eq!(copilot_response(&p,Some(&json!({"allow":true}))),json!({"outcome":{"outcome":"cancelled"}}));}
}
#[test]fn policy_change_and_endpoint_closure_irreversibly_withdraw_choice_snapshots(){
 for policy in[true,false]{let p=json!({});let(e,t)=native(Adapter::Codex,COMMAND,&p);let c=Choices::capture(&t,"review",&p).unwrap().unwrap();let answer=selected(&c,"session",true);if policy{t.invalidate_policy();}else{e.close();}assert!(c.resolve(&answer).is_err());assert!(Choices::capture(&t,"new-review",&p).is_err());}
}
#[test]fn request_ids_and_original_sources_cannot_be_exchanged_between_choice_snapshots(){
 let p=json!({});let(_e,a)=native(Adapter::Codex,COMMAND,&p);let(_f,b)=native(Adapter::Codex,COMMAND,&p);let one=Choices::capture(&a,"one",&p).unwrap().unwrap();let two=Choices::capture(&b,"two",&p).unwrap().unwrap();assert!(one.matches(&a)&&!one.matches(&b));assert!(two.resolve(&selected(&one,"session",true)).is_err());let mut wrong=selected(&one,"turnCancellation",false);wrong["allow"]=json!(true);assert!(one.resolve(&wrong).is_err());
}
#[test]fn sharing_the_framework_does_not_offer_codex_choices_to_another_native_adapter(){
 let p=json!({});let(_e,t)=native(Adapter::Antigravity,COMMAND,&p);assert!(Choices::capture(&t,"review",&p).unwrap().is_none());
}

#[test]fn a_native_grant_root_request_is_never_consumed_as_a_once_only_file_approval(){
 let p=json!({"grantRoot":"/fixture/write-root"});let method="item/fileChange/requestApproval";let(_e,t)=native(Adapter::Codex,method,&p);let c=Choices::capture(&t,"root-review",&p).unwrap().unwrap();assert!(c.resolve(&json!({"allow":true})).is_err());assert!(codex_response(method,&p,Some(&json!({"allow":true}))).is_none());let public=c.public();for entry in public.as_array().unwrap().iter().filter(|v|v["allow"]==true){assert_eq!(entry["scope"],"session");assert!(entry["detail"].as_str().unwrap().contains("/fixture/write-root"));}
 let explicit=json!({"allow":true,"nativeChoice":public[0]["id"]});assert_eq!(c.resolve(&explicit).unwrap(),json!({"decision":"accept"}));assert_eq!(c.resolve(&json!({"allow":false})).unwrap(),json!({"decision":"decline"}));
}

#[test]fn legacy_choices_use_the_exact_review_schema_and_explicit_session_or_abort(){
 for method in["execCommandApproval","applyPatchApproval"]{
  let p=json!({"command":["fixture"],"proposedExecpolicyAmendment":["never-invent"]});let(_e,t)=native(Adapter::Codex,method,&p);let c=Choices::capture(&t,"legacy",&p).unwrap().unwrap();assert_eq!(c.provenance(),"nativeLegacyProtocol");assert_eq!(c.public().as_array().unwrap().len(),4);
  assert_eq!(c.resolve(&json!({"allow":true})).unwrap(),json!({"decision":"approved"}));
  assert_eq!(c.resolve(&json!({"allow":false})).unwrap(),json!({"decision":{"denied":{"rejection":"The request was not approved."}}}));
  assert_eq!(c.resolve(&selected(&c,"session",true)).unwrap(),json!({"decision":"approved_for_session"}));
  assert_eq!(c.resolve(&selected(&c,"turnCancellation",false)).unwrap(),json!({"decision":"abort"}));
  assert!(codex_response(method,&p,Some(&json!({"allow":false,"nativeResponse":{"decision":"denied"}}))).is_none());
  assert!(codex_response(method,&p,Some(&json!({"allow":true,"nativeResponse":{"decision":"approved_mcp_policy_amendment"}}))).is_none());
 }
}
#[test]fn legacy_write_root_requires_an_explicit_scope_without_broadening_boolean_allow(){
 let method="applyPatchApproval";let p=json!({"grantRoot":"/fixture/root"});let(_e,t)=native(Adapter::Codex,method,&p);let c=Choices::capture(&t,"legacy-root",&p).unwrap().unwrap();assert!(c.resolve(&json!({"allow":true})).is_err());assert!(codex_response(method,&p,Some(&json!({"allow":true}))).is_none());
 for row in c.public().as_array().unwrap().iter().filter(|v|v["allow"]==true){assert_eq!(row["scope"],"session");assert!(row["detail"].as_str().unwrap().contains("/fixture/root"));}
 let public=c.public();assert_eq!(c.resolve(&json!({"allow":true,"nativeChoice":public[1]["id"]})).unwrap(),json!({"decision":"approved_for_session"}));
 assert!(c.resolve(&json!({"allow":false})).unwrap()["decision"]["denied"]["rejection"].is_string());
 for root in[json!("relative/root"),json!(false),json!("/bad\nroot")]{let bad=json!({"grantRoot":root});assert!(Choices::capture(&t,"bad-root",&bad).is_err());}
}
