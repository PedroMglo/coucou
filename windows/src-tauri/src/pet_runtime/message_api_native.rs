//! Retain the original native Frame/Reviewed/Approved across uncertain IO.
//! No global cache lock is held while accessing storage, config or workers.
use super::{model,PrepareInput,ConfirmInput,ReadInput,MessageInboxInput};
use crate::pet_runtime::{Repository,model::{Error,Record},authority::Authority,store::{Store,nonce},messages::{self,Target,Input,Receipt},lease_controller::{Owned,message_controller::{Review,Approved,Proposal}}};
use serde::Serialize;
use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock}};
#[derive(Clone)]pub(in crate::pet_runtime)struct Origin{pub window:String,pub frame:crate::settings::review::Frame}
impl Origin{pub(in crate::pet_runtime)fn same(&self,other:&Self)->bool{self.window==other.window&&self.frame.same(&other.frame)&&self.frame.current()&&other.frame.current()}}
#[derive(Clone,PartialEq,Eq)]pub(in crate::pet_runtime)struct Selection{pet:String,goal:String,lease:String,revision:String,plan:String,step:String,request:String}
impl Selection{pub(in crate::pet_runtime)fn from(input:&PrepareInput)->Self{Self{pet:input.pet_id.clone(),goal:input.goal_id.clone(),lease:input.lease_id.clone(),revision:input.revision.clone(),plan:input.plan_id.clone(),step:input.step_id.clone(),request:input.request_id.clone()}}}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:Proposal}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum ConfirmResult{Received{receipt:Receipt},Declined,Blocked{reason:Error,retained:bool}}
enum Native{Review(Review),Approved(Approved),Done(Receipt),Ended}
pub(in crate::pet_runtime)struct Pending{origin:Origin,selection:Selection,proposal:Proposal,native:Mutex<Native>}
impl Pending{
 fn ticket(&self,token:String)->Ticket{Ticket{token,proposal:self.proposal.clone()}}
 fn matches_receipt(&self,r:&Receipt)->bool{r.envelope()==&self.proposal.envelope&&r.scope.goal_revision==self.selection.revision&&r.scope.request_id==self.selection.request}
 pub(in crate::pet_runtime)fn choose(&self,approve:bool,goal:&Record,owner:impl Fn()->model::Result<Arc<Owned>>,observe:impl Fn()->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>,ack:impl Fn()->model::Result<Option<Receipt>>,now:u64)->model::Result<ConfirmResult>{
  let mut native=self.native.try_lock().map_err(|_|Error::Busy)?;
  // A recorded receipt is an observation, including after Stop, expiry or a
  // lost ACK. A later decline must not falsely report that data was not sent.
  if let Some(receipt)=ack()?{if !self.matches_receipt(&receipt){return Err(Error::Conflict);}*native=Native::Done(receipt.clone());return Ok(ConfirmResult::Received{receipt});}
  if let Native::Done(receipt)=&*native{return Ok(ConfirmResult::Received{receipt:receipt.clone()});}
  if matches!(*native,Native::Ended){return Err(Error::Conflict);}
  if !approve{
   if matches!(*native,Native::Approved(_)){return Ok(ConfirmResult::Blocked{reason:Error::Conflict,retained:true});}
   *native=Native::Ended;return Ok(ConfirmResult::Declined);
  }
  if !self.origin.frame.current()||goal.id!=self.selection.goal||goal.spec.owner.pet_id!=self.selection.pet||goal.revision!=self.selection.revision{return Err(Error::Conflict);}
  let owner=owner()?;
  if matches!(*native,Native::Review(_)){
   let(config,settings)=observe()?;let old=std::mem::replace(&mut*native,Native::Ended);
   let Native::Review(review)=old else{return Err(Error::Conflict);};let frame=self.origin.frame.clone();
   *native=Native::Approved(review.approve(&owner,goal,&config,&settings,now,Arc::new(move||frame.current()))?);
  }
  let Native::Approved(approved)=&*native else{return Err(Error::Conflict);};
  match owner.send_message(goal,approved,now,observe){
   Ok(receipt)=>{*native=Native::Done(receipt.clone());Ok(ConfirmResult::Received{receipt})},
   Err(reason)=>Ok(ConfirmResult::Blocked{reason,retained:true}),
  }
 }
}
#[derive(Default)]pub(in crate::pet_runtime)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
impl Reviews{
 fn sweep(&mut self,now:u64){self.pending.retain(|_,p|match p.native.try_lock(){Ok(n)=>match &*n{Native::Approved(_)=>true,Native::Ended=>false,_=>p.origin.frame.current()&&now<p.proposal.expires_at},Err(_)=>true});}
 pub(in crate::pet_runtime)fn existing(&mut self,origin:&Origin,selection:&Selection,now:u64)->model::Result<Option<Ticket>>{
  self.sweep(now);for(token,p)in &self.pending{if p.origin.window==origin.window&&p.selection.goal==selection.goal&&p.selection.request==selection.request{
   if p.selection!=*selection||!p.origin.same(origin){return Err(Error::Conflict);}return Ok(Some(p.ticket(token.clone())));
  }}Ok(None)
 }
 pub(in crate::pet_runtime)fn insert(&mut self,origin:Origin,selection:Selection,goal:&Record,review:Review,now:u64)->model::Result<Ticket>{
  if !origin.frame.current(){return Err(Error::Conflict);}if let Some(ticket)=self.existing(&origin,&selection,now)?{return Ok(ticket);}if self.pending.len()>=128{return Err(Error::LimitReached);}
  let token=nonce()?;let p=Arc::new(Pending{origin,selection,proposal:review.proposal(goal),native:Mutex::new(Native::Review(review))});let ticket=p.ticket(token.clone());self.pending.insert(token,p);Ok(ticket)
 }
 pub(in crate::pet_runtime)fn lookup(&self,origin:&Origin,input:&ConfirmInput)->model::Result<Arc<Pending>>{
  let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;let s=&p.selection;if !p.origin.same(origin)||s.pet!=input.pet_id||s.goal!=input.goal_id||s.lease!=input.lease_id||s.revision!=input.revision{return Err(Error::Conflict);}Ok(p.clone())
 }
}
fn reviews()->&'static Mutex<Reviews>{static R:OnceLock<Mutex<Reviews>>=OnceLock::new();R.get_or_init(||Mutex::new(Reviews::default()))}
fn store()->model::Result<Store>{Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(Error::NotFound)}
fn authority()->model::Result<Authority>{Authority::existing(&crate::pet_runtime::admission_review::authority_root()).map_err(|_|Error::StorageUnavailable)}
fn observe(app:&tauri::AppHandle)->model::Result<(crate::pet_config::Configuration,crate::settings::Settings)>{
 let p=crate::pet_capabilities::native_projection(app,None).map_err(|_|Error::StorageUnavailable)?;if !p.preferences_ready{return Err(Error::Conflict);}Ok((p.configuration.ok_or(Error::StorageUnavailable)?,p.settings))
}
pub(super)fn prepare(app:&tauri::AppHandle,origin:Origin,input:PrepareInput)->model::Result<Ticket>{
 if !model::key(&input.request_id){return Err(Error::InvalidInput);}let goal=store()?.read(&input.goal_id,&input.pet_id)?;if goal.revision!=input.revision{return Err(Error::Conflict);}let selection=Selection::from(&input);let now=crate::assistant_memory::now();
 if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.existing(&origin,&selection,now)?{return Ok(ticket);}
 let owner=crate::pet_runtime::lease_api::step_owner(&goal,&input.lease_id)?;
 let step=goal.plans.iter().find(|p|p.id==input.plan_id).and_then(|p|p.steps.iter().find(|s|s.id==input.step_id)).and_then(|s|s.operation.as_ref()).ok_or(Error::InvalidInput)?;
 let arguments=Input::parse(step).map_err(|_|Error::InvalidInput)?;let(config,settings)=observe(app)?;let target=Arc::new(Target::capture(&config,&settings,arguments.to_pet())?);
 let review=owner.review_message(&goal,&input.plan_id,&input.step_id,input.request_id,target,now)?;
 reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(origin,selection,&goal,review,now)
}
pub(super)fn confirm(app:&tauri::AppHandle,origin:Origin,input:ConfirmInput)->model::Result<ConfirmResult>{
 let pending=reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;
 let goal=store()?.read_control(&input.goal_id,&input.pet_id)?;let a=authority()?;let e=&pending.proposal.envelope;let row=goal.admissions.iter().find(|r|r.spec.decision_id==e.origin.decision_id).ok_or(Error::Conflict)?;
 let result=pending.choose(input.approve,&goal,||crate::pet_runtime::lease_api::step_owner(&goal,&input.lease_id),||observe(app),||{
  let Some(receipt)=a.message_at(&row.spec,row.proof.as_ref().ok_or(Error::Conflict)?,&e.cause.plan_id,&e.cause.step_id).map_err(crate::pet_runtime::authority::Failure::control_error)?else{return Ok(None);};
  if !pending.matches_receipt(&receipt){return Err(Error::Conflict);}a.message_ack(&row.spec,row.proof.as_ref().ok_or(Error::Conflict)?,&receipt.scope).map_err(crate::pet_runtime::authority::Failure::control_error)
 },crate::assistant_memory::now())?;
 if let ConfirmResult::Received{receipt}=&result{crate::pet_runtime::native_completion_api::after_message(app,receipt);}
 Ok(result)
}
pub(super)fn result(input:ReadInput)->model::Result<Option<Receipt>>{
 let goal=store()?.read_control(&input.goal_id,&input.pet_id)?;let row=goal.admissions.iter().find(|r|r.spec.decision_id==input.decision_id).ok_or(Error::NotFound)?;let a=authority()?;let proof=row.proof.as_ref().ok_or(Error::Conflict)?;
 let Some(receipt)=a.message_at(&row.spec,proof,&input.plan_id,&input.step_id).map_err(crate::pet_runtime::authority::Failure::control_error)?else{return Ok(None);};
 a.message_ack(&row.spec,proof,&receipt.scope).map_err(crate::pet_runtime::authority::Failure::control_error)
}
pub(super)fn inbox(input:MessageInboxInput)->model::Result<messages::Inbox>{authority()?.message_inbox_funding_view(&input.pet_id,input.after.as_deref(),input.limit,crate::assistant_memory::now(),input.include_historical,input.include_reply_sources,input.include_funding_sources).map_err(crate::pet_runtime::authority::Failure::control_error)}

pub(in crate::pet_runtime)fn forget_retired(message:&str){if let Ok(mut c)=reviews().lock(){c.pending.retain(|_,p|{if p.proposal.envelope.message_id!=message{return true;}let Ok(mut n)=p.native.try_lock()else{return true;};*n=Native::Ended;false});}}
