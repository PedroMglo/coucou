use super::{history, live::Store, types::{Target, Origin}};
use serde_json::{json, Value};

fn target() -> Target {
    Target { pet_id:"agent_codex".into(), provider:"codex".into(), origin:Origin::External,
        session_id:"00000000-0000-0000-0000-000000000001".into(), conversation_id:None }
}
fn thread(state:&str) -> Value {
    json!({"id":target().session_id,"status":{"type":state},"canAcceptDirectInput":true,
        "cwd":"/synthetic/workspace","updatedAt":100})
}
fn turns() -> Vec<Value> {
    vec![json!({"id":"a","status":"inProgress","items":[{"id":"user","type":"userMessage","content":[{"type":"text","text":"Pedido público"}]}]}),
        json!({"id":"b","status":"inProgress","items":[{"id":"reply","type":"agentMessage","text":"Resposta pública"}]})]
}

#[test]
fn conflicting_turns_preserve_ordered_public_history_with_readonly_controls() {
    for state in ["active","idle","systemError","notLoaded"] {
        let view=history::snapshot(&thread(state),target(),&turns()).unwrap();
        assert_eq!(view.messages.iter().map(|m|m.content.as_str()).collect::<Vec<_>>(),["Pedido público","Resposta pública"]);
        assert!(view.capabilities.history);assert!(view.busy);assert!(view.turn.is_none());
        assert!(!view.capabilities.send);assert!(!view.capabilities.cancel);assert!(view.control_conflict);
        assert!(view.notice.as_deref().unwrap().contains("vários turnos ativos"));
        let json=serde_json::to_value(view).unwrap();
        assert!(json.get("controlConflict").is_none());assert!(json.get("control_conflict").is_none());
    }
}

#[test]
fn summary_keeps_a_conflicting_session_in_the_list_without_granting_control() {
    let mut metadata=thread("active");metadata["turns"]=json!(turns());
    let summary=history::summary(&metadata,target()).unwrap();
    assert!(summary.capabilities.history);assert!(!summary.capabilities.send);assert!(!summary.capabilities.cancel);
    assert_eq!(summary.target,target());
}

#[test]
fn ambiguity_never_chooses_a_turn_by_order_and_a_fresh_unique_snapshot_restores_control() {
    let mut values=turns();values.reverse();
    assert!(history::snapshot(&thread("active"),target(),&values).unwrap().turn.is_none());
    values[1]["status"]=json!("completed");
    let unique=history::snapshot(&thread("active"),target(),&values).unwrap();
    assert_eq!(unique.turn.as_deref(),Some("b"));assert!(unique.capabilities.cancel);assert!(!unique.control_conflict);
    // Control from this different fresh snapshot must not mutate the old one.
    let empty=history::snapshot(&thread("idle"),target(),&[]).unwrap();
    assert!(!empty.busy);assert!(empty.capabilities.send);assert!(!empty.capabilities.cancel);
}

#[test]
fn duplicate_turn_ids_incomplete_history_invalid_status_and_foreign_thread_still_fail() {
    let mut values=turns();values[1]["id"]=json!("a");
    assert!(history::snapshot(&thread("active"),target(),&values).is_err());
    let mut duplicate_metadata=thread("active");duplicate_metadata["turns"]=json!(values);
    assert!(history::summary(&duplicate_metadata,target()).is_err());
    values=turns();values.push(values[1].clone());duplicate_metadata["turns"]=json!(values);
    assert!(history::summary(&duplicate_metadata,target()).is_err());
    values=turns();values[1]["itemsView"]=json!("summary");
    assert!(history::snapshot(&thread("active"),target(),&values).is_err());
    values=turns();values.push(json!({"id":"invalid","status":"mystery","items":[]}));
    assert!(history::snapshot(&thread("active"),target(),&values).is_err());
    let mut foreign=thread("active");foreign["id"]=json!("00000000-0000-0000-0000-000000000002");
    assert!(history::snapshot(&foreign,target(),&turns()).is_err());
}

#[test]
fn live_turn_started_and_partial_message_cannot_override_ambiguity_or_its_notice() {
    let snapshot=history::snapshot(&thread("active"),target(),&turns()).unwrap();
    let expected=snapshot.notice.clone();let mut store=Store::default();
    store.event(&json!({"method":"turn/started","params":{"threadId":target().session_id,"turn":{"id":"c"}}}));
    store.event(&json!({"method":"item/agentMessage/delta","params":{"threadId":target().session_id,"turnId":"c","itemId":"late","delta":"Trecho público em direto"}}));
    let view=store.merge(snapshot);
    assert_eq!(view.messages.len(),3);assert_eq!(view.messages[2].content,"Trecho público em direto");
    assert!(view.turn.is_none());assert!(!view.capabilities.send);assert!(!view.capabilities.cancel);
    assert_eq!(view.notice,expected);
}

#[test]
fn stream_memory_notice_and_missing_stream_capacity_notice_preserve_ambiguity() {
    let make=||history::snapshot(&thread("active"),target(),&turns()).unwrap();
    let mut store=Store::default();
    store.event(&json!({"method":"item/agentMessage/delta","params":{"threadId":target().session_id,"turnId":"a","itemId":"oversized","delta":"x".repeat(4*1024*1024+1)}}));
    let expected=make().notice;assert_eq!(store.merge(make()).notice,expected);
    let mut full=Store::default();
    for n in 1..=129 {
        full.event(&json!({"method":"turn/started","params":{"threadId":format!("other-{n}"),"turn":{"id":"busy"}}}));
    }
    let view=full.merge(make());assert_eq!(view.notice,expected);assert!(!view.capabilities.cancel);
}
