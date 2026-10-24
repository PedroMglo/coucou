//! Bundled format capabilities, independent from live permissions or grants.
//! The chooser imports the same JSON; unknown formats still require real text
//! decoding and unknown routes never acquire a capability through a fallback.
use serde::Deserialize;
use std::{collections::BTreeMap,path::Path,sync::OnceLock};

#[derive(Clone,Copy,Deserialize)]
#[serde(rename_all="lowercase")]
enum Route{Native,Local,Api}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Policy{schema_version:u32,routes:BTreeMap<String,Route>,images:Vec<String>,pdf:Vec<String>,unsupported_binary:Vec<String>}
fn policy()->&'static Policy{
    static POLICY:OnceLock<Policy>=OnceLock::new();
    POLICY.get_or_init(||{let p:Policy=serde_json::from_str(include_str!("../../documents.policy.json")).expect("Invalid bundled document policy");assert_eq!(p.schema_version,1);p})
}
#[derive(Debug,PartialEq,Eq)]
pub(crate) enum Format{Folder,Text,Pdf,Image,Binary}
pub(crate) fn format(name:&str,folder:bool)->Format{
    if folder{return Format::Folder;}
    let ext=Path::new(name).extension().and_then(|s|s.to_str()).unwrap_or_default().to_ascii_lowercase();let p=policy();
    if p.images.contains(&ext){Format::Image}else if p.pdf.contains(&ext){Format::Pdf}else if p.unsupported_binary.contains(&ext){Format::Binary}else{Format::Text}
}
pub(crate) fn can_analyze(provider:&str,name:&str,folder:bool)->bool{
    if crate::pet_config::integration_pet(provider).is_some_and(crate::pet_config::retired){return false;}
    match policy().routes.get(provider){
        Some(Route::Native)=>crate::cli_chat::backend(provider).is_some(),
        Some(Route::Local)=>matches!(format(name,folder),Format::Folder|Format::Text|Format::Pdf),
        Some(Route::Api)=>format(name,folder)!=Format::Binary,
        None=>false,
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn cloud_local_and_native_routes_share_a_bounded_folder_capability(){
        for provider in ["codex","copilot","antigravity","gemini-cli","claude-cli","ollama","lmstudio","anthropic","openai","google","openrouter","custom","integration_notion"]{
            assert!(can_analyze(provider,"folder.zip",true),"{provider}");assert!(can_analyze(provider,"notes.md",false));assert!(can_analyze(provider,"REPORT.PDF",false));
            assert_eq!(can_analyze(provider,"IMAGE.PNG",false),!matches!(provider,"ollama"|"lmstudio"));
            assert_eq!(can_analyze(provider,"report.docx",false),crate::cli_chat::backend(provider).is_some());
        }
        for provider in ["integration_spotify","integration_github","unknown",""]{assert!(!can_analyze(provider,"folder",true));assert!(!can_analyze(provider,"text.txt",false));}
    }
    #[test]
    fn unsupported_binary_never_masquerades_as_utf8_text_for_api_or_local_pets(){
        for ext in &policy().unsupported_binary{
            let name=format!("sample.{ext}");assert_eq!(format(&name,false),Format::Binary);
            for provider in ["ollama","lmstudio","openai","anthropic","google"]{assert!(!can_analyze(provider,&name,false),"{provider}/{ext}");}
            assert!(can_analyze("codex",&name,false));
        }
        let mut extensions=std::collections::HashSet::new();for ext in policy().images.iter().chain(&policy().pdf).chain(&policy().unsupported_binary){assert!(extensions.insert(ext));assert!(ext.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()));}
    }
}
