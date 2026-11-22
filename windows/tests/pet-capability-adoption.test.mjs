import test from 'node:test';
import assert from 'node:assert/strict';
import {validPetCapabilities,capabilityDescriptorFor} from '../src/core/pet-capabilities.ts';
import {petObservation} from '../src/core/pet-observability.ts';
import {PetObservabilityResource} from '../src/core/pet-observability-resource.ts';

const descriptor=(petId='agent_codex',provider='codex',kind='llm')=>({schemaVersion:1,petId,kind,name:petId,lifecycle:'active',
 binding:{adapter:'codex',provider,model:null,profile:'auto'},revision:'synthetic:1',observedAt:null,configurationAvailable:true,
 concurrency:{scope:'provider',maxActive:1},capabilities:[{id:'voice.respond',support:'supported',availability:'unknown',reason:'targetRequired',
 source:['bundledManifest','configuration'],observedAt:null,expiresAt:null,modelBinding:null,decision:'nativeApproval',callers:['nativeUser'],
 limits:{maxItems:null,maxBytes:null,maxCalls:null,timeoutMs:null},limitations:[]}]});
const capabilities=(pets=[descriptor()])=>({schemaVersion:1,revision:'synthetic:1',observedAt:1_700_000_000_000,configurationAvailable:true,pets});
const count=value=>({value,observed:value??0});
const inventory=total=>({scope:'mochiLibrary',coverage:'complete',observedAt:1_700_000_000,stale:false,refreshing:false,reason:null,
 counts:{total:count(total),archived:count(0),confirmedActive:count(0),working:count(0),waiting:count(0),reserved:count(0),idle:count(total),unknown:0},sessions:[],listed:0,limit:12,truncated:false});
const observation=()=>({schemaVersion:1,generatedAt:1_700_000_001,pets:[{petId:'agent_codex',provider:'codex',enabled:true,mochi:inventory(3),usage:null}],
 providers:[{provider:'codex',petIds:['agent_codex'],native:{...inventory(20),scope:'codexDaemon'},usage:null}],petCapabilities:capabilities()});
const deferred=()=>{let resolve;const promise=new Promise(r=>resolve=r);return{promise,resolve};};
const tick=()=>new Promise(r=>setTimeout(r,0));

test('bounded metadata preserves milliseconds and unknown support without inventing account readiness',()=>{
 const data=capabilities();assert.equal(validPetCapabilities(data),true);
 const d=capabilityDescriptorFor(data,'agent_codex','codex');assert.equal(d.capabilities[0].availability,'unknown');
 assert.equal(data.observedAt,1_700_000_000_000);assert.equal(d.binding.model,null);
});

test('duplicate pets/capabilities, unknown versions and malformed enums or dates are refused',()=>{
 for(const alter of [d=>d.pets.push(structuredClone(d.pets[0])),d=>d.pets[0].capabilities.push(structuredClone(d.pets[0].capabilities[0])),
  d=>d.schemaVersion=2,d=>d.pets[0].schemaVersion=2,d=>d.pets[0].kind='administrator',d=>d.pets[0].capabilities[0].support='always',
  d=>d.pets[0].capabilities[0].reason='allowAllSystem',d=>d.pets[0].capabilities[0].source=['secretEndpoint'],d=>d.observedAt=Infinity,d=>d.pets[0].capabilities[0].expiresAt=-1]){
  const data=capabilities();alter(data);assert.equal(validPetCapabilities(data),false);
 }
});

test('metadata size limits and required shapes are enforced before management projection',()=>{
 for(const alter of [d=>d.pets=Array.from({length:65},(_,n)=>descriptor(`pet_${n}`)),
  d=>d.pets[0].capabilities=Array.from({length:129},(_,n)=>({...d.pets[0].capabilities[0],id:`cap${n}`})),
  d=>d.revision='x'.repeat(257),d=>delete d.pets[0].binding,d=>d.pets[0].concurrency.scope='entire-system',
  d=>d.pets[0].capabilities[0].limits.maxCalls=Number.MAX_SAFE_INTEGER+1]){
  const data=capabilities();alter(data);assert.equal(validPetCapabilities(data),false);
 }
});

