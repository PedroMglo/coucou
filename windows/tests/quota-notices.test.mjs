import {test} from 'node:test';
import assert from 'node:assert/strict';
import {QuotaNotices,quotaForPet} from '../src/mochi/quota-notices.ts';
import {NoticeQueue} from '../src/mochi/notices.ts';

const NOW=Date.UTC(2026,9,18,12);
const MINUTE=60_000;
const RESET=NOW+24*60*MINUTE;
const configured=(...pets)=>new Set(pets);
function usage(used=90,{provider='codex',now=NOW,...extra}={}) {
  return {providers:{[provider]:{updated:now/1000,windows:[{label:'Included',used,reset_at:RESET/1000}],...extra}}};
}
const quota=(value,now=NOW,pet='agent_codex')=>quotaForPet(pet,value,now);

test('quota thresholds use measured percentages and 100 percent never infers a block',()=>{
  for(const used of [0,12,84.99])assert.equal(quota(usage(used)).status,'ok');
  for(const used of [85,95,100]) {
    const result=quota(usage(used));
    assert.equal(result.status,'warning');
    assert.equal(result.usedPercent,used);
    assert.equal(result.updatedAt,NOW);
    assert.equal(result.resetsAt,RESET);
  }
  assert.equal(quota(usage(100,{blocked:false})).status,'warning');
  assert.equal(quota(usage(100,{blocked:'true'})).status,'warning');
  assert.equal(quota(usage(12,{blocked:true})).status,'blocked');
  assert.equal(quota(usage(0,{windows:[],blocked:true})).status,'blocked');
});

test('fresh explicit blocks expire with stale, aged, future or renewed information',()=>{
  const expired=usage(100,{blocked:true,windows:[{used:100,reset_at:NOW/1000}]});
  const readings=[
    usage(100,{blocked:true,stale:true}),
    usage(100,{blocked:true,updated:(NOW-5*MINUTE-1)/1000}),
    usage(100,{blocked:true,updated:(NOW+1)/1000}),
    expired,
  ];
  for(const value of readings) {
    const result=quota(value);
    assert.equal(result.status,'stale');
    assert.equal(result.usedPercent,undefined);
    assert.equal(result.resetsAt,undefined);
    assert.doesNotMatch(result.text,/0%/);
  }
  assert.equal(quota(usage(90,{updated:(NOW-5*MINUTE)/1000})).status,'warning');
  assert.equal(quota(usage(90,{updated:(NOW+1)/1000})).updatedAt,undefined);
  assert.equal(quota(usage(90,{updated:undefined,blocked:true})).status,'unknown');
  assert.equal(quota(usage(90,{updated:NaN,blocked:true})).status,'unknown');
});

test('reset seconds and ISO timestamps agree and a passed renewal never fabricates zero',()=>{
  const iso=new Date(RESET).toISOString();
  for(const window of [{used:90,reset:RESET/1000},{used:90,reset:iso},{used:90,reset_at:iso}]) {
    const result=quota(usage(0,{windows:[window]}));
    assert.equal(result.status,'warning');
    assert.equal(result.resetsAt,RESET);
  }
  const old=quota(usage(95,{windows:[{used:95,reset:new Date(NOW-1).toISOString()}]}));
  assert.equal(old.status,'stale');assert.equal(old.usedPercent,undefined);
  const renewed=quota(usage(8,{windows:[{used:8,reset_at:(RESET+MINUTE)/1000}]}));
  assert.equal(renewed.status,'ok');assert.equal(renewed.usedPercent,8);
});

test('unknown, malformed and non-finite data stay distinct from healthy zero',()=>{
  const malformed=[undefined,null,[],{}, {providers:null},{providers:[]},{providers:{codex:null}},{providers:{codex:[]}}];
  for(const value of malformed)assert.equal(quota(value).status,'unknown');
  const values=[NaN,Infinity,-Infinity,-1,101,'90',null,undefined];
  for(const used of values) {
    const result=quota(usage(0,{windows:[{used}]}));
    assert.equal(result.status,'unknown');assert.equal(result.usedPercent,undefined);
  }
  for(const windows of [null,{},'90',[null],['bad'],[{used:20,reset:'not-a-date'}],[{used:20,reset:0}],[{used:20,reset:NaN}]]) {
    const result=quota(usage(0,{windows}));
    assert.equal(result.status,'unknown');assert.equal(result.usedPercent,undefined);
  }
  assert.equal(quota(usage(0)).status,'ok');
  assert.equal(quota(usage(90),NaN).status,'unknown');
  assert.equal(quota(usage(90),Infinity).status,'unknown');
  assert.equal(quota({providers:Object.create({codex:usage(90).providers.codex})}).status,'unknown');
});

