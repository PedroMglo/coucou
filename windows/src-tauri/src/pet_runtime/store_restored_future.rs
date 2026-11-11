//! Data-only publication through the common per-goal transaction. ACK reads
//! the original certified Spec before checking expiry/source for a NEW effect.
use super::*;
use crate::pet_runtime::restored_future::Approved;
impl Store{
 pub(in crate::pet_runtime)fn existing_restored_future(&self,pet:&str,source_id:&str,source_revision:&str,request:&str,intent:&str,criteria:&[String])->Result<Option<Receipt>>{
  if !key(pet)||!goal_id(source_id)||!token(source_revision)||!key(request){return Err(Error::InvalidInput);}
  if self.authority.is_none(){return Err(Error::InvalidInput);}let request_id=crate::pet_runtime::restored_future::request_id(pet,source_id,source_revision,request)?;let _guard=self.lock(true)?;let id=format!("g-{}",hash(format!("{pet}\0{request_id}").as_bytes()));let Some((goal,_))=self.load(&id)?else{return Ok(None);};
  let source=&goal.spec.restored_from.as_ref().ok_or(Error::Conflict)?.body.source;
  if goal.spec.owner.pet_id!=pet||goal.spec.request_id!=request_id||goal.spec.intent!=intent||goal.spec.criteria!=criteria||source.pet_id!=pet||source.goal_id!=source_id||source.revision!=source_revision{return Err(Error::Conflict);}Ok(Some(self.stabilize(goal)))
 }
 pub(in crate::pet_runtime)fn create_restored_future(&self,approved:&Approved<'_>,now:u64)->Result<Receipt>{
  self.create_restored_future_clock(approved,||now)
 }
 pub(in crate::pet_runtime)fn create_restored_future_clock(&self,approved:&Approved<'_>,clock:impl Fn()->u64)->Result<Receipt>{
  self.create_reviewed_goal_clock(approved,clock)
 }
}

impl crate::pet_runtime::goal_data_publication::Certificate for Approved<'_>{
 fn spec(&self)->&Spec{self.spec}
 fn authority_path(&self)->&std::path::Path{self.witness.authority_path()}
 fn prepared_at(&self)->u64{self.prepared_at}
 fn expires_at(&self)->u64{self.expires_at}
 fn lock_source(&self)->Result<Guard>{self.witness.lock().map_err(|e|e.control_error())}
 fn current(&self,store:&Store,_now:u64)->Result<()>{let source=self.witness.source();let(parent,_)=store.load(&source.goal_id)?.ok_or(Error::NotFound)?;self.witness.current(&parent).map_err(|e|e.control_error())?;self.witness.target_current(self.spec).map_err(|e|e.control_error())}
}
