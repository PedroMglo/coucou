use super::*;
use std::{path::{Path,PathBuf},sync::mpsc};
struct Fixture(PathBuf);
impl Fixture {
    fn new()->Self {
        use std::os::unix::fs::DirBuilderExt;
        let path=std::env::temp_dir().join(format!("coucou-owned-helper-{}",crate::assistant_memory::new_id()));
        std::fs::DirBuilder::new().mode(0o700).create(&path).unwrap();Self(path)
    }
    fn file(&self,name:&str)->PathBuf{self.0.join(name)}
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn python(script:&str,path:&Path)->Command {
    command("/usr/bin/python3",&["-I".into(),"-c".into(),script.into(),path.to_str().unwrap().into()])
}
fn session(resources:&Arc<Coordinator>,lane:Lane,millis:u64)->Session {
    Session::acquire_on(resources.clone(),lane,Duration::from_millis(millis)).unwrap()
}
async fn until(mut ready:impl FnMut()->bool){
    let end=Instant::now()+Duration::from_secs(5);
    while !ready(){assert!(Instant::now()<end,"private worker observation deadline");tokio::time::sleep(Duration::from_millis(5)).await;}
}
fn gone(pid:i32)->bool{
    let file=std::fs::read(format!("/proc/{pid}/stat"));
    match file {
        Ok(bytes)=>{let end=bytes.iter().rposition(|b|*b==b')').unwrap();matches!(bytes.get(end+2),Some(b'Z'|b'X'|b'x'))},
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>true,
        Err(e)=>panic!("cannot observe private child: {e}"),
    }
}
fn pids(path:&Path)->Vec<i32>{std::fs::read_to_string(path).unwrap().split_whitespace().map(|v|v.parse().unwrap()).collect()}
async fn drained(resources:&Arc<Coordinator>){until(||resources.busy().unwrap().is_empty()).await;}

#[tokio::test] async fn helper_lifecycle_literal_arguments_and_completed_output_are_preserved(){
    let resources=Arc::default();let s=session(&resources,Lane::AudioEffect,2000);
    let args:Vec<String>=vec!["literal; $HOME $(never-execute)".into()];
    assert_eq!(s.run(command("/usr/bin/printf",&["%s".into(),args[0].clone()]),64,Duration::from_secs(1),||Ok(())).await.unwrap(),args[0].as_bytes());
    assert!(!s.cancel.load(Ordering::Acquire));
    // The whole operation keeps its lease across native readback, not one lease
    // per subprocess. Cancellation/expiry alone never permits a second owner.
    assert!(Session::acquire_on(resources.clone(),Lane::AudioEffect,Duration::from_secs(1)).is_err());
    drop(s);drained(&resources).await;
}

#[tokio::test] async fn helper_lifecycle_abandoned_pre_spawn_wait_keeps_hold_without_starting_later(){
    let f=Fixture::new();let marker=f.file("no-effect");let resources=Arc::default();
    let s=session(&resources,Lane::AudioEffect,5000);let(entered,seen)=mpsc::channel();let(release,wait)=mpsc::channel();
    let cmd=python("import pathlib,sys;pathlib.Path(sys.argv[1]).write_text('effect')",&marker);
    let task=tokio::spawn(async move{s.run(cmd,64,Duration::from_secs(5),move||{entered.send(()).unwrap();let _=wait.recv_timeout(Duration::from_secs(4));Ok(())}).await});
    tokio::task::spawn_blocking(move||seen.recv_timeout(Duration::from_secs(2)).unwrap()).await.unwrap();
    task.abort();assert!(task.await.unwrap_err().is_cancelled());
    assert!(Session::acquire_on(resources.clone(),Lane::AudioEffect,Duration::from_secs(1)).is_err());
    release.send(()).unwrap();drained(&resources).await;assert!(!marker.exists());
}

#[tokio::test] async fn helper_lifecycle_deadline_returns_uncertainty_while_blocked_worker_retains_hold(){
    let f=Fixture::new();let marker=f.file("expired-no-effect");let resources=Arc::default();
    let s=session(&resources,Lane::DmsEffect,2000);let(entered,seen)=mpsc::channel();let(release,wait)=mpsc::channel();
    let cmd=python("import pathlib,sys;pathlib.Path(sys.argv[1]).write_text('effect')",&marker);
    let task=tokio::spawn(async move{s.run(cmd,64,Duration::from_secs(1),move||{entered.send(()).unwrap();let _=wait.recv_timeout(Duration::from_secs(4));Ok(())}).await});
    tokio::task::spawn_blocking(move||seen.recv_timeout(Duration::from_secs(2)).unwrap()).await.unwrap();
    let result=task.await.unwrap();assert!(result.is_err());
    assert!(Session::acquire_on(resources.clone(),Lane::DmsEffect,Duration::from_secs(1)).is_err());
    release.send(()).unwrap();drained(&resources).await;assert!(!marker.exists());
}

