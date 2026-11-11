//! Explicit persistent enrollment metadata. Parsing/resolution never reads a source.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::{Component, Path}};

pub(super) const MAX_BYTES:usize=256*1024;
const MAX_SOURCES:usize=64;
const MAX_OWNERS:usize=32;
const MAX_SELECTED:usize=32;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Registry {
    pub version:u32,
    pub generation:u64,
    pub sources:Vec<Source>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Source {
    pub id:String,
    pub revision:u64,
    pub kind:Kind,
    pub name:String,
    pub path:String,
    pub enabled:bool,
    pub owners:Vec<Owner>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub(crate) enum Kind {Ics}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(untagged)]
pub(crate) enum Owner {Conversation(ConversationOwner),Pet(PetOwner)}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct ConversationOwner {pub pet_id:String,pub conversation_id:String}
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct PetOwner {pub pet_id:String,pub scope:PetScope}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum PetScope {Pet}
impl Owner{
 pub(super)fn conversation(pet:&str,cid:&str)->Self{Self::Conversation(ConversationOwner{pet_id:pet.into(),conversation_id:cid.into()})}
 pub(super)fn pet(pet:&str)->Self{Self::Pet(PetOwner{pet_id:pet.into(),scope:PetScope::Pet})}
 pub(super)fn pet_id(&self)->&str{match self{Self::Conversation(o)=>&o.pet_id,Self::Pet(o)=>&o.pet_id}}
 pub(super)fn conversation_id(&self)->Option<&str>{match self{Self::Conversation(o)=>Some(&o.conversation_id),Self::Pet(_)=>None}}
}

fn source_id(s:&str)->bool{!s.is_empty()&&s.len()<=64&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))}
fn source_path(s:&str)->bool{
    !s.is_empty()&&s.len()<=4096&&!s.chars().any(char::is_control)&&Path::new(s).is_absolute()
        &&!Path::new(s).components().any(|p|matches!(p,Component::ParentDir))
        &&!s.split('/').any(|p|p==".")
        &&Path::new(s).extension().is_some_and(|e|e.eq_ignore_ascii_case("ics"))
}
impl Registry{
    pub(super) fn empty()->Self{Self{version:1,generation:0,sources:vec![]}}
    pub(super) fn validate(&self)->Result<(),&'static str>{
        if !matches!(self.version,1|2|3)||self.sources.len()>MAX_SOURCES{return Err("invalidRegistry");}
        let pets=crate::pet_config::defaults();let mut ids=BTreeSet::new();let mut selected=std::collections::BTreeMap::new();
        for source in &self.sources{
            if !source_id(&source.id)||!ids.insert(&source.id)||source.revision==0{return Err("invalidSourceIdentity");}
            if source.name.trim().is_empty()||source.name.len()>160||source.name.chars().any(char::is_control)||!source_path(&source.path){return Err("invalidSourceMetadata");}
            if source.owners.len()>MAX_OWNERS||source.enabled&&source.owners.is_empty(){return Err("invalidSourceOwners");}
            let mut owners=BTreeSet::new();
            for owner in &source.owners{
                if !pets.pets.contains_key(owner.pet_id())||match owner{Owner::Conversation(o)=>!crate::assistant_memory::valid_id(&o.conversation_id),Owner::Pet(_)=>self.version!=3}||!owners.insert(owner){return Err("invalidSourceOwners");}
                if source.enabled{
                    let count=selected.entry(owner).or_insert(0usize);*count+=1;
                    if self.version==1&&*count>1{return Err("ambiguousSourceSelection");}
                    if *count>MAX_SELECTED{return Err("sourceSelectionLimit");}
                }
            }
        }
        Ok(())
    }
    pub(super) fn selected(&self,pet_id:&str,cid:&str)->Result<Option<&Source>,&'static str>{
        let sources=self.selected_many(pet_id,cid)?;
        if sources.len()>1{return Err("ambiguousSourceSelection");}Ok(sources.into_iter().next())
    }
    pub(super)fn selected_many(&self,pet_id:&str,cid:&str)->Result<Vec<&Source>,&'static str>{
        self.validate()?;if !crate::assistant_memory::valid_id(cid){return Err("invalidOwner");}
        Ok(self.sources.iter().filter(|s|s.enabled&&s.owners.contains(&Owner::conversation(pet_id,cid))).collect())
    }
    pub(super)fn selected_pet(&self,pet_id:&str)->Result<Vec<&Source>,&'static str>{
        self.validate()?;let owner=Owner::pet(pet_id);Ok(self.sources.iter().filter(|s|s.enabled&&s.owners.contains(&owner)).collect())
    }
    /// An edit starts from the exact document revision and generation returned
    /// by the reader. Source revisions are owned by this writer, not a model.
    pub(super) fn next(mut self,previous:&Self)->Result<Self,&'static str>{
        if self.generation!=previous.generation{return Err("registryConflict");}
        if self.version<previous.version{return Err("registryFormatDowngrade");}
        self.generation=previous.generation.checked_add(1).ok_or("revisionLimit")?;
        for source in &mut self.sources{
            if let Some(old)=previous.sources.iter().find(|s|s.id==source.id){
                if source.revision!=old.revision{return Err("sourceRevisionConflict");}
                if source!=old{source.revision=old.revision.checked_add(1).ok_or("revisionLimit")?;}
            }else{
                if source.revision>1{return Err("sourceRevisionConflict");}
                source.revision=1;
            }
        }
        self.validate()?;Ok(self)
    }
}

