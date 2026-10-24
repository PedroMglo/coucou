//! A request owns immutable attachment IDs, not a live view of the collection.
use super::*;
use std::sync::atomic::{AtomicBool,Ordering};

const REVOKED:&str="A conversa, os anexos ou as permissões mudaram. O resultado foi descartado.";
struct PendingCheck{cancel:Arc<AtomicBool>,accepted:bool}
impl Drop for PendingCheck{fn drop(&mut self){if !self.accepted{self.cancel.store(true,Ordering::Release);}}}
#[derive(PartialEq,Eq)]
struct Stamp{
    size:u64,modified:Option<std::time::SystemTime>,directory:bool,
    #[cfg(unix)] native:(u64,u64,i64,i64,u32,u32,u64),
}
impl Stamp{
    fn at(path:&str)->Option<Self>{
        let m=std::fs::symlink_metadata(path).ok()?;if !m.is_file()&&!m.is_dir(){return None;}
        #[cfg(unix)] use std::os::unix::fs::MetadataExt;
        Some(Self{size:m.len(),modified:m.modified().ok(),directory:m.is_dir(),
            #[cfg(unix)] native:(m.dev(),m.ino(),m.ctime(),m.ctime_nsec(),m.uid(),m.mode(),m.nlink()),
        })
    }
}
struct Lease{
    current:Arc<dyn Fn()->bool+Send+Sync>,cancel:Arc<AtomicBool>,stop:Arc<AtomicBool>,watcher:Option<std::thread::JoinHandle<()>>,
    #[cfg(target_os="linux")] folders:Option<Arc<super::folder_watch::Folders>>,
}
impl Lease{
    fn new(current:impl Fn()->bool+Send+Sync+'static,cancel:Arc<AtomicBool>)->Result<Self,String>{
        let current:Arc<dyn Fn()->bool+Send+Sync>=Arc::new(current);let stop=Arc::new(AtomicBool::new(false));
        let mut value=Self{current:current.clone(),cancel:cancel.clone(),stop:stop.clone(),watcher:None,
            #[cfg(target_os="linux")] folders:None,
        };value.validate()?;
        value.watcher=Some(std::thread::Builder::new().name("pet-attachment-scope".into()).spawn(move||{
            while !stop.load(Ordering::Acquire){if !current(){cancel.store(true,Ordering::Release);break;}std::thread::sleep(Duration::from_millis(20));}
        }).map_err(|_|"Não foi possível acompanhar a autorização dos anexos.".to_string())?);Ok(value)
    }
    fn validate(&self)->Result<(),String>{if !self.cancel.load(Ordering::Acquire)&&(self.current)(){Ok(())}else{self.cancel.store(true,Ordering::Release);Err(REVOKED.into())}}
}
impl Drop for Lease{fn drop(&mut self){self.stop.store(true,Ordering::Release);if let Some(watcher)=self.watcher.take(){let _=watcher.join();}}}

