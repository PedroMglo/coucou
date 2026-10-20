import {test} from 'node:test';import assert from 'node:assert/strict';
import {observationNumber,observationCount,observationTokens,petObservation,observationState} from '../src/core/pet-observability.ts';
import {PetObservabilityResource} from '../src/core/pet-observability-resource.ts';
const count=value=>({value,observed:value??0});
const inventory=(total,rows=[])=>({scope:'mochiLibrary',coverage:'complete',observedAt:1700000000,stale:false,refreshing:false,reason:null,counts:{total:count(total),archived:count(2),confirmedActive:count(1),working:count(1),waiting:count(0),reserved:count(0),idle:count(0),unknown:0},sessions:rows,listed:rows.length,limit:12,truncated:false});
const row=(pet='agent_codex',id='cid-one')=>({key:'mochi:codex:'+id,provider:'codex',petId:pet,origin:'mochi',sessionId:'native-one',conversationId:id,title:'Title',cwd:'/tmp/synthetic',state:'working',archived:false,updatedAt:1700000000,observedAt:1700000000,open:{kind:'mochi',petId:pet,provider:pet==='agent_codex'?'codex':pet,conversationId:id}});
const snapshot=()=>({schemaVersion:1,generatedAt:1700000009,pets:[{petId:'agent_codex',provider:'codex',enabled:true,mochi:inventory(3,[row()]),usage:null}],providers:[{provider:'codex',petIds:['agent_codex'],native:{...inventory(20,[row()]),scope:'codexDaemon'},usage:null}]});
test('unknown, malformed and missing counters remain distinct from measured zero',()=>{
 for(const value of [null,undefined,'0',-1,.5,NaN,Infinity,Number.MAX_SAFE_INTEGER+1])assert.equal(observationNumber(value),null);
 assert.equal(observationCount({value:0,coverage:'complete',scope:'x'}),'0');
 assert.equal(observationCount({value:null,observed:4,coverage:'partial',scope:'x'}),'≥ 4');
 assert.equal(observationCount({value:7,coverage:'partial',scope:'x'}),'≥ 7');
 assert.equal(observationCount({value:0,coverage:'unavailable',scope:'x'}),'—');
});
test('decimal token counters retain precision and never turn absent measurements into zero',()=>{
 const value='900719925474099312345';assert.equal(observationTokens(value).replace(/\s/g,''),value);assert.equal(observationTokens('0'),'0');
 for(const v of [0,null,undefined,'1.5','-3','01','1e6','secret'])assert.equal(observationTokens(v),'—');
});
test('overlapping Mochi and native inventories keep separate totals and deduplicate only navigation rows',()=>{
 const result=petObservation(snapshot(),'agent_codex');assert.equal(result.total.value,20);assert.equal(result.library.total.value,3);assert.equal(result.sessions.length,1);assert.equal(result.updatedAt,1700000000);assert.match(result.scopeLabel,/local do Codex/);assert.equal(result.sessions[0].target.conversationId,'cid-one');
});
test('unavailable external inventory uses explicitly scoped library counters without claiming external totals',()=>{
 const data=snapshot();data.providers[0].native={...inventory(0),coverage:'notSupported',observedAt:null,stale:true};const result=petObservation(data,'agent_codex');assert.equal(result.primaryScope,'pet');assert.equal(result.total.value,3);assert.equal(result.active.value,1);assert.equal(result.library.total.value,3);assert.equal(result.updatedAt,1700000000);assert.match(result.scopeLabel,/Biblioteca deste pet/);assert.match(result.externalStatus,/não disponibiliza/);assert.equal(result.stale,false);assert.equal(result.sessions.length,1);
});
test('navigation retains the actual owner and origin; contradictory targets remain unavailable',()=>{
 const data=snapshot();const other=row('integration_spotify','spotify-cid');data.providers[0].native.sessions=[other,{...row('agent_codex','bad-cid'),open:{kind:'mochi',petId:'other',provider:'codex',conversationId:'bad-cid'}},{...row('agent_codex','external-cid'),origin:'external',conversationId:null,sessionId:'terminal-id',open:{kind:'external',petId:'agent_codex',provider:'codex',sessionId:'terminal-id'}}];
 const result=petObservation(data,'agent_codex');const spotify=result.sessions.find(s=>s.target?.petId==='integration_spotify');assert.equal(spotify.target.provider,'integration_spotify');assert.equal(spotify.target.sessionId,'spotify-cid');assert.equal(result.sessions.find(s=>s.id==='native-one'&&s.title==='Title'&&!s.canOpen).target,null);const external=result.sessions.find(s=>s.id==='terminal-id');assert.equal(external.origin,'external');assert.equal(external.target.origin,'external');assert.equal(external.target.sessionId,'terminal-id');
});
test('unloaded, idle and reserved states do not claim a closed terminal or an executing turn',()=>{
 assert.equal(observationState('reserved'),'A preparar');assert.equal(observationState('notLoaded'),'Guardada');assert.equal(observationState('idle'),'Sem turno em curso');assert.equal(observationState('unexpected'),'Estado por apurar');
 const data=snapshot();data.providers[0].native.sessions[0].state='reserved';data.providers[0].native.counts.confirmedActive=count(0);assert.equal(petObservation(data,'agent_codex').active.value,0);
});
test('switching pets cannot select a previous owner, duplicate owner or incompatible schema',()=>{
 const data=snapshot();assert.equal(petObservation(data,'ai_ollama'),null);data.pets.push({...data.pets[0]});assert.equal(petObservation(data,'agent_codex'),null);assert.equal(petObservation({...data,schemaVersion:2},'agent_codex'),null);
});
test('resource shares slow reads, retains dated data after failure and throttles repeated failures',async()=>{
 let now=100,read=0,resolve,fail=false;const resource=new PetObservabilityResource(()=>{read++;if(fail)throw Error('private provider detail');return new Promise(r=>resolve=r);},()=>now);
 const first=resource.refresh();assert.equal(resource.refresh(),first);await Promise.resolve();assert.equal(read,1);resolve(snapshot());await first;const original=resource.snapshot;
 fail=true;now+=5001;await resource.refresh();assert.equal(resource.snapshot,original);assert.equal(resource.snapshot.providers[0].native.observedAt,1700000000);assert.match(resource.error,/Não foi possível/);assert.doesNotMatch(resource.error,/private/);await resource.refresh();assert.equal(read,2);assert.equal(resource.loading,false);
});
test('a native completion while a snapshot is pending queues one fresh single-flight read',async()=>{
 const completions=[];let reads=0;const resource=new PetObservabilityResource(()=>{reads++;return new Promise(r=>completions.push(r));});
 const first=resource.refresh();await Promise.resolve();resource.invalidate();resource.invalidate();completions[0](snapshot());await first;await Promise.resolve();assert.equal(reads,2);assert.equal(resource.loading,true);completions[1](snapshot());await resource.refresh();assert.equal(reads,2);assert.equal(resource.loading,false);
});
const usageDimension=(tokens)=>({knownTokens:tokens,knownAttempts:tokens===null?0:1,partialAttempts:0,missingAttempts:tokens===null?1:0,unknownAttempts:0,derivedAttempts:0,coverage:tokens===null?'partial':'completeWithinTrackedAttempts'});
const measuredUsage=()=>({status:'observed',scope:'mochi-observed-attempts',provider:'codex',owner:null,since:1700000000,lastObservation:1700000002,freshness:'observed',input:usageDimension('3'),output:usageDimension('4'),total:usageDimension('900719925474099312345'),attempts:{tracked:1,pending:0,completed:1,failed:0,cancelled:0,interrupted:0},coverage:{status:'partial',historicalBeforeSince:'notCovered',outsideMochi:'notCovered',untrackedMochi:'notCovered'},persistence:{state:'memoryOnly',revision:null,lastCommittedAt:null,issue:null}});
test('provider usage is read from its own counters, never added or inferred from input/output',()=>{
 const data=snapshot();data.providers[0].usage=measuredUsage();const result=petObservation(data,'agent_codex');assert.equal(result.tokens.totalTokens,'900719925474099312345');assert.equal(result.tokens.inputTokens,'3');assert.equal(result.tokens.outputTokens,'4');assert.match(result.tokens.detail,/apenas em memória/);assert.match(result.tokens.detail,/Não inclui outras aplicações/);
 data.providers[0].usage.owner='codex';assert.equal(petObservation(data,'agent_codex').tokens,null);data.providers[0].usage.owner=null;data.providers[0].usage.provider='ollama';assert.equal(petObservation(data,'agent_codex').tokens,null);
});
test('observed attempts without token counters remain unknown while a measured zero survives',()=>{
 const data=snapshot(),usage=measuredUsage();data.providers[0].usage=usage;usage.input=usage.output=usage.total=usageDimension(null);let result=petObservation(data,'agent_codex');assert.equal(result.tokens.totalTokens,null);assert.match(result.tokens.detail,/1 sem contadores de total/);
 usage.total=usageDimension('0');assert.equal(petObservation(data,'agent_codex').tokens.totalTokens,'0');usage.total.knownAttempts=0;assert.equal(petObservation(data,'agent_codex').tokens.totalTokens,null);
});
test('two conversations sharing a native session retain separate exact navigation keys',()=>{
 const data=snapshot();data.pets[0].mochi.sessions=[row('agent_codex','first-cid'),row('agent_codex','second-cid')];data.providers[0].native.sessions=[];const sessions=petObservation(data,'agent_codex').sessions;assert.equal(sessions.length,2);assert.equal(sessions[0].id,sessions[1].id);assert.notEqual(sessions[0].key,sessions[1].key);assert.notEqual(sessions[0].target.conversationId,sessions[1].target.conversationId);
});
