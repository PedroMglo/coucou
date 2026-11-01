import {settingsIoStatus,type SettingsIoStatus} from './settings-io';
export type AutostartAction='setRegistration'|'setIndependentRegistration'|'reconcileObservation'|'reconcileArchivedObservation'|'reconcileLegacyObservation';
export type AutostartRequest={action:'setRegistration'|'setIndependentRegistration';enabled:boolean}|{action:'reconcileObservation'}|{action:'reconcileArchivedObservation';archivedEffectId:string}|{action:'reconcileLegacyObservation';archivedEffectId:string};
export type EffectPhase='prepared'|'dispatched'|'verified'|'unconfirmed'|'notDispatched'|'reconciledObservation';
export interface ComponentEffect{id:string;component:'autostart';phase:EffectPhase;binding:{profile:string;configuration:string;resource:string;plan:string};owner:string;intent:{id:string;sha256:string};outcome?:{id:string;sha256:string}}
export interface AutostartPlan{schemaVersion:1|2;component:'autostart';enabled:boolean;target:string;resource:string;launcher:string;launcherIdentity:string;beforeRevision:string;beforePresent:boolean;createsParent:boolean;expectedSha256:string|null;diff:string;postcondition:'desktopEntryMatches'|'desktopEntryAbsent';fingerprint:string}
export interface AutostartPreview{schemaVersion:1;token:string;fingerprint:string;expiresAtMs:number;action:AutostartAction;desiredEnabled:boolean;previousDesiredEnabled:boolean;profileScope:string;origin:{profile:string|null;settingsSha256:string|null;journalRevision:string;dataReview:{id:string;sha256:string}|null;slot:ComponentEffect|null};plan:AutostartPlan;archivedEffect?:ComponentEffect;impact:string;scope:'nativeUserAutostartOnly';message:string}
export interface AutostartDecision{token:string;fingerprint:string;reviewedPlan:boolean;understandEffect:boolean;keepOtherComponentsSuspended:boolean;acknowledgeExternalUncertainty?:boolean;acknowledgeLegacyUncertainty?:boolean}
export interface AutostartReceipt{schemaVersion:1;desiredEnabled:boolean;registration:'verified'|'unconfirmed'|'notDispatched'|'reconciledObservation';loginVerified:false;reactivated:boolean;effectId:string;status:SettingsIoStatus;message:string}
export interface HistoricalObservation{schemaVersion:1|2;legacyOriginUnverifiable?:true;legacyUncertaintyAcknowledged?:true;subject:ComponentEffect;profile:string;dataReview:{id:string;sha256:string}|null;nativePlan:AutostartPlan;registrationMatchesExpected:boolean;noRegistrationEffect:true;noReactivation:true;loginVerified:false;pastEffectOutcomeKnown:false}
export interface HistoricalProjection{effectId:string;reference:{id:string;sha256:string};observation:HistoricalObservation;currentForProfile:boolean}
export interface AutostartStatus{schemaVersion:1;requiresReview:true;desiredEnabled:boolean;status:SettingsIoStatus;effect:ComponentEffect|null;registrationMatches:boolean|null;independentDestination:boolean;unresolvedHistory:ComponentEffect[];observedHistory:HistoricalProjection[];historicalReconcilable:string[];legacyReconcilable:string[];inFlight:boolean;message:string}
export interface AutostartReviewActions{status():Promise<AutostartStatus>;prepare(request:AutostartRequest):Promise<AutostartPreview>;apply(request:AutostartDecision):Promise<AutostartReceipt>;cancel(request:{token:string;fingerprint:string}):Promise<void>}
const fail=()=>Error('O plano ou resultado de autostart não foi confirmado.');
function object(v:unknown):Record<string,unknown>{if(!v||typeof v!=='object'||Array.isArray(v))throw fail();return v as Record<string,unknown>;}
function str(v:unknown,max=4096):string{if(typeof v!=='string'||v.length>max)throw fail();return v;}
function bool(v:unknown):boolean{if(typeof v!=='boolean')throw fail();return v;}
function hex(v:unknown,n=64):string{const s=str(v,n);if(s.length!==n||!/^[a-f0-9]+$/.test(s))throw fail();return s;}
function ref(v:unknown){const s=object(v);return {id:hex(s.id,32),sha256:hex(s.sha256)};}
function effect(v:unknown):ComponentEffect|null{
 if(v===null)return null;const s=object(v),b=object(s.binding);if(s.component!=='autostart'||!['prepared','dispatched','verified','unconfirmed','notDispatched','reconciledObservation'].includes(String(s.phase)))throw fail();
 const outcome=s.outcome===undefined?undefined:ref(s.outcome);if(['prepared','dispatched'].includes(String(s.phase))===(outcome!==undefined))throw fail();
 return {id:hex(s.id,32),component:'autostart',phase:s.phase as EffectPhase,binding:{profile:hex(b.profile),configuration:hex(b.configuration),resource:hex(b.resource),plan:hex(b.plan)},owner:hex(s.owner),intent:ref(s.intent),...(outcome?{outcome}:{})};
}
function unknownEffect(v:unknown):ComponentEffect{const e=effect(v);if(!e||['verified','notDispatched','reconciledObservation'].includes(e.phase))throw fail();return e;}
function nativePlan(value:unknown):AutostartPlan{
 const p=object(value);if((p.schemaVersion!==1&&p.schemaVersion!==2)||p.component!=='autostart')throw fail();const enabled=bool(p.enabled),expected=p.expectedSha256===null?null:hex(p.expectedSha256);
 if(enabled!==(expected!==null)||p.postcondition!==(enabled?'desktopEntryMatches':'desktopEntryAbsent'))throw fail();
 return {schemaVersion:p.schemaVersion as 1|2,component:'autostart',enabled,target:str(p.target),resource:hex(p.resource),launcher:str(p.launcher),launcherIdentity:hex(p.launcherIdentity),beforeRevision:p.beforeRevision==='missing'?'missing':hex(p.beforeRevision),beforePresent:bool(p.beforePresent),createsParent:bool(p.createsParent),expectedSha256:expected,diff:str(p.diff,96*1024),postcondition:enabled?'desktopEntryMatches':'desktopEntryAbsent',fingerprint:hex(p.fingerprint)};
}
export function parseAutostartPreview(value:unknown):AutostartPreview{
 const v=object(value),p=nativePlan(v.plan),o=object(v.origin);if(v.schemaVersion!==1||v.scope!=='nativeUserAutostartOnly'||!['setRegistration','setIndependentRegistration','reconcileObservation','reconcileArchivedObservation','reconcileLegacyObservation'].includes(String(v.action)))throw fail();
 const enabled=bool(v.desiredEnabled);if(p.enabled!==enabled||!Number.isSafeInteger(v.expiresAtMs)||Number(v.expiresAtMs)<=0)throw fail();
 const profileScope=hex(v.profileScope);if(o.profile!==profileScope)throw fail();
 const previous=effect(o.slot);
 if(v.action==='setIndependentRegistration'&&(!previous||['verified','notDispatched','reconciledObservation'].includes(previous.phase)||previous.binding.profile===profileScope||previous.binding.resource===p.resource))throw fail();
 const archived=v.archivedEffect===undefined?undefined:unknownEffect(v.archivedEffect);
 if(v.action==='reconcileArchivedObservation'?(!previous||!archived||archived.binding.resource!==p.resource||archived.id===previous.id):v.action==='reconcileLegacyObservation'?(!previous||!archived||p.schemaVersion!==2):archived!==undefined)throw fail();
 return {schemaVersion:1,token:hex(v.token,32),fingerprint:hex(v.fingerprint),expiresAtMs:Number(v.expiresAtMs),action:v.action as AutostartAction,desiredEnabled:enabled,previousDesiredEnabled:bool(v.previousDesiredEnabled),profileScope,origin:{profile:profileScope,settingsSha256:o.settingsSha256===null?null:hex(o.settingsSha256),journalRevision:o.journalRevision==='missing'?'missing':hex(o.journalRevision),dataReview:o.dataReview===null?null:ref(o.dataReview),slot:previous},plan:p,...(archived?{archivedEffect:archived}:{}),impact:str(v.impact),scope:'nativeUserAutostartOnly',message:str(v.message)};
}
export function parseAutostartReceipt(value:unknown):AutostartReceipt{
 const v=object(value);if(v.schemaVersion!==1||v.loginVerified!==false||!['verified','unconfirmed','notDispatched','reconciledObservation'].includes(String(v.registration)))throw fail();
 const status=settingsIoStatus(v.status),reactivated=bool(v.reactivated);if(reactivated&&(v.registration!=='verified'||!status.newEffectsAllowed||status.suspendedOperations?.includes('autostart')))throw fail();
 return {schemaVersion:1,desiredEnabled:bool(v.desiredEnabled),registration:v.registration as AutostartReceipt['registration'],loginVerified:false,reactivated,effectId:hex(v.effectId,32),status,message:str(v.message)};
}
export function parseAutostartStatus(value:unknown):AutostartStatus{
 const v=object(value);if(v.schemaVersion!==1||v.requiresReview!==true)throw fail();const previous=effect(v.effect),independentDestination=v.independentDestination===undefined?false:bool(v.independentDestination);
 if(independentDestination&&(!previous||['verified','notDispatched','reconciledObservation'].includes(previous.phase)))throw fail();
 function list(value:unknown):unknown[]{if(value===undefined)return [];if(!Array.isArray(value)||value.length>64)throw fail();return value;}
 const unresolvedHistory=list(v.unresolvedHistory).map(unknownEffect);if(new Set(unresolvedHistory.map(e=>e.id)).size!==unresolvedHistory.length)throw fail();
 const historicalReconcilable=list(v.historicalReconcilable).map(id=>hex(id,32));if(new Set(historicalReconcilable).size!==historicalReconcilable.length||historicalReconcilable.some(id=>!unresolvedHistory.some(e=>e.id===id)))throw fail();
 const legacyReconcilable=list(v.legacyReconcilable).map(id=>hex(id,32));if(new Set(legacyReconcilable).size!==legacyReconcilable.length||legacyReconcilable.some(id=>id!==previous?.id&&!unresolvedHistory.some(e=>e.id===id))||legacyReconcilable.some(id=>historicalReconcilable.includes(id)||id===previous?.id&&['verified','notDispatched','reconciledObservation'].includes(previous.phase))||independentDestination&&!!previous&&legacyReconcilable.includes(previous.id))throw fail();
 const observedHistory:HistoricalProjection[]=list(v.observedHistory).map(item=>{const s=object(item),o=object(s.observation),subject=unknownEffect(o.subject),p=nativePlan(o.nativePlan),profile=hex(o.profile);
  const legacy=o.schemaVersion===2&&o.legacyOriginUnverifiable===true&&o.legacyUncertaintyAcknowledged===true;if(!(o.schemaVersion===1&&o.legacyOriginUnverifiable===undefined&&o.legacyUncertaintyAcknowledged===undefined||legacy&&p.schemaVersion===2)||o.noRegistrationEffect!==true||o.noReactivation!==true||o.loginVerified!==false||o.pastEffectOutcomeKnown!==false||!legacy&&subject.binding.resource!==p.resource)throw fail();
  const currentForProfile=bool(s.currentForProfile);if(currentForProfile&&(p.schemaVersion!==2||previous?.binding.profile!==profile))throw fail();
  return {effectId:hex(s.effectId,32),reference:ref(s.reference),currentForProfile,observation:{schemaVersion:o.schemaVersion as 1|2,...(legacy?{legacyOriginUnverifiable:true as const,legacyUncertaintyAcknowledged:true as const}:{}),subject,profile,dataReview:o.dataReview===null?null:ref(o.dataReview),nativePlan:p,registrationMatchesExpected:bool(o.registrationMatchesExpected),noRegistrationEffect:true,noReactivation:true,loginVerified:false,pastEffectOutcomeKnown:false}};
 });if(new Set(observedHistory.map(o=>o.effectId)).size!==observedHistory.length)throw fail();
 return {schemaVersion:1,requiresReview:true,desiredEnabled:bool(v.desiredEnabled),status:settingsIoStatus(v.status),effect:previous,registrationMatches:v.registrationMatches===null?null:bool(v.registrationMatches),independentDestination,unresolvedHistory,observedHistory,historicalReconcilable,legacyReconcilable,inFlight:bool(v.inFlight),message:str(v.message)};
}
