//! Real private D-Bus acceptance. Never use or change the host session bus.
use super::*;
use crate::pet_data::tests::Temp;
use std::{fs,io::{BufRead,BufReader,Read,Write},os::unix::{ffi::OsStrExt,fs::{symlink,DirBuilderExt,MetadataExt,PermissionsExt}},process::{Child,Command,Stdio},sync::mpsc,time::Instant};
const APP:&str="fr.louisraille.coucou";
struct Bus{child:Child,address:String,_temp:Temp}
impl Bus{
    fn new()->Self{
        let temp=Temp::new();let address=format!("unix:path={}",temp.0.join("bus").display());
        let mut child=Command::new("/usr/bin/dbus-daemon").args(["--session","--nofork","--nopidfile","--print-address=1"]).arg(format!("--address={address}")).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().expect("project image must provide dbus-daemon");
        let out=child.stdout.take().unwrap();let(tx,rx)=mpsc::channel();let reader=std::thread::spawn(move||{let mut line=String::new();let result=BufReader::new(out).read_line(&mut line);let _=tx.send((result,line));});
        match rx.recv_timeout(Duration::from_secs(3)){
            Ok((Ok(n),line))if n>0=>{reader.join().unwrap();let address=line.trim().to_string();local_address(&address).unwrap();Self{child,address,_temp:temp}},
            _=>{let _=child.kill();let _=child.wait();let _=reader.join();panic!("private test bus did not become ready");}
        }
    }
    fn connection(&self)->Connection{Connection::new_address(&self.address).unwrap()}
    fn owner(&self)->Connection{let owner=self.connection();assert_eq!(owner.request_name(name(APP).unwrap(),false,false,true).unwrap(),dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply::PrimaryOwner);owner}
    fn stop(&mut self){let _=self.child.kill();let _=self.child.wait();}
}
impl Drop for Bus{fn drop(&mut self){self.stop();}}
fn claim(bus:&Bus,layout:&Layout)->Result<Claim>{Claim::at(&bus.address,APP,&layout.legacy_config)}
fn rejected(result:Result<Claim>)->Error{match result{Err(error)=>error,Ok(_)=>panic!("expected native startup refusal")}}
pub(in crate::pet_data)struct Harness{owner:Connection,bus:Bus}
impl Harness{
    pub(in crate::pet_data)fn new()->Self{let bus=Bus::new();let owner=bus.owner();Self{owner,bus}}
    pub(in crate::pet_data)fn claim(&self,layout:&Layout)->Result<Claim>{claim(&self.bus,layout)}
    pub(in crate::pet_data)fn release(&self){self.owner.release_name(name(APP).unwrap()).unwrap();}
}