const FAMILY:&str="import os,pathlib,signal,sys,time\np=os.fork()\nif p==0:\n signal.signal(signal.SIGTERM,signal.SIG_IGN)\n os.close(1)\n time.sleep(30)\n os._exit(0)\npathlib.Path(sys.argv[1]).write_text(str(os.getpid())+' '+str(p))\ntime.sleep(30)\n";
#[tokio::test] async fn helper_lifecycle_timeout_reaps_leader_and_stops_group_before_new_owner(){
    let f=Fixture::new();let marker=f.file("family");let resources=Arc::default();let s=session(&resources,Lane::AudioEffect,2000);
    let result=s.run(python(FAMILY,&marker),64,Duration::from_secs(1),||Ok(())).await;
    assert!(result.is_err());drop(s);drained(&resources).await;
    let ids=pids(&marker);assert_eq!(ids.len(),2);assert!(ids.iter().all(|id|gone(*id)));
    assert_eq!(unsafe{libc::kill(ids[0],0)},-1);assert_eq!(std::io::Error::last_os_error().raw_os_error(),Some(libc::ESRCH));
    let _next=session(&resources,Lane::AudioEffect,1000);
}

#[tokio::test] async fn helper_lifecycle_caller_drop_after_spawn_does_not_detach_family(){
    let f=Fixture::new();let marker=f.file("cancel-family");let resources=Arc::default();let s=session(&resources,Lane::DmsEffect,5000);
    let cmd=python(FAMILY,&marker);
    let task=tokio::spawn(async move{s.run(cmd,64,Duration::from_secs(5),||Ok(())).await});
    until(||marker.exists()&&std::fs::read_to_string(&marker).is_ok_and(|v|v.split_whitespace().count()==2)).await;
    let ids=pids(&marker);task.abort();assert!(task.await.unwrap_err().is_cancelled());
    drained(&resources).await;assert!(ids.iter().all(|id|gone(*id)));let _next=session(&resources,Lane::DmsEffect,1000);
}

#[tokio::test] async fn helper_lifecycle_successful_leader_cannot_leave_background_writer_in_reused_slot(){
    let f=Fixture::new();let marker=f.file("exit-family");let resources=Arc::default();let s=session(&resources,Lane::AudioEffect,3000);
    let script=FAMILY;
    let at=script.rfind("time.sleep(30)").unwrap();let script=format!("{}print('observed',flush=True)\nos._exit(0)\n",&script[..at]);
    let bytes=s.run(python(&script,&marker),64,Duration::from_secs(2),||Ok(())).await.unwrap();
    assert_eq!(bytes,b"observed\n");let ids=pids(&marker);assert!(ids.iter().all(|id|gone(*id)));drop(s);drained(&resources).await;
}

#[tokio::test] async fn helper_lifecycle_output_limit_stops_worker_without_materializing_unbounded_data_or_replay(){
    let f=Fixture::new();let marker=f.file("output-pid");let resources=Arc::default();let s=session(&resources,Lane::DmsRead,2000);
    let script="import os,pathlib,sys;pathlib.Path(sys.argv[1]).write_text(str(os.getpid()));sys.stdout.buffer.write(b'x'*1000000);sys.stdout.flush()";
    assert!(s.run(python(script,&marker),64,Duration::from_secs(1),||Ok(())).await.is_err());
    drop(s);drained(&resources).await;let ids=pids(&marker);assert_eq!(ids.len(),1);assert!(gone(ids[0]));
}

#[tokio::test] async fn helper_lifecycle_nonzero_exit_keeps_error_uncertain_without_exposing_private_stderr(){
    let f=Fixture::new();let marker=f.file("not-used");let resources=Arc::default();let s=session(&resources,Lane::AudioRead,2000);
    let error=s.run(python("import sys;sys.stderr.write('PRIVATE-CANARY');print('PRIVATE-CANARY');sys.exit(7)",&marker),64,Duration::from_secs(1),||Ok(())).await.unwrap_err();
    assert!(!error.contains("CANARY"));assert!(error.contains("pode ter ocorrido"));drop(s);drained(&resources).await;
}

#[tokio::test] async fn helper_lifecycle_release_and_reads_are_independent_of_busy_normal_actions(){
    let resources=Arc::default();let audio=session(&resources,Lane::AudioEffect,2000);let dms=session(&resources,Lane::DmsEffect,2000);
    let read=session(&resources,Lane::DmsRead,2000);let release=session(&resources,Lane::DmsRelease,2000);let audio_read=session(&resources,Lane::AudioRead,2000);
    for s in [&read,&release,&audio_read]{assert_eq!(s.run(command("/usr/bin/printf",&["read-or-withdraw".into()]),64,Duration::from_secs(1),||Ok(())).await.unwrap(),b"read-or-withdraw");}
    assert!(Session::acquire_on(resources.clone(),Lane::DmsEffect,Duration::from_secs(1)).is_err());drop((audio,dms,read,release,audio_read));drained(&resources).await;
}

