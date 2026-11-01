//! Authenticated reduction of unused authority. Serialized references are audit
//! data; they cannot reconstruct a Review, consent, admission or live owner.
use super::{authority::{Grant,GrantPhase,Operation},model::{self,Record},runtime_owner::Profile};
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Reference{
    pub schema_version:u32,pub id:String,pub request_id:String,pub goal_id:String,pub pet_id:String,pub decision_id:String,
    pub source_goal_revision:String,pub source_goal_hash:String,pub source_sequence:u64,
    pub source_grant_revision:String,pub source_grant_hash:String,pub source_grant_schema:u32,pub source_grant_updated_at:u64,
    pub inventory_hash:String,pub artifact_fence:String,pub(super)profile:Profile,pub prepared_at:u64,pub approved_at:u64,
    pub history_continuity_verified:bool,
}
impl Reference{
    pub(super)fn basic(&self)->bool{
        let hash=|s:&str|super::authority::decode(s).is_some();
        self.schema_version==1&&model::token(&self.id)&&model::key(&self.request_id)&&model::goal_id(&self.goal_id)&&model::key(&self.pet_id)&&model::token(&self.decision_id)
            &&[&self.source_goal_revision,&self.source_grant_revision,&self.profile.key_id].iter().all(|s|model::token(s))
            &&[&self.source_goal_hash,&self.source_grant_hash,&self.inventory_hash,&self.artifact_fence].iter().all(|s|hash(s))
            &&self.source_sequence>0&&self.source_sequence<=model::MAX_EVENTS as u64&&matches!(self.source_grant_schema,1|4)&&self.profile.directory.inode>0
            &&self.source_grant_updated_at>0&&self.source_grant_updated_at<=self.prepared_at&&self.prepared_at<=self.approved_at&&self.approved_at<=8_640_000_000_000_000
            &&self.approved_at-self.prepared_at<30_000&&!self.history_continuity_verified
    }
    pub(super)fn source_matches(&self,goal:&Record)->bool{
        self.basic()&&goal.id==self.goal_id&&goal.spec.owner.pet_id==self.pet_id&&goal.revision==self.source_goal_revision&&goal.sequence==self.source_sequence
            &&model::fingerprint(goal).ok().as_ref()==Some(&self.source_goal_hash)&&!goal.admissions.iter().any(|row|row.spec.decision_id==self.decision_id)
    }
    pub(super)fn valid_for(&self,body:&Grant)->bool{
        if !self.basic()||body.schema_version!=8||body.phase!=GrantPhase::Revoked||!body.never_claimed_zero()||body.recovery.is_some()
            ||body.goal_id!=self.goal_id||body.decision_id!=self.decision_id||body.revision!=self.id||body.updated_at!=self.approved_at||body.approved_at>self.source_grant_updated_at{return false;}
        let mut before=body.clone();before.retirement=None;before.schema_version=self.source_grant_schema;before.phase=GrantPhase::Issued;before.revision=self.source_grant_revision.clone();before.updated_at=self.source_grant_updated_at;
        before.valid()&&before.schema_version==(if before.operation==Operation::GoalPlanning{4}else{1})
            &&model::fingerprint(&before).ok().as_ref()==Some(&self.source_grant_hash)
    }
}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Receipt{pub status:model::CommitStatus,pub record:Record,pub reference:Reference,pub execution_available:bool,pub new_admission_created:bool}
