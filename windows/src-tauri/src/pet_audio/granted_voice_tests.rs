use super::*;
use crate::pet_config::{self,system_access::{Policy,Grant,Resource,Subplugin}};
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
const A:&str="audio-0123456789abcdef";const B:&str="audio-fedcba9876543210";
struct Fixture{path:std::path::PathBuf}
impl Fixture{
 fn new()->Self{let path=std::env::temp_dir().join(format!("coucou-granted-sound-{}",{let mut b=[0u8;16];ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(),&mut b).unwrap();b.iter().map(|n|format!("{n:02x}")).collect::<String>()}));std::fs::create_dir(&path).unwrap();let f=Self{path};f.save(0);f}
 fn file(&self)->std::path::PathBuf{self.path.join("pets.json")}
 fn config(&self)->pet_config::Configuration{let mut c=pet_config::defaults();c.version=2;for(pet,ch)in[("ai_ollama",'a'),("integration_spotify",'b'),("agent_codex",'c'),("agent_copilot",'d'),("agent_antigravity",'e')]{c.pets.get_mut(pet).unwrap().system_access=Some(Policy{schema_version:1,enabled:true,revision:ch.to_string().repeat(32),grants:[Operation::Observe,Operation::Volume,Operation::Mute].into_iter().map(|operation|Grant{subplugin:Subplugin::Sound,operation,resource:Resource::AudioOutput{key:A.into()}}).collect()});}c}
 fn save(&self,mode:u8){let mut c=self.config();if mode==1{c.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().revision="f".repeat(32);}if mode==2{c.pets.get_mut("integration_spotify").unwrap().system_access.as_mut().unwrap().revision="f".repeat(32);}std::fs::write(self.file(),serde_json::to_vec(&c).unwrap()).unwrap();}
 fn scope(&self,pet:&str)->Captured{Captured::fixture(&self.file(),pet,Operation::Observe,None).unwrap()}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.path);}}
