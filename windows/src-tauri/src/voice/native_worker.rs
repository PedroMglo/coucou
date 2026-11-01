//! Fixed native speech helpers, executed through the existing resource/family
//! supervisor. Availability is not a grant or proof of audible playback.
#[cfg(target_os="linux")]use super::tts::Current;
#[cfg(target_os="linux")]use std::{fs::File,path::PathBuf,sync::Arc,os::{fd::AsRawFd,unix::fs::MetadataExt},io::Read,time::Duration};

#[cfg(target_os="linux")]
#[cfg(target_os="linux")]
#[derive(Clone,PartialEq,Eq)]struct Stamp{dev:u64,ino:u64,len:u64,mode:u32,mtime:i64,mtime_ns:i64,ctime:i64,ctime_ns:i64}
#[cfg(target_os="linux")]
impl Stamp{fn of(m:&std::fs::Metadata)->Self{Self{dev:m.dev(),ino:m.ino(),len:m.len(),mode:m.mode(),mtime:m.mtime(),mtime_ns:m.mtime_nsec(),ctime:m.ctime(),ctime_ns:m.ctime_nsec()}}}
#[derive(Clone)]pub(super) struct Plan{
    #[cfg(target_os="linux")]file:Arc<File>,#[cfg(target_os="linux")]path:PathBuf,
    #[cfg(target_os="linux")]stamp:Stamp,
}
impl Plan{
    pub(super) fn capture()->Option<Self>{
        #[cfg(target_os="linux")]{
            // Selection occurs before the renderer/worker wait, never after a
            // failed dispatch. No helper is invoked by this passive resolution.
            // spd-say delegates to a daemon outside this owned family. Until
            // message-scoped SSIP cancellation is implemented, do not start
            // that unsupported backend and leave the response as text.
            Self::open(crate::platform::find_on_path("espeak-ng")?)
        }
        #[cfg(not(target_os="linux"))]{None}
    }
    #[cfg(target_os="linux")]
    fn open(path:PathBuf)->Option<Self>{
        let path=path.canonicalize().ok()?;let mut file=File::open(&path).ok()?;let meta=file.metadata().ok()?;let mut magic=[0;4];file.read_exact(&mut magic).ok()?;
        if !meta.is_file()||meta.mode()&0o111==0||magic!=*b"\x7fELF"{return None;}
        let p=Self{path,file:Arc::new(file),stamp:Stamp::of(&meta)};p.current().then_some(p)
    }
    #[cfg(target_os="linux")]fn current(&self)->bool{
        self.file.metadata().is_ok_and(|m|Stamp::of(&m)==self.stamp)&&std::fs::metadata(&self.path).is_ok_and(|m|Stamp::of(&m)==self.stamp)
    }
    #[cfg(target_os="linux")]
    fn command(&self,text:&str,language:Option<&str>)->(std::process::Command,Option<Vec<u8>>){
        let mut args=vec![];let lang=language.map(|s|s.split('-').next().unwrap_or(s));
        args.push("--stdin".into());if let Some(lang)=lang{args.extend(["-v".into(),lang.into()]);}
        let input=Some(text.as_bytes().to_vec());
        // Linux resolves the retained ELF before CLOEXEC closes this fd. A
        // same-named replacement cannot become a different executable here.
        (crate::pet_helper::command(&format!("/proc/self/fd/{}",self.file.as_raw_fd()),&args),input)
    }
}

pub(super) async fn run(call:super::native::Invocation,plan:Plan)->bool{
    #[cfg(target_os="linux")]{
        if !call.current()||!plan.current(){return false;}
        let timeout=call.remaining().min(Duration::from_secs(150));
        let Ok(session)=crate::pet_helper::Session::acquire(crate::pet_helper::Lane::SpeechOutput,timeout)else{return false;};
        let (command,input)=plan.command(call.text(),call.language());let source=call.guard();
        let current:Current=Arc::new(move||source()&&plan.current());let before=current.clone();
        crate::net::while_current(||current(),||session.run_input(command,input,1024,timeout,
            move||if before(){Ok(())}else{Err(crate::net::CANCELLED.into())},current.clone())).await.is_ok()
    }
    #[cfg(not(target_os="linux"))]{let _=(call,plan);false}
}

#[cfg(all(test,target_os="linux"))]mod tests{
    use super::*;
    #[test]fn native_speech_helper_content_cannot_be_an_option_or_pipe_command(){
        let plan=Plan::open("/usr/bin/true".into()).unwrap();let(command,input)=plan.command("-C\n!-!CANCEL ALL",Some("pt-PT"));assert_eq!(input.unwrap(),b"-C\n!-!CANCEL ALL");assert!(!command.get_args().any(|s|s.to_string_lossy().contains("CANCEL")));
    }
    #[tokio::test]async fn native_speech_retained_elf_executes_only_a_private_non_audio_fixture(){
        let plan=Plan::open("/usr/bin/cat".into()).unwrap();let current=plan.clone();
        let s=crate::pet_helper::Session::acquire(crate::pet_helper::Lane::SpeechOutput,Duration::from_secs(2)).unwrap();
        let command=crate::pet_helper::command(&format!("/proc/self/fd/{}",plan.file.as_raw_fd()),&[]);
        let guard:Current=Arc::new(move||current.current());let before=guard.clone();
        let out=s.run_input(command,Some(b"private fixture only".to_vec()),64,Duration::from_secs(1),move||if before(){Ok(())}else{Err("revoked".into())},guard).await.unwrap();assert_eq!(out,b"private fixture only");
    }
}
