//! Shared physical admission between chat instances. No persisted row is a hold.
use std::{collections::{BTreeSet,HashMap},sync::{Arc,Mutex,Weak,atomic::{AtomicBool,Ordering}},time::Instant};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Failure{Busy,Expired,Invalid,Unavailable}
#[derive(Default)]
pub(crate) struct Coordinator{holds:Mutex<HashMap<String,Weak<Hold>>>}
pub(crate) struct Hold{coordinator:Arc<Coordinator>,provider:String,cancel:Arc<AtomicBool>,deadline:Option<Instant>}
impl Coordinator{
    pub(crate) fn acquire(self:&Arc<Self>,provider:&str,cancel:Arc<AtomicBool>,deadline:Option<Instant>)->Result<Arc<Hold>,Failure>{
        if provider.is_empty()||provider.len()>128||!provider.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err(Failure::Invalid);}
        if cancel.load(Ordering::Acquire)||deadline.is_some_and(|d|Instant::now()>=d){cancel.store(true,Ordering::Release);return Err(Failure::Expired);}
        let mut holds=self.holds.lock().map_err(|_|Failure::Unavailable)?;
        // Never upgrade/drop the last Arc while this mutex is held: Hold::drop
        // also takes the mutex. A stale weak entry is safe to retire here.
        holds.retain(|_,hold|hold.strong_count()>0);
        if holds.contains_key(provider){return Err(Failure::Busy);}
        if holds.len()>=64{return Err(Failure::Unavailable);}
        let hold=Arc::new(Hold{coordinator:self.clone(),provider:provider.into(),cancel,deadline});
        holds.insert(provider.into(),Arc::downgrade(&hold));Ok(hold)
    }
    pub(crate) fn busy(&self)->Result<BTreeSet<String>,Failure>{
        let holds=self.holds.lock().map_err(|_|Failure::Unavailable)?;
        Ok(holds.iter().filter(|(_,h)|h.strong_count()>0).map(|(p,_)|p.clone()).collect())
    }
}
impl Hold{
    pub(crate)fn cancel(&self){self.cancel.store(true,Ordering::Release);}
    pub(crate)fn provider_for<'a>(&'a self,coordinator:&Arc<Coordinator>)->Option<&'a str>{(Arc::ptr_eq(&self.coordinator,coordinator)&&self.current()).then_some(self.provider.as_str())}
    pub(crate) fn current(&self)->bool{self.current_at(Instant::now())}
    fn current_at(&self,now:Instant)->bool{
        if self.cancel.load(Ordering::Acquire)||self.deadline.is_some_and(|d|now>=d){self.cancel.store(true,Ordering::Release);return false;}
        self.coordinator.holds.lock().is_ok_and(|h|h.get(&self.provider).is_some_and(|w|std::ptr::eq(w.as_ptr(),self)))
    }
}
impl Drop for Hold{
    fn drop(&mut self){if let Ok(mut holds)=self.coordinator.holds.lock(){if holds.get(&self.provider).is_some_and(|w|std::ptr::eq(w.as_ptr(),self)){holds.remove(&self.provider);}}}
}

#[cfg(test)]mod tests{
    use super::*;use std::time::Duration;
    fn flag()->Arc<AtomicBool>{Arc::new(AtomicBool::new(false))}
    #[test]fn same_pet_cannot_acquire_two_physical_holds(){let c=Arc::new(Coordinator::default());let h=c.acquire("codex",flag(),None).unwrap();assert!(matches!(c.acquire("codex",flag(),None),Err(Failure::Busy)));drop(h);assert!(c.acquire("codex",flag(),None).is_ok());}
    #[test]fn independent_pet_and_service_owners_do_not_block_each_other(){let c=Arc::new(Coordinator::default());let holds:Vec<_>=["codex","copilot","antigravity","ollama","integration_notion"].into_iter().map(|p|c.acquire(p,flag(),None).unwrap()).collect();assert_eq!(c.busy().unwrap().len(),5);drop(holds);assert!(c.busy().unwrap().is_empty());}
    #[test]fn cancelled_worker_keeps_its_slot_until_the_last_hold_drops(){let c=Arc::new(Coordinator::default());let cancel=flag();let h=c.acquire("codex",cancel.clone(),None).unwrap();let worker=h.clone();cancel.store(true,Ordering::Release);assert!(!h.current());drop(h);assert!(matches!(c.acquire("codex",flag(),None),Err(Failure::Busy)));drop(worker);assert!(c.acquire("codex",flag(),None).is_ok());}
    #[test]fn expiration_cancels_future_operations_without_reusing_a_live_slot(){let c=Arc::new(Coordinator::default());let cancel=flag();let end=Instant::now()+Duration::from_secs(30);let h=c.acquire("ollama",cancel.clone(),Some(end)).unwrap();assert!(!h.current_at(end));assert!(cancel.load(Ordering::Acquire));assert!(matches!(c.acquire("ollama",flag(),None),Err(Failure::Busy)));drop(h);assert!(c.acquire("ollama",flag(),None).is_ok());}
    #[test]fn an_expired_queued_request_never_reserves_a_slot(){let c=Arc::new(Coordinator::default());assert!(matches!(c.acquire("copilot",flag(),Some(Instant::now())),Err(Failure::Expired)));assert!(c.busy().unwrap().is_empty());}
    #[test]fn invalid_or_cancelled_requests_are_rejected_without_new_resources(){let c=Arc::new(Coordinator::default());for p in ["","../codex","pet name"]{assert!(matches!(c.acquire(p,flag(),None),Err(Failure::Invalid)));}assert!(matches!(c.acquire("codex",Arc::new(AtomicBool::new(true)),None),Err(Failure::Expired)));assert!(c.busy().unwrap().is_empty());}
}
