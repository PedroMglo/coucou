//! Explicit shelter popout: metadata/tasks/models without opening the main Coucou window.
use serde::{Serialize,Deserialize};
use std::sync::Mutex;
use tauri::{AppHandle,Emitter,Manager,WebviewUrl,WebviewWindowBuilder,LogicalSize};
pub const LABEL:&str="pet-manager";
#[derive(Default,Clone,Serialize,Deserialize)]
pub struct Context {pub pet:Option<String>,pub section:Option<String>}
#[derive(Default)]pub struct Panel(Mutex<Context>);
pub fn create(app:&AppHandle){
 app.manage(Panel::default());
 #[allow(unused_mut)]let mut url=WebviewUrl::App("pet-manager.html".into());
 #[cfg(dev)]if let Some(mut base)=app.config().build.dev_url.clone(){base.set_path("/pet-manager.html");url=WebviewUrl::External(base);}
 match WebviewWindowBuilder::new(app,LABEL,url).additional_browser_args(crate::BROWSER_ARGS).title("Pets · Gestão")
  .inner_size(760.0,650.0).min_inner_size(320.0,280.0).decorations(false).transparent(true).shadow(false)
  .always_on_top(true).skip_taskbar(true).visible_on_all_workspaces(true).focused(false).visible(false).build(){
  Ok(win)=>{
   #[cfg(target_os="linux")]if !crate::platform::prepare_pet_panel(&win){crate::log::line("pet management popout: platform preparation failed");let _=win.destroy();return;}
   let app=app.clone();
   win.on_window_event(move|event|match event{
    tauri::WindowEvent::CloseRequested{api,..}=>{api.prevent_close();hide(&app);},
    _=>{}
   });
  },Err(error)=>crate::log::line(format!("pet management popout: {error}")),
 }
}
pub fn hide(app:&AppHandle){crate::pet_control::collapse(app);if let Some(win)=app.get_webview_window(LABEL){let _=win.emit("hub-visible",false);let _=win.hide();}}
#[tauri::command]pub fn pet_panel_hide(app:AppHandle){hide(&app);}
#[tauri::command]pub fn pet_panel_context(app:AppHandle)->Context{app.state::<Panel>().0.lock().unwrap().clone()}
fn geometry(width:f64,height:f64,x:f64,y:f64)->(f64,f64,f64,f64){
 let w=760f64.min((width-16.0).max(280.0));let h=650f64.min((height-y-8.0).max(240.0));
 (w,h,(x-w/2.0).clamp(8.0,(width-w-8.0).max(8.0)),y.clamp(8.0,(height-h-8.0).max(8.0)))
}
#[tauri::command]
pub async fn pet_panel_open(app:AppHandle,x:f64,y:f64,pet:Option<String>,section:Option<String>)->Result<(),String>{
 if !x.is_finite()||!y.is_finite(){return Err("Posição inválida para a gestão dos pets.".into());}
 if pet.as_ref().is_some_and(|id|!crate::pet_config::defaults().pets.contains_key(id)){return Err("Pet desconhecido.".into());}
 if section.as_ref().is_some_and(|s|!["tasks","sessions","providers","models","profiles"].contains(&s.as_str())){return Err("Secção desconhecida.".into());}
 let inline={let shared=app.state::<crate::Shared>();let s=shared.settings.lock().unwrap();s.companion_mode&&s.pet_control_bar};
 if inline{return crate::pet_control::open_section(app,pet,section).await;}
 let context=Context{pet,section};*app.state::<Panel>().0.lock().unwrap()=context.clone();
 let win=app.get_webview_window(LABEL).ok_or("O painel dos pets não está disponível nesta sessão.")?;
 let screen=crate::island::screen_info(&app,&app.state::<crate::Shared>().settings.lock().unwrap().screen);
 let (w,h,left,top)=geometry(screen.width,screen.height,x,y);
 let app2=app.clone();let (tx,rx)=tokio::sync::oneshot::channel();
 app.run_on_main_thread(move||{
  let _=win.set_size(LogicalSize::new(w,h));
  #[cfg(target_os="linux")]crate::platform::place_pet_panel(&app2,&win,left,top);
  #[cfg(not(target_os="linux"))]let _=win.set_position(tauri::PhysicalPosition::new(((screen.x+left)*screen.scale)as i32,((screen.y+top)*screen.scale)as i32));
  let _=win.emit("pet-panel-opened",context);let shown=win.show().is_ok();let _=win.emit("hub-visible",true);let _=win.set_focus();let _=tx.send(shown);
 }).map_err(|_|"Não foi possível abrir o painel dos pets.")?;
 if rx.await.unwrap_or(false){Ok(())}else{Err("Não foi possível mostrar o painel dos pets.".into())}
}
#[cfg(test)]mod tests{use super::*;#[test]fn popout_stays_in_its_display(){for(width,height,x,y)in[(3440.,1440.,3200.,50.),(800.,600.,20.,42.),(360.,500.,900.,400.)]{let(w,h,left,top)=geometry(width,height,x,y);assert!(left>=8.&&top>=8.);assert!(left+w<=width-8.);assert!(top+h<=height-8.);}}}
