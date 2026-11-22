//! GTK-thread only object registry and bounded native transitions.
//! No GTK object enters managed Send state or the synchronous IO writer guard.
use super::{core::{self,Begin,Facts,Hint,Manager,Plan,PreparedContext},ipc::*,Policy,Placement,native_policy::{self,Snapshot,Step,Surface}};
use gtk::prelude::*;
use gtk::glib::{self,translate::{IntoGlib,ToGlibPtr}};
use std::{cell::{Cell,RefCell},rc::Rc,sync::{Arc,Weak}};
use tauri::{AppHandle,WebviewWindow,Manager as _};

#[derive(Clone)]
struct Native {
    gtk:glib::WeakRef<gtk::ApplicationWindow>,view:glib::WeakRef<gtk::Widget>,
    incarnation:u64,manager:Weak<Manager>,policy:Policy,window:WebviewWindow,reconcile_queued:Rc<Cell<bool>>,
}
struct Entry {native:Native,signals:Vec<(glib::WeakRef<gtk::Widget>,glib::SignalHandlerId)>}
impl Drop for Entry {
    fn drop(&mut self){for (object,id) in self.signals.drain(..){if let Some(object)=object.upgrade(){object.disconnect(id);}}}
}
thread_local! {static REGISTRY:RefCell<Option<Entry>>=const {RefCell::new(None)};}
fn registered()->Result<Native,Failure>{
    REGISTRY.with(|slot|slot.borrow().as_ref().map(|e|e.native.clone())).ok_or_else(Failure::unavailable)
}
fn manager(native:&Native)->Result<Arc<Manager>,Failure>{native.manager.upgrade().ok_or_else(Failure::unavailable)}
fn objects(native:&Native)->Result<(gtk::ApplicationWindow,gtk::Widget),Failure>{
    let current=registered()?;
    if current.incarnation!=native.incarnation{return Err(Failure::unavailable());}
    Ok((native.gtk.upgrade().ok_or_else(Failure::unavailable)?,native.view.upgrade().ok_or_else(Failure::unavailable)?))
}
fn caller(platform:&tauri::webview::PlatformWebview)->Result<(Native,gtk::ApplicationWindow,gtk::Widget),Failure>{
    let native=registered()?;let(expected_gtk,expected_view)=objects(&native)?;
    let actual_view=platform.inner().upcast::<gtk::Widget>();
    let actual_gtk=actual_view.toplevel().and_then(|v|v.downcast::<gtk::ApplicationWindow>().ok())
        .ok_or_else(||Failure::binding(ErrorCode::NativeCallerMismatch,None))?;
    if !core::same_objects(&expected_gtk,&expected_view,&actual_gtk,&actual_view){return Err(Failure::binding(ErrorCode::NativeCallerMismatch,None));}
    Ok((native,actual_gtk,actual_view))
}
fn sample(gw:&gtk::ApplicationWindow,view:&gtk::Widget,policy:&Policy)->(Facts,bool,bool){
    let facts=Facts {visible:gw.is_visible()&&view.is_visible(),mapped:gw.is_mapped()&&view.is_mapped()};
    let loading=view.property::<bool>("is-loading");
    let ready=view.property::<Option<String>>("uri").as_deref().and_then(|u|tauri::Url::parse(u).ok()).is_some_and(|u|policy.allowed(&u))&&!loading;
    (facts,loading,ready)
}
fn refresh(app:&AppHandle,native:&Native)->Result<Context,Failure>{
    let(gw,view)=objects(native)?;let(facts,loading,ready)=sample(&gw,&view,&native.policy);
    // All native reads happen BEFORE manager/Core locks. Document invalidation
    // precedes a positive map update; no transient stale-document System grant.
    let (context,changed)=manager(native)?.with_current(native.incarnation,|core|{
        let a=core.document(loading,ready)?;let b=core.observe_native(facts)?;
        Ok((core.context()?,a.is_some()||b.is_some()))
    })?;
    if changed{super::emit_context(app,&context);queue_reconcile(app,native);}Ok(context)
}
fn signal_present(object:&gtk::Widget,name:&std::ffi::CStr)->bool {
    unsafe{glib::gobject_ffi::g_signal_lookup(name.as_ptr(),object.type_().into_glib())!=0}
}
fn callback_events(app:&AppHandle,native:&Native,widget:&gtk::Widget,signals:&mut Vec<(glib::WeakRef<gtk::Widget>,glib::SignalHandlerId)>){
    let app1=app.clone();let n=native.clone();signals.push((widget.downgrade(),widget.connect_show(move |_|{let _=refresh(&app1,&n);})));
    let app1=app.clone();let n=native.clone();signals.push((widget.downgrade(),widget.connect_hide(move |_|{let _=refresh(&app1,&n);})));
    let app1=app.clone();let n=native.clone();signals.push((widget.downgrade(),widget.connect_map(move |_|{let _=refresh(&app1,&n);})));
    let app1=app.clone();let n=native.clone();signals.push((widget.downgrade(),widget.connect_unmap(move |_|{let _=refresh(&app1,&n);})));
    let app1=app.clone();let n=native.clone();signals.push((widget.downgrade(),widget.connect_notify_local(Some("visible"),move |_,_|{let _=refresh(&app1,&n);})));
    let app1=app.clone();let n=native.clone();signals.push((widget.downgrade(),widget.connect_destroy(move |_|{
        // Either WebKit OR GTK can be destroyed while the other still exists.
        // Close/revoke and quiesce the exact old native object before detaching.
        let _=GtkSurface::new(&app1,&n,None).close_unstable();
        if let Ok(m)=manager(&n){if let Ok(Some(context))=m.detach(n.incarnation){super::emit_context(&app1,&context);}}
        let old=REGISTRY.with(|slot|{let mut s=slot.borrow_mut();if s.as_ref().is_some_and(|e|e.native.incarnation==n.incarnation){s.take()}else{None}});drop(old);
    })));
}
pub(crate) fn attach(app:&AppHandle,win:&WebviewWindow)->Result<(),Failure>{
    let app=app.clone();let win2=win.clone();let m=super::manager(&app)?;let policy=Policy::for_app(&app);
    win.with_webview(move |platform|{
        let mut attached=None;
        let result=(||{
            let view=platform.inner().upcast::<gtk::Widget>();
            let gw=view.toplevel().and_then(|v|v.downcast::<gtk::ApplicationWindow>().ok()).ok_or_else(Failure::unavailable)?;
            if view.find_property("is-loading").is_none_or(|p|p.value_type()!=bool::static_type())||view.find_property("uri").is_none()
                ||!signal_present(&view,c"load-changed")||!signal_present(&view,c"web-process-terminated") {return Err(Failure::unavailable());}
            let core=m.register()?;
            attached=Some(core.incarnation);
            let native=Native {gtk:gw.downgrade(),view:view.downgrade(),incarnation:core.incarnation,manager:Arc::downgrade(&m),policy,window:win2.clone(),reconcile_queued:Rc::new(Cell::new(false))};
            let mut signals=Vec::new();callback_events(&app,&native,&gw.clone().upcast::<gtk::Widget>(),&mut signals);callback_events(&app,&native,&view,&mut signals);
            let app1=app.clone();let n=native.clone();
            signals.push((view.downgrade(),view.connect_local("load-changed",false,move |values|{
                // Pinned WebKitGTK WEBKIT_LOAD_STARTED == 0. Inspect the native
                // enum signal, not an old URI/loading property after a crash.
                let started=values.get(1).filter(|v|v.type_().is_a(glib::Type::ENUM)).is_some_and(|v|unsafe{glib::gobject_ffi::g_value_get_enum(v.to_glib_none().0)==0});
                if started{if let Ok(m)=manager(&n){let _=m.with_current(n.incarnation,|c|c.load_started());}}
                let _=refresh(&app1,&n);
                // Native is-loading may settle after load-changed. Recheck on
                // this GTK thread only; latest facts never replay old System.
                let app2=app1.clone();let n2=n.clone();glib::idle_add_local_once(move||{let _=refresh(&app2,&n2);});None
            })));
            let app1=app.clone();let n=native.clone();signals.push((view.downgrade(),view.connect_notify_local(Some("is-loading"),move |_,_|{let _=refresh(&app1,&n);})));
            let app1=app.clone();let n=native.clone();signals.push((view.downgrade(),view.connect_local("web-process-terminated",false,move |_|{
                if let Ok(m)=manager(&n){if m.with_current(n.incarnation,|c|c.crashed()).is_ok(){
                    // Crash runs on GTK: revoke Core first, then synchronously
                    // clear THIS native object's input/focus/timer before return.
                    let _=native_policy::reconcile(&mut GtkSurface::new(&app1,&n,None));
                    if let Ok(c)=m.with_current(n.incarnation,|c|c.context()){super::emit_context(&app1,&c);}
                }}None
            })));
            if crate::platform::desktop_mode()==crate::platform::DesktopMode::Layer{
                let once=Cell::new(false);let n=native.clone();let app1=app.clone();
                signals.push((gw.clone().upcast::<gtk::Widget>().downgrade(),gw.connect_map_event(move|_,_|{
                    if !once.replace(true){let n=n.clone();let app=app1.clone();glib::idle_add_local_once(move||{
                        if objects(&n).is_ok_and(|(gw,_)|gw.is_visible()){
                            let mut surface=GtkSurface::new(&app,&n,None);
                            let _=native_policy::run_steps(&mut surface,&[Step::Hide,Step::Show],|s|s.context.native_available&&!s.context.pending&&!matches!(s.context.mode,Mode::Inactive));
                        }
                    });}glib::Propagation::Proceed
                })));
            }
            let old=REGISTRY.with(|slot|slot.borrow_mut().replace(Entry {native:native.clone(),signals}));drop(old);
            let fallback_manager=Arc::downgrade(&m);let incarnation=core.incarnation;let fallback_app=app.clone();
            glib::idle_add_local_once(move||win2.on_window_event(move|event|{
                if matches!(event,tauri::WindowEvent::Destroyed){if let Some(m)=fallback_manager.upgrade(){if let Ok(Some(c))=m.detach(incarnation){super::emit_context(&fallback_app,&c);}}}
            }));
            refresh(&app,&native)?;Ok::<_,Failure>(())
        })();
        if result.is_err(){if let Some(incarnation)=attached{
            let _=m.detach(incarnation);
            let old=REGISTRY.with(|slot|{let mut s=slot.borrow_mut();if s.as_ref().is_some_and(|e|e.native.incarnation==incarnation){s.take()}else{None}});drop(old);
        }}
    }).map_err(|_|Failure::unavailable())
}

