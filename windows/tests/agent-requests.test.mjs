import test from 'node:test';import assert from 'node:assert/strict';import {installFakeDom} from './fakedom.mjs';import {Bridge} from '../src/core/bridge.ts';import {emit} from './tauri.mjs';import {buildAgentRequests} from '../src/views/agent-requests.ts';
installFakeDom();
test('a late initial snapshot cannot resurrect an already-ended decision',async()=>{
 const original=Bridge.cliAgentPending;let finish;const target={petId:'agent_codex',provider:'codex',origin:'external',sessionId:'native-exact'};
 try{Bridge.cliAgentPending=()=>new Promise(resolve=>finish=resolve);const view=buildAgentRequests(()=>target,()=>{});
 emit('agent-request-ended',{id:'cli-ended'});finish([{id:'cli-ended',provider:'codex',kind:'approval',detail:'Synthetic',target}]);
 for(let i=0;i<8;i++)await Promise.resolve();assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=original;}
});
