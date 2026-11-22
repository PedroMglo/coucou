//! Native metadata reconciliation. Never replays IO, returns budget or creates a
//! controller. A retained exact operation lock and native end-proof are required through the final ACK.
use super::*;
use crate::pet_runtime::{rules::{RuleGrant,ReadinessProbe,ProbeRecovery,RecoveryOutcome},native_owner::EndedOperation};

/// Not deserializable; every source byte was authenticated by the catalogue.
pub(in crate::pet_runtime)struct RuleProbeRecoverySource{
    base:RuleGrant,staged:Option<(String,RuleGrant)>,occurrence:String,probe:ReadinessProbe,hash:String,
}
fn changed_probe(base:&RuleGrant,staged:&RuleGrant)->Option<String>{
    if !staged.valid()||staged.updated_at<base.updated_at||base.reservations.len()!=staged.reservations.len(){return None;}
    let mut plain=staged.clone();plain.schema_version=base.schema_version;plain.revision=base.revision.clone();plain.updated_at=base.updated_at;
    let mut changed=None;
    for(a,b)in base.reservations.iter().zip(plain.reservations.iter_mut()){
        if a.readiness==b.readiness{continue;}
        if changed.is_some(){return None;}
        let valid=if b.readiness.len()==a.readiness.len()+1&&b.readiness.starts_with(&a.readiness){
            b.readiness.last().is_some_and(|p|p.owner.is_some()&&p.finished_at.is_none()&&p.recovery.is_none())
        }else if b.readiness.len()==a.readiness.len()&&!a.readiness.is_empty(){
            let old=a.readiness.last()?;let next=b.readiness.last()?;let mut immutable=next.clone();immutable.finished_at=old.finished_at;immutable.observed=old.observed;immutable.recovery=old.recovery.clone();
            ((old.finished_at.is_none()&&next.finished_at.is_some())||(old.finished_at.is_some()&&old.finished_at==next.finished_at&&old.observed==next.observed&&old.recovery.is_none()&&next.recovery.as_ref().is_some_and(|r|r.outcome==RecoveryOutcome::OriginalCompletionAcknowledged)))&&old.owner.is_some()&&immutable==*old&&a.readiness[..a.readiness.len()-1]==b.readiness[..b.readiness.len()-1]
        }else{false};
        if !valid{return None;}changed=Some(a.occurrence_id.clone());b.readiness=a.readiness.clone();
    }
    if plain!=*base{return None;}changed
}
impl RuleProbeRecoverySource{
    fn selected(&self)->&RuleGrant{self.staged.as_ref().map(|(_,g)|g).unwrap_or(&self.base)}
    pub(in crate::pet_runtime)fn rule(&self)->&crate::pet_runtime::rules::Rule{&self.base.policy.rule}
    pub(in crate::pet_runtime)fn owner(&self)->Result<&crate::pet_runtime::runtime_owner::Operation>{self.probe.owner.as_ref().ok_or(Failure::RecoveryRequired)}
    pub(in crate::pet_runtime)fn preview(&self)->serde_json::Value{
        let selected=self.selected();let r=selected.reservations.iter().find(|r|r.occurrence_id==self.occurrence).unwrap();
        serde_json::json!({"schemaVersion":1,"sourceHash":self.hash,"ruleId":self.rule().id,"petId":self.rule().owner.pet_id,"occurrenceId":self.occurrence,"probeId":self.probe.id,"parentDecisionId":self.base.decision_id,"parentRevision":self.base.revision,"pendingIntent":self.staged.is_some(),"originalCompletionAvailable":self.probe.finished_at.is_some(),"originalDeadlineAt":r.deadline_at,"metadataProbesSpent":r.readiness.len(),"metadataProbesLimit":3,"model":self.base.policy.model,"destination":self.base.policy.destination,"selectedSignedSource":selected,"canonicalHash":model::fingerprint(&self.base).ok(),"stagedHash":self.staged.as_ref().and_then(|(_,g)|model::fingerprint(g).ok()),"executionAuthorityGranted":false,"ioWillReplay":false})
    }
    /// Capture once after the original native choice. Retries retain this exact
    /// candidate/revision/decision/time, including an already staged recovery.
    fn recorded(&self,candidate:&RuleGrant,current:&RuleGrant)->bool{
        if current.decision_id!=candidate.decision_id||current.policy!=candidate.policy||current.approved_at!=candidate.approved_at{return false;}
        let Some(expected)=candidate.reservations.iter().find(|r|r.occurrence_id==self.occurrence&&r.readiness.last().is_some_and(|p|p.recovery.is_some()))else{return false;};
        let Some(actual)=current.reservations.iter().find(|r|r.occurrence_id==expected.occurrence_id)else{return false;};
        if !actual.readiness.starts_with(&expected.readiness)||expected.admission.is_some()&&actual.admission!=expected.admission{return false;}
        let mut plain=actual.clone();plain.readiness=expected.readiness.clone();plain.admission=expected.admission.clone();plain==*expected
    }
    fn matches_candidate(&self,candidate:&RuleGrant)->bool{
        if candidate.updated_at<self.selected().updated_at{return false;}if self.probe.recovery.is_some(){return candidate==self.selected();}if candidate.revision==self.selected().revision{return false;}
        let Some(index)=candidate.reservations.iter().position(|r|r.occurrence_id==self.occurrence)else{return false;};
        let Some(probe)=candidate.reservations[index].readiness.last()else{return false;};let Some(recovery)=&probe.recovery else{return false;};
        let expected=if self.probe.finished_at.is_some(){RecoveryOutcome::OriginalCompletionAcknowledged}else{RecoveryOutcome::NativeWorkerEndedWithoutObservation};
        if recovery.source_hash!=self.hash||recovery.reconciled_at!=candidate.updated_at||recovery.outcome!=expected||self.probe.finished_at.is_some()&&(probe.finished_at!=self.probe.finished_at||probe.observed!=self.probe.observed){return false;}
        let mut immutable=probe.clone();immutable.finished_at=self.probe.finished_at;immutable.observed=self.probe.observed;immutable.recovery=self.probe.recovery.clone();if immutable!=self.probe{return false;}
        let mut plain=candidate.clone();*plain.reservations[index].readiness.last_mut().unwrap()=self.probe.clone();plain.schema_version=self.selected().schema_version;plain.updated_at=self.selected().updated_at;plain.revision=self.selected().revision.clone();plain==*self.selected()
    }
    pub(in crate::pet_runtime)fn candidate(&self,decision:&str,now:u64)->Result<RuleGrant>{
        if !model::token(decision)||now<self.selected().updated_at{return Err(Failure::Changed);}
        let mut g=self.selected().clone();let r=g.reservations.iter_mut().find(|r|r.occurrence_id==self.occurrence).ok_or(Failure::Changed)?;let probe=r.readiness.last_mut().ok_or(Failure::Missing)?;
        if probe!=&self.probe{return Err(Failure::Changed);}
        // A signed recovery choice already staged is acknowledged byte-exact.
        if probe.recovery.is_some(){return Ok(g);}
        let outcome=if probe.finished_at.is_some(){RecoveryOutcome::OriginalCompletionAcknowledged}else{probe.finished_at=Some(now);probe.observed=Some(false);RecoveryOutcome::NativeWorkerEndedWithoutObservation};
        probe.recovery=Some(ProbeRecovery{schema_version:1,source_hash:self.hash.clone(),native_decision_id:decision.into(),reconciled_at:now,outcome});g.schema_version=g.required_version();g.updated_at=now;g.revision=nonce().map_err(|_|Failure::Unavailable)?;
        if !g.valid(){return Err(Failure::Invalid);}Ok(g)
    }
}
impl Authority{
    pub(in crate::pet_runtime)fn prepare_rule_probe_recovery(&self,decision:&str)->Result<RuleProbeRecoverySource>{
        if !model::token(decision){return Err(Failure::Invalid);}let _lock=self.lock_shared()?;let name=format!("{decision}.json");if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
        let(base,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;if base.policy.dispatch.is_none(){return Err(Failure::Invalid);}
        let pending=self.rule_inventory_unlocked()?.into_iter().filter(|(_,g,c)|!*c&&g.decision_id==decision).collect::<Vec<_>>();
        let(staged,occurrence)=match pending.as_slice(){
            []=>(None,base.reservations.iter().find(|r|r.readiness.last().is_some_and(|p|p.finished_at.is_none())).map(|r|r.occurrence_id.clone()).ok_or(Failure::Missing)?),
            [(n,g,_)]=>(Some((n.clone(),g.clone())),changed_probe(&base,g).ok_or(Failure::RecoveryRequired)?),
            _=>return Err(Failure::RecoveryRequired),
        };
        let selected=staged.as_ref().map(|(_,g)|g).unwrap_or(&base);let probe=selected.reservations.iter().find(|r|r.occurrence_id==occurrence).and_then(|r|r.readiness.last()).cloned().ok_or(Failure::Missing)?;
        let owner=probe.owner.as_ref().ok_or(Failure::RecoveryRequired)?;if !owner.valid(&probe.id,&probe.scope_hash,probe.prepared_at){return Err(Failure::Invalid);}
        let hash=model::fingerprint(&("coucou-rule-probe-recovery-source-v1",&base,&staged,&occurrence,&probe)).map_err(|_|Failure::Invalid)?;self.fence()?;
        Ok(RuleProbeRecoverySource{base,staged,occurrence,probe,hash})
    }
    pub(in crate::pet_runtime)fn recover_rule_probe(&self,source:&RuleProbeRecoverySource,candidate:&RuleGrant,gate:&EndedOperation,now:u64,current:impl Fn()->bool)->Result<RuleGrant>{
        if !current()||gate.record()!=source.owner()?||now<candidate.updated_at||!candidate.valid()||!source.matches_candidate(candidate){return Err(Failure::Changed);}
        let _lock=self.lock()?;gate.current(self).map_err(|_|Failure::RecoveryRequired)?;
        let name=format!("{}.json",source.base.decision_id);if self.historical_file(&name){return Err(Failure::RecoveryRequired);}
        let(mut canonical,_)=self.rule_record_data(&name)?.ok_or(Failure::Missing)?;
        let pending=self.rule_inventory_unlocked()?.into_iter().filter(|(_,g,c)|!*c&&g.decision_id==source.base.decision_id).collect::<Vec<_>>();
        let check=||{if !current(){return Err(Failure::Changed);}gate.current(self).map_err(|_|Failure::RecoveryRequired)};
        // A later normal occurrence/probe must not make the original receipt
        // disappear or let an ACK retry overwrite subsequent signed progress.
        if source.recorded(candidate,&canonical){check()?;self.stabilize_rule(&canonical)?;return Ok(canonical);}
        // Original choice may have reached the canonical record before ACK loss.
        if canonical==*candidate&&pending.is_empty(){check()?;self.stabilize_rule(candidate)?;return Ok(candidate.clone());}
        if canonical==*source.selected(){
            if let[(staged_name,staged,_)]=pending.as_slice(){
                if staged!=candidate{return Err(Failure::RecoveryRequired);}check()?;self.dir.rename(staged_name,&self.dir,&name,false).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;return Ok(candidate.clone());
            }else if !pending.is_empty(){return Err(Failure::RecoveryRequired);}
        }else if canonical==source.base{
            match(&source.staged,pending.as_slice()){
                (Some((expected,body)),[(actual,staged,_)])if expected==actual&&body==staged=>{check()?;self.dir.rename(actual,&self.dir,&name,false).map_err(|_|Failure::Uncertain)?;self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;canonical=staged.clone();},
                _=>return Err(Failure::Changed),
            }
        }else{return Err(Failure::Changed);}
        if canonical!=*source.selected(){return Err(Failure::Changed);}check()?;
        if canonical==*candidate{self.stabilize_rule(candidate)?;return Ok(candidate.clone());}
        self.publish_rule(candidate,false,check)?;Ok(candidate.clone())
    }
}
#[cfg(test)]#[path="rule_probe_recovery_tests.rs"]mod tests;
