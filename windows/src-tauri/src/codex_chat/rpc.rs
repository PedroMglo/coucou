//! Bounded stdio client for the user's own Codex app-server. No credentials
//! are read here, and every child is reaped, including on timeout or failure.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

#[path="timing.rs"]
pub(crate) mod timing;
use timing::{Stage, Trace};

#[path="native_requests.rs"]pub(crate) mod native_requests;
use native_requests::{Requests,Frame};
// Match the actual DynamicToolSpec function/namespace union. Neither a group
// name nor a same-named function in another namespace selects an executor.
fn tool_available(tools:&[Value],name:&str,namespace:&Value)->bool{
    if !namespace.is_null()&&namespace.as_str().is_none(){return false;}
    tools.iter().map(|spec|{
        if namespace.is_null(){usize::from(spec["type"]=="function"&&spec["name"]==name)}
        else if spec["type"]=="namespace"&&spec["name"]==*namespace{
            spec["tools"].as_array().map(|members|members.iter().filter(|t|t["type"]=="function"&&t["name"]==name).count()).unwrap_or(0)
        }else{0}
    }).sum::<usize>()==1
}
const MAX_LINE: usize = 1024 * 1024;
const MAX_TOTAL: usize = 32 * 1024 * 1024;
const MAX_ANSWER: usize = 4 * 1024 * 1024;
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(30);
const CANCEL_POLL: Duration = Duration::from_millis(100);
pub const SIGN_IN: &str =
    "Sign in to Codex with your ChatGPT account first: run codex login, then try again.";

pub struct TurnOptions<'a> {
    pub model: &'a str,
    pub system: &'a str,
    pub history: &'a [Value],
    pub query: &'a str,
    pub tools: &'a [Value],
    pub session_id: Option<&'a str>,
    pub full_access: bool,
    pub effort: Option<&'a str>,
    pub name: Option<&'a str>,
}

pub struct Reply {
    pub text: String,
    pub acted: bool,
    pub session_id: String,
}

pub struct Client {
    #[cfg(test)] io_probe:Option<crate::settings::EffectProbe>,
    child: Child,
    input: Option<ChildStdin>,
    output: Receiver<Result<Frame, String>>,
    requests:Requests,
    deadline: Instant,
    next_id: u64,
    pending: VecDeque<Frame>,
    cancel: Option<Arc<AtomicBool>>,
    active_turn: Option<(String, String)>,
    observer: Option<Box<dyn FnMut(&Value) + Send>>,
    write_incomplete: bool,
    timing: Option<Trace>,
}

impl Client {
    pub fn start(exe: &Path, cwd: &Path, timeout: Duration, voice: bool) -> Result<Self, String> {
        Self::start_inner(exe, cwd, timeout, voice, None, INITIALIZE_TIMEOUT)
    }

    /// Attach the turn's cancellation before starting the server or handshake.
    pub fn start_with_cancel(exe: &Path, cwd: &Path, timeout: Duration, voice: bool, cancel: Arc<AtomicBool>) -> Result<Self, String> {
        Self::start_inner(exe, cwd, timeout, voice, Some(cancel), INITIALIZE_TIMEOUT)
    }

    fn start_inner(exe: &Path, cwd: &Path, timeout: Duration, voice: bool, cancel: Option<Arc<AtomicBool>>, initialize_timeout: Duration) -> Result<Self, String> {
        Self::start_inner_timed(exe, cwd, timeout, voice, cancel, initialize_timeout, None)
    }

    pub(crate) fn start_with_cancel_timed(exe: &Path, cwd: &Path, timeout: Duration, voice: bool, cancel: Arc<AtomicBool>, trace: Trace) -> Result<Self, String> {
        Self::start_inner_timed(exe, cwd, timeout, voice, Some(cancel), INITIALIZE_TIMEOUT, Some(trace))
    }

    fn start_inner_timed(exe: &Path, cwd: &Path, timeout: Duration, voice: bool, cancel: Option<Arc<AtomicBool>>, initialize_timeout: Duration, timing: Option<Trace>) -> Result<Self, String> {
        if cancel.as_ref().is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("The CLI request was cancelled.".into());
        }
        let requests=Requests::new()?;
        let started = Instant::now();
        let deadline = started + timeout;
        let mut cmd = Command::new(exe);
        cmd.args([
            "app-server",
            "--listen",
            "stdio://",
            "-c",
            "model_provider=\"openai\"",
        ])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        if let Some(dir) = exe.parent() {
            let mut dirs = vec![dir.to_path_buf()];
            if let Some(path) = std::env::var_os("PATH") {
                dirs.extend(std::env::split_paths(&path));
            }
            if let Ok(path) = std::env::join_paths(dirs) {
                cmd.env("PATH", path);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        if cancel.as_ref().is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("The CLI request was cancelled.".into());
        }
        let spawn = timing::Span::new(timing.as_ref(), Stage::Spawn);
        crate::settings::effect_allowed()?;
        let child = cmd.spawn().map_err(|err| {
            format!("Cannot start Codex. Install Codex CLI and sign in with codex login: {err}")
        });
        spawn.finish(&child);
        let mut child = child?;
        let input = child.stdin.take();
        #[cfg(target_os="linux")]
        if let Some(input)=&input {
            use std::os::fd::AsRawFd;
            let fd=input.as_raw_fd();let flags=unsafe{libc::fcntl(fd,libc::F_GETFL)};
            if flags<0||unsafe{libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{
                reap_failed_start(&mut child, timing.as_ref());return Err("Cannot prepare bounded Codex input.".into());
            }
        }
        let Some(stdout) = child.stdout.take() else {
            reap_failed_start(&mut child, timing.as_ref());
            return Err("Codex did not open stdout.".into());
        };
        // The wire byte budget and frame cap bound this queue while allowing
        // native resolutions to be read during a blocking human callback.
        let (tx, output) = mpsc::channel();let reader_requests=requests.clone();
        std::thread::Builder::new().name("coucou-codex-reader".into()).spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut total = 0;let mut frames=0usize;
            loop {
                frames+=1;if frames>65_536{reader_requests.disconnect(true);let _=tx.send(Err("Codex event stream exceeded its frame budget.".into()));break;}
                let result = read_message(&mut reader, &mut total).map(|value|reader_requests.observe(value));
                let failed=result.is_err();
                if tx.send(result).is_err()||failed{break;}
            }
            reader_requests.disconnect(false);
        }).map_err(|_| {
            reap_failed_start(&mut child, timing.as_ref());
            "Cannot start the Codex response reader.".to_string()
        })?;
        let mut client = Self {
            #[cfg(test)] io_probe:None,
            child,
            input,
            output,
            requests,
            deadline: deadline.min(started + initialize_timeout),
            next_id: 0,
            pending: VecDeque::new(),
            cancel,
            active_turn: None,
            observer: None,
            write_incomplete: false,
            timing,
        };
        client.request("initialize", json!({
            "clientInfo": { "name": "coucou_mochi", "title": "Coucou — Mochi", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": { "experimentalApi": voice }
        }))?;
        let initialized = timing::Span::new(client.timing.as_ref(), Stage::Initialized);
        let result = client.write(json!({ "method": "initialized" }));
        initialized.finish(&result);
        result?;
        // The handshake limit must not become the deadline of a long task.
        client.deadline = deadline;
        Ok(client)
    }