#[test]fn original_plugin_name_must_be_owned_before_any_lock_is_created(){
    let bus=Bus::new();let t=Temp::new();let layout=t.basic();
    assert_eq!(rejected(claim(&bus,&layout)),Error::OwnershipLost);assert!(!layout.legacy_config.join(LOCK).exists());
}
#[test]fn real_bus_pid_uid_unique_owner_and_private_file_lock_produce_a_live_claim(){
    let bus=Bus::new();let owner=bus.owner();let t=Temp::new();let layout=t.basic();
    fs::set_permissions(&layout.legacy_config,fs::Permissions::from_mode(0o755)).unwrap();
    let live=claim(&bus,&layout).unwrap();live.current().unwrap();live.for_layout(&layout).unwrap();
    assert_eq!(fs::metadata(&layout.legacy_config).unwrap().mode()&0o7777,0o755);let lock=layout.legacy_config.join(LOCK);
    assert_eq!(fs::metadata(&lock).unwrap().mode()&0o7777,0o600);assert_eq!(fs::metadata(&lock).unwrap().len(),0);
    drop(live);assert!(lock.exists());assert_eq!(owner.release_name(name(APP).unwrap()).unwrap(),dbus::blocking::stdintf::org_freedesktop_dbus::ReleaseNameReply::Released);
}
#[test]fn a_second_claim_cannot_share_the_same_profile_and_drop_releases_only_its_lock(){
    let bus=Bus::new();let _owner=bus.owner();let t=Temp::new();let layout=t.basic();
    let first=claim(&bus,&layout).unwrap();let inode=fs::metadata(layout.legacy_config.join(LOCK)).unwrap().ino();
    assert_eq!(rejected(claim(&bus,&layout)),Error::Busy);first.current().unwrap();drop(first);
    let next=claim(&bus,&layout).unwrap();next.current().unwrap();assert_eq!(fs::metadata(layout.legacy_config.join(LOCK)).unwrap().ino(),inode);
}
#[test]fn losing_and_reacquiring_name_in_the_same_pid_does_not_revive_an_old_claim(){
    let bus=Bus::new();let owner=bus.owner();let t=Temp::new();let layout=t.basic();let old=claim(&bus,&layout).unwrap();
    owner.release_name(name(APP).unwrap()).unwrap();assert_eq!(old.current(),Err(Error::OwnershipLost));
    let _new_owner=bus.owner();assert_eq!(old.current(),Err(Error::OwnershipLost));assert_eq!(rejected(claim(&bus,&layout)),Error::Busy);
    drop(old);claim(&bus,&layout).unwrap().current().unwrap();
}
#[test]fn observed_ownership_loss_is_terminal_even_if_the_same_connection_reacquires_name(){
    let bus=Bus::new();let owner=bus.owner();let t=Temp::new();let layout=t.basic();let old=claim(&bus,&layout).unwrap();
    owner.release_name(name(APP).unwrap()).unwrap();assert_eq!(old.current(),Err(Error::OwnershipLost));
    owner.request_name(name(APP).unwrap(),false,false,true).unwrap();assert_eq!(old.current(),Err(Error::OwnershipLost));
    drop(old);claim(&bus,&layout).unwrap().current().unwrap();
}
#[test]fn bus_loss_revokes_claim_without_releasing_or_recreating_profile_state(){
    let mut bus=Bus::new();let _owner=bus.owner();let t=Temp::new();let layout=t.basic();let live=claim(&bus,&layout).unwrap();let inode=fs::metadata(layout.legacy_config.join(LOCK)).unwrap().ino();
    bus.stop();assert!(live.current().is_err());assert_eq!(fs::metadata(layout.legacy_config.join(LOCK)).unwrap().ino(),inode);assert!(!layout.config().exists());assert!(!layout.data().exists());
}
#[test]fn unlink_and_replacement_cannot_turn_an_old_file_descriptor_into_current_ownership(){
    let bus=Bus::new();let _owner=bus.owner();let t=Temp::new();let layout=t.basic();let live=claim(&bus,&layout).unwrap();let path=layout.legacy_config.join(LOCK);
    fs::remove_file(&path).unwrap();t.file(&path,b"");assert!(live.current().is_err());assert_eq!(fs::read(&path).unwrap(),b"");
}
#[test]fn special_linked_unprivate_or_nonempty_locks_are_never_adopted_or_overwritten(){
    let bus=Bus::new();let _owner=bus.owner();
    for kind in ["symlink","fifo","directory","hardlink","unprivate","nonempty"]{
        let t=Temp::new();let layout=t.basic();let path=layout.legacy_config.join(LOCK);
        match kind{
            "symlink"=>symlink(t.0.join("foreign"),&path).unwrap(),
            "fifo"=>{let path=std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();assert_eq!(unsafe{libc::mkfifo(path.as_ptr(),0o600)},0);},
            "directory"=>fs::create_dir(&path).unwrap(),
            "hardlink"=>{t.file(&path,b"");fs::hard_link(&path,t.0.join("alias")).unwrap();},
            "unprivate"=>{t.file(&path,b"");fs::set_permissions(&path,fs::Permissions::from_mode(0o644)).unwrap();},
            "nonempty"=>t.file(&path,b"keep foreign contents"),
            _=>unreachable!(),
        }
        let before=fs::symlink_metadata(&path).unwrap();assert_eq!(rejected(claim(&bus,&layout)),Error::UnsafePath,"{kind}");let after=fs::symlink_metadata(&path).unwrap();assert_eq!((before.ino(),before.mode(),before.len()),(after.ino(),after.mode(),after.len()),"{kind}");
    }
}
#[test]fn native_claim_is_bound_to_the_original_profile_root(){
    let bus=Bus::new();let _owner=bus.owner();let t=Temp::new();let layout=t.basic();let live=claim(&bus,&layout).unwrap();let other=Temp::new().layout();
    assert_eq!(live.for_layout(&other),Err(Error::UnsafePath));live.current().unwrap();
}
#[test]fn replacing_profile_root_cannot_rebind_live_lock_to_a_different_directory(){
    let bus=Bus::new();let _owner=bus.owner();let t=Temp::new();let layout=t.basic();let live=claim(&bus,&layout).unwrap();
    let kept=t.0.join("kept-config");fs::rename(&layout.legacy_config,&kept).unwrap();fs::DirBuilder::new().mode(0o700).create(&layout.legacy_config).unwrap();
    assert!(live.current().is_err());assert!(kept.join(LOCK).exists());assert!(!layout.legacy_config.join(LOCK).exists());
}
#[test]fn ownership_precedes_native_review_and_copy_without_changing_original_data_paths(){
    let bus=Bus::new();let _owner=bus.owner();let t=Temp::new();let layout=t.basic();let live=claim(&bus,&layout).unwrap();
    let workspace=t.0.join("workspace");fs::DirBuilder::new().mode(0o700).create(&workspace).unwrap();
    let review=super::super::LegacyReview::inspect(layout.clone()).unwrap();assert_eq!(review.summary().excluded_roots,1);
    let output=super::super::staging::prepare(review,&workspace).unwrap();live.for_layout(&layout).unwrap();output.verify().unwrap();
    assert!(!layout.config().exists());assert!(!layout.data().exists());
}
#[test]fn session_bus_address_and_original_app_name_are_closed_native_inputs(){
    for address in ["", "tcp:host=127.0.0.1,port=1", "autolaunch:", "unix:path=/tmp/a;tcp:host=x", "unix:path=/tmp/\n"]{assert_eq!(local_address(address),Err(Error::UnsafePath));}
    for identifier in ["", ":1.20", "invalid name", "app;other"]{assert!(name(identifier).is_err());}
    assert_eq!(name(APP).unwrap(),"fr.louisraille.coucou.SingleInstance");assert!(local_address("unix:path=/private/test-bus,guid=0123").is_ok());
}

