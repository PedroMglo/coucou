//! One shared-reader history across every original admission and canonical
//! signed grant. No receipt repair, model, target IO, fsync or new authority.
use super::*;
use crate::pet_runtime::{history as view,reconciliation::{Evidence,EvidenceKind,EvidenceStatus as Status},admission::QueuePhase};
use std::collections::BTreeMap;

fn phase(phase:GrantPhase)->&'static str {
    match phase{GrantPhase::Issued=>"issued",GrantPhase::Revoked=>"revoked",GrantPhase::Claimed=>"claimed",GrantPhase::Waiting=>"waiting",GrantPhase::Draining=>"draining",GrantPhase::Closed=>"closed"}
}
impl Authority {
    /// A same-goal signed grant absent from JSON cannot silently disappear from
    /// accounting. It may be an unpublished preparation or missing history;
    /// neither interpretation proves spare allowance. Staged writes also remain
    /// visible without promoting them to canonical accounting.
    pub(in crate::pet_runtime::authority_store)fn history_inventory(&self,goal:&str,excluding:Option<&AdmissionSpec>)->Result<(BTreeMap<String,Grant>,BTreeMap<String,String>)> {
        let mut grants=BTreeMap::new();let mut pending=BTreeMap::new();
        for name in self.dir.names().map_err(|_|Failure::Unavailable)? {
            if self.historical_file(&name){continue;}
            if matches!(name.as_str(),".key.json"|".catalog.lock"|".protocol.json"){continue;}
            let canonical=name.strip_suffix(".json").filter(|s|model::token(s));
            let staged=name.strip_prefix(".grant-").is_some_and(model::token);
            if canonical.is_none()&&!staged{return Err(Failure::RecoveryRequired);}
            let(raw,_)=self.dir.bytes(&name,MAX_GRANT_BYTES).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;
            let envelope:GrantEnvelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
            let body=envelope.body;
            if !body.valid()||canonical.is_some_and(|id|id!=body.decision_id){return Err(Failure::Invalid);}
            hmac::verify(&self.key,&grant_message(&body)?,&decode(&envelope.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;
            if body.goal_id!=goal{continue;}
            if excluding.is_some_and(|spec|body.decision_id==spec.decision_id&&body.spec_hash==model::fingerprint(spec).unwrap_or_default()&&body.approved_at==spec.approved_at&&body.phase==GrantPhase::Issued&&Self::never_claimed_zero(&body)){continue;}

            if canonical.is_some(){grants.insert(body.decision_id.clone(),body);}else{pending.insert(name,model::hash(&raw));}
        }
        self.fence()?;Ok((grants,pending))
    }
    pub(in crate::pet_runtime)fn history(&self,goal:&Record,now:u64)->Result<view::Snapshot> {
        goal.validate().map_err(|_|Failure::Invalid)?;
        if now<goal.updated_at||now>8_640_000_000_000_000{return Err(Failure::Invalid);}
        let _guard=self.lock_shared()?;
        if let Some(snapshot)=super::restoration::goal_snapshot(self,goal,now)?{return Ok(snapshot);}
        let mut snapshot=view::Snapshot::empty(goal,now).map_err(|_|Failure::Invalid)?;
        let lineage=match self.verify_continuations_goal(goal).and_then(|_|self.verify_generated_goal(goal,true)).and_then(|_|self.verify_recovered_goal(goal).map(|_|())){
            Ok(())=>Evidence::verified(EvidenceKind::GoalLineage,goal.id.clone(),&snapshot.goal_hash)?,
            Err(e)=>Evidence::failed(EvidenceKind::GoalLineage,goal.id.clone(),e),
        };
        snapshot.source_consistent&=lineage.status==Status::Verified;
        snapshot.scan_complete=lineage.status==Status::Verified;
        snapshot.lineage.push(lineage);
        let(grants,pending)=self.history_inventory(&goal.id,None)?;
        let inventory_hash=model::fingerprint(&(&grants,&pending)).map_err(|_|Failure::Invalid)?;
        snapshot.pending_artifacts=pending.len();
        let mut all_charges=goal.legacy.is_empty();
        for(index,row)in goal.admissions.iter().enumerate() {
            let spec=&row.spec;
            let mut admission=view::Admission{index,decision_id:spec.decision_id.clone(),spec_hash:model::fingerprint(spec).map_err(|_|Failure::Invalid)?,source_sequence:spec.source_sequence,queue_phase:row.phase,resolved_model:spec.resolved_model.clone(),budget:spec.budget.clone(),provenance:Status::Verified,reason:None,grant:None,ledger_status:view::LedgerStatus::Unavailable,charge:None,attempts:vec![],evidence:vec![]};
            let loaded=row.proof.as_ref().ok_or(Failure::Missing).and_then(|proof|self.load(spec,proof));
            match loaded {
                Err(e)=>{
                    admission.provenance=match e{Failure::Missing=>Status::Missing,Failure::Busy|Failure::Unavailable|Failure::Uncertain=>Status::Unavailable,_=>Status::Invalid};
                    admission.reason=Some(e);snapshot.scan_complete=false;snapshot.source_consistent=false;
                },
                Ok(body)=>{
                    if now<body.updated_at{return Err(Failure::Invalid);}
                    if spec.owner!=goal.spec.owner||spec.intent_hash!=model::fingerprint(&goal.spec).map_err(|_|Failure::Invalid)?||goal.execution_context.as_ref()!=Some(&spec.context){return Err(Failure::Changed);}
                    let inspected=self.reconciliation_evidence_unlocked(goal,row,&body)?;
                    admission.attempts=inspected.attempts;admission.evidence=inspected.evidence;
                    let terminal=matches!(body.phase,GrantPhase::Closed|GrantPhase::Revoked);
                    let lease_matches=row.lease==body.lease;
                    let projection=if index+1<goal.admissions.len(){terminal&&!matches!(row.phase,QueuePhase::Queued|QueuePhase::Leased|QueuePhase::StopRequested)}else{
                        match body.phase{GrantPhase::Issued=>goal.state==State::Queued,GrantPhase::Claimed|GrantPhase::Waiting=>goal.state==State::Leased,GrantPhase::Draining=>goal.state==State::Stopping&&row.stop_reason==body.stop_reason,GrantPhase::Closed|GrantPhase::Revoked=>!matches!(goal.state,State::Queued|State::Leased|State::Stopping)}
                    };
                    snapshot.source_consistent&=lease_matches&&projection;
                    admission.grant=Some(view::Grant{revision:body.revision.clone(),hash:model::fingerprint(&body).map_err(|_|Failure::Invalid)?,phase:phase(body.phase),lease_id:body.lease.as_ref().map(|l|l.id.clone()),goal_lease_matches:lease_matches,terminal,updated_at:body.updated_at,closed_at:body.closed_at,recovery:body.recovery.as_ref().map(|c|c.reference.clone())});
                    admission.ledger_status=if body.budget_usage.is_some(){view::LedgerStatus::Recorded}else if body.lease.is_none(){view::LedgerStatus::NeverClaimed}else if body.runtime_owner.is_some(){view::LedgerStatus::NativeUnstarted}else{view::LedgerStatus::LegacyUnrecorded};
                    if admission.ledger_status!=view::LedgerStatus::LegacyUnrecorded{
                        let charge=view::Charge::from_attempts(&admission.attempts)?;
                        snapshot.authenticated_subtotal.checked_add(&charge)?;admission.charge=Some(charge);
                    }
                    snapshot.scan_complete&=admission.evidence.iter().chain(admission.attempts.iter().flat_map(|a|&a.evidence)).all(|e|!matches!(e.status,Status::Invalid|Status::Unavailable));
                },
            }
            all_charges&=admission.charge.is_some();snapshot.admissions.push(admission);
        }
        snapshot.linked_source_consistent=snapshot.source_consistent;
        for(decision,body)in &grants {
            if goal.admissions.iter().any(|row|&row.spec.decision_id==decision){continue;}
            snapshot.unlinked_grants.push(view::UnlinkedGrant{decision_id:decision.clone(),hash:model::fingerprint(body).map_err(|_|Failure::Invalid)?,phase:phase(body.phase),updated_at:body.updated_at,lease_id:body.lease.as_ref().map(|l|l.id.clone()),revoked_without_io:body.phase==GrantPhase::Revoked&&Self::never_claimed_zero(body),retirement:body.retirement.clone()});
        }
        if snapshot.unlinked_grants.iter().any(|g|!g.revoked_without_io)||!pending.is_empty(){snapshot.source_consistent=false;}
        snapshot.accounting_complete=all_charges&&snapshot.source_consistent;
        snapshot.cumulative_charge=if snapshot.accounting_complete{Some(snapshot.authenticated_subtotal.clone())}else{None};
        snapshot.outcomes_complete=snapshot.accounting_complete&&snapshot.scan_complete&&snapshot.authenticated_subtotal.unknown_model_results==0&&snapshot.authenticated_subtotal.unknown_tool_results==0;
        snapshot.source_hash=model::fingerprint(&(&snapshot.goal_hash,&inventory_hash,&snapshot.admissions,&snapshot.unlinked_grants,&snapshot.lineage,&snapshot.authenticated_subtotal,snapshot.accounting_complete,snapshot.scan_complete)).map_err(|_|Failure::Invalid)?;
        let after=self.history_inventory(&goal.id,None)?;
        if model::fingerprint(&after).map_err(|_|Failure::Invalid)?!=inventory_hash{return Err(Failure::Changed);}
        self.fence()?;Ok(snapshot)
    }
}
