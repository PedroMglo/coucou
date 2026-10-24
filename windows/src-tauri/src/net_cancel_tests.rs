//! Synthetic loopback transports only; no real model, credentials or account.
use super::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Copy)]
pub(crate) enum Stage { Headers, Body }

/// Accept exactly one complete small POST, optionally send a partial body,
/// and independently observe EOF when the caller drops its request future.
pub(crate) async fn assert_http_cancelled<F>(stage: Stage, perform: impl FnOnce(Url) -> F)
where F: std::future::Future<Output = Result<serde_json::Value, String>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = Url::parse(&format!("http://{}/synthetic", listener.local_addr().unwrap())).unwrap();
    let (accepted, gate) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut bytes = [0; 1024];
            let n = stream.read(&mut bytes).await.unwrap();
            assert_ne!(n, 0); request.extend_from_slice(&bytes[..n]);
            assert!(request.len() <= 8192);
            if let Some(end) = request.windows(4).position(|s| s == b"\r\n\r\n") {
                let header = std::str::from_utf8(&request[..end]).unwrap().to_ascii_lowercase();
                let length: usize = header.lines().find_map(|s| s.strip_prefix("content-length:")).unwrap().trim().parse().unwrap();
                if request.len() >= end + 4 + length { break; }
            }
        }
        if matches!(stage, Stage::Body) {
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"partial\":").await.unwrap();
        }
        accepted.send(()).unwrap();
        let mut byte = [0; 1];
        match tokio::time::timeout(Duration::from_secs(2), stream.read(&mut byte)).await {
            Ok(Ok(0)) => {},
            Ok(Err(e)) if matches!(e.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe) => {},
            other => panic!("The owned HTTP connection did not close on cancellation: {other:?}"),
        }
    });
    let active = AtomicBool::new(true);
    let outcome = tokio::time::timeout(Duration::from_secs(3), async {
        let (outcome, ()) = tokio::join!(
            while_current(|| active.load(Ordering::Acquire), || perform(endpoint)),
            async { gate.await.unwrap(); active.store(false, Ordering::Release); },
        );
        outcome
    }).await.expect("Cancelled request retained its wait");
    assert_eq!(outcome.err().as_deref(), Some(CANCELLED));
    server.await.unwrap();
}

#[tokio::test]
async fn invalid_scope_never_constructs_or_polls_the_request() {
    let calls = AtomicUsize::new(0);
    let result = while_current(|| false, || {
        calls.fetch_add(1, Ordering::AcqRel);
        async { Ok::<_, String>("must not run") }
    }).await;
    assert_eq!(result.err().as_deref(), Some(CANCELLED));
    assert_eq!(calls.load(Ordering::Acquire), 0);
}

#[tokio::test]
async fn scope_lost_in_factory_prevents_first_future_poll() {
    let active = AtomicBool::new(true); let polls = AtomicUsize::new(0);
    let result = while_current(|| active.load(Ordering::Acquire), || {
        active.store(false, Ordering::Release);
        async { polls.fetch_add(1, Ordering::AcqRel); Ok::<_, String>("late") }
    }).await;
    assert_eq!(result.err().as_deref(), Some(CANCELLED));
    assert_eq!(polls.load(Ordering::Acquire), 0);
}

#[tokio::test]
async fn completion_that_revokes_its_scope_does_not_publish_success() {
    let active = AtomicBool::new(true);
    let result = while_current(|| active.load(Ordering::Acquire), || async {
        active.store(false, Ordering::Release); Ok::<_, String>("late answer")
    }).await;
    assert_eq!(result.err().as_deref(), Some(CANCELLED));
}

#[tokio::test]
async fn successful_and_failed_results_are_returned_once_without_retries() {
    let calls = AtomicUsize::new(0);
    for succeeds in [true, false] {
        let result = while_current(|| true, || async {
            calls.fetch_add(1, Ordering::AcqRel);
            if succeeds { Ok("answer") } else { Err("provider error".into()) }
        }).await;
        if succeeds { assert_eq!(result.unwrap(), "answer"); }
        else { assert_eq!(result.unwrap_err(), "provider error"); }
    }
    assert_eq!(calls.load(Ordering::Acquire), 2);
}

#[tokio::test]
async fn cancellation_of_old_turn_never_cancels_a_new_turn_or_uses_its_identity() {
    let chat = crate::chat::Chat::default();
    let old = chat.begin_cli("openai", Default::default()).unwrap();
    let lease = chat.lease(&old);
    chat.cancel_provider("openai"); drop(lease);
    let next = chat.begin_cli("openai", Default::default()).unwrap();
    let calls = AtomicUsize::new(0);
    let result = while_current(|| chat.is_current(&old), || {
        calls.fetch_add(1, Ordering::AcqRel); async { Ok::<_, String>(()) }
    }).await;
    assert_eq!(result.err().as_deref(), Some(CANCELLED));
    assert_eq!(calls.load(Ordering::Acquire), 0);
    assert!(chat.is_current(&next));
    assert!(!next.cancel.load(Ordering::Acquire));
}