const ADDRESS:&str="COUCOU_PROFILE_TEST_BUS_ADDRESS";const OWNER_NAME:&str="COUCOU_PROFILE_TEST_OWNER_NAME";
struct OwnerProcess(Child);
impl Drop for OwnerProcess{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
#[test]fn another_real_process_owning_the_name_cannot_authorize_this_process(){
    let bus=Bus::new();let t=Temp::new();let layout=t.basic();
    let mut child=OwnerProcess(Command::new(std::env::current_exe().unwrap()).args(["--exact","pet_data::startup::tests::other_process_owns_name","--ignored","--nocapture","--test-threads=1"]).env(ADDRESS,&bus.address).env(OWNER_NAME,name(APP).unwrap()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap());
    let out=child.0.stdout.take().unwrap();let(tx,rx)=mpsc::channel();let reader=std::thread::spawn(move||{for line in BufReader::new(out).lines(){if line.is_ok_and(|s|s.contains("PROFILE_TEST_OWNER_READY")){let _=tx.send(());}}});
    let mut errors=child.0.stderr.take().unwrap();let diagnostic=std::thread::spawn(move||{let mut prefix=vec![];let mut buf=[0u8;1024];loop{let n=errors.read(&mut buf).unwrap();if n==0{break;}prefix.extend_from_slice(&buf[..n.min(8192-prefix.len())]);}String::from_utf8_lossy(&prefix).into_owned()});
    if rx.recv_timeout(Duration::from_secs(3)).is_err(){drop(child);reader.join().unwrap();panic!("private owner child was not ready: {}",diagnostic.join().unwrap());}
    assert_eq!(rejected(claim(&bus,&layout)),Error::OwnershipLost);assert!(!layout.legacy_config.join(LOCK).exists());
    child.0.stdin.take().unwrap().write_all(b"done\n").unwrap();let deadline=Instant::now()+Duration::from_secs(3);
    let status=loop{if let Some(status)=child.0.try_wait().unwrap(){break status;}assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));};
    reader.join().unwrap();assert!(status.success(),"private owner child {status}: {}",diagnostic.join().unwrap());
}
#[test]#[ignore="private process fixture, explicitly executed by another_real_process_owning_the_name_cannot_authorize_this_process"]
fn other_process_owns_name(){
    let address=std::env::var(ADDRESS).unwrap();local_address(&address).unwrap();let name=std::env::var(OWNER_NAME).unwrap();let owner=Connection::new_address(&address).unwrap();
    assert_eq!(owner.request_name(name,false,false,true).unwrap(),dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply::PrimaryOwner);
    println!("PROFILE_TEST_OWNER_READY");std::io::stdout().flush().unwrap();let mut line=String::new();std::io::stdin().read_line(&mut line).unwrap();
}
