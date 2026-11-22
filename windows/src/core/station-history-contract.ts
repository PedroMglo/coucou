/** Local workstation history metadata. It never counts open terminals or tokens.
 * Produced from the existing management inventory, without account refresh,
 * inference, turn/session actions or replacement of the session-control client. */
export interface StationHistoryMetric {
 value:number|null;
 observed:number;
 coverage:'completeWithinIndex'|'completeWithinCatalogue'|'partial'|'unavailable';
 scope:'localIndex'|'localCatalogue'|'localMetadataUnion'|'archiveMetadata';
}
export interface StationHistoryProvider {
 provider:'codex'|'copilot';
 /** Codex index only; Copilot DB+legacy catalogue is known.scope=localCatalogue. */
 indexedTotal:number|null;
 /** Distinct validated native IDs in the metadata union; not live processes. */
 known:StationHistoryMetric;
 /** Copilot's legacy default false is not archive evidence; remains unavailable. */
 archived:StationHistoryMetric;
}
export interface StationHistorySnapshot {
 schemaVersion:1;
 source:'station-management-inventory';
 status:'available'|'unavailable';
 observedAt:number|null;
 readAt:number;
 refreshing:boolean;
 stale:boolean;
 reason:null|'notObserved'|'collectionFailed'|'invalidInventory';
 providers:StationHistoryProvider[];
}
// pet_station_history({force?:boolean}) -> StationHistorySnapshot.
// Native cache shares one physical collection;15s cooldown (also after errors),
// explicit force2s minimum. Retains last good with its ORIGINAL observedAt.
// Native allowed metadata windows only. No arbitrary action/provider/path input.
// The collector uses the EXISTING helper list operation. That operation reads
// native metadata and local Ollama model descriptions/capability cache. It is not
// the proposed zero-file-effect passive SQLite reader; no such claim is made.
// UI: pair with independently measured process census for open CLI clients;
// keep library separate, do not sum same physical sessions across origins.

// Codex indexed zero requires helper installed:true (absent CLI skips DB).
// Copilot empty/missing catalogue has no zero provenance; remains unknown.
