import test from 'node:test';import assert from 'node:assert/strict';import {installFakeDom} from './fakedom.mjs';import {Bridge} from '../src/core/bridge.ts';import {emit} from './tauri.mjs';import {buildAgentRequests} from '../src/views/agent-requests.ts';
installFakeDom();
test('a late initial snapshot cannot resurrect an already-ended decision',async()=>{
 const original=Bridge.cliAgentPending;let finish;const target={petId:'agent_codex',provider:'codex',origin:'external',sessionId:'native-exact'};
 try{Bridge.cliAgentPending=()=>new Promise(resolve=>finish=resolve);const view=buildAgentRequests(()=>target,()=>{});
 emit('agent-request-ended',{id:'cli-ended'});finish([{id:'cli-ended',provider:'codex',kind:'approval',detail:'Synthetic',target}]);
 for(let i=0;i<8;i++)await Promise.resolve();assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=original;}
});

test('native choice submission stays pinned to its original request and disables every option during delivery',async()=>{
 const previousPending=Bridge.cliAgentPending,previousAnswer=Bridge.cliAgentAnswer;let finish;const sent=[];
 const original={petId:'agent_codex',provider:'codex',origin:'external',sessionId:'native-original'};let selected=original;
 try{
  Bridge.cliAgentPending=async()=>[];Bridge.cliAgentAnswer=(id,answer,target)=>{sent.push({id,answer,target});return new Promise(resolve=>finish=resolve);};const view=buildAgentRequests(()=>selected,()=>{});
  emit('agent-request',{id:'native-choice-request',provider:'codex',kind:'permission',detail:'Synthetic command',target:original,nativeChoices:[{id:'choice-once',scope:'once',label:'Permitir uma vez',allow:true},{id:'choice-session',scope:'session',label:'Permitir nesta sessão',allow:true}]});
  const controls=view.el.find('button');assert.equal(controls.length,2);controls[1].fire('click');selected={...original,sessionId:'another-native'};view.sync();controls[0].fire('click');assert.equal(sent.length,1);assert.deepEqual(sent[0],{id:'native-choice-request',answer:{allow:true,answers:{},nativeChoice:'choice-session'},target:original});assert.ok(controls.every(b=>b.disabled));finish();for(let i=0;i<8;i++)await Promise.resolve();assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=previousPending;Bridge.cliAgentAnswer=previousAnswer;}
});

test('a native choice rejection preserves its exact review for retry and ended events still retire it',async()=>{
 const previousPending=Bridge.cliAgentPending,previousAnswer=Bridge.cliAgentAnswer;const target={petId:'agent_copilot',provider:'copilot',origin:'external',sessionId:'native-copilot'};
 try{Bridge.cliAgentPending=async()=>[];Bridge.cliAgentAnswer=async()=>{throw new Error('The original policy changed');};const view=buildAgentRequests(()=>target,()=>{});
  emit('agent-request',{id:'retry-native-choice',provider:'copilot',kind:'permission',detail:'Synthetic',target,nativeChoices:[{id:'remember',scope:'nativeRemembered',label:'Permitir e memorizar',allow:true}]});const button=view.el.find('button')[0];button.fire('click');for(let i=0;i<8;i++)await Promise.resolve();assert.equal(button.disabled,false);assert.match(view.el.textContent,/original policy changed/);emit('agent-request-ended',{id:'retry-native-choice'});assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=previousPending;Bridge.cliAgentAnswer=previousAnswer;}
});

