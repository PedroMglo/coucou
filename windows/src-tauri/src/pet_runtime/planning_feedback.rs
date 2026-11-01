//! Bounded native final observations in the original planning journal. These
//! data never grant effects or replace the immutable recipe verification.
use super::{criteria::{CriterionResult,Status},model};
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Feedback {
    pub schema_version:u32,pub recipe_id:String,pub plan_id:String,pub report_hash:String,pub goal_spec_hash:String,
    pub observed_at:u64,pub recorded_at:u64,pub known:bool,pub criteria:Vec<CriterionResult>,
}
impl Feedback {
    #[cfg(target_os="linux")]
    pub(super)fn from_report(report:&super::criteria::Verification)->Self {
        Self {schema_version:1,recipe_id:report.reference.recipe_id.clone(),plan_id:report.reference.plan_id.clone(),report_hash:report.reference.report_hash.clone(),goal_spec_hash:report.spec_hash.clone(),
            observed_at:report.reference.observed_at,recorded_at:report.recorded_at,known:report.scope_current&&report.snapshot.is_some()&&report.results.iter().all(|r|r.status!=Status::Unknown),criteria:report.results.clone()}
    }
    pub(super)fn valid(&self)->bool {
        self.schema_version==1&&model::token(&self.recipe_id)&&model::key(&self.plan_id)
            &&[&self.report_hash,&self.goal_spec_hash].iter().all(|s|super::authority::decode(s).is_some())
            &&self.recorded_at>0&&self.recorded_at<=8_640_000_000_000_000&&self.observed_at<=self.recorded_at
            &&!self.criteria.is_empty()&&self.criteria.len()<=32&&self.criteria.iter().enumerate().all(|(i,c)|c.criterion_index as usize==i&&super::authority::decode(&c.criterion_hash).is_some())
            &&self.known==(self.observed_at>0&&self.criteria.iter().all(|c|c.status!=Status::Unknown))
    }
    pub(super)fn met(&self)->usize {self.criteria.iter().filter(|c|c.status==Status::Met).count()}
    pub(super)fn continuable(&self)->bool {self.valid()&&self.known&&self.met()<self.criteria.len()}
}
