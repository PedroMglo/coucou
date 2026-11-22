import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {calls,internals,sent} from './tauri.mjs';
installFakeDom();
window.addEventListener=()=>{};document.addEventListener=()=>{};window.setInterval=()=>0;
const timeout=window.setTimeout;window.setTimeout=(fn,delay,...args)=>{const t=timeout(fn,delay,...args);t.unref?.();return t;};
const {State,DEFAULT_SETTINGS}=await import('../src/core/state.ts');
const {buildPetOverview}=await import('../src/views/pet-overview.ts');
const {Sound}=await import('../src/core/sound.ts');Sound.setEnabled(false);
const task=id=>({id,name:id,color:'#83b5ff',state:'idle',source:'agent',steps:[],stepIndex:0});
function select(view,id){State.focusId=id;State.tasks=[task(id)];view.sync();}
function setup(id){
 State.petCardsOnly=true;State.settings={...DEFAULT_SETTINGS,companionMode:false,activeIntegrations:[id],mainPill:id};State.mode='expanded';State.view='overview';State.pendingApproval=null;State.focusId=id;State.tasks=[task(id)];State.integrations={};
 const view=buildPetOverview({setFocus(){},openTarget(){},blip(){},openSettingsWindow(){},openPetSettings(){}});view.sync();calls.length=0;return view;
}
const flush=async()=>{for(let i=0;i<16;i++)await Promise.resolve();};
const button=view=>view.el.querySelector('.pet-summary-voice');

test('LLM card starts the chosen pet voice route while wandering pets are off, without sending text or changing settings',async()=>{
 for(const[id,provider]of [['ai_ollama','ollama'],['agent_codex','codex'],['agent_copilot','copilot'],['agent_antigravity','antigravity']]){
  const view=setup(id),voice=button(view);assert.ok(voice,'direct card voice control');assert.equal(voice.hidden,false);
  assert.deepEqual(sent('voice_session_start'),[],'rendering never opens the microphone');
  voice.fire('click');await flush();assert.deepEqual(sent('voice_session_start'),[{provider}]);
  assert.deepEqual(sent('save_settings'),[]);assert.deepEqual(sent('pet_configuration_save'),[]);assert.deepEqual(sent('chat_send'),[]);
 }
});

test('native service and unsupported voice cards cannot start a generic LLM voice route',async()=>{
 for(const id of ['integration_n8n','integration_notion','integration_vercel','integration_resend','integration_calcom','integration_stripe','integration_github','ai_lmstudio']){
  const view=setup(id),voice=button(view);assert.ok(voice);assert.equal(voice.hidden,true,id);
  voice.fire('click');await flush();assert.deepEqual(sent('voice_session_start'),[],id);
 }
});

test('a pending voice admission is not duplicated and its refusal stays on its own pet',async()=>{
 const original=internals.invoke;let rejectVoice;
 internals.invoke=(cmd,args)=>cmd==='voice_session_start'?(calls.push([cmd,args]),new Promise((_,reject)=>{rejectVoice=reject;})):original(cmd,args);
 try{
  const view=setup('ai_ollama'),voice=button(view);assert.ok(voice);voice.fire('click');voice.fire('click');
  assert.deepEqual(sent('voice_session_start'),[{provider:'ollama'}]);assert.equal(voice.disabled,true);
  rejectVoice('Microfone de teste indisponível');await flush();assert.equal(voice.disabled,false);assert.match(view.el.querySelector('.pet-summary-error').textContent,/Microfone de teste indisponível/);
  voice.fire('click');select(view,'integration_n8n');rejectVoice('STALE_OLLAMA_FAILURE');await flush();
  assert.equal(button(view).hidden,true);assert.doesNotMatch(view.el.querySelector('.pet-summary-error').textContent,/STALE_OLLAMA_FAILURE/);
 }finally{internals.invoke=original;}
});
