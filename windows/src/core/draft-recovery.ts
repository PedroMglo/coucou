import {invoke} from '@tauri-apps/api/core';
import {draftWireTarget,nativeDraftTarget,parseNativeDraft} from './native-drafts';
import {sessionTargetKey} from './session-chat';
import {draftTarget,type DraftTarget,type StoredDraft} from './session-drafts';
import {draftImportReceipt,type DraftImportReceipt} from './draft-migration';
type Call=<T>(command:string,args?:Record<string,unknown>)=>Promise<T>;
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const revision=(v:unknown):v is string=>typeof v==='string'&&/^[1-9][0-9]{0,19}$/.test(v)&&BigInt(v)<=18446744073709551615n;
const catalogue=(v:unknown):v is string=>typeof v==='string'&&(v==='missing'||/^[0-9a-f]{64}$/.test(v));
export class DraftRecoveryFailure extends Error{constructor(readonly code:string){super(code);}}
export interface DraftArchive{receipt:DraftImportReceipt;archiveRevision:string}
export interface RecoverySource{reference:DraftArchive;records:number;revision:string;status:'retained'|'pending'}
export interface RecoveryEntry{target:DraftTarget;revision:string;textUnits:number;availability:'existing'|'orphan'|'unbound'|'externalReference'|'unknown';archive?:DraftArchive}
export interface RecoveryPage{catalogueRevision:string;entries:RecoveryEntry[];total:number;nextOffset:number|null}
export interface RecoveryReview{source:StoredDraft;destination:DraftTarget;destinationRecord:StoredDraft|null;expiresAt:number;fingerprint:string;raw:Record<string,unknown>}
function invalid():never{throw new DraftRecoveryFailure('invalidData');}
function archive(raw:unknown):DraftArchive{if(!object(raw)||typeof raw.archiveRevision!=='string'||!/^[0-9a-f]{64}$/.test(raw.archiveRevision))invalid();return{receipt:draftImportReceipt(raw.receipt),archiveRevision:raw.archiveRevision};}
/** Separate native-user data-management API. Does not unblock a divergent
 * legacy migration, import data, create a conversation or call a model. */
export class DraftRecovery{
 constructor(private call:Call=invoke){}
 private async native<T>(command:string,args:Record<string,unknown>):Promise<T>{
  try{if(await this.call('pet_draft_mode')!=='nativeV1')throw new DraftRecoveryFailure('unsupported');return await this.call<T>(command,args);}
  catch(e){if(e instanceof DraftRecoveryFailure)throw e;throw new DraftRecoveryFailure(typeof e==='string'?e:object(e)&&typeof e.code==='string'?e.code:'unavailable');}
 }
 async sources():Promise<RecoverySource[]>{
  const raw=await this.native<unknown>('pet_draft_sources',{});if(!Array.isArray(raw)||raw.length>8)invalid();const seen=new Set<string>();return raw.map(v=>{if(!object(v)||!Number.isInteger(v.records)||Number(v.records)<0||Number(v.records)>1024||!(v.revision==='0'||revision(v.revision))||!['retained','pending'].includes(String(v.status)))invalid();const reference=archive(v.reference),key=reference.receipt.sourceId+reference.receipt.fingerprint;if(seen.has(key))invalid();seen.add(key);return{reference,records:Number(v.records),revision:String(v.revision),status:v.status as RecoverySource['status']};});
 }
 async list(owner:DraftTarget,offset=0,expectedRevision:string|null=null,sourceArchive?:DraftArchive):Promise<RecoveryPage>{
  const exact=draftWireTarget(owner);
  const selected=sourceArchive?archive(sourceArchive):undefined;
  const raw=await this.native<unknown>('pet_draft_list',{owner:{petId:exact.petId,provider:exact.provider},offset,limit:16,expectedRevision,...(selected?{sourceArchive:selected}:{})});
  if(!object(raw)||!catalogue(raw.catalogueRevision)||!Array.isArray(raw.entries)||raw.entries.length>16||!Number.isInteger(raw.total)||Number(raw.total)<0||Number(raw.total)>1024||!(raw.nextOffset===null||Number.isInteger(raw.nextOffset)&&Number(raw.nextOffset)===offset+16&&Number(raw.nextOffset)<Number(raw.total)))invalid();
  const keys=new Set<string>();
  const entries=raw.entries.map(item=>{
   if(!object(item)||!revision(item.revision)||!Number.isInteger(item.textUnits)||Number(item.textUnits)<0||Number(item.textUnits)>65536||!['existing','orphan','unbound','externalReference','unknown'].includes(String(item.availability)))invalid();
   const target=nativeDraftTarget(item.target),key=sessionTargetKey(target);
   if(target.petId!==owner.petId||target.provider!==owner.provider||keys.has(key))invalid();keys.add(key);
   return{target,revision:item.revision,textUnits:Number(item.textUnits),availability:item.availability as RecoveryEntry['availability'],...(selected?{archive:structuredClone(selected)}:{})};
  });
  if(entries.length!==Math.min(16,Math.max(0,Number(raw.total)-offset))||(expectedRevision!==null&&raw.catalogueRevision!==expectedRevision))invalid();
  return{catalogueRevision:raw.catalogueRevision,entries,total:Number(raw.total),nextOffset:raw.nextOffset as number|null};
 }
 async review(entry:RecoveryEntry,destination:DraftTarget,catalogueRevision:string):Promise<RecoveryReview>{
  const exact=draftTarget(destination),request={catalogueRevision,source:draftWireTarget(entry.target),destination:draftWireTarget(exact),...(entry.archive?{sourceArchive:archive(entry.archive)}:{})};
  const raw=await this.native<unknown>('pet_draft_recovery_review',{request});
  if(!object(raw)||raw.schemaVersion!==1||!object(raw.request)||raw.request.catalogueRevision!==catalogueRevision||!Number.isSafeInteger(raw.expiresAt)||Number(raw.expiresAt)<=0||typeof raw.fingerprint!=='string'||!/^[0-9a-f]{64}$/.test(raw.fingerprint)||sessionTargetKey(nativeDraftTarget(raw.request.source))!==sessionTargetKey(entry.target)||sessionTargetKey(nativeDraftTarget(raw.request.destination))!==sessionTargetKey(exact))invalid();
  const source=parseNativeDraft(raw.sourceRecord,entry.target),destinationRecord=raw.destinationRecord===null?null:parseNativeDraft(raw.destinationRecord,exact);
  if(JSON.stringify(raw.request.sourceArchive?archive(raw.request.sourceArchive):null)!==JSON.stringify(entry.archive?archive(entry.archive):null))invalid();
  if(source.revision!==entry.revision||source.text.length!==entry.textUnits||exact.origin!=='mochi'||source.target.petId!==exact.petId||source.target.provider!==exact.provider)invalid();
  return{source,destination:exact,destinationRecord,expiresAt:Number(raw.expiresAt),fingerprint:raw.fingerprint,raw:structuredClone(raw)};
 }
 async copy(review:RecoveryReview):Promise<StoredDraft>{
  const raw=await this.native<unknown>('pet_draft_recovery_copy',{review:structuredClone(review.raw),approved:true});
  const saved=parseNativeDraft(raw,review.destination);
  if(saved.text!==review.source.text||BigInt(saved.revision)<=BigInt(review.destinationRecord?.revision??'0'))invalid();
  return saved;
 }
}
