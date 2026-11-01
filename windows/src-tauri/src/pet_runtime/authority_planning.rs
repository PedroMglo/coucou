//! Planning evidence and round accounting use the original authority lock/key.
//! No model output can manufacture canonical publication or effect approval.
use super::*;
use crate::pet_runtime::{effect_budget::Phase,leases::Lease,planning,planning_journal,planning_output::{self,Body,Envelope,Outcome},results};

impl Authority {
    pub(in crate::pet_runtime)fn planning_context(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,request:planning_journal::Request,now:u64)->Result<crate::pet_runtime::planning_context::Context> {
        use crate::pet_runtime::{planning_context::{Context,Input,Remaining,Observation},effect_budget::{Attempt,Kind}};
        if !request.matches(spec,lease){return Err(Failure::Invalid);}
        let _lock=self.lock_shared()?;let body=self.budget_body(spec,proof,lease,now,true)?;
        let journal=body.planning_usage.as_ref().ok_or(Failure::Invalid)?;let policy=spec.planning_policy.as_ref().ok_or(Failure::Invalid)?;
        journal.require_next(policy).map_err(|e|if e==model::Error::LimitReached{Failure::Capacity}else{Failure::RecoveryRequired})?;
        let attempts=body.budget_usage.as_ref().map(|l|l.attempts.as_slice()).unwrap_or(&[]);
        let models=attempts.iter().filter(|a|a.kind==Kind::Model).count();let tools=attempts.iter().filter(|a|a.kind==Kind::Tool).count();let bytes=attempts.iter().map(Attempt::charge).sum::<u64>();
        let model_calls=spec.budget.max_model_calls.checked_sub(models as u16).ok_or(Failure::Invalid)?;let tool_calls=spec.budget.max_tool_calls.checked_sub(tools as u16).ok_or(Failure::Invalid)?;
        let output_bytes=spec.budget.max_output_bytes.checked_sub(bytes).ok_or(Failure::Invalid)?;
        // Keep room for at least one native tool receipt. Smaller remaining
        // model output caps spend the same original budget, never a new grant.
        let model_output_limit=u64::from(policy.limits.max_output_bytes).min(output_bytes.saturating_sub(2048));
        if model_calls==0||tool_calls==0||model_output_limit==0{return Err(Failure::Capacity);}
        let max_steps_this_round=u64::from(policy.limits.max_steps).min(u64::from(tool_calls)).min((output_bytes-model_output_limit)/2048) as u8;
        if max_steps_this_round==0{return Err(Failure::Capacity);}
        let mut observations=vec![];
        for r in &journal.rounds {if let Some(feedback)=&r.feedback {let report=self.planning_feedback_unlocked(spec,proof,feedback)?;observations.push(Observation{feedback:feedback.clone(),snapshot:report.snapshot});}}
        let (consecutive_no_progress,best_verified_criteria)=journal.progress();let remaining=Remaining {model_calls,tool_calls,output_bytes,deadline_at:lease.deadline_at,rounds:policy.limits.max_rounds.saturating_sub(journal.rounds.len() as u8),consecutive_no_progress,best_verified_criteria,max_steps_this_round,model_output_limit};
        Ok(Context {request,input:Input {remaining,observations}})
    }
    pub(in crate::pet_runtime)fn planning_root(&self)->Result<std::path::PathBuf> {
        self.fence()?;Ok(self.dir.path.parent().ok_or(Failure::Unsafe)?.join("pet-runtime-planning-v1"))
    }
    fn planning_output_current(&self,spec:&AdmissionSpec,body:&Grant,e:&Envelope)->Result<()> {
        let r=&e.body;
        if !r.valid(){return Err(Failure::Invalid);}
        hmac::verify(&self.key,&planning_output::message(r)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
        let lease=body.lease.as_ref().ok_or(Failure::Missing)?;
        let round=body.planning_usage.as_ref().and_then(|j|j.rounds.iter().find(|p|p.attempt_id==r.attempt_id)).ok_or(Failure::Missing)?;
        if r.goal_id!=spec.goal_id||r.decision_id!=spec.decision_id||r.lease_id!=lease.id||r.controller_id!=lease.controller_id
            ||r.source!=round.source||r.round_index!=round.index||r.request_id!=round.request_id||r.input_hash!=round.input_hash||round.phase!=Phase::Settled
            ||round.output.as_ref().is_some_and(|v|r.reference().ok().as_ref()!=Some(v)){return Err(Failure::Invalid);}
        self.fence()
    }
    pub(in crate::pet_runtime)fn verify_planning_output(&self,spec:&AdmissionSpec,proof:&ConsentProof,e:&Envelope)->Result<()> {
        // Receipt lookup takes the authority lock itself; finish it before
        // acquiring the grant transaction lock, then recheck native membership.
        let receipt=match results::lookup(self,spec,proof,&e.body.request_id)? {results::Lookup::Available{receipt}=>receipt,_=>return Err(Failure::RecoveryRequired)};
        if model::fingerprint(&receipt).map_err(|_|Failure::Invalid)?!=e.body.model_result_hash||receipt.attempt_id!=e.body.attempt_id||receipt.completed_at!=e.body.completed_at{return Err(Failure::Invalid);}
        let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;self.planning_output_current(spec,&body,e)
    }
    pub(in crate::pet_runtime)fn seal_planning_output(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,goal:&Record,request:&str,now:u64)->Result<Envelope> {
        let captured=planning_journal::Request::capture(goal,spec,lease,request).map_err(|_|Failure::Changed)?;
        let receipt=results::recover(self,spec,proof,request)?;
        let _lock=self.lock()?;let grant=self.budget_body(spec,proof,lease,now,false)?;
        let round=grant.planning_usage.as_ref().and_then(|j|j.rounds.iter().find(|r|r.request_id==request)).ok_or(Failure::Missing)?;
        if round.phase!=Phase::Settled||round.source!=captured.source||round.attempt_id!=receipt.attempt_id||round.input_hash!=receipt.input_hash{return Err(Failure::Changed);}
        if round.output.is_some(){return Err(Failure::RecoveryRequired);}
        let policy=spec.planning_policy.as_ref().ok_or(Failure::Invalid)?;
        let mut body=Body {schema_version:1,goal_id:spec.goal_id.clone(),decision_id:spec.decision_id.clone(),lease_id:lease.id.clone(),controller_id:lease.controller_id.clone(),
            round_index:round.index,request_id:request.into(),attempt_id:receipt.attempt_id.clone(),input_hash:receipt.input_hash.clone(),source:round.source.clone(),
            model_result_hash:model::fingerprint(&receipt).map_err(|_|Failure::Invalid)?,completed_at:receipt.completed_at,recorded_at:now,outcome:Outcome::Invalid};
        body.outcome=match planning::parse(goal,policy,&receipt.text) {
            Ok(planning::Parsed::Plan(p))=>{
                let attempts=grant.budget_usage.as_ref().map(|l|l.attempts.as_slice()).unwrap_or(&[]);
                let tools=attempts.iter().filter(|a|a.kind==crate::pet_runtime::effect_budget::Kind::Tool).count();let used=attempts.iter().map(crate::pet_runtime::effect_budget::Attempt::charge).sum::<u64>();
                if tools.saturating_add(p.steps.len())>usize::from(spec.budget.max_tool_calls)||used.saturating_add(p.steps.len() as u64*2048)>spec.budget.max_output_bytes {
                    Outcome::Blocked {reason:"O orçamento original já não cobre os passos propostos e os respetivos recibos nativos.".into()}
                }else{Outcome::Plan {candidate:planning_output::candidate(p,&body)}}
            },
            Ok(planning::Parsed::Blocked(reason))=>Outcome::Blocked {reason},
            Err(_)=>Outcome::Invalid,
        };
        if !body.valid(){return Err(Failure::Invalid);}
        let tag=encode(hmac::sign(&self.key,&planning_output::message(&body)?).as_ref());let e=Envelope{body,tag};self.planning_output_current(spec,&grant,&e)?;Ok(e)
    }
    pub(in crate::pet_runtime)fn ack_planning_output(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,e:&Envelope,now:u64)->Result<()> {
        if e.body.recorded_at>now{return Err(Failure::Invalid);}
        // ACK cannot precede durable canonical evidence publication.
        planning_output::publish(self,spec,proof,e)?;
        let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;self.planning_output_current(spec,&body,e)?;
        let round=body.planning_usage.as_mut().and_then(|j|j.rounds.iter_mut().find(|r|r.attempt_id==e.body.attempt_id)).ok_or(Failure::Missing)?;
        if round.output.is_some(){return self.stabilize(&body);}
        round.output=Some(e.body.reference()?);body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
    }
}
