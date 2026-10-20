//! Backend request-context fence around EXISTING verified lifecycle authorities.
//! Does not acquire Tauri/GTK/router proof or create it from renderer flags.
use crate::lifecycle as l;
use std::sync::{Arc,Mutex,Weak};

struct Global { epoch:u64, enabled:Option<bool>, closed:bool }
/// Root must route actual committed settings changes through this ONE wrapper.
/// Keeping independent wrappers of the same setting is rejected by pointer identity.
pub(crate) struct AuthorityFence { setting:l::SystemSetting, state:Mutex<Global> }
impl AuthorityFence {
    pub(crate) fn from_backend(setting:l::SystemSetting)->Arc<Self> {
        Arc::new(Self {setting,state:Mutex::new(Global {epoch:1,enabled:None,closed:false})})
    }
    pub(crate) fn update_from_backend(&self,enabled:bool) {
        let Ok(mut state)=self.state.lock() else {self.setting.update_from_backend(false);return;};
        if state.closed {return;}
        if state.enabled!=Some(enabled) {
            let Some(epoch)=state.epoch.checked_add(1) else {state.closed=true;self.setting.update_from_backend(false);return;};
            state.epoch=epoch;state.enabled=Some(enabled);
        }
        self.setting.update_from_backend(enabled);
    }
    pub(crate) fn service(&self,config:l::Config,nonce:u128)->Result<l::Service,l::Reason> {
        if self.state.lock().map_err(|_|l::Reason::Closed)?.closed {return Err(l::Reason::Closed);}
        l::Service::new(config,nonce,&self.setting)
    }
    #[cfg(test)]
    pub(crate) fn fixture_setting(&self)->l::SystemSetting {self.setting.clone()}
}
struct WindowState {epoch:u64,visible:Option<bool>,kind:Option<l::WindowKind>,closed:bool}
/// This owns an ALREADY backend-minted authority, not a native GTK window.
/// Root owns this wrapper and supplies actual native event/router evidence.
pub(crate) struct NativeWindowOwner {
    native:Arc<l::SystemWindowAuthority>,
    binding:Arc<AuthorityFence>,
    state:Mutex<WindowState>,
}
impl NativeWindowOwner {
    pub(crate) fn from_backend(native:Arc<l::SystemWindowAuthority>,binding:&Arc<AuthorityFence>)->Arc<Self> {
        Arc::new(Self {native,binding:binding.clone(),state:Mutex::new(WindowState {epoch:1,visible:None,kind:None,closed:false})})
    }
    pub(crate) fn visibility_from_backend(&self,visible:bool) {
        let Ok(mut state)=self.state.lock() else {self.native.close_from_backend();return;};
        if state.closed {return;}
        if state.visible!=Some(visible) {
            let Some(epoch)=state.epoch.checked_add(1) else {state.closed=true;self.native.close_from_backend();return;};
            state.epoch=epoch;state.visible=Some(visible);
        }
        self.native.visibility_from_backend(visible);
    }
    pub(crate) fn presentation_from_backend(&self,kind:l::WindowKind) {
        let Ok(mut state)=self.state.lock() else {self.native.close_from_backend();return;};
        if state.closed {return;}
        if state.kind!=Some(kind) {
            let Some(epoch)=state.epoch.checked_add(1) else {state.closed=true;self.native.close_from_backend();return;};
            state.epoch=epoch;state.kind=Some(kind);
        }
        self.native.presentation_from_backend(kind);
    }
    pub(crate) fn close_from_backend(&self) {
        if let Ok(mut state)=self.state.lock() {state.closed=true;}
        self.native.close_from_backend();
    }
}
impl Drop for NativeWindowOwner {fn drop(&mut self){self.native.close_from_backend();}}

