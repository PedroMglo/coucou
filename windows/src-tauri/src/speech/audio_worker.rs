//! Native capture ownership. Timeout revokes effects; only join releases holds.
//! Uses the common ephemeral resource coordinator, not another authority ledger.
use std::{sync::{Arc,Mutex,OnceLock,mpsc,atomic::{AtomicBool,Ordering}},thread::JoinHandle,time::{Duration,Instant}};
use crate::pet_resources::{Coordinator,Hold};
#[derive(Default)]struct Pool{resources:Arc<Coordinator>,pending:Mutex<Vec<Owner>>}
pub(super)struct Authority{resources:Arc<Coordinator>,owner:String,cancel:Arc<AtomicBool>,external:Option<Arc<AtomicBool>>,holds:mpsc::Sender<Arc<Hold>>}
impl Authority{
    pub(super)fn current(&self)->bool{!self.cancel.load(Ordering::Acquire)&&!stopped(&self.external)}
    #[cfg(test)]
    pub(super)fn cancellation(&self)->Arc<AtomicBool>{self.cancel.clone()}
    pub(super)fn guard(&self)->Arc<dyn Fn()->bool+Send+Sync>{let cancel=self.cancel.clone();let external=self.external.clone();Arc::new(move||!cancel.load(Ordering::Acquire)&&!stopped(&external))}
    pub(super)fn device(&self,identity:&str)->Result<(),String>{
        if !self.current(){return Err("Audio capture was cancelled before opening the device.".into());}
        // Input capture is a read. Independent, authorized feature owners
        // can share a native endpoint; their leases never cancel each other.
        // The stable owner reservation prevents its own retarget/retry while
        // an earlier worker is still outstanding, even if the device changed.
        let read=format!("owner:{}:{}:endpoint:{}:{}",self.owner.len(),self.owner,identity.len(),identity);
        let hold=self.resources.acquire(&key("read",&read)?,self.cancel.clone(),None).map_err(|_|"This capture owner's previous endpoint read is still occupied or unavailable.".to_owned())?;
        self.holds.send(hold).map_err(|_|"The capture owner is no longer available.".to_owned())?;
        if self.current(){Ok(())}else{Err("Audio capture was cancelled before opening the device.".into())}
    }
}
pub(super)struct Owner{
    cancel:Arc<AtomicBool>,stop:Option<mpsc::Sender<()>>,thread:Option<JoinHandle<()>>,
    // These fields MUST outlive join, including on error and unwinding.
    _request:Arc<Hold>,_devices:mpsc::Receiver<Arc<Hold>>,
}
impl Owner{
    fn signal(&mut self){self.cancel.store(true,Ordering::Release);if let Some(stop)=self.stop.take(){let _=stop.send(());}}
    fn finished(&self)->bool{self.thread.as_ref().is_none_or(JoinHandle::is_finished)}
}
impl Drop for Owner{
    fn drop(&mut self){self.signal();if let Some(thread)=self.thread.take(){let _=thread.join();}}
}
fn key(kind:&str,identity:&str)->Result<String,String>{
    if identity.is_empty()||identity.len()>8192{return Err("The audio resource identity could not be confirmed.".into());}
    let hash=ring::digest::digest(&ring::digest::SHA256,identity.as_bytes()).as_ref().iter().map(|b|format!("{b:02x}")).collect::<String>();
    Ok(format!("audio_{kind}_{hash}"))
}
fn stopped(external:&Option<Arc<AtomicBool>>)->bool{external.as_ref().is_some_and(|flag|flag.load(Ordering::Acquire))}
impl Pool{
    fn reap(&self){
        let finished={let mut pending=self.pending.lock().unwrap_or_else(|e|e.into_inner());let(mut finished,mut live)=(vec![],vec![]);
            for owner in pending.drain(..){if owner.finished(){finished.push(owner);}else{live.push(owner);}}*pending=live;finished};
        // Join and last-Arc drops happen outside every registry lock.
        drop(finished);
    }
    fn retire(&self,mut owner:Owner){
        owner.signal();if owner.finished(){drop(owner);}else{
            // Each pending owner holds a distinct request reservation; the
            // common coordinator admits at most64 resources, including these.
            self.pending.lock().unwrap_or_else(|e|e.into_inner()).push(owner);
        }
    }
    #[cfg(test)]
    fn start<S:'static>(self:&Arc<Self>,identity:&str,timeout:Duration,build:impl FnOnce(&Authority)->Result<S,String>+Send+'static,play:impl FnOnce(&S)->Result<(),String>+Send+'static)->Result<Owner,String>{self.start_guarded(identity,timeout,None,build,play)}
    fn start_guarded<S:'static>(self:&Arc<Self>,identity:&str,timeout:Duration,external:Option<Arc<AtomicBool>>,
        build:impl FnOnce(&Authority)->Result<S,String>+Send+'static,
        play:impl FnOnce(&S)->Result<(),String>+Send+'static,
    )->Result<Owner,String>{
        self.reap();if stopped(&external){return Err("Audio capture is cancelled.".into());}let until=Instant::now().checked_add(timeout).ok_or("Invalid audio startup deadline.")?;let cancel=Arc::new(AtomicBool::new(false));
        let request=self.resources.acquire(&key("request",identity)?,cancel.clone(),None).map_err(|_|"The previous capture worker still owns this request or resources are unavailable.".to_owned())?;
        let(ready_tx,ready_rx)=mpsc::channel();let(stop_tx,stop_rx)=mpsc::channel();let(holds_tx,holds_rx)=mpsc::channel();
        let mut owner=Owner{cancel:cancel.clone(),stop:Some(stop_tx),thread:None,_request:request,_devices:holds_rx};
        let authority=Authority{resources:self.resources.clone(),owner:identity.into(),cancel,external:external.clone(),holds:holds_tx};
        owner.thread=Some(std::thread::Builder::new().name("coucou-audio".into()).spawn(move||{
            if !authority.current(){return;}
            let stream=match build(&authority){Ok(stream)=>stream,Err(error)=>{let _=ready_tx.send(Err(error));return;}};
            if !authority.current(){return;}
            if let Err(error)=play(&stream){let _=ready_tx.send(Err(error));return;}
            if !authority.current()||ready_tx.send(Ok(())).is_err(){return;}
            while authority.current(){match stop_rx.recv_timeout(Duration::from_millis(25)){Err(mpsc::RecvTimeoutError::Timeout)=>{},_=>break}}
            drop(stream);
        }).map_err(|error|error.to_string())?);
        loop{
            if owner.cancel.load(Ordering::Acquire)||stopped(&external){self.retire(owner);return Err("Audio capture was cancelled. Resources remain reserved until the original worker is collected.".into());}
            let remaining=until.saturating_duration_since(Instant::now());if remaining.is_zero(){self.retire(owner);return Err("The audio worker did not confirm capture. It remains reserved until it has finished and been collected; no late activation is authorized.".into());}
            match ready_rx.recv_timeout(remaining.min(Duration::from_millis(25))){
                Ok(Ok(()))if !stopped(&external)&&!owner.cancel.load(Ordering::Acquire)=>return Ok(owner),
                Ok(Ok(()))=>{self.retire(owner);return Err("Audio capture was cancelled before readiness was accepted.".into());},
                Ok(Err(error))=>{self.retire(owner);return Err(error)},
                Err(mpsc::RecvTimeoutError::Timeout)=>{},
                Err(mpsc::RecvTimeoutError::Disconnected)=>{self.retire(owner);return Err("The audio worker ended without a readiness receipt.".into());},
            }
        }
    }
}
pub(super)fn start<S:'static>(identity:&str,timeout:Duration,external:Option<Arc<AtomicBool>>,build:impl FnOnce(&Authority)->Result<S,String>+Send+'static,play:impl FnOnce(&S)->Result<(),String>+Send+'static)->Result<Owner,String>{
    static POOL:OnceLock<Arc<Pool>>=OnceLock::new();POOL.get_or_init(||Arc::new(Pool::default())).start_guarded(identity,timeout,external,build,play)
}

