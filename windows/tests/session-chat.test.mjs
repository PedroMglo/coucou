import test from 'node:test';
import assert from 'node:assert/strict';
import {SessionChat,sessionTargetKey} from '../src/core/session-chat.ts';
const target=(id,origin='external',provider='codex')=>({petId:'agent_codex',provider,origin,sessionId:id});
const snapshot=(t,extra={})=>({target:t,messages:[{id:'m',role:'assistant',content:t.sessionId}],busy:false,capabilities:{history:true,send:true,cancel:true},...extra});
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return {promise,resolve,reject};};
test('origin and pet are part of a session identity',()=>{
 assert.notEqual(sessionTargetKey(target('same')),sessionTargetKey(target('same','mochi')));
 assert.notEqual(sessionTargetKey(target('same')),sessionTargetKey({...target('same'),petId:'another'}));
});
test('a slow history read cannot replace the newly selected terminal',async()=>{
 const first=deferred(),second=deferred();const chat=new SessionChat({history:t=>t.sessionId==='a'?first.promise:second.promise},()=>{});
 const a=chat.select(target('a')),b=chat.select(target('b'));second.resolve(snapshot(target('b')));await b;first.resolve(snapshot(target('a')));await a;
 assert.equal(chat.snapshot.target.sessionId,'b');assert.equal(chat.loading,false);
});
test('a mismatched native snapshot fails closed',async()=>{
 const chat=new SessionChat({history:async()=>snapshot(target('foreign'))},()=>{});await chat.select(target('a'));
 assert.equal(chat.snapshot,null);assert.match(chat.error,/outra sessão/);assert.equal(await chat.send('hello'),false);
});
test('send remains pinned while another terminal is selected',async()=>{
 const job=deferred(),sent=[];const chat=new SessionChat({history:async t=>snapshot(t),send:(t,q)=>{sent.push([t,q]);return job.promise;}},()=>{});
 const original=target('a');await chat.select(original);const pending=chat.send('run');original.sessionId='mutated';await chat.select(target('b'));job.resolve();await pending;
 assert.equal(sent[0][0].sessionId,'a');assert.equal(chat.target.sessionId,'b');assert.equal(chat.snapshot.messages[0].content,'b');
});
test('read-only sessions, busy turns and duplicate submission never send',async()=>{
 let sends=0,view={capabilities:{history:true,send:false,cancel:false}};const job=deferred();
 const chat=new SessionChat({history:async t=>snapshot(t,view),send:()=>{sends++;return job.promise;}},()=>{});
 await chat.select(target('a'));assert.equal(await chat.send('no'),false);
 view={busy:true};await chat.refresh();assert.equal(await chat.send('no'),false);
 view={};await chat.refresh();const pending=chat.send('once');assert.equal(await chat.send('twice'),false);job.resolve();await pending;assert.equal(sends,1);
});
test('cancel captures the precise session and turn, never a newer selection',async()=>{
 const stop=deferred(),cancelled=[];const chat=new SessionChat({history:async t=>snapshot(t,{busy:true,turn:'turn-7'}),cancel:(t,turn)=>{cancelled.push([t,turn]);return stop.promise;}},()=>{});
 await chat.select(target('a'));const cancelling=chat.cancel();await chat.select(target('b'));stop.resolve();await cancelling;
 assert.equal(cancelled[0][0].sessionId,'a');assert.equal(cancelled[0][1],'turn-7');assert.equal(chat.snapshot.target.sessionId,'b');
});
test('late errors from a send cannot contaminate a different conversation',async()=>{
 const job=deferred();const chat=new SessionChat({history:async t=>snapshot(t),send:()=>job.promise},()=>{});
 await chat.select(target('a'));const pending=chat.send('a');await chat.select(target('b'));job.reject(new Error('error in a'));assert.equal(await pending,false);assert.equal(chat.error,'');
});
test('frequent live notifications coalesce without starving history snapshots',async()=>{
 const reads=[deferred(),deferred()];let count=0;
 const chat=new SessionChat({history:()=>reads[count++].promise},()=>{});
 const initial=chat.select(target('a'));for(let i=0;i<20;i++)void chat.refresh();
 assert.equal(count,1);reads[0].resolve(snapshot(target('a'),{busy:true}));await initial;
 assert.equal(count,2);assert.equal(chat.snapshot.busy,true);
 reads[1].resolve(snapshot(target('a')));await new Promise(r=>setTimeout(r,0));assert.equal(chat.snapshot.busy,false);assert.equal(chat.loading,false);
});
test('model choices are scoped to each selected session and captured at send',async()=>{
 const sent=[],job=deferred();const chat=new SessionChat({history:async t=>snapshot(t),send:(t,q,model)=>{sent.push({t,model});return job.promise;}},()=>{});
 await chat.select(target('a'));chat.chooseModel('model-a');const pending=chat.send('a');await chat.select(target('b'));assert.equal(chat.model,undefined);chat.chooseModel('model-b');job.resolve();await pending;
 assert.equal(sent[0].t.sessionId,'a');assert.equal(sent[0].model,'model-a');assert.equal(chat.model,'model-b');await chat.select(target('a'));assert.equal(chat.model,'model-a');
});
test('a committed public query replaces its optimistic bubble before the send promise settles',async()=>{
 const job=deferred(),t=target('a');let messages=[{id:'old-user',role:'user',content:'again'},{id:'old-answer',role:'assistant',content:'earlier'}];
 const chat=new SessionChat({history:async()=>snapshot(t,{messages}),send:()=>job.promise},()=>{});
 await chat.select(t);const pending=chat.send('again');assert.equal(chat.query,'again','an identical earlier question is not the new submission');
 messages=[...messages,{id:'new-user',role:'user',content:'again'},{id:'new-answer',role:'assistant',content:'done'}];
 await chat.refresh();assert.equal(chat.sending,true);assert.equal(chat.query,'','the acknowledged query is already in public history');
 job.resolve();await pending;assert.equal(chat.snapshot.messages.length,4);
});
test('an attachment is captured with its conversation even if selection changes during delivery',async()=>{
 const job=deferred(),sent=[],a=target('a','mochi'),b=target('b','mochi');
 const attachment={id:'a-file',name:'a.pdf',path:'/inbox/a.pdf',provider:'codex',conversationId:'a',sequence:7,kind:'file',entries:1,size:3};
 const chat=new SessionChat({history:async t=>snapshot(t,{attachments:t.sessionId==='a'?[attachment]:[]}),send:(t,q,m,file)=>{sent.push({t,file});return job.promise;}},()=>{});
 await chat.select(a);const pending=chat.send('Read this');await chat.select(b);job.resolve();await pending;
 assert.equal(sent[0].t.sessionId,'a');assert.equal(sent[0].file[0].path,'/inbox/a.pdf');assert.equal(chat.snapshot.attachments.length,0);
});
test('an uncertain send keeps its original text for review without restoring or replaying the draft',async()=>{
 const t=target('a');let sends=0;
 const chat=new SessionChat({history:async()=>snapshot(t),send:async()=>{sends++;throw Error('Delivery unknown');}},()=>{});
 await chat.select(t);assert.equal(await chat.send('Do this once'),false);await chat.refresh();
 assert.equal(sends,1);assert.equal(chat.failure.query,'Do this once');assert.equal(chat.drafts.has(sessionTargetKey(t)),false);
});
test('a late error refresh cannot apply the old failure to a newly selected conversation',async()=>{
 const a=target('a'),b=target('b'),read=deferred(),job=deferred();let count=0;
 const chat=new SessionChat({history:async t=>t.sessionId==='a'&&count++?read.promise:snapshot(t),send:()=>job.promise},()=>{});
 await chat.select(a);const pending=chat.send('A');job.reject(Error('A failed'));await new Promise(r=>setTimeout(r,0));
 await chat.select(b);read.resolve(snapshot(a));await pending;assert.equal(chat.target.sessionId,'b');assert.equal(chat.error,'');assert.equal(chat.failure,undefined);
});