pub(crate) async fn context(win:&WebviewWindow)->Result<Context,Failure>{
    let(tx,rx)=tokio::sync::oneshot::channel();let app=win.app_handle().clone();
    win.with_webview(move |platform|{let result=caller(&platform).and_then(|(native,_,_)|refresh(&app,&native));let _=tx.send(result);}).map_err(|_|Failure::unavailable())?;
    rx.await.map_err(|_|Failure::unavailable())?
}
pub(crate) async fn prepare(win:&WebviewWindow,expected:Revision,client:executor::Client)->Result<PreparedContext,Failure>{
    let(tx,rx)=tokio::sync::oneshot::channel();let app=win.app_handle().clone();
    win.with_webview(move |platform|{
        let result=(||{let(native,_,_)=caller(&platform)?;refresh(&app,&native)?;manager(&native)?.with_current(native.incarnation,|core|core.prepare(&client,expected))})();
        let _=tx.send(result);
    }).map_err(|_|Failure::unavailable())?;
    rx.await.map_err(|_|Failure::unavailable())?
}
pub(crate) async fn navigate(win:&WebviewWindow,request:NavigationRequest)->Result<NavigationReceipt,Failure>{
    let(tx,rx)=tokio::sync::oneshot::channel();let app=win.app_handle().clone();let win2=win.clone();
    win.with_webview(move |platform|{
        let result=(||{let(native,_,_)=caller(&platform)?;refresh(&app,&native)?;
            let begin=manager(&native)?.with_current(native.incarnation,|core|core.begin(request))?;Ok((native,begin))})();
        match result {
            Ok((_,Begin::Duplicate(receipt)))=>{let _=tx.send(Ok(receipt));},
            Ok((native,Begin::Plan(plan,pending)))=>{super::emit_context(&app,&pending);glib::idle_add_local_once(move||{
                let result=objects(&native).and_then(|(gw,view)|apply_now(&app,&win2,&native,&gw,&view,&plan,None));let _=tx.send(result);
            });},
            Err(e)=>{let _=tx.send(Err(e));}
        }
    }).map_err(|_|Failure::unavailable())?;
    rx.await.map_err(|_|Failure::unavailable())?
}
pub(crate) async fn apply(win:&WebviewWindow,plan:Plan,placement:Option<Placement>)->Result<NavigationReceipt,Failure>{
    let(tx,rx)=tokio::sync::oneshot::channel();let app=win.app_handle().clone();let win2=win.clone();
    win.with_webview(move |platform|{
        match caller(&platform){Ok((native,_,_))=>{glib::idle_add_local_once(move||{
            let result=objects(&native).and_then(|(gw,view)|apply_now(&app,&win2,&native,&gw,&view,&plan,placement));let _=tx.send(result);
        });},Err(e)=>{let _=tx.send(Err(e));}}
    }).map_err(|_|Failure::unavailable())?;rx.await.map_err(|_|Failure::unavailable())?
}
struct GtkSurface {app:AppHandle,native:Native,placement:Option<Placement>}
impl GtkSurface {fn new(app:&AppHandle,native:&Native,placement:Option<Placement>)->Self{Self{app:app.clone(),native:native.clone(),placement}}}
impl Surface for GtkSurface {
 fn current(&mut self)->Result<Snapshot,Failure>{
  let _=objects(&self.native)?;refresh(&self.app,&self.native)?;
  manager(&self.native)?.with_current(self.native.incarnation,|c|c.policy_snapshot())
 }
 fn input(&mut self,snapshot:&Snapshot)->Result<(),Failure>{
  let(gw,_)=objects(&self.native)?;
  // Actual registered GTK object, not a label lookup that could retarget a
  // replacement. No Core/manager lock crosses this native policy write.
  crate::platform::set_pet_control_native_input(&self.native.window,&gw,snapshot.layout.rects.clone(),snapshot.layout.expanded,snapshot.layout.keyboard);
  Ok(())
 }
 fn step(&mut self,step:Step)->Result<(),Failure>{
  let(gw,_)=objects(&self.native)?;manager(&self.native)?.with_current(self.native.incarnation,|c|c.context().map(|_|()))?;
  let win=&self.native.window;
  let error=||Failure::binding(ErrorCode::NativeFailure,None);
  match step {
   Step::Show=>gw.show_all(),Step::Hide=>gw.hide(),
   Step::Resizable=>win.set_resizable(true).map_err(|_|error())?,
   Step::Size=>{let p=self.placement.ok_or_else(error)?;win.set_size(tauri::LogicalSize::new(p.width,p.height)).map_err(|_|error())?;},
   Step::Pin=>{let p=self.placement.ok_or_else(error)?;if !crate::platform::pin_layer_to_monitor(win,p.x.round()as i32,p.y.round()as i32){return Err(error());}},
   Step::Margins=>{let p=self.placement.ok_or_else(error)?;crate::platform::set_layer_margins(win,(p.screen_width-p.width)/2.0,p.top);},
   Step::Position=>{let p=self.placement.ok_or_else(error)?;win.set_position(tauri::LogicalPosition::new(p.x+(p.screen_width-p.width)/2.0,p.y+p.top)).map_err(|_|error())?;},
  }Ok(())
 }
 fn close_unstable(&mut self)->Result<(),Failure>{
  let(gw,_)=objects(&self.native)?;
  manager(&self.native)?.with_current(self.native.incarnation,|c|c.close())?;
  // Permanent close prevents reentrant callbacks from minting a new plan for
  // this incarnation. A replaced GTK owner is ignored by the platform helper.
  crate::platform::set_pet_control_native_input(&self.native.window,&gw,Vec::new(),false,false);
  if objects(&self.native).is_ok(){gw.hide();}Ok(())
 }
}
fn placement_steps(show:bool)->Vec<Step>{
 let mut steps=vec![Step::Resizable,Step::Size];
 if crate::platform::desktop_mode()==crate::platform::DesktopMode::Layer{steps.extend([Step::Pin,Step::Margins]);}else{steps.push(Step::Position);}
 if show{steps.push(Step::Show);}steps
}
fn queue_reconcile(app:&AppHandle,native:&Native){
 if native.reconcile_queued.replace(true){return;}
 let app=app.clone();let native=native.clone();glib::idle_add_local_once(move||{
  native.reconcile_queued.set(false);
  if objects(&native).is_ok(){let _=native_policy::reconcile(&mut GtkSurface::new(&app,&native,None));}
 });
}
pub(crate) fn request_reconcile(app:&AppHandle){
 let Some(win)=app.get_webview_window(crate::pet_control::LABEL)else{return;};let app=app.clone();
 let _=win.with_webview(move|platform|{if let Ok((native,_,_))=caller(&platform){queue_reconcile(&app,&native);}});
}
fn apply_now(app:&AppHandle,_win:&WebviewWindow,native:&Native,_gw:&gtk::ApplicationWindow,_view:&gtk::Widget,plan:&Plan,placement:Option<Placement>)->Result<NavigationReceipt,Failure>{
 let m=manager(native)?;let mut surface=GtkSurface::new(app,native,placement);
 let mut steps=if placement.is_some(){placement_steps(false)}else{Vec::new()};
 if matches!(plan.target,Mode::Inactive){steps.push(Step::Hide);}else if plan.target.expanded()||placement.is_some(){steps.push(Step::Show);}
 let native_result=native_policy::run_steps(&mut surface,&steps,|_|m.with_current(native.incarnation,|c|Ok(c.current_plan(plan))).unwrap_or(false));
 let result=m.with_current(native.incarnation,|c|c.finish(plan,native_result.is_ok()));
 // Never leave a captured pre-GTK input policy in place, including on errors.
 let current=native_policy::reconcile(&mut surface)?;
 let receipt=result?;
 if receipt.context.revision!=current.context.revision{return Err(Failure::binding(ErrorCode::Superseded,Some(current.context)));}
 super::emit_context(app,&receipt.context);Ok(receipt)
}
/// Geometry remains a hint. Each actual step resamples current accepted policy;
/// callbacks can supersede this placement, but cannot preserve its old input mask.
pub(crate) async fn place_current(win:&WebviewWindow,placement:Placement)->Result<(),Failure>{
 let(tx,rx)=tokio::sync::oneshot::channel();let app=win.app_handle().clone();
 win.with_webview(move|platform|match caller(&platform){Ok((native,_,_))=>{
  let expected=manager(&native).and_then(|m|m.with_current(native.incarnation,|c|c.context())).map(|c|c.revision);
  glib::idle_add_local_once(move||{let result=(||{
   let expected=expected?;let mut surface=GtkSurface::new(&app,&native,Some(placement));
   native_policy::run_steps(&mut surface,&placement_steps(true),|s|s.context.revision==expected&&!s.context.pending&&!matches!(s.context.mode,Mode::Inactive))?;
   Ok(())
  })();let _=tx.send(result);});
 },Err(e)=>{let _=tx.send(Err(e));}}).map_err(|_|Failure::unavailable())?;
 rx.await.map_err(|_|Failure::unavailable())?
}
pub(crate) async fn hint(win:&WebviewWindow,hint:Hint)->Result<(),Failure>{
    let(tx,rx)=tokio::sync::oneshot::channel();let app=win.app_handle().clone();
    win.with_webview(move |platform|{
        match caller(&platform){Ok((native,_,_))=>{glib::idle_add_local_once(move||{
            let result=(||{refresh(&app,&native)?;
                manager(&native)?.with_current(native.incarnation,|core|core.layout(Some(hint),None).map(|_|()))?;
                native_policy::reconcile(&mut GtkSurface::new(&app,&native,None))?;Ok(())})();let _=tx.send(result);
        });},Err(e)=>{let _=tx.send(Err(e));}}
    }).map_err(|_|Failure::unavailable())?;rx.await.map_err(|_|Failure::unavailable())?
}
pub(crate) fn page_load(win:&WebviewWindow,started:bool){
    let app=win.app_handle().clone();let _=win.with_webview(move |platform|{
        if let Ok((native,_,_))=caller(&platform){
            // Tauri's queued hook invalidates only. Crash recovery requires the
            // synchronous native WebKit Started signal, not an old queued hook.
            if started {if let Ok(m)=manager(&native){if let Ok(Some(c))=m.with_current(native.incarnation,|core|core.document(true,false)){super::emit_context(&app,&c);}}}
            let _=refresh(&app,&native);
            let app2=app.clone();glib::idle_add_local_once(move||{let _=refresh(&app2,&native);});
        }
    });
}
use crate::executor;