test('unlimited names alone do not establish all-model availability or cancel an observed warning',()=>{
  const unknown=quota(usage(0,{windows:[],unlimited:['basic-model']}));
  assert.equal(unknown.status,'unknown');assert.equal(unknown.usedPercent,undefined);
  assert.match(unknown.text,/restantes/);
  assert.equal(quota(usage(100,{unlimited:['basic-model']})).status,'warning');
  assert.equal(quota(usage(100,{blocked:true,unlimited:['basic-model']})).status,'blocked');
  assert.equal(quota(usage(0,{windows:[{used:20},{used:'unknown'}]})).status,'unknown');
  assert.equal(quota(usage(0,{windows:[{used:90},{used:'unknown'}]})).status,'warning');
});

test('canonical pets and the eight explicit Codex services share only their own account',()=>{
  for(const [pet,provider] of [['agent_codex','codex'],['agent_copilot','copilot'],['integration_claude','claude'],['agent_antigravity','antigravity']]) {
    assert.equal(quotaForPet(pet,usage(90,{provider}),NOW).provider,provider);
    assert.equal(quotaForPet(pet,usage(90,{provider}),NOW).status,'warning');
  }
  for(const pet of ['integration_resend','integration_n8n','integration_vercel','integration_github','integration_notion','integration_calcom','integration_stripe','integration_spotify']) {
    assert.equal(quotaForPet(pet,usage(90),NOW).provider,'codex');
  }
  for(const pet of ['ai_ollama','ai_lmstudio','integration_music','integration_unknown','agent_gemini','agent_opencode','ai_openai','']) {
    assert.equal(quotaForPet(pet,usage(90),NOW),undefined);
  }
});

test('poll changes to percentage and update time do not repeat the same warning',()=>{
  const notices=new QuotaNotices(),pets=configured('agent_codex');
  const first=notices.observe(usage(85),pets,NOW);
  assert.equal(first.length,1);assert.equal(first[0].urgent,false);assert.equal(first[0].view,'overview');
  assert.equal(first[0].pet,'agent_codex');
  for(let i=1;i<=4;i++)assert.deepEqual(notices.observe(usage(85+i,{now:NOW+i*MINUTE}),pets,NOW+i*MINUTE),[]);
  assert.deepEqual(notices.observe(usage(100,{now:NOW+16*MINUTE}),pets,NOW+16*MINUTE),[]);
  assert.ok(!first[0].key.includes(String(NOW/1000)));
  assert.ok(!first[0].key.includes('85'));
});

test('new windows respect the account cooldown and can notify after it passes',()=>{
  const notices=new QuotaNotices(),pets=configured('agent_codex');
  const first=notices.observe(usage(90),pets,NOW)[0];
  const nextReset=RESET+MINUTE;
  const next=at=>usage(91,{now:at,windows:[{label:'Included',used:91,reset_at:nextReset/1000}]});
  assert.deepEqual(notices.observe(next(NOW+MINUTE),pets,NOW+MINUTE),[]);
  const later=notices.observe(next(NOW+15*MINUTE),pets,NOW+15*MINUTE);
  assert.equal(later.length,1);assert.notEqual(later[0].key,first.key);
  assert.deepEqual(notices.observe(next(NOW+16*MINUTE),pets,NOW+16*MINUTE),[]);
});

test('a real explicit block may break cooldown once; oscillations cannot repeat the bypass',()=>{
  const notices=new QuotaNotices(),pets=configured('agent_codex');
  assert.equal(notices.observe(usage(90),pets,NOW).length,1);
  const blocked=notices.observe(usage(90,{now:NOW+MINUTE,blocked:true}),pets,NOW+MINUTE);
  assert.equal(blocked.length,1);assert.match(blocked[0].key,/:blocked:/);assert.equal(blocked[0].urgent,false);
  const window=[{label:'New included window',used:90,reset_at:(RESET+MINUTE)/1000}];
  assert.deepEqual(notices.observe(usage(90,{now:NOW+2*MINUTE,windows:window}),pets,NOW+2*MINUTE),[]);
  assert.deepEqual(notices.observe(usage(90,{now:NOW+3*MINUTE,blocked:true,windows:window}),pets,NOW+3*MINUTE),[]);
  assert.equal(notices.observe(usage(90,{now:NOW+16*MINUTE,blocked:true,windows:window}),pets,NOW+16*MINUTE).length,1);
});

