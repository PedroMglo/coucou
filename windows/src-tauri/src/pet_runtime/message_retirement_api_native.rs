use super::{model,MessageIntentsInput,MessageRetireInput,MessageRetireConfirmInput};
use crate::pet_runtime::{model::{Error,Record},Repository,store::{Store,nonce},authority::Authority,message_api::native::Origin,message_retirement::{self as retire,Review,Approved},authority_store::message_retirement::{Prepared as CorePrepared,IntentInventory}};
use serde::Serialize;
use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:retire::Proposal}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Prepared{Review{ticket:Ticket},Retired{receipt:retire::Receipt}}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Confirmed{Retired{receipt:retire::Receipt},Declined,Blocked{reason:Error,retained:bool}}
enum Native{Review(Review),Approved(Approved),Done(retire::Receipt),Ended}
pub(in crate::pet_runtime)struct Pending{origin:Origin,proposal:retire::Proposal,request:String,native:Mutex<Native>}
impl Pending{
 fn ticket(&self,token:String)->Ticket{Ticket{token,proposal:self.proposal.clone()}}
 pub(in crate::pet_runtime)fn choose(&self,goal:&Record,approve:bool,now:u64)->model::Result<Confirmed>{
  let mut n=self.native.try_lock().map_err(|_|Error::Busy)?;
  // A completed cleanup is a past native receipt, never another deletion or
  // permission. Retain it for an ACK lost between IPC and the human control.
  if let Native::Done(receipt)=&*n{return Ok(Confirmed::Retired{receipt:receipt.clone()});}
  let r=match &*n{Native::Review(r)=>r,Native::Approved(a)=>&a.review,Native::Ended|Native::Done(_)=>return Err(Error::Conflict)};
  if let Some(reference)=r.authority.message_retirement_at(goal,&self.proposal.decision_id,&self.proposal.message.message_id).map_err(crate::pet_runtime::authority::Failure::control_error)?{
   let receipt=r.authority.ack_message_retirement(goal,&reference).map_err(crate::pet_runtime::authority::Failure::control_error)?;*n=Native::Done(receipt.clone());return Ok(Confirmed::Retired{receipt});
  }
  if !approve{if matches!(*n,Native::Approved(_)){return Ok(Confirmed::Blocked{reason:Error::Conflict,retained:true});}*n=Native::Ended;return Ok(Confirmed::Declined);}
  if !self.origin.frame.current()||goal.id!=self.proposal.goal_id||goal.spec.owner.pet_id!=self.proposal.pet_id||goal.revision!=self.proposal.revision{return Err(Error::Conflict);}
  if matches!(*n,Native::Review(_)){let old=std::mem::replace(&mut*n,Native::Ended);let Native::Review(review)=old else{return Err(Error::Conflict);};let frame=self.origin.frame.clone();*n=Native::Approved(review.approve(now,Arc::new(move||frame.current()))?);}
  let Native::Approved(a)=&mut*n else{return Err(Error::Conflict);};match a.confirm(now){Ok(receipt)=>{*n=Native::Done(receipt.clone());Ok(Confirmed::Retired{receipt})},Err(reason)=>Ok(Confirmed::Blocked{reason,retained:true})}
 }
}
#[derive(Default)]pub(in crate::pet_runtime)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
impl Reviews{
 fn sweep(&mut self,now:u64){self.pending.retain(|_,p|match p.native.try_lock(){Ok(n)=>match &*n{Native::Approved(_)=>true,Native::Ended=>false,_=>p.origin.frame.current()&&now<p.proposal.expires_at},Err(_)=>true});}
 pub(in crate::pet_runtime)fn existing(&mut self,origin:&Origin,input:&MessageRetireInput,now:u64)->model::Result<Option<Ticket>>{self.sweep(now);for(token,p)in &self.pending{if p.origin.window==origin.window&&p.proposal.goal_id==input.goal_id&&p.request==input.request_id{if !p.origin.same(origin)||p.proposal.pet_id!=input.pet_id||p.proposal.revision!=input.revision||p.proposal.decision_id!=input.decision_id||p.proposal.message.message_id!=input.message_id{return Err(Error::Conflict);}return Ok(Some(p.ticket(token.clone())));}}Ok(None)}
 pub(in crate::pet_runtime)fn insert(&mut self,origin:Origin,input:&MessageRetireInput,review:Review,now:u64)->model::Result<Ticket>{if !origin.frame.current(){return Err(Error::Conflict);}if let Some(t)=self.existing(&origin,input,now)?{return Ok(t);}if self.pending.len()>=128{return Err(Error::LimitReached);}let proposal=review.proposal()?;if proposal.goal_id!=input.goal_id||proposal.pet_id!=input.pet_id||proposal.revision!=input.revision||proposal.decision_id!=input.decision_id||proposal.message.message_id!=input.message_id||review.request!=input.request_id{return Err(Error::Conflict);}let token=nonce()?;let p=Arc::new(Pending{origin,proposal,request:input.request_id.clone(),native:Mutex::new(Native::Review(review))});let t=p.ticket(token.clone());self.pending.insert(token,p);Ok(t)}
 pub(in crate::pet_runtime)fn lookup(&self,origin:&Origin,input:&MessageRetireConfirmInput)->model::Result<Arc<Pending>>{let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;if !p.origin.same(origin)||p.proposal.pet_id!=input.pet_id||p.proposal.goal_id!=input.goal_id||p.proposal.revision!=input.revision||p.proposal.decision_id!=input.decision_id{return Err(Error::Conflict);}Ok(p.clone())}
}
fn reviews()->&'static Mutex<Reviews>{static R:OnceLock<Mutex<Reviews>>=OnceLock::new();R.get_or_init(||Mutex::new(Reviews::default()))}
fn store()->model::Result<Store>{Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(Error::NotFound)}
fn authority()->model::Result<Authority>{Authority::existing(&crate::pet_runtime::admission_review::authority_root()).map_err(|_|Error::StorageUnavailable)}
pub(super)fn pending(input:MessageIntentsInput)->model::Result<IntentInventory>{let goal=store()?.read_control(&input.goal_id,&input.pet_id)?;authority()?.pending_messages(&goal,&input.decision_id).map_err(crate::pet_runtime::authority::Failure::control_error)}
pub(super)fn prepare(origin:Origin,input:MessageRetireInput)->model::Result<Prepared>{
 let goal=store()?.read_control(&input.goal_id,&input.pet_id)?;if goal.revision!=input.revision||!model::key(&input.request_id){return Err(Error::Conflict);}let now=crate::assistant_memory::now();if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.existing(&origin,&input,now)?{return Ok(Prepared::Review{ticket});}
 match authority()?.prepare_message_retirement(goal,&input.decision_id,&input.message_id,input.request_id.clone(),now).map_err(crate::pet_runtime::authority::Failure::control_error)?{
  CorePrepared::Review(r)=>Ok(Prepared::Review{ticket:reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(origin,&input,r,now)?}),
  CorePrepared::Retired(receipt)=>{crate::pet_runtime::message_api::native::forget_retired(&receipt.reference.scope.envelope.message_id);Ok(Prepared::Retired{receipt})},
 }
}
pub(super)fn confirm(origin:Origin,input:MessageRetireConfirmInput)->model::Result<Confirmed>{let p=reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;let goal=store()?.read_control(&input.goal_id,&input.pet_id)?;let result=p.choose(&goal,input.approve,crate::assistant_memory::now())?;if let Confirmed::Retired{receipt}=&result{crate::pet_runtime::message_api::native::forget_retired(&receipt.reference.scope.envelope.message_id);}Ok(result)}
