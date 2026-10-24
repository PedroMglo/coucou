use super::*;
use crate::pet_runtime::{results::{self, Lookup, RecoveryPhase}, authority_store::mutate_with_authority};
use std::{fs, os::unix::fs::{PermissionsExt, symlink}};

fn lookup(a: &Authority, g: &Record, request: &str) -> Lookup {
    let row = g.admissions.last().unwrap();
    results::lookup(a, &row.spec, row.proof.as_ref().unwrap(), request).unwrap()
}
fn settled(o: &Owned, g: &Record, request: &str, text: &str) -> ModelCall {
    let call = o.prepare_model(request, g, 100, crate::assistant_memory::now()).unwrap();
    call.before_send("fixture-free", &body()).unwrap();
    call.finish(Some(text.len() as u64)).unwrap();
    call
}
fn path(a: &Authority, g: &Record, call: &ModelCall) -> std::path::PathBuf {
    a.result_root().unwrap().join(&g.admissions.last().unwrap().spec.decision_id)
        .join(format!("{}.json", call.attempt.lock().unwrap().as_ref().unwrap().id))
}
#[test]
fn missing_result_reads_do_not_create_storage_or_dispatch() {
    let (_f, a, g, _p, _o, _main) = setup("ollama");
    let root = a.result_root().unwrap();
    assert!(!root.exists());
    assert!(matches!(lookup(&a, &g, "never-started"), Lookup::NotStarted));
    assert!(!root.exists());
}
#[test]
fn every_common_owner_keeps_independent_authenticated_observations_without_provider_io() {
    for provider in ["codex", "antigravity", "copilot", "ollama", "integration_notion"] {
        let (_f, a, g, _p, o, _main) = setup(provider);
        // Synthetic accounting fixture: this deliberately makes no provider call.
        let call = settled(&o, &g, "result", "Synthetic result");
        call.record_result("Synthetic result", 16).unwrap();
        let Lookup::Available { receipt } = lookup(&a, &g, "result") else { panic!("missing result") };
        assert_eq!(receipt.pet_id, g.spec.owner.pet_id);
        assert_eq!(receipt.provider, provider);
        assert_eq!(receipt.goal_id, g.id);
        assert_eq!(receipt.lease_id, o.lease().id);
        assert_eq!(receipt.intent_hash, g.admissions[0].spec.intent_hash);
        assert_eq!(receipt.text, "Synthetic result");
        let view = serde_json::to_value(results::View::from(Lookup::Available { receipt })).unwrap();
        assert_eq!(view["criteriaVerified"], false);
        assert_eq!(view["goalComplete"], false);
        assert!(view.get("tag").is_none() && view.get("key").is_none());
    }
}
#[test]
fn interrupted_phases_and_missing_acknowledged_receipt_are_distinct_and_never_replayed() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let call = o.prepare_model("started", &g, 100, crate::assistant_memory::now()).unwrap();
    call.before_send("fixture-free", &body()).unwrap();
    assert!(matches!(lookup(&a, &g, "started"), Lookup::RecoveryRequired { phase: RecoveryPhase::Started }));
    call.finish(None).unwrap();
    assert!(matches!(lookup(&a, &g, "started"), Lookup::RecoveryRequired { phase: RecoveryPhase::Uncertain }));
    drop(call);
    let call = settled(&o, &g, "settled", "Known");
    assert!(matches!(lookup(&a, &g, "settled"), Lookup::RecoveryRequired { phase: RecoveryPhase::SettledReceiptMissing }));
    let server = crate::local_chat::Server { id: "ollama", name: "No IO", url: "http://127.0.0.1:1".into(), key: None };
    for request in ["started", "settled"] {
        let error = run(o.respond_once(None, &g, &server, request, &crate::pet_config::defaults())).unwrap_err();
        assert!(error.contains("recuperação"), "{error}");
    }
    // A staged/partial file is not a canonical receipt and cannot repair it.
    call.record_result("Known", 5).unwrap();
    let file = path(&a, &g, &call);
    fs::rename(&file, file.parent().unwrap().join(".result-incomplete")).unwrap();
    assert!(matches!(lookup(&a, &g, "settled"), Lookup::RecoveryRequired { phase: RecoveryPhase::SettledReceiptMissing }));
}
#[test]
fn reserved_accounting_is_not_misrepresented_as_a_response() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let now = crate::assistant_memory::now();
    let attempt = Attempt { id: nonce().unwrap(), request_id: "reserved".into(), input_hash: "a".repeat(64), kind: Kind::Model, output_reserved: 100, phase: Phase::Reserved, accepted_output: None, reserved_at: now, started_at: None, finished_at: None };
    let row = &g.admissions[0];
    a.reserve_attempt(&row.spec, row.proof.as_ref().unwrap(), o.lease(), &attempt, now).unwrap();
    assert!(matches!(lookup(&a, &g, "reserved"), Lookup::RecoveryRequired { phase: RecoveryPhase::Reserved }));
}
#[test]
fn actual_http_success_with_failed_receipt_publication_retains_charge_and_cannot_be_replayed() {
    let (f, a, g, _p, o, _main) = setup("ollama");
    let blocked = a.result_root().unwrap(); fs::write(&blocked, b"occupied").unwrap();
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o600)).unwrap();
    let (server, worker) = fixture(vec![("application/json", local_metadata().to_string()), ("text/event-stream", stream("Known", true))]);
    assert!(run(o.respond_once(None, &g, &server, "once", &crate::pet_config::defaults())).unwrap_err().contains("publicação"));
    assert_eq!(worker.join().unwrap().len(), 2);
    assert_eq!(ledger(&f, &g)["attempts"][0]["phase"], "settled");
    fs::remove_file(blocked).unwrap();
    assert!(matches!(lookup(&a, &g, "once"), Lookup::RecoveryRequired { phase: RecoveryPhase::SettledReceiptMissing }));
    let server = crate::local_chat::Server { id: "ollama", name: "No IO", url: "http://127.0.0.1:1".into(), key: None };
    assert!(run(o.respond_once(None, &g, &server, "once", &crate::pet_config::defaults())).unwrap_err().contains("recuperação"));
    assert_eq!(ledger(&f, &g)["attempts"].as_array().unwrap().len(), 1);
}
#[test]
fn receipt_bytes_are_immutable_and_changed_text_or_bindings_cannot_be_signed_from_ui_data() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let call = settled(&o, &g, "once", "Known");
    call.record_result("Known", 5).unwrap();
    let file = path(&a, &g, &call); let original = fs::read(&file).unwrap();
    let row = &g.admissions[0];
    let envelope: results::Envelope = serde_json::from_slice(&original).unwrap();
    results::publish(&a, &row.spec, row.proof.as_ref().unwrap(), &envelope).unwrap();
    assert_eq!(fs::read(&file).unwrap(), original);
    for field in ["text", "goalId", "petId", "model", "requestId", "intentHash", "inputHash", "attemptId", "decisionId"] {
        let mut changed: Value = serde_json::from_slice(&original).unwrap();
        changed["body"][field] = "changed".into();
        fs::write(&file, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(results::lookup(&a, &row.spec, row.proof.as_ref().unwrap(), "once").is_err(), "{field}");
    }
    fs::write(&file, &original).unwrap();
    assert!(matches!(lookup(&a, &g, "once"), Lookup::Available { .. }));
}
#[test]
fn authenticated_receipts_cannot_be_swapped_between_attempts() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let first = settled(&o, &g, "first", "First"); first.record_result("First", 5).unwrap();
    let first_path = path(&a, &g, &first); drop(first);
    let second = settled(&o, &g, "second", "Other"); second.record_result("Other", 5).unwrap();
    let second_path = path(&a, &g, &second);
    fs::write(second_path, fs::read(first_path).unwrap()).unwrap();
    let row = &g.admissions[0];
    assert!(results::lookup(&a, &row.spec, row.proof.as_ref().unwrap(), "second").is_err());
}
#[test]
fn interrupted_publication_never_adopts_a_stage_and_lost_final_ack_is_stabilized_without_rewriting() {
    use crate::pet_runtime::authority_store::Fault;
    for fault in [Fault::BeforeResultPublish, Fault::AfterResultPublish] {
        let (_f, mut a, g, _p, o, _main) = setup("ollama");
        let call = settled(&o, &g, "once", "Known");
        let row = &g.admissions[0]; let attempt = call.attempt.lock().unwrap().as_ref().unwrap().clone();
        let envelope = a.seal_result(&row.spec, row.proof.as_ref().unwrap(), o.lease(), &attempt, "Known", 5, crate::assistant_memory::now()).unwrap();
        a.fault = fault;
        assert!(results::publish(&a, &row.spec, row.proof.as_ref().unwrap(), &envelope).is_err());
        a.fault = Fault::None;
        match fault {
            Fault::BeforeResultPublish => {
                assert!(matches!(lookup(&a, &g, "once"), Lookup::RecoveryRequired { phase: RecoveryPhase::SettledReceiptMissing }));
                assert!(results::recover(&a, &row.spec, row.proof.as_ref().unwrap(), "once").is_err());
                assert!(!path(&a, &g, &call).exists());
            },
            Fault::AfterResultPublish => {
                let file = path(&a, &g, &call); let original = fs::read(&file).unwrap();
                assert_eq!(results::recover(&a, &row.spec, row.proof.as_ref().unwrap(), "once").unwrap(), envelope.body);
                assert_eq!(fs::read(file).unwrap(), original);
            },
            _ => unreachable!(),
        }
    }
}
#[test]
fn oversized_result_data_and_unknown_layouts_are_refused() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let call = settled(&o, &g, "once", "Known"); call.record_result("Known", 5).unwrap();
    let Lookup::Available { receipt } = lookup(&a, &g, "once") else { panic!("missing receipt") };
    let mut changed = receipt.clone(); changed.schema_version = 2; assert!(!changed.valid());
    let mut changed = receipt.clone(); changed.text = "x".repeat(4 * 1024 * 1024 + 1); changed.accepted_output_bytes = changed.text.len() as u64; assert!(!changed.valid());
    let mut raw = serde_json::to_value(receipt).unwrap(); raw["executionAuthority"] = json!(true);
    assert!(serde_json::from_value::<results::ModelResult>(raw).is_err());
}
#[test]
fn unsafe_result_files_are_refused_without_following_links_or_changing_targets() {
    for kind in ["symlink", "hardlink", "public", "malformed"] {
        let (_f, a, g, _p, o, _main) = setup("ollama");
        let call = settled(&o, &g, "once", "Known"); call.record_result("Known", 5).unwrap();
        let file = path(&a, &g, &call); let other = file.with_extension("unowned"); let bytes = fs::read(&file).unwrap();
        match kind {
            "symlink" => { fs::rename(&file, &other).unwrap(); symlink(&other, &file).unwrap(); },
            "hardlink" => fs::hard_link(&file, &other).unwrap(),
            "public" => fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap(),
            _ => fs::write(&file, b"{invalid").unwrap(),
        }
        let row = &g.admissions[0];
        assert!(results::lookup(&a, &row.spec, row.proof.as_ref().unwrap(), "once").is_err(), "{kind}");
        if other.exists() { assert_eq!(fs::read(&other).unwrap(), bytes); }
    }
}
#[test]
fn unconfirmed_or_over_budget_results_cannot_be_sealed() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let call = o.prepare_model("once", &g, 100, crate::assistant_memory::now()).unwrap();
    call.before_send("fixture-free", &body()).unwrap();
    let row = &g.admissions[0]; let attempt = call.attempt.lock().unwrap().as_ref().unwrap().clone();
    assert!(a.seal_result(&row.spec, row.proof.as_ref().unwrap(), o.lease(), &attempt, "Known", 5, crate::assistant_memory::now()).is_err());
    call.finish(Some(5)).unwrap();
    for (text, bytes) in [("", 5), ("Too much text", 5), ("Known", 6)] {
        assert!(a.seal_result(&row.spec, row.proof.as_ref().unwrap(), o.lease(), &attempt, text, bytes, crate::assistant_memory::now()).is_err());
    }
    assert!(!a.result_root().unwrap().exists());
}
#[test]
fn receipt_from_a_different_native_authority_is_not_adopted() {
    let (_f, a, g, _p, o, _main) = setup("ollama");
    let call = settled(&o, &g, "once", "Known"); call.record_result("Known", 5).unwrap();
    let envelope: results::Envelope = serde_json::from_slice(&fs::read(path(&a, &g, &call)).unwrap()).unwrap();
    let (_other, foreign, other_goal, _other_p, _other_o, _other_main) = setup("ollama");
    let row = &other_goal.admissions[0];
    assert!(foreign.verify_result(&row.spec, row.proof.as_ref().unwrap(), &envelope).is_err());
}
#[test]
fn paused_goal_result_survives_another_process_without_resuming_execution() {
    let (f, a, g, p, o, main) = setup("ollama");
    let (server, worker) = fixture(vec![("application/json", local_metadata().to_string()), ("text/event-stream", stream("Recovered result", true))]);
    assert_eq!(run(o.respond_once(None, &g, &server, "recover", &crate::pet_config::defaults())).unwrap(), "Recovered result");
    assert_eq!(worker.join().unwrap().len(), 2);
    let now = crate::assistant_memory::now();
    let stopping = mutate_with_authority(&f.store(), &root(&f), &g.id, &g.spec.owner.pet_id, &g.revision, "pause".into(), Action::Pause, now).unwrap().record;
    assert_eq!(stopping.state, State::Stopping);
    let drain = o.drain(DrainOutcome::Confirmed).unwrap();
    assert_eq!(f.store().drain_owned(&a, &drain, now).unwrap().record.state, State::Paused);
    drop(drain); drop(o); drop(p); drop(main); drop(a);
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "pet_runtime::lease_controller::writer::tests::result_tests::receipt_read_child", "--ignored"])
        .env("COUCOU_RESULT_TEST_ROOT", &f.path).env("COUCOU_RESULT_TEST_GOAL", &g.id)
        .output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stdout));
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
}
#[test]
#[ignore = "private child process fixture; executed explicitly by its parent test"]
fn receipt_read_child() {
    let profile = std::path::PathBuf::from(std::env::var_os("COUCOU_RESULT_TEST_ROOT").expect("private test root required"));
    let goal_id = std::env::var("COUCOU_RESULT_TEST_GOAL").unwrap();
    let store = crate::pet_runtime::store::Store::existing(&profile.join("goals")).unwrap().unwrap();
    let goal = store.read(&goal_id, "ai_ollama").unwrap();
    assert_eq!(goal.state, State::Paused);
    let authority = Authority::existing(&profile.join("authority")).unwrap();
    let view: results::View = lookup(&authority, &goal, "recover").into();
    assert!(!view.criteria_verified && !view.goal_complete);
    let Lookup::Available { receipt } = view.result else { panic!("missing receipt") };
    assert_eq!(receipt.text, "Recovered result");
    assert!(authority.verify_queued(&goal, crate::assistant_memory::now()).is_err());
}
#[test]
fn result_command_does_not_accept_receipt_text_or_authority_from_the_caller() {
    let valid = json!({"petId":"ai_ollama","goalId":"goal","decisionId":"decision","requestId":"request"});
    assert!(serde_json::from_value::<crate::pet_runtime::result_api::ResultInput>(valid.clone()).is_ok());
    for field in ["text", "receipt", "verified", "run", "lease", "authority"] {
        let mut input = valid.clone(); input[field] = json!(true);
        assert!(serde_json::from_value::<crate::pet_runtime::result_api::ResultInput>(input).is_err());
    }
}
