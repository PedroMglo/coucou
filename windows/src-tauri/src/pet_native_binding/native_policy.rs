//! Synchronous native policy sequencer used by the real GTK adapter and causal
//! fixtures. Surface methods must never retain Core/manager locks across GTK.
use super::{core::Layout,ipc::*};
#[derive(Clone,Debug,PartialEq)]
pub(crate) struct Snapshot {pub(crate) context:Context,pub(crate) layout:Layout}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Step {Resizable,Size,Pin,Margins,Position,Show,Hide}
pub(crate) trait Surface {
 fn current(&mut self)->Result<Snapshot,Failure>;
 fn input(&mut self,snapshot:&Snapshot)->Result<(),Failure>;
 fn step(&mut self,step:Step)->Result<(),Failure>;
 /// Permanently revoke this exact incarnation on nonconvergent native callbacks.
 /// Must not operate on a replacement. Native clear/hide occurs after unlocking.
 fn close_unstable(&mut self)->Result<(),Failure>;
}
const MAX_PASSES:usize=8;
/// Each native write is postvalidated against a fresh atomic Core policy. Any
/// synchronous callback change causes a CURRENT policy application before return.
/// This is not an atomic Core/GTK/compositor transaction across undelivered events.
pub(crate) fn reconcile(surface:&mut impl Surface)->Result<Snapshot,Failure>{
 for _ in 0..MAX_PASSES {
  let before=surface.current()?;
  surface.input(&before)?;
  let after_input=surface.current()?;
  if before!=after_input{continue;}
  if matches!(after_input.context.mode,Mode::Inactive)&&!after_input.context.pending&&after_input.context.native_visible{
   surface.step(Step::Hide)?;
   if surface.current()?!=after_input{continue;}
  }
  return Ok(after_input);
 }
 surface.close_unstable()?;
 Err(Failure::binding(ErrorCode::NativeFailure,None))
}
/// Placement/show are attempted only while the initiating revision/plan is current.
/// Even a failed/superseded native step is reconciled; its captured layout is never
/// reapplied afterwards. Ownership loss returns without touching the replacement.
pub(crate) fn run_steps(surface:&mut impl Surface,steps:&[Step],mut allowed:impl FnMut(&Snapshot)->bool)->Result<Snapshot,Failure>{
 let initial=reconcile(surface)?;
 if !allowed(&initial){return Err(Failure::binding(ErrorCode::Superseded,Some(initial.context)));}
 for step in steps {
  let before=surface.current()?;
  if !allowed(&before){let current=reconcile(surface)?;return Err(Failure::binding(ErrorCode::Superseded,Some(current.context)));}
  let applied=surface.step(*step);
  let current=reconcile(surface)?;
  applied?;
  if !allowed(&current){return Err(Failure::binding(ErrorCode::Superseded,Some(current.context)));}
 }
 reconcile(surface)
}
#[cfg(test)]#[path="native_policy_tests.rs"]mod tests;
