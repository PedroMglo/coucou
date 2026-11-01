//! Original-authority prepare/CAS/ack transaction. Certificates are native
//! values; neither serialized references nor a UI acknowledgment can create one.
use super::*;
use crate::pet_runtime::{generated_plans::{self,Phase as PublicationPhase,Publication,Reference},leases::Lease,planning_output::{self,Envelope}};

pub(in crate::pet_runtime)struct PlanCertified<'a,'g> {
    gate:&'a crate::pet_runtime::lease_controller::PlanGate<'g>,
    authority:&'a Authority,spec:&'a AdmissionSpec,proof:&'a ConsentProof,lease:&'a Lease,
    reference:Reference,output:Envelope,
}
impl PlanCertified<'_,'_> {
    pub(in crate::pet_runtime)fn reference(&self)->&Reference {&self.reference}
    pub(in crate::pet_runtime)fn goal_id(&self)->&str {&self.spec.goal_id}
    pub(in crate::pet_runtime)fn output(&self)->&Envelope {&self.output}
    pub(in crate::pet_runtime)fn current(&self,goal:&Record,now:u64)->Result<()> {
        if !self.gate.matches(self.spec,self.lease)||!goal.generated_plans.contains(&self.reference)&&!self.gate.current(goal,now){return Err(Failure::Revoked);}
        self.authority.published_planning_output(self.spec,self.proof,&self.output)?;
        let _lock=self.authority.lock()?;let body=self.authority.budget_body(self.spec,self.proof,self.lease,now,false)?;
        let p=body.planning_usage.as_ref().and_then(|j|j.rounds.iter().find(|r|r.attempt_id==self.reference.output.attempt_id)).and_then(|r|r.publication.as_ref()).ok_or(Failure::Missing)?;
        if p.reference!=self.reference||goal.id!=self.spec.goal_id{return Err(Failure::Changed);}
        if goal.generated_plans.contains(&self.reference) {
            if !generated_plans::content_matches(goal,&self.reference,&self.output){return Err(Failure::Changed);}
        }else {
            if p.phase!=PublicationPhase::Prepared||body.phase!=GrantPhase::Claimed||now>=self.lease.deadline_at||now<self.reference.published_at
                ||goal.state!=State::Leased||goal.revision!=self.reference.source.revision||goal.sequence!=self.reference.source.sequence||goal.updated_at!=self.reference.source.updated_at
                ||model::fingerprint(goal).map_err(|_|Failure::Invalid)?!=self.reference.source.record_hash{return Err(Failure::Changed);}
        }
        self.authority.fence()
    }
}
impl Authority {
    /// Pure check against the already authenticated grant. Safe under its lock;
    /// it must not re-enter a result lookup or a canonical Store read.
    pub(in crate::pet_runtime)fn generated_acknowledged(body:&Grant,spec:&AdmissionSpec,reference:&Reference)->Result<()> {
        if spec.schema_version!=3||!reference.valid()||reference.decision_id!=spec.decision_id
            ||body.lease.as_ref().is_none_or(|l|!reference.source.matches(spec,l))
            ||!body.planning_usage.as_ref().is_some_and(|j|j.rounds.iter().any(|r|r.attempt_id==reference.output.attempt_id
                &&r.publication.as_ref().is_some_and(|p|p.phase==PublicationPhase::Acknowledged&&p.reference==*reference))){return Err(Failure::Invalid);}
        Ok(())
    }
    pub(in crate::pet_runtime)fn generated_candidate(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,goal:&Record,plan:&str)->Result<(Reference,planning_output::Candidate)> {
        self.verify_generated_goal(goal,false)?;
        let reference=goal.generated_plans.iter().find(|r|r.decision_id==spec.decision_id&&r.plan_id==plan).ok_or(Failure::Missing)?.clone();
        {
            let _lock=self.lock_shared()?;let body=self.load(spec,proof)?;
            if body.lease.as_ref()!=Some(lease){return Err(Failure::Changed);}Self::generated_acknowledged(&body,spec,&reference)?;
        }
        let output=planning_output::read(self,spec,proof,&reference.output.attempt_id)?.ok_or(Failure::RecoveryRequired)?;
        if !generated_plans::content_matches(goal,&reference,&output){return Err(Failure::Changed);}
        let planning_output::Outcome::Plan{candidate}=output.body.outcome else{return Err(Failure::Invalid);};Ok((reference,candidate))
    }
    pub(in crate::pet_runtime)fn generated_reference(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,request:&str,now:u64)->Result<Reference> {
        let _lock=self.lock()?;let body=self.budget_body(spec,proof,lease,now,false)?;
        body.planning_usage.as_ref().and_then(|j|j.rounds.iter().find(|r|r.request_id==request)).and_then(|r|r.publication.as_ref()).map(|p|p.reference.clone()).ok_or(Failure::Missing)
    }
    pub(in crate::pet_runtime)fn abandon_unpublished_plan(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,gate:&crate::pet_runtime::lease_controller::PlanGate<'_>,absent:&crate::pet_runtime::store::PlanAbsent<'_>,now:u64)->Result<()> {
        if !gate.matches(spec,lease){return Err(Failure::Invalid);}
        absent.with_current(|goal,reference| {
            self.verify_generated_goal(goal,true)?;
            let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;
            // Native drain/closure must already be recorded. Absence alone
            // cannot assert that workers or unknown IO have finished.
            if !matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked)||goal.id!=spec.goal_id{return Err(Failure::Revoked);}
            let p=body.planning_usage.as_mut().and_then(|j|j.rounds.iter_mut().find(|r|r.attempt_id==reference.output.attempt_id)).and_then(|r|r.publication.as_mut()).ok_or(Failure::Missing)?;
            if &p.reference!=reference||p.phase==PublicationPhase::Acknowledged{return Err(Failure::Changed);}
            if p.phase==PublicationPhase::Abandoned{return self.stabilize(&body);}
            p.phase=PublicationPhase::Abandoned;body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
        })
    }
    fn published_planning_output(&self,spec:&AdmissionSpec,proof:&ConsentProof,e:&Envelope)->Result<()> {
        let stored=planning_output::read(self,spec,proof,&e.body.attempt_id)?.ok_or(Failure::RecoveryRequired)?;
        if model::fingerprint(&stored).map_err(|_|Failure::Invalid)?!=model::fingerprint(e).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}
        Ok(())
    }
    pub(in crate::pet_runtime)fn prepare_generated_plan<'a,'g>(&'a self,spec:&'a AdmissionSpec,proof:&'a ConsentProof,lease:&'a Lease,goal:&Record,e:&Envelope,gate:&'a crate::pet_runtime::lease_controller::PlanGate<'g>,now:u64)->Result<PlanCertified<'a,'g>> {
        if !gate.matches(spec,lease)||!goal.generated_plans.iter().any(|r|r.output.attempt_id==e.body.attempt_id)&&!gate.current(goal,now){return Err(Failure::Revoked);}
        self.published_planning_output(spec,proof,e)?;
        let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;
        let round=body.planning_usage.as_mut().and_then(|j|j.rounds.iter_mut().find(|r|r.attempt_id==e.body.attempt_id)).ok_or(Failure::Missing)?;
        if round.output.as_ref()!=Some(&e.body.reference()?)||round.source!=e.body.source||goal.id!=spec.goal_id{return Err(Failure::Changed);}
        let reference=if let Some(p)=&round.publication {
            if p.phase==PublicationPhase::Abandoned||!generated_plans::output_matches(&p.reference,e){return Err(Failure::Changed);}
            let present=goal.generated_plans.contains(&p.reference);
            if present {
                if !generated_plans::content_matches(goal,&p.reference,e){return Err(Failure::Changed);}
            }else if p.phase!=PublicationPhase::Prepared||model::fingerprint(goal).map_err(|_|Failure::Invalid)?!=p.reference.source.record_hash {
                return Err(Failure::Changed);
            }
            let reference=p.reference.clone();self.stabilize(&body)?;reference
        }else {
            if body.phase!=GrantPhase::Claimed||now>=lease.deadline_at{return Err(Failure::Revoked);}
            let reference=generated_plans::prepare(goal,e,now).map_err(|_|Failure::Changed)?;
            let mut candidate=goal.clone();candidate.append_generated(reference.clone(),e).map_err(|_|Failure::Invalid)?;
            round.publication=Some(Publication {phase:PublicationPhase::Prepared,reference:reference.clone()});
            body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)?;reference
        };
        Ok(PlanCertified {gate,authority:self,spec,proof,lease,reference,output:e.clone()})
    }
    pub(in crate::pet_runtime)fn acknowledge_generated_plan(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,gate:&crate::pet_runtime::lease_controller::PlanGate<'_>,published:&crate::pet_runtime::store::PlanPublished<'_>,now:u64)->Result<()> {
        if !gate.matches(spec,lease){return Err(Failure::Invalid);}
        // The opaque proof owns the canonical Store readback boundary. Its
        // closure keeps goal-lock -> authority-lock ordering throughout ACK.
        published.with_current(|goal,reference| {
            self.verify_generated_goal(goal,true)?;
            let _lock=self.lock()?;let mut body=self.budget_body(spec,proof,lease,now,false)?;
            let p=body.planning_usage.as_mut().and_then(|j|j.rounds.iter_mut().find(|r|r.attempt_id==reference.output.attempt_id)).and_then(|r|r.publication.as_mut()).ok_or(Failure::Missing)?;
            if &p.reference!=reference||goal.id!=spec.goal_id||!reference.structurally_present(goal){return Err(Failure::Changed);}
            if p.phase==PublicationPhase::Acknowledged{return self.stabilize(&body);}
            p.phase=PublicationPhase::Acknowledged;body.updated_at=now;body.revision=nonce().map_err(|_|Failure::Unavailable)?;self.publish(body,false)
        })
    }
    /// Public/native readers must inspect every planning admission, even when
    /// JSON removed all generated references. Internal publication/control reads
    /// can inspect Prepared data but can never dispatch it.
    pub(in crate::pet_runtime)fn verify_generated_goal(&self,goal:&Record,allow_pending:bool)->Result<()> {
        // Do not let JSON bypass verification by deleting every planning field
        // or pretending to be a legacy draft. A claimed native planning grant
        // remains authoritative even when the canonical record omits it.
        {
            let _lock=self.lock_shared()?;
            for name in self.dir.names().map_err(|_|Failure::Unavailable)? {
            if self.historical_file(&name){continue;}
                let Some(decision)=name.strip_suffix(".json").filter(|s|model::token(s))else{continue;};
                let (raw,_)=self.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
                let envelope:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
                let body=&envelope.body;
                if !body.valid()||body.decision_id!=decision{return Err(Failure::Invalid);}
                hmac::verify(&self.key,&grant_message(body)?,&decode(&envelope.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
                if body.goal_id==goal.id&&body.operation==Operation::GoalPlanning&&body.lease.is_some()
                    &&!goal.admissions.iter().any(|row|row.spec.schema_version==3&&row.spec.decision_id==body.decision_id){return Err(Failure::Invalid);}
            }
        }
        for row in goal.admissions.iter().filter(|r|r.spec.schema_version==3) {
            let proof=row.proof.as_ref().ok_or(Failure::Missing)?;
            let body={let _lock=self.lock_shared()?;self.load(&row.spec,proof)?};
            let journal=body.planning_usage.as_ref().ok_or(Failure::Missing)?;
            let refs=goal.generated_plans.iter().filter(|r|r.decision_id==row.spec.decision_id).collect::<Vec<_>>();let mut expected=0;
            for round in &journal.rounds {
                if let Some(feedback)=&round.feedback{self.planning_feedback_report(&row.spec,proof,feedback)?;}
                let Some(p)=&round.publication else {continue;};
                let present=goal.generated_plans.contains(&p.reference);
                if p.phase==PublicationPhase::Prepared&&!allow_pending{return Err(Failure::RecoveryRequired);}
                if p.phase==PublicationPhase::Acknowledged&&!present{return Err(Failure::Invalid);}
                if p.phase==PublicationPhase::Abandoned&&present{return Err(Failure::Invalid);}
                if present {
                    let e=planning_output::read(self,&row.spec,proof,&round.attempt_id)?.ok_or(Failure::RecoveryRequired)?;
                    if !generated_plans::content_matches(goal,&p.reference,&e){return Err(Failure::Invalid);}expected+=1;
                }else if goal.tasks.iter().any(|t|t.id==p.reference.task_id)||goal.plans.iter().any(|t|t.id==p.reference.plan_id){return Err(Failure::Invalid);}
            }
            if refs.len()!=expected{return Err(Failure::Invalid);}
            let _lock=self.lock_shared()?;let latest=self.load(&row.spec,proof)?;
            if model::fingerprint(&latest.planning_usage).map_err(|_|Failure::Invalid)?!=model::fingerprint(&body.planning_usage).map_err(|_|Failure::Invalid)?{return Err(Failure::Busy);}
        }
        self.fence()
    }
}
