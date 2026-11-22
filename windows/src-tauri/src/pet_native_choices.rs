//! Native choices are captured by Rust for one live source. The UI only selects
//! a local ID; it cannot submit a native response or broaden a default decision.
use serde_json::{json, Value};
use crate::pet_native_decisions::Ticket;

struct Choice {
    id: String,
    label: String,
    scope: &'static str,
    allow: bool,
    default: Option<bool>,
    response: Value,
    detail: Option<String>,
}
pub(crate) struct Choices {
    source: Ticket,
    provenance: &'static str,
    entries: Vec<Choice>,
}
fn text(value: &Value, limit: usize) -> Option<&str> {
    value.as_str().filter(|s| !s.is_empty() && s.len() <= limit && !s.chars().any(char::is_control))
}
fn choice(label: &str, scope: &'static str, allow: bool, default: Option<bool>, response: Value, detail: Option<String>) -> Choice {
    Choice { id: String::new(), label: label.into(), scope, allow, default, response, detail }
}
fn exact_keys(value: &Value, allowed: &[&str]) -> bool {
    value.as_object().is_some_and(|v| v.len() == allowed.len() && v.keys().all(|k| allowed.contains(&k.as_str())))
}
fn grant_root(params: &Value) -> Result<Option<&str>, String> {
    params.get("grantRoot").filter(|v| !v.is_null()).map(|v|
        text(v, 4096).filter(|p| std::path::Path::new(p).is_absolute())
            .ok_or_else(|| "A raiz de escrita pedida pelo agente é inválida.".to_string())
    ).transpose()
}
fn root_scope(c: &mut Choice, root: &str) -> Result<(), String> {
    if c.allow {
        c.label = if c.default == Some(true) { "Autorizar o pedido de escrita na raiz" }
            else { "Autorizar escrita na raiz nesta sessão" }.into();
        c.scope = "session";
        c.default = None;
        c.detail = Some(format!("Raiz pedida: {}\nO agente pede escrita sob esta raiz até ao fim da sessão. O efeito efetivo depende do agente nativo.",
            serde_json::to_string(root).map_err(|_| "Raiz de escrita inválida.")?));
    }
    Ok(())
}
fn codex_decision(value: &Value, command: bool) -> Option<Choice> {
    let response = json!({"decision":value});
    match value.as_str() {
        Some("accept") => return Some(choice("Permitir uma vez", "once", true, Some(true), response, None)),
        Some("acceptForSession") => return Some(choice(if command {"Permitir nesta sessão"} else {"Permitir estes ficheiros nesta sessão"}, "session", true, None, response, None)),
        Some("decline") => return Some(choice("Recusar", "once", false, Some(false), response, None)),
        Some("cancel") => return Some(choice("Recusar e interromper o turno", "turnCancellation", false, None, response, None)),
        Some(_) => return None,
        None => {},
    }
    if !command { return None; }
    if exact_keys(value, &["acceptWithExecpolicyAmendment"]) {
        let body = &value["acceptWithExecpolicyAmendment"];
        if !exact_keys(body, &["execpolicy_amendment"]) { return None; }
        let args = body["execpolicy_amendment"].as_array()?;
        if args.len() > 128 || args.iter().any(|v| v.as_str().is_none_or(|s|s.len()>8192 || s.contains('\0'))) { return None; }
        return Some(choice("Permitir e guardar a regra de comandos", "policy", true, None, response, Some(serde_json::to_string_pretty(value).ok()?)));
    }
    if exact_keys(value, &["applyNetworkPolicyAmendment"]) {
        let body = &value["applyNetworkPolicyAmendment"];
        if !exact_keys(body, &["network_policy_amendment"]) { return None; }
        let rule = &body["network_policy_amendment"];
        if !exact_keys(rule, &["host", "action"]) || text(&rule["host"], 1024).is_none() { return None; }
        let allow = match rule["action"].as_str()? { "allow"=>true, "deny"=>false, _=>return None };
        return Some(choice(if allow {"Permitir e guardar a regra de rede"} else {"Bloquear e guardar a regra de rede"}, "policy", allow, None, response, Some(serde_json::to_string_pretty(value).ok()?)));
    }
    None
}
fn collect(adapter: &str, method: &str, params: &Value) -> Result<Option<(&'static str, Vec<Choice>)>, String> {
    let invalid = || "As escolhas nativas deste pedido não são suportadas.".to_string();
    if serde_json::to_vec(params).map_err(|_|invalid())?.len() > 256*1024 { return Err(invalid()); }
    if adapter == "codex" && matches!(method, "execCommandApproval" | "applyPatchApproval") {
        // The actual legacy ReviewDecision schema differs from the v2 enum.
        // No policy amendment is invented from command text or unrelated fields.
        let mut rows = vec![
            choice("Permitir uma vez", "once", true, Some(true), json!({"decision":"approved"}), None),
            choice("Permitir nesta sessão", "session", true, None, json!({"decision":"approved_for_session"}), None),
            choice("Recusar", "once", false, Some(false), json!({"decision":{"denied":{"rejection":"The request was not approved."}}}), None),
            choice("Recusar e aguardar nova instrução", "turnCancellation", false, None, json!({"decision":"abort"}), None),
        ];
        if method == "applyPatchApproval" {
            if let Some(root) = grant_root(params)? {
                for c in &mut rows { root_scope(c, root)?; }
            }
        }
        return Ok(Some(("nativeLegacyProtocol", rows)));
    }
    if adapter == "codex" && matches!(method,"item/commandExecution/requestApproval"|"item/fileChange/requestApproval") {
        let command = method == "item/commandExecution/requestApproval";
        let (provenance, values) = match params.get("availableDecisions") {
            None|Some(Value::Null) => {
                // The known native v2 fallback, explicitly distinguished from
                // an authoritative per-request server list.
                let mut values = vec![json!("accept"),json!("acceptForSession"),json!("decline"),json!("cancel")];
                if command {
                    if let Some(proposal)=params.get("proposedExecpolicyAmendment").filter(|v|!v.is_null()) {
                        values.push(json!({"acceptWithExecpolicyAmendment":{"execpolicy_amendment":proposal}}));
                    }
                    if let Some(proposals)=params.get("proposedNetworkPolicyAmendments").filter(|v|!v.is_null()) {
                        let proposals=proposals.as_array().filter(|v|v.len()<=32).ok_or_else(invalid)?;
                        for proposal in proposals { values.push(json!({"applyNetworkPolicyAmendment":{"network_policy_amendment":proposal}})); }
                    }
                }
                ("protocolFallback",values)
            },
            Some(v) => ("nativeAdvertised",v.as_array().filter(|v|v.len()<=32).ok_or_else(invalid)?.clone()),
        };
        let root=if !command {grant_root(params)?}else{None};
        let mut rows:Vec<Choice>=vec![];
        for value in values {
            if let Some(mut c)=codex_decision(&value,command) {
                if let Some(root)=root.filter(|_|c.allow){
                    // Native grantRoot asks for session-scoped writes. Even an
                    // `accept` response cannot honestly be labelled once-only.
                    root_scope(&mut c,root)?;
                }
                if !rows.iter().any(|old|old.response==c.response) {rows.push(c);}
            }
        }
        if rows.is_empty() {return Err(invalid());}
        return Ok(Some((provenance,rows)));
    }
    if adapter == "codex" && method == "item/permissions/requestApproval" {
        let permissions=params.get("permissions").filter(|v|v.is_object()).ok_or_else(invalid)?;
        return Ok(Some(("nativePermissionProtocol",vec![
            choice("Permitir neste turno", "turn", true, Some(true), json!({"permissions":permissions,"scope":"turn"}),None),
            choice("Permitir nesta sessão", "session", true, None, json!({"permissions":permissions,"scope":"session"}),None),
            choice("Recusar", "once", false, Some(false),json!({"permissions":{},"scope":"turn"}),None),
        ])));
    }
    if adapter == "copilot" && method == "session/request_permission" {
        let options=params["options"].as_array().filter(|v|v.len()<=32).ok_or_else(invalid)?;
        let mut seen=std::collections::HashSet::new();let mut rows=vec![];
        for option in options {
            let id=text(&option["optionId"],512).ok_or_else(invalid)?;
            if !seen.insert(id) {return Err(invalid());}
            let (label,scope,allow,default)=match option["kind"].as_str() {
                Some("allow_once")=>("Permitir uma vez","once",true,Some(true)),
                Some("reject_once")=>("Recusar","once",false,Some(false)),
                Some("allow_always")=>("Permitir e memorizar no agente","nativeRemembered",true,None),
                Some("reject_always")=>("Recusar e memorizar no agente","nativeRemembered",false,None),
                _=>continue, // Unknown native extensions remain with the agent.
            };
            let detail=option.get("name").filter(|v|!v.is_null()).map(|v|text(v,1024).map(str::to_string).ok_or_else(invalid)).transpose()?;
            rows.push(choice(label,scope,allow,default,json!({"outcome":{"outcome":"selected","optionId":id}}),detail));
        }
        if rows.is_empty() {return Err(invalid());}
        return Ok(Some(("nativeOptions",rows)));
    }
    Ok(None)
}
impl Choices {
    pub(crate) fn capture(source:&Ticket,request_id:&str,params:&Value)->Result<Option<Self>,String>{
        if !source.current(){return Err("O pedido nativo já terminou.".into());}
        let Some((provenance,mut entries))=collect(source.identity().adapter,&source.identity().method,params)?else{return Ok(None);};
        for (index,c) in entries.iter_mut().enumerate(){c.id=format!("{request_id}-choice-{index}");}
        Ok(Some(Self{source:source.clone(),provenance,entries}))
    }
    pub(crate) fn matches(&self,source:&Ticket)->bool{self.source.same_capture(source)}
    pub(crate) fn provenance(&self)->&str{self.provenance}
    pub(crate) fn public(&self)->Value{json!(self.entries.iter().map(|c|json!({"id":c.id,"label":c.label,"scope":c.scope,"allow":c.allow,"detail":c.detail})).collect::<Vec<_>>())}
    pub(crate) fn resolve(&self,answer:&Value)->Result<Value,String>{
        if !self.source.current(){return Err("O pedido nativo já terminou.".into());}
        let allow=answer["allow"].as_bool().ok_or("Decisão nativa inválida.")?;
        let selected=if let Some(id)=answer.get("nativeChoice") {
            let id=id.as_str().ok_or("Identificador da escolha nativa inválido.")?;
            self.entries.iter().find(|c|c.id==id&&c.allow==allow)
        }else{self.entries.iter().find(|c|c.default==Some(allow))};
        selected.map(|c|c.response.clone()).ok_or_else(||"Escolhe uma das opções nativas apresentadas; o âmbito não pode ser alargado.".into())
    }
}
fn response(adapter:&str,method:&str,params:&Value,answer:&Value)->Option<Value>{
    let (_,entries)=collect(adapter,method,params).ok()??;
    let allow=answer["allow"].as_bool()?;
    if let Some(value)=answer.get("nativeResponse") {
        entries.into_iter().find(|c|c.allow==allow&&c.response==*value).map(|c|c.response)
    }else{entries.into_iter().find(|c|c.default==Some(allow)).map(|c|c.response)}
}
pub(crate)fn codex_response(method:&str,params:&Value,answer:Option<&Value>)->Option<Value>{
    response("codex",method,params,answer.unwrap_or(&json!({"allow":false})))
}
pub(crate)fn copilot_response(params:&Value,answer:Option<&Value>)->Value{
    answer.and_then(|a|response("copilot","session/request_permission",params,a)).unwrap_or_else(||json!({"outcome":{"outcome":"cancelled"}}))
}

