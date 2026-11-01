//! Bounded planning data. Parsing a model proposal never authorizes an effect.
//! The catalog is a native versioned enum, not a list supplied by a caller/model.
use super::{admission::Budget, criteria, model};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Limits {
    pub max_rounds: u8,
    pub max_steps: u8,
    pub max_output_bytes: u32,
    pub no_progress_limit: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Catalog { SpotifyTransportV1 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Policy { pub schema_version: u32, pub catalog: Catalog, pub limits: Limits }
impl Policy {
    pub(super) fn native(limits: Limits, budget: &Budget) -> model::Result<Self> {
        let p=Self { schema_version:1, catalog:Catalog::SpotifyTransportV1, limits };
        if !p.valid_for(budget) { return Err(model::Error::InvalidInput); }
        Ok(p)
    }
    fn valid(&self)->bool {
        self.schema_version==1 && (1..=8).contains(&self.limits.max_rounds)
            && (1..=8).contains(&self.limits.max_steps)
            && (2048..=65536).contains(&self.limits.max_output_bytes)
            && (1..=self.limits.max_rounds).contains(&self.limits.no_progress_limit)
    }
    pub(super) fn valid_for(&self, budget:&Budget) -> bool {
        self.valid()
            && u16::from(self.limits.max_rounds)<=budget.max_model_calls
            && u16::from(self.limits.max_steps)<=budget.max_tool_calls
            && u64::from(self.limits.max_output_bytes)<=budget.max_output_bytes
            && budget.max_tokens.is_none() && budget.max_cost_micros.is_none()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag="kind", rename_all="camelCase", deny_unknown_fields)]
enum Output {
    Plan { description:String, steps:Vec<model::StepInput>, criteria:Vec<criteria::Input> },
    Blocked { reason:String },
}
/// Native validation result; deliberately not deserializable or an execution
/// scope. All criteria must be mapped; eventual exact human review remains due.
pub(super) enum Parsed { Plan(ValidatedPlan), Blocked(String) }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub(super) struct ValidatedPlan {
    pub description:String, pub steps:Vec<model::StepInput>, pub verification:criteria::Binding,
}
fn text(s:&str, max:usize)->bool {
    !s.trim().is_empty() && s.len()<=max
        && !s.chars().any(|c|c=='\0'||c=='\u{7f}'||c.is_control()&&!matches!(c,'\n'|'\r'|'\t'))
}
pub(super) fn parse(goal:&model::Record, policy:&Policy, raw:&str)->model::Result<Parsed> {
    // Bound before allocating nested model data. Unknown fields, duplicate
    // fields, executable expressions and markdown wrappers are rejected.
    if !policy.valid() || raw.len()>policy.limits.max_output_bytes as usize || raw.is_empty() {
        return Err(model::Error::LimitReached);
    }
    let output:Output=serde_json::from_str(raw).map_err(|_|model::Error::InvalidInput)?;
    match output {
        Output::Blocked {reason} => {
            if !text(&reason,4096) {return Err(model::Error::InvalidInput);}
            Ok(Parsed::Blocked(reason))
        }
        Output::Plan {description,steps,criteria} => {
            if !text(&description,2048) || steps.is_empty() || steps.len()>policy.limits.max_steps as usize
                || criteria.len()!=goal.spec.criteria.len() {return Err(model::Error::InvalidInput);}
            for step in &steps {
                if !text(&step.description,2048) || !text(&step.postcondition,2048) {return Err(model::Error::InvalidInput);}
                let op=step.operation.as_ref().ok_or(model::Error::InvalidInput)?;
                if op.operation_id!="spotify.control" || op.version!=1 || !op.valid() {return Err(model::Error::InvalidInput);}
                use crate::pet_operations::Music;
                let action:Music=serde_json::from_value(op.arguments.clone()).map_err(|_|model::Error::InvalidInput)?;
                if !matches!(action,Music::Play|Music::Pause|Music::SetVolume{..}) {return Err(model::Error::InvalidInput);}
            }
            let verification=criteria::Binding::bind(goal,&criteria)?;
            // bind rejects duplicate/out-of-range indexes and invalid predicates;
            // equal cardinality consequently covers every original criterion.
            Ok(Parsed::Plan(ValidatedPlan {description,steps,verification}))
        }
    }
}

#[cfg(target_os="linux")]
pub(super) fn model_input(goal:&model::Record,spec:&super::admission::AdmissionSpec,context:&super::planning_context::Input)->model::Result<String> {
    let policy=spec.planning_policy.as_ref().filter(|_|spec.binding_format()).ok_or(model::Error::Conflict)?;
    if !super::plans::matches(goal,spec) {return Err(model::Error::Conflict);}
    // This catalog is prompt data, never an HTTP tool definition. Native typed
    // parsing, captured targets and human exact recipe review still follow.
    let mut input=serde_json::json!({
        "instructions":"Return exactly one JSON object, without markdown. Produce bounded proposed work or a blocked reason. Never execute operations, approve permissions or claim any criterion is verified. Use every original criterion exactly once by its zero-based index. Use the native remaining maxStepsThisRound/output limits, and treat prior native observations as historical at their timestamps, not assumed current. A plan must use only the declared catalog and predicates; it cannot change the goal, model, context, budget or policy. The native system will choose and capture the target and request separate exact human approval before effects.",
        "intent":goal.spec.intent,"criteria":goal.spec.criteria,"seedProposals":super::plans::seed_snapshot(goal,spec),
        "policy":policy,"sharedBudget":spec.budget,"nativeContext":context,"effectsAuthorized":false,"criteriaVerified":false,
        "catalog":{
            "operationId":"spotify.control","version":1,
            "arguments":[{"action":"play"},{"action":"pause"},{"action":"set_volume","volume":"integer 0..100"}],
            "predicates":[{"kind":"spotifyPlayback","status":"playing | paused | stopped"},{"kind":"spotifyVolume","percent":"integer 0..100"}]
        },
        "output":{
            "plan":{"kind":"plan","description":"nonempty text <=2048 bytes","steps":[{"description":"nonempty text <=2048 bytes","postcondition":"nonempty text <=2048 bytes","operation":{"operationId":"spotify.control","version":1,"arguments":"one exact catalog object"}}],"criteria":[{"criterionIndex":"each original index exactly once","predicate":"one exact typed predicate"}]},
            "blocked":{"kind":"blocked","reason":"nonempty text <=4096 bytes"}
        }
    });
    if let Some(c)=spec.continuation.as_ref().filter(|c|c.uncertain_effects.is_some()){
        input["historicalUncertainty"]=serde_json::json!({"reviewedAt":c.reviewed_at,"effects":c.uncertain_effects,"originalCharge":c.charge,"instructions":"These historical outcomes remain unknown and charged. This is neither proof of success nor a replay permit. Propose only the declared fresh state assignments; each new target and recipe requires its own exact native approval."});
    }
    super::plans::encode_input(&input)
}
