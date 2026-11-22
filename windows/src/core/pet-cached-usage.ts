import {Bridge} from './bridge';
import type {PetCachedUsage} from './pet-cached-usage-contract';
export type {PetCachedUsage} from './pet-cached-usage-contract';
const providers=['codex','claude','copilot','antigravity'] as const;
const stamp=(v:unknown):v is number=>typeof v==='number'&&Number.isSafeInteger(v)&&v>0&&v<=8.64e12;
const text=(v:unknown):v is string=>typeof v==='string'&&v.length>0&&v.length<=128&&!/[\x00-\x1f\x7f]/.test(v);
const record=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
export function quotaProvider(provider:string){return provider==='claude-cli'?'claude':providers.includes(provider as typeof providers[number])?provider:null;}
/** Keep only the public contract. No account, plan, note, credential or path is
 * rendered, even if an incompatible caller supplies extra properties. */
export function parseCachedUsage(input:unknown):PetCachedUsage {
 if(!record(input)||input.schemaVersion!==1||input.source!=='ia-barra-local-cache'||!stamp(input.readAt)||!['available','unavailable','notSupported'].includes(String(input.status))||!record(input.providers)||!(input.reason===null||['cacheUnavailable','invalidCacheRoot','unsupportedPlatform'].includes(String(input.reason))))throw Error('Quota guardada com formato incompatível.');
 const result:PetCachedUsage={schemaVersion:1,source:'ia-barra-local-cache',readAt:input.readAt,status:input.status as PetCachedUsage['status'],reason:input.reason as PetCachedUsage['reason'],providers:{}};
 if(result.status!=='available')return result;
 for(const provider of providers){
  const q=input.providers[provider];if(!record(q))continue;
  if(typeof q.stale!=='boolean'||q.updated!==undefined&&(!stamp(q.updated)||q.updated>result.readAt)||['available','blocked'].some(key=>q[key]!==undefined&&typeof q[key]!=='boolean'))continue;
  if(q.windows!==undefined&&(!Array.isArray(q.windows)||q.windows.length>32))continue;
  if(q.unlimited!==undefined&&(!Array.isArray(q.unlimited)||q.unlimited.length>16||!q.unlimited.every(text)))continue;
  const windows:NonNullable<PetCachedUsage['providers'][string]['windows']>=[];let valid=true;
  for(const w of (q.windows??[]) as unknown[]){
   if(!record(w)||typeof w.used!=='number'||!Number.isFinite(w.used)||w.used<0||w.used>100||['label','name'].some(key=>w[key]!==undefined&&!text(w[key]))){valid=false;break;}
   const window:typeof windows[number]={used:w.used};if(text(w.label))window.label=w.label;if(text(w.name))window.name=w.name;
   if(w.reset_at!==undefined){if(!stamp(w.reset_at)){valid=false;break;}window.reset_at=w.reset_at;}
   if(w.reset!==undefined){if(stamp(w.reset)||text(w.reset))window.reset=w.reset;else{valid=false;break;}}
   windows.push(window);
  }
  if(!valid)continue;
  result.providers[provider]={stale:q.stale||!stamp(q.updated),windows,...(stamp(q.updated)?{updated:q.updated}:{}),...(typeof q.available==='boolean'?{available:q.available}:{}),...(typeof q.blocked==='boolean'?{blocked:q.blocked}:{}),...(Array.isArray(q.unlimited)?{unlimited:q.unlimited as string[]}:{} )};
 }
 return result;
}
/** Shared metadata read. No timer or account/network refresh. Failures are
 * retried only explicitly; successful reads have a short local cooldown. */
export class CachedUsageResource {
 snapshot:PetCachedUsage|null=null;loading=false;error='';
 private pending:Promise<void>|null=null;private last=-Infinity;private listeners=new Set<()=>void>();
 constructor(private load:()=>Promise<unknown>,private now=()=>Date.now()){}
 subscribe(listener:()=>void){this.listeners.add(listener);return()=>this.listeners.delete(listener);}
 private changed(){for(const listener of this.listeners)listener();}
 refresh(manual=false):Promise<void>{
  if(this.pending)return this.pending;if(!manual&&(this.error||this.now()-this.last<30000))return Promise.resolve();
  this.last=this.now();this.loading=true;
  this.pending=Promise.resolve().then(()=>this.load()).then(value=>{this.snapshot=parseCachedUsage(value);this.error='';}).catch(()=>{this.error='Não foi possível reler a quota guardada. Os dados anteriores mantêm a data original.';}).finally(()=>{this.loading=false;this.pending=null;this.changed();});
  this.changed();return this.pending;
 }
}
export const PetCachedUsageResource=new CachedUsageResource(()=>Bridge.petCachedUsage());
export function quotaReset(value:number|string|undefined):string|null {
 if(typeof value==='number'&&stamp(value))return new Date(value*1000).toLocaleString('pt-PT',{day:'numeric',month:'short',year:'numeric',hour:'2-digit',minute:'2-digit'});
 // Only a parseable ISO timestamp may be formatted as a date. Other native
 // labels remain a label, never a fabricated local renewal timestamp.
 if(typeof value==='string'&&text(value)){
  if(/^\d{4}-\d\d-\d\dT/.test(value)&&Number.isFinite(Date.parse(value)))return new Date(value).toLocaleString('pt-PT',{day:'numeric',month:'short',year:'numeric',hour:'2-digit',minute:'2-digit'});
  return value;
 }
 return null;
}
