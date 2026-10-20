//! Local CLI process metadata. Never executes a helper/provider, reads a
//! transcript, opens a native session client or infers a running model turn.
use serde::Serialize;
use std::collections::{BTreeMap,BTreeSet};
use std::path::PathBuf;
use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};
#[cfg(target_os="linux")]
use std::os::unix::fs::MetadataExt;
pub(crate) const PROVIDERS:&[(&str,&str)]=&[("codex","codex"),("copilot","copilot"),("antigravity","agy"),("claude-cli","claude"),("gemini-cli","gemini"),("ollama","ollama")];
pub(crate) fn stamp()->u64 {SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()}
pub(crate) fn valid_uuid(id:&str)->bool {id.len()==36&&id.bytes().enumerate().all(|(i,b)|if [8,13,18,23].contains(&i){b==b'-'}else{b.is_ascii_hexdigit()})}
#[derive(Clone,Debug,Serialize,PartialEq)]
#[serde(rename_all="camelCase")]
pub(crate) struct Client {
 pub key:String,pub provider:String,pub pid:u32,pub start_time_ticks:String,
 pub channel:String,pub foreground:bool,pub paused:bool,pub cwd:Option<String>,pub session_identity:SessionIdentity,
 #[serde(skip)]pub(crate) parent:u32,#[serde(skip)]pub(crate) group:i64,#[serde(skip)]pub(crate) tty:i64,#[serde(skip)]pub(crate) wrapper:bool,
}
#[derive(Clone,Debug,Serialize,PartialEq)]
#[serde(rename_all="camelCase")]
pub(crate) struct SessionIdentity {pub state:String,pub native_session_id:Option<String>,pub sources:Vec<String>}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Counts {pub clients_observed:usize,pub terminal_clients_observed:usize,pub foreground_clients_observed:usize,pub identified_native_sessions_observed:usize,pub clients_with_unknown_session:usize,pub clients_with_conflicting_session:usize,pub distinct_native_sessions:Option<usize>}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Census {pub schema_version:u32,pub provider:String,pub scope:&'static str,pub coverage:String,pub observed_at:Option<u64>,pub stale:bool,pub refreshing:bool,pub reason:Option<String>,pub counts:Counts,pub turns:&'static str,pub clients:Vec<Client>}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct PetLiveWork {pub owner:String,pub pet_id:String,pub conversation_id:String,pub turn:String,pub phase:&'static str,pub native_provider:Option<String>,pub native_session_id:Option<String>}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct LiveWork {pub status:&'static str,pub observed_at:u64,pub reason:Option<String>,pub rows:Vec<PetLiveWork>}
impl LiveWork {pub(crate) fn unavailable(reason:&str)->Self{Self{status:"unavailable",observed_at:stamp(),reason:Some(reason.into()),rows:vec![]}}}
pub(crate) fn summarize(provider:&str,clients:Vec<Client>,coverage:&str,observed_at:Option<u64>,reason:Option<String>)->Census{
 let ids:BTreeSet<_>=clients.iter().filter_map(|c|c.session_identity.native_session_id.as_deref()).collect();
 let unknown=clients.iter().filter(|c|c.session_identity.state=="unknown").count();let conflicts=clients.iter().filter(|c|c.session_identity.state=="conflicting").count();
 let counts=Counts{clients_observed:clients.len(),terminal_clients_observed:clients.iter().filter(|c|c.channel=="terminal").count(),foreground_clients_observed:clients.iter().filter(|c|c.foreground&&c.channel=="terminal").count(),identified_native_sessions_observed:ids.len(),clients_with_unknown_session:unknown,clients_with_conflicting_session:conflicts,distinct_native_sessions:(unknown==0&&conflicts==0&&matches!(coverage,"completeWithinUserProcessScan"|"partial")).then_some(ids.len())};
 Census{schema_version:1,provider:provider.into(),scope:"localProcesses",coverage:coverage.into(),observed_at,stale:false,refreshing:false,reason,counts,turns:"notObservedByProcessCensus",clients}
}
fn deduplicate_wrappers(rows:Vec<Client>)->Vec<Client>{
 // Only a positively classified launcher is removable; two native clients are
 // never collapsed merely because one happens to be another's parent.
 let index:BTreeMap<u32,&Client>=rows.iter().map(|r|(r.pid,r)).collect();
 let remove:BTreeSet<u32>=rows.iter().filter(|r|r.wrapper).filter_map(|wrapper|{
  let matches=rows.iter().filter(|child|!child.wrapper&&child.provider==wrapper.provider&&child.tty==wrapper.tty&&child.group==wrapper.group).filter(|child|{
   let mut parent=child.parent;let mut seen=BTreeSet::new();
   for _ in 0..32 {if parent==wrapper.pid{return true;}if !seen.insert(parent){break;}let Some(p)=index.get(&parent)else{break;};if p.provider!=wrapper.provider||!p.wrapper{break;}parent=p.parent;}false
  }).count();(matches>0).then_some(wrapper.pid)
 }).collect();
 rows.into_iter().filter(|r|!remove.contains(&r.pid)).collect()
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct FileId{device:u64,inode:u64}
#[derive(Clone)]
struct Rule{provider:String,exe:FileId,entrypoint:Option<PathBuf>}
#[derive(Clone)]
struct Config{proc:PathBuf,uid:u32,rules:Vec<Rule>,session_roots:Vec<PathBuf>,max_processes:usize,max_matches:usize,max_fds:usize,budget:Duration}
#[derive(Clone,Debug,PartialEq)]
struct Stat{parent:u32,group:i64,tty:i64,foreground_group:i64,start:u64,state:char}
fn stable_incarnation(before:&Stat,after:&Stat,exe:FileId,final_exe:Option<FileId>)->bool {before.start==after.start&&before.tty==after.tty&&before.group==after.group&&final_exe==Some(exe)}
fn parse_stat(text:&str)->Option<Stat>{
 let (_,rest)=text.rsplit_once(") ")?;let f:Vec<_>=rest.split_whitespace().collect();
 Some(Stat{state:f.first()?.chars().next()?,parent:f.get(1)?.parse().ok()?,group:f.get(2)?.parse().ok()?,tty:f.get(4)?.parse().ok()?,foreground_group:f.get(5)?.parse().ok()?,start:f.get(19)?.parse().ok()?})
}
fn role(provider:&str,args:&[String])->Option<&'static str>{
 if args.is_empty(){return None;}
 if args.iter().skip(1).any(|a|matches!(a.as_str(),"--help"|"-h"|"--version")){return None;}
 // These are services/catalog tools, not open conversational clients.
 if args.get(1).is_some_and(|a|matches!(a.as_str(),"app-server"|"mcp-server"|"mcp"|"login"|"logout"|"completion"|"completions"|"features"|"agents"|"plugin")){return None;}
 if provider=="ollama" {return (args.get(1).is_some_and(|a|a=="run")).then_some("terminal");}
 if args.get(1).is_some_and(|a|matches!(a.as_str(),"exec"|"e"|"review")){Some("headless")}else{Some("terminal")}
}
#[cfg(target_os="linux")]
fn file_id(m:&std::fs::Metadata)->FileId {FileId{device:m.dev(),inode:m.ino()}}
#[cfg(target_os="linux")]
fn bounded_read(path:&std::path::Path,limit:usize)->std::io::Result<Vec<u8>>{
 use std::io::Read;let file=std::fs::File::open(path)?;let mut bytes=Vec::new();file.take(limit as u64+1).read_to_end(&mut bytes)?;if bytes.len()>limit{return Err(std::io::ErrorKind::InvalidData.into());}Ok(bytes)
}
#[cfg(target_os="linux")]
fn trusted_file(path:&std::path::Path,uid:u32)->Option<(PathBuf,FileId,bool)>{
 use std::io::Read;let p=path.canonicalize().ok()?;let m=std::fs::metadata(&p).ok()?;
 if !m.is_file()||!matches!(m.uid(),0) && m.uid()!=uid||m.mode()&0o022!=0{return None;}
 let mut header=[0u8;256];let n=std::fs::File::open(&p).ok()?.read(&mut header).ok()?;
 let native=header[..n].starts_with(b"\x7fELF");Some((p,file_id(&m),native))
}
#[cfg(target_os="linux")]
fn discover(home:&std::path::Path,uid:u32)->Config{
 let paths=std::env::var_os("PATH").map(|v|std::env::split_paths(&v).filter(|p|p.is_absolute()).collect::<Vec<_>>()).unwrap_or_default();
 let mut runtimes=Vec::new();for name in ["node","bun","python3"]{for dir in paths.iter().chain([PathBuf::from("/usr/bin"),PathBuf::from("/usr/local/bin")].iter()){if let Some((_,id,true))=trusted_file(&dir.join(name),uid){if !runtimes.contains(&id){runtimes.push(id);}}}}
 let mut rules=Vec::new();
 for (provider,program) in PROVIDERS{
  let mut candidates:Vec<_>=paths.iter().map(|p|p.join(program)).collect();candidates.extend([home.join(".local/bin").join(program),PathBuf::from("/usr/bin").join(program),PathBuf::from("/usr/local/bin").join(program)]);
  if *provider=="codex"{
   // Installed standalone Codex binaries are ELF children of this explicit
   // package root, not arbitrary argv tokens containing the word codex.
   if let Ok(entries)=std::fs::read_dir(home.join(".codex/packages/standalone/releases")){for entry in entries.flatten().take(64){candidates.push(entry.path().join("bin/codex"));}}
  }
  for p in candidates{if let Some((path,id,native))=trusted_file(&p,uid){if native{if path.file_name().and_then(|n|n.to_str())==Some(*program){rules.push(Rule{provider:(*provider).into(),exe:id,entrypoint:None});}}else{for runtime in &runtimes{rules.push(Rule{provider:(*provider).into(),exe:*runtime,entrypoint:Some(path.clone())});}}}}
 }
 let session_home=std::env::var_os("CODEX_HOME").map(PathBuf::from).unwrap_or_else(||home.join(".codex"));
 let session_roots=if session_home.is_absolute(){["sessions","archived_sessions"].into_iter().filter_map(|n|session_home.join(n).canonicalize().ok()).collect()}else{vec![]};
 Config{proc:PathBuf::from("/proc"),uid,rules,session_roots,max_processes:32768,max_matches:512,max_fds:128,budget:Duration::from_secs(2)}
}
#[cfg(target_os="linux")]
fn provider_rule(config:&Config,exe:FileId,args:&[String])->Option<(String,bool,Vec<String>)>{
 for rule in &config.rules{
  if rule.exe!=exe {continue;}
  if let Some(script)=&rule.entrypoint{
   // Exact first interpreter entrypoint only; never scan argv[0..3] for a name.
   if args.get(1).and_then(|p|std::fs::canonicalize(p).ok()).as_ref()!=Some(script){continue;}
   let mut native_args=vec![script.to_string_lossy().into_owned()];native_args.extend(args.iter().skip(2).cloned());return Some((rule.provider.clone(),true,native_args));
  }
  return Some((rule.provider.clone(),false,args.to_vec()));
 }None
}
#[cfg(target_os="linux")]
fn identities(config:&Config,dir:&std::path::Path,deadline:Instant)->SessionIdentity{
 use std::io::Read;use std::os::unix::fs::OpenOptionsExt;
 let mut ids=BTreeSet::new();let mut sources=BTreeSet::new();
 if let Ok(entries)=std::fs::read_dir(dir.join("fd")){
  for entry in entries.flatten().take(config.max_fds){
   if Instant::now()>=deadline{break;}
   let Ok(target)=std::fs::read_link(entry.path())else{continue;};
   if !config.session_roots.iter().any(|root|target.starts_with(root)){continue;}
   let Some(name)=target.file_name().and_then(|n|n.to_str()).and_then(|n|n.strip_suffix(".jsonl"))else{continue;};
   let Some(id)=name.get(name.len().saturating_sub(36)..).filter(|id|valid_uuid(id))else{continue;};
   let Ok(fd_meta)=std::fs::metadata(entry.path())else{continue;};
   let Ok(file)=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK).open(&target)else{continue;};
   let Ok(meta)=file.metadata()else{continue;};if !meta.is_file()||meta.uid()!=config.uid||file_id(&meta)!=file_id(&fd_meta){continue;}
   // Read the metadata header only; never scan subsequent chat events.
   use std::io::BufRead;
   let mut bytes=Vec::new();let mut reader=std::io::BufReader::new(file.take(65537));
   if reader.read_until(b'\n',&mut bytes).is_err()||bytes.len()>65536{continue;}
   #[derive(serde::Deserialize)]struct Header {r#type:String,payload:HeaderId}
   #[derive(serde::Deserialize)]struct HeaderId {id:String}
   if let Ok(header)=serde_json::from_slice::<Header>(&bytes){if header.r#type=="session_meta"&&header.payload.id==id{ids.insert(id.to_string());sources.insert("rolloutDescriptorHeader".to_string());}}
  }
 }
 // Environment/argv hints alone do not prove a current native session. They
 // are not read or exported here; terminal pane/cwd never becomes an identity.
 match ids.len(){0=>SessionIdentity{state:"unknown".into(),native_session_id:None,sources:vec![]},1=>SessionIdentity{state:"identified".into(),native_session_id:ids.into_iter().next(),sources:sources.into_iter().collect()},_=>SessionIdentity{state:"conflicting".into(),native_session_id:None,sources:sources.into_iter().collect()}}
}
#[cfg(target_os="linux")]
fn scan(config:&Config)->BTreeMap<String,Census>{
 let started=Instant::now();let deadline=started+config.budget;let observed=stamp();let mut rows=Vec::new();let mut partial=None;
 let supported:BTreeSet<_>=config.rules.iter().map(|r|r.provider.clone()).collect();
 let entries=match std::fs::read_dir(&config.proc){Ok(entries)=>entries,Err(_)=>return PROVIDERS.iter().map(|(p,_)|((*p).into(),summarize(p,vec![],"unavailable",None,Some("processScanUnavailable".into())))).collect()};
 let mut seen=0;
 for entry in entries{
  if Instant::now()>=deadline{partial=Some("processScanTimeout");break;}
  let Ok(entry)=entry else{partial=Some("processScanPartial");continue;};let Ok(pid)=entry.file_name().to_string_lossy().parse::<u32>()else{continue;};
  seen+=1;if seen>config.max_processes{partial=Some("processScanLimit");break;}
  let dir=entry.path();let Ok(meta)=std::fs::metadata(&dir)else{continue;};if meta.uid()!=config.uid{continue;}
  let read=||->Result<Option<Client>,()>{
   let before=parse_stat(std::str::from_utf8(&bounded_read(&dir.join("stat"),8192).map_err(|_|())?).map_err(|_|())?).ok_or(())?;
   if matches!(before.state,'Z'|'X'){return Ok(None);}
   let exe_meta=std::fs::metadata(dir.join("exe")).map_err(|_|())?;
   if !config.rules.iter().any(|r|r.exe==file_id(&exe_meta)){return Ok(None);}
   let args:Vec<_>=bounded_read(&dir.join("cmdline"),65536).map_err(|_|())?.split(|b|*b==0).filter(|b|!b.is_empty()).map(|b|String::from_utf8_lossy(b).into_owned()).collect();
   let Some((provider,wrapper,args))=provider_rule(config,file_id(&exe_meta),&args)else{return Ok(None);};let Some(requested_role)=role(&provider,&args)else{return Ok(None);};
   let terminal=before.tty!=0&&std::fs::read_link(dir.join("fd/0")).ok().is_some_and(|p|p.starts_with("/dev/pts")||p.starts_with("/dev/tty"));
   let cwd=std::fs::read_link(dir.join("cwd")).ok().and_then(|p|p.to_str().filter(|v|v.len()<=4096&&!v.chars().any(char::is_control)).map(str::to_string));
   let identity=if provider=="codex"{identities(config,&dir,deadline)}else{SessionIdentity{state:"unknown".into(),native_session_id:None,sources:vec![]}};
   let after=parse_stat(std::str::from_utf8(&bounded_read(&dir.join("stat"),8192).map_err(|_|())?).map_err(|_|())?).ok_or(())?;
   if !stable_incarnation(&before,&after,file_id(&exe_meta),std::fs::metadata(dir.join("exe")).ok().map(|m|file_id(&m))){return Err(());}
   if matches!(after.state,'Z'|'X'){return Ok(None);}
   Ok(Some(Client{key:format!("process:{pid}:{}",before.start),provider,pid,start_time_ticks:before.start.to_string(),channel:if terminal&&requested_role=="terminal"{"terminal"}else{"headless"}.into(),foreground:terminal&&after.group==after.foreground_group,paused:matches!(after.state,'T'|'t'),cwd,session_identity:identity,parent:after.parent,group:after.group,tty:after.tty,wrapper}))
  };
  match read(){Ok(Some(row))=>{rows.push(row);if rows.len()>=config.max_matches{partial=Some("processMatchLimit");break;}},Ok(None)=>{},Err(())=>{if dir.exists(){partial=Some("processScanPartial");}}}
 }
 let rows=deduplicate_wrappers(rows);
 PROVIDERS.iter().map(|(provider,_)|{
  let matched=rows.iter().filter(|r|r.provider==*provider).cloned().collect();let coverage=if !supported.contains(*provider){"notSupported"}else if partial.is_some(){"partial"}else{"completeWithinUserProcessScan"};
  ((*provider).into(),summarize(provider,matched,coverage,Some(observed),if !supported.contains(*provider){Some("noTrustedExecutableRule".into())}else{partial.map(str::to_string)}))
 }).collect()
}

