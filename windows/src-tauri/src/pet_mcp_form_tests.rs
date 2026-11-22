use super::*;
use crate::{pet_config::Adapter,pet_native_decisions::{Capture,Endpoint,PolicyBinding,Transport}};
struct Fixture { endpoint: Endpoint, source: Ticket, form: Form, params: Value }
fn params(props: Value, required: Value) -> Value {
    json!({"threadId":"native-thread","turnId":null,"serverName":"fixture","mode":"form","message":"Synthetic form",
        "requestedSchema":{"type":"object","properties":props,"required":required}})
}
fn source(p: &Value) -> (Endpoint,Ticket) {
    let endpoint=Endpoint::new(Adapter::Codex,Transport::ManagedPipe).unwrap();
    let source=endpoint.capture(Capture{request_id:Some(&json!(7)),session_id:"native-thread",turn_id:None,prompt_id:None,
        method:"mcpServer/elicitation/request",arguments:p,policy:PolicyBinding::unknown(),cancel:None}).unwrap();
    (endpoint,source)
}
fn fixture(p: Value) -> Fixture {
    let (endpoint,source)=source(&p);let form=Form::capture(&source,"private-review",&p).unwrap();Fixture{endpoint,source,form,params:p}
}
fn id<'a>(f:&'a Form,name:&str)->&'a str{&f.fields.iter().find(|v|v.name==name).unwrap().id}
fn selected(f:&Form,name:&str,indices:&[usize])->Value{json!(indices.iter().map(|i|format!("{}-option-{i}",id(f,name))).collect::<Vec<_>>())}
fn answer(f:&Form,values:Vec<(&str,Value)>)->Value{
    let mut answers=Map::new();for(name,value)in values{answers.insert(id(f,name).into(),value);}json!({"allow":true,"answers":answers})
}
#[test]fn native_content_keeps_real_types_const_values_and_multiple_choices(){
    let f=fixture(params(json!({"text":{"type":"string"},"count":{"type":"integer","minimum":1,"maximum":3},"amount":{"type":"number"},
        "confirm":{"type":"boolean","default":true},"choice":{"type":"string","oneOf":[{"const":"wire-value","title":"Friendly choice"}]},
        "tags":{"type":"array","items":{"anyOf":[{"const":"a","title":"First"},{"const":"b","title":"Second"}]},"minItems":1,"maxItems":2}}),json!(["text","count","confirm"])));
    let a=answer(&f.form,vec![("text",json!(["hello"])),("count",json!(["2"])),("amount",json!(["1.25"])),("confirm",selected(&f.form,"confirm",&[1])),("choice",selected(&f.form,"choice",&[0])),("tags",selected(&f.form,"tags",&[1,0]))]);
    assert_eq!(f.form.resolve(&a).unwrap(),json!({"action":"accept","content":{"text":"hello","count":2,"amount":1.25,"confirm":false,"choice":"wire-value","tags":["b","a"]},"_meta":null}));
    assert!(!f.form.questions().to_string().contains("wire-value"));assert!(f.form.questions().to_string().contains("Friendly choice"));
}
#[test]fn enum_variants_keep_values_distinct_from_legacy_and_modern_titles(){
    for schema in [json!({"type":"string","enum":["wire"],"enumNames":["Displayed"]}),json!({"type":"string","oneOf":[{"const":"wire","title":"Displayed"}]}),json!({"type":"string","enum":["wire"]})]{
        let f=fixture(params(json!({"x":schema}),json!(["x"])));
        assert_eq!(f.form.resolve(&answer(&f.form,vec![("x",selected(&f.form,"x",&[0]))])).unwrap()["content"]["x"],"wire");
        for raw in ["wire","Displayed","false"]{assert!(f.form.resolve(&answer(&f.form,vec![("x",json!([raw]))])).is_err());}
    }
}
#[test]fn optional_omission_is_not_an_empty_string_array_or_default(){
    let f=fixture(params(json!({"text":{"type":"string","default":"DO_NOT_USE"},"list":{"type":"array","items":{"type":"string","enum":["a"]}},"requiredEmpty":{"type":"string"}}),json!(["requiredEmpty"])));
    assert_eq!(f.form.resolve(&answer(&f.form,vec![("requiredEmpty",json!([""]))])).unwrap()["content"],json!({"requiredEmpty":""}));
    let all=answer(&f.form,vec![("requiredEmpty",json!([""])),("text",json!([""])),("list",json!([]))]);assert_eq!(f.form.resolve(&all).unwrap()["content"],json!({"requiredEmpty":"","text":"","list":[]}));
    assert!(f.form.resolve(&json!({"allow":true,"answers":{}})).is_err());
}
#[test]fn malformed_or_out_of_range_numbers_never_gain_native_accept(){
    let f=fixture(params(json!({"x":{"type":"integer","minimum":1,"maximum":3}}),json!(["x"])));
    for raw in ["true","false","null","[]","{}","\"2\"","1.5","0","4","NaN","1e9999","01"]{
        let err=f.form.resolve(&answer(&f.form,vec![("x",json!([raw]))])).unwrap_err();assert_eq!(err,INVALID);assert!(f.source.current());
    }
    assert_eq!(f.form.resolve(&answer(&f.form,vec![("x",json!(["2"]))])).unwrap()["content"]["x"],2);
}
#[test]fn unicode_string_lengths_and_multi_select_limits_are_validated_before_reply(){
    let f=fixture(params(json!({"x":{"type":"string","minLength":2,"maxLength":2},"many":{"type":"array","items":{"type":"string","enum":["a","b","c"]},"minItems":1,"maxItems":2}}),json!(["x","many"])));
    let good=answer(&f.form,vec![("x",json!(["á𝄞"])),("many",selected(&f.form,"many",&[0,1]))]);assert_eq!(f.form.resolve(&good).unwrap()["action"],"accept");
    for value in [selected(&f.form,"many",&[]),selected(&f.form,"many",&[0,1,2]),selected(&f.form,"many",&[0,0]),json!(["a"])]{
        assert!(f.form.resolve(&answer(&f.form,vec![("x",json!(["á𝄞"])),("many",value)])).is_err());
    }
    for raw in ["a","abc"]{assert!(f.form.resolve(&answer(&f.form,vec![("x",json!([raw])),("many",selected(&f.form,"many",&[0]))])).is_err());}
}
#[test]fn scalar_fields_reject_arrays_objects_null_and_multiple_values(){
    let f=fixture(params(json!({"text":{"type":"string"},"flag":{"type":"boolean"}}),json!([])));
    for bad in [Value::Null,json!({}),json!([true]),json!([]),json!(["one","two"])]{assert!(f.form.resolve(&answer(&f.form,vec![("text",bad)])).is_err());}
    for bad in [json!(["true"]),json!([true]),selected(&f.form,"flag",&[0,1])]{assert!(f.form.resolve(&answer(&f.form,vec![("flag",bad)])).is_err());}
}
#[test]fn secrets_and_defaults_never_appear_in_public_form_or_implicit_content(){
    let mut p=params(json!({"secret":{"type":"string","format":"password","default":"PRIVATE_PASSWORD"},"flag":{"type":"boolean","default":true},"pick":{"type":"string","enum":["a","b"],"default":"b"}}),json!([]));p["mode"]=json!("openai/form");let f=fixture(p);
    let public=f.form.questions().to_string()+f.form.detail();assert!(!public.contains("PRIVATE_PASSWORD"));assert!(!public.contains("default"));assert_eq!(f.form.questions()[0]["isSecret"],true);
    assert_eq!(f.form.resolve(&json!({"allow":true,"answers":{}})).unwrap()["content"],json!({}));
    assert_eq!(f.form.resolve(&answer(&f.form,vec![("secret",json!(["user supplied"]))])).unwrap()["content"]["secret"],"user supplied");
}
#[test]fn unsupported_authentication_or_schema_cannot_become_an_empty_accepted_form(){
    for mode in ["url","openai/userVerification","unknown"]{let mut p=params(json!({}),json!([]));p["mode"]=json!(mode);let(_e,s)=source(&p);assert!(Form::capture(&s,"review",&p).is_err());assert!(!Form::supported(&p));}
    for schema in [json!({}),json!({"type":"object","properties":{},"$ref":"https://fixture.invalid/schema"}),json!({"type":"object","properties":{"x":{"type":"object"}}}),json!({"type":"object","properties":{"x":{"type":"string","pattern":".*"}}}),json!({"type":"object","properties":{"x":{"type":"string","format":"password"}}}),json!({"type":"object","properties":{},"required":["missing"]})]{
        let mut p=params(json!({}),json!([]));p["requestedSchema"]=schema;let(_e,s)=source(&p);assert!(Form::capture(&s,"review",&p).is_err());
    }
    let f=fixture(params(json!({}),json!([])));assert_eq!(f.form.resolve(&json!({"allow":true,"answers":{}})).unwrap()["content"],json!({}));
}
#[test]fn ambiguous_enums_and_unfulfillable_or_unknown_constraints_are_unsupported(){
    for schema in [json!({"type":"string","enum":["same","same"]}),json!({"type":"string","enum":["a"],"enumNames":[]}),json!({"type":"string","enum":["a"],"oneOf":[{"const":"b","title":"B"}]}),json!({"type":"number","minimum":2,"maximum":1}),json!({"type":"string","minLength":2,"maxLength":1}),json!({"type":"array","items":{"type":"string","enum":["a"]},"minItems":2}),json!({"type":"string","format":"future-format"})]{let p=params(json!({"x":schema}),json!([]));assert!(!Form::supported(&p));}
}
#[test]fn changed_arguments_wrong_source_and_other_review_option_ids_never_match(){
    let f=fixture(params(json!({"pick":{"type":"string","enum":["a"]}}),json!(["pick"])));let mut changed=f.params.clone();changed["message"]=json!("changed");assert!(Form::capture(&f.source,"new-review",&changed).is_err());
    let(_other,other)=source(&f.params);assert!(!f.form.matches(&other));let other_form=Form::capture(&other,"other-review",&f.params).unwrap();assert!(f.form.resolve(&answer(&other_form,vec![("pick",selected(&other_form,"pick",&[0]))])).is_err());
}
#[test]fn ended_policy_changed_or_closed_source_irreversibly_withdraws_form(){
    for mode in 0..3{let f=fixture(params(json!({"x":{"type":"string"}}),json!([])));match mode{0=>f.source.cancel(),1=>f.source.invalidate_policy(),_=>f.endpoint.close()};assert!(f.form.resolve(&json!({"allow":true,"answers":{}})).is_err());assert!(f.form.resolve(&json!({"allow":false})).is_err());}
}
#[test]fn denial_never_sends_secret_content_or_uses_defaults(){
    let f=fixture(params(json!({"x":{"type":"string","default":"PRIVATE"}}),json!(["x"])));assert_eq!(f.form.resolve(&json!({"allow":false})).unwrap(),json!({"action":"decline","content":null,"_meta":null}));assert!(f.form.resolve(&answer(&f.form,vec![("x",json!(["user value"]))])) .is_ok());
    let mut denial=answer(&f.form,vec![("x",json!(["PRIVATE"]))]);denial["allow"]=json!(false);assert!(f.form.resolve(&denial).is_err());
}
#[test]fn schema_and_answer_caps_are_enforced_without_echoing_private_values(){
    let mut props=Map::new();for i in 0..33{props.insert(format!("f{i}"),json!({"type":"string"}));}assert!(!Form::supported(&params(json!(props),json!([]))));
    let f=fixture(params(json!({"x":{"type":"string"}}),json!([])));assert_eq!(f.form.resolve(&answer(&f.form,vec![("x",json!(["PRIVATE".repeat(2000)]))])).unwrap_err(),INVALID);
    let mut a=answer(&f.form,vec![]);a["answers"]["unknown"]=json!(["private"]);assert_eq!(f.form.resolve(&a).unwrap_err(),INVALID);
    a=json!({"allow":true,"nativeResponse":{"action":"accept","content":{}}});assert!(f.form.resolve(&a).is_err());
}
#[test]fn declared_string_formats_validate_values_instead_of_trusting_browser_controls(){
    for(format,valid,invalid)in[
        ("email",vec!["user+tag@example.com","\"quoted local\"@example.com","user@[IPv6:2001:db8::1]"],vec!["not-email","x@-bad.example","a..b@example.com","x@example..com","x@host\n"]),
        ("uri",vec!["https://example.com/a%20b","urn:isbn:123","mailto:person@example.com"],vec!["/relative","https://host/a b","https://host/%zz","https://host/<script>"]),
        ("date",vec!["2024-02-29","2026-10-24"],vec!["2025-02-29","2026-1-01","2026-10-24T12:00:00Z"]),
        ("date-time",vec!["2026-10-24T12:00:00Z","2026-10-24T12:00:00+01:00"],vec!["2026-10-24","2026-10-24T25:00:00Z","2026-10-24T12:00:00"]) ]{
        let f=fixture(params(json!({"x":{"type":"string","format":format}}),json!(["x"])));
        for raw in valid{assert!(f.form.resolve(&answer(&f.form,vec![("x",json!([raw]))])).is_ok(),"{format} should accept fixture");}
        for raw in invalid{assert!(f.form.resolve(&answer(&f.form,vec![("x",json!([raw]))])).is_err(),"{format} should reject fixture");}
    }
}
#[test]fn generic_uri_and_smtp_literal_syntax_are_not_replaced_by_url_normalization(){
    for s in["http:","scheme:","file:///tmp/a","custom://user:pass@[2001:db8::1]:12345/path?x=a/b?c#fragment","custom://[v1.future:address]/path","custom://host:/path","urn:a%20b"]{assert!(format_valid("uri",s),"valid generic URI fixture");}
    for s in["http://host:bad/path","custom://[not-ip]/","urn:a[b]","custom:a#two#fragments","custom://user@other@host/","custom://[v1.bad%20address]/"]{assert!(!format_valid("uri",s),"invalid generic URI fixture");}
    for s in["x@[001.002.003.004]","x@[ipv6:::ffff:001.002.003.004]","x@[Future:address]"]{assert!(format_valid("email",s),"valid SMTP literal syntax fixture");}
    for s in["x@[999.2.3.4]","x@[IPv6:not-an-ip]","x@[Future:bad\\literal]"]{assert!(!format_valid("email",s),"invalid SMTP literal syntax fixture");}
}
