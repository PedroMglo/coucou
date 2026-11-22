use super::*;
use serde_json::json;

fn fixture_cache()->Cache{Cache::with_instance(Some("11111111111111111111111111111111".into()))}
fn summary()->Summary{parse(&json!({"summary":{"lifetimeTokens":23,"peakDailyTokens":11,"longestRunningTurnSec":120,"currentStreakDays":0,"longestStreakDays":4}})).unwrap()}

#[test]fn only_reported_five_metrics_survive_projection(){
 let input=json!({"summary":{"lifetimeTokens":23,"peakDailyTokens":null,"email":"PRIVATE","tokens":999},"account":{"key":"SECRET"},"dailyUsageBuckets":[{"startDate":"private","tokens":999}],"threadUsage":{"estimatedUsageUsdMicros":123}});
 let parsed=parse(&input).unwrap();assert_eq!(parsed.lifetime_tokens.as_deref(),Some("23"));assert_eq!(parsed.peak_daily_tokens,None);
 let text=serde_json::to_string(&parsed).unwrap();for excluded in ["PRIVATE","SECRET","999","estimatedUsage","dailyUsage"]{assert!(!text.contains(excluded));}
 assert_eq!(serde_json::to_value(parsed).unwrap().as_object().unwrap().len(),5);
}
#[test]fn unknown_and_real_zero_are_distinct(){
 let empty=parse(&json!({"summary":{}})).unwrap();assert!(empty.lifetime_tokens.is_none());
 let zero=parse(&json!({"summary":{"lifetimeTokens":0,"peakDailyTokens":null}})).unwrap();assert_eq!(zero.lifetime_tokens.as_deref(),Some("0"));assert_eq!(zero.peak_daily_tokens,None);
}
#[test]fn signed64_server_counts_serialize_exactly_beyond_js_safe_integer(){
 let parsed=parse(&json!({"summary":{"lifetimeTokens":i64::MAX,"peakDailyTokens":9_007_199_254_740_993_u64}})).unwrap();
 assert_eq!(parsed.lifetime_tokens.as_deref(),Some("9223372036854775807"));assert_eq!(parsed.peak_daily_tokens.as_deref(),Some("9007199254740993"));
}
#[test]fn invalid_known_metrics_reject_whole_response(){
 for key in ["lifetimeTokens","peakDailyTokens","longestRunningTurnSec","currentStreakDays","longestStreakDays"]{
  for bad in [json!(-1),json!(1.5),json!("23"),json!(true),json!(i64::MAX as u64+1),json!({})]{
   let mut value=json!({"summary":{"lifetimeTokens":23}});value["summary"][key]=bad;assert_eq!(parse(&value),Err(Failure::InvalidData));
  }
 }
}
#[test]fn malformed_or_error_envelopes_never_produce_success(){
 for input in [json!(null),json!([]),json!({}),json!({"summary":null}),json!({"summary":[]}),json!({"summary":{},"error":"PRIVATE"})]{assert_eq!(parse(&input),Err(Failure::InvalidData));}
}
#[test]fn cached_reads_start_unobserved_and_do_not_admit_work(){
 let cache=fixture_cache();for _ in 0..10{let v=cache.snapshot(100);assert_eq!(v.status,"unavailable");assert!(v.stale&&!v.refreshing);assert_eq!(v.reason,Some(Failure::NotObserved));assert!(v.summary.is_none());}assert!(cache.attempt.is_none());
}
#[test]fn single_flight_lives_until_physical_worker_finishes(){
 let now=Instant::now();let mut cache=fixture_cache();let ticket=cache.begin(now).unwrap();assert_eq!(cache.begin(now+Duration::from_secs(600)),Err(RefreshError::Busy));assert!(cache.snapshot(100).refreshing);
 assert!(cache.finish(ticket,Ok(summary()),100,now+Duration::from_secs(601)));assert_eq!(cache.snapshot(100).summary,Some(summary()));
}
#[test]fn queued_old_result_cannot_overwrite_current_generation(){
 let now=Instant::now();let mut cache=fixture_cache();let a=cache.begin(now).unwrap();assert!(cache.finish(a,Ok(summary()),100,now));let b=cache.begin(now+COOLDOWN).unwrap();
 assert!(!cache.finish(a,Ok(summary()),102,now+COOLDOWN));assert!(cache.snapshot(102).refreshing);assert!(cache.snapshot(102).summary.is_none());assert!(cache.finish(b,Err(Failure::NotSignedIn),102,now+COOLDOWN));assert_eq!(cache.snapshot(102).reason,Some(Failure::NotSignedIn));
}
#[test]fn explicit_refresh_discards_old_account_data_before_native_auth(){
 let now=Instant::now();let mut cache=fixture_cache();let a=cache.begin(now).unwrap();cache.finish(a,Ok(summary()),100,now);let _=cache.begin(now+COOLDOWN).unwrap();let v=cache.snapshot(101);assert!(v.summary.is_none()&&v.observed_at.is_none()&&v.refreshing);assert_eq!(v.status,"unavailable");
}
#[test]fn failure_never_restores_an_old_account_or_claims_zero(){
 let now=Instant::now();let mut cache=fixture_cache();let a=cache.begin(now).unwrap();cache.finish(a,Ok(summary()),100,now);let b=cache.begin(now+COOLDOWN).unwrap();cache.finish(b,Err(Failure::NativeUnavailable),105,now+COOLDOWN);let v=cache.snapshot(105);assert!(v.summary.is_none()&&v.observed_at.is_none()&&!v.refreshing);assert_eq!(v.reason,Some(Failure::NativeUnavailable));
}
#[test]fn cooldown_begins_at_completion_and_does_not_queue_a_retry(){
 let now=Instant::now();let mut cache=fixture_cache();let a=cache.begin(now).unwrap();cache.finish(a,Err(Failure::NativeUnavailable),100,now+Duration::from_secs(20));assert_eq!(cache.begin(now+Duration::from_secs(24)),Err(RefreshError::Cooldown));assert!(!cache.snapshot(104).refreshing);assert!(cache.begin(now+Duration::from_secs(25)).is_ok());
}
#[test]fn stale_time_never_becomes_a_refreshed_observation(){
 let now=Instant::now();let mut cache=fixture_cache();let a=cache.begin(now).unwrap();cache.finish(a,Ok(summary()),100,now);for at in [1,99,161,MAX_TIME+1,0]{let v=cache.snapshot(at);assert!(v.stale);assert_eq!(v.observed_at,Some(100));}assert!(!cache.snapshot(160).stale);
}
#[test]fn invalid_observation_clock_does_not_publish_counters(){
 for at in [0,MAX_TIME+1]{let mut cache=fixture_cache();let now=Instant::now();let a=cache.begin(now).unwrap();cache.finish(a,Ok(summary()),at,now);let v=cache.snapshot(100);assert!(v.summary.is_none());assert_eq!(v.reason,Some(Failure::ClockUnavailable));}
}
#[test]fn generation_exhaustion_preserves_last_snapshot_and_does_not_start_io(){
 let mut cache=fixture_cache();cache.next=u64::MAX;assert_eq!(cache.begin(Instant::now()),Err(RefreshError::Unavailable));assert!(!cache.snapshot(100).refreshing);assert_eq!(cache.snapshot(100).reason,Some(Failure::NotObserved));
}
#[test]fn serialised_snapshot_has_honest_account_scope_and_no_quota_or_session_authority(){
 let mut cache=fixture_cache();let now=Instant::now();let a=cache.begin(now).unwrap();cache.finish(a,Ok(summary()),100,now);let v=serde_json::to_value(cache.snapshot(100)).unwrap();assert_eq!(v["scope"],"lastObservedCodexAccount");assert_eq!(v["source"],"codexAppServerAccountUsage");for missing in ["petId","threadId","permissions","quota","cost","apiKey","email"]{assert!(v.get(missing).is_none());}assert_eq!(v["summary"]["lifetimeTokens"],"23");
}

