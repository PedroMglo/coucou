//! Lifetime fence for a captured native callback, independent of operation
//! permissions. A retained fence cannot outlive its caller or create authority.
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use crate::{pet_native_decisions::Ticket,pet_native_write::{Guard,Use}};

#[derive(Clone)]
pub(crate) struct Fence(Arc<Guard>);
pub(crate) struct Invocation { fence:Fence, caller:Arc<AtomicBool> }
impl Invocation {
    pub(crate) fn begin(source:&Ticket,deadline:Instant,cancel:Option<Arc<AtomicBool>>)->Option<Self>{
        let caller=Arc::new(AtomicBool::new(true));
        let fence=Fence(Arc::new(Guard{deadline,cancel,caller:Some(caller.clone()),source:Some((source.clone(),Use::Decision))}));
        fence.current().then_some(Self{fence,caller})
    }
    pub(crate) fn fence(&self)->&Fence{&self.fence}
}
impl Drop for Invocation{fn drop(&mut self){self.caller.store(false,Ordering::Release);}}
impl Fence {
    pub(crate) fn current(&self)->bool{self.0.reason().is_none()}
    /// Bridge a polling native worker to this exact call. Only a local flag is
    /// cancelled; a resolved request never sets the original turn's Stop flag.
    pub(crate) fn with_cancel<T>(&self,aborted:&AtomicBool,run:impl FnOnce(&AtomicBool)->T)->Result<T,String>{
        const ENDED:&str="O pedido nativo terminou. O resultado não foi publicado; uma operação já enviada não deve ser repetida automaticamente.";
        if aborted.load(Ordering::Acquire)||!self.current(){return Err(ENDED.into());}
        let done=AtomicBool::new(false);let local=AtomicBool::new(false);
        struct Stop<'a>(&'a AtomicBool);impl Drop for Stop<'_>{fn drop(&mut self){self.0.store(true,Ordering::Release);}}
        std::thread::scope(|scope|{
            let watch=scope.spawn(||{while !done.load(Ordering::Acquire){if aborted.load(Ordering::Acquire)||!self.current(){local.store(true,Ordering::Release);break;}std::thread::sleep(Duration::from_millis(2));}});
            let stopped=Stop(&done);
            let result=if aborted.load(Ordering::Acquire)||!self.current(){None}else{Some(run(&local))};
            drop(stopped);let _=watch.join();
            if local.load(Ordering::Acquire)||aborted.load(Ordering::Acquire)||!self.current(){Err(ENDED.into())}else{result.ok_or_else(||ENDED.into())}
        })
    }
    #[cfg(test)]pub(crate) fn source(&self)->&Ticket{&self.0.source.as_ref().unwrap().0}
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn retained_fence_stops_when_the_callback_owner_departs_without_stopping_the_turn(){let(t,_)=Ticket::fixture();let stop=Arc::new(AtomicBool::new(false));let call=Invocation::begin(&t,Instant::now()+Duration::from_secs(2),Some(stop.clone())).unwrap();let fence=call.fence().clone();assert!(fence.current());drop(call);assert!(!fence.current());assert!(t.current());assert!(!stop.load(Ordering::Acquire));}
    #[test]fn source_resolution_deadline_and_actual_stop_each_retire_the_callback(){for reason in 0..3{let(t,_)=Ticket::fixture();let stop=Arc::new(AtomicBool::new(false));let call=Invocation::begin(&t,Instant::now()+Duration::from_millis(if reason==1{20}else{2000}),Some(stop.clone())).unwrap();match reason{0=>t.cancel(),1=>std::thread::sleep(Duration::from_millis(25)),_=>stop.store(true,Ordering::Release)}assert!(!call.fence().current());}}
    #[test]fn worker_cancellation_is_local_and_its_late_result_is_discarded(){let(t,_)=Ticket::fixture();let call=Invocation::begin(&t,Instant::now()+Duration::from_secs(2),None).unwrap();let original=AtomicBool::new(false);let result=call.fence().with_cancel(&original,|local|{t.cancel();let until=Instant::now()+Duration::from_secs(1);while !local.load(Ordering::Acquire){assert!(Instant::now()<until);std::thread::yield_now();}"private late result"});assert!(result.is_err());assert!(!original.load(Ordering::Acquire));}
    #[test]fn an_ended_source_never_dispatches_the_worker(){let(t,_)=Ticket::fixture();let call=Invocation::begin(&t,Instant::now()+Duration::from_secs(2),None).unwrap();t.cancel();assert!(call.fence().with_cancel(&AtomicBool::new(false),|_|panic!("late dispatch")).is_err());}
}
