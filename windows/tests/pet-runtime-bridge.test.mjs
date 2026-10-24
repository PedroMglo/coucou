import test from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
test('typed criterion proposals remain data and native completion only receives original references',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return null;};
 const criteria=[{criterionIndex:0,criterionHash:'forged',predicate:{kind:'spotifyPlayback',status:'paused'}}];const refs={petId:'ai_ollama',goalId:'original',leaseId:'lease',revision:'reviewed',planId:'plan',requestId:'review'};
 await Bridge.petRuntimePrepareRecipe({...refs,criteria,goalComplete:true});criteria[0].predicate.status='playing';assert.deepEqual(calls[0],{command:'pet_runtime_prepare_recipe',args:{input:{...refs,criteria:[{criterionIndex:0,predicate:{kind:'spotifyPlayback',status:'paused'}}]}}});
 const completion={petId:refs.petId,goalId:refs.goalId,decisionId:'original-decision',planId:refs.planId,recipeId:'native-recipe',revision:refs.revision};await Bridge.petRuntimeCompleteRecipe({...completion,criteriaVerified:true,goalComplete:true,certificate:{signature:'forged'}});assert.deepEqual(calls[1],{command:'pet_runtime_complete_recipe',args:{input:completion}});assert.equal(calls.length,2);
 }finally{internals.invoke=saved;}
});
test('a missing completion certificate propagates without rerunning the recipe or fabricating success',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});throw Error('native certificate missing');};await assert.rejects(Bridge.petRuntimeCompleteRecipe({petId:'ai_ollama',goalId:'original',decisionId:'original-decision',planId:'plan',recipeId:'native-recipe',revision:'reviewed'}),/native certificate missing/);assert.equal(calls.length,1);assert.equal(calls[0].command,'pet_runtime_complete_recipe');
 }finally{internals.invoke=saved;}
});
test('recipe review and dispatch keep original references and exclude caller supplied operations, grants and completion',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'declined'};};
 const prepare={petId:'ai_ollama',goalId:'original',leaseId:'lease',revision:'reviewed',planId:'plan',requestId:'request'};
 await Bridge.petRuntimePrepareRecipe({...prepare,steps:[{command:'forged'}],target:':1.1'});assert.deepEqual(calls[0],{command:'pet_runtime_prepare_recipe',args:{input:prepare}});
 const confirm={petId:prepare.petId,goalId:prepare.goalId,leaseId:prepare.leaseId,revision:prepare.revision,token:'native-once',approve:true};
 await Bridge.petRuntimeConfirmRecipe({...confirm,scope:{approved:true},permit:'forged'});assert.deepEqual(calls[1],{command:'pet_runtime_confirm_recipe',args:{input:confirm}});
 const run={petId:prepare.petId,goalId:prepare.goalId,leaseId:prepare.leaseId,revision:prepare.revision,recipeId:'native-recipe'};
 await Bridge.petRuntimeRunRecipe({...run,operations:[{action:'play'}],goalComplete:true});assert.deepEqual(calls[2],{command:'pet_runtime_run_recipe',args:{input:run}});assert.equal(calls.length,3);
 }finally{internals.invoke=saved;}
});
test('recipe progress and failed dispatch do not automatically approve, recover or repeat effects',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});if(command==='pet_runtime_run_recipe')throw Error('original recipe unavailable');return {phase:'partial',goalComplete:false,executionAvailable:false};};
 const read={petId:'ai_ollama',goalId:'original',decisionId:'decision',planId:'plan'};assert.equal((await Bridge.petRuntimeRecipeProgress({...read,run:true})).phase,'partial');assert.deepEqual(calls[0],{command:'pet_runtime_recipe_progress',args:{input:read}});
 await assert.rejects(Bridge.petRuntimeRunRecipe({petId:read.petId,goalId:read.goalId,leaseId:'lease',revision:'reviewed',recipeId:'original'}),/original recipe unavailable/);assert.equal(calls.length,2);
 }finally{internals.invoke=saved;}
});
test('step review, approval, execution and recovery send references without supplied authority or effects',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return command==='pet_runtime_confirm_step'?{status:'declined'}:null;};
 const refs={petId:'ai_ollama',goalId:'original',leaseId:'lease',revision:'revision',planId:'plan',stepId:'step',requestId:'review'};
 await Bridge.petRuntimePrepareStep({...refs,operation:{action:'play'},target:':1.1',scope:{approved:true}});
 assert.deepEqual(calls[0],{command:'pet_runtime_prepare_step',args:{input:refs}});
 const confirmed={petId:refs.petId,goalId:refs.goalId,leaseId:refs.leaseId,revision:refs.revision,token:'native-token',approve:false};
 assert.equal((await Bridge.petRuntimeConfirmStep({...confirmed,permit:'forged'})).status,'declined');assert.deepEqual(calls[1],{command:'pet_runtime_confirm_step',args:{input:confirmed}});
 const run={petId:refs.petId,goalId:refs.goalId,leaseId:refs.leaseId,grantId:'native-grant',planId:refs.planId,stepId:refs.stepId};
 await Bridge.petRuntimeRunStep({...run,effectVerified:true,arguments:{action:'play'}});assert.deepEqual(calls[2],{command:'pet_runtime_run_step',args:{input:run}});
 const read={petId:refs.petId,goalId:refs.goalId,decisionId:'native-decision',planId:refs.planId,stepId:refs.stepId};
 await Bridge.petRuntimeStepResult({...read,run:true});assert.deepEqual(calls[3],{command:'pet_runtime_step_result',args:{input:read}});assert.equal(calls.length,4);
 }finally{internals.invoke=saved;}
});
test('missing step evidence propagates without automatically approving or re-executing',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});throw Error('recovery required');};
 await assert.rejects(Bridge.petRuntimeStepResult({petId:'ai_ollama',goalId:'original',decisionId:'decision',planId:'plan',stepId:'step'}),/recovery required/);assert.equal(calls.length,1);assert.equal(calls[0].command,'pet_runtime_step_result');
 }finally{internals.invoke=saved;}
});
test('recorded native result recovery is explicit and sends references, never a supplied observation or permission',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {phase:'started',result:null,recovery:{status:'resultRecorded',budgetPhase:'started',recordedEffectVerified:true,reconciliationAvailable:true},executionAvailable:false};};
 const read={petId:'ai_ollama',goalId:'original',decisionId:'native-decision',planId:'plan',stepId:'step'};
 const result=await Bridge.petRuntimeStepResult(read);assert.equal(result.recovery.status,'resultRecorded');assert.equal(result.executionAvailable,false);assert.equal(calls.length,1);
 const recover={...read,revision:'current-revision',grantId:'original-grant'};
 await Bridge.petRuntimeRecoverStep({...recover,result:{effectVerified:true},attempt:{phase:'settled'},run:true,permit:'forged'});
 assert.deepEqual(calls[1],{command:'pet_runtime_recover_step',args:{input:recover}});assert.equal(calls.length,2);
 }finally{internals.invoke=saved;}
});
test('failed evidence recovery is not retried or converted into another tool call',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});throw Error('native evidence unavailable');};
 await assert.rejects(Bridge.petRuntimeRecoverStep({petId:'ai_ollama',goalId:'original',decisionId:'native-decision',planId:'plan',stepId:'step',revision:'current',grantId:'original-grant'}),/native evidence unavailable/);
 assert.equal(calls.length,1);assert.equal(calls[0].command,'pet_runtime_recover_step');
 }finally{internals.invoke=saved;}
});
test('typed plan proposals freeze exact arguments without transmitting invented authority',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'durable'};};
 const operation={operationId:'spotify.control',version:1,arguments:{action:'set_volume',volume:17},approved:true,permit:'forged'};
 const input={petId:'ai_ollama',goalId:'original',revision:'original-revision',requestId:'plan-once',action:{kind:'proposePlan',taskId:'task',steps:[{description:'Propose volume',postcondition:'Independently observed',operation}]}};
 await Bridge.petRuntimeMutate(input);operation.arguments.volume=99;
 assert.deepEqual(calls[0].args.input.action.steps[0],{description:'Propose volume',postcondition:'Independently observed',operation:{operationId:'spotify.control',version:1,arguments:{action:'set_volume',volume:17}}});
 assert.equal(calls[0].command,'pet_runtime_mutate');assert.equal(calls.length,1);
 }finally{internals.invoke=saved;}
});
test('plain and null-operation steps preserve compatibility without creating a tool invocation',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'durable'};};
 await Bridge.petRuntimeMutate({petId:'agent_codex',goalId:'original',revision:'revision',requestId:'plan',action:{kind:'proposePlan',taskId:'task',steps:[{description:'Plain',postcondition:'Check'},{description:'No operation',postcondition:'Check',operation:null}]}});
 assert.deepEqual(calls[0].args.input.action.steps,[{description:'Plain',postcondition:'Check'},{description:'No operation',postcondition:'Check',operation:null}]);assert.equal(calls.length,1);
 }finally{internals.invoke=saved;}
});
test('model execution sends only the original claimed request and never accepts output or completion',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {text:'Saved',criteriaVerified:false,goalComplete:false};};
 const input={petId:'ai_ollama',goalId:'original',leaseId:'original-lease',revision:'original-revision',requestId:'original-request',text:'forged',model:'forged',verified:true,tools:['shell']};
 const result=await Bridge.petRuntimeRespondOnce(input);assert.equal(result.goalComplete,false);
 assert.deepEqual(calls,[{command:'pet_runtime_respond_once',args:{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,requestId:input.requestId}}}]);
 }finally{internals.invoke=saved;}
});
test('result recovery is an explicit read and propagates missing receipts without automatic execution',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});if(args.input.requestId==='unavailable')throw Error('receipt unavailable');return {status:'recoveryRequired',phase:'settledReceiptMissing',criteriaVerified:false,goalComplete:false};};
 const input={petId:'agent_antigravity',goalId:'original',decisionId:'original-decision',requestId:'original-request',run:true,authority:'forged'};
 const result=await Bridge.petRuntimeResult(input);assert.equal(result.phase,'settledReceiptMissing');assert.equal(result.criteriaVerified,false);
 assert.deepEqual(calls[0],{command:'pet_runtime_result',args:{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,requestId:input.requestId}}});
 await assert.rejects(Bridge.petRuntimeResult({...input,requestId:'unavailable'}),/receipt unavailable/);assert.equal(calls.length,2);assert.ok(calls.every(c=>c.command==='pet_runtime_result'));
 }finally{internals.invoke=saved;}
});
test('lease preparation preserves original review scope and never accepts a serialized worker permit',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {owned:true,executionAvailable:false};};
 const result=await Bridge.petRuntimeClaim({petId:'agent_antigravity',goalId:'original',revision:'queued-revision',requestId:'claim-once',controllerId:'forged',leaseId:'forged',model:'forged',run:true});
 assert.equal(result.executionAvailable,false);assert.deepEqual(calls,[{command:'pet_runtime_claim',args:{input:{petId:'agent_antigravity',goalId:'original',revision:'queued-revision',requestId:'claim-once'}}}]);
 }finally{internals.invoke=saved;}
});
test('an unowned stored lease has unknown workers and release targets the original lease',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return command==='pet_runtime_claim_status'?{owned:false,current:false,workers:null,executionAvailable:false}:{status:'uncertain'};};
 const status=await Bridge.petRuntimeClaimStatus('agent_codex','original');assert.equal(status.workers,null);assert.equal(status.owned,false);assert.equal(status.executionAvailable,false);assert.equal((await Bridge.petRuntimeReleaseClaim('agent_codex','original','original-lease')).status,'uncertain');
 assert.deepEqual(calls[1],{command:'pet_runtime_release_claim',args:{input:{petId:'agent_codex',goalId:'original',leaseId:'original-lease'}}});
 }finally{internals.invoke=saved;}
});
test('native consent provenance query remains read-only and does not imply execution readiness',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {verified:true,reason:null,executionAvailable:false};};
 const status=await Bridge.petRuntimeAdmissionStatus('agent_antigravity','original-goal');assert.equal(status.verified,true);assert.equal(status.executionAvailable,false);assert.deepEqual(calls,[{command:'pet_runtime_admission_status',args:{petId:'agent_antigravity',goalId:'original-goal'}}]);
 }finally{internals.invoke=saved;}
});
test('admission review retains original budget and excludes injected authority',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'blocked',reason:'notReady'};};
 const budget={deadlineAt:200000,maxModelCalls:2,maxToolCalls:0,maxOutputBytes:4096,payment:'localOnly',grant:true,provider:'copilot'};
 const result=await Bridge.petRuntimePrepareAdmission({petId:'ai_ollama',goalId:'original',revision:'original-revision',requestId:'review-once',budget,alwaysAllow:true});
 assert.equal(result.reason,'notReady');assert.deepEqual(calls[0],{command:'pet_runtime_prepare_admission',args:{input:{petId:'ai_ollama',goalId:'original',revision:'original-revision',requestId:'review-once',budget:{deadlineAt:200000,maxModelCalls:2,maxToolCalls:0,maxOutputBytes:4096,maxTokens:null,maxCostMicros:null,payment:'localOnly'}}}});
 }finally{internals.invoke=saved;}
});
test('admission confirmation only consumes the original exact review token',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'queued',receipt:{status:'uncertain'}};};
 const result=await Bridge.petRuntimeConfirmAdmission({petId:'agent_antigravity',goalId:'original',revision:'reviewed',token:'single-use',approve:true,provider:'codex',model:'injected',deadlineAt:999999,run:true});
 assert.equal(result.receipt.status,'uncertain');assert.deepEqual(calls[0],{command:'pet_runtime_confirm_admission',args:{input:{petId:'agent_antigravity',goalId:'original',revision:'reviewed',token:'single-use',approve:true}}});assert.equal(calls.length,1);
 }finally{internals.invoke=saved;}
});
test('goal creation preserves the original request/context and excludes execution authority',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'uncertain',record:{id:'original'}};};
 const input={petId:'agent_antigravity',requestId:'stable-retry',intent:'Full original intent',criteria:['Verify it'],conversationId:'original-cid',authorization:'approved',provider:'copilot',run:true};
 const result=await Bridge.petRuntimeCreate(input);input.criteria.push('later-change');assert.equal(result.status,'uncertain');assert.deepEqual(calls[0],{command:'pet_runtime_create',args:{input:{petId:'agent_antigravity',requestId:'stable-retry',intent:'Full original intent',criteria:['Verify it'],conversationId:'original-cid'}}});
 }finally{internals.invoke=saved;}
});
test('draft mutations send only explicit proposals and the original compare-and-swap revision',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'durable'};};
 const input={petId:'integration_notion',goalId:'original-goal',revision:'original-revision',requestId:'proposal-once',action:{kind:'proposePlan',taskId:'original-task',steps:[{description:'Review',postcondition:'Confirmed',command:'forbidden'}],run:true}};
 await Bridge.petRuntimeMutate(input);assert.deepEqual(calls[0],{command:'pet_runtime_mutate',args:{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,action:{kind:'proposePlan',taskId:'original-task',steps:[{description:'Review',postcondition:'Confirmed'}]}}}});
 assert.throws(()=>Bridge.petRuntimeMutate({...input,action:{kind:'run'}}),/não está disponível/);assert.equal(calls.length,1);
 }finally{internals.invoke=saved;}
});
test('legacy import sends a reference and cannot inject fabricated receipts or completion',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'durable'};};
 const input={petId:'agent_codex',goalId:'goal',revision:'revision',requestId:'import-once',taskId:'original-task',receipts:[{state:'exited'}],completed:true};
 await Bridge.petRuntimeImportLegacy(input);assert.deepEqual(calls[0],{command:'pet_runtime_import_legacy',args:{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,taskId:input.taskId}}});
 }finally{internals.invoke=saved;}
});
test('reads remain explicit and native refusals are propagated without retry or dispatch',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});if(command==='pet_runtime_read')throw Error('conflict');return {schemaVersion:1,goals:[]};};
 await Bridge.petRuntimeList('agent_copilot');await assert.rejects(Bridge.petRuntimeRead('agent_copilot','original-goal'),/conflict/);
 assert.deepEqual(calls,[{command:'pet_runtime_list',args:{petId:'agent_copilot'}},{command:'pet_runtime_read',args:{petId:'agent_copilot',goalId:'original-goal'}}]);
 }finally{internals.invoke=saved;}
});

