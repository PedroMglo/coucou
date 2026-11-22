//! A distinct native-user attempt, not replay or proof of old-scope closure.
use super::*;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
struct PriorEffect{slot:components::Slot,original:reconciliation::NativeSubject,scope_closure_known:bool}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub(super)struct Review{
    protocol:u32,prior_effects:Vec<PriorEffect>,prior_scope_closure_claimed:bool,
    only_current_owned_registrar_operations:bool,historical_targets_used_as_addresses:bool,
    portal_application_bindings_may_be_shared:bool,
}
pub(super)fn capture(settings:&Settings,path:&Path,native:&Destination)->Result<Review,String>{
    let prior=components::shortcut_prior_artifacts(settings,path).map_err(|e|e.to_string())?;
    if prior.is_empty(){return Err("Não há um registo anterior para recuperar. Usa a revisão normal dos atalhos.".into());}
    let prior_effects=prior.into_iter().map(|(slot,intent)|Ok(PriorEffect{original:reconciliation::original(&slot,&intent)?,slot,scope_closure_known:false})).collect::<Result<_,String>>()?;
    Ok(Review{protocol:1,prior_effects,prior_scope_closure_claimed:false,only_current_owned_registrar_operations:true,historical_targets_used_as_addresses:false,portal_application_bindings_may_be_shared:matches!(native,Destination::Portal{..})})
}
