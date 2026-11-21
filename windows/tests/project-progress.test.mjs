import test from 'node:test';import assert from 'node:assert/strict';
import {installFakeDom}from './fakedom.mjs';import{buildProjectProgress}from '../src/views/project-progress.ts';import{buildMemoryProjects}from '../src/views/memory-projects.ts';
installFakeDom();const flush=async()=>{for(let i=0;i<80;i++)await Promise.resolve();};const copy=structuredClone;
const project=(x={})=>({id:'project-one',petId:'ai_ollama',creationRequestId:'project-choice',revision:2,name:'Chosen project',conversationIds:[],enabled:true,createdAt:100,updatedAt:200,...x});
const summary=(x={})=>({id:'g-one',revision:'revision-one',petId:'ai_ollama',state:'paused',updatedAt:200,intentPreview:'Finite own work',tasks:1,steps:1,project:{id:'project-one',revision:1},...x});
const goal=(x={})=>({id:'g-one',revision:'revision-one',state:'paused',updatedAt:200,schemaVersion:17,spec:{owner:{petId:'ai_ollama',provider:'ollama',adapter:'ollama',modelHint:null},intent:'Original own project work',criteria:['Verify the actual result, not only a saved reply'],context:{kind:'mochi',conversationId:'original-cid'}},executionContext:{sourceConversationId:'original-cid',memory:{schemaVersion:2,references:[],project:project({revision:1,conversationIds:['original-cid']})}},admissions:[{spec:{decisionId:'own-decision',schemaVersion:2}}],tasks:[],plans:[],events:[],legacy:[],...x});
const charge={modelCalls:1,toolCalls:0,outputBytes:65536,unknownModelResults:1,unknownToolResults:0};
const history=(x={})=>({schemaVersion:1,scope:'allRecordedAdmissions',goalId:'g-one',petId:'ai_ollama',goalRevision:'revision-one',goalState:'paused',observedAt:300,executionAvailable:false,continuationAvailable:false,accountingComplete:true,outcomesComplete:false,authenticatedSubtotal:charge,cumulativeCharge:charge,...x});
const page=(rows,extra={})=>({schemaVersion:1,goals:rows,unavailable:0,pendingArtifacts:0,complete:true,...extra});
function fixture(){const t={calls:[],rows:page([summary()]),goal:goal(),history:history(),result:{status:'available',receipt:{schemaVersion:1,petId:'ai_ollama',goalId:'g-one',decisionId:'own-decision',requestId:'response-own-decision',text:'OWN_REPLY'},criteriaVerified:false,goalComplete:false}};return Object.assign(t,{petRuntimeList:async pet=>{t.calls.push(['list',pet]);return copy(t.rows);},petRuntimeRead:async(p,id)=>{t.calls.push(['goal',p,id]);return copy(t.goal);},petRuntimeHistory:async i=>{t.calls.push(['history',copy(i)]);return copy(t.history);},petRuntimeResult:async i=>{t.calls.push(['result',copy(i)]);return copy(t.result);}});}
const button=(root,text)=>root.find('button').find(b=>b.textContent===text);async function load(v){button(v.el,'Consultar objetivos do projeto').fire('click');await flush();}async function inspect(v){button(v.el,'Consultar progresso deste objetivo').fire('click');await flush();}

test('project progress explicitly reads exact native owner/revision, criteria, unknown debit and one persisted response',async()=>{
 const t=fixture(),v=buildProjectProgress({transport:t});v.sync(project());assert.equal(t.calls.length,0);await load(v);assert.deepEqual(t.calls,[['list','ai_ollama']]);assert.match(v.el.textContent,/revisão 1; projeto consultado na revisão 2/);await inspect(v);assert.equal(t.calls.length,3);assert.deepEqual(t.calls[2],['history',{petId:'ai_ollama',goalId:'g-one',revision:'revision-one'}]);assert.match(v.el.textContent,/Verify the actual result/);assert.match(v.el.textContent,/associação atual retirada/);assert.match(v.el.textContent,/Resultados desconhecidos: 1/);assert.match(v.el.textContent,/65536 B debitados/);assert.match(v.el.textContent,/Não há referência de conclusão nativa/);button(v.el,'Consultar resposta persistida').fire('click');await flush();assert.match(v.el.textContent,/OWN_REPLY/);assert.match(v.el.textContent,/critérios de conclusão mantêm/);assert.equal(t.calls.length,4);v.dispose();
});

