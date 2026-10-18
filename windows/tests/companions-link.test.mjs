import { test } from "node:test";
import assert from "node:assert/strict";
import { emit, sent } from "./tauri.mjs";
import { Bridge } from "../src/core/bridge.ts";
import { DEFAULT_SETTINGS, State } from "../src/core/state.ts";
import { CompanionsLink } from "../src/island/companions.ts";
import { PET_EVENTS } from "../src/mochi/companions.ts";

const task = id => ({ id, name: id, state: "idle", color: "#88aaff", steps: [], stepIndex: 0, source: "agent", isIntegration: true });
const settle = async () => { for (let i = 0; i < 12; i++) await Promise.resolve(); };

test("click routing never approves a permission or opens an integration absent from the habitat", async () => {
  const original = Bridge.companionsConfigure;
  const configurations = [], opened = [];
  try {
    Bridge.companionsConfigure = async (enabled, count) => { configurations.push([enabled, count]); return { width: 1440, height: 256, top: 64 }; };
    State.settings = { ...DEFAULT_SETTINGS, companionMode: true };
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
