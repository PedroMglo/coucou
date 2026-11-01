//! Original-authority inspection/revocation of a present unused canonical
//! grant. No AdmissionSpec, old approval or controller is reconstructed.
use super::*;
use crate::pet_runtime::{retirement::Reference,retirement_controller::{self,Approved},runtime_owner::Profile,history::Charge};
use std::collections::BTreeMap;
struct Catalogue{canonical:BTreeMap<String,Grant>,staged:BTreeMap<String,(Grant,Stamp,String)>,stamps:BTreeMap<String,Stamp>}
impl Catalogue{
    fn hash(&self)->Result<String>{let staged=self.staged.iter().map(|(n,(_,_,h))|(n,h)).collect::<Vec<_>>();model::fingerprint(&(&self.canonical,staged)).map_err(|_|Failure::Invalid)}
}
/// Captured by authenticated native inspection only. Serialized audit data
/// cannot create this object or substitute the original Authority.
pub(in crate::pet_runtime)struct UnusedSource{body:Grant,profile:Profile,inventory_hash:String,artifact_fence:String,stamps:BTreeMap<String,Stamp>,charge:Charge,linked_accounting_complete:bool,unlinked:usize,staged:usize}
impl UnusedSource{
    pub(in crate::pet_runtime)fn body(&self)->&Grant{&self.body}
    pub(in crate::pet_runtime)fn charge(&self)->&Charge{&self.charge}
    pub(in crate::pet_runtime)fn linked_accounting_complete(&self)->bool{self.linked_accounting_complete}
    pub(in crate::pet_runtime)fn unlinked(&self)->usize{self.unlinked}
    pub(in crate::pet_runtime)fn staged(&self)->usize{self.staged}
    pub(in crate::pet_runtime)fn reference(&self,goal:&Record,request:&str,id:&str,prepared:u64,approved:u64)->model::Result<Reference>{
        let reference=Reference{schema_version:1,id:id.into(),request_id:request.into(),goal_id:goal.id.clone(),pet_id:goal.spec.owner.pet_id.clone(),decision_id:self.body.decision_id.clone(),source_goal_revision:goal.revision.clone(),source_goal_hash:model::fingerprint(goal)?,source_sequence:goal.sequence,source_grant_revision:self.body.revision.clone(),source_grant_hash:model::fingerprint(&self.body)?,source_grant_schema:self.body.schema_version,source_grant_updated_at:self.body.updated_at,inventory_hash:self.inventory_hash.clone(),artifact_fence:self.artifact_fence.clone(),profile:self.profile.clone(),prepared_at:prepared,approved_at:approved,history_continuity_verified:false};
        if !reference.source_matches(goal){return Err(model::Error::Conflict);}Ok(reference)
    }
}
fn same_unused(image:&Grant,canonical:&Grant)->bool{
    image.goal_id==canonical.goal_id&&image.decision_id==canonical.decision_id&&image.spec_hash==canonical.spec_hash&&image.approved_at==canonical.approved_at&&image.operation==canonical.operation
        &&image.phase==GrantPhase::Issued&&image.retirement.is_none()&&image.never_claimed_zero()
}
impl Authority{
    pub(in crate::pet_runtime)fn matches_root(&self,path:&Path)->Result<bool>{
        self.fence()?;let Some(other)=Dir::open(path,true).map_err(|_|Failure::Unsafe)?else{return Ok(false);};
        let a=self.dir.file.metadata().map_err(|_|Failure::Unavailable)?;let b=other.file.metadata().map_err(|_|Failure::Unavailable)?;other.check().map_err(|_|Failure::Changed)?;
        Ok(a.dev()==b.dev()&&a.ino()==b.ino())
    }
    fn retirement_catalogue(&self,goal:&str,own:Option<&Reference>)->Result<Catalogue>{
        let mut catalogue=Catalogue{canonical:BTreeMap::new(),staged:BTreeMap::new(),stamps:BTreeMap::new()};
        for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
            if self.historical_file(&name){continue;}
            if matches!(name.as_str(),".key.json"|".catalog.lock"|".protocol.json"){continue;}
            let canonical=name.strip_suffix(".json").filter(|s|model::token(s));let staged=name.strip_prefix(".grant-").is_some_and(model::token);
            if canonical.is_none()&&!staged{return Err(Failure::RecoveryRequired);}let(body,stamp)=self.grant_file(&name)?;
            if canonical.is_some_and(|id|id!=body.decision_id){return Err(Failure::Invalid);}if body.goal_id!=goal{continue;}
            if staged&&own.is_some_and(|r|body.retirement.as_ref()==Some(r)){continue;}
            catalogue.stamps.insert(name.clone(),stamp.clone());
            if canonical.is_some(){catalogue.canonical.insert(body.decision_id.clone(),body);}else{let hash=model::fingerprint(&body).map_err(|_|Failure::Invalid)?;catalogue.staged.insert(name,(body,stamp,hash));}
        }self.fence()?;Ok(catalogue)
    }
    fn retirement_artifact_fence(&self,decision:&str)->Result<String>{
        let models=crate::pet_runtime::results::unclaimed_directory_fence(self,decision)?;let planning=crate::pet_runtime::planning_output::unclaimed_directory_fence(self,decision)?;
        let mut facts=vec![];let mut total=0usize;let mut step_root=None;
        if let Some(dir)=self.step_directory(false)?{
            let metadata=dir.file.metadata().map_err(|_|Failure::Unavailable)?;step_root=Some((metadata.dev(),metadata.ino()));
            let mut names=dir.names().map_err(|_|Failure::Unavailable)?;names.sort();
            for name in names{
                if matches!(name.as_str(),".protocol.json"|".catalog.lock"){continue;}
                let(raw,stamp)=dir.bytes(&name,256*1024).map_err(|_|Failure::Unsafe)?.ok_or(Failure::Changed)?;total=total.checked_add(raw.len()).ok_or(Failure::Capacity)?;if total>64*1024*1024{return Err(Failure::Capacity);}
                if self.retirement_artifact_decision(&name,&raw)?==decision{return Err(Failure::RecoveryRequired);}facts.push((name,model::hash(&raw),stamp));
            }dir.check().map_err(|_|Failure::Changed)?;
        }self.fence()?;model::fingerprint(&(models,planning,step_root,facts)).map_err(|_|Failure::Invalid)
    }
    pub(in crate::pet_runtime::authority_store)fn retirement_profile(&self,reference:&Reference)->Result<()>{
        let(_,profile)=self.owner_context()?;if profile!=reference.profile{return Err(Failure::Changed);}Ok(())
    }
    fn retirement_seed(&self,goal:&Record,decision:&str,now:u64)->Result<UnusedSource>{
        let _guard=self.lock_shared()?;if !model::token(decision)||now<goal.updated_at{return Err(Failure::Invalid);}
        let history=self.history(goal,now)?;if !history.scan_complete||!history.linked_source_consistent{return Err(Failure::RecoveryRequired);}
        let catalogue=self.retirement_catalogue(&goal.id,None)?;let body=catalogue.canonical.get(decision).ok_or(Failure::Missing)?;
        if goal.admissions.iter().any(|r|r.spec.decision_id==decision)||body.phase!=GrantPhase::Issued||body.retirement.is_some()||!body.never_claimed_zero()||body.updated_at>now{return Err(Failure::RecoveryRequired);}
        let mut unlinked=0;
        for(id,grant)in &catalogue.canonical{if !goal.admissions.iter().any(|r|&r.spec.decision_id==id){unlinked+=1;if !grant.never_claimed_zero()||!matches!(grant.phase,GrantPhase::Issued|GrantPhase::Revoked){return Err(Failure::RecoveryRequired);}}}
        if catalogue.staged.values().any(|(g,_,_)|!same_unused(g,body)||g.updated_at>now){return Err(Failure::RecoveryRequired);}
        let(_,profile)=self.owner_context()?;let source=UnusedSource{body:body.clone(),profile,inventory_hash:catalogue.hash()?,artifact_fence:self.retirement_artifact_fence(decision)?,stamps:catalogue.stamps,charge:history.authenticated_subtotal,linked_accounting_complete:history.admissions.iter().all(|a|a.charge.is_some()),unlinked,staged:catalogue.staged.len()};
        self.fence()?;Ok(source)
    }
    pub(in crate::pet_runtime)fn prepare_retirement(self,goal:Record,decision:&str,request:String,now:u64)->Result<retirement_controller::Prepared>{
        {
            let _guard=self.lock_shared()?;let(body,_)=self.grant_file(&format!("{decision}.json"))?;
            if body.goal_id!=goal.id||body.updated_at>now||goal.admissions.iter().any(|r|r.spec.decision_id==decision){return Err(Failure::Changed);}
            if let Some(reference)=body.retirement.clone(){self.retirement_profile(&reference)?;drop(_guard);self.acknowledge_retirement(&goal,&reference)?;return Ok(retirement_controller::Prepared::Retired(crate::pet_runtime::retirement::Receipt{status:model::CommitStatus::Unchanged,record:goal,reference,execution_available:false,new_admission_created:false}));}
        }
        let source=self.retirement_seed(&goal,decision,now)?;
        let review=retirement_controller::Review::captured(goal,self,source,request,now).map_err(|_|Failure::Invalid)?;Ok(retirement_controller::Prepared::Review(review))
    }
    fn retirement_current_unlocked(&self,goal:&Record,source:&UnusedSource,reference:&Reference)->Result<()>{
        if !reference.source_matches(goal)||reference.inventory_hash!=source.inventory_hash||reference.artifact_fence!=source.artifact_fence||reference.source_grant_hash!=model::fingerprint(&source.body).map_err(|_|Failure::Invalid)?{return Err(Failure::Changed);}
        self.retirement_profile(reference)?;let current=self.retirement_catalogue(&goal.id,Some(reference))?;
        if current.hash()!=Ok(reference.inventory_hash.clone())||current.stamps!=source.stamps||self.retirement_artifact_fence(&reference.decision_id)?!=reference.artifact_fence{return Err(Failure::Changed);}self.fence()
    }
    pub(in crate::pet_runtime)fn retirement_current(&self,goal:&Record,source:&UnusedSource,reference:&Reference)->Result<()>{let _guard=self.lock_shared()?;self.retirement_current_unlocked(goal,source,reference)}
    fn retirement_cleanup(&self,canonical:&Grant)->Result<()>{
        let reference=canonical.retirement.as_ref().ok_or(Failure::Missing)?;
        for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
            if self.historical_file(&name){continue;}
            if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}let(image,stamp)=self.grant_file(&name)?;
            if image.goal_id!=canonical.goal_id||image.decision_id!=canonical.decision_id{continue;}
            if image.retirement.as_ref()!=Some(reference)&&!same_unused(&image,canonical){return Err(Failure::Changed);}
            self.fence()?;self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;
        }self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()
    }
    fn retirement_recorded(&self,goal:&Record,reference:&Reference)->Result<Grant>{
        let(body,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;
        if body.retirement.as_ref()!=Some(reference)||reference.goal_id!=goal.id||reference.pet_id!=goal.spec.owner.pet_id||goal.admissions.iter().any(|r|r.spec.decision_id==reference.decision_id){return Err(Failure::Changed);}
        self.retirement_profile(reference)?;Ok(body)
    }
    pub(in crate::pet_runtime)fn retire_unused(&self,goal:&Record,source:&UnusedSource,reference:&Reference,started:&mut bool)->Result<bool>{
        let _guard=self.lock()?;crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;
        let(current,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;
        if current.retirement.as_ref()==Some(reference){let body=self.retirement_recorded(goal,reference)?;self.stabilize(&body)?;self.retirement_cleanup(&body)?;return Ok(false);}
        self.retirement_current_unlocked(goal,source,reference)?;let mut body=source.body.clone();body.retirement=Some(reference.clone());body.schema_version=8;body.phase=GrantPhase::Revoked;body.revision=reference.id.clone();body.updated_at=reference.approved_at;
        if !body.valid(){return Err(Failure::Invalid);}*started=true;self.publish(body.clone(),false)?;self.retirement_cleanup(&body)?;Ok(true)
    }
    pub(in crate::pet_runtime)fn acknowledge_retirement(&self,goal:&Record,reference:&Reference)->Result<()>{
        let _guard=self.lock()?;crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;let body=self.retirement_recorded(goal,reference)?;self.stabilize(&body)?;self.retirement_cleanup(&body)
    }
    pub(in crate::pet_runtime)fn withdraw_retirement(&self,goal:&Record,source:&UnusedSource,reference:&Reference)->Result<bool>{
        let _guard=self.lock()?;crate::pet_data::bootstrap::write_allowed().map_err(|_|Failure::Unavailable)?;let(body,_)=self.grant_file(&format!("{}.json",reference.decision_id))?;
        if body.retirement.as_ref()==Some(reference){return Ok(false);}if goal.id!=reference.goal_id||goal.spec.owner.pet_id!=reference.pet_id||model::fingerprint(&body).ok().as_ref()!=Some(&reference.source_grant_hash)||model::fingerprint(&source.body).ok().as_ref()!=Some(&reference.source_grant_hash){return Err(Failure::Changed);}
        self.retirement_profile(reference)?;
        for name in self.dir.names().map_err(|_|Failure::Unavailable)?{
            if self.historical_file(&name){continue;}if !name.strip_prefix(".grant-").is_some_and(model::token){continue;}let(image,stamp)=self.grant_file(&name)?;if image.retirement.as_ref()==Some(reference){self.dir.remove_exact(&name,&stamp).map_err(|_|Failure::Uncertain)?;}}
        self.dir.durable().map_err(|_|Failure::Uncertain)?;self.fence()?;Ok(true)
    }
}