#[cfg(test)]
pub(super) mod tests{
    use super::*;
    use serde_json::{json,Value};
    pub(crate) fn fixture()->Registry{
        serde_json::from_value(json!({"version":1,"generation":1,"sources":[{"id":"agenda","revision":1,"kind":"ics","name":"Agenda pessoal","path":"/not-opened-by-metadata/calendar.ics","enabled":true,"owners":[{"petId":"agent_codex","conversationId":"cid-one"}]}]})).unwrap()
    }
    #[test]
    fn missing_registry_is_disabled_and_metadata_resolution_has_no_source_io(){
        let empty=Registry::empty();assert!(empty.validate().is_ok());assert!(empty.selected("agent_codex","cid-one").unwrap().is_none());
        let registry=fixture();assert!(registry.validate().is_ok());assert_eq!(registry.selected("agent_codex","cid-one").unwrap().unwrap().id,"agenda");
        assert!(registry.selected("agent_antigravity","cid-one").unwrap().is_none());assert!(registry.selected("agent_codex","cid-two").unwrap().is_none());
        assert!(registry.selected("agent_codex","../bad").is_err());
    }
    #[test]
    fn source_enablement_and_owner_scope_are_explicit(){
        let original=serde_json::to_value(fixture()).unwrap();
        for key in ["kind","path","enabled","owners","id","revision"]{let mut v=original.clone();v["sources"][0].as_object_mut().unwrap().remove(key);assert!(serde_json::from_value::<Registry>(v).is_err(),"{key}");}
        for key in ["petId","conversationId"]{let mut v=original.clone();v["sources"][0]["owners"][0].as_object_mut().unwrap().remove(key);assert!(serde_json::from_value::<Registry>(v).is_err());}
        let mut v=fixture();v.sources[0].enabled=false;assert!(v.selected("agent_codex","cid-one").unwrap().is_none());
    }
    #[test]
    fn unknown_properties_and_account_types_never_grant_access(){
        for layer in [0,1,2]{let mut v=serde_json::to_value(fixture()).unwrap();let target=match layer{0=>&mut v,1=>&mut v["sources"][0],_=>&mut v["sources"][0]["owners"][0]};target["permission"]=json!("always");assert!(serde_json::from_value::<Registry>(v).is_err());}
        for kind in ["imap","rss","https","postgres","directory"]{let mut v=serde_json::to_value(fixture()).unwrap();v["sources"][0]["kind"]=json!(kind);assert!(serde_json::from_value::<Registry>(v).is_err());}
        let mut v=fixture();v.sources[0].owners[0]=Owner::conversation("not-a-configured-pet","cid-one");assert!(v.validate().is_err());
    }
    #[test]
    fn limits_paths_duplicates_and_ambiguous_selection_are_rejected(){
        for path in ["relative.ics","/tmp/../private.ics","/tmp/./private.ics","/tmp/calendar.pdf","https://example.invalid/calendar.ics","/tmp/a\n.ics"]{let mut v=fixture();v.sources[0].path=path.into();assert!(v.validate().is_err(),"{path}");}
        let mut v=fixture();v.sources[0].path="/tmp/Export.ICS".into();assert!(v.validate().is_ok());
        let mut v=fixture();v.sources.push(v.sources[0].clone());assert!(v.validate().is_err());v.sources[1].id="other".into();assert_eq!(v.validate(),Err("ambiguousSourceSelection"));v.sources[1].enabled=false;assert!(v.validate().is_ok());
        let mut v=fixture();let owner=v.sources[0].owners[0].clone();v.sources[0].owners.push(owner);assert!(v.validate().is_err());
        let mut v=fixture();v.sources[0].owners.clear();assert!(v.validate().is_err());v.sources[0].enabled=false;assert!(v.validate().is_ok());
        let mut v=fixture();v.sources[0].name="x".repeat(161);assert!(v.validate().is_err());
        let mut v=fixture();let source=v.sources[0].clone();v.sources.resize(65,source);assert!(v.validate().is_err());
    }
    #[test]
    fn writer_owns_generations_and_source_revisions(){
        let old=fixture();let unchanged=old.clone().next(&old).unwrap();assert_eq!(unchanged.generation,2);assert_eq!(unchanged.sources[0].revision,1);
        let mut changed=old.clone();changed.sources[0].name="Nova agenda".into();let changed=changed.next(&old).unwrap();assert_eq!(changed.sources[0].revision,2);
        assert_eq!(old.clone().next(&changed).unwrap_err(),"registryConflict");
        let mut invalid=old.clone();invalid.sources[0].revision=0;assert_eq!(invalid.next(&old).unwrap_err(),"sourceRevisionConflict");
        let mut new=old.clone();new.generation=0;new.sources[0].revision=0;let saved=new.next(&Registry::empty()).unwrap();assert_eq!(saved.generation,1);assert_eq!(saved.sources[0].revision,1);
        let mut limit=old.clone();limit.generation=u64::MAX;assert_eq!(limit.clone().next(&limit).unwrap_err(),"revisionLimit");
    }
    #[test]
    fn duplicate_json_fields_and_invalid_versions_do_not_replace_good_metadata(){
        assert!(serde_json::from_str::<Registry>("{\"version\":1,\"version\":1,\"generation\":1,\"sources\":[]}").is_err());
        let mut v=fixture();v.version=4;assert!(v.validate().is_err());
        let mut v=serde_json::to_value(fixture()).unwrap();v["sources"][0]["enabled"]=Value::String("true".into());assert!(serde_json::from_value::<Registry>(v).is_err());
    }
}


