import {test} from 'node:test';import assert from 'node:assert/strict';
import {SessionLookupNavigation,validLookupRequest,validatedLookup,lookupKey,lookupMessage,lookupEvidence} from '../src/core/session-lookup.ts';
const A={provider:'codex',nativeSessionId:'00000000-0000-4000-8000-000000000001'},B={...A,nativeSessionId:'00000000-0000-4000-8000-000000000002'};
const evidence=(scope='mochiLibrary')=>({scope,coverage:'complete',observedAt:1700000000,stale:false,reason:null});
const mochi=(request=A)=>({schemaVersion:1,generatedAt:1700000001,request:{...request},state:'resolved',reason:null,retryAfterMs:null,owner:'codex',conversationId:'cid-one',origin:'mochi',open:{kind:'mochi',petId:'agent_codex',provider:'codex',conversationId:'cid-one'},library:evidence(),native:{...evidence('codexDaemon'),coverage:'unavailable',observedAt:null,stale:true}});
const external=(request=A)=>({...mochi(request),owner:null,conversationId:null,origin:'external',open:{kind:'external',petId:request.provider==='copilot'?'agent_copilot':'agent_codex',provider:request.provider,sessionId:request.nativeSessionId},native:evidence(request.provider==='copilot'?'copilotLiveEndpoints':'codexDaemon')});
const negative=(state,reason=null,request=A)=>({...mochi(request),state,reason,owner:null,conversationId:null,origin:'unknown',open:null,native:evidence('codexDaemon')});
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return{promise,resolve,reject};};
const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
function fixture(){
 const lookups=[],opens=[];let now=0,presented=0,current=true,unavailable='',onChange=()=>{};
 const navigation=new SessionLookupNavigation({lookup(request){const next=deferred();lookups.push({request,...next});return next.promise;},open(target){const next=deferred();opens.push({target,...next});return next.promise;}},{current:()=>current,unavailable:()=>unavailable,changed:()=>onChange(),presented:()=>presented++,clock:()=>now});
 return {navigation,lookups,opens,get presented(){return presented;},set now(value){now=value;},set current(value){current=value;},set unavailable(value){unavailable=value;},set onChange(value){onChange=value;}};
}
test('lookup requests retain exact backend and Thread.id namespaces without normalization',()=>{
 assert.equal(validLookupRequest(A),true);assert.equal(validLookupRequest({...A,provider:'copilot'}),true);assert.equal(validLookupRequest({provider:'antigravity',nativeSessionId:'session:one'}),true);
 for(const request of [{...A,provider:'Codex'},{...A,nativeSessionId:' '+A.nativeSessionId},{...A,nativeSessionId:'tree-id'}, {provider:'codex;cmd',nativeSessionId:A.nativeSessionId}])assert.equal(validLookupRequest(request),false);
 assert.notEqual(lookupKey(A),lookupKey({...A,provider:'copilot'}));
});
test('Mochi lookup preserves logical service owner and CID even when native evidence is unavailable',()=>{
 const result=mochi();result.owner=result.open.provider='integration_github';result.open.petId='integration_github';
 const checked=validatedLookup(result,A);assert.deepEqual(checked.target,{petId:'integration_github',provider:'integration_github',origin:'mochi',sessionId:'cid-one',conversationId:'cid-one'});assert.equal(Object.isFrozen(checked.target),true);
 result.open.conversationId='later';assert.equal(checked.result.open.conversationId,'cid-one');assert.doesNotMatch(lookupEvidence(checked.result),/desatualizados/);
});
test('partial or stale positive native evidence allows exact navigation without an executing-turn or terminal claim',()=>{
 const result=external();result.native.coverage='partial';result.native.stale=true;result.native.observedAt=1699999900;
 const checked=validatedLookup(result,A);assert.deepEqual(checked.target,{petId:'agent_codex',provider:'codex',origin:'external',sessionId:A.nativeSessionId});assert.match(lookupMessage(result),/não confirma um turno/);assert.doesNotMatch(lookupMessage(result),/Terminal/);assert.match(lookupEvidence(result),/Dados desatualizados.*Cobertura parcial/);
});
test('scoped notFound requires complete fresh native evidence in the matching source',()=>{
 const result=negative('notFound','notFoundInNativeScope');assert.equal(validatedLookup(result,A).target,null);assert.match(lookupMessage(result),/fonte consultada/);
 for(const update of [{stale:true},{coverage:'partial'},{scope:'other'},{observedAt:null},{observedAt:1699999985},{observedAt:1700000002}])assert.throws(()=>validatedLookup({...result,native:{...result.native,...update}},A));
});
test('contradictory or malformed targets and request echoes never navigate',()=>{
 const updates=[r=>r.request=B,r=>r.schemaVersion=2,r=>r.open.provider='integration_github',r=>r.open.conversationId='other',r=>r.conversationId='cid_with_underscore',r=>r.library.stale=true,r=>r.library.coverage='partial',r=>r.open.petId='bad pet'];
 for(const update of updates){const r=mochi();update(r);assert.throws(()=>validatedLookup(r,A));}
 for(const update of [r=>r.open.sessionId=B.nativeSessionId,r=>r.open.provider='copilot',r=>r.open.petId='integration_github',r=>r.owner='codex',r=>r.conversationId='cid',r=>r.native.coverage='unavailable']){const r=external();update(r);assert.throws(()=>validatedLookup(r,A));}
 assert.throws(()=>validatedLookup({...negative('ambiguous'),open:mochi().open},A));
});
test('retired owner remains Mochi unavailable with no fallback; unknown reasons are never shown raw',()=>{
 const result={...negative('unavailable','petBindingUnavailable'),owner:'integration_spotify',conversationId:'old-cid',origin:'mochi'};
 assert.equal(validatedLookup(result,A).target,null);assert.match(lookupMessage(result),/já não está disponível/);
 assert.doesNotMatch(lookupMessage({...result,reason:'/private/account/provider-error'}),/private|account/);
});
test('retry hints are bounded manual delays confined to busy/throttled unavailable responses',()=>{
 for(const reason of ['nativeLookupBusy','nativeLookupThrottled'])for(const delay of [1,10000])assert.equal(validatedLookup({...negative('unavailable',reason),retryAfterMs:delay},A).result.retryAfterMs,delay);
 for(const delay of [0,-1,10001,.5,NaN,Infinity,'500'])assert.throws(()=>validatedLookup({...negative('unavailable','nativeLookupBusy'),retryAfterMs:delay},A));
 assert.throws(()=>validatedLookup({...negative('notFound'),retryAfterMs:1000},A));assert.throws(()=>validatedLookup({...negative('unavailable','other'),retryAfterMs:1000},A));
});
test('one explicit lookup opens its immutable exact target and confirms presentation only after true ACK',async()=>{
 const f=fixture(),request={...A},pending=f.navigation.open(request);request.nativeSessionId=B.nativeSessionId;assert.equal(f.lookups.length,1);assert.deepEqual(f.lookups[0].request,A);assert.equal(Object.isFrozen(f.lookups[0].request),true);
 f.lookups[0].resolve(mochi());await flush();assert.equal(f.opens.length,1);assert.equal(f.opens[0].target.sessionId,'cid-one');assert.equal(f.presented,0);f.opens[0].resolve(true);await pending;assert.equal(f.presented,1);assert.equal(f.navigation.state(A).opened,true);assert.equal(f.navigation.isPending(A),false);
});
test('invalid requests, unavailable context and duplicate pending requests issue no extra lookup',async()=>{
 const f=fixture();await f.navigation.open({...A,nativeSessionId:'not-a-thread'});f.current=false;await f.navigation.open(A);assert.equal(f.lookups.length,0);f.current=true;const pending=f.navigation.open(A);await f.navigation.open(A);assert.equal(f.lookups.length,1);f.lookups[0].resolve(negative('notSupported'));await pending;assert.equal(f.opens.length,0);
});
test('obsolete lookup A cannot dispatch an opening after a newer B intent',async()=>{
 const f=fixture(),a=f.navigation.open(A),b=f.navigation.open(B);assert.equal(f.lookups.length,2);
 f.lookups[1].resolve({...mochi(B),conversationId:'cid-two',open:{...mochi(B).open,conversationId:'cid-two'}});await flush();assert.equal(f.opens.length,1);assert.equal(f.opens[0].target.sessionId,'cid-two');
 f.lookups[0].resolve(mochi(A));await a;assert.equal(f.opens.length,1);f.opens[0].resolve(true);await b;assert.equal(f.presented,1);assert.equal(f.navigation.state(A),undefined);
});
test('navigation/leave invalidates a pending lookup before dispatch without claiming remote cancellation',async()=>{
 for(const dispose of [false,true]){const f=fixture(),a=f.navigation.open(A);dispose?f.navigation.dispose():f.navigation.invalidate();f.lookups[0].resolve(mochi());await a;assert.equal(f.opens.length,0);assert.equal(f.navigation.isPending(A),false);}
});
test('after open A was dispatched, a later B intent guards feedback/hide without pretending to undo A',async()=>{
 const f=fixture(),a=f.navigation.open(A);f.lookups[0].resolve(mochi());await flush();assert.equal(f.opens.length,1);
 const b=f.navigation.open(B);f.lookups[1].resolve(mochi(B));await flush();assert.equal(f.opens.length,2);
 f.opens[1].resolve(true);await b;assert.equal(f.presented,1);f.opens[0].reject(Error('late private failure'));await a;assert.equal(f.presented,1);assert.equal(f.navigation.state(A),undefined);assert.doesNotMatch(f.navigation.state(B).message,/failure|não abriu/);
});
test('synchronous redraw invalidation at opening phase cannot slip past the dispatch guard',async()=>{
 const f=fixture();let invalidated=false;f.onChange=()=>{if(!invalidated&&f.navigation.state(A)?.phase==='opening'){invalidated=true;f.navigation.invalidate();}};const a=f.navigation.open(A);f.lookups[0].resolve(mochi());await a;assert.equal(f.opens.length,0);
});
test('inactive/retired capability prevents opening even if lookup claims a resolved destination',async()=>{
 const f=fixture();f.unavailable='Este pet não tem chat disponível.';const a=f.navigation.open(A);f.lookups[0].resolve(mochi());await a;assert.equal(f.opens.length,0);assert.match(f.navigation.state(A).message,/não tem chat/);
});
test('lookup errors and incompatible responses are recoverable and do not expose private detail',async()=>{
 const f=fixture();let a=f.navigation.open(A);f.lookups[0].reject(Error('/private/key detail'));await a;assert.doesNotMatch(f.navigation.state(A).message,/private|key/);a=f.navigation.open(A);f.lookups[1].resolve(mochi(B));await a;assert.equal(f.opens.length,0);assert.match(f.navigation.state(A).message,/identificação/);
});
test('false and failed openings restore control; retry performs a fresh explicit lookup',async()=>{
 for(const failure of [false,'error']){const f=fixture();let a=f.navigation.open(A);f.lookups[0].resolve(mochi());await flush();failure==='error'?f.opens[0].reject(Error('private')):f.opens[0].resolve(false);await a;assert.equal(f.presented,0);assert.equal(f.navigation.isPending(A),false);assert.match(f.navigation.state(A).message,/Tenta novamente/);a=f.navigation.open(A);assert.equal(f.lookups.length,2);f.lookups[1].resolve(negative('notSupported'));await a;}
});
test('manual retry delay never triggers automatic requests and expires by the local clock',async()=>{
 const f=fixture(),a=f.navigation.open(A);f.lookups[0].resolve({...negative('unavailable','nativeLookupThrottled'),retryAfterMs:2000});await a;assert.equal(f.navigation.retryWait(A),2);assert.equal(f.navigation.hasRetryWait,true);await f.navigation.open(A);assert.equal(f.lookups.length,1);f.now=2500;assert.equal(f.navigation.retryWait(A),0);await flush();assert.equal(f.lookups.length,1);const b=f.navigation.open(A);assert.equal(f.lookups.length,2);f.lookups[1].resolve(negative('notSupported'));await b;
});
test('different rows/providers remain independently callable while the latest user intent wins',async()=>{
 const f=fixture(),copilot={...B,provider:'copilot'},a=f.navigation.open(A),b=f.navigation.open(copilot);assert.deepEqual(f.lookups.map(r=>r.request.provider),['codex','copilot']);f.lookups[0].resolve(negative('unavailable','nativeLookupBusy'));f.lookups[1].resolve(external(copilot));await flush();assert.equal(f.opens.length,1);assert.equal(f.opens[0].target.provider,'copilot');f.opens[0].resolve(true);await Promise.all([a,b]);
});

test('synchronous invalidation while publishing located/ACK feedback cannot issue a query or hide the new context',async()=>{
 const initial=fixture();let invalidated=false;initial.onChange=()=>{if(!invalidated){invalidated=true;initial.navigation.invalidate();}};await initial.navigation.open(A);assert.equal(initial.lookups.length,0);
 const f=fixture();let changed=false;f.onChange=()=>{if(!changed&&f.navigation.state(A)?.phase==='result'){changed=true;f.navigation.invalidate();}};const a=f.navigation.open(A);f.lookups[0].resolve(mochi());await flush();f.opens[0].resolve(true);await a;assert.equal(f.presented,0);assert.equal(f.opens.length,1);
});

test('positive and scoped negative navigation evidence cannot substitute another source scope',()=>{
 for(const result of [mochi(),external(),negative('notFound','notFoundInNativeScope')])assert.throws(()=>validatedLookup({...result,library:{...result.library,scope:'providerGlobal'}},A));
 const result=external();assert.throws(()=>validatedLookup({...result,native:{...result.native,scope:'providerGlobal'}},A));
 const stale=external();stale.native.coverage='partial';stale.native.stale=true;assert.ok(validatedLookup(stale,A).target);
 assert.ok(validatedLookup(mochi(),A).target);
});
