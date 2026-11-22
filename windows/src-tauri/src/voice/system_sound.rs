//! Deterministic NativeUser route, before inference admission. The parser's pet,
//! original audio input, active voice owner and own grant supply the scope.
use super::{dialogue::Pet,input::Input,brain::Reply};
use crate::pet_audio::commands::SoundCommand;
use crate::pet_config::{Configuration,system_access::{Captured,Operation}};
use std::sync::{Arc,atomic::AtomicBool};
use tauri::{AppHandle,Manager};
fn command(query:&str)->Option<Result<SoundCommand,String>>{
 let text=super::wake::normalize(query);
 if let Some(command)=crate::pet_audio::commands::parse(&text){return Some(Ok(command));}
 // An explicit but malformed System command must not fall through to the
 // Spotify volume grammar or accidentally start an inference turn.
 let prefixes=["volume do sistema","volume do pc","system volume","pc volume","volume do microfone","microphone volume","mic volume","estado do som","estado do audio","audio status","sound status","silenciar sistema","silenciar o sistema","silenciar pc","mute system","mute pc","repor som do sistema","ativar som do sistema","unmute system","unmute pc","silenciar microfone","silenciar o microfone","mute microphone","mute mic","reativar microfone","reativar o microfone","unmute microphone","unmute mic"];
 prefixes.iter().any(|prefix|text==*prefix||text.strip_prefix(prefix).is_some_and(|tail|tail.starts_with(' '))).then(||Err("O comando de Som é inválido. Indica uma só operação, por exemplo volume do sistema 40 ou silenciar sistema.".into()))
}
pub(super)fn requested(query:&str)->bool{command(query).is_some()}
fn own_route(config:&Configuration,pet:Pet)->bool{pet!=Pet::Spotify||config.pets.get(pet.pill()).is_some_and(|p|p.system_access.is_some())}
fn active(settings:&crate::settings::Settings,pet:Pet)->bool{settings.main_pill==pet.pill()||settings.active_integrations.iter().any(|id|id==pet.pill())}
pub(super)fn spoken(app:&AppHandle,pet:Pet,query:&str,input:&Input,stop:&Arc<AtomicBool>)->Option<Reply>{
 let command=match command(query)?{Ok(command)=>command,Err(text)=>return Some(Reply{text,acted:false,language:Some("pt-PT".into())})};
 // No policy is the preserved Spotify legacy route. Any own policy, including
 // suspended/empty/denied policy, is handled here and NEVER falls back to music.
 if !own_route(&input.configuration,pet){return None;}
 let result=(||->Result<(String,bool),String>{
  if !super::session::current(stop)||!active(&input.settings,pet){return Err("O pedido de voz original terminou ou este pet não está ativo.".into());}
  input.source_current()?;
  if matches!(command,SoundCommand::Volume{input:true,..}|SoundCommand::Mute{input:true,..}){return Err("O microfone não faz parte desta concessão de saídas de Som.".into());}
  let scope=Captured::capture_reviewed(&input.configuration,pet.pill(),Operation::Observe,None)?;
  let original_input=input.clone();let original_stop=stop.clone();let original_app=app.clone();
  let current=Arc::new(move||super::session::current(&original_stop)&&original_input.source_current().is_ok()&&original_app.state::<crate::Shared>().settings.lock().is_ok_and(|s|active(&s,pet)));
  tauri::async_runtime::block_on(crate::pet_audio::granted_voice::spoken(scope,command,current))
 })();
 Some(match result{Ok((text,acted))=>Reply{text,acted,language:Some("pt-PT".into())},Err(text)=>Reply{text,acted:false,language:Some("pt-PT".into())}})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn existing_native_voice_pets_share_the_route_without_borrowing_spotify_or_selecting_a_model(){
  let config=crate::pet_config::defaults();
  for pet in [Pet::Codex,Pet::Copilot,Pet::Gravity,Pet::Ollama,Pet::Notion]{assert!(own_route(&config,pet));assert!(!super::super::admission::uses_pet_response("volume do sistema 40"));}
  assert!(!own_route(&config,Pet::Spotify));
  for text in ["estado do som","audio status","volume do sistema 40","mute system","silenciar microfone"]{assert!(requested(text));}
  for text in ["sim","yes","explica o som"]{assert!(!requested(text));}
  for text in ["volume do sistema 40 e apagar ficheiros","system volume 200","pc volume","sound status e volume 20"]{assert!(requested(text));assert!(command(text).unwrap().is_err());assert!(!super::super::admission::uses_pet_response(text));}
 }
 #[test]fn any_own_spotify_policy_claims_the_command_even_when_suspended_or_empty(){
  use crate::pet_config::system_access::Policy;let mut config=crate::pet_config::defaults();
  for enabled in [true,false]{config.pets.get_mut("integration_spotify").unwrap().system_access=Some(Policy{schema_version:1,enabled,revision:"a".repeat(32),grants:vec![]});assert!(own_route(&config,Pet::Spotify));}
  let mut settings=crate::settings::Settings::default();settings.main_pill="ai_ollama".into();settings.active_integrations=vec!["integration_spotify".into()];assert!(active(&settings,Pet::Ollama)&&active(&settings,Pet::Spotify));assert!(!active(&settings,Pet::Codex));
 }
}
