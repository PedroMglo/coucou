//! A trusted driver's mechanical lifetime. The hold is not caller authority;
//! every exchange supplies its own deadline and authority, with no retry.
use super::*;
pub(crate)const MAX_RESIDENT_INPUT:usize=8*1024*1024+1025;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate)enum ExchangeError{NotStarted,Uncertain}
pub(crate)struct Resident{
    // Field order keeps the physical hold until OwnedChild's family is reaped.
    owned:OwnedChild,stdin:Box<dyn Write+Send>,stdout:Box<dyn Read+Send>,_hold:Arc<Hold>,poisoned:bool,
}
fn current(deadline:Instant,authority:&dyn Fn()->bool)->bool{
    if Instant::now()>=deadline{return false;}
    let allowed=authority();allowed&&Instant::now()<deadline
}
impl Resident{
    pub(crate)fn start(lane:Lane,command:Command,deadline:Instant,authority:&dyn Fn()->bool)->Result<Self,ExchangeError>{
        Self::start_on(resources(),lane,command,deadline,authority)
    }
    fn start_on(resources:Arc<Coordinator>,lane:Lane,mut command:Command,deadline:Instant,authority:&dyn Fn()->bool)->Result<Self,ExchangeError>{
        if !current(deadline,authority){return Err(ExchangeError::NotStarted);}
        let(prefix,slots)=lane.slots();let cancel=Arc::new(AtomicBool::new(false));
        let hold=(0..slots).find_map(|n|resources.acquire(&format!("{prefix}_{n}"),cancel.clone(),None).ok()).ok_or(ExchangeError::NotStarted)?;
        if !current(deadline,authority){return Err(ExchangeError::NotStarted);}
        command.stdin(Stdio::piped());
        let mut owned=OwnedChild::new(command.spawn().map_err(|_|ExchangeError::NotStarted)?);
        let stdin=owned.child.stdin.take().ok_or(ExchangeError::NotStarted)?;
        let stdout=owned.child.stdout.take().ok_or(ExchangeError::NotStarted)?;
        nonblocking(stdin.as_raw_fd()).map_err(|_|ExchangeError::NotStarted)?;
        nonblocking(stdout.as_raw_fd()).map_err(|_|ExchangeError::NotStarted)?;
        Ok(Self{owned,stdin:Box::new(stdin),stdout:Box::new(stdout),_hold:hold,poisoned:false})
    }
    /// A private native driver may use one Unix stream instead of stdio.
    /// Only the exact supervised process group can connect. This is mechanics,
    /// never authority; the caller still binds its typed protocol/session.
    pub(crate)fn start_socket(lane:Lane,command:Command,listener:&std::os::unix::net::UnixListener,deadline:Instant,authority:&dyn Fn()->bool)->Result<Self,ExchangeError>{
        Self::start_socket_on(resources(),lane,command,listener,deadline,authority)
    }
    fn start_socket_on(resources:Arc<Coordinator>,lane:Lane,mut command:Command,listener:&std::os::unix::net::UnixListener,deadline:Instant,authority:&dyn Fn()->bool)->Result<Self,ExchangeError>{
        if !current(deadline,authority){return Err(ExchangeError::NotStarted);}
        let(prefix,slots)=lane.slots();let cancel=Arc::new(AtomicBool::new(false));
        let hold=(0..slots).find_map(|n|resources.acquire(&format!("{prefix}_{n}"),cancel.clone(),None).ok()).ok_or(ExchangeError::NotStarted)?;
        listener.set_nonblocking(true).map_err(|_|ExchangeError::NotStarted)?;
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        if !current(deadline,authority){return Err(ExchangeError::NotStarted);}
        // Local order is deliberate too: cleanup reaps the child before hold.
        let owned=OwnedChild::new(command.spawn().map_err(|_|ExchangeError::NotStarted)?);
        loop{
            if !current(deadline,authority)||owned.exited().map_err(|_|ExchangeError::NotStarted)?{return Err(ExchangeError::NotStarted);}
            match listener.accept(){
                Ok((stream,_))=>{
                    let mut cred=libc::ucred{pid:0,uid:0,gid:0};let mut len=std::mem::size_of::<libc::ucred>()as libc::socklen_t;
                    let valid=unsafe{libc::getsockopt(stream.as_raw_fd(),libc::SOL_SOCKET,libc::SO_PEERCRED,(&mut cred as*mut libc::ucred).cast(),&mut len)}==0
                        &&len as usize==std::mem::size_of::<libc::ucred>()&&cred.pid>0
                        &&cred.uid==unsafe{libc::getuid()}&&unsafe{libc::getpgid(cred.pid)}==owned.child.id()as i32;
                    if !valid||!current(deadline,authority){return Err(ExchangeError::NotStarted);}
                    stream.set_nonblocking(true).map_err(|_|ExchangeError::NotStarted)?;
                    let writer=stream.try_clone().map_err(|_|ExchangeError::NotStarted)?;
                    return Ok(Self{owned,stdin:Box::new(writer),stdout:Box::new(stream),_hold:hold,poisoned:false});
                },
                Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>std::thread::sleep(Duration::from_millis(5)),
                Err(_)=>return Err(ExchangeError::NotStarted),
            }
        }
    }
    #[cfg(test)]pub(crate)fn test_process_id(&self)->u32{self.owned.child.id()}
    pub(crate)fn alive(&self)->bool{self.owned.exited().is_ok_and(|exited|!exited)}
    /// An error after any input is sent makes the stream unusable: the caller
    /// must drop this owner before admitting another request. No EOF is needed
    /// for a successful line, so the driver can keep serving its selected data.
    pub(crate)fn exchange_line(&mut self,input:&[u8],limit:usize,deadline:Instant,authority:&dyn Fn()->bool)->Result<Vec<u8>,ExchangeError>{
        if self.poisoned{return Err(ExchangeError::NotStarted);}
        let result=self.exchange_inner(input,limit,deadline,authority);
        if result==Err(ExchangeError::Uncertain){self.poisoned=true;}
        result
    }
    fn exchange_inner(&mut self,input:&[u8],limit:usize,deadline:Instant,authority:&dyn Fn()->bool)->Result<Vec<u8>,ExchangeError>{
        if input.is_empty()||input.len()>MAX_RESIDENT_INPUT||limit==0||limit>64*1024||!current(deadline,authority){return Err(ExchangeError::NotStarted);}
        let mut written=0;let mut output=Vec::new();
        loop{
            let fail=if written==0{ExchangeError::NotStarted}else{ExchangeError::Uncertain};
            if !current(deadline,authority)||!self.alive(){return Err(fail);}
            let mut progressed=false;
            if written<input.len(){match self.stdin.write(&input[written..]){
                Ok(0)=>return Err(fail),Ok(n)=>{written+=n;progressed=true;},
                Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>{},Err(_)=>return Err(fail),
            }}
            #[cfg(test)]if written==input.len(){crate::pet_runtime::system_effect_crash_tests::commit_written(input);}
            let mut buffer=[0;4096];
            match self.stdout.read(&mut buffer){
                Ok(0)=>return Err(if written==0{ExchangeError::NotStarted}else{ExchangeError::Uncertain}),
                Ok(n)=>{progressed=true;output.extend_from_slice(&buffer[..n]);if output.len()>limit{return Err(ExchangeError::Uncertain);}},
                Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>{},Err(_)=>return Err(if written==0{ExchangeError::NotStarted}else{ExchangeError::Uncertain}),
            }
            if let Some(at)=output.iter().position(|b|*b==b'\n'){
                if at+1!=output.len()||written!=input.len(){return Err(ExchangeError::Uncertain);}
                if !current(deadline,authority){return Err(ExchangeError::Uncertain);}
                return Ok(output);
            }
            if !progressed{std::thread::sleep(Duration::from_millis(5));}
        }
    }
}
#[cfg(test)]#[path="pet_helper_resident_tests.rs"]mod tests;