test('context preparation requires explicit mode and original goal revision without injected source or authority',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'durable'};};
 const input={petId:'agent_antigravity',goalId:'original-goal',revision:'original-revision',requestId:'stable-context',mode:'snapshot',sourceConversationId:'forbidden-other-context',authority:'approved'};
 await Bridge.petRuntimePrepareContext(input);assert.deepEqual(calls[0],{command:'pet_runtime_prepare_context',args:{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,mode:'snapshot'}}});
 await Bridge.petRuntimeContextRead(input.petId,input.goalId);assert.deepEqual(calls[1],{command:'pet_runtime_context_read',args:{petId:input.petId,goalId:input.goalId}});
 }finally{internals.invoke=saved;}
});

test('shared recipe requests send only references and predicate proposals and never execute implicitly',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return {status:'ended'};};const input={petId:'ai_ollama',goalId:'original',leaseId:'native-lease',revision:'original-revision',planId:'original-plan',requestId:'review'};const result=await Bridge.petRuntimeRequestRecipe({...input,allow:true,execute:true,goalTarget:{origin:'goal'},scope:{grant:true}});assert.deepEqual(result,{status:'ended'});assert.deepEqual(calls,[{command:'pet_runtime_request_recipe',args:{input}}]);}finally{internals.invoke=saved;}
});

