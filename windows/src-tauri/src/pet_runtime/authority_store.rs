//! Private key and grant records; never export secret material to UI or models.
use super::{authority::*,admission::AdmissionSpec,model};
use crate::pet_runtime::{admission::Approved,model::{Record,State},store::{Dir,Guard,Stamp,nonce,protocol_directory}};
use ring::{hmac,rand::{SecureRandom,SystemRandom}};
use serde::{Serialize,Deserialize};
use std::{path::Path,os::{fd::AsRawFd,unix::fs::MetadataExt}};
const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-native-consent\"}\n";
const KEY:&str=".key.json";
const MAX_GRANT_BYTES:usize=128*1024;
#[path="authority_budget.rs"]mod budget;
#[path="authority_results.rs"]mod results;
#[path="authority_planning.rs"]mod planning;
#[path="authority_plan_commit.rs"]mod plan_commit;
pub(in crate::pet_runtime)use plan_commit::PlanCertified;
#[cfg(target_os="linux")]#[path="authority_steps.rs"]mod steps;
pub(in crate::pet_runtime)use steps::CompletionCertified;
#[derive(Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
struct KeyFile{schema_version:u32,key_id:String,key:String}
pub(super)struct Authority{dir:Dir,stamp:Stamp,lock_stamp:Stamp,key_id:String,key:hmac::Key,#[cfg(test)]pub(super)fault:Fault}
#[cfg(test)]#[derive(Clone,Copy,Default)]pub(super)enum Fault{#[default]None,BeforeGrantPublish,AfterGrantPublish,BeforeResultPublish,AfterResultPublish,BeforeStepPublish,AfterStepPublish,BeforeStepResultPublish,AfterStepResultPublish,BeforeStepEvidencePublish,AfterStepEvidencePublish,BeforeStepBudgetFinish,AfterStepBudgetFinish,BeforeRecipePublish,AfterRecipePublish,BeforeVerificationPublish,AfterVerificationPublish,BeforePlanningOutputPublish,AfterPlanningOutputPublish}
pub(super)struct Certified<'a>{authority:&'a Authority,approved:&'a Approved,proof:ConsentProof}
impl Authority{
    pub(super)fn existing(path:&Path)->Result<Self>{
        let dir=protocol_directory(path,PROTOCOL,false).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let(raw,stamp)=dir.bytes(KEY,512).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let(_,lock_stamp)=dir.bytes(".catalog.lock",0).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let f:KeyFile=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if f.schema_version!=1||!model::token(&f.key_id){return Err(Failure::Invalid);}let bytes=decode(&f.key).ok_or(Failure::Invalid)?;
        Ok(Self{dir,stamp,lock_stamp,key_id:f.key_id,key:hmac::Key::new(hmac::HMAC_SHA256,&bytes),#[cfg(test)]fault:Fault::None})
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
        self.dir.check().map_err(|_|Failure::Changed)?;
        if self.dir.bytes(".protocol.json",1024).map_err(|_|Failure::Changed)?.is_none_or(|(b,_)|b!=PROTOCOL)
            ||self.dir.bytes(KEY,512).map_err(|_|Failure::Changed)?.is_none_or(|(_,s)|s!=self.stamp)
            ||self.dir.bytes(".catalog.lock",0).map_err(|_|Failure::Changed)?.is_none_or(|(_,s)|s!=self.lock_stamp){return Err(Failure::Changed);}Ok(())
    }
    fn lock(&self)->Result<Guard>{
        self.fence()?;let file=self.dir.open_file(".catalog.lock",libc::O_RDWR).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        if unsafe{libc::flock(file.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0{return Err(if std::io::Error::last_os_error().kind()==std::io::ErrorKind::WouldBlock{Failure::Busy}else{Failure::Unavailable});}
        let guard=Guard(file);let current=self.dir.open_file(".catalog.lock",libc::O_RDONLY).map_err(|_|Failure::Changed)?.ok_or(Failure::Changed)?;
        let a=guard.0.metadata().map_err(|_|Failure::Changed)?;let b=current.metadata().map_err(|_|Failure::Changed)?;
        if a.dev()!=b.dev()||a.ino()!=b.ino(){return Err(Failure::Changed);}self.fence()?;Ok(guard)
    }
    fn proof(&self,spec:&AdmissionSpec)->Result<ConsentProof>{
        let planning=spec.schema_version==3;
        let mut proof=ConsentProof{schema_version:if planning{2}else{1},key_id:self.key_id.clone(),operation:if planning{Operation::GoalPlanning}else{Operation::ConversationRespond},tag:String::new()};
        proof.tag=encode(hmac::sign(&self.key,&consent_message(&proof,spec)?).as_ref());Ok(proof)
    }
    fn verify_proof(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<()>{
        if !proof.valid()||!proof.matches(spec){return Err(Failure::Invalid);}if proof.key_id!=self.key_id{return Err(Failure::WrongKey);}
        hmac::verify(&self.key,&consent_message(proof,spec)?,&decode(&proof.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)
    }
    fn load(&self,spec:&AdmissionSpec,proof:&ConsentProof)->Result<Grant>{
        self.verify_proof(spec,proof)?;
        let(raw,_)=self.dir.bytes(&format!("{}.json",spec.decision_id),MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
        let e:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        hmac::verify(&self.key,&grant_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        let b=e.body;if !b.valid()||b.budget_usage.as_ref().is_some_and(|u|!u.valid_for(spec))||b.planning_usage.as_ref().is_some_and(|u|!u.valid_for(spec,b.lease.as_ref(),b.budget_usage.as_ref()))||b.lease.as_ref().is_some_and(|l|!l.valid(spec))||b.decision_id!=spec.decision_id||b.goal_id!=spec.goal_id||b.operation!=proof.operation||b.spec_hash!=model::fingerprint(spec).map_err(|_|Failure::Invalid)?||!model::token(&b.revision)||b.approved_at!=spec.approved_at||b.updated_at<b.approved_at{return Err(Failure::Invalid);}Ok(b)
    }
    fn publish(&self,body:Grant,new:bool)->Result<()>{
        if !body.valid(){return Err(Failure::Invalid);}self.fence()?;let tag=encode(hmac::sign(&self.key,&grant_message(&body)?).as_ref());let bytes=serde_json::to_vec(&GrantEnvelope{body:body.clone(),tag}).map_err(|_|Failure::Invalid)?;
        if bytes.len()>MAX_GRANT_BYTES{return Err(Failure::Capacity);}
        let temp=format!(".grant-{}",nonce().map_err(|_|Failure::Unavailable)?);self.dir.write_new(&temp,&bytes).map_err(|_|Failure::Unavailable)?;
        #[cfg(test)]if matches!(self.fault,Fault::BeforeGrantPublish){return Err(Failure::Unavailable);}
        self.fence()?;self.dir.rename(&temp,&self.dir,&format!("{}.json",body.decision_id),new).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(self.fault,Fault::AfterGrantPublish){return Err(Failure::Uncertain);}
        self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
    pub(super)fn certify<'a>(&'a self,approved:&'a Approved,goal:&Record,now:u64)->Result<Certified<'a>>{
        approved.current(goal,now).map_err(|_|Failure::Expired)?;let _lock=self.lock()?;let spec=approved.spec();let proof=self.proof(spec)?;
        let file=format!("{}.json",spec.decision_id);
        if self.dir.open_file(&file,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.is_some(){
            let old=self.load(spec,&proof)?;if old.phase!=GrantPhase::Issued{return Err(Failure::Revoked);}
            self.dir.open_file(&file,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;
        }else{
            if self.dir.names().map_err(|_|Failure::Capacity)?.len()>=4093{return Err(Failure::Capacity);}
            self.publish(Grant{budget_usage:None,planning_usage:if spec.schema_version==3{Some(super::planning_journal::Ledger::empty())}else{None},schema_version:if spec.schema_version==3{4}else{1},decision_id:spec.decision_id.clone(),goal_id:spec.goal_id.clone(),operation:proof.operation,spec_hash:model::fingerprint(spec).map_err(|_|Failure::Invalid)?,phase:GrantPhase::Issued,revision:nonce().map_err(|_|Failure::Unavailable)?,approved_at:spec.approved_at,updated_at:spec.approved_at,lease:None,stop_reason:None,closed_at:None},true)?;
        }
        self.fence()?;Ok(Certified{authority:self,approved,proof})
    }
    pub(super)fn verify_queued(&self,goal:&Record,now:u64)->Result<()>{
        goal.validate().map_err(|_|Failure::Invalid)?;if goal.state!=State::Queued{return Err(Failure::NotQueued);}
        let row=goal.admissions.last().ok_or(Failure::Missing)?;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;
        let _lock=self.lock()?;let body=self.load(&row.spec,proof)?;if body.phase!=GrantPhase::Issued{return Err(Failure::Revoked);}
        if now<row.spec.approved_at||now>=row.spec.budget.deadline_at{return Err(Failure::Expired);}self.fence()
    }
    fn stabilize(&self,body:&Grant)->Result<()>{
        self.dir.open_file(&format!("{}.json",body.decision_id),libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
    fn active_unlocked(&self)->Result<Vec<super::leases::Lease>>{
        let mut active=vec![];
        for name in self.dir.names().map_err(|_|Failure::Capacity)?{
            if matches!(name.as_str(),".key.json"|".catalog.lock"|".protocol.json"){continue;}
            let staged=name.strip_prefix(".grant-").is_some_and(model::token);
            let canonical=name.strip_suffix(".json").filter(|s|model::token(s));
            if !staged&&canonical.is_none(){return Err(Failure::RecoveryRequired);}
            let(raw,_)=self.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
            let e:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::RecoveryRequired)?;
            if !e.body.valid()||canonical.is_some_and(|id|id!=e.body.decision_id){return Err(Failure::RecoveryRequired);}
            hmac::verify(&self.key,&grant_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
            if !staged&&matches!(e.body.phase,GrantPhase::Claimed|GrantPhase::Draining){active.push(e.body.lease.ok_or(Failure::Invalid)?);}
        }
        self.fence()?;Ok(active)
    }
    pub(super)fn claim(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease,now:u64)->Result<()>{
        if !lease.valid(spec)||now<lease.claimed_at||now>=lease.deadline_at{return Err(Failure::Expired);}
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;
        if self.active_unlocked()?.iter().any(|l|l.provider==lease.provider&&l!=lease){return Err(Failure::Busy);}
        if body.phase==GrantPhase::Claimed&&body.lease.as_ref()==Some(lease){return self.stabilize(&body);}
        if body.phase!=GrantPhase::Issued{return Err(if matches!(body.phase,GrantPhase::Claimed|GrantPhase::Draining){Failure::Busy}else{Failure::Revoked});}
        if self.active_unlocked()?.iter().any(|l|l.provider==lease.provider){return Err(Failure::Busy);}
        if now<body.updated_at{return Err(Failure::Invalid);}
        body.schema_version=if body.planning_usage.is_some(){4}else{2};body.phase=GrantPhase::Claimed;body.lease=Some(lease.clone());body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
    }
    pub(super)fn lease_current(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease,now:u64)->Result<()>{
        let _lock=self.lock()?;let body=self.load(spec,proof)?;
        if body.phase!=GrantPhase::Claimed||body.lease.as_ref()!=Some(lease){return Err(Failure::Revoked);}
        if now<lease.claimed_at||now>=lease.deadline_at{return Err(Failure::Expired);}self.fence()
    }
    pub(super)fn finish(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease,unstarted:bool,now:u64)->Result<Option<super::leases::StopReason>>{
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;
        if unstarted&&body.phase==GrantPhase::Issued&&body.lease.is_none()&&lease.valid(spec){body.schema_version=if body.planning_usage.is_some(){4}else{2};body.lease=Some(lease.clone());body.phase=GrantPhase::Claimed;}
        if body.lease.as_ref()!=Some(lease)||now<lease.claimed_at||now<body.updated_at||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
        if matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked){self.stabilize(&body)?;return Ok(body.stop_reason);}
        if !matches!(body.phase,GrantPhase::Claimed|GrantPhase::Draining){return Err(Failure::Invalid);}
        let reason=body.stop_reason;body.phase=if reason==Some(super::leases::StopReason::Cancel){GrantPhase::Revoked}else{GrantPhase::Closed};body.closed_at=Some(now);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;Ok(reason)
    }
    #[cfg(test)]pub(super)fn revoke(&self,spec:&AdmissionSpec,proof:&ConsentProof,now:u64)->Result<()>{self.request_stop(spec,proof,super::leases::StopReason::Cancel,now).map(|_|())}
    pub(super)fn request_stop(&self,spec:&AdmissionSpec,proof:&ConsentProof,reason:super::leases::StopReason,now:u64)->Result<Option<super::leases::Lease>>{
        if now==0||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
        let _lock=self.lock()?;let mut body=self.load(spec,proof)?;if now<body.updated_at{return Err(Failure::Invalid);}
        if matches!(body.phase,GrantPhase::Revoked|GrantPhase::Closed){self.stabilize(&body)?;return Ok(None);}
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
        self.approved.current(goal,now)?;let _lock=self.authority.lock().map_err(|_|model::Error::Conflict)?;
        if self.authority.load(self.approved.spec(),&self.proof).map_err(|_|model::Error::Conflict)?.phase!=GrantPhase::Issued{return Err(model::Error::Conflict);}self.authority.fence().map_err(|_|model::Error::Conflict)
    }
}

/// Revoke the independent consent record before persisting queued pause/cancel.
/// Replays are receipt-only and must never revoke a newer admission.
pub(super)fn mutate_with_authority(store:&impl crate::pet_runtime::Repository,path:&Path,id:&str,pet:&str,revision:&str,request:String,action:model::Action,now:u64)->model::Result<model::Receipt>{
    if matches!(action,model::Action::Pause|model::Action::Cancel){
        let goal=store.read_control(id,pet)?;
        if goal.replay(&request,&action)?{return store.mutate(id,pet,revision,request,action,now);}
        if goal.revision!=revision{return Err(model::Error::Conflict);}
        if matches!(goal.state,State::Queued|State::Leased|State::Stopping){if let Some(row)=goal.admissions.last(){if let Some(proof)=&row.proof{
            if let Some(lease)=&row.lease{super::lease_controller::signal_stop(lease);}
            let authority=Authority::existing(path).map_err(|_|model::Error::StorageUnavailable)?;
            let reason=if matches!(action,model::Action::Cancel){super::leases::StopReason::Cancel}else{super::leases::StopReason::Pause};
            let live=authority.request_stop(&row.spec,proof,reason,now).map_err(|_|model::Error::StorageUnavailable)?;
            if let Some(lease)=live{super::lease_controller::signal_stop(&lease);if row.lease.as_ref()!=Some(&lease){return Err(model::Error::Conflict);}}
        }}}
    }
    store.mutate(id,pet,revision,request,action,now)
}