test('only explicit project IDs link work; incomplete/old listings never imply full coverage or false absence',async()=>{
 const t=fixture(),v=buildProjectProgress({transport:t});t.rows=page([summary(),summary({id:'other',project:{id:'different-project',revision:1},intentPreview:'Unrelated same name'}),summary({id:'legacy',project:null,intentPreview:'Legacy same conversation'})]);v.sync(project());await load(v);assert.equal(v.el.find('article').length,1);assert.doesNotMatch(v.el.textContent,/Unrelated same name|Legacy same conversation/);
 t.rows=page([],{unavailable:2,pendingArtifacts:1,complete:false});await load(v);assert.match(v.el.textContent,/Consulta incompleta: 2 registos indisponíveis/);const old=summary();delete old.project;t.rows=page([old]);await load(v);assert.match(v.el.textContent,/não confirma os projetos/);assert.equal(v.el.find('article').length,0);v.dispose();
});

test('archived restored project history does not display empty live subtotal as zero consumption',async()=>{
 const t=fixture();t.rows=page([summary({historicalSource:true})]);t.history=history({schemaVersion:2,scope:'restoredSnapshotOnly',accountingComplete:false,outcomesComplete:false,cumulativeCharge:null,authenticatedSubtotal:{modelCalls:0,toolCalls:0,outputBytes:0,unknownModelResults:0,unknownToolResults:0},restoredSource:{executionAuthorityGranted:false,postSnapshotStateUnknown:true}});const v=buildProjectProgress({transport:t});v.sync(project({enabled:false}));await load(v);assert.match(v.el.textContent,/arquivado/);await inspect(v);assert.match(v.el.textContent,/Contabilidade atual e resultados posteriores desconhecidos/);assert.doesNotMatch(v.el.textContent,/0 chamadas de modelo/);assert.match(v.el.textContent,/estado posterior ao snapshot é desconhecido/);v.dispose();
});

test('late listing or history cannot fill another pet/project/revision',async()=>{
 let finish;const t=fixture();t.petRuntimeList=()=>new Promise(r=>finish=r);const v=buildProjectProgress({transport:t});v.sync(project());button(v.el,'Consultar objetivos do projeto').fire('click');v.sync(project({petId:'agent_codex',id:'codex-project'}));finish(page([summary()]));await flush();assert.equal(v.el.find('article').length,0);assert.doesNotMatch(v.el.textContent,/Finite own work/);t.petRuntimeList=async()=>page([summary()]);v.sync(project());await load(v);t.petRuntimeHistory=()=>new Promise(r=>finish=r);button(v.el,'Consultar progresso deste objetivo').fire('click');await flush();v.sync(project({revision:3}));finish(history());await flush();assert.equal(v.el.find('article').length,0);assert.doesNotMatch(v.el.textContent,/Verify the actual result/);v.dispose();
});

test('wrong owner, changed goal revision, mismatched history and foreign result are refused without model/state operations',async()=>{
 const t=fixture(),v=buildProjectProgress({transport:t});v.sync(project());t.rows=page([summary({petId:'agent_codex'})]);await load(v);assert.match(v.el.textContent,/não confirma os projetos/);t.rows=page([summary()]);t.goal=goal({revision:'changed'});await load(v);await inspect(v);assert.match(v.el.textContent,/objetivo mudou/i);assert.equal(button(v.el,'Consultar resposta persistida').hidden,true);
 t.goal=goal();t.history=history({goalRevision:'another-revision'});await load(v);await inspect(v);assert.match(v.el.textContent,/histórico não corresponde/);t.history=history();await load(v);await inspect(v);t.result.receipt.petId='agent_codex';t.result.receipt.text='OTHER_PRIVATE_REPLY';button(v.el,'Consultar resposta persistida').fire('click');await flush();assert.doesNotMatch(v.el.textContent,/OTHER_PRIVATE_REPLY/);assert.match(v.el.textContent,/resposta não pertence/i);v.dispose();
});

test('uncertain receipt remains uncertainty and never causes a fresh attempt',async()=>{
 const t=fixture();t.result={status:'recoveryRequired',phase:'uncertain',criteriaVerified:false,goalComplete:false};const v=buildProjectProgress({transport:t});v.sync(project());await load(v);await inspect(v);button(v.el,'Consultar resposta persistida').fire('click');await flush();assert.match(v.el.textContent,/resultado desconhecido ou recibo por recuperar/);assert.equal(t.calls.filter(c=>c[0]==='result').length,1);v.dispose();
});

