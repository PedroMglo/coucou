import {test} from 'node:test';import assert from 'node:assert/strict';
import {petObservation,observationCount} from '../src/core/pet-observability.ts';
const count=(value,observed=value??0)=>({value,observed});
const local={scope:'mochiLibrary',coverage:'complete',observedAt:1700000001,stale:false,refreshing:false,reason:null,counts:{total:count(6),archived:count(2),confirmedActive:count(1),working:count(1),waiting:count(0),reserved:count(0),idle:count(0),unknown:0},sessions:[],listed:0,limit:12,truncated:false};
const empty=()=>({...local,scope:'copilotLiveEndpoints',coverage:'partial',observedAt:1700000002,counts:Object.fromEntries(Object.keys(local.counts).map(k=>[k,k==='unknown'?0:count(null,0)]))});
const data=native=>({schemaVersion:1,generatedAt:1700000010,pets:[{petId:'agent_copilot',provider:'copilot',enabled:true,mochi:local,usage:null}],providers:[{provider:'copilot',petIds:['agent_copilot'],native,usage:null}]});
test('Copilot empty partial/null/observed0 inventory cannot conceal known library counters',()=>{
 const v=petObservation(data(empty()),'agent_copilot');assert.equal(v.primaryScope,'pet');assert.equal(v.total.value,6);assert.equal(v.active.value,1);assert.equal(v.archived.value,2);assert.equal(v.updatedAt,local.observedAt);assert.match(v.scopeLabel,/Biblioteca deste pet/);assert.match(v.externalStatus,/não há contagens externas confirmadas/);
});
test('positive external subtotal keeps the provider scope and visible lower bound instead of local totals',()=>{
 for(const key of ['total','archived','confirmedActive','working','waiting']){const n=empty();n.counts[key]=count(null,4);const v=petObservation(data(n),'agent_copilot');assert.equal(v.primaryScope,'provider');assert.equal(v.library.total.value,6);assert.equal(v.updatedAt,n.observedAt);if(key==='total')assert.equal(observationCount(v.total),'≥ 4');if(key==='confirmedActive')assert.equal(observationCount(v.active),'≥ 4');}
});
test('complete confirmed native zero remains provider zero rather than falling back to six local conversations',()=>{
 const n=empty();n.coverage='complete';for(const key of ['total','archived','confirmedActive','working','waiting'])n.counts[key]=count(0);const v=petObservation(data(n),'agent_copilot');assert.equal(v.primaryScope,'provider');assert.equal(v.total.value,0);assert.equal(observationCount(v.total),'0');assert.equal(v.library.total.value,6);
});
test('switching empty-partial to positive-partial and back changes all metric scopes atomically',()=>{
 const d=data(empty());assert.equal(petObservation(d,'agent_copilot').primaryScope,'pet');d.providers[0].native.counts.total=count(null,12);assert.equal(petObservation(d,'agent_copilot').primaryScope,'provider');assert.equal(observationCount(petObservation(d,'agent_copilot').total),'≥ 12');d.providers[0].native=empty();assert.equal(petObservation(d,'agent_copilot').total.value,6);assert.equal(petObservation(d,'agent_copilot').archived.value,2);
});
