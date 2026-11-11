//! Real native protocol against a private PipeWire graph. No user devices,
//! authenticated profiles, global services, model quota or installed app writes.
use super::*;
use std::{fs,io,os::unix::fs::DirBuilderExt,sync::{Mutex,OnceLock,atomic::AtomicUsize}};
use crate::pet_process_group::OwnedChild;
const CONFIG:&str=r#"
context.properties = { core.daemon=true core.name=pipewire-0 support.dbus=false mem.allow-mlock=false }
context.spa-libs = { audio.convert.*=audioconvert/libspa-audioconvert audio.adapt=audioconvert/libspa-audioconvert support.*=support/libspa-support }
context.modules = [
 { name=libpipewire-module-protocol-native }
 { name=libpipewire-module-spa-node-factory }
 { name=libpipewire-module-client-node }
 { name=libpipewire-module-metadata }
 { name=libpipewire-module-access }
 { name=libpipewire-module-adapter }
]
context.objects = [
 { factory=adapter args={ factory.name=support.null-audio-sink node.name=coucou-fixture-output node.description=PrivateOutput media.class=Audio/Sink audio.position=[ FL FR ] } }
]
"#;
fn serial()->std::sync::MutexGuard<'static,()>{static LOCK:OnceLock<Mutex<()>>=OnceLock::new();LOCK.get_or_init(||Mutex::new(())).lock().unwrap_or_else(|p|p.into_inner())}
pub(crate)struct Fixture{pulse:Option<OwnedChild>,daemon:Option<OwnedChild>,profile:Profile}
impl Fixture{
 pub(crate)fn new()->Self{
  let profile=Profile::new().unwrap();for dir in ["config","cache","data"]{fs::DirBuilder::new().mode(0o700).create(profile.0.join(dir)).unwrap();}
  fs::write(profile.0.join("private.conf"),CONFIG).unwrap();let mut f=Self{pulse:None,daemon:None,profile};f.launch();f
 }
 fn launch(&mut self){
  let mut cmd=crate::pet_helper::command("/usr/bin/pipewire",&["-c".into(),self.profile.0.join("private.conf").to_str().unwrap().into()]);
  cmd.env_clear().env("PATH","/usr/bin:/bin").env("LC_ALL","C").env("XDG_RUNTIME_DIR",&self.profile.0).env("PIPEWIRE_RUNTIME_DIR",&self.profile.0)
   .env("XDG_CONFIG_HOME",self.profile.0.join("config")).env("XDG_CACHE_HOME",self.profile.0.join("cache")).env("XDG_DATA_HOME",self.profile.0.join("data")).env("PIPEWIRE_CONFIG_DIR",&self.profile.0)
   .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
  self.daemon=Some(OwnedChild::new(cmd.spawn().unwrap()));let end=Instant::now()+Duration::from_secs(3);
  while !self.profile.0.join("pipewire-0").exists(){assert!(Instant::now()<end,"Private PipeWire startup exceeded limit");assert!(!self.daemon.as_ref().unwrap().exited().unwrap(),"Private PipeWire exited before its socket");std::thread::sleep(Duration::from_millis(10));}
 }
 pub(crate)fn restart(&mut self){let had_pulse=self.pulse.is_some();drop(self.pulse.take());drop(self.daemon.take());for name in ["pipewire-0","pipewire-0.lock"]{let q=self.profile.0.join(name);match fs::remove_file(q){Ok(())=>{},Err(e)if e.kind()==io::ErrorKind::NotFound=>{},Err(e)=>panic!("Private stopped fixture cleanup: {e}")}}self.launch();if had_pulse{self.enable_pulse();}}
 pub(crate)fn directory(&self)->&std::path::Path{&self.profile.0}
 pub(crate)fn enable_pulse(&mut self){
  assert!(self.pulse.is_none());let dir=self.profile.0.join("pulse");if !dir.exists(){fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();}
  let socket=dir.join("native");let _=fs::remove_file(&socket);
  let config=r#"context.properties = { support.dbus=false }
context.spa-libs = { support.*=support/libspa-support }
context.modules = [
 { name=libpipewire-module-protocol-native }
 { name=libpipewire-module-client-node }
 { name=libpipewire-module-protocol-pulse args={ server.address=[ "unix:@SOCKET@" ] } }
]
"#.replace("@SOCKET@",socket.to_str().unwrap());fs::write(self.profile.0.join("private-pulse.conf"),config).unwrap();
  let mut cmd=crate::pet_helper::command("/usr/bin/pipewire-pulse",&["-c".into(),self.profile.0.join("private-pulse.conf").to_str().unwrap().into()]);
  cmd.env_clear().env("PATH","/usr/bin:/bin").env("LC_ALL","C").env("PIPEWIRE_RUNTIME_DIR",&self.profile.0).env("PIPEWIRE_REMOTE","pipewire-0").env("XDG_RUNTIME_DIR",&self.profile.0)
   .env("XDG_CONFIG_HOME",self.profile.0.join("config")).env("XDG_CACHE_HOME",self.profile.0.join("cache")).env("XDG_DATA_HOME",self.profile.0.join("data")).env("PIPEWIRE_CONFIG_DIR",&self.profile.0)
   .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
  self.pulse=Some(OwnedChild::new(cmd.spawn().unwrap()));let end=Instant::now()+Duration::from_secs(3);
  while !socket.exists(){assert!(Instant::now()<end,"Private Pulse startup exceeded limit");assert!(!self.pulse.as_ref().unwrap().exited().unwrap(),"Private Pulse exited before its socket");std::thread::sleep(Duration::from_millis(10));}
 }
 pub(crate)fn external_volume(&self,percent:u8){let(r,_)=self.selected(true,Arc::new(||true));let out=r.dispatch(NativeAction::Volume{percent},Instant::now()+Duration::from_secs(5),&||true,||Ok(()));assert!(out.verified);}
 pub(crate)fn pause_core(&self){self.daemon.as_ref().unwrap().signal(libc::SIGSTOP).unwrap();}
 pub(crate)fn resume_core(&self){self.daemon.as_ref().unwrap().signal(libc::SIGCONT).unwrap();}
 pub(crate)fn open(&self,effects:bool,authority:Arc<dyn Fn()->bool+Send+Sync>)->Reader{
  Reader::start_at(Instant::now()+Duration::from_secs(10),authority,self.profile.0.as_os_str().into(),if effects{Mode::Effect}else{Mode::ReadOnly},&||true).unwrap()
 }
 fn selected(&self,effects:bool,authority:Arc<dyn Fn()->bool+Send+Sync>)->(Reader,Output){let mut r=self.open(effects,authority);let nodes=r.outputs().unwrap();assert_eq!(nodes.len(),1,"Private graph must expose exactly its own virtual node");let output=r.capture(&nodes[0]).unwrap();(r,output)}
 pub(crate)fn observed(&self)->Output{self.selected(false,Arc::new(||true)).1}
 pub(crate)fn cli(&self,args:&[String]){
  let mut cmd=crate::pet_helper::command("/usr/bin/pw-cli",args);cmd.env_clear().env("PATH","/usr/bin:/bin").env("LC_ALL","C").env("PIPEWIRE_RUNTIME_DIR",&self.profile.0).env("XDG_RUNTIME_DIR",&self.profile.0).env("XDG_CONFIG_HOME",self.profile.0.join("config"));
  crate::pet_helper::Session::acquire(Lane::AudioRead,Duration::from_secs(3)).unwrap().run_sync_input(cmd,None,64*1024,Duration::from_secs(3),||Ok(()),&||true).unwrap();
 }
}
fn no_group(group:u32)->bool{
 fs::read_dir("/proc").unwrap().take(131072).filter_map(Result::ok).all(|e|{
  let Ok(s)=fs::read_to_string(e.path().join("stat"))else{return true};let Some(at)=s.rfind(')')else{return true};let fields:Vec<_>=s[at+1..].split_whitespace().take(3).collect();fields.len()!=3||fields[2].parse::<u32>().ok()!=Some(group)||matches!(fields[0],"Z"|"X"|"x")
 })
}
fn same_state(a:&Output,b:&Output){assert_eq!(a.id,b.id);assert_eq!(a.serial,b.serial);assert_eq!(a.muted,b.muted);assert!((a.volume_percent.unwrap()-b.volume_percent.unwrap()).abs()<0.01);}
#[test]fn core_cookie_metadata_is_strict_unique_and_bounded_to_native_u32(){
 assert_eq!(parse_core_cookie(b"id: 0\n    cookie: 4294967295\nproperties:\n core.name=private"),Ok(u32::MAX));
 for data in [b"cookie: -1".as_slice(),b"cookie: 1\ncookie: 1".as_slice(),b"cookie: 4294967296".as_slice(),b"cookie: words".as_slice(),b"properties: no-cookie".as_slice()]{assert!(parse_core_cookie(data).is_err());}
}
#[test]fn typed_native_action_rejects_unknown_fields_and_out_of_range_before_effect(){
 for data in [json!({"kind":"shell","command":"anything"}),json!({"kind":"volume","percent":-1}),json!({"kind":"volume","percent":12,"petId":"other"}),json!({"kind":"mute","muted":"true"})]{assert!(serde_json::from_value::<NativeAction>(data).is_err());}
 assert!(!serde_json::from_value::<NativeAction>(json!({"kind":"volume","percent":101})).unwrap().valid());
 assert!(serde_json::from_value::<NativeAction>(json!({"kind":"volume","percent":0})).unwrap().valid());assert!(!crate::pet_audio::task::native_dispatch_available());
}
#[test]#[ignore="Private native PipeWire/Quickshell fixture; no user audio devices"]
fn native_private_effect_requires_intent_rechecks_stop_and_observes_a_separate_connection(){
 let _serial=serial();let f=Fixture::new();let baseline=f.observed();
 let(reader,_)=f.selected(true,Arc::new(||true));let pid=reader.owner.test_process_id();let denied=reader.dispatch(NativeAction::Volume{percent:61},Instant::now()+Duration::from_secs(5),&||true,||Err(()));
 assert!(!denied.possible_effect&&!denied.verified);assert!(no_group(pid));same_state(&baseline,&f.observed());
 let calls=AtomicUsize::new(0);let(reader,_)=f.selected(true,Arc::new(||true));let malformed=reader.dispatch(NativeAction::Volume{percent:101},Instant::now()+Duration::from_secs(5),&||true,||{calls.fetch_add(1,Ordering::SeqCst);Ok(())});assert!(!malformed.possible_effect);assert_eq!(calls.load(Ordering::SeqCst),0);same_state(&baseline,&f.observed());
 let flag=Arc::new(AtomicBool::new(true));let check=flag.clone();let(reader,_)=f.selected(true,Arc::new(move||check.load(Ordering::Acquire)));let stopped=reader.dispatch(NativeAction::Volume{percent:61},Instant::now()+Duration::from_secs(5),&||true,||{flag.store(false,Ordering::Release);Ok(())});assert!(!stopped.possible_effect&&!stopped.verified);same_state(&baseline,&f.observed());
 let(reader,_)=f.selected(true,Arc::new(||true));let pid=reader.owner.test_process_id();let effect=reader.dispatch(NativeAction::Volume{percent:61},Instant::now()+Duration::from_secs(5),&||true,||{calls.fetch_add(1,Ordering::SeqCst);Ok(())});
 assert!(effect.possible_effect&&effect.transport_accepted&&effect.scope_current&&effect.verified);assert_eq!(calls.load(Ordering::SeqCst),1);assert!(no_group(pid));assert!(NativeAction::Volume{percent:61}.satisfied(&f.observed()));
 let(reader,_)=f.selected(true,Arc::new(||true));let muted=reader.dispatch(NativeAction::Mute{muted:true},Instant::now()+Duration::from_secs(5),&||true,||Ok(()));assert!(muted.verified&&muted.transport_accepted);assert_eq!(f.observed().muted,Some(true));
 println!("NATIVE_PRIVATE_EFFECT intentRequired=true stopAfterIntentRefused=true independentVolume=61 independentMuted=true writerGroupsReaped=true userAudioWrites=0");
}
#[test]#[ignore="Private native PipeWire/Quickshell fixture; no user audio devices"]
fn native_private_readonly_replay_replacement_and_core_restart_cannot_retarget(){
 let _serial=serial();let mut f=Fixture::new();let baseline=f.observed();let transaction="a".repeat(64);let action=NativeAction::Volume{percent:37};
 let(mut reader,_)=f.selected(false,Arc::new(||true));assert!(reader.exchange_checked("prepare",None,Some(&transaction),Some(&action),&||true).is_err());drop(reader);same_state(&baseline,&f.observed());
 let(mut reader,_)=f.selected(true,Arc::new(||true));let prepared=reader.exchange_checked("prepare",None,Some(&transaction),Some(&action),&||true).unwrap();assert!(prepared.prepared&&!prepared.accepted);same_state(&baseline,&f.observed());
 let committed=reader.exchange_checked("commit",None,Some(&transaction),None,&||true).unwrap();assert!(committed.accepted);assert!(NativeAction::Volume{percent:37}.satisfied(&f.observed()));
 assert!(Reader::fixture_effect(f.directory()).is_err(),"Original writer still owns the physical effect slot");
 assert!(reader.exchange_checked("commit",None,Some(&transaction),None,&||true).is_err());drop(reader);
 let(other,_)=f.selected(true,Arc::new(||true));let external=other.dispatch(NativeAction::Volume{percent:23},Instant::now()+Duration::from_secs(5),&||true,||Ok(()));assert!(external.verified);assert!(NativeAction::Volume{percent:23}.satisfied(&f.observed()));
 let(mut captured,original)=f.selected(true,Arc::new(||true));f.cli(&["destroy".into(),original.id.to_string()]);
 f.cli(&["create-node".into(),"adapter".into(),"{ factory.name=support.null-audio-sink node.name=coucou-fixture-output node.description=PrivateOutput media.class=Audio/Sink audio.position=[ FL FR ] object.linger=true }".into()]);
 let replacement=f.observed();assert_ne!((replacement.id,&replacement.serial),(original.id,&original.serial));assert!(captured.read().is_err());assert!(captured.capture(&replacement).is_err());drop(captured);
 let(mut captured,old)=f.selected(true,Arc::new(||true));let old_cookie=captured.core_cookie();f.restart();let new=f.observed();assert_ne!(old_cookie,f.open(false,Arc::new(||true)).core_cookie());assert!(captured.read().is_err());assert!(captured.capture(&new).is_err());drop(captured);
 println!("NATIVE_PRIVATE_LOSS readOnlyRefused=true replayRefused=true replacementRefused=true coreRestartRefused=true idsMayRepeat={} userAudioWrites=0",old.id==new.id);
}
