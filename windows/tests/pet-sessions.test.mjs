import {test} from 'node:test';import assert from 'node:assert/strict';
import {PetSessions,sessionLabel} from '../src/core/pet-sessions.ts';
const base={id:'agent_codex',name:'Codex',state:'idle',steps:[],stepIndex:0,color:'#2dd4bf',source:'agent',isIntegration:true};
test('one pet retains independent terminal states and an explicit selection; a decision keeps its exact session',()=>{
 const sessions=new PetSessions();
 const a=sessions.observe(base,'codex','session-a','/project/a',false,1);a.state='working';a.steps.push('A');a.sessionTitle='Base de dados';
 const b=sessions.observe(base,'codex','session-b','/project/b',false,2);b.state='approval';b.steps.push('B');b.sessionTitle='Interface';
 assert.equal(sessions.list(base.id).length,2);assert.equal(sessions.current(base.id,null),b);
 assert.ok(sessions.choose(base.id,'codex','session-a'));assert.equal(sessions.current(base.id,null),a);
 const pending={pillId:base.id,backend:'codex',sessionId:'session-b'};
 assert.equal(sessions.current(base.id,pending),b);assert.deepEqual(a.steps,['A']);assert.deepEqual(b.steps,['B']);
 const projected={...base};sessions.project(projected,pending);assert.equal(projected.sessionCwd,'/project/b');assert.equal(projected.sessionId,'session-b');
 b.ended=true;assert.equal(sessions.current(base.id,null),a);assert.equal(sessions.list(base.id).length,1);
 assert.equal(sessionLabel(a),'Terminal #session- · Base de dados');
});
test('metadata restores live sessions after startup and marks the Mochi channel only by an owned job',()=>{
 const s=new PetSessions(),row=(id,name,state='working')=>({id,name,provider:'codex',cwd:'/tmp',state});
 s.merge([row('external-a','Criar BD'),row('mochi-b','Mochi · Olá'),row('old','Guardada','saved')],[{provider:'codex',backend:'codex',sessionId:'mochi-b'}],()=>base,10);
 assert.equal(s.list(base.id).length,2);assert.equal(s.list(base.id).find(x=>x.sessionId==='external-a').sessionOwned,false);
 assert.equal(s.list(base.id).find(x=>x.sessionId==='mochi-b').sessionOwned,true);
 const a=s.observe(base,'codex','external-a','/tmp',false,20);a.state='approval';
 s.merge([row('external-a','Título atualizado','idle')],[],()=>base,30);assert.equal(a.state,'approval');assert.equal(a.sessionTitle,'Título atualizado');
});
