use super::*;
use std::sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}};
#[derive(Default)]struct Fake{events:Vec<&'static str>,stop:Arc<AtomicBool>,retire:Option<&'static str>,stage:Arc<AtomicUsize>,put:Option<Mutation>,dispatch:Option<Dispatch>,initial:Option<Previous>,later:Option<Previous>,reads:usize,writes:usize,restore_put:Option<Mutation>}
impl Fake{fn event(&mut self,at:&'static str){self.events.push(at);self.stage.store(self.events.len(),Ordering::Release);if self.retire==Some(at){self.stop.store(true,Ordering::Release);}}}
impl Port for Fake{
 fn observe(&mut self,_:&dyn Fn()->bool)->Previous{self.reads+=1;if self.reads==1{self.event("read");self.initial.take().unwrap_or(Previous::Text("before".into()))}else{self.event("restore-read");self.later.take().unwrap_or(Previous::Text("synthetic".into()))}}
 fn put(&mut self,text:&str,_:&dyn Fn()->bool)->Mutation{self.writes+=1;if self.writes==1{assert_eq!(text,"synthetic");self.event("put");self.put.unwrap_or(Mutation::Done)}else{assert_eq!(text,"before");self.event("restore-put");self.restore_put.unwrap_or(Mutation::Done)}}
 fn dispatch(&mut self,_:bool,_:&dyn Fn()->bool)->Dispatch{self.event("keys");self.dispatch.unwrap_or(Dispatch::Dispatched)}
}
fn run(fake:&mut Fake)->Report{let stop=fake.stop.clone();perform(fake,"synthetic",true,&||!stop.load(Ordering::Acquire),Duration::ZERO,Duration::ZERO)}
#[test]fn retired_before_copy_has_no_native_effect(){let mut f=Fake::default();f.stop.store(true,Ordering::Release);assert_eq!(run(&mut f).outcome,Outcome::NotStarted);assert!(f.events.is_empty());}
#[test]fn retired_after_read_cannot_copy(){let mut f=Fake{retire:Some("read"),..Default::default()};assert_eq!(run(&mut f).outcome,Outcome::NotStarted);assert_eq!(f.events,["read"]);}
#[test]fn retired_after_copy_retains_partial_effect_without_keys(){let mut f=Fake{retire:Some("put"),..Default::default()};assert_eq!(run(&mut f),Report{outcome:Outcome::Copied,previous:PreviousState::Text,restore:Restore::NotRequested});assert_eq!(f.events,["read","put"]);}
#[test]fn copy_error_is_uncertain_and_never_retried(){let mut f=Fake{put:Some(Mutation::Uncertain),..Default::default()};assert_eq!(run(&mut f).outcome,Outcome::CopyUncertain);assert_eq!(f.events,["read","put"]);}
#[test]fn dispatch_error_cannot_claim_copied_only_or_success(){let mut f=Fake{dispatch:Some(Dispatch::Uncertain),..Default::default()};assert_eq!(run(&mut f).outcome,Outcome::DispatchUncertain);assert_eq!(f.events,["read","put","keys"]);}
#[test]fn no_dispatch_retains_copy_without_restore(){let mut f=Fake{dispatch:Some(Dispatch::NotSent),..Default::default()};assert_eq!(run(&mut f).outcome,Outcome::Copied);assert_eq!(f.events,["read","put","keys"]);}
#[test]fn revoke_after_dispatch_never_fabricates_no_effect_or_restores(){let mut f=Fake{retire:Some("keys"),..Default::default()};assert_eq!(run(&mut f),Report{outcome:Outcome::Dispatched,previous:PreviousState::Text,restore:Restore::Retired});assert_eq!(f.events,["read","put","keys"]);}
#[test]fn live_occurrence_owns_restore_until_return(){let mut f=Fake::default();assert_eq!(run(&mut f),Report{outcome:Outcome::Dispatched,previous:PreviousState::Text,restore:Restore::WriteAcknowledged});assert_eq!(f.events,["read","put","keys","restore-read","restore-put"]);}
#[test]fn excessive_text_is_refused_before_native_reads(){let mut f=Fake::default();let text="x".repeat(MAX_TEXT_BYTES+1);let report=perform(&mut f,&text,false,&||true,Duration::ZERO,Duration::ZERO);assert_eq!(report.outcome,Outcome::OutputTooLarge);assert!(!report.retain_output());assert!(f.events.is_empty());}
#[test]fn cancellation_during_announcement_prevents_keys(){let mut f=Fake::default();let stage=f.stage.clone();let polls=AtomicUsize::new(0);let report=perform(&mut f,"synthetic",false,&||stage.load(Ordering::Acquire)<2||polls.fetch_add(1,Ordering::AcqRel)<1,Duration::from_millis(40),Duration::ZERO);assert_eq!(report.outcome,Outcome::Copied);assert_eq!(f.events,["read","put"]);}
#[test]fn cancellation_during_restore_wait_prevents_new_write(){let mut f=Fake::default();let stage=f.stage.clone();let polls=AtomicUsize::new(0);let report=perform(&mut f,"synthetic",false,&||stage.load(Ordering::Acquire)<3||polls.fetch_add(1,Ordering::AcqRel)<1,Duration::ZERO,Duration::from_millis(700));assert_eq!(report,Report{outcome:Outcome::Dispatched,previous:PreviousState::Text,restore:Restore::Retired});assert_eq!(f.events,["read","put","keys"]);}
#[test]fn cancellation_does_not_wait_for_the_clipboard_mutex(){
 let slot=Mutex::new(Some(()));let held=slot.lock().unwrap();let stop=AtomicBool::new(false);let(tx,rx)=std::sync::mpsc::channel();
 std::thread::scope(|scope|{
  let worker=scope.spawn(||with_current::<(),()>(&slot,&||{tx.send(()).unwrap();!stop.load(Ordering::Acquire)},||panic!("must not init"),|_|panic!("must not mutate")));
  rx.recv_timeout(Duration::from_secs(1)).unwrap();stop.store(true,Ordering::Release);
  assert!(worker.join().unwrap().is_none());
 });drop(held);
}
#[test]fn native_initialization_cannot_renew_a_retired_scope(){let slot:Mutex<Option<()>>=Mutex::new(None);let live=AtomicBool::new(true);let result:Option<()>=with_current(&slot,&||live.load(Ordering::Acquire),||{live.store(false,Ordering::Release);Some(())},|_|panic!("retired init must not mutate"));assert!(result.is_none());}
#[test]fn sequence_releases_only_attempted_keys_when_cancelled(){
 let live=AtomicBool::new(true);let mut sent=vec![];
 let result=sequence(true,&||live.load(Ordering::Acquire),|k,down|{sent.push((k,down));if k==Key::Control&&down{live.store(false,Ordering::Release);}Ok(())});
 assert_eq!(result,Dispatch::Uncertain);assert_eq!(sent,[(Key::Control,true),(Key::Control,false)]);
}
#[test]fn lost_press_ack_is_not_replayed_and_gets_owned_releases(){
 let mut sent=vec![];let result=sequence(true,&||true,|k,down|{sent.push((k,down));if k==Key::V&&down{Err(KeyFailure::Uncertain)}else{Ok(())}});
 assert_eq!(result,Dispatch::Uncertain);assert_eq!(sent,[(Key::Control,true),(Key::Shift,true),(Key::V,true),(Key::V,false),(Key::Shift,false),(Key::Control,false)]);
}
#[test]fn cancelled_sequence_never_sends_a_press(){let mut sent=0;assert_eq!(sequence(true,&||false,|_,_|{sent+=1;Ok(())}),Dispatch::NotSent);assert_eq!(sent,0);}
#[test]fn successful_sequence_means_dispatch_not_target_ack(){let mut sent=vec![];assert_eq!(sequence(false,&||true,|k,down|{sent.push((k,down));Ok(())}),Dispatch::Dispatched);assert_eq!(sent,[(Key::Control,true),(Key::V,true),(Key::V,false),(Key::Control,false)]);}

