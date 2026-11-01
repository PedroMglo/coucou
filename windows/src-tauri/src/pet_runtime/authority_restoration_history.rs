//! A copied goal is an authenticated snapshot, never live zero accounting.
//! Called under the existing shared Authority lock. No grant load/closure,
//! model discovery, new namespace, mutation or continuation certificate.
use super::*;
use crate::pet_runtime::history::{Snapshot,RestoredSource};
pub(in crate::pet_runtime::authority_store)fn goal_snapshot(a:&Authority,goal:&Record,now:u64)->Result<Option<Snapshot>>{
 Ok(captured_goal_snapshot(a,goal,now)?.map(|(snapshot,_)|snapshot))
}
pub(in crate::pet_runtime::authority_store)fn captured_goal_snapshot(a:&Authority,goal:&Record,now:u64)->Result<Option<(Snapshot,Vec<(String,Stamp)>)>>{
 let Some(boundary)=&a.restoration else{return Ok(None);};
 if !boundary.body.goal_ids.contains(&goal.id){return Ok(None);}
 a.fence()?;
 let reference=boundary.body.goals.iter().find(|r|r.id==goal.id).ok_or(Failure::Missing)?;
 let goal_hash=model::fingerprint(goal).map_err(|_|Failure::Invalid)?;
 if reference.pet_id!=goal.spec.owner.pet_id||reference.hash!=goal_hash{return Err(Failure::Changed);}
 // Reuse the same bounded hash/signature/fence reader as the native restore
 // inspection. Validity here means preserved source bytes, never execution.
 let stamps=verify_historical_grants(a)?;
 let source=RestoredSource{schema_version:1,history_hash:boundary.hash.clone(),source_fingerprint:boundary.body.source.clone(),source_goal_hash:goal_hash,historical_related_grants:boundary.body.grants.iter().filter(|g|g.goal_id.as_deref()==Some(goal.id.as_str())||g.decision_id.as_ref().is_some_and(|id|reference.decisions.contains(id))).count(),unattributed_historical_grants:boundary.body.grants.iter().filter(|g|g.goal_id.is_none()).count(),owner_absence_proven:false,post_snapshot_state_unknown:true,execution_authority_granted:false};
 let mut snapshot=Snapshot::empty(goal,now).map_err(|_|Failure::Invalid)?;
 snapshot.schema_version=2;snapshot.scope="restoredSnapshotOnly";snapshot.history_coverage="immutableRestoredSnapshotOnly";
 snapshot.source_hash=model::fingerprint(&("coucou-restored-goal-history-v1",&snapshot.goal_hash,&source)).map_err(|_|Failure::Invalid)?;
 snapshot.cumulative_charge=None;snapshot.accounting_complete=false;snapshot.outcomes_complete=false;snapshot.linked_source_consistent=false;
 snapshot.scan_complete=true;snapshot.source_consistent=true;snapshot.restored_source=Some(source);
 a.fence()?;Ok(Some((snapshot,stamps)))
}
