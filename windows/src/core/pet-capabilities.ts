import type {PetCapabilitiesSnapshot,PetCapabilityDescriptor} from './bridge';

const record=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const text=(v:unknown,max:number)=>typeof v==='string'&&v.length>0&&v.length<=max&&!/[\u0000-\u001f\u007f]/.test(v);
const date=(v:unknown)=>v===null||typeof v==='number'&&Number.isSafeInteger(v)&&v>0&&v<=8.64e15;
const number=(v:unknown)=>v===null||typeof v==='number'&&Number.isSafeInteger(v)&&v>=0;
const oneOf=(v:unknown,values:readonly string[])=>typeof v==='string'&&values.includes(v);
const reasons=['notImplemented','unsupportedPlatform','notConfigured','inactivePet','retiredPet','modelUnresolved','modelUnsupported','nativeApprovalUnavailable','nativeControlUnavailable','controlConflict','turnRequired','targetRequired','staleObservation','providerUnavailable','busy','permissionDenied','policyDenied','libraryUnavailable','settingsUnconfirmed','authenticationRequired','ambiguousTarget','budgetUnavailable','unknown'];
const nullableText=(v:unknown,max:number)=>v===null||text(v,max);
const list=(v:unknown,allowed:readonly string[])=>Array.isArray(v)&&v.length<=allowed.length&&new Set(v).size===v.length&&v.every(x=>oneOf(x,allowed));

/** Bounded public metadata. Validating it never grants any execution authority. */
export function validPetCapabilities(value:unknown):value is PetCapabilitiesSnapshot {
 if(!record(value)||value.schemaVersion!==1||!text(value.revision,256)||!date(value.observedAt)||value.observedAt===null||
  typeof value.configurationAvailable!=='boolean'||!Array.isArray(value.pets)||value.pets.length>64)return false;
 const ids=new Set<string>();
 return value.pets.every(p=>{
  if(!record(p)||p.schemaVersion!==1||!text(p.petId,128)||ids.has(p.petId as string)||!text(p.name,160)||!text(p.revision,256)||
   !oneOf(p.kind,['llm','service','observer'])||!oneOf(p.lifecycle,['active','inactive','retired'])||
   typeof p.configurationAvailable!=='boolean'||!date(p.observedAt)||!record(p.binding)||
   !nullableText(p.binding.adapter,128)||!nullableText(p.binding.provider,128)||!nullableText(p.binding.model,256)||!nullableText(p.binding.profile,128)||
   !record(p.concurrency)||!(p.concurrency.scope===null||p.concurrency.scope==='provider')||!number(p.concurrency.maxActive)||
   !Array.isArray(p.capabilities)||p.capabilities.length>128)return false;
  ids.add(p.petId as string);const capabilities=new Set<string>();
  return p.capabilities.every(c=>{
   if(!record(c)||!text(c.id,128)||capabilities.has(c.id as string)||!oneOf(c.support,['supported','unsupported','unknown'])||
    !oneOf(c.availability,['ready','unavailable','unknown'])||!(c.reason===null||oneOf(c.reason,reasons))||!date(c.observedAt)||!date(c.expiresAt)||
    !nullableText(c.modelBinding,256)||!oneOf(c.decision,['none','nativeApproval','exactCommandApproval','scopedGrant'])||
    !list(c.source,['bundledManifest','configuration','providerCatalog','runtimeObservation'])||
    !list(c.callers,['nativeUser','taskRuntime','providerModel'])||!Array.isArray(c.limitations)||c.limitations.length>32||
    !c.limitations.every(x=>text(x,128))||!record(c.limits)||
    !['maxItems','maxBytes','maxCalls','timeoutMs'].every(key=>number((c.limits as Record<string,unknown>)[key])))return false;
   capabilities.add(c.id as string);return true;
  });
 });
}

export function capabilityDescriptorFor(snapshot:unknown,petId:string,provider:string):PetCapabilityDescriptor|null {
 if(!validPetCapabilities(snapshot))return null;
 const descriptor=snapshot.pets.find(p=>p.petId===petId);if(!descriptor)return null;
 const owner=descriptor.kind==='service'?petId:provider;
 if(descriptor.binding.provider!==null&&descriptor.binding.provider!==owner)return null;
 return descriptor;
}
