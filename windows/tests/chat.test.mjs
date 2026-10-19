// The chat view (src/views/chat.ts) on a fake DOM, through the real bridge:
// the model switcher asks for a provider's models only once it is picked and
// has a key, picking saves the settings, and a streamed answer grows in place.

import { beforeEach, test } from "node:test";
import assert from "node:assert/strict";
import { installFakeDom } from "./fakedom.mjs";
import { calls, emit, internals, sent } from "./tauri.mjs";

installFakeDom();
const { buildPrompt } = await import("../src/views/chat.ts");
const { DEFAULT_SETTINGS, State } = await import("../src/core/state.ts");

/** What the mocked Rust side answers, by command. */
let answers;
const plainInvoke = internals.invoke;
internals.invoke = async (cmd, args) => {
  const result = await plainInvoke(cmd, args);
  if (cmd in answers) return typeof answers[cmd] === "function" ? answers[cmd](args) : answers[cmd];
  return result;
};

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

let view;
beforeEach(() => {
  calls.length = 0;
  answers = {};
  State.settings = { ...DEFAULT_SETTINGS, chatModels: {} };
  State.chatHistory = [];
  State.stateOverride = null;
  State.chatBusyProviders.clear();
  State.preparedChat=null;
  State.view = "prompt";
  State.droppedFile = null;
  view = buildPrompt(() => {});
  view.sync();
});

const $ = (cls) => view.el.querySelector(cls);
const chips = () => view.el.find(".picker-chip").map((c) => c.textContent);
const models = () => view.el.find(".picker-model").map((m) => m.textContent);

test("the model button shows the active provider's model", () => {
  assert.equal($(".model-name").textContent, "claude-opus-5");
  State.settings = { ...State.settings, chatProvider: "google" };
  view.sync();
  assert.equal($(".model-name").textContent, "gemini-2.0-flash");
  State.settings = { ...State.settings, chatProvider: "ollama" };
  view.sync();
  assert.equal($(".model-name").textContent, "Perfil definido para o pet");
});

test("a provider without a key is never asked for its models", async () => {
  answers.secret_present = false;
  $(".model-btn").fire("click");
  await flush();
  assert.ok($(".chat-body").classList.contains("picking"));
  assert.deepEqual(chips(), ["Anthropic", "Google", "OpenAI", "OpenRouter", "Codex", "Antigravity", "Copilot"]);
  assert.deepEqual(sent("secret_present"), [{ key: "anthropic-api-key" }]);
  assert.deepEqual(sent("chat_models"), []);
  assert.match($(".picker-status").textContent, /No API key/);
});

test("with a key, the models are listed and picking one saves it", async () => {
  answers.secret_present = true;
  answers.chat_models = [
    { id: "claude-opus-5", label: "Claude Opus 5" },
    { id: "claude-sonnet-5", label: "Claude Sonnet 5" },
  ];
  $(".model-btn").fire("click");
  await flush();
  assert.deepEqual(sent("chat_models"), [{ provider: "anthropic" }]);
  assert.deepEqual(models(), ["Claude Opus 5", "Claude Sonnet 5"]);
  view.el.find(".picker-model")[1].fire("click");
  assert.equal(State.settings.model, "claude-sonnet-5");
  assert.equal(sent("save_settings").at(-1).settings.model, "claude-sonnet-5");
  assert.ok(!$(".chat-body").classList.contains("picking"));
  assert.equal($(".model-name").textContent, "claude-sonnet-5");
});

test("switching provider saves it and asks the new provider only", async () => {
  answers.secret_present = (args) => args.key === "google-api-key";
  answers.chat_models = (args) => (args.provider === "google" ? [{ id: "gemini-2.5-flash", label: "gemini-2.5-flash" }] : []);
  $(".model-btn").fire("click");
  await flush();
  assert.deepEqual(sent("chat_models"), []);
  view.el.find(".picker-chip")[1].fire("click");
  await flush();
  assert.equal(State.settings.chatProvider, "google");
  assert.deepEqual(sent("chat_models"), [{ provider: "google" }]);
  // The saved model was not offered: the flash one is kept instead, and saved.
  assert.equal(State.settings.chatModels.google, "gemini-2.5-flash");
  assert.equal(sent("save_settings").at(-1).settings.chatModels.google, "gemini-2.5-flash");
  assert.deepEqual(models(), ["gemini-2.5-flash"]);
});