test('initial unknown or stale data stays quiet; loss of known data notifies once',()=>{
  const pets=configured('agent_codex'),notices=new QuotaNotices();
  assert.deepEqual(notices.observe(undefined,pets,NOW),[]);
  assert.deepEqual(notices.observe(usage(90,{stale:true}),pets,NOW+MINUTE),[]);
  assert.deepEqual(notices.observe(usage(20,{now:NOW+2*MINUTE}),pets,NOW+2*MINUTE),[]);
  const lost=notices.observe(undefined,pets,NOW+3*MINUTE);
  assert.equal(lost.length,1);assert.match(lost[0].key,/:unknown:/);
  assert.deepEqual(notices.observe(undefined,pets,NOW+20*MINUTE),[]);
  const delayed=new QuotaNotices();
  delayed.observe(usage(90),pets,NOW);
  assert.deepEqual(delayed.observe(usage(90,{stale:true}),pets,NOW+MINUTE),[]);
  assert.equal(delayed.observe(usage(90,{stale:true}),pets,NOW+15*MINUTE).length,1);
});

test('one account notice prefers its configured canonical pet, otherwise its first configured service',()=>{
  const value=usage(95),services=['integration_notion','integration_github','integration_spotify'];
  const canonical=new QuotaNotices().observe(value,configured(...services,'agent_codex'),NOW);
  assert.equal(canonical.length,1);assert.equal(canonical[0].pet,'agent_codex');
  const service=new QuotaNotices().observe(value,configured(...services),NOW);
  assert.equal(service.length,1);assert.equal(service[0].pet,'integration_notion');
  const unknown=new QuotaNotices().observe(value,configured('integration_music','integration_unknown'),NOW);
  assert.deepEqual(unknown,[]);
});

test('disabled accounts cannot emit or consume an initial warning',()=>{
  const notices=new QuotaNotices();
  assert.deepEqual(notices.observe(usage(90),configured(),NOW),[]);
  assert.deepEqual(notices.observe(usage(90),configured('agent_copilot'),NOW),[]);
  assert.equal(notices.observe(usage(90),configured('agent_codex'),NOW).length,1);
  assert.deepEqual(notices.observe(usage(91,{now:NOW+MINUTE}),configured(),NOW+MINUTE),[]);
  assert.deepEqual(notices.observe(usage(92,{now:NOW+2*MINUTE}),configured('agent_codex'),NOW+2*MINUTE),[]);
});

test('recovery starts a new episode while still respecting the provider cooldown',()=>{
  const notices=new QuotaNotices(),pets=configured('agent_codex');
  const first=notices.observe(usage(90),pets,NOW)[0];
  assert.deepEqual(notices.observe(usage(20,{now:NOW+MINUTE}),pets,NOW+MINUTE),[]);
  assert.deepEqual(notices.observe(usage(90,{now:NOW+2*MINUTE}),pets,NOW+2*MINUTE),[]);
  const later=notices.observe(usage(90,{now:NOW+15*MINUTE}),pets,NOW+15*MINUTE);
  assert.equal(later.length,1);assert.notEqual(later[0].key,first.key);
});

test('approval notices keep priority and quota observation does not mutate source data',()=>{
  const window=Object.freeze({label:'Included',used:90,reset_at:RESET/1000});
  const raw=Object.freeze({updated:NOW/1000,windows:Object.freeze([window])});
  const value=Object.freeze({providers:Object.freeze({codex:raw})});
  const [notice]=new QuotaNotices().observe(value,configured('agent_codex'),NOW);
  const queue=new NoticeQueue();
  queue.push(notice);
  queue.push({key:'approval',pet:'agent_codex',text:'Permission required',urgent:true,view:'approval'});
  assert.equal(queue.take().key,'approval');
  assert.equal(queue.take(),undefined,'the permission decision replaces a lower-priority notice for the same pet');
  assert.equal(raw.windows[0].used,90);assert.equal(raw.updated,NOW/1000);
});
