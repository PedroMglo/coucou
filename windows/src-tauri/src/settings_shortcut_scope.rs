//! Acknowledged scope uncertainty is retained in the same hash-linked chain.
//! It is not reconciliation, a grant, or evidence of physical independence.
use super::*;
pub(super)fn validate(intent:&Artifact,prior:&[Slot])->io::Result<bool>{
    if intent.component!="shortcuts"{return Ok(false)}
    let review=&intent.evidence["review"];
    let tagged=review["schemaVersion"]==3||review["scope"]=="nativeUserShortcutsCurrentScopeOnly"||!review["recovery"].is_null();
    if !tagged{return Ok(false)}
    let recovery=&review["recovery"];
    let rows=recovery["priorEffects"].as_array().ok_or_else(||error("componentShortcutScopeHistory"))?;
    let slots=rows.iter().map(|r|serde_json::from_value::<Slot>(r["slot"].clone()).map_err(|_|error("componentShortcutScopeHistory"))).collect::<io::Result<Vec<_>>>()?;
    if intent.component!="shortcuts"||intent.phase!=Phase::Prepared||review["schemaVersion"]!=3
        ||review["scope"]!="nativeUserShortcutsCurrentScopeOnly"||review["token"]!=intent.effect
        ||review["fingerprint"]!=intent.binding.plan||review["configuration"]!=intent.binding.configuration
        ||review["resource"]!=intent.binding.resource||review["origin"]["profile"]!=intent.binding.profile
        ||prior.is_empty()||prior.len()>64||slots!=prior
        ||review["origin"]["slot"]!=serde_json::to_value(&prior[0]).map_err(|_|error("componentShortcutScopeHistory"))?
        ||recovery["protocol"]!=1||recovery["priorScopeClosureClaimed"]!=false
        ||recovery["onlyCurrentOwnedRegistrarOperations"]!=true||recovery["historicalTargetsUsedAsAddresses"]!=false
        ||recovery["portalApplicationBindingsMayBeShared"]!=serde_json::json!(review["destination"]["kind"]=="portal")
        ||["reviewedPlan","understandNativeEffects","keepOtherComponentsSuspended","acknowledgeRetainedUncertainty"].iter().any(|key|intent.evidence[*key]!=true)
        ||rows.iter().any(|r|r["scopeClosureKnown"]!=false||!r["original"].is_object()){
        return Err(error("componentShortcutScopeReviewRequired"));
    }
    Ok(true)
}
