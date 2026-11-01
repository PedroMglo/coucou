//! Native observations survive the accounting/final-receipt transaction gap.
//! Recovery only republishes authenticated evidence; it never dispatches IO.
use super::*;
use crate::pet_runtime::{steps::{Assessment,Recovery,RecoveryStatus},model::Record};

#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Evidence{schema_version:u32,scope:Scope,attempt:Option<Attempt>,result:Receipt,recorded_at:u64}
#[derive(Serialize,Deserialize)]#[serde(deny_unknown_fields)]
struct EvidenceEnvelope{body:Evidence,tag:String}
fn result_bytes(result:&Receipt)->Result<u64>{Ok(serde_json::to_vec(result).map_err(|_|Failure::Invalid)?.len()as u64)}
fn evidence_message(body:&Evidence)->Result<Vec<u8>>{
    let mut bytes=b"coucou-native-goal-step-evidence-v1\0".to_vec();
    bytes.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(bytes)
}
fn same<T:Serialize>(a:&T,b:&T)->bool{model::fingerprint(a).ok().zip(model::fingerprint(b).ok()).is_some_and(|(a,b)|a==b)}
impl Evidence{
    fn valid(&self)->bool{
        if self.schema_version!=1||!self.scope.valid()||self.recorded_at>8_640_000_000_000_000
            ||!self.result.valid_for(&self.scope,self.recorded_at)
            ||!result_bytes(&self.result).is_ok_and(|n|n<=self.scope.max_output_bytes){return false;}
        match &self.attempt{
            Some(a)=>a.valid()&&a.phase==APhase::Started&&a.kind==crate::pet_runtime::effect_budget::Kind::Tool
                &&a.request_id==self.scope.request_id&&a.input_hash==model::fingerprint(&self.scope).unwrap_or_default()
                &&a.output_reserved==self.scope.max_output_bytes&&a.started_at.is_some_and(|n|n<=self.result.finished_at),
            None=>self.result.observation.delivery==crate::spotify::goal::Delivery::NotDispatched&&!self.result.observation.effect_verified,
        }
    }
}
fn budget_attempt(record:&StepRecord,main:&Grant)->Result<Option<Attempt>>{
    let found=main.budget_usage.as_ref().and_then(|b|b.attempts.iter().find(|a|a.request_id==record.scope.request_id||record.attempt.as_ref().is_some_and(|r|r.id==a.id))).cloned();
    match (&record.attempt,&found){
        (None,None)=>{},
        (Some(_),None)if record.phase==Phase::Reserved=>{},
        (Some(original),Some(current))=>{
            if !original.same_request(current)||original.started_at.is_some_and(|n|current.started_at!=Some(n)){return Err(Failure::Invalid);}
            match record.phase{
                Phase::Reserved if !matches!(current.phase,APhase::Reserved|APhase::Started)=>return Err(Failure::Invalid),
                Phase::Started if current.phase==APhase::Reserved=>return Err(Failure::Invalid),
                Phase::Finished if original!=current=>return Err(Failure::Invalid),
                _=>{},
            }
        },
        _=>return Err(Failure::RecoveryRequired),
    }
    Ok(found)
}
impl Authority{
    fn evidence_name(scope:&Scope)->Result<String>{Ok(format!("{}.evidence.json",scope.slot().map_err(|_|Failure::Invalid)?))}
    fn evidence_load(&self,record:&StepRecord,current:Option<&Attempt>)->Result<Option<Evidence>>{
        let Some(dir)=self.step_directory(false)?else{return Ok(None);};
        let Some((raw,_))=dir.bytes(&Self::evidence_name(&record.scope)?,STEP_BYTES).map_err(|_|Failure::Unsafe)?else{return Ok(None);};
        let e:EvidenceEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if !e.body.valid()||e.body.scope!=record.scope{return Err(Failure::Invalid);}
        hmac::verify(&self.key,&evidence_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        match (&e.body.attempt,current){
            (None,None)if matches!(record.phase,Phase::Approved|Phase::Finished)=>{},
            (Some(a),Some(b))if matches!(record.phase,Phase::Started|Phase::Finished)&&a.same_request(b)&&a.started_at==b.started_at=>{
                if b.phase==APhase::Settled&&(b.accepted_output!=Some(result_bytes(&e.body.result)?)||b.finished_at.is_none_or(|n|n<e.body.result.finished_at)){return Err(Failure::Invalid);}
            },
            _=>return Err(Failure::Invalid),
        }
        if record.result.as_ref().is_some_and(|r|!same(r,&e.body.result)){return Err(Failure::Invalid);}
        self.fence()?;Ok(Some(e.body))
    }
    fn evidence_stabilize(&self,evidence:&Evidence)->Result<()>{
        let dir=self.step_directory(false)?.ok_or(Failure::Missing)?;
        let name=Self::evidence_name(&evidence.scope)?;
        dir.open_file(&name,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;
        dir.durable().map_err(|_|Failure::Uncertain)?;
        let(raw,_)=dir.bytes(&name,STEP_BYTES).map_err(|_|Failure::Changed)?.ok_or(Failure::Changed)?;
        let e:EvidenceEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if !same(&e.body,evidence){return Err(Failure::Changed);}
        hmac::verify(&self.key,&evidence_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        self.fence()
    }
    /// Only the native controller supplies this observation. IPC receives references.
    pub(in crate::pet_runtime)fn record_step_result(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,attempt:Option<&Attempt>,result:&Receipt,now:u64)->Result<()>{
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,false)?;
        let record=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
        let current=budget_attempt(&record,&main)?;
        if record.scope!=*scope||now<record.updated_at{return Err(Failure::Invalid);}
        let native=match(attempt,current.as_ref()){
            (Some(a),Some(b))if record.phase==Phase::Started&&b.phase==APhase::Started&&a.same_request(b)=>Some(b.clone()),
            (None,None)if record.phase==Phase::Approved=>None,
            _=>return Err(Failure::RecoveryRequired),
        };
        let evidence=Evidence{schema_version:1,scope:scope.clone(),attempt:native,result:result.clone(),recorded_at:now};
        if !evidence.valid(){return Err(Failure::Invalid);}
        if let Some(old)=self.evidence_load(&record,current.as_ref())?{
            if old.scope!=evidence.scope||old.attempt!=evidence.attempt||!same(&old.result,&evidence.result){return Err(Failure::RecoveryRequired);}
            return self.evidence_stabilize(&old);
        }
        let bytes=serde_json::to_vec(&EvidenceEnvelope{tag:encode(hmac::sign(&self.key,&evidence_message(&evidence)?).as_ref()),body:evidence.clone()}).map_err(|_|Failure::Invalid)?;
        if bytes.len()>STEP_BYTES{return Err(Failure::Capacity);}
        let dir=self.step_directory(true)?.ok_or(Failure::Unavailable)?;let temp=format!(".step-evidence-{}",nonce().map_err(|_|Failure::Unavailable)?);
        dir.write_new(&temp,&bytes).map_err(|_|Failure::Unavailable)?;self.fence()?;
        #[cfg(test)]if matches!(self.fault,Fault::BeforeStepEvidencePublish){return Err(Failure::Unavailable);}
        dir.rename(&temp,&dir,&Self::evidence_name(scope)?,true).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(self.fault,Fault::AfterStepEvidencePublish){return Err(Failure::Uncertain);}
        self.evidence_stabilize(&evidence)
    }
    pub(in crate::pet_runtime::authority_store)fn step_reconciliation_unlocked(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<(Assessment,Option<Receipt>,Option<String>)>>{
        let main=self.load(spec,proof)?;let Some(record)=self.step_load(spec,proof,plan,step)?else{return Ok(None);};
        let current=budget_attempt(&record,&main)?;let evidence=self.evidence_load(&record,current.as_ref())?;
        let status=if record.phase==Phase::Finished{RecoveryStatus::Finished}else if evidence.is_some(){RecoveryStatus::ResultRecorded}else{match record.phase{Phase::Approved=>RecoveryStatus::Prepared,Phase::Reserved=>RecoveryStatus::Reserved,_=>RecoveryStatus::ResultMissing}};
        let recovery=Recovery{status,budget_phase:current.map(|a|a.phase.into()),recorded_effect_verified:record.result.as_ref().or(evidence.as_ref().map(|e|&e.result)).map(|r|r.observation.effect_verified),reconciliation_available:record.phase!=Phase::Finished&&evidence.is_some(),accounting_frozen:false,recorded_result:if record.result.is_none(){evidence.as_ref().map(|e|e.result.clone())}else{None}};
        let observed=record.result.clone().or_else(||evidence.as_ref().map(|e|e.result.clone()));let evidence_hash=evidence.as_ref().map(model::fingerprint).transpose().map_err(|_|Failure::Invalid)?;
        self.fence()?;Ok(Some((Assessment{record,recovery},observed,evidence_hash)))
    }
    pub(super)fn step_assessment_unlocked(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<Assessment>>{
        Ok(self.step_reconciliation_unlocked(spec,proof,plan,step)?.map(|(a,_,_)|a))
    }
    pub(in crate::pet_runtime)fn read_step_assessment(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<Assessment>>{
        let _lock=self.lock()?;self.step_assessment_unlocked(spec,proof,plan,step)
    }
    /// Reconciles only existing native evidence, including after pause/restart.
    #[cfg(test)]pub(in crate::pet_runtime)fn recover_step(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,now:u64)->Result<Assessment>{
        let _lock=self.lock()?;self.recover_step_unlocked(spec,proof,lease,scope,now)
    }
    /// Production calls hold the original goal catalog guard through this
    /// authority transaction. A signed continuation freezes the charged grant.
    pub(in crate::pet_runtime)fn recover_step_for_goal(&self,goal:&Record,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,now:u64)->Result<Assessment>{
        let _lock=self.lock()?;self.budget_body(spec,proof,lease,now,false)?;
        if self.accounting_frozen_unlocked(goal,spec)?{
            let mut assessment=self.step_assessment_unlocked(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
            if assessment.record.scope!=*scope{return Err(Failure::Changed);}
            assessment.recovery.freeze_accounting();self.fence()?;return Ok(assessment);
        }
        self.recover_step_unlocked(spec,proof,lease,scope,now)
    }
    pub(in crate::pet_runtime)fn read_step_assessment_for_goal(&self,goal:&Record,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str,step:&str)->Result<Option<Assessment>>{
        let _lock=self.lock_shared()?;let mut assessment=self.step_assessment_unlocked(spec,proof,plan,step)?;
        if self.accounting_frozen_unlocked(goal,spec)?{if let Some(value)=&mut assessment{value.recovery.freeze_accounting();}}
        self.fence()?;Ok(assessment)
    }
    fn recover_step_unlocked(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&Scope,now:u64)->Result<Assessment>{
        let mut main=self.budget_body(spec,proof,lease,now,false)?;
        let mut record=self.step_load(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)?;
        if record.scope!=*scope||now<record.updated_at{return Err(Failure::Invalid);}
        let current=budget_attempt(&record,&main)?;let evidence=self.evidence_load(&record,current.as_ref())?;
        if record.phase!=Phase::Finished{
            let evidence=evidence.ok_or(Failure::RecoveryRequired)?;
            if now<evidence.recorded_at{return Err(Failure::Invalid);}
            self.evidence_stabilize(&evidence)?;
            if let Some(attempt)=current.as_ref(){
                if attempt.phase==APhase::Started{
                    let attempt=main.budget_usage.as_mut().and_then(|b|b.attempts.iter_mut().find(|a|a.id==attempt.id)).ok_or(Failure::Missing)?;
                    attempt.phase=APhase::Settled;attempt.accepted_output=Some(result_bytes(&evidence.result)?);attempt.finished_at=Some(now);
                    main.updated_at=now;main.revision=nonce().map_err(|_|Failure::Unavailable)?;
                    #[cfg(test)]if matches!(self.fault,Fault::BeforeStepBudgetFinish){return Err(Failure::Unavailable);}
                    self.publish(main,false)?;
                    #[cfg(test)]if matches!(self.fault,Fault::AfterStepBudgetFinish){return Err(Failure::Uncertain);}
                    main=self.load(spec,proof)?;
                }else{self.stabilize(&main)?;}
            }else{self.stabilize(&main)?;}
            let settled=budget_attempt(&record,&main)?;record.phase=Phase::Finished;record.attempt=settled;
            record.result=Some(evidence.result);record.updated_at=now;
        }else{self.stabilize(&main)?;}
        self.step_publish(&record,false)?;
        self.step_assessment_unlocked(spec,proof,&scope.plan_id,&scope.step_id)?.ok_or(Failure::Missing)
    }
}

// Authenticate the original typed artifact before retirement uses its identity
// as a negative fact. This scope is data, never an executable recovered permit.
impl Authority{
    pub(in crate::pet_runtime::authority_store)fn retirement_evidence_scope(&self,raw:&[u8])->Result<Scope>{
        let e:EvidenceEnvelope=serde_json::from_slice(raw).map_err(|_|Failure::Invalid)?;
        if !e.body.valid(){return Err(Failure::Invalid);}
        hmac::verify(&self.key,&evidence_message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;Ok(e.body.scope)
    }
}
