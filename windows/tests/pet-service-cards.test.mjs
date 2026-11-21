import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {calls,emit,sent} from './tauri.mjs';
installFakeDom();
window.addEventListener=()=>{};document.addEventListener=()=>{};
window.setInterval=()=>0;
const timeouts=new Map();let timer=0;
const nativeTimeout=window.setTimeout,nativeClear=window.clearTimeout;
window.setTimeout=(fn,delay,...args)=>{if(delay===60000){const id=++timer;timeouts.set(id,fn);return id;}const t=nativeTimeout(fn,delay,...args);t.unref?.();return t;};
window.clearTimeout=id=>{if(timeouts.has(id))timeouts.delete(id);else nativeClear(id);};
const {State,DEFAULT_SETTINGS}=await import('../src/core/state.ts');
const {buildPetOverview}=await import('../src/views/pet-overview.ts');
const {serviceCards}=await import('../src/island/phone-services.ts');
const {registerIntegrationHandlers}=await import('../src/island/integrations.ts');
registerIntegrationHandlers({reveal(){}});
const {Sound}=await import('../src/core/sound.ts');Sound.setEnabled(false);
const task=(id,extra={})=>({id,name:id,color:'#f29b38',source:'n8n',state:'idle',isIntegration:true,steps:[],stepIndex:0,...extra});
function setup(id,data={},extra={}){
 State.settings={...DEFAULT_SETTINGS,companionMode:true,activeIntegrations:[id],mainPill:id};State.mode='expanded';State.view='overview';State.focusId=id;State.paused=false;State.pendingApproval=null;State.tasks=[task(id,extra)];State.integrations={};State.integrations[id]={configured:true,loaded:true,error:null,data};calls.length=0;
 const view=buildPetOverview({setFocus:id=>{State.focusId=id;},openTarget(){},blip(){},openSettingsWindow(){},openPetSettings(){}});view.sync();return view;
}
const section=view=>view.el.querySelector('.pet-summary-integration');
const fixtures=[
 ['integration_notion',{pages:[{title:'Native project note',url:'https://notion.so/qa',lastEditedAt:Date.now()}]},'Native project note'],
 ['integration_vercel',{deployments:[{id:'native-deploy',projectName:'Native deployment',state:'READY',createdAt:Date.now(),branch:'qa-branch',commitMessage:'Native commit'}]},'Native deployment'],
 ['integration_resend',{emails:[{to:['qa@example.invalid'],subject:'Native subject',lastEvent:'delivered',createdAt:Date.now()}]},'Native subject'],
 ['integration_calcom',{bookings:[{title:'Native booking',start:new Date(Date.now()+3600000).toISOString()}]},'Native booking'],
 ['integration_stripe',{balances:[{currency:'eur',available:'12.34',pending:null}],payments:[]},'EUR'],
];
for(const[id,data,text]of fixtures)test(`${id} native card is available inside its pet without provider effects`,()=>{
 const view=setup(id,data),surface=section(view);assert.ok(surface&&!surface.hidden);assert.match(surface.textContent,new RegExp(text));assert.ok(view.el.textContent.includes('Configurar pet'));assert.deepEqual(sent('chat_send'),[]);assert.deepEqual(sent('refresh_integration'),[]);
});
test('n8n keeps native Refresh, detail/back and native activity separate from a model reply',()=>{
 const view=setup('integration_n8n');let surface=section(view);assert.ok(surface);const refresh=surface.find('button').find(b=>b.textContent==='Refresh');assert.ok(refresh);refresh.fire('click');assert.deepEqual(sent('refresh_integration'),[{id:'integration_n8n'}]);
 emit('integration',{id:'integration_n8n',data:{workflow:'Native workflow',status:'success'},error:null,event:{success:true,label:'Native workflow',detail:'Native original result'}});
 State.tasks[0]={...State.tasks[0],sessionId:'selected-own-session',sessionBackend:'integration_n8n',sessionOwned:true,state:'finished',steps:['MODEL_REPLY_MUST_NOT_BECOME_WORKFLOW']};view.sync();surface=section(view);assert.match(surface.textContent,/Native workflow/);assert.doesNotMatch(surface.textContent,/MODEL_REPLY/);
 surface.querySelector('.int-pill').fire('click');assert.match(surface.textContent,/Native original result/);assert.ok(surface.querySelector('.int-back'));surface.querySelector('.int-back').fire('click');assert.equal(surface.querySelector('.int-detail-text'),null);
 const stable=surface.firstChild;view.sync();assert.equal(surface.firstChild,stable,'unchanged state preserves focused controls');
 for(const finish of timeouts.values())finish();timeouts.clear();view.sync();assert.doesNotMatch(surface.textContent,/Native workflow|MODEL_REPLY/);assert.deepEqual(sent('chat_send'),[]);
});
test('Vercel keeps the selected deployment through refreshes and shows when it leaves the results',()=>{
 const id='integration_vercel',original=fixtures[1][1].deployments[0];
 const next={...original,id:'next',projectName:'New deployment',commitMessage:'New commit'};
 const view=setup(id,{deployments:[original]});
 const surface=section(view);
 surface.querySelector('.int-more').fire('click');
 assert.match(surface.textContent,/Native commit/);
 const refresh=deployments=>{
  emit('integration',{id,data:{deployments},error:null,event:null});
  view.sync();
 };
 refresh([next,original]);
 assert.match(surface.textContent,/Native commit/);
 assert.doesNotMatch(surface.textContent,/New commit/);
 refresh([next]);
 assert.match(surface.textContent,/Deployment outside the latest results/);
 assert.doesNotMatch(surface.textContent,/New commit|Native commit/);
 surface.querySelector('.int-back').fire('click');
 assert.match(surface.textContent,/New deployment/);
});
test('changing pets clears integration detail and does not revive retired GitHub or replace Ollama activity',()=>{
 const view=setup('integration_vercel',fixtures[1][1]);let surface=section(view);assert.ok(surface);surface.querySelector('.int-more').fire('click');assert.match(surface.textContent,/Native commit/);
 const id='integration_notion';State.tasks=[task(id)];State.focusId=id;State.integrations[id]={configured:true,loaded:true,error:null,data:fixtures[0][1]};view.sync();assert.match(surface.textContent,/Native project note/);assert.doesNotMatch(surface.textContent,/Native commit/);
 State.tasks=[task('integration_github')];State.focusId='integration_github';view.sync();assert.equal(surface.hidden,true);
 State.tasks=[task('ai_ollama',{state:'working',source:'agent',steps:['Actual model activity']})];State.focusId='ai_ollama';view.sync();assert.equal(surface.hidden,true);assert.match(view.el.querySelector('.pet-summary-live').textContent,/Actual model activity/);assert.deepEqual(sent('chat_send'),[]);
});
test('phone n8n keeps the native workflow when its own assistant session is selected',()=>{
 setup('integration_n8n');
 emit('integration',{id:'integration_n8n',data:{workflow:'Native workflow',status:'success'},error:null,event:{success:true,label:'Native workflow',detail:'NATIVE_DETAIL_MUST_STAY_ON_COMPUTER'}});
 State.tasks[0]={...State.tasks[0],sessionId:'selected-own-session',sessionBackend:'integration_n8n',sessionOwned:true,state:'error',steps:['MODEL_REPLY_MUST_NOT_BECOME_WORKFLOW']};
 const card=serviceCards().find(c=>c.id==='integration_n8n');
 assert.equal(card?.headline,'Success');assert.deepEqual(card?.items,[{label:'Native workflow',detail:''}]);
 assert.doesNotMatch(JSON.stringify(card),/MODEL_REPLY|NATIVE_DETAIL/);
 State.tasks[0].state='working';for(const finish of timeouts.values())finish();timeouts.clear();
 assert.equal(serviceCards().find(c=>c.id==='integration_n8n'),undefined,'expired native notification is not replaced by assistant progress');
 assert.deepEqual(sent('chat_send'),[]);
});
test('phone n8n cannot invent a workflow from an assistant-only result',()=>{
 setup('integration_n8n',{}, {sessionId:'selected-own-session',sessionBackend:'integration_n8n',sessionOwned:true,state:'finished',steps:['MODEL_REPLY_MUST_NOT_BECOME_WORKFLOW']});
 assert.equal(serviceCards().find(c=>c.id==='integration_n8n'),undefined);
 assert.deepEqual(sent('chat_send'),[]);
});


