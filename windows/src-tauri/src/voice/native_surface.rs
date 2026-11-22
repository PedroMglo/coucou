//! Physical GTK/WebKit identity for the two existing speech surfaces. This
//! bounded lifetime registry is not a permission store or a System grant.
use super::native::{Frame,Surface};
use tauri::WebviewWindow;
pub(crate) fn register(window:&WebviewWindow){
    #[cfg(target_os="linux")]linux::register(window);
    #[cfg(not(target_os="linux"))]let _=window;
}
pub(super) async fn capture(window:&WebviewWindow)->Result<Frame,()>{
    #[cfg(target_os="linux")]{linux::capture(window).await}
    #[cfg(not(target_os="linux"))]{let _=window;Err(())}
}

#[cfg(target_os="linux")]
mod linux{
    use super::*;use super::super::native::Owner;
    use gtk::{prelude::*,glib::{self,translate::{ToGlibPtr,IntoGlib}}};
    use std::{cell::RefCell,sync::Arc};
    #[cfg(dev)]use tauri::Manager;
    struct Entry{surface:Surface,gtk:glib::WeakRef<gtk::ApplicationWindow>,view:glib::WeakRef<gtk::Widget>,owner:Arc<Owner>,signals:Vec<(glib::WeakRef<gtk::Widget>,glib::SignalHandlerId)>}
    impl Drop for Entry{fn drop(&mut self){self.owner.close();for(object,id)in self.signals.drain(..){if let Some(object)=object.upgrade(){object.disconnect(id);}}}}
    thread_local!{static ENTRIES:RefCell<Vec<Entry>>=const{RefCell::new(Vec::new())};}
    #[derive(Clone)]struct Policy{surface:Surface,dev:Option<tauri::Url>}
    impl Policy{
        fn for_window(window:&WebviewWindow,surface:Surface)->Self{
            #[cfg(dev)]let dev=window.app_handle().config().build.dev_url.clone();
            #[cfg(not(dev))]let dev=None;
            let _=window;Self{surface,dev}
        }
        fn ready(&self,view:&gtk::Widget)->bool{
            if view.find_property("is-loading").is_none()||view.find_property("uri").is_none()||view.property::<bool>("is-loading"){return false;}
            view.property::<Option<String>>("uri").and_then(|u|tauri::Url::parse(&u).ok()).is_some_and(|u|{
                u.username().is_empty()&&u.password().is_none()&&u.query().is_none()&&u.path()==self.surface.page()
                    &&((u.scheme()=="tauri"&&u.host_str()==Some("localhost")&&u.port().is_none())||self.dev.as_ref().is_some_and(|d|d.origin()==u.origin()))
            })
        }
    }
    pub(super) fn register(window:&WebviewWindow){
        let Some(surface)=Surface::from_label(window.label())else{return;};let policy=Policy::for_window(window,surface);
        let result=window.with_webview(move|platform|{
            let view=platform.inner().upcast::<gtk::Widget>();
            let Some(gw)=view.toplevel().and_then(|v|v.downcast::<gtk::ApplicationWindow>().ok())else{return;};
            if view.find_property("is-loading").is_none_or(|p|p.value_type()!=bool::static_type())||view.find_property("uri").is_none(){return;}
            for signal in [c"load-changed",c"web-process-terminated"]{
                if unsafe{glib::gobject_ffi::g_signal_lookup(signal.as_ptr(),view.type_().into_glib())}==0{return;}
            }
            let owner=Owner::new(surface);let mut signals=vec![];
            for object in [view.clone(),gw.clone().upcast::<gtk::Widget>()]{let o=owner.clone();signals.push((object.downgrade(),object.connect_destroy(move|_|o.close())));}
            let o=owner.clone();let weak=view.downgrade();let p=policy.clone();
            signals.push((view.downgrade(),view.connect_local("load-changed",false,move|values|{
                let event=values.get(1).filter(|v|v.type_().is_a(glib::Type::ENUM)).map(|v|unsafe{glib::gobject_ffi::g_value_get_enum(v.to_glib_none().0)});
                if event==Some(0){o.invalidate();}
                if event==Some(3){let o=o.clone();let weak=weak.clone();let p=p.clone();let epoch=o.generation();glib::idle_add_local_once(move||{
                    if o.generation()==epoch{o.ready(weak.upgrade().is_some_and(|v|p.ready(&v)));}
                });}None
            })));
            let o=owner.clone();signals.push((view.downgrade(),view.connect_local("web-process-terminated",false,move|_|{o.invalidate();None})));
            owner.ready(policy.ready(&view));
            ENTRIES.with(|entries|{let mut entries=entries.borrow_mut();entries.retain(|e|e.surface!=surface);assert!(entries.len()<2);entries.push(Entry{surface,gtk:gw.downgrade(),view:view.downgrade(),owner,signals});});
        });
        if result.is_err(){crate::log::line("voice: native speech surface registration unavailable");}
    }
    pub(super) async fn capture(window:&WebviewWindow)->Result<Frame,()>{
        let surface=Surface::from_label(window.label()).ok_or(())?;let policy=Policy::for_window(window,surface);let(tx,rx)=tokio::sync::oneshot::channel();
        window.with_webview(move|platform|{
            let frame=ENTRIES.with(|entries|{
                let entries=entries.borrow();let entry=entries.iter().find(|e|e.surface==surface)?;
                let view=platform.inner().upcast::<gtk::Widget>();let gw=view.toplevel()?.downcast::<gtk::ApplicationWindow>().ok()?;
                let(expected_view,expected_gtk)=(entry.view.upgrade()?,entry.gtk.upgrade()?);
                if !crate::pet_native_binding::core::same_objects(&expected_gtk,&expected_view,&gw,&view){return None;}
                if !policy.ready(&view){entry.owner.invalidate();return None;}
                entry.owner.frame()
            });let _=tx.send(frame);
        }).map_err(|_|())?;
        tokio::time::timeout(std::time::Duration::from_secs(2),rx).await.map_err(|_|())?.map_err(|_|())?.ok_or(())
    }
}
