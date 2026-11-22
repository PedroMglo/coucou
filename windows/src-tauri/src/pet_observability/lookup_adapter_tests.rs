use super::*;

// Synthetic gates only. No Tauri app, configuration, native RPC or model.
#[test]
fn native_lookup_busy_cooldown_and_provider_isolation_require_deliberate_retry() {
    assert!(CODEX.lock().unwrap().started.is_none());
    assert!(COPILOT.lock().unwrap().started.is_none());
    let codex=acquire("codex").ok().expect("first Codex lookup admitted");
    let busy=acquire("codex").err().expect("second live lookup denied");
    assert_eq!(busy.reason,"nativeLookupBusy");assert_eq!(busy.retry_after_ms,1000);
    let copilot=acquire("copilot").ok().expect("another provider has an independent gate");
    drop(codex);
    let cooldown=acquire("codex").err().expect("completion does not bypass cooldown");
    assert_eq!(cooldown.reason,"nativeLookupThrottled");assert!((1..=10000).contains(&cooldown.retry_after_ms));
    // Expire only this synthetic gate's monotonic timestamp. No timer retries.
    CODEX.lock().unwrap().started=Some(Instant::now()-QUERY_COOLDOWN-Duration::from_millis(1));
    let retried=acquire("codex").ok().expect("a later manual lookup is admitted");
    assert!(COPILOT.lock().unwrap().running);
    drop(retried);drop(copilot);
    assert!(!CODEX.lock().unwrap().running);assert!(!COPILOT.lock().unwrap().running);
    assert_eq!(acquire("antigravity").err().unwrap().reason,"nativeInventoryNotSupported");
    // Keep global fixture state pristine for any later tests in this process.
    for gate in [&CODEX,&COPILOT] {let mut state=gate.lock().unwrap();state.started=None;state.running=false;}
}
