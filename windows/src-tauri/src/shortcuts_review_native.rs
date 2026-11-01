use super::*;
use crate::portal::{Bound,review::Outcome};

fn pending_report(stored:&Bindings)->Vec<ActionStatus>{plan(stored,typed_character).into_iter().map(|(def,result)|result.err().unwrap_or(ActionStatus{id:def.id,status:Status::Pending,typed:None,trigger:None})).collect()}
fn unconfirmed(app:&AppHandle){if let Some(registry)=app.try_state::<Registry>(){registry.invalidate_review();for r in registry.by_id.lock().unwrap().values_mut(){r.published=false;}for s in registry.report.lock().unwrap().iter_mut(){if matches!(s.status,Status::Pending|Status::Active){s.status=Status::PreferencesUnconfirmed;s.trigger=None;}}*registry.portal.lock().unwrap()=None;}let _=app.emit("shortcuts-status",status(app));}

fn cleanup_portal(app:&AppHandle,generation:u64)->Cleanup{
    unconfirmed(app);
    let registry=app.state::<Registry>();
    let reply={let link=registry.wayland.lock().unwrap();if let super::super::wayland::Link::Running{handle,..}=&*link{handle.cleanup_published(generation)}else{None}};
    let acknowledged=reply.and_then(|rx|rx.recv().ok()).unwrap_or(false);
    Cleanup{attempted:true,acknowledged,reason:"owned portal session close; not rollback of prior effects"}
}
fn cleanup_plugin(app:&AppHandle,keys:&[(&'static str,Shortcut)],proof:Closed)->Cleanup{
    unconfirmed(app);let registry=app.state::<Registry>();let Some(gs)=app.try_state::<GlobalShortcut<tauri::Wry>>()else{return Cleanup{attempted:!keys.is_empty(),acknowledged:keys.is_empty(),reason:"native plugin unavailable; held entries preserved"}};
    let keys:Vec<_>=keys.iter().map(|(_,key)|(key.id(),*key)).collect();let result=release_owned(&registry,&keys,||true,|key|gs.unregister(key).is_ok());
    let mut proof=proof;proof.complete=result.complete;*registry.closed.lock().unwrap()=Some(proof);
    Cleanup{attempted:result.started,acknowledged:result.complete,reason:"only reviewed potentially affected owned keys released; failed entries retained"}
}

pub(super)fn execute(app:&AppHandle,capture:&Capture,guard:Guard,lifetime:Guard)->NativeRun{
    if !guard()||!(capture.native.current)(){return NativeRun::refused("native scope or review changed before effect");}
    match &capture.native.destination{
        Destination::Portal{reservation,..}=>portal(app,capture,reservation,guard,lifetime),
        Destination::Plugin{previous,environment,..}=>plugin(app,capture,previous,environment,guard,lifetime),
    }
}
fn portal(app:&AppHandle,capture:&Capture,reservation:&crate::portal::Reservation,guard:Guard,lifetime:Guard)->NativeRun{
    let registry=app.state::<Registry>();let old_report=registry.report.lock().unwrap().clone();
    let old_portal=*registry.portal.lock().unwrap();
    let previous=reservation.generation-1;
    let reply={let mut link=registry.wayland.lock().unwrap();
        let super::super::wayland::Link::Running{handle,gen}=&mut *link else{return NativeRun::refused("portal worker unavailable")};
        if *gen!=previous||!guard(){return NativeRun::refused("portal generation changed")}
        let Some(reply)=handle.apply_reserved(reservation.clone(),guard,lifetime)else{return NativeRun::refused("portal worker stopped before dispatch")};
        *gen=reservation.generation;*registry.report.lock().unwrap()=pending_report(&capture.preview.desired_bindings);*registry.portal.lock().unwrap()=Some(PortalState::Pending);registry.invalidate_review();reply
    };
    let began=registry.revision.load(Ordering::Acquire);let outcome=reply.recv().unwrap_or_else(|_|Outcome::refused("portal worker ended without an outcome; dispatched state preserved",true));
    let generation=reservation.generation;let started=outcome.native_effects_started;let app=app.clone();
    NativeRun{outcome,finish:Some(Box::new(move|source,lifetime|{
        let registry=app.state::<Registry>();let commit=source.map(|source|{let lifetime=lifetime.clone();Arc::new(move||lifetime()&&source.current())as Guard});
        let reply={let link=registry.wayland.lock().unwrap();if let super::super::wayland::Link::Running{handle,..}=&*link{handle.finish_reserved(generation,commit)}else{None}};
        let published=reply.and_then(|rx|rx.recv().ok()).unwrap_or(false);
        if !published&&!started&&registry.revision.load(Ordering::Acquire)==began{
            let mut link=registry.wayland.lock().unwrap();if let super::super::wayland::Link::Running{gen,..}=&mut *link{if *gen==generation{*gen=previous;*registry.report.lock().unwrap()=old_report;*registry.portal.lock().unwrap()=old_portal;}}
            drop(link);let _=app.emit("shortcuts-status",status(&app));
        }else if !published{unconfirmed(&app);}
        if !published{return Publication::refused(started.then_some(Cleanup{attempted:true,acknowledged:false,reason:"native attempt aborted; cleanup acknowledgement not observed"}));}
        let expected=registry.revision.load(Ordering::Acquire);let current_app=app.clone();
        let current:Guard=Arc::new(move||{let registry=current_app.state::<Registry>();registry.revision.load(Ordering::Acquire)==expected
            &&registry.wayland.try_lock().is_ok_and(|link|matches!(&*link,super::super::wayland::Link::Running{gen,..}if *gen==generation))
            &&*registry.portal.lock().unwrap()==Some(PortalState::Active)});
        Publication::live(current,move||cleanup_portal(&app,generation))
    }))}
}
fn plugin(app:&AppHandle,capture:&Capture,previous:&[Held],environment:&Environment,guard:Guard,_lifetime:Guard)->NativeRun{
    let registry=app.state::<Registry>();if held(&registry)!=previous||Environment::capture().ok().as_ref()!=Some(environment)||!guard(){return NativeRun::refused("plugin scope changed before registration")}
    let Some(gs)=app.try_state::<GlobalShortcut<tauri::Wry>>()else{return NativeRun::refused("native plugin unavailable")};
    if !guard(){return NativeRun::refused("review revoked before releasing previous keys")}
    // No input publication can race an unconfirmed registration. The physical
    // gate is held by the caller; native callbacks never take that gate.
    for r in registry.by_id.lock().unwrap().values_mut(){r.published=false;}
    registry.invalidate_review();*registry.report.lock().unwrap()=pending_report(&capture.preview.desired_bindings);*registry.portal.lock().unwrap()=None;
    let proof=Closed{plan:capture.preview.fingerprint.clone(),instance:match &capture.native.destination{Destination::Plugin{instance,..}=>instance.clone(),_=>unreachable!()},effect:capture.preview.token.clone(),complete:false};
    let mut affected:Vec<_>={let rows=registry.by_id.lock().unwrap();rows.values().map(|r|(r.action,r.shortcut)).collect()};
    let previous_keys=owned_keys(&registry);let removed=release_owned(&registry,&previous_keys,||guard(),|key|gs.unregister(key).is_ok());
    let mut started=removed.started;let mut registered=vec![];let mut reason=if removed.complete{String::new()}else{"previous plugin release was not confirmed or review revoked".into()};
    if removed.complete{
        for(def,result)in plan(&capture.preview.desired_bindings,typed_character){if let Ok(shortcut)=result{
            if !guard(){reason="review revoked during native registration".into();break;}
            if !affected.iter().any(|(_,key)|*key==shortcut){affected.push((def.id,shortcut));}
            started=true;match gs.register(shortcut){Ok(())=>{registry.by_id.lock().unwrap().insert(shortcut.id(),Registered{action:def.id,shortcut,published:false});registered.push((def.id,shortcut));},Err(_)=>{reason="a native key could not be registered; partial state preserved".into();break;}}
        }}
    }
    let verified=reason.is_empty()&&guard()&&registered.iter().all(|(_,key)|gs.is_registered(*key));
    if !verified&&reason.is_empty(){reason="native postcondition or source no longer confirmed".into();}
    let observed=registered.iter().map(|(id,key)|Bound{id:(*id).into(),trigger:Some(key.to_string())}).collect();let expected=registry.revision.load(Ordering::Acquire);
    let app=app.clone();let environment=environment.clone();
    NativeRun{outcome:Outcome{verified,native_effects_started:started,reason:if verified{"native keys observed; durable receipt pending".into()}else{reason},session:None,shortcuts:Some(observed)},finish:Some(Box::new(move|source,lifetime|{
        let registry=app.state::<Registry>();let Some(gs)=app.try_state::<GlobalShortcut<tauri::Wry>>()else{return Publication::refused(None)};
        let ok=source.as_ref().is_some_and(|source|source.current())&&lifetime()&&registry.revision.load(Ordering::Acquire)==expected&&Environment::capture().ok().as_ref()==Some(&environment)&&registered.iter().all(|(_,key)|gs.is_registered(*key));
        if ok{
            for r in registry.by_id.lock().unwrap().values_mut(){r.published=true;}
            for s in registry.report.lock().unwrap().iter_mut(){if s.status==Status::Pending{s.status=Status::Active;s.trigger=registered.iter().find(|(id,_)|*id==s.id).map(|(_,key)|key.to_string());}}
            let _=app.emit("shortcuts-status",status(&app));
            let current_app=app.clone();let current_keys=registered.clone();
            let current:Guard=Arc::new(move||{
                let registry=current_app.state::<Registry>();
                if registry.revision.load(Ordering::Acquire)!=expected||Environment::capture().ok().as_ref()!=Some(&environment){return false;}
                let held={let rows=registry.by_id.lock().unwrap();current_keys.iter().all(|(_,key)|rows.get(&key.id()).is_some_and(|r|r.published&&r.shortcut==*key))};
                held&&current_app.try_state::<GlobalShortcut<tauri::Wry>>().is_some_and(|gs|current_keys.iter().all(|(_,key)|gs.is_registered(*key)))&&registry.revision.load(Ordering::Acquire)==expected
            });
            Publication::live(current,move||cleanup_plugin(&app,&affected,proof))
        }else{
            Publication::refused(Some(cleanup_plugin(&app,&affected,proof)))
        }
    }))}
}
