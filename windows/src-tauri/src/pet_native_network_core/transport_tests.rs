use super::*;
use std::future::Future;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;

struct Fixture {
    root: PathBuf,
    config: Config,
    requests: Arc<Mutex<Vec<Value>>>,
    server: tokio::task::JoinHandle<()>,
}

impl Fixture {
    fn start<F, Fut>(response: F) -> Self
    where
        F: FnOnce(UnixStream, Value, PathBuf) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("mochi-system-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = root.join("danklinux-123.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let executable = root.join("trusted-executable");
        symlink(std::env::current_exe().unwrap(), &executable).unwrap();
        let config = Config {
            socket, runtime: root.clone(), uid: unsafe { libc::geteuid() },
            trusted_executable: executable, trusted_executable_uid: unsafe { libc::geteuid() },
            deadline: Duration::from_secs(2),
        };
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let directory = root.clone();
        let server = tokio::spawn(async move {
            let Ok((mut stream, _)) = listener.accept().await else { return; };
            let mut data = Vec::new();
            let mut buffer = [0u8; 1024];
            while !data.contains(&b'\n') {
                let Ok(count) = stream.read(&mut buffer).await else { return; };
                if count == 0 { return; }
                data.extend_from_slice(&buffer[..count]);
                assert!(data.len() < 8192);
            }
            let request: Value = serde_json::from_slice(data.split(|b| *b == b'\n').next().unwrap()).unwrap();
            captured.lock().unwrap().push(request.clone());
            response(stream, request, directory).await;
        });
        Self { root, config, requests, server }
    }

    fn bytes(bytes: Vec<u8>) -> Self {
        Self::start(move |mut stream, request, _| async move {
            let text = String::from_utf8(bytes).unwrap().replace("REQUEST_ID", &request["id"].to_string());
            let _ = stream.write_all(text.as_bytes()).await;
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn greeting_and_response_are_correlated_without_executing_any_action() {
    let f = Fixture::bytes(b"{\"apiVersion\":29,\"capabilities\":[\"network\"]}\n{\"id\":REQUEST_ID,\"result\":{\"wifiEnabled\":true}}\n".to_vec());
    let result = network_snapshot(&f.config).await.unwrap();
    assert_eq!(result.api_version, Some(29));
    assert_eq!(result.capabilities, vec!["network"]);
    assert_eq!(result.result, json!({"wifiEnabled":true}));
    let requests = f.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["method"], "network.getState");
    assert_eq!(requests[0].as_object().unwrap().len(), 2);
    assert!(requests[0].get("params").is_none());
}

#[tokio::test]
async fn a_different_request_id_cannot_supply_the_result() {
    let f = Fixture::bytes(b"{\"id\":\"other\",\"result\":{\"wrong\":true}}\n{\"id\":REQUEST_ID,\"result\":{\"correct\":true}}\n".to_vec());
    assert_eq!(network_snapshot(&f.config).await.unwrap().result, json!({"correct":true}));
}

#[tokio::test]
async fn fragmented_utf8_and_lines_survive_transport_boundaries() {
    let f = Fixture::start(|mut stream, request, _| async move {
        let mut bytes = serde_json::to_vec(&json!({"id":request["id"],"result":{"label":"Ligação café"}})).unwrap();
        bytes.push(b'\n');
        for chunk in bytes.chunks(3) {
            if stream.write_all(chunk).await.is_err() { break; }
            tokio::task::yield_now().await;
        }
    });
    assert_eq!(network_snapshot(&f.config).await.unwrap().result["label"], "Ligação café");
}

#[tokio::test]
async fn malformed_json_is_unknown_instead_of_a_fabricated_empty_snapshot() {
    let f = Fixture::bytes(b"not-json\n".to_vec());
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Protocol));
}

#[tokio::test]
async fn native_errors_never_leak_the_raw_message() {
    let f = Fixture::bytes(b"{\"id\":REQUEST_ID,\"error\":\"PRIVATE credentials/device detail\"}\n".to_vec());
    let error = network_snapshot(&f.config).await.err().unwrap();
    assert_eq!(error, Error::NativeFailure);
    assert!(!format!("{error:?}").contains("PRIVATE"));
}

#[tokio::test]
async fn missing_result_is_not_success() {
    let f = Fixture::bytes(b"{\"id\":REQUEST_ID}\n".to_vec());
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Protocol));
}

#[tokio::test]
async fn unbounded_payload_is_rejected() {
    let f = Fixture::bytes(vec![b'x'; MAX_BYTES + 1]);
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Protocol));
}

