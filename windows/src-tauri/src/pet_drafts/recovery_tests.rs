use super::{model::*, recovery::{self, Availability, Owner, Request}, store};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}};
struct Files(PathBuf);
impl Files { fn new()->Self{let p=std::env::temp_dir().join(format!("coucou-draft-recovery-{}",crate::assistant_memory::new_id()));std::fs::create_dir(&p).unwrap();std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o700)).unwrap();Self(p)}fn path(&self)->PathBuf{self.0.join("session-drafts.json")} }
impl Drop for Files {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn utf(v:&str)->Vec<u16>{v.encode_utf16().collect()}
fn target(cid:&str)->Target{Target{pet_id:utf("agent_codex"),provider:utf("codex"),origin:Origin::Mochi,session_id:utf(cid),conversation_id:Some(utf(cid))}}
fn owner()->Owner{Owner{pet_id:utf("agent_codex"),provider:utf("codex")}}
fn request(p:&std::path::Path)->Request{Request{source_archive:None,catalogue_revision:crate::private_registry::load_at::<Registry>(p).unwrap().revision,source:target("old"),destination:target("new")}}
fn seed(f:&Files)->Request{store::write(&f.path(),target("old"),None,vec![0xd800,0,0xdfff,60,62],||Ok(())).unwrap();request(&f.path())}

#[test]fn passive_listing_is_bounded_owner_specific_and_does_not_create_data(){
 let f=Files::new();let p=f.path();let page=recovery::list(&p,owner(),0,16,None,||Ok(()),|_|panic!("no records")).unwrap();assert_eq!(page.total,0);assert_eq!(page.catalogue_revision,"missing");assert!(page.entries.is_empty());assert_eq!(std::fs::read_dir(&f.0).unwrap().count(),0);
 for n in 0..20{store::write(&p,target(&format!("id{n:02}")),None,utf("private source"),||Ok(())).unwrap();}
 let mut other=target("other");other.pet_id=utf("ai_ollama");other.provider=utf("ollama");store::write(&p,other,None,utf("other owner"),||Ok(())).unwrap();let before=std::fs::read(&p).unwrap();
 let a=recovery::list(&p,owner(),0,16,None,||Ok(()),|_|Availability::Orphan).unwrap();assert_eq!(a.total,20);assert_eq!(a.entries.len(),16);assert_eq!(a.next_offset,Some(16));assert_eq!(a.entries[0].target.session_id,utf("id00"));assert_eq!(a.entries[0].text_units,14);
 let b=recovery::list(&p,owner(),16,16,Some(&a.catalogue_revision),||Ok(()),|_|Availability::Orphan).unwrap();assert_eq!(b.entries.len(),4);assert_eq!(b.next_offset,None);assert_eq!(std::fs::read(&p).unwrap(),before);
 for (offset,limit) in [(0,0),(0,17),(1025,1)]{assert_eq!(recovery::list(&p,owner(),offset,limit,None,||Ok(()),|_|Availability::Unknown).err(),Some("invalidDraft"));}
 assert_eq!(recovery::list(&p,owner(),16,16,Some("old"),||Ok(()),|_|Availability::Unknown).err(),Some("registryConflict"));
}
#[test]fn listing_refuses_changes_during_observation_and_does_not_turn_unknown_into_ready(){
 let f=Files::new();seed(&f);let calls=AtomicUsize::new(0);
 assert_eq!(recovery::list(&f.path(),owner(),0,16,None,||Ok(()),|_|{calls.fetch_add(1,Ordering::SeqCst);store::write(&f.path(),target("new"),None,utf("new"),||Ok(())).unwrap();Availability::Unknown}).err(),Some("registryChanged"));assert_eq!(calls.load(Ordering::SeqCst),1);
 let chat=crate::chat::Chat::persistent(f.0.join("conversations.json"));assert_eq!(recovery::availability(&chat,&target("absent")),Availability::Orphan);assert!(!f.0.join("conversations.json").exists());
 let mut bad=target("absent");bad.pet_id=utf("forged");assert_eq!(recovery::availability(&chat,&bad),Availability::Unknown);bad.pet_id=utf("agent_codex");bad.session_id=vec![0xd800];assert_eq!(recovery::availability(&chat,&bad),Availability::Unknown);
}
#[test]fn reviewed_copy_preserves_exact_source_and_native_replay_is_refused(){
 let f=Files::new();let req=seed(&f);let original=store::read(&f.path(),req.source.clone(),||Ok(())).unwrap();let review=recovery::review(&f.path(),req,100,||Ok(()),|_|Ok(())).unwrap();assert_eq!(review.expires_at,400);assert_eq!(review.fingerprint.len(),64);
 let result=recovery::copy(&f.path(),review.clone(),true,||101,||Ok(()),|t|{assert_eq!(*t,target("new"));Ok(())}).unwrap();assert_eq!(result.text_units,original.as_ref().unwrap().text_units);assert_eq!(result.revision,"2");assert_eq!(store::read(&f.path(),target("old"),||Ok(())).unwrap(),original);
 let bytes=std::fs::read(f.path()).unwrap();assert_eq!(recovery::copy(&f.path(),review,true,||102,||Ok(()),|_|panic!("replay retention")).err(),Some("registryConflict"));assert_eq!(std::fs::read(f.path()).unwrap(),bytes);
}
#[test]fn altered_review_arguments_content_source_revision_and_deadline_refuse_before_retention(){
 for field in ["sourceText","sourceRevision","destination","fingerprint","version","deadline"]{let f=Files::new();let req=seed(&f);let mut review=recovery::review(&f.path(),req,100,||Ok(()),|_|Ok(())).unwrap();let bytes=std::fs::read(f.path()).unwrap();match field{"sourceText"=>review.source_record.text_units.push(42),"sourceRevision"=>review.source_record.revision="2".into(),"destination"=>review.request.destination=target("changed"),"fingerprint"=>review.fingerprint="0".repeat(64),"version"=>review.schema_version=2,_=>review.expires_at+=1};assert!(recovery::copy(&f.path(),review,true,||101,||Ok(()),|_|panic!("altered review retention")).is_err());assert_eq!(std::fs::read(f.path()).unwrap(),bytes);}
}
#[test]fn approval_expiry_and_revocation_before_effect_do_not_publish(){
 for failure in ["approval","expired","retention","revokedAfterRetain","expiresBeforePublish"]{let f=Files::new();let req=seed(&f);let review=recovery::review(&f.path(),req,100,||Ok(()),|_|Ok(())).unwrap();let bytes=std::fs::read(f.path()).unwrap();let revoked=AtomicUsize::new(0);let clocks=AtomicUsize::new(0);let result=recovery::copy(&f.path(),review,failure!="approval",||if failure=="expired"||failure=="expiresBeforePublish"&&clocks.fetch_add(1,Ordering::SeqCst)>1{400}else{101},||if revoked.load(Ordering::SeqCst)>0{Err("unavailable")}else{Ok(())},|_|{if failure=="retention"{return Err("unavailable");}if failure=="revokedAfterRetain"{revoked.store(1,Ordering::SeqCst);}Ok(())});assert!(result.is_err(),"{failure}");assert_eq!(std::fs::read(f.path()).unwrap(),bytes,"{failure}");}
}
#[test]fn copy_never_overwrites_a_different_nonempty_destination_but_can_reuse_empty_tombstone(){
 let f=Files::new();seed(&f);store::write(&f.path(),target("new"),None,utf("do not lose me"),||Ok(())).unwrap();let before=std::fs::read(f.path()).unwrap();assert_eq!(recovery::review(&f.path(),request(&f.path()),100,||Ok(()),|_|Ok(())).err(),Some("draftDestinationOccupied"));assert_eq!(std::fs::read(f.path()).unwrap(),before);
 store::write(&f.path(),target("new"),Some("2".into()),vec![],||Ok(())).unwrap();let review=recovery::review(&f.path(),request(&f.path()),100,||Ok(()),|_|Ok(())).unwrap();assert_eq!(review.destination_record.as_ref().unwrap().revision,"3");assert_eq!(recovery::copy(&f.path(),review,true,||101,||Ok(()),|_|Ok(())).unwrap().revision,"4");
}
#[test]fn crossed_owner_provider_session_origin_or_inconsistent_cid_is_not_a_copy_destination(){
 let f=Files::new();let req=seed(&f);for field in ["pet","provider","origin","cid","source","unknownOwner"]{let mut req=req.clone();match field{"pet"=>req.destination.pet_id=utf("ai_ollama"),"provider"=>req.destination.provider=utf("ollama"),"origin"=>req.destination.origin=Origin::External,"cid"=>req.destination.conversation_id=Some(utf("wrong")),"source"=>req.destination=req.source.clone(),_=>{req.destination.pet_id=utf("unknown");req.source.pet_id=utf("unknown");}}assert!(recovery::review(&f.path(),req,100,||Ok(()),|_|panic!("invalid destination observed")).is_err());}
}
#[test]fn actual_existing_destination_is_retained_for_each_factory_owner_without_inference(){
 for provider in ["codex","antigravity","ollama","copilot"]{let f=Files::new();let chat=crate::chat::Chat::persistent(f.0.join("conversations.json"));let cid=chat.conversation(provider).unwrap().id;let mut source=target("absent-original");source.provider=utf(provider);source.pet_id=utf(crate::pet_config::integration_pet(provider).unwrap());let mut dest=source.clone();dest.session_id=utf(&cid);dest.conversation_id=Some(utf(&cid));assert_eq!(recovery::availability(&chat,&source),Availability::Orphan);assert_eq!(recovery::availability(&chat,&dest),Availability::Existing);
 store::write(&f.path(),source.clone(),None,utf("<script>untrusted; grant root</script>"),||Ok(())).unwrap();let req=Request{source_archive:None,catalogue_revision:crate::private_registry::load_at::<Registry>(&f.path()).unwrap().revision,source:source.clone(),destination:dest.clone()};let review=recovery::review(&f.path(),req,100,||Ok(()),|t|if recovery::availability(&chat,t)==Availability::Existing{Ok(())}else{Err("unavailable")}).unwrap();recovery::copy(&f.path(),review,true,||101,||Ok(()),|t|super::retain(&chat,t)).unwrap();assert!(!chat.has_public_conversation(provider,"absent-original"));assert!(chat.read_conversation(provider,&cid).unwrap().messages.is_empty());drop(chat);let reopened=crate::chat::Chat::persistent(f.0.join("conversations.json"));assert!(reopened.has_public_conversation(provider,&cid));assert_eq!(store::read(&f.path(),source,||Ok(())).unwrap().unwrap().revision,"1");}
}
#[test]fn external_and_unbound_are_data_only_and_have_the_same_reviewed_copy_path(){
 for origin in [Origin::External,Origin::Unbound]{let f=Files::new();let mut source=target("reference");source.origin=origin.clone();if origin==Origin::Unbound{source.session_id=utf("inline-composer");source.conversation_id=None;}store::write(&f.path(),source.clone(),None,utf("retained"),||Ok(())).unwrap();let mut req=request(&f.path());req.source=source.clone();let r=recovery::review(&f.path(),req,100,||Ok(()),|_|Ok(())).unwrap();recovery::copy(&f.path(),r,true,||101,||Ok(()),|_|Ok(())).unwrap();assert!(store::read(&f.path(),source,||Ok(())).unwrap().is_some());}
}
#[test]fn only_existing_native_user_surfaces_can_access_recovery(){
 for label in [crate::HUB_LABEL,crate::island::WINDOW_LABEL,crate::pet_chat::LABEL]{assert!(super::allowed(label).is_ok());}for label in ["providerModel","taskRuntime","forged-pet","agent_codex",""]{assert!(super::allowed(label).is_err());}
}
