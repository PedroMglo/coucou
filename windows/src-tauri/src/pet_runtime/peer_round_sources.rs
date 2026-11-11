//! Bounded, historical exact metadata, sealed with the original round. Not an
//! IO permit, native approval, source text or a replacement authority ledger.
use super::{model,peer_round_data::{Reference,Consent,Fact}};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Dependency{pub schema_version:u32,pub reference:Reference,pub exact:serde_json::Value,#[serde(default,skip_serializing_if="Option::is_none")]pub history:Option<Vec<super::peer_round_history::Reference>>}
impl Dependency{
 pub(super)fn valid(&self)->bool{self.schema_version==(if self.history.is_some(){2}else{1})&&self.history.as_ref().is_none_or(|h|h.len()<=8&&h.iter().enumerate().all(|(i,r)|r.valid()&&usize::from(r.round)==i+1))&&self.history.as_ref().map_or_else(||self.exact.get("ownHistory").is_none(),|h|serde_json::to_value(h).ok().as_ref()==Some(&self.exact["ownHistory"]))&&self.reference.valid()&&model::fingerprint(&self.exact).ok().as_ref()==Some(&self.reference.exact_hash)&&serde_json::to_vec(&self.exact).is_ok_and(|v|v.len()<=256*1024)&&self.exact["observation"]["goalId"].as_str()==Some(&self.reference.goal_id)&&self.exact["observation"]["petId"].as_str()==Some(&self.reference.pet_id)&&self.exact["observation"]["decisionId"].as_str()==Some(&self.reference.decision_id)&&self.exact["observation"]["items"].as_array().is_some_and(|r|r.len()<=32)&&serde_json::from_value::<Vec<Fact>>(self.exact["facts"].clone()).is_ok_and(|r|r.len()<=32)}
 pub(super)fn for_consent(&self,c:&Consent)->bool{self.valid()&&self.reference==c.source&&self.exact["observation"]["parentGoalHash"].as_str()==Some(&c.goal_hash)}
}