#[tokio::test]
async fn excessive_unrelated_frames_are_rejected() {
    let f = Fixture::bytes(b"{}\n".repeat(MAX_FRAMES + 1));
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Protocol));
}

#[tokio::test]
async fn partial_activity_does_not_restart_the_total_deadline() {
    let mut f = Fixture::start(|mut stream, _, _| async move {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(30)).await;
            if stream.write_all(b" ").await.is_err() { break; }
        }
    });
    f.config.deadline = Duration::from_millis(100);
    let start = Instant::now();
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Timeout));
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn an_expired_budget_never_sends_even_a_metadata_request() {
    let mut f = Fixture::bytes(b"{}\n".to_vec());
    f.config.deadline = Duration::ZERO;
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Timeout));
    assert!(f.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_ready_reply_after_runtime_stall_cannot_be_accepted() {
    let (mut client, mut server) = UnixStream::pair().unwrap();
    server.write_all(b"{\"id\":321,\"result\":{\"ready\":true}}\n").await.unwrap();
    client.readable().await.unwrap();
    let deadline = Instant::now() + Duration::from_millis(20);
    std::thread::sleep(Duration::from_millis(50));
    // This demonstrates the actual Tokio ready-first behaviour underlying R1.
    let inner = timeout_at(deadline, read_reply(&mut client, 321)).await.unwrap();
    assert!(inner.is_ok());
    assert_eq!(finish_before_deadline(inner, deadline).err(), Some(Error::Timeout));
}

#[tokio::test]
async fn wrong_executable_is_rejected_before_any_request() {
    let mut f = Fixture::bytes(b"{}\n".to_vec());
    f.config.trusted_executable = PathBuf::from("/usr/bin/true");
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::UntrustedPeer));
    assert!(f.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_changed_trust_anchor_cannot_confirm_the_reply() {
    let f = Fixture::start(|mut stream, request, directory| async move {
        let path = directory.join("trusted-executable");
        std::fs::remove_file(&path).unwrap();
        symlink("/usr/bin/true", path).unwrap();
        let data = format!("{}\n", json!({"id":request["id"],"result":{}}));
        let _ = stream.write_all(data.as_bytes()).await;
    });
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::UntrustedPeer));
}

#[tokio::test]
async fn insecure_socket_permissions_are_rejected_before_connect() {
    let f = Fixture::bytes(b"{}\n".to_vec());
    std::fs::set_permissions(&f.config.socket, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::UntrustedPeer));
    assert!(f.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn socket_symlinks_are_not_accepted() {
    let mut f = Fixture::bytes(b"{}\n".to_vec());
    let link = f.root.join("danklinux-456.sock");
    symlink(&f.config.socket, &link).unwrap();
    f.config.socket = link;
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::UntrustedPeer));
}

#[tokio::test]
async fn another_runtime_owner_is_not_accepted() {
    let mut f = Fixture::bytes(b"{}\n".to_vec());
    f.config.uid = f.config.uid.saturating_add(1);
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::UntrustedPeer));
}

#[tokio::test]
async fn malformed_greeting_cannot_invent_capabilities() {
    let f = Fixture::bytes(b"{\"apiVersion\":29,\"capabilities\":[true]}\n{\"id\":REQUEST_ID,\"result\":{}}\n".to_vec());
    assert_eq!(network_snapshot(&f.config).await.err(), Some(Error::Protocol));
}

#[test]
fn fixture_peer_incarnation_is_stable() {
    // The real kernel stat is used for every fixture connection above.
    let config = Config {
        socket: PathBuf::new(), runtime: PathBuf::new(), uid: unsafe { libc::geteuid() },
        trusted_executable: std::env::current_exe().unwrap(), trusted_executable_uid: unsafe { libc::geteuid() },
        deadline: Duration::from_secs(1),
    };
    let first = peer_incarnation(std::process::id(), &config).unwrap();
    let second = peer_incarnation(std::process::id(), &config).unwrap();
    assert_eq!(first, second);
    assert!(first.start > 0);
}

#[test]
fn process_stat_start_time_handles_parentheses_in_names() {
    let mut fields = vec!["0"; 20];
    fields[19] = "98765";
    assert_eq!(process_start_time(&format!("123 (name with ) parentheses) {}", fields.join(" "))), Some(98765));
    assert_eq!(process_start_time("short"), None);
    fields[19] = "invalid";
    assert_eq!(process_start_time(&format!("123 (name) {}", fields.join(" "))), None);
}

#[path="../pet_bluetooth/transport_tests.rs"]
mod bluetooth_read_tests;
