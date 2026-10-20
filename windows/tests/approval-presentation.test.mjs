import {test} from 'node:test';import assert from 'node:assert/strict';
import {approvalContext,sameApproval,canAnswerApproval,presentApproval} from '../src/core/approval-presentation.ts';
const candidate=(id='A')=>({requestId:id,sessionId:'session-'+id,pillId:'agent_codex',presentationReady:false});
const deferred=()=>{let resolve;return {promise:new Promise(r=>resolve=r),finish:v=>resolve(v)};};
const flush=()=>new Promise(r=>setTimeout(r,0));
function fixture(){let current=candidate(),visible=true;const calls=[];const context=approvalContext(current,'approval');return {context,calls,setCurrent:value=>current=value,setVisible:value=>visible=value,host:{current:c=>sameApproval(c,current),visible:()=>visible,prepare:async()=>true,acknowledge:async c=>{calls.push(['ack',c.requestId]);return true;},ready:c=>calls.push(['ready',c.requestId]),handoff:c=>calls.push(['handoff',c.requestId])}};}
test('ACK waits for completed presentation and native confirmation before enabling a decision',async()=>{
 const f=fixture(),paint=deferred(),native=deferred();f.host.prepare=()=>paint.promise;f.host.acknowledge=c=>{f.calls.push(['ack',c.requestId]);return native.promise;};
 const pending=presentApproval(f.context,f.host);await flush();assert.deepEqual(f.calls,[]);paint.finish(true);await flush();assert.deepEqual(f.calls,[['ack','A']]);native.finish(true);assert.equal(await pending,true);assert.deepEqual(f.calls,[['ack','A'],['ready','A']]);
});
for(const failure of ['not-visible','prepare-false','native-false','throws'])test(`${failure} never claims presentation or answers anything`,async()=>{
 const f=fixture();if(failure==='not-visible')f.setVisible(false);if(failure==='prepare-false')f.host.prepare=async()=>false;if(failure==='native-false')f.host.acknowledge=async()=>false;if(failure==='throws')f.host.prepare=async()=>{throw Error('unavailable');};
 assert.equal(await presentApproval(f.context,f.host),false);assert.deepEqual(f.calls,[['handoff','A']]);
});
test('a newer B during preparation prevents ACK for A and never hands off B',async()=>{
 const f=fixture(),paint=deferred();f.host.prepare=()=>paint.promise;const pending=presentApproval(f.context,f.host);f.setCurrent(candidate('B'));paint.finish(true);
 assert.equal(await pending,false);assert.deepEqual(f.calls,[['handoff','A']]);
});
test('a late native A receipt cannot enable or close B',async()=>{
 const f=fixture(),native=deferred();f.host.acknowledge=c=>{f.calls.push(['ack',c.requestId]);return native.promise;};const pending=presentApproval(f.context,f.host);await flush();f.setCurrent(candidate('B'));native.finish(true);
 assert.equal(await pending,false);assert.deepEqual(f.calls,[['ack','A'],['handoff','A']]);
});
test('an answer needs its captured request, visible surface and confirmed presentation',()=>{
 const a=candidate(),context=approvalContext(a,'approval');assert.equal(canAnswerApproval(context,a,true),false);a.presentationReady=true;
 assert.equal(canAnswerApproval(context,a,true),true);assert.equal(canAnswerApproval(context,a,false),false);assert.equal(canAnswerApproval(context,{...candidate('B'),presentationReady:true},true),false);
 assert.equal(canAnswerApproval(context,{...a,sessionId:'other'},true),false);assert.equal(canAnswerApproval(context,{...a,pillId:'agent_copilot'},true),false);
});
test('captured IDs are immutable and never rebind when the source object changes',()=>{
 const a=candidate(),context=approvalContext(a,'question');a.requestId='B';assert.equal(context.requestId,'A');assert.equal(Object.isFrozen(context),true);assert.equal(sameApproval(context,a),false);
});
