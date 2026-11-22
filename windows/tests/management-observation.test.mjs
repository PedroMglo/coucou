import {test} from 'node:test';import assert from 'node:assert/strict';
import {managedSessionLink} from '../src/mochi/management-observation.ts';
import {sessionTree} from '../src/mochi/management.ts';
const count=value=>({value,observed:value??0});
const inventory=rows=>({scope:'mochiLibrary',coverage:'complete',observedAt:1700000000,stale:false,refreshing:false,reason:null,counts:{total:count(100),archived:count(2),confirmedActive:count(1),working:count(1),waiting:count(0),reserved:count(0),idle:count(0),unknown:0},sessions:rows,listed:rows.length,limit:12,truncated:true});
const row=(pet='agent_codex',cid='cid-one')=>({key:'mochi:codex:'+cid,provider:'codex',petId:pet,origin:'mochi',sessionId:'native-one',conversationId:cid,title:'Title',cwd:'/tmp/synthetic',state:'idle',archived:false,updatedAt:1700000000,observedAt:1700000000,open:{kind:'mochi',petId:pet,provider:pet==='agent_codex'?'codex':pet,conversationId:cid}});
const snapshot=()=>({schemaVersion:1,generatedAt:1700000001,pets:[{petId:'agent_codex',provider:'codex',enabled:true,mochi:inventory([row()]),usage:null}],providers:[{provider:'codex',petIds:['agent_codex'],native:{...inventory([row()]),scope:'codexDaemon'},usage:null}]});
test('complete management inventory is preserved independently of the bounded canonical selection',()=>{
 const data=snapshot();const rows=Array.from({length:100},(_,i)=>({provider:'codex',id:'native-'+i,name:'Session '+i,cwd:'/tmp',model:'test',state:'idle',parent:'',archived:false}));
 const tree=sessionTree(rows,[{id:'codex',name:'Codex'}],'all','',new Set(),new Set());assert.equal(tree.filter(r=>r.kind==='session').length,100);
 assert.equal(managedSessionLink(data,rows[90]).target,null);assert.match(managedSessionLink(data,rows[90]).reason,/ainda não está confirmada/);
 assert.equal(managedSessionLink(null,rows[0]).origin,'Origem por apurar');
});
test('matching helper native ID resolves to the canonical owner and conversation without native ID masquerading as CID',()=>{
 const data=snapshot();const link=managedSessionLink(data,{provider:'codex',id:'native-one'});
 assert.deepEqual(link.target,{petId:'agent_codex',provider:'codex',origin:'mochi',sessionId:'cid-one',conversationId:'cid-one'});assert.equal(link.origin,'No pet');
 assert.equal(managedSessionLink(data,{provider:'copilot',id:'native-one'}).target,null);
 const service=row('integration_github','github-cid');data.pets[0].mochi.sessions=[];data.providers[0].native.sessions=[service];
 assert.deepEqual(managedSessionLink(data,{provider:'codex',id:'native-one'}).target,{petId:'integration_github',provider:'integration_github',origin:'mochi',sessionId:'github-cid',conversationId:'github-cid'});
});
test('external origin retains its exact target without asserting terminal presence',()=>{
 const data=snapshot();data.pets[0].mochi.sessions=[];data.providers[0].native.sessions=[{...row(),origin:'external',conversationId:null,open:{kind:'external',petId:'agent_codex',provider:'codex',sessionId:'native-one'}}];
 const link=managedSessionLink(data,{provider:'codex',id:'native-one'});assert.equal(link.origin,'Externa');assert.deepEqual(link.target,{petId:'agent_codex',provider:'codex',origin:'external',sessionId:'native-one'});
});
test('conflicting owners or conversations, invalid ownership and stale/error snapshots cannot supply a chat route',()=>{
 const data=snapshot(),managed={provider:'codex',id:'native-one'};
 data.providers[0].native.sessions.push(row('integration_github','other-cid'));assert.equal(managedSessionLink(data,managed).target,null);assert.match(managedSessionLink(data,managed).reason,/mais de um proprietário/);
 data.providers[0].native.sessions=[{...row(),open:{kind:'mochi',petId:'other',provider:'codex',conversationId:'cid-one'}}];assert.equal(managedSessionLink(data,managed).target,null);
 const fresh=snapshot();fresh.providers[0].native.stale=true;assert.match(managedSessionLink(fresh,managed).reason,/desatualizada/);assert.equal(managedSessionLink(snapshot(),managed,true).target,null);
 const duplicate=snapshot();duplicate.pets.push({...duplicate.pets[0]});assert.equal(managedSessionLink(duplicate,managed).target,null);
});
