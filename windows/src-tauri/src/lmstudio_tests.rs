use super::*;
use std::{io::{Read,Write},net::TcpListener,thread};
fn server(url:&str)->Server{Server{id:"lmstudio",name:"Private LM Studio",url:url.into(),key:None}}
pub(crate)fn metadata()->Value{json!({"models":[{"type":"llm","key":"local/free","display_name":"Private model","publisher":"fixture","format":"gguf","size_bytes":1,"quantization":{"name":"Q4","bits_per_weight":4},"loaded_instances":[{"id":"exact-instance","config":{"context_length":4096}}]}]})}
#[test]fn native_keys_and_loaded_instances_are_distinct_and_model_unload_does_not_change_the_file_binding(){
    let s=server("http://127.0.0.1:1234");let value=metadata();let catalog=Catalog::parse(&s,&value).unwrap();let key=catalog.binding("local/free").unwrap();let instance=catalog.binding("exact-instance").unwrap();assert_eq!(key.model_key,instance.model_key);assert!(key.instance_id.is_none());assert_eq!(instance.instance_id.as_deref(),Some("exact-instance"));assert_ne!(key.model_hash,instance.model_hash);assert!(key.valid_for("local/free"));assert!(!key.valid_for("exact-instance"));
    let mut unloaded=value;unloaded["models"][0]["loaded_instances"]=json!([]);let unloaded=Catalog::parse(&s,&unloaded).unwrap();assert_eq!(unloaded.binding("local/free").unwrap(),key);assert!(unloaded.binding("exact-instance").is_err());assert_eq!(unloaded.models().len(),1);
}
#[test]fn alias_reassignment_endpoint_and_auth_changes_cannot_keep_an_old_engine_binding(){
    let s=server("http://127.0.0.1:1234");let value=metadata();let original=Catalog::parse(&s,&value).unwrap().binding("exact-instance").unwrap();
    let mut moved=value.clone();moved["models"][0]["key"]=json!("other/file");assert_ne!(Catalog::parse(&s,&moved).unwrap().binding("exact-instance").unwrap(),original);
    assert_ne!(Catalog::parse(&server("http://127.0.0.1:1235"),&value).unwrap().binding("exact-instance").unwrap(),original);
    let mut authenticated=server("http://127.0.0.1:1234");authenticated.key=Some("private-test-token-never-a-real-key".into());let binding=Catalog::parse(&authenticated,&value).unwrap().binding("exact-instance").unwrap();assert_ne!(binding,original);assert!(!serde_json::to_string(&binding).unwrap().contains("private-test-token"));
    let mut cosmetic=value;cosmetic["models"][0]["display_name"]=json!("Another label");cosmetic["models"][0]["description"]=json!("Not model identity");assert_eq!(Catalog::parse(&s,&cosmetic).unwrap().binding("exact-instance").unwrap(),original);
}
#[test]fn ambiguity_non_llms_incomplete_metadata_and_remote_servers_do_not_become_ready(){
    let s=server("http://127.0.0.1:1234");let source=metadata();
    for kind in ["embedding","decision"]{let mut value=source.clone();value["models"][0]["type"]=json!(kind);let c=Catalog::parse(&s,&value).unwrap();assert!(c.binding("local/free").is_err());assert!(c.models().is_empty());}
    let mut duplicate=source.clone();duplicate["models"].as_array_mut().unwrap().push(source["models"][0].clone());assert!(Catalog::parse(&s,&duplicate).is_err());
    let mut ambiguous=source.clone();let mut other=source["models"][0].clone();other["key"]=json!("another/file");ambiguous["models"].as_array_mut().unwrap().push(other);let c=Catalog::parse(&s,&ambiguous).unwrap();assert!(c.binding("exact-instance").is_err());assert!(!c.models().iter().any(|m|m.id=="exact-instance"));
    for field in ["format","size_bytes","loaded_instances"]{let mut value=source.clone();value["models"][0].as_object_mut().unwrap().remove(field);assert!(Catalog::parse(&s,&value).is_err());}
    for field in ["remote","remote_host","remote_model"]{let mut value=source.clone();value["models"][0][field]=json!(true);assert!(Catalog::parse(&s,&value).is_err());}
    assert!(Catalog::parse(&s,&json!({"data":[{"id":"name-only"}]})).is_err());assert!(Catalog::parse(&s,&json!({"models":vec![source["models"][0].clone();257]})).is_err());assert!(Catalog::parse(&server("https://example.invalid"),&source).is_err());
}
fn fixture(status:&str,data:&Value)->(Server,thread::JoinHandle<String>){
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();let url=format!("http://{}",listener.local_addr().unwrap());let status=status.to_string();let raw=data.to_string();
    let worker=thread::spawn(move||{let(mut client,_)=listener.accept().unwrap();client.set_read_timeout(Some(Duration::from_secs(3))).unwrap();let mut bytes=vec![];let mut buf=[0u8;1024];loop{let n=client.read(&mut buf).unwrap();assert!(n>0);bytes.extend_from_slice(&buf[..n]);if bytes.windows(4).any(|w|w==b"\r\n\r\n"){break;}}let request=String::from_utf8(bytes).unwrap();write!(client,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{raw}",raw.len()).unwrap();request});(server(&url),worker)
}
fn runtime()->tokio::runtime::Runtime{tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()}
#[test]fn explicit_metadata_uses_one_native_get_and_the_captured_credential_without_inference_or_management_calls(){
    let(mut s,worker)=fixture("200 OK",&metadata());s.key=Some("private-fixture-bearer".into());let binding=runtime().block_on(inspect(&s,"exact-instance",||true)).unwrap();assert_eq!(binding.instance_id.as_deref(),Some("exact-instance"));let request=worker.join().unwrap();assert!(request.starts_with("GET /api/v1/models HTTP/1.1"));assert!(request.to_lowercase().contains("authorization: bearer private-fixture-bearer"));assert!(!request.contains("POST "));
}
#[test]fn rejected_auth_unsupported_native_api_and_cancelled_metadata_do_not_fallback_or_infer(){
    for status in ["401 Unauthorized","403 Forbidden","404 Not Found"]{let(s,worker)=fixture(status,&json!({}));assert!(runtime().block_on(inspect(&s,"local/free",||true)).is_err());assert!(worker.join().unwrap().starts_with("GET /api/v1/models HTTP/1.1"));}
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();let s=server(&format!("http://{}",listener.local_addr().unwrap()));assert!(runtime().block_on(inspect(&s,"local/free",||false)).is_err());assert!(matches!(listener.accept(),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock));
}
