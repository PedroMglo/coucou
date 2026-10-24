//! One immutable recipe approval, sharing the step/key lock without granting
//! control from serialized data. Progress is a coherent native ledger read.
use super::*;
use crate::pet_runtime::{recipes::{Scope as RecipeScope,Progress,StepProgress,Phase as RecipePhase},recipes as recipe_model,lease_controller::recipe_controller::Approved};
#[path="authority_goal_completion.rs"]mod completion;
pub(in crate::pet_runtime)use completion::Certified as CompletionCertified;
const MAX_RECIPE:usize=256*1024;
#[derive(Serialize,Deserialize)]#[serde(deny_unknown_fields)]
struct Envelope{body:RecipeScope,tag:String}
fn message(scope:&RecipeScope)->Result<Vec<u8>>{let mut bytes=match scope.schema_version{1=>b"coucou-native-goal-recipe-v1\0".to_vec(),2=>b"coucou-native-goal-recipe-v2\0".to_vec(),_=>return Err(Failure::Invalid)};bytes.extend(serde_json::to_vec(scope).map_err(|_|Failure::Invalid)?);Ok(bytes)}
impl Authority{
    fn recipe_load(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str)->Result<Option<RecipeScope>>{
        let main=self.load(spec,proof)?;let lease=main.lease.as_ref().ok_or(Failure::Missing)?;
        let Some(dir)=self.step_directory(false)?else{return Ok(None);};let slot=recipe_model::slot(&spec.decision_id,plan).map_err(|_|Failure::Invalid)?;
        let Some((raw,_))=dir.bytes(&format!("{slot}.recipe.json"),MAX_RECIPE).map_err(|_|Failure::Unsafe)?else{return Ok(None);};
        let e:Envelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if !e.body.belongs(spec,lease)||e.body.plan_id!=plan{return Err(Failure::Invalid);}
        hmac::verify(&self.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;self.fence()?;Ok(Some(e.body))
    }
    fn recipe_stabilize(&self,scope:&RecipeScope)->Result<()>{
        let dir=self.step_directory(false)?.ok_or(Failure::Missing)?;let name=format!("{}.recipe.json",scope.slot().map_err(|_|Failure::Invalid)?);
        dir.open_file(&name,libc::O_RDONLY).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Missing)?.sync_all().map_err(|_|Failure::Uncertain)?;dir.durable().map_err(|_|Failure::Uncertain)?;
        let(raw,_)=dir.bytes(&name,MAX_RECIPE).map_err(|_|Failure::Changed)?.ok_or(Failure::Changed)?;let e:Envelope=serde_json::from_slice(&raw).map_err(|_|Failure::Invalid)?;
        if e.body!=*scope{return Err(Failure::Changed);}hmac::verify(&self.key,&message(&e.body)?,&decode(&e.tag).ok_or(Failure::Invalid)?).map_err(|_|Failure::Invalid)?;self.fence()
    }
    pub(in crate::pet_runtime)fn recipe_headroom(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,calls:usize,output:u64,now:u64)->Result<()>{
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,true)?;Self::recipe_budget(&main,spec,calls,output)
    }
    fn recipe_budget(main:&Grant,spec:&AdmissionSpec,calls:usize,output:u64)->Result<()>{
        if spec.budget.max_tokens.is_some()||spec.budget.max_cost_micros.is_some(){return Err(Failure::Invalid);}
        let usage=main.budget_usage.as_ref();let used=usage.map(|u|u.attempts.iter().filter(|a|a.kind==crate::pet_runtime::effect_budget::Kind::Tool).count()).unwrap_or(0);
        let bytes=usage.map(|u|u.attempts.iter().map(Attempt::charge).sum::<u64>()).unwrap_or(0);
        if used.saturating_add(calls)>usize::from(spec.budget.max_tool_calls)||bytes.checked_add(output).is_none_or(|n|n>spec.budget.max_output_bytes){return Err(Failure::Capacity);}Ok(())
    }
    pub(in crate::pet_runtime)fn issue_recipe(&self,spec:&AdmissionSpec,proof:&ConsentProof,lease:&Lease,approved:&Approved,now:u64)->Result<RecipeScope>{
        let scope=approved.scope();if !scope.belongs(spec,lease)||now<scope.approved_at||now>=scope.deadline_at{return Err(Failure::Invalid);}
        let _lock=self.lock()?;let main=self.budget_body(spec,proof,lease,now,true)?;
        if let Some(old)=self.recipe_load(spec,proof,&scope.plan_id)?{if old!=*scope{return Err(Failure::RecoveryRequired);}self.recipe_stabilize(&old)?;return Ok(old);}
        Self::recipe_budget(&main,spec,scope.steps.len(),scope.steps.iter().map(|s|s.max_output_bytes).sum())?;
        for step in &scope.steps{if self.step_load(spec,proof,&step.plan_id,&step.step_id)?.is_some(){return Err(Failure::RecoveryRequired);}}
        let bytes=serde_json::to_vec(&Envelope{body:scope.clone(),tag:encode(hmac::sign(&self.key,&message(scope)?).as_ref())}).map_err(|_|Failure::Invalid)?;
        if bytes.len()>MAX_RECIPE{return Err(Failure::Capacity);}let dir=self.step_directory(true)?.ok_or(Failure::Unavailable)?;
        let temp=format!(".recipe-{}",nonce().map_err(|_|Failure::Unavailable)?);dir.write_new(&temp,&bytes).map_err(|_|Failure::Unavailable)?;self.fence()?;
        #[cfg(test)]if matches!(self.fault,Fault::BeforeRecipePublish){return Err(Failure::Unavailable);}
        dir.rename(&temp,&dir,&format!("{}.recipe.json",scope.slot().map_err(|_|Failure::Invalid)?),true).map_err(|_|Failure::Uncertain)?;
        #[cfg(test)]if matches!(self.fault,Fault::AfterRecipePublish){return Err(Failure::Uncertain);}
        self.recipe_stabilize(scope)?;Ok(scope.clone())
    }
    pub(in crate::pet_runtime)fn read_recipe(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str)->Result<Option<RecipeScope>>{
        let _lock=self.lock()?;self.recipe_load(spec,proof,plan)
    }
    pub(in crate::pet_runtime)fn recipe_progress(&self,spec:&AdmissionSpec,proof:&ConsentProof,plan:&str)->Result<Option<Progress>>{
        let _lock=self.lock()?;let Some(scope)=self.recipe_load(spec,proof,plan)?else{return Ok(None);};
        let mut steps=Vec::with_capacity(scope.steps.len());let mut verified=0;let mut unsafe_result=false;let mut prefix_open=false;
        for step in &scope.steps{
            let saved=self.step_assessment_unlocked(spec,proof,plan,&step.step_id)?;
            if let Some(saved)=saved{
                if saved.record.scope!=*step{return Err(Failure::Invalid);}
                if saved.record.phase==Phase::Finished&&saved.record.result.as_ref().is_some_and(|r|r.observation.effect_verified&&r.observation.scope_current){verified+=1;if prefix_open{unsafe_result=true;}}
                else{prefix_open=true;if saved.record.phase!=Phase::Approved{unsafe_result=true;}}
                steps.push(StepProgress{step_id:step.step_id.clone(),phase:Some(saved.record.phase),recovery:Some(saved.recovery),result:saved.record.result});
            }else{prefix_open=true;steps.push(StepProgress{step_id:step.step_id.clone(),phase:None,recovery:None,result:None});}
        }
        let phase=if unsafe_result{RecipePhase::RecoveryRequired}else if verified==scope.steps.len(){RecipePhase::Verified}else if verified>0{RecipePhase::Partial}else{RecipePhase::Reviewed};
        let verification=self.verification_load_unlocked(spec,proof,&scope)?;
        let criteria_verified=verification.as_ref().is_some_and(|v|v.criteria_verified);
        let progress=Progress{total_steps:scope.steps.len(),scope,phase,verified_steps:verified,steps,verification,criteria_verified,goal_complete:false,execution_available:false};
        if serde_json::to_vec(&progress).map_err(|_|Failure::Invalid)?.len()>2*1024*1024{return Err(Failure::Capacity);}self.fence()?;Ok(Some(progress))
    }
}
