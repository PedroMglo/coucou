//! Shared action DATA parser; no native IO or authority.
use super::{Request,Action as AudioAction};
use serde::Deserialize;
use serde_json::Value;
#[derive(Clone,Debug,PartialEq,Eq)]pub(crate)enum Action{Volume{target_key:String,percent:u8},Mute{target_key:String,muted:bool}}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Volume{target_key:String,percent:u8}
#[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Mute{target_key:String,muted:bool}
impl Action{
 pub(crate)fn parse(id:&str,version:u32,value:&Value)->Result<Self,String>{
  if version!=1{return Err("Versão de Som não suportada.".into());}
  let action=match id{
   "system.sound.volume"=>{let v:Volume=serde_json::from_value(value.clone()).map_err(|_|"Indica a saída exata e um volume entre0 e100%.".to_string())?;if v.percent>100{return Err("Volume fora dos limites.".into());}Self::Volume{target_key:v.target_key,percent:v.percent}},
   "system.sound.mute"=>{let v:Mute=serde_json::from_value(value.clone()).map_err(|_|"Indica a saída exata e o estado de silêncio.".to_string())?;Self::Mute{target_key:v.target_key,muted:v.muted}},
   _=>return Err("Operação de Som não suportada.".into()),
  };if !crate::pet_config::system_access::output_key(action.key()){return Err("A saída de Som não é válida.".into());}Ok(action)
 }
 pub(crate)fn key(&self)->&str{match self{Self::Volume{target_key,..}|Self::Mute{target_key,..}=>target_key}}
 pub(crate)fn operation(&self)->crate::pet_config::system_access::Operation{match self{Self::Volume{..}=>crate::pet_config::system_access::Operation::Volume,Self::Mute{..}=>crate::pet_config::system_access::Operation::Mute}}
 pub(crate)fn request(&self)->Request{match self{
  Self::Volume{target_key,percent}=>Request{target_key:target_key.clone(),action:AudioAction::Volume,value:Some(f64::from(*percent)),muted:None,destination_key:None,option:None},
  Self::Mute{target_key,muted}=>Request{target_key:target_key.clone(),action:AudioAction::Mute,value:None,muted:Some(*muted),destination_key:None,option:None},
 }}
}
