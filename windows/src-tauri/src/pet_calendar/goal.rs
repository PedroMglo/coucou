//! Immutable calendar DATA for an explicitly selected goal context. Neither a
//! source enrollment nor this serializable reference is an execution permit.
//! Admission must still review the actual data/destination and obtain its own
//! BoundedGoalContext consent before hashing/reserving any model request.
use super::{linux,query,sha,unavailable};
use crate::pet_sources::Selected;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
use std::time::{Duration,Instant};

const MAX_SOURCES:usize=8;
const MAX_DATA:usize=64*1024;
const CAPTURE_TIME:Duration=Duration::from_secs(10);
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Pick{pub id:String,pub revision:u64}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Request{
 #[serde(default,skip_serializing_if="Option::is_none")]pub source_scope:Option<crate::pet_sources::PetScope>,
 pub selected:Vec<Pick>,pub query:query::Query,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct FileReference{stamp:linux::Stamp,content_hash:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Reference{id:String,revision:u64,binding:String,provenance_id:String,name:String,file:Option<FileReference>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Selection{
 pub schema_version:u32,pub goal_id:String,pub pet_id:String,pub provider:String,
 #[serde(default,skip_serializing_if="Option::is_none")]pub source_scope:Option<crate::pet_sources::PetScope>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub source_conversation_id:Option<String>,
 pub query:query::Query,pub captured_at:u64,references:Vec<Reference>,pub data_hash:String,
}
/// Bounded immutable context payload, published inside the existing context
/// header. The admission seals it; no new data store or execution grant.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Context{pub selection:Selection,pub data:Value}
impl Context{
 pub(crate)fn valid(&self,goal:&str,pet:&str,provider:&str,cid:Option<&str>)->bool{
  self.selection.valid(goal,pet,provider,cid)&&serde_json::to_vec(&self.data).is_ok_and(|b|b.len()<=MAX_DATA&&sha(&b)==self.selection.data_hash)
   &&self.data["schemaVersion"]==self.selection.schema_version&&self.data["kind"]=="explicitSelectedGoalCalendars"&&self.data["goalId"]==goal&&self.data["petId"]==pet
   &&(match cid{Some(cid)=>self.data["sourceConversationId"]==cid&&self.data.get("sourceScope").is_none(),None=>self.data["sourceScope"]=="pet"&&self.data.get("sourceConversationId").is_none()})
   &&self.data["capturedAt"]==self.selection.captured_at&&self.data["query"]==json!(self.selection.query)
   &&self.data["untrustedData"]==true&&self.data["authorizesActions"]==false&&self.data["liveSynchronization"]==false
   &&self.data["snapshots"].as_array().is_some_and(|v|v.len()==self.selection.references.len()&&v.iter().zip(&self.selection.references).all(|(snapshot,r)|{
    let source=json!({"id":r.provenance_id,"name":r.name,"revision":r.file.as_ref().map(|f|&f.content_hash)});
    if snapshot["schemaVersion"]!=1||snapshot["kind"]!="calendarEvents"||snapshot["source"]!=source{return false;}
    if snapshot["status"]=="unavailable"{return snapshot["window"]==self.selection.query.window()&&snapshot["events"].as_array().is_some_and(Vec::is_empty)&&snapshot["coverage"]["eventsComplete"]==false&&snapshot["coverage"]["fieldsComplete"]==false&&snapshot["errorCode"].as_str().is_some_and(|c|!c.is_empty()&&c.len()<=64&&c.bytes().all(|b|b.is_ascii_alphanumeric()));}
    query::bound_response(snapshot,&json!({"requestId":snapshot["requestId"],"source":source,"window":self.selection.query.window(),"limit":self.selection.query.limit}))
   }))
 }
 pub(crate)fn matches(&self,request:&Request)->bool{self.selection.matches(request)}
}
fn hash(s:&str)->bool{s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
fn id(s:&str)->bool{!s.is_empty()&&s.len()<=64&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))}
impl Request{
 fn valid(&self)->bool{
  let mut ids=std::collections::BTreeSet::new();!(self.selected.is_empty()||self.selected.len()>MAX_SOURCES)&&self.query.validate().is_ok()
   &&self.selected.iter().all(|s|id(&s.id)&&s.revision>0&&ids.insert(&s.id))
 }
}
impl Selection{
 pub(crate)fn pet_scope(&self)->bool{self.source_scope==Some(crate::pet_sources::PetScope::Pet)}
 pub(crate)fn valid(&self,goal:&str,pet:&str,provider:&str,cid:Option<&str>)->bool{
  let mut ids=std::collections::BTreeSet::new();(match(self.schema_version,self.source_scope,cid){(1,None,Some(cid))=>crate::assistant_memory::valid_id(cid),(2,Some(crate::pet_sources::PetScope::Pet),None)=>true,_=>false})&&self.goal_id==goal&&crate::pet_runtime::valid_goal_id(goal)
   &&self.pet_id==pet&&self.provider==provider&&crate::pet_config::integration_pet(provider)==Some(pet)
   &&self.source_conversation_id.as_deref()==cid&&self.query.validate().is_ok()
   &&self.captured_at>0&&self.captured_at<=8_640_000_000_000_000&&hash(&self.data_hash)&&!(self.references.is_empty()||self.references.len()>MAX_SOURCES)
   &&self.references.iter().all(|r|id(&r.id)&&r.revision>0&&ids.insert(&r.id)&&hash(&r.binding)&&hash(&r.provenance_id)
      &&!r.name.trim().is_empty()&&r.name.len()<=160&&!r.name.chars().any(char::is_control)&&r.file.as_ref().is_none_or(|f|hash(&f.content_hash)))
 }
 pub(crate)fn matches(&self,request:&Request)->bool{self.source_scope==request.source_scope&&self.query==request.query&&self.references.len()==request.selected.len()&&self.references.iter().zip(&request.selected).all(|(r,s)|r.id==s.id&&r.revision==s.revision)}
 fn request(&self)->Request{Request{source_scope:self.source_scope,selected:self.references.iter().map(|r|Pick{id:r.id.clone(),revision:r.revision}).collect(),query:self.query.clone()}}
}
struct Sealed{selected:Selected,file:Option<linux::Source>}
/// Native retained capture, deliberately not Clone/Deserialize/Serialize. Its
/// dependencies are checked again during helper IO and before data publication.
pub(crate)struct Captured{
 selection:Selection,data:Value,sources:Vec<Sealed>,cancelled:AtomicBool,external:Arc<dyn Fn()->bool+Send+Sync>,
}
impl Captured{
 pub(crate)fn native(goal:&crate::pet_runtime::GoalRecord,request:Request,current:Arc<dyn Fn()->bool+Send+Sync>)->Result<Self>{
  goal.validate().map_err(|_|"goalCalendarOwnerChanged")?;
  if goal.state!=crate::pet_runtime::GoalState::Draft||goal.execution_context.is_some(){return Err("goalCalendarOwnerChanged");}
  let cid=Self::origin(goal,request.source_scope)?;
  let sources=Self::sources(&goal.spec.owner.provider,cid)?;
  Self::collect(&goal.id,&goal.spec.owner.pet_id,&goal.spec.owner.provider,cid,request,sources,crate::assistant_memory::now(),current)
 }
 fn origin(goal:&crate::pet_runtime::GoalRecord,scope:Option<crate::pet_sources::PetScope>)->Result<Option<&str>>{
  match(&goal.spec.context,scope){(crate::pet_runtime::GoalContext::Unbound,Some(crate::pet_sources::PetScope::Pet))=>Ok(None),(crate::pet_runtime::GoalContext::Mochi{conversation_id},None)=>Ok(Some(conversation_id)),_=>Err("goalCalendarOwnerChanged")}
 }
 fn sources(provider:&str,cid:Option<&str>)->Result<Vec<Selected>>{
  match cid{Some(cid)=>crate::pet_sources::selected_many(provider,cid),None=>crate::pet_sources::selected_pet(provider)}.map_err(|_|"goalCalendarSourceChanged")
 }
 fn validate_sources(sources:&[&Selected],provider:&str,cid:Option<&str>)->Result<()>{
  match cid{Some(cid)=>crate::pet_sources::validate_group(sources,provider,cid),None=>crate::pet_sources::validate_pet_group(sources,provider)}.map_err(|_|"goalCalendarSourceChanged")
 }
 fn choose(request:&Request,sources:Vec<Selected>)->Result<Vec<Selected>>{
  if !request.valid(){return Err("invalidGoalCalendarSelection");}
  request.selected.iter().map(|pick|sources.iter().find(|s|s.id()==pick.id&&s.revision()==pick.revision).cloned().ok_or("goalCalendarSourceChanged")).collect()
 }
 fn collect(goal:&str,pet:&str,provider:&str,cid:Option<&str>,request:Request,sources:Vec<Selected>,now:u64,external:Arc<dyn Fn()->bool+Send+Sync>)->Result<Self>{
  if !crate::pet_runtime::valid_goal_id(goal)||crate::pet_config::integration_pet(provider)!=Some(pet)||match(cid,request.source_scope){(Some(cid),None)=>!crate::assistant_memory::valid_id(cid),(None,Some(crate::pet_sources::PetScope::Pet))=>false,_=>true}||now==0{return Err("goalCalendarOwnerChanged");}
  let selected=Self::choose(&request,sources)?;Self::validate_sources(&selected.iter().collect::<Vec<_>>(),provider,cid)?;
  let start=Instant::now();let mut references=Vec::new();let mut sealed=Vec::new();
  for selected in selected{
   if start.elapsed()>=CAPTURE_TIME||!external(){return Err("goalCalendarCaptureStopped");}
   let file=linux::read(selected.path()).ok();
   references.push(Self::reference(goal,&selected,file.as_ref()));sealed.push(Sealed{selected,file});
  }
  let selection=Selection{schema_version:if request.source_scope.is_some(){2}else{1},source_scope:request.source_scope,goal_id:goal.into(),pet_id:pet.into(),provider:provider.into(),source_conversation_id:cid.map(str::to_string),query:request.query,captured_at:now,references,data_hash:String::new()};
  let mut captured=Self{selection,data:Value::Null,sources:sealed,cancelled:AtomicBool::new(false),external};captured.current()?;
  let mut snapshots=Vec::new();
  for (source,reference)in captured.sources.iter().zip(&captured.selection.references){
   let provenance=json!({"id":reference.provenance_id,"name":reference.name,"revision":reference.file.as_ref().map(|f|&f.content_hash)});
   let current=||start.elapsed()<CAPTURE_TIME&&captured.current().is_ok();
   let value=if let Some(file)=&source.file{linux::query_source(file,provenance.clone(),captured.selection.query.window(),captured.selection.query.limit,&current).map_err(|_|"goalCalendarCaptureStopped")?}
    else{let mut value=unavailable("sourceUnavailable");value["source"]=provenance;value["window"]=captured.selection.query.window();value};
   if !current(){return Err("goalCalendarCaptureStopped");}snapshots.push(value);
   if serde_json::to_vec(&snapshots).map_err(|_|"invalidGoalCalendarData")?.len()>MAX_DATA{return Err("goalCalendarDataLimit");}
  }
  captured.data=if let Some(cid)=cid{json!({"schemaVersion":1,"kind":"explicitSelectedGoalCalendars","goalId":goal,"petId":pet,"sourceConversationId":cid,"capturedAt":now,"query":captured.selection.query,"snapshots":snapshots,"untrustedData":true,"authorizesActions":false,"liveSynchronization":false})}
   else{json!({"schemaVersion":2,"kind":"explicitSelectedGoalCalendars","goalId":goal,"petId":pet,"sourceScope":"pet","capturedAt":now,"query":captured.selection.query,"snapshots":snapshots,"untrustedData":true,"authorizesActions":false,"liveSynchronization":false})};
  let data=serde_json::to_vec(&captured.data).map_err(|_|"invalidGoalCalendarData")?;if data.len()>MAX_DATA{return Err("goalCalendarDataLimit");}
  captured.selection.data_hash=sha(&data);captured.current()?;
  if !captured.selection.valid(goal,pet,provider,cid){return Err("invalidGoalCalendarSelection");}Ok(captured)
 }
 fn reference(goal:&str,source:&Selected,file:Option<&linux::Source>)->Reference{
  let binding=source.binding();Reference{id:source.id().into(),revision:source.revision(),provenance_id:sha(format!("goal-calendar\0{goal}\0{binding}").as_bytes()),binding,name:source.name().into(),file:file.map(|f|FileReference{stamp:f.identity(),content_hash:f.content_hash()})}
 }
 pub(crate)fn reopen(goal:&crate::pet_runtime::GoalRecord,selection:&Selection,data:Value,current:Arc<dyn Fn()->bool+Send+Sync>)->Result<Self>{
  goal.validate().map_err(|_|"goalCalendarOwnerChanged")?;
  let cid=Self::origin(goal,selection.source_scope)?;
  if !selection.valid(&goal.id,&goal.spec.owner.pet_id,&goal.spec.owner.provider,cid){return Err("goalCalendarOwnerChanged");}
  let selected=Self::sources(&goal.spec.owner.provider,cid)?;
  Self::reopen_selected(selection,data,selected,current)
 }
 fn reopen_selected(selection:&Selection,data:Value,sources:Vec<Selected>,external:Arc<dyn Fn()->bool+Send+Sync>)->Result<Self>{
  let bytes=serde_json::to_vec(&data).map_err(|_|"invalidGoalCalendarData")?;
  if bytes.len()>MAX_DATA||sha(&bytes)!=selection.data_hash{return Err("goalCalendarDataChanged");}
  let mut sealed=Vec::new();
  for(selected,reference)in Self::choose(&selection.request(),sources)?.into_iter().zip(&selection.references){
   // A source unavailable at capture never silently becomes a new event input.
   let file=if reference.file.is_some(){Some(linux::read(selected.path()).map_err(|_|"goalCalendarSourceChanged")?)}else{None};
   if Self::reference(&selection.goal_id,&selected,file.as_ref())!=*reference{return Err("goalCalendarSourceChanged");}sealed.push(Sealed{selected,file});
  }
  let capture=Self{selection:selection.clone(),data,sources:sealed,cancelled:AtomicBool::new(false),external};capture.current()?;Ok(capture)
 }
 pub(crate)fn current(&self)->Result<()>{
  let changed=self.cancelled.load(Ordering::Acquire)||!(self.external)()
   ||Self::validate_sources(&self.sources.iter().map(|s|&s.selected).collect::<Vec<_>>(),&self.selection.provider,self.selection.source_conversation_id.as_deref()).is_err()
   ||self.sources.iter().any(|s|s.file.as_ref().is_some_and(|file|!file.unchanged(s.selected.path())));
  if changed{self.cancelled.store(true,Ordering::Release);return Err("goalCalendarSourceChanged");}Ok(())
 }
 pub(crate)fn selection(&self)->Result<&Selection>{self.current()?;Ok(&self.selection)}
 pub(crate)fn data(&self)->Result<&Value>{self.current()?;Ok(&self.data)}
 pub(crate)fn context(&self)->Result<Context>{self.current()?;Ok(Context{selection:self.selection.clone(),data:self.data.clone()})}
}

#[cfg(test)]mod goal_calendar_capture_tests{
 use super::*;
 use crate::pet_sources::test_support::Enrollment;
 fn goal()->String{format!("g-{}","a".repeat(64))}
 fn request(selected:&[Selected])->Request{Request{source_scope:None,selected:selected.iter().map(|s|Pick{id:s.id().into(),revision:s.revision()}).collect(),query:query::Query{start:"2026-10-23T00:00:00Z".into(),end:"2026-10-24T00:00:00Z".into(),time_zone:"Europe/Lisbon".into(),limit:1}}}
 fn capture(provider:&str,selected:Vec<Selected>)->Captured{
  let pet=crate::pet_config::integration_pet(provider).unwrap();Captured::collect(&goal(),pet,provider,Some("goal-source-cid"),request(&selected),selected,100_000,Arc::new(||true)).unwrap_or_else(|e|panic!("{provider}: {e}"))
 }
 #[test]fn native_selected_calendars_use_common_worker_exact_ids_query_and_owner_across_routes(){
  let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();
  for provider in["ollama","lmstudio","codex","copilot","antigravity"]{
   let f=Enrollment::new(provider,"goal-source-cid");let selected=f.collection();let captured=capture(provider,selected.clone());
   let selection=captured.selection().unwrap();assert!(selection.valid(&goal(),crate::pet_config::integration_pet(provider).unwrap(),provider,Some("goal-source-cid")));assert!(!selection.valid(&goal(),"agent_codex","codex",Some("wrong-cid")));assert!(selection.matches(&request(&selected)));
   assert!(captured.context().unwrap().valid(&goal(),crate::pet_config::integration_pet(provider).unwrap(),provider,Some("goal-source-cid")));
   let data=captured.data().unwrap();assert_eq!(data["authorizesActions"],false);assert_eq!(data["untrustedData"],true);assert_eq!(data["liveSynchronization"],false);let snapshots=data["snapshots"].as_array().unwrap();assert_eq!(snapshots.len(),2);
   for snapshot in snapshots{assert_eq!(snapshot["status"],"ready");assert_eq!(snapshot["events"].as_array().unwrap().len(),1);assert_eq!(snapshot["window"]["timeZone"],"Europe/Lisbon");}
   assert_ne!(snapshots[0]["source"]["id"],snapshots[1]["source"]["id"]);assert!(data.to_string().contains("Enrolled fixture 73")&&data.to_string().contains("Other fixture 84"));
   let reopened=Captured::reopen_selected(selection,data.clone(),selected,Arc::new(||true)).unwrap();assert_eq!(reopened.data().unwrap(),data);assert_eq!(reopened.selection().unwrap(),selection);
  }
 }
 #[test]fn captured_data_tampering_and_replacement_sources_refuse_without_refresh(){
  let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();let f=Enrollment::new("ollama","goal-source-cid");let selected=f.collection();let captured=capture("ollama",selected.clone());let selection=captured.selection().unwrap().clone();let data=captured.data().unwrap().clone();
  let mut changed=data.clone();changed["snapshots"][0]["events"]=json!([]);assert_eq!(Captured::reopen_selected(&selection,changed,selected.clone(),Arc::new(||true)).err(),Some("goalCalendarDataChanged"));
  std::fs::write(&f.source,b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n").unwrap();assert!(captured.current().is_err());assert!(Captured::reopen_selected(&selection,data,selected,Arc::new(||true)).is_err());
 }
 #[test]fn registry_revocation_and_stop_stay_revoked_and_references_are_not_execution_consent(){
  let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();let f=Enrollment::new("ollama","goal-source-cid");let selected=f.collection();let captured=capture("ollama",selected);let selection=captured.selection().unwrap().clone();let before=std::fs::read(f.root.join("sources.json")).unwrap();f.disable();assert!(captured.data().is_err());
  std::fs::write(f.root.join("sources.json"),before).unwrap();assert!(captured.data().is_err());assert!(serde_json::to_value(&selection).unwrap().get("consent").is_none());
  let f=Enrollment::new("lmstudio","goal-source-cid");let selected=f.collection();let stop=Arc::new(AtomicBool::new(false));let state=stop.clone();let captured=Captured::collect(&goal(),"ai_lmstudio","lmstudio",Some("goal-source-cid"),request(&selected),selected,100_000,Arc::new(move||!state.load(Ordering::Acquire))).unwrap();stop.store(true,Ordering::Release);assert!(captured.data().is_err());stop.store(false,Ordering::Release);assert!(captured.data().is_err());
 }
 #[test]fn malformed_duplicate_missing_and_wrong_scope_selection_refuse_before_read(){
  let f=Enrollment::new("ollama","goal-source-cid");let selected=f.collection();let base=request(&selected);
  for mode in["duplicate","missing","revision","window","limit","count"]{
   let mut request=base.clone();match mode{"duplicate"=>request.selected[1]=request.selected[0].clone(),"missing"=>request.selected[0].id="unselected".into(),"revision"=>request.selected[0].revision+=1,"window"=>request.query.start="2026-10-23".into(),"limit"=>request.query.limit=129,_=>request.selected.resize(9,request.selected[0].clone())};
   assert!(Captured::collect(&goal(),"ai_ollama","ollama",Some("goal-source-cid"),request,selected.clone(),100_000,Arc::new(||true)).is_err(),"{mode}");
  }
  assert!(Captured::collect(&goal(),"ai_ollama","ollama",Some("another-cid"),base.clone(),selected.clone(),100_000,Arc::new(||true)).is_err());assert!(Captured::collect(&goal(),"agent_codex","codex",Some("goal-source-cid"),base,selected,100_000,Arc::new(||true)).is_err());
 }
 #[test]fn pet_source_scope_capture_uses_same_parser_without_conversation_or_implicit_grants(){
  let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();
  for provider in ["ollama","lmstudio","codex","copilot","antigravity"]{
   let f=Enrollment::new(provider,"unused-origin");let selected=f.pet_collection();let pet=crate::pet_config::integration_pet(provider).unwrap();let mut pick=request(&selected);pick.source_scope=Some(crate::pet_sources::PetScope::Pet);
   assert!(Captured::collect(&goal(),pet,provider,Some("unused-origin"),pick.clone(),selected.clone(),100_000,Arc::new(||true)).is_err());
   let c=Captured::collect(&goal(),pet,provider,None,pick,selected.clone(),100_000,Arc::new(||true)).unwrap();let context=c.context().unwrap();assert!(context.valid(&goal(),pet,provider,None));assert!(!context.valid(&goal(),pet,provider,Some("unused-origin")));assert_eq!(context.selection.schema_version,2);assert!(context.data.get("sourceConversationId").is_none());assert_eq!(context.data["sourceScope"],"pet");assert!(context.data["snapshots"].as_array().unwrap().iter().all(|s|s["status"]=="ready"));
   assert_eq!(Captured::reopen_selected(&context.selection,context.data.clone(),selected.clone(),Arc::new(||true)).unwrap().data().unwrap(),&context.data);
   assert!(Captured::collect(&goal(),pet,provider,None,{let mut r=request(&selected);r.source_scope=None;r},selected.clone(),100_000,Arc::new(||true)).is_err());
   let other=if provider=="codex"{"ollama"}else{"codex"};assert!(crate::pet_sources::validate_pet_group(&selected.iter().collect::<Vec<_>>(),other).is_err());
   f.documents(false);assert!(c.current().is_err());f.documents(true);assert!(c.current().is_err());
  }
 }
 #[test]fn pet_source_scope_requires_context11_goal65_and_own_goal_data_grant10(){
  let _serial=crate::pet_calendar::TEST_SERIAL.lock().unwrap();let f=Enrollment::new("ollama","unused-origin");let selected=f.pet_collection();
  let spec=json!({"requestId":"own-pet-goal","owner":{"petId":"ai_ollama","provider":"ollama","adapter":"ollama","modelHint":null},"context":{"kind":"unbound"},"intent":"Review selected calendars","criteria":["Verify coverage"]});
  let mut g=crate::pet_runtime::GoalRecord::new(serde_json::from_value(spec).unwrap(),100_000,"a".repeat(32)).unwrap();let mut request=request(&selected);request.source_scope=Some(crate::pet_sources::PetScope::Pet);
  let c=Captured::collect(&g.id,"ai_ollama","ollama",None,request,selected,100_000,Arc::new(||true)).unwrap();let context=c.context().unwrap();
  g.execution_context=Some(serde_json::from_value(json!({"schemaVersion":11,"goalId":g.id,"petId":"ai_ollama","provider":"ollama","conversationId":format!("goal-{}",g.id.trim_start_matches("g-")),"requestId":"prepare-own","mode":"dedicated","sourceConversationId":null,"sourceMessages":0,"copiedMessages":0,"seedHash":sha(b"[]"),"preparedAt":100_000,"calendars":context})).unwrap());g.schema_version=65;assert!(g.validate().is_ok());
  let bytes=serde_json::to_vec(&g).unwrap();assert!(serde_json::from_slice::<crate::pet_runtime::GoalRecord>(&bytes).unwrap().validate().is_ok());g.schema_version=64;assert!(g.validate().is_err());g.schema_version=65;g.execution_context.as_mut().unwrap().schema_version=10;assert!(g.validate().is_err());
  let mut grant:crate::pet_memory::context_model::GoalGrant=serde_json::from_value(json!({"schemaVersion":10,"consent":"boundedGoalContext","destination":{"kind":"server","provider":"ollama","endpoint":"http://127.0.0.1:19000/v1/chat/completions","model":"fixture-free"},"maxInputBytes":crate::pet_memory::context_model::GOAL_INPUT_LIMIT,"calendarSelection":context.selection})).unwrap();assert!(grant.valid("ollama","fixture-free"));grant.schema_version=9;assert!(!grant.valid("ollama","fixture-free"));
 }
}
