import {test} from 'node:test';import assert from 'node:assert/strict';
import {IslandAvailabilityPreferences} from '../src/core/island-availability.ts';
import {DEFAULT_SETTINGS} from '../src/core/state.ts';
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return{promise,resolve,reject};};
const flush=async()=>{for(let i=0;i<10;i++)await Promise.resolve();};
function fixture(flags={petControlBar:false,petSystemCenter:false}){
 let model={...DEFAULT_SETTINGS,...flags},server=structuredClone(model);const saves=[],reads=[];let holdSave=false,holdRead=false,failSave=false,failRead=false;
 const prefs=new IslandAvailabilityPreferences({initial:model,save:async value=>{const job=deferred();saves.push({value:structuredClone(value),...job});if(holdSave)return job.promise;if(failSave)throw Error('private disk error');server={...server,...structuredClone(value)};},read:async()=>{const job=deferred();reads.push(job);if(holdRead)return job.promise;if(failRead)throw Error('private read error');return structuredClone(server);},adopt:flags=>model={...model,...flags},changed:()=>{}});
 return {prefs,saves,reads,get model(){return model;},set model(value){model=value;},set server(value){server=value;},set holdSave(value){holdSave=value;},set holdRead(value){holdRead=value;},set failSave(value){failSave=value;},set failRead(value){failRead=value;}};
}
test('the two preferences remain independently configurable while companions are off',async()=>{
 for(const bar of [false,true])for(const system of [false,true]){
  const f=fixture({petControlBar:bar,petSystemCenter:system});assert.equal(f.model.companionMode,false);await f.prefs.change('petControlBar');assert.equal(f.saves[0].value.petControlBar,!bar);assert.equal(f.saves[0].value.petSystemCenter,undefined);assert.equal(f.saves[0].value.companionMode,undefined);assert.deepEqual(Object.keys(f.saves[0].value),['petControlBar']);assert.equal(f.prefs.flags.petControlBar,!bar);assert.equal(f.prefs.flags.petSystemCenter,system);
 }
});
test('slow failing writes keep confirmed switch values and readback adopts only owned flags',async()=>{
 const f=fixture({petControlBar:true,petSystemCenter:false});f.holdSave=true;const pending=f.prefs.change('petSystemCenter');assert.equal(f.prefs.status,'saving');assert.equal(f.prefs.flags.petSystemCenter,false);assert.equal(f.model.petSystemCenter,false);f.model={...f.model,language:'pt-BR',openOnHover:true};f.saves[0].reject(Error('private failure'));await pending;assert.equal(f.prefs.status,'failed');assert.deepEqual(f.prefs.flags,{petControlBar:true,petSystemCenter:false});assert.equal(f.model.language,'pt-BR');assert.equal(f.model.openOnHover,true);assert.equal(f.saves.length,1);
});
test('a newer authoritative settings event wins over an older delayed boot snapshot',async()=>{
 const f=fixture();f.holdRead=true;const pending=f.prefs.change('petControlBar');await flush();assert.equal(f.reads.length,1);f.prefs.observe({...DEFAULT_SETTINGS,petControlBar:false,petSystemCenter:true});f.reads[0].resolve({...DEFAULT_SETTINGS,petControlBar:true,petSystemCenter:false});await pending;assert.deepEqual(f.prefs.flags,{petControlBar:false,petSystemCenter:true});assert.equal(f.prefs.status,'changed');assert.equal(f.model.petSystemCenter,true);
});
test('unknown readback preserves last values and requires a deliberate refresh, never a retry write',async()=>{
 const f=fixture({petControlBar:true,petSystemCenter:true});f.failRead=true;await f.prefs.change('petControlBar');assert.equal(f.prefs.status,'unconfirmed');assert.equal(f.prefs.known,false);assert.deepEqual(f.prefs.flags,{petControlBar:true,petSystemCenter:true});await f.prefs.change('petSystemCenter');assert.equal(f.saves.length,1);f.failRead=false;await f.prefs.refresh();assert.equal(f.prefs.known,true);assert.deepEqual(f.prefs.flags,{petControlBar:false,petSystemCenter:true});assert.equal(f.saves.length,1);
});
test('an authoritative event arriving during a failed read remains the current observation',async()=>{
 const f=fixture();f.holdRead=true;const pending=f.prefs.change('petControlBar');await flush();f.prefs.observe({...DEFAULT_SETTINGS,petControlBar:true,petSystemCenter:true});f.reads[0].reject(Error('late private failure'));await pending;assert.equal(f.prefs.known,true);assert.deepEqual(f.prefs.flags,{petControlBar:true,petSystemCenter:true});assert.equal(f.prefs.status,'saved');
});
test('duplicate or other-flag clicks while this small block is pending cannot dispatch stale parallel snapshots',async()=>{
 const f=fixture();f.holdSave=true;const pending=f.prefs.change('petControlBar');await f.prefs.change('petControlBar');await f.prefs.change('petSystemCenter');assert.equal(f.saves.length,1);f.server={...f.model,...f.saves[0].value};f.saves[0].resolve();await pending;assert.equal(f.prefs.busy,false);assert.deepEqual(f.prefs.flags,{petControlBar:true,petSystemCenter:false});
});
test('an ACK does not force the requested flag over a different value observed by readback',async()=>{
 const f=fixture();f.holdSave=true;const pending=f.prefs.change('petControlBar');f.server={...f.model,petControlBar:false,petSystemCenter:true};f.saves[0].resolve();await pending;assert.equal(f.prefs.status,'changed');assert.deepEqual(f.prefs.flags,{petControlBar:false,petSystemCenter:true});assert.equal(f.saves.length,1);
});
test('malformed readback cannot turn unknown fields into false or overwrite a newer event',async()=>{
 const f=fixture({petControlBar:true,petSystemCenter:true});f.holdRead=true;const pending=f.prefs.refresh();f.reads[0].resolve({petControlBar:false});await pending;assert.equal(f.prefs.known,false);assert.deepEqual(f.prefs.flags,{petControlBar:true,petSystemCenter:true});f.prefs.observe({...DEFAULT_SETTINGS,petControlBar:true,petSystemCenter:false});assert.equal(f.prefs.known,true);assert.equal(f.prefs.status,'saved');
});

test('unobserved defaults are not an initial confirmation and cannot dispatch a patch',async()=>{
 let writes=0;const preferences=new IslandAvailabilityPreferences({initial:null,save:async()=>{writes++;},read:async()=>({...DEFAULT_SETTINGS,petControlBar:true}),adopt:()=>{},changed:()=>{}});assert.equal(preferences.known,false);assert.equal(preferences.status,'unconfirmed');await preferences.change('petControlBar');assert.equal(writes,0);await preferences.refresh();assert.equal(preferences.known,true);assert.equal(preferences.flags.petControlBar,true);assert.equal(writes,0);
});
