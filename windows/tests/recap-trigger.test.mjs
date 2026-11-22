// When the weekly recap opens on its own (src/recap/recap.ts), driven through
// the real bridge with Rust's answers stubbed.

import { afterEach, beforeEach, mock, test } from "node:test";
import assert from "node:assert/strict";
import { calls, internals, sent } from "./tauri.mjs";
import { RecapController } from "../src/recap/recap.ts";
import { sampleHistory, previousWeekStart } from "../src/recap/summary.ts";
import { State } from "../src/core/state.ts";

const MONDAY_9AM = new Date(2026, 9, 5, 9, 0);
const original = internals.invoke;

let Recap;
let prefs;
let history;
let shown;
const host = { alert: (view) => shown.push(view) };

beforeEach(() => {
  mock.timers.enable({ apis: ["setTimeout"] });
  calls.length = 0;
  Recap=new RecapController();
  shown = [];
  prefs = { enabled: true, hideProjects: false, lastShownWeek: "" };
  history = sampleHistory(previousWeekStart(MONDAY_9AM));
  internals.invoke = async (cmd, args) => {
    if (cmd.startsWith("recap_") && cmd !== "recap_mark_shown") calls.push([cmd, args]);
    if (cmd === "recap_prefs") return prefs;
    if (cmd === "recap_history") return { ...history, prefs };
    if(cmd==='recap_mark_shown'){calls.push([cmd,args]);prefs={...prefs,lastShownWeek:args.week};return {prefs:{...prefs},persisted:true,historyCleared:false,errorCode:null};}
    return original(cmd, args);
  };
  State.mode = "hidden";
  State.view = "overview";
  State.paused = false;
  State.pendingApproval = null;
});

afterEach(() => {
  internals.invoke = original;
  mock.timers.reset();
});

const settle = () => new Promise((resolve) => setImmediate(resolve));

test("on Monday morning the recap opens once, after a short delay", async () => {
  await Recap.check(host, MONDAY_9AM);
  assert.deepEqual(shown, []);
  mock.timers.tick(1500);
  await settle();
  assert.deepEqual(shown, ["recap"]);
  assert.deepEqual(sent("recap_mark_shown"), [{ week: "2026-10-05" }]);
  assert.equal(Recap.summary.sessionCount, 6);
  // The history was asked from last Monday on.
  assert.equal(sent("recap_history")[0].since, previousWeekStart(MONDAY_9AM).getTime() / 1000);
});

test("not twice in the same week, not on Tuesday, not before 8", async () => {
  prefs.lastShownWeek = "2026-10-05";
  await Recap.check(host, MONDAY_9AM);
  prefs.lastShownWeek = "";
  await Recap.check(host, new Date(2026, 9, 6, 9));
  await Recap.check(host, new Date(2026, 9, 5, 7, 30));
  mock.timers.tick(5000);
  assert.deepEqual(shown, []);
});

test("an empty week or a disabled history shows nothing", async () => {
  history = { turns: [], decisions: [], prefs };
  await Recap.check(host, MONDAY_9AM);
  prefs = { ...prefs, enabled: false };
  history = sampleHistory(previousWeekStart(MONDAY_9AM));
  await Recap.check(host, MONDAY_9AM);
  mock.timers.tick(5000);
  assert.deepEqual(shown, []);
  assert.deepEqual(sent("recap_mark_shown"), []);
});

test("it never covers a permission request or a chat in progress", async () => {
  State.pendingApproval = { requestId: "r", sessionId: "s", tool: "Bash", command: "ls" };
  await Recap.check(host, MONDAY_9AM);
  State.pendingApproval = null;
  State.mode = "expanded";
  State.view = "prompt";
  await Recap.check(host, MONDAY_9AM);
  mock.timers.tick(5000);
  assert.deepEqual(shown, []);

  // A request arriving during the delay wins too.
  State.mode = "hidden";
  await Recap.check(host, MONDAY_9AM);
  State.pendingApproval = { requestId: "r", sessionId: "s", tool: "Bash", command: "ls" };
  mock.timers.tick(1500);
  await settle();
  assert.deepEqual(shown, []);
  assert.deepEqual(sent("recap_mark_shown"), []);
});

