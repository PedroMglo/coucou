use super::*;
#[test]fn native_output_shape_is_bounded_and_no_write_protocol_is_exposed(){
 let parse=|v:Value|serde_json::from_value::<Output>(v);
 assert!(parse(json!({"id":1,"serial":"12","volumePercent":50.0,"muted":false})).unwrap().valid());
 for v in [json!({"id":0,"serial":"12","volumePercent":0.0,"muted":false}),json!({"id":1,"serial":"../../x","volumePercent":0.0,"muted":false}),json!({"id":1,"serial":"12","volumePercent":151.0,"muted":false})]{assert!(!parse(v).unwrap().valid());}
 assert!(parse(json!({"id":1,"serial":"12","volumePercent":50.0,"muted":false,"provider":"other"})).is_err());
 assert!(!crate::pet_audio::task::native_dispatch_available());
}
#[test]#[ignore="Requires installed Quickshell and the real user PipeWire service; read-only native observation"]
fn native_readonly_retains_one_output_object_without_changing_audio(){
 let mut owner=Reader::start(Instant::now()+Duration::from_secs(12),Arc::new(||true)).unwrap();
 let outputs=owner.outputs().unwrap();assert!(!outputs.is_empty(),"No output available for native read-only acceptance");
 let original=outputs[0].clone();let captured=owner.capture(&original).unwrap();
 for _ in 0..2{let current=owner.read().unwrap();assert_eq!(current.id,captured.id);assert_eq!(current.serial,captured.serial);}
 assert!(matches!(owner.capture(&original),Err(ExchangeError::NotStarted)));
 let hostile=json!({"version":2,"session":owner.session,"sequence":owner.sequence+1,"operation":"setVolume","id":null,"serial":null});
 let mut frame=serde_json::to_vec(&hostile).unwrap();frame.push(b'\n');
 assert_eq!(owner.owner.exchange_line(&frame,2048,Instant::now()+Duration::from_secs(2),&||true),Err(ExchangeError::Uncertain));
 println!("NATIVE_PIPEWIRE_READ_ONLY outputs={} retainedReads=2 unsupportedMutationRefused=true physicalWrites=0",outputs.len());
 drop(owner);
}
