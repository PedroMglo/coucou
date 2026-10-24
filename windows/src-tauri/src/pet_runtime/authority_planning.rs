//! Planning evidence and round accounting use the original authority lock/key.
//! No model output can manufacture canonical publication or effect approval.
use super::*;
use crate::pet_runtime::{effect_budget::Phase,leases::Lease,planning,planning_journal,planning_output::{self,Body,Envelope,Outcome},results};

impl Authority {
    /// Read-only eligibility, not an IO permit. The actual round and budget are
    /// still reserved atomically at the original model HTTP boundary.
    pub(in crate::pet_runtime)fn preflight_planning(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,request:&planning_journal::Request,now:u64)->Result<()> {
        if !request.matches(spec,lease){return Err(Failure::Invalid);}
        let _lock=self.lock()?;let body=self.budget_body(spec,proof,lease,now,true)?;
        let journal=body.planning_usage.as_ref().ok_or(Failure::Invalid)?;let policy=spec.planning_policy.as_ref().ok_or(Failure::Invalid)?;
        journal.require_next(policy).map_err(|e|if e==model::Error::LimitReached{Failure::Capacity}else{Failure::RecoveryRequired})?;
        let used=body.budget_usage.as_ref().map(|b|b.attempts.iter().map(crate::pet_runtime::effect_budget::Attempt::charge).sum::<u64>()).unwrap_or(0);
        if used>=spec.budget.max_output_bytes{return Err(Failure::Capacity);}
        self.fence()
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
        let _lock=self.lock()?;let body=self.load(spec,proof)?;self.planning_output_current(spec,&body,e)
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
            Ok(planning::Parsed::Plan(p))=>Outcome::Plan {candidate:planning_output::candidate(p,&body)},
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
