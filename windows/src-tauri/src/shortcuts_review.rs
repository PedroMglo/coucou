//! Human-review capture over the existing native shortcuts transport. Preparing
//! this plan never registers/releases keys and is not execution authority.
use super::*;
use crate::settings::{self, Settings, review::Frame, durable::components::{self, Context}};
use serde::{Deserialize, Serialize};
use std::{path::{Path,PathBuf}, sync::{OnceLock,Arc,atomic::{AtomicBool,Ordering}}, time::{Duration,Instant,SystemTime,UNIX_EPOCH}};

const TTL:Duration=Duration::from_secs(300);
fn refused()->String{"A revisão dos atalhos mudou, expirou ou foi cancelada. Consulta o estado antes de preparar outra tentativa; não repitas um registo automaticamente.".into()}
fn hash(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|n|format!("{n:02x}")).collect()}
fn random()->Result<String,String>{use ring::rand::SecureRandom;let mut bytes=[0;16];ring::rand::SystemRandom::new().fill(&mut bytes).map_err(|_|refused())?;Ok(bytes.iter().map(|n|format!("{n:02x}")).collect())}
fn encoded<T:Serialize>(value:&T)->Result<Vec<u8>,String>{let bytes=serde_json::to_vec(value).map_err(|_|refused())?;if bytes.len()>96*1024{return Err(refused());}Ok(bytes)}
type ContextReader=fn(&Settings,&Path)->std::io::Result<Context>;
fn native_context(s:&Settings,p:&Path)->std::io::Result<Context>{components::context(s,p,"shortcuts")}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Environment{display:Option<String>,wayland_display:Option<String>,session_type:Option<String>}
impl Environment{
    fn capture()->Result<Self,String>{
        fn value(name:&str)->Result<Option<String>,String>{match std::env::var(name){Ok(s)if s.len()<=256&&!s.chars().any(char::is_control)=>Ok(Some(s)),Err(std::env::VarError::NotPresent)=>Ok(None),_=>Err("A identidade da sessão gráfica não pode ser revista.".into())}}
        Ok(Self{display:value("DISPLAY")?,wayland_display:value("WAYLAND_DISPLAY")?,session_type:value("XDG_SESSION_TYPE")?})
    }
}
#[derive(Clone,Serialize)]
#[serde(tag="kind",rename_all="camelCase")]
enum Destination{Plugin{instance:String,environment:Environment,owner:crate::pet_runtime::native_process_identity::ProcessIdentity,previous:Vec<Held>},Portal{instance:String,reservation:crate::portal::Reservation,environment:Environment,owner:crate::pet_runtime::native_process_identity::ProcessIdentity}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]#[serde(rename_all="camelCase")]
struct Held{id:u32,action:String,keys:String,published:bool}
#[derive(Clone)]pub(super)struct Closed{plan:String,instance:String,effect:String,pub(super)complete:bool}
fn held(registry:&Registry)->Vec<Held>{let mut rows:Vec<_>=registry.by_id.lock().unwrap().iter().map(|(id,r)|Held{id:*id,action:r.action.into(),keys:r.shortcut.to_string(),published:r.published}).collect();rows.sort_by_key(|r|r.id);rows}
impl Destination{
    fn resource(&self)->Result<String,String>{
        // Identity of the same registrar; changing desired keys or allocating
        // another session must not manufacture an independent physical resource.
        let identity=match self{
            Self::Plugin{instance,environment,..}=>serde_json::json!({"kind":"plugin","instance":instance,"environment":environment}),
            Self::Portal{instance,reservation,environment,..}=>serde_json::json!({"kind":"portal","instance":instance,"peer":reservation.peer,"client":reservation.client,"busId":reservation.bus_id,"environment":environment}),
        };
        Ok(hash(&encoded(&identity)?))
    }
}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct ActionView{id:String,description:String,keys:String,enabled:bool,ported:bool,planned_status:Status,typed:Option<String>,xdg_trigger:Option<String>}
fn actions(stored:&Bindings)->Result<Vec<ActionView>,String>{
    if stored.len()>64||stored.iter().any(|(id,v)|id.len()>128||v.keys.len()>256){return Err("Os atalhos excedem os limites da revisão; os dados foram conservados.".into());}
    Ok(plan(stored,typed_character).into_iter().map(|(def,outcome)|{let binding=effective(def,stored);let(status,typed)=outcome.err().map(|v|(v.status,v.typed)).unwrap_or((Status::Pending,None));ActionView{id:def.id.into(),description:crate::i18n::t(description(def.id)),keys:binding.keys.clone(),enabled:binding.enabled,ported:def.ported,planned_status:status,typed,xdg_trigger:crate::portal::xdg_trigger(&binding.keys)}}).collect())
}
struct NativeCapture{destination:Destination,current:Box<dyn Fn()->bool+Send>}
impl NativeCapture{
    fn capture(app:&AppHandle,stored:&Bindings)->Result<Self,String>{
        let registry=app.try_state::<Registry>().ok_or("shortcuts registry unavailable")?;
        let instance=if let Some(s)=registry.instance.get(){s.clone()}else{let new=random()?;let _=registry.instance.set(new);registry.instance.get().ok_or_else(refused)?.clone()};
        let revision=registry.revision.load(Ordering::Acquire);if revision==u64::MAX{return Err(refused());}
        let environment=Environment::capture()?;let owner=crate::pet_runtime::native_process_identity::capture().map_err(str::to_owned)?;
        let destination=match crate::platform::global_shortcuts_blocked(){
            Some("wayland")=>{let reservation=super::wayland::reserve(app,stored)?;if !matches!(reservation.previous.phase,"idle"|"bound"){return Err("O registo nativo anterior ainda está pendente; termina ou suspende esse pedido antes de preparar outra revisão.".into());}Destination::Portal{instance:instance.clone(),reservation,environment:environment.clone(),owner:owner.clone()}},
            None if app.try_state::<GlobalShortcut<tauri::Wry>>().is_some()=>Destination::Plugin{instance:instance.clone(),environment:environment.clone(),owner,previous:held(&registry)},
            _=>return Err("O executor nativo dos atalhos está indisponível; não foi escolhido outro destino.".into()),
        };
        let portal_gen=match &destination{Destination::Portal{reservation,..}=>Some(reservation.generation.checked_sub(1).ok_or_else(refused)?),_=>None};
        let app=app.clone();
        let current=Box::new(move||{
            let Some(registry)=app.try_state::<Registry>()else{return false};
            if registry.instance.get()!=Some(&instance)||registry.revision.load(Ordering::Acquire)!=revision||Environment::capture().ok().as_ref()!=Some(&environment){return false;}
            if let Some(expected)=portal_gen{let Ok(link)=registry.wayland.try_lock()else{return false};if !matches!(&*link,super::wayland::Link::Running{gen,..}if *gen==expected){return false;}}
            true
        });
        let capture=Self{destination,current};if !(capture.current)(){return Err(refused());}Ok(capture)
    }
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct Preview{
    schema_version:u32,pub token:String,pub fingerprint:String,expires_at_ms:u64,scope:&'static str,
    origin:Context,configuration:String,resource:String,destination:Destination,desired_bindings:Bindings,
    actions:Vec<ActionView>,unknown_action_ids:Vec<String>,activation_allowed:bool,approval_available:bool,message:&'static str,
    #[serde(skip_serializing_if="Option::is_none")]recovery:Option<scope_recovery::Review>,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Key{token:String,fingerprint:String}
struct Capture{preview:Preview,frame:Frame,expires:Instant,native:NativeCapture,path:PathBuf,alive:Arc<AtomicBool>}
impl Capture{fn authorized(&self)->bool{self.frame.current()&&self.alive.load(Ordering::Acquire)&&Instant::now()<self.expires}fn binding(&self)->components::Binding{components::Binding{profile:self.preview.origin.profile.clone().unwrap_or_default(),configuration:self.preview.configuration.clone(),resource:self.preview.resource.clone(),plan:self.preview.fingerprint.clone()}}}
struct Running{key:Key,frame:Frame,alive:Arc<AtomicBool>}
#[derive(Default)]
pub(crate)struct Controller{pending:Mutex<Option<Capture>>,running:Mutex<Option<Running>>,reconciliation:Mutex<Option<reconciliation::Capture>>,gate:Mutex<()>}
fn controller()->&'static Controller{static C:OnceLock<Controller>=OnceLock::new();C.get_or_init(Controller::default)}
impl Controller{
    fn prepare_with(&self,frame:&Frame,current:&Mutex<Settings>,path:&Path,context:ContextReader,native:impl FnOnce(&Bindings)->Result<NativeCapture,String>)->Result<Preview,String>{
        self.prepare_mode_with(frame,current,path,context,false,native)
    }
    fn prepare_mode_with(&self,frame:&Frame,current:&Mutex<Settings>,path:&Path,context:ContextReader,current_scope:bool,native:impl FnOnce(&Bindings)->Result<NativeCapture,String>)->Result<Preview,String>{
        let _gate=match self.gate.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))if current_scope=>e.into_inner(),Err(_)=>return Err("O worker anterior ainda está em curso; cancelar não prova que terminou.".into())};
        if !frame.current(){return Err(refused());}
        let (stored,origin)={let s=current.try_lock().map_err(|_|"As preferências estão ocupadas.".to_string())?;settings::io_status(&s).check()?;(s.shortcuts.clone(),context(&s,path).map_err(|e|e.to_string())?)};
        if origin.profile.is_none(){return Err("A revisão exige o perfil nativo admitido; os dados não foram alterados.".into());}
        if !current_scope && origin.slot.as_ref().is_some_and(|slot|!slot.phase.replaceable()){return Err("Existe um registo anterior sem resultado confirmado. É necessária reconciliação por observação ou uma revisão própria de novo pedido no contexto atual; nenhum efeito será repetido automaticamente.".into());}
        if !current_scope{let s=current.try_lock().map_err(|_|refused())?;if !components::unresolved_history(&s,path,"shortcuts").map_err(|e|e.to_string())?.is_empty(){return Err("Há efeitos incertos no histórico dos atalhos. Não foram ignorados nem repetidos.".into());}}
        let action_views=actions(&stored)?;
        let native=native(&stored)?;
        let recovery={let s=current.try_lock().map_err(|_|refused())?;if current_scope{Some(scope_recovery::capture(&s,path,&native.destination)?)}else{reconciliation::registration_continuity(&s,path,&origin,&native.destination)?;None}};
        let mut preview=Preview{schema_version:2,token:random()?,fingerprint:String::new(),expires_at_ms:u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|refused())?.as_millis()+TTL.as_millis()).map_err(|_|refused())?,scope:"nativeUserShortcutsOnly",origin,configuration:hash(&encoded(&stored)?),resource:native.destination.resource()?,destination:native.destination.clone(),desired_bindings:stored.clone(),actions:action_views,unknown_action_ids:stored.keys().filter(|id|find(id).is_none()).cloned().collect(),activation_allowed:false,approval_available:true,recovery:None,message:"Revê as teclas/ações pretendidas e os registos anteriores que serão libertados. Aprovar autoriza uma tentativa neste destino: o ambiente gráfico pode pedir confirmação e escolher outras teclas. Falha parcial ou falta de recibo mantém incerteza, sem repetição nem reposição automática. Cancelar pode libertar apenas recursos próprios desta tentativa. Isto não concede permissões a pets nem reativa outras componentes; outro registo ou reinício exige nova revisão."};
        if current_scope{preview.schema_version=3;preview.scope="nativeUserShortcutsCurrentScopeOnly";preview.message="Este é um novo pedido no contexto atual. Revê as identidades anteriores, o destino atual e as teclas/ações: os efeitos antigos mantêm os seus recibos e a incerteza sobre o encerramento. O executor só usa o registrar/cliente atual e os seus recursos próprios; não limpa endereços históricos. Pode haver conflito de teclas e, no portal, configurações partilhadas pela aplicação e confirmação nativa. Sucesso atual não resolve o passado. Não há repetição, fallback ou reposição automática.";}
        preview.recovery=recovery;preview.fingerprint=hash(&encoded(&preview)?);
        let capture=Capture{preview:preview.clone(),frame:frame.clone(),expires:Instant::now()+TTL,native,path:path.into(),alive:Arc::new(AtomicBool::new(true))};
        self.validate_capture(&capture,frame,current,context)?;
        *self.reconciliation.lock().map_err(|_|refused())?=None;
        *self.pending.lock().map_err(|_|refused())?=Some(capture);
        if current_scope{self.gate.clear_poison();}
        Ok(preview)
    }
    fn validate_capture(&self,c:&Capture,frame:&Frame,current:&Mutex<Settings>,context:ContextReader)->Result<(),String>{
        if !c.frame.same(frame)||!frame.current()||!c.authorized()||!(c.native.current)(){return Err(refused());}
        let s=current.try_lock().map_err(|_|refused())?;settings::io_status(&s).check()?;
        if s.shortcuts!=c.preview.desired_bindings||context(&s,&c.path).map_err(|e|e.to_string())?!=c.preview.origin||!(c.native.current)(){return Err(refused());}Ok(())
    }
    fn inspect_with(&self,frame:&Frame,current:&Mutex<Settings>,key:&Key,context:ContextReader)->Result<Preview,String>{
        let slot=self.pending.lock().map_err(|_|refused())?;let c=slot.as_ref().ok_or_else(refused)?;
        if key.token.len()!=32||key.fingerprint.len()!=64||c.preview.token!=key.token||c.preview.fingerprint!=key.fingerprint{return Err(refused());}
        self.validate_capture(c,frame,current,context)?;Ok(c.preview.clone())
    }
    fn cancel(&self,frame:&Frame,key:Key)->Result<(),String>{
        if let Some(running)=self.running.lock().map_err(|_|refused())?.as_ref(){if running.frame.same(frame)&&key.token.len()==32&&key.fingerprint.len()==64&&running.key.token==key.token&&running.key.fingerprint==key.fingerprint{running.alive.store(false,Ordering::Release);return Ok(());}}
        if self.cancel_reconciliation(frame,&key)?{return Ok(());}
        let mut slot=self.pending.lock().map_err(|_|refused())?;let c=slot.as_ref().ok_or_else(refused)?;
        if !c.frame.same(frame)||key.token.len()!=32||key.fingerprint.len()!=64||c.preview.token!=key.token||c.preview.fingerprint!=key.fingerprint{return Err(refused());}
        c.alive.store(false,Ordering::Release);*slot=None;Ok(())
    }
}
pub(super)fn stop(){if let Ok(running)=controller().running.try_lock(){if let Some(r)=running.as_ref(){r.alive.store(false,Ordering::Release);}}}

