//! Canonical, bounded ancestry. Textual PetIds/ancestor arrays are never proof.
//! Callers hold this same Authority guard; final effect writers do not re-flock.
use super::*;
use crate::pet_runtime::{delegation::{self,Payload,State as Branch},received_request::Source,messages::Envelope};
use std::collections::BTreeSet;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(in crate::pet_runtime)struct Node{pub pet_id:String,pub goal_id:String,pub decision_id:String}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(in crate::pet_runtime)struct Lineage{pub nodes:Vec<Node>,pub max_depth:u8,pub deadline_at:u64}
impl Authority{
 pub(in crate::pet_runtime)fn delegation_lineage_unlocked(&self,source:&Source,child_goal:Option<&str>,now:u64)->Result<Lineage>{
  self.delegation_lineage_mode_unlocked(source,child_goal,now,false)
 }
 pub(in crate::pet_runtime)fn delegation_result_lineage_unlocked(&self,source:&Source,now:u64)->Result<Lineage>{self.delegation_lineage_mode_unlocked(source,None,now,true)}
 fn delegation_lineage_mode_unlocked(&self,source:&Source,child_goal:Option<&str>,now:u64,result_data:bool)->Result<Lineage>{
  if !source.valid()||child_goal.is_some_and(|g|!model::goal_id(g)){return Err(Failure::Invalid);}
  let mut cursor=source.clone();let mut recipient_goal=child_goal.map(str::to_owned);let mut pets=BTreeSet::new();let mut goals=BTreeSet::new();let mut decisions=BTreeSet::new();let mut edges:Vec<(Node,Envelope)>=vec![];let mut root=false;let mut deadline=source.deadline_at;
  for _ in 0..8{
   let received=self.received_source_unlocked(&cursor.reference,&cursor.recipient_pet_id,now)?;
   if crate::pet_runtime::authority_store::received_request::Witness::fact(&cursor.reference,&received)?!=cursor{return Err(Failure::Changed);}
   let e=received.envelope();let payload=e.delegation.as_ref().ok_or(Failure::Changed)?;
   if !matches!(e.schema_version,4|7|12|14|16)||e.kind!=crate::pet_runtime::messages::Kind::Delegation{return Err(Failure::Changed);}
   if matches!(e.schema_version,12|14|16){self.received_planned_data_unlocked(&cursor.reference,&cursor.recipient_pet_id,now)?;}
   let(parent,_)=self.grant_file(&format!("{}.json",cursor.reference.decision_id))?;
   if parent.goal_id!=cursor.sender_goal_id||child_goal==Some(parent.goal_id.as_str())||!pets.insert(cursor.sender_pet_id.clone())||!goals.insert(parent.goal_id.clone())||!decisions.insert(parent.decision_id.clone()){return Err(Failure::Changed);}
   if now<parent.updated_at||parent.stop_reason==Some(crate::pet_runtime::leases::StopReason::Cancel){return Err(Failure::Revoked);}
   if parent.messages.as_ref().is_none_or(|m|!m.entries.iter().any(|r|r.envelope()==e&&model::fingerprint(&r.scope).ok().as_ref()==Some(&cursor.reference.scope_hash))){return Err(Failure::Changed);}
   let branch=parent.delegation_reservations.iter().find(|r|r.message_id==cursor.reference.message_id&&r.scope_hash==cursor.reference.scope_hash&&r.recipient_pet_id==cursor.recipient_pet_id&&r.allowance==payload.allowance&&r.deadline_at==cursor.deadline_at&&r.reserved_at==cursor.received_at).ok_or(Failure::Changed)?;
   if !Self::delegation_parent_available(&parent,now)&&!Self::recovered_parent_bound_available(&parent,branch,recipient_goal.as_deref(),now){return Err(Failure::Revoked);}
   if !(if result_data{matches!(branch.state,Branch::Bound{..}|Branch::Settled{cancellation:None,..})}else{matches!(branch.state,Branch::Pending|Branch::Bound{..})}){return Err(Failure::Revoked);}
   deadline=deadline.min(cursor.deadline_at);edges.push((Node{pet_id:cursor.sender_pet_id.clone(),goal_id:parent.goal_id.clone(),decision_id:parent.decision_id.clone()},e.clone()));
   let Some(f)=&parent.funded_by else{if parent.delegation_pool.is_none(){return Err(Failure::Changed);}root=true;break;};
   // Only the explicitly typed recursive pool may be an intermediate owner.
   // No old funded leaf/response becomes a pool by containing an ancestor array.
   if !(matches!(parent.schema_version,28|32|35)&&f.schema_version==5||parent.schema_version==47&&parent.hybrid_pool()&&f.schema_version==9)||!f.previous.is_empty()||!f.basic()||f.source.recipient_pet_id!=cursor.sender_pet_id||parent.delegation_pool.as_ref().is_none_or(|b|b.deadline_at>f.source.deadline_at||b.max_model_calls>f.allowance.model_calls||b.max_tool_calls>f.allowance.tool_calls||b.max_output_bytes>f.allowance.output_bytes)||parent.approved_at<f.source.received_at{return Err(Failure::Changed);}
   let prior=self.received_source_unlocked(&f.source.reference,&f.source.recipient_pet_id,now)?;if parent.schema_version==47{if e.schema_version!=16||e.planned_data.as_ref().and_then(|p|p.source.received_request.as_deref()).is_none_or(|r|r.source!=f.source||r.own_origin_hash!=f.own_origin_hash){return Err(Failure::Changed);}}else if e.schema_version!=7||e.reply_to.as_ref()!=Some(&f.source.reference)||e.correlation_id!=prior.envelope().correlation_id{return Err(Failure::Changed);}
   let(grandparent,_)=self.grant_file(&format!("{}.json",f.source.reference.decision_id))?;
   let bound=grandparent.delegation_reservations.iter().find(|r|r.message_id==f.source.reference.message_id&&r.scope_hash==f.source.reference.scope_hash&&r.allowance==f.allowance&&r.recipient_pet_id==f.source.recipient_pet_id).ok_or(Failure::Changed)?;
   if !(match &bound.state{Branch::Bound{binding}=>Some(binding),Branch::Settled{binding,cancellation:None,..}if result_data=>Some(binding),_=>None}).is_some_and(|binding|binding.goal_id==parent.goal_id&&binding.pet_id==cursor.sender_pet_id&&binding.decision_id==parent.decision_id&&binding.admission_hash==parent.spec_hash){return Err(Failure::Revoked);}
   recipient_goal=Some(parent.goal_id.clone());cursor=f.source.clone();
  }
  if !root||pets.contains(&source.recipient_pet_id){return Err(Failure::Changed);}edges.reverse();let mut ancestors=vec![];let mut nodes=vec![];let mut max_depth=8;
  for(index,(node,e))in edges.into_iter().enumerate(){
   ancestors.push(node.pet_id.clone());let p=e.delegation.as_ref().ok_or(Failure::Changed)?;
   if p.ancestors!=ancestors||p.max_depth==0||p.max_depth>max_depth||index+1>usize::from(p.max_depth)||(if index==0{!matches!(e.schema_version,4|12|14|16)||p.schema_version!=1}else{!matches!(e.schema_version,7|16)||p.schema_version!=2}){return Err(Failure::Changed);}max_depth=p.max_depth;nodes.push(node);
  }
  self.fence()?;Ok(Lineage{nodes,max_depth,deadline_at:deadline})
 }
 pub(in crate::pet_runtime)fn recursive_payload(&self,spec:&AdmissionSpec,proof:&ConsentProof,args:&delegation::Arguments,now:u64)->Result<Payload>{
  let _guard=self.lock_shared()?;let body=self.load(spec,proof)?;
  if body.phase!=GrantPhase::Claimed||body.native_completion.is_some(){return Err(Failure::Revoked);}self.recursive_payload_unlocked(spec,args,now)
 }
 pub(in crate::pet_runtime)fn recursive_payload_unlocked(&self,spec:&AdmissionSpec,args:&delegation::Arguments,now:u64)->Result<Payload>{
  self.funding_current_unlocked(spec,now)?;self.recursive_proposed_payload_unlocked(spec,args,now)
 }
 pub(in crate::pet_runtime::authority_store)fn recursive_proposed_payload_unlocked(&self,spec:&AdmissionSpec,args:&delegation::Arguments,now:u64)->Result<Payload>{
  let f=spec.funding.as_ref().ok_or(Failure::Invalid)?;
  if !(spec.schema_version==17&&f.schema_version==5||spec.peer_funded_iteration()&&f.schema_version==9)||spec.continuation.is_some()||!args.allowance.within(&spec.budget){return Err(Failure::Invalid);}
  let lineage=self.delegation_lineage_unlocked(&f.source,Some(&spec.goal_id),now)?;
  if lineage.nodes.iter().any(|n|n.pet_id==spec.owner.pet_id||n.goal_id==spec.goal_id)||args.max_depth>lineage.max_depth||lineage.nodes.len()+1>usize::from(args.max_depth)||args.expires_at>lineage.deadline_at{return Err(Failure::Changed);}
  let mut ancestors:Vec<_>=lineage.nodes.iter().map(|n|n.pet_id.clone()).collect();ancestors.push(spec.owner.pet_id.clone());
  if ancestors.contains(&args.to_pet_id){return Err(Failure::Changed);}self.fence()?;Ok(Payload{schema_version:2,subgoal:args.subgoal.clone(),criteria:args.criteria.clone(),allowance:args.allowance.clone(),max_depth:args.max_depth,ancestors})
 }
}

