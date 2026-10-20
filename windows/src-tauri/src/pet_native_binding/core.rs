//! Bounded accepted-intent state. No GTK, Shared.settings, emit or await here.
use super::ipc::*;
use crate::{authority_fence::{AuthorityFence,NativeWindowOwner,WindowRef},executor,lifecycle};
use std::collections::VecDeque;
use std::sync::{Arc,Mutex,MutexGuard,Weak,atomic::{AtomicU64,Ordering}};

struct Sequence(AtomicU64);
impl Sequence {
    fn next(&self)->Result<u64,Failure>{
        let mut old=self.0.load(Ordering::Relaxed);
        loop {
            let new=old.checked_add(1).ok_or_else(||Failure::binding(ErrorCode::Capacity,None))?;
            match self.0.compare_exchange_weak(old,new,Ordering::Relaxed,Ordering::Relaxed){Ok(_)=>return Ok(new),Err(n)=>old=n}
        }
    }
}
#[derive(Clone,Copy)]
pub(crate) struct Facts {pub(crate) visible:bool,pub(crate) mapped:bool}
#[derive(Clone,Copy,Default)]
pub(crate) struct Hint {pub(crate) x:f64,pub(crate) y:f64,pub(crate) w:f64,pub(crate) h:f64,pub(crate) keyboard:bool}
#[derive(Clone,Debug,PartialEq)]
pub(crate) struct Layout {pub(crate) rects:Vec<(f64,f64,f64,f64)>,pub(crate) expanded:bool,pub(crate) keyboard:bool}
#[derive(Clone)]
pub(crate) struct Plan {
    pub(crate) incarnation:u64,
    pub(crate) document:u64,
    pub(crate) key:u64,
    pub(crate) target:Mode,
    pub(crate) intent_id:String,
    request:Option<NavigationRequest>,
}
pub(crate) enum Begin {Duplicate(NavigationReceipt),Plan(Plan,Context)}
struct Seen {request:NavigationRequest}
struct State {
    alive:bool,document:u64,revision:Revision,ready:bool,loading:bool,crashed:bool,
    visible:bool,mapped:bool,enabled:bool,mode:Mode,pending:Option<Plan>,
    seen:VecDeque<Seen>,hint:Hint,
}
pub(crate) struct Core {
    pub(crate) incarnation:u64,
    owner:Arc<NativeWindowOwner>,
    state:Mutex<State>,
    sequence:Arc<Sequence>,
}
/// Backend-only immutable preparation; neither Clone nor serde nor native pointers.
pub(crate) struct PreparedContext {
    prepared:executor::Prepared,
    core:Weak<Core>,
    incarnation:u64,document:u64,revision:Revision,
}
impl PreparedContext {
    pub(crate) fn into_prepared(self)->executor::Prepared {self.prepared}
    /// Future strict audio seam only. No audio command is changed or called.
    pub(crate) fn validate(&self)->Result<(),Failure>{
        let core=self.core.upgrade().ok_or_else(Failure::unavailable)?;
        let state=core.lock()?;
        if core.incarnation!=self.incarnation||state.document!=self.document||state.revision!=self.revision {
            return Err(core.error(ErrorCode::StaleContext,&state));
        }
        core.eligible(&state)?;
        self.prepared.validate().map_err(Failure::network)
    }
}
impl Core {
    fn new(setting:&lifecycle::SystemSetting,binding:&Arc<AuthorityFence>,sequence:Arc<Sequence>,enabled:bool)->Result<Arc<Self>,Failure>{
        let incarnation=sequence.next()?;
        let revision=Revision(sequence.next()?);
        let native=Arc::new(lifecycle::SystemWindowAuthority::from_backend(lifecycle::WindowKind::Other,false,setting));
        Ok(Arc::new(Self {incarnation,owner:NativeWindowOwner::from_backend(native,binding),sequence,
            state:Mutex::new(State {alive:true,document:1,revision,ready:false,loading:false,crashed:false,visible:false,mapped:false,enabled,
                mode:Mode::Inactive,pending:None,seen:VecDeque::new(),hint:Hint::default()})}))
    }
    fn lock(&self)->Result<MutexGuard<'_,State>,Failure>{
        self.state.lock().map_err(|_|{self.owner.close_from_backend();Failure::unavailable()})
    }
    fn context_of(&self,s:&State)->Context {
        Context {schema_version:1,revision:s.revision,mode:s.mode,pending:s.pending.is_some(),native_available:s.alive,
            native_visible:s.visible,native_mapped:s.mapped,document_ready:s.ready,system_enabled:s.enabled,
            system_eligible:s.alive&&s.ready&&s.visible&&s.mapped&&s.enabled&&s.pending.is_none()&&s.mode.system(),
            authority_contract:"nativeIdentityAndAcceptedIntent"}
    }
    pub(crate) fn context(&self)->Result<Context,Failure>{let s=self.lock()?;Ok(self.context_of(&s))}
    fn error(&self,code:ErrorCode,s:&State)->Failure{Failure::binding(code,Some(self.context_of(s)))}
    fn bump(&self,s:&mut State)->Result<(),Failure>{
        match self.sequence.next(){Ok(n)=>{s.revision=Revision(n);Ok(())},Err(e)=>{s.alive=false;s.mode=Mode::Inactive;s.pending=None;self.owner.close_from_backend();Err(e)}}
    }
    fn sync_authority(&self,s:&State){
        let system=s.alive&&s.ready&&s.enabled&&s.pending.is_none()&&s.mode.system();
        self.owner.presentation_from_backend(if system{lifecycle::WindowKind::System}else{lifecycle::WindowKind::Other});
        // Actual native surface facts remain distinct from accepted purpose.
        self.owner.visibility_from_backend(s.alive&&s.visible&&s.mapped);
    }
    fn revoke_purpose(&self){self.owner.presentation_from_backend(lifecycle::WindowKind::Other);}
    fn eligible(&self,s:&State)->Result<(),Failure>{
        if !s.alive{return Err(self.error(ErrorCode::Unavailable,s));}
        if !s.ready{return Err(self.error(ErrorCode::DocumentInactive,s));}
        if !s.enabled{return Err(self.error(ErrorCode::SystemDisabled,s));}
        if s.pending.is_some(){return Err(self.error(ErrorCode::TransitionPending,s));}
        if !s.mode.system()||!s.visible||!s.mapped{return Err(self.error(ErrorCode::StaleContext,s));}
        Ok(())
    }
    pub(crate) fn prepare(self:&Arc<Self>,client:&executor::Client,expected:Revision)->Result<PreparedContext,Failure>{
        let state=self.lock()?;
        if state.revision!=expected{return Err(self.error(ErrorCode::StaleContext,&state));}
        self.eligible(&state)?;
        // Same Core section as expected context validation; sync capture ONCE.
        let prepared=client.prepare(&WindowRef::from_backend(&self.owner)).map_err(Failure::network)?;
        Ok(PreparedContext {prepared,core:Arc::downgrade(self),incarnation:self.incarnation,document:state.document,revision:state.revision})
    }
    pub(crate) fn observe_native(&self,facts:Facts)->Result<Option<Context>,Failure>{
        let mut s=self.lock()?;
        if !s.alive{return Ok(None);}
        if (s.visible,s.mapped)==(facts.visible,facts.mapped){return Ok(None);}
        let lost=s.visible&&s.mapped&&!(facts.visible&&facts.mapped);
        if lost {
            self.owner.visibility_from_backend(false);
            s.hint.keyboard=false;
            if s.pending.as_ref().is_some_and(|p|p.target.expanded()) {
                self.revoke_purpose();s.pending=None;s.mode=Mode::Inactive;
            }
        }
        s.visible=facts.visible;s.mapped=facts.mapped;self.bump(&mut s)?;self.sync_authority(&s);
        Ok(Some(self.context_of(&s)))
    }
    pub(crate) fn document(&self,loading:bool,allowed_ready:bool)->Result<Option<Context>,Failure>{
        let mut s=self.lock()?;
        if !s.alive{return Ok(None);}
        if s.crashed{return Ok(None);}
        if loading {
            if s.loading&&!s.ready{return Ok(None);}
            self.revoke_purpose();s.mode=Mode::Inactive;s.pending=None;s.seen.clear();s.ready=false;s.loading=true;
            s.document=s.document.checked_add(1).ok_or_else(||{self.owner.close_from_backend();Failure::unavailable()})?;
        }else if allowed_ready {
            if s.ready&&!s.loading{return Ok(None);}
            s.ready=true;s.loading=false;
        }else {
            if !s.ready&&!s.loading{return Ok(None);}
            self.revoke_purpose();s.mode=Mode::Inactive;s.pending=None;s.seen.clear();s.ready=false;s.loading=false;
            s.document=s.document.checked_add(1).ok_or_else(||{self.owner.close_from_backend();Failure::unavailable()})?;
        }
        self.bump(&mut s)?;self.sync_authority(&s);Ok(Some(self.context_of(&s)))
    }
    /// Only an explicit native WebKit/Tauri load START event clears a crash.
    /// Polling an old URI/is-loading value can never recover a crashed document.
    pub(crate) fn load_started(&self)->Result<Option<Context>,Failure>{
        let mut s=self.lock()?;if !s.alive{return Ok(None);}
        if !s.crashed&&s.loading&&!s.ready{return Ok(None);}
        self.revoke_purpose();s.crashed=false;s.mode=Mode::Inactive;s.pending=None;s.seen.clear();s.ready=false;s.loading=true;
        s.document=s.document.checked_add(1).ok_or_else(||{self.owner.close_from_backend();Failure::unavailable()})?;
        self.bump(&mut s)?;self.sync_authority(&s);Ok(Some(self.context_of(&s)))
    }
    pub(crate) fn crashed(&self)->Result<Context,Failure>{
        let mut s=self.lock()?;if !s.alive{return Err(self.error(ErrorCode::Unavailable,&s));}self.revoke_purpose();s.mode=Mode::Inactive;s.pending=None;s.seen.clear();s.ready=false;s.loading=false;s.crashed=true;
        s.document=s.document.checked_add(1).ok_or_else(Failure::unavailable)?;self.bump(&mut s)?;self.sync_authority(&s);Ok(self.context_of(&s))
    }
    pub(crate) fn settings_committed(&self,enabled:bool)->Result<Option<Context>,Failure>{
        let mut s=self.lock()?;if !s.alive||s.enabled==enabled{return Ok(None);}
        self.revoke_purpose();s.enabled=enabled;s.pending=None;
        if s.mode.system(){s.mode=Mode::Compact;}
        self.bump(&mut s)?;self.sync_authority(&s);Ok(Some(self.context_of(&s)))
    }
    pub(crate) fn close(&self)->Result<Context,Failure>{
        let mut s=self.lock()?;if !s.alive{return Ok(self.context_of(&s));}self.owner.close_from_backend();s.alive=false;s.visible=false;s.mapped=false;s.ready=false;s.pending=None;s.mode=Mode::Inactive;
        self.bump(&mut s)?;Ok(self.context_of(&s))
    }
    pub(crate) fn begin(&self,request:NavigationRequest)->Result<Begin,Failure>{
        let mut s=self.lock()?;
        if !s.alive{return Err(self.error(ErrorCode::Unavailable,&s));}
        if let Some(pending)=s.pending.as_ref().and_then(|p|p.request.as_ref()).filter(|p|p.intent_id==request.intent_id){
            if pending!=&request{return Err(self.error(ErrorCode::IntentConflict,&s));}
            return Ok(Begin::Duplicate(NavigationReceipt {intent_id:request.intent_id.as_str().into(),applied:false,context:self.context_of(&s)}));
        }
        if let Some(seen)=s.seen.iter().find(|v|v.request.intent_id==request.intent_id){
            if seen.request!=request{return Err(self.error(ErrorCode::IntentConflict,&s));}
            return Ok(Begin::Duplicate(NavigationReceipt {intent_id:request.intent_id.as_str().into(),applied:false,context:self.context_of(&s)}));
        }
        if s.revision!=request.expected_revision{return Err(self.error(ErrorCode::StaleContext,&s));}
        if !s.ready{return Err(self.error(ErrorCode::DocumentInactive,&s));}
        if !s.visible||!s.mapped{return Err(self.error(ErrorCode::Unavailable,&s));}
        let target=match request.intent {NavigationIntent::Expand{route}|NavigationIntent::Navigate{route}=>Mode::Expanded{route},NavigationIntent::Collapse{}=>Mode::Compact};
        if target.system()&&!s.enabled{return Err(self.error(ErrorCode::SystemDisabled,&s));}
        self.start(&mut s,target,request.intent_id.as_str().into(),Some(request))
    }
    pub(crate) fn begin_trusted(&self,target:Mode)->Result<(Plan,Context),Failure>{
        let mut s=self.lock()?;
        if !s.alive{return Err(self.error(ErrorCode::Unavailable,&s));}
        if target.expanded()&&!s.ready{return Err(self.error(ErrorCode::DocumentInactive,&s));}
        if target.system()&&!s.enabled{return Err(self.error(ErrorCode::SystemDisabled,&s));}
        match self.start(&mut s,target,String::new(),None)? {Begin::Plan(p,c)=>Ok((p,c)),_=>unreachable!()}
    }
    fn start(&self,s:&mut State,target:Mode,intent_id:String,request:Option<NavigationRequest>)->Result<Begin,Failure>{
        // Pending/Other is a real backend policy transition, not fake GTK hide.
        self.revoke_purpose();s.hint.keyboard=false;self.bump(s)?;
        let intent_id=if intent_id.is_empty(){format!("internal-{}",s.revision.0)}else{intent_id};
        let plan=Plan {incarnation:self.incarnation,document:s.document,key:s.revision.0,target,intent_id,request};
        s.pending=Some(plan.clone());self.sync_authority(s);Ok(Begin::Plan(plan,self.context_of(s)))
    }
    pub(crate) fn current_plan(&self,plan:&Plan)->bool {
        self.lock().is_ok_and(|s|s.alive&&self.incarnation==plan.incarnation&&s.document==plan.document&&s.pending.as_ref().is_some_and(|p|p.key==plan.key))
    }
    pub(crate) fn finish(&self,plan:&Plan,native_ok:bool)->Result<NavigationReceipt,Failure>{
        let mut s=self.lock()?;
        if !s.alive||self.incarnation!=plan.incarnation||s.document!=plan.document||!s.pending.as_ref().is_some_and(|p|p.key==plan.key){return Err(self.error(ErrorCode::Superseded,&s));}
        let valid=native_ok&&(!plan.target.expanded()||(s.ready&&s.visible&&s.mapped))&&(!plan.target.system()||s.enabled);
        self.revoke_purpose();s.pending=None;s.mode=if valid{plan.target}else{Mode::Inactive};self.bump(&mut s)?;self.sync_authority(&s);
        if !valid{return Err(self.error(ErrorCode::NativeFailure,&s));}
        if let Some(request)=plan.request.clone(){if s.seen.len()>=128{s.seen.pop_front();}s.seen.push_back(Seen{request});}
        Ok(NavigationReceipt {intent_id:plan.intent_id.clone(),applied:true,context:self.context_of(&s)})
    }
    pub(crate) fn layout(&self,hint:Option<Hint>,target:Option<&Plan>)->Result<Layout,Failure>{
        let mut s=self.lock()?;
        if let Some(h)=hint {
            if ![h.x,h.y,h.w,h.h].iter().all(|v|v.is_finite()&&*v>=0.0&&*v<10000.0){return Err(self.error(ErrorCode::InvalidRequest,&s));}
            s.hint=h;
        }
        self.layout_of(&s,target)
    }
    pub(crate) fn policy_snapshot(&self)->Result<super::native_policy::Snapshot,Failure>{
        let s=self.lock()?;let mut layout=self.layout_of(&s,None)?;
        if !s.alive||!s.visible||!s.mapped||!s.ready{layout=Layout{rects:Vec::new(),expanded:false,keyboard:false};}
        Ok(super::native_policy::Snapshot{context:self.context_of(&s),layout})
    }
    fn layout_of(&self,s:&State,target:Option<&Plan>)->Result<Layout,Failure>{
        let mode=if let Some(p)=target {
            if !s.pending.as_ref().is_some_and(|a|a.key==p.key&&a.document==p.document){return Err(self.error(ErrorCode::Superseded,&s));}p.target
        }else if s.pending.is_some(){Mode::Compact}else{s.mode};
        let expanded=mode.expanded();let h=s.hint;let max_h=if expanded{700.0}else{32.0};
        let rect=(h.x.min(760.0),h.y.min(max_h),h.w.min((760.0-h.x).max(0.0)),h.h.min((max_h-h.y).max(0.0)));
        let rects=if matches!(mode,Mode::Inactive)||!s.alive{Vec::new()}else{vec![rect]};
        // Legacy expanded/keyboard cannot select mode. Keyboard only within an
        // already accepted expanded context; a transition always resets it.
        let keyboard=target.is_none()&&s.pending.is_none()&&expanded&&s.visible&&s.mapped&&h.keyboard;
        Ok(Layout {rects,expanded,keyboard})
    }
}

