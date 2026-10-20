use super::*;
use serde_json::json;

fn environment(config: Config) -> (Service, SystemSetting, SystemWindowAuthority, Instant) {
    let setting = SystemSetting::from_backend(true);
    let window = SystemWindowAuthority::from_backend(WindowKind::System, true, &setting);
    let service = Service::new(config, 17, &setting).unwrap();
    (service, setting, window, Instant::now())
}
fn peer(n: u8) -> ProducerId { ProducerId::from_verified_backend([n;16]).unwrap() }
fn target(kind: TargetKind, key: &str, label: &str) -> NativeTarget {
    NativeTarget::from_verified_backend(kind,key,label).unwrap()
}
fn targets() -> Vec<NativeTarget> {
    vec![target(TargetKind::WifiDevice,"/org/freedesktop/NetworkManager/Devices/1","wlan0"),
         target(TargetKind::AccessPoint,"/org/freedesktop/NetworkManager/AccessPoint/3","Mesmo nome"),
         target(TargetKind::AccessPoint,"/org/freedesktop/NetworkManager/AccessPoint/4","Mesmo nome"),
         target(TargetKind::VpnProfile,"12345678-1234-1234-1234-123456789abc","VPN")]
}
fn observation(producer: ProducerId, items: Vec<NativeTarget>, support: ScanSupport) -> NativeObservation {
    NativeObservation { producer,api_version:Some(34),capabilities:vec!["network".into()],
        result:json!({"wifiEnabled":true,"wifiConnected":false,"wifiSignal":0,
          "wifiNetworks":[{"ssid":"Mesmo nome","connected":false},{"ssid":"Mesmo nome","connected":true}],
          "wifiDevices":[{"name":"wlan0"}],"savedWifiNetworks":[],"vpnProfiles":[{"name":"VPN"}]}),
        targets:items,scan_support:support,observed_at_ms:1_792_500_000_000 }
}
fn populate(service:&Service, window:&SystemWindowAuthority, now:Instant) -> Ticket {
    let admission=service.read(window,true,now).unwrap();
    let mut work=admission.work.unwrap();
    assert_eq!(work.begin_dispatch(peer(1),now),Ok(Operation::ReadNetwork));
    assert_eq!(work.finish_read(observation(peer(1),targets(),ScanSupport::VerifiedByAdapter),now),Ok(()));
    service.ticket(window,&admission.ticket.id,now).unwrap()
}
fn scope(service:&Service,window:&SystemWindowAuthority,now:Instant)->String {
    service.view(window,now).unwrap().scan_scope.unwrap()
}
fn expect<T>(result:Result<T,Reason>,reason:Reason) { assert_eq!(result.err(),Some(reason)); }
fn public<T:Serialize>(v:&T)->Value { serde_json::to_value(v).unwrap() }

