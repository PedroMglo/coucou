//! Typed reviewed predicates, never executable verification code or authority.
use super::model;
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Playback{Playing,Paused,Stopped}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Predicate{SpotifyPlayback{status:Playback},SpotifyVolume{percent:u8},SystemSoundVolume{percent:u8},SystemSoundMute{muted:bool},SystemHealthObserved,SystemHealthBaselineMeasured,SystemHostInventoryObserved,PetMessagesPersisted}
impl Predicate{
    fn valid(&self)->bool{!matches!(self,Self::SpotifyVolume{percent}|Self::SystemSoundVolume{percent}if *percent>100)}
    #[cfg(target_os="linux")]pub(super)fn matches(&self,s:&crate::spotify::goal::Snapshot)->bool{match self{
        Self::SpotifyPlayback{status}=>s.playback_status==match status{Playback::Playing=>"Playing",Playback::Paused=>"Paused",Playback::Stopped=>"Stopped"},
        Self::SpotifyVolume{percent}=>(s.volume-f64::from(*percent)/100.0).abs()<=0.000001,
        Self::SystemHostInventoryObserved|Self::SystemHealthObserved|Self::SystemHealthBaselineMeasured|Self::PetMessagesPersisted|Self::SystemSoundVolume{..}|Self::SystemSoundMute{..}=>false,
    }}
}
#[cfg(target_os="linux")]impl Predicate{
 pub(super)fn target_status(&self,s:&super::physical_target::Snapshot)->Status{
  let matched=match s{
   super::physical_target::Snapshot::Inventory(s)=>if matches!(self,Self::SystemHostInventoryObserved)&&s.complete(){Some(true)}else{None},
   super::physical_target::Snapshot::HealthBaseline(s)=>if matches!(self,Self::SystemHealthBaselineMeasured)&&s.measured(){Some(true)}else{None},
   super::physical_target::Snapshot::Health(s)=>if matches!(self,Self::SystemHealthObserved)&&s.complete(){Some(true)}else{None},
   super::physical_target::Snapshot::Spotify(s)=>if matches!(self,Self::SpotifyPlayback{..}|Self::SpotifyVolume{..}){Some(self.matches(s))}else{None},
   super::physical_target::Snapshot::Sound(s)=>match self{Self::SystemSoundVolume{percent}=>s.volume_percent.map(|v|(v-f64::from(*percent)).abs()<=0.5),Self::SystemSoundMute{muted}=>s.muted.map(|v|v==*muted),_=>None},
  };match matched{Some(true)=>Status::Met,Some(false)=>Status::Unmet,None=>Status::Unknown}
 }

}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Input{pub criterion_index:u32,pub predicate:Predicate}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Condition{pub criterion_index:u32,pub criterion_hash:String,pub predicate:Predicate}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Binding{pub schema_version:u32,pub spec_hash:String,pub criteria_count:u32,pub max_observations:u8,pub conditions:Vec<Condition>}
impl Binding{
    pub(super)fn valid(&self)->bool{
        let mut seen=std::collections::BTreeSet::new();matches!(self.schema_version,1|2|3|4|5|6|7)&&self.max_observations==1&&super::authority::decode(&self.spec_hash).is_some()
            &&(1..=32).contains(&self.criteria_count)&&(self.schema_version==3||!self.conditions.is_empty())&&self.conditions.len()<=self.criteria_count as usize
            &&self.conditions.iter().all(|c|c.criterion_index<self.criteria_count&&seen.insert(c.criterion_index)&&super::authority::decode(&c.criterion_hash).is_some()&&c.predicate.valid()&&match self.schema_version{1=>matches!(c.predicate,Predicate::SpotifyPlayback{..}|Predicate::SpotifyVolume{..}),2|3=>matches!(c.predicate,Predicate::PetMessagesPersisted),4|5=>matches!(c.predicate,Predicate::SystemSoundVolume{..}|Predicate::SystemSoundMute{..}),6|7=>matches!(c.predicate,Predicate::SystemHostInventoryObserved|Predicate::SystemHealthObserved|Predicate::SystemHealthBaselineMeasured),_=>false})
    }
    pub(super)fn peer_for(g:&model::Record)->model::Result<Self>{
        let conditions=g.spec.criteria.iter().enumerate().filter(|(_,s)|s.as_str()==NATIVE_MESSAGE_CRITERION).map(|(i,s)|Ok(Condition{criterion_index:i as u32,criterion_hash:model::fingerprint(s)?,predicate:Predicate::PetMessagesPersisted})).collect::<model::Result<Vec<_>>>()?;
        let b=Self{schema_version:3,spec_hash:model::fingerprint(&g.spec)?,criteria_count:g.spec.criteria.len()as u32,max_observations:1,conditions};if !b.matches(g){return Err(model::Error::InvalidInput);}Ok(b)
    }
    pub(super)fn native_for(g:&model::Record)->model::Result<Option<Self>>{
        let conditions=g.spec.criteria.iter().enumerate().filter(|(_,c)|c.as_str()==NATIVE_MESSAGE_CRITERION).map(|(i,c)|Ok(Condition{criterion_index:i as u32,criterion_hash:model::fingerprint(c)?,predicate:Predicate::PetMessagesPersisted})).collect::<model::Result<Vec<_>>>()?;
        if conditions.is_empty(){return Ok(None);}let b=Self{schema_version:2,spec_hash:model::fingerprint(&g.spec)?,criteria_count:g.spec.criteria.len()as u32,max_observations:1,conditions};if !b.matches(g){return Err(model::Error::InvalidInput);}Ok(Some(b))
    }
    pub(super)fn matches(&self,g:&model::Record)->bool{self.valid()&&self.spec_hash==model::fingerprint(&g.spec).unwrap_or_default()&&self.criteria_count as usize==g.spec.criteria.len()
        &&self.conditions.iter().all(|c|g.spec.criteria.get(c.criterion_index as usize).is_some_and(|s|model::fingerprint(s).ok().as_ref()==Some(&c.criterion_hash)&&(self.schema_version!=3||s==NATIVE_MESSAGE_CRITERION)))}
    pub(super)fn system_for(g:&model::Record,input:&[Input])->model::Result<Self>{let mut b=Self::bind(g,input)?;if !matches!(b.schema_version,4|6)||b.conditions.len()!=g.spec.criteria.len(){return Err(model::Error::InvalidInput);}b.schema_version=if b.schema_version==6{7}else{5};if !b.matches(g){return Err(model::Error::InvalidInput);}Ok(b)}
    pub(super)fn bind(g:&model::Record,input:&[Input])->model::Result<Self>{
        let conditions=input.iter().map(|c|Ok(Condition{criterion_index:c.criterion_index,criterion_hash:model::fingerprint(g.spec.criteria.get(c.criterion_index as usize).ok_or(model::Error::InvalidInput)?)?,predicate:c.predicate.clone()})).collect::<model::Result<Vec<_>>>()?;
        let b=Self{schema_version:if input.iter().any(|c|matches!(c.predicate,Predicate::SystemHostInventoryObserved|Predicate::SystemHealthObserved|Predicate::SystemHealthBaselineMeasured)){6}else if input.iter().any(|c|matches!(c.predicate,Predicate::SystemSoundVolume{..}|Predicate::SystemSoundMute{..})){4}else{1},spec_hash:model::fingerprint(&g.spec)?,criteria_count:g.spec.criteria.len()as u32,max_observations:1,conditions};if !b.matches(g){return Err(model::Error::InvalidInput);}Ok(b)
    }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
#[serde(tag="kind")]
pub enum CompletionSource{NativeMessages{#[serde(rename="verificationId")]verification_id:String}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CompletionRef{
    #[serde(default,skip_serializing_if="Option::is_none")]pub source:Option<CompletionSource>,
    pub schema_version:u32,pub goal_id:String,pub pet_id:String,#[serde(default,skip_serializing_if="String::is_empty")]pub recipe_id:String,pub decision_id:String,pub lease_id:String,#[serde(default,skip_serializing_if="String::is_empty")]pub plan_id:String,
    pub source_revision:String,pub target_revision:String,pub report_hash:String,pub observed_at:u64,
}
impl CompletionRef{
    pub(super)fn id(&self)->&str{match &self.source{Some(CompletionSource::NativeMessages{verification_id})=>verification_id,None=>&self.recipe_id}}
    pub(super)fn valid(&self)->bool{(match(self.schema_version,&self.source){(1,None)=>model::token(&self.recipe_id)&&model::key(&self.plan_id),(2,Some(CompletionSource::NativeMessages{verification_id}))=>self.recipe_id.is_empty()&&self.plan_id.is_empty()&&model::token(verification_id),_=>false})&&model::goal_id(&self.goal_id)&&model::key(&self.pet_id)&&model::token(&self.decision_id)&&model::token(&self.lease_id)
        &&model::token(&self.source_revision)&&model::token(&self.target_revision)&&self.source_revision!=self.target_revision&&super::authority::decode(&self.report_hash).is_some()&&self.observed_at>0&&self.observed_at<=8_640_000_000_000_000}
}
#[cfg(target_os="linux")]
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Verification{
    pub schema_version:u32,pub reference:CompletionRef,pub recipe_hash:String,pub spec_hash:String,
    pub source_sequence:u64,pub source_updated_at:u64,pub source_record_hash:String,
    pub snapshot:Option<super::physical_target::Snapshot>,pub scope_current:bool,pub recorded_at:u64,
    pub results:Vec<CriterionResult>,pub criteria_verified:bool,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub enum Status{Met,Unmet,Unknown}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CriterionResult{pub criterion_index:u32,pub criterion_hash:String,pub status:Status}
#[cfg(target_os="linux")]impl Verification{
    pub(super)fn content_hash(&self)->model::Result<String>{let mut body=self.clone();body.reference.report_hash.clear();model::fingerprint(&body)}
    pub(super)fn valid_for(&self,scope:&super::recipes::Scope)->bool{
        let Some(binding)=scope.verification.as_ref().filter(|b|matches!((b.schema_version,scope.schema_version),(1,1|2|3)|(4,4)|(5,5)|(7,6)))else{return false;};let c=&self.reference;
        if self.schema_version!=(if scope.schema_version==6{4}else if scope.schema_version==5{3}else if scope.schema_version==4{2}else{1})||c.schema_version!=1||c.goal_id!=scope.goal_id||c.pet_id!=scope.pet_id||c.recipe_id!=scope.id||c.decision_id!=scope.decision_id||c.lease_id!=scope.lease_id||c.plan_id!=scope.plan_id
            ||!model::token(&c.source_revision)||!model::token(&c.target_revision)||c.source_revision==c.target_revision||self.source_sequence==0||self.source_updated_at==0||self.source_updated_at>self.recorded_at
            ||self.recipe_hash!=model::fingerprint(scope).unwrap_or_default()||self.spec_hash!=binding.spec_hash||super::authority::decode(&self.source_record_hash).is_none()
            ||self.recorded_at<self.source_updated_at||self.recorded_at>8_640_000_000_000_000||self.results.len()!=binding.criteria_count as usize||self.content_hash().ok().as_ref()!=Some(&c.report_hash){return false;}
        if let Some(s)=&self.snapshot{if s.observed_at()!=c.observed_at||!s.valid_between(scope.approved_at,self.recorded_at)||scope.steps.first().is_none_or(|step|!s.matches_resource(&step.resource)){return false;}}
        else if c.observed_at!=0{return false;}
        for(i,r)in self.results.iter().enumerate(){
            if r.criterion_index as usize!=i||super::authority::decode(&r.criterion_hash).is_none(){return false;}
            let condition=binding.conditions.iter().find(|v|v.criterion_index==r.criterion_index);
            if condition.is_some_and(|v|v.criterion_hash!=r.criterion_hash){return false;}
            let status=match(condition,self.snapshot.as_ref(),self.scope_current){(Some(c),Some(s),true)=>c.predicate.target_status(s),_=>Status::Unknown};
            if status!=r.status{return false;}
        }
        let all=self.scope_current&&self.snapshot.is_some()&&self.results.iter().all(|r|r.status==Status::Met)&&binding.conditions.len()==binding.criteria_count as usize;
        self.criteria_verified==all&&(!all||(c.valid()&&c.observed_at<scope.deadline_at&&self.recorded_at<scope.deadline_at))
    }
}

/// The catalog gives this text one precise, deterministic meaning. Arbitrary
/// user prose is never mapped to a native predicate by fuzzy/model judgment.
pub(super)const NATIVE_MESSAGE_CRITERION:&str="A mensagem fica persistida para o destinatário indicado; não autoriza execução.";
#[cfg(target_os="linux")]
pub(super)trait Certificate{
 fn reference(&self)->&CompletionRef;
 fn current(&self,goal:&model::Record,now:u64)->super::authority::Result<()>;
}
/// Shared source/criteria fence for every completion evidence kind.
pub(super)fn verify_source(goal:&model::Record,reference:&CompletionRef,sequence:u64,updated_at:u64,record_hash:&str,spec_hash:&str,results:&[CriterionResult])->model::Result<()>{
 goal.validate()?;let original=if goal.state==model::State::Completed{if goal.completion.as_ref()!=Some(reference){return Err(model::Error::Conflict);}goal.source_before_completion(sequence,updated_at)?}else{goal.clone()};
 if original.state!=model::State::Leased||original.id!=reference.goal_id||original.spec.owner.pet_id!=reference.pet_id||original.revision!=reference.source_revision||original.sequence!=sequence||original.updated_at!=updated_at||model::fingerprint(&original)?!=record_hash||model::fingerprint(&original.spec)?!=spec_hash||original.spec.criteria.len()!=results.len()||original.spec.criteria.iter().zip(results).any(|(text,r)|model::fingerprint(text).ok().as_ref()!=Some(&r.criterion_hash)){return Err(model::Error::Conflict);}Ok(())
}
