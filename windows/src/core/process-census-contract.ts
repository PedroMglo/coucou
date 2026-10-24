// Proposed independent source. Process liveness != model turn activity/history.
export interface ProcessCensus {
 schemaVersion:1;
 provider:string;
 scope:'localProcesses';
 coverage:'completeWithinUserProcessScan'|'partial'|'unavailable'|'notSupported';
 observedAt:number|null;
 stale:boolean;
 refreshing:boolean;
 reason:string|null; // stable backend code, never raw paths/environ/stderr
 counts:{
  processStates?:{foreground:number;background:number;suspended:number;headless:number};
  terminalStates?:{foreground:number;backgroundOnly:number;suspendedOnly:number}|null;
  clientsObserved:number; // verified CLI clients after exact launcher/native-child dedup
  terminalClientsObserved:number;
  terminalInstancesObserved?:number|null;pausedClientsObserved?:number|null;
  foregroundClientsObserved:number;
  identifiedNativeSessionsObserved:number;
  clientsWithUnknownSession:number;
  clientsWithConflictingSession:number;
  /** null when even one observed client lacks an unambiguous native identity.
   * Scoped to these observed clients only; never a provider-global inventory. */
  distinctNativeSessions:number|null;
 };
 /** A process is not evidence that a native model turn is currently running. */
 turns:'notObservedByProcessCensus';
 clients:ProcessClient[];
}
export interface ProcessClient {
 key:string;provider:string;pid:number;startTimeTicks:string;
 channel:'terminal'|'headless';foreground:boolean;paused:boolean;
 cwd:string|null;
 sessionIdentity:{state:'unknown'|'identified'|'conflicting';nativeSessionId:string|null;sources:string[]};
 // No send/stop/grant capability. Census IDs never become approval authority.
}
export interface PetLiveWork {
 owner:string;petId:string;conversationId:string;turn:string;
 phase:'reserved'|'busy'|'waiting';
 nativeProvider:string|null;nativeSessionId:string|null;
 // nativeSessionId is present only if observed in THIS live turn, not saved history.
}
export interface ProviderCensusExtension { processCensus?:ProcessCensus|null }
export interface SnapshotLiveWorkExtension {
 // Snapshot top level. Separate logical-owner evidence, not a process count.
 livePetWork?:{status:'observed'|'unavailable';observedAt:number;reason:string|null;rows:PetLiveWork[];
  sessionStates?:{schemaVersion:1;scope:'loadedPetConversations';rows:{owner:string;petId:string;conversationId:string;turn:string|null;state:'idle'|'reserved'|'busy'|'waiting'|'stopping'|'unknown';reason:string|null}[]}|null;
 };
}
// native(codexDaemon), mochi(library) and usage retain their current semantics.
// Historical index/archive totals remain separate sources with their own coverage.
// UI may show “N clients observed · session identity unknown”; never turn N into
// N unique native sessions/working turns, infer fork membership from cwd HOME,
// sum duplicate pet+terminal aliases, or use census for Always Allow policy.

// Exact placement in existing pet_observability({}) snapshot:
// snapshot.providers[i].processCensus?:ProcessCensus|null
// snapshot.livePetWork?:{status,observedAt,reason,rows} (TOP LEVEL)
// The frontend owner maintains both shared TS interfaces in its independent FE34.
