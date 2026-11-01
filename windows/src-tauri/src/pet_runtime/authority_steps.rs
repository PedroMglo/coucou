//! Signed single-use step records share the native grant lock/key, but have
//! their own directory and message domain. Ambiguous publication never replays IO.
use super::*;
use crate::pet_runtime::{leases::Lease,effect_budget::{Attempt,Phase as APhase},steps::{self,StepRecord,Phase,Receipt,Scope}};
#[path="authority_step_recovery.rs"]mod recovery;
#[path="authority_recipes.rs"]mod recipes;
pub(in crate::pet_runtime)use recipes::CompletionCertified;
const STEP_PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-native-goal-steps\"}\n";
const STEP_BYTES:usize=64*1024;
#[derive(Serialize,Deserialize)]#[serde(deny_unknown_fields)]struct Envelope{body:StepRecord,tag:String}
fn message(record:&StepRecord)->Result<Vec<u8>>{
    let mut bytes=match record.scope.schema_version{1=>b"coucou-native-goal-step-state-v1\0".to_vec(),2=>b"coucou-native-generated-step-state-v1\0".to_vec(),_=>return Err(Failure::Invalid)};bytes.extend(serde_json::to_vec(record).map_err(|_|Failure::Invalid)?);Ok(bytes)
}
impl Authority{
    pub(in crate::pet_runtime::authority_store)fn step_directory(&self,create:bool)->Result<Option<Dir>>{
        self.fence()?;let path=self.dir.path.parent().ok_or(Failure::Unsafe)?.join("pet-runtime-step-consent-v1");
        protocol_directory(&path,STEP_PROTOCOL,create).map_err(|_|Failure::Unsafe)
    }
    pub(super)fn step_load(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<StepRecord>>{
        let main=self.load(spec,proof)?;let Some(lease)=main.lease.as_ref()else{return Err(Failure::Missing);};
        let Some(dir)=self.step_directory(false)?else{return Ok(None);};let slot=steps::slot(&spec.decision_id,plan,step).map_err(|_|Failure::Invalid)?;
        let Some((raw,_))=dir.bytes(&format!("{slot}.json"),STEP_BYTES).map_err(|_|Failure::Unsafe)?else{return Ok(None);};
        let envelope:Envelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if !envelope.body.valid()||envelope.body.scope.plan_id!=plan||envelope.body.scope.step_id!=step||!envelope.body.scope.belongs(spec,lease){return Err(Failure::Invalid);}
        if let Some(reference)=&envelope.body.scope.generated_plan{Self::generated_acknowledged(&main,spec,reference)?;}
        hmac::verify(&self.key,&message(&envelope.body)?,&decode(&envelope.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        self.fence()?;Ok(Some(envelope.body))
    }
    fn step_publish(&self,record:&StepRecord,new:bool)->Result<()>{
        if !record.valid(){return Err(Failure::Invalid);}
        let dir=self.step_directory(true)?.ok_or(Failure::Unavailable)?;
        let bytes=serde_json::to_vec(&Envelope{body:record.clone(),tag:encode(hmac::sign(&self.key,&message(record)?).as_ref())}).map_err(|_|Failure::Invalid)?;
        if bytes.len()>STEP_BYTES{return Err(Failure::Capacity);}
        let temp=format!(".step-{}",nonce().map_err(|_|Failure::Unavailable)?);dir.write_new(&temp,&bytes).map_err(|_|Failure::Unavailable)?;self.fence()?;
        #[cfg(test)]if matches!(self.fault,Fault::BeforeStepPublish)||(record.phase==Phase::Finished&&matches!(self.fault,Fault::BeforeStepResultPublish)){return Err(Failure::Unavailable);}
        dir.rename(&temp,&dir,&format!("{}.json",record.scope.slot().map_err(|_|Failure::Invalid)?),new).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(self.fault,Fault::AfterStepPublish)||(record.phase==Phase::Finished&&matches!(self.fault,Fault::AfterStepResultPublish)){return Err(Failure::Uncertain);}
        dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;
        let (raw,_)=dir.bytes(&format!("{}.json",record.scope.slot().map_err(|_|Failure::Invalid)?),STEP_BYTES).map_err(|_|Failure::Uncertain)?.ok_or(Failure::Uncertain)?;
        if raw!=bytes{return Err(Failure::Uncertain);}Ok(())
    }
    pub(in crate::pet_runtime)fn issue_step(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,approved:&steps::Approved,now:u64)->Result<Scope>{
        let scope=approved.scope();if !scope.belongs(spec,lease)||now<scope.approved_at||now>=scope.deadline_at||spec.budget.max_tool_calls==0{return Err(Failure::Invalid);}
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,true)?;
        if let Some(reference)=&scope.generated_plan{Self::generated_acknowledged(&main,spec,reference)?;}
        if let Some(old)=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?{
            if old.scope!=*scope||old.phase!=Phase::Approved{return Err(Failure::RecoveryRequired);}
            self.step_publish(&old,false)?;return Ok(old.scope);
        }
        let record=StepRecord{schema_version:1,scope:scope.clone(),phase:Phase::Approved,updated_at:now,attempt:None,result:None};self.step_publish(&record,true)?;Ok(scope.clone())
    }
    pub(in crate::pet_runtime)fn read_step(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<StepRecord>>{
        let _lock=self.lock()?;self.step_load(spec,proof,plan,step)
    }
    pub(in crate::pet_runtime)fn reserve_step(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,attempt:&Attempt,now:u64)->Result<()>{
        let _lock=self.lock()?;self.budget_body(spec,proof,lease,now,true)?;
        let mut record=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
        if record.scope!=*scope||record.phase!=Phase::Approved||now<record.updated_at||now>=scope.deadline_at{return Err(Failure::RecoveryRequired);}
        record.phase=Phase::Reserved;record.attempt=Some(attempt.clone());record.updated_at=now;self.step_publish(&record,false)
    }
    pub(in crate::pet_runtime)fn start_step(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,attempt:&Attempt,now:u64)->Result<()>{
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,true)?;
        let stored=main.budget_usage.as_ref().and_then(|l|l.attempts.iter().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
        let mut record=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
        if record.scope!=*scope||record.phase!=Phase::Reserved||stored.phase!=APhase::Started||!stored.same_request(attempt)||record.attempt.as_ref().is_none_or(|a|!a.same_request(attempt))||now<record.updated_at{return Err(Failure::RecoveryRequired);}
        record.phase=Phase::Started;record.attempt=Some(stored.clone());record.updated_at=now;self.step_publish(&record,false)
    }
    pub(in crate::pet_runtime)fn finish_step(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,attempt:Option<&Attempt>,result:Receipt,now:u64)->Result<StepRecord>{
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,false)?;
        let mut record=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
        if record.scope!=*scope||now<record.updated_at{return Err(Failure::Invalid);}
        if record.phase==Phase::Finished{self.step_publish(&record,false)?;return Ok(record);}
        let settled=match attempt{
            Some(attempt)=>{
                let stored=main.budget_usage.as_ref().and_then(|l|l.attempts.iter().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
                if record.phase!=Phase::Started||!stored.same_request(attempt)||!matches!(stored.phase,APhase::Settled|APhase::Uncertain)||record.attempt.as_ref().is_none_or(|a|!a.same_request(attempt)){return Err(Failure::RecoveryRequired);}
                Some(stored.clone())
            },
            None=>{if record.phase!=Phase::Approved||result.observation.delivery!=crate::spotify::goal::Delivery::NotDispatched{return Err(Failure::RecoveryRequired);}None},
        };
        record.phase=Phase::Finished;record.attempt=settled;record.result=Some(result);record.updated_at=now;self.step_publish(&record,false)?;Ok(record)
    }
}

impl Authority{
    /// Caller holds the shared/exclusive native grant guard. Every supported
    /// artifact is decoded and authenticated with its original message domain;
    /// arbitrary JSON identities are never accepted as proof of another owner.
    pub(in crate::pet_runtime::authority_store)fn retirement_artifact_decision(&self,name:&str,raw:&[u8])->Result<String>{
        let canonical=|suffix:&str|name.strip_suffix(suffix).filter(|v|decode(v).is_some());
        let staged=|prefix:&str|name.strip_prefix(prefix).is_some_and(model::token);
        if canonical(".verification.json").is_some()||staged(".verification-"){
            let(decision,slot)=self.retirement_verification_identity(raw)?;if canonical(".verification.json").is_some_and(|v|v!=slot){return Err(Failure::Invalid);}return Ok(decision);
        }
        if canonical(".recipe.json").is_some()||staged(".recipe-"){
            let scope=self.retirement_recipe_scope(raw)?;if canonical(".recipe.json").is_some_and(|v|scope.slot().ok().as_deref()!=Some(v)){return Err(Failure::Invalid);}return Ok(scope.decision_id);
        }
        if canonical(".evidence.json").is_some()||staged(".step-evidence-"){
            let scope=self.retirement_evidence_scope(raw)?;if canonical(".evidence.json").is_some_and(|v|scope.slot().ok().as_deref()!=Some(v)){return Err(Failure::Invalid);}return Ok(scope.decision_id);
        }
        if canonical(".json").is_some()||staged(".step-"){
            let e:Envelope=serde_json::from_slice(raw).map_err(|_|Failure::Invalid)?;
            if !e.body.valid()||canonical(".json").is_some_and(|v|e.body.scope.slot().ok().as_deref()!=Some(v)){return Err(Failure::Invalid);}
            hmac::verify(&self.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;return Ok(e.body.scope.decision_id);
        }Err(Failure::RecoveryRequired)
    }
}
