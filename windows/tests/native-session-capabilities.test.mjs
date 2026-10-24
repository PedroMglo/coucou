import test from 'node:test';
import assert from 'node:assert/strict';
import {SessionChat,nativeSessionReady} from '../src/core/session-chat.ts';

const target=(id='a')=>({petId:'agent_codex',provider:'codex',origin:'external',sessionId:id});
function snapshot(t=target(),extra={}) {
 const s={target:t,messages:[],busy:false,capabilities:{history:true,send:true,cancel:false},...extra};
 const at=Date.now(),expires=at+15_000;
 s.capabilityDescriptor={schemaVersion:1,target:{...t},revision:'native:synthetic:1',observedAt:at,expiresAt:expires,
  busy:s.busy,turn:s.turn??null,model:s.model??null,capabilities:['session.history','session.send','session.interrupt','session.approve','session.continueBackground'].map(id=>({
   id,support:'supported',availability:'ready',observedAt:at,expiresAt:expires,
  }))};
 return s;
}
const deferred=()=>{let resolve;const promise=new Promise(r=>resolve=r);return{promise,resolve};};
const tick=()=>new Promise(r=>setTimeout(r,0));

test('native control requires the same exact pet, provider, origin, session and conversation',()=>{
 for(const [field,value] of [['petId','other'],['provider','copilot'],['origin','mochi'],['sessionId','b'],['conversationId','foreign']]){
  const s=snapshot();s.capabilityDescriptor.target[field]=value;
  assert.equal(nativeSessionReady(s,'session.send'),false,field);
 }
 assert.equal(nativeSessionReady(snapshot(),'session.send'),true);
});

test('expired, future, malformed and duplicate metadata do not advertise ready controls',()=>{
 for(const change of [
  d=>d.observedAt=Date.now()+1000,d=>d.expiresAt=Date.now(),d=>d.expiresAt=d.observedAt+15_001,
  d=>d.observedAt=null,d=>d.schemaVersion=2,d=>d.revision='',d=>d.capabilities.push({...d.capabilities[1]}),
  d=>d.capabilities.find(c=>c.id==='session.send').support='unknown',
  d=>d.capabilities.find(c=>c.id==='session.send').availability='unknown',
  d=>d.capabilities.find(c=>c.id==='session.send').expiresAt++,
 ]){const s=snapshot();change(s.capabilityDescriptor);assert.equal(nativeSessionReady(s,'session.send'),false);}
});

test('native busy, model and exact turn must agree with the returned history',()=>{
 for(const change of [d=>d.busy=true,d=>d.model='another-model',d=>d.turn='another-turn']){
  const s=snapshot();change(s.capabilityDescriptor);assert.equal(nativeSessionReady(s,'session.send'),false);
 }
 const s=snapshot(target(),{busy:true,turn:'turn-a',model:'synthetic-model'});
 assert.equal(nativeSessionReady(s,'session.interrupt'),true);
 s.turn='turn-b';assert.equal(nativeSessionReady(s,'session.interrupt'),false);
});

test('shared consumer intersects metadata with original native booleans',async()=>{
 let sends=0,cancels=0;
 const s=snapshot(target(),{capabilities:{history:true,send:false,cancel:false}});
 const chat=new SessionChat({history:async()=>s,send:async()=>{sends++;},cancel:async()=>{cancels++;}},()=>{});
 await chat.select(s.target);assert.equal(await chat.send('Never widen'),false);await chat.cancel();
 assert.equal(sends,0);assert.equal(cancels,0);
});

test('an invalidation closes controls immediately and unrelated targets do not invalidate them',async()=>{
 let sends=0;const t=target();const chat=new SessionChat({history:async()=>snapshot(t),send:async()=>{sends++;}},()=>{});
 await chat.select(t);chat.invalidate(target('other'));assert.equal(chat.snapshot.capabilities.send,true);
 chat.invalidate(t);assert.equal(chat.snapshot.capabilities.send,false);assert.equal(await chat.send('Do not send stale work'),false);
 assert.equal(sends,0);await chat.refresh();assert.equal(chat.snapshot.capabilities.send,true);
});

test('an invalidation during a read forces another read and cannot re-enable old controls',async()=>{
 const reads=[deferred(),deferred()];let count=0;const t=target();
 const chat=new SessionChat({history:()=>reads[count++].promise},()=>{});
 const initial=chat.select(t);chat.invalidate(t);reads[0].resolve(snapshot(t));await initial;
 assert.equal(count,2);assert.equal(chat.snapshot.capabilities.send,false);
 reads[1].resolve(snapshot(t));await tick();assert.equal(chat.snapshot.capabilities.send,true);
});

test('click-time checks reject metadata that expires after a successful refresh',async()=>{
 let sends=0;const t=target(),s=snapshot(t);
 const chat=new SessionChat({history:async()=>s,send:async()=>{sends++;}},()=>{});await chat.select(t);
 chat.snapshot.capabilityDescriptor.expiresAt=Date.now();
 assert.equal(await chat.send('Expired'),false);assert.equal(sends,0);
});

test('native cancellation preserves its original target and turn and closes controls during execution',async()=>{
 const t=target(),job=deferred();let captured;
 const chat=new SessionChat({history:async()=>snapshot(t,{busy:true,turn:'original-turn',capabilities:{history:true,send:false,cancel:true}}),
  cancel:(exact,turn)=>{captured={exact,turn};return job.promise;}},()=>{});
 await chat.select(t);const pending=chat.cancel();assert.deepEqual(captured,{exact:t,turn:'original-turn'});
 assert.equal(chat.snapshot.capabilities.cancel,false);job.resolve(true);await pending;
});
