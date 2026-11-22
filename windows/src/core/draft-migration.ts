import {DraftStorageFailure,DRAFT_LIMITS,draftTarget,parseStoredDraft,parseUsage,type StoredDraft} from './session-drafts';
export interface LegacyDraftBatch {schemaVersion:1;sourceId:string;revision:string;records:StoredDraft[]}
export interface DraftImportReceipt {schemaVersion:1;sourceId:string;fingerprint:string}
export interface DraftMigration {snapshot():Promise<LegacyDraftBatch>;ack(receipt:DraftImportReceipt):Promise<void>;ackResolved?(receipt:DraftImportReceipt,expected:LegacyDraftBatch):Promise<void>}
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
export function draftImportReceipt(raw:unknown):DraftImportReceipt{
 if(!object(raw)||raw.schemaVersion!==1||typeof raw.sourceId!=='string'||!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(raw.sourceId)||typeof raw.fingerprint!=='string'||!/^[0-9a-f]{64}$/.test(raw.fingerprint))throw new DraftStorageFailure('invalidData');return{schemaVersion:1,sourceId:raw.sourceId,fingerprint:raw.fingerprint};
}
export function canonicalLegacyBatch(batch:LegacyDraftBatch):string{
 if(batch.schemaVersion!==1||!Array.isArray(batch.records)||batch.records.length>DRAFT_LIMITS.records)throw new DraftStorageFailure('invalidData');
 draftImportReceipt({schemaVersion:1,sourceId:batch.sourceId,fingerprint:'0'.repeat(64)});
 const rows=batch.records.map(r=>parseStoredDraft(r,draftTarget(r.target))),units=rows.reduce((n,r)=>n+r.text.length,0);
 parseUsage({key:'usage',revision:batch.revision,records:rows.length,units});
 if(new Set(rows.map(r=>r.key)).size!==rows.length||rows.some(r=>r.target.origin==='unbound'||BigInt(r.revision)>BigInt(batch.revision)))throw new DraftStorageFailure('invalidData');
 rows.sort((a,b)=>a.key<b.key?-1:a.key>b.key?1:0);return JSON.stringify({schemaVersion:1,sourceId:batch.sourceId,revision:batch.revision,records:rows});
}
function migration(raw:unknown):{key:'nativeMigration';schemaVersion:1;sourceId:string;receipt?:DraftImportReceipt}{
 if(!object(raw)||raw.key!=='nativeMigration'||raw.schemaVersion!==1||typeof raw.sourceId!=='string'||!/^([0-9a-f]{8}-[0-9a-f]{4}-){1}[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(raw.sourceId))throw new DraftStorageFailure('invalidData');
 if(raw.receipt!==undefined&&draftImportReceipt(raw.receipt).sourceId!==raw.sourceId)throw new DraftStorageFailure('invalidData');
 return raw as ReturnType<typeof migration>;
}
/** Only the dedicated Coucou draft DB is opened. v2 fences old v1 writers;
 * original drafts/usage are retained unchanged, including after native ACK. */
export class IndexedDraftMigration implements DraftMigration {
 private opened:Promise<IDBDatabase>|null=null;
 constructor(private factory:()=>IDBFactory|undefined=()=>globalThis.indexedDB,private source:()=>string=()=>crypto.randomUUID()){}
 private open(){
  if(this.opened)return this.opened;let pending:Promise<IDBDatabase>;
  pending=new Promise((resolve,reject)=>{
   let request:IDBOpenDBRequest,abandoned=false;
   try{const factory=this.factory();if(!factory)throw new DraftStorageFailure('unavailable');request=factory.open('coucou.sessionDrafts.v1',2);}catch{reject(new DraftStorageFailure('unavailable'));return;}
   request.onupgradeneeded=e=>{
    const db=request.result,tx=request.transaction!;
    if(abandoned){tx.abort();return;}
    try{
     if((e as IDBVersionChangeEvent).oldVersion===0){db.createObjectStore('drafts',{keyPath:'key'});db.createObjectStore('meta',{keyPath:'key'}).put({key:'usage',revision:'0',records:0,units:0});}
     // Missing/contradictory legacy stores abort instead of manufacturing data.
     tx.objectStore('drafts');const meta=tx.objectStore('meta');
     meta.add(migration({key:'nativeMigration',schemaVersion:1,sourceId:this.source()}));
    }catch{tx.abort();}
   };
   request.onblocked=()=>{abandoned=true;reject(new DraftStorageFailure('blocked'));};
   request.onerror=()=>reject(new DraftStorageFailure('unavailable'));
   request.onsuccess=()=>{const db=request.result;if(abandoned){db.close();return;}db.onversionchange=()=>{db.close();if(this.opened===pending)this.opened=null;};resolve(db);};
  });
  this.opened=pending;void pending.catch(()=>{if(this.opened===pending)this.opened=null;});return pending;
 }
 async snapshot():Promise<LegacyDraftBatch>{
  const db=await this.open();return new Promise((resolve,reject)=>{
   let tx:IDBTransaction,error:unknown,batch:LegacyDraftBatch;
   try{tx=db.transaction(['drafts','meta'],'readonly');}catch{reject(new DraftStorageFailure('unavailable'));return;}
   const meta=tx.objectStore('meta'),usage=meta.get('usage'),source=meta.get('nativeMigration'),rows:StoredDraft[]=[];
   const cursor=tx.objectStore('drafts').openCursor();cursor.onsuccess=()=>{try{
    const c=cursor.result;if(!c)return;if(rows.length>=DRAFT_LIMITS.records)throw new DraftStorageFailure('limit');
    rows.push(parseStoredDraft(c.value,draftTarget(c.value?.target)));c.continue();
   }catch(e){error=e;tx.abort();}};
   tx.oncomplete=()=>{try{
    const stats=parseUsage(usage.result),origin=migration(source.result),keys=new Set(rows.map(r=>r.key));
    if(keys.size!==rows.length||stats.records!==rows.length||stats.units!==rows.reduce((n,r)=>n+r.text.length,0)||rows.some(r=>BigInt(r.revision)>BigInt(stats.revision)))throw new DraftStorageFailure('invalidData');
    batch={schemaVersion:1,sourceId:origin.sourceId,revision:stats.revision,records:rows};resolve(batch);
   }catch(e){reject(e);}};
   tx.onabort=()=>reject(error??new DraftStorageFailure('unavailable'));tx.onerror=()=>{};
  });
 }
 async ack(receipt:DraftImportReceipt){
  receipt=draftImportReceipt(receipt);
  const db=await this.open();return new Promise<void>((resolve,reject)=>{
   let tx:IDBTransaction,error:unknown;try{tx=db.transaction(['meta'],'readwrite');}catch{reject(new DraftStorageFailure('unavailable'));return;}
   const store=tx.objectStore('meta'),read=store.get('nativeMigration');read.onsuccess=()=>{try{
    const source=migration(read.result);if(source.sourceId!==receipt.sourceId||source.receipt&&source.receipt.fingerprint!==receipt.fingerprint)throw new DraftStorageFailure('migrationConflict');
    store.put({...source,receipt});
   }catch(e){error=e;tx.abort();}};
   tx.oncomplete=()=>resolve();tx.onabort=()=>reject(error??new DraftStorageFailure('unavailable'));tx.onerror=()=>{};
  });
 }
 /** Only after native human reconciliation. Capture all original rows and
  * usage in the same readwrite transaction that replaces its old receipt. */
 async ackResolved(receipt:DraftImportReceipt,expected:LegacyDraftBatch){
  receipt=draftImportReceipt(receipt);const exact=canonicalLegacyBatch(expected);
  if(receipt.sourceId!==expected.sourceId)throw new DraftStorageFailure('migrationConflict');
  const db=await this.open();return new Promise<void>((resolve,reject)=>{
   let tx:IDBTransaction,error:unknown;try{tx=db.transaction(['drafts','meta'],'readwrite');}catch{reject(new DraftStorageFailure('unavailable'));return;}
   const meta=tx.objectStore('meta'),usage=meta.get('usage'),origin=meta.get('nativeMigration'),rows:StoredDraft[]=[];
   const cursor=tx.objectStore('drafts').openCursor();cursor.onsuccess=()=>{try{
    const c=cursor.result;if(c){if(rows.length>=DRAFT_LIMITS.records)throw new DraftStorageFailure('limit');rows.push(parseStoredDraft(c.value,draftTarget(c.value?.target)));c.continue();return;}
    const stats=parseUsage(usage.result),source=migration(origin.result);
    if(stats.records!==rows.length||stats.units!==rows.reduce((n,r)=>n+r.text.length,0)||canonicalLegacyBatch({schemaVersion:1,sourceId:source.sourceId,revision:stats.revision,records:rows})!==exact)throw new DraftStorageFailure('migrationConflict');
    meta.put({...source,receipt});
   }catch(e){error=e;tx.abort();}};
   tx.oncomplete=()=>resolve();tx.onabort=()=>reject(error??new DraftStorageFailure('unavailable'));tx.onerror=()=>{};
  });
 }
 close(){void this.opened?.then(db=>db.close(),()=>{});this.opened=null;}
}
