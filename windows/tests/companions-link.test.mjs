import { test,beforeEach,afterEach,mock } from "node:test";
import assert from "node:assert/strict";
import { emit, sent } from "./tauri.mjs";
import { Bridge } from "../src/core/bridge.ts";
import { DEFAULT_SETTINGS, State } from "../src/core/state.ts";
import { CompanionsLink } from "../src/island/companions.ts";
import { PET_EVENTS } from "../src/mochi/companions.ts";

test('a completed document replaces an older pending summon with its action chooser',async()=>{
 const configure=Bridge.companionsConfigure,conversation=Bridge.chatConversation,documents=Bridge.petDocumentContexts,opened=[];
 try{
  Bridge.companionsConfigure=async()=>({width:1440,height:320,top:0});
  Bridge.chatConversation=async()=>({id:'ollama-doc-chat',messages:[]});
  State.settings={...DEFAULT_SETTINGS,companionMode:true,mainPill:'agent_codex',activeIntegrations:['ai_ollama']};
  State.tasks=[task('agent_codex'),task('ai_ollama')];State.pendingApproval=null;State.clearAllDialogueContexts();
  const link=new CompanionsLink((id,view)=>opened.push({id,view}),()=> 'none');await link.init();
  let file={id:'a',provider:'ollama',kind:'file',size:3,entries:1,name:'a.pdf',path:'/private/inbox/a.pdf',conversationId:'ollama-doc-chat',sequence:11};
  Bridge.petDocumentContexts=async()=>[file];
  emit('pet-document',{pet:'ai_ollama',files:[file]});await settle();
  assert.deepEqual(opened.at(-1),{id:'ai_ollama',view:'choose'});
  assert.ok(sent('plugin:event|emit_to').some(e=>e.event==='companions-present'&&e.payload.id==='ai_ollama'&&e.payload.view==='choose'));
  assert.equal(State.fileFor('ollama').sequence,11);assert.equal(State.fileFor('codex'),null);
  const before=opened.length;State.pendingApproval={pillId:'agent_codex',requestId:'native-decision'};
  file={...file,sequence:12};emit('pet-document',{pet:'ai_ollama',files:[file]});await settle();
  assert.equal(opened.length,before+1,'another pet waiting for approval must not block document actions');
  assert.deepEqual(opened.at(-1),{id:'ai_ollama',view:'choose'});
  assert.equal(State.pendingApproval.requestId,'native-decision');
  assert.equal(State.fileFor('ollama').sequence,12);
  State.pendingApproval={pillId:'ai_ollama',requestId:'own-decision'};
  file={...file,sequence:13};emit('pet-document',{pet:'ai_ollama',files:[file]});await settle();
  assert.equal(opened.length,before+1,'the receiving pet keeps its own pending decision');
  assert.equal(State.pendingApproval.requestId,'own-decision');
  assert.deepEqual(sent('chat_send'),[]);assert.deepEqual(sent('approval_decision'),[]);
 }finally{Bridge.companionsConfigure=configure;Bridge.chatConversation=conversation;Bridge.petDocumentContexts=documents;State.pendingApproval=null;}
});

const task = id => ({ id, name: id, state: "idle", color: "#88aaff", steps: [], stepIndex: 0, source: "agent", isIntegration: true });
const settle = async () => { for (let i = 0; i < 12; i++) await Promise.resolve(); };
beforeEach(()=>mock.timers.enable({apis:['setInterval']}));
afterEach(()=>mock.timers.reset());

test("click routing never approves a permission or opens an integration absent from the habitat", async () => {
  const original = Bridge.companionsConfigure;
  const configurations = [], opened = [];
  try {
    Bridge.companionsConfigure = async (enabled, count) => { configurations.push([enabled, count]); return { width: 1440, height: 256, top: 64 }; };
    State.settings = { ...DEFAULT_SETTINGS, companionMode: true, mainPill:"agent_codex", activeIntegrations:["integration_spotify"] };
    State.tasks = [task("agent_codex"), task("integration_spotify")];
    State.paused = false;
    const link = new CompanionsLink(id => opened.push(id), () => "none");
    await link.init();
    await link.launch();
    assert.equal(link.active, true);
    assert.deepEqual(configurations, [[false, 0], [true, 2]]);
    emit(PET_EVENTS.open, { pillId: "agent_codex" });
    emit(PET_EVENTS.open, { pillId: "missing-agent" });
    assert.deepEqual(opened, ["agent_codex"]);
    assert.deepEqual(sent("approval_decision"), []);
    assert.deepEqual(sent("spotify_control"), []);
    emit(PET_EVENTS.action, { action: "pause" });
    await settle();
    assert.equal(State.paused, true);
    assert.equal(configurations.length, 2, "a pause updates presentation without remapping the native window");
    State.settings.companionMode = false;
    State.notify();
    await settle();
    assert.equal(link.active, false);
    assert.deepEqual(configurations.at(-1), [false, 2]);
    emit(PET_EVENTS.open, { pillId: "agent_codex" });
    assert.deepEqual(opened, ["agent_codex"]);
  } finally { Bridge.companionsConfigure = original; }
});