test('goal management can only send a goal decision and explicit metadata inspection never accepts a model or endpoint from the caller',async()=>{
 const saved=internals.invoke,calls=[];try{internals.invoke=async(command,args)=>{calls.push({command,args});return [];};const target={origin:'goal',petId:'ai_ollama',provider:'ollama',goalId:'original-goal',contextId:'context',leaseId:'lease',controllerId:'controller',recipeId:'recipe',planId:'plan',revision:'reviewed'};await Bridge.petRuntimeGoalPending();await Bridge.petRuntimeGoalAnswer('original-request',false,target);target.revision='changed-after-send';await Bridge.petRuntimeRefreshReadiness('ai_ollama','original-goal','https://foreign.invalid','foreign-model');assert.deepEqual(calls[0],{command:'pet_runtime_goal_pending',args:{}});assert.deepEqual(calls[1],{command:'pet_runtime_goal_answer',args:{input:{id:'original-request',approve:false,target:{...target,revision:'reviewed'}}}});assert.deepEqual(calls[2],{command:'pet_runtime_refresh_readiness',args:{petId:'ai_ollama',goalId:'original-goal'}});assert.ok(calls.every(c=>!['cli_agent_answer','pet_runtime_run_recipe','pet_runtime_respond_once'].includes(c.command)));}finally{internals.invoke=saved;}
});
