//! Common CAS/absence, source lock, final fence, durable publication and ACK.
//! This never creates a grant, reservation, worker, context or model request.
use super::*;
impl Store{
 pub(in crate::pet_runtime)fn create_reviewed_goal_clock(&self,c:&impl crate::pet_runtime::goal_data_publication::Certificate,clock:impl Fn()->u64)->Result<Receipt>{
  let spec=c.spec();spec.validate()?;if self.authority.as_deref()!=Some(c.authority_path()){return Err(Error::Conflict);}let _guard=self.lock(true)?;let id=id_for(spec);
  // An existing exact target is an immutable fact, even after expiry/Frame loss.
  if let Some((record,_))=self.load(&id)?{if record.spec!=*spec{return Err(Error::Conflict);}return Ok(self.stabilize(record));}
  let now=clock();if now<c.prepared_at()||now>=c.expires_at(){return Err(Error::Conflict);}
  let _source=c.lock_source()?;c.current(self,now)?;
  if self.dir.names()?.iter().filter(|n|n.strip_suffix(".json").is_some_and(goal_id)).count()>=MAX_GOALS{return Err(Error::LimitReached);}
  let record=Record::new(spec.clone(),now,nonce()?)?;
  self.publish_current(record,None,false,||{let later=clock();if later<now||later>=c.expires_at(){return Err(Error::Conflict);}c.current(self,later)})
 }
}