struct ManagerState {enabled:bool,closed:bool,current:Option<Arc<Core>>}
pub(crate) struct Manager {
    setting:lifecycle::SystemSetting,
    pub(crate) binding:Arc<AuthorityFence>,
    sequence:Arc<Sequence>,
    state:Mutex<ManagerState>,
    network:Mutex<Option<executor::Executor>>,
}
impl Manager {
    fn lock(&self)->Result<MutexGuard<'_,ManagerState>,Failure>{
        self.state.lock().map_err(|_|{self.binding.update_from_backend(false);Failure::unavailable()})
    }

    pub(crate) fn new(enabled:bool)->Arc<Self>{
        let setting=lifecycle::SystemSetting::from_backend(enabled);let binding=AuthorityFence::from_backend(setting.clone());binding.update_from_backend(enabled);
        Arc::new(Self {setting,binding,sequence:Arc::new(Sequence(AtomicU64::new(0))),state:Mutex::new(ManagerState {enabled,closed:false,current:None}),network:Mutex::new(None)})
    }
    /// Called only by the native attachment adapter after actual object capture.
    pub(super) fn register(&self)->Result<Arc<Core>,Failure>{
        let mut state=self.lock()?;
        if state.closed{return Err(Failure::unavailable());}
        if let Some(old)=state.current.take(){let _=old.close();}
        let core=Core::new(&self.setting,&self.binding,self.sequence.clone(),state.enabled)?;
        state.current=Some(core.clone());Ok(core)
    }
    pub(crate) fn current(&self)->Result<Arc<Core>,Failure>{
        self.lock()?.current.clone().ok_or_else(Failure::unavailable)
    }
    pub(crate) fn with_current<T>(&self,incarnation:u64,apply:impl FnOnce(&Arc<Core>)->Result<T,Failure>)->Result<T,Failure>{
        let state=self.lock()?;
        if state.closed{return Err(Failure::unavailable());}
        let core=state.current.as_ref().filter(|c|c.incarnation==incarnation).ok_or_else(Failure::unavailable)?;
        apply(core)
    }
    pub(crate) fn detach(&self,incarnation:u64)->Result<Option<Context>,Failure>{
        let mut state=self.lock()?;
        if state.current.as_ref().is_none_or(|c|c.incarnation!=incarnation){return Ok(None);}
        let current=state.current.take().unwrap();current.close().map(Some)
    }
    pub(crate) fn settings_committed(&self,enabled:bool)->Result<Option<Context>,Failure>{
        let mut state=self.lock()?;
        if state.closed||state.enabled==enabled{return Ok(None);}
        // Invoked by the durable settings publish transaction, BEFORE events.
        self.binding.update_from_backend(enabled);state.enabled=enabled;
        state.current.as_ref().map(|c|c.settings_committed(enabled)).transpose().map(|v|v.flatten())
    }
    pub(crate) fn install_network(&self,owner:executor::Executor)->Result<(),Failure>{
        let state=self.lock()?;
        if state.closed{return Err(Failure::unavailable());}
        let mut slot=self.network.lock().map_err(|_|Failure::unavailable())?;
        if slot.is_some(){return Err(Failure::unavailable());}*slot=Some(owner);Ok(())
    }
    pub(crate) fn client(&self)->Result<executor::Client,Failure>{
        if self.lock()?.closed{return Err(Failure::unavailable());}
        self.network.lock().map_err(|_|Failure::unavailable())?.as_ref().map(|o|o.client()).ok_or_else(Failure::unavailable)
    }
    pub(crate) fn shutdown(&self){
        let owner=if let Ok(mut state)=self.state.lock(){state.closed=true;if let Some(core)=state.current.take(){let _=core.close();}self.network.lock().ok().and_then(|mut s|s.take())}else{self.binding.update_from_backend(false);None};
        drop(owner);
    }
}
impl Drop for Manager {fn drop(&mut self){self.shutdown();}}

/// Used by the GTK adapter with live GObject refs; integer fixtures only test this
/// generic equality policy and never pretend to instantiate native GTK objects.
pub(crate) fn same_objects<G:PartialEq,W:PartialEq>(registered_gtk:&G,registered_webkit:&W,caller_gtk:&G,caller_webkit:&W)->bool {
    registered_gtk==caller_gtk&&registered_webkit==caller_webkit
}
#[cfg(test)]
pub(crate) fn fixture_register(manager:&Manager)->Arc<Core>{manager.register().unwrap()}
#[cfg(all(test,target_os="linux"))]
#[path="core_tests.rs"]
mod tests;
