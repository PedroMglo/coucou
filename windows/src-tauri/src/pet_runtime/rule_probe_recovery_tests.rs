use super::*;
use crate::pet_runtime::{rules::{RulePolicy,RuleScope,RuleKind,RulePhase,DispatchPolicy},native_owner::EndedOperation};
use serde_json::json;
use std::{fs,path::PathBuf,os::unix::fs::PermissionsExt,process::{Child,Command,Stdio},time::{Duration,Instant},thread};
struct F{path:PathBuf,grant:RuleGrant}
impl F{
 fn new()->Self{
  let path=std::env::temp_dir().join(format!("coucou-rule-probe-recovery-{}",nonce().unwrap()));fs::create_dir(&path).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();
  let definition=serde_json::from_value(json!({"name":"Original metadata scope","intent":"One finite response","criteria":["Verify actual result"],"schedule":{"kind":"daily","firstDate":"2026-11-10","days":2,"localTime":"09:00","timeZone":"Etc/UTC","weekdays":[1,2,3,4,5,6,7],"overlap":"earlier","gap":"skip"},"missed":"skip","graceSeconds":300,"dataScope":"none","limits":{"maxOccurrences":2,"maxModelCalls":2,"maxToolCalls":0,"maxOutputBytes":8192,"occurrenceOutputBytes":4096,"occurrenceTimeoutSeconds":900}})).unwrap();
  let rule=crate::pet_runtime::rules::Rule{id:format!("rule-{}","a".repeat(64)),revision:1,owner:crate::pet_runtime::model::Owner{pet_id:"ai_ollama".into(),provider:"ollama".into(),adapter:"ollama".into(),model_hint:None},state:crate::pet_runtime::rules::State::Draft,created_at:1000,updated_at:1000,definition};
  let schedule={let _s=crate::pet_calendar::TEST_SERIAL.lock().unwrap();crate::pet_calendar::schedule::resolve(&rule.definition.schedule,||true).unwrap()};let first=schedule.slots[0].at.unwrap();let last=schedule.slots.last().unwrap().at.unwrap();
  let policy=RulePolicy{dispatch:Some(DispatchPolicy::bounded()),schema_version:2,scope:RuleScope::BoundedAutomaticRuleRecurrenceWithoutSelectedData,rule,payment:crate::pet_runtime::admission::Payment::LocalOnly,destination:crate::pet_memory::context_model::Destination::Server{provider:"ollama".into(),endpoint:"http://127.0.0.1:11434/v1/chat/completions".into(),model:"fixture-free".into()},model:"fixture-free".into(),engine_binding:None,policy_revision:"b".repeat(64),descriptor_semantics:"c".repeat(64),schedule,prepared_at:first-60000,valid_until:last+900000};
  let grant=RuleGrant{schema_version:3,kind:RuleKind::RuleRecurrence,decision_id:nonce().unwrap(),revision:nonce().unwrap(),approved_at:policy.prepared_at,updated_at:policy.prepared_at,revoked_at:None,phase:RulePhase::Active,policy,reservations:vec![],intent_retirements:vec![],succession:None,revision_authority:None};assert!(grant.valid());let f=Self{path,grant};let a=f.a();a.issue_rule(&f.grant,f.grant.approved_at,||true).unwrap();fs::write(f.path.join("grant.json"),serde_json::to_vec(&f.grant).unwrap()).unwrap();f
 }
 fn a(&self)->Authority{Authority::ensure(&self.path.join("authority")).unwrap()}
 fn at(&self)->u64{self.grant.policy.schedule.slots[0].at.unwrap()}
 fn bytes(&self)->Vec<u8>{fs::read(self.path.join("authority").join(format!("{}.json",self.grant.decision_id))).unwrap()}
 fn child(&self,mode:&str)->ChildGuard{
  let ready=self.path.join("ready.json");assert!(!ready.exists());let mut cmd=Command::new(std::env::current_exe().unwrap());cmd.args(["--exact","pet_runtime::authority_store::rule_probe_recovery::tests::rule_probe_owner_child","--ignored","--nocapture"]).env_clear().env("PATH","/usr/bin:/bin").env("COUCOU_RULE_PROBE_PROFILE_NATIVE",&self.path).env("COUCOU_RULE_PROBE_PHASE_NATIVE",mode).stdout(Stdio::null()).stderr(Stdio::null());if let Some(v)=std::env::var_os("LD_LIBRARY_PATH"){cmd.env("LD_LIBRARY_PATH",v);}
  let mut child=ChildGuard(cmd.spawn().unwrap());let end=Instant::now()+Duration::from_secs(8);while !ready.exists(){assert!(child.0.try_wait().unwrap().is_none(),"Own metadata fixture exited before ready");assert!(Instant::now()<end,"Own metadata fixture readiness timeout");thread::sleep(Duration::from_millis(10));}child
 }
}
impl Drop for F{fn drop(&mut self){let _=fs::remove_dir_all(&self.path);}}
struct ChildGuard(Child);impl ChildGuard{fn end(&mut self){if self.0.try_wait().unwrap().is_none(){self.0.kill().unwrap();}self.0.wait().unwrap();}}
impl Drop for ChildGuard{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
/// Fixture producer explicitly invoked in a separate original native process.
#[test]#[ignore]fn rule_probe_owner_child(){
 let Ok(path)=std::env::var("COUCOU_RULE_PROBE_PROFILE_NATIVE")else{return;};let path=PathBuf::from(path);let mode=std::env::var("COUCOU_RULE_PROBE_PHASE_NATIVE").unwrap();let g:RuleGrant=serde_json::from_slice(&fs::read(path.join("grant.json")).unwrap()).unwrap();let mut a=Authority::existing(&path.join("authority")).unwrap();let at=g.policy.schedule.slots[0].at.unwrap();let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,at,||true).unwrap();
 if mode=="pendingBegin"{a.fault=Fault::BeforeGrantPublish;assert!(a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,at,||true).is_err());}
 else if mode=="unlockedLive"{a.fault=Fault::AfterGrantPublish;assert!(a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,at,||true).is_err());}
 else{
  let owned=a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,at,||true).unwrap();assert!(owned.current(&a));
  if mode=="pendingFinish"{a.fault=Fault::BeforeGrantPublish;assert!(a.finish_rule_probe(&g.decision_id,&r.occurrence_id,owned.probe(),true,at+1).is_err());}
  fs::write(path.join("ready.json"),b"{}").unwrap();loop{thread::sleep(Duration::from_secs(1));std::hint::black_box(&owned);}
 }
 fs::write(path.join("ready.json"),b"{}").unwrap();loop{thread::sleep(Duration::from_secs(1));}
}
#[test]fn rule_probe_recovery_core_requires_original_process_absence_and_retains_spent_allowance(){
 for mode in ["hold","unlockedLive"]{
  let f=F::new();let a=f.a();let mut child=f.child(mode);let source=a.prepare_rule_probe_recovery(&f.grant.decision_id).unwrap();assert!(EndedOperation::capture(&a,source.owner().unwrap()).is_err(),"A live original process, even without FD, is never absent");child.end();let gate=EndedOperation::capture(&a,source.owner().unwrap()).unwrap();
  let candidate=source.candidate(&nonce().unwrap(),f.at()+7000).unwrap();assert_eq!(candidate.schema_version,4);let before=f.bytes();let mut fake=candidate.clone();fake.policy.rule.definition.limits.max_occurrences=3;fake.policy.rule.definition.limits.max_model_calls=3;fake.policy.rule.definition.limits.max_output_bytes=12288;let scope=ReadinessProbe::scope(&fake.policy,&fake.reservations[0]).unwrap();fake.reservations[0].readiness[0].scope_hash=scope.clone();fake.reservations[0].readiness[0].owner.as_mut().unwrap().scope_hash=scope;assert!(fake.valid());assert!(a.recover_rule_probe(&source,&fake,&gate,f.at()+7001,||true).is_err());assert_eq!(f.bytes(),before);
  let mut fake=candidate.clone();fake.reservations[0].readiness[0].recovery.as_mut().unwrap().source_hash="e".repeat(64);assert!(a.recover_rule_probe(&source,&fake,&gate,f.at()+7001,||true).is_err());assert_eq!(f.bytes(),before);
  let recovered=a.recover_rule_probe(&source,&candidate,&gate,f.at()+7001,||true).unwrap();let spent=&recovered.reservations[0];assert_eq!(spent.deadline_at,f.at()+900000);assert_eq!(spent.readiness.len(),1);assert_eq!(spent.readiness[0].observed,Some(false));assert_eq!(spent.readiness[0].recovery.as_ref().unwrap().outcome,RecoveryOutcome::NativeWorkerEndedWithoutObservation);assert_eq!(recovered.policy,f.grant.policy);assert_eq!(recovered.approved_at,f.grant.approved_at);
  let bytes=f.bytes();assert_eq!(a.recover_rule_probe(&source,&candidate,&gate,f.at()+7002,||true).unwrap(),candidate);assert_eq!(f.bytes(),bytes);assert_eq!(a.begin_owned_rule_probe(&recovered.decision_id,&spent.occurrence_id,f.at()+7003,||true).err().map(|_|()),Some(()));
  let next=a.begin_owned_rule_probe(&recovered.decision_id,&spent.occurrence_id,f.at()+37000,||true).unwrap();assert!(next.current(&a));assert_eq!(a.rule_grant(&recovered.decision_id).unwrap().reservations[0].readiness.len(),2);let advanced=f.bytes();assert_eq!(a.recover_rule_probe(&source,&candidate,&gate,f.at()+37001,||true).unwrap().reservations[0].readiness.len(),2);assert_eq!(f.bytes(),advanced);
 }
}
#[test]fn rule_probe_recovery_core_acknowledges_only_exact_staged_begin_finish_and_recovery_candidates(){
 for(mode,fault)in [("pendingBegin",Fault::BeforeGrantPublish),("pendingFinish",Fault::BeforeGrantPublish),("hold",Fault::AfterGrantPublish)]{
  let f=F::new();let mut a=f.a();let mut child=f.child(mode);child.end();let source=a.prepare_rule_probe_recovery(&f.grant.decision_id).unwrap();let gate=EndedOperation::capture(&a,source.owner().unwrap()).unwrap();let candidate=source.candidate(&nonce().unwrap(),f.at()+7000).unwrap();let original=candidate.clone();a.fault=fault;assert!(a.recover_rule_probe(&source,&candidate,&gate,f.at()+7001,||true).is_err());a.fault=Fault::None;
  let acknowledged=if mode!="hold"{
   // Recreate the native review after loss of the in-memory candidate. Only
   // the original signed recovery body/decision/time may be ACKed.
   drop(gate);let reopened=Authority::existing(&f.path.join("authority")).unwrap();let next_source=reopened.prepare_rule_probe_recovery(&candidate.decision_id).unwrap();let next_gate=EndedOperation::capture(&reopened,next_source.owner().unwrap()).unwrap();let same=next_source.candidate(&nonce().unwrap(),f.at()+8000).unwrap();assert_eq!(same,original);reopened.recover_rule_probe(&next_source,&same,&next_gate,f.at()+8001,||true).unwrap()
  }else{a.recover_rule_probe(&source,&candidate,&gate,f.at()+7002,||true).unwrap()};
  assert_eq!(acknowledged,original);let restored=a.rule_grant(&candidate.decision_id).unwrap();assert_eq!(restored.reservations[0].readiness.len(),1);assert_eq!(a.rule_status(&restored.policy.rule,true).unwrap().pending_artifacts,0);
  if mode=="pendingFinish"{assert_eq!(restored.reservations[0].readiness[0].observed,Some(true));assert_eq!(restored.reservations[0].readiness[0].finished_at,Some(f.at()+1));assert_eq!(restored.reservations[0].readiness[0].recovery.as_ref().unwrap().outcome,RecoveryOutcome::OriginalCompletionAcknowledged);}
 }
}
#[test]fn rule_probe_recovery_core_refuses_changed_source_revocation_cancel_and_replaced_owner_lock(){
 let f=F::new();let a=f.a();let mut child=f.child("hold");child.end();let source=a.prepare_rule_probe_recovery(&f.grant.decision_id).unwrap();let gate=EndedOperation::capture(&a,source.owner().unwrap()).unwrap();let candidate=source.candidate(&nonce().unwrap(),f.at()+7000).unwrap();let before=f.bytes();assert!(a.recover_rule_probe(&source,&candidate,&gate,f.at()+7001,||false).is_err());assert_eq!(before,f.bytes());
 a.revoke_rule("ai_ollama",&f.grant.policy.rule.id,&f.grant.decision_id,f.at()+7001).unwrap();assert!(a.recover_rule_probe(&source,&candidate,&gate,f.at()+7002,||true).is_err());drop(gate);
 let source=a.prepare_rule_probe_recovery(&f.grant.decision_id).unwrap();let gate=EndedOperation::capture(&a,source.owner().unwrap()).unwrap();let candidate=source.candidate(&nonce().unwrap(),f.at()+7003).unwrap();let recovered=a.recover_rule_probe(&source,&candidate,&gate,f.at()+7004,||true).unwrap();assert_eq!(recovered.phase,RulePhase::Revoked);assert!(a.begin_owned_rule_probe(&recovered.decision_id,&recovered.reservations[0].occurrence_id,f.at()+40000,||true).is_err());
 let owner=source.owner().unwrap();let(dir,_)=a.owner_context().unwrap();let path=dir.join(format!("op-{}.lock",owner.operation_id));fs::rename(&path,path.with_extension("original")).unwrap();fs::write(&path,b"").unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o600)).unwrap();assert!(gate.current(&a).is_err());assert!(EndedOperation::capture(&a,owner).is_err());
}
#[test]fn rule_probe_recovery_core_preserves_legacy_bytes_and_quarantines_restored_owner_evidence(){
 let f=F::new();let a=f.a();let g=&f.grant;let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(),||true).unwrap();a.begin_rule_probe(&g.decision_id,&r.occurrence_id,f.at(),||true).unwrap();let legacy=f.bytes();assert_eq!(a.rule_grant(&g.decision_id).unwrap().schema_version,3);assert!(a.prepare_rule_probe_recovery(&g.decision_id).is_err());assert_eq!(f.bytes(),legacy);
 let f=F::new();let a=f.a();let mut child=f.child("hold");child.end();let source=a.prepare_rule_probe_recovery(&f.grant.decision_id).unwrap();let gate=EndedOperation::capture(&a,source.owner().unwrap()).unwrap();let candidate=source.candidate(&nonce().unwrap(),f.at()+7000).unwrap();a.recover_rule_probe(&source,&candidate,&gate,f.at()+7001,||true).unwrap();drop(gate);let bytes=f.bytes();let review=crate::pet_runtime::authority_store::restoration::Review::capture(&f.path.join("authority"),&f.path.join("goals"),&"d".repeat(64),||Ok(())).unwrap();let fp=review.fingerprint().unwrap();review.apply(&fp,||Ok(())).unwrap();assert_eq!(f.bytes(),bytes);assert!(a.prepare_rule_probe_recovery(&candidate.decision_id).is_err());let reopened=Authority::existing(&f.path.join("authority")).unwrap();assert!(reopened.rule_status(&candidate.policy.rule,true).unwrap().snapshot_only);
}

#[test]fn rule_probe_recovery_core_same_backend_requires_known_native_lifetime_end_not_just_an_unlocked_fd(){
 let f=F::new();let a=f.a();let g=&f.grant;let r=a.reserve_rule_occurrence(&g.decision_id,&g.policy.hash().unwrap(),0,f.at(),||true).unwrap();let owned=a.begin_owned_rule_probe(&g.decision_id,&r.occurrence_id,f.at(),||true).unwrap();let source=a.prepare_rule_probe_recovery(&g.decision_id).unwrap();assert!(EndedOperation::capture(&a,source.owner().unwrap()).is_err());drop(owned);
 let gate=EndedOperation::capture(&a,source.owner().unwrap()).unwrap();assert_eq!(gate.witness(),"sameBackendOperationEnded");assert_eq!(source.owner().unwrap().process.pid,std::process::id());let candidate=source.candidate(&nonce().unwrap(),f.at()+10).unwrap();let recovered=a.recover_rule_probe(&source,&candidate,&gate,f.at()+11,||true).unwrap();assert_eq!(recovered.reservations[0].readiness.len(),1);assert_eq!(recovered.reservations[0].readiness[0].observed,Some(false));assert_eq!(recovered.reservations[0].deadline_at,r.deadline_at);
}