#[tokio::test] async fn helper_lifecycle_expiry_does_not_renew_budget_for_second_helper_or_start_an_effect(){
    let f=Fixture::new();let marker=f.file("must-not-start");let resources=Arc::default();let s=session(&resources,Lane::AudioEffect,1000);
    assert_eq!(s.run(command("/usr/bin/printf",&["first".into()]),64,Duration::from_secs(1),||Ok(())).await.unwrap(),b"first");
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert!(s.run(python("import pathlib,sys;pathlib.Path(sys.argv[1]).touch()",&marker),64,Duration::from_secs(1),||Ok(())).await.is_err());
    assert!(Session::acquire_on(resources.clone(),Lane::AudioEffect,Duration::from_secs(1)).is_err());drop(s);drained(&resources).await;assert!(!marker.exists());
}

#[tokio::test] async fn helper_lifecycle_failed_spawn_is_not_misreported_as_observed_success(){
    let resources=Arc::default();let s=session(&resources,Lane::DmsEffect,2000);
    let result=s.run(command("/definitely-absent-coucou-fixture-helper",&[]),64,Duration::from_secs(1),||Ok(())).await;
    assert!(result.unwrap_err().contains("iniciar"));drop(s);drained(&resources).await;
}

#[tokio::test] async fn helper_lifecycle_owned_cleanup_does_not_signal_an_unrelated_process(){
    let mut sentinel=OwnedChild::new(command("/usr/bin/sleep",&["30".into()]).spawn().unwrap());
    let f=Fixture::new();let marker=f.file("own-family");let resources=Arc::default();let s=session(&resources,Lane::DmsEffect,2000);
    assert!(s.run(python(FAMILY,&marker),64,Duration::from_secs(1),||Ok(())).await.is_err());
    drop(s);drained(&resources).await;assert!(!sentinel.exited().unwrap());assert_eq!(unsafe{libc::kill(sentinel.child.id()as i32,0)},0);sentinel.stop_and_reap().unwrap();
}

#[tokio::test]async fn helper_lifecycle_speech_input_is_literal_bounded_and_separate_from_audio_controls(){
    let resources=Arc::default();let speech=session(&resources,Lane::SpeechOutput,3000);let audio=session(&resources,Lane::AudioEffect,3000);
    let text=b"-C\n!-!CANCEL ALL\n$(never execute)".to_vec();
    let out=speech.run_input(command("/usr/bin/cat",&[]),Some(text.clone()),128,Duration::from_secs(1),||Ok(()),Arc::new(||true)).await.unwrap();assert_eq!(out,text);
    assert!(speech.run_input(command("/usr/bin/cat",&[]),Some(vec![0;64*1024+1]),128,Duration::from_secs(1),||panic!("oversize effect"),Arc::new(||true)).await.is_err());drop((speech,audio));drained(&resources).await;
}

#[tokio::test]async fn helper_lifecycle_early_exit_does_not_claim_all_speech_input_was_delivered(){
    let resources=Arc::default();let speech=session(&resources,Lane::SpeechOutput,2000);
    // A fast exit can still buffer every byte. Close the child's read endpoint
    // in pre_exec, before spawn acknowledges exec, to prove a broken pipe.
    let mut closed=command("/usr/bin/true",&[]);
    unsafe{closed.pre_exec(||{libc::close(libc::STDIN_FILENO);Ok(())});}
    assert!(speech.run_input(closed,Some(vec![b'x';64*1024]),16,Duration::from_secs(1),||Ok(()),Arc::new(||true)).await.is_err());drop(speech);drained(&resources).await;
}

#[tokio::test]async fn helper_lifecycle_revoke_during_backpressured_speech_input_keeps_family_owned_until_reaped(){
    let f=Fixture::new();let marker=f.file("speech-family");let resources=Arc::default();let speech=session(&resources,Lane::SpeechOutput,4000);let alive=Arc::new(AtomicBool::new(true));let a=alive.clone();
    let cmd=python(FAMILY,&marker);let task=tokio::spawn(async move{speech.run_input(cmd,Some(vec![b'x';64*1024]),64,Duration::from_secs(3),||Ok(()),Arc::new(move||a.load(Ordering::Acquire))).await});
    until(||marker.exists()&&std::fs::read_to_string(&marker).is_ok_and(|v|v.split_whitespace().count()==2)).await;let ids=pids(&marker);
    assert!(Session::acquire_on(resources.clone(),Lane::SpeechOutput,Duration::from_secs(1)).is_err());alive.store(false,Ordering::Release);assert!(task.await.unwrap().is_err());drained(&resources).await;assert!(ids.iter().all(|id|gone(*id)));
}