test("a local answer streams into one reply, then the finished text replaces it", async () => {
  let finish;
  answers.chat_send = () => new Promise((resolve) => (finish = resolve));
  const input = $(".chat-input");
  input.value = "hello";
  $(".send-btn").fire("click");
  await flush();
  view.sync(); // what State.notify() does in the island
  assert.ok($(".model-btn").disabled, "no switching mid-answer");
  assert.ok($(".typing"), "dots until the first visible text");

  emit("chat-delta", ""); // still thinking
  assert.ok($(".typing"));
  emit("chat-delta", "Hel");
  emit("chat-delta", "Hello **there**");
  view.sync(); // another view update mid-stream must not wipe the live answer
  assert.equal($(".typing"), null);
  const replies = view.el.find(".reply");
  assert.equal(replies.length, 1);
  assert.equal(replies[0].find("STRONG")[0].textContent, "there");

  finish({ text: "Hello **there**!" });
  await flush();
  view.sync();
  assert.equal(view.el.find(".reply").length, 1);
  assert.equal(view.el.find(".reply")[0].textContent, "Hello there!");
  assert.deepEqual(State.chatHistory.map((m) => m.role), ["user", "assistant"]);
  assert.ok(!$(".model-btn").disabled);
});

test("Codex uses the signed-in account without asking for an API key", async () => {
  answers.secret_present = false;
  answers.chat_models = (args) => args.provider === "codex" ? [{ id: "account-model", label: "My Codex model" }] : [];
  $(".model-btn").fire("click");
  await flush();
  const chip = view.el.find(".picker-chip").find((c) => c.textContent === "Codex");
  assert.ok(chip);
  chip.fire("click");
  await flush();
  assert.equal(State.settings.chatProvider, "codex");
  assert.deepEqual(sent("chat_models"), [{ provider: "codex" }]);
  assert.equal(sent("secret_present").length, 1, "only the initial Anthropic provider asked for a key");
  assert.equal(State.settings.chatModels.codex, undefined,"opening the catalog preserves the default automatic pet profile");
  assert.ok(models().includes("Automático · rápido ou raciocínio"));
  view.el.find(".picker-model").find(row=>row.textContent==="My Codex model").fire("click");
  assert.equal(State.settings.chatModels.codex,"account-model","an explicit manual choice is retained");
});

test("Codex login errors stay visible without switching to a paid API provider", async () => {
  State.settings = { ...State.settings, chatProvider: "codex" };
  answers.chat_models = () => { throw new Error("Sign in with codex login"); };
  $(".model-btn").fire("click");
  await flush();
  assert.match($(".picker-status").textContent, /codex login/);
  assert.equal(State.settings.chatProvider, "codex");
  assert.deepEqual(sent("secret_present"), []);
});

test("native CLI permissions are answered explicitly and command text stays plain", async () => {
  await flush();
  emit("agent-request", { id: "req-1", provider: "Codex", kind: "permission", detail: "echo '<script>hi</script>'" });
  const card = $(".cli-request");
  assert.ok(card);
  assert.match(card.textContent, /<script>hi<\/script>/);
  assert.equal(card.find("SCRIPT").length, 0);
  assert.deepEqual(sent("cli_agent_answer"), []);
  card.querySelector(".cli-allow").fire("click");
  await flush();
  assert.deepEqual(sent("cli_agent_answer"), [{ id: "req-1", answer: { allow: true, answers: {} } }]);
  assert.equal($(".cli-request"), null);
});

test("CLI questions retain their ids and answers, and expired cards disappear", async () => {
  await flush();
  emit("agent-request", { id: "req-2", provider: "Codex", kind: "question", detail: "", questions: [{ id: "destination", question: "Where should it go?" }] });
  $(".cli-request").querySelector(".chat-input").value = "/tmp/my-project";
  $(".cli-allow").fire("click");
  await flush();
  assert.deepEqual(sent("cli_agent_answer")[0], { id: "req-2", answer: { allow: true, answers: { destination: ["/tmp/my-project"] } } });
  emit("agent-request", { id: "req-3", provider: "Codex", kind: "permission", detail: "write" });
  emit("agent-request-ended", { id: "req-3" });
  assert.equal($(".cli-request"), null);
});

