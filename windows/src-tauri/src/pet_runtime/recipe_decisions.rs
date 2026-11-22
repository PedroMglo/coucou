//! A recipe decision has its own origin and native lifetime. It is neither a
//! terminal permission nor a single local command; serialized data cannot make it.
use super::{model::{self,Record},store::Store,lease_controller::{Owned,Validity,recipe_controller::{Review,Proposal}}};
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use serde::{Serialize,Deserialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Target {
    pub origin:Origin,pub pet_id:String,pub provider:String,pub goal_id:String,
    pub context_id:String,pub lease_id:String,pub controller_id:String,
    pub recipe_id:String,pub plan_id:String,pub revision:String,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Origin {Goal,GoalPeerPlan,GoalPeerData}
impl Target {
    pub(crate) fn valid(&self)->bool {
        (match self.origin{Origin::Goal=>crate::pet_config::pet_id(&self.provider)==Some(self.pet_id.as_str()),Origin::GoalPeerPlan|Origin::GoalPeerData=>crate::pet_config::blueprint(&self.pet_id).is_some_and(|b|!b.retired)&&model::key(&self.provider)})
            &&[&self.goal_id,&self.context_id,&self.lease_id,&self.controller_id,&self.recipe_id,&self.plan_id,&self.revision].iter().all(|v|model::key(v))
    }
}
#[derive(Clone)]
pub(crate) struct Source(Arc<Inner>);
struct Peer{batch:Arc<super::peer_plan_review::Batch>,observe:super::peer_plan_review::Observe}
struct RoundData{review:Arc<super::peer_round_review::Review>,observe:super::peer_round_review::Observe}
struct Inner {
    wait_only:bool,round_data:Option<RoundData>,peer:Option<Peer>,
    target:Target,review:serde_json::Value,scope_hash:String,goal_hash:String,owner:Arc<Owned>,store:Arc<Store>,
    configuration:Arc<dyn Fn()->model::Result<crate::pet_config::Configuration>+Send+Sync>,
    prepared_at:u64,expires_at:u64,deadline:Instant,ended:AtomicBool,
}
/// Created only after a live native decision transport has consumed its exact
/// registered answer. Neither JSON nor a serialized target constructs it.
pub(crate)struct Decision {source:Source,approved:bool,answered_at:u64}
impl Decision {
    pub(crate)fn from_transport(source:Source,approved:bool)->Option<Self> {
        if source.0.wait_only{return None;}if if approved{!source.current()}else{!source.pending()}{return None;}let answered_at=crate::assistant_memory::now();source.retire();Some(Self{source,approved,answered_at})
    }
    pub(crate)fn approved(&self)->bool {self.approved}
    pub(super)fn answered_at(&self)->u64{self.answered_at}
    pub(super)fn matches_peer_data(&self,review:&super::peer_round_review::Review,goal:&Record,owner:&Owned,now:u64)->bool{
        let i=&self.source.0;
        self.approved&&i.round_data.as_ref().is_some_and(|p|p.review.id==review.id&&p.review.hash==review.hash)&&i.target.origin==Origin::GoalPeerData
            &&self.answered_at>=i.prepared_at&&self.answered_at<i.expires_at&&now>=self.answered_at&&now<i.expires_at
            &&i.goal_hash==review.goal_hash&&model::fingerprint(goal).ok().as_ref()==Some(&i.goal_hash)&&i.scope_hash==review.hash
            &&i.owner.same_live(owner)&&i.target.lease_id==owner.lease().id&&i.target.controller_id==owner.lease().controller_id&&i.target.revision==goal.revision
    }
    pub(super)fn matches_peer(&self,batch:&super::peer_plan_review::Batch,goal:&Record,owner:&Owned,now:u64)->bool{
        let i=&self.source.0;
        self.approved&&i.peer.as_ref().is_some_and(|p|p.batch.id==batch.id&&p.batch.hash==batch.hash)&&i.target.origin==Origin::GoalPeerPlan
            &&self.answered_at>=i.prepared_at&&self.answered_at<i.expires_at&&now>=self.answered_at&&now<i.expires_at
            &&i.goal_hash==batch.goal_hash&&model::fingerprint(goal).ok().as_ref()==Some(&i.goal_hash)&&i.scope_hash==batch.hash
            &&i.owner.same_live(owner)&&i.target.lease_id==owner.lease().id&&i.target.controller_id==owner.lease().controller_id&&i.target.revision==goal.revision
    }
    pub(super)fn matches(&self,proposal:&Proposal,goal:&Record,owner:&Owned,now:u64)->bool {
        let i=&self.source.0;let s=&proposal.scope;
        i.peer.is_none()&&i.round_data.is_none()&&self.answered_at>=i.prepared_at&&self.answered_at<i.expires_at&&now>=self.answered_at&&now<i.expires_at
            &&model::fingerprint(goal).ok().as_ref()==Some(&i.goal_hash)&&model::fingerprint(s).ok().as_ref()==Some(&i.scope_hash)
            &&i.target.lease_id==owner.lease().id&&i.target.controller_id==owner.lease().controller_id&&i.target.recipe_id==s.id
            &&i.target.revision==goal.revision&&i.target.plan_id==s.plan_id
    }
}
impl Source {
    pub(super) fn capture(owner:Arc<Owned>,goal:&Record,review:&Review,store:Arc<Store>,configuration:Arc<dyn Fn()->model::Result<crate::pet_config::Configuration>+Send+Sync>)->model::Result<Self> {
        let p:Proposal=review.proposal();let s=&p.scope;let now=crate::assistant_memory::now();
        owner.step_goal(goal,&configuration()?,now)?;
        if p.goal_revision!=goal.revision||!s.matches(goal)||s.lease_id!=owner.lease().id||s.controller_id!=owner.lease().controller_id||now<p.prepared_at||now>=p.expires_at{return Err(model::Error::Conflict);}
        let target=Target{origin:Origin::Goal,pet_id:s.pet_id.clone(),provider:s.provider.clone(),goal_id:s.goal_id.clone(),context_id:owner.lease().conversation_id.clone(),lease_id:s.lease_id.clone(),controller_id:s.controller_id.clone(),recipe_id:s.id.clone(),plan_id:s.plan_id.clone(),revision:p.goal_revision.clone()};
        if !target.valid(){return Err(model::Error::InvalidRecord);}
        let row=goal.admissions.last().ok_or(model::Error::Conflict)?;
        // Full original intent/criteria appear once. Each step keeps its exact
        // native scope/arguments/postcondition, without repeating the context.
        let recipe=p.decision_review();
        let original=serde_json::json!({"recipe":recipe,"budget":row.spec.budget,"approvalScope":"exactReviewedRecipe","executionStarted":false,"modelPlanningGranted":false,"additionalModelPlanningGranted":false,"existingGoalConsentOperation":if row.spec.planning(){"goalPlanning"}else{"conversationRespond"}});
        let source=Self(Arc::new(Inner{wait_only:false,round_data:None,peer:None,target,review:original,scope_hash:model::fingerprint(s)?,goal_hash:model::fingerprint(goal)?,owner,store,configuration,prepared_at:p.prepared_at,expires_at:p.expires_at,deadline:Instant::now().checked_add(Duration::from_millis(p.expires_at-now)).ok_or(model::Error::Conflict)?,ended:AtomicBool::new(false)}));
        if !source.current(){return Err(model::Error::Conflict);}Ok(source)
    }
    pub(super)fn capture_peer(owner:Arc<Owned>,goal:&Record,batch:Arc<super::peer_plan_review::Batch>,store:Arc<Store>,observe:super::peer_plan_review::Observe)->model::Result<Self>{
        let now=crate::assistant_memory::now();owner.message_goal(goal,now)?;if batch.current(&owner,goal,&observe,now)!=Validity::Current{return Err(model::Error::Conflict);}
        let target=Target{origin:Origin::GoalPeerPlan,pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),goal_id:goal.id.clone(),context_id:owner.lease().conversation_id.clone(),lease_id:owner.lease().id.clone(),controller_id:owner.lease().controller_id.clone(),recipe_id:batch.id.clone(),plan_id:batch.id.clone(),revision:goal.revision.clone()};
        if !target.valid(){return Err(model::Error::InvalidRecord);}let observed=observe.clone();let configuration=Arc::new(move||observed().map(|(c,_)|c));
        let review=serde_json::json!({"recipe":batch.proposal(),"budget":goal.admissions.last().ok_or(model::Error::Conflict)?.spec.budget,"approvalScope":"exactReviewedNativePeerPlan","executionStarted":false,"modelPlanningGranted":false,"additionalModelPlanningGranted":false,"existingGoalConsentOperation":if goal.admissions.last().is_some_and(|r|r.spec.peer_planning()){"goalPlanning"}else{"goalDeterministic"},"recipientExecutionAuthorized":false,"providerDataUseAuthorized":false});
        let prepared_at=batch.prepared_at;let expires_at=batch.expires_at;let scope_hash=batch.hash.clone();let goal_hash=batch.goal_hash.clone();
        let source=Self(Arc::new(Inner{wait_only:false,round_data:None,peer:Some(Peer{batch,observe}),target,review,scope_hash,goal_hash,owner,store,configuration,prepared_at,expires_at,deadline:Instant::now().checked_add(Duration::from_millis(expires_at-now)).ok_or(model::Error::Conflict)?,ended:AtomicBool::new(false)}));
        if !source.current(){return Err(model::Error::Conflict);}Ok(source)
    }
    pub(in crate::pet_runtime)fn capture_peer_data(owner:Arc<Owned>,goal:&Record,data:Arc<super::peer_round_review::Review>,store:Arc<Store>,observe:super::peer_round_review::Observe)->model::Result<Self>{
        let now=crate::assistant_memory::now();owner.message_goal(goal,now)?;if data.pending_validity(&owner,goal,&observe,now)!=Validity::Current{return Err(model::Error::Conflict);}
        let target=Target{origin:Origin::GoalPeerData,pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),goal_id:goal.id.clone(),context_id:owner.lease().conversation_id.clone(),lease_id:owner.lease().id.clone(),controller_id:owner.lease().controller_id.clone(),recipe_id:data.id.clone(),plan_id:data.id.clone(),revision:goal.revision.clone()};
        if !target.valid(){return Err(model::Error::InvalidRecord);}let observed=observe.clone();let configuration=Arc::new(move||observed().map(|(c,_)|c));
        let review=serde_json::json!({"data":data.proposal(),"budget":goal.admissions.last().ok_or(model::Error::Conflict)?.spec.budget,"approvalScope":"exactReviewedPeerPlanningData","executionStarted":false,"modelPlanningGranted":false,"additionalModelPlanningGranted":false,"existingGoalConsentOperation":"goalPlanning","recipientExecutionAuthorized":false,"newBudgetGranted":false,"requiresOriginalJournalAndBudgetBeforeDispatch":true});
        let prepared_at=data.prepared_at;let expires_at=data.expires_at;let scope_hash=data.hash.clone();let goal_hash=data.goal_hash.clone();
        let source=Self(Arc::new(Inner{wait_only:false,round_data:Some(RoundData{review:data,observe}),peer:None,target,review,scope_hash,goal_hash,owner,store,configuration,prepared_at,expires_at,deadline:Instant::now().checked_add(Duration::from_millis(expires_at-now)).ok_or(model::Error::Conflict)?,ended:AtomicBool::new(false)}));
        if !source.current(){return Err(model::Error::Conflict);}Ok(source)
    }
    /// Stable parent/policy wait, not a child snapshot and not a human decision.
    /// Child progress cannot revoke it; original parent change/Stop/deadline can.
    pub(in crate::pet_runtime)fn capture_peer_wait(owner:Arc<Owned>,goal:&Record,store:Arc<Store>,configuration:Arc<dyn Fn()->model::Result<crate::pet_config::Configuration>+Send+Sync>)->model::Result<Self>{
        let now=crate::assistant_memory::now();if !goal.admissions.last().is_some_and(|a|a.spec.peer_iteration()){return Err(model::Error::Conflict);}owner.message_goal(goal,now)?;
        let target=Target{origin:Origin::GoalPeerData,pet_id:goal.spec.owner.pet_id.clone(),provider:goal.spec.owner.provider.clone(),goal_id:goal.id.clone(),context_id:owner.lease().conversation_id.clone(),lease_id:owner.lease().id.clone(),controller_id:owner.lease().controller_id.clone(),recipe_id:super::store::nonce()?,plan_id:super::store::nonce()?,revision:goal.revision.clone()};
        if !target.valid(){return Err(model::Error::Conflict);}let expires_at=owner.lease().deadline_at;let deadline=Instant::now().checked_add(Duration::from_millis(expires_at.checked_sub(now).ok_or(model::Error::Conflict)?)).ok_or(model::Error::Conflict)?;let scope_hash=model::fingerprint(&configuration()?.pets.get(&goal.spec.owner.pet_id))?;
        let source=Self(Arc::new(Inner{wait_only:true,round_data:None,peer:None,target,review:serde_json::Value::Null,scope_hash,goal_hash:model::fingerprint(goal)?,owner,store,configuration,prepared_at:now,expires_at,deadline,ended:AtomicBool::new(false)}));if !source.current(){return Err(model::Error::Conflict);}Ok(source)
    }
    pub(crate) fn target(&self)->&Target{&self.0.target}
    pub(crate) fn review(&self)->&serde_json::Value{&self.0.review}
    pub(crate) fn deadline(&self)->Instant{self.0.deadline}
    pub(super)fn resource_entry(&self,owner:&Owned,now:u64,generation:u64)->model::Result<super::resource_wait::Entry>{
        if self.0.peer.is_some()&&!self.peer_planning()||!self.0.owner.same_live(owner)||!self.current(){return Err(model::Error::Conflict);}
        Ok(super::resource_wait::Entry{id:super::store::nonce()?,recipe_id:self.0.target.recipe_id.clone(),goal_revision:self.0.target.revision.clone(),goal_hash:self.0.goal_hash.clone(),generation,yielded_at:now,expires_at:self.0.expires_at,resumed_at:None})
    }
    pub(super)fn matches_wait(&self,owner:&Owned,entry:&super::resource_wait::Entry,now:u64)->bool{
        self.0.owner.same_live(owner)&&entry.recipe_id==self.0.target.recipe_id&&entry.goal_revision==self.0.target.revision&&entry.goal_hash==self.0.goal_hash&&entry.expires_at==self.0.expires_at&&now>=self.0.prepared_at&&now<self.0.expires_at&&Instant::now()<self.0.deadline
    }
    fn peer_planning(&self)->bool{self.0.peer.is_some()&&self.0.review["existingGoalConsentOperation"]=="goalPlanning"}
    pub(super)fn wait_configuration_current(&self)->model::Result<bool>{
        if self.0.wait_only{return Ok(model::fingerprint(&(self.0.configuration)()?.pets.get(&self.0.target.pet_id))?==self.0.scope_hash);}
        if let Some(data)=&self.0.round_data{let goal=self.0.store.read(&self.0.target.goal_id,&self.0.target.pet_id)?;return match data.review.metadata_validity(&self.0.owner,&goal,&data.observe,crate::assistant_memory::now()){Validity::Current=>Ok(true),Validity::Busy=>Err(model::Error::Busy),Validity::Unknown=>Err(model::Error::StorageUnavailable),Validity::Changed=>Ok(false)};}
        if let Some(peer)=&self.0.peer{if !self.peer_planning(){return Ok(false);}let goal=self.0.store.read(&self.0.target.goal_id,&self.0.target.pet_id)?;return match peer.batch.metadata_validity(&self.0.owner,&goal,&peer.observe,crate::assistant_memory::now()){Validity::Current=>Ok(true),Validity::Busy=>Err(model::Error::Busy),Validity::Unknown=>Err(model::Error::StorageUnavailable),Validity::Changed=>Ok(false)};}
        let config=(self.0.configuration)()?;Ok(config.permissions(&self.0.target.pet_id).is_ok_and(|p|p.music))
    }
    pub(crate) fn retire(&self){self.0.ended.store(true,Ordering::Release);}
    pub(crate) fn current(&self)->bool {
        self.validity()==Validity::Current
    }
    /// Unknown proof/policy availability retains this original bounded request;
    /// it never authorizes an affirmative answer/effect until strict validation
    /// succeeds. An exact refusal reduces authority and remains available.
    pub(crate) fn pending(&self)->bool{self.validity()!=Validity::Changed}
    fn validity(&self)->Validity {
        let now=crate::assistant_memory::now();let i=&self.0;
        if i.ended.load(Ordering::Acquire)||Instant::now()>=i.deadline||now<i.prepared_at||now>=i.expires_at{return Validity::Changed;}
        // Stop/deadline stays immediate even if storage is temporarily busy.
        if i.owner.pending_validity(now)==Validity::Changed{return Validity::Changed;}
        let goal=match i.store.read(&i.target.goal_id,&i.target.pet_id){Ok(g)=>g,Err(model::Error::Busy|model::Error::StorageUnavailable)=>return Validity::Unknown,Err(_)=>return Validity::Changed};
        if goal.revision!=i.target.revision||model::fingerprint(&goal).ok().as_ref()!=Some(&i.goal_hash){return Validity::Changed;}
        if let Some(data)=&i.round_data{return data.review.pending_validity(&i.owner,&goal,&data.observe,now);}
        let Ok(config)=(i.configuration)()else{return Validity::Unknown;};
        if i.wait_only{return if model::fingerprint(&config.pets.get(&i.target.pet_id)).ok().as_ref()==Some(&i.scope_hash){i.owner.pending_validity(now)}else{Validity::Changed};}
        if let Some(peer)=&i.peer{return peer.batch.current(&i.owner,&goal,&peer.observe,now);}
        i.owner.pending_step_validity(&goal,&config,now)
    }
    pub(crate) fn matches(&self,request:&serde_json::Value)->bool {
        !self.0.wait_only&&request.get("target").is_none()&&request["goalTarget"]==serde_json::to_value(self.target()).unwrap_or_default()
            &&request["goalReview"]==*self.review()&&request["kind"]=="permission"&&request["approvalScope"]=="recipe"
            &&request["provider"]==self.target().provider
            &&request["detail"].as_str().is_some_and(|detail|serde_json::from_str::<serde_json::Value>(detail).ok().as_ref()==Some(self.review()))
    }
}

