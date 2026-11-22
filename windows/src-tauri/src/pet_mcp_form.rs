//! Bounded MCP form conversion. Rust keeps the original types/values and source;
//! the UI receives field/option IDs, without schema defaults or authentication.
use serde_json::{json, Map, Value};
use crate::pet_native_decisions::Ticket;
const INVALID: &str = "A resposta não corresponde aos campos, opções ou limites do formulário MCP.";
const UNSUPPORTED: &str = "Este formulário ou modo MCP precisa do cliente nativo.";
const MAX_FIELDS: usize = 32;
const MAX_TEXT: usize = 8192;

enum Kind {
    Text { min: u64, max: Option<u64>, format: Option<String>, secret: bool },
    Number { integer: bool, min: Option<f64>, max: Option<f64> },
    Select { options: Vec<(String, Value)>, multiple: bool, min: u64, max: Option<u64>, boolean: bool },
}
struct Field { name: String, id: String, question: String, required: bool, kind: Kind }
pub(crate) struct Form { source: Ticket, fields: Vec<Field>, detail: String }
fn object_keys(v: &Value, allowed: &[&str]) -> Result<(), String> {
    let o = v.as_object().ok_or(UNSUPPORTED)?;
    if o.keys().any(|k| !allowed.contains(&k.as_str())) { return Err(UNSUPPORTED.into()); }
    Ok(())
}
fn text(v: &Value, max: usize) -> Result<&str, String> {
    v.as_str().filter(|s| s.len() <= max && !s.contains('\0')).ok_or_else(|| UNSUPPORTED.into())
}
fn optional_text<'a>(v: &'a Value, key: &str, max: usize) -> Result<Option<&'a str>, String> {
    v.get(key).filter(|v| !v.is_null()).map(|v| text(v, max)).transpose()
}
fn count(v: &Value, key: &str) -> Result<Option<u64>, String> {
    v.get(key).filter(|v| !v.is_null()).map(|v| v.as_u64().ok_or_else(|| UNSUPPORTED.to_string())).transpose()
}
fn number(v: &Value, key: &str) -> Result<Option<f64>, String> {
    v.get(key).filter(|v| !v.is_null()).map(|v| v.as_f64().filter(|v| v.is_finite()).ok_or_else(|| UNSUPPORTED.to_string())).transpose()
}
fn options(schema: &Value, titled_key: &str) -> Result<Vec<(String, Value)>, String> {
    let values = schema.get(titled_key).or_else(|| schema.get("enum")).and_then(Value::as_array)
        .filter(|v| !v.is_empty() && v.len() <= 32).ok_or(UNSUPPORTED)?;
    let titled = schema.get(titled_key).is_some();
    let names = schema.get("enumNames").filter(|v| !v.is_null()).map(|v| v.as_array().filter(|a| a.len() == values.len()).ok_or(UNSUPPORTED)).transpose()?;
    if titled && (schema.get("enum").is_some() || names.is_some()) { return Err(UNSUPPORTED.into()); }
    let mut result = Vec::new();
    for (index, v) in values.iter().enumerate() {
        let (label, value) = if titled {
            object_keys(v, &["const", "title"])?;
            (text(&v["title"], 1024)?.to_string(), text(&v["const"], MAX_TEXT)?.to_string())
        } else {
            let value = text(v, MAX_TEXT)?.to_string();
            (names.map(|a| text(&a[index], 1024).map(str::to_string)).transpose()?.unwrap_or_else(|| value.clone()), value)
        };
        if result.iter().any(|(_, old)| old == &json!(value)) { return Err(UNSUPPORTED.into()); }
        result.push((label, json!(value)));
    }
    Ok(result)
}
fn kind(schema: &Value, extension: bool) -> Result<Kind, String> {
    let common = ["type", "title", "description", "default"];
    // Defaults are annotations only: they are not stored, displayed or selected.
    optional_text(schema, "title", 1024)?;
    optional_text(schema, "description", 4096)?;
    match schema["type"].as_str() {
        Some("boolean") => {
            object_keys(schema, &common)?;
            Ok(Kind::Select { options: vec![("Sim".into(), json!(true)), ("Não".into(), json!(false))], multiple: false, min: 1, max: Some(1), boolean: true })
        }
        Some("number" | "integer") => {
            object_keys(schema, &["type", "title", "description", "default", "minimum", "maximum"])?;
            let min = number(schema, "minimum")?; let max = number(schema, "maximum")?;
            if min.zip(max).is_some_and(|(a,b)| a > b) { return Err(UNSUPPORTED.into()); }
            Ok(Kind::Number { integer: schema["type"] == "integer", min, max })
        }
        Some("string") if schema.get("enum").is_some() || schema.get("oneOf").is_some() => {
            object_keys(schema, &["type", "title", "description", "default", "enum", "enumNames", "oneOf"])?;
            Ok(Kind::Select { options: options(schema, "oneOf")?, multiple: false, min: 1, max: Some(1), boolean: false })
        }
        Some("string") => {
            object_keys(schema, &["type", "title", "description", "default", "minLength", "maxLength", "format"])?;
            let min = count(schema, "minLength")?.unwrap_or(0); let max = count(schema, "maxLength")?;
            if min > MAX_TEXT as u64 || max.is_some_and(|v| v < min) { return Err(UNSUPPORTED.into()); }
            let format = optional_text(schema, "format", 64)?;
            if format.is_some_and(|v| !matches!(v,"email"|"uri"|"date"|"date-time") && !(extension && v == "password")) { return Err(UNSUPPORTED.into()); }
            Ok(Kind::Text { min, max, format: format.map(str::to_string), secret: format == Some("password") })
        }
        Some("array") => {
            object_keys(schema, &["type", "title", "description", "default", "items", "minItems", "maxItems"])?;
            let items = &schema["items"];
            let titled = items.get("anyOf").is_some();
            if titled { object_keys(items, &["anyOf"])?; }
            else { object_keys(items, &["type", "enum"])?; if items["type"] != "string" { return Err(UNSUPPORTED.into()); } }
            let values = options(items, "anyOf")?;
            let min = count(schema, "minItems")?.unwrap_or(0); let max = count(schema, "maxItems")?;
            if min > values.len() as u64 || max.is_some_and(|v| v < min) { return Err(UNSUPPORTED.into()); }
            Ok(Kind::Select { options: values, multiple: true, min, max, boolean: false })
        }
        _ => Err(UNSUPPORTED.into()),
    }
}
// RFC 5321 Mailbox: ASCII dot-string or quoted local part, DNS domain or
// IPv4/IPv6 address literal. Display-name and SMTPUTF8 are different formats.
fn smtp_ipv4(s: &str) -> Option<[u8; 4]> {
    let parts=s.split('.').collect::<Vec<_>>();if parts.len()!=4{return None;}let mut ip=[0;4];
    for (i,p) in parts.iter().enumerate(){if p.is_empty()||p.len()>3||!p.bytes().all(|b|b.is_ascii_digit()){return None;}ip[i]=p.parse().ok()?;}Some(ip)
}
fn email(s: &str) -> bool {
    if !s.is_ascii() || s.len() > 254 { return false; }
    let Some((local, domain)) = s.rsplit_once('@') else { return false; };
    if local.is_empty() || local.len() > 64 || domain.is_empty() || domain.len() > 255 { return false; }
    let local_valid = if local.starts_with('"') && local.ends_with('"') && local.len() >= 2 {
        let mut bytes = local.as_bytes()[1..local.len()-1].iter(); let mut valid = true;
        while let Some(&b) = bytes.next() {
            if b == b'\\' { if !bytes.next().is_some_and(|v| (32..=126).contains(v)) { valid = false; break; } }
            else if !(32..=126).contains(&b) || b == b'"' { valid = false; break; }
        }
        valid
    } else {
        local.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-/=?^_`{|}~".contains(&b)))
    };
    if !local_valid { return false; }
    if let Some(literal) = domain.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        if literal.get(..5).is_some_and(|v| v.eq_ignore_ascii_case("IPv6:")) {
            let ip=&literal[5..];if ip.parse::<std::net::Ipv6Addr>().is_ok(){return true;}
            // SMTP Snum permits leading zeros, including an embedded IPv4 tail.
            return ip.rsplit_once(':').is_some_and(|(prefix,tail)|smtp_ipv4(tail).is_some_and(|v|format!("{prefix}:{}.{}.{}.{}",v[0],v[1],v[2],v[3]).parse::<std::net::Ipv6Addr>().is_ok()));
        }
        if smtp_ipv4(literal).is_some(){return true;}
        return literal.split_once(':').is_some_and(|(tag,data)|!tag.is_empty()&&tag.as_bytes().last().is_some_and(|b|b.is_ascii_alphanumeric())
            &&tag.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')&&!data.is_empty()&&data.bytes().all(|b|(33..=90).contains(&b)||(94..=126).contains(&b)));
    }
    domain.split('.').all(|label| !label.is_empty() && label.len() <= 63
        && label.as_bytes()[0].is_ascii_alphanumeric() && label.as_bytes()[label.len()-1].is_ascii_alphanumeric()
        && label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
}
// RFC 3986 generic URI syntax. A URL parser's scheme-specific normalization
// would reject valid URI forms or silently repair malformed input.
fn uri_chars(s:&str,extra:&[u8])->bool{
    let b=s.as_bytes();let mut i=0;while i<b.len(){let c=b[i];
        if c==b'%' {if i+2>=b.len()||!b[i+1].is_ascii_hexdigit()||!b[i+2].is_ascii_hexdigit(){return false;}i+=3;continue;}
        if !(c.is_ascii_alphanumeric()||b"-._~!$&'()*+,;=".contains(&c)||extra.contains(&c)){return false;}i+=1;
    }true
}
fn uri_authority(authority:&str)->bool{
    let hostport=if let Some((user,host))=authority.rsplit_once('@'){if !uri_chars(user,b":"){return false;}host}else{authority};
    if let Some(literal)=hostport.strip_prefix('['){
        let Some((host,tail))=literal.split_once(']')else{return false;};
        if !tail.is_empty()&&!tail.strip_prefix(':').is_some_and(|p|p.bytes().all(|b|b.is_ascii_digit())){return false;}
        if host.parse::<std::net::Ipv6Addr>().is_ok(){return true;}
        return host.strip_prefix('v').or_else(||host.strip_prefix('V')).and_then(|v|v.split_once('.')).is_some_and(|(version,address)|!version.is_empty()&&version.bytes().all(|b|b.is_ascii_hexdigit())&&!address.is_empty()&&!address.contains('%')&&uri_chars(address,b":"));
    }
    let host=if let Some((host,port))=hostport.rsplit_once(':'){if !port.bytes().all(|b|b.is_ascii_digit()){return false;}host}else{hostport};
    uri_chars(host,b"")
}
fn uri(s:&str)->bool{
    let Some((scheme,rest))=s.split_once(':')else{return false;};
    if scheme.is_empty()||!scheme.as_bytes()[0].is_ascii_alphabetic()||!scheme.bytes().all(|b|b.is_ascii_alphanumeric()||b"+-.".contains(&b)){return false;}
    let before_fragment=if let Some((head,fragment))=rest.split_once('#'){if !uri_chars(fragment,b":@/?"){return false;}head}else{rest};
    let path=if let Some((head,query))=before_fragment.split_once('?'){if !uri_chars(query,b":@/?"){return false;}head}else{before_fragment};
    if let Some(rest)=path.strip_prefix("//"){
        let(authority,path)=rest.find('/').map(|i|(&rest[..i],&rest[i..])).unwrap_or((rest,""));
        uri_authority(authority)&&uri_chars(path,b":@/")
    }else{uri_chars(path,b":@/")}
}
fn format_valid(format: &str, raw: &str) -> bool {
    match format {
        "password" => true,
        "email" => email(raw),
        "date" => raw.len() == 10 && chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").is_ok_and(|d| d.format("%Y-%m-%d").to_string() == raw),
        "date-time" => raw.len() >= 20 && chrono::DateTime::parse_from_rfc3339(raw).is_ok(),
        "uri" => uri(raw),
        _ => false,
    }
}
impl Field {
    fn public(&self) -> Value {
        let (input_type, secret, options) = match &self.kind {
            Kind::Text { secret, .. } => ("text", *secret, Value::Null),
            Kind::Number { integer, .. } => (if *integer { "integer" } else { "number" }, false, Value::Null),
            Kind::Select { options, multiple, boolean, .. } => (if *multiple { "multi" } else if *boolean { "boolean" } else { "enum" }, false,
                json!(options.iter().enumerate().map(|(i,(label,_))| json!({"label":label,"value":format!("{}-option-{i}", self.id)})).collect::<Vec<_>>())),
        };
        let hint = match &self.kind {
            Kind::Text { min, max, format, .. } => format!("Texto. Comprimento mínimo: {min}.{}{}", max.map(|v|format!(" Máximo: {v}.")).unwrap_or_default(),format.as_ref().map(|f|format!(" Formato: {f}.")).unwrap_or_default()),
            Kind::Number { integer, min, max } => format!("{}.{}{}",if *integer {"Número inteiro"}else{"Número"},min.map(|v|format!(" Mínimo: {v}.")).unwrap_or_default(),max.map(|v|format!(" Máximo: {v}.")).unwrap_or_default()),
            Kind::Select { multiple: true, min, max, .. } => format!("Seleção múltipla. Mínimo: {min}.{}",max.map(|v|format!(" Máximo: {v}.")).unwrap_or_default()),
            Kind::Select { .. } => "Escolhe uma opção.".into(),
        };
        json!({"id":self.id,"question":self.question,"required":self.required,"inputType":input_type,"isSecret":secret,"options":options,"hint":hint})
    }
    fn value(&self, values: &Value) -> Result<Value, String> {
        let list = values.as_array().ok_or(INVALID)?;
        if list.len() > 32 { return Err(INVALID.into()); }
        let strings = list.iter().map(|v| v.as_str().filter(|v| v.len() <= MAX_TEXT && !v.contains('\0')).ok_or(INVALID)).collect::<Result<Vec<_>, _>>()?;
        match &self.kind {
            Kind::Text { min, max, format, .. } => {
                if strings.len() != 1 { return Err(INVALID.into()); }
                let raw = strings[0]; let len = raw.chars().count() as u64;
                if len < *min || max.is_some_and(|v| len > v) || format.as_ref().is_some_and(|f| !format_valid(f, raw)) { return Err(INVALID.into()); }
                Ok(json!(raw))
            }
            Kind::Number { integer, min, max } => {
                if strings.len() != 1 { return Err(INVALID.into()); }
                let v: Value = serde_json::from_str(strings[0]).map_err(|_| INVALID)?;
                let n = v.as_f64().filter(|n| n.is_finite()).ok_or(INVALID)?;
                if !v.is_number() || *integer && n.fract() != 0.0 || min.is_some_and(|v| n < v) || max.is_some_and(|v| n > v) { return Err(INVALID.into()); }
                Ok(v)
            }
            Kind::Select { options, multiple, min, max, .. } => {
                if !*multiple && strings.len() != 1 || strings.len() < *min as usize || max.is_some_and(|v| strings.len() as u64 > v) { return Err(INVALID.into()); }
                let mut seen = std::collections::HashSet::new(); let mut result = Vec::new();
                for s in strings {
                    if !seen.insert(s) { return Err(INVALID.into()); }
                    let index = options.iter().enumerate().find(|(i,_)| s == format!("{}-option-{i}", self.id)).map(|(i,_)|i).ok_or(INVALID)?;
                    result.push(options[index].1.clone());
                }
                if *multiple { Ok(json!(result)) } else { result.pop().ok_or_else(|| INVALID.into()) }
            }
        }
    }
}
fn compile(params: &Value, review_id: &str) -> Result<(Vec<Field>, String), String> {
        let extension = match params["mode"].as_str() { Some("form") => false, Some("openai/form" | "openaiForm") => true, _ => return Err(UNSUPPORTED.into()) };
        let schema = &params["requestedSchema"];
        object_keys(schema, &["$schema", "type", "properties", "required"])?;
        optional_text(schema, "$schema", 2048)?;
        if schema["type"] != "object" { return Err(UNSUPPORTED.into()); }
        let props = schema["properties"].as_object().filter(|p| p.len() <= MAX_FIELDS).ok_or(UNSUPPORTED)?;
        let required = schema.get("required").filter(|v| !v.is_null()).map(|v| v.as_array().ok_or(UNSUPPORTED)).transpose()?;
        let mut required_names = std::collections::HashSet::new();
        if let Some(required) = required { for v in required {
            let name = text(v, 256)?;
            if !props.contains_key(name) || !required_names.insert(name) { return Err(UNSUPPORTED.into()); }
        } }
        let mut fields = Vec::new();
        for (i, (name, schema)) in props.iter().enumerate() {
            if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) { return Err(UNSUPPORTED.into()); }
            let field_kind = kind(schema, extension)?;
            let title = optional_text(schema, "title", 1024)?.unwrap_or(name);
            let description = optional_text(schema, "description", 4096)?.unwrap_or("");
            fields.push(Field { name: name.clone(), id: format!("{review_id}-field-{i}"), question: if description.is_empty() { title.into() } else { format!("{title}\n{description}") }, required: required_names.contains(name.as_str()), kind: field_kind });
        }
        let server = text(&params["serverName"], 512)?; let message = text(&params["message"], 8192)?;
        Ok((fields, format!("MCP · {server}\n{message}")))
    }
impl Form {
    pub(crate) fn capture(source: &Ticket, review_id: &str, params: &Value) -> Result<Self, String> {
        if !source.current() || source.identity().adapter != "codex" || source.identity().method != "mcpServer/elicitation/request" { return Err(UNSUPPORTED.into()); }
        if review_id.is_empty() || review_id.len() > 160 || review_id.chars().any(char::is_control) { return Err(UNSUPPORTED.into()); }
        let scope = crate::pet_codex_request_scope::extract("mcpServer/elicitation/request", params).ok_or(UNSUPPORTED)?;
        if source.identity().session_id != scope.session || source.identity().turn_id.as_deref() != scope.turn { return Err(UNSUPPORTED.into()); }
        let bytes = serde_json::to_vec(params).map_err(|_| UNSUPPORTED)?;
        if bytes.len() > 256*1024 { return Err(UNSUPPORTED.into()); }
        let hash = ring::digest::digest(&ring::digest::SHA256, &bytes).as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>();
        if hash != source.identity().arguments_hash { return Err(UNSUPPORTED.into()); }
        let (fields, detail) = compile(params, review_id)?;
        Ok(Self { source: source.clone(), fields, detail })
    }
    /// Presentation probe only; it neither captures a source nor permits IO.
    pub(crate) fn supported(params: &Value) -> bool {
        serde_json::to_vec(params).is_ok_and(|b| b.len() <= 256*1024) && compile(params, "schema-probe").is_ok()
    }

    pub(crate) fn matches(&self, source: &Ticket) -> bool { self.source.same_capture(source) }
    pub(crate) fn questions(&self) -> Value { json!(self.fields.iter().map(Field::public).collect::<Vec<_>>()) }
    pub(crate) fn detail(&self) -> &str { &self.detail }
    pub(crate) fn resolve(&self, answer: &Value) -> Result<Value, String> {
        if !self.source.current() { return Err("Este pedido MCP já terminou.".into()); }
        object_keys(answer, &["allow", "answers"]).map_err(|_| INVALID)?;
        if serde_json::to_vec(answer).map_err(|_| INVALID)?.len() > 64*1024 { return Err(INVALID.into()); }
        let allow = answer["allow"].as_bool().ok_or(INVALID)?;
        let answers = answer.get("answers").map(|v| v.as_object().ok_or(INVALID)).transpose()?;
        if let Some(answers) = answers { if answers.keys().any(|k| !self.fields.iter().any(|f| f.id == *k)) { return Err(INVALID.into()); } }
        if !allow {
            if answers.is_some_and(|a| a.values().any(|v| v.as_array().is_none_or(|a| !a.is_empty()))) { return Err(INVALID.into()); }
            return Ok(json!({"action":"decline","content":null,"_meta":null}));
        }
        let mut content = Map::new();
        for field in &self.fields {
            match answers.and_then(|a| a.get(&field.id)) {
                Some(v) => { content.insert(field.name.clone(), field.value(v)?); }
                None if field.required => return Err(INVALID.into()),
                None => {},
            }
        }
        Ok(json!({"action":"accept","content":content,"_meta":null}))
    }
}

#[cfg(test)]
#[path="pet_mcp_form_tests.rs"]mod tests;