test("Antigravity selects models from its signed-in CLI account", async () => {
  State.settings = { ...State.settings, chatProvider: "antigravity" };
  answers.chat_models = [{ id: "claude-sonnet-4-6", label: "Claude Sonnet 4.6 (Thinking)" }];
  $(".model-btn").fire("click");
  await flush();
  assert.deepEqual(sent("secret_present"), []);
  assert.deepEqual(sent("chat_models"), [{ provider: "antigravity" }]);
  assert.equal(State.settings.chatModels.antigravity, undefined);
  view.el.find(".picker-model").find(row=>row.textContent==="Claude Sonnet 4.6 (Thinking)").fire("click");
  assert.equal(State.settings.chatModels.antigravity,"claude-sonnet-4-6");
});

test("a CLI turn can be stopped without showing a failure screen", async () => {
  State.settings = { ...State.settings, chatProvider: "codex" };
  answers.chat_conversation={id:'stop-exact',messages:[],busy:false,turn:0};view.sync();await flush();
  let reject;
  answers.chat_send = () => new Promise((_, no) => (reject = no));
  $(".chat-input").value = "run this task";
  $(".send-btn").fire("click");
  await flush();
  view.sync();
  assert.equal($(".send-btn").disabled,true,'Stop is unavailable until the native turn identity is known');
  emit('chat-turn-state',{provider:'codex',conversationId:'stop-exact',turn:1,busy:true,query:'run this task'});view.sync();
  assert.equal($(".send-btn").title, "Stop");
  $(".send-btn").fire("click");
  await flush();
  assert.deepEqual(sent("chat_cancel"),[{provider:'codex',conversationId:'stop-exact',turn:1}]);
  reject(new Error("The CLI request was cancelled."));
  await flush();
  assert.equal(State.view, "prompt");
});


test("switching pets closes the old model picker and keeps each draft with its owner", async () => {
  State.settings = { ...State.settings, companionMode: true, chatProvider: "ollama", ollamaUrl: "http://localhost:11434" };
  State.focusId = "agent_codex";
  view.sync();
  let finishModels;
  answers.chat_models = () => new Promise(resolve => { finishModels = resolve; });
  $(".chat-input").value = "Codex draft";
  $(".model-btn").fire("click");
  await flush();
  assert.deepEqual(chips(), ["Codex"]);
  State.focusId = "ai_ollama";
  view.sync();
  assert.equal($(".chat-input").value, "");
  assert.ok(!$(".chat-body").classList.contains("picking"));
  finishModels([{id:"late-codex-model",label:"Late Codex model"}]);
  await flush();
  assert.deepEqual(sent("save_settings"), [], "a stale model response cannot modify another pet's settings");
  $(".chat-input").value = "Ollama draft";
  answers.chat_models = [{id:"local-model",label:"Local model"}];
  $(".model-btn").fire("click");
  await flush();
  assert.deepEqual(chips(), ["Ollama"]);
  State.focusId = "agent_codex";
  view.sync();
  assert.equal($(".chat-input").value, "Codex draft");
  assert.ok(!$(".chat-body").classList.contains("picking"));
  assert.equal(State.settings.chatProvider, "ollama", "pet navigation preserves the global voice provider");
});


test("restored CLI history and voice follow-ups belong to their exact pet",async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama"};State.focusId="agent_codex";State.chatHistory=[];
 answers.chat_conversation=({provider})=>({id:`dialogue-${provider}`,messages:provider==="codex"?[{role:"user",content:"voice task"},{role:"assistant",content:"ready"}]:[]});
 view.sync();await flush();view.sync();assert.deepEqual(State.chatHistory.map(m=>m.content),["voice task","ready"]);
 State.focusId="agent_antigravity";State.chatHistory=[];view.sync();await flush();assert.equal(State.chatHistory.length,0);
 State.focusId="agent_codex";view.sync();await flush();assert.equal(State.chatHistory.length,2);
 await flush();emit("assistant-history-changed","codex");await flush();assert.equal(State.chatHistory.length,2);
});

