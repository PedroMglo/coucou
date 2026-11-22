//! A one-shot worker with a sealed, embedded dependency archive. No venv or
//! package installation is needed on the host. Its stdout is data, never code.
use std::{fs::File, io::{Read, Write}, os::{fd::{AsRawFd, FromRawFd}, unix::process::CommandExt}, process::{Child, Command, Stdio}, sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use serde_json::Value;

const RUNTIME: &[u8] = include_bytes!("runtime.zip");
const WORKER: &str = include_str!("worker.py");
const BOOTSTRAP: &str = "import sys,zoneinfo;sys.path.insert(0,sys.argv[1]);zoneinfo.reset_tzpath(());exec(compile(sys.argv[2],'<coucou-calendar>','exec'),{'__name__':'__main__'})";
const MAX_INPUT: usize = 2 * 1024 * 1024;
const MAX_OUTPUT: usize = 64 * 1024 + 1;
const MAX_ERROR: usize = 4096;
const TIMEOUT: Duration = Duration::from_secs(5);

struct Slot;
static RUNNING: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static TEST_SERIAL:std::sync::Mutex<()>=std::sync::Mutex::new(());
impl Slot {
    fn acquire() -> Result<Self, &'static str> {
        let mut n = RUNNING.load(Ordering::Acquire);
        loop {
            if n >= 2 { return Err("contextBusy"); }
            match RUNNING.compare_exchange_weak(n, n + 1, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return Ok(Self),
                Err(actual) => n = actual,
            }
        }
    }
}
impl Drop for Slot { fn drop(&mut self) { RUNNING.fetch_sub(1, Ordering::Release); } }

struct OwnedChild { child: Child, reaped: bool }
impl Drop for OwnedChild {
    fn drop(&mut self) {
        // The child owns this process group; no other assistant or native CLI
        // is targeted. Always kill and reap on timeout, cancellation or errors.
        if !self.reaped {
            unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL); }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn archive() -> Result<File, &'static str> {
    let fd = unsafe { libc::memfd_create(c"coucou-calendar".as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
    if fd < 0 { return Err("runtimeUnavailable"); }
    let file = unsafe { File::from_raw_fd(fd) };
    let mut file = if fd < 3 {
        // Stdio may be closed in a launcher. Keep the archive away from the
        // descriptors Command assigns to the child's stdin/stdout/stderr.
        let relocated = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };
        if relocated < 0 { return Err("runtimeUnavailable"); }
        drop(file);
        unsafe { File::from_raw_fd(relocated) }
    } else { file };
    let fd = file.as_raw_fd();
    file.write_all(RUNTIME).map_err(|_| "runtimeUnavailable")?;
    let seals = libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    if unsafe { libc::fcntl(fd, libc::F_ADD_SEALS, seals) } < 0 { return Err("runtimeUnavailable"); }
    Ok(file)
}

fn nonblocking(fd: i32) -> Result<(), &'static str> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        Err("workerFailed")
    } else { Ok(()) }
}

