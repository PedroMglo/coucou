//! Native interpretation of the island's acknowledged pet roster operation.
use serde::Deserialize;
use serde_json::Value;

const MAX_NAMES:usize=32;
const MAX_LABEL_CHARS:usize=128;
const MAX_ANSWER_BYTES:usize=64*1024;
const MAX_ACTIVE:usize=6;

fn label(text:&str)->bool {
    !text.trim().is_empty() && text.len()<=MAX_LABEL_CHARS*4
        && text.chars().count()<=MAX_LABEL_CHARS && !text.chars().any(char::is_control)
}
fn labels(items:&[String])->bool { items.len()<=MAX_NAMES && items.iter().all(|item|label(item)) }
fn unique(items:&[String])->bool { items.iter().collect::<std::collections::BTreeSet<_>>().len()==items.len() }

pub(crate) fn valid_request(input:&Value)->bool {
    let Some(object)=input.as_object() else {return false};
    if object.keys().any(|key|!matches!(key.as_str(),"action"|"names")){return false;}
    let Some(action)=input["action"].as_str().filter(|action|matches!(*action,"list"|"add"|"remove"|"set_main")) else{return false};
    let names=match input.get("names") {
        None=>&[][..],
        Some(Value::Array(names)) if names.len()<=MAX_NAMES=>names.as_slice(),
        _=>return false,
    };
    if action=="list" {return names.is_empty();}
    !names.is_empty() && names.iter().all(|name|name.as_str().is_some_and(label))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Acknowledgement {done:Vec<String>,failed:Vec<String>,now:Description}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Description {main:String,shown:Vec<String>,max_besides_main:usize,available:Vec<String>}

pub(crate) fn confirmed(input: &Value, answer: &str) -> bool {
    if !valid_request(input) || input["action"]=="list" || answer.len()>MAX_ANSWER_BYTES{return false;}
    let Ok(reply)=serde_json::from_str::<Acknowledgement>(answer) else{return false};
    let requested=input["names"].as_array().map_or(0,Vec::len);
    if !reply.failed.is_empty() || reply.done.len()!=requested || !labels(&reply.done){return false;}
    let current=reply.now;
    if current.max_besides_main!=MAX_ACTIVE || current.shown.len()>MAX_ACTIVE
        || !label(&current.main) || !labels(&current.shown) || !unique(&current.shown)
        || !labels(&current.available) || !unique(&current.available)
        || !current.available.contains(&current.main)
        || current.shown.contains(&current.main)
        || current.shown.iter().chain(&reply.done).any(|name|!current.available.contains(name)){return false;}
    match input["action"].as_str() {
        Some("add")=>reply.done.iter().all(|name|name==&current.main || current.shown.contains(name)),
        Some("remove")=>reply.done.iter().all(|name|name!=&current.main && !current.shown.contains(name)),
        Some("set_main")=>reply.done.last()==Some(&current.main),
        _=>false,
    }
}

/// Validate before dispatch, invoke exactly once, and preserve the exact result
/// text even for partial/failed work. This boolean never authorizes another action.
pub(crate) fn run(input:&Value,ask:impl FnOnce(&Value)->Option<String>)->(String,bool){
    if !valid_request(input){return ("Pedido de pets inválido. Indica uma ação e até 32 nomes; listar não recebe nomes.".into(),false);}
    let answer=ask(input).unwrap_or_else(||"the island did not answer".into());
    let acted=confirmed(input,&answer);
    (answer,acted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::Cell;

    fn answer(done:&[&str],failed:&[&str],main:&str,shown:&[&str])->String{
        json!({"done":done,"failed":failed,"now":{"main":main,"shown":shown,"maxBesidesMain":6,"available":["Codex","Gravity","Copilot CLI","Ollama","Spotify","unknown pet"]}}).to_string()
    }

    #[test]
    fn failed_roster_persistence_is_not_a_confirmed_voice_action() {
        let input=json!({"action":"remove","names":["Copilot"]});
        let answer=json!({"done":[],"failed":["Copilot CLI: não foi possível guardar"],"now":{"main":"Codex","shown":["Copilot CLI","Antigravity","Ollama","Spotify"],"maxBesidesMain":6,"available":["Codex","Antigravity","Copilot CLI","Ollama","Spotify"]}});
        assert!(!confirmed(&input,&answer.to_string()));
    }

    #[test]
    fn complete_typed_acknowledgements_match_the_requested_intention(){
        for (input,reply) in [
            (json!({"action":"add","names":["Ollama"]}),answer(&["Ollama"],&[],"Codex",&["Gravity","Ollama"])),
            (json!({"action":"remove","names":["Copilot"]}),answer(&["Copilot CLI"],&[],"Codex",&["Gravity","Ollama"])),
            (json!({"action":"set_main","names":["Gravity"]}),answer(&["Gravity"],&[],"Gravity",&["Ollama"])),
        ] {assert!(confirmed(&input,&reply));}
    }

    #[test]
    fn partial_failure_timeout_and_unstructured_text_never_confirm_a_whole_action(){
        let input=json!({"action":"remove","names":["Copilot","Ollama"]});
        for reply in [
            answer(&["Copilot CLI"],&["Ollama: write failed"],"Codex",&["Gravity","Ollama"]),
            answer(&["Copilot CLI"],&[],"Codex",&["Gravity","Ollama"]),
            "the island did not answer".into(),"error: disk failed".into(),"✓".into(),"{malformed".into(),"{}".into(),
        ] {assert!(!confirmed(&input,&reply),"{reply}");}
    }

    #[test]
    fn a_snapshot_contradicting_the_acknowledged_intention_is_not_confirmation(){
        for (input,reply) in [
            (json!({"action":"add","names":["Ollama"]}),answer(&["Ollama"],&[],"Codex",&["Gravity"])),
            (json!({"action":"remove","names":["Copilot"]}),answer(&["Copilot CLI"],&[],"Codex",&["Copilot CLI"])),
            (json!({"action":"set_main","names":["Gravity"]}),answer(&["Gravity"],&[],"Codex",&["Gravity"])),
        ] {assert!(!confirmed(&input,&reply));}
    }

    #[test]
    fn malformed_requests_never_dispatch_to_the_island(){
        for input in [
            Value::Null,json!([]),json!({}),json!({"action":"remove"}),json!({"action":"toggle","names":["Ollama"]}),
            json!({"action":"remove","names":[]}),json!({"action":"remove","names":"Ollama"}),
            json!({"action":"add","names":[true]}),json!({"action":"add","names":[""]}),json!({"action":"add","names":["\n"]}),
            json!({"action":"add","names":["Ollama"],"activeIntegrations":[]}),json!({"action":"list","names":["Ollama"]}),
            json!({"action":"add","names":vec!["Ollama";MAX_NAMES+1]}),json!({"action":"add","names":["é".repeat(MAX_LABEL_CHARS+1)]}),
        ] {
            let called=Cell::new(false);
            let (_,acted)=run(&input,|_|{called.set(true);Some("unreachable".into())});
            assert!(!acted);assert!(!called.get(),"{input}");
        }
    }

    #[test]
    fn typed_results_reject_null_numbers_duplicates_and_unknown_fields(){
        let input=json!({"action":"remove","names":["Copilot"]});
        let good:Value=serde_json::from_str(&answer(&["Copilot CLI"],&[],"Codex",&["Gravity"])).unwrap();
        for (key,value) in [("done",Value::Null),("done",json!([true])),("failed",json!([0])),("now",json!([])),("unexpected",json!(true))]{
            let mut invalid=good.clone();invalid[key]=value;assert!(!confirmed(&input,&invalid.to_string()));
        }
        for (key,value) in [("main",json!([])),("shown",json!([false])),("available",Value::Null),("maxBesidesMain",json!(6.0)),("maxBesidesMain",json!(7)),("extra",json!(false))]{
            let mut invalid=good.clone();invalid["now"][key]=value;assert!(!confirmed(&input,&invalid.to_string()));
        }
        assert!(!confirmed(&input,r#"{"done":["Copilot CLI"],"done":[],"failed":[],"now":{}}"#));
    }

    #[test]
    fn inconsistent_catalogs_and_duplicate_visible_pets_are_not_proof(){
        let input=json!({"action":"remove","names":["Copilot"]});
        for (main,shown) in [("Codex",vec!["Gravity","Gravity"]),("Codex",vec!["Codex"]),("not-in-catalog",vec!["Gravity"])]{
            assert!(!confirmed(&input,&answer(&["Copilot CLI"],&[],main,&shown)));
        }
        assert!(!confirmed(&input,&answer(&["not-in-catalog"],&[],"Codex",&["Gravity"])));
    }

    #[test]
    fn original_result_is_preserved_and_a_valid_request_dispatches_exactly_once(){
        let input=json!({"action":"remove","names":["Copilot","Ollama"]});
        let partial=answer(&["Copilot CLI"],&["Ollama: disk failed"],"Codex",&["Gravity","Ollama"]);
        let calls=Cell::new(0);
        let (text,acted)=run(&input,|captured|{assert_eq!(captured,&input);calls.set(calls.get()+1);Some(partial.clone())});
        assert_eq!(text,partial);assert!(!acted);assert_eq!(calls.get(),1);
        let (text,acted)=run(&json!({"action":"remove","names":["Copilot"]}),|_|None);
        assert_eq!(text,"the island did not answer");assert!(!acted);
    }

    #[test]
    fn listing_is_read_only_and_does_not_require_names(){
        for input in [json!({"action":"list"}),json!({"action":"list","names":[]})]{
            assert!(valid_request(&input));let calls=Cell::new(0);
            let (text,acted)=run(&input,|_|{calls.set(calls.get()+1);Some("listing data".into())});
            assert_eq!(text,"listing data");assert!(!acted);assert_eq!(calls.get(),1);
        }
    }

    #[test]
    fn bounded_unicode_and_words_inside_labels_do_not_become_status_heuristics(){
        assert!(valid_request(&json!({"action":"add","names":["é".repeat(MAX_LABEL_CHARS)]})));
        let input=json!({"action":"add","names":["unknown pet"]});
        assert!(confirmed(&input,&answer(&["unknown pet"],&[],"Codex",&["unknown pet"])));
        let huge=format!("{}{}",answer(&["unknown pet"],&[],"Codex",&["unknown pet"])," ".repeat(MAX_ANSWER_BYTES));
        assert!(!confirmed(&input,&huge));
    }
}
