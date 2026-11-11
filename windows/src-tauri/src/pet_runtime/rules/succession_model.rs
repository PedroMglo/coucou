//! A new native decision references an irrevocably revoked original allocation.
//! No copied reservations or second balance: Authority derives the whole chain.
use super::*;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RuleSuccession{
 pub schema_version:u32,pub root_decision_id:String,pub predecessor_decision_id:String,
 pub generation:u8,pub source_hash:String,pub predecessor_commitments_hash:String,
}
impl RuleSuccession{
 pub(crate)fn valid(&self,decision:&str)->bool{
  self.schema_version==1&&(1..64).contains(&self.generation)&&model::token(&self.root_decision_id)
   &&model::token(&self.predecessor_decision_id)&&decision!=self.root_decision_id&&decision!=self.predecessor_decision_id
   &&(self.generation!=1||self.predecessor_decision_id==self.root_decision_id)
   &&[&self.source_hash,&self.predecessor_commitments_hash].iter().all(|h|h.len()==64&&super::super::authority::decode(h).is_some())
 }
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RuleLineageObservation{
 pub root_decision_id:String,pub predecessor_decision_id:String,pub generation:u8,
 pub original_approved_at:u64,pub inherited_occurrences:usize,pub inherited_output_bytes:u64,
 #[serde(skip_serializing_if="Option::is_none")]pub allocation:Option<RuleAllocationObservation>,
}
impl RuleGrant{
 /// Stable across observation/ACK/retirement progress. Full signed source is
 /// separately captured/reviewed; this digest never hides committed counters.
 pub(crate)fn commitments_hash(&self)->model::Result<String>{
  let reservations=self.reservations.iter().map(|r|(&r.occurrence_id,r.slot_index,r.at,r.deadline_at,r.reserved_at,r.model_calls,r.tool_calls,r.output_bytes)).collect::<Vec<_>>();
  if self.revision_authority.is_some(){return model::fingerprint(&("coucou-rule-commitments-v2",&self.decision_id,&self.policy,self.approved_at,&self.revision_authority,&reservations));}
  model::fingerprint(&("coucou-rule-commitments-v1",&self.decision_id,&self.policy,self.approved_at,reservations))
 }
}
