//! Native reviewed data uses the same Store commit, publication and absence
//! proofs as generated plans. No side ledger or executor is constructed here.
use super::*;
use crate::pet_runtime::{authority::{Authority,Failure},authority_store::NativePlanCertified,continuation,native_plan_revisions::{Input,Reference,Seal},native_plan_review::{Review,Receipt as RevisionReceipt}};
use std::sync::Arc;
impl Store{
 pub(in crate::pet_runtime)fn native_plan_inventory(&self,path:&Path,id:&str,pet:&str,revision:&str,cursor:Option<&str>)->Result<crate::pet_runtime::native_plan_inventory::Inventory>{
  let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;self.require_current_goal(&goal)?;
  if goal.spec.owner.pet_id!=pet||goal.revision!=revision{return Err(Error::Conflict);}let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  authority.native_plan_inventory(&goal,cursor).map_err(Failure::control_error)
 }
 pub(in crate::pet_runtime)fn prepare_native_plan(&self,path:&Path,id:&str,pet:&str,revision:&str,inputs:Vec<Input>,request:String,now:u64)->Result<Review>{
  let _guard=self.lock(false)?;let(goal,_)=self.load(id)?.ok_or(Error::NotFound)?;
  if goal.spec.owner.pet_id!=pet||goal.revision!=revision||now<goal.updated_at{return Err(Error::Conflict);}
  let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let captured=authority.capture_continuation(&goal,now,false).map_err(Failure::control_error)?.map_err(|_|Error::Conflict)?;
  Review::capture(goal,captured,inputs,request,now)
 }
 pub(in crate::pet_runtime)fn certify_native_plan<'a>(&self,authority:&'a Authority,source_goal:&Record,source:&continuation::Reference,reference:&Reference,approved_at:u64,cache:&mut Option<Seal>,now:u64,human:Arc<dyn Fn()->bool+Send+Sync>)->Result<NativePlanCertified<'a>>{
  let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(&source_goal.id,true)?.ok_or(Error::NotFound)?;self.bound_authority(authority)?;
  if goal.spec.owner!=source_goal.spec.owner{return Err(Error::Conflict);}authority.certify_native_plan(&goal,source,reference,approved_at,cache,now,human).map_err(Failure::control_error)
 }
 pub(in crate::pet_runtime)fn ack_native_plan(&self,path:&Path,id:&str,pet:&str,revision:&str,review_id:&str,now:u64)->Result<RevisionReceipt>{
  let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let seal={let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;if goal.spec.owner.pet_id!=pet{return Err(Error::Conflict);}let s=authority.native_plan_seal(&goal,review_id).map_err(Failure::control_error)?;
   if revision!=s.reference.source_revision&&revision!=s.reference.target_revision{return Err(Error::Conflict);}s};
  let cert={let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;authority.native_plan_ack_certificate(&goal,&seal).map_err(Failure::control_error)?};
  let commit=self.commit_plan(&cert,now)?;let acknowledged=if let Some(p)=commit.published.as_ref(){authority.acknowledge_native_plan(p).map_err(Failure::control_error)?;true}else{false};
  Ok(RevisionReceipt{receipt:commit.receipt,reference:seal.reference,acknowledged,execution_authorized:false,new_admission_created:false})
 }
 pub(in crate::pet_runtime)fn abandon_native_plan(&self,path:&Path,id:&str,pet:&str,revision:&str,review_id:&str,now:u64)->Result<Seal>{
  let authority=Authority::existing(path).map_err(Failure::control_error)?;self.bound_authority(&authority)?;
  let seal={let _guard=self.lock(false)?;let(goal,_)=self.load_with_pending(id,true)?.ok_or(Error::NotFound)?;if goal.spec.owner.pet_id!=pet||goal.revision!=revision{return Err(Error::Conflict);}authority.native_plan_seal(&goal,review_id).map_err(Failure::control_error)?};
  authority.abandon_native_plan(&self.native_plan_absence(seal.clone()),&seal,now).map_err(Failure::control_error)
 }
}
