//! One canonical receipt observation on the existing Authority, without a
//! creation Frame retained for the whole autonomous goal or a new grant ledger.
use super::*;
use crate::pet_runtime::{received_context::Selection,messages::Envelope};
pub(in crate::pet_runtime)struct Witness{authority:Authority,selection:Selection,envelope:Envelope,goal_id:String,review_id:String,dependencies:Option<crate::pet_runtime::received_result_data::Dependencies>}
impl Witness{
 pub(in crate::pet_runtime)fn capture(path:&Path,goal:&Record,selection:&Selection,now:u64)->Result<Self>{
  Self::capture_with_memory(path,&crate::pet_memory::context::Capture::catalog_path(),goal,selection,now)
 }
 pub(in crate::pet_runtime)fn capture_with_memory(path:&Path,memory:&Path,goal:&Record,selection:&Selection,now:u64)->Result<Self>{
  goal.validate().map_err(|_|Failure::Invalid)?;if now<selection.source.received_at{return Err(Failure::Expired);}if !selection.valid_for(goal){return Err(Failure::Changed);}let origin=goal.spec.received_from.as_ref().ok_or(Failure::Invalid)?;let authority=Authority::existing(path)?;let _lock=authority.lock_shared()?;
  // Historical DATA can be inspected by native users, but not rematerialized
  // into new inference input merely by restoring its old certified origin.
  authority.restoration_current(&origin.body.review_id,&goal.id)?;authority.verify_received_origin(goal)?;
  let r=authority.received_source_unlocked(&selection.source.reference,&goal.spec.owner.pet_id,now)?;
  let fact=super::received_request::Witness::fact(&selection.source.reference,&r)?;
  let shape=match selection.schema_version{7=>matches!(r.envelope().schema_version,12|14|16)&&r.envelope().kind==crate::pet_runtime::messages::Kind::Delegation&&r.envelope().delegation==origin.body.delegation&&r.envelope().planned_data.as_ref().is_some_and(|p|matches!(p.schema_version,2|3|4)),6=>matches!(r.envelope().schema_version,10|11|13|15)&&r.envelope().kind==crate::pet_runtime::messages::Kind::Request&&r.envelope().planned_data.is_some(),1=>r.envelope().schema_version==1&&r.envelope().kind==crate::pet_runtime::messages::Kind::Request,2=>r.envelope().schema_version==4&&r.envelope().kind==crate::pet_runtime::messages::Kind::Delegation&&r.envelope().delegation==origin.body.delegation,4=>r.envelope().schema_version==7&&r.envelope().kind==crate::pet_runtime::messages::Kind::Delegation&&r.envelope().delegation==origin.body.delegation,5=>r.envelope().schema_version==9&&r.envelope().kind==crate::pet_runtime::messages::Kind::Reply&&r.envelope().derived_result.as_ref().is_some_and(|d|d.schema_version==2),3=>r.envelope().schema_version==6&&r.envelope().kind==crate::pet_runtime::messages::Kind::Reply&&r.envelope().derived_result.is_some(),_=>false};
  if fact!=selection.source||!shape{return Err(Failure::Changed);}authority.funded_context_current_unlocked(selection,&goal.id,r.envelope(),now)?;
  let envelope=r.envelope().clone();let dependencies=if matches!(selection.schema_version,3|5|6|7){Some(crate::pet_runtime::received_result_data::Dependencies::capture(&authority,memory,&selection.source.reference,&envelope,now)?)}else{None};drop(_lock);Ok(Self{authority,selection:selection.clone(),envelope,goal_id:goal.id.clone(),review_id:origin.body.review_id.clone(),dependencies})
 }
 pub(in crate::pet_runtime)fn source_depth(&self)->usize{self.dependencies.as_ref().map(|d|d.depth()).unwrap_or(1)}
 pub(in crate::pet_runtime)fn selection(&self)->&Selection{&self.selection}
 pub(in crate::pet_runtime)fn envelope(&self)->&Envelope{&self.envelope}
 pub(in crate::pet_runtime)fn goal_id(&self)->&str{&self.goal_id}
 pub(in crate::pet_runtime)fn verify(&self,now:u64)->Result<()>{
  // This is an immutable canonical DATA snapshot, not a grant mutation.
  // Runtime fences also run inside same-Authority effect writers; taking a
  // second flock here would reject our own writer. grant_file verifies the
  // canonical atomic file/HMAC; exact receipt/envelope+key/history fences are
  // checked again. Source grant phases are not received-data permissions.
  if now<self.selection.source.received_at{return Err(Failure::Expired);}self.authority.fence()?;self.authority.restoration_current(&self.review_id,&self.goal_id)?;
  let r=self.authority.received_source_unlocked(&self.selection.source.reference,&self.selection.source.recipient_pet_id,now)?;
  if super::received_request::Witness::fact(&self.selection.source.reference,&r)?!=self.selection.source||r.envelope()!=&self.envelope{return Err(Failure::Changed);}self.authority.funded_context_current_unlocked(&self.selection,&self.goal_id,&self.envelope,now)?;if let Some(d)=&self.dependencies{d.verify(&self.authority,now)?;}self.authority.fence()
 }
}