#[cfg(target_os="linux")]
#[derive(Default)]
pub(crate) struct Cache {pub rows:BTreeMap<String,Census>,pub inflight:bool,pub last:Option<Instant>}
#[cfg(target_os="linux")]
impl Cache {
 pub(crate) fn finish(&mut self,result:BTreeMap<String,Census>){
  for (provider,new) in result {
   if (new.coverage=="unavailable"||new.coverage=="partial"&&new.clients.is_empty())&&self.rows.get(&provider).is_some_and(|v|!v.clients.is_empty()) {
    let previous=self.rows.get_mut(&provider).unwrap();previous.stale=true;previous.refreshing=false;previous.reason=new.reason.or(Some("processRefreshFailed".into()));
   }else{self.rows.insert(provider,new);}
  }self.inflight=false;
 }
}

#[cfg(test)]
#[path="process_census_tests.rs"]
mod tests;

#[cfg(target_os="linux")]
pub(crate) fn collect_now(home:&std::path::Path)->BTreeMap<String,Census>{scan(&discover(home,unsafe{libc::geteuid()}))}
#[cfg(not(target_os="linux"))]
pub(crate) fn collect_now(_: &std::path::Path)->BTreeMap<String,Census>{PROVIDERS.iter().map(|(p,_)|((*p).into(),summarize(p,vec![],"notSupported",None,Some("unsupportedPlatform".into())))).collect()}
