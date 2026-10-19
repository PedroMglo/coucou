//! Native Antigravity streaming protocol. Keep the user's CLI configuration;
//! do not read tokens, change permission rules, or attach to unrelated sessions.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

const MAX_LINE: usize = 1024 * 1024;
const MAX_TOTAL: usize = 32 * 1024 * 1024;

struct Process {
    child: Child,
    input: Option<ChildStdin>,
    output: Receiver<Result<String, String>>,
    deadline: Instant,
    cancel: Option<Arc<AtomicBool>>,
}

impl Process {
    fn start(exe: &Path, cwd: &Path, args: &[&str], timeout: Duration) -> Result<Self, String> {
        let mut cmd = Command::new(exe);
        cmd.args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(target_os="linux")]
        cmd.env(crate::gravity_hook::OWNED_ENV,if args.contains(&"--input-format") && !args.contains(&"--dangerously-skip-permissions") {"1"} else {"0"});
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let mut child = cmd.spawn().map_err(|_| {
            "Cannot start Antigravity. Run agy in a terminal and sign in first.".to_string()
        })?;
        let input = child.stdin.take();
        #[cfg(target_os="linux")]
        if let Some(input)=&input{
            use std::os::fd::AsRawFd;
            let fd=input.as_raw_fd();let flags=unsafe{libc::fcntl(fd,libc::F_GETFL)};
            if flags<0||unsafe{libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{
                // The leader is still unreaped, so this is only our new group.
                unsafe{libc::kill(-(child.id()as i32),libc::SIGKILL);}let _=child.kill();let _=child.wait();
                return Err("Cannot prepare cancellable Antigravity input.".into());
            }
        }
        let stdout = child
            .stdout
            .take()
            .ok_or("Antigravity did not open stdout.")?;
        let (tx, output) = mpsc::sync_channel(32);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut total = 0;
            loop {
                let mut bytes = Vec::new();
                let n = match (&mut reader)
                    .take(MAX_LINE as u64 + 1)
                    .read_until(b'\n', &mut bytes)
                {
                    Ok(n) => n,
                    Err(_) => {
                        let _ = tx.send(Err("Cannot read Antigravity output.".into()));
                        break;
                    }
                };
                if n == 0 {
                    break;
                }
                total += n;
                if n > MAX_LINE || total > MAX_TOTAL {
                    let _ = tx.send(Err("Antigravity response exceeded the size limit.".into()));
                    break;
                }
                if tx
                    .send(Ok(String::from_utf8_lossy(&bytes).to_string()))
                    .is_err()
                {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            deadline: Instant::now() + timeout,
            cancel: None,
        })
    }

    fn line(&self) -> Result<String, String> {
        loop {
            if self
                .cancel
                .as_ref()
                .is_some_and(|c| c.load(Ordering::Relaxed))
            {
                return Err("The CLI request was cancelled.".into());
            }
            let left = self.deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err("Antigravity did not finish in time. Try again.".into());
            }
            match self.output.recv_timeout(left.min(Duration::from_millis(100))) {
                Ok(line) => return line,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Err("Antigravity closed the connection. Run agy in a terminal to check your account and permissions.".into()),
            }
        }
    }

    fn write_message(&mut self,message:&Value)->Result<(),String>{
        let mut bytes=serde_json::to_vec(message).map_err(|_|"Cannot prepare Antigravity input.")?;
        if bytes.len()>MAX_LINE{return Err("Antigravity input exceeded the size limit.".into());}
        bytes.push(b'\n');
        let deadline=self.deadline.min(Instant::now()+Duration::from_secs(30));
        let mut offset=0;
        while offset<bytes.len(){
            if self.cancel.as_ref().is_some_and(|flag|flag.load(Ordering::Relaxed)){return Err("The CLI request was cancelled.".into());}
            if Instant::now()>=deadline{return Err("Antigravity did not receive the request in time. Check the native session before sending again.".into());}
            let input=self.input.as_mut().ok_or("Antigravity did not open stdin.")?;
            match input.write(&bytes[offset..]){
                Ok(0)=>return Err("Antigravity closed its input.".into()),
                Ok(size)=>offset+=size,
                Err(error)if error.kind()==std::io::ErrorKind::WouldBlock=>std::thread::sleep(Duration::from_millis(10)),
                Err(error)if error.kind()==std::io::ErrorKind::Interrupted=>{},
                Err(_)=>return Err("Cannot write to Antigravity.".into()),
            }
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.input.take();
        #[cfg(target_os="linux")]
        {
            // Leave the leader unreaped until the whole owned group is cleaned.
            // Reaping first can both orphan tools and allow its PID to be reused.
            std::thread::sleep(Duration::from_millis(300));
            unsafe{libc::kill(-(self.child.id()as i32),libc::SIGTERM);}
            std::thread::sleep(Duration::from_millis(100));
            unsafe{libc::kill(-(self.child.id()as i32),libc::SIGKILL);}
            let _=self.child.kill();let _=self.child.wait();return;
        }
        #[cfg(not(target_os="linux"))]
        {
        for _ in 0..50 {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        #[cfg(unix)]
        {
            let _ = Command::new("/bin/kill")
                .args(["-TERM", "--", &format!("-{}", self.child.id())])
                .status();
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let _ = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &self.child.id().to_string()])
                .creation_flags(0x08000000)
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        }
    }
}

