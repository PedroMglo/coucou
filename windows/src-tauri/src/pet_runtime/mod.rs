//! Durable goals. Reading/opening this repository never dispatches work.
mod model;
mod plans;
mod planning;
mod generated_plans;
mod planning_journal;
mod planning_feedback;
#[cfg(target_os="linux")]mod planning_context;
#[cfg(target_os="linux")]mod planning_output;
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
mod resource_wait;
mod runtime_owner;
#[cfg(target_os="linux")]pub(crate)mod native_process_identity;
mod reconciliation;
mod history;
mod restored_future;
mod restored_future_api;
pub use restored_future_api::*;
mod continuation;
mod uncertain_effects;
mod retirement;
mod retirement_api;
pub use retirement_api::*;
#[cfg(target_os="linux")]mod retirement_controller;
mod recovery;
#[cfg(target_os="linux")]mod recovery_controller;
mod recovery_api;
pub use recovery_api::*;
mod reconciliation_api;
pub use reconciliation_api::*;
#[cfg(target_os="linux")]mod native_owner;
#[cfg(target_os="linux")]mod results;
mod result_api;
pub use result_api::*;
#[cfg(target_os="linux")]mod native_validity;
#[cfg(target_os="linux")]mod lease_controller;
#[cfg(target_os="linux")]pub(crate)use lease_controller::DrainProof as NativeDrainProof;
#[cfg(target_os="linux")]pub(crate)use lease_controller::resource_wait_controller::YieldProof as NativeYieldProof;
mod driver_api;
pub use driver_api::*;
mod lease_api;
pub use lease_api::*;
#[cfg(target_os="linux")]mod authority_store;
#[cfg(target_os="linux")]pub(crate)use authority_store::restoration;
mod admission_review;
pub use admission_review::*;
#[cfg(target_os="linux")]mod store;
#[cfg(target_os="linux")]pub(crate) mod context;
#[cfg(target_os="linux")]mod memory_review;
mod adapter;
pub use adapter::*;
/// Backend-neutral transaction boundary. Implementations must preserve CAS,
/// idempotency and uncertain-publication receipts; this trait never dispatches.
pub(crate) trait Repository{
    fn read(&self,id:&str,pet_id:&str)->model::Result<model::Record>;
    fn read_control(&self,id:&str,pet_id:&str)->model::Result<model::Record>{self.read(id,pet_id)}
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
#[cfg(all(test,target_os="linux"))]mod planning_tests;
#[cfg(all(test,target_os="linux"))]mod generated_plan_tests;
#[cfg(all(test,target_os="linux"))]mod generated_effect_tests;

#[cfg(all(test,target_os="linux"))]mod owner_tests;
#[cfg(all(test,target_os="linux"))]mod reconciliation_tests;
#[cfg(all(test,target_os="linux"))]mod recovery_tests;

#[cfg(all(test,target_os="linux"))]mod history_tests;

#[cfg(all(test,target_os="linux"))]mod continuation_tests;

#[cfg(all(test,target_os="linux"))]mod retirement_tests;
