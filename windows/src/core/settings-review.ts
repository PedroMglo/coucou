import {settingsIoStatus,type SettingsIoStatus} from './settings-io';
export const suspendedOperations=['voice','mobile','autostart','shortcuts','hooks','credentials'] as const;
export interface SettingsReviewPreview {
 schemaVersion:1;token:string;fingerprint:string;expiresAtMs:number;
 origin:{selectedPath:string;settingsSha256:string|null;ioSha256:string|null};observedPhase:string;
 values:Record<string,unknown>;runningValues:Record<string,unknown>;builtInDefaults:Record<string,unknown>;
 defaultedFields:string[];unknownFields:string[];unknownFieldCount:number;priorRecoveryProvenanceUncertain:boolean;
 petsAndDestinations:{revision:string;values:Record<string,unknown>};credentialReferences:string[];
 suspendedOperations:string[];scope:'dataOnlyKeepComponentsSuspended';willPreserveCurrentBytes:boolean;priorOutcomeWillBeRetained:true;message:string;
}
export interface SettingsReviewDecision{token:string;fingerprint:string;reviewedAllValues:boolean;understandPriorUncertainty:boolean;keepComponentsSuspended:boolean;acceptRecoveredValues:boolean}
export interface SettingsReviewReceipt{schemaVersion:1;dataAdopted:true;retainedReview:string;status:SettingsIoStatus;message:string}
const error=()=>Error('A revisão das preferências não foi confirmada.');
function object(v:unknown):Record<string,unknown>{if(!v||typeof v!=='object'||Array.isArray(v))throw error();return v as Record<string,unknown>;}
function boundedString(v:unknown,limit:number):string{if(typeof v!=='string'||v.length>limit)throw error();return v;}
function hex(v:unknown,n:number):string{const s=boundedString(v,n);if(s.length!==n||!/^[a-f0-9]+$/.test(s))throw error();return s;}
function list(v:unknown,max=256):string[]{if(!Array.isArray(v)||v.length>max||v.some(s=>typeof s!=='string'||s.length>256))throw error();return [...v] as string[];}
export function parseSettingsReviewPreview(value:unknown):SettingsReviewPreview{
 const v=object(value);if(v.schemaVersion!==1||v.scope!=='dataOnlyKeepComponentsSuspended'||v.priorOutcomeWillBeRetained!==true)throw error();
 const o=object(v.origin),p=object(v.petsAndDestinations);const operations=list(v.suspendedOperations,6);
 if(operations.length!==6||new Set(operations).size!==6||operations.some(s=>!suspendedOperations.includes(s as typeof suspendedOperations[number])))throw error();
 for(const name of ['priorRecoveryProvenanceUncertain','willPreserveCurrentBytes'])if(typeof v[name]!=='boolean')throw error();
 if(!Number.isSafeInteger(v.expiresAtMs)||Number(v.expiresAtMs)<=0||!Number.isSafeInteger(v.unknownFieldCount)||Number(v.unknownFieldCount)<0)throw error();
 return {schemaVersion:1,token:hex(v.token,32),fingerprint:hex(v.fingerprint,64),expiresAtMs:Number(v.expiresAtMs),origin:{selectedPath:boundedString(o.selectedPath,4096),settingsSha256:o.settingsSha256===null?null:hex(o.settingsSha256,64),ioSha256:o.ioSha256===null?null:hex(o.ioSha256,64)},observedPhase:boundedString(v.observedPhase,64),values:object(v.values),runningValues:object(v.runningValues),builtInDefaults:object(v.builtInDefaults),defaultedFields:list(v.defaultedFields),unknownFields:list(v.unknownFields,128),unknownFieldCount:Number(v.unknownFieldCount),priorRecoveryProvenanceUncertain:v.priorRecoveryProvenanceUncertain as boolean,petsAndDestinations:{revision:boundedString(p.revision,256),values:object(p.values)},credentialReferences:list(v.credentialReferences,64),suspendedOperations:operations,scope:'dataOnlyKeepComponentsSuspended',willPreserveCurrentBytes:v.willPreserveCurrentBytes as boolean,priorOutcomeWillBeRetained:true,message:boundedString(v.message,4096)};
}
export function parseSettingsReviewReceipt(value:unknown):SettingsReviewReceipt{const v=object(value);if(v.schemaVersion!==1||v.dataAdopted!==true)throw error();return {schemaVersion:1,dataAdopted:true,retainedReview:hex(v.retainedReview,32),status:settingsIoStatus(v.status),message:boundedString(v.message,2048)};}
export interface SettingsReviewActions{prepare:()=>Promise<SettingsReviewPreview>;apply:(request:SettingsReviewDecision)=>Promise<SettingsReviewReceipt>;cancel:(request:{token:string;fingerprint:string})=>Promise<void>}
