//! Native identity for human reviews, below data/runtime authority. No grant
//! is minted from a label or from renderer-supplied readiness/visibility.
use super::{Owner,Frame};
fn page(label:&str)->Option<&'static str>{match label{
    crate::HUB_LABEL=>Some("/settings.html"),crate::companions::LABEL=>Some("/companions.html"),
    crate::pet_control::LABEL=>Some("/pet-control.html"),crate::pet_panel::LABEL=>Some("/pet-manager.html"),_=>None,
}}
fn trusted(url:&tauri::Url,page:&str,dev:Option<&tauri::Url>)->bool{
    url.username().is_empty()&&url.password().is_none()&&url.query().is_none()&&url.path()==page
        &&((url.scheme()=="tauri"&&url.host_str()==Some("localhost")&&url.port().is_none())||dev.is_some_and(|d|d.origin()==url.origin()))
}
pub(super)fn register(window:&tauri::WebviewWindow){
    if page(window.label()).is_none(){return;}
    let owner=if window.label()==crate::HUB_LABEL{super::controller().register()}else{Owner::new()};
    let o=owner.clone();window.on_window_event(move|event|{
        if matches!(event,tauri::WindowEvent::CloseRequested{..}|tauri::WindowEvent::Destroyed){retire(&o);}
    });
    #[cfg(target_os="linux")]linux::register(window,owner);
    #[cfg(not(target_os="linux"))]let _=owner;
}
fn retire(owner:&Owner){owner.invalidate();crate::pet_runtime::recovery_review_context_changed();}
pub(super)async fn capture(window:&tauri::WebviewWindow)->Result<Frame,()>{
    #[cfg(target_os="linux")]{linux::capture(window).await}
    #[cfg(not(target_os="linux"))]{let _=window;Err(())}
}
#[cfg(target_os="linux")]mod linux{
    use super::*;
    use gtk::{prelude::*,glib::{self,translate::{ToGlibPtr,IntoGlib}}};
    use std::cell::RefCell;
    #[cfg(dev)]use tauri::Manager;
    struct Entry{label:String,gtk:glib::WeakRef<gtk::ApplicationWindow>,view:glib::WeakRef<gtk::Widget>,owner:Owner,signals:Vec<(glib::WeakRef<gtk::Widget>,glib::SignalHandlerId)>}
    impl Drop for Entry{fn drop(&mut self){retire(&self.owner);for(object,id)in self.signals.drain(..){if let Some(object)=object.upgrade(){object.disconnect(id);}}}}
    thread_local!{static ENTRIES:RefCell<Vec<Entry>>=const{RefCell::new(Vec::new())};}
    #[derive(Clone)]struct Policy{page:&'static str,dev:Option<tauri::Url>}
    impl Policy{
        fn new(window:&tauri::WebviewWindow)->Option<Self>{
            let page=page(window.label())?;
            #[cfg(dev)]let dev=window.app_handle().config().build.dev_url.clone();
            #[cfg(not(dev))]let dev=None;
            Some(Self{page,dev})
        }
        fn ready(&self,view:&gtk::Widget,window:&gtk::ApplicationWindow)->bool{
            window.is_visible()&&window.is_mapped()&&view.is_visible()&&view.is_mapped()
                &&view.find_property("is-loading").is_some_and(|p|p.value_type()==bool::static_type())&&!view.property::<bool>("is-loading")
                &&view.find_property("uri").is_some()&&view.property::<Option<String>>("uri").and_then(|s|tauri::Url::parse(&s).ok()).is_some_and(|u|trusted(&u,self.page,self.dev.as_ref()))
        }
    }
    pub(super)fn register(window:&tauri::WebviewWindow,owner:Owner){
        let label=window.label().to_owned();let Some(_policy)=Policy::new(window)else{return;};let failed=owner.clone();
        let result=window.with_webview(move|platform|{
            let view=platform.inner().upcast::<gtk::Widget>();let Some(gw)=view.toplevel().and_then(|v|v.downcast::<gtk::ApplicationWindow>().ok())else{retire(&owner);return;};
            if view.find_property("is-loading").is_none_or(|p|p.value_type()!=bool::static_type())||view.find_property("uri").is_none(){retire(&owner);return;}
            for signal in [c"load-changed",c"web-process-terminated"]{if unsafe{glib::gobject_ffi::g_signal_lookup(signal.as_ptr(),view.type_().into_glib())}==0{retire(&owner);return;}}
            let mut signals=Vec::new();
            for object in [view.clone(),gw.clone().upcast::<gtk::Widget>()]{
                let o=owner.clone();signals.push((object.downgrade(),object.connect_destroy(move|_|retire(&o))));
                let o=owner.clone();signals.push((object.downgrade(),object.connect_unmap(move|_|retire(&o))));
            }
            let o=owner.clone();signals.push((view.downgrade(),view.connect_local("load-changed",false,move|values|{
                let event=values.get(1).filter(|v|v.type_().is_a(glib::Type::ENUM)).map(|v|unsafe{glib::gobject_ffi::g_value_get_enum(v.to_glib_none().0)});
                if event==Some(0){retire(&o);}None
            })));
            let o=owner.clone();signals.push((view.downgrade(),view.connect_local("web-process-terminated",false,move|_|{retire(&o);None})));
            ENTRIES.with(|entries|{let mut entries=entries.borrow_mut();entries.retain(|e|e.label!=label);assert!(entries.len()<4);entries.push(Entry{label,gtk:gw.downgrade(),view:view.downgrade(),owner,signals});});
        });
        if result.is_err(){retire(&failed);crate::log::line("native review window registration unavailable");}
    }
    pub(super)async fn capture(window:&tauri::WebviewWindow)->Result<Frame,()>{
        let label=window.label().to_owned();let policy=Policy::new(window).ok_or(())?;let(tx,rx)=tokio::sync::oneshot::channel();
        window.with_webview(move|platform|{
            let frame=ENTRIES.with(|entries|{
                let entries=entries.borrow();let e=entries.iter().find(|e|e.label==label)?;let view=platform.inner().upcast::<gtk::Widget>();let gw=view.toplevel()?.downcast::<gtk::ApplicationWindow>().ok()?;
                let(expected_view,expected_gtk)=(e.view.upgrade()?,e.gtk.upgrade()?);
                if !crate::pet_native_binding::core::same_objects(&expected_gtk,&expected_view,&gw,&view){return None;}
                if !policy.ready(&view,&gw){retire(&e.owner);return None;}
                let f=Frame{owner:e.owner.clone(),epoch:e.owner.epoch.load(std::sync::atomic::Ordering::Acquire)};f.current().then_some(f)
            });let _=tx.send(frame);
        }).map_err(|_|())?;
        tokio::time::timeout(std::time::Duration::from_secs(2),rx).await.map_err(|_|())?.map_err(|_|())?.ok_or(())
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn review_window_document_policy_is_exact_and_never_accepts_renderer_urls(){
        for label in [crate::HUB_LABEL,crate::companions::LABEL,crate::pet_control::LABEL,crate::pet_panel::LABEL]{
            let p=page(label).unwrap();assert!(trusted(&tauri::Url::parse(&format!("tauri://localhost{p}")).unwrap(),p,None));
            for bad in [format!("https://example.test{p}"),format!("tauri://elsewhere{p}"),format!("tauri://user@localhost{p}"),format!("tauri://localhost{p}?ready=true"),"tauri://localhost/other.html".into()]{assert!(!trusted(&tauri::Url::parse(&bad).unwrap(),p,None));}
        }assert!(page("renderer-supplied-window").is_none());
    }
}
