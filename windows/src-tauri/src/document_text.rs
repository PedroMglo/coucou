//! Local PDF text extraction; no upload, shell, OCR or unbounded child process.
#[cfg(unix)]
fn with_input(path:&str,run:impl FnOnce(std::fs::File)->Result<String,String>)->Result<String,String>{
 use std::os::unix::fs::{MetadataExt,OpenOptionsExt};
 let error=||"A cópia do PDF está indisponível ou mudou; o resultado foi descartado.".to_string();
 let file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC).open(path).map_err(|_|error())?;
 let meta=file.metadata().map_err(|_|error())?;if !meta.is_file()||meta.len()>100*1024*1024{return Err(error());}
 let stamp=|m:&std::fs::Metadata|(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec(),m.nlink());let before=stamp(&meta);
 let result=run(file.try_clone().map_err(|_|error())?);
 if !file.metadata().is_ok_and(|m|m.is_file()&&stamp(&m)==before)||!std::fs::symlink_metadata(path).is_ok_and(|m|m.is_file()&&stamp(&m)==before){return Err(error());}result
}
pub fn pdf(path:&str)->Result<String,String>{
 #[cfg(unix)]{with_input(path,|input|{
  use std::{io::Read,os::unix::io::AsRawFd,process::{Command,Stdio},time::{Duration,Instant}};
  let mut child=Command::new("pdftotext").args(["-enc","UTF-8","-f","1","-l","40","-layout","-","-"]).stdin(Stdio::from(input)).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_|"A leitura local de PDF requer pdftotext (poppler-utils).".to_string())?;
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
  let _=child.kill();let _=child.wait();result.map(|text|format!("[PDF text extraction: at most the first 40 pages, text layer only, no OCR. Completeness beyond this extraction is unknown.]\n{text}"))
 }) }
 #[cfg(not(unix))]{let _=path;Err("Extração local de PDF não disponível nesta plataforma.".into())}
}

#[cfg(all(test,unix))]
mod tests{
 use super::*;use std::io::Read;
 #[test]
 fn pdf_input_is_a_bounded_pinned_regular_file_and_never_a_link_or_directory(){
  let files=crate::pet_documents::fixtures::Files::new();files.write("source.pdf",b"pinned bytes");let path=files.root.join("source.pdf");
  let result=with_input(path.to_str().unwrap(),|mut file|{let mut text=String::new();file.read_to_string(&mut text).unwrap();Ok(text)}).unwrap();assert_eq!(result,"pinned bytes");
  std::os::unix::fs::symlink(&path,files.root.join("link.pdf")).unwrap();assert!(with_input(files.root.join("link.pdf").to_str().unwrap(),|_|panic!("Link must not reach the extractor")).is_err());assert!(with_input(files.root.to_str().unwrap(),|_|panic!("Directory must not reach extractor")).is_err());
  std::fs::OpenOptions::new().write(true).open(&path).unwrap().set_len(100*1024*1024+1).unwrap();assert!(with_input(path.to_str().unwrap(),|_|panic!("Over-limit PDF")).is_err());
 }
 #[test]
 fn pdf_replacement_or_mutation_during_extraction_discards_the_result(){
  let files=crate::pet_documents::fixtures::Files::new();files.write("source.pdf",b"before");let path=files.root.join("source.pdf");
  assert!(with_input(path.to_str().unwrap(),|_|{std::fs::write(&path,b"after!").unwrap();Ok("discard this result".into())}).is_err());
  files.write("replacement.pdf",b"after!");assert!(with_input(path.to_str().unwrap(),|_|{std::fs::rename(files.root.join("replacement.pdf"),&path).unwrap();Ok("discard again".into())}).is_err());
 }
 #[test]
 fn real_pdf_extraction_reads_the_pinned_stdin_without_inventing_ocr(){
  let files=crate::pet_documents::fixtures::Files::new();let stream="BT /F1 12 Tf 72 720 Td (Pinned PDF sentinel) Tj ET";
  let objects=vec!["<< /Type /Catalog /Pages 2 0 R >>".to_string(),"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".into(),"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),format!("<< /Length {} >>\nstream\n{stream}\nendstream",stream.len())];
  let mut data=String::from("%PDF-1.4\n");let mut offsets=vec![0];for(i,object)in objects.iter().enumerate(){offsets.push(data.len());data.push_str(&format!("{} 0 obj\n{object}\nendobj\n",i+1));}
  let xref=data.len();data.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n",offsets.len()));for offset in offsets.iter().skip(1){data.push_str(&format!("{offset:010} 00000 n \n"));}data.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",offsets.len()));
  files.write("synthetic.pdf",data.as_bytes());let result=pdf(files.root.join("synthetic.pdf").to_str().unwrap());
  let text=result.expect("The project test image must include pdftotext, declared in linux/Containerfile.fedora");assert!(text.contains("Pinned PDF sentinel"));assert!(text.contains("first 40 pages"));assert!(text.contains("no OCR"));
 }
}
