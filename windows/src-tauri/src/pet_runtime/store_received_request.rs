//! The received-task certificate adopts the common reviewed-goal writer.
use super::*;
use crate::pet_runtime::{received_request::{self,Certificate},messages::ReceivedReference};
impl crate::pet_runtime::goal_data_publication::Certificate for Certificate<'_>{
 fn spec(&self)->&Spec{self.approved.signed.as_ref().expect("native signed data certificate")}
 fn authority_path(&self)->&std::path::Path{self.approved.review.witness.authority_path()}
 fn prepared_at(&self)->u64{self.approved.review.proposal.prepared_at}
 fn expires_at(&self)->u64{self.approved.review.proposal.expires_at}
 fn lock_source(&self)->Result<Guard>{self.approved.review.witness.lock().map_err(|e|e.control_error())}
 fn current(&self,_store:&Store,now:u64)->Result<()>{self.approved.current(now)?;self.approved.review.witness.current(now).map_err(|e|e.control_error())?;self.approved.review.witness.target_current(self.approved.signed.as_ref().ok_or(Error::Conflict)?,now).map_err(|e|e.control_error())}
}
impl Store{
 pub(in crate::pet_runtime)fn received_work(&self,path:&std::path::Path,input:&crate::pet_runtime::received_work::Input,now:u64)->Result<crate::pet_runtime::received_work::View>{
  if self.authority.as_deref()!=Some(path)||!key(&input.pet_id)||!goal_id(&input.goal_id)||!token(&input.revision)||!input.source.valid(){return Err(Error::InvalidInput);}
  let _lock=self.lock(false)?;let(goal,stamp)=self.load_with_pending(&input.goal_id,true)?.ok_or(Error::NotFound)?;
  if goal.spec.owner.pet_id!=input.pet_id||goal.revision!=input.revision||goal.spec.received_from.as_ref().is_none_or(|o|o.body.source.reference!=input.source){return Err(Error::Conflict);}
  let a=crate::pet_runtime::authority::Authority::existing(path).map_err(Self::proof_error)?;
  let view=crate::pet_runtime::received_work::View::base(&goal,now)?;
  let view=if crate::pet_runtime::restoration::matches_historical_goal(&a,&goal).map_err(Self::proof_error)?{view}else{view.observed(&goal,a.reconciliation(&goal,now).map_err(Self::proof_error)?)?};
  if !self.dir.has_stamp(&format!("{}.json",goal.id),&stamp)?{return Err(Error::Conflict);}Ok(view)
 }
 pub(in crate::pet_runtime)fn existing_received_request(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String])->Result<Option<Receipt>>{
  self.existing_origin_requests(pet,reference,request,intent,criteria,&[received_request::Purpose::Request,received_request::Purpose::PlannedData])
 }
 // DATA receipt lookup never revalidates the live sender. Expiry/revocation
 // still prevent fresh creation through the native review/certificate writer.
 pub(in crate::pet_runtime)fn existing_funded_request(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String])->Result<Option<Receipt>>{self.existing_origin_request(pet,reference,request,intent,criteria,received_request::Purpose::Funded)}
 pub(in crate::pet_runtime)fn existing_planned_request(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String])->Result<Option<Receipt>>{self.existing_origin_request(pet,reference,request,intent,criteria,received_request::Purpose::PlannedData)}
 pub(in crate::pet_runtime)fn existing_result_request(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String])->Result<Option<Receipt>>{self.existing_origin_request(pet,reference,request,intent,criteria,received_request::Purpose::Result)}
 pub(in crate::pet_runtime)fn existing_recursive_funded_request(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String])->Result<Option<Receipt>>{self.existing_origin_request(pet,reference,request,intent,criteria,received_request::Purpose::RecursiveFunded)}
 fn existing_origin_request(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String],purpose:received_request::Purpose)->Result<Option<Receipt>>{
  self.existing_origin_requests(pet,reference,request,intent,criteria,&[purpose])
 }
 fn existing_origin_requests(&self,pet:&str,reference:&ReceivedReference,request:&str,intent:&str,criteria:&[String],purposes:&[received_request::Purpose])->Result<Option<Receipt>>{
  if self.authority.is_none()||!key(request){return Err(Error::InvalidInput);}let _lock=self.lock(true)?;let mut found=None;
  // Inspect both typed request identities under one catalog lock. An ambiguous
  // durable record is a conflict, never a choice of another goal or authority.
  for purpose in purposes.iter().copied(){
   let funded=matches!(purpose,received_request::Purpose::Funded|received_request::Purpose::RecursiveFunded|received_request::Purpose::DerivedFunded);
   let request_id=purpose.request(pet,reference,request)?;let id=format!("g-{}",hash(format!("{pet}\0{request_id}").as_bytes()));let Some((goal,_))=self.load(&id)?else{continue;};let origin=goal.spec.received_from.as_ref().ok_or(Error::Conflict)?;
   let version=match purpose{received_request::Purpose::Result=>matches!(origin.body.schema_version,3|5),received_request::Purpose::Funded=>matches!(origin.body.schema_version,2|7),_=>origin.body.schema_version==purpose.version()};
   if goal.spec.owner.pet_id!=pet||goal.spec.intent!=intent||goal.spec.criteria!=criteria||origin.body.source.reference!=*reference||!version||origin.body.delegation.is_some()!=funded||!funded&&origin.body.client_request_id!=request||found.is_some(){return Err(Error::Conflict);}found=Some(goal);
  }
  Ok(found.map(|goal|self.stabilize(goal)))
 }
}
