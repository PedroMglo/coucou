//! In-memory pagination and time fixtures; never contacts any daemon or account.
use super::*;
use std::{cell::RefCell, collections::VecDeque, future::ready, rc::Rc, sync::{Arc, atomic::{AtomicUsize, Ordering}}, time::Duration};
fn id(n:usize)->String { format!("00000000-0000-0000-0000-{n:012}") }
fn thread(n:usize)->Value { json!({"id":id(n),"name":format!("synthetic {n}"),"cwd":"/synthetic","updatedAt":n as i64,"status":{"type":"idle"},"canAcceptDirectInput":true,"model":"synthetic:unchanged"}) }
fn small()->ListLimits { ListLimits { pages:16, rows:100, cursor_bytes:100, cursor_size:32, loaded_reads:8 } }
async fn script(pages:Vec<(&'static str,Value)>,limits:ListLimits)->Result<Vec<Summary>,String>{
 let mut pages:VecDeque<_>=pages.into();
 let result=list_with(|method,_|ready({let (expected,response)=pages.pop_front().expect("unexpected extra RPC");assert_eq!(method,expected);Ok(response)}),limits).await;
 if result.is_ok(){assert!(pages.is_empty());}result
}
#[tokio::test]
async fn normal_inventory_exceeds_twelve_preserves_archived_and_includes_loaded_only(){
 let calls=Rc::new(RefCell::new(Vec::new()));let recorded=calls.clone();
 let rows=list_with(move|method,params|{recorded.borrow_mut().push((method,params.clone()));ready(Ok(match method{
  "thread/list" if params["archived"]==false=>json!({"data":(1..=16).map(thread).collect::<Vec<_>>()}),
  "thread/list"=>json!({"data":[thread(17)]}),
  "thread/loaded/list"=>json!({"data":[id(1),id(18)]}),
  "thread/read"=>json!({"thread":thread(18)}),_=>panic!("unexpected {method}")
 }))},small()).await.unwrap();
 assert_eq!(rows.len(),18);assert_eq!(rows[0].target.session_id,id(18));
 let archived=rows.iter().find(|r|r.target.session_id==id(17)).unwrap();assert_eq!(archived.state,"archived");assert!(!archived.capabilities.send&&!archived.capabilities.cancel);
 assert!(rows.iter().all(|r|r.target.provider=="codex"&&r.target.pet_id=="agent_codex"&&r.target.origin==Origin::External&&r.model.as_deref()==Some("synthetic:unchanged")));
 let calls=calls.borrow();assert_eq!(calls.len(),4);assert_eq!(calls[0].1["limit"],100);assert_eq!(calls[0].1["sortKey"],"updated_at");assert_eq!(calls[0].1["sourceKinds"],json!(SOURCES));assert_eq!(calls[1].1["archived"],true);assert_eq!(calls[3].1,json!({"threadId":id(18),"includeTurns":false}));
}
#[tokio::test]
async fn empty_terminal_pages_are_a_complete_empty_inventory(){
 let rows=script(vec![("thread/list",json!({"data":[]})),("thread/list",json!({"data":[],"nextCursor":null})),("thread/loaded/list",json!({"data":[]}))],small()).await.unwrap();assert!(rows.is_empty());
}
#[tokio::test]
async fn fresh_cursors_on_empty_pages_are_bounded_in_each_of_three_phases(){
 for slow_phase in 0..3{
  let calls=Rc::new(RefCell::new(0));let observed=calls.clone();let mut phase_pages=0;
  let result=list_with(move|method,p|{*observed.borrow_mut()+=1;let phase=if method=="thread/loaded/list"{2}else if p["archived"]==true{1}else{0};
   ready(Ok(if phase==slow_phase{phase_pages+=1;json!({"data":[],"nextCursor":format!("fresh-{phase_pages}")})}else{json!({"data":[]})}))
  },ListLimits{pages:5,..small()}).await;
  assert!(result.unwrap_err().contains("excedeu um limite"));assert_eq!(*calls.borrow(),5);
 }
}
#[tokio::test]
async fn raw_duplicate_loaded_rows_are_counted_before_deduplication(){
 let reads=Rc::new(RefCell::new(0));let observed=reads.clone();let mut loaded=0;
 let result=list_with(move|method,_|ready(Ok(match method{
  "thread/list"=>json!({"data":[]}),"thread/loaded/list"=>{loaded+=1;json!({"data":[id(1),id(1),id(1)],"nextCursor":format!("fresh-{loaded}")})},
  "thread/read"=>{*observed.borrow_mut()+=1;json!({"thread":thread(1)})},_=>panic!()
 })),ListLimits{rows:5,..small()}).await;
 assert!(result.unwrap_err().contains("excedeu um limite"));assert_eq!(*reads.borrow(),1);
}
#[tokio::test]
async fn row_budget_is_cumulative_across_active_archived_and_loaded(){
 let result=script(vec![("thread/list",json!({"data":[thread(1),thread(2)]})),("thread/list",json!({"data":[thread(3),thread(4)]})),("thread/loaded/list",json!({"data":[id(5),id(6)]}))],ListLimits{rows:5,..small()}).await;
 assert!(result.unwrap_err().contains("lista completa não foi confirmada"));
}
#[tokio::test]
async fn cursor_byte_budget_is_cumulative_across_all_three_phases(){
 let result=script(vec![("thread/list",json!({"data":[],"nextCursor":"ab"})),("thread/list",json!({"data":[]})),("thread/list",json!({"data":[],"nextCursor":"ab"})),("thread/list",json!({"data":[]})),("thread/loaded/list",json!({"data":[],"nextCursor":"ab"}))],ListLimits{cursor_bytes:5,..small()}).await;
 assert!(result.unwrap_err().contains("excedeu um limite"));
}
#[tokio::test]
async fn oversized_and_malformed_cursors_never_publish_partial_rows(){
 for cursor in [json!("x".repeat(33)),json!(""),json!(true),json!(123),json!({"unexpected":true})]{
  assert!(script(vec![("thread/list",json!({"data":[thread(1)],"nextCursor":cursor}))],small()).await.is_err());
 }
}
#[tokio::test]
async fn repeated_cursor_fails_instead_of_returning_partial_inventory(){
 let result=script(vec![("thread/list",json!({"data":[thread(1)],"nextCursor":"same"})),("thread/list",json!({"data":[thread(2)],"nextCursor":"same"}))],small()).await;
 assert!(result.unwrap_err().contains("repetiu o cursor"));
}
#[tokio::test]
async fn loaded_only_metadata_reads_have_their_own_budget(){
 let result=script(vec![("thread/list",json!({"data":[]})),("thread/list",json!({"data":[]})),("thread/loaded/list",json!({"data":[id(1),id(2),id(3)]})),("thread/read",json!({"thread":thread(1)})),("thread/read",json!({"thread":thread(2)}))],ListLimits{loaded_reads:2,..small()}).await;
 assert!(result.unwrap_err().contains("excedeu um limite"));
}
#[tokio::test]
async fn malformed_loaded_id_is_rejected_before_any_metadata_read(){
 let result=script(vec![("thread/list",json!({"data":[]})),("thread/list",json!({"data":[]})),("thread/loaded/list",json!({"data":["invalid/id"]}))],small()).await;assert!(result.is_err());
}
#[tokio::test]
async fn metadata_identity_mismatch_is_not_a_complete_result(){
 let result=script(vec![("thread/list",json!({"data":[]})),("thread/list",json!({"data":[]})),("thread/loaded/list",json!({"data":[id(1)]})),("thread/read",json!({"thread":thread(2)}))],small()).await;assert!(result.unwrap_err().contains("outra sessão"));
}
#[tokio::test]
async fn one_async_deadline_includes_acquisition_and_does_not_reset_per_page(){
 let calls=Arc::new(AtomicUsize::new(0));let observed=calls.clone();let start=tokio::time::Instant::now();
 let result=list_before(start+Duration::from_millis(25),async{
  tokio::time::sleep(Duration::from_millis(12)).await;
  list_with(move|_,_|{observed.fetch_add(1,Ordering::SeqCst);async{tokio::time::sleep(Duration::from_millis(10)).await;Ok(json!({"data":[]}))}},small()).await
 }).await;
 assert!(result.unwrap_err().contains("excedeu o prazo"));assert!(calls.load(Ordering::SeqCst)<3);assert!(start.elapsed()<Duration::from_secs(1));
}
#[tokio::test]
async fn deadline_includes_waiting_for_the_client_mutex(){
 let lock=tokio::sync::Mutex::new(());let _held=lock.lock().await;let entered=AtomicUsize::new(0);
 let result=list_before(tokio::time::Instant::now()+Duration::from_millis(5),async{let _guard=lock.lock().await;entered.fetch_add(1,Ordering::SeqCst);Ok(Vec::<Summary>::new())}).await;
 assert!(result.unwrap_err().contains("tarefas em curso não foram interrompidas"));assert_eq!(entered.load(Ordering::SeqCst),0);
}
#[test]
fn counter_overflow_also_fails_closed(){let mut count=usize::MAX;assert!(ListBudget::add(&mut count,1,usize::MAX).is_err());}

#[tokio::test]
async fn expired_deadline_never_starts_acquisition(){
 let entered=AtomicUsize::new(0);
 let result=list_before(tokio::time::Instant::now()-Duration::from_millis(1),async{entered.fetch_add(1,Ordering::SeqCst);Ok(())}).await;
 assert!(result.is_err());assert_eq!(entered.load(Ordering::SeqCst),0);
}
#[tokio::test]
async fn synchronous_work_is_not_preempted_but_cannot_publish_a_late_success(){
 let result=list_before(tokio::time::Instant::now()+Duration::from_millis(1),async{std::thread::sleep(Duration::from_millis(5));Ok(Vec::<Summary>::new())}).await;
 assert!(result.unwrap_err().contains("excedeu o prazo"));
}
