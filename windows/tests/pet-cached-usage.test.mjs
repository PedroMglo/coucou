import {test} from 'node:test';import assert from 'node:assert/strict';
import {parseCachedUsage,CachedUsageResource,quotaProvider,quotaReset} from '../src/core/pet-cached-usage.ts';import {Bridge} from '../src/core/bridge.ts';import {internals,calls} from './tauri.mjs';
const snapshot=(q={updated:1700000000,stale:false,windows:[{used:42,label:'5 horas',reset_at:1700001000}]})=>({schemaVersion:1,source:'ia-barra-local-cache',readAt:1700000010,status:'available',reason:null,providers:{codex:q}});
test('quota parser preserves source date and measured zero, stripping private/identity fields',()=>{
 const v=snapshot({updated:1700000000,stale:false,account:'private',plan:'secret',note:'secret',windows:[{used:0,name:'Semana',label:'Período',reset:1700001000,token:'secret'}],unlimited:['Pedidos gratuitos']});v.providers.spoty={stale:false,windows:[{used:99}]};const p=parseCachedUsage(v);assert.equal(p.providers.codex.updated,1700000000);assert.equal(p.providers.codex.windows[0].used,0);assert.deepEqual(Object.keys(p.providers),['codex']);assert.doesNotMatch(JSON.stringify(p),/private|secret|account|plan|note|token/);
});
test('invalid percentages, dates, booleans and oversized windows are unknown rather than zero',()=>{
 for(const q of [{stale:false,windows:[{used:'1'}]},{stale:false,windows:[{used:-1}]},{stale:false,windows:[{used:101}]},{stale:false,windows:[{used:NaN}]},{stale:'false',windows:[]},{stale:false,updated:1700001000},{stale:false,unlimited:['bad\nlabel']},{stale:false,windows:Array.from({length:33},()=>({used:1}))}])assert.equal(parseCachedUsage(snapshot(q)).providers.codex,undefined);
});
test('missing cache, unsupported platform and an absent provider remain distinct',()=>{
 for(const status of ['unavailable','notSupported']){const v=parseCachedUsage({...snapshot(),status,reason:status==='unavailable'?'cacheUnavailable':'unsupportedPlatform'});assert.equal(v.status,status);assert.deepEqual(v.providers,{});}assert.equal(parseCachedUsage(snapshot()).providers.copilot,undefined);assert.equal(parseCachedUsage(snapshot({stale:false,windows:[{used:1}]})).providers.codex.stale,true);for(const v of [null,{}, {...snapshot(),schemaVersion:2},{...snapshot(),source:'account-network'}])assert.throws(()=>parseCachedUsage(v));
});
test('quota ownership uses explicit provider mapping and excludes Spoty and local models',()=>{
 assert.equal(quotaProvider('codex'),'codex');assert.equal(quotaProvider('antigravity'),'antigravity');assert.equal(quotaProvider('claude-cli'),'claude');for(const id of ['integration_spotify','integration_github','ollama','unknown',''])assert.equal(quotaProvider(id),null);
});
test('cache resource deduplicates pending reads, retains dated values and requires manual retry after failure',async()=>{
 let n=0,resolve,fail=false,now=1;const r=new CachedUsageResource(()=>{n++;if(fail)throw Error('private path');return new Promise(done=>resolve=done);},()=>now);const p=r.refresh();assert.equal(r.refresh(true),p);await Promise.resolve();resolve(snapshot());await p;const original=r.snapshot;now+=30001;fail=true;await r.refresh();assert.equal(r.snapshot,original);assert.equal(r.snapshot.providers.codex.updated,1700000000);assert.doesNotMatch(r.error,/private/);now+=90000;await r.refresh();assert.equal(n,2);await r.refresh(true);assert.equal(n,3);
});
test('reset dates keep source timestamps and native renewal labels without fabricating dates',()=>{
 assert.equal(quotaReset(undefined),null);assert.equal(quotaReset(Number.MAX_SAFE_INTEGER),null);assert.equal(quotaReset('fim da semana'),'fim da semana');assert.doesNotMatch(quotaReset(1700001000),/Invalid/);assert.doesNotMatch(quotaReset('2026-10-20T01:00:00Z'),/Invalid/);
});
test('Bridge uses only cache command with no path/provider/refresh payload and propagates refusal',async()=>{
 const original=internals.invoke;calls.length=0;try{internals.invoke=async(c,a)=>{calls.push([c,a]);if(c==='pet_cached_usage')return snapshot();throw Error('unexpected');};await Bridge.petCachedUsage();assert.deepEqual(calls,[['pet_cached_usage',{}]]);internals.invoke=async()=>{throw Error('refused');};await assert.rejects(Bridge.petCachedUsage(),/refused/);}finally{internals.invoke=original;}
});
