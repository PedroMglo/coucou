//! The original native drain proof survives publication/ACK uncertainty. Only
//! cleanup is retried; no model, tool, approval or lease can be recreated here.
use super::*;
pub(super)struct Closing{
    proof:DrainProof,phase:Phase,round:u8,reason:Option<String>,completion:Option<Record>,
    authority:Mutex<Option<Authority>>,acknowledged:AtomicBool,released:AtomicBool,retired:AtomicBool,
}
impl Closing{
    fn attempt(&self,driver:&Driver)->model::Result<()>{
        let mut source=self.authority.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if source.is_none(){*source=Some(driver.authority()?);}
        let authority=source.as_ref().ok_or(model::Error::StorageUnavailable)?;
        self.proof.original_authority(authority)?;
        if !self.acknowledged.load(Ordering::Acquire){
            if let Some(goal)=&self.completion{authority.verify_completed_goal(goal).map_err(crate::pet_runtime::authority::Failure::control_error)?;}
            let receipt=driver.store.drain_owned(authority,&self.proof,crate::assistant_memory::now())?;
            if receipt.status==CommitStatus::Uncertain{return Err(model::Error::StorageUnavailable);}
            self.acknowledged.store(true,Ordering::Release);
        }
        if !self.released.load(Ordering::Acquire){
            driver.owner.live.hold.release_drained(&self.proof).map_err(|_|model::Error::Conflict)?;
            self.released.store(true,Ordering::Release);
        }
        if !self.retired.load(Ordering::Acquire){driver.environment.drained(&self.proof)?;self.retired.store(true,Ordering::Release);}
        Ok(())
    }
}
impl Driver{
    fn prepare_closure(&self,phase:Phase,round:u8,reason:Option<&str>,completion:Option<Record>)->model::Result<Arc<Closing>>{
        let mut state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;
        if let Some(closing)=&state.closing{return Ok(closing.clone());}
        let proof=self.owner.drain(DrainOutcome::Confirmed)?;
        let closing=Arc::new(Closing{proof,phase,round,reason:reason.map(str::to_owned),completion,authority:Mutex::new(None),acknowledged:AtomicBool::new(false),released:AtomicBool::new(false),retired:AtomicBool::new(false)});
        state.closing=Some(closing.clone());Ok(closing)
    }
    pub(super)async fn resume_closure(&self,closing:Arc<Closing>)->model::Result<()>{
        self.notice(Phase::Closing,closing.round,Some("IO terminado; a confirmar o fecho original, sem repetir chamadas ou ações."))?;
        let until=Instant::now()+Duration::from_secs(5);let mut delay=Duration::from_millis(25);let mut last_notice=Instant::now();
        loop{
            match closing.attempt(self){
                Ok(())=>{
                    let mut state=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?;
                    if !state.closing.as_ref().is_some_and(|c|Arc::ptr_eq(c,&closing)){return Err(model::Error::Conflict);}
                    state.closing=None;state.awaiting=None;state.retained=None;drop(state);
                    return self.notice(closing.phase,closing.round,closing.reason.as_deref());
                }
                Err(error@ (model::Error::Busy|model::Error::StorageUnavailable))=>{
                    if Instant::now()>=until{
                        self.notice(Phase::Closing,closing.round,Some("O fecho original aguarda confirmação; a prova foi conservada para repetir só o fecho."))?;return Err(error);
                    }
                    if last_notice.elapsed()>=Duration::from_secs(1){self.notice(Phase::Closing,closing.round,Some("A aguardar o armazenamento do fecho original; não foram repetidos efeitos."))?;last_notice=Instant::now();}
                    tokio::time::sleep(delay).await;delay=(delay*2).min(Duration::from_millis(200));
                }
                Err(error)=>{self.notice(Phase::RecoveryRequired,closing.round,Some("A origem do fecho mudou ou não pode ser validada; a prova original permanece retida."))?;return Err(error);}
            }
        }
    }
    pub(super)async fn finish(&self,phase:Phase,round:u8,reason:Option<&str>)->model::Result<()>{
        let closing=self.prepare_closure(phase,round,reason,None)?;self.resume_closure(closing).await
    }
    pub(super)async fn close_completed(&self,goal:&Record,round:u8)->model::Result<()>{
        let closing=self.prepare_closure(Phase::Completed,round,None,Some(goal.clone()))?;self.resume_closure(closing).await
    }
}
#[cfg(test)]#[path="driver_closure_tests.rs"]mod tests;
