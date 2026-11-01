//! Native-only preparation of a NEW authority when the reviewed source had
//! none. Same HMAC/denial ledger; no grants and no operational publication.
use super::*;
#[derive(Clone)]pub(crate)struct IndependentScope{path:PathBuf,source:String,goals:Vec<GoalRef>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)enum IndependentStep{KeyPersisted,BaseReady,HistoryPrepared}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate)enum IndependentKeyState{Missing,Valid,Invalid}
impl IndependentScope{
 pub(crate)fn capture(path:&Path,source:&str)->Result<Self>{if !hash_valid(source){return Err(Failure::Invalid);}let r=Self{path:path.into(),source:source.into(),goals:goals(path)?};r.current()?;Ok(r)}
 fn current(&self)->Result<()>{if goals(&self.path)?!=self.goals{return Err(Failure::Changed);}Ok(())}
 pub(crate)fn review(&self)->Result<serde_json::Value>{self.current()?;Ok(serde_json::json!({"schemaVersion":1,"purpose":"independentFutureAuthorityPreparation","originalAuthorityPresent":false,"source":self.source,"goals":self.goals,"historicalDecisionIds":self.goals.iter().flat_map(|g|g.decisions.iter()).collect::<BTreeSet<_>>(),"priorEffectAccounting":"preservedUnreconciled","grantsCreated":0,"executionAuthorityGranted":false,"authorityPublishedToRuntime":false}))}
 /// Pure inspection of the same key format used by execution. This neither
 /// repairs a prefix nor returns key material, a writable authority or a grant.
 pub(crate)fn key_state(&self,path:&Path)->Result<IndependentKeyState>{
  self.current()?;
  let Some(dir)=Dir::open(path,true).map_err(|_|Failure::Unsafe)?else{return Ok(IndependentKeyState::Missing);};
  let state=match dir.bytes(KEY,512).map_err(|_|Failure::Unsafe)?{
   None=>IndependentKeyState::Missing,
   Some((raw,_))=>if KeyFile::parse(&raw).is_ok(){IndependentKeyState::Valid}else{IndependentKeyState::Invalid},
  };
  dir.check().map_err(|_|Failure::Changed)?;self.current()?;Ok(state)
 }
 pub(crate)fn prepare(&self,path:&Path,new_generation:bool,current:impl Fn()->Result<()>,mut step:impl FnMut(IndependentStep)->Result<()>)->Result<serde_json::Value>{
  current()?;self.current()?;let dir=Dir::open(path,true).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;let names=dir.names().map_err(|_|Failure::Capacity)?;
  if names.iter().any(|n|!matches!(n.as_str(),KEY|".catalog.lock"|".protocol.json"|INDEX)&&!n.strip_prefix(".restore-index-").is_some_and(token)&&!n.strip_prefix(".restore-protocol-").is_some_and(token)){return Err(Failure::RecoveryRequired);}
  if dir.bytes(KEY,512).map_err(|_|Failure::Unsafe)?.is_none(){
   if !new_generation||!names.is_empty(){return Err(Failure::RecoveryRequired);}current()?;self.current()?;
   let mut secret=[0u8;32];SystemRandom::new().fill(&mut secret).map_err(|_|Failure::Unavailable)?;
   let file=KeyFile{schema_version:1,key_id:nonce().map_err(|_|Failure::Unavailable)?,key:encode(&secret)};
   dir.write_new(KEY,&serde_json::to_vec(&file).map_err(|_|Failure::Invalid)?).map_err(|_|Failure::Uncertain)?;dir.durable().map_err(|_|Failure::Uncertain)?;
  }
  let(raw,_)=dir.bytes(KEY,512).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?;
  KeyFile::parse(&raw).map_err(|_|Failure::RecoveryRequired)?;step(IndependentStep::KeyPersisted)?;
  for(name,bytes)in[(".protocol.json",PROTOCOL),(".catalog.lock",b"".as_slice())]{
   current()?;self.current()?;if let Some((found,_))=dir.bytes(name,1024).map_err(|_|Failure::Unsafe)?{
    if name==".protocol.json"{protocol_hash(&found)?;}else if found!=bytes{return Err(Failure::RecoveryRequired);}
   }else{dir.write_new(name,bytes).map_err(|_|Failure::Uncertain)?;dir.durable().map_err(|_|Failure::Uncertain)?;}
  }
  step(IndependentStep::BaseReady)?;current()?;self.current()?;
  let r=Review::capture(path,&self.path,&self.source,||{current()?;self.current()})?;
  if r.body.goals!=self.goals||!r.body.grants.is_empty(){return Err(Failure::Changed);}let hash=r.fingerprint()?;r.apply(&hash,||{current()?;self.current()})?;
  step(IndependentStep::HistoryPrepared)?;current()?;self.verify(path)
 }
 pub(crate)fn verify(&self,path:&Path)->Result<serde_json::Value>{
  self.current()?;verify_retained_independent(path,&self.review()?)
 }
}
/// Pure data validation of a retained seed. Never constructs a writable review,
/// a generation opportunity, an admission or a grant from serialized metadata.
pub(crate)fn verify_retained_independent(path:&Path,review:&serde_json::Value)->Result<serde_json::Value>{
  if review["schemaVersion"]!=1||review["purpose"]!="independentFutureAuthorityPreparation"||review["originalAuthorityPresent"]!=false||review["grantsCreated"]!=0{return Err(Failure::Invalid);}
  let source=review["source"].as_str().filter(|s|hash_valid(s)).ok_or(Failure::Invalid)?;let expected:Vec<GoalRef>=serde_json::from_value(review["goals"].clone()).map_err(|_|Failure::Invalid)?;
  let a=Authority::existing(path)?;let _lock=a.lock_shared()?;let b=a.restoration.as_ref().ok_or(Failure::RecoveryRequired)?;
  if b.body.schema_version!=1||b.body.source!=source||b.body.goals!=expected||!b.body.grants.is_empty()||b.body.files!=files(&a)?||b.body.files.iter().any(|f|!matches!(f.name.as_str(),KEY|".catalog.lock"|".protocol.json")){return Err(Failure::Changed);}
  let mut staged=0;for name in a.dir.names().map_err(|_|Failure::Capacity)?{if matches!(name.as_str(),KEY|".catalog.lock"|".protocol.json"|INDEX){continue;}if !name.strip_prefix(".restore-index-").is_some_and(token)&&!name.strip_prefix(".restore-protocol-").is_some_and(token){return Err(Failure::Changed);}staged+=1;if staged>16{return Err(Failure::Capacity);}a.dir.bytes(&name,MAX_INDEX).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;}
  a.fence()?;Ok(serde_json::json!({"schemaVersion":1,"purpose":"independentFutureAuthorityPreparation","keyId":a.key_id,"historyHash":b.hash,"originalAuthorityPresent":false,"historicalGoals":b.body.goal_ids.len(),"historicalDecisions":b.body.decisions.len(),"priorEffectAccounting":"preservedUnreconciled","grantsCreated":0,"executionAuthorityGranted":false,"authorityPublishedToRuntime":false,"runtimeActivationAvailable":false}))
}

