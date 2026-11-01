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
  let spec=approved.spec;spec.validate()?;if self.authority.as_deref()!=Some(approved.witness.authority_path()){return Err(Error::Conflict);}let _guard=self.lock(true)?;let id=id_for(spec);
  if let Some((record,_))=self.load(&id)?{if record.spec!=*spec{return Err(Error::Conflict);}return Ok(self.stabilize(record));}
  let now=clock();if now<approved.prepared_at||now>=approved.expires_at{return Err(Error::Conflict);}
  let source=approved.witness.source();let(parent,_)=self.load(&source.goal_id)?.ok_or(Error::NotFound)?;
  let _source_lock=approved.witness.lock().map_err(|e|e.control_error())?;approved.witness.current(&parent).map_err(|e|e.control_error())?;approved.witness.target_current(spec).map_err(|e|e.control_error())?;
  if self.dir.names()?.iter().filter(|n|n.strip_suffix(".json").is_some_and(goal_id)).count()>=MAX_GOALS{return Err(Error::LimitReached);}
  let record=Record::new(spec.clone(),now,nonce()?)?;
  self.publish_current(record,None,false,||{let current=clock();if current<now||current>=approved.expires_at{return Err(Error::Conflict);}approved.witness.current(&parent).map_err(|e|e.control_error())})
 }
}
