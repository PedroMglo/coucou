import {test} from 'node:test';
import assert from 'node:assert/strict';
import {NativePetSpeech,nativeSpeechTarget,petDialogueOrigin} from '../src/mochi/native-speech.ts';
const pet=(id='A',owned=false)=>({id:'agent_codex',message:'Resultado',activity:'done',session:{id,backend:'codex',owned,cwd:'/project',title:'Origem '+id}});
const row=(id='A',extra={})=>({target:{petId:'agent_codex',provider:'codex',origin:'external',sessionId:id,...extra}});
test('native speech captures A independently of current B, mutations and duplicate snapshots',()=>{
 const d=new NativePetSpeech(),a=pet();d.observe(a,0);const speech=d.entries.get(a.id);
 a.session.id='mutated';d.observe({...pet('B'),message:undefined},100);
 assert.equal(speech.session.id,'A');assert.equal(d.current(speech,100),true);
 d.observe(pet(),200);assert.equal(d.entries.get(a.id),speech);assert.equal(speech.at,0);
 assert.equal(nativeSpeechTarget(speech,[row('B'),row()]).sessionId,'A');
 assert.equal(d.current(speech,9001),false);
});
test('identical text from a new source replaces the entry and invalidates a delayed lookup',async()=>{
 const d=new NativePetSpeech();d.observe(pet(),0);const a=d.entries.get('agent_codex');let complete;
 const lookup=new Promise(resolve=>complete=resolve);d.observe(pet('B'),1);complete([row()]);
 const target=nativeSpeechTarget(a,await lookup);assert.equal(target.sessionId,'A');assert.equal(d.current(a,2),false);
 assert.equal(d.entries.get('agent_codex').session.id,'B');
});
test('missing, ambiguous and wrong owner/backend sources never substitute a current conversation',()=>{
 const d=new NativePetSpeech();d.observe(pet(),0);const a=d.entries.get('agent_codex');
 for(const rows of [[],[row('B')],[row(),row()],[row('A',{provider:'copilot'})],[row('A',{petId:'agent_copilot'})]])assert.equal(nativeSpeechTarget(a,rows),undefined);
 d.observe({...pet(),session:undefined},1);assert.equal(nativeSpeechTarget(d.entries.get('agent_codex'),[row()]),undefined);
});
test('owned native IDs cannot be invented as external or as Mochi conversation IDs',()=>{
 const d=new NativePetSpeech();d.observe(pet('A',true),0);const a=d.entries.get('agent_codex');
 assert.equal(nativeSpeechTarget(a,[row()]),undefined);
 assert.equal(nativeSpeechTarget(a,[row('A',{origin:'mochi'})]),undefined);
 for(const conversationId of ['canonical-cid','A'])assert.equal(nativeSpeechTarget(a,[row('A',{origin:'mochi',conversationId})]),undefined);
 d.observe(pet('A',false),1);const unowned=d.entries.get('agent_codex');
 assert.equal(nativeSpeechTarget(unowned,[row('A',{origin:'mochi',conversationId:'A'})]),undefined);
 const canonical=row();const target=nativeSpeechTarget(unowned,[canonical]);canonical.target.sessionId='B';
 assert.equal(target.sessionId,'A');assert.equal(Object.isFrozen(target),true);
});
test('a new decision B has its own origin even while the public message from A remains readable',()=>{
 const d=new NativePetSpeech();d.observe(pet(),0);const a=d.entries.get('agent_codex'),b=pet('B').session;
 assert.match(petDialogueOrigin(false,b,undefined,undefined,a),/#A/);
 assert.match(petDialogueOrigin(true,b,undefined,'Mochi #old',a),/#B/);
 assert.equal(petDialogueOrigin(true,b,row('decision-B').target,'Mochi #old',a),'Terminal #decision');
 assert.equal(petDialogueOrigin(true,undefined,undefined,'Mochi #old',a),'');
});
