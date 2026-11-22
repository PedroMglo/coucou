import {invoke} from '@tauri-apps/api/core';
import {sessionTargetKey} from './session-chat';
import {DraftStorageFailure,IndexedDraftStorage,DRAFT_LIMITS,draftTarget,parseStoredDraft,type DraftTarget,type DraftStorage,type DraftFailure,type DraftWrite,type StoredDraft} from './session-drafts';
import {IndexedDraftMigration,type DraftMigration,type DraftImportReceipt} from './draft-migration';
type Call=<T>(command:string,args?:Record<string,unknown>)=>Promise<T>;
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const units=(s:string)=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i));
function text(raw:unknown,max:number){
 if(!Array.isArray(raw)||raw.length>max||raw.some(v=>!Number.isInteger(v)||v<0||v>65535))throw new DraftStorageFailure('invalidData');
 let value='';for(let i=0;i<raw.length;i+=1024)value+=String.fromCharCode(...raw.slice(i,i+1024));return value;
}
export function draftWireTarget(target:DraftTarget){const t=draftTarget(target);return{petId:units(t.petId),provider:units(t.provider),origin:t.origin,sessionId:units(t.sessionId),conversationId:t.conversationId===undefined?null:units(t.conversationId)};}
export function nativeDraftTarget(raw:unknown):DraftTarget{
 if(!object(raw))throw new DraftStorageFailure('invalidData');
 return draftTarget({petId:text(raw.petId,128),provider:text(raw.provider,128),origin:raw.origin,sessionId:text(raw.sessionId,512),...(raw.conversationId===null?{}:{conversationId:text(raw.conversationId,512)})});
}
export function parseNativeDraft(raw:unknown,target:DraftTarget):StoredDraft{
 if(!object(raw)||!object(raw.target))throw new DraftStorageFailure('invalidData');const t=raw.target;
 const exact=nativeDraftTarget(t);
 return parseStoredDraft({schemaVersion:1,key:sessionTargetKey(exact),target:exact,revision:raw.revision,text:text(raw.textUnits,DRAFT_LIMITS.text)},target);
}
async function guarded<T>(call:()=>Promise<T>):Promise<T>{try{return await call();}catch(e){if(e instanceof DraftStorageFailure)throw e;const code=typeof e==='string'?e:object(e)?e.code:undefined;throw new DraftStorageFailure(['blocked','invalidData','limit','quota','migrationConflict'].includes(code as string)?code as DraftFailure:'unavailable');}}
export class NativeDraftStorage implements DraftStorage {
 private preparing:Promise<void>|null=null;
 constructor(private call:Call=invoke,private legacy:DraftMigration=new IndexedDraftMigration()){}
 /** Eager full known-record migration, not only the selected conversation. */
 ready(){
  if(this.preparing)return this.preparing;
  const pending=guarded(async()=>{
   const batch=await this.legacy.snapshot();
   const wire={schemaVersion:1,sourceId:batch.sourceId,revision:batch.revision,records:batch.records.map(r=>{const exact=parseStoredDraft(r,r.target);return{target:draftWireTarget(exact.target),revision:exact.revision,textUnits:units(exact.text)};})};
   const receipt=await this.call<DraftImportReceipt>('pet_draft_import',{batch:wire});
   if(!object(receipt)||receipt.schemaVersion!==1||receipt.sourceId!==batch.sourceId||typeof receipt.fingerprint!=='string'||!/^[0-9a-f]{64}$/.test(receipt.fingerprint))throw new DraftStorageFailure('invalidData');
   await this.legacy.ack(receipt);
  });
  this.preparing=pending;void pending.catch(()=>{if(this.preparing===pending)this.preparing=null;});return pending;
 }
 async read(target:DraftTarget){const exact=draftTarget(target);await this.ready();return guarded(async()=>{const raw=await this.call<unknown>('pet_draft_read',{target:draftWireTarget(exact)});return raw===null?null:parseNativeDraft(raw,exact);});}
 async write(target:DraftTarget,expected:string|null,value:string):Promise<DraftWrite>{
  const exact=draftTarget(target);if(typeof value!=='string'||value.length>DRAFT_LIMITS.text)throw new DraftStorageFailure('limit');await this.ready();
  return guarded(async()=>{
   const raw=await this.call<unknown>('pet_draft_write',{target:draftWireTarget(exact),expectedRevision:expected,textUnits:units(value)});
   if(!object(raw)||(raw.kind!=='saved'&&raw.kind!=='conflict'))throw new DraftStorageFailure('invalidData');
   return raw.kind==='saved'?{kind:'saved',record:parseNativeDraft(raw.record,exact)}:{kind:'conflict',record:raw.record===null?null:parseNativeDraft(raw.record,exact)};
  });
 }
}
/** Backend declares the storage implementation. No error-triggered fallback or
 * v2 migration on platforms that still implement only IndexedDB persistence. */
export class ApplicationDraftStorage implements DraftStorage {
 private selected:Promise<DraftStorage>|null=null;
 constructor(private call:Call=invoke){}
 private select(){if(this.selected)return this.selected;const pending=guarded(async()=>{const mode=await this.call<string>('pet_draft_mode');if(mode==='nativeV1'){const storage=new NativeDraftStorage(this.call);await storage.ready();return storage;}if(mode==='indexedV1')return new IndexedDraftStorage();throw new DraftStorageFailure('unavailable');});this.selected=pending;void pending.catch(()=>{if(this.selected===pending)this.selected=null;});return pending;}
 async ready(){await this.select();}
 async read(target:DraftTarget){return(await this.select()).read(target);}
 async write(target:DraftTarget,expected:string|null,value:string){return(await this.select()).write(target,expected,value);}
}
