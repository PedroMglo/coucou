//! Original island motion and transparent input mask, independent of agents.
use std::sync::Mutex;
use tauri::{AppHandle,Emitter,Manager,WebviewUrl,WebviewWindowBuilder};
pub const LABEL:&str="pet-control";
#[derive(Default)]struct Control(Mutex<Option<crate::pet_native_binding::ipc::OpenedEvent>>);
/// Only the compact island reserves a lane in the companions habitat.
pub fn width(display:f64,system_center:bool)->f64{
 (if system_center{432.0_f64}else{288.0_f64}).min((display-16.0).max(240.0))
}
fn control_enabled(companions:bool,control_bar:bool,system_center:bool)->bool{
 system_center||(companions&&control_bar)
}
fn top_offset(system_center:bool,native_height:Option<f64>)->f64{
 if system_center{return 0.0;}
 native_height.filter(|h|h.is_finite()&&(24.0..=80.0).contains(h)).map(|h|h+4.0).unwrap_or(0.0)
}
pub(crate) fn valid_section(section:&Option<String>)->bool{
 section.as_ref().is_none_or(|s|["home","system","tasks","sessions","providers","models","profiles"].contains(&s.as_str()))
}
pub fn create(app:&AppHandle){
 app.manage(Control::default());
 #[allow(unused_mut)]let mut url=WebviewUrl::App("pet-control.html".into());
 #[cfg(dev)]if let Some(mut base)=app.config().build.dev_url.clone(){base.set_path("/pet-control.html");url=WebviewUrl::External(base);}
 let navigation_app=app.clone();
 match WebviewWindowBuilder::new(app,LABEL,url).additional_browser_args(crate::BROWSER_ARGS)
  .on_navigation(move|url|crate::pet_native_binding::navigation_allowed(&navigation_app,url))
  .on_page_load(|win,payload|crate::pet_native_binding::page_load(&win,matches!(payload.event(),tauri::webview::PageLoadEvent::Started)))
  .title("Coucou · Pets").inner_size(760.0,700.0).decorations(false).transparent(true).shadow(false)
  .always_on_top(true).skip_taskbar(true).visible_on_all_workspaces(true).focusable(false).focused(false).visible(false)
  .resizable(true).maximizable(false).minimizable(false).closable(false).disable_drag_drop_handler().build(){
   Ok(win)=>{#[cfg(target_os="linux")]if !crate::platform::prepare_pet_control(&win){let _=win.destroy();return;}
    if crate::pet_native_binding::attach(app,&win).is_err(){crate::log::line("pet control native binding unavailable");}
    #[cfg(target_os="linux")]if let Err(error)=crate::linux_documents::install(&win){crate::log::line(format!("pet control drop: {error}"));}},
   Err(e)=>crate::log::line(format!("pet control bar: {e}")),
 }
}
pub fn collapse(app:&AppHandle){crate::pet_native_binding::collapse(app);}
pub async fn open_section(app:AppHandle,pet:Option<String>,section:Option<String>)->Result<(),String>{
 use crate::pet_native_binding::{self as binding,ipc::{Mode,Route,OpenedEvent}};
 if let Some(id)=&pet {let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();if id!=&s.main_pill&&!s.active_integrations.contains(id){return Err("Este pet não está configurado.".into());}}
 let route=Route::from_section(section.as_deref()).ok_or("Secção desconhecida.")?;
 let win=app.get_webview_window(LABEL).ok_or("A ilha dos pets não está disponível.")?;
 let(system_center,pref)={let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();(s.pet_system_center,s.screen.clone())};
 let plan=binding::begin_trusted(&app,Mode::Expanded{route}).map_err(|_|"A navegação da ilha ainda não está disponível.")?;
 let receipt=binding::apply(&win,plan,Some(placement(&app,system_center,&pref))).await.map_err(|_|"Não foi possível abrir a ilha.")?;
 if !binding::is_current(&app,receipt.context.revision){return Err("A navegação da ilha foi substituída.".into());}
 let context=OpenedEvent {pet,section,navigation_revision:receipt.context.revision,context:receipt.context};
 *app.state::<Control>().0.lock().unwrap()=Some(context.clone());
 win.emit("pet-control-opened",context).map_err(|_|"Não foi possível abrir a ilha.".into())
}
/// Legacy display hint only; a query never replays or authorizes an old open.
#[tauri::command]pub fn pet_control_context(app:AppHandle)->Option<crate::pet_panel::Context>{
 let old=app.state::<Control>().0.lock().unwrap().clone()?;
 crate::pet_native_binding::is_current(&app,old.navigation_revision).then_some(crate::pet_panel::Context {pet:old.pet,section:old.section})
}
#[tauri::command]pub async fn pet_control_open(app:AppHandle,pet:Option<String>)->Result<(),String>{open_section(app,pet,Some("sessions".into())).await}
#[derive(serde::Deserialize)]pub struct Rect{x:f64,y:f64,w:f64,h:f64}
#[tauri::command]
pub async fn pet_control_input(_app:AppHandle,window:tauri::WebviewWindow,rect:Rect,expanded:bool,keyboard:Option<bool>)->Result<(),String>{
 let _=expanded; // A spring packet cannot select the backend presentation mode.
 crate::pet_native_binding::hint(&window,crate::pet_native_binding::Hint{x:rect.x,y:rect.y,w:rect.w,h:rect.h,keyboard:keyboard.unwrap_or(false)})
  .await.map_err(|_|"Não foi possível atualizar a ilha.".into())
}
#[tauri::command]
pub async fn pet_control_configure(app:AppHandle,window:tauri::WebviewWindow)->Result<(),String>{
 use crate::pet_native_binding::{self as binding,ipc::Mode};
 if window.label()!=crate::island::WINDOW_LABEL{return Err("O estado da barra pertence à ilha.".into());}
 let(enabled,system_center,pref)={let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();(control_enabled(s.companion_mode,s.pet_control_bar,s.pet_system_center),s.pet_system_center,s.screen.clone())};
 if !system_center{crate::pet_system::release_lease().await;}
 let win=app.get_webview_window(LABEL).ok_or("A barra dos pets não está disponível.")?;
 let context=binding::manager(&app).and_then(|m|m.current()).and_then(|c|c.context()).map_err(|_|"A barra dos pets ainda não está disponível.")?;
 if !enabled||matches!(context.mode,Mode::Inactive){
  let target=if enabled{Mode::Compact}else{Mode::Inactive};
  let plan=binding::begin_trusted(&app,target).map_err(|_|"Não foi possível configurar a ilha.")?;
  binding::apply(&win,plan,enabled.then(||placement(&app,system_center,&pref))).await.map_err(|_|"Não foi possível configurar a ilha.")?;
 }else{
  binding::place_current(&win,placement(&app,system_center,&pref)).await.map_err(|_|"Não foi possível posicionar a ilha.")?;
 }
 Ok(())
}
fn placement(app:&AppHandle,system_center:bool,pref:&str)->crate::pet_native_binding::Placement{
 let screen=crate::island::screen_info(app,pref);let top=top_offset(system_center,crate::companions::native_bar_height(&screen));
 crate::pet_native_binding::Placement {x:screen.x,y:screen.y,screen_width:screen.width,width:760.0_f64.min(screen.width),height:700.0_f64.min((screen.height-top-8.0).max(32.0)),top}
}

#[cfg(test)]mod tests{
 #[test]fn system_access_survives_disabling_companions_or_the_pet_bar(){
  for companions in [false,true]{for bar in [false,true]{
   assert!(super::control_enabled(companions,bar,true));
   assert_eq!(super::control_enabled(companions,bar,false),companions&&bar);
  }}
 }
 #[test]fn pets_only_controls_do_not_cover_the_restored_native_clock(){
  assert_eq!(super::top_offset(false,Some(44.0)),48.0);
  assert_eq!(super::top_offset(false,Some(60.0)),64.0);
  for height in [None,Some(f64::NAN),Some(-1.0),Some(500.0)]{assert_eq!(super::top_offset(false,height),0.0);}
  assert_eq!(super::top_offset(true,Some(44.0)),0.0);
 }
}
