import {test} from 'node:test';import assert from 'node:assert/strict';
import {InlineDrafts} from '../src/core/inline-drafts.ts';
import {sessionTargetKey} from '../src/core/session-chat.ts';
import {IndexedDraftStorage,draftTarget} from '../src/core/session-drafts.ts';
import {petForChatProvider,petBlueprint,PetFactory} from '../src/core/pet-factory.ts';
const tick=()=>new Promise(r=>setImmediate(r));const idle=async()=>{await tick();await tick();};
const identity=(cid='A',provider='codex',petId='agent_codex')=>({petId,provider,conversationId:cid});
const bound=(cid='A',provider='codex',petId='agent_codex')=>({petId,provider,origin:'mochi',sessionId:cid,conversationId:cid});
const pending=(provider='codex',petId='agent_codex')=>({petId,provider,origin:'unbound',sessionId:'inline-composer'});
const saved=(target,text,revision='1')=>({schemaVersion:1,key:sessionTargetKey(target),target,revision,text});
const deferred=()=>{let resolve;const promise=new Promise(r=>resolve=r);return{promise,resolve};};
function storage(rows=[]){const data=new Map(rows.map(r=>[r.key,structuredClone(r)])),writes=[];let counter=100n;return{data,writes,read:async target=>structuredClone(data.get(sessionTargetKey(target))??null),write:async(target,expected,text)=>{writes.push({target,expected,text});const key=sessionTargetKey(target),prior=data.get(key)??null;if((prior?.revision??null)!==expected)return{kind:'conflict',record:structuredClone(prior)};const record=saved(target,text,String(++counter));data.set(key,structuredClone(record));return{kind:'saved',record};}};}
test('complete target isolation and late hydration never move text between pets or CIDs',async()=>{
 const a=deferred(),s=storage([saved(bound('B'),'B saved')]),read=s.read;s.read=target=>target.sessionId==='A'?a.promise:read(target);const r=new InlineDrafts(s);r.select(identity('A'));await tick();r.select(identity('B'));await idle();a.resolve(saved(bound('A'),'late A'));await idle();assert.equal(r.text,'B saved');assert.equal(r.key,sessionTargetKey(bound('B')));assert.equal(s.writes.length,0);
 r.edit(r.target,'B own edit');r.select(identity('C','antigravity','agent_antigravity'));await idle();assert.equal(r.text,'');await r.flush();assert.equal(s.data.get(sessionTargetKey(bound('B'))).text,'B own edit');
});
test('pre-CID typing is durably unbound and cannot produce a send capture until explicit copy',async()=>{
 const s=storage(),r=new InlineDrafts(s);r.select(identity(null));await idle();r.edit(r.target,'before identity 🐈\ud800');await r.flush();r.select(identity('new-cid'));await idle();assert.equal(r.target.origin,'unbound');assert.equal(r.needsChoice,true);assert.equal(r.capture(),null);assert.equal(s.data.has(sessionTargetKey(bound('new-cid'))),false);
 assert.equal(await r.choose(true),true);await r.flush();assert.equal(r.capture().target.sessionId,'new-cid');assert.equal(s.data.get(sessionTargetKey(bound('new-cid'))).text,'before identity 🐈\ud800');assert.equal(s.data.get(sessionTargetKey(pending())).text,'before identity 🐈\ud800');
});
test('opening identified conversation without copying preserves pending and existing bound drafts',async()=>{
 const s=storage([saved(bound('A'),'original destination')]),r=new InlineDrafts(s);r.select(identity(null));await idle();r.edit(r.target,'unbound source');await r.flush();r.select(identity('A'));assert.equal(await r.choose(false),true);await idle();assert.equal(r.text,'original destination');assert.equal(s.data.get(sessionTargetKey(pending())).text,'unbound source');assert.equal(s.writes.filter(w=>w.target.origin==='mochi').length,0);
});
test('copy into occupied destination becomes an explicit normal CAS conflict, never a silent overwrite',async()=>{
 const s=storage([saved(bound('A'),'other view winner','7')]),r=new InlineDrafts(s);r.select(identity(null));await idle();r.edit(r.target,'pending local');await r.flush();r.select(identity());assert.equal(await r.choose(true),true);await idle();assert.equal(r.text,'pending local');assert.equal(r.resource.status(r.target).phase,'conflict');assert.equal(r.resource.status(r.target).storedText,'other view winner');assert.equal(s.data.get(sessionTargetKey(bound())).text,'other view winner');assert.equal(s.data.get(sessionTargetKey(pending())).text,'pending local');
});
test('stale pending choice after owner, CID or typed text changes cannot write a new destination',async()=>{
 for(const mutation of ['owner','cid','typing']){const s=storage(),hold=deferred(),write=s.write;
  let blocking=false;s.write=async(...args)=>{if(blocking)await hold.promise;return write(...args);};const r=new InlineDrafts(s);r.select(identity(null));await idle();blocking=true;r.edit(r.target,'first');r.select(identity('A'));const task=r.choose(true);await tick();if(mutation==='owner')r.select(identity('G','antigravity','agent_antigravity'));else if(mutation==='cid')r.select(identity('B'));else r.edit(r.target,'new typing');hold.resolve();assert.equal(await task,false);await r.flush();assert.ok(!s.writes.some(w=>w.target.origin==='mochi'));}
});
test('late pending read is a recoverable preview, never assigned automatically to a known conversation',async()=>{
 const s=storage([saved(bound(),'bound text')]),read=s.read,hold=deferred();s.read=target=>target.origin==='unbound'?hold.promise:read(target);const r=new InlineDrafts(s);r.select(identity(null));r.select(identity());hold.resolve(saved(pending(),'pending persisted'));await idle();assert.equal(r.text,'bound text');assert.equal(r.recovery.text,'pending persisted');assert.equal(r.target.origin,'mochi');assert.equal(s.writes.length,0);
});
test('saved send clears only exact unchanged original draft, with no effect on another pet or newer typing',async()=>{
 const s=storage(),r=new InlineDrafts(s);r.select(identity());await idle();r.edit(r.target,'original');const original=r.capture();r.edit(r.target,'new');r.edit(r.target,'original');assert.equal(r.complete(original),false);assert.equal(r.text,'original');const fresh=r.capture();r.select(identity('G','antigravity','agent_antigravity'));await idle();r.edit(r.target,'Gravity retained');assert.equal(r.complete(fresh),true);await r.flush();assert.equal(r.text,'Gravity retained');assert.equal(s.data.get(sessionTargetKey(bound())).text,'');assert.equal(s.data.get(sessionTargetKey(bound('G','antigravity','agent_antigravity'))).text,'Gravity retained');
});
test('failed/manual pending send has no automatic clear or resend and all inactive dirty data flush',async()=>{
 const s=storage(),r=new InlineDrafts(s);r.select(identity());await idle();r.edit(r.target,'send failed before receipt');const captured=r.capture();assert.equal(captured.text,'send failed before receipt');r.select(identity('B'));r.edit(r.target,'other conversation');await r.flush();assert.equal(s.data.get(sessionTargetKey(bound())).text,captured.text);assert.equal(s.data.get(sessionTargetKey(bound('B'))).text,'other conversation');
});
test('recovering unbound data first saves the current bound draft and refuses a stale revision',async()=>{
 const s=storage([saved(pending(),'saved pending','7')]),r=new InlineDrafts(s);r.select(identity());await idle();r.edit(r.target,'bound own typing');assert.equal(await r.reviewRecovery('wrong'),false);assert.equal(await r.reviewRecovery('7'),true);await idle();assert.equal(r.target.origin,'unbound');assert.equal(r.text,'saved pending');assert.equal(s.data.get(sessionTargetKey(bound())).text,'bound own typing');assert.equal(r.needsChoice,true);
});
test('draft-only origin is bounded and remains unavailable in the old compatibility IndexedDB writer',async()=>{
 assert.equal(draftTarget(pending()).origin,'unbound');for(const raw of [{...pending(),sessionId:'arbitrary'},{...pending(),conversationId:'A'}])assert.throws(()=>draftTarget(raw));let opened=0;const old=new IndexedDraftStorage(()=>{opened++;throw Error();});await assert.rejects(old.write(pending(),null,'never old v1'),e=>e.code==='unsupported');assert.equal(opened,0);
});
test('every factory chat route has its own actual PetId, including APIs with no task route',()=>{
 for(const id of Object.keys(PetFactory.configuration().pets)){const p=petBlueprint(id);if(p.routing.chat)assert.equal(petForChatProvider(p.routing.chat),id);}assert.equal(petForChatProvider('unknown'),null);
});
