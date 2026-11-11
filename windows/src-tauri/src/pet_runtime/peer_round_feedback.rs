//! Authenticated metadata observations, never completion predicates or consent.
use super::{model,peer_round_data::Reference,generated_plans};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Feedback{pub schema_version:u32,pub goal_spec_hash:String,pub plan:generated_plans::Reference,pub source:Reference,pub information_keys:Vec<String>,pub observed_at:u64}
impl Feedback{
 pub(super)fn valid(&self)->bool{self.schema_version==1&&super::authority::decode(&self.goal_spec_hash).is_some()&&self.plan.valid()&&self.source.valid()&&self.source.decision_id==self.plan.decision_id&&self.observed_at>0&&self.observed_at<self.source.expires_at&&self.information_keys.len()<=32&&self.information_keys.iter().all(|s|super::authority::decode(s).is_some())&&self.information_keys.windows(2).all(|w|w[0]<w[1])&&model::fingerprint(&self.information_keys).ok().as_ref()==Some(&self.source.information_hash)}
 pub(super)fn matches_data(&self,d:&super::peer_round_data::Consent)->bool{self.valid()&&self.source.same_observation(&d.source)&&self.observed_at<=d.approved_at}
}
