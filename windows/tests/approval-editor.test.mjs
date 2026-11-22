import {beforeEach,test} from 'node:test';
import assert from 'node:assert/strict';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {buildMail} from '../src/views/upload.ts';
import {Bridge} from '../src/core/bridge.ts';
import {installFakeDom} from './fakedom.mjs';
installFakeDom();
const request={requestId:'synthetic',sessionId:'qa',pillId:'agent_codex',tool:'Read',command:'synthetic'};
const walk=el=>[el,...(el.children??[]).flatMap(walk)];
beforeEach(()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true};State.tasks=[];State.loadIntegrationTasks();
 for(const [id,name]of [['agent_codex','Codex'],['ai_ollama','Ollama'],['agent_antigravity','Gravity']])State.upsertExternalAgent(id,name,'#2dd4bf');
 State.pendingApproval=null;State.focusBeforeApproval=null;State.focusId='ai_ollama';State.mode='expanded';State.view='mail';
 State.setFileContext('ollama',{name:'Sintético.pdf',path:'/tmp/synthetic.pdf',kind:'file'});
});
test('an unrelated decision returns to the interrupted email owner and preserves its attachment',()=>{
 const file=State.fileFor('ollama');State.beginApproval(request);State.view='approval';
 assert.equal(State.chatProvider,'codex');assert.equal(State.endApproval(),'mail');
 assert.equal(State.focusId,'ai_ollama');assert.equal(State.chatProvider,'ollama');assert.equal(State.droppedFile,file);
});
test('the editor captured before collapse survives and an explicit pet switch takes priority',()=>{
 State.mode='hidden';State.beginApproval(request,'mail');State.view='approval';assert.equal(State.endApproval(),'mail');
 State.focusId='ai_ollama';State.beginApproval(request,'mail');State.setFocus('agent_antigravity');
 assert.equal(State.endApproval(),undefined);assert.equal(State.focusId,'agent_antigravity');
});
test('ordinary companion cards return to the selected pet after a background decision',()=>{
 State.view='overview';State.beginApproval(request);State.view='approval';State.endApproval();assert.equal(State.focusId,'ai_ollama');
});
test('email fields stay with their owner while a different pet needs a decision; stale clicks explain instead of doing nothing',async()=>{
 const view=buildMail({setView(){}});view.sync();const nodes=walk(view.el),to=nodes.find(n=>n.getAttribute('aria-label')==='Destinatário do email');to.value='synthetic@example.invalid';
 const send=nodes.find(n=>n.tagName==='BUTTON'&&n.textContent==='Abrir rascunho com anexo');
 const original=Bridge.petMailDraft,calls=[];Bridge.petMailDraft=async(...args)=>{calls.push(args);return 'Rascunho preparado';};
 try{
  State.beginApproval(request);State.view='approval';view.sync();assert.equal(to.value,'synthetic@example.invalid');
  await send.listeners.get('click')[0]({});assert.equal(calls.length,0);assert.match(view.el.textContent,/Volta ao pet/);
  assert.equal(State.endApproval(),'mail');State.view='mail';view.sync();await send.listeners.get('click')[0]({});
  assert.deepEqual(calls[0].slice(0,3),['ollama','/tmp/synthetic.pdf','synthetic@example.invalid']);
 }finally{Bridge.petMailDraft=original;}
});
