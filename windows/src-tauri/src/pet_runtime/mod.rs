//! Durable goals. Reading/opening this repository never dispatches work.
pub(crate)mod rules;
pub(crate)mod messages;
pub(crate)mod delegation;
mod funding;
mod peer_feedback;
mod peer_round_data;
mod peer_round_feedback;
mod peer_round_sources;
pub(crate)mod peer_round_history;
#[cfg(target_os="linux")]mod peer_round_review;
mod peer_feedback_api;
pub use peer_feedback_api::*;
mod delegation_api;
pub use delegation_api::*;
mod delegation_branch_api;
pub use delegation_branch_api::*;
mod message_retirement;
mod received_request;
#[cfg(target_os="linux")]mod received_result_data;
mod received_work;
pub(crate)mod work_report;
pub(crate)mod work_disclosure;
mod work_report_api;
pub use work_report_api::*;
mod received_work_api;
pub use received_work_api::*;
pub(crate)mod received_context;
#[cfg(target_os="linux")]mod goal_data_message;
mod received_request_api;
pub use received_request_api::*;
#[cfg(target_os="linux")]mod goal_data_publication;
mod message_retirement_api;
pub use message_retirement_api::*;
mod message_api;
pub use message_api::*;
mod model;
// Internal typed identity for source capture; this exports no authority or IPC.
pub(crate)use model::{Record as GoalRecord,Context as GoalContext,State as GoalState,goal_id as valid_goal_id};
mod plans;
mod planning;
mod planned_peer;
mod planned_peer_data;
mod generated_plans;
#[cfg(target_os="linux")]mod plan_publication;
mod planning_journal;
mod planning_feedback;
#[cfg(target_os="linux")]mod planning_context;
#[cfg(target_os="linux")]mod planning_output;
#[cfg(target_os="linux")]pub(crate)mod physical_target;
mod criteria;
mod native_verification;
mod native_drain;
mod native_plan_revisions;
#[cfg(target_os="linux")]mod native_plan_review;
#[cfg(target_os="linux")]mod native_plan_inventory;
mod native_plan_revision_api;
pub use native_plan_revision_api::*;
mod native_completion_api;
pub use native_completion_api::*;
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
mod deterministic;
mod native_system_plan;
mod native_peer_plan;
#[cfg(target_os="linux")]pub(in crate::pet_runtime)use lease_controller::peer_plan_review;
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
pub(crate)mod derived_result_plan;
mod disclosure_consent;
pub(crate)mod result_disclosure_policy;
#[cfg(target_os="linux")]pub(crate)mod result_disclosure;
#[cfg(target_os="linux")]mod result_delivery_data;
mod result_api;
pub use result_api::*;
#[cfg(target_os="linux")]mod native_validity;
#[cfg(target_os="linux")]mod lease_controller;
mod owner_resumption;
#[cfg(target_os="linux")]pub(crate)use lease_controller::DrainProof as NativeDrainProof;
#[cfg(target_os="linux")]pub(crate)use lease_controller::resource_wait_controller::YieldProof as NativeYieldProof;
mod driver_api;
pub use driver_api::*;
mod lease_api;
pub use lease_api::*;
#[cfg(target_os="linux")]pub(crate)fn start(app:tauri::AppHandle){lease_api::start_runtime(app);}
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
    fn stop_peer_owner_handoff(&self,_path:&std::path::Path,_id:&str,_pet:&str,_revision:&str,_request:&str,_action:&model::Action,_now:u64)->model::Result<Option<model::Receipt>>{Ok(None)}
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

#[cfg(all(test,target_os="linux"))]mod deterministic_runtime_tests;
#[cfg(all(test,target_os="linux"))]mod native_system_admission_tests;
#[cfg(all(test,target_os="linux"))]mod system_orphan_retirement_tests;
#[cfg(all(test,target_os="linux"))]mod system_preservation_tests;

/// Passive identity support for the bounded native catalog; no inference or grant.
pub(crate)fn deterministic_identity(d:&crate::pet_capabilities::Descriptor)->bool{d.schema_version==1&&model::Owner::from_runtime_descriptor(d).is_ok()}

#[cfg(all(test,target_os="linux"))]#[path="native_completion_tests.rs"]mod native_completion_tests;
#[cfg(all(test,target_os="linux"))]mod native_continuation_tests;
#[cfg(all(test,target_os="linux"))]mod native_drain_tests;

#[cfg(all(test,target_os="linux"))]mod native_plan_revision_tests;

#[cfg(all(test,target_os="linux"))]#[path="planned_peer_tests.rs"]mod planned_peer_tests;

#[cfg(all(test,target_os="linux"))]pub(crate)mod system_effect_crash_tests;

#[cfg(all(test,target_os="linux"))]mod health_runtime_tests;
#[cfg(all(test,target_os="linux"))]mod inventory_runtime_tests;