test("late history loading cannot resurrect a chat the user explicitly cleared",async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:"codex"};State.chatHistory=[];
 let finish;answers.chat_conversation=()=>new Promise(resolve=>finish=messages=>resolve({id:"old",messages}));
 view.sync();await flush();State.chatHistory=[];
 finish([{role:"user",content:"discarded task"}]);await flush();assert.deepEqual(State.chatHistory,[]);
});

test("different written pets work concurrently and keep streamed replies and Stop controls with their owners",async()=>{
 const finish=new Map(),native=new Map();answers.chat_send=({provider,query})=>new Promise(resolve=>finish.set(provider,reply=>{native.set(provider,[{role:'user',content:query},{role:'assistant',content:reply.text}]);resolve(reply);}));
 answers.chat_conversation=({provider})=>({id:`parallel-${provider}`,messages:native.get(provider)??[],busy:false,turn:native.has(provider)?provider==='codex'?1:2:0});
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama"};State.focusId="agent_codex";State.chatHistory=[];view.sync();await flush();
 $(".chat-input").value="codex task";$(".send-btn").fire("click");await flush();emit('chat-turn-state',{provider:'codex',conversationId:'parallel-codex',turn:1,busy:true,query:'codex task'});view.sync();
 assert.equal($(".send-btn").title,"Stop");
 State.focusId="agent_antigravity";State.chatHistory=[];view.sync();await flush();
 assert.equal($(".chat-input").disabled,false);$(".chat-input").value="gravity task";$(".send-btn").fire("click");await flush();emit('chat-turn-state',{provider:'antigravity',conversationId:'parallel-antigravity',turn:2,busy:true,query:'gravity task'});view.sync();
 assert.deepEqual(sent("chat_send").map(a=>a.provider),["codex","antigravity"]);
 emit("chat-delta",{provider:"codex",text:"codex progress"});
 emit("chat-delta",{provider:"antigravity",text:"gravity progress"});
 assert.match($(".chat-log").textContent,/gravity progress/);assert.doesNotMatch($(".chat-log").textContent,/codex progress/);
 State.focusId="agent_codex";view.sync();assert.match($(".chat-log").textContent,/codex progress/);
 $(".send-btn").fire("click");await flush();assert.deepEqual(sent("chat_cancel").at(-1),{provider:"codex",conversationId:'parallel-codex',turn:1});
 finish.get("antigravity")({text:"gravity done"});await flush();view.sync();assert.equal($(".send-btn").title,"Stop");assert.match($(".chat-log").textContent,/codex progress/);
 finish.get("codex")({text:"codex done"});await flush();view.sync();
 assert.deepEqual(State.chatHistoryFor("codex").map(m=>m.content),["codex task","codex done"]);
 assert.deepEqual(State.chatHistoryFor("antigravity").map(m=>m.content),["gravity task","gravity done"]);
 assert.equal(State.chatBusyProviders.size,0);
});

test("one Gravity pet selects every native model family without extra pets or API keys",async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama"};State.focusId="agent_antigravity";view.sync();
 answers.chat_models=[{id:"gemini-example",label:"gemini-example"},{id:"claude-example",label:"claude-example"}];
 for(const model of ["gemini-example","claude-example"]){
  $(".model-btn").fire("click");await flush();assert.equal(sent("chat_models").at(-1).provider,"antigravity");
  view.el.find(".picker-model").find(row=>row.textContent===model).fire("click");assert.equal(State.settings.chatModels.antigravity,model);
 }
 assert.deepEqual(sent("secret_present"),[]);assert.equal(State.settings.chatModels['gemini-cli'],undefined);assert.equal(State.settings.chatModels['claude-cli'],undefined);
});

test("a monitor pet cannot silently submit a task to the globally selected AI",async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"codex"};State.focusId="agent_cursor";view.sync();
 $(".chat-input").value="must not run";$(".send-btn").fire("click");await flush();
 assert.deepEqual(sent("chat_send"),[]);assert.equal($(".chat-bar").hidden,true);
 assert.match($(".pet-chat-unavailable").textContent,/sem chat integrado/);
});