#[test]
fn visible_system_and_current_backend_setting_are_required() {
    let (s,setting,w,now)=environment(Config::default());
    let other=SystemWindowAuthority::from_backend(WindowKind::Other,true,&setting);
    let hidden=SystemWindowAuthority::from_backend(WindowKind::System,false,&setting);
    expect(s.read(&other,true,now),Reason::Unauthorized);
    expect(s.read(&hidden,true,now),Reason::Hidden);
    setting.update_from_backend(false);
    expect(s.view(&w,now),Reason::SystemDisabled);
    assert!(s.state.lock().unwrap().flight.is_none());
}
#[test]
fn independent_backend_setting_cannot_authorize_another_service() {
    let (s,_,_,now)=environment(Config::default());
    let unrelated=SystemSetting::from_backend(true);
    let window=SystemWindowAuthority::from_backend(WindowKind::System,true,&unrelated);
    expect(s.read(&window,true,now),Reason::Unauthorized);
    expect(s.extension(&window,FutureOperation::ConnectWifi),Reason::Unauthorized);
}
#[test]
fn hiding_then_showing_never_resurrects_a_queued_lease() {
    let (s,_,w,now)=environment(Config::default());
    let a=s.read(&w,true,now).unwrap();let mut work=a.work.unwrap();
    w.visibility_from_backend(false);w.visibility_from_backend(true);
    assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::Unauthorized));
    let ticket=s.ticket(&w,&a.ticket.id,now).unwrap();
    assert_eq!((ticket.phase,ticket.reason),(Phase::Failed,Some(Reason::Revoked)));
    expect(s.read(&w,true,now),Reason::Busy);
    drop(work);
    assert!(s.read(&w,true,now).unwrap().work.is_some());
}
#[test]
fn close_and_owner_drop_revoke_live_work() {
    let (s,setting,w,now)=environment(Config::default());
    let a=s.read(&w,true,now).unwrap();let mut work=a.work.unwrap();
    w.close_from_backend();expect(s.view(&w,now),Reason::Closed);
    assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::Unauthorized));drop(work);
    let w2=SystemWindowAuthority::from_backend(WindowKind::System,true,&setting);
    let mut work=s.read(&w2,true,now).unwrap().work.unwrap();drop(w2);
    assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::Unauthorized));
}
#[test]
fn setting_epoch_revokes_work_and_previous_cache_handles() {
    let (s,setting,w,now)=environment(Config::default());populate(&s,&w,now);
    let old=scope(&s,&w,now);let mut work=s.read(&w,true,now).unwrap().work.unwrap();
    setting.update_from_backend(false);setting.update_from_backend(true);
    assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::Unauthorized));drop(work);
    let view=s.view(&w,now).unwrap();assert!(view.stale);assert!(view.data.is_some());
    assert_eq!(view.reason,Some(Reason::Revoked));assert!(view.scan_scope.is_none());
    expect(s.scan(&w,&old,"old",now),Reason::StaleTarget);
}
#[test]
fn dropping_service_revokes_outstanding_executor_work() {
    let (s,_,w,now)=environment(Config::default());let mut work=s.read(&w,true,now).unwrap().work.unwrap();
    drop(s);assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::Closed));
}
#[test]
fn read_requests_share_one_physical_work_and_result() {
    let (s,setting,w,now)=environment(Config::default());
    let w2=SystemWindowAuthority::from_backend(WindowKind::System,true,&setting);
    let a=s.read(&w,true,now).unwrap();let b=s.read(&w2,true,now).unwrap();
    assert!(b.work.is_none());assert_ne!(a.ticket.id,b.ticket.id);
    expect(s.ticket(&w2,&a.ticket.id,now),Reason::Unauthorized);
    let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    work.finish_read(observation(peer(1),targets(),ScanSupport::VerifiedByAdapter),now).unwrap();
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Observed);
    assert_eq!(s.ticket(&w2,&b.ticket.id,now).unwrap().phase,Phase::Observed);
    assert!(!s.view(&w2,now).unwrap().stale);
}
#[test]
fn simultaneous_threads_admit_exactly_one_physical_work() {
    let (s,_,w,now)=environment(Config::default());let s=Arc::new(s);let w=Arc::new(w);
    let barrier=Arc::new(std::sync::Barrier::new(3));let mut threads=Vec::new();
    for _ in 0..2 {let s=s.clone();let w=w.clone();let b=barrier.clone();
        threads.push(std::thread::spawn(move||{b.wait();s.read(&w,true,now).unwrap()}));}
    barrier.wait();let mut admissions:Vec<_>=threads.into_iter().map(|t|t.join().unwrap()).collect();
    assert_eq!(admissions.iter().filter(|a|a.work.is_some()).count(),1);
    let mut work=admissions.iter_mut().find_map(|a|a.work.take()).unwrap();
    work.begin_dispatch(peer(1),now).unwrap();
    work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now).unwrap();
    for a in admissions {assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Observed);}
}
#[test]
fn a_read_joining_running_work_is_running_and_cancels_as_unknown() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();
    let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    let joined=s.read(&w,true,now).unwrap();assert!(joined.work.is_none());
    assert_eq!(joined.ticket.phase,Phase::Running);
    let t=s.cancel(&w,&joined.ticket.id,now).unwrap();assert_eq!(t.reason,Some(Reason::CancelledAfterDispatch));
    work.finish_read(observation(peer(1),targets(),ScanSupport::VerifiedByAdapter),now).unwrap();
    assert!(!s.view(&w,now).unwrap().stale);
    assert_eq!(s.ticket(&w,&joined.ticket.id,now).unwrap().phase,Phase::Unknown);
}
#[test]
fn cached_reads_and_repeated_views_do_not_dispatch_or_renew_time() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);
    let first=public(&s.view(&w,now).unwrap());let cached=s.read(&w,false,now+Duration::from_secs(2)).unwrap();
    assert!(cached.work.is_none());assert_eq!(cached.ticket.phase,Phase::Observed);
    let again=public(&s.view(&w,now+Duration::from_secs(3)).unwrap());
    assert_eq!(first["data"]["observedAt"],1_792_500_000_000u64);
    assert_eq!(first["data"],again["data"]);assert_eq!(first["generation"],again["generation"]);
    assert_eq!(first["scanScope"],again["scanScope"]);assert!(s.state.lock().unwrap().flight.is_none());
}
#[test]
fn native_failure_keeps_previous_data_date_and_generation_stale() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let before=public(&s.view(&w,now).unwrap());
    let mut work=s.read(&w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    assert_eq!(work.fail(NativeError::Unavailable,now),Err(Reason::NativeUnavailable));
    let after=public(&s.view(&w,now).unwrap());assert_eq!(before["data"],after["data"]);
    assert_eq!(before["generation"],after["generation"]);assert_eq!(after["stale"],true);
    assert_eq!(after["reason"],"nativeUnavailable");assert!(after["scanScope"].is_null());
}
#[test]
fn expired_cache_is_explicit_and_polling_never_refreshes_it() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let later=now+Duration::from_secs(15);
    let view=s.view(&w,later).unwrap();assert!(view.stale);assert_eq!(view.reason,Some(Reason::CacheExpired));
    assert!(view.data.is_some());assert!(view.targets.is_empty());assert!(s.state.lock().unwrap().flight.is_none());
    assert!(s.read(&w,false,later).unwrap().work.is_some());
}
#[test]
fn target_ttl_never_extends_when_cache_can_live_longer() {
    let mut cfg=Config::default();cfg.target_ttl=Duration::from_secs(2);
    let (s,_,w,now)=environment(cfg);populate(&s,&w,now);let old=scope(&s,&w,now);
    assert_eq!(scope(&s,&w,now+Duration::from_secs(1)),old);
    let later=now+Duration::from_secs(2);let view=s.view(&w,later).unwrap();
    assert!(!view.stale);assert_eq!(view.reason,Some(Reason::StaleTarget));assert!(view.scan_scope.is_none());
    assert!(view.targets.is_empty());assert!(s.read(&w,false,later).unwrap().work.is_none());
    expect(s.scan(&w,&old,"expired",later),Reason::StaleTarget);
}
#[test]
fn deadline_does_not_release_a_physical_worker_or_accept_late_data() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();
    let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();let later=now+Duration::from_secs(3);
    let t=s.ticket(&w,&a.ticket.id,later).unwrap();assert_eq!((t.phase,t.reason),(Phase::Unknown,Some(Reason::Deadline)));
    expect(s.read(&w,true,later),Reason::Busy);
    assert_eq!(work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),later),Err(Reason::Deadline));
    assert!(s.view(&w,later).unwrap().data.is_none());assert!(s.read(&w,true,later).unwrap().work.is_some());
}
#[test]
fn queued_deadline_also_holds_slot_until_guard_ends() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();let mut work=a.work.unwrap();
    let later=now+Duration::from_secs(3);assert_eq!(work.begin_dispatch(peer(1),later),Err(Reason::Deadline));
    let t=s.ticket(&w,&a.ticket.id,later).unwrap();assert_eq!((t.phase,t.reason),(Phase::Failed,Some(Reason::Deadline)));
    expect(s.read(&w,true,later),Reason::Busy);drop(work);assert!(s.read(&w,true,later).unwrap().work.is_some());
}
#[test]
fn dropping_work_distinguishes_before_and_after_actual_dispatch() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();drop(a.work);
    let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Cancelled,Some(Reason::AbandonedBeforeDispatch)));
    let b=s.read(&w,true,now).unwrap();let mut work=b.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();drop(work);
    let t=s.ticket(&w,&b.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Unknown,Some(Reason::UnknownAfterDispatch)));
}
#[test]
fn dispatch_is_once_even_when_second_call_has_a_different_peer() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"once",now).unwrap();let mut work=a.work.unwrap();
    assert_eq!(work.begin_dispatch(peer(1),now),Ok(Operation::ScanWifi));
    assert_eq!(work.begin_dispatch(peer(2),now),Err(Reason::AlreadyDispatched));
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Running);
    work.finish_scan(peer(1),now).unwrap();
}
#[test]
fn cancelling_all_read_receipts_prevents_fresh_cache_publication() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();
    let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();s.cancel(&w,&a.ticket.id,now).unwrap();
    assert_eq!(work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now),Err(Reason::CancelledAfterDispatch));
    assert!(s.view(&w,now).unwrap().data.is_none());
}
#[test]
fn cancelling_one_queued_join_does_not_cancel_other_read_request() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();let b=s.read(&w,true,now).unwrap();
    s.cancel(&w,&a.ticket.id,now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now).unwrap();
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Cancelled);
    assert_eq!(s.ticket(&w,&b.ticket.id,now).unwrap().phase,Phase::Observed);
}
#[test]
fn scan_intent_deduplicates_receipt_and_ack_is_never_effect_confirmation() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"intent-1",now).unwrap();let b=s.scan(&w,&token,"intent-1",now).unwrap();
    assert!(b.work.is_none());assert_eq!(a.ticket.id,b.ticket.id);
    expect(s.scan(&w,"other","intent-1",now),Reason::IntentConflict);
    let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    let during=s.view(&w,now).unwrap();assert!(during.stale);assert!(during.scan_scope.is_none());assert!(during.refreshing);
    expect(s.read(&w,true,now),Reason::Busy);work.finish_scan(peer(1),now).unwrap();
    let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!(t.phase,Phase::Acknowledged);
    assert!(!t.effect_confirmed);assert!(t.refresh_recommended);assert!(s.view(&w,now).unwrap().stale);
    let duplicate=s.scan(&w,&token,"intent-1",now).unwrap();assert_eq!(duplicate.ticket.id,a.ticket.id);assert!(duplicate.work.is_none());
    expect(s.scan(&w,&token,"intent-2",now),Reason::StaleTarget);
}
#[test]
fn cancel_scan_before_dispatch_prevents_write_and_retains_slot_until_drop() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"cancel-queued",now).unwrap();let mut work=a.work.unwrap();
    let t=s.cancel(&w,&a.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Cancelled,Some(Reason::CancelledBeforeDispatch)));
    assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::CancelledBeforeDispatch));
    expect(s.read(&w,true,now),Reason::Busy);drop(work);
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().phase,Phase::Cancelled);
}
#[test]
fn cancel_scan_after_dispatch_reports_unknown_without_rollback_or_retry() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"cancel-running",now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    let t=s.cancel(&w,&a.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Unknown,Some(Reason::CancelledAfterDispatch)));
    expect(s.read(&w,true,now),Reason::Busy);assert_eq!(work.finish_scan(peer(1),now),Err(Reason::CancelledAfterDispatch));
    let duplicate=s.scan(&w,&token,"cancel-running",now).unwrap();assert_eq!(duplicate.ticket.phase,Phase::Unknown);assert!(duplicate.work.is_none());
}
#[test]
fn targets_are_bound_to_window_kind_generation_and_source() {
    let (s,setting,w,now)=environment(Config::default());populate(&s,&w,now);let v=s.view(&w,now).unwrap();
    let token=v.scan_scope.unwrap();let other=SystemWindowAuthority::from_backend(WindowKind::System,true,&setting);
    expect(s.scan(&other,&token,"foreign",now),Reason::StaleTarget);
    expect(s.scan(&w,&v.targets[0].handle,"wrong-kind",now),Reason::WrongTarget);
    populate(&s,&w,now);expect(s.scan(&w,&token,"old-generation",now),Reason::StaleTarget);
}
#[test]
fn changed_producer_at_actual_writer_is_rejected_without_retargeting() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"source",now).unwrap();let mut work=a.work.unwrap();
    assert_eq!(work.begin_dispatch(peer(2),now),Err(Reason::SourceChanged));
    let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Failed,Some(Reason::SourceChanged)));
    assert!(s.view(&w,now).unwrap().stale);drop(work);
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().reason,Some(Reason::SourceChanged));
}
#[test]
fn completion_from_different_producer_cannot_publish_read_or_ack() {
    let (s,_,w,now)=environment(Config::default());let mut work=s.read(&w,true,now).unwrap().work.unwrap();
    work.begin_dispatch(peer(1),now).unwrap();
    assert_eq!(work.finish_read(observation(peer(2),vec![],ScanSupport::Unproven),now),Err(Reason::InvalidObservation));
    populate(&s,&w,now);let token=scope(&s,&w,now);let mut work=s.scan(&w,&token,"wrong-ack",now).unwrap().work.unwrap();
    work.begin_dispatch(peer(1),now).unwrap();assert_eq!(work.finish_scan(peer(2),now),Err(Reason::InvalidObservation));
}
#[test]
fn duplicate_labels_keep_distinct_opaque_identities_and_do_not_leak_native_keys() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let v=s.view(&w,now).unwrap();
    let aps:Vec<_>=v.targets.iter().filter(|t|t.kind==TargetKind::AccessPoint).collect();
    assert_eq!(aps.len(),2);assert_eq!(aps[0].label,aps[1].label);assert_ne!(aps[0].handle,aps[1].handle);
    let text=public(&v).to_string();assert!(!text.contains("/org/freedesktop"));assert!(!text.contains("12345678-1234"));
    assert_eq!(public(&v)["data"]["wifi"]["nearby"]["observedRows"],2);
}
#[test]
fn duplicate_native_identity_including_uuid_case_is_rejected() {
    let (s,_,w,now)=environment(Config::default());
    for items in [vec![targets().remove(0),targets().remove(0)],
        vec![target(TargetKind::VpnProfile,"12345678-1234-1234-1234-123456789abc","one"),
             target(TargetKind::VpnProfile,"12345678-1234-1234-1234-123456789ABC","two")]] {
        let mut work=s.read(&w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
        assert_eq!(work.finish_read(observation(peer(1),items,ScanSupport::VerifiedByAdapter),now),Err(Reason::InvalidObservation));
    }
}
#[test]
fn unproven_scan_support_never_mints_scope_or_claims_control_capability() {
    let (s,_,w,now)=environment(Config::default());let mut work=s.read(&w,true,now).unwrap().work.unwrap();
    work.begin_dispatch(peer(1),now).unwrap();work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now).unwrap();
    let v=s.view(&w,now).unwrap();assert!(!v.stale);assert!(v.scan_scope.is_none());assert!(!v.capabilities.scan);
    assert!(!v.capabilities.connect&&!v.capabilities.radio&&!v.capabilities.credentials);
    expect(s.scan(&w,"unminted","unsupported",now),Reason::StaleTarget);
}
#[test]
fn raw_payload_credentials_errors_and_ids_are_excluded_from_public_output() {
    let (s,_,w,now)=environment(Config::default());let mut o=observation(peer(1),targets(),ScanSupport::VerifiedByAdapter);
    o.result["wifiIP"]=json!("PRIVATE_IP");o.result["wifiBSSID"]=json!("PRIVATE_MAC");o.result["lastError"]=json!("PRIVATE_ERROR");
    o.result["credentials"]=json!({"psk":"PRIVATE_PASSWORD"});o.result["vpnProfiles"][0]["uuid"]=json!("PRIVATE_UUID");
    let a=s.read(&w,true,now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();work.finish_read(o,now).unwrap();
    let mut text=public(&s.view(&w,now).unwrap()).to_string();text.push_str(&public(&s.ticket(&w,&a.ticket.id,now).unwrap()).to_string());
    for private in ["PRIVATE_","/org/freedesktop","12345678-1234"] {assert!(!text.contains(private),"{private}");}
}
#[test]
fn future_connect_radio_and_credentials_remain_unsupported_without_work() {
    let (s,_,w,_)=environment(Config::default());
    for op in [FutureOperation::ConnectWifi,FutureOperation::SetRadio,FutureOperation::SubmitCredentials] {assert_eq!(s.extension(&w,op),Err(Reason::Unsupported));}
    assert!(s.state.lock().unwrap().flight.is_none());assert!(s.state.lock().unwrap().tickets.is_empty());
}
#[test]
fn recreated_window_cannot_read_or_cancel_previous_window_receipt() {
    let (s,setting,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();drop(a.work);drop(w);
    let new=SystemWindowAuthority::from_backend(WindowKind::System,true,&setting);
    expect(s.ticket(&new,&a.ticket.id,now),Reason::Unauthorized);expect(s.cancel(&new,&a.ticket.id,now),Reason::Unauthorized);
}
#[test]
fn ticket_capacity_and_retention_are_explicit_and_never_replay_old_scope() {
    let mut cfg=Config::default();cfg.max_tickets=1;
    let (s,_,w,now)=environment(cfg);let a=s.read(&w,true,now).unwrap();drop(a.work);
    expect(s.read(&w,true,now),Reason::Capacity);
    let later=now+Duration::from_secs(120);assert!(s.read(&w,true,later).unwrap().work.is_some());
    expect(s.ticket(&w,&a.ticket.id,later),Reason::NotFound);
}
#[test]
fn registry_capacity_is_an_error_without_silent_control_downgrade() {
    let mut cfg=Config::default();cfg.max_bindings=1;let (s,_,w,now)=environment(cfg);populate(&s,&w,now);
    expect(s.view(&w,now),Reason::Capacity);
    assert!(s.state.lock().unwrap().bindings.is_empty());
}
#[test]
fn config_counter_and_generation_overflow_fail_closed() {
    let setting=SystemSetting::from_backend(true);
    for cfg in [Config{max_tickets:0,..Config::default()},Config{max_bindings:1025,..Config::default()},
        Config{operation_timeout:Duration::ZERO,..Config::default()},
        Config{target_ttl:Duration::from_secs(121),..Config::default()}] {expect(Service::new(cfg,1,&setting),Reason::Capacity);}
    expect(Service::new(Config::default(),0,&setting),Reason::Capacity);
    let (s,_,w,now)=environment(Config::default());s.state.lock().unwrap().next=u64::MAX;
    expect(s.read(&w,true,now),Reason::Capacity);assert!(s.state.lock().unwrap().flight.is_none());
    let (s,_,w,now)=environment(Config::default());s.state.lock().unwrap().generation=u64::MAX;
    let mut work=s.read(&w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    assert_eq!(work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now),Err(Reason::Capacity));
    assert!(s.view(&w,now).unwrap().data.is_none());
}
#[test]
fn target_identity_and_label_validation_reject_unproven_or_ambiguous_values() {
    expect(ProducerId::from_verified_backend([0;16]),Reason::InvalidObservation);
    for (kind,key,label) in [(TargetKind::WifiDevice,"wlan0","x"),
        (TargetKind::AccessPoint,"/org/freedesktop/NetworkManager/AccessPoint/3/../4","x"),
        (TargetKind::VpnProfile,"00000000-0000-0000-0000-000000000000","x"),
        (TargetKind::WifiDevice,"/org/freedesktop/NetworkManager/Devices/1","bad\nlabel"),
        (TargetKind::WifiDevice,"/org/freedesktop/NetworkManager/Devices/1","\u{202e}spoof")] {
        expect(NativeTarget::from_verified_backend(kind,key,label),Reason::InvalidObservation);
    }
}
#[test]
fn malformed_observation_and_excessive_target_count_cannot_refresh_cache() {
    let mut cfg=Config::default();cfg.max_targets=1;let (s,_,w,now)=environment(cfg);
    for mut o in [observation(peer(1),targets(),ScanSupport::VerifiedByAdapter),observation(peer(1),vec![],ScanSupport::Unproven)] {
        if o.targets.is_empty(){o.observed_at_ms=u64::MAX;}
        let mut work=s.read(&w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
        assert_eq!(work.finish_read(o,now),Err(Reason::InvalidObservation));assert!(s.view(&w,now).unwrap().data.is_none());
    }
}
#[test]
fn completion_before_dispatch_or_with_wrong_operation_is_rejected() {
    let (s,_,w,now)=environment(Config::default());let work=s.read(&w,true,now).unwrap().work.unwrap();
    assert_eq!(work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now),Err(Reason::InvalidObservation));
    let mut work=s.read(&w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(1),now).unwrap();
    assert_eq!(work.finish_scan(peer(1),now),Err(Reason::InvalidObservation));assert!(s.view(&w,now).unwrap().data.is_none());
}
#[test]
fn intent_and_receipt_inputs_are_bounded_without_any_native_effect() {
    let (s,_,w,now)=environment(Config::default());
    for intent in ["", "space invalid", "é", &"a".repeat(65)] {expect(s.scan(&w,"scope",intent,now),Reason::InvalidIntent);}
    expect(s.scan(&w,&"s".repeat(129),"valid",now),Reason::StaleTarget);
    expect(s.ticket(&w,&"r".repeat(129),now),Reason::NotFound);expect(s.cancel(&w,&"r".repeat(129),now),Reason::NotFound);
    assert!(s.state.lock().unwrap().flight.is_none());
}
#[test]
fn newer_observation_creates_new_generation_handles_and_preserves_old_receipts() {
    let (s,_,w,now)=environment(Config::default());let first=populate(&s,&w,now);let before=s.view(&w,now).unwrap();
    let mut work=s.read(&w,true,now).unwrap().work.unwrap();work.begin_dispatch(peer(2),now).unwrap();
    let mut o=observation(peer(2),targets(),ScanSupport::VerifiedByAdapter);o.observed_at_ms+=1;
    work.finish_read(o,now).unwrap();let after=s.view(&w,now).unwrap();
    assert_eq!(before.generation,Some("1".into()));assert_eq!(after.generation,Some("2".into()));assert_ne!(before.scan_scope,after.scan_scope);
    assert_eq!(s.ticket(&w,&first.id,now).unwrap().phase,Phase::Observed);
    expect(s.scan(&w,&before.scan_scope.unwrap(),"old",now),Reason::StaleTarget);
}

#[test]
fn scan_target_expiring_while_queued_cannot_be_sent_later() {
    let mut cfg=Config::default();cfg.target_ttl=Duration::from_secs(2);
    let (s,_,w,now)=environment(cfg);populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"expires-queued",now+Duration::from_millis(1900)).unwrap();let mut work=a.work.unwrap();
    let later=now+Duration::from_secs(2);assert_eq!(work.begin_dispatch(peer(1),later),Err(Reason::StaleTarget));
    assert_eq!(s.ticket(&w,&a.ticket.id,later).unwrap().phase,Phase::Failed);drop(work);
    assert_eq!(s.view(&w,later).unwrap().reason,Some(Reason::StaleTarget));
}
#[test]
fn source_cache_expiring_before_target_ttl_also_prevents_dispatch() {
    let mut cfg=Config::default();cfg.cache_age=Duration::from_secs(1);
    let (s,_,w,now)=environment(cfg);populate(&s,&w,now);let token=scope(&s,&w,now);
    let mut work=s.scan(&w,&token,"cache-expires",now+Duration::from_millis(900)).unwrap().work.unwrap();
    assert_eq!(work.begin_dispatch(peer(1),now+Duration::from_secs(1)),Err(Reason::StaleTarget));
}
#[test]
fn changing_native_presentation_and_back_revokes_old_visible_lease() {
    let (s,_,w,now)=environment(Config::default());let a=s.read(&w,true,now).unwrap();let mut work=a.work.unwrap();
    w.presentation_from_backend(WindowKind::Other);expect(s.view(&w,now),Reason::Unauthorized);
    w.presentation_from_backend(WindowKind::System);assert_eq!(work.begin_dispatch(peer(1),now),Err(Reason::Unauthorized));
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().reason,Some(Reason::Revoked));
}
#[test]
fn cancelling_and_expiring_joins_cannot_grow_a_physical_flight_unbounded() {
    let mut cfg=Config::default();cfg.target_ttl=Duration::from_nanos(1);cfg.ticket_retention=Duration::from_nanos(1);cfg.max_tickets=2;
    let (s,_,w,now)=environment(cfg);let first=s.read(&w,true,now).unwrap();let joined=s.read(&w,true,now).unwrap();
    s.cancel(&w,&joined.ticket.id,now).unwrap();let later=now+Duration::from_nanos(2);
    expect(s.read(&w,true,later),Reason::Capacity);
    {let state=s.state.lock().unwrap();assert_eq!(state.tickets.len(),2);assert_eq!(state.flight.as_ref().unwrap().tickets.len(),2);}
    drop(first.work);assert!(s.read(&w,true,later).unwrap().work.is_some());
}
#[test]
fn revoking_one_join_only_revokes_its_receipt_when_worker_owner_remains_valid() {
    let (s,setting,w,now)=environment(Config::default());let w2=SystemWindowAuthority::from_backend(WindowKind::System,true,&setting);
    let first=s.read(&w,true,now).unwrap();let joined=s.read(&w2,true,now).unwrap();let mut work=first.work.unwrap();
    work.begin_dispatch(peer(1),now).unwrap();w2.visibility_from_backend(false);w2.visibility_from_backend(true);
    work.finish_read(observation(peer(1),vec![],ScanSupport::Unproven),now).unwrap();
    assert_eq!(s.ticket(&w,&first.ticket.id,now).unwrap().phase,Phase::Observed);
    let t=s.ticket(&w2,&joined.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Unknown,Some(Reason::Revoked)));
}
#[test]
fn unknown_scan_receipts_recommend_readback_and_never_automatic_action_retry() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"unknown",now).unwrap();let mut work=a.work.unwrap();work.begin_dispatch(peer(1),now).unwrap();drop(work);
    let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!(t.phase,Phase::Unknown);assert!(t.refresh_recommended);assert!(!t.effect_confirmed);
    let retry=s.scan(&w,&token,"unknown",now).unwrap();assert!(retry.work.is_none());assert_eq!(retry.ticket.phase,Phase::Unknown);
    assert!(s.state.lock().unwrap().flight.is_none());
}
#[test]
fn failed_scan_reason_survives_guard_teardown_without_becoming_abandonment() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);let token=scope(&s,&w,now);
    let a=s.scan(&w,&token,"source-rejected",now).unwrap();let mut work=a.work.unwrap();
    assert_eq!(work.begin_dispatch(peer(2),now),Err(Reason::SourceChanged));drop(work);
    assert_eq!(s.view(&w,now).unwrap().reason,Some(Reason::SourceChanged));
    assert_eq!(s.ticket(&w,&a.ticket.id,now).unwrap().reason,Some(Reason::SourceChanged));
}

