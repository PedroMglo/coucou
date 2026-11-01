//! Selected speech destinations; no implicit engine or system-voice fallback.
use std::sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc};
use tauri::{AppHandle, Manager};

pub(super) type Current=Arc<dyn Fn()->bool+Send+Sync>;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Plan {
    pub output:String, pub gender:String,
    python:std::path::PathBuf,
    local:std::collections::BTreeMap<String,String>,
}
impl Plan {
    pub(super) fn silent()->Self{Self{output:"off".into(),gender:"female".into(),python:Default::default(),local:Default::default()}}
    pub(super) fn prepare(settings:&crate::settings::Settings,prefs:&crate::flow::store::Prefs)->Result<Self,String>{
        if !matches!(prefs.voice_output.as_str(),"off"|"system"|"edge"|"local"|"elevenlabs")||!matches!(prefs.voice_gender.as_str(),"female"|"male"){
            return Err("A seleção de voz não é suportada. Não foi escolhido outro serviço.".into());
        }
        let python=settings.edge_voice_python.strip_prefix("~/").map(|p|crate::platform::home_dir().join(p)).unwrap_or_else(||settings.edge_voice_python.clone().into());
        Ok(Self{output:prefs.voice_output.clone(),gender:prefs.voice_gender.clone(),python,local:settings.local_voice_dirs.clone()})
    }
    pub(super) fn matches(&self,prefs:&crate::flow::store::Prefs)->bool{self.output==prefs.voice_output&&self.gender==prefs.voice_gender}
    pub(super) fn system_allowed(&self)->bool{self.output=="system"}
    fn local_directory(&self,provider:Option<&str>)->Option<&str>{
        provider.and_then(|p|self.local.get(p)).filter(|p|!p.trim().is_empty())
            .or_else(||self.local.get("default").filter(|p|!p.trim().is_empty())).map(String::as_str)
    }
}

/// Preview has its own captured output, without requiring ASR or microphone.
/// This checks provenance and optional-component suspension, never grants.
pub(super) fn preview(app:&AppHandle)->Result<(Plan,Current),String>{
    crate::settings::optional::check(crate::settings::optional::Operation::Voice)?;
    let (settings,origin)={let shared=app.state::<crate::Shared>();let settings=shared.settings.lock().map_err(|_|"As preferências da voz estão indisponíveis.")?;
        (settings.clone(),crate::settings::capture_origin(&settings))};
    let flow=crate::flow::get(app).ok_or("As preferências da voz estão indisponíveis.")?;
    let plan=Plan::prepare(&settings,&flow.store.lock().map_err(|_|"As preferências da voz estão ocupadas.")?.prefs)?;
    let expected=plan.clone();let owner=Arc::downgrade(&flow);
    let diagnosed=AtomicBool::new(false);
    let current:Current=Arc::new(move||{
        let failure=||->Option<String>{
            if crate::settings::optional::check_origin(crate::settings::optional::Operation::Voice,&origin).is_err(){return Some("captured optional origin no longer current".into());}
            let Some(flow)=owner.upgrade()else{return Some("preference owner missing".into());};
            let Ok(store)=flow.store.try_lock()else{return Some("preference reader busy".into());};
            if !expected.matches(&store.prefs){return Some("output preference changed".into());}None
        };
        if let Some(reason)=failure(){if !diagnosed.swap(true,Ordering::AcqRel){crate::log::line(format!("voice preview retired: {reason}"));}false}else{true}
    });
    if !current(){return Err("A seleção de voz mudou antes do preview.".into());}Ok((plan,current))
}

pub(super) fn speak_streaming(app:&AppHandle,plan:&Plan,text:&str,current:Current)->Option<bool>{
    #[cfg(target_os="linux")]{
        if !current(){return Some(false);}
        let began=std::time::Instant::now();
        let result=crate::speech::edge_tts::speak(&plan.python,text,||current(),
            ||crate::log::line(format!("voice: streaming audio started after {} ms",began.elapsed().as_millis())));
        if result==Some(false)&&current(){crate::flow::pill::announce(app,"voice","#FFB84D","Não foi possível reproduzir a voz. A resposta continua disponível em texto.".into(),None,7);}
        result
    }
    #[cfg(not(target_os="linux"))]{let _=(app,plan,text,current);None}
}

