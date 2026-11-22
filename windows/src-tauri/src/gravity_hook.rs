//! A per-call human decision for an app-owned Gravity shell tool. The official
//! relay and ordinary external CLI sessions keep their existing behaviour.
//! https://www.antigravity.google/docs/hooks/#pretooluse
use serde_json::{json,Value};
use std::io::{Read,Write};

const MAX_INPUT:usize=256*1024;
pub const OWNED_ENV:&str="COUCOU_ASSISTANT_GATE";

fn request(raw:&Value)->Option<Value> {
    if raw["toolCall"]["name"]!="run_command" {return None;}
    let session=raw["conversationId"].as_str().filter(|s|!s.is_empty() && s.len()<=512&&!s.chars().any(char::is_control))?;
    let command=raw["toolCall"]["args"]["CommandLine"].as_str()?;
    let mut payload=json!({"hook_event_name":"PermissionRequest","coucou_agent":"antigravity","coucou_assistant_gate":true,
        "session_id":session,"cwd":raw["toolCall"]["args"]["Cwd"].as_str().or_else(||raw["workspacePaths"][0].as_str()).unwrap_or(""),
        "tool_name":"Bash","tool_input":{"command":command},"coucou_native_call":{"name":"run_command","args":raw["toolCall"]["args"]}});
    if let Some(step)=raw["stepIdx"].as_u64(){payload["coucou_native_step"]=json!(step);}
    Some(payload)
}
fn decision(reply:&str)->Value {
    match reply.trim() {
        // Only the app's explicit human click can produce this response.
        "allow"=>json!({"decision":"auto_approve","reason":"Approved once by the user in Coucou."}),
        "deny"=>json!({"decision":"deny","reason":"Declined by the user in Coucou."}),
        _=>json!({"decision":"ask"}),
    }
}

#[cfg(target_os="linux")]
fn ask(payload:&Value)->Option<String> {
    ask_at(payload,&crate::platform::relay_socket_path()?)
}
#[cfg(target_os="linux")]
fn ask_at(payload:&Value,path:&std::path::Path)->Option<String> {
    use std::os::unix::net::UnixStream;
    use std::time::Duration;
    let mut stream=UnixStream::connect(path).ok()?;
    stream.set_write_timeout(Some(Duration::from_millis(300))).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(110))).ok()?;
    serde_json::to_writer(&mut stream,payload).ok()?;stream.write_all(b"\n").ok()?;
    // Keep the write side open: the server observes EOF if the owning CLI dies.
    let mut reply=String::new();stream.take(64).read_to_string(&mut reply).ok()?;Some(reply)
}
#[cfg(not(target_os="linux"))]
fn ask(_: &Value)->Option<String>{None}

/// Runs before Tauri or audio starts. No credentials/configuration are loaded.
pub fn maybe_run()->bool {
    let args:Vec<_>=std::env::args().collect();
    if !args.iter().any(|a|a=="--gravity-pre-tool") {return false;}
    let mut raw=Vec::new();let _=std::io::stdin().take(MAX_INPUT as u64+1).read_to_end(&mut raw);
    let payload=(raw.len()<=MAX_INPUT).then(||serde_json::from_slice::<Value>(&raw).ok()).flatten();
    let owned=std::env::var(OWNED_ENV).is_ok_and(|v|v=="1");
    if let Some(payload)=payload.as_ref().filter(|_|owned).and_then(request) {
        let reply=ask(&payload).unwrap_or_default();println!("{}",decision(&reply));return true;
    }
    // The installer supplies the existing trusted relay path. Do not reimplement
    // its normalisation or external-agent policy here.
    if let Some(relay)=args.windows(2).find(|a|a[0]=="--relay").map(|a|&a[1]) {
        use std::process::{Command,Stdio};
        if let Ok(mut child)=Command::new(relay).args(["--agent","antigravity","PreToolUse"]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            if let Some(mut input)=child.stdin.take() {let _=input.write_all(&raw);}
            let _=child.wait();
        }
    }
    println!("{}",decision(""));true
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn only_an_explicit_single_allow_click_grants_the_current_tool(){
        assert_eq!(decision("allow\n")["decision"],"auto_approve");assert_eq!(decision("deny")["decision"],"deny");
        for reply in ["","always","true","unowned","allow\ndeny","{\"allow\":true}"] {assert_eq!(decision(reply)["decision"],"ask");}
    }
    #[test]fn exact_native_arguments_are_preserved_privately_without_transcript_or_model_metadata(){
        let mut raw=json!({"conversationId":"owned-id","toolCall":{"name":"run_command","args":{"CommandLine":"pwd","Cwd":"/tmp/project","private":"secret"}},"transcriptPath":"private","modelName":"model"});
        let p=request(&raw).unwrap();assert_eq!(p["session_id"],"owned-id");assert_eq!(p["tool_input"],json!({"command":"pwd"}));assert!(p.get("transcriptPath").is_none()&&p.get("modelName").is_none());assert_eq!(p["coucou_native_call"]["args"],raw["toolCall"]["args"]);assert!(p.get("nativeIdentity").is_none());
        raw["toolCall"]["name"]=json!("view_file");assert!(request(&raw).is_none());raw["toolCall"]["name"]=json!("run_command");raw["conversationId"]=json!("");assert!(request(&raw).is_none());
    }
    #[test]fn a_native_step_index_is_optional_evidence_never_a_request_or_turn_id(){
        let mut raw=json!({"conversationId":"native-session","stepIdx":7,"toolCall":{"id":"unadvertised-id","name":"run_command","args":{"CommandLine":"synthetic"}}});
        let p=request(&raw).unwrap();assert_eq!(p["coucou_native_step"],7);assert!(p.get("requestId").is_none()&&p.get("turnId").is_none());assert!(p["coucou_native_call"].get("id").is_none());
        raw["stepIdx"]=json!(-1);assert!(request(&raw).unwrap().get("coucou_native_step").is_none());raw["conversationId"]=json!("bad\nidentity");assert!(request(&raw).is_none());
    }
    #[cfg(target_os="linux")]
    #[test]fn a_private_socket_carries_one_exact_request_and_disconnect_never_grants_it(){
        use std::os::unix::net::UnixListener;
        use std::io::BufRead;
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path=std::env::temp_dir().join(format!("coucou-gate-{}-{nonce:x}",std::process::id()));
        let listener=UnixListener::bind(&path).unwrap();let expected=json!({"session_id":"own-1","tool_input":{"command":"pwd"}});let copy=expected.clone();
        let server=std::thread::spawn(move||{let (mut socket,_)=listener.accept().unwrap();let mut line=String::new();std::io::BufReader::new(socket.try_clone().unwrap()).read_line(&mut line).unwrap();assert_eq!(serde_json::from_str::<Value>(&line).unwrap(),copy);socket.write_all(b"deny\n").unwrap();});
        assert_eq!(decision(&ask_at(&expected,&path).unwrap())["decision"],"deny");server.join().unwrap();std::fs::remove_file(&path).unwrap();
        assert!(ask_at(&expected,&path).is_none());assert_eq!(decision("")["decision"],"ask");
    }
}
