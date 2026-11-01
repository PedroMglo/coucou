import test from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
test('autostart bridge sends only one explicit field intention, never the stale settings snapshot or a claimed window',async()=>{
 const previous=internals.invoke;let sent;internals.invoke=async(command,args)=>{sent={command,args};return {schemaVersion:1,enabled:true,change:'changed',registration:'executorAccepted'};};try{const reply=await Bridge.autostartSettingsPatch({enabled:true,autostart:false,soundVolume:0.1,window:'settings'});assert.deepEqual(sent,{command:'pet_autostart_settings_patch',args:{patch:{enabled:true}}});assert.equal(reply.registration,'executorAccepted');}finally{internals.invoke=previous;}
});
test('autostart bridge captures the original boolean before asynchronous transport waits',async()=>{
 const previous=internals.invoke;let finish,sent;const hold=new Promise(r=>{finish=r;});internals.invoke=async(command,args)=>{await hold;sent={command,args};return {schemaVersion:1,enabled:false,change:'unchanged',registration:'notAttempted'};};try{const patch={enabled:false};const pending=Bridge.autostartSettingsPatch(patch);patch.enabled=true;finish();await pending;assert.deepEqual(sent.args,{patch:{enabled:false}});}finally{internals.invoke=previous;}
});
