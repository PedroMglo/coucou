//! Bounded mechanics for the fixed audio/DMS/speech helpers, below typed authority.
//! No renderer/model command entry point and no LLM RunPermit construction.
use std::{io::{Read,Write}, os::{fd::AsRawFd, unix::process::CommandExt}, process::{Command,Stdio},
    sync::{Arc,OnceLock,atomic::{AtomicBool,Ordering}}, time::{Duration,Instant}};
use crate::{pet_process_group::OwnedChild,pet_resources::{Coordinator,Hold}};

#[derive(Clone,Copy)] pub(crate) enum Lane { AudioEffect, AudioRead, DmsEffect, DmsRead, DmsRelease, SpeechOutput }
impl Lane {
    fn slots(self) -> (&'static str,usize) { match self {
        Self::AudioEffect=>("system_audio_effect",1), Self::AudioRead=>("system_audio_read",16),
        Self::DmsEffect=>("system_dms_effect",1), Self::DmsRead=>("system_dms_read",2),
        Self::DmsRelease=>("system_dms_release",1), Self::SpeechOutput=>("voice_native_output",1),
    }}
}
fn resources()->Arc<Coordinator> {
    static RESOURCES:OnceLock<Arc<Coordinator>>=OnceLock::new();
    RESOURCES.get_or_init(Arc::default).clone()
}
pub(crate) struct Session { hold:Arc<Hold>, cancel:Arc<AtomicBool>, deadline:Instant }
impl Session {
    pub(crate) fn acquire(lane:Lane,timeout:Duration)->Result<Self,String>{Self::acquire_on(resources(),lane,timeout)}
    fn acquire_on(resources:Arc<Coordinator>,lane:Lane,timeout:Duration)->Result<Self,String>{
        let deadline=Instant::now().checked_add(timeout).ok_or("Prazo inválido do serviço do sistema.")?;
        let cancel=Arc::new(AtomicBool::new(false));let(prefix,slots)=lane.slots();
        for n in 0..slots {
            match resources.acquire(&format!("{prefix}_{n}"),cancel.clone(),Some(deadline)) {
                Ok(hold)=>return Ok(Self{hold,cancel,deadline}),
                Err(crate::pet_resources::Failure::Busy)=>continue,
                Err(_)=>return Err("O serviço do sistema não está disponível para outro pedido.".into()),
            }
        }
        Err("O serviço do sistema ainda está ocupado ou a terminar o pedido anterior.".into())
    }
    pub(crate) async fn run(&self,command:Command,limit:usize,timeout:Duration,
        before_spawn:impl FnOnce()->Result<(),String>+Send+'static)->Result<Vec<u8>,String>{
        self.run_owned(command,None,limit,timeout,before_spawn,Arc::new(||true)).await
    }
    /// Bounded input for trusted typed helpers. No shell/renderer API. The
    /// worker retains its physical hold through cancellation and family reap.
    pub(crate) async fn run_input(&self,command:Command,input:Option<Vec<u8>>,limit:usize,timeout:Duration,
        before_spawn:impl FnOnce()->Result<(),String>+Send+'static,current:Arc<dyn Fn()->bool+Send+Sync>,
    )->Result<Vec<u8>,String>{
        if input.as_ref().is_some_and(|b|b.len()>64*1024){return Err("O conteúdo do serviço excede o limite.".into());}
        self.run_owned(command,input,limit,timeout,before_spawn,current).await
    }
    async fn run_owned(&self,mut command:Command,input:Option<Vec<u8>>,limit:usize,timeout:Duration,
        before_spawn:impl FnOnce()->Result<(),String>+Send+'static,current:Arc<dyn Fn()->bool+Send+Sync>,
    )->Result<Vec<u8>,String>{
        command.stdin(if input.is_some(){Stdio::piped()}else{Stdio::null()});
        let deadline=self.deadline.min(Instant::now()+timeout);
        let hold=self.hold.clone();let cancel=self.cancel.clone();
        // Dropping this future cancels its worker but cannot drop the worker's
        // hold or Child. No Tokio abort/timeout releases physical ownership.
        struct Cancel(Option<Arc<AtomicBool>>);
        impl Drop for Cancel{fn drop(&mut self){if let Some(flag)=&self.0{flag.store(true,Ordering::Release);}}}
        let mut guard=Cancel(Some(cancel.clone()));
        let (tx,rx)=tokio::sync::oneshot::channel();
        let _worker=tokio::task::spawn_blocking(move||{
            let _hold=hold;
            let mut owned=None;
            let result=supervise(&mut command,input,limit,deadline,&cancel,before_spawn,&current,&mut owned);
            // Errors are uncertain immediately; cleanup may need longer than
            // the UI deadline. The worker keeps its hold throughout Drop.
            let _=tx.send(result);
            drop(owned);
        });
        let result=tokio::select! {
            result=rx=>result.map_err(|_|"A recolha do serviço do sistema não pôde ser confirmada. Consulta o estado antes de outro pedido.".to_string())?,
            _=tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))=>return Err("O serviço do sistema excedeu o prazo. O resultado pode ser incerto; a recolha mantém o recurso ocupado. Consulta o estado antes de outro pedido.".into()),
        };
        guard.0=None; // Completed worker; do not cancel a later readback on this same session.
        result
    }
}
impl Drop for Session { fn drop(&mut self){self.cancel.store(true,Ordering::Release);} }

