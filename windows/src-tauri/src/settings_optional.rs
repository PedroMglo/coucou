//! Admission for optional effects. This is a provenance prerequisite, never a
//! grant, durable effect receipt, or replacement for a component's own policy.
//! Call outside the Settings mutex, and again after waits before new IO.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Operation { Voice, Mobile, Autostart, Shortcuts, Hooks, Credentials }
impl Operation {
    pub(crate)fn name(self)->&'static str{match self{Self::Voice=>"voice",Self::Mobile=>"mobile",Self::Autostart=>"autostart",Self::Shortcuts=>"shortcuts",Self::Hooks=>"hooks",Self::Credentials=>"credentials"}}
    fn restored_feature(self)->Option<&'static str>{match self {
        Self::Voice=>Some("voice"),Self::Mobile=>Some("mobile"),Self::Autostart=>Some("autostart"),
        Self::Shortcuts|Self::Hooks|Self::Credentials=>None,
    }}
}
fn checked(operation:Operation,mut provenance:impl FnMut()->Result<(),String>,restored:impl FnOnce(&str)->Result<(),String>)->Result<(),String>{
    provenance()?;
    if let Some(name)=operation.restored_feature(){restored(name)?;}
    // The profile lookup can wait. Never carry the earlier observation over it.
    provenance()
}
pub(crate) fn check(operation:Operation)->Result<(),String>{
    checked(operation,||super::effects::optional_effect_allowed(operation.name()),|name|{
        #[cfg(target_os="linux")] { crate::pet_data::bootstrap::feature_allowed(name) }
        #[cfg(not(target_os="linux"))] { let _=name;Ok(()) }
    })
}
/// For a native operation whose origin was captured before waiting. This uses
/// the original owner/witness, never a fresh global observation as a renewal.
pub(crate) fn check_origin(operation:Operation,origin:&super::CapturedOrigin)->Result<(),String>{
    checked(operation,||origin.operation(operation.name()),|name|{
        #[cfg(target_os="linux")]{crate::pet_data::bootstrap::feature_allowed(name)}
        #[cfg(not(target_os="linux"))]{let _=name;Ok(())}
    })
}
#[cfg(all(test,target_os="linux"))]
#[path="settings_optional_tests.rs"]mod tests;
