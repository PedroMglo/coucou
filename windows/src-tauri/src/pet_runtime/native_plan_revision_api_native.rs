use super::{model,PrepareNativePlanInput,ConfirmNativePlanInput,NativePlanListInput,NativePlanControlInput};
use crate::pet_runtime::{model::{Error,Record},Repository,store::{Store,nonce},message_api::native::Origin,native_plan_review::{Review,Approved,Proposal,Receipt},native_plan_revisions::Reference,native_plan_inventory::Inventory};
use serde::Serialize;
use std::{collections::BTreeMap,path::Path,sync::{Arc,Mutex,OnceLock}};
const MAX_PENDING:usize=32;
const MAX_BYTES:usize=32*1024*1024;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct Ticket{pub token:String,pub proposal:Proposal}
#[derive(Serialize)]#[serde(tag="status",rename_all="camelCase")]
pub enum Confirmed{Revised{receipt:Receipt},Declined,Blocked{reason:Error,retained:bool}}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Withdrawn{pub reference:Reference,pub withdrawn:bool,pub execution_authorized:bool,pub budget_refund:bool}
enum Native{Review(Review),Approved(Approved),Done(Reference),Ended}
pub(in crate::pet_runtime)struct Pending{origin:Origin,proposal:Proposal,request:String,inputs_hash:String,retained_bytes:usize,native:Mutex<Native>}
impl Pending{
 fn ticket(&self,token:String)->Ticket{Ticket{token,proposal:self.proposal.clone()}}
 pub(in crate::pet_runtime)fn choose(&self,store:&Store,path:&Path,goal:&Record,approve:bool,now:u64)->model::Result<Confirmed>{
  let mut n=self.native.try_lock().map_err(|_|Error::Busy)?;let r=&self.proposal.changes;
  if goal.id!=self.proposal.goal_id||goal.spec.owner.pet_id!=self.proposal.pet_id{return Err(Error::Conflict);}
  if matches!(&*n,Native::Done(saved)if saved!=r){return Err(Error::Conflict);}
  if matches!(*n,Native::Done(_))||goal.native_plan_revisions.iter().any(|v|v==r){
   let receipt=store.ack_native_plan(path,&r.goal_id,&r.pet_id,&r.source_revision,&r.id,now)?;*n=Native::Done(r.clone());return Ok(Confirmed::Revised{receipt});
  }
  if !approve{
   if matches!(&*n,Native::Approved(a)if a.seal.is_some()){return Ok(Confirmed::Blocked{reason:Error::Conflict,retained:true});}
   *n=Native::Ended;return Ok(Confirmed::Declined);
  }
  if !self.origin.frame.current()||goal.revision!=self.proposal.revision{return Err(Error::Conflict);}
  if matches!(*n,Native::Review(_)){let old=std::mem::replace(&mut*n,Native::Ended);let Native::Review(review)=old else{return Err(Error::Conflict);};let frame=self.origin.frame.clone();*n=Native::Approved(review.approve(now,Arc::new(move||frame.current()))?);}
  let Native::Approved(a)=&mut*n else{return Err(Error::Conflict);};
  match a.confirm(store,now){Ok(receipt)=>{if receipt.acknowledged{*n=Native::Done(r.clone());}Ok(Confirmed::Revised{receipt})},Err(reason)=>Ok(Confirmed::Blocked{reason,retained:true})}
 }
}
#[derive(Default)]pub(in crate::pet_runtime)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
impl Reviews{
 fn sweep(&mut self,now:u64){self.pending.retain(|_,p|match p.native.try_lock(){Ok(n)=>match &*n{Native::Approved(a)if a.seal.is_some()=>true,Native::Ended=>false,_=>p.origin.frame.current()&&now<p.proposal.expires_at},Err(_)=>true});}
 pub(in crate::pet_runtime)fn existing(&mut self,origin:&Origin,input:&PrepareNativePlanInput,now:u64)->model::Result<Option<Ticket>>{
  self.sweep(now);let hash=model::fingerprint(&input.inputs)?;
  for(token,p)in &self.pending{if p.origin.window==origin.window&&p.proposal.goal_id==input.goal_id&&p.request==input.request_id{
   if !p.origin.same(origin)||p.proposal.pet_id!=input.pet_id||p.proposal.revision!=input.revision||p.inputs_hash!=hash{return Err(Error::Conflict);}return Ok(Some(p.ticket(token.clone())));
  }}Ok(None)
 }
 pub(in crate::pet_runtime)fn insert(&mut self,origin:Origin,input:&PrepareNativePlanInput,review:Review,now:u64)->model::Result<Ticket>{
  if !origin.frame.current(){return Err(Error::Conflict);}if let Some(t)=self.existing(&origin,input,now)?{return Ok(t);}let proposal=review.proposal()?;let hash=model::fingerprint(&input.inputs)?;
  let expected=review.reference.patches.iter().map(|p|crate::pet_runtime::native_plan_revisions::Input{cause:p.cause.clone(),after:p.after.clone()}).collect::<Vec<_>>();
  if proposal.pet_id!=input.pet_id||proposal.goal_id!=input.goal_id||proposal.revision!=input.revision||review.reference.request_id!=input.request_id||model::fingerprint(&expected)?!=hash{return Err(Error::Conflict);}
  let bytes=serde_json::to_vec(&review.goal).map_err(|_|Error::InvalidRecord)?.len().saturating_add(crate::pet_runtime::plans::encode_input(&proposal)?.len());
  let total=self.pending.values().try_fold(bytes,|sum,p|sum.checked_add(p.retained_bytes)).ok_or(Error::LimitReached)?;
  if self.pending.len()>=MAX_PENDING||total>MAX_BYTES{return Err(Error::LimitReached);}let token=nonce()?;let p=Arc::new(Pending{origin,proposal,request:input.request_id.clone(),inputs_hash:hash,retained_bytes:bytes,native:Mutex::new(Native::Review(review))});let ticket=p.ticket(token.clone());self.pending.insert(token,p);Ok(ticket)
 }
 pub(in crate::pet_runtime)fn lookup(&self,origin:&Origin,input:&ConfirmNativePlanInput)->model::Result<Arc<Pending>>{
  let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;
  if !p.origin.same(origin)||p.proposal.pet_id!=input.pet_id||p.proposal.goal_id!=input.goal_id||p.proposal.revision!=input.revision{return Err(Error::Conflict);}Ok(p.clone())
 }
 fn withdrawn(&mut self,id:&str){self.pending.retain(|_,p|p.proposal.changes.id!=id);}
}
fn reviews()->&'static Mutex<Reviews>{static R:OnceLock<Mutex<Reviews>>=OnceLock::new();R.get_or_init(||Mutex::new(Reviews::default()))}
fn store()->model::Result<Store>{Store::existing_native(&crate::settings::config_dir().join("pet-runtime-v1"))?.ok_or(Error::NotFound)}
fn root()->std::path::PathBuf{crate::pet_runtime::admission_review::authority_root()}
pub(super)fn inventory(input:NativePlanListInput)->model::Result<Inventory>{store()?.native_plan_inventory(&root(),&input.goal_id,&input.pet_id,&input.revision,input.cursor.as_deref())}
pub(super)fn prepare(app:tauri::AppHandle,origin:Origin,input:PrepareNativePlanInput)->model::Result<Ticket>{
  let now=crate::assistant_memory::now();if !model::key(&input.request_id)||!origin.frame.current(){return Err(Error::Conflict);}
  if let Some(t)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.existing(&origin,&input,now)?{return Ok(t);}
  let review=store()?.prepare_native_plan(&root(),&input.goal_id,&input.pet_id,&input.revision,input.inputs.clone(),input.request_id.clone(),now)?;
  let observe:crate::pet_runtime::result_disclosure::Observe=Arc::new(move||{let p=crate::pet_capabilities::native_projection(&app,None).map_err(|_|Error::Conflict)?;Ok((p.configuration.ok_or(Error::Conflict)?,p.settings))});
  let review=review.with_result_data(&root(),&crate::pet_memory::context::Capture::catalog_path(),observe,crate::assistant_memory::now())?;
  reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(origin,&input,review,crate::assistant_memory::now())
}
pub(super)fn confirm(origin:Origin,input:ConfirmNativePlanInput)->model::Result<Confirmed>{
  let pending=reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;let store=store()?;let goal=store.read_control(&input.goal_id,&input.pet_id)?;
  pending.choose(&store,&root(),&goal,input.approve,crate::assistant_memory::now())
}
pub(super)fn ack(origin:Origin,input:NativePlanControlInput)->model::Result<Receipt>{if !origin.frame.current(){return Err(Error::Conflict);}store()?.ack_native_plan(&root(),&input.goal_id,&input.pet_id,&input.revision,&input.review_id,crate::assistant_memory::now())}
pub(super)fn abandon(origin:Origin,input:NativePlanControlInput)->model::Result<Withdrawn>{
  if !origin.frame.current(){return Err(Error::Conflict);}let seal=store()?.abandon_native_plan(&root(),&input.goal_id,&input.pet_id,&input.revision,&input.review_id,crate::assistant_memory::now())?;
  reviews().lock().map_err(|_|Error::StorageUnavailable)?.withdrawn(&seal.reference.id);Ok(Withdrawn{reference:seal.reference,withdrawn:true,execution_authorized:false,budget_refund:false})
}
