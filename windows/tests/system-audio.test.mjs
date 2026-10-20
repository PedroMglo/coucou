import {test} from 'node:test';
import assert from 'node:assert/strict';
import {AudioController,systemDeviceSnapshot} from '../src/core/system-audio.ts';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
const snap=(observedAt=100,extra={})=>({available:true,observedAt,outputs:[],inputs:[],playback:[],recording:[],cards:[],error:null,...extra});
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return {promise,resolve,reject};};
test('failed or unavailable reads retain the last values as stale and block controls',async()=>{
 let reading=snap(),writes=0;const c=new AudioController({snapshot:async()=>reading,control:async()=>{writes++;throw Error('must not run');}});
 await c.refresh(true);assert.equal(c.stale,false);assert.equal(c.snapshot.observedAt,100);
 reading=snap(null,{available:false,error:'Offline'});await c.refresh(true);assert.equal(c.stale,true);assert.equal(c.snapshot.observedAt,100);assert.equal(c.error,'Offline');
 assert.equal(await c.control({targetKey:'gone',action:'mute',muted:true}),false);assert.equal(writes,0);
});
test('a delayed poll cannot replace the receipt of a newer explicit command',async()=>{
 const slow=deferred();let calls=0;const c=new AudioController({snapshot:()=>++calls===1?Promise.resolve(snap(100)):slow.promise,control:async()=>({snapshot:snap(300),confirmed:true,warning:null})});
 await c.refresh(true);const poll=c.refresh(true);assert.equal(c.loading,true);
 assert.equal(await c.control({targetKey:'x',action:'volume',value:40}),true);assert.equal(c.snapshot.observedAt,300);slow.resolve(snap(200));await poll;assert.equal(c.snapshot.observedAt,300);assert.equal(c.pending,false);assert.equal(c.loading,false);
});
test('pending controls are not optimistic and a duplicate is not sent',async()=>{
 const pending=deferred();let writes=0;const original=snap(100,{outputs:[{key:'x',volume:50}]});const c=new AudioController({snapshot:async()=>original,control:()=>{writes++;return pending.promise;}});
 await c.refresh(true);const first=c.control({targetKey:'x',action:'volume',value:80});assert.equal(c.pending,true);assert.equal(c.snapshot.outputs[0].volume,50);
 assert.equal(await c.control({targetKey:'x',action:'volume',value:90}),false);assert.equal(writes,1);
 pending.resolve({snapshot:snap(200,{outputs:[{key:'x',volume:80}]}),confirmed:true,warning:null});assert.equal(await first,true);assert.equal(c.snapshot.outputs[0].volume,80);
});
test('an unconfirmed native receipt stays explicit and uses observed volume',async()=>{
 const c=new AudioController({snapshot:async()=>snap(100),control:async()=>({snapshot:snap(200,{outputs:[{key:'x',volume:50}]}),confirmed:false,warning:null})});await c.refresh(true);
 assert.equal(await c.control({targetKey:'x',action:'volume',value:80}),false);assert.match(c.warning,/não confirmou/);assert.equal(c.snapshot.outputs[0].volume,50);assert.equal(c.stale,false);
});
test('rejected controls retain observed values and require a fresh read before retry',async()=>{
 let failed=true,writes=0;const c=new AudioController({snapshot:async()=>snap(100),control:async()=>{writes++;if(failed)throw Error('Device disconnected');return {snapshot:snap(300),confirmed:true,warning:null};}});await c.refresh(true);
 assert.equal(await c.control({targetKey:'x',action:'mute',muted:true}),false);assert.equal(c.stale,true);assert.equal(c.snapshot.observedAt,100);assert.match(c.error,/disconnected/);
 failed=false;assert.equal(await c.control({targetKey:'x',action:'mute',muted:true}),false);assert.equal(writes,1);await c.refresh(true);assert.equal(await c.control({targetKey:'x',action:'mute',muted:true}),true);assert.equal(writes,2);
});
test('backwards timestamps never replace a newer observation',async()=>{
 let value=snap(300);const c=new AudioController({snapshot:async()=>value,control:async()=>{throw Error();}});await c.refresh(true);value=snap(100);await c.refresh(true);assert.equal(c.snapshot.observedAt,300);
});
test('System projection preserves device-only scope without fabricating observed stream zeros',()=>{
 const p=systemDeviceSnapshot({...snap(100),scope:'systemDevices',playback:[{private:'application'}],recording:[{private:'recording'}]});
 assert.equal(p.scope,'systemDevices');assert.equal('playback' in p,false);assert.equal('recording' in p,false);assert.doesNotMatch(JSON.stringify(p),/private|application/);
 for(const input of [{...p,scope:'spoty'}, {...p,observedAt:NaN},{...p,observedAt:8_640_000_000_000_001},{...p,outputs:[{kind:'playback'}]}])assert.throws(()=>systemDeviceSnapshot(input));
});
test('hiding invalidates an old poll, retains dated values and requires a new read before control',async()=>{
 const slow=deferred();let reads=0,writes=0;const c=new AudioController({snapshot:()=>++reads===1?Promise.resolve(snap(100)):reads===2?slow.promise:Promise.resolve(snap(300)),control:async()=>{writes++;return {snapshot:snap(400),confirmed:true,warning:null};}});
 await c.refresh(true);const old=c.refresh(true);c.invalidate();slow.resolve(snap(200));await old;assert.equal(c.snapshot.observedAt,100);assert.equal(c.stale,true);assert.equal(await c.control({targetKey:'x',action:'mute',muted:true}),false);assert.equal(writes,0);await c.refresh(true);assert.equal(c.snapshot.observedAt,300);assert.equal(c.stale,false);
});
test('a command accepted before hide is not rolled back and its late receipt does not republish hidden state',async()=>{
 const slow=deferred();let writes=0;const c=new AudioController({snapshot:async()=>snap(100),control:()=>{writes++;return slow.promise;}});await c.refresh(true);const operation=c.control({targetKey:'x',action:'mute',muted:true});c.invalidate();slow.resolve({snapshot:snap(200),confirmed:true,warning:null});assert.equal(await operation,false);assert.equal(writes,1);assert.equal(c.snapshot.observedAt,100);assert.equal(c.pending,false);assert.equal(c.stale,true);
});
test('System and Spoty controllers keep independent pending and observed state',async()=>{
 const slow=deferred();const spoty=new AudioController({snapshot:async()=>snap(100),control:()=>slow.promise});const system=new AudioController({snapshot:async()=>systemDeviceSnapshot({...snap(200),scope:'systemDevices'}),control:async()=>({snapshot:systemDeviceSnapshot({...snap(300),scope:'systemDevices'}),confirmed:true,warning:null})});await spoty.refresh(true);await system.refresh(true);const operation=spoty.control({targetKey:'app',action:'mute',muted:true});assert.equal(spoty.pending,true);assert.equal(system.pending,false);assert.equal(await system.control({targetKey:'output',action:'mute',muted:true}),true);assert.equal(system.snapshot.observedAt,300);assert.equal(spoty.snapshot.observedAt,100);slow.resolve({snapshot:snap(400),confirmed:true,warning:null});await operation;assert.equal(system.snapshot.observedAt,300);
});
test('System bridge uses only its independently authorised commands, with no payload on reads or Spoty fallback',async()=>{
 const original=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push([command,args]);return null;};const request={targetKey:'opaque-device',action:'volume',value:42};await Bridge.petSystemAudioSnapshot();await Bridge.petSystemAudioControl(request);assert.deepEqual(calls,[['pet_system_audio_snapshot',{}],['pet_system_audio_control',{request}]]);internals.invoke=async()=>{throw Error('System disabled');};await assert.rejects(Bridge.petSystemAudioSnapshot(),/System disabled/);await assert.rejects(Bridge.petSystemAudioControl(request),/System disabled/);}finally{internals.invoke=original;}
});
