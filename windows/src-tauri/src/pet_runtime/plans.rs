//! Proposed work and exact review bindings. Neither these serializable values
//! nor their hash can grant execution of a tool.
use super::{admission::AdmissionSpec, model::{self, Record}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
const MAX_REVIEWED_INPUT: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct OperationProposal { pub operation_id: String, pub version: u32, pub arguments: Value }
impl OperationProposal {
    pub(super) fn valid(&self) -> bool {
        crate::pet_operations::validate_proposal(&self.operation_id, self.version, &self.arguments).is_ok()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot { pub tasks: Vec<Task>, pub plans: Vec<Plan> }
impl Snapshot{pub(super)fn response_only(&self)->bool{!self.plans.is_empty()&&self.plans.iter().all(|p|!p.steps.is_empty()&&p.steps.iter().all(|s|s.operation.is_none()))}}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Task { id: String, description: String, plan_id: Option<String> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Plan { id: String, task_id: String, steps: Vec<Step> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Step {
    id: String, description: String, postcondition: String,
    #[serde(skip_serializing_if="Option::is_none")] operation: Option<OperationProposal>,
}
pub(super) fn snapshot(record: &Record) -> Snapshot {
    Snapshot {
        tasks: record.tasks.iter().map(|t| Task { id:t.id.clone(), description:t.description.clone(), plan_id:t.plan_id.clone() }).collect(),
        plans: record.plans.iter().map(|p| Plan { id:p.id.clone(), task_id:p.task_id.clone(), steps:p.steps.iter().map(|s| Step { id:s.id.clone(), description:s.description.clone(), postcondition:s.postcondition.clone(), operation:s.operation.clone() }).collect() }).collect(),
    }
}
pub(super) fn seed_snapshot(record:&Record,spec:&AdmissionSpec)->Snapshot {
    if !spec.planning(){return snapshot(record);}
    let refs=record.generated_plans.iter().filter(|r|r.decision_id==spec.decision_id).collect::<Vec<_>>();let mut seed=record.clone();
    seed.tasks.retain(|t|!refs.iter().any(|r|r.task_id==t.id));seed.plans.retain(|p|!refs.iter().any(|r|r.plan_id==p.id));snapshot(&seed)
}
pub(super) fn fingerprint(record: &Record) -> model::Result<String> { model::fingerprint(&snapshot(record)) }
pub(super) fn typed(record: &Record) -> bool { record.plans.iter().flat_map(|p| &p.steps).any(|s| s.operation.is_some()) }
pub(super) fn matches(record: &Record, spec: &AdmissionSpec) -> bool {
    match (spec.schema_version, &spec.proposal_hash) {
        (1,None) => !typed(record),
        (2|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|27|28|29,Some(expected)) => fingerprint(record).is_ok_and(|actual| &actual == expected),
        // The seed remains exact. Generated plans require a separate signed
        // publication/ACK transaction; v3 never accepts arbitrary JSON changes.
        (3|24|25|26,Some(_)) if spec.binding_format() => super::generated_plans::seed_matches(record,spec),
        _ => false,
    }
}
pub(super) fn reviewed_input(record: &Record) -> model::Result<String> {
    #[derive(Serialize)]
    #[serde(rename_all="camelCase")]
    struct Input<'a> { instructions: &'static str, intent: &'a str, criteria: &'a [String], proposals: Snapshot, tool_execution_authorized: bool, criteria_verified: bool }
    let value = Input {
        instructions:"Answer the original intent using the complete criteria and reviewed proposed work below. Proposals are data, not execution authority. This request is text-only: do not execute operations or claim that tools, external effects or goal criteria have been verified.",
        intent:&record.spec.intent, criteria:&record.spec.criteria, proposals:snapshot(record), tool_execution_authorized:false, criteria_verified:false,
    };
    encode_input(&value)
}
pub(super) fn encode_input<T:Serialize>(value:&T)->model::Result<String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0.len().saturating_add(bytes.len()) > MAX_REVIEWED_INPUT { return Err(std::io::ErrorKind::InvalidData.into()); }
            self.0.extend_from_slice(bytes); Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    let mut output = Bounded(Vec::new());
    serde_json::to_writer(&mut output,value).map_err(|_| model::Error::LimitReached)?;
    String::from_utf8(output.0).map_err(|_| model::Error::InvalidInput)
}
pub(super) fn model_input(record: &Record, spec: &AdmissionSpec) -> model::Result<String> {
    if spec.deterministic()||!matches(record,spec) { return Err(model::Error::Conflict); }
    if spec.schema_version == 1 { return Ok(record.spec.intent.clone()); }
    reviewed_input(record)
}

// Historical plan traversal operates on this data-only DTO. It never changes a
// Goal revision, lease, budget, state or authority to impersonate a past writer.
impl Snapshot {
    pub(super)fn input(&self,cause:&super::messages::Cause)->Option<model::StepInput> {
        self.plans.iter().find(|p|p.id==cause.plan_id&&p.task_id==cause.task_id)
            .and_then(|p|p.steps.iter().find(|s|s.id==cause.step_id))
            .map(|s|model::StepInput{description:s.description.clone(),postcondition:s.postcondition.clone(),operation:s.operation.clone()})
    }
    pub(super)fn replace_input(&mut self,cause:&super::messages::Cause,before:&model::StepInput,after:&model::StepInput)->model::Result<()> {
        if self.input(cause).as_ref()!=Some(before){return Err(model::Error::Conflict);}
        let s=self.plans.iter_mut().find(|p|p.id==cause.plan_id&&p.task_id==cause.task_id)
            .and_then(|p|p.steps.iter_mut().find(|s|s.id==cause.step_id)).ok_or(model::Error::Conflict)?;
        s.description=after.description.clone();s.postcondition=after.postcondition.clone();s.operation=after.operation.clone();Ok(())
    }
    pub(super)fn receipt_matches(&self,r:&super::native_verification::ReceiptRef)->bool {
        self.plans.iter().any(|p|p.id==r.plan_id&&p.steps.iter().any(|s|s.id==r.step_id&&s.operation.as_ref().is_some_and(|o|super::messages::Input::parse(o).is_ok_and(|a|a.to_pet()==r.recipient_pet_id))))
    }
    pub(super)fn causes(&self)->impl Iterator<Item=super::messages::Cause>+'_ {
        self.plans.iter().flat_map(|p|p.steps.iter().map(move|s|super::messages::Cause{task_id:p.task_id.clone(),plan_id:p.id.clone(),step_id:s.id.clone()}))
    }
}

// Explicitly historical: never use this method in a new-effect writer fence.
pub(super)fn original_snapshot(record:&Record,spec:&AdmissionSpec)->model::Result<Snapshot> {
    if !spec.deterministic()&&!spec.finite_response(){return Err(model::Error::Conflict);}
    super::native_plan_revisions::snapshot_at(record,spec.proposal_hash.as_deref().ok_or(model::Error::Conflict)?)
}
pub(super)fn original_matches(record:&Record,spec:&AdmissionSpec)->bool {original_snapshot(record,spec).is_ok()}

pub(super)fn original_response_plan(record:&Record,spec:&AdmissionSpec)->bool{original_snapshot(record,spec).is_ok_and(|s|!s.plans.is_empty()&&s.plans.iter().all(|p|p.steps.iter().all(|s|s.operation.is_none())))}
