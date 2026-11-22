import test from 'node:test';import assert from 'node:assert/strict';import{installFakeDom}from'./fakedom.mjs';import{Bridge}from'../src/core/bridge.ts';import{buildRuntimeReceivedWork}from'../src/views/runtime-received-work.ts';import{internals}from'./tauri.mjs';
installFakeDom();const flush=async()=>{for(let i=0;i<15;i++)await Promise.resolve();};
const source={reference:{decisionId:'source-decision',messageId:'message',scopeHash:'scope'},senderPetId:'ai_lmstudio',senderGoalId:'source-goal',recipientPetId:'ai_ollama',receiptHash:'receipt',envelopeHash:'envelope',receivedAt:1,deadlineAt:999999};
const goal=()=>({id:'own-goal',revision:'r1',spec:{owner:{petId:'ai_ollama'},receivedFrom:{body:{source}}}});
const reply=(state='proposed')=>({schemaVersion:1,scope:'ownGoalLinkedToReceivedRequest',petId:'ai_ollama',goalId:'own-goal',goalRevision:'r1',goalHash:'goal-hash',source:structuredClone(source),originHash:'origin-hash',observedAt:2,state,admission:null,ownerState:null,attempts:{reserved:0,started:0,settled:0,uncertain:0,unknownOutcomes:0},evidenceComplete:true,criteriaVerified:false,requestWithinDeadline:true,executionAuthorized:false});
const button=v=>v.el.find('button')[0];
test('own work status is explicit read only; update has no IO and a new revision never redirects captured query',async()=>{
 let resolve;const sent=[],v=buildRuntimeReceivedWork({petRuntimeReceivedWork:async i=>{sent.push(structuredClone(i));return new Promise(r=>resolve=r);}});v.update(goal());v.update(goal());assert.equal(sent.length,0);button(v).fire('click');v.update({...goal(),revision:'r2'});resolve(reply('sessionReserved'));await flush();assert.deepEqual(sent,[{petId:'ai_ollama',goalId:'own-goal',revision:'r1',source:source.reference}]);assert.match(v.el.textContent,/sem prova de tentativa em curso/);assert.match(v.el.textContent,/revisão anterior/);v.dispose();
});
test('queued, inflight, saved result, verified completion and historical work have separate labels',async()=>{
 for(const[state,label]of[['acceptedQueued',/autorizado na fila/],['attemptInFlight',/Tentativa própria em curso/],['awaitingVerification',/conclusão ainda por verificar/],['completed',/Conclusão própria verificada pelo runtime/],['historical',/estado posterior desconhecido/]]){const v=buildRuntimeReceivedWork({petRuntimeReceivedWork:async()=>reply(state)});v.update(goal());button(v).fire('click');await flush();assert.match(v.el.textContent,label);assert.equal(v.el.find('button').length,1);v.dispose();}
});
test('forged status scope and permission are refused and stale response cannot follow another goal',async()=>{
 for(const mutate of[v=>v.executionAuthorized=true,v=>v.source.reference.messageId='other',v=>v.goalId='other',v=>v.source.receiptHash='forged']){const v=buildRuntimeReceivedWork({petRuntimeReceivedWork:async()=>{const r=reply('completed');mutate(r);return r;}});v.update(goal());button(v).fire('click');await flush();assert.match(v.el.textContent,/Consulta indisponível/);assert.doesNotMatch(v.el.textContent,/Conclusão própria verificada pelo runtime/);v.dispose();}
 let resolve;const v=buildRuntimeReceivedWork({petRuntimeReceivedWork:async()=>new Promise(r=>resolve=r)});v.update(goal());button(v).fire('click');v.update({...goal(),id:'different'});resolve(reply('completed'));await flush();assert.doesNotMatch(v.el.textContent,/Conclusão própria verificada pelo runtime/);v.dispose();
});
test('bridge sends only own original references, never a caller, authority, observed state or prompt',async()=>{
 const saved=internals.invoke,calls=[];internals.invoke=async(command,args)=>{calls.push({command,args:structuredClone(args)});return reply();};try{await Bridge.petRuntimeReceivedWork({petId:'ai_ollama',goalId:'own-goal',revision:'r1',source:{...source.reference,caller:'providerModel'},authority:'/other',state:'completed',prompt:'not context'});assert.deepEqual(calls,[{command:'pet_runtime_received_work',args:{input:{petId:'ai_ollama',goalId:'own-goal',revision:'r1',source:source.reference}}}]);}finally{internals.invoke=saved;}
});

test('historical or incomplete counts stay unknown rather than displaying zero attempts',async()=>{
 for(const state of ['historical','unavailable']){const v=buildRuntimeReceivedWork({petRuntimeReceivedWork:async()=>({...reply(state),evidenceComplete:false,attempts:null})});v.update(goal());button(v).fire('click');await flush();assert.match(v.el.textContent,/Tentativas atuais desconhecidas/);assert.doesNotMatch(v.el.textContent,/0 reservadas|0 liquidadas|0 resultados desconhecidos/);v.dispose();}
});