#[test]fn state_revision_advances_on_begin_and_every_matching_settlement(){
 let now=Instant::now();let mut cache=fixture_cache();assert_eq!(cache.snapshot(100).revision,"0");
 for (i,result,at)in [(0_u32,Ok(summary()),100),(1,Err(Failure::NotSignedIn),100),(2,Ok(summary()),0)].into_iter(){
  let time=now+COOLDOWN*(i+1);let ticket=cache.begin(time).unwrap();assert_eq!(cache.snapshot(100).revision,(2*i+1).to_string());cache.finish(ticket,result,at,time);assert_eq!(cache.snapshot(100).revision,(2*i+2).to_string());
 }
 assert_eq!(cache.snapshot(100).reason,Some(Failure::ClockUnavailable));
}
#[test]fn cached_query_started_during_refresh_has_older_revision_than_finish_ack(){
 let now=Instant::now();let mut cache=fixture_cache();let ticket=cache.begin(now).unwrap();let old=cache.snapshot(100);assert!(old.refreshing&&old.summary.is_none());
 cache.finish(ticket,Ok(summary()),100,now);let ack=cache.snapshot(100);assert_eq!(old.source_instance,ack.source_instance);assert!(old.revision.parse::<u64>().unwrap()<ack.revision.parse::<u64>().unwrap());assert!(!ack.refreshing&&ack.summary.is_some());
}
#[test]fn rejected_begin_and_obsolete_completion_do_not_advance_revision(){
 let now=Instant::now();let mut cache=fixture_cache();let a=cache.begin(now).unwrap();let before=cache.snapshot(100).revision;assert_eq!(cache.begin(now),Err(RefreshError::Busy));assert_eq!(cache.snapshot(100).revision,before);
 cache.finish(a,Ok(summary()),100,now);let after=cache.snapshot(100).revision;assert!(!cache.finish(a,Err(Failure::NativeUnavailable),100,now));assert_eq!(cache.begin(now),Err(RefreshError::Cooldown));assert_eq!(cache.snapshot(100).revision,after);
}
#[test]fn completion_capacity_is_reserved_before_any_data_is_cleared(){
 let now=Instant::now();let mut cache=fixture_cache();cache.revision=u64::MAX-1;cache.summary=Some(summary());cache.observed_at=Some(100);
 assert_eq!(cache.begin(now),Err(RefreshError::Unavailable));let unchanged=cache.snapshot(100);assert_eq!(unchanged.revision,(u64::MAX-1).to_string());assert_eq!(unchanged.summary,Some(summary()));assert!(!unchanged.refreshing);
 cache.revision=u64::MAX-2;let ticket=cache.begin(now).unwrap();assert_eq!(cache.snapshot(100).revision,(u64::MAX-1).to_string());assert!(cache.finish(ticket,Ok(summary()),100,now));assert_eq!(cache.snapshot(100).revision,u64::MAX.to_string());assert_eq!(cache.begin(now+COOLDOWN),Err(RefreshError::Unavailable));
}
#[test]fn missing_or_malformed_producer_incarnation_refuses_native_admission(){
 for instance in [None,Some("".into()),Some("../private/path".into()),Some("A".repeat(32)),Some("f".repeat(33))]{let mut cache=Cache::with_instance(instance);let snapshot=cache.snapshot(100);assert!(snapshot.source_instance.is_none()&&snapshot.summary.is_none());assert_eq!(snapshot.reason,Some(Failure::NativeUnavailable));assert_eq!(cache.begin(Instant::now()),Err(RefreshError::Unavailable));assert_eq!(cache.snapshot(100).revision,"0");}
}
#[test]fn producer_incarnation_separates_restart_revision_reset(){
 let a=fixture_cache();let b=Cache::with_instance(Some("22222222222222222222222222222222".into()));assert_eq!(a.snapshot(100).revision,b.snapshot(100).revision);assert_ne!(a.snapshot(100).source_instance,b.snapshot(100).source_instance);
}
#[test]fn query_time_fields_can_change_without_versioning_unchanged_cache_state(){
 let now=Instant::now();let mut cache=fixture_cache();let ticket=cache.begin(now).unwrap();cache.finish(ticket,Ok(summary()),100,now);
 let fresh=cache.snapshot(100);let stale=cache.snapshot(161);assert_eq!(fresh.revision,stale.revision);assert_eq!(fresh.summary,stale.summary);assert_eq!(fresh.observed_at,stale.observed_at);assert_eq!(fresh.refreshing,stale.refreshing);assert!(!fresh.stale&&stale.stale);assert_ne!(fresh.read_at,stale.read_at);
}
#[test]fn change_event_is_only_a_scoped_hint_without_account_state_or_auth(){
 let cache=fixture_cache();let value=serde_json::to_value(cache.snapshot(100).changed().unwrap()).unwrap();assert_eq!(value["sourceInstance"],"11111111111111111111111111111111");assert_eq!(value["revision"],"0");assert_eq!(value.as_object().unwrap().len(),5);for forbidden in ["summary","observedAt","readAt","accountId","email","authorization"]{assert!(value.get(forbidden).is_none());}
 assert!(Cache::default().snapshot(100).changed().is_none());
}
