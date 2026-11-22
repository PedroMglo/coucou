import { beforeEach, test } from "node:test";
import assert from "node:assert/strict";
import { installFakeDom } from "./fakedom.mjs";
import { calls, internals, sent } from "./tauri.mjs";

installFakeDom();
const { buildPrompt } = await import("../src/views/chat.ts");
const { DEFAULT_SETTINGS, State } = await import("../src/core/state.ts");
const { SessionChat } = await import("../src/core/session-chat.ts");
const offered = [
  { id: "local-fast", label: "Rápido Local" },
  { id: "gpt-6-luna", label: "GPT 6 Luna" },
  { id: "code-deep", label: "Código Profundo" },
];
let answers,view;
const invoke=internals.invoke;
internals.invoke=async(cmd,args)=>{
  const fallback=await invoke(cmd,args);
  return cmd in answers ? typeof answers[cmd]==="function"?answers[cmd](args):answers[cmd] : fallback;
};
const flush=()=>new Promise(resolve=>setTimeout(resolve,0));
const $=selector=>view.el.querySelector(selector);
const models=()=>view.el.find(".picker-model").map(row=>row.textContent);
const search=query=>{const input=$(".picker-search");input.value=query;input.fire("input");};
const open=async()=>{$(".model-btn").fire("click");await flush();};
function keydown(element,key){
  const event={key,defaultPrevented:false,stopped:false,preventDefault(){this.defaultPrevented=true;},stopPropagation(){this.stopped=true;}};
  for(const listener of element.listeners.get("keydown")??[])listener(event);
  return event;
}
beforeEach(()=>{
  calls.length=0;
  answers={
    secret_present:true,chat_models:offered,
    chat_conversation:{id:"picker-test-conversation",messages:[],busy:false,turn:0},
  };
  State.settings={...DEFAULT_SETTINGS,chatProvider:"codex",chatModels:{}};
  State.chatHistory=[];State.chatBusyProviders.clear();State.chatResettingProviders.clear();
  State.chatFeedback.clear();State.clearAllDialogueContexts();State.preparedChat=null;
  State.view="prompt";State.stateOverride=null;
  document.activeElement=null;
  view=buildPrompt(()=>{});view.sync();
});

test("model search matches names, identifiers, accents and every query word",async()=>{
  await open();
  search("RAPIDO local");assert.deepEqual(models(),["Rápido Local"]);
  search("gpt-6-luna");assert.deepEqual(models(),["GPT 6 Luna"]);
  search("LUNA gPt");assert.deepEqual(models(),["GPT 6 Luna"]);
  search("codigo profundo");assert.deepEqual(models(),["Código Profundo"]);
  search("local luna");assert.deepEqual(models(),[]);
});

test("filtering performs no model requests, setting writes or prompts",async()=>{
  await open();
  const catalogs=sent("chat_models").length,writes=sent("save_settings").length;
  const chosen=State.settings.chatModels.codex;
  for(const query of ["Luna","nothing","rapido",""])search(query);
  assert.equal(sent("chat_models").length,catalogs);
  assert.equal(sent("save_settings").length,writes);
  assert.equal(State.settings.chatModels.codex,chosen);
  assert.equal(sent("chat_send").length,0);
});

test("an empty result explains itself and clearing search restores original order",async()=>{
  await open();const original=models();
  search("no-such-model");
  assert.deepEqual(models(),[]);assert.match($(".picker-status").textContent,/pesquisa/);
  search("  ");assert.deepEqual(models(),original);
  assert.equal(sent("chat_models").length,1);
});

test("a query typed while the catalog loads applies when it arrives",async()=>{
  let finish;answers.chat_models=()=>new Promise(resolve=>{finish=resolve;});
  await open();search("luna");
  assert.match($(".picker-status").textContent,/Loading/);
  finish(offered);await flush();
  assert.deepEqual(models(),["GPT 6 Luna"]);assert.equal(sent("chat_models").length,1);
});

