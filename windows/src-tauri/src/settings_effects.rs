//! One weak reference to the actual backend owner, not another settings store
//! or permission ledger. New dispatch never uses a captured turn's old witness
//! as if an unrelated successful preference edit revoked that turn.
use super::Settings;
use serde::Serialize;
use std::sync::{Arc, Mutex, OnceLock, TryLockError, Weak};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum IoPhase { Ready, Writing, Changed, ReadUnavailable, ReviewRequired, PublicationUnconfirmed, Unbound, Unavailable, PlatformNotTracked }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IoStatus {
    pub schema_version: u32, pub phase: IoPhase, pub new_effects_allowed: bool, pub message: &'static str, #[serde(skip_serializing_if="Vec::is_empty")]pub suspended_operations:Vec<String>,
}
impl IoStatus {
    fn from_phase(phase: IoPhase) -> Self {
        let message = match phase {
            IoPhase::Ready => "",
            IoPhase::Writing => "A gravação das preferências está em curso. Não foi iniciado outro efeito.",
            IoPhase::Changed => "As preferências mudaram fora do backend. Novos pedidos aguardam revisão; o histórico e Stop continuam disponíveis.",
            IoPhase::ReadUnavailable | IoPhase::Unbound => "A origem das preferências não foi confirmada. Não será escolhido outro provider ou modelo por omissão.",
            IoPhase::ReviewRequired => "As preferências foram conservadas, mas os valores recuperados ainda precisam de revisão antes de novos pedidos.",
            IoPhase::PublicationUnconfirmed => "A gravação pode ter ocorrido sem confirmação. Não repitas efeitos; é necessária reconciliação.",
            IoPhase::Unavailable => "O estado das preferências está indisponível. Novos efeitos não foram autorizados.",
            IoPhase::PlatformNotTracked => "A verificação da origem das preferências ainda não está implementada nesta plataforma.",
        };
        Self { schema_version: 1, phase, new_effects_allowed: phase == IoPhase::Ready, message,suspended_operations:vec![] }
    }
    pub(crate) fn check(&self) -> Result<(), String> {
        if self.new_effects_allowed { Ok(()) } else { Err(self.message.into()) }
    }
    fn operation(&self,name:&str)->Result<(),String>{self.check()?;
        if self.suspended_operations.iter().any(|s|s==name){return Err(format!("A componente {name} conserva a preferência desejada e aguarda reativação própria após revisão dos dados."));}Ok(())
    }
}
pub(crate) fn io_status(settings: &Settings) -> IoStatus {
    #[cfg(target_os = "linux")] { let mut status=IoStatus::from_phase(super::revision::phase(settings));status.suspended_operations=super::revision::suspended(settings);if status.phase==IoPhase::Ready&&!status.suspended_operations.is_empty(){status.message="Os dados foram revistos. Novos registos ou efeitos das componentes indicadas exigem revisão própria; consulta o estado nativo para saber o que está atualmente ativo.";}status }
    #[cfg(not(target_os = "linux"))] { let _ = settings; IoStatus::from_phase(IoPhase::PlatformNotTracked) }
}

/// An already captured native origin, not permission or another settings store.
/// Reuse the same owner epoch + durable file/journal witnesses as component
/// executions. Ordinary readers of the presentation mutex cannot revoke it.
pub(crate) struct CapturedOrigin {
    #[cfg(target_os="linux")]source:super::revision::Captured,
}
pub(crate) fn capture_origin(settings:&Settings)->CapturedOrigin{
    #[cfg(not(target_os="linux"))]let _=settings;
    CapturedOrigin{#[cfg(target_os="linux")]source:super::revision::capture(settings)}
}
impl CapturedOrigin {
    pub(crate) fn operation(&self,name:&str)->Result<(),String>{
        #[cfg(target_os="linux")]{
            let mut status=IoStatus::from_phase(self.source.phase());status.suspended_operations=self.source.suspended();status.operation(name)?;
            if !self.source.owner_current(){return Err("O dono da configuração capturada mudou antes do efeito.".into());}Ok(())
        }
        #[cfg(not(target_os="linux"))]{let _=name;Ok(())}
    }
}

#[derive(Clone)]
pub(crate) struct EffectProbe(Weak<Mutex<Settings>>);
pub(crate) fn effect_probe(current: &Arc<Mutex<Settings>>) -> EffectProbe { EffectProbe(Arc::downgrade(current)) }
impl EffectProbe {
    pub(crate) fn status(&self) -> IoStatus {
        let Some(current) = self.0.upgrade() else { return IoStatus::from_phase(IoPhase::Unavailable); };
        let status = match current.try_lock() {
            Ok(current) => io_status(&current),
            Err(TryLockError::WouldBlock) => IoStatus::from_phase(IoPhase::Writing),
            Err(TryLockError::Poisoned(_)) => IoStatus::from_phase(IoPhase::Unavailable),
        }; status
    }
    pub(crate) fn check(&self) -> Result<(), String> { self.status().check() }
    pub(crate)fn operation(&self,name:&str)->Result<(),String>{
        self.status().operation(name)
    }
}
fn source() -> &'static OnceLock<EffectProbe> { static SOURCE: OnceLock<EffectProbe> = OnceLock::new(); &SOURCE }
pub(crate) fn install_effect_source(current: &Arc<Mutex<Settings>>) -> std::io::Result<()> {
    source().set(effect_probe(current)).map_err(|_| std::io::Error::other("A origem de preferências deste backend já foi registada."))
}
pub(crate) fn effect_allowed() -> Result<(), String> {
    #[cfg(target_os = "linux")] {
        // Isolated fixtures have no Tauri owner. Native QA compiles the actual
        // non-test owner and can never take this fixture-only branch.
        #[cfg(test)] if source().get().is_none() { return Ok(()); }
        source().get().ok_or_else(|| IoStatus::from_phase(IoPhase::Unavailable).message.to_string())?.check()
    }
    #[cfg(not(target_os = "linux"))] { Ok(()) } // Legacy behavior; no Linux-origin proof claimed.
}
pub(crate)fn optional_effect_allowed(name:&str)->Result<(),String>{
 #[cfg(target_os="linux")]{
  #[cfg(test)]if source().get().is_none(){return Ok(());}
  source().get().ok_or_else(||IoStatus::from_phase(IoPhase::Unavailable).message.to_string())?.operation(name)
 }
 #[cfg(not(target_os="linux"))]{let _=name;Ok(())}
}
pub(crate)fn component_revalidated(name:&str)->bool{
 #[cfg(target_os="linux")]{let Some(probe)=source().get()else{return false;};let Some(owner)=probe.0.upgrade()else{return false;};let Ok(settings)=owner.try_lock()else{return false;};super::revision::component_revalidated(&settings,name)}
 #[cfg(not(target_os="linux"))]{let _=name;false}
}
pub(crate) fn effect_status() -> IoStatus {
    source().get().map(EffectProbe::status).unwrap_or_else(|| IoStatus::from_phase(IoPhase::Unavailable))
}

#[cfg(all(test,target_os="linux"))]
#[path="settings_effects_tests.rs"]
pub(crate) mod tests;
