import {test} from 'node:test';import assert from 'node:assert/strict';
import {petObservation,observationCount} from '../src/core/pet-observability.ts';
const count=(value,observed=value??0)=>({value,observed});
const inventory=(n,scope='mochiLibrary',coverage='complete')=>({scope,coverage,observedAt:1700000001,stale:false,refreshing:false,reason:null,counts:{total:count(n),archived:count(n?2:0),confirmedActive:count(n?4:0),working:count(n?3:0),waiting:count(n?1:0),reserved:count(0),idle:count(20),unknown:0},sessions:[],listed:0,limit:12,truncated:false});
function data(provider,native){return {schemaVersion:1,generatedAt:1700000010,pets:[{petId:'pet',provider,enabled:true,mochi:inventory(6),usage:null}],providers:[{provider,petIds:['pet'],native,usage:null}]};}
test('all LLM providers keep opened sessions unknown under v1 despite in-flight, idle and history counts',()=>{
 for(const provider of ['codex','copilot','ollama','antigravity','custom'])for(const coverage of ['complete','partial','unavailable','notSupported']){const v=petObservation(data(provider,inventory(20,'providerGlobal',coverage)),'pet');assert.equal(observationCount(v.opened),'—');assert.equal(v.opened.coverage,'unavailable');assert.match(v.opened.detail,/pet e no terminal/);}
});
test('native total and exact pet library stay separate, never summed or silently swapped',()=>{
 const v=petObservation(data('codex',inventory(100,'codexDaemon')),'pet');assert.deepEqual(v.history.map(s=>[s.key,s.total.value]),[['external',100],['pet',6]]);assert.equal(v.history.some(s=>s.total.value===106),false);assert.match(v.history[1].detail,/não são somados/);
});
test('unavailable native history cannot become zero or inherit known local totals',()=>{
 const n=inventory(0,'codexDaemon','unavailable');n.reason='refreshFailed';const v=petObservation(data('codex',n),'pet');assert.equal(v.primaryScope,'pet');assert.equal(v.history[0].total.value,null);assert.equal(observationCount(v.history[0].total),'—');assert.equal(v.history[1].total.value,6);assert.match(v.history[0].detail,/falhou/);
});
test('complete source zero is scoped history only and never proof there are no open sessions',()=>{
 const v=petObservation(data('copilot',inventory(0,'copilotLiveEndpoints')),'pet');assert.equal(observationCount(v.history[0].total),'0');assert.match(v.history[0].detail,/desta fonte/);assert.equal(observationCount(v.opened),'—');assert.equal(v.history[1].total.value,6);
});
test('partial native observed subtotal keeps lower bound while local archive and dates remain independent',()=>{
 const n=inventory(null,'codexDaemon','partial');n.counts.total=count(null,90);n.counts.archived=count(null,12);n.observedAt=1700000009;n.stale=true;const v=petObservation(data('codex',n),'pet');assert.equal(observationCount(v.history[0].total),'≥ 90');assert.equal(observationCount(v.history[0].archived),'≥ 12');assert.equal(v.history[0].updatedAt,1700000009);assert.equal(v.history[0].stale,true);assert.equal(v.history[1].updatedAt,1700000001);assert.equal(v.history[1].archived.value,2);
});
test('bounded empty rows do not erase authoritative history counts or become a census',()=>{
 const n=inventory(100,'codexDaemon');n.limit=0;n.truncated=true;const v=petObservation(data('codex',n),'pet');assert.equal(v.sessions.length,0);assert.equal(v.history[0].total.value,100);assert.equal(observationCount(v.opened),'—');
});
