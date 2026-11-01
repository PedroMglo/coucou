//! Public data bindings, not credentials, permission tickets or executable grants.
use serde::{Deserialize,Serialize};
use super::model::{Owner,Scope,Entry};
pub(crate)const MAX_SELECTED:usize=16;
pub(crate)const MAX_CONTEXT_BYTES:usize=64*1024;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Selected{pub id:String,pub revision:u64}
/// Immutable selected data references. The parent goal binds the real pet and
/// optional source public CID; a generated private CID grants no source scope.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Selection{pub schema_version:u32,pub references:Vec<Reference>}
impl Selection{
 pub(crate)fn valid(&self,pet:&str,source_cid:Option<&str>)->bool{
  let mut ids=std::collections::BTreeSet::new();self.schema_version==1&&!self.references.is_empty()&&self.references.len()<=MAX_SELECTED
   &&self.references.iter().all(|r|r.valid(pet,source_cid.unwrap_or(""))&&ids.insert(&r.id)&&match &r.owner.scope{Scope::Pet{}=>true,Scope::Conversation{conversation_id}=>source_cid==Some(conversation_id.as_str())})
 }
 pub(crate)fn matches(&self,selected:&[Selected])->bool{self.references.len()==selected.len()&&self.references.iter().zip(selected).all(|(r,s)|r.id==s.id&&r.revision==s.revision)}
}
pub(crate)fn same_selection(stored:Option<&Selection>,requested:Option<&[Selected]>)->bool{match(stored,requested){(None,None)=>true,(Some(a),Some(b))=>a.matches(b),_=>false}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Reference{pub id:String,pub revision:u64,pub owner:Owner,pub content_hash:String,pub expires_at:Option<u64>}
pub(crate)fn digest(bytes:&[u8])->String{ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn text(s:&str,max:usize,empty:bool)->bool{(empty||!s.trim().is_empty())&&s.len()<=max&&!s.chars().any(char::is_control)}
fn hash(s:&str)->bool{s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
pub(crate)fn owner_scope(owner:&Owner,pet:&str,cid:&str)->bool{owner.pet_id==pet&&match &owner.scope{Scope::Pet{}=>true,Scope::Conversation{conversation_id}=>conversation_id==cid}}
impl Reference{
 pub(super)fn capture(entry:&Entry)->Result<Self,&'static str>{Ok(Self{id:entry.id.clone(),revision:entry.revision,owner:entry.owner.clone(),content_hash:digest(&serde_json::to_vec(entry).map_err(|_|"invalidMemory")?),expires_at:entry.expires_at})}
 pub(crate)fn valid(&self,pet:&str,cid:&str)->bool{crate::assistant_memory::valid_id(&self.id)&&self.revision>0&&self.owner.valid()&&owner_scope(&self.owner,pet,cid)&&hash(&self.content_hash)&&self.expires_at.is_none_or(|v|v>0)}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub(crate)enum Destination{
 Server{provider:String,endpoint:String,model:String},
 Api{provider:String,endpoint:String,model:String},
 NativeSession{provider:String,adapter:String,model:String,workspace:String,session_id:Option<String>},
}
impl Destination{
 pub(crate)fn provider(&self)->&str{match self{Self::Server{provider,..}|Self::Api{provider,..}|Self::NativeSession{provider,..}=>provider}}
 pub(crate)fn native(&self)->bool{matches!(self,Self::NativeSession{..})}
 pub(crate)fn valid(&self)->bool{
  self.structurally_valid()&&crate::pet_config::integration_pet(self.provider()).is_some_and(|p|!crate::pet_config::retired(p))
 }
 fn structurally_valid(&self)->bool{
  let provider=self.provider();if !text(provider,64,false)||crate::pet_config::integration_pet(provider).is_none(){return false;}
  match self{
   Self::Server{provider,endpoint,model}|Self::Api{provider,endpoint,model}=>{
    let kind=match self{Self::Server{..}=>["ollama","lmstudio"].contains(&provider.as_str()),_=>["anthropic","openai","google"].contains(&provider.as_str())};
    kind&&text(model,256,false)&&reqwest::Url::parse(endpoint).is_ok_and(|u|matches!(u.scheme(),"http"|"https")&&u.host_str().is_some()&&u.username().is_empty()&&u.password().is_none()&&u.query().is_none()&&u.fragment().is_none())
   },
   Self::NativeSession{adapter,model,workspace,session_id,..}=>crate::cli_chat::backend(provider)==Some(adapter.as_str())&&text(model,256,true)&&!model.starts_with('@')&&text(workspace,4096,false)&&std::path::Path::new(workspace).is_absolute()&&session_id.as_ref().is_none_or(|s|text(s,512,false)),
  }
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum Consent{ThisRequest,NativeSessionContext}
impl Consent{pub(crate)fn matches(self,destination:&Destination)->bool{matches!((self,destination.native()),(Self::ThisRequest,false)|(Self::NativeSessionContext,true))}}
/// Goal data consent is a separate scope from a manual turn/native session.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate)enum GoalConsent{BoundedGoalContext}
pub(crate)const GOAL_INPUT_LIMIT:usize=256*1024;
/// Signed as part of the existing admission. JSON/this format check alone is
/// not an executable permit; Authority still requires the original Approved.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct GoalGrant{
 pub schema_version:u32,pub consent:GoalConsent,pub destination:Destination,pub max_input_bytes:usize,
}
impl GoalGrant{
 pub(crate)fn valid(&self,provider:&str,model:&str)->bool{
  self.schema_version==1&&self.consent==GoalConsent::BoundedGoalContext&&self.max_input_bytes==GOAL_INPUT_LIMIT
   &&self.destination.valid()&&matches!(&self.destination,Destination::Server{provider:p,endpoint,model:m}
    if p==provider&&m==model&&model.trim()==model&&!model.starts_with('@')&&reqwest::Url::parse(endpoint).is_ok_and(|url|crate::net::is_loopback_url(&url)))
 }
}
/// Complete public review material; never Deserialize and never an authority.
#[derive(Clone,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate)struct GoalPreview{
 pub schema_version:u32,pub goal_id:String,pub pet_id:String,
 pub source_conversation_id:Option<String>,pub destination:Destination,
 pub records:Vec<Entry>,pub required_consent:GoalConsent,pub max_input_bytes:usize,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct Decision{pub token:String,pub consent:Consent}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate)struct NativeContext{
 pub schema_version:u32,pub pet_id:String,pub provider:String,pub conversation_id:String,
 pub destination:Destination,pub references:Vec<Reference>,pub possibly_delivered_at:u64,pub revoked:bool,
}
impl NativeContext{
 pub(crate)fn valid_for(&self,provider:&str,cid:&str)->bool{
  let mut ids=std::collections::BTreeSet::new();
  self.schema_version==1&&self.provider==provider&&self.conversation_id==cid&&crate::assistant_memory::valid_id(cid)&&crate::pet_config::integration_pet(provider)==Some(self.pet_id.as_str())
   &&self.destination.native()&&self.destination.structurally_valid()&&self.destination.provider()==provider&&self.possibly_delivered_at>0&&!(self.references.is_empty()||self.references.len()>MAX_SELECTED)
   &&self.references.iter().all(|r|r.valid(&self.pet_id,cid)&&ids.insert(&r.id))
 }
}
