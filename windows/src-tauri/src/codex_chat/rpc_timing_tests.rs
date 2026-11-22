use super::*;
use super::timing::{Entry, Report, Trace};
use std::path::{Path, PathBuf};

const SERVER: &str = r#"import json,os,pathlib,sys,time
root=pathlib.Path('.')
mode=(root/'mode').read_text()
(root/'pid').write_text(str(os.getpid()))
def send(v): print(json.dumps(v),flush=True)
for line in sys.stdin:
 v=json.loads(line);m=v.get('method');p=v.get('params',{})
 with (root/'requests.jsonl').open('a') as f:f.write(json.dumps(v)+'\n')
 if m=='initialized':continue
 if m=='initialize':
  (root/'initialize.ready').write_text('ready')
  if mode=='hang':
   while True:time.sleep(1)
 result={}
 if m=='account/read':result={'account':{'type':'chatgpt'}}
 if m=='config/read':
  if mode=='error':send({'id':v['id'],'error':{'message':'PRIVATE_NATIVE_ERROR_SECRET'}});continue
  result={'config':{'developer_instructions':'PRIVATE_NATIVE_INSTRUCTIONS_SECRET'}}
 if m in ('thread/start','thread/resume'):result={'thread':{'id':'fixture-thread','name':None}}
 if m=='turn/start':
  send({'method':'item/agentMessage/delta','params':{'threadId':'fixture-thread','turnId':'fixture-turn','itemId':'a','delta':'PRIVATE_EARLY_ANSWER_SECRET'}})
  result={'turn':{'id':'fixture-turn'}}
 if m is not None and 'id' in v:send({'id':v['id'],'result':result})
 if m=='turn/start':
  send({'method':'item/agentMessage/delta','params':{'threadId':'foreign-thread','turnId':'foreign-turn','itemId':'foreign','delta':'PRIVATE_FOREIGN_SECRET'}})
  if mode=='tool':send({'id':77,'method':'item/tool/call','params':{'threadId':'fixture-thread','turnId':'fixture-turn','callId':'fixture-call','tool':'fixture_tool','arguments':{'private':'PRIVATE_TOOL_ARGS_SECRET'}}})
  else:send({'id':78,'method':'item/commandExecution/requestApproval','params':{'threadId':'fixture-thread','turnId':'fixture-turn','command':'PRIVATE_APPROVAL_COMMAND_SECRET'}})
 if 'method' not in v and v.get('id')==77:
  send({'id':78,'method':'item/commandExecution/requestApproval','params':{'threadId':'fixture-thread','turnId':'fixture-turn','command':'PRIVATE_APPROVAL_COMMAND_SECRET'}})
 if 'method' not in v and v.get('id')==78:
  send({'method':'item/completed','params':{'threadId':'fixture-thread','turnId':'fixture-turn','item':{'type':'agentMessage','id':'a','text':'PRIVATE_FINAL_ANSWER_SECRET'}}})
  send({'method':'turn/completed','params':{'threadId':'fixture-thread','turnId':'fixture-turn','turn':{'id':'fixture-turn','status':'completed'}}})
