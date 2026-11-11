//! PET-01: effective identity/capability snapshots; never refresh a provider here.
mod types;
mod cache;
mod native;
pub use native::TargetDescriptor;
pub(crate) use native::Ticket as NativeTicket;
pub(crate) use cache::Ticket;
pub use types::*;
use cache::{Cache,Key,Evidence,Observation};
use crate::{pet_config::{Configuration,Pet},settings::Settings};
use std::{collections::{BTreeMap,BTreeSet},sync::{Mutex,OnceLock}};
use ring::rand::{SecureRandom,SystemRandom};
use tauri::{AppHandle,Manager};
const ERROR:&str="O registo de capacidades está indisponível. As configurações foram preservadas.";
struct Revision{public:Vec<u8>,private:Option<Key>,configuration:Vec<u8>,number:u64}
struct Registry{instance:String,generation:u64,revisions:BTreeMap<String,Revision>,cache:Cache}
impl Registry{
    fn new()->Result<Self,String>{let mut bytes=[0u8;32];SystemRandom::new().fill(&mut bytes).map_err(|_|ERROR)?;Ok(Self{instance:bytes.iter().map(|b|format!("{b:02x}")).collect(),generation:0,revisions:BTreeMap::new(),cache:Cache::default()})}
    fn snapshot(&mut self,settings:&Settings,configuration:Option<&Configuration>,healthy:bool,busy:&BTreeSet<String>,now:u64)->Result<Snapshot,String>{
        self.snapshot_with_io(settings,configuration,healthy,busy,now,true)
    }
    fn admission_snapshot(&mut self,settings:&Settings,configuration:Option<&Configuration>,healthy:bool,now:u64,preferences_ready:bool)->Result<Snapshot,String>{
        self.snapshot_with_io(settings,configuration,healthy,&BTreeSet::new(),now,preferences_ready)
    }
    fn snapshot_with_io(&mut self,settings:&Settings,configuration:Option<&Configuration>,healthy:bool,busy:&BTreeSet<String>,now:u64,preferences_ready:bool)->Result<Snapshot,String>{
        if now==0||now>cache::MAX_DATE{return Err(ERROR.into());}let defaults=crate::pet_config::defaults();let config=configuration.unwrap_or(&defaults);
        if !matches!(config.version,1|2)||config.pets.len()>64||config.pets.keys().ne(defaults.pets.keys())||config.pets.iter().any(|(id,p)|!crate::pet_config::blueprint(id).is_some_and(|b|b.accepts_provider(&p.provider))||p.name.is_empty()||p.name.chars().count()>40||p.name.chars().any(char::is_control)){return Err(ERROR.into());}
        crate::pet_config::validate(config).map_err(|_|ERROR.to_owned())?;
        let mut pets=vec![];let mut changed=false;
        for(id,p)in &config.pets{
            let configured=configuration.is_some();let kind=kind(id);let binding=binding(settings,id,p,configured);let key=key(settings,&binding,p,None);
            let lifecycle=if crate::pet_config::retired(id){Lifecycle::Retired}else if id==&settings.main_pill||settings.active_integrations.contains(id){Lifecycle::Active}else{Lifecycle::Inactive};
            let is_busy=binding.provider.as_ref().is_some_and(|p|busy.contains(p));
            let mut d=Descriptor{schema_version:1,pet_id:id.clone(),kind,name:p.name.clone(),lifecycle,binding,revision:String::new(),observed_at:None,configuration_available:configured,capabilities:vec![],concurrency:Concurrency{scope:None,max_active:None}};
            if d.binding.adapter.is_some()&&d.binding.adapter!=Some("spotify"){d.concurrency=Concurrency{scope:Some("provider"),max_active:Some(1)};}
            let catalog=key.as_ref().map(|k|self.cache.get(k,now));
            let observed_model=d.binding.model.as_ref().and_then(|m|key.clone().map(|mut k|{k.model=Some(m.clone());self.cache.get(&k,now)}));
            if let Some(model)=&d.binding.model{if let Some(key)=&key{let mut selected=key.clone();selected.model=Some(model.clone());d.binding.engine_binding=self.cache.engine_binding(&selected,now);}}
            build(&mut d,p,configured,healthy,is_busy,catalog.as_ref(),observed_model.as_ref());
            if !preferences_ready{gate_preferences(&mut d);}
            d.observed_at=d.capabilities.iter().filter_map(|c|c.observed_at).max();
            let public=serde_json::to_vec(&d).map_err(|_|ERROR)?;
            let configuration=serde_json::to_vec(&(p,if p.inference.is_some(){None}else{d.binding.provider.as_ref().and_then(|p|settings.chat_models.get(p))})).map_err(|_|ERROR)?;
            let number=if let Some(old)=self.revisions.get(id){if old.public==public&&old.private==key&&old.configuration==configuration{old.number}else{changed=true;old.number.checked_add(1).ok_or(ERROR)?}}else{changed=true;1};
            self.revisions.insert(id.clone(),Revision{public,private:key,configuration,number});d.revision=format!("{}:{number}",self.instance);pets.push(d);
        }
        if changed{self.generation=self.generation.checked_add(1).ok_or(ERROR)?;}
        Ok(Snapshot{schema_version:1,revision:format!("{}:{}",self.instance,self.generation),observed_at:now,configuration_available:configuration.is_some(),pets})
    }
}
fn registry()->Result<&'static Mutex<Registry>,String>{static REGISTRY:OnceLock<Result<Mutex<Registry>,String>>=OnceLock::new();REGISTRY.get_or_init(||Registry::new().map(Mutex::new)).as_ref().map_err(Clone::clone)}
fn preservation_capability(id:&str)->bool{matches!(id,
    "history.read"|"conversation.cancel"|"conversation.archive"|"conversation.restore"|
    "session.discover"|"session.history"|"session.interrupt"|"models.tools"|"models.reasoning"|
    "usage.observedAttempts"|"goals.nativePlanRevision"|"goals.inspect"|"goals.cancel"|"goals.retirementReview"|"memory.manage"|
    "documents.revoke"|"documents.retention")}
