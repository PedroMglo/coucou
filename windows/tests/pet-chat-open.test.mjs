import {test} from 'node:test';
import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';
import {internals,calls,sent} from './tauri.mjs';
import {registerCardPresentation} from '../src/core/card-presentation.ts';
test('the original card is notified only after the floating conversation has opened',async()=>{
 const original=internals.invoke;let finish;calls.length=0;
 internals.invoke=(command,args)=>command==='pet_chat_open'?new Promise(resolve=>finish=resolve):original(command,args);
 try{
  const opening=Bridge.petChatOpen('ai_ollama');
  assert.equal(sent('plugin:event|emit_to').length,0);
  finish(true);assert.equal(await opening,true);
  const event=sent('plugin:event|emit_to')[0];assert.equal(event.event,'pet-chat-presented');assert.deepEqual(event.payload,{pet:'ai_ollama'});
 }finally{internals.invoke=original;}
});
test('a failed open keeps the original card and propagates its explanation',async()=>{
 const original=internals.invoke;calls.length=0;
 internals.invoke=(command,args)=>command==='pet_chat_open'?Promise.reject(Error('Não foi possível abrir')):original(command,args);
 try{await assert.rejects(Bridge.petChatOpen('ai_ollama'),/abrir/);assert.equal(sent('plugin:event|emit_to').length,0);}
 finally{internals.invoke=original;}
});

test('reviewing a decision passes the exact native target without substituting the current conversation',async()=>{
 const original=internals.invoke;calls.length=0;internals.invoke=(command,args)=>original(command,args);
 try{const target={petId:'integration_github',provider:'integration_github',origin:'mochi',sessionId:'exact-owned',conversationId:'exact-owned'};
 await Bridge.petChatOpen(target.petId,undefined,false,target);assert.deepEqual(sent('pet_chat_open')[0].request.target,target);
 assert.equal(sent('pet_chat_open')[0].request.pet,'integration_github');assert.equal(sent('pet_chat_open')[0].request.session,undefined);
 assert.deepEqual(sent('chat_send'),[]);assert.deepEqual(sent('approval_decision'),[]);
 }finally{internals.invoke=original;}
});

test('a withdrawn or superseded opening does not claim presentation or fold a card',async()=>{
 const original=internals.invoke;calls.length=0;
 internals.invoke=async(command,args)=>command==='pet_chat_open'?false:original(command,args);
 try{assert.equal(await Bridge.petChatOpen('ai_ollama'),false);assert.deepEqual(sent('plugin:event|emit_to'),[]);}
 finally{internals.invoke=original;}
});

test('Help carries its request ID and immutable canonical target into the actual presentation receipt',async()=>{
 const original=internals.invoke;calls.length=0;
 internals.invoke=async(command,args)=>{const result=await original(command,args);return command==='pet_chat_open'?true:result;};
 try{
  const target={petId:'integration_github',provider:'integration_github',origin:'mochi',sessionId:'github-cid',conversationId:'github-cid'};
  assert.equal(await Bridge.petChatOpen(target.petId,undefined,false,target,'exact-request'),true);
  assert.equal(sent('pet_chat_open')[0].request.requestId,'exact-request');
  assert.deepEqual(sent('plugin:event|emit_to')[0].payload,{pet:target.petId,target,requestId:'exact-request'});
 }finally{internals.invoke=original;}
});

test('a delayed presentation receipt retains the initiating context instead of adopting a newer card',async()=>{
 const original=internals.invoke;let finish,context='card-A';calls.length=0;
 const unregister=registerCardPresentation(()=>context);
 internals.invoke=(command,args)=>command==='pet_chat_open'?new Promise(resolve=>finish=resolve):original(command,args);
 try{const opening=Bridge.petChatOpen('ai_ollama');context='card-B';finish(true);await opening;assert.equal(sent('plugin:event|emit_to')[0].payload.sourceContext,'card-A');}
 finally{unregister();internals.invoke=original;}
});