test('external typed forms preserve option IDs optional omission and the original owner through retry',async()=>{
 const previousPending=Bridge.cliAgentPending,previousAnswer=Bridge.cliAgentAnswer;const target={petId:'agent_codex',provider:'codex',origin:'external',sessionId:'original-form-session'};const sent=[];let selected=target;
 try{Bridge.cliAgentPending=async()=>[];Bridge.cliAgentAnswer=async(id,answer,owner)=>{sent.push({id,answer,owner});if(sent.length===1)throw new Error('Revê os limites do campo');};const view=buildAgentRequests(()=>selected,()=>{});
  emit('agent-request',{id:'typed-external',provider:'codex',kind:'question',nativeForm:'mcp',detail:'MCP fixture',target,questions:[{id:'pick',question:'Pick',inputType:'enum',required:true,options:[{label:'Friendly',value:'opaque-option'}]},{id:'optional',question:'Optional',inputType:'text',required:false}]});
  const select=view.el.find('select')[0];select.value='opaque-option';view.el.find('button').find(b=>b.classList.contains('cli-allow')).fire('click');for(let i=0;i<8;i++)await Promise.resolve();assert.equal(select.disabled,false);assert.equal(select.value,'opaque-option');assert.match(view.el.textContent,/limites do campo/);assert.deepEqual(sent[0],{id:'typed-external',answer:{allow:true,answers:{pick:['opaque-option']}},owner:target});
  selected={...target,sessionId:'another-session'};view.sync();view.el.find('button').find(b=>b.classList.contains('cli-deny')).fire('click');for(let i=0;i<8;i++)await Promise.resolve();assert.deepEqual(sent[1],{id:'typed-external',answer:{allow:false,answers:{pick:[],optional:[]}},owner:target});assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=previousPending;Bridge.cliAgentAnswer=previousAnswer;}
});

test('a background goal review keeps the foreground conversation and sends its distinct original target',async()=>{
 const savedPending=Bridge.cliAgentPending,savedAnswer=Bridge.cliAgentAnswer;const selected={petId:'ai_ollama',provider:'ollama',origin:'mochi',sessionId:'foreground',conversationId:'foreground'};const sent=[];
 const goalTarget={origin:'goal',petId:'ai_ollama',provider:'ollama',goalId:'original-goal',contextId:'private-background',leaseId:'native-lease',controllerId:'native-controller',recipeId:'native-recipe',planId:'reviewed-plan',revision:'reviewed-revision'};
 try{Bridge.cliAgentPending=async()=>[];Bridge.cliAgentAnswer=async(id,answer,target)=>{sent.push({id,answer,target});};const view=buildAgentRequests(()=>selected,()=>{});emit('agent-request',{id:'background-recipe',provider:'ollama',kind:'permission',detail:'Original intent, exact ordered steps, original criteria, predicates, resource, budget and deadline',approvalScope:'recipe',voiceCode:'004321',goalTarget});
 assert.equal(view.el.children.length,1);assert.equal(view.el.children[0].hidden,false);assert.equal(selected.sessionId,'foreground');assert.equal(selected.conversationId,'foreground');assert.match(view.el.textContent,/rever receita 004321/);assert.match(view.el.textContent,/Autorizar esta receita/);assert.doesNotMatch(view.el.textContent,/Permitir neste turno/);
 view.el.find('button').find(b=>b.classList.contains('cli-allow')).fire('click');for(let i=0;i<8;i++)await Promise.resolve();assert.deepEqual(sent,[{id:'background-recipe',answer:{allow:true,answers:{}},target:goalTarget}]);assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=savedPending;Bridge.cliAgentAnswer=savedAnswer;}
});
test('a goal review is scoped to its logical pet and a decision from another surface retires it',async()=>{
 const saved=Bridge.cliAgentPending;let selected={petId:'agent_codex',provider:'codex',origin:'mochi',sessionId:'foreground'};
 try{Bridge.cliAgentPending=async()=>[];const view=buildAgentRequests(()=>selected,()=>{});emit('agent-request',{id:'goal-other-surface',provider:'integration_spotify',kind:'permission',detail:'Native exact recipe',approvalScope:'recipe',goalTarget:{origin:'goal',petId:'integration_spotify',provider:'integration_spotify',goalId:'background-goal',contextId:'private',leaseId:'lease',controllerId:'controller',recipeId:'recipe',planId:'plan',revision:'revision'}});assert.equal(view.el.children[0].hidden,true);selected={...selected,petId:'integration_spotify'};view.sync();assert.equal(view.el.children[0].hidden,false);assert.equal(selected.sessionId,'foreground');emit('agent-request-ended',{id:'goal-other-surface'});assert.equal(view.el.children.length,0);
 }finally{Bridge.cliAgentPending=saved;}
});
