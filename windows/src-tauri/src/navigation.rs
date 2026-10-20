//! A dropped document must never replace an application surface (WebKit's default).
#[cfg(dev)]use tauri::Manager;
fn allowed(url:&reqwest::Url,dev:Option<&reqwest::Url>)->bool {
 if url.as_str()=="about:blank"{return true;}
 let origin=(url.scheme()=="tauri" && url.host_str()==Some("localhost"))
  || (["http","https"].contains(&url.scheme()) && url.host_str()==Some("tauri.localhost") && url.port().is_none())
  || dev.is_some_and(|base|url.origin()==base.origin());
 #[cfg(target_os="linux")]
 let pages=["/pet-cards.html","/companions.html","/settings.html","/pet-control.html","/pet-chat.html","/pill.html","/recap.html"];
 #[cfg(not(target_os="linux"))]
 let pages=["","/","/index.html","/companions.html","/mochi.html","/settings.html","/pet-manager.html","/pet-control.html","/pet-chat.html","/pill.html","/recap.html"];
 origin && pages.contains(&url.path())
}
fn allowed_surface(label:&str,url:&reqwest::Url,dev:Option<&reqwest::Url>)->bool{
 if !allowed(url,dev){return false;}
 #[cfg(target_os="linux")]{
  if url.as_str()=="about:blank"{return true;}
  let page=match label{
   "island"=>"/pet-cards.html","companions"=>"/companions.html","settings"=>"/settings.html",
   "pet-control"=>"/pet-control.html","pet-chat"=>"/pet-chat.html","pill"=>"/pill.html","recap"=>"/recap.html",
   _=>return false,
  };
  url.path()==page
 }
 #[cfg(not(target_os="linux"))]{let _=label;true}
}
pub fn plugin()->tauri::plugin::TauriPlugin<tauri::Wry>{
 tauri::plugin::Builder::new("surface-navigation").js_init_script(r#"
window.addEventListener('error',event=>{
 if(performance.now()>10000)return;
 const message='startup '+location.pathname+': '+String(event.message).slice(0,240);
 window.__TAURI_INTERNALS__?.invoke('log_line',{message}).catch(()=>{});
});
window.addEventListener('unhandledrejection',event=>{
 if(performance.now()>10000)return;
 const message='startup '+location.pathname+': '+String(event.reason).slice(0,240);
 window.__TAURI_INTERNALS__?.invoke('log_line',{message}).catch(()=>{});
});
"#).on_navigation(|view,url|{
  #[cfg(dev)]let dev=view.app_handle().config().build.dev_url.as_ref();
  #[cfg(not(dev))]let dev=None;
  let allowed=allowed_surface(view.label(),url,dev);
  if !allowed{crate::log::line(format!("surface navigation refused: {} {} {} {}",view.label(),url.scheme(),url.host_str().unwrap_or(""),if url.scheme()=="tauri"{url.path()}else{"[redacted]"}));}
  allowed
 }).build()
}
#[cfg(test)]mod tests{use super::*;#[test]fn documents_and_external_pages_cannot_replace_any_pet_surface(){
 for uri in ["about:blank","tauri://localhost/companions.html","http://tauri.localhost/pet-control.html"]{assert!(allowed(&uri.parse().unwrap(),None));}
 for uri in ["file:///tmp/document.pdf","http://example.com/companions.html","http://tauri.localhost/document.pdf","data:text/html,test","javascript:alert(1)","http://tauri.localhost:8080/index.html"]{assert!(!allowed(&uri.parse().unwrap(),None));}
 let dev="http://localhost:1420".parse().unwrap();assert!(allowed(&"http://localhost:1420/companions.html".parse().unwrap(),Some(&dev)));assert!(!allowed(&"http://localhost:1421/companions.html".parse().unwrap(),Some(&dev)));
}
#[cfg(target_os="linux")]
#[test]fn every_configured_linux_entry_can_boot_but_retired_presentations_cannot_navigate(){
 let config:serde_json::Value=serde_json::from_str(include_str!("../tauri.linux.conf.json")).unwrap();
 for window in config["app"]["windows"].as_array().unwrap(){
  let page=window["url"].as_str().unwrap();
  let label=window["label"].as_str().unwrap();
  assert!(allowed_surface(label,&format!("tauri://localhost/{page}").parse().unwrap(),None),"configured entry was blocked: {page}");
 }
 for origin in ["tauri://localhost","http://tauri.localhost","https://tauri.localhost"]{
  for path in ["","/","/index.html","/mochi.html","/pet-manager.html"]{
   assert!(!allowed(&format!("{origin}{path}").parse().unwrap(),None),"retired presentation is reachable: {origin}{path}");
  }
  for path in ["/pet-cards.html","/pet-control.html","/pet-chat.html","/settings.html","/companions.html","/pill.html"]{
   assert!(allowed(&format!("{origin}{path}").parse().unwrap(),None));
  }
 }
 for page in ["/settings.html","/pet-control.html","/companions.html","/pet-chat.html","/pill.html"]{
  assert!(!allowed_surface("island",&format!("tauri://localhost{page}").parse().unwrap(),None));
 }
 assert!(!allowed_surface("pet-control",&"tauri://localhost/pet-cards.html".parse().unwrap(),None));
}
}
