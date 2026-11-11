//! Private key and grant records; never export secret material to UI or models.
#[path="authority_work_disclosure.rs"]mod work_disclosure;
#[path="authority_messages.rs"]mod messages;
#[path="authority_funding.rs"]mod funding;
#[path="authority_delegation_lineage.rs"]mod delegation_lineage;
#[path="authority_delegation_subtree.rs"]mod delegation_subtree;
pub(in crate::pet_runtime)use delegation_subtree::project_stop_queues;
#[path="authority_delegation.rs"]pub(in crate::pet_runtime)mod delegation;
#[path="authority_delegation_branch.rs"]pub(in crate::pet_runtime)mod delegation_branch;
#[path="authority_received_message.rs"]mod received_message;
pub(in crate::pet_runtime)use received_message::Received;
#[path="authority_received_request.rs"]pub(in crate::pet_runtime)mod received_request;
#[path="authority_result_disclosure.rs"]mod result_disclosure;
#[path="authority_received_result.rs"]mod received_result;
#[path="authority_received_planned_data.rs"]mod received_planned_data;
#[path="authority_peer_round_data.rs"]mod peer_round_data;
#[path="authority_peer_round_sources.rs"]mod peer_round_sources;
#[path="authority_peer_round_history.rs"]mod peer_round_history;
#[path="authority_peer_publication.rs"]mod peer_publication;
pub(in crate::pet_runtime)use peer_publication::ReconciledPeerPublication;
#[path="authority_received_context.rs"]pub(in crate::pet_runtime)mod received_context;
#[path="authority_native_completion.rs"]mod native_completion;
#[path="authority_native_drain.rs"]mod native_drain;
#[path="authority_native_plan.rs"]mod native_plan;
#[path="authority_native_plan_inventory.rs"]mod native_plan_inventory;
pub(in crate::pet_runtime)use native_plan::NativePlanCertified;
#[path="authority_message_retirement.rs"]pub(in crate::pet_runtime)mod message_retirement;
use super::{authority::*,admission::AdmissionSpec,model};
use crate::pet_runtime::{admission::Approved,model::{Record,State},store::{Dir,Guard,Stamp,nonce,protocol_directory}};
use ring::{hmac,rand::{SecureRandom,SystemRandom}};
use serde::{Serialize,Deserialize};
use std::{path::Path,os::{fd::AsRawFd,unix::fs::MetadataExt}};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-native-consent\"}\n";
const KEY:&str=".key.json";
const MAX_GRANT_BYTES:usize=128*1024;
#[path="authority_restoration.rs"]pub(crate)mod restoration;
#[path="authority_restored_future.rs"]pub(in crate::pet_runtime)mod restored_future;
#[path="authority_rules.rs"]pub(in crate::pet_runtime)mod rules;
#[path="authority_rule_readiness.rs"]mod rule_readiness;
#[path="authority_rule_probe_recovery.rs"]mod rule_probe_recovery;
pub(in crate::pet_runtime)use rule_probe_recovery::RuleProbeRecoverySource;
#[path="authority_rule_intent_retirement.rs"]mod rule_intent_retirement;
pub(in crate::pet_runtime)use rule_intent_retirement::RuleIntentRetirementSource;
#[path="authority_rule_lineage.rs"]mod rule_lineage;
#[path="authority_rule_succession.rs"]mod rule_succession;
pub(in crate::pet_runtime)use rule_succession::RuleSuccessionSource;
#[path="authority_budget.rs"]mod budget;
#[path="authority_results.rs"]mod results;
#[path="authority_planning.rs"]mod planning;
#[path="authority_plan_commit.rs"]mod plan_commit;
#[path="authority_planning_feedback.rs"]mod planning_feedback;
#[path="authority_resource_wait.rs"]mod resource_wait;
#[path="authority_owner.rs"]mod owner;
#[path="authority_admission_ack.rs"]mod admission_ack;
#[path="authority_reconciliation.rs"]mod reconciliation;
#[path="authority_history.rs"]mod history;
#[path="authority_continuation.rs"]mod continuation;
#[path="authority_uncertain_effects.rs"]mod uncertain_effects;
#[path="authority_recovery.rs"]mod recovery;
#[path="authority_peer_resume.rs"]mod peer_resume;
#[path="authority_retirement.rs"]mod retirement;
pub(in crate::pet_runtime)use retirement::UnusedSource;
pub(in crate::pet_runtime)use owner::LostOwnerSource;
pub(in crate::pet_runtime)use plan_commit::PlanCertified;
#[cfg(target_os="linux")]#[path="authority_steps.rs"]mod steps;
pub(in crate::pet_runtime)use steps::CompletionCertified;
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct KeyFile{schema_version:u32,key_id:String,key:String}
impl KeyFile{
    fn parse(raw:&[u8])->Result<(Self,[u8;32])>{
        let key:Self=serde_json::from_slice(raw).map_err(|_|Failure::Invalid)?;
        if key.schema_version!=1||!model::token(&key.key_id){return Err(Failure::Invalid);}
        let bytes=decode(&key.key).ok_or(Failure::Invalid)?;Ok((key,bytes))
    }
}
pub(super)struct Authority{dir:Dir,stamp:Stamp,lock_stamp:Stamp,key_id:String,key:hmac::Key,protocol:Vec<u8>,restoration:Option<restoration::Boundary>,#[cfg(test)]pub(super)fault:Fault}
#[cfg(test)]#[derive(Clone,Copy,Default)]pub(super)enum Fault{#[default]None,BeforeGrantPublish,AfterGrantPublish,BeforeRuleIntentCleanup,AfterRuleIntentCleanup,BeforeResultPublish,AfterResultPublish,BeforeStepPublish,AfterStepPublish,BeforeStepResultPublish,AfterStepResultPublish,BeforeStepEvidencePublish,AfterStepEvidencePublish,BeforeStepBudgetFinish,AfterStepBudgetFinish,BeforeRecipePublish,AfterRecipePublish,BeforeVerificationPublish,AfterVerificationPublish,BeforePlanningOutputPublish,AfterPlanningOutputPublish}
pub(super)struct Certified<'a>{authority:&'a Authority,approved:&'a Approved,proof:ConsentProof}
impl Authority{
    pub(super)fn existing(path:&Path)->Result<Self>{
        let mut authority=Self::restoration_base(path)?;
        authority.restoration=restoration::load(&authority)?;authority.fence()?;Ok(authority)
    }
    fn restoration_base(path:&Path)->Result<Self>{
        let dir=Dir::open(path,true).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let(protocol,_)=dir.bytes(".protocol.json",1024).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        restoration::protocol_hash(&protocol)?;
        let(raw,stamp)=dir.bytes(KEY,512).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let(_,lock_stamp)=dir.bytes(".catalog.lock",0).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let(f,bytes)=KeyFile::parse(&raw)?;
        Ok(Self{dir,stamp,lock_stamp,key_id:f.key_id,key:hmac::Key::new(hmac::HMAC_SHA256,&bytes),protocol,restoration:None,#[cfg(test)]fault:Fault::None})
    }
    pub(super)fn ensure(path:&Path)->Result<Self>{
        match std::fs::symlink_metadata(path){Ok(_)=>return Self::existing(path),Err(e)if e.kind()==std::io::ErrorKind::NotFound=>{},Err(_)=>return Err(Failure::Unsafe)}
        let parent=Dir::open(path.parent().ok_or(Failure::Unsafe)?,false).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let name=path.file_name().and_then(|s|s.to_str()).ok_or(Failure::Unsafe)?;let setup=format!(".coucou-consent-setup-{}",nonce().map_err(|_|Failure::Unavailable)?);
        let staged=protocol_directory(&parent.path.join(&setup),PROTOCOL,true).map_err(|_|Failure::Unavailable)?.ok_or(Failure::Unavailable)?;
        let mut bytes=[0u8;32];SystemRandom::new().fill(&mut bytes).map_err(|_|Failure::Unavailable)?;
        let file=KeyFile{schema_version:1,key_id:nonce().map_err(|_|Failure::Unavailable)?,key:encode(&bytes)};
        staged.write_new(KEY,&serde_json::to_vec(&file).map_err(|_|Failure::Invalid)?).map_err(|_|Failure::Unavailable)?;staged.sync().map_err(|_|Failure::Uncertain)?;
        if parent.rename(&setup,&parent,name,true).is_err(){return Self::existing(path);}
        parent.sync().map_err(|_|Failure::Uncertain)?;Self::existing(path)
    }
    fn fence(&self)->Result<()>{
        self.restoration_raw_fence()?;restoration::fence(self)
    }
    fn restoration_raw_fence(&self)->Result<()>{
        self.dir.check().map_err(|_|Failure::Changed)?;
        if self.dir.bytes(".protocol.json",1024).map_err(|_|Failure::Changed)?.is_none_or(|(b,_)|b!=self.protocol)
            ||self.dir.bytes(KEY,512).map_err(|_|Failure::Changed)?.is_none_or(|(_,s)|s!=self.stamp)
            ||self.dir.bytes(".catalog.lock",0).map_err(|_|Failure::Changed)?.is_none_or(|(_,s)|s!=self.lock_stamp){return Err(Failure::Changed);}Ok(())
    }
    fn lock(&self)->Result<Guard>{
        self.lock_mode(true)
    }
    /// Proof observations may coexist. Every mutation keeps the exclusive lock;
    /// both modes retain the same key/directory/lock-inode fences.
    fn lock_shared(&self)->Result<Guard>{self.lock_mode(false)}
    fn lock_mode(&self,write:bool)->Result<Guard>{
        self.fence()?;let file=self.dir.open_file(".catalog.lock",if write{libc::O_RDWR}else{libc::O_RDONLY}).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        // Wait only to acquire this original descriptor before any mutation.
        // Readers stay nonblocking; no publication or effect is retried here.
        let until=std::time::Instant::now()+std::time::Duration::from_millis(if write{100}else{0});
        loop{
            if unsafe{libc::flock(file.as_raw_fd(),(if write{libc::LOCK_EX}else{libc::LOCK_SH})|libc::LOCK_NB)}==0{break;}
            if std::io::Error::last_os_error().kind()!=std::io::ErrorKind::WouldBlock{return Err(Failure::Unavailable);}
            if !write||std::time::Instant::now()>=until{return Err(Failure::Busy);}
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let guard=Guard(file);let current=self.dir.open_file(".catalog.lock",libc::O_RDONLY).map_err(|_|Failure::Changed)?.ok_or(Failure::Changed)?;
        let a=guard.0.metadata().map_err(|_|Failure::Changed)?;let b=current.metadata().map_err(|_|Failure::Changed)?;
        if a.dev()!=b.dev()||a.ino()!=b.ino(){return Err(Failure::Changed);}self.fence()?;Ok(guard)
    }
    fn proof(&self,spec:&AdmissionSpec)->Result<ConsentProof>{
        let planning=spec.planning();
        let mut proof=ConsentProof{schema_version:if spec.deterministic(){3}else if planning{2}else{1},key_id:self.key_id.clone(),operation:if spec.deterministic(){Operation::GoalDeterministic}else if planning{Operation::GoalPlanning}else{Operation::ConversationRespond},tag:String::new()};
        proof.tag=encode(hmac::sign(&self.key,&consent_message(&proof,spec)?).as_ref());Ok(proof)
    }
    fn verify_proof(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<()>{
        if !proof.valid()||!proof.matches(spec){return Err(Failure::Invalid);}if proof.key_id!=self.key_id{return Err(Failure::WrongKey);}
        hmac::verify(&self.key,&consent_message(proof,spec)?,&decode(&proof.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)
    }
    fn grant_file(&self,name:&str)->Result<(Grant,Stamp)>{
        if self.historical_file(name){return Err(Failure::RecoveryRequired);}self.grant_file_data(name)
    }
    fn grant_file_data(&self,name:&str)->Result<(Grant,Stamp)>{
        let(raw,stamp)=self.dir.bytes(name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let envelope:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if !envelope.body.valid(){return Err(Failure::Invalid);}
        hmac::verify(&self.key,&grant_message(&envelope.body)?,&decode(&envelope.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        Ok((envelope.body,stamp))
    }
    fn load(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<Grant>{
        self.restoration_current(&spec.decision_id,&spec.goal_id)?;
        self.verify_proof(spec,proof)?;
        let(raw,_)=self.dir.bytes(&format!("{}.json",spec.decision_id),MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let e:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        hmac::verify(&self.key,&grant_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        self.validate_grant_binding(spec,proof,e.body)
    }
    /// Same complete spec/ledger binding for current grants and authenticated
    /// historical DATA. Only load() performs the executable-current check.
    fn validate_grant_binding(&self,spec:&AdmissionSpec,proof:&ConsentProof,b:Grant)->Result<Grant>{
        if b.native_system_plan_hash!=spec.native_system_plan.as_ref().map(model::fingerprint).transpose().map_err(|_|Failure::Invalid)?||b.peer_memory_planning!=(if spec.peer_selected_iteration(){super::planned_peer_data::Snapshot::from_spec(spec).map_err(|_|Failure::Invalid)?}else{None})||b.peer_planning!=spec.peer_planning()||b.result_disclosure!=spec.result_disclosure||b.funded_by!=spec.funding||b.delegation_pool.as_ref()!=(if spec.pool_budget(){Some(&spec.budget)}else{None})||b.work_disclosure!=spec.work_disclosure||!b.valid()||b.native_completion.as_ref().is_some_and(|r|!r.matches_spec(spec))||b.budget_usage.as_ref().is_some_and(|u|!u.valid_for(spec))||b.planning_usage.as_ref().is_some_and(|u|!u.valid_for(spec,b.lease.as_ref(),b.budget_usage.as_ref()))||b.lease.as_ref().is_some_and(|l|!l.valid(spec))||b.decision_id!=spec.decision_id||b.goal_id!=spec.goal_id||b.operation!=proof.operation||b.spec_hash!=model::fingerprint(spec).map_err(|_|Failure::Invalid)?||!model::token(&b.revision)||b.approved_at!=spec.approved_at||b.updated_at<b.approved_at{return Err(Failure::Invalid);}Ok(b)
    }
    fn publish(&self,body:Grant,new:bool)->Result<()>{self.publish_guarded(body,new,||Ok(()))}
    fn publish_guarded(&self,mut body:Grant,new:bool,current:impl Fn()->Result<()>)->Result<()>{
        // Optional message receipts remain format9 through later budget/Stop/
        // recovery writers; old records keep their exact original format.
        if let Some(m)=&mut body.messages{m.refresh_schema();}if body.peer_planning{body.schema_version=body.message_schema();}else if body.operation==Operation::GoalDeterministic{body.schema_version=body.native_schema();}else if body.funded_by.is_some(){body.schema_version=body.funded_schema();}else if body.messages.is_some(){body.schema_version=body.message_schema();}current()?;
        self.restoration_current(&body.decision_id,&body.goal_id)?;
        if !body.valid(){return Err(Failure::Invalid);}self.fence()?;let tag=encode(hmac::sign(&self.key,&grant_message(&body)?).as_ref());let bytes=serde_json::to_vec(&GrantEnvelope{body:body.clone(),tag}).map_err(|_|Failure::Invalid)?;
        self.publish_record_bytes(&body.decision_id,&bytes,new,current)
    }
    /// Shared fenced writer for every typed Authority record. No effects here.
    fn publish_record_bytes(&self,decision:&str,bytes:&[u8],new:bool,current:impl Fn()->Result<()>)->Result<()>{
        if !model::token(decision)||bytes.len()>MAX_GRANT_BYTES{return Err(Failure::Capacity);}current()?;self.fence()?;
        let temp=format!(".grant-{}",nonce().map_err(|_|Failure::Unavailable)?);self.dir.write_new(&temp,bytes).map_err(|_|Failure::Unavailable)?;
        #[cfg(test)]if matches!(self.fault,Fault::BeforeGrantPublish){return Err(Failure::Unavailable);}
        current()?;self.fence()?;self.dir.rename(&temp,&self.dir,&format!("{decision}.json"),new).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(self.fault,Fault::AfterGrantPublish){return Err(Failure::Uncertain);}
        self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
    pub(super)fn certify<'a>(&'a self,approved:&'a Approved,goal:&Record,now:u64)->Result<Certified<'a>>{
        approved.current(goal,now).map_err(|_|Failure::Expired)?;let _lock=self.lock()?;self.verify_goal_origin(goal)?;let spec=approved.spec();self.recursive_admission_unlocked(spec,goal,now)?;let proof=self.proof(spec)?;
        if spec.funding.is_some()&&spec.continuation.is_none(){let(grants,pending)=self.history_inventory(&goal.id,Some(spec))?;if !pending.is_empty()||grants.values().any(|b|b.phase!=GrantPhase::Revoked||!b.never_claimed_zero()){return Err(Failure::RecoveryRequired);}}
        if let Some(reference)=&spec.continuation{self.continuation_current_unlocked(goal,reference,Some(spec))?;}
        self.funding_preflight_unlocked(spec,now)?;let file=format!("{}.json",spec.decision_id);
        if self.dir.open_file(&file,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.is_some(){
            let old=self.load(spec,&proof)?;if old.phase!=GrantPhase::Issued{return Err(Failure::Revoked);}
            self.dir.open_file(&file,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;
        }else{
            if self.dir.names().map_err(|_|Failure::Capacity)?.len()>=4093{return Err(Failure::Capacity);}
            self.publish(Grant{native_system_plan_hash:spec.native_system_plan.as_ref().map(model::fingerprint).transpose().map_err(|_|Failure::Invalid)?,peer_memory_planning:if spec.peer_selected_iteration(){super::planned_peer_data::Snapshot::from_spec(spec).map_err(|_|Failure::Invalid)?}else{None},owner_resumptions:vec![],peer_planning:spec.peer_planning(),result_disclosure:spec.result_disclosure.clone(),funded_by:spec.funding.clone(),delegation_pool:if spec.pool_budget(){Some(spec.budget.clone())}else{None},delegation_reservations:vec![],work_disclosure:spec.work_disclosure.clone(),native_plan_revisions:vec![],native_drain:None,native_completion:None,messages:None,retirement:None,recovery:None,runtime_owner:None,resource_usage:None,budget_usage:None,planning_usage:if spec.planning(){Some(super::planning_journal::Ledger::empty())}else{None},schema_version:if spec.planning(){4}else{1},decision_id:spec.decision_id.clone(),goal_id:spec.goal_id.clone(),operation:proof.operation,spec_hash:model::fingerprint(spec).map_err(|_|Failure::Invalid)?,phase:GrantPhase::Issued,revision:nonce().map_err(|_|Failure::Unavailable)?,approved_at:spec.approved_at,updated_at:spec.approved_at,lease:None,stop_reason:None,closed_at:None},true)?;
        }
        if spec.continuation.is_some(){self.retire_admission_staging(spec)?;}
        self.bind_funding_unlocked(spec,&proof,now)?;self.fence()?;Ok(Certified{authority:self,approved,proof})
    }
    pub(super)fn verify_queued(&self,goal:&Record,now:u64)->Result<()>{
        goal.validate().map_err(|_|Failure::Invalid)?;if goal.state!=State::Queued{return Err(Failure::NotQueued);}
        let row=goal.admissions.last().ok_or(Failure::Missing)?;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;
        let _lock=self.lock_shared()?;let body=self.load(&row.spec,proof)?;self.funding_current_unlocked(&row.spec,now)?;if body.phase!=GrantPhase::Issued{return Err(Failure::Revoked);}
        if now<row.spec.approved_at||now>=row.spec.budget.deadline_at{return Err(Failure::Expired);}if !row.spec.execution_due(now){return Err(Failure::NotDue);}self.fence()
    }
    /// ACK only: no new consent, grant, revision, lease or budget is issued.
    pub(super)fn acknowledge_requested(&self,goal:&Record,now:u64)->Result<()>{
        goal.validate().map_err(|_|Failure::Invalid)?;
        let row=goal.admissions.last().ok_or(Failure::Missing)?;
        if goal.state!=State::Queued||!row.spec.execution_requested()||now<row.spec.approved_at||now>=row.spec.budget.deadline_at{return Err(Failure::Expired);}
        let _lock=self.lock_shared()?;let body=self.load(&row.spec,row.proof.as_ref().ok_or(Failure::Missing)?)?;
        if body.phase!=GrantPhase::Issued||!body.never_claimed_zero(){return Err(Failure::RecoveryRequired);}self.funding_current_unlocked(&row.spec,now)?;self.stabilize(&body)
    }
    fn stabilize(&self,body:&Grant)->Result<()>{
        self.dir.open_file(&format!("{}.json",body.decision_id),libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
    fn active_unlocked(&self)->Result<Vec<super::leases::Lease>>{
        let mut active=vec![];
        for name in self.dir.names().map_err(|_|Failure::Capacity)?{
            if self.historical_file(&name){continue;}
            if matches!(name.as_str(),".key.json"|".catalog.lock"|".protocol.json"){continue;}
            let staged=name.strip_prefix(".grant-").is_some_and(model::token);
            let canonical=name.strip_suffix(".json").filter(|s|model::token(s));
            if !staged&&canonical.is_none(){return Err(Failure::RecoveryRequired);}
            if let Some((rule,_))=self.rule_record_data(&name)?{if canonical.is_some_and(|id|id!=rule.decision_id){return Err(Failure::RecoveryRequired);}continue;}
            let(raw,_)=self.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
            let e:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::RecoveryRequired)?;
            if !e.body.valid()||canonical.is_some_and(|id|id!=e.body.decision_id){return Err(Failure::RecoveryRequired);}
            hmac::verify(&self.key,&grant_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
            if !staged&&matches!(e.body.phase,GrantPhase::Claimed|GrantPhase::Draining){active.push(e.body.lease.ok_or(Failure::Invalid)?);}
        }
        self.fence()?;Ok(active)
    }
    pub(super)fn claim(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease,owner:&super::native_owner::OwnerLease,now:u64)->Result<()>{
        if !spec.execution_due(now){return Err(Failure::NotDue);}
        if !lease.valid(spec)||now<lease.claimed_at||now>=lease.deadline_at{return Err(Failure::Expired);}
        if !owner.matches(self,lease){return Err(Failure::Changed);}
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;self.funding_current_unlocked(spec,now)?;
        if self.active_unlocked()?.iter().any(|l|l.provider==lease.provider&&l!=lease){return Err(Failure::Busy);}
        if body.phase==GrantPhase::Claimed&&body.lease.as_ref()==Some(lease){if body.runtime_owner.as_ref()!=Some(owner.record()){return Err(Failure::Changed);}return self.stabilize(&body);}
        if body.phase!=GrantPhase::Issued{return Err(if matches!(body.phase,GrantPhase::Claimed|GrantPhase::Draining){Failure::Busy}else{Failure::Revoked});}
        if self.active_unlocked()?.iter().any(|l|l.provider==lease.provider){return Err(Failure::Busy);}
        if now<body.updated_at{return Err(Failure::Invalid);}
        body.schema_version=6;body.runtime_owner=Some(owner.record().clone());body.phase=GrantPhase::Claimed;body.lease=Some(lease.clone());body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
    }
    pub(super)fn finish(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease,unstarted:bool,expected_reason:Option<super::leases::StopReason>,now:u64)->Result<Option<super::leases::StopReason>>{
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;
        if unstarted&&body.lease.is_none(){
            if !lease.valid(spec)||now<lease.claimed_at||now<body.updated_at||now>8_640_000_000_000_000||!matches!(body.phase,GrantPhase::Issued|GrantPhase::Revoked){return Err(Failure::Invalid);}
            // A revoked, not-yet-claimed grant must wait for its authenticated
            // control projection. Publish only terminal audit metadata; never
            // reopen a grant or publish a synthetic Claimed transition.
            if body.phase==GrantPhase::Revoked&&expected_reason.is_none(){return Err(Failure::Busy);}
            if body.phase==GrantPhase::Issued&&matches!(expected_reason,Some(super::leases::StopReason::Pause|super::leases::StopReason::Cancel)){return Err(Failure::Changed);}
            body.schema_version=if body.planning_usage.is_some(){4}else{2};body.lease=Some(lease.clone());body.stop_reason=expected_reason;
            body.phase=if body.phase==GrantPhase::Revoked||expected_reason==Some(super::leases::StopReason::Cancel){GrantPhase::Revoked}else{GrantPhase::Closed};
            body.closed_at=Some(now);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;return Ok(expected_reason);
        }
        if body.lease.as_ref()!=Some(lease)||now<lease.claimed_at||now<body.updated_at||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
        if unstarted&&body.phase==GrantPhase::Claimed&&body.stop_reason.is_none()&&expected_reason==Some(super::leases::StopReason::Expired){body.stop_reason=expected_reason;}
        if body.stop_reason!=expected_reason{
            // Native Stop publishes the revocation before the goal projection.
            // Wait for that exact projection rather than close from stale data.
            if unstarted&&body.phase==GrantPhase::Closed&&body.stop_reason.is_none()&&expected_reason==Some(super::leases::StopReason::Expired){self.stabilize(&body)?;return Ok(None);}
            return Err(if (body.phase==GrantPhase::Draining&&body.stop_reason.is_some())||(body.phase==GrantPhase::Revoked&&body.stop_reason==Some(super::leases::StopReason::Cancel)){Failure::Busy}else{Failure::Changed});
        }
        if matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked){self.stabilize(&body)?;return Ok(body.stop_reason);}
        if !matches!(body.phase,GrantPhase::Claimed|GrantPhase::Waiting|GrantPhase::Draining){return Err(Failure::Invalid);}
        let reason=body.stop_reason;body.phase=if reason==Some(super::leases::StopReason::Cancel){GrantPhase::Revoked}else{GrantPhase::Closed};body.closed_at=Some(now);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;Ok(reason)
    }
    #[cfg(test)]pub(super)fn revoke(&self,spec:&AdmissionSpec,proof:&ConsentProof,now:u64)->Result<()>{self.request_stop(spec,proof,super::leases::StopReason::Cancel,now).map(|_|())}
    /// Authenticated control DATA for closing the original owner, never an IO
    /// permit. In particular a signed Cancel must not be projected as Pause.
    pub(in crate::pet_runtime)fn recorded_stop_reason(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<Option<super::leases::StopReason>>{let _guard=self.lock_shared()?;Ok(self.load(spec,proof)?.stop_reason)}
    pub(in crate::pet_runtime)fn original_lease_closed(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease)->Result<bool>{let _guard=self.lock_shared()?;let body=self.load(spec,proof)?;Ok(body.lease.as_ref()==Some(lease)&&matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)&&body.closed_at.is_some()&&body.runtime_owner.is_some()&&body.recovery.is_none()&&body.operation==Operation::GoalDeterministic&&(body.native_completion.is_some()||body.native_drain.as_ref().is_some_and(|d|d.reference.confirmed)))}
    pub(super)fn request_stop(&self,spec:&AdmissionSpec,proof:&ConsentProof,reason:super::leases::StopReason,now:u64)->Result<Option<super::leases::Lease>>{
        if now==0||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;if now<body.updated_at{
            // Same already-persisted reduction: acknowledge only, without a
            // new write, lease renewal or change to reason/target/allowance.
            if body.phase==GrantPhase::Revoked&&body.lease.is_none()||matches!(body.phase,GrantPhase::Draining|GrantPhase::Revoked|GrantPhase::Closed)&&body.stop_reason==Some(reason){
                self.stabilize(&body)?;return Ok(if body.phase==GrantPhase::Draining{body.lease.clone()}else{None});
            }
            return Err(Failure::Invalid);
        }
        if matches!(body.phase,GrantPhase::Revoked|GrantPhase::Closed){
            // The old execution remains closed and its closure receipt is immutable.
            // An existing pool still owns bound descendants: an explicit later
            // Stop must reduce their authority in this same signed ledger.
            if body.recovery.is_some()&&body.delegation_pool.is_none(){self.stabilize(&body)?;return Ok(None);}
            if body.lease.is_some()&&((reason==super::leases::StopReason::Cancel&&body.stop_reason!=Some(reason))||(body.phase==GrantPhase::Closed&&body.stop_reason.is_none()&&reason==super::leases::StopReason::Pause)){
                body.stop_reason=Some(reason);if reason==super::leases::StopReason::Cancel{body.phase=GrantPhase::Revoked;}
                body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;let live=body.lease.clone();self.publish(body,false)?;return Ok(live);
            }
            self.stabilize(&body)?;return Ok(None);
        }
        let live=body.lease.clone();
        if body.phase==GrantPhase::Draining&&body.stop_reason==Some(reason){self.stabilize(&body)?;return Ok(live);}
        if body.phase==GrantPhase::Issued{body.phase=GrantPhase::Revoked;}else{body.phase=GrantPhase::Draining;if body.stop_reason!=Some(super::leases::StopReason::Cancel){body.stop_reason=Some(reason);}}
        body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;Ok(live)
    }

}
impl Certified<'_>{
    pub(super)fn approved(&self)->&Approved{self.approved}
    pub(super)fn proof(&self)->&ConsentProof{&self.proof}
    pub(super)fn current(&self,goal:&Record,now:u64)->model::Result<()>{
        self.approved.current(goal,now)?;let _lock=self.authority.lock().map_err(Failure::control_error)?;
        self.authority.funding_current_unlocked(self.approved.spec(),now).map_err(Failure::control_error)?;
        if let Some(reference)=&self.approved.spec().continuation{self.authority.continuation_current_unlocked(goal,reference,Some(self.approved.spec())).map_err(Failure::control_error)?;}
        if self.authority.load(self.approved.spec(),&self.proof).map_err(Failure::control_error)?.phase!=GrantPhase::Issued{return Err(model::Error::Conflict);}self.authority.fence().map_err(Failure::control_error)
    }
}

/// Revoke the independent consent record before persisting queued pause/cancel.
/// Replays are receipt-only and must never revoke a newer admission.
pub(super)fn mutate_with_authority(store:&impl crate::pet_runtime::Repository,path:&Path,id:&str,pet:&str,revision:&str,request:String,action:model::Action,now:u64)->model::Result<model::Receipt>{
    if matches!(action,model::Action::Pause|model::Action::Cancel){if let Some(receipt)=store.stop_peer_owner_handoff(path,id,pet,revision,&request,&action,now)?{return Ok(receipt);}}
    let mut targets=vec![];
    if matches!(action,model::Action::Pause|model::Action::Cancel){
        let goal=store.read_control(id,pet)?;
        if goal.replay(&request,&action)?{
            // Recover only this original Stop's subtree, never the admission
            // current when a historical UI request happens to be replayed.
            let event=goal.events.iter().find(|e|e.request_id==request).ok_or(model::Error::Conflict)?;
            if let Some(row)=goal.admissions.iter().rev().find(|r|r.spec.source_sequence<event.sequence){
                if let Some(proof)=row.proof.as_ref(){
                    let authority=Authority::existing(path).map_err(Failure::control_error)?;
                    targets=authority.stopped_pool_descendants(&row.spec,proof,now).map_err(Failure::control_error)?;
                }
            }
            let receipt=store.mutate(id,pet,revision,request,action,now)?;
            if receipt.status!=model::CommitStatus::Uncertain{project_stop_queues(store,path,&targets,now)?;}
            return Ok(receipt);
        }
        if goal.revision!=revision{return Err(model::Error::Conflict);}
        // Refuse an invalid queue transition before any ledger reduction.
        let mut candidate=goal.clone();candidate.apply(request.clone(),action.clone(),now,nonce()?)?;
        if matches!(goal.state,State::Queued|State::Leased|State::Stopping)||(matches!(action,model::Action::Cancel)&&goal.state==State::Paused)||(matches!(action,model::Action::Pause|model::Action::Cancel)&&goal.state==State::RecoveryRequired){if let Some(row)=goal.admissions.last(){if let Some(proof)=&row.proof{
            if let Some(lease)=&row.lease{super::lease_controller::signal_stop(lease);}
            let authority=Authority::existing(path).map_err(Failure::control_error)?;
            if authority.verify_recovered_goal(&goal).map_err(Failure::control_error)?&&!authority.native_plan_pending_control(&goal).map_err(Failure::control_error)?{return Err(model::Error::Busy);}
            let reason=if matches!(action,model::Action::Cancel){super::leases::StopReason::Cancel}else{super::leases::StopReason::Pause};
            let live=authority.request_stop(&row.spec,proof,reason,now).map_err(Failure::control_error)?;
            if let Some(lease)=live{
                if row.lease.as_ref()!=Some(&lease)&&!(row.lease.is_none()&&goal.state==State::Queued&&lease.source_revision==goal.revision&&lease.valid(&row.spec)){return Err(model::Error::Conflict);}
                super::lease_controller::signal_stop(&lease);
            }
            targets=authority.stopped_pool_descendants(&row.spec,proof,now).map_err(Failure::control_error)?;
        }}}
    }
    let receipt=store.mutate(id,pet,revision,request,action,now)?;
    if receipt.status!=model::CommitStatus::Uncertain{project_stop_queues(store,path,&targets,now)?;}
    Ok(receipt)
}
