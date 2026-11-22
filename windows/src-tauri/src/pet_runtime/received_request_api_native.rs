//! Keep the same native decision over uncertain data IO. No scheduler/ledger.
use super::*;
use crate::pet_runtime::{message_api::native::Origin,received_request::{Review,Approved,Caller},store::{Store,nonce},model::{Error,Result,CommitStatus},messages::Target};
use std::{collections::BTreeMap,sync::{Arc,Mutex,OnceLock},path::PathBuf};
enum Native{Review(Option<Review>),Approved(Approved),Done,Ended}
pub(in crate::pet_runtime)struct Pending{origin:Origin,input:ReceivedGoalPrepareInput,proposal:received_request::Proposal,root:PathBuf,authority:PathBuf,human:Arc<dyn Fn()->bool+Send+Sync>,state:Mutex<Native>}
impl Pending{
 fn ticket(&self,token:String)->ReceivedGoalTicket{ReceivedGoalTicket{token,proposal:self.proposal.clone()}}
 pub(in crate::pet_runtime)fn choose(&self,approve:bool,now:u64)->Result<ReceivedGoalConfirmed>{
  let mut native=self.state.try_lock().map_err(|_|Error::Busy)?;let store=Store::existing_verified(&self.root,&self.authority)?.ok_or(Error::NotFound)?;let i=&self.input;
  // Resolve the durable fact before choice/source/expiry/policy. This grants no
  // new authority and never changes intent/criteria or resurrects the sender.
  if let Some(receipt)=if self.proposal.schema_version==4{store.existing_recursive_funded_request(&i.pet_id,&i.source,&i.request_id,&i.intent,&i.criteria)?}else if matches!(self.proposal.schema_version,3|5){store.existing_result_request(&i.pet_id,&i.source,&i.request_id,&i.intent,&i.criteria)?}else if self.proposal.schema_version==6{store.existing_planned_request(&i.pet_id,&i.source,&i.request_id,&i.intent,&i.criteria)?}else if matches!(self.proposal.schema_version,2|7){store.existing_funded_request(&i.pet_id,&i.source,&i.request_id,&i.intent,&i.criteria)?}else{store.existing_received_request(&i.pet_id,&i.source,&i.request_id,&i.intent,&i.criteria)?}{if receipt.status!=CommitStatus::Uncertain{*native=Native::Done;}return Ok(ReceivedGoalConfirmed::Created{receipt,execution_authority_granted:false});}
  if !self.origin.frame.current(){return Err(Error::Conflict);}
  if let Native::Review(review)=&mut *native{
   if !approve{*native=Native::Ended;return Ok(ReceivedGoalConfirmed::Declined);}
   if !(self.human)(){return Err(Error::Conflict);}let original=review.take().ok_or(Error::Conflict)?;
   // approve consumes the review, so retain an explicit ended state on refusal.
   *native=Native::Ended;let approved=original.approve(now,self.human.clone())?;*native=Native::Approved(approved);
  }
  let Native::Approved(approved)=&mut *native else{return Err(Error::Conflict);};
  if !approve{return Err(Error::Conflict);}let receipt=approved.confirm_clock(&store,crate::assistant_memory::now)?;if receipt.status!=CommitStatus::Uncertain{*native=Native::Done;}Ok(ReceivedGoalConfirmed::Created{receipt,execution_authority_granted:false})
 }
}
#[derive(Default)]pub(in crate::pet_runtime)struct Reviews{pending:BTreeMap<String,Arc<Pending>>}
impl Reviews{
 fn sweep(&mut self,now:u64){self.pending.retain(|_,p|match p.state.try_lock(){Ok(s)=>match &*s{Native::Approved(_)=>true,Native::Done|Native::Ended=>false,Native::Review(_)=>p.origin.frame.current()&&now<p.proposal.expires_at},Err(_)=>true});}
 pub(in crate::pet_runtime)fn existing(&mut self,origin:&Origin,input:&ReceivedGoalPrepareInput,now:u64)->Result<Option<ReceivedGoalTicket>>{
  self.sweep(now);for(token,p)in &self.pending{if p.input.pet_id==input.pet_id&&p.input.request_id==input.request_id{if p.input!=*input||!p.origin.same(origin){return Err(Error::Conflict);}return Ok(Some(p.ticket(token.clone())));}}Ok(None)
 }
 pub(in crate::pet_runtime)fn insert(&mut self,origin:Origin,input:ReceivedGoalPrepareInput,review:Review,root:PathBuf,authority:PathBuf,human:Arc<dyn Fn()->bool+Send+Sync>,now:u64)->Result<ReceivedGoalTicket>{
  if let Some(ticket)=self.existing(&origin,&input,now)?{return Ok(ticket);}if self.pending.len()>=128{return Err(Error::LimitReached);}if !origin.frame.current()||!human(){return Err(Error::Conflict);}let proposal=review.proposal();if proposal.future.owner.pet_id!=input.pet_id||proposal.source.reference!=input.source||proposal.future.intent!=input.intent||proposal.future.criteria!=input.criteria||proposal.client_request_id!=input.request_id||now<proposal.prepared_at||now>=proposal.expires_at{return Err(Error::Conflict);}
  let token=nonce()?;let p=Arc::new(Pending{origin,input,proposal,root,authority,human,state:Mutex::new(Native::Review(Some(review)))});let ticket=p.ticket(token.clone());self.pending.insert(token,p);Ok(ticket)
 }
 pub(in crate::pet_runtime)fn lookup(&self,origin:&Origin,input:&ReceivedGoalConfirmInput)->Result<Arc<Pending>>{let p=self.pending.get(&input.token).ok_or(Error::Conflict)?;if !p.origin.same(origin)||input.pet_id!=p.input.pet_id||input.source!=p.input.source{return Err(Error::Conflict);}Ok(p.clone())}
}
fn reviews()->&'static Mutex<Reviews>{static REVIEWS:OnceLock<Mutex<Reviews>>=OnceLock::new();REVIEWS.get_or_init(||Mutex::new(Reviews::default()))}
fn funded_reviews()->&'static Mutex<Reviews>{static R:OnceLock<Mutex<Reviews>>=OnceLock::new();R.get_or_init(||Mutex::new(Reviews::default()))}
pub(super)fn prepare(app:&tauri::AppHandle,origin:Origin,input:ReceivedGoalPrepareInput)->Result<ReceivedGoalPrepared>{prepare_mode(app,origin,input,received_request::Purpose::Request)}
pub(super)fn prepare_funded(app:&tauri::AppHandle,origin:Origin,input:ReceivedGoalPrepareInput)->Result<ReceivedGoalPrepared>{prepare_mode(app,origin,input,received_request::Purpose::Funded)}
fn result_reviews()->&'static Mutex<Reviews>{static R:OnceLock<Mutex<Reviews>>=OnceLock::new();R.get_or_init(||Mutex::new(Reviews::default()))}
pub(super)fn prepare_result(app:&tauri::AppHandle,origin:Origin,input:ReceivedGoalPrepareInput)->Result<ReceivedGoalPrepared>{prepare_mode(app,origin,input,received_request::Purpose::Result)}
fn prepare_mode(app:&tauri::AppHandle,origin:Origin,input:ReceivedGoalPrepareInput,purpose:received_request::Purpose)->Result<ReceivedGoalPrepared>{
 let recursive=purpose==received_request::Purpose::RecursiveFunded;let funded=matches!(purpose,received_request::Purpose::Funded|received_request::Purpose::RecursiveFunded);let derived=purpose==received_request::Purpose::Result;
 let reviews=||if recursive{recursive_reviews()}else if derived{result_reviews()}else if funded{funded_reviews()}else{self::reviews()};
 if !model::key(&input.pet_id)||!model::key(&input.request_id)||!input.source.valid(){return Err(Error::InvalidInput);}let root=crate::settings::config_dir().join("pet-runtime-v1");let authority=crate::pet_runtime::admission_review::authority_root();let store=Store::ensure_native(&root)?;
 if let Some(receipt)=if recursive{store.existing_recursive_funded_request(&input.pet_id,&input.source,&input.request_id,&input.intent,&input.criteria)?}else if derived{store.existing_result_request(&input.pet_id,&input.source,&input.request_id,&input.intent,&input.criteria)?}else if funded{store.existing_funded_request(&input.pet_id,&input.source,&input.request_id,&input.intent,&input.criteria)?}else{store.existing_received_request(&input.pet_id,&input.source,&input.request_id,&input.intent,&input.criteria)?}{return Ok(ReceivedGoalPrepared::Existing{receipt,execution_authority_granted:false});}
 if let Some(ticket)=reviews().lock().map_err(|_|Error::StorageUnavailable)?.existing(&origin,&input,crate::assistant_memory::now())?{return Ok(ReceivedGoalPrepared::Review{ticket});}
 let p=crate::pet_capabilities::native_projection(app,None).map_err(|_|Error::StorageUnavailable)?;if !p.preferences_ready{return Err(Error::Conflict);}let config=p.configuration.as_ref().ok_or(Error::Conflict)?;let descriptor=p.snapshot.pets.iter().find(|d|d.pet_id==input.pet_id).ok_or(Error::InvalidInput)?;
 let caller=Caller::capture(origin.clone(),descriptor)?;let expected=caller.owner().clone();let target=Arc::new(Target::capture(config,&p.settings,&input.pet_id)?);let native_app=app.clone();let frame=origin.frame.clone();
 let human:Arc<dyn Fn()->bool+Send+Sync>=Arc::new(move||{if !frame.current(){return false;}let Ok(p)=crate::pet_capabilities::native_projection(&native_app,None)else{return false;};p.preferences_ready&&p.configuration.as_ref().is_some_and(|c|target.current(c,&p.settings))&&p.snapshot.pets.iter().find(|d|d.pet_id==expected.pet_id).and_then(|d|model::Owner::from_runtime_descriptor(d).ok()).as_ref()==Some(&expected)});
 let now=crate::assistant_memory::now();let review=if recursive{Review::prepare_recursive_funded(&authority,caller,input.source.clone(),input.request_id.clone(),input.intent.clone(),input.criteria.clone(),now)?}else if derived{Review::prepare_result(&authority,caller,input.source.clone(),input.request_id.clone(),input.intent.clone(),input.criteria.clone(),now)?}else if funded{Review::prepare_funded(&authority,caller,input.source.clone(),input.request_id.clone(),input.intent.clone(),input.criteria.clone(),now)?}else{Review::prepare(&authority,caller,input.source.clone(),input.request_id.clone(),input.intent.clone(),input.criteria.clone(),now)?};let ticket=reviews().lock().map_err(|_|Error::StorageUnavailable)?.insert(origin,input,review,root,authority,human,crate::assistant_memory::now())?;Ok(ReceivedGoalPrepared::Review{ticket})
}
pub(super)fn confirm(origin:Origin,input:ReceivedGoalConfirmInput)->Result<ReceivedGoalConfirmed>{let pending=reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;pending.choose(input.approve,crate::assistant_memory::now())}

pub(super)fn confirm_funded(origin:Origin,input:ReceivedGoalConfirmInput)->Result<ReceivedGoalConfirmed>{let pending=funded_reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;pending.choose(input.approve,crate::assistant_memory::now())}

pub(super)fn confirm_result(origin:Origin,input:ReceivedGoalConfirmInput)->Result<ReceivedGoalConfirmed>{let pending=result_reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;pending.choose(input.approve,crate::assistant_memory::now())}

fn recursive_reviews()->&'static Mutex<Reviews>{static R:OnceLock<Mutex<Reviews>>=OnceLock::new();R.get_or_init(||Mutex::new(Reviews::default()))}
pub(super)fn prepare_recursive_funded(app:&tauri::AppHandle,origin:Origin,input:ReceivedGoalPrepareInput)->Result<ReceivedGoalPrepared>{prepare_mode(app,origin,input,received_request::Purpose::RecursiveFunded)}
pub(super)fn confirm_recursive_funded(origin:Origin,input:ReceivedGoalConfirmInput)->Result<ReceivedGoalConfirmed>{let pending=recursive_reviews().lock().map_err(|_|Error::StorageUnavailable)?.lookup(&origin,&input)?;pending.choose(input.approve,crate::assistant_memory::now())}