fn gate_preferences(pet:&mut Descriptor){
    for cap in &mut pet.capabilities{if cap.support!=Support::Unsupported&&!preservation_capability(cap.id){
        cap.availability=Availability::Unavailable;cap.reason=Some(Reason::SettingsUnconfirmed);
        if !cap.source.contains(&Source::RuntimeObservation){cap.source.push(Source::RuntimeObservation);}
    }}
}
fn kind(id:&str)->Kind{
    use crate::pet_config::RuntimeKind;
    match crate::pet_config::blueprint(id).map(|p|p.runtime.kind){Some(RuntimeKind::Llm)=>Kind::Llm,Some(RuntimeKind::Service)=>Kind::Service,_=>Kind::Observer}
}
fn binding(settings:&Settings,id:&str,p:&Pet,configured:bool)->Binding{
    if !configured{return Binding{engine_binding:None,adapter:None,provider:None,model:None,profile:None};}
    let declared=crate::pet_config::blueprint(id);
    if let Some(choice)=&p.inference{return Binding{engine_binding:None,adapter:Some(choice.backend.key()),provider:declared.and_then(|b|b.routing.task_provider.clone()),model:Some(choice.model.clone()),profile:Some("binding".into())};}
    let adapter=declared.and_then(|b|b.configured_adapter(&p.provider)).map(|a|a.key());
    if adapter==Some("spotify"){return Binding{engine_binding:None,adapter,provider:Some(id.into()),model:None,profile:None};}
    let provider=if crate::pet_config::service(id){id}else{p.provider.as_str()};

    if adapter.is_none(){return Binding{engine_binding:None,adapter:None,provider:None,model:None,profile:None};}
    let selected=settings.chat_models.get(provider).map(String::as_str).unwrap_or("@default");
    let mode=match selected{"@default"=>p.mode.as_str(),"@fast"=>"fast","@reasoning"=>"reasoning","@auto"=>"auto",_=>"manual"};
    let candidate=match mode{"fast"=>p.fast.model.as_str(),"reasoning"=>p.reasoning.model.as_str(),"manual"=>selected,_=>""};
    Binding{engine_binding:None,adapter,provider:Some(provider.into()),model:cache::model_id(candidate).then(||candidate.into()),profile:Some(mode.into())}
}
fn key(settings:&Settings,b:&Binding,p:&Pet,model:Option<String>)->Option<Key>{
    if let Some(choice)=&p.inference{
        let endpoint=match choice.backend{
            crate::pet_inference::Backend::Ollama=>cache::endpoint(&choice.endpoint)?,
            crate::pet_inference::Backend::Lmstudio=>crate::local_chat::lmstudio::endpoint_key_for(&choice.endpoint,match choice.credential{
                crate::pet_inference::CredentialReference::None=>crate::local_chat::lmstudio::MetadataAuth::Anonymous,
                crate::pet_inference::CredentialReference::ConfiguredLmstudioEndpoint=>crate::local_chat::lmstudio::MetadataAuth::Configured(crate::local_chat::lmstudio_key_generation()),
            })?,
        };return Some(Key{provider:choice.backend.key().into(),endpoint,model});
    }
    let provider=b.provider.as_ref()?;let endpoint=match b.adapter?{"ollama"=>cache::endpoint(&settings.ollama_url)?,"lmstudio"=>crate::local_chat::lmstudio::endpoint_key(&settings.lmstudio_url,crate::local_chat::lmstudio_key_generation())?,"spotify"=>return None,_=>String::new()};Some(Key{provider:provider.clone(),endpoint,model})
}
fn observed(c:&mut Capability,o:Option<&Observation>){if let Some(o)=o{c.observed_at=o.at;c.expires_at=o.expires;if o.at.is_some(){c.source.push(Source::ProviderCatalog);}}}
fn gate(c:&mut Capability,d:&Descriptor,configured:bool,healthy:bool,busy:bool,permission:bool,target:bool){
    if c.support==Support::Unsupported{return;}
    let state=if !configured{Some((Availability::Unknown,Reason::NotConfigured))}
        else if d.lifecycle==Lifecycle::Retired{Some((Availability::Unavailable,Reason::RetiredPet))}
        else if d.lifecycle==Lifecycle::Inactive{Some((Availability::Unavailable,Reason::InactivePet))}
        else if !healthy{Some((Availability::Unavailable,Reason::LibraryUnavailable))}
        else if !permission{Some((Availability::Unavailable,Reason::PermissionDenied))}
        else if d.binding.adapter.is_none(){Some((Availability::Unavailable,Reason::NotConfigured))}
        else if busy{Some((Availability::Unavailable,Reason::Busy))}
        else if target{Some((Availability::Unknown,Reason::TargetRequired))}else{None};
    if let Some((available,reason))=state{c.availability=available;c.reason=Some(reason);}
}
fn base(id:&'static str)->Capability{let mut c=Capability::unsupported(id);c.support=Support::Supported;c.availability=Availability::Unknown;c.reason=Some(Reason::Unknown);c}
fn build(d:&mut Descriptor,p:&Pet,configured:bool,healthy:bool,busy:bool,catalog:Option<&Observation>,model:Option<&Observation>){
    let adapter=d.binding.adapter;let llm=matches!(adapter,Some("codex"|"copilot"|"antigravity"|"ollama"|"lmstudio"))||(!configured&&crate::pet_config::blueprint(&d.pet_id).and_then(|p|p.runtime.adapter).is_some_and(|a|a!=crate::pet_config::Adapter::Spotify));let permissions=p.permissions.clone().unwrap_or_default();
    let mut caps=vec![];
    let history_owner=crate::pet_config::blueprint(&d.pet_id).is_some_and(|p|p.history_owner());
    let mut history=if history_owner{base("history.read")}else{Capability::unsupported("history.read")};history.callers=vec![Caller::NativeUser];if history_owner{history.availability=if healthy{Availability::Ready}else{Availability::Unavailable};history.reason=(!healthy).then_some(Reason::LibraryUnavailable);}caps.push(history);
    for id in ["conversation.respond","conversation.stream","conversation.cancel","conversation.archive","conversation.restore","models.list","models.select"]{
        let mut c=if llm{base(id)}else{Capability::unsupported(id)};if !llm&&d.kind==Kind::Llm{c.support=Support::Unknown;c.availability=Availability::Unknown;c.reason=Some(Reason::NotConfigured);}
        if matches!(id,"conversation.respond"|"conversation.stream"){c.callers=vec![Caller::NativeUser,Caller::TaskRuntime];c.model_binding=d.binding.model.clone();
            if matches!(adapter,Some("ollama"|"lmstudio")){
                observed(&mut c,model);match model.and_then(|o|o.evidence.as_ref()){
                    Some(Evidence::Model{completion:Some(true),remote:false,..})=>{c.availability=Availability::Ready;c.reason=None;},
                    Some(Evidence::Model{completion:Some(false),..})=>{c.support=Support::Unsupported;c.availability=Availability::Unavailable;c.reason=Some(Reason::ModelUnsupported);},
                    Some(Evidence::Model{remote:true,..})=>{c.availability=Availability::Unavailable;c.reason=Some(Reason::PolicyDenied);},
                    Some(Evidence::Unavailable)=>c.reason=Some(Reason::ProviderUnavailable),
                    _=>c.reason=Some(if d.binding.model.is_none(){Reason::ModelUnresolved}else if model.is_some_and(|o|o.stale){Reason::StaleObservation}else{Reason::Unknown}),
                }
            }else if d.binding.model.is_none(){c.reason=Some(Reason::ModelUnresolved);}
        }
        if id=="models.list"{observed(&mut c,catalog);if catalog.is_some_and(|o|matches!(o.evidence,Some(Evidence::Catalog|Evidence::LmStudio(_)))){c.availability=Availability::Ready;c.reason=None;}else if catalog.is_some_and(|o|o.stale){c.reason=Some(Reason::StaleObservation);}}
        if id=="models.select"&&adapter!=Some("ollama"){c.limitations.push("nativeSelectionNeedsSession");}
        if matches!(id,"conversation.archive"|"conversation.restore"){c.decision=Decision::NativeApproval;}
        gate(&mut c,d,configured,healthy,busy,true,matches!(id,"conversation.cancel"|"conversation.archive"|"conversation.restore"|"models.select"));caps.push(c);
    }
    for id in ["session.discover","session.history","session.send","session.interrupt","session.approve","session.continueBackground"]{
        let native=matches!(adapter,Some("codex"|"copilot"|"antigravity"));let mut c=if native{base(id)}else{Capability::unsupported(id)};
        c.decision=Decision::NativeApproval;if adapter==Some("antigravity")&&matches!(id,"session.approve"|"session.continueBackground"){c.support=Support::Unknown;c.reason=Some(Reason::NativeApprovalUnavailable);c.limitations.push("interactiveContinuationUnverified");}
        gate(&mut c,d,configured,healthy,busy,true,true);caps.push(c);
    }
    let mut command=if adapter==Some("ollama"){base("tools.exactCommand")}else{Capability::unsupported("tools.exactCommand")};command.decision=Decision::ExactCommandApproval;command.callers=vec![Caller::NativeUser,Caller::TaskRuntime,Caller::ProviderModel];command.model_binding=d.binding.model.clone();
    if adapter==Some("ollama"){
        observed(&mut command,model);match model.and_then(|o|o.evidence.as_ref()){
            Some(Evidence::Model{completion:Some(true),tools:Some(true),remote:false,..})=>{command.availability=Availability::Ready;command.reason=None;},
            Some(Evidence::Model{completion:Some(false),..}|Evidence::Model{tools:Some(false),..})=>{command.support=Support::Unsupported;command.availability=Availability::Unavailable;command.reason=Some(Reason::ModelUnsupported);},
            Some(Evidence::Model{remote:true,..})=>{command.availability=Availability::Unavailable;command.reason=Some(Reason::PolicyDenied);},
            Some(Evidence::Unavailable)=>{command.support=Support::Unknown;command.reason=Some(Reason::ProviderUnavailable);},
            _=>{command.support=Support::Unknown;command.reason=Some(if d.binding.model.is_none(){Reason::ModelUnresolved}else if model.is_some_and(|o|o.stale){Reason::StaleObservation}else{Reason::Unknown});},
        }
        let policy=p.local_execution.clone().unwrap_or_default();command.limits.max_calls=Some(policy.max_calls);command.limits.timeout_ms=Some(policy.task_timeout_ms);command.limits.max_bytes=Some(policy.max_output_bytes as u64);command.limitations.push("desktopUserNotFilesystemSandbox");
        if policy.mode!=crate::pet_config::LocalExecutionMode::Ask&&command.support!=Support::Unsupported{command.availability=Availability::Unavailable;command.reason=Some(Reason::PolicyDenied);}
    }if !cfg!(target_os="linux")&&adapter==Some("ollama"){command.support=Support::Unsupported;command.availability=Availability::Unavailable;command.reason=Some(Reason::UnsupportedPlatform);}gate(&mut command,d,configured,healthy,busy,true,false);caps.push(command);
    for id in ["models.tools","models.reasoning"]{
        let mut c=base(id);c.support=Support::Unknown;c.model_binding=d.binding.model.clone();observed(&mut c,model);
        if adapter==Some("ollama"){
            if let Some(Evidence::Model{tools,thinking,remote,..})=model.and_then(|o|o.evidence.as_ref()){
                let value=if id=="models.tools"{tools}else{thinking};if let Some(value)=value{c.support=if *value{Support::Supported}else{Support::Unsupported};c.availability=if *value{Availability::Ready}else{Availability::Unavailable};c.reason=(!*value).then_some(Reason::ModelUnsupported);}
                if *remote&&c.support!=Support::Unsupported{c.availability=Availability::Unavailable;c.reason=Some(Reason::PolicyDenied);}
            }
        }
        if d.binding.model.is_none(){c.reason=Some(Reason::ModelUnresolved);}else if model.is_some_and(|o|o.stale){c.reason=Some(Reason::StaleObservation);}
        if !llm&&d.kind!=Kind::Llm{c=Capability::unsupported(id);}
        gate(&mut c,d,configured,healthy,busy,true,false);caps.push(c);
    }
    let mut native=if matches!(adapter,Some("codex"|"copilot"|"antigravity")){base("tools.native")}else{Capability::unsupported("tools.native")};native.decision=Decision::NativeApproval;native.callers=vec![Caller::NativeUser,Caller::TaskRuntime,Caller::ProviderModel];gate(&mut native,d,configured,healthy,busy,true,true);caps.push(native);
    for(id,file,folder)in [("documents.text","note.txt",false),("documents.folder","folder",true),("documents.pdf","note.pdf",false),("documents.image","image.png",false)]{
        let supported=d.binding.provider.as_deref().is_some_and(|p|crate::pet_documents::policy::can_analyze(p,file,folder));let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.decision=Decision::ScopedGrant;c.callers=vec![Caller::NativeUser,Caller::TaskRuntime,Caller::ProviderModel];c.limits.max_items=Some(32);c.limits.max_bytes=Some(100*1024*1024);if id=="documents.pdf"&&adapter==Some("ollama"){c.limitations.push("textLayerOnlyNoOcr");}
        if id=="documents.image"&&supported{c.support=Support::Unknown;c.reason=Some(Reason::ModelUnresolved);c.limitations.push("nativeModelFormatNeedsVerification");}gate(&mut c,d,configured,healthy,busy,permissions.documents,true);caps.push(c);
    }
    for id in ["documents.multi","documents.renew","documents.retention","sources.calendar"]{
        let mut c=if llm{base(id)}else{Capability::unsupported(id)};c.decision=Decision::ScopedGrant;c.limitations.push("originalOwnerAndResourceRequired");
        if matches!(id,"documents.renew"|"documents.retention"){c.callers=vec![Caller::NativeUser];c.decision=Decision::NativeApproval;}
        else{c.callers=vec![Caller::NativeUser,Caller::TaskRuntime,Caller::ProviderModel];}
        if !cfg!(target_os="linux")&&matches!(id,"documents.renew"|"documents.retention"){c.support=Support::Unsupported;c.availability=Availability::Unavailable;c.reason=Some(Reason::UnsupportedPlatform);}gate(&mut c,d,configured,healthy,busy,id=="documents.retention"||permissions.documents,true);caps.push(c);
    }
    // Enrollment is native user metadata management, independent of inference.
    // Calendar query tools still require their own exact captured sources.
    {let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&!crate::pet_config::retired(&d.pet_id);
        let mut c=if supported{base("sources.calendar.manage")}else{Capability::unsupported("sources.calendar.manage")};
        c.callers=vec![Caller::NativeUser];c.decision=Decision::NativeApproval;c.model_binding=None;c.limits.max_items=Some(32);c.limits.max_bytes=Some(256*1024);
        c.limitations.extend(["metadataOnlyNoFileOrModelIo","exactPetConversationAndRegistryRevision","explicitPersistentReadConsentForAddedMembership","source2ExplicitCollectionNoLegacyDowngrade","noTaskRuleModelOrAccountAuthority","querySharesExistingThirtyThreeSourcesEightCalls128KiBLimits"]);
        if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else if d.lifecycle==Lifecycle::Inactive{Reason::InactivePet}else{Reason::TargetRequired});}caps.push(c);
    }
    {let supported=cfg!(target_os="linux")&&matches!(adapter,Some("ollama"|"lmstudio"))&&d.kind!=Kind::Observer&&!crate::pet_config::retired(&d.pet_id);
        let mut c=if supported{base("sources.calendar.goalContext")}else{Capability::unsupported("sources.calendar.goalContext")};
        c.callers=vec![Caller::NativeUser,Caller::TaskRuntime];c.decision=Decision::ScopedGrant;c.model_binding=d.binding.model.clone();c.limits.max_items=Some(8);c.limits.max_bytes=Some(64*1024);
        c.limitations.extend(["exactSourceIdsRevisionsAndExplicitWindow","sourceConversationIsOriginNotAutonomousConsent","dedicatedGoalContextWithoutTranscriptCopy","originalBoundedGoalDataConsentAndDestinationRequired","oneFiniteLocalResponseNoToolsOrForwarding","sourceChangeAndRevocationInvalidateCapture","notProviderModelTool"]);
        gate(&mut c,d,configured,healthy,false,permissions.documents,true);
        if supported&&configured&&healthy&&permissions.documents&&d.lifecycle==Lifecycle::Active{c.availability=Availability::Unknown;c.reason=Some(Reason::TargetRequired);}caps.push(c);
    }
    for id in ["tasks.resume","tasks.stop"]{let mut c=if llm{base(id)}else{Capability::unsupported(id)};c.decision=Decision::NativeApproval;gate(&mut c,d,configured,healthy,false,true,true);caps.push(c);}
    for id in ["voice.dictate","voice.respond","voice.interrupt","voice.decisions","desktop.observe","desktop.control","tools.mcp"]{
        let implemented=if id.starts_with("voice."){crate::voice::admission::implemented_for_pet(&d.pet_id)}else{llm};
        let mut c=if implemented{base(id)}else{Capability::unsupported(id)};if implemented&&matches!(id,"voice.decisions"|"desktop.observe"|"desktop.control"|"tools.mcp"){c.support=Support::Unknown;}
        if id=="tools.mcp"&&matches!(adapter,Some("ollama"|"lmstudio")){c=Capability::unsupported(id);}
        c.decision=Decision::NativeApproval;gate(&mut c,d,configured,healthy,false,true,true);caps.push(c);
    }
    let native_audio=crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|b.runtime.adapter==Some(crate::pet_config::Adapter::Spotify));
    for id in ["service.musicControl","service.audioControl"]{let mut c=if native_audio{base(id)}else{Capability::unsupported(id)};c.decision=Decision::ScopedGrant;gate(&mut c,d,configured,healthy,false,permissions.music,true);c.limitations.push("notSystemPluginGrant");caps.push(c);}
    let mut usage=if matches!(adapter,Some("ollama"|"lmstudio")){base("usage.observedAttempts")}else{Capability::unsupported("usage.observedAttempts")};usage.limitations.push("observedAttemptsOnlyUnknownIsNotZero");gate(&mut usage,d,configured,healthy,false,true,false);caps.push(usage);
    for id in ["usage.account","usage.quota","usage.resourceState"]{let supported=match id{"usage.account"=>d.binding.provider.as_deref()==Some("codex"),"usage.quota"=>matches!(adapter,Some("codex"|"copilot"|"antigravity")),_=>llm};let mut c=if supported{base(id)}else{Capability::unsupported(id)};gate(&mut c,d,configured,healthy,false,true,false);c.limitations.push("cacheCoverageUnknownIsNotZero");caps.push(c);}
    for id in ["goals.draft","goals.inspect","goals.admissionReview","goals.continuationReview","goals.retirementReview"]{
        let native_review=matches!(id,"goals.continuationReview"|"goals.retirementReview");
        let supported=if native_review{llm}else{id=="goals.inspect"||d.kind!=Kind::Observer};
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        if id=="goals.continuationReview"{c.decision=Decision::NativeApproval;c.limitations.extend(["authenticatedClosedSourceRequired","ordinaryAdditionalAllowanceOrFundedRemainingOriginalAllowance","unknownToolEffectsRequireReconciliation","doesNotEnableProviderExecution"]);}
        if id=="goals.retirementReview"{c.decision=Decision::NativeApproval;c.limitations.extend(["signedUnusedCanonicalGrantRequired","noLinkedAdmission","nativeNoIoEvidenceRequired","authorityReductionOnly","doesNotEnableProviderExecution"]);}
        c.callers=vec![Caller::NativeUser];c.limitations.push("executionRequiresNativeGrantAndLiveController");
        if cfg!(target_os="linux"){
            if id=="goals.draft"{gate(&mut c,d,configured,true,false,true,true);}else if !native_review||supported{c.reason=Some(Reason::TargetRequired);}
        }else{c.support=Support::Unsupported;c.availability=Availability::Unavailable;c.reason=Some(Reason::UnsupportedPlatform);}
        caps.push(c);
    }
    // Per-pet native System scopes are independent of inference and music.
    // Task eligibility is separate from addressed native UI and model tools.
    #[cfg(target_os="linux")]let system_task_transport=crate::pet_audio::task::native_dispatch_available();
    #[cfg(not(target_os="linux"))]let system_task_transport=false;
    for(id,operation)in [("system.access.manage",None),("system.sound.observe",Some(crate::pet_config::system_access::Operation::Observe)),("system.sound.volume",Some(crate::pet_config::system_access::Operation::Volume)),("system.sound.mute",Some(crate::pet_config::system_access::Operation::Mute))]{
        let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&!crate::pet_config::retired(&d.pet_id);
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};c.callers=vec![Caller::NativeUser];c.decision=if operation.is_none(){Decision::NativeApproval}else{Decision::ScopedGrant};c.model_binding=None;
        c.limitations.extend(["nativeUserRequiresVisibleAddressedPet","noInferenceRequired","noImplicitMusicOrSystemUiGrant","capturedPerPetRevisionAndPhysicalOutputKey","providerModelNotAdmitted"]);
        if supported&&matches!(operation,Some(crate::pet_config::system_access::Operation::Volume|crate::pet_config::system_access::Operation::Mute))&&system_task_transport{c.callers.push(Caller::TaskRuntime);c.limitations.extend(["taskRequiresOriginalDeterministicSystemAdmissionAndLiveOwner","exactNativeIncarnationAndCompleteCriteriaBeforeReview","nativeStepOrRecipeReviewAndOriginalBudgetRequired","inheritedPhysicalEffectExclusionUntilHelperFamilyEnds"]);}else{c.limitations.push("nativeUserOnlyNoTaskOperation");}
        c.limits.max_items=Some(if operation.is_none(){32}else{512});
        if supported{
            let granted=operation.is_none_or(|op|p.system_access.as_ref().is_some_and(|policy|policy.enabled&&policy.grants.iter().any(|g|g.subplugin==crate::pet_config::system_access::Subplugin::Sound&&g.operation==op)));
            let(availability,reason)=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else if operation.is_some()&&d.lifecycle==Lifecycle::Inactive{(Availability::Unavailable,Reason::InactivePet)}else if !granted{(Availability::Unavailable,Reason::PermissionDenied)}else{(Availability::Unknown,Reason::TargetRequired)};c.availability=availability;c.reason=Some(reason);
        }else if d.lifecycle==Lifecycle::Retired{c.reason=Some(Reason::RetiredPet);}else if !cfg!(target_os="linux"){c.reason=Some(Reason::UnsupportedPlatform);}caps.push(c);
    }
    // Passive health metadata performs no procfs IO and never selects a model.
    for id in ["system.health.observe","system.health.baseline"]{
        let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&!crate::pet_config::retired(&d.pet_id);
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=vec![Caller::NativeUser,Caller::TaskRuntime];c.decision=Decision::ScopedGrant;c.model_binding=None;c.limits.max_items=Some(if id=="system.health.baseline"{6}else{3});c.limits.max_bytes=Some(8192);
        c.limitations.extend(["nativeUserRequiresVisibleAddressedPet","ownHealthObserveHostHealthGrant","threeBoundedFixedProcfsCountersOnly","noInferenceOrAutomaticMemoryPublication","nativeTaskRequiresOriginalFiniteSystemAdmissionAndExactHost","noProviderModelToolOrImplicitExecutionStart","cpuCountersNeedSeparateBaseline"]);
        if id=="system.health.baseline"{c.limitations.extend(["twoSamplesFixedMinimumOneSecondIntervalMaximumSixSeconds","counterEstimateMayBeIndeterminate","originalSignedMeasurementReceiptNoVerifierResample"]);}
        if supported{
            let granted=p.system_access.as_ref().is_some_and(|policy|policy.permits_health());
            let state=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else if d.lifecycle==Lifecycle::Inactive{(Availability::Unavailable,Reason::InactivePet)}else if !granted{(Availability::Unavailable,Reason::PermissionDenied)}else{(Availability::Unknown,Reason::TargetRequired)};
            c.availability=state.0;c.reason=Some(state.1);
        }else if d.lifecycle==Lifecycle::Retired{c.reason=Some(Reason::RetiredPet);}else if !cfg!(target_os="linux"){c.reason=Some(Reason::UnsupportedPlatform);}
        caps.push(c);
    }
    // Passive projection never opens inventory sources or selects inference.
    {
        let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&!crate::pet_config::retired(&d.pet_id);
        let mut c=if supported{base("system.inventory.observe")}else{Capability::unsupported("system.inventory.observe")};
        c.callers=vec![Caller::NativeUser,Caller::TaskRuntime];c.decision=Decision::ScopedGrant;c.model_binding=None;c.limits.max_items=Some(2);c.limits.max_bytes=Some(8192);
        c.limitations.extend(["nativeUserRequiresVisibleAddressedPet","ownInventoryObserveHostComponentsGrant","fixedDistributionAndRunningKernelOnly","observedDataNotSysworkspaceManagedOrDesiredState","nativeTaskRequiresOriginalFiniteInventorySystemAdmission","originalSignedInventoryReceiptNoVerifierResample","noProviderModelMemoryDelegationImplicitStartOrMutation","sourceIoOnlyOnExplicitAuthorizedRequest"]);
        if supported{
            let granted=p.system_access.as_ref().is_some_and(|policy|policy.permits_inventory());
            let state=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else if d.lifecycle==Lifecycle::Inactive{(Availability::Unavailable,Reason::InactivePet)}else if !granted{(Availability::Unavailable,Reason::PermissionDenied)}else{(Availability::Unknown,Reason::TargetRequired)};
            c.availability=state.0;c.reason=Some(state.1);
        }else if d.lifecycle==Lifecycle::Retired{c.reason=Some(Reason::RetiredPet);}else if !cfg!(target_os="linux"){c.reason=Some(Reason::UnsupportedPlatform);}
        caps.push(c);
    }
    // Native identity is independent of inference readiness. These capabilities
    // still need a concrete goal, original human grant and bounded literal plan.
    for id in ["goals.deterministicAdmission","goals.systemAdmission","goals.systemRequestedExecution","goals.messageDeliver","goals.messageReply","goals.nativeVerify","goals.nativeClose"]{
        let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|!b.retired);
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};c.callers=if matches!(id,"goals.systemRequestedExecution"|"goals.messageDeliver"|"goals.messageReply"|"goals.nativeVerify"|"goals.nativeClose"){vec![Caller::NativeUser,Caller::TaskRuntime]}else{vec![Caller::NativeUser]};c.decision=Decision::NativeApproval;
        if matches!(id,"goals.systemAdmission"|"goals.systemRequestedExecution"){c.model_binding=None;c.limitations.extend(["exactNativeSystemTargetAndOwnGrantBeforeReview","completeTypedTerminalCriteriaRequired","noModelOrSelectedContext","manualOrExplicitSignedFiniteSystemStartOnCommonDriver","noModelGeneratedPlanOrAdditionalGrant"]);}
        if id=="goals.messageReply"{c.limitations.extend(["canonicalReceivedRootSourceRequired","depthOneReplyOnly","ownAdmissionAndExactHumanReviewRequired","sourceDeadlineCannotBeExtended","replyDoesNotAcceptOrExecuteParent"]);}
        if !matches!(id,"goals.systemAdmission"|"goals.systemRequestedExecution"){c.limitations.extend(["nativeLiteralPeerDataOnly","runtimeOnlyBudgetZeroModelCalls","originalNativeOwnerAndBudgetRequired","exactHumanMessageApprovalRequired","recipientExecutionNotAuthorized","noModelMemoryUse","restartNeedsExplicitRecovery"]);}
        if supported{let state=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if d.lifecycle==Lifecycle::Retired{(Availability::Unavailable,Reason::RetiredPet)}else if d.lifecycle==Lifecycle::Inactive{(Availability::Unavailable,Reason::InactivePet)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else if !crate::pet_runtime::deterministic_identity(d){(Availability::Unavailable,Reason::NotConfigured)}else{(Availability::Unknown,Reason::TargetRequired)};c.availability=state.0;c.reason=Some(state.1);}
        else if d.lifecycle==Lifecycle::Retired{c.reason=Some(Reason::RetiredPet);}else if !cfg!(target_os="linux"){c.reason=Some(Reason::UnsupportedPlatform);}caps.push(c);
    }
    for id in ["goals.peerPlanningReview","goals.peerPlanningExecution"]{
        let supported=cfg!(target_os="linux")&&d.kind==Kind::Llm&&matches!(adapter,Some("ollama"|"lmstudio"));
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};c.callers=if id=="goals.peerPlanningReview"{vec![Caller::NativeUser]}else{vec![Caller::NativeUser,Caller::TaskRuntime]};
        c.decision=Decision::NativeApproval;c.limits.max_items=Some(8);c.limitations.extend(["explicitPeerCorrespondenceV1Catalog","oneLocalModelRoundOriginalBudget","dedicatedContextNoSelectedMemoryOrReceivedDataYet","exactNativeDataRecipientDecisionAfterSignedPlan","noDelegationOrRecipientExecutionGrant","localReceiptsDoNotVerifyFreeCriteria","sameDriverJournalAuthorityAndResourceWait"]);
        if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else{Reason::TargetRequired});}caps.push(c);
    }
    for id in ["goals.peerMemoryPlanningReview","goals.peerMemoryPlanningExecution"]{
        let supported=cfg!(target_os="linux")&&d.kind==Kind::Llm&&matches!(adapter,Some("ollama"|"lmstudio"));let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=if id=="goals.peerMemoryPlanningReview"{vec![Caller::NativeUser]}else{vec![Caller::NativeUser,Caller::TaskRuntime]};c.decision=Decision::NativeApproval;c.limits.max_items=Some(8);
        c.limitations.extend(["explicitPeerCorrespondenceMemoryV1Catalogue","ownDedicatedSelectedMemoryAndR43Consent","allSourceRevisionsConservativeDependencies","oneLocalRoundOriginalBudget","exactNativeTextRecipientDecision","recipientOwnR43AdmissionNoInheritedModelConsent","noReceivedTransitiveForwardingOrDelegation","noRawSourceMemoryInRecipientPrompt"]);
        if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else{Reason::TargetRequired});}caps.push(c);
    }
    {
        let id="goals.delegationOutcomeEvidence";let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer;let mut c=if supported{base(id)}else{Capability::unsupported(id)};c.callers=vec![Caller::NativeUser];c.limits.max_items=Some(16);c.limitations.extend(["explicitNativeUserReadOnly","exactParentReservationAndOwnChildBinding","commonReconciliationAndModelReceiptVerification","noResultTextMemoryOrModelUseConsent","availableModelResultNeverVerifiesTaskCriteria","partialFailureDoesNotEraseIndependentBranches","observationHashesNeverExecutionAuthority"]);if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else{Reason::TargetRequired});}caps.push(c);
    }
    for id in ["goals.peerDelegationPlanningReview","goals.peerDelegationPlanningExecution"]{
        let supported=cfg!(target_os="linux")&&d.kind==Kind::Llm&&matches!(adapter,Some("ollama"|"lmstudio"));let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=if id=="goals.peerDelegationPlanningReview"{vec![Caller::NativeUser]}else{vec![Caller::NativeUser,Caller::TaskRuntime]};c.decision=Decision::NativeApproval;c.limits.max_items=Some(8);
        c.limitations.extend(["explicitPeerDelegationV1Catalogue","oneOwnRoundSharedTotalBudgetWithChildReservations","sameCanonicalGrantWriterAndDriver","noSelectedOrReceivedContextForwarding","exactNativeRecipientCriteriaDeadlineAllowanceDecision","recipientOwnAdmissionAndR43Consent","unknownConsumptionRetained","localReceiptNeverProvesChildSuccess"]);
        if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else{Reason::TargetRequired});}caps.push(c);
    }
    for id in ["goals.peerIterationPlanningReview","goals.peerIterationPlanningExecution"]{
        let supported=cfg!(target_os="linux")&&d.kind==Kind::Llm&&matches!(adapter,Some("ollama"|"lmstudio"));let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=if id=="goals.peerIterationPlanningReview"{vec![Caller::NativeUser]}else{vec![Caller::NativeUser,Caller::TaskRuntime]};c.decision=Decision::NativeApproval;c.limits.max_items=Some(8);
        c.limitations.extend(["explicitPeerDelegationIterationV1Catalogue","twoToEightOwnRoundsWithinOriginalSharedAllowance","oneCanonicalGrantJournalDriverAndResourceQueue","dedicatedContextWithExplicitOwnMemoryRevisions","receivedAndCopiedContextForwardingUnsupported","ownMemoryNeedsOriginalF2AndPerRecipientLineage","sourceRevocationAlsoFencesLiteralRecipients","eachFurtherRoundNeedsExactNativeDataBodyDestinationConsent","noModelSelfApprovalOrRecipientExecutionGrant","newInformationAndRemainingBalanceRequired","unknownConsumptionAndOriginalDeadlinePreserved","oldOneRoundAdmissionsNeverPromoted"]);
        gate(&mut c,d,configured,healthy,false,true,true);caps.push(c);
    }
    for id in ["goals.nativePeerPlanReview","goals.nativePeerExecution","goals.nativePeerSequenceDecision"]{
        let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|!b.retired);
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=if matches!(id,"goals.nativePeerPlanReview"|"goals.nativePeerSequenceDecision"){vec![Caller::NativeUser]}else{vec![Caller::NativeUser,Caller::TaskRuntime]};
        c.decision=Decision::NativeApproval;c.limits.max_items=Some(16);
        c.limitations.extend([if id=="goals.nativePeerSequenceDecision"{"exactNativeSequenceDecisionRequired"}else{"ownSignedNativePeerDataConsentRequired"},"exactPlanSourcesRecipientsAndOriginalDeadline","sameDispatcherDriverReceiptsResourcesAndBudget","noOwnInferenceRecipientExecutionOrModelDataUse","manualAdmissionsNeverPromoted","uncertainDeliveryNeverRecreated"]);
        if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else{Reason::TargetRequired});}caps.push(c);
    }
    for id in ["goals.executionStartReview","goals.requestedExecution"]{
        let supported=cfg!(target_os="linux")&&d.kind==Kind::Llm&&matches!(adapter,Some("ollama"|"lmstudio"));
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=if id=="goals.executionStartReview"{vec![Caller::NativeUser]}else{vec![Caller::NativeUser,Caller::TaskRuntime]};
        c.decision=Decision::NativeApproval;c.limits.max_items=Some(1);
        c.limitations.extend(["explicitSignedInitialExecutionChoice","oldManualAdmissionsNeverPromoted","oneLocalResponseZeroToolsOrExactResultDisclosure","sameNativeDispatcherResourcesDriverAndOriginalBudget","captureOwnerContextDestinationBeforeWait","stopAndRevocationBeforeEveryEffect","claimedOrUncertainEffectsNeverReplayed","backendAvailabilityRequiredNoHostServiceActivation"]);
        if supported{c.availability=Availability::Unknown;c.reason=Some(if !configured{Reason::NotConfigured}else if !healthy{Reason::LibraryUnavailable}else{Reason::TargetRequired});}caps.push(c);
    }
    // A received-task review creates own goal DATA, with no model/admission.
    let mut received={let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|!b.retired);if supported{base("goals.receivedRequest")}else{Capability::unsupported("goals.receivedRequest")}};
    received.callers=vec![Caller::NativeUser];received.decision=Decision::NativeApproval;
    received.limitations.extend(["canonicalUnexpiredReceivedRequestRequired","nativeFrameAndOwnIdentityRequired","ownReviewedGoalDataOnly","separateContextBudgetAndExecutionRequired","noAutomaticProviderDataUse","noDelegationOrTransferredAuthority"]);
    if received.support==Support::Supported{let state=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if d.lifecycle==Lifecycle::Retired{(Availability::Unavailable,Reason::RetiredPet)}else if d.lifecycle==Lifecycle::Inactive{(Availability::Unavailable,Reason::InactivePet)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else if !crate::pet_runtime::deterministic_identity(d){(Availability::Unavailable,Reason::NotConfigured)}else{(Availability::Unknown,Reason::TargetRequired)};received.availability=state.0;received.reason=Some(state.1);}else if !cfg!(target_os="linux"){received.reason=Some(Reason::UnsupportedPlatform);}else if d.lifecycle==Lifecycle::Retired{received.reason=Some(Reason::RetiredPet);}caps.push(received);
    let mut received={let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|!b.retired);if supported{base("goals.receivedResult")}else{Capability::unsupported("goals.receivedResult")}};
    received.callers=vec![Caller::NativeUser];received.decision=Decision::NativeApproval;
    received.limitations.extend(["canonicalUnexpiredDerivedResultRequired","originalSourceDependenciesMustRemainCurrent","noRawSourceMemoryOrTransferredBalance","nativeFrameAndOwnIdentityRequired","ownReviewedGoalDataOnly","separateContextBudgetAndExecutionRequired","noAutomaticProviderDataUse","noDelegationOrTransferredAuthority"]);
    if received.support==Support::Supported{let state=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if d.lifecycle==Lifecycle::Retired{(Availability::Unavailable,Reason::RetiredPet)}else if d.lifecycle==Lifecycle::Inactive{(Availability::Unavailable,Reason::InactivePet)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else if !crate::pet_runtime::deterministic_identity(d){(Availability::Unavailable,Reason::NotConfigured)}else{(Availability::Unknown,Reason::TargetRequired)};received.availability=state.0;received.reason=Some(state.1);}else if !cfg!(target_os="linux"){received.reason=Some(Reason::UnsupportedPlatform);}else if d.lifecycle==Lifecycle::Retired{received.reason=Some(Reason::RetiredPet);}caps.push(received);
    let mut result_response=if cfg!(target_os="linux")&&d.kind==Kind::Llm{base("goals.receivedResultResponse")}else{Capability::unsupported("goals.receivedResultResponse")};
    result_response.callers=vec![Caller::NativeUser,Caller::TaskRuntime];result_response.decision=Decision::NativeApproval;result_response.limits.max_items=Some(1);
    result_response.limitations.extend(["localOllamaOrLmstudioOwnFiniteResponseOnly","separateExactGoalDataConsentRequired","ownBudgetOneModelZeroTools","noSenderMemoryOrDelegationBudgetTransfer","nativeReceiptIsNotSemanticCompletion","noAutomaticContinuationOrOnwardSharing"]);
    if result_response.support==Support::Supported{result_response.availability=Availability::Unknown;result_response.reason=Some(if healthy{Reason::TargetRequired}else{Reason::LibraryUnavailable});}caps.push(result_response);
    let mut work=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.receivedWork")}else{Capability::unsupported("goals.receivedWork")};
    work.callers=vec![Caller::NativeUser];work.decision=Decision::None;work.limits.max_items=Some(1);
    work.limitations.extend(["ownCertifiedReceivedGoalAndExactRevisionRequired","readOnlyLatestAdmissionFacts","noAckRepairOrExecutionAuthority","historicalStateAfterSnapshotUnknown","queuedOrLeasedDoesNotProveActiveEffect","noProviderForwardingPermission"]);
    if work.support==Support::Supported{work.availability=Availability::Unknown;work.reason=Some(if healthy{Reason::TargetRequired}else{Reason::LibraryUnavailable});}caps.push(work);
    let mut escrow=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.delegationEscrow")}else{Capability::unsupported("goals.delegationEscrow")};escrow.callers=vec![Caller::NativeUser];escrow.decision=Decision::NativeApproval;escrow.limits.max_items=Some(16);escrow.limitations.extend(["senderSignedPoolAndExactRequestReview","canonicalReceiptAndReservationOneTransaction","pendingReservationNotChildAuthority","recipientOwnNativeAdmissionRequired","withdrawOnlyProvenUnboundReservation"]);if escrow.support==Support::Supported{escrow.availability=Availability::Unknown;escrow.reason=Some(Reason::TargetRequired);}caps.push(escrow);
    let mut funded=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.fundedDelegation")}else{Capability::unsupported("goals.fundedDelegation")};funded.callers=vec![Caller::NativeUser];funded.decision=Decision::NativeApproval;funded.limits.max_items=Some(1);funded.limitations.extend(["ownExactDataAcceptanceAndIndependentNativeAdmission","oneCanonicalParentChildBinding","deterministicPeerOperationsOnly","nativeLiteralRouteHasNoInference","fundedPlanningNotImplemented;recursiveFundingRequiresSeparateOwnReview","boundAllowanceHeldUntilExplicitReconciliation","requestReceiptIsNotRecipientAuthority"]);if funded.support==Support::Supported{funded.availability=Availability::Unknown;funded.reason=Some(Reason::TargetRequired);}caps.push(funded);
    let mut result_plan=if cfg!(target_os="linux")&&d.kind==Kind::Llm{base("goals.resultReplyPlan")}else{Capability::unsupported("goals.resultReplyPlan")};result_plan.callers=vec![Caller::NativeUser];result_plan.decision=Decision::NativeApproval;result_plan.limits.max_items=Some(1);result_plan.limitations.extend(["authenticatedOwnResultReceiptAndExactUtf8Excerpt","originalDelegationRecipientAndDeadline","planDataOnlyNoDisclosureOrExecutionConsent","samePlanRevisionPublicationAndRecovery","derivedDeliveryRequiresSeparateSignedDisclosureAdmission"]);if result_plan.support==Support::Supported{result_plan.availability=Availability::Unknown;result_plan.reason=Some(Reason::TargetRequired);}caps.push(result_plan);
    for id in ["goals.resultDisclosureReview","goals.resultReply"]{let mut c=if cfg!(target_os="linux")&&d.kind==Kind::Llm{base(id)}else{Capability::unsupported(id)};c.callers=if id=="goals.resultReply"{vec![Caller::NativeUser,Caller::TaskRuntime]}else{vec![Caller::NativeUser]};c.decision=Decision::NativeApproval;c.limits.max_items=Some(1);c.limitations.extend(["fundedLocalModelHistoryAndAuthenticatedOwnResultRequired","separateBoundedResultDisclosureNotF2OrWorkMetadata","singleLocalReplyZeroNewInference","sameOriginalReservationAndCumulativeBudget","exactRecipientConfigAndSelectedSourcesRevalidated","localReceiptDoesNotVerifyArbitraryCriteriaOrRecipientExecution"]);if c.support==Support::Supported{c.availability=Availability::Unknown;c.reason=Some(Reason::TargetRequired);}caps.push(c);}
    let mut recursive=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.recursiveDelegation")}else{Capability::unsupported("goals.recursiveDelegation")};recursive.callers=vec![Caller::NativeUser];recursive.decision=Decision::NativeApproval;recursive.limits.max_items=Some(16);recursive.limitations.extend(["ownExactDataAdmissionAndEachPeerOperationReview","canonicalAncestorsDepth8Nodes128AndNoCycles","sameOriginalAllowanceAndDeadline;noBudgetRenewal","ancestorPauseRetainsBranches;cancelReducesSubtreeBeforeStop","originalWorkersMustDrainBeforeCreditOrPhysicalRelease","nestedInferenceRequiresOwnExactF2Consent;recursiveContinuationAndPlanRevisionNotImplemented","noShellSystemOrNativeCliPromotion"]);if recursive.support==Support::Supported{recursive.availability=Availability::Unknown;recursive.reason=Some(Reason::TargetRequired);}caps.push(recursive);
    let mut tools=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.fundedToolContinuation")}else{Capability::unsupported("goals.fundedToolContinuation")};tools.callers=vec![Caller::NativeUser];tools.decision=Decision::NativeApproval;tools.limitations.extend(["confirmedOriginalNativeClosureAndAuthenticatedReceiptCarry","cumulativePriorChargeAndNewLimitsWithinOriginalAllowanceAndDeadline","exactOwnReviewAndPerPeerOperationApproval","noInferenceRecursiveFundingSystemOrShellPromotion","unknownOutcomesRemainBlocking"]);if tools.support==Support::Supported{tools.availability=Availability::Unknown;tools.reason=Some(Reason::TargetRequired);}caps.push(tools);
    let mut response=if cfg!(target_os="linux")&&d.kind!=Kind::Observer&&d.binding.adapter.is_some_and(|a|["ollama","lmstudio"].contains(&a)){base("goals.fundedResponse")}else{Capability::unsupported("goals.fundedResponse")};response.callers=vec![Caller::NativeUser];response.decision=Decision::NativeApproval;response.limits.max_items=Some(1);response.limitations.extend(["ownExactF2GoalDataConsentAndConfiguredLoopbackDestination","oneModelCallZeroToolsPerExplicitReviewNoAutomaticContinuation","canonicalSourceBindingAndOwnAttemptBudget","sameOriginalOwnerDriverAndDrain","persistedResponseNotVerifiedDelegatedCriteria"]);if response.support==Support::Supported{response.availability=Availability::Unknown;response.reason=Some(Reason::TargetRequired);}caps.push(response);
    let mut nested=if cfg!(target_os="linux")&&d.kind!=Kind::Observer&&d.binding.adapter.is_some_and(|a|["ollama","lmstudio"].contains(&a)){base("goals.recursiveResponse")}else{Capability::unsupported("goals.recursiveResponse")};nested.callers=vec![Caller::NativeUser];nested.decision=Decision::NativeApproval;nested.limits.max_items=Some(1);nested.limits.max_calls=Some(1);nested.limitations.extend(["ownSelectedRecursiveSourceAndExactF2GoalConsent","configuredLoopbackDestinationModelAndActualComposedBody","oneOwnModelCallZeroToolsNoContinuationOrImplicitDisclosure","sameOriginalAllowanceDeadlineAndAncestorRevocationFences","originalOwnerDriverDrainAndUnknownConsumptionRetained","persistedArtefactNotVerifiedCriteria"]);if nested.support==Support::Supported{nested.availability=Availability::Unknown;nested.reason=Some(Reason::TargetRequired);}caps.push(nested);
    let mut branch=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.delegationBranchControl")}else{Capability::unsupported("goals.delegationBranchControl")};branch.callers=vec![Caller::NativeUser];branch.decision=Decision::NativeApproval;branch.limits.max_items=Some(1);branch.limitations.extend(["exactCanonicalChildBindingAndHumanReview","persistBranchRevocationBeforeOwnStop","originalWorkerDrainRequiredForSettlement","observedAndUnknownConsumptionSeparate","noNewExecutionOrTransferredAuthority"]);if branch.support==Support::Supported{branch.availability=Availability::Unknown;branch.reason=Some(Reason::TargetRequired);}caps.push(branch);
    let mut report=if cfg!(target_os="linux")&&d.kind!=Kind::Observer{base("goals.workReport")}else{Capability::unsupported("goals.workReport")};
    report.callers=vec![Caller::NativeUser];report.decision=Decision::NativeApproval;report.limits.max_items=Some(1);
    report.limitations.extend(["ownCertifiedSubjectAndRequesterOnly","separateCurrentReportingAdmissionAndExactMetadataReview","datedObservationNotCurrentStatePromise","doesNotBorrowRevokedSubjectAuthority","noAutomaticProviderUseOrRecipientExecution"]);
    if report.support==Support::Supported{report.availability=Availability::Unknown;report.reason=Some(Reason::TargetRequired);}caps.push(report);
    // Editing closed native plan DATA is a human preservation operation. It
    // neither reactivates an inactive pet nor invokes its inference binding.
    let mut edit={
        let supported=cfg!(target_os="linux")&&d.kind!=Kind::Observer&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|!b.retired);
        let mut c=if supported{base("goals.nativePlanRevision")}else{Capability::unsupported("goals.nativePlanRevision")};
        c.callers=vec![Caller::NativeUser];c.decision=Decision::NativeApproval;
        c.limits.max_items=Some(16);c.limits.max_bytes=Some(256*1024);
        c.limitations.extend(["authenticatedClosedNativeGoalRequired","exactHumanDataReviewAndCurrentFrameRequired","goalPetAndRevisionBound","coveredCausesAndReceiptsImmutable","dataOnlyNoBudgetWorkerOrInference","separateExecutionConsentRequired","unknownEffectsRequireReconciliation","ackDoesNotRenewAuthorityOrRefund","notProviderModelTool"]);
        if supported{let state=if !configured{(Availability::Unknown,Reason::NotConfigured)}else if !healthy{(Availability::Unavailable,Reason::LibraryUnavailable)}else{(Availability::Unknown,Reason::TargetRequired)};c.availability=state.0;c.reason=Some(state.1);}
        else if d.lifecycle==Lifecycle::Retired{c.reason=Some(Reason::RetiredPet);}else if !cfg!(target_os="linux"){c.reason=Some(Reason::UnsupportedPlatform);}c
    };
    edit.model_binding=None;caps.push(edit);
    // Implementation support is per native adapter and bounded catalog. A
    // shared factory never turns another pet/session into an executable owner.
    for id in ["goals.planBounded","goals.executeRecipe","goals.verifyCriteria"] {
        let supported=cfg!(target_os="linux")&&matches!(adapter,Some("ollama"|"lmstudio"))&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|d.binding.provider.as_deref().is_some_and(|p|b.recognizes_owner(p,adapter.unwrap())));
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=vec![Caller::NativeUser,Caller::TaskRuntime];c.decision=Decision::ScopedGrant;c.model_binding=d.binding.model.clone();
        c.limitations.extend(["originalNativeOwnerAndBudgetRequired",if adapter==Some("lmstudio"){"boundedLocalLmStudioOnly"}else{"boundedLocalOllamaOnly"},"spotifyTransportV1Only","exactHumanRecipeApprovalRequired","nativeFinalCriteriaOnly","restartNeedsExplicitRecovery"]);
        gate(&mut c,d,configured,healthy,busy,permissions.music,true);caps.push(c);
    }
    // Human catalogue access is deterministic and does not expose a model
    // retrieval tool. Concrete data use always needs a target-bound review.
    let mut manage=if cfg!(target_os="linux"){base("memory.manage")}else{Capability::unsupported("memory.manage")};
    manage.callers=vec![Caller::NativeUser];manage.limits.max_items=Some(256);manage.limits.max_bytes=Some(1024*1024);
    manage.limitations.extend(["explicitUserRecordsOnly","petAndConversationScope","notModelMemory","noAutomaticSharing"]);
    if cfg!(target_os="linux"){
        if !configured{manage.reason=Some(Reason::NotConfigured);}
        else if d.lifecycle==Lifecycle::Retired{manage.availability=Availability::Unavailable;manage.reason=Some(Reason::RetiredPet);}
        else if !healthy{manage.availability=Availability::Unavailable;manage.reason=Some(Reason::LibraryUnavailable);}
        else{manage.availability=Availability::Ready;manage.reason=None;}
    }else{manage.reason=Some(Reason::UnsupportedPlatform);}caps.push(manage);
    #[cfg(target_os="linux")]let data_route=crate::pet_memory::manual::configured_route(&d.pet_id,p).is_some();
    #[cfg(not(target_os="linux"))]let data_route=false;
    for id in ["memory.contextReview","memory.goalContext"]{
        let supported=if id=="memory.contextReview"{data_route}else{cfg!(target_os="linux")&&matches!(adapter,Some("ollama"|"lmstudio"))&&crate::pet_config::blueprint(&d.pet_id).is_some_and(|b|d.binding.provider.as_deref().is_some_and(|p|b.recognizes_owner(p,adapter.unwrap())))};
        let mut c=if supported{base(id)}else{Capability::unsupported(id)};
        c.callers=if id=="memory.goalContext"{vec![Caller::NativeUser,Caller::TaskRuntime]}else{vec![Caller::NativeUser]};
        c.decision=if id=="memory.goalContext"{Decision::ScopedGrant}else{Decision::NativeApproval};c.model_binding=d.binding.model.clone();
        c.limits.max_items=Some(16);c.limits.max_bytes=Some(64*1024);c.limitations.extend(["explicitSelectedRevisionsOnly","nativeDataConsentRequired","destinationAndOriginalOwnerBound","revocationDoesNotRetractDeliveredData","notProviderModelTool"]);
        if id=="memory.goalContext"{c.limitations.extend(["originalGoalGrantAndWriterRequired","manualOrCliConsentIsNotGoalConsent","optionalCertifiedReceivedRequestSource","receivedDataCountsWithinSharedReferenceAndByteLimit","oneReceivedSourceMaximum","receivedSourceContextRequiresOwnGoalDataConsent"]);}
        gate(&mut c,d,configured,healthy,false,true,true);
        // API review uses the explicit factory API route, not a CLI adapter.
        if id=="memory.contextReview"&&supported&&configured&&healthy&&d.lifecycle==Lifecycle::Active{c.availability=Availability::Unknown;c.reason=Some(Reason::TargetRequired);}
        caps.push(c);
    }
    let mut retrieval=Capability::unsupported("memory.retrieve");retrieval.callers=vec![Caller::ProviderModel];retrieval.limitations.push("catalogueCrudIsNotModelRetrieval");caps.push(retrieval);
    for id in ["goals.run","rules.schedule","rules.events","delegation.send","sources.rss","sources.imapRead","tools.typedRecipe","usage.nativeThread","system.audio","system.network","system.bluetooth"]{caps.push(Capability::unsupported(id));}
    if p.inference.is_some(){for c in &mut caps{
        if c.id=="tools.exactCommand"{c.callers.retain(|caller|*caller!=Caller::TaskRuntime);}
        let pending=c.id=="tools.exactCommand"&&d.binding.provider.as_deref()!=Some("ollama")
            ||c.id=="voice.respond"&&(d.pet_id!="ai_ollama"||adapter!=Some("ollama"));
        if pending{c.support=Support::Unknown;c.availability=Availability::Unavailable;c.reason=Some(Reason::NotImplemented);c.limitations.push("explicitBindingRuntimeIntegrationPending");}
        if c.id=="models.select"{c.limitations.push("nativeBindingFieldRevisionRequired");}
    }}
    for c in &mut caps{if configured&&c.support!=Support::Unsupported{c.source.push(Source::Configuration);}}
    debug_assert!(caps.len()<=128&&caps.iter().map(|c|c.id).collect::<BTreeSet<_>>().len()==caps.len());d.capabilities=caps;
}
pub(crate) fn begin_catalog(settings:&Settings,provider:&str)->Option<Ticket>{
    let config=crate::pet_config::read_existing().ok()?;begin_catalog_configured(settings,provider,&config)
}
pub(crate)fn begin_catalog_configured(settings:&Settings,provider:&str,config:&Configuration)->Option<Ticket>{
    let id=crate::pet_config::integration_pet(provider)?;let p=config.pets.get(id)?;let b=binding(settings,id,p,true);let key=key(settings,&b,p,None)?;registry().ok()?.lock().ok()?.cache.begin(key,crate::assistant_memory::now())
}
pub(crate) fn finish_catalog(ticket:Option<Ticket>,success:bool){if let (Some(t),Ok(r))=(ticket,registry()){if let Ok(mut r)=r.lock(){r.cache.finish(t,if success{Evidence::Catalog}else{Evidence::Unavailable},crate::assistant_memory::now());}}}
pub(crate)fn valid_model_id(value:&str)->bool{cache::model_id(value)}
pub(crate)fn begin_lmstudio(endpoint:&str,generation:u64)->Option<Ticket>{begin_lmstudio_scoped(endpoint,crate::local_chat::lmstudio::MetadataAuth::Legacy(generation))}
pub(crate)fn begin_lmstudio_scoped(endpoint:&str,auth:crate::local_chat::lmstudio::MetadataAuth)->Option<Ticket>{let key=Key{provider:"lmstudio".into(),endpoint:crate::local_chat::lmstudio::endpoint_key_for(endpoint,auth)?,model:None};registry().ok()?.lock().ok()?.cache.begin(key,crate::assistant_memory::now())}
pub(crate)fn finish_lmstudio(ticket:Option<Ticket>,catalog:Option<&crate::local_chat::lmstudio::Catalog>){if let(Some(t),Ok(r))=(ticket,registry()){if let Ok(mut r)=r.lock(){r.cache.finish(t,catalog.map(|c|Evidence::LmStudio(c.clone())).unwrap_or(Evidence::Unavailable),crate::assistant_memory::now());}}}
pub(crate) fn begin_local(endpoint:&str,model:&str)->Option<Ticket>{
    if !cache::model_id(model){return None;}let key=Key{provider:"ollama".into(),endpoint:cache::endpoint(endpoint)?,model:Some(model.into())};registry().ok()?.lock().ok()?.cache.begin(key,crate::assistant_memory::now())
}
pub(crate) fn finish_local(ticket:Option<Ticket>,value:Option<&serde_json::Value>){if let (Some(t),Ok(r))=(ticket,registry()){if let Ok(mut r)=r.lock(){r.cache.finish(t,value.map(cache::model_evidence).unwrap_or(Evidence::Unavailable),crate::assistant_memory::now());}}}
/// Passive reuse of exact endpoint/model evidence; never IO or an execution grant.
pub(crate)fn fresh_installed_local_model(endpoint:&str,model:&str,now:u64)->bool{
 let Some(endpoint)=cache::endpoint(endpoint)else{return false;};if !cache::model_id(model){return false;}
 registry().ok().and_then(|r|r.lock().ok().map(|r|r.cache.fresh_installed_local_model(&Key{provider:"ollama".into(),endpoint,model:Some(model.into())},now))).unwrap_or(false)
}
fn allowed(label:&str)->bool{[crate::island::WINDOW_LABEL,crate::HUB_LABEL,"settings",crate::companions::LABEL,crate::pet_chat::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL].contains(&label)}
fn native_registry()->Result<&'static Mutex<native::Store>,String>{
    static NATIVE:OnceLock<Result<Mutex<native::Store>,String>>=OnceLock::new();
    NATIVE.get_or_init(||{
        let instance=registry()?.lock().map_err(|_|ERROR)?.instance.clone();
        Ok(Mutex::new(native::Store::new(format!("native:{instance}"))))
    }).as_ref().map_err(Clone::clone)
}
pub(crate) fn begin_native(target:&crate::session_chat::types::Target)->Option<NativeTicket>{
    native_registry().ok()?.lock().ok()?.begin(target,crate::assistant_memory::now())
}
pub(crate) fn finish_native(ticket:Option<NativeTicket>,snapshot:Option<&crate::session_chat::types::Snapshot>){
    if let (Some(ticket),Ok(store))=(ticket,native_registry()){if let Ok(mut store)=store.lock(){store.finish(ticket,snapshot,crate::assistant_memory::now());}}
}
pub(crate) fn invalidate_native(target:&crate::session_chat::types::Target){
    if let Ok(store)=native_registry(){if let Ok(mut store)=store.lock(){store.invalidate(target);}}
}
pub(crate) fn invalidate_native_provider(provider:&str){
    if let Ok(store)=native_registry(){if let Ok(mut store)=store.lock(){store.invalidate_provider(provider);}}
}
fn gate_native(d:&mut TargetDescriptor,pet:&Descriptor){
    d.revision.push('/');d.revision.push_str(&pet.revision);
    let denied=if !pet.configuration_available||pet.binding.adapter!=Some(d.target.provider.as_str())||pet.binding.provider.as_deref()!=Some(d.target.provider.as_str()){Some((Availability::Unknown,Reason::NotConfigured))}
        else if pet.lifecycle==Lifecycle::Retired{Some((Availability::Unavailable,Reason::RetiredPet))}
        else if pet.lifecycle==Lifecycle::Inactive{Some((Availability::Unavailable,Reason::InactivePet))}
        else if pet.capabilities.iter().any(|c|c.reason==Some(Reason::LibraryUnavailable)){Some((Availability::Unavailable,Reason::LibraryUnavailable))}else{None};
    for c in &mut d.capabilities{
        if pet.configuration_available{c.source.push(Source::Configuration);}
        if c.support!=Support::Unsupported{if let Some((a,r))=denied{c.availability=a;c.reason=Some(r);}}
        if c.support!=Support::Unsupported&&!preservation_capability(c.id)&&pet.capabilities.iter().any(|p|p.reason==Some(Reason::SettingsUnconfirmed)){
            c.availability=Availability::Unavailable;c.reason=Some(Reason::SettingsUnconfirmed);
        }
    }
}
pub(crate) fn native_target_snapshot(app:&AppHandle,target:&crate::session_chat::types::Target)->Result<TargetDescriptor,String>{
    native::validate(target)?;
    let pets=passive_snapshot(app)?;
    let now=crate::assistant_memory::now();
    let pet=pets.pets.iter().find(|p|p.pet_id==target.pet_id).ok_or(ERROR)?;
    let mut d=native_registry()?.lock().map_err(|_|ERROR)?.view(target,now);
    gate_native(&mut d,pet);Ok(d)
}
/// Common passive projection used by IPC, voice preparation and management.
/// Reading existing configuration/caches never prepares a provider or microphone.
/// Internal capture: descriptor and policy fingerprint must use the same
/// settings/configuration values. Never serialize this object to the frontend.
pub(crate)struct NativeProjection{pub snapshot:Snapshot,pub settings:Settings,pub configuration:Option<Configuration>,pub preferences_ready:bool}
pub(crate)fn native_projection(app:&AppHandle,owned:Option<&crate::pet_resources::Hold>)->Result<NativeProjection,String>{
 project(app,owned,false)
}
/// Consent to queued work needs real protocol/model evidence, not an idle slot.
/// This internal projection never authorizes acquisition or an effect; public
/// status, claim and execution continue to use the actual occupancy projection.
pub(crate)fn admission_projection(app:&AppHandle)->Result<NativeProjection,String>{project(app,None,true)}
fn project(app:&AppHandle,owned:Option<&crate::pet_resources::Hold>,admission:bool)->Result<NativeProjection,String>{
 let(settings,preferences_ready)={let shared=app.state::<crate::Shared>();let current=shared.settings.lock().map_err(|_|ERROR)?;
     #[cfg(target_os="linux")]let ready=crate::settings::io_status(&current).new_effects_allowed;
     #[cfg(not(target_os="linux"))]let ready=true;
     (current.clone(),ready)};
 let configuration=crate::pet_config::read_existing().ok();let chat=app.state::<crate::chat::Chat>();
 let(healthy,mut busy)=chat.capability_health();
 if let Some(config)=&configuration{busy.extend(chat.runtime_resources().busy_with_config(config,&settings).map_err(|_|ERROR.to_owned())?);}
 if let Some(hold)=owned{let resources=chat.runtime_resources();let provider=hold.provider_for(&resources).ok_or_else(||ERROR.to_owned())?;busy.remove(provider);}
 let mut registry=registry()?.lock().map_err(|_|ERROR)?;
 let snapshot=if admission{registry.admission_snapshot(&settings,configuration.as_ref(),healthy,crate::assistant_memory::now(),preferences_ready)?}else{registry.snapshot_with_io(&settings,configuration.as_ref(),healthy,&busy,crate::assistant_memory::now(),preferences_ready)?};
 Ok(NativeProjection{snapshot,settings,configuration,preferences_ready})
}
pub(crate) fn passive_snapshot(app:&AppHandle)->Result<Snapshot,String>{native_projection(app,None).map(|p|p.snapshot)}
pub(crate) fn provider_descriptor(app:&AppHandle,provider:&str)->Result<Descriptor,String>{
    let id=crate::pet_config::integration_pet(provider).ok_or("Este pet não tem um provider de conversa configurável.")?;
    passive_snapshot(app)?.pets.into_iter().find(|p|p.pet_id==id).ok_or_else(||ERROR.into())
}
#[tauri::command]
pub async fn pet_capabilities_target(app:AppHandle,window:tauri::WebviewWindow,target:crate::session_chat::types::Target)->Result<TargetDescriptor,String>{
    if !allowed(window.label()){return Err("Abre a gestão ou a conversa do pet para consultar capacidades.".into());}
    tauri::async_runtime::spawn_blocking(move||native_target_snapshot(&app,&target)).await.map_err(|_|ERROR.to_owned())?
}
#[tauri::command]
pub async fn pet_capabilities_snapshot(app:AppHandle,window:tauri::WebviewWindow,pet_id:Option<String>)->Result<Snapshot,String>{
    if !allowed(window.label()){return Err("Abre a gestão ou a conversa do pet para consultar capacidades.".into());}
    if pet_id.as_ref().is_some_and(|id|!crate::pet_config::defaults().pets.contains_key(id)){return Err("Pet desconhecido.".into());}
    tauri::async_runtime::spawn_blocking(move||{
        let mut result=passive_snapshot(&app)?;
        if let Some(id)=pet_id{result.pets.retain(|p|p.pet_id==id);}Ok(result)
    }).await.map_err(|_|ERROR.to_owned())?
}
#[cfg(test)]#[path="../pet_capabilities_tests.rs"]mod tests;

