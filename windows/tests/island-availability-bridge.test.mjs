import {test} from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
test('island preference bridge sends only supplied flag fields to the dedicated patch and propagates refusal',async()=>{
 const saved=internals.invoke,calls=[];let fail=false;
 try{internals.invoke=async(command,args)=>{calls.push({command,args});if(fail)throw Error('Janela não autorizada');return {petControlBar:true,petSystemCenter:false};};
 assert.deepEqual(await Bridge.petCenterSettingsPatch({petControlBar:true,companionMode:true}),{petControlBar:true,petSystemCenter:false});assert.deepEqual(calls[0],{command:'pet_center_settings_patch',args:{request:{petControlBar:true}}});
 await Bridge.petCenterSettingsPatch({petSystemCenter:false});assert.deepEqual(calls[1].args,{request:{petSystemCenter:false}});fail=true;await assert.rejects(Bridge.petCenterSettingsPatch({petControlBar:false}),/não autorizada/);assert.ok(calls.every(c=>c.command==='pet_center_settings_patch'));
 }finally{internals.invoke=saved;}
});