#[cfg(test)]mod tests{
    use super::*;use std::time::Instant;
    fn until(mut ready:impl FnMut()->bool){let end=Instant::now()+Duration::from_secs(3);while !ready(){assert!(Instant::now()<end,"original worker did not finish");std::thread::sleep(Duration::from_millis(1));}}
    fn ready(pool:&Arc<Pool>,request:&str,device:&'static str)->Result<Owner,String>{pool.start(request,Duration::from_secs(1),move|a|{a.device(device)?;Ok(())},|_|Ok(()))}
    #[test]fn startup_timeout_keeps_the_actual_endpoint_until_original_worker_finishes_and_is_joined(){
        let pool=Arc::new(Pool::default());let(release_tx,release_rx)=mpsc::channel();let(held_tx,held_rx)=mpsc::channel();let played=Arc::new(AtomicBool::new(false));let play=played.clone();
        let result=pool.start("manual",Duration::from_millis(200),move|a|{a.device("actual-microphone")?;held_tx.send(()).unwrap();release_rx.recv().unwrap();Ok(())},move|_|{play.store(true,Ordering::Release);Ok(())});assert!(result.is_err());held_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(!played.load(Ordering::Acquire));assert!(ready(&pool,"manual","different-device").is_err());let reader=ready(&pool,"dictation","actual-microphone").unwrap();drop(reader);assert!(ready(&pool,"manual","actual-microphone").is_err());
        release_tx.send(()).unwrap();until(||{pool.reap();pool.resources.busy().unwrap().is_empty()});assert!(!played.load(Ordering::Acquire),"late build must never play after caller timeout");drop(ready(&pool,"fresh-review","actual-microphone").unwrap());assert!(pool.resources.busy().unwrap().is_empty());
    }
    struct SlowCleanup{entered:mpsc::Sender<()>,release:mpsc::Receiver<()>}
    impl Drop for SlowCleanup{fn drop(&mut self){self.entered.send(()).unwrap();self.release.recv_timeout(Duration::from_secs(3)).unwrap();}}
    #[test]fn native_failure_with_cleanup_in_progress_does_not_release_a_live_endpoint(){
        let pool=Arc::new(Pool::default());let(entered_tx,entered_rx)=mpsc::channel();let(release_tx,release_rx)=mpsc::channel();
        assert!(pool.start("manual",Duration::from_secs(1),move|a|{a.device("mic")?;Ok(SlowCleanup{entered:entered_tx,release:release_rx})},|_|Err("native play failed".into())).is_err());entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(ready(&pool,"manual","different-mic").is_err());let reader=ready(&pool,"notes","mic").unwrap();drop(reader);assert!(ready(&pool,"manual","mic").is_err());release_tx.send(()).unwrap();until(||{pool.reap();pool.resources.busy().unwrap().is_empty()});drop(ready(&pool,"manual","mic").unwrap());
    }
    #[test]fn microphone_and_system_on_distinct_endpoints_remain_independent(){
        let pool=Arc::new(Pool::default());let microphone=ready(&pool,"microphone","mic").unwrap();let system=ready(&pool,"system","loopback").unwrap();assert_eq!(pool.resources.busy().unwrap().len(),4);
        drop(microphone);assert_eq!(pool.resources.busy().unwrap().len(),2);assert!(ready(&pool,"system","different-loopback").is_err());drop(system);pool.reap();assert!(pool.resources.busy().unwrap().is_empty());
    }
    #[test]fn native_panic_or_inventory_failure_never_becomes_ready_or_loses_ownership(){
        let pool=Arc::new(Pool::default());
        assert!(pool.start::<()>("panic",Duration::from_secs(1),|a|{a.device("mic")?;panic!("synthetic native worker panic")},|_|Ok(())).is_err());
        until(||{pool.reap();pool.resources.busy().unwrap().is_empty()});
        assert!(pool.start::<()>("missing",Duration::from_secs(1),|_|Err("inventory unavailable".into()),|_|panic!("failed build played")).is_err());until(||{pool.reap();pool.resources.busy().unwrap().is_empty()});
    }
    #[test]fn cancellation_after_native_preparation_prevents_play_and_keeps_holds_until_cleanup(){
        let pool=Arc::new(Pool::default());let played=Arc::new(AtomicBool::new(false));let p=played.clone();
        assert!(pool.start("manual",Duration::from_secs(1),|a|{a.device("mic")?;a.cancellation().store(true,Ordering::Release);Ok(())},move|_|{p.store(true,Ordering::Release);Ok(())}).is_err());until(||{pool.reap();pool.resources.busy().unwrap().is_empty()});assert!(!played.load(Ordering::Acquire));
    }
    #[test]fn external_stop_returns_during_blocked_startup_but_cannot_free_the_original_endpoint(){
        let pool=Arc::new(Pool::default());let outside=Arc::new(AtomicBool::new(false));let(entered_tx,entered_rx)=mpsc::channel();let(release_tx,release_rx)=mpsc::channel();let(result_tx,result_rx)=mpsc::channel();let p=pool.clone();let cancel=outside.clone();
        let caller=std::thread::spawn(move||{let r=p.start_guarded("manual",Duration::from_secs(10),Some(cancel),move|a|{a.device("mic")?;entered_tx.send(()).unwrap();release_rx.recv().unwrap();Ok(())},|_|panic!("stopped startup played"));result_tx.send(r).unwrap();});
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();outside.store(true,Ordering::Release);assert!(result_rx.recv_timeout(Duration::from_secs(1)).unwrap().is_err());assert!(ready(&pool,"manual","different-mic").is_err());release_tx.send(()).unwrap();caller.join().unwrap();until(||{pool.reap();pool.resources.busy().unwrap().is_empty()});
    }
    struct StreamDrop(Arc<AtomicBool>);
    impl Drop for StreamDrop{fn drop(&mut self){self.0.store(true,Ordering::Release);}}
    #[test]fn external_revocation_stops_delivery_and_stream_but_normal_capture_completion_does_not_cancel_the_turn(){
        let pool=Arc::new(Pool::default());let outside=Arc::new(AtomicBool::new(false));
        let first=pool.start_guarded("manual",Duration::from_secs(1),Some(outside.clone()),|a|{a.device("mic")?;Ok(())},|_|Ok(())).unwrap();drop(first);assert!(!outside.load(Ordering::Acquire),"closing capture must not cancel inference that follows it");
        let dropped=Arc::new(AtomicBool::new(false));let d=dropped.clone();let(tx,rx)=mpsc::channel();
        let owner=pool.start_guarded("manual",Duration::from_secs(1),Some(outside.clone()),move|a|{a.device("mic")?;tx.send(a.guard()).unwrap();Ok(StreamDrop(d))},|_|Ok(())).unwrap();let data_allowed=rx.recv().unwrap();assert!(data_allowed());outside.store(true,Ordering::Release);assert!(!data_allowed());until(||dropped.load(Ordering::Acquire));assert_eq!(pool.resources.busy().unwrap().len(),2,"worker completion is not collection by its owner");drop(owner);assert!(pool.resources.busy().unwrap().is_empty());
    }
    #[test]fn independent_voice_and_recording_readers_share_endpoint_without_sharing_cancellation_or_ownership(){
        let pool=Arc::new(Pool::default());let voice_stop=Arc::new(AtomicBool::new(false));let recording_stop=Arc::new(AtomicBool::new(false));
        let voice=pool.start_guarded("voice",Duration::from_secs(1),Some(voice_stop.clone()),|a|{a.device("same-microphone")?;Ok(())},|_|Ok(())).unwrap();
        let recording=pool.start_guarded("notesMic",Duration::from_secs(1),Some(recording_stop.clone()),|a|{a.device("same-microphone")?;Ok(())},|_|Ok(())).unwrap();assert_eq!(pool.resources.busy().unwrap().len(),4);
        voice_stop.store(true,Ordering::Release);drop(voice);assert!(!recording_stop.load(Ordering::Acquire));assert_eq!(pool.resources.busy().unwrap().len(),2);assert!(ready(&pool,"notesMic","another-device").is_err());drop(recording);pool.reap();assert!(pool.resources.busy().unwrap().is_empty());
    }
}