/// Observation epochs describe freshness, not a different human authorization.
/// Preserve every semantic field; only the public epoch and observation times
/// may refresh between prepare and confirm. Callers still check fresh readiness.
pub(crate)fn admission_semantics(descriptor:&Descriptor)->Result<String,String>{
    let mut value=descriptor.clone();value.revision.clear();value.observed_at=None;
    for capability in &mut value.capabilities{capability.observed_at=None;capability.expires_at=None;}
    let raw=serde_json::to_vec(&value).map_err(|_|ERROR.to_string())?;
    Ok(ring::digest::digest(&ring::digest::SHA256,&raw).as_ref().iter().map(|b|format!("{b:02x}")).collect())
}

/// Real cache/registry producer for cross-module regression fixtures. No global
/// cache, real provider, app profile or production authority is constructed.
#[cfg(test)]pub(crate)fn fixture_local_projection(settings:Settings,configuration:Configuration,observed_at:u64,now:u64)->NativeProjection {
    let mut registry=Registry::new().unwrap();
    let key=Key{provider:"ollama".into(),endpoint:cache::endpoint(&settings.ollama_url).unwrap(),model:settings.chat_models.get("ollama").cloned()};
    let ticket=registry.cache.begin(key,observed_at).unwrap();
    let evidence=cache::model_evidence(&serde_json::json!({"model_info":{"general.architecture":"synthetic"},"capabilities":["completion"]}));
    assert!(registry.cache.finish(ticket,evidence,observed_at));
    let snapshot=registry.snapshot(&settings,Some(&configuration),true,&BTreeSet::new(),now).unwrap();
    NativeProjection{snapshot,settings,configuration:Some(configuration),preferences_ready:true}
}

