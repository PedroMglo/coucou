import {test} from 'node:test';import assert from 'node:assert/strict';
import {petObservation,observationCount} from '../src/core/pet-observability.ts';
import {inventoryFeedback,tokenFeedback} from '../src/core/pet-observability-feedback.ts';
const count=n=>({value:n,observed:n??0});
const inv=(n,coverage='complete',extra={})=>({scope:'mochiLibrary',coverage,observedAt:1700000000,stale:false,refreshing:false,reason:null,counts:{total:count(n),archived:count(n?1:0),confirmedActive:count(n?1:0),working:count(n?1:0),waiting:count(0),reserved:count(0),idle:count(0),unknown:0},sessions:[],listed:0,limit:12,truncated:false,...extra});
const data=(pet,provider,native,n=4)=>({schemaVersion:1,generatedAt:1700000010,pets:[{petId:pet,provider,enabled:true,mochi:inv(n),usage:null}],providers:[{provider,petIds:[pet],native,usage:null}]});
for(const [pet,provider]of [['agent_codex','codex'],['agent_copilot','copilot'],['agent_antigravity','antigravity'],['integration_github','codex'],['ai_ollama','ollama']])test(`${pet}: library metrics appear immediately with unknown or unsupported external source, including measured zero`,()=>{
 for(const native of [undefined,inv(0,'notSupported'),inv(0,'unavailable'),inv(0,'unavailable',{refreshing:true}),inv(0,'unavailable',{reason:'refreshFailed'})])for(const n of [0,4]){
  const value=petObservation(data(pet,provider,native,n),pet);assert.equal(value.primaryScope,'pet');assert.equal(value.total.value,n);assert.equal(value.active.value,n?1:0);assert.equal(value.archived.value,n?1:0);assert.equal(observationCount(value.total),String(n));assert.match(value.scopeLabel,/Biblioteca deste pet/);assert.match(value.scopeDetail,/Não incluem as sessões externas/);assert.ok(value.externalStatus);assert.equal(value.provider,provider);
 }
});
test('partial external evidence remains partial and does not become the zero local library',()=>{
 const native=inv(20,'partial',{scope:'codexDaemon'});native.counts.confirmedActive={value:null,observed:4};const v=petObservation(data('agent_codex','codex',native,0),'agent_codex');assert.equal(v.primaryScope,'provider');assert.equal(v.total.value,20);assert.equal(observationCount(v.total),'≥ 20');assert.equal(v.active.value,null);assert.equal(v.active.observed,4);assert.equal(observationCount(v.active),'≥ 4');assert.match(v.scopeLabel,/Codex/);
});
test('late native collection switches source and all main metrics without adding overlapping counts',()=>{
 const d=data('agent_codex','codex',inv(0,'unavailable',{refreshing:true}));assert.equal(petObservation(d,'agent_codex').total.value,4);d.providers[0].native=inv(70,'complete',{scope:'codexDaemon',observedAt:1700000008});const v=petObservation(d,'agent_codex');assert.equal(v.primaryScope,'provider');assert.equal(v.total.value,70);assert.equal(v.library.total.value,4);assert.equal(v.updatedAt,1700000008);
});
test('external diagnostics distinguish unsupported, pending, disabled, explicit unavailable and generic failure',()=>{
 assert.match(inventoryFeedback(inv(0,'notSupported')),/não disponibiliza/);assert.match(inventoryFeedback(inv(0,'unavailable',{refreshing:true})),/A consultar/);assert.match(inventoryFeedback(inv(0,'unavailable',{reason:'nativePetDisabled'})),/desativado/);assert.match(inventoryFeedback(inv(0,'unavailable',{reason:'clientUnavailable'})),/cliente nativo/);const generic=inventoryFeedback(inv(0,'unavailable',{reason:'refreshFailed'}));assert.match(generic,/consulta.*falhou/);assert.doesNotMatch(generic,/cliente nativo/);
});
test('noObservations retains ledger period and pending work without inventing consumed zero',()=>{
 const v=tokenFeedback({scope:'mochi-observed-attempts',provider:'ollama',owner:null,status:'noObservations',since:1700000000,attempts:{pending:2}},'ollama');assert.equal(v.status,'noObservations');assert.equal(v.since,1700000000);assert.match(v.detail,/não confirma consumo zero/);assert.match(v.detail,/2 pedidos por concluir/);
});
test('unknown snapshots and foreign token scopes never become provider totals',()=>{
 const s={scope:'mochi-observed-attempts',provider:'codex',owner:null,status:'unavailable',since:null};assert.equal(tokenFeedback(s,'codex').status,'unavailable');assert.match(tokenFeedback(s,'codex').detail,/quota da conta/);for(const u of [undefined,{...s,provider:'ollama'},{...s,owner:'integration_github'},{...s,scope:'last-turn'}])assert.equal(tokenFeedback(u,'codex').status,'unknown');
});
