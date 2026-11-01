//! Reviewed historical uncertainty is data, never a repaired receipt or permit.
use super::{model,plans::OperationProposal};
use serde::{Deserialize,Serialize};
pub(super)const MAX_EFFECTS:usize=32;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Effect{
    pub decision_id:String,pub attempt_id:String,pub grant_id:String,
    pub plan_id:String,pub step_id:String,pub scope_hash:String,
    pub operation:OperationProposal,pub target_label:String,
}
pub(super)fn state_assignment(operation:&OperationProposal)->bool{
    operation.operation_id=="spotify.control"&&operation.version==1&&operation.valid()
        &&serde_json::from_value::<crate::pet_operations::Music>(operation.arguments.clone())
            .is_ok_and(|a|matches!(a,crate::pet_operations::Music::Play|crate::pet_operations::Music::Pause|crate::pet_operations::Music::SetVolume{..}))
}
impl Effect{
    pub(super)fn valid(&self)->bool{
        [&self.decision_id,&self.attempt_id,&self.grant_id].iter().all(|s|model::token(s))
            &&model::key(&self.plan_id)&&model::key(&self.step_id)&&super::authority::decode(&self.scope_hash).is_some()
            &&state_assignment(&self.operation)&&!self.target_label.trim().is_empty()&&self.target_label.len()<=512
            &&!self.target_label.chars().any(char::is_control)
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn uncertainty_does_not_make_relative_transport_launch_mail_or_invalid_inputs_repeatable(){
        for action in [serde_json::json!({"action":"next"}),serde_json::json!({"action":"previous"}),serde_json::json!({"action":"play_pause"}),serde_json::json!({"action":"set_volume","volume":101}),serde_json::json!({"action":"pause","volume":17})]{assert!(!state_assignment(&OperationProposal{operation_id:"spotify.control".into(),version:1,arguments:action}));}
        for (id,args)in [("desktop.launch",serde_json::json!({"name":"App"})),("mail.prepare",serde_json::json!({"recipient":"qa@example.invalid","subject":"QA","body":"QA"}))]{assert!(!state_assignment(&OperationProposal{operation_id:id.into(),version:1,arguments:args}));}
    }
}
