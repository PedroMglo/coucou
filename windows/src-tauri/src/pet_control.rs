//! Original island motion and transparent input mask, independent of agents.
use std::sync::Mutex;
use tauri::{AppHandle,Emitter,Manager,WebviewUrl,WebviewWindowBuilder,LogicalSize};
pub const LABEL:&str="pet-control";
#[derive(Default)]struct Control(Mutex<Option<crate::pet_panel::Context>>);
/// Only the compact island reserves a lane in the companions habitat.
pub fn width(display:f64,system_center:bool)->f64{
 (if system_center{432.0_f64}else{288.0_f64}).min((display-16.0).max(240.0))
}
pub fn create(app:&AppHandle){
 app.manage(Control::default());
 #[allow(unused_mut)]let mut url=WebviewUrl::App("pet-control.html".into());
 #[cfg(dev)]if let Some(mut base)=app.config().build.dev_url.clone(){base.set_path("/pet-control.html");url=WebviewUrl::External(base);}
 match WebviewWindowBuilder::new(app,LABEL,url).additional_browser_args(crate::BROWSER_ARGS)
  .title("Coucou · Pets").inner_size(760.0,700.0).decorations(false).transparent(true).shadow(false)
  .always_on_top(true).skip_taskbar(true).visible_on_all_workspaces(true).focusable(false).focused(false).visible(false)
  .resizable(true).maximizable(false).minimizable(false).closable(false).disable_drag_drop_handler().build(){
   Ok(win)=>{#[cfg(target_os="linux")]if !crate::platform::prepare_pet_control(&win){let _=win.destroy();return;}
    #[cfg(target_os="linux")]if let Err(error)=crate::linux_documents::install(&win){crate::log::line(format!("pet control drop: {error}"));}},
   Err(e)=>crate::log::line(format!("pet control bar: {e}")),
 }
}
pub fn collapse(app:&AppHandle){if let Some(w)=app.get_webview_window(LABEL){let _=w.emit("pet-control-collapse",());}}
pub async fn open_section(app:AppHandle,pet:Option<String>,section:Option<String>)->Result<(),String>{
 if let Some(id)=&pet {let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();if id!=&s.main_pill&&!s.active_integrations.contains(id){return Err("Este pet não está configurado.".into());}}
 if section.as_ref().is_some_and(|s|!["tasks","sessions","providers","models","profiles"].contains(&s.as_str())){return Err("Secção desconhecida.".into());}
 let context=crate::pet_panel::Context{pet,section};*app.state::<Control>().0.lock().unwrap()=Some(context.clone());
 let win=app.get_webview_window(LABEL).ok_or("A ilha dos pets não está disponível.")?;
 win.emit("pet-control-opened",context).map_err(|_|"Não foi possível abrir a ilha.")?;let _=win.show();Ok(())
}
#[tauri::command]pub fn pet_control_context(app:AppHandle)->Option<crate::pet_panel::Context>{app.state::<Control>().0.lock().unwrap().clone()}
#[tauri::command]pub async fn pet_control_open(app:AppHandle,pet:Option<String>)->Result<(),String>{open_section(app,pet,Some("sessions".into())).await}
#[derive(serde::Deserialize)]pub struct Rect{x:f64,y:f64,w:f64,h:f64}
#[tauri::command]
pub fn pet_control_input(app:AppHandle,window:tauri::WebviewWindow,rect:Rect,expanded:bool)->Result<(),String>{
 if window.label()!=LABEL||![rect.x,rect.y,rect.w,rect.h].iter().all(|v|v.is_finite()&&*v>=0.0&&*v<10000.0){return Err("Área inválida para a ilha.".into());}
 let win=window.clone();app.run_on_main_thread(move||{
  #[cfg(target_os="linux")]crate::platform::set_pet_control_input(&win,vec![(rect.x,rect.y,rect.w,rect.h)],expanded);
 }).map_err(|_|"Não foi possível atualizar a ilha.".into())
}
#[tauri::command]
pub async fn pet_control_configure(app:AppHandle,window:tauri::WebviewWindow)->Result<(),String>{
 if window.label()!=crate::island::WINDOW_LABEL{return Err("O estado da barra pertence à ilha.".into());}
 let(enabled,pref)={let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();(s.companion_mode&&s.pet_control_bar,s.screen.clone())};
 let win=app.get_webview_window(LABEL).ok_or("A barra dos pets não está disponível.")?;
 if !enabled{let _=win.hide();return Ok(());}
 let screen=crate::island::screen_info(&app,&pref);let w=760.0_f64.min(screen.width);let h=700.0_f64.min(screen.height-8.0);
 let other=app.clone();let(tx,rx)=tokio::sync::oneshot::channel();
 app.run_on_main_thread(move||{
  let _=win.set_resizable(true);let _=win.set_size(LogicalSize::new(w,h));
  #[cfg(target_os="linux")]crate::platform::place_pet_panel(&other,&win,(screen.width-w)/2.0,0.0);
  #[cfg(not(target_os="linux"))]let _=win.set_position(tauri::LogicalPosition::new(screen.x+(screen.width-w)/2.0,screen.y));
  let _=tx.send(win.show().map_err(|_|"Não foi possível mostrar a barra dos pets.".to_string()));
 }).map_err(|_|"Não foi possível posicionar a barra dos pets.")?;
 rx.await.map_err(|_|"A barra dos pets não respondeu.".to_string())?
}