#[test]fn a_press_never_sent_does_not_authorize_a_release(){
 let mut events=vec![];let result=sequence(true,&||true,|key,down|{events.push((key,down));Err(KeyFailure::NotSent)});assert_eq!(result,Dispatch::NotSent);assert_eq!(events,[(Key::Control,true)]);
}
#[test]fn unsent_shift_releases_prior_ctrl_but_never_releases_shift(){
 let mut events=vec![];let result=sequence(true,&||true,|key,down|{events.push((key,down));if key==Key::Shift&&down{Err(KeyFailure::NotSent)}else{Ok(())}});assert_eq!(result,Dispatch::Uncertain);assert_eq!(events,[(Key::Control,true),(Key::Shift,true),(Key::Control,false)]);
}

#[test]fn unavailable_oversized_or_non_text_previous_content_is_never_silently_overwritten(){
 for previous in[Previous::NoText,Previous::Unavailable,Previous::TooLarge]{let mut f=Fake{initial:Some(previous),..Default::default()};let report=run(&mut f);assert_eq!(report.outcome,Outcome::PreservationBlocked);assert!(report.retain_output());assert_eq!(f.events,["read"]);assert!(!report.preservation_message().is_empty());}
}
#[test]fn observed_vacancy_allows_copy_without_inventing_a_backup(){let mut f=Fake{initial:Some(Previous::Vacant),..Default::default()};let report=run(&mut f);assert_eq!(report,Report{outcome:Outcome::Dispatched,previous:PreviousState::Vacant,restore:Restore::NoPrevious});assert_eq!(f.events,["read","put","keys"]);}
#[test]fn second_observation_cannot_overwrite_changed_or_unreadable_content(){
 for (later,expected)in[(Previous::Text("external".into()),Restore::Changed),(Previous::Vacant,Restore::Changed),(Previous::NoText,Restore::Changed),(Previous::Unavailable,Restore::Unavailable),(Previous::TooLarge,Restore::TooLarge),(Previous::Retired,Restore::Retired)]{let mut f=Fake{later:Some(later),..Default::default()};let report=run(&mut f);assert_eq!(report.outcome,Outcome::Dispatched);assert_eq!(report.restore,expected);assert_eq!(f.events,["read","put","keys","restore-read"]);}
}
#[test]fn cancellation_during_second_observation_cannot_write_the_previous_text(){let mut f=Fake{retire:Some("restore-read"),..Default::default()};let report=run(&mut f);assert_eq!(report.outcome,Outcome::Dispatched);assert_eq!(report.restore,Restore::Retired);assert_eq!(f.events,["read","put","keys","restore-read"]);}
#[test]fn restore_error_preserves_uncertainty_without_replaying_copy_or_keys(){let mut f=Fake{restore_put:Some(Mutation::Uncertain),..Default::default()};let report=run(&mut f);assert_eq!(report.restore,Restore::WriteUncertain);assert!(report.restore_warning().is_some());assert_eq!(f.writes,2);assert_eq!(f.reads,2);}
#[test]fn a_write_refused_after_observation_does_not_claim_restore(){let mut f=Fake{restore_put:Some(Mutation::Skipped),..Default::default()};let report=run(&mut f);assert_eq!(report.restore,Restore::WriteSkipped);assert!(report.restore_warning().is_some());}