"#;
struct Fixture { cwd: PathBuf, exe: PathBuf }
impl Fixture {
    fn new(mode: &str) -> Self {
        let (cwd, exe) = super::tests::mock();
        std::fs::write(cwd.join("app-server"), SERVER).unwrap();
        std::fs::write(cwd.join("mode"), mode).unwrap();
        Self { cwd, exe }
    }
    fn frames(&self) -> Vec<Value> {
        std::fs::read_to_string(self.cwd.join("requests.jsonl")).unwrap_or_default()
            .lines().map(|s| serde_json::from_str(s).unwrap()).collect()
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.cwd); } }
fn wait_file(path: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(s) = std::fs::read_to_string(path) { if !s.is_empty() { return s; } }
        assert!(Instant::now() < deadline, "Synthetic readiness deadline");
        std::thread::sleep(Duration::from_millis(5));
    }
}
// The guard also cleans up if a readiness assertion or deadline check fails.
struct Attempt<T> {
    worker: Option<std::thread::JoinHandle<()>>, result: mpsc::Receiver<T>,
    cancel: Arc<AtomicBool>, pid_path: PathBuf,
}
impl<T: Send + 'static> Attempt<T> {
    fn new(f: &Fixture, cancel: Arc<AtomicBool>, run: impl FnOnce() -> T + Send + 'static) -> Self {
        let (tx, result) = mpsc::channel();
        Self { worker: Some(std::thread::spawn(move || { let _ = tx.send(run()); })),
            result, cancel, pid_path: f.cwd.join("pid") }
    }
    fn finish(mut self) -> T {
        let result = self.result.recv_timeout(Duration::from_secs(3)).expect("Synthetic worker deadline");
        self.worker.take().unwrap().join().unwrap(); result
    }
}
impl<T> Drop for Attempt<T> {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            self.cancel.store(true, Ordering::Release);
            if !worker.is_finished() {
                if let Some(pid) = std::fs::read_to_string(&self.pid_path).ok()
                    .and_then(|s|s.trim().parse::<libc::pid_t>().ok()).filter(|pid|*pid > 1) {
                    unsafe { libc::kill(pid, libc::SIGKILL); }
                }
            }
            let _ = worker.join();
        }
    }
}
fn has(report: &Report, stage: &str, point: &str) -> bool {
    serde_json::to_value(report).unwrap()["events"].as_array().unwrap().iter()
        .any(|v| v["stage"] == stage && v["point"] == point)
}
fn assert_metadata_only(report: &Report) {
    let v = serde_json::to_value(report).unwrap();
    assert_eq!(v.as_object().unwrap().len(), 6);
    let data = serde_json::to_string(report).unwrap();
    for secret in ["PRIVATE_", "fixture-thread", "fixture-turn", "foreign", "fixture_tool", "model", "params", "command", "cwd"] {
        assert!(!data.contains(secret), "Payload/identity escaped timing record: {secret}");
    }
    assert!(data.len() < 32 * 1024);
    let events = v["events"].as_array().unwrap();
    assert!(events.windows(2).all(|p| p[0]["atMicros"].as_u64() <= p[1]["atMicros"].as_u64()));
}
fn answer(client: &mut Client, resume: bool, history: &[Value], tool: bool) -> Result<Reply, String> {
    let tools = if tool { vec![json!({"type":"function","name":"fixture_tool","inputSchema":{"type":"object"}})] } else { vec![] };
    client.answer(TurnOptions {
        model: "PRIVATE_MODEL_SECRET", system: "PRIVATE_SYSTEM_SECRET", history,
        query: "PRIVATE_QUERY_SECRET", tools: &tools,
        session_id: resume.then_some("fixture-thread"), full_access: false,
        effort: Some("low"), name: Some("Mochi · fixture"),
    }, |_, _| ("PRIVATE_TOOL_RESULT_SECRET".into(), false), |_, _| Some(json!({"decision":"decline"})), |_| {}, |_| {})
}

