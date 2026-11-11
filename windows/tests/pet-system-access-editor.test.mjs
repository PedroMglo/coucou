import {test} from 'node:test';import assert from 'node:assert/strict';

test('Guardian reviews only its own Health grant without enabling Sound or contacting a model',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{
  const before=f.doc,editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'agent_system_guardian');editor.inputs.get('health').checked=true;editor.prepare();
  assert.equal(f.calls.length,0);assert.match(editor.details.textContent,/CPU\/memória\/uptime/);assert.deepEqual(editor.choice.grants,[{subplugin:'health',operation:'observe',resource:{kind:'hostHealth'}}]);
  await editor.apply();assert.equal(f.calls.length,1);assert.equal(f.calls[0].command,'pet_system_access_set');assert.deepEqual(f.doc.config.pets.ai_ollama,before.config.pets.ai_ollama);assert.equal(f.doc.config.pets.agent_system_guardian.inference,undefined);
 }finally{internals.invoke=original;}
});
test('Health query is explicit, belongs to the selected pet and discards a late result',async()=>{
 const original=internals.invoke,f=fixture();let finish;const calls=[];
 internals.invoke=async(command,args)=>{calls.push({command,args});assert.equal(command,'pet_health_snapshot');return await new Promise(resolve=>finish=resolve);};
 try{
  const editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'agent_system_guardian');assert.equal(calls.length,0);
  const pending=editor.observeHealth();assert.deepEqual(calls,[{command:'pet_health_snapshot',args:{petId:'agent_system_guardian'}}]);editor.show(f.doc,'ai_ollama');const before=editor.status.textContent;
  finish({petId:'agent_system_guardian',inferenceUsed:false,memory:{value:null},cpu:{value:null},uptime:{value:null},findings:[],observedAt:1});await pending;assert.equal(editor.status.textContent,before);
 }finally{internals.invoke=original;}
});
import {installFakeDom} from './fakedom.mjs';import {internals} from './tauri.mjs';
import {PetFactory} from '../src/core/pet-factory.ts';import {PetSystemAccessEditor} from '../src/desktop/pet-system-access-editor.ts';import {PetSettings} from '../src/desktop/pet-settings.ts';
installFakeDom();const clone=v=>structuredClone(v);
function fixture(){
 let doc={config:PetFactory.configuration(),path:'/generated/pets.json',revision:'before'},counter=0,active=true;const calls=[],saved=[];
 const hooks={active:()=>active,canStart:()=>active,generalDirty:()=>false,busy:()=>{},saved:d=>saved.push(clone(d))};
 const invoke=async(command,args)=>{calls.push({command,args:clone(args)});if(command==='pet_configuration')return clone(doc);if(command==='pet_roster')return ['ai_ollama','integration_spotify'];if(command==='pet_system_access_set'){assert.equal(args.request.expectedRevision,doc.revision);doc=clone(doc);doc.config.version=2;doc.config.pets[args.request.petId].systemAccess={schemaVersion:1,enabled:true,revision:(++counter).toString(16).padStart(32,'0'),grants:clone(args.request.grants)};doc.revision=`after-${counter}`;return clone(doc);}throw Error('Unexpected effect: '+command);};
 return{hooks,calls,saved,invoke,get doc(){return clone(doc);},set doc(d){doc=clone(d);},set active(v){active=v;}};
}
test('grant review has no IO; confirm binds one pet and preserves exact-resource grants and the other pet',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{const doc=f.doc;doc.config.version=2;doc.config.pets.ai_ollama.systemAccess={schemaVersion:1,enabled:true,revision:'a'.repeat(32),grants:[{subplugin:'sound',operation:'observe',resource:{kind:'audioOutput',key:'audio-0123456789abcdef'}}]};f.doc=doc;const editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'ai_ollama');editor.inputs.get('observe').checked=true;editor.inputs.get('volume').checked=true;editor.prepare();assert.equal(f.calls.length,0);assert.match(editor.details.textContent,/ai_ollama/);assert.match(editor.details.textContent,/todas as saídas/);await editor.apply();assert.equal(f.calls.length,1);assert.equal(f.calls[0].command,'pet_system_access_set');assert.equal(f.calls[0].args.request.petId,'ai_ollama');assert.equal(f.saved.length,1);assert.equal(f.doc.config.pets.ai_ollama.systemAccess.grants.length,3);assert.equal(f.doc.config.pets.ai_ollama.systemAccess.grants[0].resource.key,'audio-0123456789abcdef');assert.deepEqual(f.doc.config.pets.integration_spotify,doc.config.pets.integration_spotify);assert.equal(editor.dirty,false);}finally{internals.invoke=original;}
});
test('pet switches, changed arguments and ended origins require a new native review',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{const editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'ai_ollama');editor.inputs.get('observe').checked=true;editor.prepare();editor.inputs.get('mute').checked=true;await editor.apply();assert.equal(f.calls.length,0);editor.prepare();editor.show(f.doc,'integration_spotify');editor.show(f.doc,'ai_ollama');assert.equal(editor.inputs.get('mute').checked,true);await editor.apply();assert.equal(f.calls.length,0);editor.prepare();f.active=false;await editor.apply();assert.equal(f.calls.length,0);}finally{internals.invoke=original;}
});
test('old-revision ACK is refused and the existing management view can review its own grant draft',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{const settings=new PetSettings();await settings.load('ai_ollama');assert.equal(settings.systemAccess.el.disabled,false);const editor=settings.systemAccess;editor.inputs.get('observe').checked=true;editor.prepare();assert.equal(editor.review.hidden,false,'own grant draft must not be mistaken for unrelated unsaved preferences');await editor.apply();assert.equal(f.calls.filter(c=>c.command==='pet_system_access_set').length,1);
 const standalone=new PetSystemAccessEditor(f.hooks);standalone.show(f.doc,'ai_ollama');standalone.inputs.get('mute').checked=true;standalone.prepare();const previous=f.doc;internals.invoke=async()=>{const stale=clone(previous);stale.config.pets.ai_ollama.systemAccess.grants=clone(standalone.choice.grants);return stale;};await standalone.apply();assert.equal(f.saved.length,0);assert.match(standalone.status.textContent,/não foi confirmada/);assert.equal(standalone.dirty,true);
 }finally{internals.invoke=original;}
});