test("unsupported desktops keep the singleton fallback available", async () => {
  const original = Bridge.companionsConfigure;
  try {
    Bridge.companionsConfigure = async () => null;
    State.settings = { ...DEFAULT_SETTINGS, companionMode: true };
    const link = new CompanionsLink(() => assert.fail("must not open"), () => "none");
    await link.init();
    await link.launch();
    assert.equal(link.supported, false);
    assert.equal(link.active, false);
  } finally { Bridge.companionsConfigure = original; }
});

test("a native placement failure leaves the singleton fallback available", async () => {
  const original = Bridge.companionsConfigure;
  try {
    Bridge.companionsConfigure = async enabled => enabled ? null : { width: 0, height: 0, top: 0 };
    State.settings = { ...DEFAULT_SETTINGS, companionMode: true };
    const link = new CompanionsLink(() => {}, () => "none");
    await link.init();
    await link.launch();
    assert.equal(link.supported, false);
    assert.equal(link.active, false);
  } finally { Bridge.companionsConfigure = original; }
});

test("reconciles an already-open client before hooks and wakes it only on a real presence change",async()=>{
 const configure=Bridge.companionsConfigure,presence=Bridge.companionsPresence;
 try{
  Bridge.companionsConfigure=async()=>({width:1440,height:320,top:0});
  Bridge.companionsPresence=async()=>({running:['agent_codex'],activity:{agent_codex:1}});
  State.settings={...DEFAULT_SETTINGS,companionMode:true,mainPill:'agent_codex',activeIntegrations:['agent_antigravity']};State.tasks=[task('agent_codex'),task('agent_antigravity')];
  const link=new CompanionsLink(()=>{},()=> 'none');await link.init();await link.launch();
  const state=()=>sent('plugin:event|emit_to').filter(e=>e.event===PET_EVENTS.state).at(-1).payload;
  assert.equal(state().pets[0].active,true);
  assert.equal(state().pets[0].busy,false,'opening a CLI must not pretend the model is working');
  assert.equal(state().pets[1].active,false);
  const activity=state().pets[0].activity;
  await link.sync();assert.equal(state().pets[0].activity,activity);
  State.tasks[0].sessionId='missed-session-end';State.tasks[0].state='working';
  emit('companions-presence',{running:[],activity:{agent_codex:2}});await settle();
  assert.equal(state().pets[0].active,false);
  assert.equal(State.tasks[0].sessionId,null,'closing the last client clears a missed SessionEnd');
  emit('companions-presence',{running:['agent_codex'],activity:{agent_codex:3}});await settle();
  assert.equal(state().pets[0].active,true);assert.notEqual(state().pets[0].activity,activity);
 }finally{Bridge.companionsConfigure=configure;Bridge.companionsPresence=presence;}
});

test("a service opens its own conversation without sending a job to the global provider",async()=>{
 const original=Bridge.companionsConfigure,opened=[];
 try{
  Bridge.companionsConfigure=async()=>({width:1440,height:320,top:0});
  State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama"};State.tasks=[task("integration_github")];State.preparedChat=null;
  const link=new CompanionsLink((id,view)=>opened.push([id,view]),()=>"none");await link.init();
  emit(PET_EVENTS.open,{pillId:"integration_github",view:"prompt"});
  assert.deepEqual(opened,[["integration_github","prompt"]]);assert.equal(State.preparedChat,null);assert.equal(State.settings.chatProvider,"ollama");assert.deepEqual(sent("chat_send"),[]);
  emit(PET_EVENTS.delegate,{pet:"missing-pet"});assert.equal(opened.length,1);
 }finally{Bridge.companionsConfigure=original;}
});


