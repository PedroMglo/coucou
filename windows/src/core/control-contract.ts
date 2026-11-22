/** Native binding R2 with additive network View2 consumer; mirrored for the frontend; no caller identity in payloads. */
export type Revision = string; // canonical bounded u64 decimal, never authority
export type Route = 'home' | 'system' | 'tasks' | 'sessions' | 'providers' | 'models' | 'profiles';
export type AcceptedMode = { kind: 'inactive' } | { kind: 'compact' } | { kind: 'expanded'; route: Route };
export interface ControlContext {
  schemaVersion: 1;
  revision: Revision;
  mode: AcceptedMode;
  pending: boolean;
  nativeAvailable: boolean;
  nativeVisible: boolean;
  nativeMapped: boolean;
  documentReady: boolean;
  systemEnabled: boolean;
  systemEligible: boolean;
  authorityContract: 'nativeIdentityAndAcceptedIntent';
}
export type NavigationIntent =
  | { kind: 'expand'; route: Route }
  | { kind: 'navigate'; route: Route }
  | { kind: 'collapse' };
export interface NavigationRequest {
  expectedRevision: Revision;
  intentId: string; // 1..64 ASCII alphanumeric / dash / underscore, idempotence only
  intent: NavigationIntent;
}
export interface NavigationReceipt {
  intentId: string;
  applied: boolean;
  context: ControlContext; // current accepted context; apply monotonically
}
export interface ContextPrecondition { expectedRevision: Revision }
export type BindingError = 'unavailable' | 'nativeCallerMismatch' | 'staleContext'
  | 'documentInactive' | 'systemDisabled' | 'transitionPending' | 'unsupported'
  | 'invalidRequest' | 'intentConflict' | 'nativeFailure' | 'superseded' | 'capacity';
export interface NetworkReadRequest extends ContextPrecondition { force: boolean }
export interface NetworkScanRequest extends ContextPrecondition {
  scope: string; // existing opaque current global source scope; max128
  intentId: string; // existing lifecycle idempotence; max64
}
export interface NetworkEnableWifiRequest extends ContextPrecondition {
  scope: string; // enableWifiScope only, never scanScope; max128
  intentId: string; // explicit enable-only gesture id; ASCII1..64, not authority
}
export interface NetworkTicketRequest extends ContextPrecondition { ticketId: string } // max128
export interface NewCommands {
  pet_control_navigation_context: { args: Record<string,never>; result: ControlContext; error: BindingFailure };
  pet_control_navigate: { args: { request: NavigationRequest }; result: NavigationReceipt; error: BindingFailure };
  pet_system_network_read: { args: { request: NetworkReadRequest }; result: NetworkTicket; error: SystemFailure };
  pet_system_network_view: { args: { request: ContextPrecondition }; result: NetworkView; error: SystemFailure };
  pet_system_network_scan: { args: { request: NetworkScanRequest }; result: NetworkTicket; error: SystemFailure };
  pet_system_network_enable_wifi: { args: { request: NetworkEnableWifiRequest }; result: NetworkTicket; error: SystemFailure };
  pet_system_network_ticket: { args: { request: NetworkTicketRequest }; result: NetworkTicket; error: SystemFailure };
  pet_system_network_cancel: { args: { request: NetworkTicketRequest }; result: NetworkTicket; error: SystemFailure };
}
// Caller WebviewWindow is injected by Tauri, never an args label/window/PID/token.
// New Rust request DTOs reject unknown fields and bound every string before copies.
// Network View2 is additive; View1 read/scan remains supported. Snapshot/context
// versions are unchanged. Source generation is not a navigation-context revision.
// GTK identity/accepted intent is not DOM, physical click or rendered System proof.

export interface BindingFailure { kind: 'binding'; code: BindingError; context: ControlContext | null }
export interface NetworkFailure { kind: 'network'; code: NetworkReason }
export type SystemFailure = BindingFailure | NetworkFailure;
export type NetworkReason = 'unauthorized' | 'hidden' | 'systemDisabled' | 'closed'
  | 'busy' | 'capacity' | 'invalidIntent' | 'intentConflict' | 'notFound'
  | 'notObserved' | 'cacheExpired' | 'staleTarget' | 'wrongTarget' | 'sourceChanged'
  | 'deadline' | 'revoked' | 'cancelledBeforeDispatch' | 'cancelledAfterDispatch'
  | 'abandonedBeforeDispatch' | 'unknownAfterDispatch' | 'alreadyDispatched'
  | 'invalidObservation' | 'nativeUnavailable' | 'nativeRejected' | 'unsupported';
