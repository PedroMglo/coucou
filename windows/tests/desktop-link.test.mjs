import {test,mock} from 'node:test';
import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {DesktopLink} from '../src/island/desktop.ts';
const flush=async()=>{for(let i=0;i<20;i++)await Promise.resolve();};
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return{promise,resolve,reject};};

test('configured desktop pets use durable visibility intentions, preserve pause/identity/models and recover after rejection',async()=>{
 const previous={configure:Bridge.companionsConfigure,patch:Bridge.petRosterSettingsPatch,report:Bridge.reportSettingsError,settings:State.settings,cards:State.petCardsOnly,paused:State.paused,pending:State.pendingApproval};
 mock.timers.enable({apis:['setInterval']});const requests=[],jobs=[],errors=[];
 try{
  Bridge.companionsConfigure=async()=>({width:1440,height:320,top:0});
  Bridge.petRosterSettingsPatch=async request=>{requests.push(request);const job=deferred();jobs.push(job);return job.promise;};
  Bridge.reportSettingsError=error=>errors.push(String(error));
  State.petCardsOnly=true;State.paused=true;State.settings={...structuredClone(DEFAULT_SETTINGS),mainPill:'agent_codex',activeIntegrations:['integration_n8n'],companionMode:false,chatProvider:'codex',chatModels:{codex:'chosen-model'}};
  State.pendingApproval={pillId:'agent_codex',requestId:'keep-own-pending'};const baseline=structuredClone(State.settings);
  const link=new DesktopLink({reveal(){},openFromDesktop(){},wardrobeFromDesktop(){},dizzyFromDesktop(){}});await link.init();await link.launch();
  assert.equal(link.mode,'off','The retired singleton remains off');assert.equal(link.supported,true,'Current native pet surface supplies the capability even while hidden');
  const first=link.flyOutOrHome(),second=link.flyOutOrHome();await flush();assert.deepEqual(requests,[{action:'toggleDesktopVisibility'}]);assert.deepEqual(State.settings,baseline);
  jobs[0].resolve({mainPill:'agent_codex',activeIntegrations:['integration_n8n'],companionMode:true,chatProvider:'unwanted-provider'});await first;await flush();
  assert.deepEqual(State.settings,{...baseline,companionMode:true});assert.deepEqual(requests,[{action:'toggleDesktopVisibility'},{action:'toggleDesktopVisibility'}]);
  jobs[1].resolve({mainPill:'agent_codex',activeIntegrations:['integration_n8n'],companionMode:false});await second;assert.deepEqual(State.settings,baseline);assert.equal(State.paused,true);assert.equal(State.pendingApproval.requestId,'keep-own-pending');
  const failed=link.flyOutOrHome();await flush();jobs[2].reject(Error('synthetic disk failure'));await failed;assert.deepEqual(State.settings,baseline);assert.equal(errors.length,1);
  const next=link.flyOutOrHome();await flush();jobs[3].resolve({mainPill:'agent_codex',activeIntegrations:['integration_n8n'],companionMode:true});await next;assert.equal(State.settings.companionMode,true);assert.equal(State.paused,true);
 }finally{
  Bridge.companionsConfigure=previous.configure;Bridge.petRosterSettingsPatch=previous.patch;Bridge.reportSettingsError=previous.report;State.settings=previous.settings;State.petCardsOnly=previous.cards;State.paused=previous.paused;State.pendingApproval=previous.pending;mock.timers.reset();
 }
});
