//! Opt-in, in-memory timing for one Mochi Codex call. No payload/identity fields,
//! external callback, logging, I/O, config lookup or execution authority.
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub(crate) const MAX_EVENTS: usize = 128;
const MAX_MICROS: u64 = 24 * 60 * 60 * 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Entry { Text, Direct }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Stage {
    CliEntry, RunEntry, Queued, WorkerStarted, SelectionReady, ContextReady,
    Spawn, Initialize, Initialized, AccountRead, ConfigRead, ThreadStart,
    ThreadResume, ThreadNameSet, HistoryInject, TurnStart, Answer,
    FirstAcceptedEvent, FirstPublicText, AcceptedTurnCompletion,
    NativeRequestCallback, DynamicToolCallback, Cleanup, ChildReaped, Commit, Returned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Point { Started, Written, Observed, Returned, Failed, Abandoned }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Event { stage: Stage, point: Point, at_micros: u64 }

/// Fixed vocabulary only; fields are private to keep serialized size bounded.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Report {
    schema_version: u8,
    enabled: bool,
    entry: Entry,
    events: Vec<Event>,
    limited: bool,
    complete: bool,
}
impl Report {
    pub(crate) fn disabled(entry: Entry) -> Self {
        Self { schema_version: 1, enabled: false, entry, events: Vec::new(), limited: false, complete: true }
    }
}
struct State { report: Report }
#[derive(Clone)]
pub(crate) struct Trace { began: Instant, state: Arc<Mutex<State>> }
impl Trace {
    pub(crate) fn new(entry: Entry) -> Self {
        Self { began: Instant::now(), state: Arc::new(Mutex::new(State {
            report: Report { schema_version: 1, enabled: true, entry,
                events: Vec::with_capacity(MAX_EVENTS), limited: false, complete: false },
        })) }
    }
    fn record(&self, stage: Stage, point: Point, terminal: bool) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        // Sample while locked: event order also orders the monotonic clock.
        let micros = u64::try_from(self.began.elapsed().as_micros()).unwrap_or(u64::MAX);
        Self::record_at(&mut state.report, stage, point, micros, terminal);
    }
    fn record_at(report: &mut Report, stage: Stage, point: Point, micros: u64, terminal: bool) {
        if report.complete { return; }
        // Reserve one slot for the final outcome even when callbacks are noisy.
        if report.events.len() >= if terminal { MAX_EVENTS } else { MAX_EVENTS - 1 } {
            report.limited = true;
        } else {
            report.limited |= micros > MAX_MICROS;
            report.events.push(Event { stage, point, at_micros: micros.min(MAX_MICROS) });
        }
        if terminal { report.complete = true; }
    }
    pub(crate) fn finish<T, E>(&self, result: &Result<T, E>) {
        self.record(Stage::Returned, if result.is_ok() { Point::Returned } else { Point::Failed }, true);
    }
    pub(crate) fn report(&self) -> Report {
        self.state.lock().unwrap_or_else(|p| p.into_inner()).report.clone()
    }
}