test('restored suspended grants remain desired data until a fresh native confirmation',async()=>{const original=internals.invoke,f=fixture();internals.invoke=f.invoke;try{const doc=f.doc;doc.config.version=2;doc.config.pets.ai_ollama.systemAccess={schemaVersion:1,enabled:false,revision:'a'.repeat(32),grants:[{subplugin:'sound',operation:'observe',resource:{kind:'audioOutputs'}}]};f.doc=doc;const editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'ai_ollama');assert.match(editor.status.textContent,/suspensas/);assert.equal(f.calls.length,0);editor.prepare();assert.equal(f.calls.length,0);await editor.apply();assert.equal(f.doc.config.pets.ai_ollama.systemAccess.enabled,true);assert.notEqual(f.doc.config.pets.ai_ollama.systemAccess.revision,'a'.repeat(32));}finally{internals.invoke=original;}});

test('inventory review creates only its distinct OS/kernel grant and preserves Health and other owners',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{const doc=f.doc;doc.config.version=2;doc.config.pets.agent_system_guardian.systemAccess={schemaVersion:1,enabled:true,revision:'a'.repeat(32),grants:[{subplugin:'health',operation:'observe',resource:{kind:'hostHealth'}}]};f.doc=doc;
  const editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'agent_system_guardian');editor.inputs.get('inventory').checked=true;editor.prepare();assert.equal(f.calls.length,0);assert.deepEqual(editor.choice.grants,[{subplugin:'health',operation:'observe',resource:{kind:'hostHealth'}},{subplugin:'inventory',operation:'observe',resource:{kind:'hostComponents'}}]);assert.match(editor.details.textContent,/sem processos/);await editor.apply();assert.equal(f.calls[0].command,'pet_system_access_set');assert.deepEqual(f.doc.config.pets.ai_ollama,doc.config.pets.ai_ollama);
 }finally{internals.invoke=original;}
});
test('inventory query never redirects an in-flight result after changing pet',async()=>{
 const original=internals.invoke,f=fixture();let resolve;const calls=[];internals.invoke=(command,args)=>{calls.push({command,args});return new Promise(done=>resolve=done);};
 try{const editor=new PetSystemAccessEditor(f.hooks);editor.show(f.doc,'agent_system_guardian');assert.equal(calls.length,0);const pending=editor.observeInventory();assert.deepEqual(calls,[{command:'pet_host_inventory',args:{petId:'agent_system_guardian'}}]);editor.show(f.doc,'ai_ollama');const before=editor.status.textContent;resolve({petId:'agent_system_guardian',kind:'systemHostInventory',inferenceUsed:false,desiredState:'unknownNotCompared'});await pending;assert.equal(editor.status.textContent,before);}finally{internals.invoke=original;}
});
