//! Aggregate prompt budgets and bounded regular-file reads, shared by clients.
use std::{io::Read,path::Path,time::{Duration,Instant}};
use serde_json::{json,Value};
pub(crate) const MAX_MEDIA:usize=8*1024*1024;
const MAX_TEXT:usize=256*1024;
pub(crate) struct Budget{text:usize,media:usize,started:Instant}
impl Default for Budget{fn default()->Self{Self{text:0,media:0,started:Instant::now()}}}
impl Budget{
    pub(crate) fn available(&self)->bool{self.started.elapsed()<Duration::from_secs(8)}
    pub(crate) fn text(&mut self,text:String)->String{
        let left=MAX_TEXT.saturating_sub(self.text);
        if text.len()<=left{self.text+=text.len();return text;}
        let mut end=left.saturating_sub(100).min(text.len());while !text.is_char_boundary(end){end-=1;}
        self.text=MAX_TEXT;format!("{}\n[Partial attachment context: combined text limit reached; missing content is unknown.]",&text[..end])
    }
    pub(crate) fn block(&mut self,part:Value,name:&str)->Value{
        if part["type"]=="text"{return json!({"type":"text","text":self.text(part["text"].as_str().unwrap_or("").into())});}
        let size=serde_json::to_vec(&part).map_or(MAX_MEDIA+1,|v|v.len());
        if self.media.saturating_add(size)>MAX_MEDIA{return unavailable(name,"combined media limit");}
        self.media+=size;part
    }
}
pub(crate) fn unavailable(name:&str,reason:&str)->Value{json!({"type":"text","text":format!("Attachment {name:?}: content unavailable ({reason}). Do not infer its contents or treat this as an empty document.")})}

pub(crate) fn bytes(path:&str,limit:usize)->Option<Vec<u8>>{
    read(path,limit,false)
}
fn read(path:&str,limit:usize,prefix:bool)->Option<Vec<u8>>{
    let mut options=std::fs::OpenOptions::new();options.read(true);
    #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
    let file=options.open(path).ok()?;let before=file.metadata().ok()?;
    if !before.is_file()||before.len()>if prefix{super::MAX_CONVERSATION_BYTES}else{limit as u64}{return None;}
    let fingerprint=|m:&std::fs::Metadata|{
        #[cfg(unix)] {use std::os::unix::fs::MetadataExt;(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec())}
        #[cfg(not(unix))] {(m.len(),m.modified().ok())}
    };
    let before=fingerprint(&before);let mut data=vec![];(&file).take(limit as u64+1).read_to_end(&mut data).ok()?;
    if !prefix&&data.len()>limit||fingerprint(&file.metadata().ok()?)!=before{return None;}
    let after=std::fs::symlink_metadata(Path::new(path)).ok()?;
    if !after.is_file()||fingerprint(&after)!=before{return None;}data.truncate(limit);Some(data)
}
fn decode(mut data:Vec<u8>,prefix:bool)->Option<String>{
    if data.iter().any(|b|matches!(*b,0..=8|11|14..=31|127)){return None;}
    match String::from_utf8(data){
        Ok(text)=>Some(text),
        Err(e)if prefix&&e.utf8_error().error_len().is_none()=>{let valid=e.utf8_error().valid_up_to();data=e.into_bytes();data.truncate(valid);String::from_utf8(data).ok()},
        Err(_)=>None,
    }
}
pub(crate) fn text(path:&str,limit:usize)->Option<String>{decode(bytes(path,limit)?,false)}
pub(crate) fn text_prefix(path:&str,limit:usize)->Option<String>{decode(read(path,limit,true)?,true)}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn text_prefix_keeps_its_byte_bound_and_utf8_boundary_without_following_links(){
        let files=crate::pet_documents::fixtures::Files::new();files.write("long.txt","é".repeat(100).as_bytes());let path=files.root.join("long.txt");let path=path.to_str().unwrap();
        assert_eq!(text_prefix(path,5),Some("éé".into()));assert!(text(path,5).is_none());
        #[cfg(unix)]{let link=files.root.join("link.txt");std::os::unix::fs::symlink(path,&link).unwrap();assert!(text_prefix(link.to_str().unwrap(),50).is_none());}
        files.write("incomplete.txt",&[b'a',0xc3]);let path=files.root.join("incomplete.txt");assert_eq!(text_prefix(path.to_str().unwrap(),2),Some("a".into()));assert!(text(path.to_str().unwrap(),2).is_none());
    }
    #[test]
    fn binary_control_bytes_are_not_reported_as_decoded_text(){
        let files=crate::pet_documents::fixtures::Files::new();files.write("binary.txt",b"seemingly readable\0binary payload");files.write("text.md",b"normal\ttext\nsecond line");
        assert!(text(files.root.join("binary.txt").to_str().unwrap(),100).is_none());assert!(text_prefix(files.root.join("binary.txt").to_str().unwrap(),100).is_none());assert_eq!(text(files.root.join("text.md").to_str().unwrap(),100),Some("normal\ttext\nsecond line".into()));
    }
    #[test]
    fn text_and_media_budgets_are_aggregate_and_never_break_utf8_or_binary_blocks(){
        let mut budget=Budget::default();let first=budget.text("a".repeat(MAX_TEXT-10));assert_eq!(first.len(),MAX_TEXT-10);let clipped=budget.text("é".repeat(100));assert!(clipped.contains("Partial attachment context"));assert!(clipped.is_char_boundary(clipped.len()));
        let image=json!({"type":"image_url","image_url":{"url":"x".repeat(MAX_MEDIA/2)}});assert_eq!(budget.block(image.clone(),"first"),image);let next=budget.block(image,"second");assert_eq!(next["type"],"text");assert!(next["text"].as_str().unwrap().contains("combined media limit"));
    }
    #[test]
    fn regular_file_reads_are_bounded_and_do_not_follow_links(){
        let dir=std::env::temp_dir().join(format!("coucou-context-bytes-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&dir).unwrap();let file=dir.join("fixture");std::fs::write(&file,b"hello").unwrap();assert_eq!(bytes(file.to_str().unwrap(),5).unwrap(),b"hello");assert!(bytes(file.to_str().unwrap(),4).is_none());assert!(bytes(dir.to_str().unwrap(),100).is_none());
        #[cfg(unix)] {std::os::unix::fs::symlink(&file,dir.join("link")).unwrap();assert!(bytes(dir.join("link").to_str().unwrap(),100).is_none());}
        std::fs::remove_dir_all(dir).unwrap();
    }
}
