export interface PetCachedUsage {
 schemaVersion:1;
 source:'ia-barra-local-cache';
 /** Time of this file read, NOT a provider quota update. Unix seconds. */
 readAt:number;
 status:'available'|'unavailable'|'notSupported';
 reason:'cacheUnavailable'|'invalidCacheRoot'|'unsupportedPlatform'|null;
 /** Sanitized public quota fields only. No account/plan/private paths/credentials.
  * Provider.updated is original source timestamp; missing/stale stays unknown.
  * Keys may be codex,claude,copilot,antigravity; absent is NOT zero/unlimited. */
 providers:Record<string,{updated?:number;available?:boolean;blocked?:boolean;stale:boolean;windows?:{used:number;label?:string;name?:string;reset?:number|string;reset_at?:number}[];unlimited?:string[]}>;
}
// invoke('pet_cached_usage') with NO payload/path/provider/refresh parameters.
// Cache-only, no helpers, network, native session client or model invocation.
// Uses current process XDG_CACHE_HOME; never falls back to real-home cache when QA root is set.
// Native windows island/settings/companions/pet-chat/pet-control/pet-panel only.