/// Exact non-persistent refusals. `allow:false` alone is insufficient: native
/// choices may still persist a network rule or a remembered reject policy.
pub(crate)fn pure_refusal(adapter:&str,method:&str,params:&Value,result:&Value)->bool{
    if adapter=="codex"&&codex_fixed_refusal(method,result){return true;}
    if adapter=="copilot"&&method=="session/request_permission"&&*result==json!({"outcome":{"outcome":"cancelled"}}){return true;}
    collect(adapter,method,params).ok().flatten().is_some_and(|(_,entries)|entries.into_iter().any(|c|!c.allow&&matches!(c.scope,"once"|"turnCancellation")&&c.response==*result))
}
/// Control-socket replies retain only the native method/hash, not a second
/// copy of private arguments. Only these fixed protocol refusal forms qualify;
/// the response must already have passed the original request's choice broker.
pub(crate)fn codex_fixed_refusal(method:&str,result:&Value)->bool{
    match method{
        "execCommandApproval"|"applyPatchApproval"=>*result==json!({"decision":"abort"})||*result==json!({"decision":{"denied":{"rejection":"The request was not approved."}}}),
        "item/commandExecution/requestApproval"|"item/fileChange/requestApproval"=>*result==json!({"decision":"decline"})||*result==json!({"decision":"cancel"}),
        "item/permissions/requestApproval"=>*result==json!({"permissions":{},"scope":"turn"}),
        "mcpServer/elicitation/request"=>*result==json!({"action":"decline","content":null,"_meta":null}),
        "item/tool/requestUserInput"=>exact_keys(result,&["answers"])&&result["answers"].as_object().is_some_and(|a|a.len()<=64&&a.iter().all(|(id,v)|!id.is_empty()&&id.len()<=256&&!id.chars().any(char::is_control)&&*v==json!({"answers":[]}))),
        _=>false,
    }
}

#[cfg(test)]
#[path="pet_native_choices_tests.rs"]mod tests;