/// Root-issued handle only. A weak owner does not retain native authority lifetime.
#[derive(Clone)]
pub(crate) struct WindowRef(Weak<NativeWindowOwner>);
impl WindowRef {
    pub(crate) fn from_backend(owner:&Arc<NativeWindowOwner>)->Self {Self(Arc::downgrade(owner))}
    pub(crate) fn capture(&self,expected:&Arc<AuthorityFence>)->Result<Stamp,l::Reason> {
        let owner=self.0.upgrade().ok_or(l::Reason::Closed)?;
        if !Arc::ptr_eq(&owner.binding,expected) {return Err(l::Reason::Unauthorized);}
        // Lock order is always global -> window -> existing lifecycle state.
        let global=expected.state.lock().map_err(|_|l::Reason::Closed)?;
        let window=owner.state.lock().map_err(|_|l::Reason::Closed)?;
        if global.closed||window.closed {return Err(l::Reason::Closed);}
        Ok(Stamp {owner:self.0.clone(),global_epoch:global.epoch,window_epoch:window.epoch})
    }
    #[cfg(test)]
    pub(crate) fn owner_gone(&self)->bool {self.0.upgrade().is_none()}
}
/// Immutable backend request context, captured BEFORE it enters the actor queue.
/// No constructor/fields/serde/native facts are exposed to a renderer.
#[derive(Clone)]
pub(crate) struct Stamp {owner:Weak<NativeWindowOwner>,global_epoch:u64,window_epoch:u64}
impl Stamp {
    /// Locks remain held through the synchronous admission/dispatch/publication.
    /// Nested Result keeps fence denial distinct from lifecycle guard denial.
    pub(crate) fn run<T>(&self,expected:&Arc<AuthorityFence>,apply:impl FnOnce(&l::SystemWindowAuthority)->T)->Result<T,l::Reason> {
        let owner=self.owner.upgrade().ok_or(l::Reason::Closed)?;
        if !Arc::ptr_eq(&owner.binding,expected) {return Err(l::Reason::Unauthorized);}
        let global=expected.state.lock().map_err(|_|l::Reason::Closed)?;
        let window=owner.state.lock().map_err(|_|l::Reason::Closed)?;
        if global.closed||window.closed {return Err(l::Reason::Closed);}
        if global.epoch!=self.global_epoch||window.epoch!=self.window_epoch {return Err(l::Reason::Revoked);}
        Ok(apply(&owner.native))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context()->(Arc<AuthorityFence>,Arc<NativeWindowOwner>,WindowRef) {
        let setting=l::SystemSetting::from_backend(true);let binding=AuthorityFence::from_backend(setting.clone());
        let native=Arc::new(l::SystemWindowAuthority::from_backend(l::WindowKind::System,true,&setting));
        let owner=NativeWindowOwner::from_backend(native,&binding);let window=WindowRef::from_backend(&owner);(binding,owner,window)
    }
    #[test]fn old_window_setting_and_route_epochs_do_not_revive(){
        let(b,o,w)=context();let old=w.capture(&b).unwrap();o.visibility_from_backend(false);o.visibility_from_backend(true);assert_eq!(old.run(&b,|_|()),Err(l::Reason::Revoked));
        let old=w.capture(&b).unwrap();b.update_from_backend(false);b.update_from_backend(true);assert_eq!(old.run(&b,|_|()),Err(l::Reason::Revoked));
        let old=w.capture(&b).unwrap();o.presentation_from_backend(l::WindowKind::Other);o.presentation_from_backend(l::WindowKind::System);assert_eq!(old.run(&b,|_|()),Err(l::Reason::Revoked));
        assert_eq!(w.capture(&b).unwrap().run(&b,|_|()),Ok(()));
    }
    #[test]fn context_cannot_move_to_foreign_binding_even_with_same_underlying_setting(){
        let(b,_owner,w)=context();let other=AuthorityFence::from_backend(b.fixture_setting());assert!(matches!(w.capture(&other),Err(l::Reason::Unauthorized)));
    }
    #[test]fn window_and_global_counter_overflow_close_authority(){
        let(b,o,w)=context();o.state.lock().unwrap().epoch=u64::MAX;o.visibility_from_backend(false);assert!(matches!(w.capture(&b),Err(l::Reason::Closed)));
        let(b,_owner,w)=context();b.state.lock().unwrap().epoch=u64::MAX;b.update_from_backend(false);assert!(matches!(w.capture(&b),Err(l::Reason::Closed)));
    }
    #[test]fn window_and_global_fences_stay_locked_through_actual_service_admission(){
        let(b,o,w)=context();let stamp=w.capture(&b).unwrap();let service=b.service(l::Config::default(),17).unwrap();
        let admission=stamp.run(&b,|native|{
            assert!(matches!(b.state.try_lock(),Err(std::sync::TryLockError::WouldBlock)));
            assert!(matches!(o.state.try_lock(),Err(std::sync::TryLockError::WouldBlock)));
            service.read(native,true,std::time::Instant::now())
        }).unwrap().unwrap();
        o.close_from_backend();let mut work=admission.work.unwrap();let peer=l::ProducerId::from_verified_backend([1;16]).unwrap();
        assert_eq!(work.begin_dispatch(peer,std::time::Instant::now()),Err(l::Reason::Unauthorized));
    }
}