pub(crate) fn command(program:&str,args:&[String])->Command{
    let mut command=Command::new(program);
    command.args(args).env("LC_ALL","C").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).process_group(0);
    command
}
fn supervise(command:&mut Command,input:Option<Vec<u8>>,limit:usize,deadline:Instant,cancel:&AtomicBool,
    before_spawn:impl FnOnce()->Result<(),String>,authority:&Arc<dyn Fn()->bool+Send+Sync>,owned:&mut Option<OwnedChild>)->Result<Vec<u8>,String>{
    let current=||!cancel.load(Ordering::Acquire)&&Instant::now()<deadline&&authority();
    if !current(){return Err("O pedido terminou antes de iniciar o serviço do sistema.".into());}
    before_spawn()?;
    if !current(){return Err("O pedido terminou antes de iniciar o serviço do sistema.".into());}
    *owned=Some(OwnedChild::new(command.spawn().map_err(|_|"Não foi possível iniciar o serviço do sistema.".to_string())?));
    let child=owned.as_mut().unwrap();
    // From here every error is uncertain: the daemon may already have received
    // the request. OwnedChild's drop finishes cleanup before the hold is freed.
    let uncertain=||"O serviço do sistema não confirmou o pedido. Uma alteração pode ter ocorrido; consulta o estado antes de outro pedido.".to_string();
    let mut stdout=child.child.stdout.take().ok_or_else(uncertain)?;
    let flags=unsafe{libc::fcntl(stdout.as_raw_fd(),libc::F_GETFL)};
    if flags<0||unsafe{libc::fcntl(stdout.as_raw_fd(),libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{return Err(uncertain());}
    let mut stdin=child.child.stdin.take();let input=input.unwrap_or_default();let mut written=0;
    if let Some(pipe)=&stdin{let flags=unsafe{libc::fcntl(pipe.as_raw_fd(),libc::F_GETFL)};if flags<0||unsafe{libc::fcntl(pipe.as_raw_fd(),libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{return Err(uncertain());}}
    let mut bytes=Vec::new();let mut eof=false;
    loop {
        if !current(){return Err(uncertain());}
        if written==input.len(){drop(stdin.take());}
        if let Some(pipe)=&mut stdin{
            match pipe.write(&input[written..]){
                Ok(0)=>return Err(uncertain()),Ok(n)=>written+=n,
                Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>{},
                Err(_)=>return Err(uncertain()),
            }
        }
        if !eof {
            let mut buffer=[0;4096];let cap=(limit.saturating_add(1)-bytes.len()).min(buffer.len());
            match stdout.read(&mut buffer[..cap]) {
                Ok(0)=>eof=true,
                Ok(n)=>{bytes.extend_from_slice(&buffer[..n]);if bytes.len()>limit{return Err(uncertain());}},
                Err(e) if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>{},
                Err(_)=>return Err(uncertain()),
            }
        }
        if eof&&child.exited().map_err(|_|uncertain())? {
            // No post-spawn authority check can erase an observed result.
            let status=child.stop_and_reap().map_err(|_|uncertain())?;
            return if status.success()&&written==input.len(){Ok(bytes)}else{Err(uncertain())};
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)] #[path="pet_helper_tests.rs"] mod tests;