#[cfg(test)]pub(crate)fn fixture_copied_goal_catalog(path:&Path)->Record{
 use crate::pet_runtime::{admission_tests::Fixture,authority_tests::{root,queued}};use std::{fs,os::unix::fs::{DirBuilderExt,PermissionsExt}};
 let f=Fixture::new();let draft=f.goal("codex");let a=Authority::ensure(&root(&f)).unwrap();let old=queued(&f,&draft,&a);fs::DirBuilder::new().mode(0o700).create(path).unwrap();
 for e in fs::read_dir(f.path.join("goals")).unwrap(){let e=e.unwrap();assert!(e.path().is_file());let target=path.join(e.file_name());fs::copy(e.path(),&target).unwrap();fs::set_permissions(target,fs::Permissions::from_mode(0o600)).unwrap();}old
}
#[cfg(test)]pub(crate)fn fixture_prepared_seed_denies_old_ids(path:&Path,old:&Record)->bool{
 let a=Authority::existing(path).unwrap();a.verify_queued(old,100_006).is_err()&&a.restoration_current(&old.admissions[0].spec.decision_id,"independent-future-goal").is_err()
}

#[cfg(test)]pub(crate)fn fixture_fresh_independent_goal(path:&Path,authority:&Path)->Record{
 use crate::pet_runtime::{admission_tests::{Fixture,approved},store::Store};let f=Fixture::new();let template=f.goal_named("codex","fresh-after-independent-admission");let store=Store::ensure(path).unwrap();let g=store.create(template.spec,100_000).unwrap().record;let main=crate::chat::Chat::persistent(f.path.join("fresh-main.json"));let(binding,held)=crate::pet_runtime::context::prepare(&f.path.join("fresh-contexts"),&g,&main,crate::pet_runtime::model::PreparationMode::Dedicated,"fresh-independent-context").unwrap();drop(held);let g=store.mutate(&g.id,&g.spec.owner.pet_id,&g.revision,"fresh-independent-context".into(),crate::pet_runtime::model::Action::BindContext{binding},100_001).unwrap().record;let a=Authority::existing(authority).unwrap();let consent=approved(&g,"fresh-independent-queue");let cert=a.certify(&consent,&g,100_004).unwrap();let queued=store.enqueue_certified(&cert,100_005).unwrap().record;assert!(a.verify_queued(&queued,100_006).is_ok());queued
}