#[test]
fn native_read_errors_before_dispatch_keep_typed_reason_without_claiming_started_io() {
    let (s,_,w,now)=environment(Config::default());populate(&s,&w,now);
    for (error,reason) in [(NativeError::Unavailable,Reason::NativeUnavailable),(NativeError::Rejected,Reason::NativeRejected),(NativeError::InvalidResponse,Reason::InvalidObservation)] {
        let before=public(&s.view(&w,now).unwrap());let a=s.read(&w,true,now).unwrap();
        assert_eq!(a.work.unwrap().fail(error,now),Err(reason));
        let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Failed,Some(reason)));
        assert!(!t.effect_confirmed&&!t.refresh_recommended);
        let after=public(&s.view(&w,now).unwrap());assert_eq!(before["data"],after["data"]);
        assert_eq!(after["stale"],true);assert!(s.state.lock().unwrap().flight.is_none());
    }
}
#[test]
fn native_scan_errors_before_dispatch_keep_typed_reason_without_unknown_action() {
    let (s,_,w,now)=environment(Config::default());
    for (i,(error,reason)) in [(NativeError::Unavailable,Reason::NativeUnavailable),(NativeError::Rejected,Reason::NativeRejected),(NativeError::InvalidResponse,Reason::InvalidObservation)].into_iter().enumerate() {
        populate(&s,&w,now);let token=scope(&s,&w,now);let before=public(&s.view(&w,now).unwrap());
        let a=s.scan(&w,&token,&format!("not-sent-{i}"),now).unwrap();
        assert_eq!(a.work.unwrap().fail(error,now),Err(reason));
        let t=s.ticket(&w,&a.ticket.id,now).unwrap();assert_eq!((t.phase,t.reason),(Phase::Failed,Some(reason)));
        assert!(!t.effect_confirmed&&!t.refresh_recommended);
        assert_eq!(before["data"],public(&s.view(&w,now).unwrap())["data"]);
        assert!(s.state.lock().unwrap().flight.is_none());
    }
}

include!("enable_lifecycle_tests.rs");
