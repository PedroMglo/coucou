//! Actual production sequencer + real pinned Core/facade/lifecycle; fake native
//! operations inject synchronous reentry causally. No GTK/compositor proof.
use super::*;
use crate::pet_native_binding::core::{self,Core,Facts,Hint,Manager};
use std::{sync::Arc,collections::VecDeque};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Point{BeforeStep(Step),AfterStep(Step),BeforeInput,AfterInput}
#[derive(Clone,Copy,Debug)]enum Event{Disable,Pending,Hide,Remap,Crash,Destroy,Replace,Hint,Churn}
struct Fake {
 m:Arc<Manager>,c:Arc<Core>,owned:bool,visible:bool,mapped:bool,
 physical:core::Layout,focus:bool,timer:bool,hooks:VecDeque<(Point,Event)>,
 steps:Vec<Step>,writes:usize,input_calls:usize,closed:bool,
 replacement_layout:core::Layout,repeat_churn:bool,fail:Option<Step>,
}
impl Fake {
 fn new()->Self{
  let m=Manager::new(true);let c=core::fixture_register(&m);
  c.document(false,true).unwrap();c.observe_native(Facts{visible:true,mapped:true}).unwrap();
  let(p,_)=c.begin_trusted(Mode::Expanded{route:Route::System}).unwrap();c.finish(&p,true).unwrap();
  c.layout(Some(Hint{x:0.,y:0.,w:700.,h:650.,keyboard:true}),None).unwrap();
  let layout=c.policy_snapshot().unwrap().layout;
  Self{m,c,owned:true,visible:true,mapped:true,physical:layout.clone(),focus:true,timer:true,hooks:VecDeque::new(),steps:vec![],writes:0,input_calls:0,closed:false,replacement_layout:core::Layout{rects:vec![(8.,8.,9.,9.)],expanded:false,keyboard:false},repeat_churn:false,fail:None}
 }
 fn facts(&self){self.c.observe_native(Facts{visible:self.visible,mapped:self.mapped}).unwrap();}
 fn event(&mut self,event:Event){
  match event{
   Event::Disable=>{self.m.settings_committed(false).unwrap();},
   Event::Pending=>{self.c.begin_trusted(Mode::Expanded{route:Route::Sessions}).unwrap();},
   Event::Hide=>{self.visible=false;self.mapped=false;self.facts();self.focus=false;self.timer=false;},
   Event::Remap=>{self.mapped=false;self.facts();self.focus=false;self.timer=false;self.mapped=true;self.facts();},
   Event::Crash=>{self.c.crashed().unwrap();},
   Event::Destroy=>{self.m.detach(self.c.incarnation).unwrap();self.owned=false;self.focus=false;self.timer=false;self.physical=empty();},
   Event::Replace=>{core::fixture_register(&self.m);self.owned=false;self.focus=false;self.timer=false;self.physical=empty();},
   Event::Hint=>{self.c.layout(Some(Hint{x:1.,y:2.,w:77.,h:88.,keyboard:false}),None).unwrap();},
   Event::Churn=>{if let Ok((p,_))=self.c.begin_trusted(Mode::Expanded{route:Route::System}){self.c.finish(&p,true).unwrap();}},
  }
 }
 fn hook(&mut self,point:Point){
  if self.hooks.front().is_some_and(|(p,_)|*p==point){let(_,e)=self.hooks.pop_front().unwrap();self.event(e);}
  if self.repeat_churn&&point==Point::AfterInput{self.event(Event::Churn);}
 }
 fn current_physical(&mut self){let current=self.current().unwrap();assert_eq!(self.physical,current.layout);assert_eq!(self.focus,current.layout.keyboard);assert_eq!(self.timer,current.layout.expanded&&current.context.native_visible&&current.context.native_mapped);}
}
fn empty()->core::Layout{core::Layout{rects:vec![],expanded:false,keyboard:false}}
impl Surface for Fake {
 fn current(&mut self)->Result<Snapshot,Failure>{
  if !self.owned{return Err(Failure::unavailable());}
  // Models native URI remaining the old allowed URI after a process crash.
  // The Core crash latch must keep document inactive until a fresh load start.
  self.c.document(false,true)?;self.c.observe_native(Facts{visible:self.visible,mapped:self.mapped})?;
  self.m.with_current(self.c.incarnation,|c|c.policy_snapshot())
 }
 fn input(&mut self,snapshot:&Snapshot)->Result<(),Failure>{
  self.input_calls+=1;self.hook(Point::BeforeInput);
  if !self.owned{return Err(Failure::unavailable());}
  // Deliberately apply the captured input even if a synchronous callback just
  // changed Core, reproducing R1's stale write. Production sequencer must repair.
  self.physical=snapshot.layout.clone();self.focus=self.physical.keyboard;self.timer=self.physical.expanded&&self.visible&&self.mapped;self.writes+=1;
  self.hook(Point::AfterInput);Ok(())
 }
 fn step(&mut self,step:Step)->Result<(),Failure>{
  self.hook(Point::BeforeStep(step));if !self.owned{return Err(Failure::unavailable());}
  self.steps.push(step);
  match step{Step::Show=>{self.visible=true;self.mapped=true;self.facts();},Step::Hide=>{self.visible=false;self.mapped=false;self.facts();self.focus=false;self.timer=false;},_=>{}}
  self.hook(Point::AfterStep(step));
  if self.fail==Some(step){return Err(Failure::binding(ErrorCode::NativeFailure,None));}Ok(())
 }
 fn close_unstable(&mut self)->Result<(),Failure>{
  if !self.owned{return Err(Failure::unavailable());}self.c.close()?;self.closed=true;
  self.visible=false;self.mapped=false;self.physical=empty();self.focus=false;self.timer=false;Ok(())
 }
}
const PLACEMENT:[Step;6]=[Step::Resizable,Step::Size,Step::Pin,Step::Margins,Step::Position,Step::Show];
#[test]fn causal_disable_before_and_after_every_placement_step_repairs_old_mask_and_focus(){
 for step in PLACEMENT{for point in[Point::BeforeStep(step),Point::AfterStep(step)]{
  let mut f=Fake::new();let expected=f.c.context().unwrap().revision;f.hooks.push_back((point,Event::Disable));
  assert!(run_steps(&mut f,&PLACEMENT,|s|s.context.revision==expected&&!s.context.pending).is_err());
  f.current_physical();assert!(!f.focus&&!f.timer);assert!(!f.c.context().unwrap().system_eligible);assert!(f.physical.rects.iter().all(|r|r.3<=32.));
 }}
}
#[test]fn causal_pending_before_and_after_every_placement_step_repairs_old_expanded_policy(){
 for step in PLACEMENT{for point in[Point::BeforeStep(step),Point::AfterStep(step)]{
  let mut f=Fake::new();let expected=f.c.context().unwrap().revision;f.hooks.push_back((point,Event::Pending));
  assert!(run_steps(&mut f,&PLACEMENT,|s|s.context.revision==expected&&!s.context.pending).is_err());
  f.current_physical();assert!(!f.focus&&!f.timer);assert!(f.c.context().unwrap().pending);assert!(!f.c.context().unwrap().system_eligible);
 }}
}
#[test]fn causal_native_hide_and_remap_during_each_step_never_restore_old_keyboard(){
 for event in[Event::Hide,Event::Remap]{for step in PLACEMENT{for point in[Point::BeforeStep(step),Point::AfterStep(step)]{
  let mut f=Fake::new();let expected=f.c.context().unwrap().revision;let(client,_inbox)=crate::executor::fixture_client(&f.m.binding);let guard=f.c.prepare(&client,expected).unwrap();f.hooks.push_back((point,event));
  assert!(run_steps(&mut f,&PLACEMENT,|s|s.context.revision==expected).is_err());f.current_physical();assert!(!f.focus);assert!(guard.validate().is_err());
 }}}
}
#[test]fn causal_crash_mapped_with_keyboard_and_timer_quiesces_before_return_and_blocks_old_uri(){
 for point in[Point::BeforeInput,Point::AfterInput,Point::BeforeStep(Step::Size),Point::AfterStep(Step::Show)]{
  let mut f=Fake::new();let expected=f.c.context().unwrap().revision;f.hooks.push_back((point,Event::Crash));
  assert!(run_steps(&mut f,&PLACEMENT,|s|s.context.revision==expected).is_err());f.current_physical();assert_eq!(f.physical,empty());assert!(!f.focus&&!f.timer&&!f.visible);let c=f.c.context().unwrap();assert!(!c.system_eligible&&!c.document_ready);assert!(matches!(c.mode,Mode::Inactive));
 }
}
#[test]fn crash_callback_uses_same_production_reconcile_immediately_and_recovers_only_fresh_load(){
 let mut f=Fake::new();let old=f.c.context().unwrap().revision;f.event(Event::Crash);assert!(f.focus&&f.timer); // native mapped crash, before callback's quiesce phase
 let after=reconcile(&mut f).unwrap();assert_eq!(after.layout,empty());assert!(!f.focus&&!f.timer&&!f.visible);assert!(!after.context.document_ready);
 f.c.document(false,true).unwrap();assert!(!f.c.context().unwrap().document_ready);assert!(f.c.begin_trusted(Mode::Expanded{route:Route::System}).is_err());
 f.c.document(true,false).unwrap();f.c.document(false,true).unwrap();assert!(!f.c.context().unwrap().document_ready);
 f.c.load_started().unwrap();f.c.document(false,true).unwrap();let c=f.c.context().unwrap();assert!(c.document_ready&&!c.system_eligible);assert_ne!(old,c.revision);assert!(matches!(c.mode,Mode::Inactive));
}
#[test]fn callbacks_during_input_write_reconcile_current_layout_even_without_revision_change(){
 for event in[Event::Disable,Event::Pending,Event::Hide,Event::Remap,Event::Hint]{for point in[Point::BeforeInput,Point::AfterInput]{
  let mut f=Fake::new();let revision=f.c.context().unwrap().revision;f.hooks.push_back((point,event));reconcile(&mut f).unwrap();f.current_physical();assert!(!f.focus);assert!(f.writes>=2);
  if matches!(event,Event::Hint){assert_eq!(revision,f.c.context().unwrap().revision);assert_eq!(f.physical.rects[0],(1.,2.,77.,88.));}
 }}
}
#[test]fn replacement_or_destroy_before_or_after_step_prevents_late_writes_to_replacement(){
 for event in[Event::Destroy,Event::Replace]{for point in[Point::BeforeInput,Point::AfterInput,Point::BeforeStep(Step::Size),Point::AfterStep(Step::Size)]{
  let mut f=Fake::new();let expected=f.c.context().unwrap().revision;let replacement=f.replacement_layout.clone();f.hooks.push_back((point,event));assert!(run_steps(&mut f,&PLACEMENT,|s|s.context.revision==expected).is_err());
  let writes=f.writes;let calls=f.input_calls;assert!(reconcile(&mut f).is_err());assert_eq!(writes,f.writes);assert_eq!(calls,f.input_calls);assert_eq!(f.replacement_layout,replacement);assert!(!f.c.context().unwrap().native_available);
 }}
}
#[test]fn failed_native_step_still_reconciles_current_policy(){
 let mut f=Fake::new();let expected=f.c.context().unwrap().revision;f.fail=Some(Step::Size);f.hooks.push_back((Point::AfterStep(Step::Size),Event::Disable));assert!(run_steps(&mut f,&PLACEMENT,|s|s.context.revision==expected).is_err());f.current_physical();assert!(!f.focus&&!f.timer);assert!(!f.steps.contains(&Step::Show));
}
#[test]fn first_map_remap_preserves_current_mode_but_never_old_focus_or_request_guard(){
 let mut f=Fake::new();let(client,_inbox)=crate::executor::fixture_client(&f.m.binding);let guard=f.c.prepare(&client,f.c.context().unwrap().revision).unwrap();
 run_steps(&mut f,&[Step::Hide,Step::Show],|s|s.context.native_available&&!s.context.pending&&!matches!(s.context.mode,Mode::Inactive)).unwrap();f.current_physical();assert!(f.c.context().unwrap().system_eligible);assert!(!f.focus);assert!(guard.validate().is_err());
}
#[test]fn superseding_remap_with_inactive_or_pending_never_reopens_departed_surface(){
 for event in[Event::Crash,Event::Pending]{let mut f=Fake::new();f.hooks.push_back((Point::AfterStep(Step::Hide),event));assert!(run_steps(&mut f,&[Step::Hide,Step::Show],|s|s.context.native_available&&!s.context.pending&&!matches!(s.context.mode,Mode::Inactive)).is_err());assert!(!f.steps.contains(&Step::Show));assert!(!f.visible&&!f.focus&&!f.timer);f.current_physical();}
}
#[test]fn nonconverging_native_callbacks_are_bounded_and_close_exact_incarnation(){
 let mut f=Fake::new();f.repeat_churn=true;assert!(reconcile(&mut f).is_err());assert_eq!(f.writes,MAX_PASSES);assert!(f.closed);assert!(!f.c.context().unwrap().native_available);assert_eq!(f.physical,empty());assert!(!f.focus&&!f.timer);assert!(f.c.begin_trusted(Mode::Expanded{route:Route::System}).is_err());
}

