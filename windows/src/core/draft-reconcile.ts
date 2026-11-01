import {invoke} from '@tauri-apps/api/core';
import {IndexedDraftMigration,canonicalLegacyBatch,draftImportReceipt,type DraftMigration,type DraftImportReceipt,type LegacyDraftBatch} from './draft-migration';
import {draftWireTarget,nativeDraftTarget,parseNativeDraft} from './native-drafts';
import {sessionTargetKey} from './session-chat';
import type {StoredDraft} from './session-drafts';
type Call=<T>(command:string,args?:Record<string,unknown>)=>Promise<T>;
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const hash=(v:unknown):v is string=>typeof v==='string'&&/^[0-9a-f]{64}$/.test(v);
const units=(s:string)=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i));
export type ReconcilePolicy='retainOnly'|'mergeCompatible';
export interface ReconcilePage{batch:LegacyDraftBatch;policy:ReconcilePolicy;raw:Record<string,unknown>;receipt:DraftImportReceipt;nativeRevision:string;expiresAt:number;fingerprint:string;summary:{total:number;missing:number;identical:number;conflicts:number;canMerge:boolean;alreadyRetained:boolean};rows:{original:StoredDraft;native:StoredDraft|null;status:'missing'|'identical'|'conflict'}[];nextOffset:number|null;offset:number}
export class DraftReconcileFailure extends Error{constructor(readonly code:string){super(code);}}
function invalid():never{throw new DraftReconcileFailure('invalidData');}
function wire(batch:LegacyDraftBatch){return{schemaVersion:1,sourceId:batch.sourceId,revision:batch.revision,records:batch.records.map(r=>({target:draftWireTarget(r.target),revision:r.revision,textUnits:units(r.text)}))};}
/** Reconciliation does not silently replace the ordinary import/ACK path. */
export class DraftReconcile{
 constructor(private call:Call=invoke,private legacy:DraftMigration=new IndexedDraftMigration()){}
 private async assertNative(){try{if(await this.call('pet_draft_mode')!=='nativeV1')throw new DraftReconcileFailure('unsupported');}catch(e){if(e instanceof DraftReconcileFailure)throw e;throw new DraftReconcileFailure('unavailable');}}
 private async native<T>(command:string,args:Record<string,unknown>):Promise<T>{
  try{if(await this.call('pet_draft_mode')!=='nativeV1')throw new DraftReconcileFailure('unsupported');return await this.call<T>(command,args);}
  catch(e){if(e instanceof DraftReconcileFailure)throw e;throw new DraftReconcileFailure(typeof e==='string'?e:object(e)&&typeof e.code==='string'?e.code:'unavailable');}
 }
 async review(policy:ReconcilePolicy='retainOnly',prior?:ReconcilePage,offset=0):Promise<ReconcilePage>{
  await this.assertNative();
  const current=structuredClone(await this.legacy.snapshot());canonicalLegacyBatch(current);
  if(prior&&canonicalLegacyBatch(prior.batch)!==canonicalLegacyBatch(current))throw new DraftReconcileFailure('reviewChanged');
  const batch=prior?structuredClone(prior.batch):current;
  const raw=await this.native<unknown>('pet_draft_reconcile_review',{batch:wire(batch),policy,offset,expectedRevision:prior?.nativeRevision??null,expiresAt:prior?.expiresAt??null});
  if(!object(raw)||!object(raw.review)||!Array.isArray(raw.rows)||raw.rows.length>16)invalid();const token=raw.review,summary=token.summary;
  const receipt=draftImportReceipt(token.receipt);
  if(token.schemaVersion!==1||receipt.sourceId!==batch.sourceId||token.policy!==policy||!hash(token.fingerprint)||!(hash(token.nativeRevision)||token.nativeRevision==='missing')||!Number.isSafeInteger(token.expiresAt)||Number(token.expiresAt)<=0||prior&&(token.nativeRevision!==prior.nativeRevision||token.expiresAt!==prior.expiresAt)||!object(summary))invalid();
  for(const key of ['total','missing','identical','conflicts'])if(!Number.isInteger(summary[key])||Number(summary[key])<0||Number(summary[key])>1024)invalid();
  if(summary.total!==batch.records.length||Number(summary.missing)+Number(summary.identical)+Number(summary.conflicts)!==summary.total||typeof summary.canMerge!=='boolean'||typeof summary.alreadyRetained!=='boolean'||summary.alreadyRetained&&(summary.canMerge||policy!=='retainOnly'))invalid();
  const seen=new Set<string>();const rows=raw.rows.map(value=>{
   if(!object(value))invalid();const t=nativeDraftTarget(object(value.original)?value.original.target:null),key=sessionTargetKey(t),expected=batch.records.find(r=>r.key===key);if(!expected||seen.has(key))invalid();seen.add(key);
   const original=parseNativeDraft(value.original,t),native=value.native===null?null:parseNativeDraft(value.native,t);
   if(original.text!==expected.text||original.revision!==expected.revision)invalid();
   const status:ReconcilePage['rows'][number]['status']=native===null?'missing':native.text===original.text&&native.revision===original.revision?'identical':'conflict';if(value.status!==status)invalid();return{original,native,status};
  });
  if(rows.length!==Math.min(16,Math.max(0,batch.records.length-offset))||!(raw.nextOffset===null||raw.nextOffset===offset+16&&Number(raw.nextOffset)<batch.records.length))invalid();
  return{batch,policy,raw:structuredClone(token),receipt,nativeRevision:token.nativeRevision,expiresAt:Number(token.expiresAt),fingerprint:token.fingerprint,summary:summary as unknown as ReconcilePage['summary'],rows,nextOffset:raw.nextOffset as number|null,offset};
 }
 async confirm(page:ReconcilePage):Promise<void>{
  await this.assertNative();
  if(!this.legacy.ackResolved)throw new DraftReconcileFailure('unsupported');
  if(canonicalLegacyBatch(await this.legacy.snapshot())!==canonicalLegacyBatch(page.batch))throw new DraftReconcileFailure('reviewChanged');
  const receipt=draftImportReceipt(await this.native('pet_draft_reconcile_commit',{batch:wire(page.batch),review:structuredClone(page.raw),approved:true}));
  if(receipt.sourceId!==page.receipt.sourceId||receipt.fingerprint!==page.receipt.fingerprint)invalid();
  await this.legacy.ackResolved(receipt,structuredClone(page.batch));
 }
}
