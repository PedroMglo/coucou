use super::*;
use std::{io::Write,os::unix::{fs::{DirBuilderExt,PermissionsExt},net::UnixStream},os::fd::IntoRawFd};
pub(in crate::pet_documents) struct Fixture{pub root:PathBuf,pub leaf:PathBuf,pub attachment:Attachment}
impl Fixture{
    pub(in crate::pet_documents) fn new()->Self{
        let root=std::env::temp_dir().join(format!("coucou-folder-lease-{}",crate::assistant_memory::new_id()));std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let folder=root.join("folder");std::fs::DirBuilder::new().mode(0o700).create(&folder).unwrap();std::fs::DirBuilder::new().mode(0o700).create(folder.join("nested")).unwrap();let leaf=folder.join("nested/a.txt");std::fs::write(&leaf,b"old").unwrap();std::fs::set_permissions(&leaf,std::fs::Permissions::from_mode(0o600)).unwrap();
        let attachment=Attachment{id:"fixture-folder".into(),provider:"ollama".into(),conversation_id:"original-cid".into(),sequence:7,file:crate::files::DroppedFile{name:"folder".into(),path:folder.to_string_lossy().into_owned(),size:3,kind:"folder".into(),entries:2}};
        Self{root,leaf,attachment}
    }
    pub(in crate::pet_documents) fn capture(&self,cancel:Arc<AtomicBool>)->Folders{Folders::capture(&self.root,std::slice::from_ref(&self.attachment),deadline(),cancel).unwrap().unwrap()}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
fn deadline()->Instant{Instant::now()+Duration::from_secs(8)}
#[test]
fn complete_folder_snapshot_keeps_quiet_reads_and_unrelated_inbox_imports_valid(){
    let fixture=Fixture::new();let folders=fixture.capture(Arc::default());assert!(folders.current());assert!(folders.contents_current(deadline()));
    assert_eq!(std::fs::read(&fixture.leaf).unwrap(),b"old");std::fs::create_dir(fixture.root.join("unrelated")).unwrap();std::fs::write(fixture.root.join("unrelated/another.txt"),b"unrelated").unwrap();
    for _ in 0..10{assert!(folders.current());}assert!(folders.contents_current(deadline()));
}
#[test]
fn nested_mutation_replacement_permissions_links_and_directory_changes_revoke_sticky_scope(){
    for reason in ["edit","replace","delete","add","rename-dir","chmod","hardlink","symlink"]{
        let fixture=Fixture::new();let folders=fixture.capture(Arc::default());
        match reason{
            "edit"=>std::fs::write(&fixture.leaf,b"new").unwrap(),
            "replace"=>{let replacement=fixture.root.join("replacement");std::fs::write(&replacement,b"old").unwrap();std::fs::set_permissions(&replacement,std::fs::Permissions::from_mode(0o600)).unwrap();std::fs::rename(replacement,&fixture.leaf).unwrap();},
            "delete"=>std::fs::remove_file(&fixture.leaf).unwrap(),
            "add"=>std::fs::write(fixture.leaf.with_file_name("new.txt"),b"new").unwrap(),
            "rename-dir"=>std::fs::rename(fixture.leaf.parent().unwrap(),fixture.leaf.parent().unwrap().with_file_name("moved")).unwrap(),
            "chmod"=>std::fs::set_permissions(&fixture.leaf,std::fs::Permissions::from_mode(0o644)).unwrap(),
            "hardlink"=>std::fs::hard_link(&fixture.leaf,fixture.root.join("alias")).unwrap(),
            _=>std::os::unix::fs::symlink("/etc/passwd",fixture.leaf.with_file_name("link")).unwrap(),
        }
        assert!(!folders.current(),"{reason}");assert!(!folders.contents_current(deadline()),"{reason}");assert!(!folders.current(),"revocation is sticky: {reason}");
    }
}
#[test]
fn setup_race_never_rebinds_to_the_new_nested_revision(){
    let fixture=Fixture::new();let result=Folders::capture_using(&fixture.root,std::slice::from_ref(&fixture.attachment),deadline(),Arc::default(),|_|{std::fs::write(&fixture.leaf,b"new").unwrap();});assert!(result.is_err());
}
#[test]
fn multiple_folders_share_one_scope_while_other_requests_stay_independent(){
    let first=Fixture::new();let second=Fixture::new();let other=Fixture::new();let unrelated=other.capture(Arc::default());
    let destination=first.root.join("second");std::fs::rename(&second.attachment.file.path,&destination).unwrap();let mut attachment=second.attachment.clone();attachment.id="second-folder".into();attachment.sequence+=1;attachment.file.path=destination.to_string_lossy().into_owned();
    let folders=Folders::capture(&first.root,&[first.attachment.clone(),attachment],deadline(),Arc::default()).unwrap().unwrap();assert!(folders.contents_current(deadline()));
    std::fs::write(destination.join("nested/a.txt"),b"new").unwrap();assert!(!folders.current());assert!(unrelated.current());assert!(unrelated.contents_current(deadline()));
}
#[test]
fn duplicate_inode_watch_setup_is_rejected_instead_of_overwriting_a_watch(){
    let fixture=Fixture::new();assert!(Folders::capture(&fixture.root,&[fixture.attachment.clone(),fixture.attachment.clone()],deadline(),Arc::default()).is_err());
}
#[test]
fn full_metadata_and_hash_boundary_detects_a_missed_event(){
    let fixture=Fixture::new();let folders=fixture.capture(Arc::default());std::fs::write(&fixture.leaf,b"new").unwrap();
    // Simulate a notification lost by the OS. No production API can clear the
    // queue or reset sticky revocation; the full boundary must still catch it.
    {let mut queue=folders.monitor.file.lock().unwrap();let mut bytes=[0;4096];while queue.read(&mut bytes).is_ok(){}}
    assert!(folders.current(),"root metadata is unchanged by a deep in-place edit");assert!(!folders.contents_current(deadline()));assert!(!folders.current());
}
#[test]
fn mapped_writes_are_checked_by_the_full_boundary_even_without_inotify(){
    let fixture=Fixture::new();let file=std::fs::OpenOptions::new().read(true).write(true).open(&fixture.leaf).unwrap();
    struct Map(*mut libc::c_void);impl Drop for Map{fn drop(&mut self){unsafe{libc::munmap(self.0,3);}}}
    let address=unsafe{libc::mmap(std::ptr::null_mut(),3,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED,file.as_raw_fd(),0)};assert_ne!(address,libc::MAP_FAILED);let map=Map(address);
    let folders=fixture.capture(Arc::default());unsafe{std::ptr::write_volatile(map.0.cast::<u8>(),b'X');}assert!(!folders.contents_current(deadline()));
}
#[test]
fn overflow_unknown_events_and_broken_queue_are_conservative_revocations(){
    for payload in [libc::IN_Q_OVERFLOW.to_ne_bytes().to_vec(),vec![1]]{
        let(mut writer,reader)=UnixStream::pair().unwrap();reader.set_nonblocking(true).unwrap();let monitor=Monitor{file:Mutex::new(unsafe{File::from_raw_fd(reader.into_raw_fd())}),revoked:AtomicBool::new(false)};
        assert!(monitor.current());writer.write_all(&payload).unwrap();assert!(!monitor.current());assert!(!monitor.current());
    }
    let(writer,reader)=UnixStream::pair().unwrap();reader.set_nonblocking(true).unwrap();let monitor=Monitor{file:Mutex::new(unsafe{File::from_raw_fd(reader.into_raw_fd())}),revoked:AtomicBool::new(false)};drop(writer);assert!(!monitor.current());
}
#[test]
fn folder_count_deadline_cancellation_and_private_copy_contract_are_enforced(){
    assert!(Folders::capture(Path::new("/never-open"),&[],Instant::now(),Arc::default()).unwrap().is_none());
    let fixture=Fixture::new();let many=vec![fixture.attachment.clone();MAX_CONVERSATION_ATTACHMENTS+1];assert!(Folders::capture(&fixture.root,&many,deadline(),Arc::default()).is_err());
    assert!(Folders::capture(&fixture.root,std::slice::from_ref(&fixture.attachment),Instant::now()-Duration::from_secs(1),Arc::default()).is_err());assert!(Folders::capture(&fixture.root,std::slice::from_ref(&fixture.attachment),deadline(),Arc::new(AtomicBool::new(true))).is_err());
    let mut wrong=fixture.attachment.clone();wrong.file.entries+=1;assert!(Folders::capture(&fixture.root,&[wrong],deadline(),Arc::default()).is_err());std::fs::set_permissions(&fixture.leaf,std::fs::Permissions::from_mode(0o644)).unwrap();assert!(Folders::capture(&fixture.root,std::slice::from_ref(&fixture.attachment),deadline(),Arc::default()).is_err());
}
#[test]
fn read_budget_hashes_exact_bytes_and_honors_original_deadline_and_cancel(){
    let fixture=Fixture::new();let file=File::open(&fixture.leaf).unwrap();let cancel=Arc::new(AtomicBool::new(false));let mut budget=ReadBudget{deadline:deadline(),bytes:0,cancel:cancel.clone()};let hash=budget.hash(&file).unwrap();assert_eq!(hash.as_slice(),ring::digest::digest(&ring::digest::SHA256,b"old").as_ref());assert_eq!(budget.bytes,3);
    cancel.store(true,Ordering::Release);assert!(budget.hash(&file).is_err());
    let mut expired=ReadBudget{deadline:Instant::now()-Duration::from_secs(1),bytes:0,cancel:Arc::default()};assert!(expired.hash(&file).is_err());
}
