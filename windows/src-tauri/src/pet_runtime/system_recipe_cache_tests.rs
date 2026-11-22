use super::*;
#[path="system_task_cache_fixture.rs"]mod fixture;
use fixture::*;
fn review(c:&Case,a:&Audio)->(Review,std::sync::Weak<dyn Send+Sync>){let(t,w)=c.target(a);(c.owner.review_recipe_with_criteria(&c.goal,&c.goal.plans[0].id,"cache-recipe",t,&c.config,&[],clock()).unwrap(),w)}
fn retained(c:&Case,a:&Audio)->(Arc<Retained>,std::sync::Weak<dyn Send+Sync>){let(r,w)=review(c,a);let(approved,ready)=r.approve(&c.owner,&c.goal,&c.config,clock()).unwrap();(Arc::new(Retained{window:"window".into(),revision:c.goal.revision.clone(),token:"fixture".into(),approved,ready}),w)}
#[test]#[ignore="Explicit private PipeWire/Pulse readers; synthetic owner, no user audio/provider"]
fn system_task_cache_recipe_defers_expiry_rejected_insert_capacity_and_exact_lease_release(){
 let _serial=serial();let audio=Audio::new();let c=Case::new(&audio);let cache=Mutex::new(Cache::default());
 let(r,first)=review(&c,&audio);let expires=r.proposal().expires_at;with_cache_at(&cache,|k|k.insert_pending("original".into(),"window".into(),"request".into(),r,clock())).unwrap();let(r,rejected)=review(&c,&audio);
 let retired={let mut k=cache.lock().unwrap();assert!(k.insert_pending("original".into(),"window".into(),"request".into(),r,clock()).is_err());assert!(rejected.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(rejected.upgrade().is_none()&&first.upgrade().is_some());
 let retired={let mut k=cache.lock().unwrap();k.sweep(expires);assert!(k.pending.is_empty()&&first.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(first.upgrade().is_none());
 let(r,first)=retained(&c,&audio);let deadline=r.ready.scope().deadline_at;with_cache_at(&cache,|k|{k.remember(r.clone())?;for i in 1..64{k.ready.insert(format!("capacity-fixture-{i}"),r.clone());}Ok(())}).unwrap();drop(r);let(r,rejected)=retained(&c,&audio);
 let retired={let mut k=cache.lock().unwrap();assert!(k.remember(r).is_err());assert!(rejected.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(rejected.upgrade().is_none()&&first.upgrade().is_some());
 with_cache_at(&cache,|k|{k.release("unrelated");Ok(())}).unwrap();assert!(first.upgrade().is_some());let retired={let mut k=cache.lock().unwrap();k.sweep(deadline);assert!(k.ready.is_empty()&&first.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(first.upgrade().is_none());
 let(r,released)=retained(&c,&audio);with_cache_at(&cache,|k|k.remember(r)).unwrap();let retired={let mut k=cache.lock().unwrap();k.release(&c.owner.lease().id);assert!(k.ready.is_empty()&&released.upgrade().is_some());k.take_retired()};assert!(cache.try_lock().is_ok());drop(retired);assert!(released.upgrade().is_none());c.unchanged();println!("SYSTEM_TASK_CACHE recipe=true retainedNativeTargets=true expiryCapacityLeaseReleaseOutsideGlobal=true authorityBudgetAndPolicyUnchanged=true effects=0");
}
