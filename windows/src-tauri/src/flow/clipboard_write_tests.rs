use super::*;
use std::cell::{Cell,RefCell};
fn req(text:&str,sequence:u64)->Header{Header{version:1,phase:Phase::Prepare,sequence,request_id:format!("{sequence:032x}"),route_hash:"a".repeat(64),deadline_ms:4000,bytes:text.len(),sha256:sha(text.as_bytes())}}
fn raw(h:&Header,body:&[u8])->Vec<u8>{let mut b=serde_json::to_vec(h).unwrap();b.push(b'\n');b.extend_from_slice(body);b}
fn cycle(h:&Header,text:&str,phase:Phase)->Vec<u8>{let mut b=frame(h,text).unwrap();b.extend(control(h,phase).unwrap().1);b}
fn run(input:&[u8],output:&mut Vec<u8>,now:impl Fn()->Option<u64>,put:impl FnMut(&str,&dyn Fn()->bool)->Status)->Result<(),()>{serve(input,output,&"a".repeat(64),now,|_|Status::Prepared,put)}
fn reply(h:&Header,status:Status)->Vec<u8>{let mut b=Vec::new();respond(&mut b,h,status).unwrap();b}
fn final_line(bytes:&[u8])->&[u8]{bytes.split_inclusive(|b|*b==b'\n').last().unwrap()}
#[test]fn clipboard_write_wire_preserves_literal_utf8_and_binds_each_phase_ack(){
    let text="ação\n$(literal)\0";let a=req(text,1);let b=req("second",2);let mut input=cycle(&a,text,Phase::Commit);input.extend(cycle(&b,"second",Phase::Commit));let writes=RefCell::new(Vec::new());let mut output=Vec::new();
    run(&input,&mut output,||Some(1000),|text,current|{assert!(current());writes.borrow_mut().push(text.to_owned());Status::Accepted}).unwrap();
    assert_eq!(*writes.borrow(),[text,"second"]);let lines:Vec<_>=output.split_inclusive(|b|*b==b'\n').collect();assert_eq!(lines.len(),4);
    assert!(matches!(ack(lines[0],&a),Ok(Status::Prepared)));assert!(matches!(ack(lines[1],&control(&a,Phase::Commit).unwrap().0),Ok(Status::Accepted)));assert!(ack(lines[1],&a).is_err());assert!(ack(lines[0],&b).is_err());
}
#[test]fn clipboard_write_invalid_headers_cannot_initialize_or_write(){
    for index in 0..9{let mut h=req("x",1);match index{0=>h.version=2,1=>h.sequence=2,2=>h.sequence=0,3=>h.request_id="G".repeat(32),4=>h.route_hash="b".repeat(64),5=>h.bytes=MAX_TEXT+1,6=>h.sha256="x".repeat(64),7=>h.sequence=u64::MAX,_=>h.phase=Phase::Commit};
        assert!(serve(raw(&h,b"x").as_slice(),Vec::new(),&"a".repeat(64),||Some(1000),|_|panic!("invalid init"),|_,_|panic!("invalid write")).is_err());}
}
#[test]fn clipboard_write_body_hash_truncation_utf8_and_header_limits_fail_closed(){
    let mut h=req("x",1);
    for bytes in [raw(&h,b""),raw(&h,b"y"),vec![b'x';MAX_HEADER+1]]{assert!(run(&bytes,&mut Vec::new(),||Some(1000),|_,_|panic!("malformed effect")).is_err());}
    h.sha256=sha(&[255]);assert!(run(&raw(&h,&[255]),&mut Vec::new(),||Some(1000),|_,_|panic!("invalid UTF8 effect")).is_err());
}
#[test]fn clipboard_write_unknown_keys_and_duplicate_sequence_do_not_gain_authority(){
    let h=req("x",1);let mut object=serde_json::to_value(&h).unwrap();object["grant"]=serde_json::json!(true);let mut bytes=serde_json::to_vec(&object).unwrap();bytes.extend(b"\nx");assert!(run(&bytes,&mut Vec::new(),||Some(1000),|_,_|panic!("unknown key effect")).is_err());
    let writes=Cell::new(0);let mut repeated=cycle(&h,"x",Phase::Commit);repeated.extend(cycle(&h,"x",Phase::Commit));assert!(run(&repeated,&mut Vec::new(),||Some(1000),|_,_|{writes.set(writes.get()+1);Status::Accepted}).is_err());assert_eq!(writes.get(),1);
}
#[test]fn clipboard_write_expired_or_overlong_deadline_has_no_init_or_effect(){
    let h=req("x",1);
    for now in [Some(4000),Some(4001),Some(999),None]{let mut out=Vec::new();serve(frame(&h,"x").unwrap().as_slice(),&mut out,&h.route_hash,||now,|_|panic!("expired init"),|_,_|panic!("expired effect")).unwrap();assert!(matches!(ack(&out,&h),Ok(Status::Retired)));}
}
#[test]fn clipboard_write_init_and_effect_are_fenced_without_false_success(){
    let valid=Cell::new(true);let mut slot=None;
    let status=prepare_with(&mut slot,&||valid.get(),||{valid.set(false);Ok(())});assert!(matches!(status,Status::Retired));
    valid.set(true);let status=put_with(&mut slot,"x",&||valid.get(),||panic!("second init"),|_,_|{valid.set(false);Ok(())});assert!(matches!(status,Status::Uncertain));
    let status=put_with(&mut slot,"x",&||false,||panic!("retired init"),|_,_|panic!("retired write"));assert!(matches!(status,Status::Retired));
}
#[test]fn clipboard_write_ack_rejects_changed_operation_target_phase_and_protocol(){
    let prepare=req("x",1);let h=control(&prepare,Phase::Commit).unwrap().0;let out=reply(&h,Status::Accepted);
    for key in ["version","sequence","requestId","routeHash","bytes","sha256","phase"]{let mut value:serde_json::Value=serde_json::from_slice(&out).unwrap();value[key]=match key{"version"|"sequence"|"bytes"=>serde_json::json!(77),"phase"=>serde_json::json!("prepare"),_=>serde_json::json!("changed")};let mut bytes=serde_json::to_vec(&value).unwrap();bytes.push(b'\n');assert!(ack(&bytes,&h).is_err());}
    let mut extra=out.clone();extra.extend(b"extra\n");assert!(ack(&extra,&h).is_err());assert!(ack(&[],&h).is_err());
}
#[test]fn clipboard_write_maximum_body_stays_within_driver_input_bound(){
    let text="x".repeat(MAX_TEXT);let h=req(&text,1);let input=frame(&h,&text).unwrap();assert!(input.len()<=MAX_TEXT+MAX_HEADER+1);let mut out=Vec::new();
    run(&cycle(&h,&text,Phase::Commit),&mut out,||Some(1000),|body,_|{assert_eq!(body.len(),MAX_TEXT);Status::Accepted}).unwrap();assert!(matches!(ack(final_line(&out),&control(&h,Phase::Commit).unwrap().0),Ok(Status::Accepted)));
    assert!(matches!(write(&text,&||false),Mutation::Skipped));assert!(matches!(write(&"x".repeat(MAX_TEXT+1),&||panic!("oversize current")),Mutation::Skipped));
}
#[test]fn clipboard_write_prepare_alone_and_changed_commit_cannot_write(){
    let h=req("x",1);let prepared=Cell::new(0);
    assert!(serve(frame(&h,"x").unwrap().as_slice(),Vec::new(),&h.route_hash,||Some(1000),|_|{prepared.set(prepared.get()+1);Status::Prepared},|_,_|panic!("no commit")).is_err());assert_eq!(prepared.get(),1);
    let mut wrong=control(&h,Phase::Commit).unwrap().0;wrong.sha256="b".repeat(64);let mut input=frame(&h,"x").unwrap();input.extend(raw(&wrong,&[]));assert!(run(&input,&mut Vec::new(),||Some(1000),|_,_|panic!("changed commit")).is_err());
}
#[test]fn clipboard_write_abort_after_expiry_discards_preparation_without_writing(){
    let h=req("x",1);let now=Cell::new(1000);let mut out=Vec::new();
    serve(cycle(&h,"x",Phase::Abort).as_slice(),&mut out,&h.route_hash,||Some(now.get()),|_|{now.set(5000);Status::Prepared},|_,_|panic!("abort effect")).unwrap();assert!(matches!(ack(final_line(&out),&control(&h,Phase::Abort).unwrap().0),Ok(Status::Aborted)));
}
#[test]fn clipboard_write_parent_revalidates_after_init_and_sends_only_abort_after_stop(){
    let h=req("x",1);let allowed=Cell::new(true);let phases=RefCell::new(Vec::new());
    let(result,disposition)=transact(&h,"x",Instant::now()+TIMEOUT,&||allowed.get(),|input,_,current|{
        let mut rest=input;let c=header(&mut rest).unwrap().unwrap();phases.borrow_mut().push(c.phase);
        match c.phase{Phase::Prepare=>{assert!(current());allowed.set(false);Ok(reply(&c,Status::Prepared))},Phase::Abort=>Ok(reply(&c,Status::Aborted)),Phase::Commit=>panic!("commit after Stop")}
    });assert!(matches!(result,Mutation::Skipped));assert!(matches!(disposition,Disposition::Advance));assert!(*phases.borrow()==[Phase::Prepare,Phase::Abort]);
}
#[test]fn clipboard_write_lost_commit_ack_retires_without_abort_or_replay(){
    let h=req("x",1);let phases=RefCell::new(Vec::new());
    let(result,disposition)=transact(&h,"x",Instant::now()+TIMEOUT,&||true,|input,_,_|{
        let mut rest=input;let c=header(&mut rest).unwrap().unwrap();phases.borrow_mut().push(c.phase);if c.phase==Phase::Prepare{Ok(reply(&c,Status::Prepared))}else{Err(ExchangeError::Uncertain)}
    });assert!(matches!(result,Mutation::Uncertain));assert!(matches!(disposition,Disposition::Retire));assert!(*phases.borrow()==[Phase::Prepare,Phase::Commit]);
}
#[test]fn clipboard_writer_process_descriptor_is_pinned_and_close_on_exec(){
    use std::os::fd::AsRawFd;
    let fd=process_fd(unsafe{libc::getpid()}).unwrap();assert!(!process_ended(&fd,0).unwrap());assert_ne!(unsafe{libc::fcntl(fd.as_raw_fd(),libc::F_GETFD)}&libc::FD_CLOEXEC,0);
}
#[test]fn clipboard_writer_invalid_process_descriptors_fail_closed(){assert!(process_fd(-1).is_err());assert!(process_fd(0).is_err());}
#[test]fn clipboard_writer_process_descriptor_survives_worker_thread_and_observes_process_death(){
    use crate::pet_process_group::OwnedChild;
    use std::os::fd::AsRawFd;
    let mut child=OwnedChild::new(crate::pet_helper::command("/usr/bin/python3",&["-I".into(),"-c".into(),"import threading,time\nt=threading.Thread(target=lambda:None);t.start();t.join()\nprint('worker-ended',flush=True)\ntime.sleep(30)".into()]).spawn().unwrap());
    let fd=process_fd(child.child.id()as i32).unwrap();let mut out=child.child.stdout.take().unwrap();crate::pet_helper::nonblocking(out.as_raw_fd()).unwrap();let mut bytes=Vec::new();let end=Instant::now()+Duration::from_secs(3);
    while !bytes.ends_with(b"\n"){assert!(Instant::now()<end);let mut buf=[0;64];match out.read(&mut buf){Ok(n)=>bytes.extend_from_slice(&buf[..n]),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>std::thread::sleep(Duration::from_millis(2)),Err(e)=>panic!("private child observation: {e}")}}
    assert_eq!(bytes,b"worker-ended\n");assert!(!process_ended(&fd,0).unwrap());child.stop_and_reap().unwrap();assert!(process_ended(&fd,1000).unwrap());
}
