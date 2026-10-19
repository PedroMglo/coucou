import {test} from 'node:test';
import assert from 'node:assert/strict';
import {matchingSessions,mergeSessionLists} from '../src/desktop/session-list.ts';
const row=(id,origin,title,updatedAt=1)=>({target:{petId:'agent_codex',provider:'codex',origin,sessionId:id},title,cwd:'/projetos/açores',updatedAt,state:'Disponível',model:'local:8b',capabilities:{history:true,send:true,cancel:true}});
test('session search combines accent-insensitive terms and sorts recent conversations first',()=>{
 const rows=[row('one','mochi','Revisão da interface',1),row('two','mochi','Revisão da interface',3),row('one','external','Revisão da interface',9)];
 assert.deepEqual(matchingSessions(rows,'revisao acores 8b','mochi').map(s=>s.target.sessionId),['two','one']);
 assert.equal(matchingSessions(rows,'revisao ausente','mochi').length,0);
 assert.equal(matchingSessions(rows,'one','external').length,1);
});
test('partial refresh preserves the failed source and never merges terminal and pet sessions by ID alone',()=>{
 const own=row('same','mochi','Guardada'),external=row('same','external','Terminal');
 const updated={...own,title:'Atualizada'};
 assert.deepEqual(mergeSessionLists([own,external],[updated],null),[updated,external]);
 assert.deepEqual(mergeSessionLists([own,external],null,[]),[own]);
 assert.equal(mergeSessionLists([], [own,own], [external]).length,2);
});
