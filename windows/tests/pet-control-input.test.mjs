import {test} from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {calls} from './tauri.mjs';
const rect={x:10,y:0,w:640,h:490};
test('legacy callers explicitly default to no keyboard request',async()=>{await Bridge.petControlInput(rect,true);assert.deepEqual(calls.at(-1),['pet_control_input',{rect,expanded:true,keyboard:false}]);});
test('an intentional field or control request carries keyboard independently of geometry',async()=>{await Bridge.petControlInput(rect,true,true);assert.deepEqual(calls.at(-1),['pet_control_input',{rect,expanded:true,keyboard:true}]);});
test('leaving clears keyboard even when the visible shape is still animating',async()=>{await Bridge.petControlInput(rect,false,false);assert.deepEqual(calls.at(-1),['pet_control_input',{rect,expanded:false,keyboard:false}]);});

test('unknown/automatic companion opens default to automatic lifetime',async()=>{await Bridge.companionsOpen('ai_ollama',100,20,'overview');assert.deepEqual(calls.at(-1),['companions_open',{pillId:'ai_ollama',x:100,y:20,view:'overview',manual:false}]);});
test('a real companion hover/click can explicitly mark its own manual lifetime',async()=>{await Bridge.companionsOpen('integration_spotify',100,20,'prompt',true);assert.deepEqual(calls.at(-1),['companions_open',{pillId:'integration_spotify',x:100,y:20,view:'prompt',manual:true}]);});
