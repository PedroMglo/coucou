//! A bounded private snapshot of an explicitly dropped folder, never a live workspace.
use std::{path::{Path,PathBuf},io::{Read,Write}};
const MAX_ENTRIES:usize=256;
const MAX_BYTES:u64=100*1024*1024;
fn private_dir(path:&Path)->std::io::Result<()>{let mut b=std::fs::DirBuilder::new();#[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;b.mode(0o700);}b.create(path)}
fn copy_tree(src:&Path,dst:&Path,base:&Path,depth:usize,count:&mut usize,total:&mut u64)->Result<(),String>{
 if depth>12{return Err("A pasta ultrapassa 12 níveis de profundidade.".into());}
 for entry in std::fs::read_dir(src).map_err(|_|"Não foi possível ler a pasta.")?{
  let entry=entry.map_err(|_|"Não foi possível ler um elemento da pasta.")?;*count+=1;
  if *count>MAX_ENTRIES{return Err("A pasta ultrapassa 256 elementos. Entrega uma pasta menor.".into());}
  let path=entry.path();let meta=std::fs::symlink_metadata(&path).map_err(|_|"Um documento deixou de estar disponível.")?;
  if meta.file_type().is_symlink() || !path.canonicalize().map_err(|_|"Caminho indisponível.")?.starts_with(base){return Err("A pasta contém ligações para outros locais. Entrega apenas documentos normais.".into());}
  let target=dst.join(entry.file_name());
  if meta.is_dir(){private_dir(&target).map_err(|e|e.to_string())?;copy_tree(&path,&target,base,depth+1,count,total)?;}
  else if meta.is_file(){
   if meta.len()>MAX_BYTES.saturating_sub(*total){return Err("Os documentos da pasta excedem 100 MiB.".into());}
   let mut input=std::fs::OpenOptions::new();input.read(true);
   #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;input.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
   let input=input.open(&path).map_err(|_|"Não foi possível copiar um documento.")?;
   if !input.metadata().map_err(|e|e.to_string())?.is_file(){return Err("A pasta contém um elemento incompatível.".into());}
   let mut output=std::fs::OpenOptions::new();output.write(true).create_new(true);
   #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;output.mode(0o600).custom_flags(libc::O_NOFOLLOW);}
   let mut output=output.open(&target).map_err(|e|e.to_string())?;
   *total+=std::io::copy(&mut input.take(MAX_BYTES.saturating_sub(*total)+1),&mut output).map_err(|e|e.to_string())?;
   if *total>MAX_BYTES{return Err("Os documentos da pasta excedem 100 MiB.".into());}output.flush().map_err(|e|e.to_string())?;
  }else{return Err("A pasta contém um elemento que não é um documento normal.".into());}
 }Ok(())
}
pub fn ingest(source:&str)->Result<crate::files::DroppedFile,String>{
 if !crate::files::take_dropped(source){return Err("Entrega a pasta arrastando-a para o pet.".into());}
 let src=Path::new(source);if !std::fs::symlink_metadata(src).is_ok_and(|m|m.is_dir()){return Err("Entrega uma pasta normal, sem ligação simbólica.".into());}
 let base=src.canonicalize().map_err(|e|e.to_string())?;let inbox=crate::files::inbox_dir();
 crate::platform::ensure_private_dir(&crate::settings::local_dir()).map_err(|e|e.to_string())?;crate::platform::ensure_private_dir(&inbox).map_err(|e|e.to_string())?;
 if inbox.canonicalize().map_err(|e|e.to_string())?.starts_with(&base){return Err("Escolhe uma pasta de documentos específica, sem incluir a caixa de entrada dos pets.".into());}
 let name=base.file_name().ok_or("Pasta inválida.")?.to_string_lossy().to_string();let mut dest:Option<PathBuf>=None;
 for i in 1..10000{let p=inbox.join(if i==1{name.clone()}else{format!("{name} ({i})")});match private_dir(&p){Ok(())=>{dest=Some(p);break;},Err(e)if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.to_string())}}
 let dest=dest.ok_or("Não foi possível reservar a pasta do pet.")?;let mut entries=0;let mut size=0;
 if let Err(e)=copy_tree(&base,&dest,&base,0,&mut entries,&mut size){let _=std::fs::remove_dir_all(&dest);return Err(e);}
 Ok(crate::files::DroppedFile{name,path:dest.to_string_lossy().to_string(),size,kind:"folder".into(),entries})
}
#[cfg(test)]mod tests{use super::*;#[test]fn folder_copy_preserves_sources_and_rejects_links_and_excessive_depth(){
 let root=std::env::temp_dir().join(format!("coucou-folders-{}",std::process::id()));let src=root.join("source");let dst=root.join("copy");std::fs::create_dir_all(src.join("nested")).unwrap();std::fs::create_dir_all(&dst).unwrap();std::fs::write(src.join("nested/test.txt"),"PT and English").unwrap();let base=src.canonicalize().unwrap();
 let(mut n,mut size)=(0,0);copy_tree(&src,&dst,&base,0,&mut n,&mut size).unwrap();assert_eq!(n,2);assert_eq!(std::fs::read(dst.join("nested/test.txt")).unwrap(),std::fs::read(src.join("nested/test.txt")).unwrap());assert!(copy_tree(&src,&dst,&base,13,&mut n,&mut size).is_err());
 #[cfg(unix)]{std::os::unix::fs::symlink("/etc",src.join("outside")).unwrap();let other=root.join("rejected");std::fs::create_dir(&other).unwrap();assert!(copy_tree(&src,&other,&base,0,&mut 0,&mut 0).is_err());}
 std::fs::remove_dir_all(root).unwrap();
}}
