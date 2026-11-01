//! Invalid storage must not spend an attempt or disclose a model's input.
use super::*;
use crate::pet_runtime::{results, store::protocol_directory};
use std::{fs, os::unix::fs::{PermissionsExt, symlink}, path::{Path, PathBuf}};

const PROTOCOL: &[u8] = b"{\"version\":1,\"kind\":\"coucou-model-results\"}\n";

fn bytes(path: &Path) -> Vec<(PathBuf, u32, Vec<u8>)> {
    fn visit(root: &Path, path: &Path, rows: &mut Vec<(PathBuf, u32, Vec<u8>)>) {
        let m = fs::symlink_metadata(path).unwrap();
        let data = if m.is_symlink() { fs::read_link(path).unwrap().as_os_str().as_encoded_bytes().to_vec() }
            else if m.is_file() { fs::read(path).unwrap() } else { vec![] };
        rows.push((path.strip_prefix(root).unwrap().into(), m.permissions().mode(), data));
        if m.is_dir() { for entry in fs::read_dir(path).unwrap() { visit(root, &entry.unwrap().path(), rows); } }
    }
    let mut rows = vec![]; visit(path, path, &mut rows); rows.sort(); rows
}
fn incompatible(a: &Authority, g: &Record, kind: &str) {
    let root = a.result_root().unwrap();
    let target = if let Some(_) = kind.strip_prefix("decision-") {
        protocol_directory(&root, PROTOCOL, true).unwrap();
        root.join(&g.admissions[0].spec.decision_id)
    } else { root };
    let variant = kind.strip_prefix("decision-").unwrap_or(kind);
    match variant {
        "file" => { fs::write(&target, b"occupied").unwrap(); fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap(); },
        "symlink" => {
            let other = target.with_extension("unrelated"); fs::create_dir(&other).unwrap();
            fs::set_permissions(&other, fs::Permissions::from_mode(0o700)).unwrap();
            fs::write(other.join("private-note"), b"do not follow or modify").unwrap(); symlink(other, target).unwrap();
        },
        "public" => {
            protocol_directory(&target, PROTOCOL, true).unwrap();
            fs::set_permissions(target, fs::Permissions::from_mode(0o755)).unwrap();
        },
        "protocol" => {
            protocol_directory(&target, PROTOCOL, true).unwrap();
            fs::write(target.join(".protocol.json"), b"{\"version\":99}\n").unwrap();
        },
        "untyped" => {
            fs::create_dir(&target).unwrap(); fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
            fs::write(target.join("opaque-note.bin"), b"preserve unknown bytes").unwrap();
        },
        _ => panic!("unknown fixture kind"),
    }
}

#[test]
fn incompatible_result_sinks_refuse_before_http_and_leave_all_bytes_and_budget_unchanged() {
    for kind in ["file", "untyped", "protocol", "symlink", "public", "decision-file", "decision-untyped", "decision-protocol", "decision-symlink", "decision-public"] {
        let (f, a, g, _p, o, _main) = setup("ollama"); incompatible(&a, &g, kind);
        let before = bytes(&f.path);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap(); listener.set_nonblocking(true).unwrap();
        let server = crate::local_chat::Server { id: "ollama", name: "Must not receive even metadata", url: format!("http://{}", listener.local_addr().unwrap()), key: None };
        let error = run(o.respond_once(None, &g, &server, "never-dispatched", &crate::pet_config::defaults())).unwrap_err();
        assert!(error.contains("armazenamento dos resultados"), "{kind}: {error}");
        assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock, "{kind}");
        assert!(ledger(&f, &g).is_null()); assert_eq!(bytes(&f.path), before, "{kind}"); assert_eq!(o.workers(), 0);
    }
}

#[test]
fn final_body_gate_checks_storage_for_every_common_owner_before_creating_an_attempt() {
    for provider in ["codex", "antigravity", "copilot", "ollama", "lmstudio", "integration_notion"] {
        let (f, a, g, _p, o, _main) = if provider == "lmstudio" {
            let server = crate::local_chat::Server { id: "lmstudio", name: "Synthetic accounting only", url: "http://127.0.0.1:1234".into(), key: None };
            setup_lmstudio(&server, &lmstudio_metadata())
        } else { setup(provider) };
        let call = o.prepare_model("exact-body", &g, 100, crate::assistant_memory::now()).unwrap();
        incompatible(&a, &g, "untyped"); let before = bytes(&f.path);
        assert!(call.before_send("fixture-free", &body()).unwrap_err().contains("armazenamento dos resultados"));
        assert!(call.attempt.lock().unwrap().is_none()); assert!(!call.dispatched.load(Ordering::Acquire));
        assert!(ledger(&f, &g).is_null()); assert_eq!(bytes(&f.path), before);
        assert!(call.before_send("fixture-free", &body()).is_err()); // single-use call, not implicit retry
    }
}

#[test]
fn sink_changed_during_metadata_is_refused_before_budget_and_inference_on_both_local_engines() {
    for provider in ["ollama", "lmstudio"] {
        let destination = Arc::new(Mutex::new(None::<PathBuf>)); let mutated = destination.clone();
        let metadata = if provider == "ollama" { local_metadata() } else { lmstudio_metadata() };
        let (mut server, http) = fixture_with_hook(vec![("application/json", metadata.to_string())], move |path| {
            assert!(matches!(path, "/api/show" | "/api/v1/models"));
            let path = mutated.lock().unwrap().clone().unwrap(); fs::create_dir(&path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            fs::write(path.join("opaque.bin"), b"not a results protocol").unwrap();
        }); server.id = provider;
        let (f, a, g, _p, o, _main) = if provider == "ollama" { setup(provider) } else { setup_lmstudio(&server, &metadata) };
        *destination.lock().unwrap() = Some(a.result_root().unwrap());
        let config = if provider == "lmstudio" { lmstudio_config() } else { crate::pet_config::defaults() };
        let error = run(o.respond_once(None, &g, &server, "metadata-cut", &config)).unwrap_err();
        assert!(error.contains("armazenamento dos resultados"), "{provider}: {error}");
        assert_eq!(http.join().unwrap().len(), 1); assert!(ledger(&f, &g).is_null()); assert_eq!(o.workers(), 0);
        let root = a.result_root().unwrap(); assert!(!root.join(".protocol.json").exists());
        assert_eq!(fs::read(root.join("opaque.bin")).unwrap(), b"not a results protocol");
    }
}

#[test]
fn valid_missing_and_previous_round_sinks_remain_read_only_and_do_not_block_later_rounds() {
    let (f, a, g, _p, o, _main) = setup("ollama"); let decision = &g.admissions[0].spec.decision_id;
    let before = bytes(&f.path); results::preflight(&a, decision).unwrap(); assert_eq!(bytes(&f.path), before);
    assert!(!a.result_root().unwrap().exists());
    for request in ["first-round", "second-round"] {
        let call = o.prepare_model(request, &g, 100, crate::assistant_memory::now()).unwrap();
        call.before_send("fixture-free", &body()).unwrap(); call.finish(Some(5)).unwrap(); call.record_result("Known", 5).unwrap();
        let before = bytes(&f.path); results::preflight(&a, decision).unwrap(); assert_eq!(bytes(&f.path), before);
        let row = &g.admissions[0]; assert!(matches!(results::lookup(&a, &row.spec, row.proof.as_ref().unwrap(), request).unwrap(), results::Lookup::Available { .. }));
    }
    assert_eq!(ledger(&f, &g)["attempts"].as_array().unwrap().len(), 2);
}
