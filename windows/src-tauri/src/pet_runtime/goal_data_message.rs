//! One bounded DATA message used by native review and the actual runtime body.
//! Notes-only retains its original bytes; an additional source is never system.
use super::model;
use serde_json::{Value,json};
pub(super)fn with_calendars(goal_id:&str,existing:Option<Value>,calendars:Option<Value>)->model::Result<Option<Value>>{
 let Some(calendars)=calendars else{return Ok(existing);};
 let data=json!({"schemaVersion":1,"kind":"explicitSelectedGoalData","goalId":goal_id,"untrustedData":true,"otherSelectedData":existing,"calendars":calendars,"authorizesActions":false});
 let message=json!({"role":"user","content":serde_json::to_string(&data).map_err(|_|model::Error::InvalidRecord)?});
 if serde_json::to_vec(&message).map_err(|_|model::Error::InvalidRecord)?.len()>crate::pet_memory::context_model::MAX_CONTEXT_BYTES{return Err(model::Error::LimitReached);}Ok(Some(message))
}
pub(super)fn compose(goal_id:&str,notes:Option<Value>,received:Option<Value>)->model::Result<Option<Value>>{
 let has_received=received.is_some();let message=match(notes,received){
  (None,None)=>return Ok(None),
  (Some(notes),None)=>notes,
  (None,Some(received))=>received,
  (Some(notes),Some(received))=>{
   if notes.get("role").and_then(Value::as_str)!=Some("user")||received.get("role").and_then(Value::as_str)!=Some("user"){return Err(model::Error::InvalidRecord);}
   let notes=notes.get("content").and_then(Value::as_str).ok_or(model::Error::InvalidRecord)?;
   let request=received.get("content").and_then(Value::as_str).ok_or(model::Error::InvalidRecord)?;
   let data=json!({"schemaVersion":1,"kind":"explicitSelectedGoalData","goalId":goal_id,"untrustedData":true,"notes":notes,"receivedRequest":request,"authorizesActions":false});
   json!({"role":"user","content":serde_json::to_string(&data).map_err(|_|model::Error::InvalidRecord)?})
  },
 };
 if message.get("role").and_then(Value::as_str)!=Some("user")||!message.get("content").is_some_and(Value::is_string)||serde_json::to_vec(&message).map_err(|_|model::Error::InvalidRecord)?.len()>if has_received{crate::pet_memory::context_model::MAX_CONTEXT_BYTES}else{crate::pet_memory::context_model::GOAL_INPUT_LIMIT}{return Err(model::Error::LimitReached);}
 Ok(Some(message))
}
