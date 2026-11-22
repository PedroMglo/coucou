import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {taskNoticeFor} from '../src/core/task-notice.ts';
import {buildTaskNotice} from '../src/views/task-notice.ts';
const target={petId:'ai_ollama',provider:'ollama',origin:'mochi',sessionId:'A',conversationId:'A'};
const notice={taskId:'task-A',provider:'ollama',conversationId:'A',phase:'stopped',query:'Pedido original <script>inofensivo</script>',message:'Verifica os efeitos antes de retomar.'};
for(const phase of ['stopped','interrupted','failed'])test(`${phase} metadata is readable for its exact canonical target`,()=>{
 assert.equal(taskNoticeFor(target,{...notice,phase}).phase,phase);
});
test('another provider, conversation, external target or missing canonical CID cannot borrow the notice',()=>{
 for(const t of [{...target,provider:'codex'},{...target,sessionId:'B',conversationId:'B'},{...target,origin:'external'},{...target,conversationId:undefined}])assert.equal(taskNoticeFor(t,notice),undefined);
 assert.equal(taskNoticeFor(target,undefined),undefined);assert.equal(taskNoticeFor(null,notice),undefined);
 assert.equal(taskNoticeFor(target,{...notice,phase:'done'}),undefined);
});
test('busy/new successful turns hide old metadata without inferring a task from the archive',()=>{
 assert.equal(taskNoticeFor(target,notice,true),undefined);
 installFakeDom();const v=buildTaskNotice();v.sync(target,{busy:false,taskNotice:notice});assert.equal(v.el.hidden,false);
 v.sync(target,{busy:false});assert.equal(v.el.hidden,true);assert.equal(v.el.textContent,'Pedido original');
});
test('the original request is safe plain text, separate from assistant bubbles, with no action buttons',()=>{
 installFakeDom();const v=buildTaskNotice();v.sync(target,{busy:false,taskNotice:notice});
 assert.equal(v.el.hidden,false);assert.match(v.el.textContent,/<script>inofensivo<\/script>/);assert.equal(v.el.find('button').length,0);
 assert.equal(v.el.classList.contains('task-notice'),true);assert.equal(v.el.find('pre')[0].textContent,notice.query);
});
test('persistence warning is preserved by the existing feedback while duplicate task text is omitted',()=>{
 installFakeDom();const v=buildTaskNotice();const warning='Não foi possível guardar. '+notice.message;
 v.sync(target,{busy:false,notice:warning,taskNotice:notice});assert.doesNotMatch(v.el.textContent,/Verifica os efeitos/);assert.match(v.el.textContent,/Pedido original/);
});
test('switching targets clears the visible request instead of retaining a hidden old query',()=>{
 installFakeDom();const v=buildTaskNotice();v.sync(target,{busy:false,taskNotice:notice});v.sync({...target,sessionId:'B',conversationId:'B'},{busy:false,taskNotice:notice});
 assert.equal(v.el.hidden,true);assert.doesNotMatch(v.el.textContent,/<script>/);
});
test('a new send pending acknowledgement hides the previous task notice immediately',()=>{
 installFakeDom();const v=buildTaskNotice();v.sync(target,{busy:false,taskNotice:notice},true);assert.equal(v.el.hidden,true);
});
