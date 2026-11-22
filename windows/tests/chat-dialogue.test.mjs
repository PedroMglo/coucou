import {beforeEach,test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {calls,internals,sent} from './tauri.mjs';
installFakeDom();
const {State,DEFAULT_SETTINGS}=await import('../src/core/state.ts');
const {resetDialogue,restoreDialogue,dialogueId}=await import('../src/core/chat-dialogue.ts');
const original=internals.invoke;
let reset,restore;
internals.invoke=async(cmd,args)=>{const result=await original(cmd,args);return cmd==='chat_reset'?reset(args):cmd==='chat_restore_conversation'?restore(args):result;};
beforeEach(()=>{
 calls.length=0;reset=()=>undefined;restore=()=>undefined;
 State.settings={...DEFAULT_SETTINGS,companionMode:false,chatProvider:'codex'};
 State.chatBusyProviders.clear();State.chatResettingProviders.clear();State.chatFeedback.clear();
 State.replaceChatHistory('codex',[]);State.replaceChatHistory('antigravity',[]);
 State.clearDialogueContext('codex');State.clearDialogueContext('antigravity');
});
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));

test('new chat waits for native acknowledgement and rejects a voice-owned busy task without losing context',async()=>{
 let reject;reset=()=>new Promise((_resolve,no)=>{reject=no;});
 const history=[{id:1,role:'user',content:'prepara postgres'}];
 State.replaceChatHistory('codex',history);State.setFileContext('codex',{name:'a.txt',path:'/tmp/a.txt'});
 const request=resetDialogue();await tick();
 assert.equal(State.chatHistory,history);assert.ok(State.chatResettingProviders.has('codex'));
 reject(Error('Este pet já está a executar uma tarefa.'));
 assert.equal(await request,false);assert.equal(State.chatHistory,history);
 assert.equal(State.droppedFile.path,'/tmp/a.txt');assert.match(State.chatFeedback.get('codex'),/executar/);
 assert.ok(!State.chatResettingProviders.has('codex'));
});

test('a delayed reset clears only its captured owner after switching pets',async()=>{
 let finish;reset=()=>new Promise(resolve=>{finish=resolve;});
 State.replaceChatHistory('codex',[{id:1,role:'user',content:'codex'}]);
 const other=[{id:2,role:'user',content:'gravity'}];State.replaceChatHistory('antigravity',other);
 State.setFileContext('codex',{name:'a.txt',path:'/tmp/a.txt'});
 State.setFileContext('antigravity',{name:'b.txt',path:'/tmp/b.txt'});
 const request=resetDialogue('codex');await tick();
 State.settings={...State.settings,chatProvider:'antigravity'};finish();assert.equal(await request,true);
 assert.deepEqual(State.chatHistoryFor('codex'),[]);assert.equal(State.chatHistory,other);
 assert.equal(State.fileFor('codex'),null);assert.equal(State.droppedFile.path,'/tmp/b.txt');
 assert.equal(State.promptContext.name,'b.txt');
});

test('duplicate resets share one acknowledgement and written busy work never calls reset',async()=>{
 let finish;reset=()=>new Promise(resolve=>{finish=resolve;});
 const one=resetDialogue('codex'),two=resetDialogue('codex');await tick();
 assert.equal(sent('chat_reset').length,1);finish();assert.deepEqual(await Promise.all([one,two]),[true,true]);
 calls.length=0;State.chatBusyProviders.add('codex');
 assert.equal(await resetDialogue('codex'),false);assert.equal(sent('chat_reset').length,0);
});

test('file and window context never follows the user into another pet',()=>{
 State.setFileContext('codex',{name:'private.txt',path:'/tmp/private.txt'});
 State.settings={...State.settings,chatProvider:'antigravity'};
 assert.equal(State.droppedFile,null);assert.equal(State.promptContext,null);
 State.promptContext={kind:'window',appName:'Demo',title:'Gravity only'};
 State.settings={...State.settings,chatProvider:'codex'};
 assert.equal(State.droppedFile.name,'private.txt');assert.equal(State.promptContext.kind,'file');
});

test('restoring a conversation waits for acknowledgement and replaces only its pet without carrying attachments',async()=>{
 let finish;restore=()=>new Promise(resolve=>finish=resolve);
 const current=[{id:1,role:'user',content:'current'}];State.replaceChatHistory('codex',current);
 State.setFileContext('codex',{name:'a.txt',path:'/tmp/a.txt'});
 const pending=restoreDialogue('codex','archived');await tick();
 assert.equal(State.chatHistory,current);assert.ok(State.chatResettingProviders.has('codex'));
 const other=[{id:2,role:'user',content:'other'}];State.replaceChatHistory('antigravity',other);
 State.settings={...State.settings,chatProvider:'antigravity'};
 finish({id:'archived',messages:[{role:'user',content:'saved'},{role:'assistant',content:'ready'}]});
 assert.equal(await pending,true);assert.equal(dialogueId('codex'),'archived');
 assert.deepEqual(State.chatHistoryFor('codex').map(m=>m.content),['saved','ready']);
 assert.equal(State.chatHistory,other);assert.equal(State.fileFor('codex'),null);
 assert.deepEqual(sent('chat_restore_conversation'),[{provider:'codex',id:'archived'}]);
});

test('a rejected restore preserves the complete displayed conversation and its context',async()=>{
 const current=[{id:1,role:'user',content:'current'}];State.replaceChatHistory('codex',current);
 State.setFileContext('codex',{name:'a.txt',path:'/tmp/a.txt'});
 restore=()=>{throw Error('O pet está ocupado pela voz.');};
 assert.equal(await restoreDialogue('codex','archived'),false);
 assert.equal(State.chatHistory,current);assert.equal(State.droppedFile.name,'a.txt');
 assert.match(State.chatFeedback.get('codex'),/ocupado/);
});