test("a folder is validated and saved for its original pet even if the user switches pets while saving",async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama",cliWorkspace:"/tmp/shared",cliWorkspaces:{antigravity:"/tmp/gravity"}};State.focusId="agent_codex";view.sync();
 $(".chat-folder").fire("click");$(".chat-folder-input").value="/tmp/codex";
 let finish;answers.assistant_set_workspace=()=>new Promise(resolve=>finish=resolve);
 $(".chat-folder-editor").find("BUTTON")[0].fire("click");await flush();
 State.focusId="agent_antigravity";view.sync();assert.equal($(".chat-folder-editor").hidden,true);
 finish("/tmp/codex");await flush();
 assert.deepEqual(sent("assistant_set_workspace"),[{provider:"codex",path:"/tmp/codex"}]);
 assert.equal(State.settings.cliWorkspaces.codex,"/tmp/codex");assert.equal(State.settings.cliWorkspaces.antigravity,"/tmp/gravity");assert.equal(State.settings.cliWorkspace,"/tmp/shared");
});

test("prepared service requests stay editable and never overwrite an existing user draft or auto-submit",async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true};State.focusId="agent_codex";view.sync();
 $(".chat-input").value="my own draft";State.preparedChat={provider:"codex",source:"GitHub",text:"review CI"};view.sync();
 assert.equal($(".chat-input").value,"my own draft");assert.equal($(".prepared-chat").hidden,false);assert.deepEqual(sent("chat_send"),[]);
 $(".prepared-chat").fire("click");assert.equal($(".chat-input").value,"review CI");assert.equal(State.preparedChat,null);assert.deepEqual(sent("chat_send"),[]);
 $(".chat-input").value="";State.preparedChat={provider:"codex",source:"n8n",text:"review flows"};view.sync();assert.equal($(".chat-input").value,"review flows");assert.equal(State.preparedChat,null);assert.deepEqual(sent("chat_send"),[]);
});

test('reset acknowledgement blocks only that pet and reset failures stay outside model messages',async()=>{
  State.settings={...State.settings,chatProvider:'codex'};State.replaceChatHistory('codex',[]);
  State.chatResettingProviders.add('codex');State.chatFeedback.set('codex','O pet ainda está a trabalhar.');view.sync();
  assert.ok($('.chat-input').disabled);assert.ok($('.send-btn').disabled);
  assert.equal($('.chat-feedback').textContent,'O pet ainda está a trabalhar.');
  $('.chat-input').value='não enviar';$('.chat-input').fire('keydown',{key:'Enter'});await flush();
  assert.equal(sent('chat_send').length,0);assert.deepEqual(State.chatHistory,[]);
  State.settings={...State.settings,chatProvider:'antigravity'};view.sync();
  assert.equal($('.chat-input').disabled,false);assert.ok($('.chat-feedback').hidden);
  State.chatResettingProviders.clear();State.chatFeedback.clear();
});

test('a Codex attachment cannot be sent with Gravity and a late copy keeps its original owner',async()=>{
  answers.chat_send={text:'ok'};
  State.setFileContext('codex',{name:'private.txt',path:'/tmp/original-private.txt'});
  State.settings={...State.settings,chatProvider:'antigravity'};State.replaceChatHistory('antigravity',[]);State.clearDialogueContext('antigravity');view.sync();
  $('.chat-input').value='pedido Gravity';$('.send-btn').fire('click');await flush();
  assert.equal(sent('chat_send').at(-1).provider,'antigravity');assert.equal(sent('chat_send').at(-1).context,null);
  State.setFileContext('codex',{name:'private.txt',path:'/tmp/inbox/private.txt'});view.sync();
  assert.equal(State.droppedFile,null);assert.ok(!$('.chip-row').textContent.includes('private.txt'));
  State.settings={...State.settings,chatProvider:'codex'};State.replaceChatHistory('codex',[]);view.sync();
  assert.match($('.chip-row').textContent,/private.txt/);
  $('.chat-input').value='pedido Codex';$('.send-btn').fire('click');await flush();
  assert.deepEqual(sent('chat_send').at(-1).context,{kind:'file',name:'private.txt',path:'/tmp/inbox/private.txt'});
  assert.equal(sent('chat_send').at(-1).provider,'codex');State.clearDialogueContext('codex');
});


