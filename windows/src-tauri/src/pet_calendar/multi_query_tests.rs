//! Offline routing proofs; synthetic runners never contact a model or account.
use super::*;
use std::sync::atomic::AtomicUsize;

pub(crate) fn source(letter:char)->Value{
    json!({"id":letter.to_string().repeat(64),"revision":format!("revision-{letter}"),"name":"same-name.ics"})
}
pub(crate) fn arguments(letter:char)->Value{
    json!({"sourceId":source(letter)["id"],"start":"2026-11-01T00:00:00Z","end":"2026-11-02T00:00:00Z","timeZone":"UTC","limit":32})
}
pub(crate) fn ready(source:&Value,query:&Query)->Value{
    json!({"schemaVersion":1,"kind":"calendarEvents","status":"ready","source":source,"window":query.window(),"events":[{"uid":"same-uid","summary":source["revision"]}],"hasMore":false,"coverage":{"eventsComplete":true,"fieldsComplete":true,"reasons":[]}})
}
pub(in crate::pet_calendar) fn fixture(run:impl Fn(&Value,&Query,&AtomicBool)->Result<Value,String>+Send+Sync+'static)->(super::super::Guard,Service,Arc<AtomicBool>,Arc<AtomicBool>){
    let(guard,_,current,cancelled)=super::tests::fixture(|_,_|unreachable!());
    let run=Arc::new(run);
    let routes=['a','b'].into_iter().map(|letter|{
        let source=source(letter);let captured=source.clone();let run=run.clone();
        Route::new(source,move|query,aborted|run(&captured,query,aborted))
    }).collect();
    let service=Service::multiple_at(&guard,Some("2026-10-23T12:00:00Z".into()),routes).unwrap();
    (guard,service,current,cancelled)
}
pub(crate) fn note_fixture()->super::super::Note{
    let(guard,service,_,_)=fixture(|source,query,_|Ok(ready(source,query)));
    super::super::Note{text:"Synthetic selected sources, no account or model IO".into(),guard:Some(guard),query:Some(service)}
}
fn value(service:&Service,args:Value)->Value{
    serde_json::from_str(&service.execute_auto(args,&AtomicBool::new(false)).unwrap()).unwrap()
}

#[test]
fn multi_source_strict_and_native_schemas_require_the_exact_captured_ids(){
    let(_guard,service,_,_)=fixture(|_,_,_|unreachable!());
    for(native,definitions)in [(false,service.definitions()),(true,service.native_definitions())]{
        assert_eq!(definitions.len(),2);
        for definition in definitions{
            let schema=&definition["input_schema"];
            assert_eq!(schema["additionalProperties"],false);
            assert_eq!(schema["properties"]["sourceId"]["enum"],json!([source('a')["id"],source('b')["id"]]));
            let required=schema["required"].as_array().unwrap();assert!(required.contains(&json!("sourceId")));
            assert_eq!(required.contains(&json!("limit")),!native);
            for forbidden in ["path","account","permission","observedAt"]{assert!(schema["properties"].get(forbidden).is_none());}
        }
    }
    let(_guard,single,_,_)=super::tests::fixture(|_,_|unreachable!());
    for definition in single.definitions(){assert!(definition["input_schema"]["properties"].get("sourceId").is_none());}
}

#[test]
fn invalid_source_membership_precedes_time_zone_resolution_and_source_reads(){
    for id in [Value::Null,json!(7),json!("/never-read.ics"),source('c')["id"].clone()]{
        let(_guard,service,_,_)=fixture(|_,_,_|panic!("Unknown source must not start any runner"));
        // Even an invalid resolver input cannot replace the membership error.
        let mut args=json!({"date":"today","timeZone":"../../not-a-zone"});
        if !id.is_null(){args["sourceId"]=id;}
        let result=value(&service,args);assert_eq!(result["errorCode"],"invalidSource");assert_eq!(result["events"],json!([]));
    }
    let(_guard,service,_,_)=fixture(|_,_,_|panic!("Extra model authority must not start a runner"));
    for key in ["path","permission","account"]{let mut args=arguments('a');args[key]=json!("forbidden");assert_eq!(value(&service,args)["errorCode"],"invalidWindow");}
}

