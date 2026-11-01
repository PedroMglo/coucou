//! Human observation of one original shortcut effect. Same Controller, native
//! actor, source journal and physical gate; no registration and no replay.
use super::*;
use crate::pet_runtime::native_process_identity::{self as process,ProcessIdentity,ProcessPresence};
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Observation{schema_version:u32,current_instance:String,current_process:ProcessIdentity,current_environment:Environment,current_transport:String,original_instance:Option<String>,original_process:Option<ProcessPresence>,same_instance:bool,can_reconcile:bool,reason:String,portal:Option<crate::portal::observation::Observation>,plugin_cleanup_acknowledged:Option<bool>}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(super)struct NativeSubject{instance:String,owner:Option<ProcessIdentity>,environment:Environment,kind:String,portal:Option<crate::portal::observation::Subject>}
fn instance(d:&Destination)->&str{match d{Destination::Plugin{instance,..}|Destination::Portal{instance,..}=>instance}}
fn kind(d:&Destination)->&'static str{match d{Destination::Plugin{..}=>"plugin",Destination::Portal{..}=>"portal"}}
fn environment(d:&Destination)->&Environment{match d{Destination::Plugin{environment,..}|Destination::Portal{environment,..}=>environment}}
fn owner(d:&Destination)->&ProcessIdentity{match d{Destination::Plugin{owner,..}|Destination::Portal{owner,..}=>owner}}
pub(super)fn original(slot:&components::Slot,intent:&components::Artifact)->Result<NativeSubject,String>{
 let review=&intent.evidence["review"];if review["token"].as_str()!=Some(&slot.id)||review["fingerprint"].as_str()!=Some(&slot.binding.plan){return Err("A intenção nativa original não corresponde ao efeito.".into());}
 let d=&review["destination"];let instance=d["instance"].as_str().filter(|s|s.len()==32).ok_or_else(refused)?.into();let environment:Environment=serde_json::from_value(d["environment"].clone()).map_err(|_|refused())?;
 let kind=d["kind"].as_str().filter(|s|matches!(*s,"plugin"|"portal")).ok_or_else(refused)?.into();
 let owner=if d.get("owner").is_none(){None}else{Some(serde_json::from_value(d["owner"].clone()).map_err(|_|refused())?)};
 let portal=if kind=="portal"{let p=&d["reservation"];if let Some(bus)=p["busId"].as_str(){let mut sessions=vec![];if p["shortcuts"].as_array().is_some_and(|s|!s.is_empty()){sessions.push(p["session"].as_str().ok_or_else(refused)?.to_owned());}if let Some(path)=p["previous"]["session"].as_str(){if !sessions.iter().any(|s|s==path){sessions.push(path.into());}}Some(crate::portal::observation::Subject{bus_id:bus.into(),peer:p["peer"].as_str().ok_or_else(refused)?.into(),client:p["client"].as_str().ok_or_else(refused)?.into(),sessions})}else{None}}else{None};
 Ok(NativeSubject{instance,owner,environment,kind,portal})
}
fn current_subject(destination:&Destination)->NativeSubject{
 let portal=match destination{Destination::Portal{reservation,..}=>Some(crate::portal::observation::Subject{bus_id:reservation.bus_id.clone(),peer:reservation.peer.clone(),client:reservation.client.clone(),sessions:if reservation.previous.phase=="bound"{reservation.previous.session.iter().cloned().collect()}else{vec![]}}),Destination::Plugin{..}=>None};
 NativeSubject{instance:instance(destination).into(),owner:Some(owner(destination).clone()),environment:environment(destination).clone(),kind:kind(destination).into(),portal}
}
fn local_plugin(e:&Environment)->bool{e.display.as_ref().is_some_and(|d|d.strip_prefix(':').is_some_and(|s|!s.is_empty()&&s.bytes().all(|b|b.is_ascii_digit()||b==b'.')))}
pub(super)fn registration_continuity(settings:&Settings,path:&Path,origin:&Context,native:&Destination)->Result<(),String>{
 let Some(slot)=&origin.slot else{return Ok(())};if slot.phase==components::Phase::NotDispatched{return Ok(())}
 let Some((_,intent,outcome))=components::current_artifacts(settings,path,"shortcuts").map_err(|e|e.to_string())?else{return Err(refused())};
 if slot.phase==components::Phase::ReconciledObservation&&outcome.as_ref().is_some_and(|o|o.evidence.get("reconciliationProtocol")==Some(&serde_json::json!(1))&&o.evidence.pointer("/nativeObservation/currentInstance").and_then(serde_json::Value::as_str)==Some(instance(native))&&o.evidence.pointer("/nativeObservation/currentProcess")==Some(&serde_json::to_value(owner(native)).unwrap())&&o.evidence.pointer("/nativeObservation/currentEnvironment")==Some(&serde_json::to_value(environment(native)).unwrap())&&o.evidence.pointer("/nativeObservation/currentTransport").and_then(serde_json::Value::as_str)==Some(kind(native))){return Ok(())}
 let old=original(slot,&intent)?;
 if slot.phase==components::Phase::Verified&&old.instance==instance(native)&&old.owner.as_ref()==Some(owner(native))&&&old.environment==environment(native)&&old.kind==kind(native){return Ok(())}
 Err("O dono nativo anterior exige observação e reconciliação próprias antes de outra tentativa. Não foi iniciado um novo registo.".into())
}
/// A successful explicit Stop of a fully committed registration can retain
/// the exact native cleanup fact. Never infer coverage for an uncertain slot.
pub(in crate::shortcuts)fn closed_candidate<R:Runtime>(app:&AppHandle<R>)->Option<Closed>{
 let registry=app.try_state::<Registry>()?;let shared=app.try_state::<crate::Shared>()?;let s=shared.settings.try_lock().ok()?;
 let(slot,intent,outcome)=components::current_artifacts(&s,&path(),"shortcuts").ok()??;
 if slot.phase!=components::Phase::Verified||outcome.as_ref()?.evidence.get("publication")!=Some(&serde_json::json!("published")){return None;}
 let original=original(&slot,&intent).ok()?;
 if original.kind!="plugin"||registry.instance.get()!=Some(&original.instance)||original.owner.as_ref()!=Some(&process::capture().ok()?)||original.environment!=Environment::capture().ok()?{return None;}
 Some(Closed{effect:slot.id,plan:slot.binding.plan,instance:original.instance,complete:false})
}
fn observe(app:&AppHandle,slot:&components::Slot,intent:&components::Artifact,native:&NativeCapture)->Result<Observation,String>{
 let old=original(slot,intent)?;let same_instance=old.instance==instance(&native.destination);let original_process=old.owner.as_ref().map(process::observe);
 let mut out=Observation{schema_version:1,current_instance:instance(&native.destination).into(),current_process:owner(&native.destination).clone(),current_environment:environment(&native.destination).clone(),current_transport:kind(&native.destination).into(),original_instance:Some(old.instance.clone()),original_process,same_instance,can_reconcile:false,reason:String::new(),portal:None,plugin_cleanup_acknowledged:None};
 if old.owner.is_none(){out.reason="Origem legada sem encarnação do processo; não foi inferida a morte do dono.".into();return Ok(out);}
 if matches!(original_process,Some(ProcessPresence::Unavailable|ProcessPresence::IdentityMismatch)){out.reason="A encarnação original não pode ser comprovada; PID, arranque ou namespace diferem ou estão indisponíveis.".into();return Ok(out);}
 if old.environment!=*environment(&native.destination){out.reason="O destino gráfico atual difere do original; não foi escolhido outro recurso.".into();return Ok(out);}
 if slot.phase==components::Phase::Prepared&&(original_process==Some(ProcessPresence::OriginalAbsent)||same_instance&&old.owner.as_ref()==Some(owner(&native.destination))){out.can_reconcile=true;out.reason="Intenção durável sem despacho; o worker original já não detém os gates.".into();return Ok(out);}
 match(&native.destination,old.kind.as_str()){
  (Destination::Portal{..},"portal")=>{
   let Some(subject)=old.portal else{out.reason="Plano legado sem identidade do barramento; nomes D-Bus não são uma prova suficiente.".into();return Ok(out)};
   let registry=app.state::<Registry>();let reply={let link=registry.wayland.lock().map_err(|_|refused())?;let super::super::wayland::Link::Running{handle,..}=&*link else{return Err("O observador do portal está indisponível.".into())};handle.observe_subject(subject).ok_or_else(refused)?};
   let value=reply.recv_timeout(Duration::from_secs(30)).map_err(|_|"A observação nativa não terminou; não foi repetido qualquer efeito.".to_string())?;out.can_reconcile=value.bus_matches&&value.sessions_closed;out.reason=value.reason.into();out.portal=Some(value);
  },
  (Destination::Plugin{..},"plugin")=>{
   let registry=app.state::<Registry>();let closed=registry.closed.lock().map_err(|_|refused())?.as_ref().is_some_and(|c|c.effect==slot.id&&c.plan==slot.binding.plan&&c.instance==old.instance&&c.complete);
   out.plugin_cleanup_acknowledged=Some(closed);out.can_reconcile=same_instance&&old.owner.as_ref()==Some(owner(&native.destination))&&closed||original_process==Some(ProcessPresence::OriginalAbsent)&&local_plugin(&old.environment);
   out.reason=if out.can_reconcile{if closed{"Todas as teclas próprias potencialmente afetadas têm ACK de limpeza nesta instância."}else{"Processo original ausente na mesma encarnação do sistema; transporte X11 local sem worker original."}}else{"O dono original continua vivo sem prova completa de limpeza. Não foi usado o registo local do plugin como prova do estado do OS; fecha o processo original antes de reconciliar."}.into();
  },_=>out.reason="O transporte nativo atual difere do original.".into()
 }
 Ok(out)
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Preview{schema_version:u32,token:String,fingerprint:String,expires_at_ms:u64,scope:&'static str,origin:Context,subject:components::Slot,original:NativeSubject,current:NativeSubject,observation:Observation,no_registration_effect:bool,no_reactivation:bool,past_effect_outcome_known:bool,message:&'static str}
pub(super)struct Capture{preview:Preview,frame:Frame,expires:Instant,alive:Arc<AtomicBool>,native:NativeCapture,path:PathBuf,intent:components::Artifact}
impl Capture{fn authorized(&self)->bool{self.frame.current()&&self.alive.load(Ordering::Acquire)&&Instant::now()<self.expires}}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Decision{token:String,fingerprint:String,preview_schema_version:u32,reviewed_observation:bool,understand_past_uncertainty:bool,keep_registration_suspended:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{schema_version:u32,effect_id:String,observation_recorded:bool,no_registration_effect:bool,no_reactivation:bool,past_effect_outcome_known:bool,status:settings::IoStatus,message:&'static str}
impl Controller{
 pub(super)fn cancel_reconciliation(&self,frame:&Frame,key:&Key)->Result<bool,String>{let mut pending=self.reconciliation.lock().map_err(|_|refused())?;let Some(c)=pending.as_ref()else{return Ok(false)};if c.frame.same(frame)&&key.token==c.preview.token&&key.fingerprint==c.preview.fingerprint{c.alive.store(false,Ordering::Release);*pending=None;return Ok(true)}Ok(false)}
 fn prepare_reconciliation(&self,frame:&Frame,current:&Mutex<Settings>,path:&Path,context:ContextReader,native:impl FnOnce(&Bindings)->Result<NativeCapture,String>,observer:impl Fn(&components::Slot,&components::Artifact,&NativeCapture)->Result<Observation,String>)->Result<Preview,String>{
  let _gate=match self.gate.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O worker original ainda está em curso.".into())};
  if !frame.current()||self.running.lock().map_err(|_|refused())?.is_some(){return Err(refused());}
  let(origin,stored,slot,intent)={let s=current.try_lock().map_err(|_|refused())?;settings::io_status(&s).check()?;let origin=context(&s,path).map_err(|e|e.to_string())?;let(slot,intent,_)=components::current_artifacts(&s,path,"shortcuts").map_err(|e|e.to_string())?.ok_or("Não existe um efeito nativo para reconciliar.")?;(origin,s.shortcuts.clone(),slot,intent)};
  if origin.profile.is_none(){return Err(refused());}let original=original(&slot,&intent)?;let native=native(&stored)?;let observation=observer(&slot,&intent,&native)?;
  let mut preview=Preview{schema_version:2,token:random()?,fingerprint:String::new(),expires_at_ms:u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|refused())?.as_millis()+TTL.as_millis()).map_err(|_|refused())?,scope:"nativeUserShortcutsReconciliationOnly",origin,subject:slot,original,current:current_subject(&native.destination),observation,no_registration_effect:true,no_reactivation:true,past_effect_outcome_known:false,message:"Compara a identidade e o destino nativos originais com a observação atual. Dados originais em falta continuam desconhecidos. As sessões atuais são apenas as conhecidas pelo executor; a lista não prova libertação. Registar esta observação conserva o resultado histórico, não regista teclas nem renova aprovações. Outra tentativa exige uma aprovação distinta."};preview.fingerprint=hash(&encoded(&preview)?);
  let capture=Capture{preview:preview.clone(),frame:frame.clone(),expires:Instant::now()+TTL,alive:Arc::new(AtomicBool::new(true)),native,path:path.into(),intent};
  {let s=current.try_lock().map_err(|_|refused())?;if !capture.authorized()||!(capture.native.current)()||context(&s,path).map_err(|e|e.to_string())?!=preview.origin{return Err(refused());}}
  *self.pending.lock().map_err(|_|refused())?=None;*self.reconciliation.lock().map_err(|_|refused())?=Some(capture);Ok(preview)
 }
 fn inspect_reconciliation(&self,frame:&Frame,current:&Mutex<Settings>,key:&Key,context:ContextReader,observer:impl Fn(&components::Slot,&components::Artifact,&NativeCapture)->Result<Observation,String>)->Result<Preview,String>{
  let pending=self.reconciliation.lock().map_err(|_|refused())?;let c=pending.as_ref().ok_or_else(refused)?;
  if !c.frame.same(frame)||key.token!=c.preview.token||key.fingerprint!=c.preview.fingerprint||!c.authorized()||!(c.native.current)(){return Err(refused());}
  if observer(&c.preview.subject,&c.intent,&c.native)?!=c.preview.observation{return Err(refused());}
  let s=current.try_lock().map_err(|_|refused())?;if context(&s,&c.path).map_err(|e|e.to_string())?!=c.preview.origin{return Err(refused());}Ok(c.preview.clone())
 }
 fn apply_reconciliation(&self,frame:&Frame,current:&Mutex<Settings>,decision:Decision,context:ContextReader,admit:Guard,observer:impl Fn(&components::Slot,&components::Artifact,&NativeCapture)->Result<Observation,String>)->Result<Receipt,String>{
  if decision.preview_schema_version!=2{return Err("É necessária a revisão 2 com comparação entre a identidade nativa original e a atual.".into());}
  if !decision.reviewed_observation||!decision.understand_past_uncertainty||!decision.keep_registration_suspended{return Err("A reconciliação exige as três confirmações da observação, incerteza passada e suspensão.".into());}
  let _gate=match self.gate.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O worker original ainda está em curso.".into())};
  let key=Key{token:decision.token,fingerprint:decision.fingerprint};self.inspect_reconciliation(frame,current,&key,context,&observer)?;
  let c=self.reconciliation.lock().map_err(|_|refused())?.take().ok_or_else(refused)?;if !c.preview.observation.can_reconcile{return Err("A observação não prova as condições para reconciliar. O efeito continua incerto.".into());}
  *self.running.lock().map_err(|_|refused())?=Some(Running{key:Key{token:c.preview.token.clone(),fingerprint:c.preview.fingerprint.clone()},frame:c.frame.clone(),alive:c.alive.clone()});let _flight=Flight{controller:self,token:c.preview.token.clone()};
  let guard=||if c.authorized()&&admit()&&(c.native.current)(){Ok(())}else{Err("shortcutObservationRevoked")};
  if observer(&c.preview.subject,&c.intent,&c.native)?!=c.preview.observation{return Err(refused());}
  let s=current.lock().map_err(|_|refused())?;if context(&s,&c.path).map_err(|e|e.to_string())?!=c.preview.origin{return Err(refused());}
  let old=&c.preview.subject;let artifact=components::Artifact{version:1,generation:1,id:random()?,effect:old.id.clone(),component:"shortcuts".into(),binding:old.binding.clone(),phase:components::Phase::ReconciledObservation,evidence:serde_json::json!({"reconciliationProtocol":1,"review":c.preview,"subject":old,"nativeObservation":c.preview.observation,"noRegistrationEffect":true,"noReactivation":true,"pastEffectOutcomeKnown":false,"reviewedObservation":true,"understandPastUncertainty":true,"keepRegistrationSuspended":true}),previous:None};
  let reference=components::retain(&c.path,&artifact,guard).map_err(|e|e.to_string())?;components::transition(&s,&c.path,"shortcuts",&old.id,components::Phase::ReconciledObservation,Some(reference),guard).map_err(|e|e.to_string())?;self.gate.clear_poison();
  Ok(Receipt{schema_version:1,effect_id:old.id.clone(),observation_recorded:true,no_registration_effect:true,no_reactivation:true,past_effect_outcome_known:false,status:settings::io_status(&s),message:"Observação e reconciliação registadas, sem alterar teclas ou resultado passado. O registo continua suspenso; uma nova tentativa exige outro plano e aprovação."})
 }
}
#[tauri::command]
pub(crate)async fn shortcuts_reconcile_prepare(app:AppHandle,window:tauri::WebviewWindow)->Result<Preview,String>{
 if !permitted(&window){return Err(refused());}let frame=settings::review::controller().frame()?;let path=path();
 tauri::async_runtime::spawn_blocking(move||{let registry=app.state::<Registry>();let _physical=match registry.physical.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O recurso nativo ainda está ocupado.".into())};let shared=app.state::<crate::Shared>();controller().prepare_reconciliation(&frame,&shared.settings,&path,native_context,|s|{if crate::platform::global_shortcuts_blocked()==Some("wayland"){super::super::wayland::refresh_observer(&app);}NativeCapture::capture(&app,s)},|slot,intent,native|observe(&app,slot,intent,native))}).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)async fn shortcuts_reconcile_inspect(app:AppHandle,window:tauri::WebviewWindow,request:Key)->Result<Preview,String>{
 if !permitted(&window){return Err(refused());}let frame=settings::review::controller().frame()?;
 tauri::async_runtime::spawn_blocking(move||{let shared=app.state::<crate::Shared>();controller().inspect_reconciliation(&frame,&shared.settings,&request,native_context,|slot,intent,native|observe(&app,slot,intent,native))}).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)async fn shortcuts_reconcile_apply(app:AppHandle,window:tauri::WebviewWindow,request:Decision)->Result<Receipt,String>{
 if !permitted(&window){return Err(refused());}let frame=settings::review::controller().frame()?;
 tauri::async_runtime::spawn_blocking(move||{let registry=app.state::<Registry>();let _physical=match registry.physical.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O recurso nativo ainda está ocupado.".into())};let shared=app.state::<crate::Shared>();let receipt=controller().apply_reconciliation(&frame,&shared.settings,request,native_context,Arc::new(||crate::pet_data::bootstrap::write_allowed().is_ok()),|slot,intent,native|observe(&app,slot,intent,native))?;registry.physical.clear_poison();let _=app.emit("settings-review-state-changed",&receipt.status);Ok(receipt)}).await.map_err(|_|"O worker de reconciliação terminou sem resposta confirmada. Consulta o estado; não registes teclas automaticamente.".to_string())?
}
#[cfg(test)]#[path="shortcuts_reconciliation_tests.rs"]mod tests;
