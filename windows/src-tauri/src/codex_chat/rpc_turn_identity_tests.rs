//! Synthetic child protocol only: no Codex executable, account or inference.
use super::*;
use std::sync::{Arc, Mutex};

struct Fixture { cwd: std::path::PathBuf, exe: std::path::PathBuf }
impl Fixture {
    fn new(events: Vec<Value>, own_terminal: bool) -> Self {
        let (cwd, exe) = super::tests::mock();
        std::fs::write(cwd.join("terminal-events.json"), serde_json::to_vec(&events).unwrap()).unwrap();
        if !own_terminal { std::fs::write(cwd.join("skip-terminal"), "").unwrap(); }
        let path = cwd.join("app-server");
        let mut source = std::fs::read_to_string(&path).unwrap();
        let marker = "    if method=='initialized': continue";
        assert_eq!(source.matches(marker).count(), 1);
        source = source.replace(marker, &format!("{marker}\n    if method=='turn/interrupt': pathlib.Path('interrupt.json').write_text(json.dumps(p))"));
        let marker = "    if method=='turn/start':\n        send({'id':99";
        assert_eq!(source.matches(marker).count(), 1);
        source = source.replace(marker,
            "    if method=='turn/start':\n        for event in json.loads(pathlib.Path('terminal-events.json').read_text()):\n            send({'method':'turn/completed','params':event})\n        send({'id':99");
        let marker = "        send({'method':'turn/completed','params':{'threadId':'owned-thread','turn':{'id':'turn-1','status':'completed'}}})";
        assert_eq!(source.matches(marker).count(), 1);
        source = source.replace(marker, &format!("        if not pathlib.Path('skip-terminal').exists(): {}", marker.trim()));
        std::fs::write(path, source).unwrap();
        Self { cwd, exe }
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.cwd); } }

struct ResultView {
    reply: Result<Reply, String>,
    requests: usize,
    observed: Vec<Value>,
    active_before_drop: Option<(String, String)>,
    interrupted: Option<Value>,
}
fn drive(events: Vec<Value>, own_terminal: bool) -> ResultView {
    let f = Fixture::new(events, own_terminal);
    let mut client = Client::start(&f.exe, &f.cwd, Duration::from_secs(5), false).unwrap();
    if !own_terminal { client.deadline = Instant::now() + Duration::from_millis(500); }
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = observed.clone(); client.observe(move |event| sink.lock().unwrap().push(event.clone()));
    let mut requests = 0;
    let reply = client.answer(TurnOptions {
        model: "fixture-model", system: "synthetic", history: &[], query: "synthetic",
        tools: &[], session_id: None, full_access: false, effort: None, name: None,
    }, |_, _| panic!("No dynamic tool is part of this synthetic fixture"), |method, params| {
        assert_eq!(method, "item/commandExecution/requestApproval");
        assert_eq!(params["threadId"], "owned-thread");assert_eq!(params["turnId"], "turn-1");
        requests += 1; Some(json!({"decision":"decline"}))
    }, |_| {}, |_| {});
    let active_before_drop = client.active_turn.clone();
    drop(client); // The actual existing Drop closes/reaps only this fake child.
    let interrupted = std::fs::read(f.cwd.join("interrupt.json")).ok().map(|v| serde_json::from_slice(&v).unwrap());
    let observed = observed.lock().unwrap().clone();
    ResultView { reply, requests, observed, active_before_drop, interrupted }
}
fn assert_owned_reply(result: ResultView) {
    let reply = result.reply.unwrap();
    assert_eq!(reply.text, "denied");assert_eq!(reply.session_id, "owned-thread");
    assert_eq!(result.requests, 1);assert!(result.active_before_drop.is_none());assert!(result.interrupted.is_none());
    let completed: Vec<_> = result.observed.iter().filter(|m| m["method"] == "turn/completed").collect();
    assert_eq!(completed.len(), 1);assert_eq!(completed[0]["params"]["turn"]["id"], "turn-1");
}

#[test]
fn nested_foreign_terminal_cannot_finish_current_answer_or_reach_observer() {
    for flat in [false, true] {
        let mut params = json!({"threadId":"owned-thread","turn":{"id":"other-turn","status":"completed"}});
        if flat { params["turnId"] = json!("turn-1"); }
        assert_owned_reply(drive(vec![params], true));
    }
}
#[test]
fn malformed_or_absent_nested_terminal_identity_never_substitutes_current_turn() {
    for turn in [Value::Null, json!({}), json!({"id":null}), json!({"id":12}), json!({"id":""}), json!({"id":[]}), json!({"id":{}})] {
        let mut params = json!({"threadId":"owned-thread","turnId":"turn-1","turn":turn});
        if params["turn"].is_object() { params["turn"]["status"] = json!("completed"); }
        assert_owned_reply(drive(vec![params], true));
    }
    assert_owned_reply(drive(vec![json!({"threadId":"owned-thread","turnId":"turn-1"})], true));
}
#[test]
fn conflicting_flat_terminal_identity_and_foreign_thread_remain_rejected() {
    assert_owned_reply(drive(vec![
        json!({"threadId":"owned-thread","turnId":"other-turn","turn":{"id":"turn-1","status":"completed"}}),
        json!({"threadId":"other-thread","turn":{"id":"turn-1","status":"completed"}}),
        json!({"threadId":"owned-thread","turn":{"id":"other-turn","status":"failed","error":{"message":"FOREIGN_ERROR"}}}),
    ], true));
}
#[test]
fn exact_nested_completion_without_flat_id_still_finishes_the_owned_turn() {
    assert_owned_reply(drive(Vec::new(), true));
}
#[test]
fn exact_nested_failure_keeps_original_failure_and_cleanup_contract() {
    let result = drive(vec![json!({"threadId":"owned-thread","turn":{"id":"turn-1","status":"failed","error":{"message":"OWN_NATIVE_FAILURE"}}})], true);
    match result.reply { Err(error) => assert_eq!(error, "OWN_NATIVE_FAILURE"), Ok(_) => panic!("Exact failed turn cannot become success") }
    assert_eq!(result.requests, 0);
    assert_eq!(result.active_before_drop, Some(("owned-thread".into(), "turn-1".into())));
    assert_eq!(result.interrupted, Some(json!({"threadId":"owned-thread","turnId":"turn-1"})));
}
#[test]
fn foreign_terminal_without_owned_completion_preserves_deadline_and_exact_interrupt() {
    let result = drive(vec![json!({"threadId":"owned-thread","turn":{"id":"other-turn","status":"completed"}})], false);
    assert!(result.reply.is_err());assert_eq!(result.requests, 1);
    assert_eq!(result.active_before_drop, Some(("owned-thread".into(), "turn-1".into())));
    assert_eq!(result.interrupted, Some(json!({"threadId":"owned-thread","turnId":"turn-1"})));
    assert!(!result.observed.iter().any(|m| m["method"] == "turn/completed"));
}