impl Source{
    pub(crate)fn voice_approval_available(&self)->bool{!self.0.wait_only&&self.0.round_data.is_none()&&self.0.peer.as_ref().is_none_or(|p|p.batch.proposal().get("roundDataProvenance").is_none())}
    pub(crate)fn spoken_review(&self,code:&str)->String{
        if self.0.round_data.is_some(){return format!("Dados para planeamento {code}, pet {}, objetivo {}. Revê no cartão as fontes, o destino, o modelo e o corpo integral antes de decidir. A leitura integral destes dados ainda não está disponível por voz; não li o conteúdo integral. Para recusar diz recusar receita {code}.",self.target().pet_id,self.target().goal_id);}
        if self.0.peer.is_some(){return format!("Plano entre pets {code}, pet {}, objetivo {}. Para rever todos os dados, destinatários, fontes e prazos consulta o pedido completo. A leitura integral desta sequência ainda não está disponível por voz; não li o conteúdo integral. Para recusar diz recusar receita {code}.",self.target().pet_id,self.target().goal_id);}
        let r=&self.0.review;let p=&r["recipe"];let text=|v:&serde_json::Value|v.as_str().unwrap_or("").to_string();
        let criteria=p["criteria"].as_array().map(|rows|rows.iter().map(text).collect::<Vec<_>>().join("; ")).unwrap_or_default();
        let steps=p["steps"].as_array().map(|rows|rows.iter().enumerate().map(|(i,s)|format!("Passo {}: {} {}. Verificação do passo: {}. Recurso: {}.",i+1,text(&s["scope"]["operationId"]),s["arguments"],text(&s["postcondition"]),s["scope"]["resource"])).collect::<Vec<_>>().join(" ")).unwrap_or_default();
        let speech=format!("Receita {code}, pet {}, objetivo {}. Intenção: {}. Critérios originais: {}. {} Predicados de conclusão revistos: {}. Orçamento máximo: {}. Prazo de execução: {} milissegundos Unix. Validade desta revisão: {} milissegundos Unix. Autorizar cobre só esta sequência e não inicia a execução nem concede planeamento adicional. Para autorizar, diz aprovar receita {code}; para recusar, diz recusar receita {code}.",self.target().pet_id,self.target().goal_id,text(&p["intent"]),criteria,steps,p["scope"]["verification"],r["budget"],p["scope"]["deadlineAt"],p["expiresAt"]);
        if speech.len()>12_000{format!("A receita {code} excede o limite da leitura por voz. Consulta a revisão completa no cartão deste pet antes de decidir. Não li o conteúdo integral.")}else{speech}
    }
}
