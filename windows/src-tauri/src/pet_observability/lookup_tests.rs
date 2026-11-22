use super::{core::*, lookup::*};

fn id(n:u32)->String {format!("00000000-0000-0000-0000-{n:012}")}
fn request()->LookupRequest {LookupRequest{provider:"codex".into(),native_session_id:id(40)}}
fn binding(owner:&str)->PetBinding {PetBinding{pet_id:"agent_codex".into(),provider:"codex".into(),owner:owner.into(),enabled:true}}
fn library()->LibraryCapture {LibraryCapture{ownership_complete:true,..LibraryCapture::default()}}
fn hint(owner:&str,cid:&str)->Ownership {Ownership{backend:Some("codex".into()),session_id:id(40),pet_id:Some("agent_codex".into()),owner:owner.into(),conversation_id:cid.into()}}
fn row()->SessionRow {SessionRow{key:"native-40".into(),provider:"codex".into(),pet_id:Some("agent_codex".into()),origin:Origin::External,session_id:Some(id(40)),conversation_id:None,title:"Synthetic session".into(),cwd:None,state:Activity::Unknown,archived:None,updated_at:None,observed_at:Some(100),open:Some(OpenTarget::External{pet_id:"agent_codex".into(),provider:"codex".into(),session_id:id(40)})}}
fn native()->InventoryInput {InventoryInput{scope:"codexDaemon".into(),coverage:Coverage::Complete,observed_at:Some(100),reason:None,rows:vec![row()]}}

#[test]
fn full_ownership_index_resolves_row40_even_when_display_has_twelve_rows() {
    let mut lib=library();
    for n in 1..=40 {let mut h=hint("codex",&format!("conversation-{n}"));h.session_id=id(n);lib.ownership.push(h);}
    let visible=Inventory::build(InventoryInput{rows:(1..=40).map(|n|{let mut r=row();r.key=format!("row-{n}");r.session_id=Some(id(n));r}).collect(),..native()},100,15,false);
    assert_eq!(visible.sessions.len(),12);assert!(visible.truncated);
    assert!(!needs_native(&request(),&lib));
    let result=resolve(request(),&[binding("codex")],&lib,None,100);
    assert_eq!(result.state,LookupState::Resolved);assert_eq!(result.origin,Origin::Mochi);
    assert_eq!(result.open,Some(OpenTarget::Mochi{pet_id:"agent_codex".into(),provider:"codex".into(),conversation_id:"conversation-40".into()}));
}

#[test]
fn legacy_ambiguous_and_incomplete_ownership_never_fall_back_to_native() {
    let mut lib=library();lib.ownership=vec![hint("codex","a"),hint("codex","b")];
    let result=resolve(request(),&[binding("codex")],&lib,Some(&native()),100);
    assert_eq!(result.state,LookupState::Ambiguous);assert!(result.open.is_none());assert!(!needs_native(&request(),&lib));
    lib.ownership=vec![hint("codex","a")];lib.ownership[0].backend=None;
    assert_eq!(resolve(request(),&[binding("codex")],&lib,Some(&native()),100).reason.as_deref(),Some("unknownLibraryBackend"));
    lib.ownership.clear();lib.ownership_complete=false;
    assert!(!needs_native(&request(),&lib));
    let result=resolve(request(),&[binding("codex")],&lib,Some(&native()),100);
    assert_eq!(result.state,LookupState::Unavailable);assert!(result.open.is_none());
}

#[test]
fn retired_spoty_keeps_historical_owner_without_an_external_or_llm_target() {
    let mut lib=library();lib.ownership=vec![hint("integration_spotify","old-conversation")];
    let result=resolve(request(),&[binding("codex")],&lib,Some(&native()),100);
    assert_eq!(result.state,LookupState::Unavailable);assert_eq!(result.origin,Origin::Mochi);
    assert_eq!(result.owner.as_deref(),Some("integration_spotify"));assert_eq!(result.conversation_id.as_deref(),Some("old-conversation"));assert!(result.open.is_none());
}

#[test]
fn logical_owner_namespace_and_duplicates_preserve_the_exact_mochi_target() {
    let mut lib=library();lib.ownership=vec![hint("integration_github","github-conversation"),hint("integration_github","github-conversation")];
    let mut b=binding("integration_github");b.pet_id="integration_github".into();
    for h in &mut lib.ownership {h.pet_id=Some(b.pet_id.clone());}
    let result=resolve(request(),&[b.clone(),b],&lib,Some(&InventoryInput::unavailable("codexDaemon","offline")),100);
    assert_eq!(result.open,Some(OpenTarget::Mochi{pet_id:"integration_github".into(),provider:"integration_github".into(),conversation_id:"github-conversation".into()}));
    // A known other backend with the same ID does not claim this namespace.
    for h in &mut lib.ownership {h.backend=Some("copilot".into());}
    assert!(needs_native(&request(),&lib));
    assert_eq!(resolve(request(),&[binding("codex")],&lib,Some(&native()),100).origin,Origin::External);
}