#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Decision{token:String,fingerprint:String,reviewed_plan:bool,understand_native_effects:bool,keep_other_components_suspended:bool,#[serde(default)]preview_schema_version:Option<u32>,#[serde(default)]acknowledge_retained_uncertainty:bool}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt{retained_scope_uncertainty:bool,schema_version:u32,effect_id:String,registration:&'static str,publication:&'static str,requires_new_review:bool,status:settings::IoStatus,observed:crate::portal::review::Outcome,cleanup:Option<Cleanup>,message:String}
type Guard=crate::portal::review::Guard;
struct NativeRun{outcome:crate::portal::review::Outcome,finish:Option<Box<dyn FnOnce(Option<Arc<components::Execution>>,Guard)->Publication+Send>>}
impl NativeRun{fn refused(reason:&str)->Self{Self{outcome:crate::portal::review::Outcome::refused(reason,false),finish:Some(Box::new(|_,_|Publication::refused(None)))}}}
impl Drop for NativeRun{fn drop(&mut self){if let Some(finish)=self.finish.take(){finish(None,Arc::new(||false));}}}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Cleanup{attempted:bool,acknowledged:bool,reason:&'static str}
struct Publication{published:bool,current:Guard,abort:Option<Box<dyn FnOnce()->Cleanup+Send>>,cleanup:Option<Cleanup>}
impl Publication{
    fn refused(cleanup:Option<Cleanup>)->Self{Self{published:false,current:Arc::new(||false),abort:None,cleanup}}
    fn live(current:Guard,abort:impl FnOnce()->Cleanup+Send+'static)->Self{Self{published:true,current,abort:Some(Box::new(abort)),cleanup:None}}
    fn abort_now(&mut self){self.published=false;if let Some(abort)=self.abort.take(){self.cleanup=Some(abort());}}
    fn commit(&mut self){self.abort=None;}
}
impl Drop for Publication{fn drop(&mut self){if self.abort.is_some(){self.abort_now();}}}
struct Flight<'a>{controller:&'a Controller,token:String}
impl Drop for Flight<'_>{fn drop(&mut self){if let Ok(mut running)=self.controller.running.lock(){if running.as_ref().is_some_and(|r|r.key.token==self.token){if let Some(r)=running.take(){r.alive.store(false,Ordering::Release);}}}}}
impl Controller{
    fn apply_with(&self,frame:&Frame,current:&Mutex<Settings>,decision:Decision,context:ContextReader,admit:Guard,execute:impl FnOnce(&Capture,Guard,Guard)->NativeRun)->Result<Receipt,String>{
        if !decision.reviewed_plan||!decision.understand_native_effects||!decision.keep_other_components_suspended{return Err("A aprovação exige confirmação do plano, impacto nativo e âmbito restrito.".into());}
        let _gate=self.gate.try_lock().map_err(|_|"O worker anterior ainda está em curso.".to_string())?;
        let capture={let mut pending=self.pending.lock().map_err(|_|refused())?;let c=pending.as_ref().ok_or_else(refused)?;
            let approved=if c.preview.schema_version==3{decision.preview_schema_version==Some(3)&&decision.acknowledge_retained_uncertainty&&c.preview.recovery.is_some()}else{c.preview.schema_version==2&&decision.preview_schema_version.is_none_or(|v|v==2)&&!decision.acknowledge_retained_uncertainty};
            if !approved||decision.token.len()!=32||decision.fingerprint.len()!=64||decision.token!=c.preview.token||decision.fingerprint!=c.preview.fingerprint{return Err(refused());}
            self.validate_capture(c,frame,current,context)?;
            let c=pending.take().ok_or_else(refused)?;
            *self.running.lock().map_err(|_|refused())?=Some(Running{key:Key{token:c.preview.token.clone(),fingerprint:c.preview.fingerprint.clone()},frame:c.frame.clone(),alive:c.alive.clone()});c};
        let _flight=Flight{controller:self,token:capture.preview.token.clone()};
        let (authority_frame,alive,expires)=(capture.frame.clone(),capture.alive.clone(),capture.expires);
        let lifetime:Guard=Arc::new(move||authority_frame.current()&&alive.load(Ordering::Acquire)&&Instant::now()<expires&&admit());
        let binding=capture.binding();let id=capture.preview.token.clone();
        let execution={let s=current.lock().map_err(|_|refused())?;
            if !lifetime()||!(capture.native.current)()||s.shortcuts!=capture.preview.desired_bindings||context(&s,&capture.path).map_err(|e|e.to_string())?!=capture.preview.origin{return Err(refused());}
            let intent=components::Artifact{version:1,generation:1,id:id.clone(),effect:id.clone(),component:"shortcuts".into(),binding:binding.clone(),phase:components::Phase::Prepared,evidence:serde_json::json!({"review":capture.preview,"reviewedPlan":true,"understandNativeEffects":true,"keepOtherComponentsSuspended":true}),previous:capture.preview.origin.slot.clone()};
            let guard=||if lifetime()&&(capture.native.current)(){Ok(())}else{Err("shortcutReviewRevoked")};
            let mut intent=intent;if capture.preview.recovery.is_some(){intent.evidence["acknowledgeRetainedUncertainty"]=serde_json::json!(true);}
            let reference=components::retain(&capture.path,&intent,guard).map_err(|e|e.to_string())?;
            let owner=hash(format!("native-shortcuts-worker-v1:{}:{}",std::process::id(),random()?).as_bytes());
            let slot=components::Slot{id:id.clone(),component:"shortcuts".into(),phase:components::Phase::Prepared,binding:binding.clone(),owner,intent:reference,outcome:None};
            if capture.preview.recovery.is_some(){components::prepare_current_shortcuts(&s,&capture.path,slot,guard)}else{components::prepare(&s,&capture.path,slot,guard)}.map_err(|e|e.to_string())?;
            components::transition(&s,&capture.path,"shortcuts",&id,components::Phase::Dispatched,None,guard).map_err(|e|e.to_string())?;
            Arc::new(components::execution_for_component(&s,&capture.path,"shortcuts",&id,&binding,capture.preview.origin.data_review.as_ref()).map_err(|e|e.to_string())?)
        };
        let authority=lifetime.clone();let source=execution.clone();let io:Guard=Arc::new(move||authority()&&source.current());
        let mut run=execute(&capture,io.clone(),lifetime.clone());
        let mut phase=if run.outcome.native_effects_started{components::Phase::Unconfirmed}else{components::Phase::NotDispatched};
        let (pending,witness)={let s=current.lock().map_err(|_|refused())?;
            let exact=context(&s,&capture.path).is_ok_and(|c|c.profile==capture.preview.origin.profile&&c.data_review==capture.preview.origin.data_review&&c.slot.is_some_and(|slot|slot.id==id&&slot.binding==binding&&slot.phase==components::Phase::Dispatched));
            let observed=run.outcome.verified&&io()&&exact&&s.shortcuts==capture.preview.desired_bindings;
            // A crash after this write must remain uncertain to every reader,
            // including readers predating this publication protocol.
            if observed{phase=components::Phase::Unconfirmed;}
            let result=(||->std::io::Result<_>{
                let mut evidence=serde_json::json!({"nativeObservation":run.outcome,"nativeEffectsStarted":run.outcome.native_effects_started,"noAutomaticReplay":true,"requiresNewReview":true});
                if observed{evidence["publicationProtocol"]=serde_json::json!(1);evidence["publication"]=serde_json::json!("pending");}
                let artifact=components::Artifact{version:1,generation:1,id:random().map_err(std::io::Error::other)?,effect:id.clone(),component:"shortcuts".into(),binding:binding.clone(),phase,evidence,previous:None};
                let reference=components::retain(&capture.path,&artifact,||Ok(()))?;
                components::transition(&s,&capture.path,"shortcuts",&id,phase,Some(reference.clone()),||Ok(()))?;Ok(reference)
            })();
            match result{
                Ok(reference)if observed=>{let witness=components::shortcut_publication_execution(&s,&capture.path,&id,&binding,&reference,capture.preview.origin.data_review.as_ref()).ok().map(Arc::new);(Some(reference),witness)},
                Ok(_)=>(None,None),Err(_)=>{phase=components::Phase::Unconfirmed;(None,None)}
            }
        };
        let native_verified=witness.is_some();
        let mut publication=(run.finish.take().unwrap())(witness.clone(),lifetime.clone());
        let mut complete=false;
        if publication.published&&(publication.current)()&&lifetime()&&witness.as_ref().is_some_and(|w|w.current()){
            let s=current.lock().map_err(|_|refused())?;
            let exact=context(&s,&capture.path).is_ok_and(|c|c.profile==capture.preview.origin.profile&&c.data_review==capture.preview.origin.data_review&&c.slot.is_some_and(|slot|slot.id==id&&slot.binding==binding&&slot.phase==components::Phase::Unconfirmed&&slot.outcome==pending));
            if exact&&s.shortcuts==capture.preview.desired_bindings{
                let result=(||->std::io::Result<()>{
                    let artifact=components::Artifact{version:1,generation:1,id:random().map_err(std::io::Error::other)?,effect:id.clone(),component:"shortcuts".into(),binding:binding.clone(),phase:components::Phase::Verified,evidence:serde_json::json!({"publicationProtocol":1,"publication":"published","pendingOutcome":pending,"nativeObservation":run.outcome,"nativeEffectsStarted":run.outcome.native_effects_started,"noAutomaticReplay":true,"requiresNewReview":true}),previous:None};
                    let guard=||if lifetime()&&(publication.current)(){Ok(())}else{Err("shortcutPublicationRevoked")};
                    let reference=components::retain(&capture.path,&artifact,guard)?;
                    components::transition(&s,&capture.path,"shortcuts",&id,components::Phase::Verified,Some(reference),guard)
                })();complete=result.is_ok();
            }
        }
        if complete{phase=components::Phase::Verified;publication.commit();}else{publication.abort_now();}
        let status={let s=current.lock().map_err(|_|refused())?;settings::io_status(&s)};
        let registration=if native_verified{"verified"}else if phase==components::Phase::NotDispatched{"notDispatched"}else{"unconfirmed"};
        Ok(Receipt{retained_scope_uncertainty:capture.preview.recovery.is_some(),schema_version:1,effect_id:id,registration,publication:if complete{"published"}else if native_verified{"unconfirmed"}else{"notPublished"},requires_new_review:true,status,observed:run.outcome.clone(),cleanup:publication.cleanup.clone(),message:if complete&&capture.preview.recovery.is_some(){"Registo atual, publicação e recibo confirmados. O histórico e a incerteza sobre o encerramento dos efeitos antigos foram conservados. Outra tentativa exige nova revisão."}else if complete{"Registo nativo, publicação e recibo confirmados nesta sessão. Outro registo ou reinício exige nova revisão."}else if registration=="notDispatched"{"Nenhum efeito nativo foi despachado. Não houve repetição; uma nova tentativa exige outra revisão."}else{"O registo, a publicação ou a limpeza ficou sem confirmação integral. O diário conserva a incerteza; consulta o estado e reconcilia antes de outra tentativa."}.into()})
    }
}
#[path="shortcuts_review_native.rs"]mod native;
#[path="shortcuts_scope_recovery.rs"]mod scope_recovery;
#[path="shortcuts_reconciliation.rs"]pub(crate)mod reconciliation;
fn permitted(window:&tauri::WebviewWindow)->bool{window.label()==crate::HUB_LABEL&&window.is_visible().unwrap_or(false)}
fn path()->PathBuf{crate::platform::config_dir().join("settings.json")}
#[tauri::command]
pub(crate)async fn shortcuts_review_prepare(app:AppHandle,window:tauri::WebviewWindow)->Result<Preview,String>{
    if !permitted(&window){return Err("Abre as definições para rever os atalhos.".into());}let frame=settings::review::controller().frame()?;let path=path();
    tauri::async_runtime::spawn_blocking(move||{let shared=app.state::<crate::Shared>();controller().prepare_with(&frame,&shared.settings,&path,native_context,|stored|NativeCapture::capture(&app,stored))}).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)async fn shortcuts_review_prepare_current_scope(app:AppHandle,window:tauri::WebviewWindow)->Result<Preview,String>{
    if !permitted(&window){return Err("Abre as definições para rever o novo pedido no contexto atual.".into());}let frame=settings::review::controller().frame()?;let path=path();
    tauri::async_runtime::spawn_blocking(move||{let registry=app.state::<Registry>();let _physical=match registry.physical.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O executor anterior ainda detém o recurso nativo.".into())};let shared=app.state::<crate::Shared>();controller().prepare_mode_with(&frame,&shared.settings,&path,native_context,true,|stored|{if crate::platform::global_shortcuts_blocked()==Some("wayland"){super::wayland::refresh_observer(&app);}NativeCapture::capture(&app,stored)})}).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)async fn shortcuts_review_inspect(app:AppHandle,window:tauri::WebviewWindow,request:Key)->Result<Preview,String>{
    if !permitted(&window){return Err(refused());}let frame=settings::review::controller().frame()?;
    tauri::async_runtime::spawn_blocking(move||{let shared=app.state::<crate::Shared>();controller().inspect_with(&frame,&shared.settings,&request,native_context)}).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)fn shortcuts_review_cancel(window:tauri::WebviewWindow,request:Key)->Result<(),String>{
    if window.label()!=crate::HUB_LABEL{return Err(refused());}controller().cancel(&settings::review::controller().frame()?,request)
}
#[tauri::command]
pub(crate)async fn shortcuts_review_apply(app:AppHandle,window:tauri::WebviewWindow,request:Decision)->Result<Receipt,String>{
    if !permitted(&window){return Err("A aprovação exige as definições abertas.".into());}let frame=settings::review::controller().frame()?;
    tauri::async_runtime::spawn_blocking(move||{
        let registry=app.state::<Registry>();let _physical=match registry.physical.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))if request.preview_schema_version==Some(3)&&request.acknowledge_retained_uncertainty=>e.into_inner(),Err(_)=>return Err("O executor anterior ainda detém o recurso nativo.".into())};
        let shared=app.state::<crate::Shared>();let receipt=controller().apply_with(&frame,&shared.settings,request,native_context,Arc::new(||crate::pet_data::bootstrap::write_allowed().is_ok()),|capture,guard,lifetime|native::execute(&app,capture,guard,lifetime))?;
        registry.physical.clear_poison();let _=app.emit("settings-review-state-changed",&receipt.status);Ok(receipt)
    }).await.map_err(|_|"O worker terminou sem resposta confirmada. Consulta o estado; não repitas o registo.".to_string())?
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct ReviewStatus{retained_scope_effects:Option<Vec<components::Slot>>,schema_version:u32,effect:Option<components::Slot>,effect_available:bool,status:settings::IoStatus,native:Report,in_flight:bool,message:&'static str}
#[tauri::command]
pub(crate)async fn shortcuts_review_status(app:AppHandle,window:tauri::WebviewWindow)->Result<ReviewStatus,String>{
    if window.label()!=crate::HUB_LABEL{return Err(refused());}
    tauri::async_runtime::spawn_blocking(move||{let shared=app.state::<crate::Shared>();let(status,effect,retained)={let s=shared.settings.lock().map_err(|_|refused())?;let status=settings::io_status(&s);let effect=if status.new_effects_allowed{native_context(&s,&path()).ok().map(|c|c.slot)}else{None};let retained=components::unresolved_history(&s,&path(),"shortcuts").ok();(status,effect,retained)};
        Ok(ReviewStatus{retained_scope_effects:retained,schema_version:1,effect_available:effect.is_some(),effect:effect.flatten(),status,native:super::status(&app),in_flight:controller().running.lock().map_err(|_|refused())?.is_some(),message:"Consulta sem registo nativo. O recibo conserva factos anteriores; o estado nativo indica esta sessão. Não há repetição automática nem reativação após reinício."})
    }).await.map_err(|_|refused())?
}

#[cfg(test)]#[path="shortcuts_review_tests.rs"]mod tests;
