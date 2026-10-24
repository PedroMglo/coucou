//! Durable goals. Reading/opening this repository never dispatches work.
mod model;
mod plans;
mod criteria;
#[cfg(target_os="linux")]mod recipes;
#[cfg(target_os="linux")]mod steps;
mod step_api;
pub use step_api::*;
#[cfg(target_os="linux")]pub(crate) mod recipe_decisions;
mod readiness_api;
pub use readiness_api::*;
mod recipe_api;
pub use recipe_api::*;
mod admission;
mod authority;
mod leases;
mod effect_budget;
#[cfg(target_os="linux")]mod results;
mod result_api;
pub use result_api::*;
#[cfg(target_os="linux")]mod native_validity;
#[cfg(target_os="linux")]mod lease_controller;
mod lease_api;
pub use lease_api::*;
#[cfg(target_os="linux")]mod authority_store;
mod admission_review;
pub use admission_review::*;
#[cfg(target_os="linux")]mod store;
#[cfg(target_os="linux")]pub(crate) mod context;
mod adapter;
pub use adapter::*;
/// Backend-neutral transaction boundary. Implementations must preserve CAS,
/// idempotency and uncertain-publication receipts; this trait never dispatches.
pub(crate) trait Repository{
    fn read(&self,id:&str,pet_id:&str)->model::Result<model::Record>;
    fn create(&self,spec:model::Spec,now:u64)->model::Result<model::Receipt>;
    fn mutate(&self,id:&str,pet_id:&str,revision:&str,request:String,action:model::Action,now:u64)->model::Result<model::Receipt>;
    fn list(&self,pet_id:Option<&str>)->model::Result<model::Listing>;
}
#[cfg(all(test,target_os="linux"))]mod tests;

#[cfg(all(test,target_os="linux"))]mod context_tests;
#[cfg(all(test,target_os="linux"))]mod admission_tests;
#[cfg(all(test,target_os="linux"))]mod authority_tests;
#[cfg(all(test,target_os="linux"))]mod lease_tests;

#[cfg(all(test,target_os="linux"))]mod effect_budget_tests;
#[cfg(all(test,target_os="linux"))]mod plans_tests;

#[cfg(all(test,target_os="linux"))]mod owner_tests;