test("switching provider clears search and removes stale selectable rows before awaiting",async()=>{
  let finish;
  answers.chat_models=({provider})=>provider==="google"?new Promise(resolve=>{finish=resolve;}):offered;
  await open();search("luna");
  view.el.find(".picker-chip").find(chip=>chip.textContent==="Google").fire("click");
  assert.equal($(".picker-search").value,"");assert.deepEqual(models(),[]);
  search("flash");await flush();
  finish([{id:"gemini-flash",label:"Gemini Flash"},{id:"gemini-pro",label:"Gemini Pro"}]);
  await flush();assert.deepEqual(models(),["Gemini Flash"]);
  view.el.find(".picker-chip").find(chip=>chip.textContent==="Codex").fire("click");
  await flush();assert.equal($(".picker-search").value,"");
  assert.ok(models().includes("GPT 6 Luna"));assert.equal(sent("chat_models").length,2);
});

test("picking a filtered model saves its exact identifier and closes the picker",async()=>{
  await open();search("profundo");
  $(".picker-model").fire("click");
  assert.equal(State.settings.chatModels.codex,"code-deep");
  assert.equal(sent("save_settings").at(-1).settings.chatModels.codex,"code-deep");
  assert.ok(!$(".chat-body").classList.contains("picking"));
  assert.equal(sent("chat_send").length,0);
});

test("the external session picker filters without changing the pet's saved model",async()=>{
  const target={petId:"agent_codex",provider:"codex",origin:"external",sessionId:"native-session"};
  const session=new SessionChat({
    history:async()=>({target,messages:[],busy:false,model:"native-model",capabilities:{history:true,send:true,cancel:true}}),
    send:async()=>{},cancel:async()=>{},
  },()=>{});
  await session.select(target);view=buildPrompt(()=>{},session);view.sync();
  await open();search("luna");assert.deepEqual(models(),["GPT 6 Luna"]);
  $(".picker-model").fire("click");
  assert.equal(session.model,"gpt-6-luna");assert.equal(State.settings.chatModels.codex,undefined);
  assert.equal(sent("save_settings").length,0);assert.equal(sent("session_chat_send").length,0);
});

test("search keyboard controls keep Enter local and restore focus on Escape",async()=>{
  const button=$(".model-btn"),input=$(".picker-search");
  let focused=0,returned=0;
  button.focus=()=>{returned++;document.activeElement=button;};
  input.focus=()=>{document.activeElement=input;};
  document.activeElement=button;await open();assert.equal(document.activeElement,input);
  const enter=keydown(input,"Enter");assert.ok(enter.defaultPrevented&&enter.stopped);
  $(".picker-model").focus=()=>{focused++;};
  const down=keydown(input,"ArrowDown");assert.ok(down.defaultPrevented);assert.equal(focused,1);
  const escape=keydown(input,"Escape");assert.ok(escape.defaultPrevented&&escape.stopped);
  assert.ok(!$(".chat-body").classList.contains("picking"));assert.equal(returned,1);
  assert.equal(sent("chat_send").length,0);
});

test("reopening clears the old query and uses the cached catalog",async()=>{
  await open();search("luna");$(".model-btn").fire("click");await open();
  assert.equal($(".picker-search").value,"");assert.ok(models().includes("Rápido Local"));
  assert.equal(sent("chat_models").length,1);
});

test("closing the picker ignores a late catalog and does not save a model",async()=>{
  let finish;answers.chat_models=()=>new Promise(resolve=>{finish=resolve;});
  await open();search("luna");$(".model-btn").fire("click");
  finish(offered);await flush();
  assert.ok(!$(".chat-body").classList.contains("picking"));
  assert.deepEqual(models(),[]);assert.equal(sent("save_settings").length,0);
});

test("typing a search preserves the missing-key explanation",async()=>{
  State.settings={...State.settings,chatProvider:"anthropic"};view.sync();answers.secret_present=false;
  await open();search("sonnet");
  assert.match($(".picker-status").textContent,/No API key/);
  assert.equal(sent("chat_models").length,0);
});