test('native service cards offer their workflow or service functions without a generic chat or model lookup',async()=>{
 const {petCapabilities}=await import('../src/mochi/capabilities.ts');
 for(const id of ['integration_n8n',...fixtures.map(([id])=>id)]){
  const view=setup(id);const caps=petCapabilities(id);assert.equal(caps.chat,null);assert.equal(caps.voice,null);assert.equal(caps.execute,false);assert.equal(view.el.querySelector('.pet-summary-primary').hidden,true);
  assert.ok(section(view));assert.deepEqual(sent('chat_models'),[]);assert.deepEqual(sent('chat_conversations'),[]);assert.deepEqual(sent('pet_chat_open'),[]);
 }
});

test('retired GitHub cache is preserved but cannot become a phone service card',()=>{
 setup('integration_notion',{pages:[]});
 const legacy={configured:true,loaded:true,error:null,data:{totalStars:42,totalRepos:7}};
 State.integrations.integration_github=legacy;
 State.settings.activeIntegrations.push('integration_github');
 assert.equal(serviceCards().some(c=>c.id==='integration_github'),false);
 assert.ok(serviceCards().some(c=>c.id==='integration_notion'));
 assert.equal(State.integrations.integration_github,legacy);
 assert.deepEqual(legacy.data,{totalStars:42,totalRepos:7});
 assert.ok(State.settings.activeIntegrations.includes('integration_github'));
});

test('phone settings offer existing services without reactivating the retired entity',async()=>{
 const {phoneSection}=await import('../src/settings/phone.ts');
 const {h}=await import('../src/views/dom.ts');
 const {internals}=await import('./tauri.mjs');const before=internals.invoke;
 internals.invoke=(cmd,args)=>cmd==='phone_link_status'?Promise.resolve({running:true,clients:0,host:'127.0.0.1',port:1}):before(cmd,args);
 try{
  const section=phoneSection(true,()=>h('button'),{...DEFAULT_SETTINGS});
  for(let i=0;i<20;i++)await Promise.resolve();
  const labels=section.find('label').map(n=>n.textContent.trim());
  assert.ok(labels.includes('Notion'));
  assert.ok(labels.includes('n8n'));
  assert.ok(!labels.includes('GitHub'));
 }finally{internals.invoke=before;}
});