test("using a pet profile changes only its next model preference and never starts or moves a task",async()=>{
 const original=Bridge.companionsConfigure;
 try{
  Bridge.companionsConfigure=async()=>({width:1440,height:320,top:0});State.settings={...DEFAULT_SETTINGS,chatProvider:"ollama",chatModels:{codex:"manual",ollama:"local"}};
  const link=new CompanionsLink(()=>{},()=>"none");await link.init();
  emit("pet-use-profile",{provider:"codex"});await settle();
  assert.equal(State.settings.chatModels.codex,"@default");assert.equal(State.settings.chatModels.ollama,"local");assert.equal(State.settings.chatProvider,"ollama");assert.deepEqual(sent("chat_send"),[]);
  emit("pet-use-profile",{provider:"unknown"});await settle();assert.equal(State.settings.chatModels.unknown,undefined);
 }finally{Bridge.companionsConfigure=original;}
});


test("the shelter lists only configured entities and rejects unconfigured services without running models",async()=>{
 const original=Bridge.companionsConfigure,opened=[];
 try{
  Bridge.companionsConfigure=async()=>({width:1440,height:320,top:0});
  State.settings={...DEFAULT_SETTINGS,companionMode:true,mainPill:"agent_codex",activeIntegrations:["agent_antigravity","ai_ollama","integration_spotify"]};State.tasks=[task("agent_codex")];
  const link=new CompanionsLink(id=>opened.push(id),()=>"none");await link.init();await link.launch();
  const snapshot=sent("plugin:event|emit_to").filter(e=>e.event===PET_EVENTS.state).at(-1).payload;
  assert.equal(snapshot.pets.length,1);assert.equal(snapshot.catalog.length,4);
  for(const id of ["agent_codex","agent_antigravity","ai_ollama","integration_spotify"])assert.ok(snapshot.catalog.some(p=>p.id===id),id);
  emit(PET_EVENTS.summon,{pillId:"integration_spotify"});await settle();assert.equal(opened.at(-1),"integration_spotify");
  assert.ok(State.tasks.some(t=>t.id==="integration_spotify"));
  assert.ok(sent('plugin:event|emit_to').some(e=>e.event==='companions-present'&&e.payload.id==='integration_spotify'));
  emit(PET_EVENTS.summon,{pillId:"integration_vercel"});emit(PET_EVENTS.summon,{pillId:"ai_openai"});emit(PET_EVENTS.summon,{pillId:"missing"});await settle();assert.equal(opened.length,1);
  assert.deepEqual(sent("chat_send"),[]);assert.deepEqual(sent("spotify_control"),[]);assert.deepEqual(sent("approval_decision"),[]);
 }finally{Bridge.companionsConfigure=original;}
});

for(const scenario of ['display-during-habitat','bar-during-control','display-during-failed-habitat'])test(`${scenario}: an obsolete placement cannot consume a newer layout change`,async()=>{
 const original=Bridge.companionsConfigure,control=Bridge.petControlConfigure;
 let habitatCalls=0,controlCalls=0,release;
 const current={width:3440,height:320,top:0,band:{height:48,lanes:[[8,500]]}};
 try{
  Bridge.petControlConfigure=async()=>{controlCalls++;if(scenario==='bar-during-control'&&controlCalls===1)await new Promise(resolve=>release=resolve);};
  Bridge.companionsConfigure=async enabled=>{
   habitatCalls++;
   if(enabled&&habitatCalls===2&&scenario!=='bar-during-control')return new Promise(resolve=>release=resolve);
   return current;
  };
  State.settings={...DEFAULT_SETTINGS,companionMode:true,mainPill:'agent_codex',activeIntegrations:[]};State.tasks=[task('agent_codex')];State.pendingApproval=null;
  const link=new CompanionsLink(()=>assert.fail('layout updates must not open a card'),()=> 'none');await link.init();
  const running=link.launch();await settle();assert.equal(typeof release,'function');
  emit(scenario==='bar-during-control'?'companions-layout-changed':'screen-changed',null);
  if(scenario==='display-during-habitat')emit('companions-layout-changed',null);
  await settle();release(scenario==='display-during-failed-habitat'?null:{width:1440,height:320,top:0,band:{height:40,lanes:[[8,200]]}});
  await running;await settle();
  assert.equal(controlCalls,2,'Coalesced layout changes require a fresh control placement');
  assert.equal(habitatCalls,scenario==='bar-during-control'?2:3,'An obsolete control placement must not configure the habitat');
  assert.equal(link.supported,true);assert.equal(link.active,true,'An obsolete failure must not disable the current surface');
  const latest=sent('plugin:event|emit_to').filter(e=>e.event===PET_EVENTS.state).at(-1).payload;
  assert.deepEqual(latest.band,current.band,'Only the new geometry may reach the pets');
  assert.deepEqual(sent('chat_send'),[]);assert.deepEqual(sent('approval_decision'),[]);
 }finally{Bridge.companionsConfigure=original;Bridge.petControlConfigure=control;}
});
