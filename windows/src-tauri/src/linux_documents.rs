//! GTK owns file-drop negotiation. WebKit must never open the dropped URI.
//! Paths enter the one-use grant list only after a completed OS drop.
use gtk::{gdk,prelude::*};
use std::{cell::RefCell,rc::Rc};
use tauri::{Emitter,EventTarget,Manager,WebviewWindow};

// getCurrentWebview().listen registers the Webview target, not WebviewWindow.
// Emitter::emit broadcasts even when invoked on a particular window: another
// surface must never claim this window's drop token or interpret its position.
fn event_target(label:&str)->EventTarget{EventTarget::webview(label)}

struct PendingDrop{
 context:gdk::DragContext,
 capture:crate::pet_documents::NativeDrop,
 position:(i32,i32,i32),
 time:u32,
}

fn local_path(uri:&str)->Option<String>{
 let url=reqwest::Url::parse(uri).ok()?;
 if url.scheme()!="file"||url.query().is_some()||url.fragment().is_some(){return None;}
 let path=url.to_file_path().ok()?;
 let text=path.to_str()?;
 (!text.is_empty()&&text.len()<=4096&&!text.contains('\0')).then(||text.into())
}
fn emit(win:&WebviewWindow,kind:&str,x:i32,y:i32,scale:i32,paths:Option<Vec<String>>,drop_id:Option<String>){
 let _=win.emit_to(event_target(win.label()),"pet-file-drag",serde_json::json!({"type":kind,"position":{"x":x*scale,"y":y*scale},"paths":paths,"dropId":drop_id}));
}
pub fn install(win:&WebviewWindow)->Result<(),String>{
 let destination=win.clone();
 win.with_webview(move|native|{
  let view=native.inner();
  let Ok(receiver)=destination.gtk_window()else{return;};
  // WebKit connects its HTML drop handlers during construction, before the
  // application's handlers. Their TRUE accumulator can otherwise consume the
  // signal before our COPY destination sees it. Replace only these DND signal
  // handlers on this WebView; all pointer, key and navigation handlers remain.
  use gtk::glib::translate::IntoGlib;
  for name in [c"drag-motion",c"drag-drop",c"drag-data-received"]{
   unsafe{
    let signal=gtk::glib::gobject_ffi::g_signal_lookup(name.as_ptr(),view.type_().into_glib());
    let count=gtk::glib::gobject_ffi::g_signal_handlers_block_matched(view.as_ptr()as*mut _,gtk::glib::gobject_ffi::G_SIGNAL_MATCH_ID,signal,0,std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut());
    crate::log::line(format!("{}: document destination {} replaced {} handler(s)",destination.label(),name.to_string_lossy(),count));
   }
  }
  // Do not depend on WebKit's private target-info numbering or its HTML
  // navigation policy to acknowledge a file. Request URI data only on drop.
  const URI:u32=0x434f;
  // A WebKit widget can re-establish its private HTML destination internally.
  // Let GTK route a drop to its parent window instead; the window's native
  // input mask still limits acceptance to the actual pet/card under the mouse.
  view.drag_dest_unset();
  receiver.drag_dest_set(gtk::DestDefaults::empty(),&[gtk::TargetEntry::new("text/uri-list",gtk::TargetFlags::empty(),URI)],gdk::DragAction::COPY);
  let pending:Rc<RefCell<Option<PendingDrop>>>=Rc::new(RefCell::new(None));
  let win=destination.clone();
  let seen=std::cell::Cell::new(false);
  receiver.connect_drag_motion(move|widget,context,x,y,time|{
   if !seen.replace(true){crate::log::line(format!("{}: native document motion ({} targets)",win.label(),context.list_targets().len()));}
   let uri=gdk::Atom::intern("text/uri-list");
   if !context.list_targets().contains(&uri){return false;}
   context.drag_status(gdk::DragAction::COPY,time);
   emit(&win,"over",x,y,widget.scale_factor(),None,None);true
  });
  let expected=pending.clone();
  let drop_window=destination.clone();
  receiver.connect_drag_drop(move|widget,context,x,y,time|{
   crate::log::line(format!("{}: native document drop signal",drop_window.label()));
   let uri=gdk::Atom::intern("text/uri-list");
   if !context.list_targets().contains(&uri){return false;}
   if expected.borrow().as_ref().is_some_and(|drop|drop.context==*context){return true;}
   // Mouse release is the ownership boundary. URI delivery may arrive after
   // another conversation was opened: never capture its owner at that time.
   let capture=match crate::pet_documents::prepare_drop(drop_window.app_handle()){
    Ok(capture)=>capture,
    Err(error)=>{
     let _=drop_window.emit_to(event_target(drop_window.label()),"pet-file-rejected",error);
     context.drag_finish(false,false,time);return true;
    },
   };
   let previous=expected.borrow_mut().replace(PendingDrop{context:context.clone(),capture,position:(x,y,widget.scale_factor()),time});
   if let Some(previous)=previous{previous.context.drag_finish(false,false,previous.time);}
   widget.drag_get_data(context,&uri,time);true
  });
  let win=destination.clone();
  receiver.connect_drag_data_received(move|widget,context,_x,_y,data,info,time|{
   // WebKit may reset the target table on realization, including target-info
   // numbering. A requested drop context, and validated URI data, are the gate.
   if !pending.borrow().as_ref().is_some_and(|drop|drop.context==*context){return;}
   let _=info;
   let Some(drop)=pending.borrow_mut().take()else{return;};
   let (x,y,scale)=drop.position;
   let uris=data.uris();
   let paths:Option<Vec<_>>=if uris.len()>32{None}else{uris.iter().map(|uri|local_path(uri.as_str())).collect()};
   let mut valid=false;
   if let Some(paths)=paths.filter(|p|!p.is_empty()){
    match crate::pet_documents::capture_drop(drop.capture,&paths){
     Ok(drop_id)=>{crate::files::allow_dropped(paths.clone());valid=true;
      crate::log::line(format!("{}: completed document drop ({} items)",win.label(),paths.len()));
      emit(&win,"drop",x,y,scale,Some(paths),Some(drop_id));
     },Err(error)=>{let _=win.emit_to(event_target(win.label()),"pet-file-rejected",error);},
    }
   }else{
    let _=win.emit_to(event_target(win.label()),"pet-file-rejected","Esta entrega não contém documentos locais válidos ou excede o limite de 32 itens.");
   }
   context.drag_finish(valid,false,time);
   widget.stop_signal_emission_by_name("drag-data-received");
  });
  receiver.connect_drag_leave(move|widget,_context,_time|{
   // No position on leave: the last point may be the pet accepting the drop.
   let _=destination.emit_to(event_target(destination.label()),"pet-file-drag",serde_json::json!({"type":"leave"}));
   let _=widget;
  });
 }).map_err(|e|format!("Document drop could not be prepared: {e}"))
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn document_events_target_only_the_originating_webview(){
  for label in [crate::island::WINDOW_LABEL,crate::companions::LABEL,crate::pet_chat::LABEL,crate::pet_control::LABEL]{
   let target=event_target(label);
   assert_eq!(target,EventTarget::webview(label));
   assert_ne!(target,EventTarget::Any);
   assert_ne!(target,EventTarget::webview_window(label));
   for other in [crate::island::WINDOW_LABEL,crate::companions::LABEL,crate::pet_chat::LABEL,crate::pet_control::LABEL]{
    if other!=label{assert_ne!(target,event_target(other));}
   }
  }
 }
 #[test]fn only_local_file_uris_become_drop_paths(){
  assert_eq!(local_path("file:///tmp/Ol%C3%A1%20test.pdf"),Some("/tmp/Olá test.pdf".into()));
  for uri in ["https://example.org/doc.pdf","file://remote.invalid/tmp/a","file:///tmp/a#fragment","file:///tmp/a?query","file:///tmp/a%00b"]{assert!(local_path(uri).is_none(),"{uri}");}
 }
 #[test]fn local_file_paths_decode_spaces_and_unicode(){
  for (uri,path) in [
   ("file:///tmp/Relat%C3%B3rio%20final.pdf","/tmp/Relatório final.pdf"),
   ("file:///tmp/%E8%B3%87%E6%96%99/%C3%A1rvore%20final.txt","/tmp/資料/árvore final.txt"),
   ("file:///tmp/pet%20%F0%9F%90%BE.png","/tmp/pet 🐾.png"),
   ("file:///tmp/ação/rascunho final.md","/tmp/ação/rascunho final.md"),
  ]{assert_eq!(local_path(uri).as_deref(),Some(path),"{uri}");}
 }
 #[test]fn percent_encoded_delimiters_are_filename_characters_not_url_components(){
  for (uri,path) in [
   ("file:///tmp/hash%23name.txt","/tmp/hash#name.txt"),
   ("file:///tmp/query%3Fname.txt","/tmp/query?name.txt"),
   ("file:///tmp/100%25%20%23%3F%2B.txt","/tmp/100% #?+.txt"),
   ("file:///tmp/a+b.txt","/tmp/a+b.txt"),
   ("file:///tmp/literal%2523%253F%2500.txt","/tmp/literal%23%3F%00.txt"),
  ]{assert_eq!(local_path(uri).as_deref(),Some(path),"{uri}");}
 }
 #[test]fn local_directory_uris_preserve_their_trailing_separator(){
  // Conversion is pure: filesystem existence and document grants belong to
  // the completed native drop, not to URI parsing.
  for (uri,path) in [
   ("file:///","/"),
   ("file:///tmp/","/tmp/"),
   ("file:///tmp/Folder%20with%20spaces/","/tmp/Folder with spaces/"),
   ("file:///tmp/Documentos%20%C3%BAteis","/tmp/Documentos úteis"),
  ]{assert_eq!(local_path(uri).as_deref(),Some(path),"{uri}");}
 }
 #[test]fn localhost_uses_the_native_file_url_semantics(){
  for uri in ["file://localhost/tmp/a.pdf","file://LOCALHOST/tmp/a.pdf"]{
   let native=reqwest::Url::parse(uri).unwrap().to_file_path().unwrap();
   assert_eq!(native.to_str(),Some("/tmp/a.pdf"));
   assert_eq!(local_path(uri).as_deref(),native.to_str(),"{uri}");
  }
 }
 #[test]fn remote_hosts_and_non_file_schemes_never_become_local_paths(){
  for uri in [
   "file://remote.invalid/tmp/a.pdf","file://127.0.0.1/tmp/a.pdf","file://[::1]/tmp/a.pdf",
   "http://localhost/tmp/a.pdf","https://example.org/doc.pdf","ftp://example.org/doc.pdf",
   "data:text/plain,document","smb://server/share/doc.pdf",
  ]{assert!(local_path(uri).is_none(),"{uri}");}
 }
 #[test]fn query_and_fragment_components_are_rejected_even_when_empty(){
  for uri in [
   "file:///tmp/a?query","file:///tmp/a?","file:///tmp/a#fragment","file:///tmp/a#",
   "file:///tmp/a?query#fragment","file:///tmp/a%23b?query","file:///tmp/a%3Fb#fragment",
   "file://localhost/tmp/a?query","file://localhost/tmp/a#fragment",
  ]{assert!(local_path(uri).is_none(),"{uri}");}
 }
 #[test]fn relative_paths_and_invalid_uris_are_not_drop_uris(){
  for uri in [
   "",".","./document.pdf","../document.pdf","folder/document.pdf","/tmp/document.pdf",
   "//localhost/tmp/document.pdf","://invalid","file://[invalid/tmp/document.pdf",
   "file://user:password@localhost/tmp/document.pdf","file://localhost:80/tmp/document.pdf",
  ]{assert!(local_path(uri).is_none(),"{uri}");}
 }
 #[test]fn nul_and_non_utf8_path_bytes_are_rejected(){
  for uri in [
   "file:///tmp/a%00b","file:///tmp/%00","file:///tmp/a\0b",
   "file:///tmp/%FF.txt","file:///tmp/%C3%28.txt","file:///tmp/%ED%A0%80.txt",
  ]{assert!(local_path(uri).is_none(),"{uri:?}");}
 }
 #[test]fn path_limit_applies_to_decoded_utf8_bytes(){
  let path=format!("/{}","a".repeat(4095));
  let uri=format!("file://{path}");
  assert_eq!(local_path(&uri).as_deref(),Some(path.as_str()));
  assert!(local_path(&format!("{uri}a")).is_none());
  let path=format!("/{}a","é".repeat(2047));
  let uri=format!("file:///{}a","%C3%A9".repeat(2047));
  assert_eq!(path.len(),4096);assert!(uri.len()>4096);
  assert_eq!(local_path(&uri).as_deref(),Some(path.as_str()));
  assert!(local_path(&format!("{uri}a")).is_none());
 }
}
