//! Bounded canonical own output data. Neither JSON nor this capture is consent.
use super::model;
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Reference{pub round:u8,pub request_id:String,pub attempt_id:String,pub input_hash:String,pub output_hash:String,pub receipt_hash:String,pub content_hash:String,pub file_hash:String,pub dependency_hash:Option<String>}
impl Reference{
 pub(super)fn valid(&self)->bool{(1..=8).contains(&self.round)&&model::key(&self.request_id)&&model::token(&self.attempt_id)&&[&self.input_hash,&self.output_hash,&self.receipt_hash,&self.content_hash,&self.file_hash].iter().all(|h|super::authority::decode(h).is_some())&&self.dependency_hash.as_ref().is_none_or(|h|super::authority::decode(h).is_some())}
}
/// Built only from the original native Authority. No Clone/Deserialize or text
/// constructor is exposed to callers, models or stored review DTOs.
pub(crate)struct Captured{references:Vec<Reference>,messages:Vec<serde_json::Value>}
impl Captured{
 pub(super)fn verified(references:Vec<Reference>,messages:Vec<serde_json::Value>)->model::Result<Self>{if references.len()>8||references.len()!=messages.len()||references.iter().enumerate().any(|(i,r)|!r.valid()||usize::from(r.round)!=i+1)||serde_json::to_vec(&messages).map_err(|_|model::Error::InvalidInput)?.len()>64*1024{return Err(model::Error::LimitReached);}Ok(Self{references,messages})}
 pub(super)fn references(&self)->&[Reference]{&self.references}
 pub(crate)fn messages(&self)->&[serde_json::Value]{&self.messages}
}
