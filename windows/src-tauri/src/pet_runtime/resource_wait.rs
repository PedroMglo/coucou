//! Signed observations of resource waits. These data never recreate an owner,
//! physical hold, approval or permission to start IO.
use super::{leases::Lease,model};
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Entry{
    pub id:String,pub recipe_id:String,pub goal_revision:String,pub goal_hash:String,
    pub generation:u64,pub yielded_at:u64,pub expires_at:u64,pub resumed_at:Option<u64>,
}
impl Entry{
    pub(super)fn valid(&self,lease:&Lease)->bool{
        model::token(&self.id)&&model::token(&self.recipe_id)&&model::token(&self.goal_revision)
            &&super::authority::decode(&self.goal_hash).is_some()&&self.generation>0&&self.generation<9
            &&self.yielded_at>=lease.claimed_at&&self.yielded_at<self.expires_at&&self.expires_at<=lease.deadline_at
            &&self.resumed_at.is_none_or(|n|n>=self.yielded_at&&n<self.expires_at)
    }
    pub(super)fn same_wait(&self,other:&Self)->bool{let mut copy=self.clone();copy.resumed_at=None;copy==*other}
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super)struct Ledger{pub schema_version:u32,pub entries:Vec<Entry>}
impl Ledger{
    pub(super)fn valid(&self,lease:&Lease,updated:u64)->bool{
        if self.schema_version!=1||self.entries.is_empty()||self.entries.len()>8{return false;}
        let mut previous=lease.claimed_at;
        for(i,e)in self.entries.iter().enumerate(){
            if !e.valid(lease)||e.generation!=i as u64+1||e.yielded_at<previous
                ||e.yielded_at>updated||e.resumed_at.is_some_and(|n|n>updated)
                ||self.entries[..i].iter().any(|old|old.id==e.id||old.recipe_id==e.recipe_id){return false;}
            if let Some(n)=e.resumed_at{previous=n;}else if i+1!=self.entries.len(){return false;}
        }true
    }
    pub(super)fn generation(&self)->u64{self.entries.last().map_or(1,|e|e.generation+u64::from(e.resumed_at.is_some()))}
    pub(super)fn waiting(&self)->bool{self.entries.last().is_some_and(|e|e.resumed_at.is_none())}
}
