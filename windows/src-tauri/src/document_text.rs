//! Local PDF text extraction; no upload, shell, OCR or unbounded child process.
pub fn pdf(path:&str)->Result<String,String>{
 #[cfg(unix)]{
  use std::{io::Read,os::unix::io::AsRawFd,process::{Command,Stdio},time::{Duration,Instant}};
  let mut child=Command::new("pdftotext").args(["-enc","UTF-8","-f","1","-l","40","-layout",path,"-"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_|"A leitura local de PDF requer pdftotext (poppler-utils).".to_string())?;
  let mut out=child.stdout.take().unwrap();
  unsafe{let fd=out.as_raw_fd();let flags=libc::fcntl(fd,libc::F_GETFL);libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK);}
  let result=(||{
   let mut data=Vec::new();let start=Instant::now();let mut buf=[0;4096];let mut eof=false;
   loop{
    loop{match out.read(&mut buf){Ok(0)=>{eof=true;break;},Ok(n)=>{data.extend_from_slice(&buf[..n]);if data.len()>100000{return Ok(String::from_utf8_lossy(&data[..100000]).into_owned()+"\n[truncated]");}},Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>break,Err(_)=>return Err("Não foi possível ler o conteúdo do PDF.".into())}}
    if let Some(status)=child.try_wait().map_err(|_|"A leitura do PDF foi interrompida.")?.filter(|_|eof){
     if !status.success(){return Err("PDF danificado, protegido ou incompatível. Não foi possível extrair o texto.".into());}
     let text=String::from_utf8_lossy(&data).trim().to_string();
     return if text.is_empty(){Err("O PDF não contém texto extraível. Pode precisar de OCR; não vou inventar o conteúdo.".into())}else{Ok(text)};
    }
    if start.elapsed()>Duration::from_secs(4){return Err("A leitura do PDF excedeu o tempo disponível.".into());}
    std::thread::sleep(Duration::from_millis(10));
   }
  })();
  let _=child.kill();let _=child.wait();result
 }
 #[cfg(not(unix))]{let _=path;Err("Extração local de PDF não disponível nesta plataforma.".into())}
}
