import {test} from 'node:test';
import assert from 'node:assert/strict';
import {SessionListLoading,mergeManagedReceipt} from '../src/desktop/session-list-loading.ts';
const a={pet:'agent_codex',provider:'codex'},b={pet:'agent_copilot',provider:'copilot'};
const target=(owner,id,origin='external')=>({petId:owner.pet,provider:owner.provider,origin,sessionId:id});
const row=(owner,id,origin='external')=>({target:target(owner,id,origin),title:id,cwd:'',updatedAt:1,state:'Disponível',capabilities:{history:true,send:true,cancel:true}});
const receipt=(id,over={})=>({target:target(a,id),handle:id,generation:1,phase:'ready',turn:null,materialized:true,terminalLaunched:false,cwd:'',model:null,notice:null,...over});
const tick=()=>new Promise(resolve=>setImmediate(resolve));
function setup(changed=()=>{}){
 const requests=[];const loader=source=>owner=>new Promise((resolve,reject)=>requests.push({source,owner,resolve,reject}));
 const list=new SessionListLoading({mochi:loader('mochi'),external:loader('external'),managed:loader('managed')},changed);
 const get=(source,pet=a.pet)=>requests.filter(r=>r.source===source&&r.owner.pet===pet).at(-1);
 list.reset(a);return {list,requests,get};
}
test('fast pet conversations publish while external and managed queries remain pending',async()=>{
 const {list,get}=setup();list.refresh();await tick();get('mochi').resolve([row(a,'local','mochi')]);await tick();
 assert.equal(list.state.mochi.rows[0].title,'local');assert.equal(list.state.mochi.loading,false);assert.equal(list.state.external.loading,true);assert.equal(list.state.managed.loading,true);
 get('managed').resolve([receipt('new')]);await tick();assert.equal(list.state.managed.rows[0].handle,'new');assert.equal(list.state.external.loading,true);
});
test('repeated refresh deduplicates each still pending query, including owner A to B to A',async()=>{
 const {list,requests,get}=setup();list.refresh();list.refresh(true);await tick();assert.equal(requests.length,3);
 list.reset(b);list.refresh();await tick();assert.equal(requests.length,5);
 list.reset(a);list.refresh(true);await tick();assert.equal(requests.length,5);
 get('external',b.pet).resolve([row(b,'wrong-owner')]);await tick();assert.deepEqual(list.state.external.rows,[]);assert.equal(list.state.external.loading,true);
 get('external').resolve([row(a,'a')]);await tick();assert.equal(list.state.external.rows[0].title,'a');assert.equal(list.state.external.loading,false);
});
test('late owner results cannot replace data, errors or loading in the new owner',async()=>{
 const {list,get}=setup();list.refresh();await tick();list.reset(b);list.refresh();await tick();
 get('mochi',b.pet).resolve([row(b,'b','mochi')]);get('mochi').reject(Error('old error'));get('external').resolve([row(a,'a')]);await tick();
 assert.equal(list.state.mochi.rows[0].title,'b');assert.equal(list.state.mochi.error,null);assert.equal(list.state.external.loading,true);assert.deepEqual(list.state.external.rows,[]);
});
test('failure retains last successful data and automatic refresh never retries a failed source',async()=>{
 const {list,requests,get}=setup();list.refresh();await tick();get('mochi').resolve([row(a,'saved','mochi')]);await tick();list.refresh();await tick();get('mochi').reject(Error('offline'));await tick();
 assert.equal(list.state.mochi.phase,'partial');assert.equal(list.state.mochi.rows[0].title,'saved');assert.equal(list.state.mochi.error,'offline');
 const count=requests.length;list.refresh();list.refresh();await tick();assert.equal(requests.length,count);
 list.refresh(true,'mochi');await tick();assert.equal(requests.length,count+1);get('mochi').resolve([]);await tick();assert.equal(list.state.mochi.phase,'ready');assert.deepEqual(list.state.mochi.rows,[]);assert.equal(list.state.mochi.error,null);
});
test('an initial error is unknown, while a successful empty result is ready and known',async()=>{
 const {list,get}=setup();list.refresh();await tick();get('external').reject(Error('native failed'));get('mochi').resolve([]);await tick();
 assert.equal(list.state.external.phase,'error');assert.equal(list.state.external.loading,false);assert.equal(list.state.mochi.phase,'ready');assert.deepEqual(list.state.mochi.rows,[]);
});
test('no provider and non Codex owner do not issue inapplicable resource queries',async()=>{
 const {list,requests}=setup();list.reset({pet:'no-chat',provider:''});list.refresh();await tick();assert.deepEqual(requests.map(r=>r.source),['external']);assert.equal(list.state.mochi.phase,'notApplicable');assert.equal(list.state.managed.phase,'notApplicable');
});
test('a late managed inventory merges confirmed create and release without losing unrelated rows',async()=>{
 const {list,get}=setup();list.refresh();await tick();list.managedReceipt(receipt('new'));list.managedReceipt(receipt('old',{phase:'released'}));
 assert.equal(list.state.managed.rows[0].handle,'new');assert.equal(list.state.managed.loading,true);
 get('managed').resolve([receipt('old'),receipt('other')]);await tick();assert.deepEqual(list.state.managed.rows.map(r=>r.handle),['other','new']);assert.equal(list.state.managed.phase,'ready');
});
test('managed receipts are owner bound, retain latest metadata and respect generation changes',async()=>{
 const {list,get}=setup();list.refresh();await tick();list.managedReceipt({...receipt('wrong'),target:target(b,'wrong')});assert.equal(list.state.managed.rows.length,0);
 list.managedReceipt(receipt('same',{notice:'new ACK'}));get('managed').resolve([receipt('same',{notice:'old list'})]);await tick();assert.equal(list.state.managed.rows[0].notice,'new ACK');
 assert.deepEqual(mergeManagedReceipt([receipt('g',{generation:2})],receipt('g',{phase:'released'})),[receipt('g',{generation:2})]);
});
test('a close detaches publications while retaining pending-query dedup on reopen',async()=>{
 const {list,requests,get}=setup();list.refresh();await tick();list.reset(null);get('mochi').resolve([row(a,'closed','mochi')]);await tick();assert.deepEqual(list.state.mochi.rows,[]);
 list.reset(a);list.refresh();await tick();assert.equal(requests.filter(r=>r.source==='external').length,1);assert.equal(requests.filter(r=>r.source==='mochi').length,2);
});
test('synchronous failure and malformed lists terminate their own loading with a retryable error',async()=>{
 const list=new SessionListLoading({mochi:()=>{throw Error('sync');},external:async()=>null,managed:async()=>[]},()=>{});list.reset(a);list.refresh();await tick();
 assert.equal(list.state.mochi.error,'sync');assert.equal(list.state.external.phase,'error');assert.equal(list.state.external.loading,false);assert.equal(list.state.managed.phase,'ready');
});
test('navigation during publication prevents issuing leftover queries for the old owner',async()=>{
 let list,enabled=false;list=new SessionListLoading({mochi:async()=>[],external:async()=>{throw Error('should not run');},managed:async()=>[]},()=>{if(enabled){enabled=false;list.reset(b);}});list.reset(a);enabled=true;list.refresh();await tick();assert.equal(list.state.external.loading,false);assert.equal(list.state.managed.phase,'notApplicable');
});