export interface NetworkTicket {
  id: string;
  kind: 'read' | 'scan' | 'enableWifi';
  phase: 'queued' | 'running' | 'observed' | 'acknowledged' | 'cancelled' | 'unknown' | 'failed';
  reason: NetworkReason | null;
  effectConfirmed: false;
  refreshRecommended: boolean;
}
interface NetworkViewBase {
  source: 'dmsNetwork';
  generation: string | null;
  stale: boolean;
  refreshing: boolean;
  data: NetworkSnapshot | null;
  scanScope: string | null;
  targets: Array<{ handle: string; kind: 'wifiDevice' | 'accessPoint' | 'vpnProfile'; label: string }>;
  capabilities: { read: boolean; scan: boolean; connect: false; radio: false; credentials: false };
  reason: NetworkReason | null;
}
export interface NetworkView1 extends NetworkViewBase {schemaVersion:1}
export interface NetworkView2 extends NetworkViewBase {
  schemaVersion:2;
  enableWifiScope:string|null;
  capabilities:NetworkViewBase['capabilities']&{enableWifi:boolean};
}
export type NetworkView=NetworkView1|NetworkView2;
// View1 remains read/scan-only. Snapshot and native ControlContext stay schema1.
// Public capabilities/scopes never replace the native producer/Prepared fence.
export interface NetworkCollection<T> {
  entries: T[]; complete: boolean; observedRows: number; rejectedRows: number;
}
export interface WifiNetwork {
  name: string; signal: number | null; secured: boolean | null;
  saved: boolean | null; connected: boolean | null; outOfRange: boolean | null;
}
export interface NetworkSnapshot {
  schemaVersion: 1;
  observedAt: number; // Unix MILLISECONDS, bounded to JS Date range; never renew stale time
  source: 'dmsNetwork';
  readOnly: true; // metadata projection only, not View.capabilities
  wifi: {
    enabled: boolean | null; connected: boolean | null; connecting: boolean | null;
    currentName: string | null; signal: number | null;
    nearby: NetworkCollection<WifiNetwork> | null;
    saved: NetworkCollection<WifiNetwork> | null;
    devices: NetworkCollection<{ name: string }> | null;
  };
  ethernetConnected: boolean | null;
  cellularConnected: boolean | null;
  cellularEnabled: boolean | null;
  cellularHardwareEnabled: boolean | null;
  hotspot: {
    supported: boolean | null; available: boolean | null; configured: boolean | null;
    enabled: boolean | null; activating: boolean | null; secured: boolean | null; name: string | null;
  };
  vpnProfiles: NetworkCollection<{ name: string; state: string | null }> | null;
  activeVpns: NetworkCollection<{ name: string; state: string | null }> | null;
  warnings: string[]; // existing bounded static projection warnings, never raw native errors
}

// Tauri wire behavior: Result::Ok(T) resolves invoke() with RAW T (no {ok} envelope).
// Result::Err(E) rejects invoke() with serialized E: BindingFailure or NetworkFailure.
// Stale/conflicting navigation from a verified native caller rejects with current
// context in BindingFailure; unidentified caller uses context:null. No native replay.
// Exact duplicate nav intent returns applied:false and CURRENT context, never old
// route reapplication. Caller renders monotonically and does not auto-retry scan.

export interface ControlOpenedEvent {
  pet: string | null;
  section: Route | null;
  navigationRevision: Revision;
  context: ControlContext;
}
export interface ControlCollapseEvent {
  navigationRevision: Revision;
  context: ControlContext;
}
export interface ControlEvents {
  'pet-control-navigation-context': ControlContext;
  'pet-control-opened': ControlOpenedEvent;
  'pet-control-collapse': ControlCollapseEvent;
}
// Context is the CURRENT accepted snapshot at event emission, with monotonically
// allocated backend revision. No callbacks/emit while Core lock is held.
// Open/collapse payload navigationRevision MUST equal payload.context.revision.
// Emit legacy opened only for matching accepted open; no event from superseded plan.
// Configure/internal opens/revocations use the SAME context event/revision rules.
// Driver first bootstraps current context query, ignores historical last-open as
// authority, then consumes events/ACK monotonically and ignores older revisions.
// Same-route child System→Audio/Network/Back uses explicit navigate('system') once
// per actual child navigation, NEVER per frame/poll. It invalidates prior context.
