//! A dropped document must never replace an application surface (WebKit's default).
#[cfg(dev)]use tauri::Manager;
fn allowed(url:&reqwest::Url,dev:Option<&reqwest::Url>)->bool {
 if url.as_str()=="about:blank"{return true;}
 let origin=(url.scheme()=="tauri" && url.host_str()==Some("localhost"))
  || (["http","https"].contains(&url.scheme()) && url.host_str()==Some("tauri.localhost") && url.port().is_none())
  || dev.is_some_and(|base|url.origin()==base.origin());
 origin && ["","/","/index.html","/companions.html","/mochi.html","/settings.html","/pet-manager.html","/pet-control.html","/pet-chat.html","/pill.html","/recap.html"].contains(&url.path())
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
  let allowed=allowed(url,dev);
  if !allowed{crate::log::line(format!("surface navigation refused: {} {} {} {}",view.label(),url.scheme(),url.host_str().unwrap_or(""),if url.scheme()=="tauri"{url.path()}else{"[redacted]"}));}
  allowed
 }).build()
}
#[cfg(test)]mod tests{use super::*;#[test]fn documents_and_external_pages_cannot_replace_any_pet_surface(){
 for uri in ["about:blank","tauri://localhost","tauri://localhost/companions.html","http://tauri.localhost/pet-manager.html"]{assert!(allowed(&uri.parse().unwrap(),None));}
 for uri in ["file:///tmp/document.pdf","http://example.com/companions.html","http://tauri.localhost/document.pdf","data:text/html,test","javascript:alert(1)","http://tauri.localhost:8080/index.html"]{assert!(!allowed(&uri.parse().unwrap(),None));}
 let dev="http://localhost:1420".parse().unwrap();assert!(allowed(&"http://localhost:1420/companions.html".parse().unwrap(),Some(&dev)));assert!(!allowed(&"http://localhost:1421/companions.html".parse().unwrap(),Some(&dev)));
}}
