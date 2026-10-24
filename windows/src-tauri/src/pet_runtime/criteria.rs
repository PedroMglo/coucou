//! Typed reviewed predicates, never executable verification code or authority.
use super::model;
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Playback{Playing,Paused,Stopped}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Predicate{SpotifyPlayback{status:Playback},SpotifyVolume{percent:u8}}
impl Predicate{
    fn valid(&self)->bool{!matches!(self,Self::SpotifyVolume{percent}if *percent>100)}
    #[cfg(target_os="linux")]pub(super)fn matches(&self,s:&crate::spotify::goal::Snapshot)->bool{match self{
        Self::SpotifyPlayback{status}=>s.playback_status==match status{Playback::Playing=>"Playing",Playback::Paused=>"Paused",Playback::Stopped=>"Stopped"},
        Self::SpotifyVolume{percent}=>(s.volume-f64::from(*percent)/100.0).abs()<=0.000001,
    }}
}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Input{pub criterion_index:u32,pub predicate:Predicate}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Condition{pub criterion_index:u32,pub criterion_hash:String,pub predicate:Predicate}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Binding{pub schema_version:u32,pub spec_hash:String,pub criteria_count:u32,pub max_observations:u8,pub conditions:Vec<Condition>}
impl Binding{
    pub(super)fn valid(&self)->bool{
        let mut seen=std::collections::BTreeSet::new();self.schema_version==1&&self.max_observations==1&&super::authority::decode(&self.spec_hash).is_some()
            &&(1..=32).contains(&self.criteria_count)&&!self.conditions.is_empty()&&self.conditions.len()<=self.criteria_count as usize
            &&self.conditions.iter().all(|c|c.criterion_index<self.criteria_count&&seen.insert(c.criterion_index)&&super::authority::decode(&c.criterion_hash).is_some()&&c.predicate.valid())
    }
    pub(super)fn matches(&self,g:&model::Record)->bool{self.valid()&&self.spec_hash==model::fingerprint(&g.spec).unwrap_or_default()&&self.criteria_count as usize==g.spec.criteria.len()
        &&self.conditions.iter().all(|c|g.spec.criteria.get(c.criterion_index as usize).is_some_and(|s|model::fingerprint(s).ok().as_ref()==Some(&c.criterion_hash)))}
    pub(super)fn bind(g:&model::Record,input:&[Input])->model::Result<Self>{
        let conditions=input.iter().map(|c|Ok(Condition{criterion_index:c.criterion_index,criterion_hash:model::fingerprint(g.spec.criteria.get(c.criterion_index as usize).ok_or(model::Error::InvalidInput)?)?,predicate:c.predicate.clone()})).collect::<model::Result<Vec<_>>>()?;
        let b=Self{schema_version:1,spec_hash:model::fingerprint(&g.spec)?,criteria_count:g.spec.criteria.len()as u32,max_observations:1,conditions};if !b.matches(g){return Err(model::Error::InvalidInput);}Ok(b)
    }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CompletionRef{
    pub schema_version:u32,pub goal_id:String,pub pet_id:String,pub recipe_id:String,pub decision_id:String,pub lease_id:String,pub plan_id:String,
    pub source_revision:String,pub target_revision:String,pub report_hash:String,pub observed_at:u64,
}
impl CompletionRef{
    pub(super)fn valid(&self)->bool{self.schema_version==1&&model::goal_id(&self.goal_id)&&model::key(&self.pet_id)&&model::token(&self.recipe_id)&&model::token(&self.decision_id)&&model::token(&self.lease_id)&&model::key(&self.plan_id)
        &&model::token(&self.source_revision)&&model::token(&self.target_revision)&&self.source_revision!=self.target_revision&&super::authority::decode(&self.report_hash).is_some()&&self.observed_at>0&&self.observed_at<=8_640_000_000_000_000}
}
#[cfg(target_os="linux")]
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Verification{
    pub schema_version:u32,pub reference:CompletionRef,pub recipe_hash:String,pub spec_hash:String,
    pub source_sequence:u64,pub source_updated_at:u64,pub source_record_hash:String,
    pub snapshot:Option<crate::spotify::goal::Snapshot>,pub scope_current:bool,pub recorded_at:u64,
    pub results:Vec<CriterionResult>,pub criteria_verified:bool,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Status{Met,Unmet,Unknown}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CriterionResult{pub criterion_index:u32,pub criterion_hash:String,pub status:Status}
#[cfg(target_os="linux")]impl Verification{
    pub(super)fn content_hash(&self)->model::Result<String>{let mut body=self.clone();body.reference.report_hash.clear();model::fingerprint(&body)}
    pub(super)fn valid_for(&self,scope:&super::recipes::Scope)->bool{
        let Some(binding)=scope.verification.as_ref()else{return false;};let c=&self.reference;
        if self.schema_version!=1||c.schema_version!=1||c.goal_id!=scope.goal_id||c.pet_id!=scope.pet_id||c.recipe_id!=scope.id||c.decision_id!=scope.decision_id||c.lease_id!=scope.lease_id||c.plan_id!=scope.plan_id
            ||!model::token(&c.source_revision)||!model::token(&c.target_revision)||c.source_revision==c.target_revision||self.source_sequence==0||self.source_updated_at==0||self.source_updated_at>self.recorded_at
            ||self.recipe_hash!=model::fingerprint(scope).unwrap_or_default()||self.spec_hash!=binding.spec_hash||super::authority::decode(&self.source_record_hash).is_none()
            ||self.recorded_at<self.source_updated_at||self.recorded_at>8_640_000_000_000_000||self.results.len()!=binding.criteria_count as usize||self.content_hash().ok().as_ref()!=Some(&c.report_hash){return false;}
        if let Some(s)=&self.snapshot{if s.observed_at!=c.observed_at||s.observed_at<scope.approved_at||s.observed_at>self.recorded_at||!matches!(s.playback_status.as_str(),"Playing"|"Paused"|"Stopped")||!s.volume.is_finite()||!(0.0..=1.0).contains(&s.volume){return false;}}
        else if c.observed_at!=0{return false;}
        for(i,r)in self.results.iter().enumerate(){
            if r.criterion_index as usize!=i||super::authority::decode(&r.criterion_hash).is_none(){return false;}
            let condition=binding.conditions.iter().find(|v|v.criterion_index==r.criterion_index);
            if condition.is_some_and(|v|v.criterion_hash!=r.criterion_hash){return false;}
            let status=match(condition,self.snapshot.as_ref(),self.scope_current){(Some(c),Some(s),true)=>if c.predicate.matches(s){Status::Met}else{Status::Unmet},_=>Status::Unknown};
            if status!=r.status{return false;}
        }
        let all=self.scope_current&&self.snapshot.is_some()&&self.results.iter().all(|r|r.status==Status::Met)&&binding.conditions.len()==binding.criteria_count as usize;
        self.criteria_verified==all&&(!all||(c.valid()&&c.observed_at<scope.deadline_at&&self.recorded_at<scope.deadline_at))
    }
}