pub(crate) fn mark(trace: Option<&Trace>, stage: Stage) {
    if let Some(trace) = trace { trace.record(stage, Point::Observed, false); }
}
fn rpc_stage(method: &str) -> Option<Stage> {
    Some(match method {
        "initialize" => Stage::Initialize,
        "account/read" => Stage::AccountRead,
        "config/read" => Stage::ConfigRead,
        "thread/start" => Stage::ThreadStart,
        "thread/resume" => Stage::ThreadResume,
        "thread/name/set" => Stage::ThreadNameSet,
        "thread/inject_items" => Stage::HistoryInject,
        "turn/start" => Stage::TurnStart,
        _ => return None,
    })
}
pub(crate) fn written(trace: Option<&Trace>, method: &str) {
    if let (Some(trace), Some(stage)) = (trace, rpc_stage(method)) {
        trace.record(stage, Point::Written, false);
    }
}
/// Returned means the call returned, not that a tool effect/permission succeeded.
pub(crate) struct Span { trace: Option<Trace>, stage: Stage, done: bool }
impl Span {
    pub(crate) fn new(trace: Option<&Trace>, stage: Stage) -> Self {
        if let Some(trace) = trace { trace.record(stage, Point::Started, false); }
        Self { trace: trace.cloned(), stage, done: false }
    }
    pub(crate) fn rpc(trace: Option<&Trace>, method: &str) -> Self {
        match rpc_stage(method) {
            Some(stage) => Self::new(trace, stage),
            None => Self::new(None, Stage::Answer),
        }
    }
    pub(crate) fn returned(mut self) {
        if let Some(trace) = &self.trace { trace.record(self.stage, Point::Returned, false); }
        self.done = true;
    }
    pub(crate) fn finish<T, E>(mut self, result: &Result<T, E>) {
        if let Some(trace) = &self.trace {
            trace.record(self.stage, if result.is_ok() { Point::Returned } else { Point::Failed }, false);
        }
        self.done = true;
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if !self.done {
            if let Some(trace) = &self.trace { trace.record(self.stage, Point::Abandoned, false); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_is_bounded_and_reserves_the_terminal_outcome() {
        let t = Trace::new(Entry::Text);
        for _ in 0..MAX_EVENTS * 4 { mark(Some(&t), Stage::FirstPublicText); }
        t.finish(&Ok::<_, ()>(()));
        let r = t.report();
        assert_eq!(r.events.len(), MAX_EVENTS);
        assert!(r.complete && r.limited);
        assert_eq!(r.events.last().unwrap().stage, Stage::Returned);
        assert!(serde_json::to_vec(&r).unwrap().len() < 32 * 1024);
    }
    #[test]
    fn one_origin_orders_events_and_seal_rejects_late_marks() {
        let t = Trace::new(Entry::Direct);
        mark(Some(&t), Stage::CliEntry);
        Span::new(Some(&t), Stage::Spawn).returned();
        t.finish(&Err::<(), _>("private native error must not be stored"));
        let before = serde_json::to_value(t.report()).unwrap();
        mark(Some(&t), Stage::FirstPublicText);
        t.finish(&Ok::<_, ()>(()));
        assert_eq!(serde_json::to_value(t.report()).unwrap(), before);
        assert!(t.report().events.windows(2).all(|w| w[0].at_micros <= w[1].at_micros));
        assert!(!serde_json::to_string(&before).unwrap().contains("private"));
    }
    #[test]
    fn disabled_and_unknown_methods_have_no_trace_events() {
        mark(None, Stage::Spawn);
        Span::new(None, Stage::Initialize).returned();
        let t = Trace::new(Entry::Direct);
        Span::rpc(Some(&t), "secret/unknown-path").returned();
        written(Some(&t), "secret/unknown-path");
        assert!(t.report().events.is_empty());
        let r = Report::disabled(Entry::Text);
        assert!(!r.enabled && r.complete && r.events.is_empty());
    }
    #[test]
    fn missing_and_abandoned_spans_are_not_zero_duration_successes() {
        let t = Trace::new(Entry::Direct);
        drop(Span::new(Some(&t), Stage::ThreadStart));
        let r = t.report();
        assert!(!r.complete);
        assert_eq!(r.events.iter().map(|e|e.point).collect::<Vec<_>>(), vec![Point::Started, Point::Abandoned]);
        assert!(!r.events.iter().any(|e| e.stage == Stage::HistoryInject));
    }
    #[test]
    fn elapsed_values_are_finite_and_capped() {
        let mut r = Trace::new(Entry::Direct).report();
        Trace::record_at(&mut r, Stage::Spawn, Point::Observed, u64::MAX, false);
        assert_eq!(r.events[0].at_micros, MAX_MICROS);
        assert!(r.limited);
    }
}
