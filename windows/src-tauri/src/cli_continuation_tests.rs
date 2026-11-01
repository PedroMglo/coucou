//! Shared CLI continuation intent. No provider executable, account or model IO.
use super::*;
use crate::chat::CliSession;
fn session(full_access:bool)->CliSession{CliSession{id:"original-native-session".into(),cwd:"/private/original".into(),full_access}}
#[test]fn exact_native_sessions_and_explicit_new_conversations_keep_all_declared_cli_owners(){
 for(owner,provider)in[("codex","codex"),("copilot","copilot"),("antigravity","antigravity"),("gemini-cli","antigravity"),("claude-cli","antigravity"),("integration_notion","codex"),("integration_spotify","codex")]{
  for full in[false,true]{let saved=session(full);assert_eq!(managed_session(provider,owner,Some(&saved),"/private/original",full).unwrap(),Some("original-native-session"));assert_eq!(managed_session(provider,owner,None,"/private/original",full).unwrap(),None);}
 }
}
#[test]fn changed_workspaces_or_creation_preferences_never_become_a_fresh_session(){
 for provider in["codex","antigravity","copilot"]{for full in[false,true]{let saved=session(full);assert!(managed_session(provider,provider,Some(&saved),"/private/other",full).unwrap_err().contains("Nova conversa"));
  let changed=managed_session(provider,provider,Some(&saved),"/private/original",!full);if provider=="copilot"{assert_eq!(changed.unwrap(),Some("original-native-session"));}else{assert!(changed.unwrap_err().contains("acesso total"));}
 }}
}
#[test]fn invalid_ids_and_crossed_native_adapters_cannot_select_or_create_a_binding(){
 for id in[""," ","native\nrequest","native\0request"]{let mut saved=session(false);saved.id=id.into();assert!(managed_session("codex","codex",Some(&saved),"/private/original",false).is_err());}
 let mut saved=session(false);saved.id="x".repeat(513);assert!(managed_session("antigravity","antigravity",Some(&saved),"/private/original",false).is_err());
 for(provider,owner)in[("codex","antigravity"),("antigravity","copilot"),("copilot","codex"),("copilot","integration_spotify"),("lmstudio","lmstudio"),("codex","unknown")]{assert!(managed_session(provider,owner,None,"/private/original",false).is_err());}
}
