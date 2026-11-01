import test from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
import {RuntimeGoalActions,musicGoalPlan} from '../src/core/runtime-goals.ts';
import {PetSettings} from '../src/desktop/pet-settings.ts';import {PetFactory} from '../src/core/pet-factory.ts';import {PetPreferences} from '../src/mochi/pet-preferences.ts';import {installFakeDom} from './fakedom.mjs';
test('LM Studio credentials use an explicit distinct scope while the existing custom-server call stays identical',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});};
 await Bridge.localSetKey('http://127.0.0.1:1234','synthetic-private-token','lmstudio');await Bridge.localSetKey('https://example.invalid','synthetic-custom');
 assert.deepEqual(calls,[{command:'local_set_key',args:{url:'http://127.0.0.1:1234',key:'synthetic-private-token',provider:'lmstudio'}},{command:'local_set_key',args:{url:'https://example.invalid',key:'synthetic-custom'}}]);
 await assert.rejects(Bridge.localSetKey('http://127.0.0.1:1234','synthetic','codex'),/desconhecido/);assert.equal(calls.length,2);
 }finally{internals.invoke=saved;}
});
test('opening LM Studio settings cannot bind it; explicit bind and unbind preserve every other pet and permission',async()=>{
 installFakeDom();const originalInvoke=internals.invoke,previous=PetPreferences.config,source=PetFactory.configuration(),before=structuredClone(source),saves=[],unexpected=[];
 let stored={config:source,path:'/private-fixture/pets.json',revision:'native-before'};
 try{internals.invoke=async(command,args)=>{if(command==='pet_configuration')return structuredClone(stored);if(command==='pet_roster')return Object.keys(source.pets);if(command==='pet_configuration_save'){saves.push(structuredClone(args));stored={config:structuredClone(args.config),path:stored.path,revision:`native-after-${saves.length}`};return structuredClone(stored);}unexpected.push(command);throw Error(`Unexpected IO: ${command}`);};
 const view=new PetSettings();await view.load('ai_lmstudio');assert.equal(source.pets.ai_lmstudio.provider,'');assert.equal(view.engine.checked,false);assert.equal(view.profiles.hidden,true);assert.equal(saves.length,0);assert.deepEqual(source,before);
 view.engine.checked=true;view.engine.onchange();assert.equal(view.profiles.hidden,false);assert.equal(saves.length,0);assert.equal(source.pets.ai_lmstudio.provider,'');assert.equal(await view.store(),true);
 const bound=structuredClone(before);bound.pets.ai_lmstudio.provider='lmstudio';assert.deepEqual(saves[0],{config:bound,expectedRevision:'native-before'});assert.deepEqual(source,before);
 view.engine.checked=false;view.engine.onchange();assert.equal(view.profiles.hidden,true);assert.equal(await view.store(),true);assert.deepEqual(saves[1],{config:before,expectedRevision:'native-after-1'});assert.deepEqual(unexpected,[]);
 }finally{internals.invoke=originalInvoke;PetPreferences.update(previous);}
});
test('LM Studio goal preparation refreshes only its own metadata and preserves the exact reviewed engine binding',async()=>{
 const sent=[],engine={schemaVersion:1,endpoint:'http://127.0.0.1:1234',modelKey:'local/free',instanceId:'chosen-instance',modelHash:'a'.repeat(64)};
 const goal={id:'original-lm-goal',revision:'source',state:'draft',spec:{owner:{petId:'ai_lmstudio',provider:'lmstudio',adapter:'lmstudio'},criteria:['Paused'],intent:'Original intent'},executionContext:{},plans:[]};
 const transport={petRuntimeRefreshReadiness:async(...args)=>{sent.push(['metadata',...args]);},petRuntimePreparePlanning:async input=>{sent.push(['review',structuredClone(input)]);return {status:'review',ticket:{token:'native-only',proposal:{goalId:goal.id,revision:goal.revision,owner:goal.spec.owner,engineBinding:engine}}};}};
 const a=new RuntimeGoalActions(()=>{},transport),budget={deadlineAt:10000,maxModelCalls:3,maxToolCalls:6,maxOutputBytes:100000,maxTokens:null,maxCostMicros:null,payment:'localOnly'},limits={maxRounds:3,maxSteps:2,maxOutputBytes:2048,noProgressLimit:2};
 await a.preparePlanning(goal,budget,limits,'native-review');assert.deepEqual(sent[0],['metadata','ai_lmstudio',goal.id]);assert.deepEqual(sent[1][1],{petId:'ai_lmstudio',goalId:goal.id,revision:'source',requestId:'native-review',budget,limits});assert.deepEqual(a.reviews.get(goal.id).ticket.proposal.engineBinding,engine);assert.equal(sent.length,2);
});
