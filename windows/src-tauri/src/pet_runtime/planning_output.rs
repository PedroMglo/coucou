//! Immutable signed planning observations. An output reference is data, never
//! an approval, canonical-plan acknowledgment or execution capability.
use super::{admission::AdmissionSpec,authority::{Authority,ConsentProof,Failure,Result},criteria,model,planning,planning_journal::Source,store::{Dir,nonce,protocol_directory}};
use serde::{Deserialize,Serialize};

const PROTOCOL:&[u8]=b"{\"version\":1,\"kind\":\"coucou-native-planning-output\"}\n";
const MAX_BYTES:usize=128*1024;
pub(super)use super::planning_journal::{OutputKind as Kind,OutputRef as Reference};
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Candidate {
    pub task_id:String,pub plan_id:String,pub target_revision:String,pub target_sequence:u64,
    pub description:String,pub steps:Vec<model::StepInput>,pub verification:criteria::Binding,
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub(super) enum Outcome {Plan {candidate:Candidate},Blocked {reason:String},Invalid}
impl Outcome {pub(super)fn kind(&self)->Kind {match self{Self::Plan{..}=>Kind::Plan,Self::Blocked{..}=>Kind::Blocked,Self::Invalid=>Kind::Invalid}}}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Body {
    pub schema_version:u32,pub goal_id:String,pub decision_id:String,pub lease_id:String,pub controller_id:String,
    pub round_index:u8,pub request_id:String,pub attempt_id:String,pub input_hash:String,pub source:Source,
    pub model_result_hash:String,pub completed_at:u64,pub recorded_at:u64,pub outcome:Outcome,
}
impl Body {
    pub(super)fn reference(&self)->Result<Reference> {Ok(Reference {schema_version:1,attempt_id:self.attempt_id.clone(),body_hash:model::fingerprint(self).map_err(|_|Failure::Invalid)?,kind:self.outcome.kind(),recorded_at:self.recorded_at})}
    pub(super)fn valid(&self)->bool {
        self.schema_version==1&&model::goal_id(&self.goal_id)&&model::token(&self.decision_id)&&model::token(&self.lease_id)&&model::token(&self.controller_id)
            && (1..=8).contains(&self.round_index)&&model::key(&self.request_id)&&model::token(&self.attempt_id)&&self.source.valid()
            && self.source.lease_id==self.lease_id&&self.source.controller_id==self.controller_id
            && [&self.input_hash,&self.model_result_hash].iter().all(|s|super::authority::decode(s).is_some())
            && self.completed_at>=self.source.updated_at&&self.recorded_at>=self.completed_at&&self.recorded_at<=8_640_000_000_000_000
            && match &self.outcome {
                Outcome::Invalid=>true,
                Outcome::Blocked{reason}=>!reason.trim().is_empty()&&reason.len()<=4096,
                Outcome::Plan{candidate:c}=>c.task_id==format!("planning-task-{}",self.attempt_id)&&c.plan_id==format!("planning-plan-{}",self.attempt_id)
                    && c.target_revision==model::hash(format!("coucou-planning-target-v1\0{}\0{}\0{}",self.decision_id,self.source.record_hash,self.attempt_id).as_bytes())[..32]
                    && c.target_sequence==self.source.sequence.saturating_add(1)&&model::token(&c.target_revision)&&c.target_revision!=self.source.revision
                    && !c.description.trim().is_empty()&&c.description.len()<=2048&&!c.steps.is_empty()&&c.steps.len()<=8&&c.verification.valid(),
            }
    }
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Envelope {pub body:Body,pub tag:String}
pub(super)fn message(body:&Body)->Result<Vec<u8>> {let mut bytes=b"coucou-native-planning-output-v1\0".to_vec();bytes.extend(serde_json::to_vec(body).map_err(|_|Failure::Invalid)?);Ok(bytes)}
fn directory(authority:&Authority,spec:&AdmissionSpec,create:bool)->Result<Option<Dir>> {
    if !model::token(&spec.decision_id)||!spec.binding_format(){return Err(Failure::Invalid);}
    let root=protocol_directory(&authority.planning_root()?,PROTOCOL,create).map_err(|_|Failure::Unsafe)?;
    let Some(root)=root else {return Ok(None);};root.check().map_err(|_|Failure::Changed)?;
    protocol_directory(&root.path.join(&spec.decision_id),PROTOCOL,create).map_err(|_|Failure::Unsafe)
}
pub(super)fn read(authority:&Authority,spec:&AdmissionSpec,proof:&ConsentProof,attempt:&str)->Result<Option<Envelope>> {
    if !model::token(attempt){return Err(Failure::Invalid);}
    let Some(dir)=directory(authority,spec,false)? else {return Ok(None);};
    let Some((bytes,_))=dir.bytes(&format!("{attempt}.json"),MAX_BYTES).map_err(|_|Failure::Unsafe)? else {return Ok(None);};
    let e:Envelope=serde_json::from_slice(&bytes).map_err(|_|Failure::Invalid)?;
    if e.body.attempt_id!=attempt{return Err(Failure::Invalid);}
    authority.verify_planning_output(spec,proof,&e)?;dir.check().map_err(|_|Failure::Changed)?;Ok(Some(e))
}
pub(super)fn publish(authority:&Authority,spec:&AdmissionSpec,proof:&ConsentProof,e:&Envelope)->Result<()> {
    authority.verify_planning_output(spec,proof,e)?;
    let bytes=serde_json::to_vec(e).map_err(|_|Failure::Invalid)?;if bytes.len()>MAX_BYTES{return Err(Failure::Capacity);}
    let dir=directory(authority,spec,true)?.ok_or(Failure::Unavailable)?;let name=format!("{}.json",e.body.attempt_id);
    if let Some((old,_))=dir.bytes(&name,MAX_BYTES).map_err(|_|Failure::Unsafe)? {if old!=bytes{return Err(Failure::RecoveryRequired);}}
    else {
        let temp=format!(".planning-output-{}",nonce().map_err(|_|Failure::Unavailable)?);dir.write_new(&temp,&bytes).map_err(|_|Failure::Unavailable)?;
        #[cfg(test)]if matches!(authority.fault,super::authority_store::Fault::BeforePlanningOutputPublish){return Err(Failure::Unavailable);}
        authority.verify_planning_output(spec,proof,e)?;dir.rename(&temp,&dir,&name,true).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(authority.fault,super::authority_store::Fault::AfterPlanningOutputPublish){return Err(Failure::Uncertain);}
    }
    dir.open_file(&name,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;dir.durable().map_err(|_|Failure::Uncertain)?;
    if dir.bytes(&name,MAX_BYTES).map_err(|_|Failure::Unsafe)?.is_none_or(|(b,_)|b!=bytes){return Err(Failure::Changed);}
    authority.verify_planning_output(spec,proof,e)
}
/// Retry only receipt/artifact/ledger publication. No inference, recapture or
/// new measurement is performed when any earlier acknowledgment was uncertain.
pub(super)fn reconcile(authority:&Authority,spec:&AdmissionSpec,proof:&ConsentProof,lease:&super::leases::Lease,goal:&model::Record,request:&str,now:u64)->Result<Envelope> {
    let (_,attempt)=authority.result_attempt(spec,proof,request)?;let attempt=attempt.ok_or(Failure::Missing)?;
    let e=match read(authority,spec,proof,&attempt.id)? {
        Some(e)=>e,
        None=>authority.seal_planning_output(spec,proof,lease,goal,request,now)?,
    };
    authority.ack_planning_output(spec,proof,lease,&e,now)?;Ok(e)
}

pub(super)fn candidate(plan:planning::ValidatedPlan,body:&Body)->Candidate {
    Candidate {task_id:format!("planning-task-{}",body.attempt_id),plan_id:format!("planning-plan-{}",body.attempt_id),
        target_revision:model::hash(format!("coucou-planning-target-v1\0{}\0{}\0{}",body.decision_id,body.source.record_hash,body.attempt_id).as_bytes())[..32].into(),
        target_sequence:body.source.sequence+1,description:plan.description,steps:plan.steps,verification:plan.verification}
}
