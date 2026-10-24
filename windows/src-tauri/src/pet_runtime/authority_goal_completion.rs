//! Final native observations are immutable evidence. Only a verified, exact
//! certificate can cross the goal-store completion transaction boundary.
use super::*;
use crate::pet_runtime::{criteria::{Verification,CompletionRef},model::{Record,State},lease_controller::recipe_controller::Observed};
const MAX_VERIFICATION:usize=64*1024;
#[derive(Serialize,Deserialize)]#[serde(deny_unknown_fields)]struct Envelope{body:Verification,tag:String}
fn message(body:&Verification)->Result<Vec<u8>>{let mut v=b"coucou-native-goal-verification-v1\0".to_vec();v.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(v)}
pub(in crate::pet_runtime)struct Certified<'a>{authority:&'a Authority,spec:AdmissionSpec,proof:ConsentProof,scope:RecipeScope,report:Verification}
impl Authority{
    pub(super)fn verification_load_unlocked(&self,spec:&AdmissionSpec,proof:&ConsentProof,scope:&RecipeScope)->Result<Option<Verification>>{
        if scope.verification.is_none(){return Ok(None);}let stored=self.recipe_load(spec,proof,&scope.plan_id)?.ok_or(Failure::Missing)?;
        if stored!=*scope{return Err(Failure::Invalid);}let dir=self.step_directory(false)?.ok_or(Failure::Missing)?;
        let Some((raw,_))=dir.bytes(&format!("{}.verification.json",scope.slot().map_err(|_|Failure::Invalid)?),MAX_VERIFICATION).map_err(|_|Failure::Unsafe)?else{return Ok(None);};
        let e:Envelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if !e.body.valid_for(scope)||e.body.spec_hash!=spec.intent_hash{return Err(Failure::Invalid);}
        hmac::verify(&self.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        for s in &scope.steps{let saved=self.step_assessment_unlocked(spec,proof,&s.plan_id,&s.step_id)?.ok_or(Failure::Missing)?;
            if saved.record.scope!=*s||saved.record.phase!=Phase::Finished||saved.record.result.as_ref().is_none_or(|r|!r.observation.effect_verified||!r.observation.scope_current){return Err(Failure::RecoveryRequired);}}
        self.fence()?;Ok(Some(e.body))
    }
    fn verification_stabilize(&self,scope:&RecipeScope,report:&Verification)->Result<()>{
        let dir=self.step_directory(false)?.ok_or(Failure::Missing)?;let name=format!("{}.verification.json",scope.slot().map_err(|_|Failure::Invalid)?);
        dir.open_file(&name,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;dir.durable().map_err(|_|Failure::Uncertain)?;
        let(raw,_)=dir.bytes(&name,MAX_VERIFICATION).map_err(|_|Failure::Changed)?.ok_or(Failure::Changed)?;let e:Envelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if e.body.content_hash().ok()!=report.content_hash().ok(){return Err(Failure::Changed);}hmac::verify(&self.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;self.fence()
    }
    pub(in crate::pet_runtime)fn record_verification(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,scope:&RecipeScope,observed:&Observed,now:u64)->Result<Verification>{
        let report=observed.report();if !report.valid_for(scope)||report.recorded_at>now{return Err(Failure::Invalid);}
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,report.criteria_verified)?;
        if !scope.belongs(spec,lease)||report.spec_hash!=spec.intent_hash||main.lease.as_ref()!=Some(lease){return Err(Failure::Invalid);}
        if let Some(old)=self.verification_load_unlocked(spec,proof,scope)?{self.verification_stabilize(scope,&old)?;return Ok(old);}
        // Every original step must have a verified native receipt; snapshots
        // cannot bypass an uncertain or missing operation result.
        for s in &scope.steps{let saved=self.step_assessment_unlocked(spec,proof,&s.plan_id,&s.step_id)?.ok_or(Failure::Missing)?;
            if saved.record.scope!=*s||saved.record.phase!=Phase::Finished||saved.record.result.as_ref().is_none_or(|r|!r.observation.effect_verified||!r.observation.scope_current){return Err(Failure::RecoveryRequired);}}
        let dir=self.step_directory(false)?.ok_or(Failure::Missing)?;let bytes=serde_json::to_vec(&Envelope{body:report.clone(),tag:encode(hmac::sign(&self.key,&message(report)?).as_ref())}).map_err(|_|Failure::Invalid)?;
        if bytes.len()>MAX_VERIFICATION{return Err(Failure::Capacity);}let temp=format!(".verification-{}",nonce().map_err(|_|Failure::Unavailable)?);dir.write_new(&temp,&bytes).map_err(|_|Failure::Unavailable)?;self.fence()?;
        #[cfg(test)]if matches!(self.fault,Fault::BeforeVerificationPublish){return Err(Failure::Unavailable);}
        dir.rename(&temp,&dir,&format!("{}.verification.json",scope.slot().map_err(|_|Failure::Invalid)?),true).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(self.fault,Fault::AfterVerificationPublish){return Err(Failure::Uncertain);}
        self.verification_stabilize(scope,report)?;Ok(report.clone())
    }
    pub(in crate::pet_runtime)fn certify_completion(&self,spec:&AdmissionSpec,proof:&ConsentProof,scope:&RecipeScope,goal:&Record)->Result<Certified<'_>>{
        let _lock=self.lock()?;let report=self.verification_load_unlocked(spec,proof,scope)?.ok_or(Failure::Missing)?;
        if !report.criteria_verified{return Err(Failure::RecoveryRequired);}self.verification_stabilize(scope,&report)?;
        verify_goal_source(goal,&report)?;Ok(Certified{authority:self,spec:spec.clone(),proof:proof.clone(),scope:scope.clone(),report})
    }
    pub(in crate::pet_runtime)fn verify_completed_goal(&self,goal:&Record)->Result<()>{
        goal.validate().map_err(|_|Failure::Invalid)?;if goal.state!=State::Completed{return Err(Failure::Invalid);}
        let row=goal.admissions.last().ok_or(Failure::Missing)?;let proof=row.proof.as_ref().ok_or(Failure::Missing)?;
        let scope=self.read_recipe(&row.spec,proof,&goal.completion.as_ref().ok_or(Failure::Missing)?.plan_id)?.ok_or(Failure::Missing)?;
        self.certify_completion(&row.spec,proof,&scope,goal).map(|_|())
    }
}
fn verify_goal_source(goal:&Record,report:&Verification)->Result<()>{
    goal.validate().map_err(|_|Failure::Invalid)?;
    let original=if goal.state==State::Completed{
        if goal.completion.as_ref()!=Some(&report.reference){return Err(Failure::Invalid);}goal.source_before_completion(report.source_sequence,report.source_updated_at).map_err(|_|Failure::Invalid)?
    }else{goal.clone()};
    if original.state!=State::Leased||original.id!=report.reference.goal_id||original.spec.owner.pet_id!=report.reference.pet_id
        ||original.revision!=report.reference.source_revision||original.sequence!=report.source_sequence||original.updated_at!=report.source_updated_at
        ||model::fingerprint(&original).ok().as_ref()!=Some(&report.source_record_hash)||model::fingerprint(&original.spec).ok().as_ref()!=Some(&report.spec_hash)
        ||original.spec.criteria.len()!=report.results.len()||original.spec.criteria.iter().zip(&report.results).any(|(text,r)|model::fingerprint(text).ok().as_ref()!=Some(&r.criterion_hash)){return Err(Failure::Invalid);}Ok(())
}
impl Certified<'_>{
    pub(in crate::pet_runtime)fn reference(&self)->&CompletionRef{&self.report.reference}
    pub(in crate::pet_runtime)fn current(&self,goal:&Record,now:u64)->Result<()>{
        let _lock=self.authority.lock()?;let report=self.authority.verification_load_unlocked(&self.spec,&self.proof,&self.scope)?.ok_or(Failure::Missing)?;
        if !report.criteria_verified||report.reference!=self.report.reference{return Err(Failure::Changed);}verify_goal_source(goal,&report)?;
        if goal.state!=State::Completed{let lease=goal.admissions.last().and_then(|r|r.lease.as_ref()).ok_or(Failure::Missing)?;self.authority.budget_body(&self.spec,&self.proof,lease,now,true)?;}
        self.authority.fence()
    }
}