impl Authority{
 pub(in crate::pet_runtime::authority_store)fn recursive_message_unlocked(&self,spec:&AdmissionSpec,scope:&crate::pet_runtime::messages::Scope,now:u64)->Result<()>{
  let e=&scope.envelope;
  if e.schema_version==7||spec.peer_funded_iteration()&&e.delegation.is_some(){
   if spec.peer_funded_iteration()&&e.schema_version!=16{return Err(Failure::Invalid);}
   let payload=e.delegation.as_ref().ok_or(Failure::Invalid)?;let args=delegation::Arguments{to_pet_id:e.recipient.pet_id.clone(),subgoal:e.text.clone(),criteria:payload.criteria.clone(),allowance:payload.allowance.clone(),expires_at:e.deadline_at,max_depth:payload.max_depth};
   if self.recursive_payload_unlocked(spec,&args,now)?!=*payload{return Err(Failure::Changed);}
  }else if spec.schema_version==17&&e.delegation.is_some(){return Err(Failure::Invalid);}
  if e.schema_version==9&&(spec.schema_version!=20||spec.funding.as_ref().is_none_or(|f|f.schema_version!=8||e.reply_to.as_ref()!=Some(&f.source.reference))||spec.result_disclosure.as_ref().is_none_or(|p|!p.matches_envelope(e))){return Err(Failure::Invalid);}if e.schema_version==8&&(spec.schema_version!=18||spec.funding.as_ref().is_none_or(|f|f.schema_version!=6||e.reply_to.as_ref()!=Some(&f.source.reference))){return Err(Failure::Invalid);}Ok(())
 }
}