#[derive(Default)]
pub(crate) struct Request(Option<Lease>);
impl Request{
    pub(crate) fn capture(chat:&crate::chat::Chat,turn:&crate::chat::Turn,context:Option<&crate::chat::ChatContext>)->Result<Self,String>{
        let result=Self::capture_at(chat,turn,context,Instant::now()+Duration::from_secs(8));if result.is_err(){turn.cancel.store(true,Ordering::Release);}result
    }
    pub(crate) async fn capture_async(chat:&crate::chat::Chat,turn:&crate::chat::Turn,context:Option<&crate::chat::ChatContext>)->Result<Self,String>{
        if !context.is_some_and(|c|c.has_files()){return Ok(Self::default());}
        let deadline=Instant::now()+Duration::from_secs(8);let(chat,turn,context)=(chat.clone(),turn.clone(),context.cloned());let cancel=turn.cancel.clone();
        let mut pending=PendingCheck{cancel:cancel.clone(),accepted:false};
        let result=bounded_copy(deadline,move||Self::capture_at(&chat,&turn,context.as_ref(),deadline)).await.and_then(|result|result);
        pending.accepted=result.is_ok();result
    }
    fn capture_at(chat:&crate::chat::Chat,turn:&crate::chat::Turn,context:Option<&crate::chat::ChatContext>,deadline:Instant)->Result<Self,String>{
        let Some(context)=context.filter(|c|c.has_files())else{return Ok(Self::default())};
        let origin=chat.usage_origin(turn).ok_or(REVOKED)?;
        let canonical=checked_context(&turn.provider,&origin.0,context.clone(),None)?;
        let crate::chat::ChatContext::Files{files}=canonical else{return Err(REVOKED.into())};
        let stamps=files.iter().map(|f|Stamp::at(&f.path).ok_or_else(||REVOKED.to_string())).collect::<Result<Vec<_>,_>>()?;
        let attachments=store().lock().map_err(|_|REVOKED.to_string())?.references(&turn.provider,&origin.0,&files)?;
        chat.capture_attachment_history(turn,attachments.iter().map(crate::attachment_history::Reference::from).collect())?;
        #[cfg(target_os="linux")]
        let folders=super::folder_watch::Folders::capture(&crate::files::inbox_dir(),&attachments,deadline,turn.cancel.clone())?.map(Arc::new);
        #[cfg(target_os="linux")]
        let watched=folders.clone();
        #[cfg(not(target_os="linux"))]
        let _=deadline;
        let (chat,captured,cancel)=(chat.clone(),turn.clone(),turn.cancel.clone());
        let mut lease=Lease::new(move||{
            if chat.usage_origin(&captured).as_ref()!=Some(&origin){return false;}
            if !store().lock().is_ok_and(|state|state.references(&captured.provider,&origin.0,&files).is_ok()){return false;}
            let paths=files.iter().map(|f|f.path.as_str()).collect::<Vec<_>>();
            #[cfg(target_os="linux")]
            if watched.as_ref().is_some_and(|folders|!folders.current()){return false;}
            crate::pet_files::check_existing_many(&captured.provider,&paths).is_ok()
                &&files.iter().zip(&stamps).all(|(f,before)|Stamp::at(&f.path).as_ref()==Some(before))
        },cancel)?;
        #[cfg(target_os="linux")]
        {lease.folders=folders;}
        Ok(Self(Some(lease)))
    }
    pub(crate) fn validate(&self)->Result<(),String>{self.0.as_ref().map_or(Ok(()),Lease::validate)}
    pub(crate) fn validate_contents_blocking(&self)->Result<(),String>{
        self.validate()?;
        #[cfg(target_os="linux")]
        if let Some(lease)=&self.0{if lease.folders.as_ref().is_some_and(|folders|!folders.contents_current(Instant::now()+Duration::from_secs(8))){lease.cancel.store(true,Ordering::Release);return Err(REVOKED.into());}}
        self.validate()
    }
    pub(crate) async fn validate_contents(&self)->Result<(),String>{
        self.validate()?;
        #[cfg(target_os="linux")]
        if let Some(lease)=&self.0{if let Some(folders)=&lease.folders{
            let mut pending=PendingCheck{cancel:lease.cancel.clone(),accepted:false};
            let folders=folders.clone();let deadline=Instant::now()+Duration::from_secs(8);
            let valid=bounded_copy(deadline,move||folders.contents_current(deadline)).await.unwrap_or(false);
            if !valid{lease.cancel.store(true,Ordering::Release);return Err(REVOKED.into());}
            pending.accepted=true;
        }}self.validate()
    }
    pub(crate) fn callback_guard(&self)->impl Fn()->bool+Send+Sync+'static{
        let scope=self.0.as_ref().map(|lease|(lease.current.clone(),lease.cancel.clone()));
        move||scope.as_ref().is_none_or(|(current,cancel)|{if !cancel.load(Ordering::Acquire)&&current(){true}else{cancel.store(true,Ordering::Release);false}})
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[cfg(target_os="linux")]
    fn folder_request(chat:&crate::chat::Chat,turn:&crate::chat::Turn,folders:Arc<super::super::folder_watch::Folders>)->Request{
        let(origin,chat,turn)=(chat.usage_origin(turn).unwrap(),chat.clone(),turn.clone());let cancel=turn.cancel.clone();let watched=folders.clone();
        let mut lease=Lease::new(move||chat.usage_origin(&turn).as_ref()==Some(&origin)&&watched.current(),cancel).unwrap();lease.folders=Some(folders);Request(Some(lease))
    }
    #[cfg(target_os="linux")]
    #[test]
    fn nested_folder_change_cancels_only_the_original_turn_and_survives_callback_drop(){
        let fixture=super::super::folder_watch::tests::Fixture::new();let chat=crate::chat::Chat::default();let turn=chat.begin_cli("ollama",Default::default()).unwrap();let other=chat.begin_cli("codex",Default::default()).unwrap();
        let folders=Arc::new(fixture.capture(turn.cancel.clone()));let request=folder_request(&chat,&turn,folders);let callback=request.callback_guard();assert!(callback());drop(request);assert!(!turn.cancel.load(Ordering::Acquire));
        std::fs::write(&fixture.leaf,b"new").unwrap();assert!(!callback());assert!(turn.cancel.load(Ordering::Acquire));assert!(!other.cancel.load(Ordering::Acquire));assert!(chat.is_current(&other));
    }
    #[cfg(target_os="linux")]
    #[test]
    fn folder_watcher_detects_deep_changes_without_a_model_callback(){
        let fixture=super::super::folder_watch::tests::Fixture::new();let chat=crate::chat::Chat::default();let turn=chat.begin_cli("ollama",Default::default()).unwrap();let folders=Arc::new(fixture.capture(turn.cancel.clone()));let request=folder_request(&chat,&turn,folders);
        std::fs::write(&fixture.leaf,b"new").unwrap();let until=Instant::now()+Duration::from_secs(1);while !turn.cancel.load(Ordering::Acquire){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}assert!(request.validate().is_err());
    }
    #[cfg(target_os="linux")]
    #[tokio::test]
    async fn full_folder_boundary_runs_with_the_same_original_lease(){
        let _serial=super::super::tests::COPY_TEST_SERIAL.lock().unwrap();let fixture=super::super::folder_watch::tests::Fixture::new();let chat=crate::chat::Chat::default();let turn=chat.begin_cli("ollama",Default::default()).unwrap();let folders=Arc::new(fixture.capture(turn.cancel.clone()));let request=folder_request(&chat,&turn,folders);
        request.validate_contents().await.unwrap();assert!(!turn.cancel.load(Ordering::Acquire));std::fs::write(&fixture.leaf,b"new").unwrap();assert!(request.validate_contents().await.is_err());assert!(turn.cancel.load(Ordering::Acquire));
    }
    #[tokio::test]
    async fn no_file_capture_needs_no_io_slot_or_live_turn(){
        let _serial=super::super::tests::COPY_TEST_SERIAL.lock().unwrap();let all=copy_slots().clone().acquire_many_owned(4).await.unwrap();let chat=crate::chat::Chat::default();let turn=chat.begin_cli("codex",Default::default()).unwrap();turn.cancel.store(true,Ordering::Release);
        let request=Request::capture_async(&chat,&turn,None).await.unwrap();request.validate_contents().await.unwrap();assert!(request.callback_guard()());drop(all);
    }
    #[test]
    fn dropped_pending_checks_cancel_only_their_original_flag(){
        let original=Arc::new(AtomicBool::new(false));let other=Arc::new(AtomicBool::new(false));drop(PendingCheck{cancel:original.clone(),accepted:false});assert!(original.load(Ordering::Acquire));assert!(!other.load(Ordering::Acquire));drop(PendingCheck{cancel:other.clone(),accepted:true});assert!(!other.load(Ordering::Acquire));
    }
    #[test]
    fn no_document_request_has_no_file_or_configuration_io(){
        let chat=crate::chat::Chat::default();let turn=chat.begin_cli("codex",Default::default()).unwrap();turn.cancel.store(true,Ordering::Release);
        assert!(Request::capture(&chat,&turn,None).unwrap().validate().is_ok());assert!(Request::default().callback_guard()());
    }
    #[test]
    fn revoked_original_set_cancels_only_its_turn_and_static_callback_outlives_the_watcher(){
        let current=Arc::new(AtomicBool::new(true));let original=Arc::new(AtomicBool::new(false));let other=Arc::new(AtomicBool::new(false));let observed=current.clone();
        let request=Request(Some(Lease::new(move||observed.load(Ordering::Acquire),original.clone()).unwrap()));let callback=request.callback_guard();assert!(callback());drop(request);assert!(!original.load(Ordering::Acquire));
        current.store(false,Ordering::Release);assert!(!callback());assert!(original.load(Ordering::Acquire));assert!(!other.load(Ordering::Acquire));
    }
    #[test]
    fn monitor_observes_revocation_without_a_model_callback(){
        let current=Arc::new(AtomicBool::new(true));let cancel=Arc::new(AtomicBool::new(false));let observed=current.clone();let request=Request(Some(Lease::new(move||observed.load(Ordering::Acquire),cancel.clone()).unwrap()));
        current.store(false,Ordering::Release);let until=Instant::now()+Duration::from_secs(1);while !cancel.load(Ordering::Acquire){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}assert!(request.validate().is_err());
    }
    #[test]
    fn replacing_a_copied_file_or_redirecting_it_revokes_its_stamp(){
        let dir=std::env::temp_dir().join(format!("coucou-attachment-stamp-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&dir).unwrap();let path=dir.join("synthetic");std::fs::write(&path,"before").unwrap();let before=Stamp::at(path.to_str().unwrap()).unwrap();
        std::fs::write(dir.join("next"),"before").unwrap();std::fs::rename(dir.join("next"),&path).unwrap();assert!(Stamp::at(path.to_str().unwrap()).as_ref()!=Some(&before));
        #[cfg(unix)] {std::fs::remove_file(&path).unwrap();std::os::unix::fs::symlink(dir.join("elsewhere"),&path).unwrap();assert!(Stamp::at(path.to_str().unwrap()).is_none());}
        std::fs::remove_dir_all(dir).unwrap();
    }
}
