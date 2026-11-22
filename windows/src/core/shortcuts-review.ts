import {parseShortcutNativeProcessIdentity,parseShortcutNativeEnvironment,parseShortcutNativeSubject,parseShortcutEffectSlot,type ShortcutNativeSubject} from './shortcut-native-subject';
export {parseShortcutNativeProcessIdentity,parseShortcutNativeEnvironment} from './shortcut-native-subject';
export type {ShortcutNativeProcessIdentity,ShortcutNativeEnvironment} from './shortcut-native-subject';
import {SHORTCUTS} from './shortcuts';
import {settingsIoStatus,type SettingsIoStatus} from './settings-io';
export interface ShortcutReviewKey{token:string;fingerprint:string}
export interface ShortcutReviewPreview extends ShortcutReviewKey{
 schemaVersion:1|2|3;expiresAtMs:number;scope:'nativeUserShortcutsPreviewOnly'|'nativeUserShortcutsOnly'|'nativeUserShortcutsCurrentScopeOnly';activationAllowed:false;approvalAvailable?:true;recovery?:ShortcutScopeRecovery;
 configuration:string;resource:string;origin:Record<string,unknown>;destination:Record<string,unknown>;
 desiredBindings:Record<string,{keys:string;enabled:boolean}>;actions:Array<{id:string;description:string;keys:string;enabled:boolean;ported:boolean;plannedStatus:string;typed:string|null;xdgTrigger:string|null}>;unknownActionIds:string[];message:string;
}
export interface ShortcutScopeRecovery{protocol:1;priorEffects:Array<{slot:Record<string,unknown>;original:ShortcutNativeSubject;scopeClosureKnown:false}>;priorScopeClosureClaimed:false;onlyCurrentOwnedRegistrarOperations:true;historicalTargetsUsedAsAddresses:false;portalApplicationBindingsMayBeShared:boolean}
export interface ShortcutReviewDecision extends ShortcutReviewKey{reviewedPlan:boolean;understandNativeEffects:boolean;keepOtherComponentsSuspended:boolean;previewSchemaVersion?:2|3;acknowledgeRetainedUncertainty?:boolean}
export interface ShortcutReviewReceipt{retainedScopeUncertainty?:boolean;schemaVersion:1;effectId:string;registration:'verified'|'unconfirmed'|'notDispatched';publication:'published'|'notPublished'|'unconfirmed';requiresNewReview:true;status:SettingsIoStatus;observed:Record<string,unknown>;cleanup?:{attempted:boolean;acknowledged:boolean;reason:string}|null;message:string}
export interface ShortcutReviewStatus{retainedScopeEffects?:Record<string,unknown>[]|null;schemaVersion:1;effect:Record<string,unknown>|null;effectAvailable:boolean;status:SettingsIoStatus;native:Record<string,unknown>;inFlight:boolean;message:string}
export interface ShortcutReviewActions{prepare():Promise<ShortcutReviewPreview>;prepareCurrentScope?():Promise<ShortcutReviewPreview>;inspect(key:ShortcutReviewKey):Promise<ShortcutReviewPreview>;cancel(key:ShortcutReviewKey):Promise<void>;apply?(decision:ShortcutReviewDecision):Promise<ShortcutReviewReceipt>;status?():Promise<ShortcutReviewStatus>}
const invalid=()=>Error('A revisão nativa dos atalhos não foi confirmada.');
function obj(v:unknown):Record<string,unknown>{if(!v||typeof v!=='object'||Array.isArray(v))throw invalid();return v as Record<string,unknown>;}
function str(v:unknown,max=1024):string{if(typeof v!=='string'||v.length>max)throw invalid();return v;}
function hex(v:unknown,n=64):string{const s=str(v,n);if(s.length!==n||!/^[0-9a-f]+$/.test(s))throw invalid();return s;}
function bool(v:unknown):boolean{if(typeof v!=='boolean')throw invalid();return v;}
function list(v:unknown,max=64):unknown[]{if(!Array.isArray(v)||v.length>max)throw invalid();return v;}
function nullable(v:unknown,max=1024):string|null{return v===null?null:str(v,max);}
export function parseShortcutReviewPreview(value:unknown):ShortcutReviewPreview{
 const wire=JSON.stringify(value);if(typeof wire!=='string'||wire.length>128*1024)throw invalid();
 const v=obj(value);const recoverySchema=v.schemaVersion===3&&v.scope==='nativeUserShortcutsCurrentScopeOnly';const approvedSchema=(v.schemaVersion===2&&v.scope==='nativeUserShortcutsOnly'||recoverySchema)&&v.approvalAvailable===true;
 if(!(approvedSchema||v.schemaVersion===1&&v.scope==='nativeUserShortcutsPreviewOnly'&&v.approvalAvailable===undefined)||v.activationAllowed!==false||!Number.isSafeInteger(v.expiresAtMs)||Number(v.expiresAtMs)<=0)throw invalid();
 const origin=obj(v.origin);hex(origin.profile);if(origin.settingsSha256!==null)hex(origin.settingsSha256);if(origin.journalRevision!=='missing')hex(origin.journalRevision);
 if(origin.dataReview!==null){const ref=obj(origin.dataReview);hex(ref.id,32);hex(ref.sha256);}
 if(origin.slot!==null&&obj(origin.slot).component!=='shortcuts')throw invalid();
 const desiredBindings:ShortcutReviewPreview['desiredBindings']=Object.create(null);const raw=obj(v.desiredBindings);if(Object.keys(raw).length>64)throw invalid();
 for(const [id,value]of Object.entries(raw)){if(id.length>128)throw invalid();const b=obj(value);desiredBindings[id]={keys:str(b.keys,256),enabled:bool(b.enabled)};}
 const known=new Set(SHORTCUTS.map(d=>d.id as string));
 const actions=list(v.actions,SHORTCUTS.length).map(value=>{const a=obj(value),id=str(a.id,128),status=str(a.plannedStatus,32);if(!known.has(id)||!['pending','off','notPorted','invalid','duplicate','typesCharacter'].includes(status))throw invalid();const def=SHORTCUTS.find(d=>d.id===id)!;const binding=Object.hasOwn(desiredBindings,id)?desiredBindings[id]:{keys:def.defaultKeys,enabled:def.enabledByDefault};if(a.keys!==binding.keys||a.enabled!==binding.enabled||a.ported!==def.ported)throw invalid();return {id,description:str(a.description),keys:str(a.keys,256),enabled:bool(a.enabled),ported:bool(a.ported),plannedStatus:status,typed:nullable(a.typed,64),xdgTrigger:nullable(a.xdgTrigger,256)};});
 if(actions.length!==SHORTCUTS.length||new Set(actions.map(a=>a.id)).size!==SHORTCUTS.length)throw invalid();
 const unknownActionIds=list(v.unknownActionIds).map(value=>str(value,128));if(new Set(unknownActionIds).size!==unknownActionIds.length||unknownActionIds.some(id=>known.has(id)||!Object.hasOwn(desiredBindings,id))||Object.keys(desiredBindings).filter(id=>!known.has(id)).some(id=>!unknownActionIds.includes(id)))throw invalid();
 const destination=obj(v.destination);hex(destination.instance,32);parseShortcutNativeEnvironment(destination.environment);
 if(destination.owner!==undefined)parseShortcutNativeProcessIdentity(destination.owner);
 if(recoverySchema&&destination.owner===undefined)throw invalid();
 if(destination.kind==='portal'){
  const r=obj(destination.reservation);if(r.busId!==undefined)hex(r.busId,32);if(destination.owner!==undefined&&r.busId===undefined)throw invalid();const generation=str(r.generation,20);if(!/^[1-9][0-9]*$/.test(generation)||BigInt(generation)>18446744073709551615n)throw invalid();
  for(const key of ['peer','client'])if(!/^:[0-9]+\.[0-9]+$/.test(str(r[key],128)))throw invalid();
  const client=str(r.client,128).slice(1).replaceAll('.','_');
  for(const [key,kind]of [['session','session'],['createRequest','request'],['bindRequest','request']]){const path=str(r[key],512);if(!/^\/[A-Za-z0-9_/]+$/.test(path)||!path.startsWith(`/org/freedesktop/portal/desktop/${kind}/${client}/`))throw invalid();}
  if(r.createRequest===r.bindRequest)throw invalid();
  const wanted=list(r.shortcuts,32),ids=new Set<string>();
  for(const item of wanted){const w=obj(item),id=str(w.id,128);if(ids.has(id))throw invalid();ids.add(id);const action=actions.find(a=>a.id===id);if(!action||action.plannedStatus!=='pending'||!action.ported||!action.enabled||nullable(w.trigger,256)!==action.xdgTrigger)throw invalid();str(w.description);}
  if(actions.some(a=>a.plannedStatus==='pending'&&!ids.has(a.id)))throw invalid();
  if(approvedSchema){const previous=obj(r.previous);if(!['idle','bound'].includes(str(previous.phase,16)))throw invalid();if(previous.phase==='idle'){if(previous.generation!==null||previous.session!==null||previous.request!==null||list(previous.shortcuts,32).length)throw invalid();}else{const gen=str(previous.generation,20);if(!/^[1-9][0-9]*$/.test(gen)||BigInt(gen)>=BigInt(generation)||previous.request!==null||!/^\/[A-Za-z0-9_/]+$/.test(str(previous.session,512)))throw invalid();list(previous.shortcuts,32);if(previous.accepted!==null)list(previous.accepted,32);}}
 }else if(destination.kind!=='plugin')throw invalid();
 if(approvedSchema&&destination.kind==='plugin'){const previous=list(destination.previous,32);const ids=new Set<number>();for(const row of previous){const p=obj(row);if(!Number.isSafeInteger(p.id)||Number(p.id)<0||Number(p.id)>0xffffffff||ids.has(Number(p.id))||!known.has(str(p.action,128)))throw invalid();ids.add(Number(p.id));str(p.keys,256);bool(p.published);}}
 let recovery:ShortcutScopeRecovery|undefined;
 if(recoverySchema){
  const r=obj(v.recovery);if(r.protocol!==1||r.priorScopeClosureClaimed!==false||r.onlyCurrentOwnedRegistrarOperations!==true||r.historicalTargetsUsedAsAddresses!==false||r.portalApplicationBindingsMayBeShared!==(destination.kind==='portal'))throw invalid();
  const priorEffects=list(r.priorEffects,64).map(value=>{const row=obj(value);if(row.scopeClosureKnown!==false)throw invalid();return {slot:parseShortcutEffectSlot(row.slot),original:parseShortcutNativeSubject(row.original),scopeClosureKnown:false as const};});
  if(!priorEffects.length||new Set(priorEffects.map(p=>p.slot.id)).size!==priorEffects.length||JSON.stringify(parseShortcutEffectSlot(origin.slot))!==JSON.stringify(priorEffects[0].slot))throw invalid();
  recovery={protocol:1,priorEffects,priorScopeClosureClaimed:false,onlyCurrentOwnedRegistrarOperations:true,historicalTargetsUsedAsAddresses:false,portalApplicationBindingsMayBeShared:r.portalApplicationBindingsMayBeShared as boolean};
 }else if(v.recovery!==undefined)throw invalid();
 return {schemaVersion:v.schemaVersion as 1|2|3,token:hex(v.token,32),fingerprint:hex(v.fingerprint),expiresAtMs:Number(v.expiresAtMs),scope:v.scope as ShortcutReviewPreview['scope'],activationAllowed:false,...(approvedSchema?{approvalAvailable:true as const}:{}),...(recovery?{recovery}:{}),configuration:hex(v.configuration),resource:hex(v.resource),origin,destination,desiredBindings,actions,unknownActionIds,message:str(v.message,4096)};
}
export function parseShortcutReviewReceipt(value:unknown):ShortcutReviewReceipt{
 const v=obj(value);if(v.schemaVersion!==1||v.requiresNewReview!==true||!['verified','unconfirmed','notDispatched'].includes(str(v.registration,32))||!['published','notPublished','unconfirmed'].includes(str(v.publication,32))||v.publication==='published'&&v.registration!=='verified')throw invalid();
 const observed=obj(v.observed);bool(observed.verified);bool(observed.nativeEffectsStarted);str(observed.reason,4096);if(observed.session!==null)str(observed.session,512);if(observed.shortcuts!==null)list(observed.shortcuts,32);
 if(v.registration==='verified'&&observed.verified!==true||v.registration==='notDispatched'&&observed.nativeEffectsStarted!==false)throw invalid();
 let cleanup:ShortcutReviewReceipt['cleanup'];
 if(v.cleanup!==undefined){if(v.cleanup===null)cleanup=null;else{const c=obj(v.cleanup);cleanup={attempted:bool(c.attempted),acknowledged:bool(c.acknowledged),reason:str(c.reason,1024)};}}
 const status=settingsIoStatus(v.status);if(status.newEffectsAllowed&&!status.suspendedOperations?.includes('shortcuts'))throw invalid();
 return {...(v.retainedScopeUncertainty===undefined?{}:{retainedScopeUncertainty:bool(v.retainedScopeUncertainty)}),schemaVersion:1,effectId:hex(v.effectId,32),registration:v.registration as ShortcutReviewReceipt['registration'],publication:v.publication as ShortcutReviewReceipt['publication'],requiresNewReview:true,status,observed,...(v.cleanup===undefined?{}:{cleanup}),message:str(v.message,4096)};
}
export function parseShortcutReviewStatus(value:unknown):ShortcutReviewStatus{
 const v=obj(value);if(v.schemaVersion!==1)throw invalid();const effect=v.effect===null?null:obj(v.effect);if(effect&&effect.component!=='shortcuts')throw invalid();const available=bool(v.effectAvailable);if(!available&&effect!==null)throw invalid();
 return {...(v.retainedScopeEffects===undefined?{}:{retainedScopeEffects:v.retainedScopeEffects===null?null:list(v.retainedScopeEffects,64).map(parseShortcutEffectSlot)}),schemaVersion:1,effect,effectAvailable:available,status:settingsIoStatus(v.status),native:obj(v.native),inFlight:bool(v.inFlight),message:str(v.message,4096)};
}