fn node(key:&str,default:bool)->super::super::Node{super::super::Node{key:key.into(),kind:Kind::Output,label:"Synthetic native output".into(),description:"Fixture".into(),volume:Some(20.0),muted:Some(false),balance:None,is_default:default,device_key:None,monitor:false,controllable:true,can_volume:true,can_balance:false,active:true,options:vec![],active_option:None,selector:"private-selector".into(),index:1,native_id:None,native_serial:None}}
fn snapshot(key:&str)->Snapshot{Snapshot{available:true,observed_at:Some(1),outputs:vec![node(key,true)],..Snapshot::default()}}
fn current()->Current{Arc::new(||true)}
#[tokio::test]async fn system_sound_own_native_voice_grant_pins_each_pets_output_without_models_or_spotify_permissions(){
 let f=Fixture::new();
 for pet in ["ai_ollama","integration_spotify","agent_codex","agent_copilot","agent_antigravity"]{
  let c=f.config();assert!(c.pets[pet].inference.is_none());let count=AtomicUsize::new(0);
  let result=execute(f.scope(pet),SoundCommand::Volume{input:false,percent:35.0},current(),|check|async move{assert!(check());snapshot(A)},|request,check|{let count=&count;async move{count.fetch_add(1,Ordering::SeqCst);assert_eq!(request.target_key,A);assert_eq!(request.action,Action::Volume);assert_eq!(request.value,Some(35.0));assert!(check());Ok(Receipt{snapshot:snapshot(A),confirmed:true,warning:None})}}).await.unwrap();
  assert!(result.1);assert_eq!(count.load(Ordering::SeqCst),1);
 }
}
#[tokio::test]async fn system_sound_default_output_is_not_replaced_by_a_different_granted_output(){
 let f=Fixture::new();let r=execute(f.scope("ai_ollama"),SoundCommand::Mute{input:false,muted:true},current(),|_|async{let mut s=snapshot(B);s.outputs.push(node(A,false));s},|_,_|async{panic!("must not choose another output")}).await;assert!(r.is_err());
}
#[tokio::test]async fn system_sound_voice_stop_or_revocation_during_discovery_prevents_control_and_other_pet_changes_do_not(){
 for cause in [0,1,2]{let f=Fixture::new();let live=Arc::new(AtomicBool::new(true));let expected=live.clone();let count=AtomicUsize::new(0);
  let result=execute(f.scope("ai_ollama"),SoundCommand::Mute{input:false,muted:true},Arc::new(move||expected.load(Ordering::Acquire)),|check|{let f=&f;let live=&live;async move{assert!(check());match cause{0=>live.store(false,Ordering::Release),1=>f.save(1),_=>f.save(2)};snapshot(A)}},|_,check|{let count=&count;async move{count.fetch_add(1,Ordering::SeqCst);assert!(check());Ok(Receipt{snapshot:snapshot(A),confirmed:true,warning:None})}}).await;
  if cause<2{assert!(result.is_err());assert_eq!(count.load(Ordering::SeqCst),0);}else{assert!(result.unwrap().1);assert_eq!(count.load(Ordering::SeqCst),1);}
 }
}
#[tokio::test]async fn system_sound_native_executor_wait_keeps_the_original_key_and_late_revocation_cannot_publish_success(){
 let f=Fixture::new();let result=execute(f.scope("ai_ollama"),SoundCommand::Volume{input:false,percent:40.0},current(),|_|async{snapshot(A)},|request,check|async move{assert_eq!(request.target_key,A);assert!(check());f.save(1);assert!(!check());Ok(Receipt{snapshot:snapshot(B),confirmed:true,warning:None})}).await.unwrap();assert!(!result.1&&result.0.contains("não deve ser repetida"));
 let f=Fixture::new();let result=execute(f.scope("ai_ollama"),SoundCommand::Mute{input:false,muted:true},current(),|_|async{snapshot(A)},|_,_|async{Ok(Receipt{snapshot:snapshot(A),confirmed:true,warning:Some("Native effect provenance is uncertain".into())})}).await.unwrap();assert!(!result.1&&result.0.contains("uncertain"));
}
#[tokio::test]async fn system_sound_status_reads_only_the_authorized_default_and_never_controls_or_describes_microphones(){
 let f=Fixture::new();let r=execute(f.scope("ai_ollama"),SoundCommand::Status,current(),|_|async{let mut s=snapshot(A);let mut mic=node(B,true);mic.kind=Kind::Input;mic.label="PRIVATE_MICROPHONE".into();s.inputs.push(mic);s},|_,_|async{panic!("status is read-only")}).await.unwrap();assert!(!r.1&&!r.0.contains("PRIVATE_MICROPHONE"));
}
#[tokio::test]async fn system_sound_malformed_microphone_stopped_and_missing_operation_requests_refuse_before_service_io(){
 let f=Fixture::new();
 for command in [SoundCommand::Volume{input:true,percent:20.0},SoundCommand::Mute{input:true,muted:true},SoundCommand::Volume{input:false,percent:f64::NAN},SoundCommand::Volume{input:false,percent:101.0}]{assert!(execute(f.scope("ai_ollama"),command,current(),|_|async{panic!("unsupported request queried service")},|_,_|async{panic!("unsupported control")}).await.is_err());}
 assert!(execute(f.scope("ai_ollama"),SoundCommand::Status,Arc::new(||false),|_|async{panic!("stopped request queried service")},|_,_|async{panic!("stopped control")}).await.is_err());
 let stale=f.scope("ai_ollama");f.save(1);assert!(execute(stale,SoundCommand::Status,current(),|_|async{panic!("stale recognition grant queried service")},|_,_|async{panic!("stale control")}).await.is_err());f.save(0);
 let mut c=f.config();c.pets.get_mut("ai_ollama").unwrap().system_access.as_mut().unwrap().grants.retain(|g|g.operation==Operation::Observe);std::fs::write(f.file(),serde_json::to_vec(&c).unwrap()).unwrap();
 assert!(execute(f.scope("ai_ollama"),SoundCommand::Mute{input:false,muted:true},current(),|_|async{panic!("ungranted operation queried service")},|_,_|async{panic!("ungranted control")}).await.is_err());
}
#[test]fn system_sound_narrowed_output_aliases_share_observed_revocation_and_cannot_borrow_or_refresh_authority(){
 let f=Fixture::new();let origin=f.scope("ai_ollama");let pinned=origin.pin_output(Operation::Mute,A).unwrap();assert!(origin.pin_output(Operation::Mute,B).is_err());f.save(1);assert!(origin.check().is_err());f.save(0);assert!(pinned.check().is_err());
 assert!(Captured::fixture(&f.file(),"agent_gemini",Operation::Observe,None).is_err());assert!(Captured::fixture(&f.file(),"integration_github",Operation::Observe,None).is_err());
}
