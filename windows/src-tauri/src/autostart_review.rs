//! Native-user review of one frozen registration effect. It is not a pet grant.
use super::Controller;
use crate::{settings::{self, Settings, review::Frame, durable::components::{self, Context, Slot, Binding, Artifact, Phase}}, pet_autostart::native::{Plan, View}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

const TTL: Duration = Duration::from_secs(300);
const COMPONENT: &str = "autostart";
type ContextReader = fn(&Settings, &Path) -> io::Result<Context>;
fn native_context(s: &Settings, p: &Path) -> io::Result<Context> { components::context(s,p,COMPONENT) }
fn hash(b: &[u8]) -> String { ring::digest::digest(&ring::digest::SHA256,b).as_ref().iter().map(|b|format!("{b:02x}")).collect() }
fn random() -> Result<String,String> { use ring::rand::SecureRandom; let mut b=[0;16];ring::rand::SystemRandom::new().fill(&mut b).map_err(|_|refused())?;Ok(b.iter().map(|v|format!("{v:02x}")).collect()) }
fn refused() -> String { "O plano mudou, expirou ou foi cancelado. Consulta o estado e prepara outra revisão; nenhum efeito será repetido automaticamente.".into() }
fn io_error() -> io::Error { io::Error::other(refused()) }
#[derive(Debug)]enum GuardFault{Profile,Revoked,Plan,Fence,SettingsBusy,Context}
impl GuardFault{fn code(&self)->&'static str{match self{Self::Profile=>"profileUnavailable",Self::Revoked=>"reviewRevoked",Self::Plan=>"planChanged",Self::Fence=>"resourceFenceChanged",Self::SettingsBusy=>"settingsBusy",Self::Context=>"effectContextChanged"}}}
impl std::fmt::Display for GuardFault{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(self.code())}}
impl std::error::Error for GuardFault{}
fn fault(value:GuardFault)->io::Error{io::Error::other(value)}
fn now_ms() -> Result<u64,String> { u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|refused())?.as_millis()).map_err(|_|refused()) }
fn encoded(bytes: Option<&[u8]>) -> Option<String> { bytes.map(|b|b.iter().map(|v|format!("{v:02x}")).collect()) }
fn ready(s: &Settings) -> Result<(),String> { settings::io_status(s).check() }
fn process() -> Result<String,String> {
    static NONCE:std::sync::OnceLock<String>=std::sync::OnceLock::new();
    let nonce=if let Some(n)=NONCE.get(){n.clone()}else{let n=random()?;let _=NONCE.set(n);NONCE.get().ok_or_else(refused)?.clone()};
    Ok(hash(format!("native-autostart-worker-v1:{}:{nonce}",std::process::id()).as_bytes()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Action { SetRegistration, SetIndependentRegistration, ReconcileObservation, ReconcileArchivedObservation, ReconcileLegacyObservation }
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Request { pub action:Action, #[serde(default)]pub enabled:Option<bool>,#[serde(default)]pub archived_effect_id:Option<String> }
#[derive(Clone, Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Preview {
    pub schema_version:u32,pub token:String,pub fingerprint:String,pub expires_at_ms:u64,pub action:Action,
    pub desired_enabled:bool,pub previous_desired_enabled:bool,pub profile_scope:String,pub origin:Context,pub plan:View,
    #[serde(skip_serializing_if="Option::is_none")]pub archived_effect:Option<Slot>,
    pub impact:&'static str,pub scope:&'static str,pub message:&'static str,
}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Key { pub token:String,pub fingerprint:String }
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Decision {pub token:String,pub fingerprint:String,pub reviewed_plan:bool,pub understand_effect:bool,pub keep_other_components_suspended:bool,#[serde(default)]pub acknowledge_external_uncertainty:bool,#[serde(default)]pub acknowledge_legacy_uncertainty:bool}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]#[serde(rename_all="camelCase")]
pub(crate)enum Registration { Verified, Unconfirmed, NotDispatched, ReconciledObservation }
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Receipt {pub schema_version:u32,pub desired_enabled:bool,pub registration:Registration,pub login_verified:bool,pub reactivated:bool,pub effect_id:String,pub status:settings::IoStatus,pub message:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub(crate)struct Status {pub schema_version:u32,pub requires_review:bool,pub desired_enabled:bool,pub status:settings::IoStatus,pub effect:Option<Slot>,pub registration_matches:Option<bool>,pub independent_destination:bool,pub unresolved_history:Vec<Slot>,pub observed_history:Vec<components::HistoricalProjection>,pub historical_reconcilable:Vec<String>,pub legacy_reconcilable:Vec<String>,pub in_flight:bool,pub message:String}

struct Capture {preview:Preview,plan:Plan,path:PathBuf,frame:Frame,expires:Instant,alive:Arc<AtomicBool>}
impl Capture {
    fn authorized(&self) -> bool {self.frame.current()&&self.alive.load(Ordering::Acquire)&&Instant::now()<self.expires}
    fn binding(&self) -> Binding {Binding{profile:self.preview.profile_scope.clone(),configuration:components::autostart_configuration(if matches!(self.preview.action,Action::ReconcileArchivedObservation|Action::ReconcileLegacyObservation){self.preview.previous_desired_enabled}else{self.preview.desired_enabled}),resource:self.preview.plan.resource.clone(),plan:self.preview.plan.fingerprint.clone()}}
}
struct ReviewSlot {token:String,fingerprint:String,frame:Frame,alive:Arc<AtomicBool>,pending:Option<Capture>}
#[derive(Default)]pub(super)struct ReviewState {slot:Option<ReviewSlot>}
struct Flight<'a>{controller:&'a Controller,token:String}
impl Drop for Flight<'_>{fn drop(&mut self){if let Ok(mut s)=self.controller.review.lock(){if s.slot.as_ref().is_some_and(|v|v.token==self.token&&v.pending.is_none()){if let Some(v)=s.slot.take(){v.alive.store(false,Ordering::Release);}}}}}

impl Controller {
    fn prepare_review(&self,frame:&Frame,current:&Mutex<Settings>,path:&Path,plan:Plan,action:Action,context:ContextReader)->Result<Preview,String>{
        self.prepare_review_selected(frame,current,path,plan,action,context,None)
    }
    fn prepare_review_selected(&self,frame:&Frame,current:&Mutex<Settings>,path:&Path,plan:Plan,action:Action,context:ContextReader,archived_effect:Option<Slot>)->Result<Preview,String>{
        if !frame.current()||!plan.current(){return Err(refused());}
        let settings=current.try_lock().map_err(|_|"As preferências estão ocupadas; não foi iniciada uma revisão.".to_string())?;ready(&settings)?;
        let origin=context(&settings,path).map_err(|e|e.to_string())?;let scope=origin.profile.clone().ok_or_else(refused)?;
        let historical=components::history_for_resource(&settings,path,COMPONENT,&plan.view.resource).map_err(|e|e.to_string())?.unresolved;
        if plan.view.schema_version!=2{return Err(refused());}
        if action==Action::ReconcileArchivedObservation{
            if archived_effect.as_ref().is_none_or(|old|old.binding.resource!=plan.view.resource||!historical.contains(old)){return Err("O efeito arquivado não corresponde a um ancestral incerto no recurso nativo atual.".into());}
            let subject=archived_effect.as_ref().ok_or_else(refused)?;let(actual,intent)=components::historical_subject(&settings,path,&subject.id).map_err(|e|e.to_string())?;
            let original:View=serde_json::from_value(intent.evidence.get("nativePlan").cloned().ok_or_else(refused)?).map_err(|_|refused())?;
            if &actual!=subject||original.enabled!=plan.view.enabled||original.resource!=subject.binding.resource||original.fingerprint!=subject.binding.plan{return Err(refused());}
            if origin.slot.as_ref().is_some_and(|current|!current.phase.replaceable()&&(!current.independent_autostart_destination(&scope,&plan.view.resource)||components::legacy_slot(path,current).unwrap_or(true))){return Err("Reconcilia primeiro a operação atual incerta no seu recurso nativo; observar um ancestral não pode contornar essa barreira.".into());}
        }else if action==Action::ReconcileLegacyObservation{
            let subject=archived_effect.as_ref().ok_or_else(refused)?;let(actual,intent)=components::legacy_subject(&settings,path,&subject.id).map_err(|e|e.to_string())?;
            let original:View=serde_json::from_value(intent.evidence.get("nativePlan").cloned().ok_or_else(refused)?).map_err(|_|refused())?;
            if &actual!=subject||original.enabled!=plan.view.enabled||!(historical.contains(subject)||origin.slot.as_ref()==Some(subject)){return Err(refused());}
            if origin.slot.as_ref().is_some_and(|current|!current.phase.replaceable()&&current!=subject){return Err("Revê primeiro a operação atual incerta; a origem legada não prova outro recurso físico.".into());}
        }else if archived_effect.is_some(){return Err(refused());}
        let active_observation=action==Action::ReconcileObservation&&origin.slot.as_ref().is_some_and(|current|!current.phase.replaceable()&&current.binding.resource==plan.view.resource);
        if !matches!(action,Action::ReconcileArchivedObservation|Action::ReconcileLegacyObservation)&&!active_observation&&historical.iter().map(|old|components::blocks_resource(path,old,&plan.view.resource)).collect::<io::Result<Vec<_>>>().map_err(|e|e.to_string())?.into_iter().any(|blocked|blocked){return Err("Existe uma operação incerta arquivada para este recurso. Não foi repetida nem declarada reconciliada; é necessária reconciliação explícita desse recurso.".into());}
        match action {
            Action::SetRegistration if origin.slot.as_ref().is_some_and(|s|!s.phase.replaceable()) => return Err("Há uma operação anterior sem confirmação. Prepara primeiro uma reconciliação por observação.".into()),
            Action::SetIndependentRegistration if origin.slot.as_ref().is_none_or(|s|!s.independent_autostart_destination(&scope,&plan.view.resource)) => return Err("O novo trabalho exige um perfil e um recurso nativos diferentes. A operação anterior continua incerta e não foi repetida.".into()),
            Action::ReconcileObservation if origin.slot.as_ref().is_none_or(|s|s.phase.replaceable()) => return Err("Não há uma operação incerta a reconciliar. Prepara a alteração de registo pretendida.".into()),
            _=>{}
        }
        if matches!(action,Action::ReconcileObservation|Action::SetIndependentRegistration)&&origin.slot.as_ref().map(|s|components::legacy_slot(path,s)).transpose().map_err(|e|e.to_string())?.unwrap_or(false){return Err("A origem física desta operação legada é desconhecida. Prepara a revisão específica de origem legada; não foi presumido um recurso diferente.".into());}
        if action==Action::ReconcileObservation&&origin.slot.as_ref().is_some_and(|s|s.binding.resource!=plan.view.resource){return Err("O recurso da operação anterior não corresponde ao destino nativo atual. Os dados foram conservados; não foi escolhido outro alvo.".into());}
        let impact=match action {
            Action::SetRegistration=>"Guardar a preferência e criar, substituir ou remover apenas o registo apresentado. Pode criar o diretório indicado e um ficheiro privado de exclusão; não inicia o programa nem prova o próximo login.",
            Action::SetIndependentRegistration=>"A operação anterior permanece incerta no recurso antigo. Não será observada, repetida, revertida ou declarada reconciliada. Guardar a preferência e criar, substituir ou remover apenas o registo no novo destino apresentado, com exclusão própria. Não inicia o programa nem prova o próximo login.",
            Action::ReconcileObservation=>"Observar o recurso nativo e registar o resultado da reconciliação. Não repetir a operação anterior, não alterar a preferência e não reativar a componente.",
            Action::ReconcileLegacyObservation=>"A origem física antiga não pode ser comprovada. Aprovas apenas observar este destino nativo atual e conservar essa limitação; não se afirma que é o recurso antigo nem que o efeito passado terminou. A preferência/registo não serão alterados e a componente continua suspensa. Outra alteração exige nova decisão neste recurso, perfil e revisão de dados.",
            Action::ReconcileArchivedObservation=>"Observar apenas o recurso nativo atual associado ao efeito arquivado apresentado. O resultado antigo continua desconhecido e os originais são conservados. Esta observação não altera o registo ou a preferência, não reativa a componente e só permite preparar outra revisão no perfil e revisão de dados atuais.",
        };
        let token=random()?;let mut preview=Preview{schema_version:1,token:token.clone(),fingerprint:String::new(),expires_at_ms:now_ms()?.checked_add(TTL.as_millis()as u64).ok_or_else(refused)?,action,desired_enabled:plan.view.enabled,previous_desired_enabled:settings.autostart,profile_scope:scope,origin,plan:plan.view.clone(),archived_effect,impact,scope:"nativeUserAutostartOnly",message:"Revê destino, launcher, original/diff, valor pretendido e estado anterior. A decisão vale apenas para este plano; as restantes componentes e permissões mantêm as suas barreiras."};
        preview.fingerprint=hash(&serde_json::to_vec(&preview).map_err(|_|refused())?);
        let capture=Capture{preview:preview.clone(),plan,path:path.into(),frame:frame.clone(),expires:Instant::now()+TTL,alive:Arc::new(AtomicBool::new(true))};
        if !capture.authorized()||!capture.plan.current(){return Err(refused());}drop(settings);
        let _gate=match self.gate.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O worker anterior ainda está em curso. Cancelar não prova que terminou.".into())};
        let mut state=self.review.lock().map_err(|_|refused())?;
        if state.slot.as_ref().is_some_and(|v|v.pending.is_none()){return Err(refused());}
        if let Some(old)=state.slot.take(){old.alive.store(false,Ordering::Release);}
        state.slot=Some(ReviewSlot{token,fingerprint:preview.fingerprint.clone(),frame:frame.clone(),alive:capture.alive.clone(),pending:Some(capture)});Ok(preview)
    }
    fn cancel_review(&self,frame:&Frame,key:Key)->Result<(),String>{
        let mut state=self.review.lock().map_err(|_|refused())?;let slot=state.slot.as_ref().ok_or_else(refused)?;
        if key.token.len()!=32||key.fingerprint.len()!=64||slot.token!=key.token||slot.fingerprint!=key.fingerprint||!slot.frame.same(frame){return Err(refused());}
        slot.alive.store(false,Ordering::Release);if slot.pending.is_some(){state.slot=None;}Ok(())
    }
    fn apply_review(&self,frame:&Frame,current:&Mutex<Settings>,decision:Decision,context:ContextReader,admit:impl Fn()->io::Result<()>,publish:impl Fn(&Settings))->Result<Receipt,String>{
        if !decision.reviewed_plan||!decision.understand_effect||!decision.keep_other_components_suspended{return Err("A operação exige aceitação explícita do plano, impacto e âmbito restrito.".into());}
        let _gate=match self.gate.try_lock(){Ok(g)=>g,Err(std::sync::TryLockError::Poisoned(e))=>e.into_inner(),Err(_)=>return Err("O worker anterior ainda está em curso; não foi iniciado outro efeito.".into())};
        let capture={let mut state=self.review.lock().map_err(|_|refused())?;let slot=state.slot.as_mut().ok_or_else(refused)?;
            if decision.token.len()!=32||decision.fingerprint.len()!=64||slot.token!=decision.token||slot.fingerprint!=decision.fingerprint||!slot.frame.same(frame){return Err(refused());}
            let pending=slot.pending.as_ref().ok_or_else(refused)?;if !pending.authorized(){return Err(refused());}
            if pending.preview.action==Action::ReconcileLegacyObservation&&!decision.acknowledge_legacy_uncertainty{return Err("É necessário reconhecer explicitamente que a origem física legada não pode ser comprovada e que só o estado presente é observado.".into());}
            if decision.acknowledge_legacy_uncertainty&&pending.preview.action!=Action::ReconcileLegacyObservation{return Err(refused());}
            if pending.preview.action==Action::SetIndependentRegistration&&!decision.acknowledge_external_uncertainty{return Err("É necessário reconhecer explicitamente que a operação no recurso antigo continua incerta.".into());}
            slot.pending.take().ok_or_else(refused)?};
        let _flight=Flight{controller:self,token:capture.preview.token.clone()};
        let authorized=||->io::Result<()>{admit().map_err(|_|fault(GuardFault::Profile))?;if capture.authorized(){Ok(())}else{Err(fault(GuardFault::Revoked))}};
        let strict=||->io::Result<()>{authorized()?;if capture.plan.current(){Ok(())}else{Err(fault(GuardFault::Plan))}};
        let guard=||strict().map_err(|_|"nativeAutostartReviewRevoked");
        let mut settings=current.lock().map_err(|_|refused())?;ready(&settings)?;
        if settings.autostart!=capture.preview.previous_desired_enabled||context(&settings,&capture.path).map_err(|e|e.to_string())?!=capture.preview.origin{ return Err(refused()); }
        strict().map_err(|e|e.to_string())?;
        if matches!(capture.preview.action,Action::ReconcileArchivedObservation|Action::ReconcileLegacyObservation){
            let subject=capture.preview.archived_effect.as_ref().ok_or_else(refused)?;let binding=capture.binding();let id=capture.preview.token.clone();
            let history=components::historical_witness(&settings,&capture.path,subject).map_err(|e|e.to_string())?;
            let source=components::read_execution(&settings,&capture.path,&capture.preview.profile_scope).map_err(|e|e.to_string())?;
            let read_guard=||{strict()?;if source.current()&&history.current(){Ok(())}else{Err(io_error())}};
            let intent=Artifact{version:1,generation:1,id:id.clone(),effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::Prepared,evidence:json!({"nativePlan":capture.preview.plan,"review":capture.preview,"historicalSubject":subject,"legacyUncertaintyAcknowledged":decision.acknowledge_legacy_uncertainty,"reviewedPlan":true,"understandEffect":true,"keepOtherComponentsSuspended":true,"noRegistrationEffect":true}),previous:capture.preview.origin.slot.clone()};
            let intent_ref=components::retain(&capture.path,&intent,||read_guard().map_err(|_|"nativeObservationChanged")).map_err(|e|e.to_string())?;
            drop(settings);let fence=capture.plan.fence(read_guard).map_err(|e|e.to_string())?;
            let observed=||{read_guard()?;if fence.current(){Ok(())}else{Err(io_error())}};observed().map_err(|e|e.to_string())?;
            let claim=components::HistoricalObservation{schema_version:if capture.preview.action==Action::ReconcileLegacyObservation{2}else{1},legacy_origin_unverifiable:capture.preview.action==Action::ReconcileLegacyObservation,legacy_uncertainty_acknowledged:decision.acknowledge_legacy_uncertainty,subject:subject.clone(),profile:capture.preview.profile_scope.clone(),data_review:capture.preview.origin.data_review.clone(),native_plan:capture.preview.plan.clone(),registration_matches_expected:capture.plan.already_matches(),no_registration_effect:true,no_reactivation:true,login_verified:false,past_effect_outcome_known:false};
            let outcome=Artifact{version:1,generation:1,id:random()?,effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::ReconciledObservation,evidence:json!({"archivedObservation":claim}),previous:None};
            let outcome_ref=components::retain(&capture.path,&outcome,||observed().map_err(|_|"nativeObservationChanged")).map_err(|e|e.to_string())?;
            settings=current.lock().map_err(|_|refused())?;
            if context(&settings,&capture.path).map_err(|e|e.to_string())?!=capture.preview.origin{return Err(refused());}observed().map_err(|e|e.to_string())?;
            let slot=Slot{id:id.clone(),component:COMPONENT.into(),phase:Phase::ReconciledObservation,binding,owner:process()?,intent:intent_ref,outcome:Some(outcome_ref)};
            // component_change intentionally advances the real Revision epoch.
            // Shared is now held; its own source/chain witnesses guard publication.
            components::record_historical_observation(&settings,&capture.path,slot,||{if !fence.current()||!history.current(){return Err("nativeObservationChanged");}guard()}).map_err(|e|e.to_string())?;
            self.gate.clear_poison();return Ok(receipt(&settings,Registration::ReconciledObservation,&id,"Observação atual registada sem alterar o registo ou a preferência. O resultado antigo continua desconhecido e conservado. A componente permanece suspensa; uma alteração exige nova revisão."));
        }
        if capture.preview.action==Action::ReconcileObservation{
            let old=capture.preview.origin.slot.as_ref().ok_or_else(refused)?;
            // Profile ownership and the local gate establish exclusion; never a
            // timeout/expired lease. This executor has no child processes.
            drop(settings);let fence=capture.plan.fence(strict).map_err(|e|e.to_string())?;strict().map_err(|e|e.to_string())?;
            settings=current.lock().map_err(|_|refused())?;
            if context(&settings,&capture.path).map_err(|e|e.to_string())?!=capture.preview.origin{return Err(refused());}
            let artifact=Artifact{version:1,generation:1,id:random()?,effect:old.id.clone(),component:COMPONENT.into(),binding:old.binding.clone(),phase:Phase::ReconciledObservation,evidence:json!({"nativeObservation":capture.preview.plan,"registrationMatchesExpected":capture.plan.already_matches(),"noRegistrationEffect":true,"noReactivation":true,"loginVerified":false}),previous:None};
            let observe_guard=||{if !fence.current(){return Err("nativeAutostartFenceChanged");}guard()};
            let reference=components::retain(&capture.path,&artifact,observe_guard).map_err(|e|e.to_string())?;
            components::transition(&settings,&capture.path,COMPONENT,&old.id,Phase::ReconciledObservation,Some(reference),observe_guard).map_err(|e|e.to_string())?;
            self.gate.clear_poison();return Ok(receipt(&settings,Registration::ReconciledObservation,&old.id,"Recurso observado e reconciliação registada. A componente continua suspensa; uma alteração exige outro plano e aprovação."));
        }
        let binding=capture.binding();let id=capture.preview.token.clone();
        let independent=capture.preview.action==Action::SetIndependentRegistration;
        let artifact=Artifact{version:1,generation:1,id:id.clone(),effect:id.clone(),component:COMPONENT.into(),binding:binding.clone(),phase:Phase::Prepared,evidence:json!({"nativePlan":capture.preview.plan,"originalBytesHex":encoded(capture.plan.original()),"expectedBytesHex":encoded(capture.plan.expected()),"review":capture.preview,"reviewedPlan":true,"understandEffect":true,"keepOtherComponentsSuspended":true,"externalUncertaintyAcknowledged":independent&&decision.acknowledge_external_uncertainty,"previousEffectRemainsUnresolved":independent}),previous:capture.preview.origin.slot.clone()};
        let reference=components::retain(&capture.path,&artifact,guard).map_err(|e|e.to_string())?;
        let slot=Slot{id:id.clone(),component:COMPONENT.into(),phase:Phase::Prepared,binding:binding.clone(),owner:process()?,intent:reference,outcome:None};
        if independent{components::prepare_independent_autostart(&settings,&capture.path,slot,guard)}else{components::prepare(&settings,&capture.path,slot,guard)}.map_err(|e|e.to_string())?;
        if components::save_autostart(&mut settings,&capture.path,capture.preview.desired_enabled,strict).is_err(){
            return Ok(record_outcome(&mut settings,&capture,Phase::NotDispatched,json!({"noRegistrationEffect":true,"reason":"preferenceNotConfirmed"}),||Ok(())));
        }
        let fresh=settings.clone();drop(settings);publish(&fresh);
        let fence=match capture.plan.fence(strict){Ok(v)=>v,Err(_)=>{let mut s=current.lock().map_err(|_|refused())?;return Ok(record_outcome(&mut s,&capture,Phase::NotDispatched,json!({"noRegistrationEffect":true,"reason":"resourceFenceUnavailableOrReviewRevoked"}),||Ok(())));}};
        let execution={
            let s=current.lock().map_err(|_|refused())?;
            if !effect_current(&s,&capture,Phase::Prepared,context){return Ok(receipt(&s,Registration::Unconfirmed,&id,"A intenção ou configuração mudou antes do despacho. Consulta o estado e reconcilia; o registo não foi repetido."));}
            if components::transition(&s,&capture.path,COMPONENT,&id,Phase::Dispatched,None,||{if !fence.current(){return Err("nativeAutostartFenceChanged");}guard()}).is_err(){return Ok(receipt(&s,Registration::Unconfirmed,&id,"O despacho não foi confirmado. Consulta o estado e reconcilia antes de outro pedido."));}
            components::execution(&s,&capture.path,&id,&binding,capture.preview.origin.data_review.as_ref()).map_err(|e|e.to_string())?
        };
        let result=capture.plan.execute(||{
            authorized()?;if !fence.current(){return Err(fault(GuardFault::Fence));}
            if execution.current(){authorized()}else{Err(fault(GuardFault::Context))}
        });
        let mut settings=current.lock().map_err(|_|refused())?;
        let verified=result.is_ok()&&execution.current()&&fence.current()&&authorized().is_ok()&&effect_current(&settings,&capture,Phase::Dispatched,context);
        let evidence=match result{Ok(observed)=>json!({"observation":observed,"reviewStillCurrent":verified}),Err(error)=>json!({"reason":"executorOrPostconditionUnconfirmed","guardFailure":error.get_ref().and_then(|e|e.downcast_ref::<GuardFault>()).map(GuardFault::code).unwrap_or("targetOrFilesystem"),"errorKind":format!("{:?}",error.kind()),"noAutomaticReplay":true})};
        let phase=if verified{Phase::Verified}else{Phase::Unconfirmed};
        let receipt=record_outcome(&mut settings,&capture,phase,evidence,||if phase==Phase::Verified{if !fence.current(){return Err(io_error());}authorized()}else{Ok(())});
        drop(fence);self.gate.clear_poison();Ok(receipt)
    }
}
fn effect_current(settings:&Settings,capture:&Capture,phase:Phase,context:ContextReader)->bool{
    settings.autostart==capture.preview.desired_enabled&&context(settings,&capture.path).is_ok_and(|c|c.profile.as_deref()==Some(capture.preview.profile_scope.as_str())&&c.data_review==capture.preview.origin.data_review&&c.slot.is_some_and(|s|s.id==capture.preview.token&&s.phase==phase&&s.binding==capture.binding()))
}
fn receipt(settings:&Settings,registration:Registration,id:&str,message:&str)->Receipt{
    let status=settings::io_status(settings);let reactivated=registration==Registration::Verified&&status.new_effects_allowed&&!status.suspended_operations.iter().any(|n|n==COMPONENT);
    Receipt{schema_version:1,desired_enabled:settings.autostart,registration,login_verified:false,reactivated,effect_id:id.into(),status,message:message.into()}
}
fn record_outcome(settings:&mut Settings,capture:&Capture,phase:Phase,evidence:Value,guard:impl Fn()->io::Result<()>)->Receipt{
    let done=(||->io::Result<()>{let artifact=Artifact{version:1,generation:1,id:random().map_err(io::Error::other)?,effect:capture.preview.token.clone(),component:COMPONENT.into(),binding:capture.binding(),phase,evidence,previous:None};let reference=components::retain(&capture.path,&artifact,||guard().map_err(|_|"nativeAutostartReviewRevoked"))?;components::transition(settings,&capture.path,COMPONENT,&capture.preview.token,phase,Some(reference),||guard().map_err(|_|"nativeAutostartReviewRevoked"))})();
    if done.is_err(){return receipt(settings,Registration::Unconfirmed,&capture.preview.token,"O resultado pode ter ocorrido sem recibo confirmado. Consulta o estado e reconcilia; não repitas a operação automaticamente.");}
    match phase{
        Phase::Verified=>receipt(settings,Registration::Verified,&capture.preview.token,if capture.preview.action==Action::SetIndependentRegistration{"Novo registo verificado no recurso aprovado. A operação no recurso antigo continua incerta e conservada; o próximo login não foi verificado."}else{"Ficheiro de registo verificado no recurso aprovado. O funcionamento no próximo login continua por verificar."}),
        Phase::NotDispatched=>receipt(settings,Registration::NotDispatched,&capture.preview.token,"O registo não foi despachado. A preferência pode ter sido guardada; a componente continua suspensa e uma alteração exige nova revisão."),
        _=>receipt(settings,Registration::Unconfirmed,&capture.preview.token,"O registo pode ter ocorrido ou ficado parcial. A componente continua suspensa; reconcilia por observação, sem repetir o efeito."),
    }
}

fn permitted(window:&tauri::WebviewWindow)->bool{window.label()==crate::HUB_LABEL&&window.is_visible().unwrap_or(false)}
fn path()->PathBuf{crate::platform::config_dir().join("settings.json")}
#[tauri::command]
pub(crate)async fn pet_autostart_review_prepare(app:tauri::AppHandle,window:tauri::WebviewWindow,request:Request)->Result<Preview,String>{
    if !permitted(&window){return Err("Abre as definições para rever o registo de arranque.".into());}let frame=settings::review::controller().frame()?;
    tauri::async_runtime::spawn_blocking(move||{use tauri::Manager;let shared=app.state::<crate::Shared>();let path=path();
        let mut archived_effect=None;
        let enabled=match(request.action,request.enabled,request.archived_effect_id){(Action::SetRegistration|Action::SetIndependentRegistration,Some(v),None)=>v,(Action::ReconcileObservation,None,None)=>{let s=shared.settings.lock().map_err(|_|refused())?;let artifact=components::intent_artifact(&s,&path,COMPONENT).map_err(|e|e.to_string())?.ok_or_else(refused)?;let view:View=serde_json::from_value(artifact.evidence.get("nativePlan").cloned().ok_or_else(refused)?).map_err(|_|refused())?;view.enabled},(Action::ReconcileArchivedObservation|Action::ReconcileLegacyObservation,None,Some(id))=>{let s=shared.settings.lock().map_err(|_|refused())?;let(subject,artifact)=if request.action==Action::ReconcileLegacyObservation{components::legacy_subject(&s,&path,&id)}else{components::historical_subject(&s,&path,&id)}.map_err(|e|e.to_string())?;let view:View=serde_json::from_value(artifact.evidence.get("nativePlan").cloned().ok_or_else(refused)?).map_err(|_|refused())?;archived_effect=Some(subject);view.enabled},_=>return Err("O pedido de revisão não corresponde à operação indicada.".into())};
        let plan=crate::pet_autostart::native::native_plan(enabled).map_err(|e|e.to_string())?;
        let preview=super::controller().prepare_review_selected(&frame,&shared.settings,&path,plan,request.action,native_context,archived_effect)?;
        if !frame.current(){let _=super::controller().cancel_review(&frame,Key{token:preview.token.clone(),fingerprint:preview.fingerprint.clone()});return Err(refused());}Ok(preview)
    }).await.map_err(|_|refused())?
}
#[tauri::command]
pub(crate)async fn pet_autostart_review_apply(app:tauri::AppHandle,window:tauri::WebviewWindow,request:Decision)->Result<Receipt,String>{
    if !permitted(&window){return Err("A aprovação exige a janela de definições aberta.".into());}let frame=settings::review::controller().frame()?;
    tauri::async_runtime::spawn_blocking(move||{use tauri::{Manager,Emitter};let shared=app.state::<crate::Shared>();
        let result=super::controller().apply_review(&frame,&shared.settings,request,native_context,crate::pet_data::bootstrap::write_allowed,|s|{let _=app.emit("settings-changed",s);})?;
        let _=app.emit("settings-review-state-changed",&result.status);Ok(result)
    }).await.map_err(|_|"O worker terminou sem resposta confirmada. Consulta o estado; não repitas o efeito.".to_string())?
}
#[tauri::command]
pub(crate)fn pet_autostart_review_cancel(window:tauri::WebviewWindow,request:Key)->Result<(),String>{
    if window.label()!=crate::HUB_LABEL{return Err(refused());}super::controller().cancel_review(&settings::review::controller().frame()?,request)
}
#[tauri::command]
pub(crate)async fn pet_autostart_review_status(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Status,String>{
    if window.label()!=crate::HUB_LABEL{return Err(refused());}
    tauri::async_runtime::spawn_blocking(move||{use tauri::Manager;let shared=app.state::<crate::Shared>();let s=shared.settings.lock().map_err(|_|refused())?;
        let status=settings::io_status(&s);let origin=if status.new_effects_allowed{native_context(&s,&path()).ok()}else{None};let effect=origin.as_ref().and_then(|c|c.slot.clone());let enabled=s.autostart;
        let plan=crate::pet_autostart::native::native_plan(enabled).ok();
        let(history,observed_history)=if status.new_effects_allowed{let h=components::history_for_resource(&s,&path(),COMPONENT,plan.as_ref().map(|p|p.view.resource.as_str()).unwrap_or("")).map_err(|e|e.to_string())?;(h.unresolved,h.observed)}else{(vec![],vec![])};
        let mut legacy_reconcilable=vec![];for old in effect.iter().filter(|e|!e.phase.replaceable()).chain(history.iter()){if components::legacy_slot(&path(),old).map_err(|e|e.to_string())?{legacy_reconcilable.push(old.id.clone());}}
        let registration_matches=plan.as_ref().map(|p|p.already_matches());
        let independent_destination=effect.as_ref().is_some_and(|e|!legacy_reconcilable.contains(&e.id)&&origin.as_ref().and_then(|c|c.profile.as_deref()).zip(plan.as_ref()).is_some_and(|(scope,p)|e.independent_autostart_destination(scope,&p.view.resource)));
        let historical_reconcilable=history.iter().filter(|old|!legacy_reconcilable.contains(&old.id)&&plan.as_ref().is_some_and(|p|p.view.resource==old.binding.resource)).map(|old|old.id.clone()).collect();drop(s);
        let in_flight=super::controller().review.lock().map_err(|_|refused())?.slot.as_ref().is_some_and(|s|s.pending.is_none());
        Ok(Status{schema_version:1,requires_review:true,desired_enabled:enabled,status,effect,registration_matches,independent_destination,unresolved_history:history,observed_history,historical_reconcilable,legacy_reconcilable,in_flight,message:"Consulta passiva: não cria registos, não autentica contas, não inicia modelos e não prova o próximo login.".into()})
    }).await.map_err(|_|refused())?
}

#[cfg(test)]#[path="autostart_review_tests.rs"]mod tests;