impl Authority{
 pub(in crate::pet_runtime::authority_store)fn recursive_admission_unlocked(&self,spec:&AdmissionSpec,goal:&Record,now:u64)->Result<()>{
  if !matches!(spec.schema_version,17|18|19|20){return Ok(());}let f=spec.funding.as_ref().ok_or(Failure::Invalid)?;
  if !f.valid_for(spec)||!f.matches(goal){return Err(Failure::Changed);}self.delegation_lineage_unlocked(&f.source,Some(&spec.goal_id),now)?;
  if matches!(spec.schema_version,18|19|20){return if !crate::pet_runtime::delegation::is_pool(goal){Ok(())}else{Err(Failure::Invalid)};}
  let mut allocated=delegation::Allowance::default();let mut count=0usize;let mut delegated=0usize;
  for step in goal.plans.iter().flat_map(|p|&p.steps){
   count+=1;allocated.add(&delegation::Allowance{model_calls:0,tool_calls:1,output_bytes:0}).ok_or(Failure::Capacity)?;
   if let Some(crate::pet_runtime::messages::Input::Delegate(args))=step.operation.as_ref().and_then(|o|crate::pet_runtime::messages::Input::parse(o).ok()){
    delegated+=1;self.recursive_proposed_payload_unlocked(spec,&args,now)?;allocated.add(&args.allowance).ok_or(Failure::Capacity)?;
   }
  }
  if count>crate::pet_runtime::messages::MAX_MESSAGES||delegated==0||!allocated.within(&spec.budget){return Err(Failure::Capacity);}self.fence()
 }
}
