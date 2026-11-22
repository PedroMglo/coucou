import test from 'node:test';import assert from 'node:assert/strict';
import {settingsIoStatus} from '../src/core/settings-io.ts';
import {buildSettingsIoStatus} from '../src/settings/io-status.ts';
import {installFakeDom} from './fakedom.mjs';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
installFakeDom();
const ready={schemaVersion:1,phase:'ready',newEffectsAllowed:true,message:''};
const blocked={schemaVersion:1,phase:'changed',newEffectsAllowed:false,message:'Revisão necessária'};
test('settings IO decoder refuses contradictory readiness and unknown formats',()=>{
 for(const value of [null,[],{...ready,schemaVersion:2},{...ready,phase:'future'},{...blocked,newEffectsAllowed:true},{...ready,newEffectsAllowed:false},{...blocked,message:'x'.repeat(1025)}])assert.throws(()=>settingsIoStatus(value));
 assert.deepEqual(settingsIoStatus({...blocked,fileRevision:'forged',grant:true}),blocked);
});
test('settings IO bridge only queries native status and never supplies authority or settings',async()=>{
 const previous=internals.invoke;const sent=[];internals.invoke=async(command,args)=>{sent.push({command,args});return blocked;};
 try{assert.deepEqual(await Bridge.settingsIoStatus(),blocked);assert.equal(sent.length,1);assert.equal(sent[0].command,'settings_io_status');assert.ok(sent[0].args==null||Object.keys(sent[0].args).length===0);}finally{internals.invoke=previous;}
});
test('settings IO refresh displays denial as text and only issues reads',async()=>{
 let reads=0;let state={...blocked,message:'<script>untrusted diagnostic</script>'};const view=buildSettingsIoStatus(async()=>{reads++;return state;});
 await view.refresh();assert.equal(view.el.hidden,false);assert.match(view.el.textContent,/<script>/);assert.equal(view.el.dataset.settingsIo,'changed');
 state=ready;view.el.find('button')[0].fire('click');for(let i=0;i<8;i++)await Promise.resolve();assert.equal(reads,2);assert.equal(view.el.hidden,true);
});
test('settings IO newer refresh wins over delayed ready and unavailable status stays visible',async()=>{
 let release;let reads=0;const view=buildSettingsIoStatus(()=>++reads===1?new Promise(r=>{release=r;}):Promise.resolve(blocked));
 const old=view.refresh();await view.refresh();release(ready);await old;assert.equal(view.el.dataset.settingsIo,'changed');assert.equal(view.el.hidden,false);assert.equal(view.el.find('button')[0].disabled,false);
 const failed=buildSettingsIoStatus(async()=>{throw Error('estado indisponível');});await failed.refresh();assert.equal(failed.el.hidden,false);assert.equal(failed.el.dataset.settingsIo,'unavailable');assert.match(failed.el.textContent,/indisponível/);
});