fn drain(reader: &mut impl Read, output: &mut Vec<u8>, limit: usize) -> Result<bool, &'static str> {
    let mut bytes = [0; 4096];
    loop {
        match reader.read(&mut bytes) {
            Ok(0) => return Ok(true),
            Ok(n) => {
                if output.len() + n > limit { return Err("workerOutputLimit"); }
                output.extend_from_slice(&bytes[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("workerFailed"),
        }
    }
}

pub(super) fn run(request: &Value, mut current: impl FnMut() -> bool) -> Result<Value, &'static str> {
    run_worker(request, &mut current, WORKER, TIMEOUT, |_| {})
}

fn run_worker(request: &Value, mut current: impl FnMut() -> bool, worker: &str, timeout: Duration, mut spawned: impl FnMut(u32)) -> Result<Value, &'static str> {
    let bytes = serde_json::to_vec(request).map_err(|_| "invalidRequest")?;
    if bytes.len() > MAX_INPUT { return Err("requestLimit"); }
    let _slot = Slot::acquire()?;
    if !current() { return Err("contextRevoked"); }
    let runtime = archive()?;
    let fd = runtime.as_raw_fd();
    let path = format!("/proc/self/fd/{fd}");
    let mut command = Command::new("/usr/bin/python3");
    command.args(["-I", "-S", "-B", "-u", "-c", BOOTSTRAP, &path, worker])
        .env_clear().env("LC_ALL", "C.UTF-8").current_dir("/")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    // Do not duplicate over a guessed descriptor number: Rust also creates an
    // exec-error pipe. Only this already-open descriptor loses CLOEXEC.
    unsafe { command.pre_exec(move || {
        if libc::setpgid(0, 0) < 0 { return Err(std::io::Error::last_os_error()); }
        if libc::fcntl(fd, libc::F_SETFD, 0) < 0 { return Err(std::io::Error::last_os_error()); }
        Ok(())
    }); }
    let mut child = OwnedChild { child: command.spawn().map_err(|_| "runtimeUnavailable")?, reaped: false };
    spawned(child.child.id());
    drop(runtime);
    let mut input = child.child.stdin.take();
    let mut output = child.child.stdout.take().ok_or("workerFailed")?;
    let mut error = child.child.stderr.take().ok_or("workerFailed")?;
    nonblocking(input.as_ref().ok_or("workerFailed")?.as_raw_fd())?;
    nonblocking(output.as_raw_fd())?;
    nonblocking(error.as_raw_fd())?;
    let deadline = Instant::now() + timeout;
    let (mut offset, mut stdout, mut stderr) = (0, Vec::new(), Vec::new());
    loop {
        if !current() { return Err("contextRevoked"); }
        if Instant::now() >= deadline { return Err("workerTimeout"); }
        if let Some(pipe) = input.as_mut() {
            match pipe.write(&bytes[offset..]) {
                Ok(0) => return Err("workerFailed"),
                Ok(n) => offset += n,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {},
                Err(_) => return Err("workerFailed"),
            }
            if offset == bytes.len() { input.take(); }
        }
        let out_done = drain(&mut output, &mut stdout, MAX_OUTPUT)?;
        let err_done = drain(&mut error, &mut stderr, MAX_ERROR)?;
        // Reap only once both pipes are closed. Before that, the unreaped PID
        // cannot be reused while cancellation targets this owned group.
        if out_done && err_done {
            if let Some(status) = child.child.try_wait().map_err(|_| "workerFailed")? {
                child.reaped = true;
                if !status.success() { return Err("workerFailed"); }
                if !stderr.is_empty() { return Err("workerFailed"); }
                let value = serde_json::from_slice(&stdout).map_err(|_| "workerFailed")?;
                if !current() { return Err("contextRevoked"); }
                return Ok(value);
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use super::TEST_SERIAL as SERIAL;
    fn request() -> Value {
        let body = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Coucou//EN\r\nBEGIN:VEVENT\r\nUID:synthetic\r\nDTSTART:20261021T090000Z\r\nDTEND:20261021T100000Z\r\nSUMMARY:Reunião sintética\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        json!({"schemaVersion":1,"requestId":"packaged-test","source":{"id":"synthetic","name":"synthetic.ics","revision":super::super::sha(body.as_bytes())},"calendar":body,"window":{"start":"2026-10-21T00:00:00Z","end":"2026-10-22T00:00:00Z","timeZone":"Europe/Lisbon"},"limit":32})
    }
    #[test]
    fn sealed_archive_is_read_only_and_matches_manifest() {
        let mut file = archive().unwrap();
        assert!(file.write_all(b"change").is_err());
        let manifest: Value = serde_json::from_str(include_str!("runtime-manifest.json")).unwrap();
        assert_eq!(manifest["runtimeSha256"], super::super::sha(RUNTIME));
    }
    #[test]
    fn packaged_worker_uses_embedded_deps_and_timezone_without_site_packages() {
        let _serial = SERIAL.lock().unwrap();
        let result = run(&request(), || true).unwrap();
        assert_eq!(result["status"], "ready", "{result}");
        assert_eq!(result["events"][0]["summary"], "Reunião sintética");
        assert_eq!(result["events"][0]["start"]["value"], "2026-10-21T09:00:00+00:00");
        assert_eq!(result["window"]["start"], "2026-10-21T01:00:00+01:00");
    }
    #[test]
    fn packaged_day_resolver_handles_dst_relative_days_and_skipped_dates(){
        let _serial=SERIAL.lock().unwrap();
        for (date,zone,reference,start,end) in [
            ("2026-03-29","Europe/Lisbon","2026-10-22T00:00:00Z","2026-03-29T00:00:00Z","2026-03-29T23:00:00Z"),
            ("2026-10-25","Europe/Lisbon","2026-10-22T00:00:00Z","2026-10-24T23:00:00Z","2026-10-26T00:00:00Z"),
            ("tomorrow","Asia/Tokyo","2026-12-31T20:00:00Z","2027-01-01T15:00:00Z","2027-01-02T15:00:00Z"),
            ("2018-11-04","America/Sao_Paulo","2026-10-22T00:00:00Z","2018-11-04T03:00:00Z","2018-11-05T02:00:00Z"),
            ("2020-11-01","America/Havana","2026-10-22T00:00:00Z","2020-11-01T04:00:00Z","2020-11-02T05:00:00Z"),
            ("2011-12-29","Pacific/Apia","2026-10-22T00:00:00Z","2011-12-29T10:00:00Z","2011-12-30T10:00:00Z"),
        ]{
            let day=super::super::day::Day::parse(json!({"date":date,"timeZone":zone})).unwrap();let query=day.resolve(reference,||true).unwrap();
            for (actual,expected) in [(&query.start,start),(&query.end,end)]{assert_eq!(super::super::query::instant(actual),super::super::query::instant(expected),"{date} {zone}");}
        }
        let skipped=super::super::day::Day::parse(json!({"date":"2011-12-30","timeZone":"Pacific/Apia"})).unwrap();assert_eq!(skipped.resolve("2026-10-22T00:00:00Z",||true).unwrap_err(),"skippedDate");
        let day=super::super::day::Day::parse(json!({"date":"today","timeZone":"UTC"})).unwrap();assert_eq!(day.resolve("2026-10-22T00:00:00Z",||false).unwrap_err(),"contextRevoked");
        let checked=std::cell::Cell::new(0);assert_eq!(day.resolve("2026-10-22T00:00:00Z",||{checked.set(checked.get()+1);checked.get()<3}).unwrap_err(),"contextRevoked");assert_eq!(RUNNING.load(Ordering::Acquire),0);
    }
    #[test]
    fn day_service_freezes_reference_and_runs_the_original_source_bound_event_reader(){
        let _serial=SERIAL.lock().unwrap();let(_guard,base,_,cancelled)=super::super::query::tests::fixture(|_,_|panic!("unused"));
        let _keep=base;
        let service=super::super::query::Service::new_at(&_guard,Some("2026-10-20T23:30:00Z".into()),|q,_|{
            let mut request=request();request["window"]=q.window();request["limit"]=json!(q.limit);
            let result=run(&request,||true).unwrap();assert!(super::super::query::bound_response(&result,&request));Ok(result)
        });
        for _ in 0..2{let result:Value=serde_json::from_str(&service.execute_auto(json!({"date":"today","timeZone":"Europe/Lisbon"}),&std::sync::atomic::AtomicBool::new(false)).unwrap()).unwrap();assert_eq!(result["status"],"ready");assert_eq!(result["events"][0]["summary"],"Reunião sintética");assert_eq!(result["source"]["id"],"synthetic");assert_eq!(super::super::query::instant(result["window"]["start"].as_str().unwrap()),super::super::query::instant("2026-10-20T23:00:00Z"));}
        assert!(!cancelled.load(Ordering::Acquire));
    }
    #[test]
    fn queries_beyond_default_window_keep_exact_instants_and_explicit_dense_coverage(){
        let _serial=SERIAL.lock().unwrap();let mut request=request();
        let mut body="BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Coucou//EN\r\n".to_string();
        for n in 0..150 {body.push_str(&format!("BEGIN:VEVENT\r\nUID:synthetic-{n}\r\nDTSTART:20261121T090000Z\r\nDTEND:20261121T100000Z\r\nSUMMARY:Synthetic {n}\r\nEND:VEVENT\r\n"));}
        body.push_str("END:VCALENDAR\r\n");request["calendar"]=json!(body);request["source"]["revision"]=json!(super::super::sha(body.as_bytes()));
        request["window"]=json!({"start":"2026-11-21T00:00:00Z","end":"2026-11-22T00:00:00Z","timeZone":"Europe/Lisbon"});request["limit"]=json!(128);
        let response=run(&request,||true).unwrap();assert!(super::super::query::bound_response(&response,&request));assert_eq!(response["events"].as_array().unwrap().len(),128);assert_eq!(response["hasMore"],true);assert_eq!(response["coverage"]["eventsComplete"],false);
        let bounded=super::super::bounded_data(response);assert!(bounded.to_string().len()<=super::super::MAX_PROMPT);assert_eq!(bounded["status"],"partial");assert_eq!(bounded["hasMore"],true);
    }
    #[test]
    fn cancelled_before_spawn_or_during_worker_never_returns_data() {
        let _serial = SERIAL.lock().unwrap();
        assert_eq!(run(&request(), || false).unwrap_err(), "contextRevoked");
        let mut calls = 0;
        assert_eq!(run(&request(), || { calls += 1; calls < 3 }).unwrap_err(), "contextRevoked");
        assert_eq!(RUNNING.load(Ordering::Acquire), 0);
    }
    #[test]
    fn timeout_output_limit_and_invalid_json_reap_only_the_owned_child() {
        let _serial = SERIAL.lock().unwrap();
        for (worker, code, timeout) in [
            ("import time;time.sleep(30)", "workerTimeout", Duration::from_millis(100)),
            ("import sys;sys.stdout.buffer.write(b'x'*100000)", "workerOutputLimit", Duration::from_secs(2)),
            ("import sys;sys.stdout.write('not JSON PRIVATE-CANARY')", "workerFailed", Duration::from_secs(2)),
        ] {
            let mut pid = 0;
            assert_eq!(run_worker(&request(), || true, worker, timeout, |id| pid = id).unwrap_err(), code);
            assert_ne!(pid, 0);
            assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
            assert_eq!(RUNNING.load(Ordering::Acquire), 0);
        }
    }
}