/// Test input producer: authentic registry/cache resolution with isolated
/// metadata. Never edits a descriptor/caller or reads an authenticated profile.
#[cfg(test)]pub(crate)fn fixture_inference_projection(settings:Settings,configuration:Configuration,pet_id:&str,metadata:&serde_json::Value,observed_at:u64,now:u64)->NativeProjection{
 let mut registry=Registry::new().unwrap();let p=&configuration.pets[pet_id];let b=binding(&settings,pet_id,p,true);
 let model=if b.adapter==Some("lmstudio"){None}else{b.model.clone()};let k=key(&settings,&b,p,model).unwrap();
 let evidence=if let Some(choice)=&p.inference{if choice.backend==crate::pet_inference::Backend::Lmstudio{
  assert_eq!(choice.credential,crate::pet_inference::CredentialReference::None,"fixture does not read keys");
  Evidence::LmStudio(crate::local_chat::lmstudio::Catalog::parse(&crate::local_chat::Server{id:"lmstudio",name:"Synthetic registry",url:choice.endpoint.clone(),key:None},metadata).unwrap())
 }else{cache::model_evidence(metadata)}}else{panic!("explicit inference fixture required")};
 let ticket=registry.cache.begin(k,observed_at).unwrap();assert!(registry.cache.finish(ticket,evidence,observed_at));
 let snapshot=registry.snapshot(&settings,Some(&configuration),true,&BTreeSet::new(),now).unwrap();NativeProjection{snapshot,settings,configuration:Some(configuration),preferences_ready:true}
}
