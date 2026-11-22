export type AccountUsageFailure='notObserved'|'nativeUnavailable'|'notSignedIn'|'invalidData'|'clockUnavailable'|'unsupportedPlatform';
export type AccountUsageRefreshError='busy'|'cooldown'|'unavailable';
export interface CodexAccountSummary {
 lifetimeTokens:string|null;
 peakDailyTokens:string|null;
 longestRunningTurnSec:string|null;
 currentStreakDays:string|null;
 longestStreakDays:string|null;
}
export interface CodexAccountUsage {
 schemaVersion:1;
 provider:'codex';
 source:'codexAppServerAccountUsage';
 sourceInstance:string|null;
 revision:string;
 scope:'lastObservedCodexAccount';
 status:'available'|'unavailable';
 observedAt:number|null;
 readAt:number|null;
 stale:boolean;
 refreshing:boolean;
 summary:CodexAccountSummary|null;
 reason:AccountUsageFailure|null;
}
export interface CodexAccountUsageChanged {
 schemaVersion:1;
 provider:'codex';
 source:'codexAppServerAccountUsage';
 sourceInstance:string;
 revision:string;
}
// invoke('pet_account_usage', {}) returns cached metadata, no native client/IO.
// invoke('pet_account_usage_refresh', {}) explicitly authenticates/reads fixed
// account metadata only. Result::Ok resolves raw DTO; Err rejects static enum.
// No provider, threadId, path, URL or arbitrary method parameter is accepted.
// Summary strings are exact nonnegative signed64 decimals; null is unknown.
// lastObservedCodexAccount is not proof of the currently logged-in account after
// another native login, and is not a pet's measured usage or quota.
// observedAt is local response observation time; stale is cache age only.
// Never invoke refresh from rendering, hover, navigation, timers or retries.
// sourceInstance is a non-secret32-lowercase-hex native Runtime incarnation,
// never an account ID/permission. null means source unavailable: do not bind it.
// revision is canonical checked u64 decimal; reserve begin+finish before work.
// Cache mutations advance revision, queries do not. Same revision may have new
// readAt/stale/clock-derived reason; use latest local query ownership and do not
// allow an earlier fresh observation to erase a later stale observation.
// 'pet-account-usage-changed' is a post-unlock BEST-EFFORT invalidation hint.
// Events may reorder/be lost/be renderer-spoofed: never publish their state,
// trust their instance as a binding or mint a permanent revision floor from them.
// Coalesce a fresh cached query only; authenticated refresh is EXPLICIT only.
// Bootstrap through native cached query. Bind/rebind to a new source instance
// only from that native observation with a new local lifetime. Old instance ACKs
// and snapshots never reset the binding. Compare revisions only within instance.
// Ignore older state revisions, including reads initiated DURING refresh and
// late refresh ACKs. busy/cooldown are local feedback, not new global state.
