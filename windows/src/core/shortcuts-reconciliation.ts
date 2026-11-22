import {parseShortcutNativeSubject,type ShortcutNativeSubject} from './shortcut-native-subject';
import {settingsIoStatus,type SettingsIoStatus} from './settings-io';
import {parseShortcutNativeProcessIdentity,parseShortcutNativeEnvironment,type ShortcutReviewKey} from './shortcuts-review';
export type {ShortcutNativeSubject} from './shortcut-native-subject';
export interface ShortcutReconciliationPreview extends ShortcutReviewKey{schemaVersion:2;expiresAtMs:number;scope:'nativeUserShortcutsReconciliationOnly';origin:Record<string,unknown>;subject:Record<string,unknown>;original:ShortcutNativeSubject;current:ShortcutNativeSubject;observation:Record<string,unknown>&{canReconcile:boolean;reason:string};noRegistrationEffect:true;noReactivation:true;pastEffectOutcomeKnown:false;message:string}
export interface ShortcutReconciliationDecision extends ShortcutReviewKey{previewSchemaVersion:2;reviewedObservation:boolean;understandPastUncertainty:boolean;keepRegistrationSuspended:boolean}
export interface ShortcutReconciliationReceipt{schemaVersion:1;effectId:string;observationRecorded:true;noRegistrationEffect:true;noReactivation:true;pastEffectOutcomeKnown:false;status:SettingsIoStatus;message:string}
export interface ShortcutReconciliationActions{prepare():Promise<ShortcutReconciliationPreview>;inspect(key:ShortcutReviewKey):Promise<ShortcutReconciliationPreview>;cancel(key:ShortcutReviewKey):Promise<void>;apply(decision:ShortcutReconciliationDecision):Promise<ShortcutReconciliationReceipt>}
function invalid():never{throw Error('A observação dos atalhos não foi confirmada.');}
function obj(v:unknown):Record<string,unknown>{if(!v||typeof v!=='object'||Array.isArray(v))invalid();return v as Record<string,unknown>;}
function str(v:unknown,max=4096):string{if(typeof v!=='string'||v.length>max)invalid();return v;}
function hex(v:unknown,n=64):string{const s=str(v,n);if(s.length!==n||!/^[0-9a-f]+$/.test(s))invalid();return s;}
function bool(v:unknown):boolean{if(typeof v!=='boolean')invalid();return v;}
function bounded(v:unknown){const wire=JSON.stringify(v);if(!wire||wire.length>128*1024)invalid();}
export function parseShortcutReconciliationPreview(value:unknown):ShortcutReconciliationPreview{
 bounded(value);const v=obj(value);if(v.schemaVersion!==2||v.scope!=='nativeUserShortcutsReconciliationOnly'||v.noRegistrationEffect!==true||v.noReactivation!==true||v.pastEffectOutcomeKnown!==false||!Number.isSafeInteger(v.expiresAtMs)||Number(v.expiresAtMs)<=0)invalid();
 const origin=obj(v.origin);hex(origin.profile);if(origin.settingsSha256!==null)hex(origin.settingsSha256);if(origin.journalRevision!=='missing')hex(origin.journalRevision);
 const subject=obj(v.subject);if(subject.component!=='shortcuts')invalid();hex(subject.id,32);if(!['prepared','dispatched','verified','unconfirmed','notDispatched','reconciledObservation'].includes(str(subject.phase,32)))invalid();const binding=obj(subject.binding);for(const k of ['profile','configuration','resource','plan'])hex(binding[k]);
 const original=parseShortcutNativeSubject(v.original),current=parseShortcutNativeSubject(v.current);
 const o=obj(v.observation);if(o.schemaVersion!==1)invalid();hex(o.currentInstance,32);if(o.originalInstance!==null)hex(o.originalInstance,32);const presence=o.originalProcess;if(presence!==null&&!['originalLive','originalAbsent','identityMismatch','unavailable'].includes(str(presence,32)))invalid();bool(o.sameInstance);const canReconcile=bool(o.canReconcile);if(canReconcile&&(presence===null||presence==='identityMismatch'||presence==='unavailable'))invalid();
 const currentProcess=parseShortcutNativeProcessIdentity(o.currentProcess),currentEnvironment=parseShortcutNativeEnvironment(o.currentEnvironment);if(o.currentTransport!=='plugin'&&o.currentTransport!=='portal')invalid();
 if(current.owner===null||current.instance!==o.currentInstance||current.kind!==o.currentTransport||current.kind==='portal'&&current.portal===null||JSON.stringify(current.owner)!==JSON.stringify(currentProcess)||JSON.stringify(current.environment)!==JSON.stringify(currentEnvironment))invalid();
 if(o.originalInstance!==original.instance||o.sameInstance!==(o.currentInstance===original.instance)||original.owner===null&&(presence!==null||canReconcile)||original.owner!==null&&presence===null)invalid();
 if(canReconcile&&(JSON.stringify(original.environment)!==JSON.stringify(currentEnvironment)||subject.phase!=='prepared'&&(original.kind!==o.currentTransport||original.kind==='portal'&&original.portal===null)))invalid();
 if(o.portal!==null){if(original.kind!=='portal'||o.currentTransport!=='portal')invalid();const p=obj(o.portal);bool(p.busMatches);bool(p.sessionsClosed);for(const key of ['originalClientLive','originalPeerLive'])if(p[key]!==null)bool(p[key]);str(p.reason,1024);if(canReconcile&&(!p.busMatches||!p.sessionsClosed)&&subject.phase!=='prepared')invalid();}
 if(o.pluginCleanupAcknowledged!==null)bool(o.pluginCleanupAcknowledged);
 return {schemaVersion:2,token:hex(v.token,32),fingerprint:hex(v.fingerprint),expiresAtMs:Number(v.expiresAtMs),scope:'nativeUserShortcutsReconciliationOnly',origin,subject,original,current,observation:{...o,currentProcess,currentEnvironment,currentTransport:o.currentTransport,canReconcile,reason:str(o.reason)},noRegistrationEffect:true,noReactivation:true,pastEffectOutcomeKnown:false,message:str(v.message)};
}
export function parseShortcutReconciliationReceipt(value:unknown):ShortcutReconciliationReceipt{
 bounded(value);const v=obj(value);if(v.schemaVersion!==1||v.observationRecorded!==true||v.noRegistrationEffect!==true||v.noReactivation!==true||v.pastEffectOutcomeKnown!==false)invalid();const status=settingsIoStatus(v.status);if(status.newEffectsAllowed&&!status.suspendedOperations?.includes('shortcuts'))invalid();return {schemaVersion:1,effectId:hex(v.effectId,32),observationRecorded:true,noRegistrationEffect:true,noReactivation:true,pastEffectOutcomeKnown:false,status,message:str(v.message)};
}