    fn write(&mut self, message: Value) -> Result<(), String> {
        self.write_frame(message,self.deadline,false)
    }

    /// Cleanup may interrupt its exact native turn even after cancellation.
    fn write_message(&mut self, message: Value) -> Result<(), String> {
        self.write_frame(message,Instant::now()+Duration::from_millis(100),true)
    }
    fn write_frame(&mut self,message:Value,deadline:Instant,cleanup:bool)->Result<(),String>{
        match self.write_frame_guarded(message,deadline,cleanup,None,crate::pet_native_write::Use::Decision)?{
            crate::pet_native_write::Outcome::Written=>Ok(()),
            crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::Cancelled)=>Err("The CLI request was cancelled.".into()),
            crate::pet_native_write::Outcome::Skipped(crate::pet_native_write::Reason::PreferencesUnconfirmed)=>Err("A origem das preferências não está confirmada; o novo pedido Codex não foi enviado.".into()),
            crate::pet_native_write::Outcome::Skipped(_)=>Err("Codex did not receive the request in time.".into()),
        }
    }
    fn write_frame_guarded(&mut self,message:Value,deadline:Instant,cleanup:bool,source:Option<&crate::pet_native_decisions::Ticket>,use_:crate::pet_native_write::Use)->Result<crate::pet_native_write::Outcome,String>{
        if !cleanup{self.check_cancel()?;if self.requests.overflowed(){return Err("Codex event stream exceeded its bounds; the native request was not answered.".into());}}
        if self.write_incomplete{return Err("Codex input contains an interrupted frame.".into());}
        if source.is_some_and(|t|message.get("id")!=Some(&t.identity().request_id)){return Err("The response does not belong to the captured native Codex request.".into());}
        let mut bytes=serde_json::to_vec(&message).map_err(|_|"Cannot encode Codex request.")?;
        if bytes.len()>=MAX_TOTAL{return Err("Codex request exceeds the local transport limit.".into());}bytes.push(b'\n');
        let dispatch=if cleanup||matches!(use_,crate::pet_native_write::Use::Refusal|crate::pet_native_write::Use::Withdrawal){crate::pet_native_write::Dispatch::Control}else{crate::pet_native_write::Dispatch::NewEffect};
        #[cfg(test)]let dispatch=match (&dispatch,&self.io_probe){(crate::pet_native_write::Dispatch::NewEffect,Some(p))=>crate::pet_native_write::Dispatch::ProbedEffect(p.clone()),_=>dispatch};
        let guard=crate::pet_native_write::Guard{dispatch,deadline,cancel:if cleanup{None}else{self.cancel.clone()},caller:None,source:source.map(|t|(t.clone(),use_))};
        let result=crate::pet_native_write::frame(self.input.as_mut().ok_or("Codex closed its input.")?,&bytes,&guard);
        match result{
            Ok(outcome)=>Ok(outcome),
            Err(fault)=>{
                if fault.partial{self.write_incomplete=true;}
                // A partial frame cannot be completed or followed by cleanup.
                // Retire only this owned pipe; Drop reaps its original child.
                self.input.take();self.requests.disconnect(false);
                Err(if fault.reason==crate::pet_native_write::Reason::Cancelled{"The CLI request was cancelled."}else if fault.reason==crate::pet_native_write::Reason::Expired{"Codex did not receive the request in time; the owned transport was retired without retry."}else if fault.partial{"The native Codex response lost validity during a partial write; the owned transport was retired without retry."}else{"Cannot write to Codex."}.into())
            }
        }
    }

    fn read(&mut self) -> Result<Frame, String> {
        self.check_cancel()?;
        if let Some(message) = self.pending.pop_front() {
            return Ok(message);
        }
        self.read_raw()
    }

    fn read_raw(&self) -> Result<Frame, String> {
        loop {
            self.check_cancel()?;
            if self.requests.overflowed(){return Err("Codex event stream exceeded its bounds; reconnect explicitly to inspect the original session.".into());}
            let left = self.deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err("Codex did not finish in time. Try again.".into());
            }
            match self.output.recv_timeout(left.min(CANCEL_POLL)) {
                Ok(message) => {
                    self.check_cancel()?;
                    return message;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Err("Codex closed the connection before finishing.".into()),
            }
        }
    }

    fn check_cancel(&self) -> Result<(), String> {
        if self.cancel.as_ref().is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            Err("The CLI request was cancelled.".into())
        } else {
            Ok(())
        }
    }

    pub fn observe(&mut self, observer: impl FnMut(&Value) + Send + 'static) { self.observer = Some(Box::new(observer)); }

    pub fn cancel_with(&mut self, cancel: Arc<AtomicBool>) {
        self.cancel = Some(cancel);
    }

    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let span = timing::Span::rpc(self.timing.as_ref(), method);
        let result = self.request_inner(method, params);
        span.finish(&result);
        result
    }

    fn request_inner(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id=self.next_id.checked_add(1).ok_or("Codex request ID space is exhausted.")?;
        let id = self.next_id;
        self.requests.expect_handshake(&json!(id),method,&params)?;
        self.write(json!({ "id": id, "method": method, "params": params }))?;
        timing::written(self.timing.as_ref(), method);
        loop {
            let message = self.read_raw()?;
            if message["id"] == id && message.get("method").is_none() {
                if let Some(error) = message.get("error") {
                    return Err(rpc_error(error));
                }
                return message
                    .get("result")
                    .cloned()
                    .ok_or_else(|| "Invalid Codex response.".into());
            }
            let method = message["method"].as_str().unwrap_or("");
            if method.starts_with("item/") || method.starts_with("turn/") || crate::pet_codex_request_scope::supported(method) {
                self.pending.push_back(message);
            } else {
                self.reject_request(&message)?;
            }
        }
    }

    fn reject_request(&mut self, message: &Frame) -> Result<(), String> {
        if message.get("id").is_some() && message.get("method").is_some() {
            let response=json!({ "id": message["id"], "error": { "code": -32601, "message": "This request is unavailable in Mochi chat." } });
            self.write_frame_guarded(response,self.deadline,false,message.ticket.as_ref(),crate::pet_native_write::Use::Refusal)?;
        }
        Ok(())
    }

    fn withdraw_policy_request(&mut self,message:&Frame)->Result<(),String>{
        self.write_frame_guarded(json!({"id":message["id"],"error":{"code":-32000,"message":"The native session policy changed while this request was pending. Request a fresh native decision under the current policy; do not replay a previous effect."}}),Instant::now()+Duration::from_millis(100),true,message.ticket.as_ref(),crate::pet_native_write::Use::Withdrawal).map(|_|())
    }

    fn finish_skipped_response(&mut self,message:&Frame,outcome:crate::pet_native_write::Outcome)->Result<(),String>{
        use crate::pet_native_write::{Outcome,Reason,Use};
        if matches!(outcome,Outcome::Skipped(_))&&message.ticket.as_ref().is_some_and(|t|t.policy_invalidated()){
            return self.withdraw_policy_request(message);
        }
        if outcome==Outcome::Skipped(Reason::PreferencesUnconfirmed)&&message.ticket.as_ref().is_some_and(|t|t.current()){
            // Safe fixed control only: no tool data or positive decision. A
            // previous tool effect may already have happened; never invite replay.
            self.write_frame_guarded(json!({"id":message["id"],"error":{"code":-32000,"message":"Preferences are unconfirmed. This response was not delivered; a previous tool effect may have occurred. Do not retry effects automatically."}}),Instant::now()+Duration::from_millis(100),true,message.ticket.as_ref(),Use::Refusal)?;
        }
        Ok(())
    }

    pub fn require_account(&mut self) -> Result<(), String> {
        let result = self.request("account/read", json!({ "refreshToken": false }))?;
        if result["account"]["type"] != "chatgpt" {
            return Err(SIGN_IN.into());
        }
        Ok(())
    }

    pub fn models(&mut self) -> Result<Vec<(String, String)>, String> {
        self.require_account()?;
        let mut models = Vec::new();
        let mut cursor = Value::Null;
        for _ in 0..20 {
            let result = self.request("model/list", json!({ "limit": 100, "cursor": cursor }))?;
            for model in result["data"]
                .as_array()
                .ok_or("Invalid Codex model list.")?
            {
                if let Some(id) = model["model"].as_str() {
                    models.push((
                        id.to_string(),
                        model["displayName"].as_str().unwrap_or(id).to_string(),
                    ));
                }
            }
            let next = result["nextCursor"].clone();
            if next.is_null() {
                return Ok(models);
            }
            if next == cursor {
                return Err("Codex repeated its model-list cursor.".into());
            }
            cursor = next;
        }
        Err("Codex returned too many model-list pages.".into())
    }

    #[cfg(test)]
    pub fn answer(&mut self,options:TurnOptions<'_>,on_tool:impl FnMut(&str,&Value)->(String,bool),mut on_request:impl FnMut(&str,&Value)->Option<Value>,on_session:impl FnMut(&str),on_delta:impl FnMut(String))->Result<Reply,String>{
        self.answer_native(options,on_tool,|method,params,_|on_request(method,params),on_session,on_delta)
    }
    #[cfg(test)]
    pub fn answer_native(&mut self,options:TurnOptions<'_>,mut on_tool:impl FnMut(&str,&Value)->(String,bool),on_request:impl FnMut(&str,&Value,Option<&native_requests::Ticket>)->Option<Value>,on_session:impl FnMut(&str),on_delta:impl FnMut(String))->Result<Reply,String>{
        self.answer_native_tools(options,|name,args,_|on_tool(name,args),on_request,on_session,on_delta)
    }
    pub(crate) fn answer_native_tools(
        &mut self,
        options: TurnOptions<'_>,
        on_tool: impl FnMut(&str, &Value,&crate::pet_native_invocation::Fence) -> (String, bool),
        on_request: impl FnMut(&str, &Value,Option<&native_requests::Ticket>) -> Option<Value>,
        on_session: impl FnMut(&str),
        on_delta: impl FnMut(String),
    ) -> Result<Reply, String> {
        let span = timing::Span::new(self.timing.as_ref(), Stage::Answer);
        let result = self.answer_inner(options, on_tool, on_request, on_session, on_delta);
        span.finish(&result);
        result
    }

    fn answer_inner(
        &mut self,
        options: TurnOptions<'_>,
        mut on_tool: impl FnMut(&str, &Value,&crate::pet_native_invocation::Fence) -> (String, bool),
        mut on_request: impl FnMut(&str, &Value,Option<&native_requests::Ticket>) -> Option<Value>,
        mut on_session: impl FnMut(&str),
        mut on_delta: impl FnMut(String),
    ) -> Result<Reply, String> {
        self.require_account()?;
        let TurnOptions {
            model,
            system,
            history,
            query,
            tools,
            session_id,
            full_access,
            effort,
            name,
        } = options;
        let configuration = self.request("config/read", json!({ "includeLayers": false }))?;
        let native = configuration["config"]["developer_instructions"]
            .as_str()
            .unwrap_or("");
        let instructions = if native.is_empty() {
            system.to_string()
        } else {
            format!("{native}\n\n{system}")
        };
        // Keep the CLI's native tools, MCP servers, skills and permissions.
        // Explicit full access applies only when creating a new session.
        // Resuming must inherit the current native policy, including a change
        // made in the terminal after this conversation was originally created.
        let mut params = json!({
            "model": if model.is_empty() { Value::Null } else { json!(model) },
            "modelProvider": "openai", "developerInstructions": instructions
        });
        let method = if let Some(id) = session_id {
            params["threadId"] = json!(id);
            "thread/resume"
        } else {
            if !tools.is_empty() {
                params["dynamicTools"] = json!(tools);
            }
            "thread/start"
        };
        if full_access && session_id.is_none() {
            params["sandbox"] = json!("danger-full-access");
            params["approvalPolicy"] = json!("never");
        }
        let thread = self.request(method, params)?;
        let thread_id = thread["thread"]["id"]
            .as_str()
            .ok_or("Codex did not create a conversation.")?
            .to_string();
        if session_id.is_some_and(|expected|expected!=thread_id){return Err("Codex resumed a different native thread; the requested session was not replaced.".into());}
        on_session(&thread_id);
        if let Some(name)=name {
            let existing=thread["thread"].get("name");
            if session_id.is_none() || existing.is_some_and(|v|v.is_null() || v.as_str().is_some_and(|n|n.starts_with("Mochi ·"))) {
                let _=self.request("thread/name/set",json!({"threadId":thread_id,"name":name}));
            }
        }
        if session_id.is_none() && !history.is_empty() {
            self.request(
                "thread/inject_items",
                json!({ "threadId": thread_id, "items": history_items(history) }),
            )?;
        }
        let mut turn_params = json!({
            "threadId": thread_id, "input": [{ "type": "text", "text": query }]
        });
        if let Some(effort) = effort { turn_params["effort"] = json!(effort); }
        let turn = self.request("turn/start", turn_params)?;
        let turn_id = turn["turn"]["id"]
            .as_str()
            .ok_or("Codex did not start a reply.")?
            .to_string();
        self.active_turn = Some((thread_id.clone(), turn_id.clone()));
        let mut answer = Answer::default();
        let mut acted = false;
        let mut tool_calls = 0;
        let mut seen_tool_calls=std::collections::HashSet::new();
        let mut first_event = true;
        let mut first_text = true;
        loop {
            let message = self.read()?;
            let params = &message["params"];
            if message["method"] == "item/tool/call"
                && message.get("id").is_some()
            {
                if message.invalid_request||message.ticket.is_none()||crate::pet_codex_request_scope::extract("item/tool/call",params).is_none_or(|s|!s.matches(&thread_id,Some(&turn_id))){
                    self.reject_request(&message)?;if let Some(ticket)=&message.ticket{self.requests.finish(ticket);}continue;
                }
                let source=message.ticket.as_ref().unwrap();
                if !source.current(){if source.policy_invalidated(){self.withdraw_policy_request(&message)?;}self.requests.finish(source);continue;}
                let Some(call)=crate::pet_native_invocation::Invocation::begin(source,self.deadline,self.cancel.clone())else{self.requests.finish(source);continue;};
                tool_calls += 1;
                let name = params["tool"].as_str().unwrap_or("");
                let fresh=tool_calls<=100&&seen_tool_calls.insert(params["callId"].as_str().unwrap().to_string());
                let allowed = fresh && tool_available(tools,name,&params["namespace"]);
                let (mut text, did) = if allowed {
                    let callback = timing::Span::new(self.timing.as_ref(), Stage::DynamicToolCallback);
                    let result = on_tool(name, &params["arguments"],call.fence());
                    callback.returned();
                    result
                } else {
                    (if !fresh{"This native tool call has already been handled or the call limit was reached. It will not be executed again."}else{"This tool or namespace is unavailable."}.into(), false)
                };
                if !call.fence().current(){if source.policy_invalidated(){self.withdraw_policy_request(&message)?;}self.requests.finish(source);continue;}
                acted |= did;
                let oversized=text.len()>256*1024;
                if oversized{text="The tool callback returned an oversized result; its effect is unconfirmed. Do not retry the operation automatically.".into();}
                let mut response=json!({ "id": message["id"], "result": {
                    "contentItems": [{ "type": "inputText", "text": text }], "success": allowed&&!oversized
                } });
                if serde_json::to_vec(&response).map_err(|_|"Invalid native tool response.")?.len()>256*1024{
                    response=json!({"id":message["id"],"result":{"contentItems":[{"type":"inputText","text":"The tool callback returned an oversized result; its effect is unconfirmed. Do not retry the operation automatically."}],"success":false}});
                }
                let outcome=self.write_frame_guarded(response,self.deadline,false,Some(source),crate::pet_native_write::Use::Decision)?;
                self.finish_skipped_response(&message,outcome)?;
                self.requests.finish(source);
                continue;
            }
            if message.get("id").is_some() && message.get("method").is_some() {
                let method = message["method"].as_str().unwrap_or("");
                let supported=crate::pet_codex_request_scope::supported(method);
                let wrong_scope=if supported {
                    message.ticket.is_none() || crate::pet_codex_request_scope::extract(method,params).is_none_or(|scope|!scope.matches(&thread_id,Some(&turn_id)))
                }else{params.get("threadId").is_some_and(|v|v!=thread_id.as_str())||params.get("turnId").is_some_and(|v|v!=turn_id.as_str())};
                if message.invalid_request||wrong_scope{
                    self.reject_request(&message)?;if let Some(ticket)=&message.ticket{self.requests.finish(ticket);}continue;
                }
                if let Some(ticket)=message.ticket.as_ref().filter(|ticket|!ticket.current()){
                    if ticket.policy_invalidated(){self.withdraw_policy_request(&message)?;self.requests.finish(ticket);}continue;
                }
                let callback = timing::Span::new(self.timing.as_ref(), Stage::NativeRequestCallback);
                let result = on_request(method, params,message.ticket.as_ref());
                callback.returned();
                if let Some(ticket)=message.ticket.as_ref().filter(|ticket|!ticket.current()){
                    if ticket.policy_invalidated(){self.withdraw_policy_request(&message)?;self.requests.finish(ticket);}continue;
                }
                if let Some(result) = result {
                    let use_=if crate::pet_native_choices::pure_refusal("codex",method,params,&result){crate::pet_native_write::Use::Refusal}else{crate::pet_native_write::Use::Decision};
                    let outcome=self.write_frame_guarded(json!({ "id": message["id"], "result": result }),self.deadline,false,message.ticket.as_ref(),use_)?;
                    self.finish_skipped_response(&message,outcome)?;
                } else {
                    self.reject_request(&message)?;
                }
                if let Some(ticket)=&message.ticket{self.requests.finish(ticket);}
                continue;
            }
            if params["threadId"] != thread_id {
                continue;
            }
            if params.get("turnId").is_some() && params["turnId"] != turn_id {
                continue;
            }
            // Terminal notifications identify their turn in the nested object.
            // A matching thread (or optional flat ID) cannot substitute that ID.
            if message["method"] == "turn/completed"
                && params["turn"]["id"].as_str() != Some(turn_id.as_str())
            {
                continue;
            }
            if first_event {
                timing::mark(self.timing.as_ref(), Stage::FirstAcceptedEvent);
                first_event = false;
            }
            if let Some(observer) = &mut self.observer { observer(&message); }
            let before = answer.text();
            if message["method"] == "item/completed"
                && matches!(
                    params["item"]["type"].as_str(),
                    Some("commandExecution" | "fileChange" | "mcpToolCall" | "dynamicToolCall")
                )
            {
                acted = true;
            }
            if let Some(result) = answer.event(&message)? {
                timing::mark(self.timing.as_ref(), Stage::AcceptedTurnCompletion);
                self.active_turn = None;
                return Ok(Reply {
                    text: result,
                    acted,
                    session_id: thread_id,
                });
            }
            let text = answer.text();
            if text != before {
                if first_text && !text.is_empty() {
                    timing::mark(self.timing.as_ref(), Stage::FirstPublicText);
                    first_text = false;
                }
                on_delta(text);
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.requests.disconnect(false);
        let cleanup = timing::Span::new(self.timing.as_ref(), Stage::Cleanup);
        if let Some((thread, turn)) = self.active_turn.take().filter(|_|!self.write_incomplete) {
            let _ = self.write_message(json!({ "id": self.next_id + 1, "method": "turn/interrupt", "params": { "threadId": thread, "turnId": turn } }));
        }
        self.input.take();
        for _ in 0..50 {
            if self.child.try_wait().ok().flatten().is_some() {
                timing::mark(self.timing.as_ref(), Stage::ChildReaped);
                cleanup.returned();
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
        let result = self.child.wait();
        if result.is_ok() { timing::mark(self.timing.as_ref(), Stage::ChildReaped); }
        cleanup.finish(&result);
    }
}

fn reap_failed_start(child: &mut Child, trace: Option<&Trace>) {
    let cleanup = timing::Span::new(trace, Stage::Cleanup);
    let _ = child.kill();
    let result = child.wait();
    if result.is_ok() { timing::mark(trace, Stage::ChildReaped); }
    cleanup.finish(&result);
}

fn read_message(reader: &mut impl BufRead, total: &mut usize) -> Result<Value, String> {
    let mut line = Vec::new();
    let n = reader
        .take(MAX_LINE as u64 + 1)
        .read_until(b'\n', &mut line)
        .map_err(|_| "Cannot read from Codex.")?;
    *total += n;
    if n == 0 {
        return Err("Codex closed the connection before finishing.".into());
    }
    if n > MAX_LINE || *total > MAX_TOTAL {
        return Err("Codex response exceeded the size limit.".into());
    }
    serde_json::from_slice(&line).map_err(|_| "Codex returned an invalid protocol message.".into())
}

fn rpc_error(error: &Value) -> String {
    error["message"]
        .as_str()
        .unwrap_or("Codex could not complete this request.")
        .chars()
        .take(2000)
        .collect()
}

fn history_items(history: &[Value]) -> Vec<Value> {
    history.iter().filter_map(|m| {
        let role = m["role"].as_str()?;
        if !matches!(role, "user" | "assistant") { return None; }
        let text = m["content"].as_str()?;
        Some(json!({ "type": "message", "role": role,
            "content": [{ "type": if role == "assistant" { "output_text" } else { "input_text" }, "text": text }]
        }))
    }).collect()
}

#[derive(Default)]
struct Answer {
    items: Vec<(String, String)>,
}

impl Answer {
    fn text(&self) -> String {
        self.items
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn event(&mut self, message: &Value) -> Result<Option<String>, String> {
        let p = &message["params"];
        let (id, text, append) = match message["method"].as_str() {
            Some("item/agentMessage/delta") => (p["itemId"].as_str(), p["delta"].as_str(), true),
            Some("item/completed") if p["item"]["type"] == "agentMessage" => {
                (p["item"]["id"].as_str(), p["item"]["text"].as_str(), false)
            }
            Some("turn/completed") => {
                if p["turn"]["status"] != "completed" {
                    return Err(rpc_error(&p["turn"]["error"]));
                }
                // Commentary belongs in progress bubbles, not the spoken final answer.
                let text = self.items.last().map(|(_, text)| text.clone()).unwrap_or_default();
                if text.trim().is_empty() {
                    return Err("Codex returned an empty reply.".into());
                }
                return Ok(Some(text));
            }
            _ => return Ok(None),
        };
        if let (Some(id), Some(text)) = (id, text) {
            if let Some((_, current)) = self.items.iter_mut().find(|(key, _)| key == id) {
                if append {
                    current.push_str(text);
                } else {
                    *current = text.to_string();
                }
            } else {
                self.items.push((id.to_string(), text.to_string()));
            }
        }
        if self.items.iter().map(|(_, text)| text.len()).sum::<usize>() > MAX_ANSWER {
            return Err("Codex reply exceeded the size limit.".into());
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_items_replace_deltas_without_duplicating_text() {
        let mut answer = Answer::default();
        answer
            .event(
                &json!({"method":"item/agentMessage/delta","params":{"itemId":"a","delta":"Olá"}}),
            )
            .unwrap();
        answer.event(&json!({"method":"item/completed","params":{"item":{"id":"a","type":"agentMessage","text":"Olá!"}}})).unwrap();
        assert_eq!(answer.text(), "Olá!");
        assert_eq!(
            answer
                .event(&json!({"method":"turn/completed","params":{"turn":{"status":"completed"}}}))
                .unwrap(),
            Some("Olá!".into())
        );
    }

    #[test]
    fn commentary_is_progress_and_only_the_final_message_is_returned_for_speech() {
        let mut answer=Answer::default();
        answer.event(&json!({"method":"item/completed","params":{"item":{"id":"progress","type":"agentMessage","text":"Vou criar o Postgres."}}})).unwrap();
        answer.event(&json!({"method":"item/completed","params":{"item":{"id":"final","type":"agentMessage","text":"O Postgres está pronto."}}})).unwrap();
        assert!(answer.text().contains("Vou criar"));
        assert_eq!(answer.event(&json!({"method":"turn/completed","params":{"turn":{"status":"completed"}}})).unwrap(),Some("O Postgres está pronto.".into()));
    }

    #[test]
    fn a_failed_or_interrupted_turn_never_commits_partial_text() {
        for status in ["failed", "interrupted"] {
            let mut answer = Answer {
                items: vec![("a".into(), "partial".into())],
            };
            assert!(answer.event(&json!({"method":"turn/completed","params":{"turn":{"status":status,"error":{"message":"Try later"}}}})).is_err());
        }
        assert!(Answer::default()
            .event(&json!({"method":"turn/completed","params":{"turn":{"status":"completed"}}}))
            .is_err());
    }

    #[test]
    fn history_keeps_roles_and_skips_native_tool_blocks() {
        let items = history_items(&[
            json!({"role":"user","content":"Lembra-te: azul"}),
            json!({"role":"assistant","content":"Combinado"}),
            json!({"role":"assistant","content":[{"type":"tool_use"}]}),
        ]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["content"][0]["type"], "input_text");
        assert_eq!(items[1]["content"][0]["type"], "output_text");
    }

    #[test]
    fn the_reader_bounds_lines_total_bytes_and_invalid_json() {
        for data in [vec![b'x'; MAX_LINE + 1], b"not json\n".to_vec(), vec![]] {
            assert!(read_message(&mut data.as_slice(), &mut 0).is_err());
        }
        let mut total = MAX_TOTAL;
        assert!(read_message(&mut b"{}\n".as_slice(), &mut total).is_err());
    }

    #[cfg(unix)]
    pub(super) fn mock() -> (std::path::PathBuf, std::path::PathBuf) {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "coucou-rpc-{}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed),
            crate::flow::text::new_id()
        ));
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        // Execute the stable interpreter, with our fixture as the `app-server`
        // argument. Parallel forks can briefly retain a freshly written script
        // descriptor and make direct shebang exec fail with ETXTBSY on Linux.
        let exe = std::path::PathBuf::from("/usr/bin/python3");
        std::fs::write(dir.join("app-server"), r#"#!/usr/bin/python3
import json,sys,pathlib,time
def send(message):
    print(json.dumps(message),flush=True)
for line in sys.stdin:
    m=json.loads(line); method=m.get('method'); p=m.get('params',{}); result={}
    if method=='initialized': continue
    if method=='account/read': result={'account':{'type':'apiKey' if pathlib.Path('api-key').exists() else 'chatgpt'}}
    if method=='thread/name/set':pathlib.Path('name.json').write_text(json.dumps(p))
    if method in ('thread/start','thread/resume'):
        pathlib.Path('request.json').write_text(json.dumps(p))
        pathlib.Path('request-method.txt').write_text(method)
        result={'thread':{'id':'owned-thread'}}
        if method=='thread/resume':
            result['thread'].update({'name':'Original native terminal session'})
    if method=='turn/start':
        pathlib.Path('turn.json').write_text(json.dumps(p))
        send({'method':'item/agentMessage/delta','params':{'threadId':'owned-thread','turnId':'turn-1','itemId':'a','delta':'partial'}})
        result={'turn':{'id':'turn-1'}}
    if method is not None:
        send({'id':m['id'],'result':result})
    if method=='turn/start':
        send({'id':99,'method':'item/commandExecution/requestApproval','params':{'threadId':'owned-thread','turnId':'turn-1','command':'echo hello'}})
    if m.get('id')==99 and 'method' not in m:
        text='accepted' if m.get('result',{}).get('decision')=='accept' else 'denied'
        send({'method':'item/completed','params':{'threadId':'owned-thread','turnId':'turn-1','item':{'type':'agentMessage','id':'a','text':text}}})
        send({'method':'turn/completed','params':{'threadId':'owned-thread','turn':{'id':'turn-1','status':'completed'}}})
"#).unwrap();
        (dir, exe)
    }

    #[test]
    #[cfg(unix)]
    fn native_permissions_are_inherited_and_early_events_are_not_lost() {
        for (full_access,session_id) in [(false,None),(true,None),(false,Some("owned-thread")),(true,Some("owned-thread"))] {
            let (cwd, exe) = mock();
            let mut client = Client::start(&exe, &cwd, Duration::from_secs(5), true).unwrap();
            let mut asked = false;
            let mut deltas = Vec::new();
            let reply = client
                .answer(
                    TurnOptions {
                        model: "account-model",
                        system: "Mochi",
                        history: &[],
                        query: "hello",
                        tools: &[],
                        session_id,
                        full_access,
                        effort: if full_access {Some("low")} else {None},
                        name: Some("Mochi · Codex"),
                    },
                    |_, _| (String::new(), false),
                    |method, params| {
                        assert_eq!(method, "item/commandExecution/requestApproval");
                        assert_eq!(params["command"], "echo hello");
                        asked = true;
                        Some(json!({"decision":"accept"}))
                    },
                    |_| {},
                    |text| deltas.push(text),
                )
                .unwrap();
            let sent_turn: Value = serde_json::from_slice(&std::fs::read(cwd.join("turn.json")).unwrap()).unwrap();
            assert_eq!(sent_turn["effort"], if full_access {json!("low")} else {Value::Null});
            if session_id.is_none(){let label:Value=serde_json::from_slice(&std::fs::read(cwd.join("name.json")).unwrap()).unwrap();assert_eq!(label["threadId"],"owned-thread");assert_eq!(label["name"],"Mochi · Codex");}else{assert!(!cwd.join("name.json").exists(),"resume preserves the original native session name");}
            assert!(asked);
            assert!(deltas.iter().any(|text| text == "partial"));
            assert_eq!(reply.text, "accepted");
            assert_eq!(reply.session_id, "owned-thread");
            let request: Value =
                serde_json::from_slice(&std::fs::read(cwd.join("request.json")).unwrap()).unwrap();
            assert_eq!(
                request["sandbox"],
                if full_access && session_id.is_none() {
                    json!("danger-full-access")
                } else {
                    Value::Null
                }
            );
            assert_eq!(std::fs::read_to_string(cwd.join("request-method.txt")).unwrap(),if session_id.is_some(){"thread/resume"}else{"thread/start"});
            assert_eq!(request.get("approvalPolicy").cloned(),if full_access&&session_id.is_none(){Some(json!("never"))}else{None});
            if session_id.is_some(){assert_eq!(request["threadId"],"owned-thread");assert!(request.get("sandbox").is_none(),"resume never reapplies a cached full-access policy");}
            assert!(
                request.get("config").is_none(),
                "native MCP and shell configuration is inherited"
            );
            drop(client);
            std::fs::remove_dir_all(cwd).unwrap();
        }
    }

    #[test]
    #[cfg(unix)]
    fn an_api_key_account_is_never_used_as_a_chatgpt_account() {
        let (cwd, exe) = mock();
        std::fs::write(cwd.join("api-key"), "").unwrap();
        let mut client = Client::start(&exe, &cwd, Duration::from_secs(5), false).unwrap();
        assert_eq!(client.require_account().unwrap_err(), SIGN_IN);
        drop(client);
        std::fs::remove_dir_all(cwd).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn cancellation_stops_waiting_for_a_cli_reply() {
        let (cwd, exe) = mock();
        let mut client = Client::start(&exe, &cwd, Duration::from_secs(5), false).unwrap();
        client.cancel_with(Arc::new(AtomicBool::new(true)));
        assert_eq!(
            client.require_account().unwrap_err(),
            "The CLI request was cancelled."
        );
        drop(client);
        std::fs::remove_dir_all(cwd).unwrap();
    }
}

// Synthetic stdio server only: stable /usr/bin/python3 + the existing mock()
// fixture directory. No real Codex, accounts, inference or global settings.
#[cfg(all(test, unix))]
mod initialize_cancel_tests {
    use super::*;
    use std::io::Write;
    use super::tests::mock;
    use std::path::PathBuf;

    const CANCELLED: &str = "The CLI request was cancelled.";
    const SERVER: &str = r#"import json,os,pathlib,sys,time
root=pathlib.Path('.')
mode=(root/'fixture-mode').read_text()
(root/'pid').write_text(str(os.getpid()))
def send(value):
    print(json.dumps(value),flush=True)
for line in sys.stdin:
    value=json.loads(line)
    with (root/'requests.jsonl').open('a') as log:
        log.write(json.dumps(value)+'\n')
    method=value.get('method')
    if method=='initialize':
        (root/'pid').write_text(str(os.getpid()))
        (root/'initialize.ready').write_text('ready')
        if mode=='hang-initialize':
            while True: time.sleep(1)
    elif method=='initialized':
        (root/'initialized.ready').write_text('ready')
        if mode=='block-input':
            while True: time.sleep(1)
        continue
    elif method=='probe/hang':
        (root/'probe.ready').write_text('ready')
        continue
    elif method=='turn/interrupt':
        (root/'interrupt.json').write_text(json.dumps(value))
    if 'id' in value:
        send({'id':value['id'],'result':{'synthetic':True}})
"#;

    struct Fixture { cwd:PathBuf, exe:PathBuf }
    impl Fixture {
        fn new(mode:&str) -> Self {
            let(cwd,exe)=mock();
            // Only overwrite the script inside this existing test fixture.
            // Execute the stable interpreter, never a newly written executable.
            std::fs::write(cwd.join("app-server"),SERVER).unwrap();
            std::fs::write(cwd.join("fixture-mode"),mode).unwrap();
            Self{cwd,exe}
        }
        fn pid(&self)->libc::pid_t {wait_file(&self.cwd.join("pid")).trim().parse().unwrap()}
        fn requests(&self)->Vec<Value> {
            std::fs::read_to_string(self.cwd.join("requests.jsonl")).unwrap_or_default()
                .lines().map(|line|serde_json::from_str(line).unwrap()).collect()
        }
        fn methods(&self)->Vec<String> {
            self.requests().iter().map(|value|value["method"].as_str().unwrap().to_owned()).collect()
        }
    }
    impl Drop for Fixture {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.cwd);}}

    fn wait_file(path:&Path)->String {
        let deadline=Instant::now()+Duration::from_secs(2);
        loop {
            if let Ok(text)=std::fs::read_to_string(path) {if !text.is_empty(){return text;}}
            assert!(Instant::now()<deadline,"Synthetic server did not reach its readiness marker");
            // Poll a concrete readiness condition, not an assumed startup delay.
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    // Bound a broken initialization regression too. On an assertion/receive
    // timeout, terminate only this synthetic child and join its owning thread.
    struct Attempt<T> {
        result:mpsc::Receiver<T>, worker:Option<std::thread::JoinHandle<()>>,
        cancel:Arc<AtomicBool>, pid_path:PathBuf,
    }
    impl<T:Send+'static> Attempt<T> {
        fn new(fixture:&Fixture,cancel:Arc<AtomicBool>,job:impl FnOnce()->T+Send+'static)->Self {
            let(tx,result)=mpsc::channel();
            let worker=std::thread::spawn(move||{let _=tx.send(job());});
            Self{result,worker:Some(worker),cancel,pid_path:fixture.cwd.join("pid")}
        }
        fn finish(mut self)->T {
            let result=self.result.recv_timeout(Duration::from_secs(3)).expect("Synthetic worker exceeded its bounded deadline");
            self.worker.take().unwrap().join().unwrap();result
        }
    }
    impl<T> Drop for Attempt<T> {
        fn drop(&mut self) {
            let Some(worker)=self.worker.take()else{return;};
            self.cancel.store(true,Ordering::Release);
            if !worker.is_finished() {
                if let Some(pid)=std::fs::read_to_string(&self.pid_path).ok().and_then(|text|text.trim().parse::<libc::pid_t>().ok()).filter(|pid|*pid>1) {
                    // This path is written only by our fixture's own process.
                    unsafe{libc::kill(pid,libc::SIGKILL);}
                }
            }
            let _=worker.join();
        }
    }

    fn assert_reaped(pid:libc::pid_t) {
        assert!(pid>1);
        assert_eq!(unsafe{libc::kill(pid,0)},-1,"Synthetic child is still alive or unreaped");
        assert_eq!(std::io::Error::last_os_error().raw_os_error(),Some(libc::ESRCH));
        let mut status=0;
        assert_eq!(unsafe{libc::waitpid(pid,&mut status,libc::WNOHANG)},-1);
        assert_eq!(std::io::Error::last_os_error().raw_os_error(),Some(libc::ECHILD));
    }

    fn assert_no_account_or_turn_start(fixture:&Fixture) {
        assert!(fixture.methods().iter().all(|method|method!="account/read"&&method!="turn/start"));
    }

    #[test]
    fn pre_cancelled_start_refuses_before_spawning_or_initializing() {
        let f=Fixture::new("handshake");let cancel=Arc::new(AtomicBool::new(true));
        let missing=f.cwd.join("definitely-not-an-executable");
        let before=Instant::now();
        let result=Client::start_with_cancel(&missing,&f.cwd,Duration::from_secs(1800),false,cancel);
        assert_eq!(result.err().unwrap(),CANCELLED);assert!(before.elapsed()<Duration::from_secs(3));
        assert!(!f.cwd.join("pid").exists());assert!(!f.cwd.join("initialize.ready").exists());
        assert!(f.requests().is_empty());
    }

    #[test]
    fn cancellation_during_initialize_is_prompt_and_reaps_the_child() {
        let f=Fixture::new("hang-initialize");let cancel=Arc::new(AtomicBool::new(false));
        let flag=cancel.clone();let exe=f.exe.clone();let cwd=f.cwd.clone();let began=Instant::now();
        let attempt=Attempt::new(&f,cancel.clone(),move||Client::start_with_cancel(&exe,&cwd,Duration::from_secs(1800),true,flag));
        wait_file(&f.cwd.join("initialize.ready"));let pid=f.pid();
        cancel.store(true,Ordering::Release);
        assert_eq!(attempt.finish().err().unwrap(),CANCELLED);
        assert!(began.elapsed()<Duration::from_secs(3));assert_reaped(pid);
        assert_eq!(f.methods(),vec!["initialize"]);assert_no_account_or_turn_start(&f);
    }

    #[test]
    fn initialize_timeout_is_bounded_independently_from_a_thirty_minute_task() {
        let f=Fixture::new("hang-initialize");let cancel=Arc::new(AtomicBool::new(false));
        let flag=cancel.clone();let exe=f.exe.clone();let cwd=f.cwd.clone();let began=Instant::now();
        let attempt=Attempt::new(&f,cancel,move||Client::start_inner(&exe,&cwd,Duration::from_secs(1800),false,Some(flag),Duration::from_secs(1)));
        let error=attempt.finish().err().unwrap();
        assert!(error.contains("in time"));assert!(began.elapsed()<Duration::from_secs(3));
        assert_reaped(f.pid());assert_eq!(f.methods(),vec!["initialize"]);assert_no_account_or_turn_start(&f);
    }

    #[test]
    fn a_successful_initialize_restores_the_original_five_second_deadline() {
        let f=Fixture::new("handshake");
        let mut client=Client::start_inner(&f.exe,&f.cwd,Duration::from_secs(5),false,None,Duration::from_secs(1)).unwrap();
        wait_file(&f.cwd.join("initialized.ready"));let pid=client.child.id()as libc::pid_t;
        // Proves the handshake deadline did not become the whole task deadline,
        // without relying on an arbitrary sleep past that shorter deadline.
        assert!(client.deadline.saturating_duration_since(Instant::now())>Duration::from_secs(3));
        assert_eq!(client.request("probe/metadata",json!({})).unwrap(),json!({"synthetic":true}));
        drop(client);assert_reaped(pid);
        assert_eq!(f.methods(),vec!["initialize","initialized","probe/metadata"]);
        assert_no_account_or_turn_start(&f);
    }

    #[test]
    fn cancelling_after_initialize_prevents_account_turn_and_notification_writes() {
        let f=Fixture::new("handshake");let cancel=Arc::new(AtomicBool::new(false));
        let mut client=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(5),false,cancel.clone()).unwrap();
        wait_file(&f.cwd.join("initialized.ready"));let pid=client.child.id()as libc::pid_t;
        cancel.store(true,Ordering::Release);
        assert_eq!(client.require_account().unwrap_err(),CANCELLED);
        assert_eq!(client.request("turn/start",json!({"threadId":"synthetic-thread","input":[]})).unwrap_err(),CANCELLED);
        assert_eq!(client.write(json!({"method":"synthetic/notification"})).unwrap_err(),CANCELLED);
        drop(client);assert_reaped(pid);
        assert_eq!(f.methods(),vec!["initialize","initialized"]);assert_no_account_or_turn_start(&f);
    }

    #[test]
    fn drop_interrupts_its_exact_turn_even_when_the_cancellation_flag_is_set() {
        let f=Fixture::new("handshake");let cancel=Arc::new(AtomicBool::new(false));
        let mut client=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(5),false,cancel.clone()).unwrap();
        wait_file(&f.cwd.join("initialized.ready"));let pid=client.child.id()as libc::pid_t;
        client.active_turn=Some(("synthetic-exact-thread".into(),"synthetic-exact-turn".into()));
        cancel.store(true,Ordering::Release);drop(client);assert_reaped(pid);
        let request:Value=serde_json::from_slice(&std::fs::read(f.cwd.join("interrupt.json")).unwrap()).unwrap();
        assert_eq!(request["method"],"turn/interrupt");
        assert_eq!(request["params"],json!({"threadId":"synthetic-exact-thread","turnId":"synthetic-exact-turn"}));
        assert_eq!(f.methods(),vec!["initialize","initialized","turn/interrupt"]);
        assert_no_account_or_turn_start(&f);
    }

    #[test]
    fn the_initial_cancel_token_remains_attached_while_waiting_after_handshake() {
        let f=Fixture::new("handshake");let cancel=Arc::new(AtomicBool::new(false));
        let flag=cancel.clone();let exe=f.exe.clone();let cwd=f.cwd.clone();let began=Instant::now();
        let attempt=Attempt::new(&f,cancel.clone(),move||{
            let mut client=Client::start_with_cancel(&exe,&cwd,Duration::from_secs(1800),false,flag)?;
            client.request("probe/hang",json!({}))
        });
        wait_file(&f.cwd.join("probe.ready"));let pid=f.pid();cancel.store(true,Ordering::Release);
        assert_eq!(attempt.finish().unwrap_err(),CANCELLED);
        assert!(began.elapsed()<Duration::from_secs(3));assert_reaped(pid);
        assert_eq!(f.methods(),vec!["initialize","initialized","probe/hang"]);
        assert_no_account_or_turn_start(&f);
    }

    #[cfg(target_os="linux")]
    fn monitor_pipe(client:&Client)->std::os::fd::OwnedFd{
        use std::os::fd::{AsRawFd,FromRawFd};
        let fd=unsafe{libc::dup(client.input.as_ref().unwrap().as_raw_fd())};assert!(fd>=0);
        unsafe{std::os::fd::OwnedFd::from_raw_fd(fd)}
    }
    #[cfg(target_os="linux")]
    fn wait_full_pipe(fd:&std::os::fd::OwnedFd){
        use std::os::fd::AsRawFd;
        let size=unsafe{libc::fcntl(fd.as_raw_fd(),libc::F_GETPIPE_SZ)};assert!(size>0);
        let deadline=Instant::now()+Duration::from_secs(2);
        loop{
            let mut bytes=0;assert_eq!(unsafe{libc::ioctl(fd.as_raw_fd(),libc::FIONREAD,&mut bytes)},0);
            if bytes>=size{return;}
            assert!(Instant::now()<deadline,"Synthetic write never reached real backpressure");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]#[cfg(target_os="linux")]
    fn a_large_write_to_a_stalled_child_obeys_stop_and_reaps_only_that_child(){
        let f=Fixture::new("block-input");let cancel=Arc::new(AtomicBool::new(false));
        let mut client=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(1800),false,cancel.clone()).unwrap();
        wait_file(&f.cwd.join("initialized.ready"));let pid=f.pid();let monitor=monitor_pipe(&client);
        client.active_turn=Some(("synthetic-thread".into(),"synthetic-turn".into()));
        let attempt=Attempt::new(&f,cancel.clone(),move||client.write(json!({"method":"probe/large","params":{"text":"x".repeat(2*1024*1024)}})));
        wait_full_pipe(&monitor);cancel.store(true,Ordering::Release);
        assert_eq!(attempt.finish().unwrap_err(),CANCELLED);drop(monitor);assert_reaped(pid);
        assert_eq!(f.methods(),vec!["initialize","initialized"]);
    }
    #[test]#[cfg(target_os="linux")]
    fn a_stalled_write_uses_the_task_deadline_instead_of_hanging_inside_write_all(){
        let f=Fixture::new("block-input");let cancel=Arc::new(AtomicBool::new(false));
        let mut client=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(1800),false,cancel.clone()).unwrap();
        wait_file(&f.cwd.join("initialized.ready"));let pid=f.pid();let monitor=monitor_pipe(&client);
        client.deadline=Instant::now()+Duration::from_secs(1);
        let attempt=Attempt::new(&f,cancel,move||client.write(json!({"method":"probe/large","params":{"text":"x".repeat(2*1024*1024)}})));
        wait_full_pipe(&monitor);assert!(attempt.finish().unwrap_err().contains("in time"));drop(monitor);assert_reaped(pid);
    }
    #[test]#[cfg(target_os="linux")]
    fn drop_interrupt_has_its_own_short_budget_when_the_pipe_is_full(){
        use std::os::fd::AsRawFd;
        let f=Fixture::new("block-input");let cancel=Arc::new(AtomicBool::new(false));
        let mut client=Client::start_with_cancel(&f.exe,&f.cwd,Duration::from_secs(1800),false,cancel.clone()).unwrap();
        wait_file(&f.cwd.join("initialized.ready"));let pid=f.pid();
        let flags=unsafe{libc::fcntl(client.input.as_ref().unwrap().as_raw_fd(),libc::F_GETFL)};
        assert_ne!(flags&libc::O_NONBLOCK,0,"Fixture filling must itself be nonblocking");
        loop{match client.input.as_mut().unwrap().write(b"{}\n"){
            Ok(3)=>{},Err(error)if error.kind()==std::io::ErrorKind::WouldBlock=>break,
            other=>panic!("Unexpected synthetic pipe fill: {other:?}"),
        }}
        client.active_turn=Some(("synthetic-thread".into(),"synthetic-turn".into()));
        let attempt=Attempt::new(&f,cancel,move||{let began=Instant::now();drop(client);began.elapsed()});
        assert!(attempt.finish()<Duration::from_secs(2));assert_reaped(pid);
    }
}

#[cfg(test)]
#[path="rpc_timing_tests.rs"]
mod timing_tests;

#[cfg(all(test,unix))]
#[path="rpc_turn_identity_tests.rs"]
mod turn_identity_tests;

#[cfg(all(test,unix))]#[path="rpc_native_requests_tests.rs"]mod native_request_tests;

#[cfg(all(test,target_os="linux"))]#[path="rpc_native_policy_tests.rs"]mod native_policy_tests;

#[cfg(all(test,unix))]#[path="rpc_native_tool_tests.rs"]mod native_tool_tests;