#[cfg(test)]mod persistent_collection_tests{
 use super::*;
 #[test]fn persistent_collection_requires_version2_and_never_falls_back_to_first_source(){
  let old=super::tests::fixture();let bytes=serde_json::to_vec(&old).unwrap();let mut next=old.clone();let mut source=next.sources[0].clone();source.id="other-calendar".into();source.revision=0;next.sources.push(source);
  assert_eq!(next.clone().next(&old).unwrap_err(),"ambiguousSourceSelection");next.version=2;let saved=next.next(&old).unwrap();assert_eq!(saved.selected_many("agent_codex","cid-one").unwrap().len(),2);assert_eq!(saved.selected("agent_codex","cid-one"),Err("ambiguousSourceSelection"));assert!(saved.selected_many("agent_codex","cid-two").unwrap().is_empty());assert_eq!(saved.sources[0],old.sources[0]);assert_eq!(saved.sources[1].revision,1);assert_eq!(serde_json::to_vec(&old).unwrap(),bytes);
 }
 #[test]fn persistent_collection_limit_and_format_downgrade_are_closed(){
  let mut book=super::tests::fixture();book.version=2;book.sources=(0..32).map(|i|{let mut s=book.sources[0].clone();s.id=format!("calendar-{i}");s}).collect();assert!(book.validate().is_ok());let mut extra=book.sources[0].clone();extra.id="calendar-33".into();book.sources.push(extra);assert_eq!(book.validate(),Err("sourceSelectionLimit"));book.sources.truncate(1);let mut older=book.clone();older.version=1;assert_eq!(older.next(&book).unwrap_err(),"registryFormatDowngrade");
 }
}

#[cfg(test)]mod pet_source_scope_tests{
 use super::*;
 #[test]fn explicit_pet_membership_requires_new_format_and_never_widens_conversations(){
  let old=super::tests::fixture();let legacy=serde_json::to_vec(&old).unwrap();let mut book=old.clone();book.sources[0].owners.push(Owner::pet("agent_codex"));assert!(book.validate().is_err());book.version=2;assert!(book.validate().is_err());book.version=3;assert!(book.validate().is_ok());
  assert_eq!(book.selected_pet("agent_codex").unwrap().len(),1);assert!(book.selected_pet("agent_antigravity").unwrap().is_empty());assert!(book.selected_many("agent_codex","unrelated-cid").unwrap().is_empty());assert_eq!(book.selected_many("agent_codex","cid-one").unwrap().len(),1);
  let mut downgrade=book.clone();downgrade.sources[0].owners.retain(|o|o.conversation_id().is_some());downgrade.version=2;assert_eq!(downgrade.next(&book),Err("registryFormatDowngrade"));assert_eq!(serde_json::to_vec(&old).unwrap(),legacy);
  assert_eq!(serde_json::to_vec(&Owner::conversation("agent_codex","cid-one")).unwrap(),br#"{"petId":"agent_codex","conversationId":"cid-one"}"#);
 }
 #[test]fn owner_scope_is_typed_closed_and_cannot_be_disguised_as_legacy(){
  for raw in [r#"{"petId":"agent_codex"}"#,r#"{"petId":"agent_codex","scope":"all"}"#,r#"{"petId":"agent_codex","scope":"pet","conversationId":"cid-one"}"#,r#"{"petId":"agent_codex","scope":"pet","grant":true}"#]{assert!(serde_json::from_str::<Owner>(raw).is_err(),"{raw}");}
  let mut book=super::tests::fixture();book.version=3;book.sources[0].owners=vec![Owner::pet("agent_codex"),Owner::pet("agent_codex")];assert!(book.validate().is_err());
  book.sources[0].owners.pop();let original=serde_json::to_vec(&book).unwrap();book.sources[0].enabled=false;let suspended=serde_json::to_vec(&book).unwrap();assert_eq!(crate::pet_sources::suspend_for_restoration(&suspended).unwrap(),suspended);assert!(serde_json::from_slice::<Registry>(&original).unwrap().validate().is_ok());
 }
}