test('existing project manager requires explicit catalog read and invalidates progress after metadata edit',async()=>{
 const t=fixture(),p=project();let calls=0;const transport={...t,petMemoryProjectList:async()=>{calls++;return{schemaVersion:1,petId:p.petId,revision:'catalog',items:[p],executionAuthorized:false};},petMemoryProjectMutate:async()=>({schemaVersion:1,petId:p.petId,revision:'changed-catalog',project:{...p,revision:3,name:'Explicit new name'},executionAuthorized:false}),petMemoryList:async request=>({schemaVersion:1,owner:request.owner,revision:'notes',items:[],hasMore:false,nextOffset:null}),petMemoryMutate:async()=>{throw Error('no note write');}};const v=buildMemoryProjects({transport});v.sync({petId:p.petId,conversationId:'cid'});assert.equal(t.calls.length,0);assert.equal(button(v.el,'Consultar objetivos do projeto').disabled,true);button(v.el,'Consultar projetos').fire('click');await flush();button(v.el,'Consultar objetivos do projeto').fire('click');await flush();assert.match(v.el.textContent,/Finite own work/);assert.equal(calls,1);v.el.find('input').find(i=>i.getAttribute('aria-label')==='Novo nome do projeto selecionado').value='Explicit new name';button(v.el,'Guardar nome do projeto').fire('click');await flush();assert.equal(button(v.el,'Consultar objetivos do projeto').disabled,true);assert.doesNotMatch(v.el.textContent,/Finite own work/);v.dispose();
});

const progressGoal=(notes=false)=>{
 const captured=project({revision:1,conversationIds:['original-cid']});
 return goal({schemaVersion:69,executionContext:{schemaVersion:12,sourceConversationId:'original-cid',projectProgress:{schemaVersion:1,selection:{schemaVersion:1,sourceConversationId:'original-cid',project:captured,goals:[{goalId:'g-source',specHash:'source-spec-hash'}],includeLatestResponse:true},observedAt:150,observations:[]},...(notes?{memory:{schemaVersion:2,references:[],project:copy(captured)}}:{})}});
};

test('own progress occurrences use their captured project with or without optional notes',async()=>{
 for(const notes of [false,true]){
  const t=fixture();t.goal=progressGoal(notes);const v=buildProjectProgress({transport:t});v.sync(project());await load(v);await inspect(v);
  assert.match(v.el.textContent,/Progresso consultado\./);assert.match(v.el.textContent,/Verify the actual result/);assert.match(v.el.textContent,/"project": \{/);assert.match(v.el.textContent,/"id": "project-one"/);assert.equal(button(v.el,'Consultar resposta persistida').hidden,false);
  button(v.el,'Consultar resposta persistida').fire('click');await flush();assert.match(v.el.textContent,/OWN_REPLY/);assert.equal(t.calls.length,4);v.dispose();
 }
});

test('progress project identity must match the native context, listing and optional notes',async()=>{
 for(const corrupt of [
  g=>delete g.executionContext.projectProgress,
  g=>g.executionContext.projectProgress.selection.project.id='other-project',
  g=>g.executionContext.projectProgress.selection.project.petId='agent_codex',
  g=>g.executionContext.projectProgress.selection.project.revision=9,
  g=>g.executionContext.memory.project.name='Different captured project',
  g=>g.executionContext.memory.schemaVersion=1,
  g=>g.executionContext.schemaVersion=2,
 ]){
  const t=fixture();t.goal=progressGoal(true);corrupt(t.goal);const v=buildProjectProgress({transport:t});v.sync(project());await load(v);await inspect(v);
  assert.match(v.el.textContent,/O objetivo mudou ou não pertence ao projeto original/);assert.equal(button(v.el,'Consultar resposta persistida').hidden,true);assert.equal(t.calls.filter(c=>c[0]==='history'||c[0]==='result').length,0);v.dispose();
 }
});

test('progress without notes preserves unknown debit and restored history qualifications',async()=>{
 const t=fixture();t.goal=progressGoal();t.result={status:'recoveryRequired',phase:'uncertain',criteriaVerified:false,goalComplete:false};const v=buildProjectProgress({transport:t});v.sync(project());await load(v);await inspect(v);
 assert.match(v.el.textContent,/65536 B debitados/);assert.match(v.el.textContent,/Resultados desconhecidos: 1/);button(v.el,'Consultar resposta persistida').fire('click');await flush();assert.match(v.el.textContent,/resultado desconhecido ou recibo por recuperar/);
 t.rows=page([summary({historicalSource:true})]);t.history=history({schemaVersion:2,scope:'restoredSnapshotOnly',accountingComplete:false,outcomesComplete:false,cumulativeCharge:null,authenticatedSubtotal:{modelCalls:0,toolCalls:0,outputBytes:0,unknownModelResults:0,unknownToolResults:0},restoredSource:{executionAuthorityGranted:false,postSnapshotStateUnknown:true}});await load(v);await inspect(v);
 assert.match(v.el.textContent,/Histórico consultado; o estado posterior ao snapshot é desconhecido/);assert.match(v.el.textContent,/Não existe saldo ou autorização recuperados/);assert.doesNotMatch(v.el.textContent,/0 chamadas de modelo/);v.dispose();
});
