//! Explicit persistent enrollment metadata. Parsing/resolution never reads a source.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::{Component, Path}};

pub(super) const MAX_BYTES:usize=256*1024;
const MAX_SOURCES:usize=64;
const MAX_OWNERS:usize=32;

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
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Owner {pub pet_id:String,pub conversation_id:String}

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
        if self.version!=1||self.sources.len()>MAX_SOURCES{return Err("invalidRegistry");}
        let pets=crate::pet_config::defaults();let mut ids=BTreeSet::new();let mut selected=BTreeSet::new();
        for source in &self.sources{
            if !source_id(&source.id)||!ids.insert(&source.id)||source.revision==0{return Err("invalidSourceIdentity");}
            if source.name.trim().is_empty()||source.name.len()>160||source.name.chars().any(char::is_control)||!source_path(&source.path){return Err("invalidSourceMetadata");}
            if source.owners.len()>MAX_OWNERS||source.enabled&&source.owners.is_empty(){return Err("invalidSourceOwners");}
            let mut owners=BTreeSet::new();
            for owner in &source.owners{
                if !pets.pets.contains_key(&owner.pet_id)||!crate::assistant_memory::valid_id(&owner.conversation_id)||!owners.insert(owner){return Err("invalidSourceOwners");}
                // The current calendar service owns one selected source. Keep
                // other enrollments disabled rather than silently merging or
                // choosing a first source. Multi-source tools require their
                // own explicit selection contract.
                if source.enabled&&!selected.insert(owner){return Err("ambiguousSourceSelection");}
            }
        }
        Ok(())
    }
    pub(super) fn selected(&self,pet_id:&str,cid:&str)->Result<Option<&Source>,&'static str>{
        self.validate()?;
        if !crate::assistant_memory::valid_id(cid){return Err("invalidOwner");}
        Ok(self.sources.iter().find(|s|s.enabled&&s.owners.iter().any(|o|o.pet_id==pet_id&&o.conversation_id==cid)))
    }
    /// An edit starts from the exact document revision and generation returned
    /// by the reader. Source revisions are owned by this writer, not a model.
    pub(super) fn next(mut self,previous:&Self)->Result<Self,&'static str>{
        if self.generation!=previous.generation{return Err("registryConflict");}
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
        let mut v=fixture();v.sources[0].owners[0].pet_id="not-a-configured-pet".into();assert!(v.validate().is_err());
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
        let mut v=fixture();v.version=2;assert!(v.validate().is_err());
        let mut v=serde_json::to_value(fixture()).unwrap();v["sources"][0]["enabled"]=Value::String("true".into());assert!(serde_json::from_value::<Registry>(v).is_err());
    }
}
