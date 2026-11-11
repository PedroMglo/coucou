//! Receipt for DATA consolidation only; never a new grant or worker-end proof.
use super::*;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RuleIntentRetirement{
 pub schema_version:u32,pub native_decision_id:String,pub canonical_hash:String,
 pub inventory_hash:String,pub merged_ledger_hash:String,pub source_schema_version:u32,
 pub source_updated_at:u64,pub approved_at:u64,pub intents:Vec<RetiredRuleIntent>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RetiredRuleIntent{pub name:String,pub body_hash:String}
impl RuleGrant{
 pub(crate)fn valid_intent_retirements(&self)->bool{
  let mut ids=std::collections::BTreeSet::new();let mut previous=self.approved_at;
  self.intent_retirements.len()<=64&&self.intent_retirements.iter().map(|r|r.intents.len()).sum::<usize>()<=128&&(self.intent_retirements.is_empty()||self.phase==RulePhase::Revoked)&&self.intent_retirements.iter().all(|r|{
   let valid=r.schema_version==1&&model::token(&r.native_decision_id)&&ids.insert(&r.native_decision_id)
    &&[&r.canonical_hash,&r.inventory_hash,&r.merged_ledger_hash].iter().all(|h|h.len()==64&&super::super::authority::decode(h).is_some())
    &&(1..=7).contains(&r.source_schema_version)&&r.source_updated_at>=previous&&r.source_updated_at<=r.approved_at&&r.approved_at<=self.updated_at
    &&!r.intents.is_empty()&&r.intents.len()<=32&&r.intents.windows(2).all(|p|p[0].name<p[1].name)
    &&r.intents.iter().all(|i|i.name.strip_prefix(".grant-").is_some_and(model::token)&&i.body_hash.len()==64&&super::super::authority::decode(&i.body_hash).is_some())
    &&model::fingerprint(&r.intents).ok().as_ref()==Some(&r.inventory_hash);
   previous=r.approved_at;valid
  })
 }
}