test('services sharing Codex keep their own logical owner and cannot borrow another descriptor',()=>{
 const notion=descriptor('integration_notion','integration_notion','service'),stripe=descriptor('integration_stripe','integration_stripe','service');
 const data=capabilities([descriptor(),notion,stripe]);
 assert.equal(capabilityDescriptorFor(data,'integration_notion','codex').petId,'integration_notion');
 assert.equal(capabilityDescriptorFor(data,'integration_stripe','codex').binding.provider,'integration_stripe');
 notion.binding.provider='codex';assert.equal(capabilityDescriptorFor(data,'integration_notion','codex'),null);
 assert.equal(capabilityDescriptorFor(data,'missing','codex'),null);
});

test('missing configuration remains explicit instead of falling back to a configured provider',()=>{
 const d=descriptor();d.configurationAvailable=false;d.binding={adapter:null,provider:null,model:null,profile:null};
 const data=capabilities([d]);data.configurationAvailable=false;
 assert.equal(validPetCapabilities(data),true);assert.equal(capabilityDescriptorFor(data,'agent_codex','codex').binding.provider,null);
});

test('management adopts exact identity without changing inventories or merging their scopes',()=>{
 const data=observation(),before=petObservation({...data,petCapabilities:undefined},'agent_codex');
 const after=petObservation(data,'agent_codex');assert.equal(after.capabilityDescriptor.petId,'agent_codex');
 for(const field of ['active','total','archived','working','waiting','library','tokens'])assert.deepEqual(after[field],before[field]);
 data.petCapabilities=capabilities([descriptor('agent_copilot','copilot')]);assert.equal(petObservation(data,'agent_codex').capabilityDescriptor,null);
});

test('failed refresh retains original measurement times but removes unconfirmed capabilities',async()=>{
 let fail=false;const resource=new PetObservabilityResource(async()=>{if(fail)throw Error('offline');return observation();});
 await resource.refresh();const time=resource.snapshot.pets[0].mochi.observedAt;
 fail=true;await resource.refresh(true);assert.ok(resource.error);assert.equal(resource.snapshot.petCapabilities,null);
 assert.equal(resource.snapshot.pets[0].mochi.observedAt,time);assert.equal(resource.snapshot.pets[0].mochi.counts.total.value,3);
});

test('invalidation clears shared capability metadata immediately for current observers',async()=>{
 const resource=new PetObservabilityResource(async()=>observation());await resource.refresh();let notified=false;
 resource.subscribe(()=>{notified=true;assert.equal(resource.snapshot.petCapabilities,null);});
 resource.invalidate();assert.equal(resource.snapshot.petCapabilities,null);assert.equal(notified,true);
});

test('late reads after a configuration event cannot restore old capability metadata',async()=>{
 const reads=[deferred(),deferred()];let n=0;const resource=new PetObservabilityResource(()=>reads[n++].promise);
 const initial=resource.refresh();await tick();resource.invalidate();reads[0].resolve(observation());await initial;
 assert.equal(resource.snapshot.petCapabilities,null);await tick();assert.equal(n,2);
 const fresh=observation();fresh.petCapabilities.revision='synthetic:2';reads[1].resolve(fresh);await tick();
 assert.equal(resource.snapshot.petCapabilities.revision,'synthetic:2');
});

test('an incompatible capability envelope cannot replace valid measurement data',async()=>{
 let bad=false;const resource=new PetObservabilityResource(async()=>{const data=observation();if(bad){data.pets[0].mochi.counts.total.value=999;data.petCapabilities.schemaVersion=2;}return data;});
 await resource.refresh();bad=true;await resource.refresh(true);assert.ok(resource.error);
 assert.equal(resource.snapshot.petCapabilities,null);assert.equal(resource.snapshot.pets[0].mochi.counts.total.value,3);
});
