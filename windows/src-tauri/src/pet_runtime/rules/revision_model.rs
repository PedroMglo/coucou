//! Reviewed policy changes and explicitly NEW finite allocations stay in the
//! original Authority chain. Save/retry/continuation cannot create a budget.
use super::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum RuleRevisionMode{CarryRemaining,RenewFiniteAllocation}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct RuleRevisionAuthority{
 pub schema_version:u32,pub mode:RuleRevisionMode,pub root_decision_id:String,
 pub predecessor_decision_id:String,pub generation:u8,pub source_hash:String,
 pub predecessor_commitments_hash:String,pub future_after:u64,
}
impl RuleRevisionAuthority{
 pub(crate)fn valid(&self,g:&RuleGrant)->bool{
  self.schema_version==1&&(1..64).contains(&self.generation)&&model::token(&self.root_decision_id)
   &&model::token(&self.predecessor_decision_id)&&g.decision_id!=self.root_decision_id&&g.decision_id!=self.predecessor_decision_id
   &&(self.generation!=1||self.predecessor_decision_id==self.root_decision_id)&&self.future_after==g.approved_at
   &&[&self.source_hash,&self.predecessor_commitments_hash].iter().all(|h|h.len()==64&&super::super::authority::decode(h).is_some())
 }
}
/// Native-derived DATA; effect writers independently revalidate the same chain.
pub(crate)struct RuleExecutionBounds{pub prior:Vec<RuleReservation>,pub future_after:u64}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct RuleAllocationObservation{
 pub root_decision_id:String,pub approved_at:u64,pub valid_until:u64,pub max_occurrences:u16,
 pub committed_occurrences:usize,pub remaining_occurrences:u16,pub cumulative_committed_occurrences:usize,
 pub cumulative_committed_output_bytes:u64,pub future_after:u64,
}
impl RuleGrant{
 pub(crate)fn parent_id(&self)->Option<&str>{self.revision_authority.as_ref().map(|r|r.predecessor_decision_id.as_str()).or_else(||self.succession.as_ref().map(|s|s.predecessor_decision_id.as_str()))}
 pub(crate)fn starts_allocation(&self)->bool{self.revision_authority.as_ref().is_some_and(|r|r.mode==RuleRevisionMode::RenewFiniteAllocation)}
}
