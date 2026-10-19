import test from 'node:test';import assert from 'node:assert/strict';import {petOpenView} from '../src/mochi/pet-navigation.ts';
test('reopening a pet asking for help leads to its decision rather than home',()=>{
 assert.equal(petOpenView('approval'),'approval');assert.equal(petOpenView('question'),'question');assert.equal(petOpenView('approval','overview'),'approval');assert.equal(petOpenView('question','overview'),'question');
 assert.equal(petOpenView('working'),'overview');assert.equal(petOpenView('idle'),'overview');
});
test('native chat decisions lead to the chat and explicit document routes remain explicit',()=>{
 assert.equal(petOpenView('approval',undefined,true),'prompt');assert.equal(petOpenView('question','overview',true),'prompt');
 assert.equal(petOpenView('approval','choose'),'choose');assert.equal(petOpenView('question','upload'),'upload');
});
