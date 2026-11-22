import {test} from 'node:test';import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';import {Bridge} from '../src/core/bridge.ts';import {secretControls} from '../src/settings/secret-controls.ts';
installFakeDom();const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
test('empty Save cannot clear a stored key; only explicit Remove clears the exact key',async()=>{
 const set=Bridge.secretSet,clear=Bridge.secretClear,calls=[];
 try{Bridge.secretSet=async(...args)=>calls.push(['set',...args]);Bridge.secretClear=async key=>calls.push(['clear',key]);const field=secretControls('fixture-key',true,'placeholder',()=>{});
 assert.equal(field.save.disabled,true);field.save.onclick();await flush();assert.deepEqual(calls,[]);assert.equal(field.remove.hidden,false);
 field.remove.onclick();await flush();assert.deepEqual(calls,[['clear','fixture-key']]);assert.equal(field.remove.hidden,true);assert.match(field.feedback.textContent,/Removido/);
 }finally{Bridge.secretSet=set;Bridge.secretClear=clear;}
});
test('write failure keeps the draft and reports a readable error; explicit retry saves it once',async()=>{
 const original=Bridge.secretSet;let fail=true;const sent=[];
 try{Bridge.secretSet=async(...args)=>{sent.push(args);if(fail)throw Error('synthetic keystore unavailable');};const field=secretControls('fixture-provider',false,'placeholder',()=>{});field.input.value='synthetic-key';field.input.fire('input');assert.equal(field.save.disabled,false);
 field.save.onclick();assert.equal(field.save.disabled,true);await flush();assert.equal(field.input.value,'synthetic-key');assert.match(field.feedback.textContent,/keystore unavailable/);
 fail=false;field.save.onclick();await flush();assert.equal(field.input.value,'');assert.equal(field.remove.hidden,false);assert.deepEqual(sent,[['fixture-provider','synthetic-key'],['fixture-provider','synthetic-key']]);
 }finally{Bridge.secretSet=original;}
});