pub(super) fn generate(app:&AppHandle,plan:&Plan,provider:Option<&str>,text:&str,current:Current)->Option<String>{
    if !current(){return None;}
    if plan.output=="edge"{
        let result=crate::speech::edge_tts::generate(&plan.python,text,||current());
        if result.is_none()&&current(){crate::flow::pill::announce(app,"voice","#FFB84D",crate::i18n::t("Raquel is unavailable. Check the internet connection and voice runtime. The reply remains as text."),None,7);}
        return if current(){result.map(|b|format!("data:audio/mpeg;base64,{}",super::base64(&b)))}else{None};
    }
    if plan.output!="local"{return None;}
    let result=plan.local_directory(provider).and_then(|directory|{
        let guard=current.clone();crate::speech::local_tts::generate(std::path::Path::new(directory),text,move||guard())
    });
    if result.is_none()&&current(){crate::flow::pill::announce(app,"voice","#FFB84D",crate::i18n::t("Local voice unavailable. Check the pt-PT model folder. The reply remains as text."),None,7);}
    if current(){result.map(|b|format!("data:audio/wav;base64,{}",super::base64(&b)))}else{None}
}

// Cancellation revokes output, never renews an old native worker. The shared
// helper supervisor owns its resource and child family until collected.
static NATIVE_EPOCH: AtomicU64 = AtomicU64::new(0);
pub fn epoch()->u64{NATIVE_EPOCH.load(Ordering::Acquire)}
pub fn cancel_native(){let _=NATIVE_EPOCH.fetch_update(Ordering::AcqRel,Ordering::Acquire,|n|Some(n.saturating_add(1)));}

#[cfg(test)]
mod plan_tests{
    use super::*;
    #[test]
    fn playback_plan_keeps_runtime_and_per_pet_local_voice_without_later_reread(){
        let mut settings=crate::settings::Settings::default();settings.edge_voice_python="/fixture/original-python".into();settings.local_voice_dirs.insert("codex".into(),"/fixture/codex-voice".into());settings.local_voice_dirs.insert("default".into(),"/fixture/explicit-default".into());
        let mut prefs=crate::flow::store::Prefs::default();prefs.voice_output="local".into();
        let plan=Plan::prepare(&settings,&prefs).unwrap();settings.edge_voice_python="/changed/python".into();settings.local_voice_dirs.insert("codex".into(),"/changed/voice".into());prefs.voice_output="elevenlabs".into();
        assert_eq!(plan.output,"local");assert_eq!(plan.python,std::path::PathBuf::from("/fixture/original-python"));assert_eq!(plan.local_directory(Some("codex")),Some("/fixture/codex-voice"));assert_eq!(plan.local_directory(Some("ollama")),Some("/fixture/explicit-default"));assert!(!plan.matches(&prefs));
    }
    #[test]
    fn only_an_explicit_system_selection_may_request_system_speech(){
        let settings=crate::settings::Settings::default();let mut prefs=crate::flow::store::Prefs::default();
        for output in ["off","system","edge","local","elevenlabs"]{
            prefs.voice_output=output.into();let plan=Plan::prepare(&settings,&prefs).unwrap();assert_eq!(plan.system_allowed(),output=="system");
        }
        prefs.voice_output="unknown".into();assert!(Plan::prepare(&settings,&prefs).is_err());prefs.voice_output="system".into();prefs.voice_gender="unknown".into();assert!(Plan::prepare(&settings,&prefs).is_err());
    }
    #[test]
    fn a_local_voice_requires_an_explicit_pet_or_default_directory(){
        let settings=crate::settings::Settings::default();let plan=Plan::prepare(&settings,&crate::flow::store::Prefs::default()).unwrap();assert_eq!(plan.local_directory(Some("codex")),None);assert_eq!(plan.local_directory(None),None);
    }
}
