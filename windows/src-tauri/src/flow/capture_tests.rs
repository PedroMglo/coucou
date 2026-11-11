use super::*;
use std::{fs,sync::Mutex};

struct Fixture{root:std::path::PathBuf,flow:Arc<Flow>}
impl Fixture{
 fn new()->Self{let root=std::env::temp_dir().join(format!("coucou-flow-capture-{}-{}",std::process::id(),text::new_id()));fs::create_dir(&root).unwrap();
  let mut store=Store::load_from(root.clone());store.prefs.enabled=true;store.prefs.mic="original".into();store.words.replacements.push(Replacement{id:"r".into(),from:"cloud".into(),to:"original".into()});
  Self{root,flow:Arc::new(Flow{store:Mutex::new(store),running:Mutex::new(None),prefs_update:Mutex::new(()),dictation_cancel:Mutex::new(None)})}
 }
 fn input(&self)->Input{let settings=crate::settings::Settings::default();let origin=Arc::new(crate::settings::capture_origin(&settings));let recognition=RecognitionScope::from_parts(settings,crate::pet_config::defaults(),origin).unwrap();
  let options=Options::capture(&self.flow.store.lock().unwrap(),None).unwrap();let source=audio::Source::Mic(Some(options.mic.clone()));let stop=Arc::new(AtomicBool::new(false));let work=Work::claim(&self.flow,stop.clone()).unwrap();Input{recognition,options,source,owner:Arc::downgrade(&self.flow),front:None,stop,_work:work}
 }
}
impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
#[test]fn dictation_capture_freezes_text_options_instead_of_applying_new_dictionary(){
 let f=Fixture::new();let input=f.input();assert_eq!(text::finish(&input.prepare("cloud").text,&input.prepare("cloud").snippets),"original");
 f.flow.store.lock().unwrap().words.replacements[0].to="replacement".into();assert!(input.owner_current().is_err());assert_eq!(input.prepare("cloud").text,"original");
}
#[test]fn dictation_capture_rejects_mic_key_retention_and_cleanup_changes(){
 for field in[0,1,2,3]{let f=Fixture::new();let input=f.input();let mut s=f.flow.store.lock().unwrap();match field{0=>s.prefs.mic="different".into(),1=>s.prefs.key=FlowKey::RightAlt,2=>s.prefs.keep_days=0,_=>s.prefs.cleanup=true};drop(s);assert!(input.owner_current().is_err());}
}
#[test]fn dictation_capture_does_not_depend_on_playback_or_visual_preferences(){
 let f=Fixture::new();let input=f.input();let mut s=f.flow.store.lock().unwrap();s.prefs.voice_output="unsupported-output".into();s.prefs.voice_gender="changed".into();s.prefs.edge=super::super::store::Edge::Left;s.prefs.open_at_launch=false;drop(s);assert!(input.owner_current().is_ok());
}
#[test]fn dictation_capture_cancel_keeps_exclusion_until_the_owner_returns(){
 let f=Fixture::new();let input=f.input();cancel_processing(&f.flow);assert!(input.stop.load(Ordering::Acquire));assert!(input.owner_current().is_err());assert!(Work::claim(&f.flow,Arc::new(AtomicBool::new(false))).is_err());drop(input);assert!(f.flow.dictation_cancel.lock().unwrap().is_none());
}
#[test]fn dictation_capture_old_completion_cannot_clear_a_replacement_token(){
 let f=Fixture::new();let old=Arc::new(AtomicBool::new(false));let work=Work::claim(&f.flow,old).unwrap();let successor=Arc::new(AtomicBool::new(false));*f.flow.dictation_cancel.lock().unwrap()=Some(successor.clone());drop(work);assert!(f.flow.dictation_cancel.lock().unwrap().as_ref().is_some_and(|s|Arc::ptr_eq(s,&successor)));
}
#[test]fn dictation_capture_send_cancel_revokes_without_a_key_listener_or_queue(){
 let f=Fixture::new();let input=f.input();f.flow.send(super::super::KeyEvent::Cancel);assert!(input.stop.load(Ordering::Acquire));
}

#[test]fn dictation_capture_bounds_selected_context_before_copying_or_recording(){
 let f=Fixture::new();let mut store=f.flow.store.lock().unwrap();store.words.words=vec!["word".into();4097];assert!(Options::capture(&store,None).is_err());store.words.words=vec!["x".repeat(8*1024*1024+1)];assert!(Options::capture(&store,None).is_err());
}
#[test]fn dictation_capture_reader_contention_does_not_retarget_or_disable_cancel(){
 let f=Fixture::new();let input=f.input();std::thread::scope(|s|{let held=f.flow.store.lock().unwrap();let stop=input.stop.clone();let flow=f.flow.clone();let waiting=s.spawn(||input.owner_current());let cancel=s.spawn(move||{cancel_processing(&flow);assert!(stop.load(Ordering::Acquire));});cancel.join().unwrap();drop(held);assert!(waiting.join().unwrap().is_err());});
}

