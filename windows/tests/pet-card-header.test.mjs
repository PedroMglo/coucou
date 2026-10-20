import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {buildPetCardHeader} from '../src/views/pet-card-header.ts';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {calls,internals} from './tauri.mjs';
installFakeDom();
test('contextual navigation uses the exact pet, guards repeats and retains the card on native failure',async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:false};State.petCardsOnly=true;State.pendingApproval=null;
 State.mode='expanded';
 State.tasks=[{id:'ai_ollama',name:'Ollama',color:'#eab308',state:'idle',steps:[],stepIndex:0}];State.focusId='ai_ollama';
 let closed=0,resolve,reject;
 const gate=()=>new Promise((a,b)=>{resolve=a;reject=b;});let pending=gate();
 const invoke=internals.invoke;
 internals.invoke=async(cmd,args)=>{calls.push([cmd,args]);if(cmd==='pet_panel_open')return pending;return null;};
 const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
 try{
  const header=buildPetCardHeader({collapse(){closed++;},setView(){}});header.sync();
  const buttons=header.el.find('button'),sessions=buttons.find(b=>b.getAttribute('aria-label')==='Sessões deste pet');
  assert.equal(buttons.some(b=>b.getAttribute('aria-label')==='Overview'),false);
  sessions.fire('click');sessions.fire('click');await flush();
  assert.equal(calls.filter(([cmd])=>cmd==='pet_panel_open').length,1);
  assert.equal(calls.find(([cmd])=>cmd==='pet_panel_open')[1].pet,'ai_ollama');
  reject(new Error('painel indisponível'));await flush();assert.equal(closed,0);assert.match(header.el.textContent,/painel indisponível/);assert.equal(sessions.disabled,false);
  pending=gate();sessions.fire('click');resolve(null);await flush();assert.equal(closed,1);
  assert.equal(calls.filter(([cmd])=>cmd==='pet_panel_open').length,2);
 }finally{internals.invoke=invoke;State.petCardsOnly=false;}
});

test('false or a late opening for a changed card never folds the current owner',async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:false};State.petCardsOnly=true;State.mode='expanded';State.view='overview';
 State.tasks=[{id:'ai_ollama',name:'Ollama',color:'#eab308',state:'idle',steps:[],stepIndex:0}];State.focusId='ai_ollama';
 const invoke=internals.invoke;let finish,closed=0;
 internals.invoke=(cmd,args)=>cmd==='pet_chat_open'?new Promise(resolve=>finish=resolve):invoke(cmd,args);
 const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
 try{
  const header=buildPetCardHeader({collapse(){closed++;},setView(){}});header.sync();
  const chat=header.el.find('button').find(b=>b.getAttribute('aria-label')==='Conversar com este pet');
  chat.fire('click');finish(false);await flush();assert.equal(closed,0);
  chat.fire('click');State.lastActivity++;finish(true);await flush();assert.equal(closed,0);
 }finally{internals.invoke=invoke;State.petCardsOnly=false;}
});