test("all eight service pets restore only their own public conversation",async()=>{
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama"};
 const owners=["integration_github","integration_spotify","integration_n8n","integration_notion","integration_vercel","integration_resend","integration_calcom","integration_stripe"];
 answers.chat_conversation=({provider})=>({id:`dialogue-${provider}`,messages:[{role:"user",content:`saved ${provider}`},{role:"assistant",content:`reply ${provider}`}]});
 for(const pet of owners){State.focusId=pet;State.chatHistory=[];view.sync();await flush();view.sync();assert.deepEqual(State.chatHistory.map(m=>m.content),[`saved ${pet}`,`reply ${pet}`]);}
 assert.equal(State.settings.chatProvider,"ollama");assert.deepEqual(sent("chat_send"),[]);
});

test("profile model and thinking effort are visible without opening the picker",async()=>{
 const {PetPreferences}=await import("../src/mochi/pet-preferences.ts");
 State.settings={...DEFAULT_SETTINGS,companionMode:true,chatProvider:"ollama",chatModels:{codex:"@default"}};State.focusId="agent_codex";
 PetPreferences.update({version:1,reasoningTerms:[],reasoningLength:1600,pets:{agent_codex:{name:"Codex",aliases:["codex"],provider:"codex",mode:"auto",fast:{model:"gpt-6-luna",effort:"low"},reasoning:{model:"gpt-6.1-sol",effort:"high"}}}});view.sync();
 assert.match($(".model-name").textContent,/gpt-6-luna.*low.*gpt-6.1-sol.*high/);assert.match($(".model-btn").getAttribute("aria-label"),/Automático/);
});

test('Ollama voice history is hydrated with a conversation id and written replies use that exact channel',async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:'ollama'};State.chatHistory=[];
 answers.chat_conversation=()=>({id:'ollama-owned',messages:[{role:'user',content:'spoken'},{role:'assistant',content:'saved answer'}]});
 answers.chat_send={text:'written answer'};
 view.sync();await flush();view.sync();
 assert.deepEqual(State.chatHistory.map(m=>m.content),['spoken','saved answer']);
 $('.chat-input').value='continue in writing';$('.send-btn').fire('click');await flush();
 assert.equal(sent('chat_send').at(-1).conversationId,'ollama-owned');
 assert.equal(sent('chat_send').at(-1).provider,'ollama');
});

test('conversation library searches within its captured pet and restores only after native acknowledgement',async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:'ollama'};State.chatHistory=[];
 answers.chat_conversation=()=>({id:'current',messages:[{role:'user',content:'current'}]});
 answers.chat_conversations=[{id:'archived',provider:'ollama',title:'<private title>',preview:'old reply',created:1,updated:2,messageCount:2,current:false}];
 let finish;answers.chat_restore_conversation=()=>new Promise(resolve=>finish=resolve);
 view.sync();await flush();view.sync();
 $('.chat-library-toggle').fire('click');await flush();
 assert.match($('.chat-library-entry').textContent,/<private title>/);
 $('.chat-library-search').value='old';$('.chat-library-search').fire('input');await flush();
 assert.deepEqual(sent('chat_conversations').at(-1),{provider:'ollama',query:'old'});
 $('.chat-library-entry').fire('click');await flush();
 assert.equal(State.chatHistory[0].content,'current');
 finish({id:'archived',messages:[{role:'user',content:'old question'},{role:'assistant',content:'old reply'}]});await flush();view.sync();
 assert.deepEqual(State.chatHistory.map(m=>m.content),['old question','old reply']);assert.equal($('.chat-library').hidden,true);
});

test('late library search results never display another pet after navigation',async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:'ollama'};State.chatHistory=[];view.sync();await flush();
 let finish;answers.chat_conversations=()=>new Promise(resolve=>finish=resolve);
 $('.chat-library-toggle').fire('click');await flush();
 State.settings={...State.settings,chatProvider:'codex'};view.sync();
 finish([{id:'archived',provider:'ollama',title:'private Ollama',preview:'private',created:1,updated:2,messageCount:2,current:false}]);await flush();
 assert.equal($('.chat-library').hidden,true);assert.equal(view.el.find('.chat-library-entry').length,0);
});