test("the tray opens it any day, even for an empty week", async () => {
  history = { turns: [], decisions: [], prefs };
  const before = Recap.version;
  await Recap.open(host);
  await settle();
  assert.deepEqual(shown, ["recap"]);
  assert.equal(Recap.summary, null);
  assert.equal(Recap.version, before + 1);
});
test('failed persistence of markShown keeps native prefs, reports the failure and never automatically retries this week',async()=>{
 const delegate=internals.invoke;internals.invoke=async(cmd,args)=>{if(cmd==='recap_mark_shown'){calls.push([cmd,args]);return {prefs:{...prefs},persisted:false,historyCleared:false,errorCode:'writeFailed'};}return delegate(cmd,args);};
 await Recap.check(host,MONDAY_9AM);mock.timers.tick(1500);await settle();assert.deepEqual(shown,['recap']);assert.equal(Recap.prefs.lastShownWeek,'');assert.match(Recap.notice,/não foi possível guardar/);await Recap.check(host,MONDAY_9AM);mock.timers.tick(1500);await settle();assert.equal(sent('recap_mark_shown').length,1);
});
test('markShown transport refusal is handled without a false saved week or unhandled rejection',async()=>{
 const delegate=internals.invoke;internals.invoke=async(cmd,args)=>{if(cmd==='recap_mark_shown'){calls.push([cmd,args]);throw Error('private path');}return delegate(cmd,args);};
 await Recap.check(host,MONDAY_9AM);mock.timers.tick(1500);await settle();assert.deepEqual(shown,['recap']);assert.equal(Recap.prefs.lastShownWeek,'');assert.match(Recap.notice,/não foi possível confirmar/);assert.doesNotMatch(Recap.notice,/private path/);
});
test('a stale delayed preference failure cannot replace newer privacy receipt feedback',async()=>{
 const delegate=internals.invoke;let reads=0,rejectRead;const slow=new Promise((_,reject)=>rejectRead=reject);
 internals.invoke=async(cmd,args)=>{if(cmd==='recap_prefs'&&++reads===2)return slow;if(cmd==='recap_set_hide_projects')return {prefs:{...prefs,hideProjects:true},persisted:false,historyCleared:false,errorCode:'writeFailed'};return delegate(cmd,args);};
 await Recap.check(host,MONDAY_9AM);mock.timers.tick(1500);await settle();await Recap.setHideProjects(true);const notice=Recap.notice;rejectRead(Error('old metadata failure'));await settle();assert.equal(Recap.prefs.hideProjects,true);assert.equal(Recap.notice,notice);assert.equal(Recap.noticeFrom,'hide');assert.deepEqual(shown,[]);
});
test('a read started during markShown cannot replace its acknowledged week',async()=>{
 const delegate=internals.invoke;let resolveMark,resolveRead,holding=false;const marker=new Promise(resolve=>resolveMark=resolve),reading=new Promise(resolve=>resolveRead=resolve);
 internals.invoke=async(cmd,args)=>{if(cmd==='recap_mark_shown')return marker;if(cmd==='recap_prefs'&&holding)return reading;return delegate(cmd,args);};
 await Recap.check(host,MONDAY_9AM);mock.timers.tick(1500);await settle();holding=true;const observer=Recap.readPreferences();resolveMark({prefs:{...prefs,lastShownWeek:'2026-10-05'},persisted:true,historyCleared:false,errorCode:null});await settle();resolveRead({...prefs});assert.equal(await observer,false);assert.equal(Recap.prefs.lastShownWeek,'2026-10-05');
});
test('settlement of an older markShown cannot invalidate a newer hide intent or its fresh read',async()=>{
 const delegate=internals.invoke;let resolveMark,resolveRead,holding=false;const marker=new Promise(resolve=>resolveMark=resolve),reading=new Promise(resolve=>resolveRead=resolve);
 internals.invoke=async(cmd,args)=>{if(cmd==='recap_mark_shown')return marker;if(cmd==='recap_set_hide_projects')return {prefs:{...prefs,hideProjects:true},persisted:true,historyCleared:false,errorCode:null};if(cmd==='recap_prefs'&&holding)return reading;return delegate(cmd,args);};
 await Recap.check(host,MONDAY_9AM);mock.timers.tick(1500);await settle();await Recap.setHideProjects(true);holding=true;const observer=Recap.readPreferences();resolveMark({prefs:{...prefs,lastShownWeek:'2026-10-05'},persisted:true,historyCleared:false,errorCode:null});await settle();resolveRead({...prefs,hideProjects:true,lastShownWeek:'2026-10-05'});assert.equal(await observer,true);assert.equal(Recap.prefs.hideProjects,true);assert.equal(Recap.notice,'');
});