#[test]fn webkit_destroy_quiesces_surviving_native_window_before_detach(){
 let mut f=Fake::new();assert!(f.visible&&f.focus&&f.timer);
 // Same Surface::close_unstable hook used synchronously by GTK/WebKit destroy
 // callback before Manager::detach, while the actual old Gtk window is alive.
 f.close_unstable().unwrap();assert_eq!(f.physical,empty());assert!(!f.focus&&!f.timer&&!f.visible);assert!(!f.c.context().unwrap().native_available);
 f.m.detach(f.c.incarnation).unwrap();f.owned=false;let calls=f.input_calls;assert!(reconcile(&mut f).is_err());assert_eq!(calls,f.input_calls);
}
#[test]fn causal_r1_placement_semantics_reproduce_reported_p2_and_r2_repairs_same_event(){
 // Exact ordering of R1 place_current: capture policy/revision, GTK placement,
 // apply captured input, show_all, refresh-only. This is a fake native causal
 // reproduction of that code ordering, never a claim about the live .19 app.
 let mut old=Fake::new();let captured=old.current().unwrap();old.hooks.push_back((Point::AfterStep(Step::Size),Event::Disable));
 for step in[Step::Resizable,Step::Size,Step::Pin,Step::Margins,Step::Position]{old.step(step).unwrap();}
 old.input(&captured).unwrap();old.step(Step::Show).unwrap();let current=old.current().unwrap();
 assert!(old.physical.expanded&&old.focus&&old.timer);assert!(!current.context.system_eligible);assert!(!current.layout.expanded&&!current.layout.keyboard);assert_ne!(old.physical,current.layout);
 let mut fixed=Fake::new();let expected=fixed.c.context().unwrap().revision;fixed.hooks.push_back((Point::AfterStep(Step::Size),Event::Disable));
 assert!(run_steps(&mut fixed,&PLACEMENT,|s|s.context.revision==expected).is_err());fixed.current_physical();assert!(!fixed.focus&&!fixed.timer);
}
