//! Same Authority key, distinct data-creation domain. No grant is issued here.
use super::*;
use crate::pet_runtime::restored_future::{self,Body,Origin,Source};
pub(in crate::pet_runtime)struct Witness{authority:Authority,source:Source,stamps:Vec<(String,Stamp)>,review_id:String}
impl Witness{
 pub(in crate::pet_runtime)fn capture(path:&Path,goal:&Record,now:u64)->Result<Self>{
  let authority=Authority::existing(path)?;let lock=authority.lock_shared()?;
  let(snapshot,stamps)=restoration::captured_goal_snapshot(&authority,goal,now)?.ok_or(Failure::RecoveryRequired)?;let origin=snapshot.restored_source.ok_or(Failure::RecoveryRequired)?;
  let source=Source{pet_id:goal.spec.owner.pet_id.clone(),goal_id:goal.id.clone(),revision:goal.revision.clone(),goal_hash:snapshot.goal_hash,history_hash:origin.history_hash,source_fingerprint:origin.source_fingerprint,snapshot_hash:snapshot.source_hash,historical_related_grants:origin.historical_related_grants,unattributed_historical_grants:origin.unattributed_historical_grants};
  drop(lock);Ok(Self{authority,source,stamps,review_id:nonce().map_err(|_|Failure::Unavailable)?})
 }
 pub(in crate::pet_runtime)fn source(&self)->&Source{&self.source}
 pub(in crate::pet_runtime)fn review_id(&self)->&str{&self.review_id}
 pub(in crate::pet_runtime)fn authority_path(&self)->&Path{&self.authority.dir.path}
 pub(in crate::pet_runtime)fn target_current(&self,spec:&model::Spec)->Result<()>{let id=model::id_for(spec);self.authority.restoration_current(&self.review_id,&id)?;let(grants,pending)=self.authority.history_inventory(&id,None)?;if !grants.is_empty()||!pending.is_empty(){return Err(Failure::RecoveryRequired);}self.authority.fence()}
 pub(in crate::pet_runtime)fn current(&self,goal:&Record)->Result<()>{
  if goal.id!=self.source.goal_id||goal.spec.owner.pet_id!=self.source.pet_id||goal.revision!=self.source.revision||model::fingerprint(goal).map_err(|_|Failure::Invalid)?!=self.source.goal_hash{return Err(Failure::Changed);}
  self.authority.fence()?;for(n,s)in &self.stamps{if !self.authority.dir.has_stamp(n,s).map_err(|_|Failure::Changed)?{return Err(Failure::Changed);}}self.authority.fence()
 }
 pub(in crate::pet_runtime)fn lock(&self)->Result<Guard>{self.authority.lock_shared()}
 pub(in crate::pet_runtime)fn seal(&self,spec:&model::Spec,client_request_id:&str,prepared:u64,expires:u64,approved:u64)->Result<Origin>{
  self.target_current(spec)?;let body=Body{schema_version:1,purpose:"restoredFutureDraftDataOnly".into(),key_id:self.authority.key_id.clone(),review_id:self.review_id.clone(),client_request_id:client_request_id.into(),prepared_at:prepared,approved_at:approved,expires_at:expires,source:self.source.clone(),target_goal_id:model::id_for(spec),base_spec_hash:restored_future::base_hash(spec).map_err(|_|Failure::Invalid)?,unknown_state_accepted:true,execution_authority_granted:false,accounting_known:false,owner_absence_proven:false};
  let tag=encode(hmac::sign(&self.authority.key,&restored_future::message(&body).map_err(|_|Failure::Invalid)?).as_ref());let origin=Origin{body,tag};let mut proposed=spec.clone();proposed.restored_from=Some(origin.clone());if !origin.valid_for(&proposed){return Err(Failure::Invalid);}self.authority.fence()?;Ok(origin)
 }
}
impl Authority{
 pub(in crate::pet_runtime)fn verify_restored_origin(&self,goal:&Record)->Result<()>{
  let Some(origin)=&goal.spec.restored_from else{return Ok(());};
  // A later epoch preserves this certificate as DATA behind a new GoalRef.
  // It cannot revive an old identity, grant or key in the current runtime.
  if restoration::matches_historical_goal(self,goal)?{return Ok(());}
  if !origin.valid_for(&goal.spec){return Err(Failure::Invalid);}if origin.body.key_id!=self.key_id{return Err(Failure::WrongKey);}
  hmac::verify(&self.key,&restored_future::message(&origin.body).map_err(|_|Failure::Invalid)?,&decode(&origin.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;self.fence()
 }
}
