import {test,beforeEach} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {documentActions} from '../src/mochi/documents.ts';
import {providerForPill} from '../src/core/providers.ts';
import {installFakeDom} from './fakedom.mjs';
import {Bridge} from '../src/core/bridge.ts';
import {PetPreferences} from '../src/mochi/pet-preferences.ts';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {buildChoose,buildMail} from '../src/views/upload.ts';
import {buildPrompt} from '../src/views/chat.ts';
const defaults=JSON.parse(readFileSync(new URL('../src-tauri/pets.default.json',import.meta.url),'utf8'));
installFakeDom();
const walk=el=>[el,...(el.children??[]).flatMap(walk)];
beforeEach(()=>{PetPreferences.update(structuredClone(defaults));State.settings={...DEFAULT_SETTINGS,chatProvider:'ollama'};State.pendingApproval=null;State.focusId='ai_ollama';State.view='prompt';State.setFileContext('ollama',{name:'Teste.pdf',path:'/private/ollama/Teste.pdf',kind:'file'});});
test('document actions remain reachable from the compact chat and preserve a waiting decision',()=>{
 let opened=0;State.tasks=[];State.pendingApproval=null;State.view='prompt';
 const view=buildPrompt(()=>{},undefined,()=>opened++);view.sync();
 const button=walk(view.el).find(n=>n.tagName==='BUTTON'&&n.textContent==='Ações do documento');assert.ok(button);button.fire('click');assert.equal(opened,1);
 State.pendingApproval={pillId:'agent_codex',requestId:'decision'};view.sync();assert.equal(button.disabled,false);button.fire('click');assert.equal(opened,2);
 assert.equal(State.pendingApproval.requestId,'decision');
 State.pendingApproval={pillId:'ai_ollama',requestId:'own-decision'};view.sync();assert.equal(button.disabled,true);button.fire('click');assert.equal(opened,2);
 State.pendingApproval=null;
});
test('every pet advertises only implemented document actions; only Ollama has email by default',()=>{
 for(const [id,p]of Object.entries(defaults.pets)){
  const file=documentActions(defaults,id,'file'),folder=documentActions(defaults,id,'folder');
  assert.equal(file.email,id==='ai_ollama',id);assert.equal(folder.email,false,id);
  assert.equal(file.analysis,!!providerForPill(id)&&p.permissions.documents,id);
  if(!providerForPill(id))assert.deepEqual(folder,{analysis:false,email:false},id);
 }
 assert.deepEqual(documentActions(null,'agent_codex'),{analysis:false,email:false});
 const c=structuredClone(defaults);c.pets.ai_ollama.permissions.documents=false;
 assert.deepEqual(documentActions(c,'ai_ollama'),{analysis:false,email:true});assert.deepEqual(documentActions(c,'ai_ollama','folder'),{analysis:false,email:false});
});
test('the document chooser switches plugins per owner and kind without starting an inference',()=>{
 const views=[],view=buildChoose({setView:x=>views.push(x),cancelDrop(){}});view.sync();
 const buttons=walk(view.el).filter(b=>b.tagName==='BUTTON'),email=buttons.find(b=>b.textContent==='Preparar email'),analysis=buttons.find(b=>b.textContent==='Analisar / conversar');
 assert.equal(email.hidden,false);assert.equal(analysis.hidden,false);email.fire('click');assert.deepEqual(views,['mail']);
 State.settings={...State.settings,chatProvider:'codex'};State.setFileContext('codex',{name:'docs',path:'/private/codex/docs',kind:'folder',entries:2});view.sync();assert.equal(email.hidden,true);assert.equal(analysis.hidden,false);
 assert.equal(State.promptContext.kind,'file');assert.equal(State.droppedFile.kind,'folder');
});
test('an email draft uses only the exact owner and delivered file, including errors',async()=>{
 const original=Bridge.petMailDraft,calls=[];
 try{Bridge.petMailDraft=async(...args)=>{calls.push(args);throw Error('Portal unavailable');};
 const view=buildMail({setView(){}});view.sync();
 for(const input of walk(view.el).filter(el=>['INPUT','TEXTAREA'].includes(el.tagName)))input.value='';const button=walk(view.el).find(b=>b.textContent==='Abrir rascunho com anexo'&&b.tagName==='BUTTON');await button.listeners.get('click')[0]({});
 assert.equal(calls.length,1);assert.equal(calls[0][0],'ollama');assert.equal(calls[0][1],'/private/ollama/Teste.pdf');assert.match(view.el.textContent,/Portal unavailable/);
 }finally{Bridge.petMailDraft=original;}
});
