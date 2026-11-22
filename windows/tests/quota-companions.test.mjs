import {test,mock} from "node:test";
import assert from "node:assert/strict";
import {calls,sent} from "./tauri.mjs";
import {Bridge} from "../src/core/bridge.ts";
import {DEFAULT_SETTINGS,State} from "../src/core/state.ts";
import {CompanionsLink} from "../src/island/companions.ts";
import {companionIdentities} from "../src/mochi/companions.ts";

const task=(id,state="idle")=>({id,name:id,state,color:"#88aaff",steps:[],stepIndex:0,source:"agent",isIntegration:true});
const now=Date.now();
const quota=(used,extra={})=>({updated:now/1000,windows:[{label:"5 h",used,reset_at:(now+3600000)/1000}],...extra});
const settle=async()=>{for(let i=0;i<60;i++)await Promise.resolve();};

test("quota metadata preserves approvals, real rate limits and task activity",()=>{
  const tasks=[task("agent_codex","working"),task("agent_copilot","ratelimit")];
  const before=companionIdentities(tasks,null,false,"none");
  const usage={providers:{codex:quota(90),copilot:quota(12)}};
  const pets=companionIdentities(tasks,{pillId:"agent_codex",requestId:"native-decision"},false,"none",new Set(),new Map(),{
    usage,configuredPets:new Set(tasks.map(t=>t.id)),now,
  });
  assert.equal(pets[0].state,"approval");assert.equal(pets[0].attention,true);
  assert.equal(pets[0].quota.status,"warning");assert.equal(pets[0].quota.usedPercent,90);
  assert.equal(pets[1].state,"ratelimit");assert.equal(pets[1].attention,true);
  assert.equal(pets[1].quota.status,"ok");assert.equal(pets[1].activity,before[1].activity);
  assert.equal(tasks[0].state,"working");assert.equal(tasks[1].state,"ratelimit");
});

test("unconfigured external pets receive no quota and missing supported data stays unknown",()=>{
  const pets=companionIdentities([task("agent_codex"),task("agent_copilot"),task("ai_ollama")],null,false,"none",new Set(),new Map(),{
    usage:{providers:{}},configuredPets:new Set(["agent_copilot","ai_ollama"]),now,
  });
  assert.equal(pets[0].quota,undefined);
  assert.equal(pets[1].quota.status,"unknown");assert.equal(pets[1].quota.usedPercent,undefined);
  assert.equal(pets[2].quota,undefined);
});

test("updating only a quota timestamp does not wake or change a pet's task",()=>{
  const make=updated=>companionIdentities([task("agent_codex")],null,false,"none",new Set(),new Map(),{
    usage:{providers:{codex:quota(90,{updated})}},configuredPets:new Set(["agent_codex"]),now,
  })[0];
  const old=make((now-10000)/1000),current=make(now/1000);
  assert.equal(old.activity,current.activity);assert.equal(old.attention,false);assert.equal(current.attention,false);
  assert.equal(old.active,false);assert.equal(current.active,false);
});

test("existing inventory and plans feed notices without quota refreshes or prompts",async()=>{
  mock.timers.enable({apis:["setInterval"]});
  calls.length=0;
  const actions=[];
  let inventory={sessions:[],usage:{providers:{codex:quota(90),copilot:{updated:now/1000,windows:[],blocked:false},claude:quota(100,{blocked:true})}}};
  mock.method(Bridge,"companionsConfigure",async()=>({width:1440,height:320,top:0}));
  mock.method(Bridge,"petManagement",async action=>{actions.push(action);return inventory;});
  mock.method(Bridge,"assistantTasks",async()=>[]);
  mock.method(Bridge,"companionsPresence",async()=>({running:[],activity:{}}));
  try{
    State.settings={...DEFAULT_SETTINGS,companionMode:true,mainPill:"agent_codex",activeIntegrations:["agent_copilot"]};
    State.tasks=[task("agent_codex"),task("agent_copilot"),{...task("integration_claude","working"),sessionId:"unconfigured-native"}];
    State.pendingApproval=null;State.chatBusyProviders.clear();State.paused=false;State.mode="collapsed";
    State.planUsage=null;State.codexPlanUsage=null;
    const link=new CompanionsLink(()=>{},()=>"none");
    await link.init();await settle();await link.launch();await settle();
    const notices=()=>sent("plugin:event|emit_to").filter(e=>e.event==="companions-notice").map(e=>e.payload);
    const latest=()=>sent("plugin:event|emit_to").filter(e=>e.event==="companions-state"&&e.target?.label==="companions").at(-1)?.payload
      ??sent("plugin:event|emit_to").filter(e=>e.event==="companions-state").at(-1).payload;
    assert.equal(notices().length,1);assert.equal(notices()[0].pet,"agent_codex");
    assert.equal(notices()[0].urgent,false);
    assert.equal(latest().pets.find(p=>p.id==="agent_copilot").quota.status,"unknown");
    assert.equal(latest().pets.find(p=>p.id==="agent_copilot").quota.usedPercent,undefined);
    assert.equal(latest().pets.find(p=>p.id==="integration_claude").quota,undefined);
    mock.timers.tick(15000);await settle();assert.equal(notices().length,1,"unchanged metadata does not repeat a notice");
    State.pendingApproval={pillId:"agent_codex",requestId:"human-only",sessionId:"native",tool:"shell",command:"test"};
    inventory={sessions:[],usage:{providers:{codex:quota(90),copilot:quota(100,{blocked:true})}}};
    mock.timers.tick(15000);await settle();
    assert.equal(notices().length,1,"an approval keeps the quota notice deferred");
    assert.equal(latest().pets.find(p=>p.id==="agent_codex").state,"approval");
    State.pendingApproval=null;State.notify();await settle();
    assert.equal(notices().length,2);assert.equal(notices()[1].pet,"agent_copilot");
    State.tasks.find(t=>t.id==="agent_copilot").state="ratelimit";State.chatBusyProviders.add("copilot");
    inventory={sessions:[],usage:{providers:{codex:quota(90),copilot:quota(10,{blocked:false})}}};
    mock.timers.tick(15000);await settle();
    assert.equal(latest().pets.find(p=>p.id==="agent_copilot").state,"ratelimit","a fresh low quota never clears the actual native rate limit");
    inventory={sessions:[]};
    State.codexPlanUsage={updatedAt:Date.now(),fiveHour:{usedPct:94,resetsAt:Date.now()+3600000}};
    mock.timers.tick(15000);await settle();
    assert.equal(latest().pets.find(p=>p.id==="agent_codex").quota.usedPercent,94,"the existing plan is a source when no cache is supplied");
    assert.equal(latest().pets.find(p=>p.id==="agent_copilot").quota.status,"unknown");
    assert.ok(actions.length>1);assert.ok(actions.every(action=>action==="list"),"only the existing inventory cadence is used");
    assert.deepEqual(sent("chat_send"),[]);assert.deepEqual(sent("chat_models"),[]);assert.deepEqual(sent("approval_decision"),[]);
    const before=actions.length;State.settings.companionMode=false;State.notify();await settle();
    mock.timers.tick(15000);await settle();assert.equal(actions.length,before,"disabled companions make no inventory requests");
  }finally{
    State.pendingApproval=null;State.settings.companionMode=false;mock.restoreAll();mock.timers.reset();
  }
});
