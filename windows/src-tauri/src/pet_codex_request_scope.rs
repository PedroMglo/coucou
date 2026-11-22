//! Schema-specific native-request correlation shared by both Codex transports.
//! A legacy callId is not a turn ID; MCP may have no correlated turn at all.
use serde_json::Value;

pub(crate) struct Scope<'a> {
    pub session: &'a str,
    pub turn: Option<&'a str>,
}
impl Scope<'_> {
    pub(crate) fn matches(&self, session: &str, turn: Option<&str>) -> bool {
        self.session == session && self.turn.is_none_or(|native| Some(native) == turn)
    }
}
fn key<'a>(params: &'a Value, name: &str) -> Option<&'a str> {
    params[name].as_str().filter(|s| !s.is_empty() && s.len() <= 512 && !s.chars().any(char::is_control))
}
pub(crate) fn supported(method: &str) -> bool {
    matches!(method,
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
        | "item/permissions/requestApproval" | "item/tool/requestUserInput"
        | "execCommandApproval" | "applyPatchApproval" | "mcpServer/elicitation/request" | "item/tool/call")
}
pub(crate) fn extract<'a>(method: &str, params: &'a Value) -> Option<Scope<'a>> {
    match method {
        "item/tool/call" => {
            key(params,"callId")?;key(params,"tool")?;params.get("arguments")?;
            if params.get("namespace").is_some_and(|v|!v.is_null()&&key(params,"namespace").is_none()){return None;}
            Some(Scope{session:key(params,"threadId")?,turn:Some(key(params,"turnId")?)})
        }
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
        | "item/permissions/requestApproval" | "item/tool/requestUserInput" => {
            Some(Scope { session: key(params, "threadId")?, turn: Some(key(params, "turnId")?) })
        }
        "execCommandApproval" | "applyPatchApproval" => {
            // These schemas contain conversationId and callId, without turnId.
            // Unexpected v2 correlation fields cannot supply a different owner.
            let session = key(params, "conversationId")?;
            key(params, "callId")?;
            if params.get("threadId").is_some_and(|v| v != session)
                || params.get("turnId").is_some_and(|v| !v.is_null()) { return None; }
            Some(Scope { session, turn: None })
        }
        "mcpServer/elicitation/request" => {
            let session = key(params, "threadId")?;
            key(params, "serverName")?;
            let turn = match params.get("turnId") {
                None | Some(Value::Null) => None,
                Some(_) => Some(key(params, "turnId")?),
            };
            Some(Scope { session, turn })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_call_ids_never_become_turn_or_request_ids() {
        for method in ["execCommandApproval", "applyPatchApproval"] {
            let p = json!({"conversationId":"session", "callId":"other-turn"});
            let s = extract(method, &p).unwrap();
            assert!(s.turn.is_none());
            assert!(s.matches("session", Some("active-turn")));
            assert!(!s.matches("other-session", Some("other-turn")));
        }
    }
    #[test]
    fn nullable_mcp_correlation_never_relaxes_owner_or_explicit_turn() {
        let mut p = json!({"threadId":"session", "serverName":"fixture"});
        assert!(extract("mcpServer/elicitation/request", &p).unwrap().turn.is_none());
        p["turnId"] = Value::Null;
        assert!(extract("mcpServer/elicitation/request", &p).unwrap().matches("session", Some("turn")));
        p["turnId"] = json!("other-turn");
        assert!(!extract("mcpServer/elicitation/request", &p).unwrap().matches("session", Some("turn")));
        for bad in [json!(""), json!(false), json!(7), json!(["turn"])] {
            p["turnId"] = bad;
            assert!(extract("mcpServer/elicitation/request", &p).is_none());
        }
    }
    #[test]
    fn modern_scope_remains_strict_and_cannot_borrow_legacy_fields() {
        for method in ["item/commandExecution/requestApproval", "item/fileChange/requestApproval",
            "item/permissions/requestApproval", "item/tool/requestUserInput"] {
            assert!(supported(method));
            for p in [json!({"threadId":"session"}), json!({"threadId":"session", "turnId":null}),
                json!({"conversationId":"session", "callId":"turn"})] {
                assert!(extract(method, &p).is_none());
            }
        }
    }
    #[test]
    fn missing_or_conflicting_schema_fields_do_not_create_a_scope() {
        for p in [json!({"conversationId":"session"}),
            json!({"conversationId":"session", "callId":"call", "threadId":"foreign"}),
            json!({"conversationId":"session", "callId":"call", "turnId":"invented"})] {
            assert!(extract("execCommandApproval", &p).is_none());
        }
        assert!(extract("mcpServer/elicitation/request", &json!({"threadId":"session"})).is_none());
        assert!(!supported("future/request"));
        assert!(extract("future/request", &json!({"threadId":"session", "turnId":"turn"})).is_none());
    }
    #[test]fn dynamic_tools_keep_call_and_rpc_correlation_separate_and_require_real_fields(){
        let mut p=json!({"threadId":"session","turnId":"turn","callId":"call","tool":"music","arguments":{"action":"play"}});let s=extract("item/tool/call",&p).unwrap();assert!(s.matches("session",Some("turn")));assert!(!s.matches("session",Some("call")));
        for field in["callId","tool","arguments","threadId","turnId"]{let mut bad=p.clone();bad.as_object_mut().unwrap().remove(field);assert!(extract("item/tool/call",&bad).is_none());}
        p["namespace"]=json!(false);assert!(extract("item/tool/call",&p).is_none());p["namespace"]=json!("captured-namespace");assert!(extract("item/tool/call",&p).is_some());
    }
}
