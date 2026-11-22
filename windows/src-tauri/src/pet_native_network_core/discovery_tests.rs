use super::*;
use std::{os::unix::fs::{PermissionsExt, symlink}, sync::atomic::{AtomicU64, Ordering}};

struct Fixture { root: PathBuf, native: Native }
impl Fixture {
    fn new(body: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("coucou-dms-discovery-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&root).unwrap(); fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let program = root.join("fixture-dms");
        let header = "#!/usr/bin/python3 -I\nimport json,os,pathlib,subprocess,sys,time\nassert sys.argv[1:]==['ipc','call','pets-system','backendEnvironment']\nassert sys.stdin.read()==''\nassert 'DMS_SOCKET' not in os.environ\n";
        fs::write(&program, format!("{header}{body}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        Self { root, native: Native::fixture(program, unsafe { libc::geteuid() }) }
    }
    fn success() -> Self {
        Self::new("print(json.dumps({'schemaVersion':1,'dmsSocket':str(pathlib.Path(__file__).parent/'dms-123.sock')}))")
    }
    async fn endpoint(&self) -> Result<PathBuf, Error> {
        self.native.endpoint(&self.root, Instant::now() + Duration::from_secs(3)).await
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }
fn dead(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat")).ok()
        .is_none_or(|s| s.rsplit_once(") ").is_some_and(|(_, rest)| rest.starts_with('Z')))
}
async fn wait_pid(path: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(pid) = fs::read_to_string(path).ok().and_then(|s| s.parse::<u32>().ok()) { return pid; }
        assert!(Instant::now() < deadline, "fixture child did not start");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

#[test]
fn recovery_native_metadata_is_exact_bounded_and_duplicate_free() {
    let runtime = Path::new("/run/user/1000");
    for name in ["dms-123.sock", "danklinux-123.sock"] {
        let value = serde_json::json!({"schemaVersion":1,"dmsSocket":format!("/run/user/1000/{name}")});
        assert_eq!(parse(value.to_string().as_bytes(), runtime).unwrap(), runtime.join(name));
    }
    for bytes in [b"".to_vec(), vec![b' ';4097], b"[]".to_vec(), b"{}".to_vec(),
        b"{\"schemaVersion\":true,\"dmsSocket\":\"/run/user/1000/dms-1.sock\"}".to_vec(),
        b"{\"schemaVersion\":1,\"schemaVersion\":1,\"dmsSocket\":\"/run/user/1000/dms-1.sock\"}".to_vec(),
        b"{\"schemaVersion\":1,\"dmsSocket\":\"/run/user/1000/dms-1.sock\",\"extra\":true}".to_vec(),
        b"{\"schemaVersion\":1,\"dmsSocket\":false}".to_vec()] {
        assert!(parse(&bytes, runtime).is_err());
    }
    for path in ["/tmp/dms-1.sock", "/run/user/1001/dms-1.sock", "/run/user/1000/dms-１２.sock",
        "/run/user/1000/dms-.sock", "/run/user/1000/sub/dms-1.sock", "/run/user/1000/./dms-1.sock",
        "/run/user/1000//dms-1.sock", "/run/user/1000/other-1.sock"] {
        let value = serde_json::json!({"schemaVersion":1,"dmsSocket":path});
        assert_eq!(parse(value.to_string().as_bytes(), runtime).err(), Some(Error::UntrustedPeer));
    }
}

#[test]
fn recovery_missing_inherited_endpoint_defers_discovery_without_inventing_a_path() {
    let c = super::super::Config::from_inherited(1000, None).unwrap();
    assert!(c.socket.as_os_str().is_empty()); assert!(c.discovery.is_some());
    assert_eq!(c.runtime, Path::new("/run/user/1000"));
    for path in ["/tmp/dms-1.sock", "/run/user/1001/dms-1.sock", "/run/user/1000/unreviewed.sock"] {
        assert_eq!(super::super::Config::from_inherited(1000, Some(path.into())).err(), Some(Error::UntrustedPeer));
    }
}

#[tokio::test]
async fn recovery_helper_uses_fixed_literal_argv_null_stdin_and_no_inherited_daemon_socket() {
    let f = Fixture::success(); assert_eq!(f.endpoint().await.unwrap(), f.root.join("dms-123.sock"));
    // Parsed metadata alone is never remembered as an authenticated endpoint.
    assert!(f.native.remembered().is_none());
}

#[tokio::test]
async fn recovery_helper_owner_permissions_and_symlinks_are_checked_before_spawn() {
    let f = Fixture::success();
    let mut wrong = f.native.clone(); wrong.owner = wrong.owner.saturating_add(1);
    assert_eq!(wrong.endpoint(&f.root, Instant::now()+Duration::from_secs(1)).await.err(), Some(Error::UntrustedPeer));
    fs::set_permissions(&f.native.executable, fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(f.endpoint().await.err(), Some(Error::UntrustedPeer));
    fs::set_permissions(&f.native.executable, fs::Permissions::from_mode(0o700)).unwrap();
    let alias = f.root.join("alias"); symlink(&f.native.executable, &alias).unwrap();
    wrong.executable = alias; wrong.owner = f.native.owner;
    assert_eq!(wrong.endpoint(&f.root, Instant::now()+Duration::from_secs(1)).await.err(), Some(Error::UntrustedPeer));
}

#[tokio::test]
async fn recovery_helper_invalid_output_and_failure_never_yield_a_target() {
    for (body, error) in [("print('x'*5000)", Error::Protocol), ("print('{}')", Error::Protocol),
        ("print('[]')", Error::Protocol), ("print('not json')", Error::Protocol),
        ("print('{}');sys.exit(7)", Error::Unavailable)] {
        let f = Fixture::new(body); assert_eq!(f.endpoint().await.err(), Some(error),"synthetic body={body:?}; diagnostic={:?}",diagnostics());
        assert!(f.native.remembered().is_none());
    }
}

#[tokio::test]
async fn recovery_helper_exec_failure_with_a_still_writable_image_never_yields_a_target(){
 let mut f=Fixture::success();f.native.fixture_interpreter=false;let writer=std::fs::OpenOptions::new().write(true).open(&f.native.executable).unwrap();
 assert_eq!(f.endpoint().await.err(),Some(Error::Unavailable));assert_eq!(diagnostics(),vec![("spawn",libc::ETXTBSY)]);assert!(f.native.remembered().is_none());drop(writer);
}

#[tokio::test]
async fn recovery_helper_timeout_stops_owned_group_and_reaps_leader() {
    let f = Fixture::new("root=pathlib.Path(__file__).parent\n(root/'leader').write_text(str(os.getpid()))\np=subprocess.Popen(['/usr/bin/sleep','30'])\n(root/'descendant').write_text(str(p.pid))\nwhile True: time.sleep(1)");
    let result = f.native.endpoint(&f.root, Instant::now()+Duration::from_millis(400)).await;
    assert_eq!(result.err(), Some(Error::Timeout));
    let leader: u32 = fs::read_to_string(f.root.join("leader")).unwrap().parse().unwrap();
    let descendant: u32 = fs::read_to_string(f.root.join("descendant")).unwrap().parse().unwrap();
    assert!(!Path::new(&format!("/proc/{leader}")).exists());
    // SIGKILL and leader wait do not schedule the descendant atomically.
    // Match the bounded observation already used by the normal-exit case.
    let deadline=Instant::now()+Duration::from_secs(1);
    while !dead(descendant) { assert!(Instant::now()<deadline,"owned helper descendant survived timeout cleanup"); tokio::time::sleep(Duration::from_millis(5)).await; }
}

#[tokio::test]
async fn recovery_cancelling_metadata_future_cleans_child_and_releases_single_slot() {
    let f = Fixture::new("root=pathlib.Path(__file__).parent\nif (root/'once').exists():\n print(json.dumps({'schemaVersion':1,'dmsSocket':str(root/'dms-123.sock')}))\nelse:\n (root/'once').write_text('1')\n (root/'leader').write_text(str(os.getpid()))\n while True: time.sleep(1)");
    let native = f.native.clone(); let runtime = f.root.clone();
    let task = tokio::spawn(async move { native.endpoint(&runtime, Instant::now()+Duration::from_secs(3)).await });
    let pid = wait_pid(&f.root.join("leader")).await;
    // A queued caller has the same slot but cannot stop the current child.
    assert_eq!(f.native.endpoint(&f.root, Instant::now()+Duration::from_millis(60)).await.err(), Some(Error::Timeout));
    assert!(!dead(pid));
    task.abort(); let _ = task.await;
    assert!(!Path::new(&format!("/proc/{pid}")).exists());
    // Reuse the SAME Native/slot; another fixture would not prove release.
    assert!(f.endpoint().await.is_ok());
}

#[tokio::test]
async fn recovery_normal_helper_exit_stops_children_even_after_they_close_stdout() {
    let f = Fixture::new("root=pathlib.Path(__file__).parent\np=subprocess.Popen(['/usr/bin/sleep','30'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)\n(root/'descendant').write_text(str(p.pid))\nprint(json.dumps({'schemaVersion':1,'dmsSocket':str(root/'dms-123.sock')}))");
    assert!(f.endpoint().await.is_ok());
    let pid: u32 = fs::read_to_string(f.root.join("descendant")).unwrap().parse().unwrap();
    // killpg queues SIGKILL; the kernel may schedule the descendant after
    // the helper leader has already been reaped. Observe its termination
    // within a bound rather than assuming both become dead atomically.
    let deadline=Instant::now()+Duration::from_secs(1);
    while !dead(pid) { assert!(Instant::now()<deadline,"owned helper descendant survived cleanup"); tokio::time::sleep(Duration::from_millis(5)).await; }
}

#[tokio::test]
async fn recovery_helper_replacement_cannot_publish_an_old_executable_result() {
    let f = Fixture::new("root=pathlib.Path(__file__).parent\n(root/'leader').write_text(str(os.getpid()))\ntime.sleep(0.15)\nprint(json.dumps({'schemaVersion':1,'dmsSocket':str(root/'dms-123.sock')}))");
    let native = f.native.clone(); let runtime = f.root.clone();
    let task = tokio::spawn(async move { native.endpoint(&runtime, Instant::now()+Duration::from_secs(3)).await });
    let pid = wait_pid(&f.root.join("leader")).await;
    fs::write(&f.native.executable, "#!/usr/bin/python3 -I\nraise SystemExit(1)\n").unwrap();
    assert_eq!(task.await.unwrap().err(), Some(Error::OriginChanged));
    assert!(!Path::new(&format!("/proc/{pid}")).exists());
}

#[tokio::test]
async fn recovery_exited_helper_with_inherited_stdout_holder_has_bounded_cleanup() {
    let f = Fixture::new("root=pathlib.Path(__file__).parent\np=subprocess.Popen(['/usr/bin/sleep','30'])\n(root/'descendant').write_text(str(p.pid))\nprint(json.dumps({'schemaVersion':1,'dmsSocket':str(root/'dms-123.sock')}))");
    assert_eq!(f.endpoint().await.err(), Some(Error::Timeout), "inherited stdout diagnostic={:?}", diagnostics());
    let pid:u32=fs::read_to_string(f.root.join("descendant")).unwrap().parse().unwrap();
    let deadline=Instant::now()+Duration::from_secs(1);
    while !dead(pid) { assert!(Instant::now()<deadline); tokio::time::sleep(Duration::from_millis(5)).await; }
}

#[tokio::test]
#[ignore = "manual opt-in native UI metadata read; no network or Bluetooth action"]
async fn recovery_native_ui_metadata_probe() {
    assert_eq!(std::env::var("COUCOU_NATIVE_DMS_METADATA_PROBE").as_deref(), Ok("1"));
    let uid=unsafe{libc::geteuid()}; let runtime=PathBuf::from(format!("/run/user/{uid}"));
    let native=Native::current(); let socket=native.endpoint(&runtime,Instant::now()+Duration::from_secs(3)).await.unwrap();
    use std::os::unix::fs::FileTypeExt;
    let m=fs::symlink_metadata(&socket).unwrap();
    assert!(m.file_type().is_socket()); assert_eq!(m.uid(),uid); assert_eq!(m.mode()&0o777,0o600);
    assert!(native.remembered().is_none());
    println!("{}",serde_json::json!({"nativeMetadataConfirmed":true,"socketName":socket.file_name().unwrap().to_str().unwrap(),"socketUid":uid,"deviceRequests":0,"modelCalls":0}));
}