#[test]
fn duplicate_names_and_uids_never_select_or_merge_another_source(){
    let(_guard,service,_,_)=fixture(|source,query,_|Ok(ready(source,query)));
    let a=value(&service,arguments('a'));let b=value(&service,arguments('b'));
    assert_eq!(a["source"],source('a'));assert_eq!(b["source"],source('b'));
    assert_eq!(a["events"][0]["uid"],b["events"][0]["uid"]);
    assert_ne!(a["events"][0]["summary"],b["events"][0]["summary"]);
    assert_eq!(a["events"].as_array().unwrap().len(),1);assert_eq!(b["events"].as_array().unwrap().len(),1);
}

#[test]
fn both_sources_clones_and_day_queries_share_the_same_eight_call_budget(){
    let count=Arc::new(AtomicUsize::new(0));let seen=count.clone();
    let(_guard,service,_,_)=fixture(move|source,query,_|{seen.fetch_add(1,Ordering::AcqRel);Ok(ready(source,query))});
    let clone=service.clone();
    // Invalid dates also consume the common allowance without starting IO.
    assert_eq!(value(&service,json!({"sourceId":source('a')["id"],"date":"not-a-date","timeZone":"UTC"}))["errorCode"],"invalidWindow");
    for i in 0..7{assert_eq!(value(&clone,arguments(if i%2==0{'a'}else{'b'}))["status"],"ready");}
    assert_eq!(value(&service,arguments('b'))["errorCode"],"queryLimit");assert_eq!(count.load(Ordering::Acquire),7);
}

#[test]
fn cumulative_output_allowance_cannot_be_multiplied_by_selecting_more_sources(){
    let(_guard,service,_,_)=fixture(|source,query,_|{let mut result=ready(source,query);result["events"][0]["summary"]=json!("x".repeat(20*1024));Ok(result)});
    for i in 0..6{assert!(service.execute(arguments(if i%2==0{'a'}else{'b'}),&AtomicBool::new(false)).is_ok());}
    assert!(service.clone().execute(arguments('b'),&AtomicBool::new(false)).unwrap_err().contains("limite de dados"));
}

#[test]
fn a_runner_response_for_a_different_source_is_removed_before_publication(){
    let(_guard,service,_,_)=fixture(|_,query,_|Ok(ready(&source('c'),query)));
    let result=value(&service,arguments('a'));assert_eq!(result["status"],"unavailable");assert_eq!(result["errorCode"],"workerResponseMismatch");assert_eq!(result["source"],source('a'));assert_eq!(result["events"],json!([]));
}

#[test]
fn multi_source_abort_revocation_and_note_close_preserve_original_turn_boundaries(){
    let(guard,service,current,cancelled)=fixture(|_,_,_|panic!("No authority, no source read"));
    assert!(service.execute(arguments('a'),&AtomicBool::new(true)).is_err());assert!(!cancelled.load(Ordering::Acquire));
    current.store(false,Ordering::Release);assert!(service.execute(arguments('b'),&AtomicBool::new(false)).is_err());assert!(cancelled.load(Ordering::Acquire));drop(guard);
    let(guard,service,_,cancelled)=fixture(|_,_,_|panic!("Closed note"));drop(guard);
    assert!(service.execute(arguments('a'),&AtomicBool::new(false)).is_err());assert!(!cancelled.load(Ordering::Acquire));
}

#[test]
fn revocation_during_one_source_runner_discards_the_captured_result(){
    let switch=Arc::new(Mutex::new(None::<Arc<AtomicBool>>));let observed=switch.clone();
    let(_guard,service,current,cancelled)=fixture(move|source,query,_|{observed.lock().unwrap().as_ref().unwrap().store(false,Ordering::Release);Ok(ready(source,query))});
    *switch.lock().unwrap()=Some(current);
    assert!(service.execute(arguments('b'),&AtomicBool::new(false)).is_err());assert!(cancelled.load(Ordering::Acquire));
}

#[test]
fn the_route_constructor_rejects_duplicate_ids_invalid_ids_and_unbounded_lists(){
    let(guard,_,_,_)=super::tests::fixture(|_,_|unreachable!());
    for ids in [vec![],vec!["a".repeat(64)],vec!["a".repeat(64);2],vec!["A".repeat(64),"b".repeat(64)],vec!["/path".into(),"b".repeat(64)],(0..34).map(|i|format!("{i:064x}")).collect()]{
        let routes=ids.into_iter().map(|id|Route::new(json!({"id":id}),|_,_|unreachable!())).collect();
        assert!(Service::multiple_at(&guard,None,routes).is_err());
    }
}
