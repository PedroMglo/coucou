import {sessionTargetKey,type SessionTarget} from './session-chat';
export const DRAFT_LIMITS={records:1024,text:65536,total:2097152} as const;
const DB_NAME='coucou.sessionDrafts.v1',DB_VERSION=1,CHANNEL='coucou.sessionDrafts.changed.v1';
const MAX_REV=18446744073709551615n;
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const ordinal=(v:unknown):v is string=>typeof v==='string'&&/^(0|[1-9][0-9]{0,19})$/.test(v)&&BigInt(v)<=MAX_REV;
const id=(v:unknown,max=512):v is string=>typeof v==='string'&&v.length>0&&v.length<=max&&!v.includes('\0');
export function draftTarget(raw:unknown):SessionTarget{
 if(!object(raw)||!id(raw.petId,128)||!id(raw.provider,128)||(raw.origin!=='mochi'&&raw.origin!=='external')||!id(raw.sessionId)||!(raw.conversationId===undefined||id(raw.conversationId)))throw new DraftStorageFailure('invalidData');
 return {petId:raw.petId,provider:raw.provider,origin:raw.origin,sessionId:raw.sessionId,...(raw.conversationId===undefined?{}:{conversationId:raw.conversationId})};
}
function targetFromKey(key:string){try{const v=JSON.parse(key);if(!Array.isArray(v)||v.length!==5||typeof v[4]!=='string')return null;const t=draftTarget({petId:v[0],provider:v[1],origin:v[2],sessionId:v[3],...(v[4]?{conversationId:v[4]}:{})});return sessionTargetKey(t)===key?t:null;}catch{return null;}}
export interface StoredDraft {schemaVersion:1;key:string;target:SessionTarget;revision:string;text:string}
interface Usage {key:'usage';revision:string;records:number;units:number}
export function parseStoredDraft(raw:unknown,target:SessionTarget):StoredDraft{
 if(!object(raw)||raw.schemaVersion!==1||raw.key!==sessionTargetKey(target)||!ordinal(raw.revision)||raw.revision==='0'||typeof raw.text!=='string'||raw.text.length>DRAFT_LIMITS.text)throw new DraftStorageFailure('invalidData');
 const exact=draftTarget(raw.target);if(sessionTargetKey(exact)!==raw.key)throw new DraftStorageFailure('invalidData');
 return{schemaVersion:1,key:raw.key,target:exact,revision:raw.revision,text:raw.text};
}
function parseUsage(raw:unknown):Usage{
 if(!object(raw)||raw.key!=='usage'||!ordinal(raw.revision)||typeof raw.records!=='number'||!Number.isSafeInteger(raw.records)||raw.records<0||raw.records>DRAFT_LIMITS.records||typeof raw.units!=='number'||!Number.isSafeInteger(raw.units)||raw.units<0||raw.units>DRAFT_LIMITS.total)throw new DraftStorageFailure('invalidData');
 return{key:'usage',revision:raw.revision,records:raw.records,units:raw.units};
}
export type DraftFailure='unavailable'|'blocked'|'invalidData'|'limit'|'quota';
export class DraftStorageFailure extends Error{constructor(readonly code:DraftFailure){super(code);}}
const failure=(error:unknown):DraftFailure=>error instanceof DraftStorageFailure?error.code:object(error)&&error.name==='QuotaExceededError'?'quota':'unavailable';
export type DraftWrite={kind:'saved';record:StoredDraft}|{kind:'conflict';record:StoredDraft|null};
export interface DraftStorage {read(target:SessionTarget):Promise<StoredDraft|null>;write(target:SessionTarget,expected:string|null,text:string):Promise<DraftWrite>}
/** A dedicated origin-local database. No profiles, paths, IPC or network calls. */
export class IndexedDraftStorage implements DraftStorage {
 private opened:Promise<IDBDatabase>|null=null;
 constructor(private factory:()=>IDBFactory|undefined=()=>globalThis.indexedDB){}
 private open(){
  if(this.opened)return this.opened;
  let pending:Promise<IDBDatabase>;
  pending=new Promise((resolve,reject)=>{
   let request:IDBOpenDBRequest,abandoned=false;
   try{const factory=this.factory();if(!factory)throw new DraftStorageFailure('unavailable');request=factory.open(DB_NAME,DB_VERSION);}catch(error){reject(new DraftStorageFailure(failure(error)));return;}
   request.onupgradeneeded=()=>{
    const db=request.result;
    if(!db.objectStoreNames.contains('drafts'))db.createObjectStore('drafts',{keyPath:'key'});
    if(!db.objectStoreNames.contains('meta')){const store=db.createObjectStore('meta',{keyPath:'key'});store.put({key:'usage',revision:'0',records:0,units:0});}
   };
   request.onblocked=()=>{abandoned=true;reject(new DraftStorageFailure('blocked'));};
   request.onerror=()=>reject(new DraftStorageFailure(failure(request.error)));
   request.onsuccess=()=>{
    const db=request.result;if(abandoned){db.close();return;}
    db.onversionchange=()=>{db.close();if(this.opened===pending)this.opened=null;};resolve(db);
   };
  });
  this.opened=pending;void pending.catch(()=>{if(this.opened===pending)this.opened=null;});return pending;
 }
 async read(target:SessionTarget):Promise<StoredDraft|null>{
  const exact=draftTarget(target),db=await this.open();
  return new Promise((resolve,reject)=>{
   let transaction:IDBTransaction,record:StoredDraft|null=null,error:unknown;
   try{transaction=db.transaction(['drafts','meta'],'readonly');}catch(e){reject(new DraftStorageFailure(failure(e)));return;}
   const abort=(e:unknown)=>{error=e;transaction.abort();};
   const meta=transaction.objectStore('meta').get('usage');meta.onsuccess=()=>{try{parseUsage(meta.result);}catch(e){abort(e);}};
   const request=transaction.objectStore('drafts').get(sessionTargetKey(exact));request.onsuccess=()=>{try{record=request.result===undefined?null:parseStoredDraft(request.result,exact);}catch(e){abort(e);}};
   transaction.oncomplete=()=>resolve(record);transaction.onabort=()=>reject(new DraftStorageFailure(failure(error??transaction.error)));transaction.onerror=()=>{};
  });
 }
 async write(target:SessionTarget,expected:string|null,text:string):Promise<DraftWrite>{
  const exact=draftTarget(target);if(typeof text!=='string'||text.length>DRAFT_LIMITS.text)throw new DraftStorageFailure('limit');if(!(expected===null||ordinal(expected)&&expected!=='0'))throw new DraftStorageFailure('invalidData');
  const db=await this.open();
  return new Promise((resolve,reject)=>{
   let transaction:IDBTransaction,result:DraftWrite|undefined,error:unknown;
   try{transaction=db.transaction(['drafts','meta'],'readwrite');}catch(e){reject(new DraftStorageFailure(failure(e)));return;}
   const abort=(e:unknown)=>{error=e;transaction.abort();};
   const drafts=transaction.objectStore('drafts'),meta=transaction.objectStore('meta'),read=drafts.get(sessionTargetKey(exact));
   read.onsuccess=()=>{try{
    const prior=read.result===undefined?null:parseStoredDraft(read.result,exact);
    if((prior?.revision??null)!==expected){result={kind:'conflict',record:prior};return;}
    const usage=meta.get('usage');usage.onsuccess=()=>{try{
     const stats=parseUsage(usage.result),records=stats.records+(prior?0:1),units=stats.units-(prior?.text.length??0)+text.length;
     if(records>DRAFT_LIMITS.records||units>DRAFT_LIMITS.total||units<0||BigInt(stats.revision)===MAX_REV)throw new DraftStorageFailure('limit');
     const revision=(BigInt(stats.revision)+1n).toString(),record:StoredDraft={schemaVersion:1,key:sessionTargetKey(exact),target:exact,revision,text};
     drafts.put(record);meta.put({key:'usage',revision,records,units});result={kind:'saved',record};
    }catch(e){abort(e);}};
   }catch(e){abort(e);}};
   transaction.oncomplete=()=>result?resolve(result):reject(new DraftStorageFailure('unavailable'));
   transaction.onabort=()=>reject(new DraftStorageFailure(failure(error??transaction.error)));transaction.onerror=()=>{};
  });
 }
 close(){void this.opened?.then(db=>db.close(),()=>{});this.opened=null;}
}
export type DraftPhase='loading'|'saving'|'saved'|'restored'|'error'|'conflict';
export interface DraftStatus {phase:DraftPhase;busy:boolean;error:DraftFailure|null;storedText:string|null;revision:string|null;localText:string}
interface Entry {key:string;target:SessionTarget;loaded:boolean;base:StoredDraft|null;edit:number;dirty:boolean;conflict:boolean;incoming:StoredDraft|null;error:DraftFailure|null;loading:boolean;writing:boolean;restored:boolean;hintPending:boolean;read:number;loadTask:Promise<void>|null;writeTask:Promise<void>|null}
class DraftMap extends Map<string,string>{
 constructor(private edit:(key:string,value:string)=>void){super();}
 override set(key:string,value:string){const prior=super.get(key);super.set(key,value);if((prior??'')!==value)this.edit(key,value);return this;}
 raw(key:string,value:string){super.set(key,value);}
 override delete(key:string){const had=this.has(key);this.set(key,'');super.delete(key);return had;}
 override clear(){for(const key of this.keys())this.delete(key);}
}
export interface DraftHooks {changed?:(key:string)=>void;restore?:(target:SessionTarget,text:string,previous:string)=>boolean;published?:(key:string)=>void}
/** Exact-target RAM drafts plus asynchronous, compare-and-set persistence. */
export class SessionDrafts {
 readonly map:DraftMap;
 private entries=new Map<string,Entry>();private active='';private context=0;private activation=0;private disposed=false;
 constructor(private storage:DraftStorage,private hooks:DraftHooks={}){this.map=new DraftMap((key,text)=>this.edit(key,text));}
 private entry(target:SessionTarget){const exact=draftTarget(target),key=sessionTargetKey(exact);let e=this.entries.get(key);if(!e){e={key,target:exact,loaded:false,base:null,edit:0,dirty:false,conflict:false,incoming:null,error:null,loading:false,writing:false,restored:false,hintPending:false,read:0,loadTask:null,writeTask:null};this.entries.set(key,e);}return e;}
 private changed(e:Entry){if(!this.disposed)this.hooks.changed?.(e.key);}
 activate(target:SessionTarget|null,context=0){
  let key='';try{if(target)key=sessionTargetKey(draftTarget(target));}catch{/* Invalid identities stay RAM-only. */}
  if(key===this.active&&context===this.context)return;this.active=key;this.context=context;this.activation++;
  if(key&&target){const e=this.entry(target);if(!e.dirty&&!e.writing&&!e.conflict){if(e.loading){e.read++;e.loading=false;e.loadTask=null;}void this.load(e);}this.changed(e);}
 }
 status(target:SessionTarget|null):DraftStatus|null{
  if(!target)return null;let e:Entry;try{e=this.entry(target);}catch{return{phase:'error',busy:false,error:'invalidData',storedText:null,revision:null,localText:''};}
  return{busy:e.loading||e.writing,phase:e.conflict?'conflict':e.error?'error':e.loading?'loading':e.dirty||e.writing?'saving':e.restored?'restored':'saved',error:e.error,storedText:e.conflict?e.incoming?.text??'':null,revision:e.conflict?e.incoming?.revision??null:e.base?.revision??null,localText:this.map.get(e.key)??''};
 }
 private edit(key:string,text:string){
  const target=targetFromKey(key);if(!target||this.disposed)return;const e=this.entry(target);e.edit++;e.dirty=true;e.restored=false;e.error=text.length>DRAFT_LIMITS.text?'limit':null;this.changed(e);
  if(!e.loaded&&!e.loading)void this.load(e);else this.queue(e);
 }
 private queue(e:Entry){queueMicrotask(()=>{if(!this.disposed&&e.loaded&&e.dirty&&!e.loading&&!e.writing&&!e.conflict&&!e.error)void this.write(e);});}
 private load(e:Entry):Promise<void>{
  if(e.loadTask)return e.loadTask;const read=++e.read,edit=e.edit,activation=this.activation;e.loading=true;e.error=null;this.changed(e);
  const task=Promise.resolve().then(()=>this.storage.read(e.target)).then(record=>{
   if(this.disposed||read!==e.read)return;
   record=record===null?null:parseStoredDraft(record,e.target);
   if(e.loaded&&e.base&&record&&(BigInt(record.revision)<BigInt(e.base.revision)||record.revision===e.base.revision&&record.text!==e.base.text))throw new DraftStorageFailure('invalidData');
   const local=this.map.get(e.key)??'',sameBase=e.loaded&&(e.base?.revision??null)===(record?.revision??null);
   if(e.dirty){
    if(record?.text===local){e.base=record;e.loaded=true;e.dirty=false;e.conflict=false;e.incoming=null;}
    else if(sameBase||!e.loaded&&record===null){e.base=record;e.loaded=true;}
    else{e.conflict=true;e.incoming=record;e.loaded=true;}
   }else if(edit===e.edit&&activation===this.activation&&this.active===e.key){
    const text=record?.text??'';
    if(local!==text){this.map.raw(e.key,text);if(this.hooks.restore&&!this.hooks.restore(e.target,text,local)){this.map.raw(e.key,local);e.conflict=true;e.incoming=record;e.loaded=true;return;}}
    e.base=record;e.loaded=true;e.conflict=false;e.incoming=null;e.restored=!!text;
   }
   this.changed(e);
  }).catch(error=>{if(!this.disposed&&read===e.read){e.error=failure(error);this.changed(e);}}).finally(()=>{
   if(read===e.read){e.loading=false;e.loadTask=null;if((this.map.get(e.key)??'').length>DRAFT_LIMITS.text)e.error='limit';this.changed(e);const again=e.hintPending;e.hintPending=false;if(again&&this.active===e.key&&!e.dirty&&!e.conflict&&!e.error)void this.load(e);else this.queue(e);}
  });
  e.loadTask=task;return task;
 }
 private write(e:Entry):Promise<void>{
  if(e.writeTask)return e.writeTask;const text=this.map.get(e.key)??'',edit=e.edit,expected=e.base?.revision??null;e.writing=true;e.error=null;this.changed(e);
  const task=Promise.resolve().then(()=>this.storage.write(e.target,expected,text)).then(receipt=>{
   if(this.disposed)return;
   if(!object(receipt)||(receipt.kind!=='saved'&&receipt.kind!=='conflict'))throw new DraftStorageFailure('invalidData');
   if(receipt.kind==='conflict'){e.conflict=true;e.incoming=receipt.record===null?null:parseStoredDraft(receipt.record,e.target);}
   else{const saved=parseStoredDraft(receipt.record,e.target);if(saved.text!==text||BigInt(saved.revision)<=BigInt(expected??'0'))throw new DraftStorageFailure('invalidData');e.base=saved;e.loaded=true;e.dirty=e.edit!==edit||(this.map.get(e.key)??'')!==text;e.restored=false;this.hooks.published?.(e.key);}
   this.changed(e);
  }).catch(error=>{if(!this.disposed){e.error=failure(error);this.changed(e);}}).finally(()=>{e.writing=false;e.writeTask=null;this.changed(e);this.queue(e);});
  e.writeTask=task;return task;
 }
 /** Identity-only hints never publish text or set a version. */
 hint(key:unknown){if(typeof key!=='string'||key!==this.active||!targetFromKey(key))return;const e=this.entries.get(key);if(e&&!e.writing&&!e.dirty&&!e.conflict){if(e.loading)e.hintPending=true;else void this.load(e);}}
 async retry(target:SessionTarget){const e=this.entry(target);if(e.writing)return;e.error=null;if(!e.conflict)await this.load(e);this.queue(e);}
 async useStored(target:SessionTarget,expectedConflict:string|null){
  const e=this.entry(target),activation=this.activation,edit=e.edit;if(this.active!==e.key||!e.conflict||e.loading||e.writing||(e.incoming?.revision??null)!==expectedConflict)return;
  e.loading=true;e.error=null;this.changed(e);
  try{
   const raw=await this.storage.read(e.target),record=raw===null?null:parseStoredDraft(raw,e.target);
   if(this.disposed||activation!==this.activation||this.active!==e.key||edit!==e.edit)return;
   const previous=this.map.get(e.key)??'',text=record?.text??'';this.map.raw(e.key,text);
   if(this.hooks.restore&&!this.hooks.restore(e.target,text,previous)){this.map.raw(e.key,previous);return;}
   e.base=record;e.loaded=true;e.dirty=false;e.conflict=false;e.incoming=null;e.restored=!!text;
  }catch(error){if(!this.disposed&&activation===this.activation&&edit===e.edit)e.error=failure(error);}
  finally{e.loading=false;this.changed(e);}
 }
 keepLocal(target:SessionTarget,expectedConflict:string|null){
  const e=this.entry(target);if(this.active!==e.key||!e.conflict||e.loading||(e.incoming?.revision??null)!==expectedConflict)return;
  e.base=e.incoming;e.loaded=true;e.conflict=false;e.incoming=null;e.error=(this.map.get(e.key)??'').length>DRAFT_LIMITS.text?'limit':null;e.dirty=true;this.changed(e);this.queue(e);
 }
 async flush(target:SessionTarget){const e=this.entry(target);if(e.loadTask)await e.loadTask;if(e.writeTask)await e.writeTask;if(e.dirty&&!e.error&&!e.conflict&&e.loaded)await this.write(e);}
 dispose(){this.disposed=true;this.active='';this.activation++;}
}
export function draftBroadcast(resource:SessionDrafts):{publish:(key:string)=>void;close:()=>void}{
 try{if(typeof BroadcastChannel==='undefined')throw Error();const channel=new BroadcastChannel(CHANNEL);channel.onmessage=e=>{if(object(e.data)&&e.data.schemaVersion===1)resource.hint(e.data.key);};return{publish:key=>channel.postMessage({schemaVersion:1,key}),close:()=>channel.close()};}
 catch{return{publish:()=>{},close:()=>{}};}
}
