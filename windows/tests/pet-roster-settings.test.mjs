import {test} from 'node:test';
import assert from 'node:assert/strict';
import {PetRosterPreferences} from '../src/core/pet-roster-settings.ts';
import {Bridge} from '../src/core/bridge.ts';
import {DEFAULT_SETTINGS,State} from '../src/core/state.ts';
import {internals} from './tauri.mjs';
import {pills} from '../src/island/voice.ts';
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return{promise,resolve,reject};};
const flush=async()=>{for(let i=0;i<10;i++)await Promise.resolve();};
const initial=()=>({...structuredClone(DEFAULT_SETTINGS),mainPill:'agent_codex',activeIntegrations:['agent_copilot','agent_antigravity','ai_ollama','integration_spotify']});

test('dedicated requests contain one intention and cannot carry a stale settings list',async()=>{
 const previous=internals.invoke;let sent;
 internals.invoke=async(cmd,args)=>{sent={cmd,args};return {mainPill:'agent_codex',activeIntegrations:['agent_antigravity']};};
 try{await Bridge.petRosterSettingsPatch({action:'setActive',id:'agent_copilot',enabled:false,activeIntegrations:[],soundEnabled:false});assert.deepEqual(sent,{cmd:'pet_roster_settings_patch',args:{request:{action:'setActive',id:'agent_copilot',enabled:false}}});}
 finally{internals.invoke=previous;}
});
test('only confirmed roster fields are adopted and a failed write leaves preferences unchanged',async()=>{
 let model=initial(),fail=false;const requests=[];
 const prefs=new PetRosterPreferences({save:async request=>{requests.push(request);if(fail)throw Error('synthetic disk failure');return {mainPill:model.mainPill,activeIntegrations:[...model.activeIntegrations,'integration_notion'],soundEnabled:false};},adopt:roster=>{model={...model,...roster};}});
 await prefs.change({action:'setActive',id:'integration_notion',enabled:true});assert.equal(model.soundEnabled,DEFAULT_SETTINGS.soundEnabled);assert.ok(model.activeIntegrations.includes('agent_antigravity'));
 const before=structuredClone(model);fail=true;await assert.rejects(prefs.change({action:'setActive',id:'agent_copilot',enabled:false}));assert.deepEqual(model,before);assert.equal(requests.length,2);
});
test('a newer native settings observation wins over a delayed older write response',async()=>{
 let model=initial();const job=deferred();const prefs=new PetRosterPreferences({save:()=>job.promise,adopt:roster=>{model={...model,...roster};}});
 const pending=prefs.change({action:'setActive',id:'agent_copilot',enabled:false});await flush();
 prefs.observe({...model,mainPill:'agent_antigravity',activeIntegrations:['ai_ollama','agent_copilot']});
 job.resolve({mainPill:'agent_codex',activeIntegrations:['ai_ollama','agent_antigravity']});await pending;
 assert.equal(model.mainPill,'agent_antigravity');assert.deepEqual(model.activeIntegrations,['ai_ollama','agent_copilot']);
});
test('requests capture caller values and serialize without blocking later work after rejection',async()=>{
 const jobs=[],requests=[];const prefs=new PetRosterPreferences({save:request=>{requests.push(request);const job=deferred();jobs.push(job);return job.promise;},adopt:()=>{}});
 const request={action:'setActive',id:'agent_copilot',enabled:false};const first=prefs.change(request);request.id='agent_antigravity';
 const second=prefs.change({action:'setActive',id:'integration_notion',enabled:true});await flush();assert.equal(jobs.length,1);assert.equal(requests[0].id,'agent_copilot');
 jobs[0].reject(Error('synthetic failure'));await assert.rejects(first);await flush();assert.equal(jobs.length,2);jobs[1].resolve({mainPill:'agent_codex',activeIntegrations:['agent_antigravity','integration_notion']});await second;
});
test('malformed acknowledgements cannot corrupt the roster or replace unrelated settings',async()=>{
 for(const reply of [null,{},[],{mainPill:'agent_codex',activeIntegrations:'bad'},{mainPill:'agent_codex',activeIntegrations:[null]}]){
  let adopted=false;const prefs=new PetRosterPreferences({save:async()=>reply,adopt:()=>{adopted=true;}});await assert.rejects(prefs.change({action:'setMain',id:'agent_codex'}));assert.equal(adopted,false);
 }
});
test('spoken changes wait for durable acknowledgement and never use general save_settings',async()=>{
 State.settings=initial();State.tasks=[];const original=internals.invoke;const calls=[];const job=deferred();
 internals.invoke=async(cmd,args)=>{calls.push({cmd,args});if(cmd==='pet_roster_settings_patch')return job.promise;throw Error('unexpected command');};
 try{let finished=false;const pending=pills({action:'remove',names:['Copilot']}).then(reply=>{finished=true;return JSON.parse(reply);});await flush();assert.equal(finished,false);assert.ok(State.settings.activeIntegrations.includes('agent_copilot'));
  job.resolve({mainPill:'agent_codex',activeIntegrations:['agent_antigravity','ai_ollama','integration_spotify']});const reply=await pending;assert.deepEqual(reply.done,['Copilot CLI']);assert.deepEqual(reply.failed,[]);assert.ok(State.settings.activeIntegrations.includes('agent_antigravity'));assert.ok(!State.settings.activeIntegrations.includes('agent_copilot'));assert.deepEqual(calls,[{cmd:'pet_roster_settings_patch',args:{request:{action:'setActive',id:'agent_copilot',enabled:false}}}]);
 }finally{internals.invoke=original;}
});
test('spoken failures and retired or unknown pets never report a successful change',async()=>{
 State.settings=initial();const before=structuredClone(State.settings),original=internals.invoke;let calls=0;
 internals.invoke=async()=>{calls++;throw Error('synthetic disk failure');};
 try{const reply=JSON.parse(await pills({action:'remove',names:['Copilot','GitHub','unknown-pet']}));assert.deepEqual(reply.done,[]);assert.equal(reply.failed.length,3);assert.equal(calls,1);assert.deepEqual(State.settings,before);await assert.rejects(pills({action:'unknown',names:['Ollama']}));}
 finally{internals.invoke=original;}
});
