use super::*;
fn waiting(c:&Arc<Coordinator>,provider:&str)->Arc<Hold>{Arc::new(Hold{coordinator:c.clone(),provider:provider.into(),resource:provider.into(),captured:None,cancel:Arc::default(),deadline:Some(Instant::now()+Duration::from_secs(60)),generation:AtomicU64::new(1)})}
fn request(h:&Arc<Hold>)->Result<(),Failure>{h.coordinator.state.lock().unwrap().reacquire(h,1,2,Instant::now())}
#[test]fn initial_requested_goals_use_the_same_fifo_and_do_not_release_metadata_io_on_stop(){
 let c=Arc::new(Coordinator::default());let manual=c.acquire("ollama",Arc::default(),None).unwrap();let initial=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let resumed=waiting(&c,"ollama");request(&resumed).unwrap_err();assert_eq!(initial.waiting_info().unwrap().position,1);assert_eq!(resumed.waiting_info().unwrap().position,2);drop(manual);
 for _ in 0..MAX_INTERACTIVE_BYPASSES{drop(c.acquire("ollama",Arc::default(),None).unwrap());}assert!(c.acquire("ollama",Arc::default(),None).is_err());assert_eq!(request(&resumed),Err(Failure::Busy));let worker=initial.poll().unwrap();assert_eq!(worker.generation(),1);initial.cancel();drop(initial);assert_eq!(request(&resumed),Err(Failure::Busy));drop(worker);request(&resumed).unwrap();assert!(resumed.current());
}
#[test]fn initial_request_cancellation_and_deadline_do_not_change_a_different_provider_or_owner(){
 let c=Arc::new(Coordinator::default());let manual=c.acquire("ollama",Arc::default(),None).unwrap();let independent=c.acquire("copilot",Arc::default(),None).unwrap();let first=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();let second=c.request_goal("ollama",Arc::default(),Instant::now()+Duration::from_secs(60)).unwrap();first.cancel();assert!(first.poll().is_err());assert_eq!(second.waiting_info().unwrap().position,1);assert!(manual.current()&&independent.current());assert!(c.request_goal("ollama",Arc::default(),Instant::now()).is_err());drop(manual);assert!(second.poll().unwrap().current());assert!(independent.current());
}
#[test]fn interactive_burst_is_bounded_and_background_order_is_fifo(){
 let c=Arc::new(Coordinator::default());let busy=c.acquire("ollama",Arc::default(),None).unwrap();let first=waiting(&c,"ollama");let second=waiting(&c,"ollama");assert_eq!(request(&first),Err(Failure::Busy));assert_eq!(request(&second),Err(Failure::Busy));assert_eq!(first.waiting_info().unwrap().position,1);assert_eq!(second.waiting_info().unwrap().position,2);drop(busy);
 for count in 1..=MAX_INTERACTIVE_BYPASSES{let manual=c.acquire("ollama",Arc::default(),None).unwrap();assert_eq!(first.waiting_info().unwrap().interactive_bypasses,count);drop(manual);}
 assert!(matches!(c.acquire("ollama",Arc::default(),None),Err(Failure::Busy)));assert!(matches!(c.acquire_goal("ollama",Arc::default(),None),Err(Failure::Busy)));assert_eq!(request(&second),Err(Failure::Busy));request(&first).unwrap();assert!(first.current());assert_eq!(first.generation(),2);assert!(first.waiting_info().is_none());assert_eq!(second.waiting_info().unwrap().position,1);
 let foreign=c.acquire("copilot",Arc::default(),None).unwrap();assert!(foreign.current());first.cancel();let worker=first.clone();drop(first);assert_eq!(request(&second),Err(Failure::Busy));drop(worker);request(&second).unwrap();assert!(second.current()&&foreign.current());
}
#[test]fn cancelling_a_waiter_removes_only_it_and_never_the_active_holder(){
 let c=Arc::new(Coordinator::default());let holder=c.acquire("ollama",Arc::default(),None).unwrap();let a=waiting(&c,"ollama");let b=waiting(&c,"ollama");request(&a).unwrap_err();request(&b).unwrap_err();a.cancel();assert!(a.waiting_info().is_none());assert_eq!(b.waiting_info().unwrap().position,1);assert!(holder.current());assert_eq!(request(&a),Err(Failure::Expired));drop(holder);request(&b).unwrap();assert!(b.current());
}
#[test]fn passive_position_does_not_renew_freshness_and_stale_ticket_cannot_monopolize_idle_slot(){
 let c=Arc::new(Coordinator::default());let holder=c.acquire("ollama",Arc::default(),None).unwrap();let h=waiting(&c,"ollama");request(&h).unwrap_err();let original_deadline=h.deadline;
 let last=c.state.lock().unwrap().waiters[0].last_poll;for _ in 0..10{assert_eq!(h.waiting_info().unwrap().position,1);}assert_eq!(c.state.lock().unwrap().waiters[0].last_poll,last);
 c.state.lock().unwrap().waiters[0].last_poll=Instant::now().checked_sub(WAITER_FRESHNESS+Duration::from_millis(1)).unwrap();assert!(h.waiting_info().is_none());drop(holder);let manual=c.acquire("ollama",Arc::default(),None).unwrap();assert!(manual.current());assert_eq!(h.deadline,original_deadline);assert_eq!(h.generation(),1);assert_eq!(request(&h),Err(Failure::Busy));assert_eq!(h.waiting_info().unwrap().interactive_bypasses,0);
}
#[test]fn cancellation_flag_never_waits_for_the_arbitration_mutex(){
 let c=Arc::new(Coordinator::default());let holder=c.acquire("ollama",Arc::default(),None).unwrap();let h=waiting(&c,"ollama");request(&h).unwrap_err();let state=c.state.lock().unwrap();h.cancel();assert!(h.cancel.load(Ordering::Acquire));drop(state);assert!(h.waiting_info().is_none());assert!(holder.current());
}
#[test]fn queue_capacity_does_not_evict_live_waiters_or_leak_a_slot(){
 let c=Arc::new(Coordinator::default());let holder=c.acquire("ollama",Arc::default(),None).unwrap();let holds=(0..MAX_WAITERS).map(|_|waiting(&c,"ollama")).collect::<Vec<_>>();for h in &holds{assert_eq!(request(h),Err(Failure::Busy));}let extra=waiting(&c,"ollama");assert_eq!(request(&extra),Err(Failure::Unavailable));assert_eq!(holds[0].waiting_info().unwrap().position,1);assert_eq!(holds[MAX_WAITERS-1].waiting_info().unwrap().position,256);assert!(extra.waiting_info().is_none());assert!(holder.current());
}
#[test]fn expiry_and_wrong_generation_cannot_acquire_or_renew_original_hold(){
 let c=Arc::new(Coordinator::default());let expired=Arc::new(Hold{coordinator:c.clone(),provider:"ollama".into(),resource:"ollama".into(),captured:None,cancel:Arc::default(),deadline:Some(Instant::now()),generation:AtomicU64::new(1)});assert_eq!(request(&expired),Err(Failure::Expired));let h=waiting(&c,"ollama");let end=h.deadline;assert_eq!(c.state.lock().unwrap().reacquire(&h,2,3,Instant::now()),Err(Failure::Invalid));assert!(!h.occupies_slot());request(&h).unwrap();request(&h).unwrap();assert_eq!(h.generation(),2);assert_eq!(h.deadline,end);assert!(h.waiting_info().is_none());assert_eq!(c.busy().unwrap().len(),1);
}