#[test]
fn revoked_remapped_or_conflicting_bindings_cannot_open_or_prove_absence() {
    for rows in [vec![row()],vec![]] {
        let input=InventoryInput{rows,..native()};
        let mut disabled=binding("codex");disabled.enabled=false;
        let mut remapped=binding("codex");remapped.provider="copilot".into();
        for bindings in [vec![],vec![disabled],vec![remapped],vec![binding("codex"),binding("integration_github")]] {
            let result=resolve(request(),&bindings,&library(),Some(&input),100);
            assert_ne!(result.state,LookupState::Resolved);assert_ne!(result.state,LookupState::NotFound);assert!(result.open.is_none());
        }
    }
    let mut lib=library();lib.ownership=vec![hint("codex","one")];
    let mut conflicting=hint("codex","one");conflicting.pet_id=Some("another-pet".into());lib.ownership.push(conflicting);
    assert_eq!(resolve(request(),&[binding("codex")],&lib,None,100).state,LookupState::Ambiguous);
}

#[test]
fn stale_partial_native_positive_is_navigation_without_a_liveness_claim() {
    for (coverage,at) in [(Coverage::Partial,100),(Coverage::Complete,70)] {
        let input=InventoryInput{coverage,observed_at:Some(at),..native()};
        let result=resolve(request(),&[binding("codex")],&library(),Some(&input),100);
        assert_eq!(result.state,LookupState::Resolved);assert_eq!(result.native.coverage,coverage);
        assert_eq!(result.native.stale,at==70);assert_eq!(result.open,row().open);
        let encoded=serde_json::to_value(result).unwrap();assert!(encoded.get("capabilities").is_none());assert!(encoded.get("busy").is_none());
    }
}

#[test]
fn scoped_not_found_requires_complete_fresh_expected_source() {
    let empty=InventoryInput{rows:vec![],..native()};
    assert_eq!(resolve(request(),&[binding("codex")],&library(),Some(&empty),100).state,LookupState::NotFound);
    for input in [InventoryInput{coverage:Coverage::Partial,..empty.clone()},InventoryInput{observed_at:Some(70),..empty.clone()},InventoryInput{observed_at:Some(101),..empty.clone()},InventoryInput{reason:Some("refreshFailed".into()),..empty.clone()},InventoryInput{scope:"providerGlobal".into(),..empty.clone()}] {
        let result=resolve(request(),&[binding("codex")],&library(),Some(&input),100);
        assert_eq!(result.state,LookupState::Unavailable);assert!(result.open.is_none());
    }
}

#[test]
fn contradictory_rows_or_missing_history_capability_do_not_resolve() {
    let original=row();let mut contradictory=original.clone();contradictory.title="Different metadata".into();
    let input=InventoryInput{rows:vec![original.clone(),contradictory],..native()};
    assert_eq!(resolve(request(),&[binding("codex")],&library(),Some(&input),100).state,LookupState::Ambiguous);
    let input=InventoryInput{rows:vec![original.clone(),original],..native()};
    assert_eq!(resolve(request(),&[binding("codex")],&library(),Some(&input),100).state,LookupState::Resolved);
    let mut no_history=row();no_history.open=None;
    assert_eq!(resolve(request(),&[binding("codex")],&library(),Some(&InventoryInput{rows:vec![no_history],..native()}),100).state,LookupState::Unavailable);
}

#[test]
fn invalid_ids_are_not_normalized_and_invalid_library_cids_block_fallback() {
    for session in [format!(" {}",id(40)),id(40).replace('-',"_"),"tree-name".into()] {
        let request=LookupRequest{native_session_id:session,..request()};
        assert!(!valid_request(&request));assert_eq!(resolve(request,&[binding("codex")],&library(),Some(&native()),100).state,LookupState::InvalidRequest);
    }
    for cid in ["invalid_id","invalid.id","invalid:id"] {
        let mut lib=library();lib.ownership=vec![hint("codex",cid)];
        let result=resolve(request(),&[binding("codex")],&lib,Some(&native()),100);
        assert_eq!(result.state,LookupState::Unavailable);assert!(result.open.is_none());assert!(!needs_native(&request(),&lib));
    }
}

#[test]
fn unsupported_native_provider_can_still_resolve_its_proven_library_owner() {
    let request=LookupRequest{provider:"antigravity".into(),native_session_id:"native-gravity-id".into()};
    let mut lib=library();let mut h=hint("antigravity","gravity-conversation");h.backend=Some(request.provider.clone());h.session_id=request.native_session_id.clone();h.pet_id=Some("agent_antigravity".into());lib.ownership=vec![h];
    let b=PetBinding{pet_id:"agent_antigravity".into(),provider:"antigravity".into(),owner:"antigravity".into(),enabled:true};
    assert_eq!(resolve(request.clone(),&[b],&lib,None,100).state,LookupState::Resolved);
    assert_eq!(resolve(request,&[],&library(),None,100).state,LookupState::NotSupported);
}