test('the model displayed at click time remains the expected model even if a synchronous refresh clears local overrides',async()=>{
 const sent=[];let chat;
 chat=new SessionChat({history:async t=>snapshot(t),send:async(t,q,model)=>{sent.push(model);}},()=>{if(chat?.target)chat.models.delete(sessionTargetKey(chat.target));});
 await chat.select(target('ollama-conversation'));
 assert.equal(await chat.send('Pedido sintético','modelo-apresentado:4b'),true);
 assert.deepEqual(sent,['modelo-apresentado:4b']);assert.equal(chat.models.size,0);
});

test('an unconfirmed cancellation of a still-running turn is visible without cancelling another turn',async()=>{
 for(const ack of [false,null,undefined]){
  const t=target('cancel-unconfirmed');const chat=new SessionChat({history:async()=>snapshot(t,{busy:true,turn:'exact-7'}),cancel:async()=>ack},()=>{});
  await chat.select(t);await chat.cancel();assert.match(chat.error,/confirmar a interrupção/);assert.equal(chat.snapshot.busy,true);
 }
});
test('a turn that finished before stop acknowledgement does not show a false cancellation error',async()=>{
 const t=target('already-finished');let busy=true;const chat=new SessionChat({history:async()=>snapshot(t,{busy,turn:'exact-8'}),cancel:async()=>{busy=false;return false;}},()=>{});
 await chat.select(t);await chat.cancel();assert.equal(chat.error,'');assert.equal(chat.snapshot.busy,false);
});
test('a request captures the complete set before changed callbacks or later mutations and retains attachments after reply',async()=>{
 const t=target('cid','mochi'),a={id:'a',name:'same',path:'/inbox/a',provider:t.provider,conversationId:'cid',sequence:1,kind:'file',size:3,entries:1},b={...a,id:'b',path:'/inbox/b'},files=[a,b],gate=deferred(),received=[];
 let chat=new SessionChat({history:async()=>snapshot(t,{attachments:files}),send:async(_t,_q,_m,attachments)=>{received.push(attachments);return gate.promise;}},()=>{});
 await chat.select(t);const job=chat.send('both');a.path='/changed';files.push({...b,id:'c'});await chat.refresh();gate.resolve();await job;assert.deepEqual(received[0].map(f=>f.id),['a','b']);assert.equal(received[0][0].path,'/inbox/a');assert.equal(chat.snapshot.attachments.length,3);
});
