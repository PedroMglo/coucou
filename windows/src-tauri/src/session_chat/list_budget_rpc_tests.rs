// Included in rpc::endpoint_tests for access to the existing in-memory queued client.
// No socket, native daemon, account or inference is used by this fixture.
#[tokio::test]
async fn list_budget_deadline_drops_real_rpc_pending_without_interrupting_a_turn(){
 let (client,mut packets)=queued_client();let client=Arc::new(client);let pending=client.pending.clone();
 let calls=Arc::new(Mutex::new(Vec::<String>::new()));let observed=calls.clone();
 let writer=tokio::spawn(async move{
  while let Some(packet)=packets.recv().await{
   if packet.expired(){continue;}
   let method=packet.message["method"].as_str().unwrap().to_string();observed.lock().unwrap().push(method.clone());
   let _=packet.written.send(Ok(()));
   if method=="fixture/barrier"{
    let id=packet.message["id"].as_str().unwrap();let entry=pending.lock().unwrap().remove(id);
    if let Some(entry)=entry{let _=entry.answer.send(Ok(json!({"stillConnected":true})));}
   }
   // List requests are acknowledged but deliberately left without a response.
  }
 });
 for _ in 0..132{
  let result=super::super::codex::list_before(tokio::time::Instant::now()+Duration::from_millis(2),super::super::codex::list(&client)).await;
  assert!(result.unwrap_err().contains("excedeu o prazo"));assert!(client.pending.lock().unwrap().is_empty());
 }
 assert!(client.connected());let result=client.call("fixture/barrier",json!({})).await.unwrap();assert_eq!(result["stillConnected"],true);
 let methods=calls.lock().unwrap().clone();assert!(methods.iter().any(|m|m=="thread/list"));assert!(methods.iter().all(|m|m=="thread/list"||m=="fixture/barrier"));
 assert!(client.pending.lock().unwrap().is_empty());writer.abort();let _=writer.await;
}