test('another window observes the exact active conversation, streams public text and stops its captured turn',async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:'ollama'};State.chatHistory=[];
 let snapshot={id:'observed',messages:[],busy:false,turn:0};answers.chat_conversation=()=>snapshot;
 view.sync();await flush();view.sync();
 emit('chat-turn-state',{provider:'ollama',conversationId:'observed',turn:10,busy:true,query:'spoken question'});view.sync();
 assert.equal($('.chat-input').disabled,false);assert.equal($('.send-btn').textContent,'■');
 emit('chat-delta',{provider:'ollama',conversationId:'observed',turn:10,text:'public streamed answer'});view.sync();
 assert.match($('.chat-log').textContent,/spoken question/);assert.match($('.chat-log').textContent,/public streamed answer/);
 $('.send-btn').fire('click');await flush();assert.deepEqual(sent('chat_cancel').at(-1),{provider:'ollama',conversationId:'observed',turn:10});
 snapshot={id:'observed',turn:10,busy:false,messages:[{role:'user',content:'spoken question'},{role:'assistant',content:'finished'}]};
 emit('chat-turn-state',{provider:'ollama',conversationId:'observed',turn:10,busy:false,query:''});await flush();view.sync();
 assert.equal($('.chat-input').disabled,false);assert.equal(State.chatBusyProviders.has('ollama'),false);
 assert.equal(view.el.find('.reply').length,1);assert.match($('.chat-log').textContent,/finished/);
 assert.equal(sent('chat_send').length,0);
});

test('opening midway hydrates busy state and stream while old turn events cannot stop the newer observation',async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:'ollama'};State.chatHistory=[];
 answers.chat_conversation=()=>({id:'midway',messages:[],busy:true,turn:20,query:'question in progress',stream:'answer so far'});
 view.sync();await flush();view.sync();
 assert.equal($('.chat-input').disabled,false);assert.match($('.chat-log').textContent,/answer so far/);
 emit('chat-turn-state',{provider:'ollama',conversationId:'midway',turn:21,busy:true,query:'newer question'});view.sync();
 emit('chat-delta',{provider:'ollama',conversationId:'midway',turn:21,text:'newer answer'});
 emit('chat-delta',{provider:'ollama',conversationId:'wrong-archive',turn:21,text:'private other conversation'});
 emit('chat-turn-state',{provider:'ollama',conversationId:'midway',turn:20,busy:false,query:''});view.sync();
 assert.equal($('.chat-input').disabled,false);assert.match($('.chat-log').textContent,/newer answer/);
 assert.doesNotMatch($('.chat-log').textContent,/private other conversation/);
 $('.send-btn').fire('click');await flush();assert.equal(sent('chat_cancel').at(-1).turn,21);
});

test('the pet composer keeps Shift Enter and IME confirmation in the draft',async()=>{
 answers.chat_conversation={id:'explicit-multiline',messages:[],busy:false,turn:0};view.sync();await flush();
 const input=$('.chat-input');assert.equal(input.tagName,'TEXTAREA');input.value='Rascunho multilinha';
 assert.equal(input.fire('keydown',{key:'Enter',shiftKey:true,isComposing:false}).defaultPrevented,false);
 assert.equal(input.fire('keydown',{key:'Enter',shiftKey:false,isComposing:true}).defaultPrevented,false);
 await flush();assert.equal(sent('chat_send').length,0);assert.equal(input.value,'Rascunho multilinha');
 answers.chat_send={text:'Resposta sintética'};
 assert.equal(input.fire('keydown',{key:'Enter',shiftKey:false,isComposing:false}).defaultPrevented,true);
 await flush();assert.equal(sent('chat_send').length,1);
});
test('a refused stop is visible on its own pet and keeps the next draft',async()=>{
 State.settings={...DEFAULT_SETTINGS,chatProvider:'codex'};State.chatFeedback.clear();
 answers.chat_conversation={id:'stop-error',messages:[],busy:true,turn:37,query:'Pedido em curso'};
 answers.chat_cancel=()=>{throw Error('Interrupção sintética recusada');};view.sync();await flush();view.sync();
 $('.chat-input').value='Próximo pedido';assert.equal($('.chat-input').disabled,false);
 $('.send-btn').fire('click');await flush();view.sync();
 assert.equal(State.chatFeedback.get('codex'),'Interrupção sintética recusada');assert.match($('.chat-feedback').textContent,/sintética recusada/);
 assert.equal($('.chat-input').value,'Próximo pedido');assert.equal(sent('chat_send').length,0);
});