pub fn models(exe: &Path, cwd: &Path) -> Result<Vec<(String, String)>, String> {
    let mut process = Process::start(exe, cwd, &["models"], Duration::from_secs(30))?;
    process.input.take();
    let mut output = String::new();
    loop {
        match process
            .output
            .recv_timeout(process.deadline.saturating_duration_since(Instant::now()))
        {
            Ok(Ok(line)) => output.push_str(&line),
            Ok(Err(err)) => return Err(err),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(_) => return Err("Antigravity model discovery timed out.".into()),
        }
    }
    let models = parse_models(&output);
    if models.is_empty() {
        return Err(
            "Antigravity returned no models. Run agy in a terminal and sign in first.".into(),
        );
    }
    Ok(models)
}

pub fn parse_models(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter_map(|line| {
            let (id, label) = line.split_once('\t')?;
            let id = id.trim();
            if id.is_empty()
                || !id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-._/".contains(c))
            {
                return None;
            }
            Some((id.to_string(), label.trim().to_string()))
        })
        .collect()
}

pub struct Reply {
    pub text: String,
    pub acted: bool,
}

pub fn answer(
    exe: &Path,
    cwd: &Path,
    model: &str,
    system: &str,
    history: &[Value],
    query: &str,
    session_id: Option<&str>,
    full_access: bool,
    cancel: Option<Arc<AtomicBool>>,
    mut on_session: impl FnMut(&str),
    mut on_delta: impl FnMut(String),
    mut on_event: impl FnMut(&Value),
) -> Result<Reply, String> {
    if cancel.as_ref().is_some_and(|flag|flag.load(Ordering::Relaxed)){return Err("The CLI request was cancelled.".into());}
    let mut args = vec![
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
    ];
    if !model.is_empty() {
        args.extend(["--model", model]);
    }
    if let Some(id) = session_id {
        args.extend(["--conversation", id]);
    }
    if full_access {
        args.push("--dangerously-skip-permissions");
    }
    let mut process = Process::start(exe, cwd, &args, Duration::from_secs(1800))?;
    process.cancel = cancel;
    let prompt = if session_id.is_some() {
        format!("{system}\n\nUser: {query}")
    } else if history.is_empty() {
        format!("{system}\n\nUser: {query}")
    } else {
        format!(
            "{system}\n\nEarlier conversation (JSON data):\n{}\n\nUser: {query}",
            serde_json::to_string(history).unwrap_or_default()
        )
    };
    process.write_message(&json!({ "event": "user", "message": { "content": prompt } }))?;
    let mut stream = Answer::default();
    loop {
        let line = process.line()?;
        let message: Value = serde_json::from_str(&line)
            .map_err(|_| "Antigravity returned an invalid protocol message.")?;
        on_event(&message);
        if message["event"] == "init" {
            if let Some(id) = message["conversation_id"].as_str() {
                on_session(id);
            }
        }
        let before=stream.text.len();
        if let Some(reply) = stream.event(&message)? {
            return Ok(reply);
        }
        if stream.text.len()!=before && !stream.text.is_empty() {
            on_delta(stream.text.clone());
        }
    }
}

#[derive(Default)]
struct Answer {
    text: String,
    acted: bool,
}
impl Answer {
    fn event(&mut self, message: &Value) -> Result<Option<Reply>, String> {
        match message["event"].as_str() {
            Some("step_update") => {
                let step = &message["step_update"];
                if step["step_type"] == "agent_response" {
                    if let Some(text) = step["text_delta"].as_str() {
                        self.text.push_str(text);
                    }
                }
                if step["step_type"] == "tool" && step["state"] == "DONE" {
                    self.acted = true;
                }
            }
            Some("result") => {
                let result = &message["result"];
                if result["status"] != "SUCCESS" {
                    return Err(result["error"]
                        .as_str()
                        .unwrap_or("Antigravity did not complete this turn.")
                        .chars()
                        .take(2000)
                        .collect());
                }
                let text = result["response"].as_str().unwrap_or("");
                if text.trim().is_empty() {
                    return Err("Antigravity returned an empty reply.".into());
                }
                return Ok(Some(Reply {
                    text: text.into(),
                    acted: self.acted,
                }));
            }
            _ => {}
        }
        if self.text.len() > 4 * 1024 * 1024 {
            return Err("Antigravity reply exceeded the size limit.".into());
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_uses_real_cli_slugs_and_labels() {
        let models = parse_models("Fetching available models...\ngemini-3.8-flash-high\tGemini 3.8 Flash (High)\nclaude-sonnet-4-6\tClaude Sonnet 4.6 (Thinking)\n");
        assert_eq!(models.len(), 2);
        assert_eq!(
            models[1],
            (
                "claude-sonnet-4-6".into(),
                "Claude Sonnet 4.6 (Thinking)".into()
            )
        );
    }
    #[test]
    fn result_replaces_deltas_and_failed_turns_never_succeed() {
        let mut answer = Answer::default();
        answer.event(&json!({"event":"step_update","step_update":{"step_type":"agent_response","text_delta":"Olá"}})).unwrap();
        assert_eq!(answer.text, "Olá");
        let reply = answer
            .event(&json!({"event":"result","result":{"status":"SUCCESS","response":"Olá!"}}))
            .unwrap()
            .unwrap();
        assert_eq!(reply.text, "Olá!");
        for status in ["ERROR", "CANCELED", "INTERRUPTED", "WAITING"] {
            assert!(answer
                .event(&json!({"event":"result","result":{"status":status,"response":"partial"}}))
                .is_err());
        }
    }
}
