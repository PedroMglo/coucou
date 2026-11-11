use super::*;
#[path="system_task_cache_fixture.rs"]mod fixture;
use fixture::*;
fn review(c:&Case,a:&Audio)->(Review,std::sync::Weak<dyn Send+Sync>){let(t,w)=c.target(a);let p=&c.goal.plans[0];(c.owner.review_step(&c.goal,&p.id,&p.steps[0].id,t,&c.config,clock()).unwrap(),w)}
#[test]#[ignore="Explicit private PipeWire/Pulse readers; synthetic owner, no user audio/provider"]
fn system_task_cache_step_defers_expiry_rejected_insert_conflicting_ready_and_lease_release(){
 let _serial=serial();let audio=Audio::new();let c=Case::new(&audio);let cache=Mutex::new(Cache::default());
 let(r,first)=review(&c,&audio);let expires=r.proposal().expires_at;with_cache_at(&cache,|k|k.insert_pending("original".into(),"window".into(),"request".into(),r,clock())).unwrap();
 let(r,rejected)=review(&c,&audio);let retired={let mut k=cache.lock().unwrap();assert!(k.insert_pending("original".into(),"window".into(),"request".into(),r,clock()).is_err());assert!(rejected.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(rejected.upgrade().is_none()&&first.upgrade().is_some());
 let retired={let mut k=cache.lock().unwrap();k.sweep(expires);assert!(k.pending.is_empty()&&first.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(first.upgrade().is_none());
 let(r,first)=review(&c,&audio);let(_,ready)=r.approve(&c.owner,&c.goal,&c.config,clock()).unwrap();let scope=ready.scope.clone();with_cache_at(&cache,|k|k.remember(ready,clock())).unwrap();
 let(r,second)=review(&c,&audio);let(_,ready)=r.approve(&c.owner,&c.goal,&c.config,clock()).unwrap();let retired={let mut k=cache.lock().unwrap();assert!(k.remember(ready,clock()).is_err());assert!(second.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(second.upgrade().is_none()&&first.upgrade().is_some());
 with_cache_at(&cache,|k|{k.release("unrelated");Ok(())}).unwrap();assert!(first.upgrade().is_some());let retired={let mut k=cache.lock().unwrap();k.release(&scope.lease_id);assert!(k.ready.is_empty()&&first.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(first.upgrade().is_none());
 let(r,expired)=review(&c,&audio);let(_,ready)=r.approve(&c.owner,&c.goal,&c.config,clock()).unwrap();let end=ready.scope.deadline_at;with_cache_at(&cache,|k|k.remember(ready,clock())).unwrap();let retired={let mut k=cache.lock().unwrap();k.sweep(end);assert!(expired.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(expired.upgrade().is_none());
 c.unchanged();println!("SYSTEM_TASK_CACHE step=true retainedNativeTargets=true expiryConflictLeaseReleaseOutsideGlobal=true authorityBudgetAndPolicyUnchanged=true effects=0");
}
