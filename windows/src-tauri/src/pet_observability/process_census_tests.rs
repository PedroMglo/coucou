use super::*;
use std::{fs,os::unix::fs::{symlink,PermissionsExt},sync::atomic::{AtomicU64,Ordering}};
fn uuid(n:usize)->String{format!("00000000-0000-0000-0000-{n:012}")}
fn client(pid:u32,parent:u32,wrapper:bool)->Client{Client{key:format!("process:{pid}:123"),provider:"codex".into(),pid,start_time_ticks:"123".into(),channel:"terminal".into(),foreground:true,paused:false,cwd:Some("/synthetic".into()),session_identity:SessionIdentity{state:"unknown".into(),native_session_id:None,sources:vec![]},parent,group:9,tty:34816,wrapper}}
fn stat(parent:u32,group:i64,tty:i64,start:u64,state:char)->String{let mut fields=vec!["0".to_string();20];fields[0]=state.to_string();fields[1]=parent.to_string();fields[2]=group.to_string();fields[4]=tty.to_string();fields[5]=group.to_string();fields[19]=start.to_string();format!("100 (name with ) spaces) {}",fields.join(" "))}
struct Fixture{root:PathBuf,exe:PathBuf,proc:PathBuf}
impl Fixture{
 fn new()->Self{static NEXT:AtomicU64=AtomicU64::new(0);let root=std::env::temp_dir().join(format!("coucou-census-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));fs::create_dir(&root).unwrap();fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();let exe=root.join("codex");fs::write(&exe,b"synthetic executable identity, never executed").unwrap();let proc=root.join("proc");fs::create_dir(&proc).unwrap();Self{root,exe,proc}}
 fn config(&self)->Config{Config{proc:self.proc.clone(),uid:unsafe{libc::geteuid()},rules:vec![Rule{provider:"codex".into(),exe:file_id(&fs::metadata(&self.exe).unwrap()),entrypoint:None}],session_roots:vec![self.root.join("sessions")],max_processes:100,max_matches:50,max_fds:32,budget:Duration::from_secs(2)}}
 fn process(&self,pid:u32,parent:u32,args:&[&str]){let p=self.proc.join(pid.to_string());fs::create_dir(&p).unwrap();fs::create_dir(p.join("fd")).unwrap();fs::write(p.join("stat"),stat(parent,9,34816,pid as u64*10,'S')).unwrap();fs::write(p.join("cmdline"),args.join("\0")+"\0").unwrap();symlink(&self.exe,p.join("exe")).unwrap();symlink("/dev/pts/99",p.join("fd/0")).unwrap();symlink("/synthetic/project",p.join("cwd")).unwrap();}
 fn rollout(&self,pid:u32,n:usize,payload_id:&str,second:&str){let root=self.root.join("sessions");fs::create_dir_all(&root).unwrap();let file=root.join(format!("rollout-2026-10-20-{}.jsonl",uuid(n)));fs::write(&file,format!("{}\n{second}",serde_json::json!({"type":"session_meta","payload":{"id":payload_id,"private":"NOT_EXPORTED"}}))).unwrap();symlink(file,self.proc.join(pid.to_string()).join("fd").join((n+1).to_string())).unwrap();}
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
#[test]fn process_stat_uses_starttime_and_tty_even_with_spaces_and_parentheses(){let s=parse_stat(&stat(5,7,34816,999,'S')).unwrap();assert_eq!(s.parent,5);assert_eq!(s.group,7);assert_eq!(s.tty,34816);assert_eq!(s.start,999);assert!(parse_stat("short").is_none());}
#[test]fn scheduler_state_changes_are_not_pid_reuse(){let before=parse_stat(&stat(5,7,34816,999,'S')).unwrap();let after=parse_stat(&stat(5,7,34816,999,'R')).unwrap();let exe=FileId{device:1,inode:2};assert!(stable_incarnation(&before,&after,exe,Some(exe)));}
#[test]fn pid_reuse_and_changed_executable_fail_identity(){let a=parse_stat(&stat(5,7,34816,999,'S')).unwrap();let b=parse_stat(&stat(5,7,34816,1000,'S')).unwrap();let exe=FileId{device:1,inode:2};assert!(!stable_incarnation(&a,&b,exe,Some(exe)));assert!(!stable_incarnation(&a,&a,exe,Some(FileId{device:1,inode:3})));}
#[test]fn known_native_child_replaces_only_a_proven_launcher(){let rows=deduplicate_wrappers(vec![client(1,0,true),client(2,1,false)]);assert_eq!(rows.len(),1);assert_eq!(rows[0].pid,2);}
#[test]fn two_native_clients_are_never_merged_by_generic_parent_heuristics(){assert_eq!(deduplicate_wrappers(vec![client(1,0,false),client(2,1,false)]).len(),2);}
#[test]fn another_provider_or_tty_cannot_remove_a_launcher(){let launcher=client(1,0,true);let mut child=client(2,1,false);child.provider="copilot".into();assert_eq!(deduplicate_wrappers(vec![launcher.clone(),child]).len(),2);let mut child=client(2,1,false);child.tty=34817;assert_eq!(deduplicate_wrappers(vec![launcher,child]).len(),2);}
#[test]fn several_native_children_are_preserved_without_counting_the_launcher(){assert_eq!(deduplicate_wrappers(vec![client(1,0,true),client(2,1,false),client(3,1,false)]).len(),2);}
#[test]fn clients_without_uuid_remain_useful_positive_counts_for_arbitrary_totals(){for n in [0,1,4,9]{let rows=(0..n).map(|i|client(i as u32+1,0,false)).collect();let s=summarize("codex",rows,"completeWithinUserProcessScan",Some(10),None);assert_eq!(s.counts.terminal_clients_observed,n);assert_eq!(s.counts.clients_observed,n);assert_eq!(s.counts.distinct_native_sessions,if n==0{Some(0)}else{None});assert_eq!(s.turns,"notObservedByProcessCensus");}}
#[test]fn two_clients_of_the_same_verified_native_session_are_one_identity(){let mut a=client(1,0,false);a.session_identity=SessionIdentity{state:"identified".into(),native_session_id:Some(uuid(1)),sources:vec!["rolloutDescriptorHeader".into()]};let mut b=a.clone();b.pid=2;let s=summarize("codex",vec![a,b],"completeWithinUserProcessScan",Some(10),None);assert_eq!(s.counts.clients_observed,2);assert_eq!(s.counts.distinct_native_sessions,Some(1));}
#[test]fn conflict_is_unknown_for_distinct_sessions_not_arbitrarily_chosen(){let mut a=client(1,0,false);a.session_identity.state="conflicting".into();let s=summarize("codex",vec![a],"partial",Some(10),None);assert_eq!(s.counts.clients_with_conflicting_session,1);assert_eq!(s.counts.distinct_native_sessions,None);}
#[test]fn unsupported_source_cannot_claim_exact_zero_sessions(){let s=summarize("codex",vec![],"notSupported",None,None);assert_eq!(s.counts.distinct_native_sessions,None);}
#[test]fn actual_collector_counts_synthetic_foreground_native_clients_without_uuid(){let f=Fixture::new();for n in 1..=4{f.process(n,0,&["codex"]);}let s=scan(&f.config()).remove("codex").unwrap();assert_eq!(s.coverage,"completeWithinUserProcessScan");assert_eq!(s.counts.clients_observed,4);assert_eq!(s.counts.foreground_clients_observed,4);assert_eq!(s.counts.clients_with_unknown_session,4);assert_eq!(s.counts.distinct_native_sessions,None);}
#[test]fn helper_false_positive_from_an_unrelated_executable_is_rejected(){let f=Fixture::new();f.process(1,0,&["python","script.py","codex"]);let mut cfg=f.config();cfg.rules[0].exe.inode+=1;assert_eq!(scan(&cfg)["codex"].counts.clients_observed,0);}
#[test]fn interpreter_rule_requires_the_exact_script_entrypoint(){let f=Fixture::new();let script=f.root.join("copilot.js");fs::write(&script,b"never run").unwrap();let mut cfg=f.config();cfg.rules[0].entrypoint=Some(script.canonicalize().unwrap());cfg.rules[0].provider="copilot".into();let id=cfg.rules[0].exe;assert!(provider_rule(&cfg,id,&["node".into(),"other.js".into(),script.display().to_string()]).is_none());assert!(provider_rule(&cfg,id,&["node".into(),script.display().to_string()]).is_some());}
#[test]fn app_server_help_and_ollama_service_are_not_terminal_conversations(){for args in [vec!["codex","app-server"],vec!["codex","--help"],vec!["codex","mcp"]]{assert!(role("codex",&args.into_iter().map(str::to_string).collect::<Vec<_>>()).is_none());}assert!(role("ollama",&["ollama".into(),"serve".into()]).is_none());assert_eq!(role("ollama",&["ollama".into(),"run".into(),"fixed-model".into()]),Some("terminal"));}
#[test]fn server_mode_is_excluded_and_exec_is_a_headless_client(){let f=Fixture::new();f.process(1,0,&["codex","app-server"]);f.process(2,0,&["codex","exec","synthetic"]);let s=scan(&f.config()).remove("codex").unwrap();assert_eq!(s.counts.clients_observed,1);assert_eq!(s.counts.terminal_clients_observed,0);}
#[test]fn dead_zombie_is_not_a_live_client(){let f=Fixture::new();f.process(1,0,&["codex"]);fs::write(f.proc.join("1/stat"),stat(0,9,34816,10,'Z')).unwrap();assert_eq!(scan(&f.config())["codex"].counts.clients_observed,0);}
#[test]fn another_uid_is_not_counted(){let f=Fixture::new();f.process(1,0,&["codex"]);let mut cfg=f.config();cfg.uid=cfg.uid.saturating_add(1);assert_eq!(scan(&cfg)["codex"].counts.clients_observed,0);}
#[test]fn descriptor_header_binds_metadata_only_and_never_scans_chat_events(){let f=Fixture::new();f.process(1,0,&["codex"]);f.rollout(1,1,&uuid(1),"THIS IS NOT JSON; DO NOT READ TRANSCRIPTS");let s=scan(&f.config()).remove("codex").unwrap();assert_eq!(s.clients[0].session_identity.native_session_id.as_deref(),Some(uuid(1).as_str()));assert_eq!(s.counts.distinct_native_sessions,Some(1));let public=serde_json::to_string(&s).unwrap();assert!(!public.contains("NOT_EXPORTED"));assert!(!public.contains("TRANSCRIPTS"));assert!(!public.contains("cmdline"));assert!(!public.contains("environ"));}
#[test]fn filename_and_header_must_agree(){let f=Fixture::new();f.process(1,0,&["codex"]);f.rollout(1,1,&uuid(2),"");let s=scan(&f.config()).remove("codex").unwrap();assert_eq!(s.clients[0].session_identity.state,"unknown");}
#[test]fn several_held_rollout_identities_do_not_choose_the_latest(){let f=Fixture::new();f.process(1,0,&["codex"]);f.rollout(1,1,&uuid(1),"");f.rollout(1,2,&uuid(2),"");let s=scan(&f.config()).remove("codex").unwrap();assert_eq!(s.clients[0].session_identity.state,"conflicting");assert_eq!(s.counts.distinct_native_sessions,None);}
#[test]fn rollout_outside_the_backend_root_does_not_supply_identity(){let f=Fixture::new();f.process(1,0,&["codex"]);f.rollout(1,1,&uuid(1),"");let mut cfg=f.config();cfg.session_roots=vec![f.root.join("different-root")];assert_eq!(scan(&cfg)["codex"].clients[0].session_identity.state,"unknown");}
#[test]fn bounded_scan_keeps_positive_rows_but_marks_partial(){let f=Fixture::new();for n in 1..=3{f.process(n,0,&["codex"]);}let mut cfg=f.config();cfg.max_matches=1;let s=scan(&cfg).remove("codex").unwrap();assert_eq!(s.counts.clients_observed,1);assert_eq!(s.coverage,"partial");assert_eq!(s.reason.as_deref(),Some("processMatchLimit"));}
#[test]fn missing_proc_is_unavailable_instead_of_complete_empty(){let f=Fixture::new();let mut cfg=f.config();cfg.proc=f.root.join("absent");assert_eq!(scan(&cfg)["codex"].coverage,"unavailable");}
#[test]fn zero_budget_is_partial_not_complete_empty(){let f=Fixture::new();f.process(1,0,&["codex"]);let mut cfg=f.config();cfg.budget=Duration::ZERO;let s=scan(&cfg).remove("codex").unwrap();assert_eq!(s.coverage,"partial");assert_eq!(s.reason.as_deref(),Some("processScanTimeout"));}
#[test]fn failed_refresh_preserves_last_positive_reading_and_its_timestamp(){let mut cache=Cache::default();cache.rows.insert("codex".into(),summarize("codex",vec![client(1,0,false)],"completeWithinUserProcessScan",Some(10),None));cache.finish(BTreeMap::from([("codex".into(),summarize("codex",vec![],"unavailable",None,Some("processScanUnavailable".into())))]));let s=&cache.rows["codex"];assert_eq!(s.counts.clients_observed,1);assert_eq!(s.observed_at,Some(10));assert!(s.stale);assert!(!cache.inflight);}
#[test]fn complete_empty_refresh_can_replace_an_old_positive_count(){let mut cache=Cache::default();cache.rows.insert("codex".into(),summarize("codex",vec![client(1,0,false)],"completeWithinUserProcessScan",Some(10),None));cache.finish(BTreeMap::from([("codex".into(),summarize("codex",vec![],"completeWithinUserProcessScan",Some(20),None))]));assert_eq!(cache.rows["codex"].counts.clients_observed,0);assert_eq!(cache.rows["codex"].observed_at,Some(20));}

#[test]fn several_clients_on_one_tty_are_one_terminal_and_paused_is_not_foreground_work(){
 let a=client(1,0,false);let mut b=client(2,0,false);b.paused=true;let mut c=client(3,0,false);c.tty=34820;
 let counts=summarize("codex",vec![a,b,c],"completeWithinUserProcessScan",Some(1),None).counts;
 assert_eq!(counts.terminal_clients_observed,3);assert_eq!(counts.terminal_instances_observed,Some(2));assert_eq!(counts.paused_clients_observed,1);assert_eq!(counts.foreground_clients_observed,2);
}
#[test]fn missing_tty_identity_never_fabricates_a_distinct_terminal_count(){
 let mut c=client(1,0,false);c.tty=0;let counts=summarize("codex",vec![c],"partial",Some(1),None).counts;assert_eq!(counts.terminal_clients_observed,1);assert!(counts.terminal_instances_observed.is_none());
}

#[test]fn mixed_foreground_and_suspended_clients_share_a_foreground_terminal(){
 let a=client(1,0,false);let mut b=client(2,0,false);b.paused=true;let mut c=client(3,0,false);c.tty=34820;
 let counts=summarize("codex",vec![a,b,c],"completeWithinUserProcessScan",Some(1),None).counts;
 let terminals=counts.terminal_states.unwrap();assert_eq!((terminals.foreground,terminals.background_only,terminals.suspended_only),(2,0,0));
 assert_eq!((counts.process_states.foreground,counts.process_states.background,counts.process_states.suspended,counts.process_states.headless),(2,0,1,0));
}
#[test]fn background_and_suspended_only_terminals_are_distinct_from_headless_clients(){
 let mut a=client(1,0,false);a.foreground=false;let mut b=client(2,0,false);b.paused=true;b.tty=34820;
 let mut c=client(3,0,false);c.channel="headless".into();c.tty=0;
 let mut d=client(4,0,false);d.channel="headless".into();d.tty=0;d.paused=true;
 let counts=summarize("codex",vec![a,b,c,d],"partial",Some(1),None).counts;
 let terminals=counts.terminal_states.unwrap();assert_eq!((terminals.foreground,terminals.background_only,terminals.suspended_only),(0,1,1));
 assert_eq!((counts.process_states.foreground,counts.process_states.background,counts.process_states.suspended,counts.process_states.headless),(0,1,2,1));
}
#[test]fn missing_terminal_identity_keeps_process_states_but_not_terminal_states(){
 let mut c=client(1,0,false);c.tty=0;let census=summarize("codex",vec![c],"partial",Some(1),None);
 assert!(census.counts.terminal_states.is_none());assert_eq!(census.counts.process_states.foreground,1);assert_eq!(census.turns,"notObservedByProcessCensus");
}
#[test]fn lifecycle_projection_is_shared_across_all_declared_process_providers(){
 for(provider,_)in PROVIDERS{let mut c=client(1,0,false);c.provider=(*provider).into();c.paused=true;
  let census=summarize(provider,vec![c],"partial",Some(1),None);
  assert_eq!(census.counts.process_states.suspended,1);assert_eq!(census.counts.terminal_states.unwrap().suspended_only,1);assert_eq!(census.turns,"notObservedByProcessCensus");
 }
}
#[test]fn failed_refresh_preserves_previous_lifecycle_without_claiming_freshness(){
 let mut c=client(1,0,false);c.paused=true;let mut cache=Cache::default();cache.rows.insert("codex".into(),summarize("codex",vec![c],"partial",Some(10),None));
 cache.finish(BTreeMap::from([("codex".into(),summarize("codex",vec![],"unavailable",None,Some("processScanUnavailable".into())))]));
 let old=&cache.rows["codex"];assert!(old.stale);assert_eq!(old.observed_at,Some(10));assert_eq!(old.counts.process_states.suspended,1);assert_eq!(old.counts.terminal_states.as_ref().unwrap().suspended_only,1);
}
