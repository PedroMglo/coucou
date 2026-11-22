//! Private generated native System owner; zero inference or user device IO.
use crate::pet_runtime::{admission_tests::Fixture,native_system_admission_tests as original,authority::Authority,lease_controller::{Controller,Owned,Validity},model::Record,physical_target::Target};
use crate::{chat::Chat,pet_config::Configuration};
use std::sync::{Arc,Weak};
pub(super)use crate::pet_runtime::lease_controller::native_system_sound_task_tests::{Audio,serial};
pub(super)fn clock()->u64{crate::assistant_memory::now()}
pub(super)struct Case{pub f:Fixture,pub goal:Record,pub owner:Owned,pub config:Configuration,_chat:Chat,authority:Authority,before:Vec<u8>}
impl Case{
 pub fn new(audio:&Audio)->Self{let f=Fixture::new();let a=Authority::ensure(&f.path.join("authority")).unwrap();let g=original::draft(&f,audio,"ollama");let c=original::config(audio,&g);original::save(&f,&c);let review=original::review(&f,audio,&g,&c);let approved=review.approve(&g,original::evidence(&g,&c,clock()),clock()).unwrap();let cert=a.certify(&approved,&g,clock()).unwrap();let q=f.store().enqueue_certified(&cert,clock()).unwrap().record;let chat=Chat::persistent(f.path.join("main.json"));let pending=Controller::new().unwrap().prepare(&q,&f.path.join("authority"),&f.path.join("contexts"),&chat,&original::evidence(&q,&c,clock()),"cache-fixture-claim".into(),clock(),Arc::new(|_,_,_|Validity::Current)).unwrap();let(receipt,owner)=pending.claim(&f.store(),&a,&original::evidence(&q,&c,clock()),clock()).unwrap();let goal=receipt.record;let before=std::fs::read(f.path.join("authority").join(format!("{}.json",goal.admissions[0].spec.decision_id))).unwrap();Self{f,goal,owner:owner.unwrap(),config:c,_chat:chat,authority:a,before}}
 pub fn target(&self,audio:&Audio)->(Target,Weak<dyn Send+Sync>){let t=audio.target(&self.f.path.join("system.json"),&self.goal.spec.owner.pet_id,false);let weak=t.fixture_sound_lifetime();(t,weak)}
 pub fn unchanged(&self){assert_eq!(std::fs::read(self.f.path.join("authority").join(format!("{}.json",self.goal.admissions[0].spec.decision_id))).unwrap(),self.before);assert_eq!(std::fs::read(self.f.path.join("system.json")).unwrap(),serde_json::to_vec(&self.config).unwrap());self.authority.owner_observation(&self.goal.admissions[0].spec,self.goal.admissions[0].proof.as_ref().unwrap()).unwrap();}
}
