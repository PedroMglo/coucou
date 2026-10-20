//! Passive permission projections from ONE connected RPC Client. No requests,
//! mutation, native approval decisions or authorization capabilities exist here.
//! Receive order is local evidence, never a server revision or mutation receipt.
use std::collections::BTreeMap;
use serde_json::Value;

const MAX_THREADS:usize=128;
const MAX_ROOT_BYTES:usize=16*1024;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum ApprovalPolicy {
    Untrusted,OnRequest,Never,
    Granular{sandbox_approval:bool,rules:bool,mcp_elicitations:bool,request_permissions:bool,skill_approval:bool},
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Reviewer {User,AutoReview,GuardianSubagent}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum SandboxPolicy {
    DangerFullAccess,ReadOnly{network_access:bool},
    ExternalSandbox{network_access:String},
    WorkspaceWrite{network_access:bool,writable_roots:Vec<String>,exclude_tmpdir_env_var:bool,exclude_slash_tmp:bool},
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Profile {pub id:String,pub extends:Option<String>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Settings {
    pub approval_policy:ApprovalPolicy,pub reviewer:Reviewer,pub sandbox_policy:SandboxPolicy,
    pub profile:Option<Profile>,pub cwd:String,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Reason {NotObserved,InvalidThread,InvalidNotification,ThreadClosed,Disconnected,Capacity,Expired,SequenceExhausted}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Observation {
    pub thread_id:String,
    /// Historical projection only. Even a non-stale observation is NOT an
    /// authority to answer requests, write policy, or claim a matching terminal.
    pub last_observed:Option<Settings>,
    pub observed_at:Option<u64>,pub sequence:Option<u64>,pub stale:bool,pub reason:Option<Reason>,
}
struct Entry {last:Option<Settings>,at:Option<u64>,sequence:Option<u64>,reason:Option<Reason>,closed:bool}
pub(super) struct Cache {entries:BTreeMap<String,Entry>,sequence:u64,retired:Option<Reason>}
impl Default for Cache {fn default()->Self{Self{entries:BTreeMap::new(),sequence:0,retired:None}}}

pub(super) fn thread_id(id:&str)->Option<String>{
    (id.len()==36&&id.bytes().enumerate().all(|(i,b)|if [8,13,18,23].contains(&i){b==b'-'}else{b.is_ascii_hexdigit()})
        &&id.bytes().any(|b|b.is_ascii_hexdigit()&&b!=b'0')).then(||id.to_ascii_lowercase())
}
fn string(value:&Value,max:usize)->Option<String>{
    let text=value.as_str()?;(!text.is_empty()&&text.len()<=max&&!text.chars().any(char::is_control)).then(||text.to_owned())
}
fn path(value:&Value)->Option<String>{
    let text=string(value,4096)?;let p=std::path::Path::new(&text);
    (p.is_absolute()&&!p.components().any(|c|matches!(c,std::path::Component::ParentDir|std::path::Component::CurDir))).then_some(text)
}
fn keys(value:&Value,allowed:&[&str])->Option<()> {
    value.as_object()?.keys().all(|key|allowed.contains(&key.as_str())).then_some(())
}
fn default_bool(value:&Value,key:&str)->Option<bool>{match value.get(key){None=>Some(false),Some(v)=>v.as_bool()}}
fn approval(value:&Value)->Option<ApprovalPolicy>{
    match value.as_str(){Some("untrusted")=>return Some(ApprovalPolicy::Untrusted),Some("on-request")=>return Some(ApprovalPolicy::OnRequest),Some("never")=>return Some(ApprovalPolicy::Never),Some(_)=>return None,None=>{}}
    keys(value,&["granular"])?;let v=value.get("granular")?;
    keys(v,&["sandbox_approval","rules","mcp_elicitations","request_permissions","skill_approval"])?;
    Some(ApprovalPolicy::Granular{sandbox_approval:v.get("sandbox_approval")?.as_bool()?,rules:v.get("rules")?.as_bool()?,mcp_elicitations:v.get("mcp_elicitations")?.as_bool()?,request_permissions:default_bool(v,"request_permissions")?,skill_approval:default_bool(v,"skill_approval")?})
}
fn sandbox(value:&Value)->Option<SandboxPolicy>{
    Some(match value.get("type")?.as_str()?{
        "dangerFullAccess"=>{keys(value,&["type"])?;SandboxPolicy::DangerFullAccess},
        "readOnly"=>{keys(value,&["type","networkAccess"])?;SandboxPolicy::ReadOnly{network_access:default_bool(value,"networkAccess")?}},
        "externalSandbox"=>{keys(value,&["type","networkAccess"])?;let network=value.get("networkAccess").map_or(Some("restricted"),Value::as_str)?;if !matches!(network,"restricted"|"enabled"){return None;}SandboxPolicy::ExternalSandbox{network_access:network.into()}},
        "workspaceWrite"=>{
            keys(value,&["type","networkAccess","writableRoots","excludeTmpdirEnvVar","excludeSlashTmp"])?;
            let roots=match value.get("writableRoots"){None=>vec![],Some(v)=>{let rows=v.as_array()?;if rows.len()>32{return None;}rows.iter().map(path).collect::<Option<Vec<_>>>()?}};
            if roots.iter().map(String::len).sum::<usize>()>MAX_ROOT_BYTES{return None;}
            SandboxPolicy::WorkspaceWrite{network_access:default_bool(value,"networkAccess")?,writable_roots:roots,exclude_tmpdir_env_var:default_bool(value,"excludeTmpdirEnvVar")?,exclude_slash_tmp:default_bool(value,"excludeSlashTmp")?}
        },_=>return None,
    })
}
fn settings(value:&Value)->Option<Settings>{
    let profile=match value.get("activePermissionProfile"){None|Some(Value::Null)=>None,Some(v)=>{
        keys(v,&["id","extends"])?;
        Some(Profile{id:string(v.get("id")?,256)?,extends:match v.get("extends"){None|Some(Value::Null)=>None,Some(v)=>Some(string(v,256)?)}})
    }};
    Some(Settings{approval_policy:approval(value.get("approvalPolicy")?)?,reviewer:match value.get("approvalsReviewer")?.as_str()?{"user"=>Reviewer::User,"auto_review"=>Reviewer::AutoReview,"guardian_subagent"=>Reviewer::GuardianSubagent,_=>return None},sandbox_policy:sandbox(value.get("sandboxPolicy")?)?,profile,cwd:path(value.get("cwd")?)?})
}
impl Cache {
    fn entry(&mut self,id:&str)->Option<&mut Entry>{
        if !self.entries.contains_key(id)&&self.entries.len()>=MAX_THREADS{return None;}
        Some(self.entries.entry(id.into()).or_insert(Entry{last:None,at:None,sequence:None,reason:Some(Reason::NotObserved),closed:false}))
    }
    fn invalidate_all(&mut self,reason:Reason){for entry in self.entries.values_mut(){entry.reason=Some(reason);}}
    pub(super) fn disconnect(&mut self){if self.retired.is_none(){self.retired=Some(Reason::Disconnected);self.invalidate_all(Reason::Disconnected);}}
    /// Called in the RPC reader, before the bounded broadcast. Consumer lag
    /// cannot make this cache miss a settings event already read on this Client.
    /// Returns only a public invalidation marker; raw settings/instructions are
    /// never forwarded to consumers by this path.
    pub(super) fn event(&mut self,event:&Value,now:u64)->Option<Value>{
        if self.retired.is_some()||event.get("id").is_some(){return None;}
        let method=event.get("method")?.as_str()?;
        if method=="thread/closed"||method=="thread/started"{
            let raw=event.pointer("/params/threadId").or_else(||event.pointer("/params/thread/id"))?.as_str()?;
            let id=thread_id(raw)?;
            if let Some(entry)=self.entry(&id){if method=="thread/closed"{entry.reason=Some(Reason::ThreadClosed);entry.closed=true;}else if entry.closed{*entry=Entry{last:None,at:None,sequence:None,reason:Some(Reason::NotObserved),closed:false};}}
            return None;
        }
        if method!="thread/settings/updated"{return None;}
        let id=event.pointer("/params/threadId").and_then(Value::as_str).and_then(thread_id);
        let Some(id)=id else{self.invalidate_all(Reason::InvalidNotification);return Some(serde_json::json!({"method":"mochi/threadSettingsObserved","params":{"unknown":true}}));};
        let parsed=event.pointer("/params/threadSettings").and_then(settings).filter(|_|now>0);
        let Some(sequence)=self.sequence.checked_add(1)else{self.retired=Some(Reason::SequenceExhausted);self.invalidate_all(Reason::SequenceExhausted);return None;};self.sequence=sequence;
        if let Some(entry)=self.entry(&id){
            if !entry.closed{if let Some(parsed)=parsed{entry.last=Some(parsed);entry.at=Some(now);entry.sequence=Some(sequence);entry.reason=None;}else{entry.reason=Some(Reason::InvalidNotification);}}
        }
        Some(serde_json::json!({"method":"mochi/threadSettingsObserved","params":{"threadId":id,"sequence":sequence.to_string(),"observationOnly":true}}))
    }
    pub(super) fn snapshot(&self,raw:&str,now:u64,max_age:u64)->Observation{
        let id=thread_id(raw);let entry=id.as_ref().and_then(|id|self.entries.get(id));
        let mut reason=if id.is_none(){Some(Reason::InvalidThread)}else if let Some(reason)=self.retired{Some(reason)}else{entry.and_then(|e|e.reason).or_else(||entry.is_none().then_some(if self.entries.len()>=MAX_THREADS{Reason::Capacity}else{Reason::NotObserved}))};
        if reason.is_none()&&entry.is_some_and(|e|e.at.is_none_or(|at|at>now||now-at>max_age)){reason=Some(Reason::Expired);}
        Observation{thread_id:id.unwrap_or_default(),last_observed:entry.and_then(|e|e.last.clone()),observed_at:entry.and_then(|e|e.at),sequence:entry.and_then(|e|e.sequence),stale:reason.is_some(),reason}
    }
}

#[cfg(test)]
#[path="native_settings_tests.rs"]
mod tests;