#[test]fn manual_clipboard_work_is_independent_of_asr_and_is_cancelled_by_the_same_stop(){
 let f=Fixture::new();f.flow.store.lock().unwrap().prefs.enabled=false;
 let work=Work::claim(&f.flow,Arc::new(AtomicBool::new(false))).unwrap();assert!(work.current());f.flow.send(super::super::KeyEvent::Cancel);assert!(!work.current());assert!(Work::claim(&f.flow,Arc::new(AtomicBool::new(false))).is_err());drop(work);assert!(Work::claim(&f.flow,Arc::new(AtomicBool::new(false))).unwrap().current());
}
#[test]fn manual_clipboard_old_owner_cannot_use_a_replacement_slot(){
 let f=Fixture::new();let work=Work::claim(&f.flow,Arc::new(AtomicBool::new(false))).unwrap();let successor=Arc::new(AtomicBool::new(false));*f.flow.dictation_cancel.lock().unwrap()=Some(successor.clone());assert!(!work.current());drop(work);assert!(Arc::ptr_eq(f.flow.dictation_cancel.lock().unwrap().as_ref().unwrap(),&successor));
}
#[test]fn cancelled_dictation_does_not_add_history_after_output(){
 let f=Fixture::new();let input=f.input();input.stop.store(true,Ordering::Release);assert!(input.record("synthetic text",None,1.0).is_err());assert!(f.flow.store.lock().unwrap().last_text().is_none());assert!(!f.root.join("history.json").exists());
}
#[test]fn changed_retention_does_not_add_history_after_output(){
 let f=Fixture::new();let input=f.input();f.flow.store.lock().unwrap().prefs.keep_days=0;assert!(input.record("synthetic text",None,1.0).is_err());assert!(f.flow.store.lock().unwrap().last_text().is_none());
}

#[test]fn clearing_or_replacing_last_output_invalidates_a_waiting_manual_copy(){
 for change in[0,1,2]{let f=Fixture::new();f.flow.store.lock().unwrap().record("old selected text",None,1.0,super::super::store::today(),super::super::store::now_ms());let source=LastText::capture(&f.flow).unwrap().unwrap();assert!(source.current());let mut store=f.flow.store.lock().unwrap();match change{0=>store.clear_history(),1=>{store.record("new text",None,1.0,super::super::store::today(),super::super::store::now_ms());},_=>{*store=Store::load_from(f.root.clone());}}drop(store);assert!(!source.current());assert_eq!(source.text,"old selected text");}
}
#[test]fn stop_revokes_last_text_copy_without_releasing_its_writer_early(){let f=Fixture::new();f.flow.store.lock().unwrap().record("synthetic",None,1.0,super::super::store::today(),super::super::store::now_ms());let source=LastText::capture(&f.flow).unwrap().unwrap();f.flow.send(super::super::KeyEvent::Cancel);assert!(!source.current());assert!(LastText::capture(&f.flow).is_err());drop(source);assert!(LastText::capture(&f.flow).unwrap().unwrap().current());}
#[test]fn no_history_keeps_only_the_current_text_in_memory_and_clear_revokes_it(){let f=Fixture::new();f.flow.store.lock().unwrap().prefs.keep_days=0;f.flow.store.lock().unwrap().record("private unkept output",None,1.0,super::super::store::today(),super::super::store::now_ms());assert!(f.flow.store.lock().unwrap().history.is_empty());let source=LastText::capture(&f.flow).unwrap().unwrap();assert_eq!(source.text,"private unkept output");f.flow.store.lock().unwrap().clear_history();assert!(!source.current());assert!(!f.flow.store.lock().unwrap().has_last_text());}

#[test]fn manual_last_text_wait_observes_stop_before_the_store_mutex_is_released(){
 let f=Fixture::new();f.flow.store.lock().unwrap().record("selected",None,1.0,super::super::store::today(),super::super::store::now_ms());let source=LastText::capture(&f.flow).unwrap().unwrap();
 std::thread::scope(|scope|{let held=f.flow.store.lock().unwrap();let waiting=scope.spawn(||source.current());f.flow.send(super::super::KeyEvent::Cancel);assert!(!waiting.join().unwrap());drop(held);});
}