#[test]
fn traced_and_untraced_clients_emit_identical_frames_and_reply() {
    for tool in [false, true] {
        let mut runs = Vec::new();
        for enabled in [false, true] {
            let f = Fixture::new(if tool { "tool" } else { "normal" });
            let trace = Trace::new(Entry::Direct);
            let mut client = if enabled {
                Client::start_with_cancel_timed(&f.exe, &f.cwd, Duration::from_secs(5), true, Arc::default(), trace.clone()).unwrap()
            } else {
                Client::start_with_cancel(&f.exe, &f.cwd, Duration::from_secs(5), true, Arc::default()).unwrap()
            };
            let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
            let observer = seen.clone();
            client.observe(move |v| observer.lock().unwrap().push(v["method"].clone()));
            let reply = answer(&mut client, false, &[], tool).unwrap();
            let pid = client.child.id() as libc::pid_t;
            drop(client);
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            let observed = seen.lock().unwrap().clone();
            runs.push((f.frames(), reply.text, reply.acted, observed));
            if enabled {
                trace.finish(&Ok::<_, ()>(()));
                let report = trace.report();
                for stage in ["spawn", "initialize", "accountRead", "configRead", "threadStart", "turnStart", "answer", "cleanup"] {
                    assert!(has(&report, stage, "started"), "missing {stage}");
                    assert!(has(&report, stage, "returned"), "unfinished {stage}");
                }
                assert!(has(&report, "firstPublicText", "observed"));
                assert!(has(&report, "acceptedTurnCompletion", "observed"));
                assert!(has(&report, "childReaped", "observed"));
                assert!(has(&report, "nativeRequestCallback", "returned"));
                assert_eq!(has(&report, "dynamicToolCallback", "returned"), tool);
                assert_metadata_only(&report);
            } else { assert!(serde_json::to_value(trace.report()).unwrap()["events"].as_array().unwrap().is_empty()); }
        }
        assert_eq!(runs[0], runs[1]);
    }
}
#[test]
fn resume_and_history_spans_follow_the_actual_optional_rpc_sequence() {
    for resume in [false, true] {
        let f = Fixture::new("normal");
        let trace = Trace::new(Entry::Direct);
        let mut client = Client::start_with_cancel_timed(&f.exe, &f.cwd, Duration::from_secs(5), true, Arc::default(), trace.clone()).unwrap();
        let result = answer(&mut client, resume, &[json!({"role":"user","content":"PRIVATE_HISTORY_SECRET"})], false);
        assert!(result.is_ok());drop(client);trace.finish(&result);
        let r = trace.report();
        assert_eq!(has(&r, "threadResume", "returned"), resume);
        assert_eq!(has(&r, "threadStart", "returned"), !resume);
        assert_eq!(has(&r, "historyInject", "returned"), !resume);
        assert_metadata_only(&r);
    }
}
#[test]
fn native_failure_is_a_fixed_failed_point_without_error_content() {
    let f = Fixture::new("error");let t = Trace::new(Entry::Direct);
    let mut client = Client::start_with_cancel_timed(&f.exe, &f.cwd, Duration::from_secs(5), true, Arc::default(), t.clone()).unwrap();
    let result = answer(&mut client, false, &[], false);assert!(result.is_err());
    drop(client);t.finish(&result);let r = t.report();
    assert!(has(&r, "configRead", "failed"));
    assert!(!has(&r, "threadStart", "started"));
    assert!(has(&r, "cleanup", "returned"));assert_metadata_only(&r);
}
#[test]
fn trace_does_not_start_a_pre_cancelled_child() {
    let f = Fixture::new("normal");let t = Trace::new(Entry::Direct);
    let result = Client::start_with_cancel_timed(&f.cwd.join("nonexistent"), &f.cwd, Duration::from_secs(5), true, Arc::new(AtomicBool::new(true)), t.clone());
    assert!(result.is_err());t.finish(&result);
    assert!(!f.cwd.join("pid").exists());
    assert!(!has(&t.report(), "spawn", "started"));assert_metadata_only(&t.report());
}
#[test]
fn trace_preserves_cancel_during_first_initialize_and_owned_reaping() {
    let f = Fixture::new("hang");let t = Trace::new(Entry::Direct);let cancel = Arc::new(AtomicBool::new(false));
    let (exe, cwd, flag, trace) = (f.exe.clone(), f.cwd.clone(), cancel.clone(), t.clone());
    let attempt = Attempt::new(&f, cancel.clone(), move || Client::start_with_cancel_timed(&exe, &cwd, Duration::from_secs(1800), true, flag, trace).map(|_| ()));
    wait_file(&f.cwd.join("initialize.ready"));let pid: libc::pid_t = wait_file(&f.cwd.join("pid")).parse().unwrap();
    cancel.store(true, Ordering::Release);
    let result = attempt.finish();assert!(result.is_err());
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    t.finish(&result);let r = t.report();
    assert!(has(&r, "initialize", "failed"));assert!(has(&r, "childReaped", "observed"));
    assert!(!has(&r, "accountRead", "started"));assert_metadata_only(&r);
}
#[test]
fn trace_preserves_initialization_cap_and_original_task_deadline() {
    let f = Fixture::new("normal");let t = Trace::new(Entry::Direct);
    let mut client = Client::start_inner_timed(&f.exe, &f.cwd, Duration::from_secs(5), false, None, Duration::from_secs(1), Some(t.clone())).unwrap();
    assert!(client.deadline.saturating_duration_since(Instant::now()) > Duration::from_secs(3));
    assert!(client.request("fixture/privateUnknownMethod", json!({"private":"PRIVATE_PARAMS_SECRET"})).is_ok());
    drop(client);assert_metadata_only(&t.report());
}
#[test]
fn tracing_has_no_observer_during_initialize_or_extra_protocol_requests() {
    let f = Fixture::new("normal");let t = Trace::new(Entry::Direct);
    let mut client = Client::start_with_cancel_timed(&f.exe, &f.cwd, Duration::from_secs(5), true, Arc::default(), t.clone()).unwrap();
    assert!(client.observer.is_none());
    client.cancel_with(Arc::new(AtomicBool::new(true)));
    assert!(client.request("account/read", json!({"refreshToken":false})).is_err());
    drop(client);
    let methods = f.frames().iter().map(|v|v["method"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
    assert_eq!(methods, vec!["initialize", "initialized"]);
    assert!(has(&t.report(), "accountRead", "failed"));assert_metadata_only(&t.report());
}

#[test]
fn traced_initialize_timeout_does_not_inherit_the_thirty_minute_task_budget() {
    let f = Fixture::new("hang");let t = Trace::new(Entry::Direct);let cancel = Arc::new(AtomicBool::new(false));
    let (exe, cwd, flag, trace) = (f.exe.clone(), f.cwd.clone(), cancel.clone(), t.clone());
    let attempt = Attempt::new(&f, cancel, move || Client::start_inner_timed(&exe, &cwd,
        Duration::from_secs(1800), true, Some(flag), Duration::from_secs(1), Some(trace)).map(|_| ()));
    let pid: libc::pid_t = wait_file(&f.cwd.join("pid")).parse().unwrap();
    let result = attempt.finish();assert!(result.is_err());t.finish(&result);
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    assert!(has(&t.report(), "initialize", "failed"));
    assert!(has(&t.report(), "childReaped", "observed"));assert_metadata_only(&t.report());
}
