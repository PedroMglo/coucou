// Offline final QA: actual Island methods, official mocked Tauri transport.
// No native IPC, DOM, GUI, filesystem ingestion, model or package installation.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {Island} from '../src/island/island.ts';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {rememberDialogue} from '../src/core/chat-dialogue.ts';
import {Sound} from '../src/core/sound.ts';
import {UploadSeq} from '../src/upload/sequence.ts';
import {calls,internals,sent} from './tauri.mjs';

const originalInvoke=internals.invoke;
const originalSound=Sound.enabled;
let ingest;
let currentIds;
const flush=async()=>{for(let i=0;i<16;i++)await Promise.resolve();};
const deferred=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return {promise,resolve,reject};};
internals.invoke=async(cmd,args)=>{
  if(cmd==='ingest_file'){
    calls.push([cmd,args]);
    assert.ok(ingest,'test must explicitly supply an ingest outcome');
    return await ingest(args);
  }
  if(cmd==='chat_conversation'){
    calls.push([cmd,args]);
    return {id:currentIds[args.provider],messages:[],busy:false};
  }
  return originalInvoke(cmd,args);
};
Sound.setEnabled(false);

function reset(){
  UploadSeq.deactivate();calls.length=0;ingest=undefined;
  currentIds={ollama:'fixture-ollama-A',codex:'fixture-codex-B'};
  for(const [provider,id] of Object.entries(currentIds))rememberDialogue(provider,{id,messages:[]});
  State.settings={...DEFAULT_SETTINGS,companionMode:true};
  State.mode='expanded';State.view='overview';State.focusId='ai_ollama';
  State.tasks=[{id:'ai_ollama',pillBadge:null},{id:'agent_codex',pillBadge:null}];
  State.pendingApproval=null;State.paused=false;State.fileDragOver=false;
  State.isPinned=false;State.mouseInIsland={x:320,y:260};
  State.clearAllDialogueContexts();State.chatFeedback.clear();
  State.chatBusyProviders.clear();State.chatResettingProviders.clear();
  State.replaceChatHistory('ollama',[]);State.replaceChatHistory('codex',[]);
  const card=Object.create(Island.prototype);
  const effects={morph:[],gulp:0,emotes:[],geometry:[]};
  card.desktop={companionsActive:true};
  card.engine={animateMorph(value){effects.morph.push(value);},gulp(){effects.gulp++;},triggerEmote(value){effects.emotes.push(value);}};
  card.fsm={forceHome(){},mouseLeft(){},pinned:false};
  card.animateGeometry=()=>effects.geometry.push(card.targetSize());
  card.wasInIsland=true;card.dragHeight=null;card.dragOwner=null;
  card.dropSequence=0;card.providerDrops=new Map();card.providerDropTokens=new Map();
  return {card,effects};
}
function notices(){return calls.filter(([cmd,args])=>cmd==='plugin:event|emit_to'&&args.event==='companions-notice').map(([,args])=>args.payload);}
function presentations(){return calls.filter(([cmd,args])=>cmd==='plugin:event|emit_to'&&args.event==='companions-present');}
async function run(name,body){
  await test(name,async()=>{
    const ctx=reset();
    try{await body(ctx);}
    finally{UploadSeq.deactivate();await flush();}
  });
}

try{
  await run('reopening the pet home restores its exact pending approval or question',async({card})=>{
    State.pendingApproval={pillId:'ai_ollama',requestId:'help-approval'};
    card.openFromDesktop('ai_ollama','overview');assert.equal(State.view,'approval');
    assert.equal(State.pendingApproval.requestId,'help-approval');
    State.pendingApproval={pillId:'ai_ollama',requestId:'help-question',questions:[{id:'q'}]};
    card.openFromDesktop('ai_ollama');assert.equal(State.view,'question');
    assert.equal(State.pendingApproval.requestId,'help-question');
  });
  await run('a completed direct-to-pet delivery retires the hover canvas before opening actions',async({card})=>{
    card.onDragDrop({type:'over'});
    State.setFileContext('ollama',{name:'direct.txt',path:'/fixture/direct.txt',conversationId:'fixture-ollama-A',sequence:2});
    card.openFromDesktop('ai_ollama','choose');await flush();
    assert.equal(State.view,'choose');assert.equal(UploadSeq.isActive,false);
    assert.equal(card.dragHeight,null);assert.equal(card.dragOwner,null);assert.equal(State.fileDragOver,false);
  });
  await run('a different pet waiting for approval cannot block document actions',async({card})=>{
    State.pendingApproval={pillId:'agent_codex',id:'fixture-other-approval'};
    ingest=async()=>({name:'document.txt',path:'/fixture/inbox/document.txt',conversationId:'fixture-ollama-A',sequence:1});
    card.onDragDrop({type:'enter'});
    card.onDragDrop({type:'drop',paths:['/fixture/document.txt'],dropId:'fixture-other-pet'});
    await flush();
    assert.equal(State.view,'choose');assert.equal(UploadSeq.isActive,false);
    assert.equal(State.pendingApproval.id,'fixture-other-approval');
  });
  await run('a received document exposes real actions instead of the pre-drop canvas',async({card})=>{
    ingest=async()=>({name:'document.txt',path:'/fixture/inbox/document.txt',conversationId:'fixture-ollama-A',sequence:1});
    card.onDragDrop({type:'enter'});
    card.onDragDrop({type:'drop',paths:['/fixture/document.txt'],dropId:'fixture-success'});
    await flush();
    assert.equal(State.view,'choose');assert.equal(UploadSeq.isActive,false);
    assert.equal(State.fileFor('ollama')?.name,'document.txt');
  });
  await run('expanded-over-retains-296-and-does-not-reroute',async({card})=>{
    const before=card.targetSize();
    card.onDragDrop({type:'over'});await flush();
    const after=card.targetSize();
    assert.equal(before.h,296);assert.equal(after.h,296);
    assert.equal(State.view,'upload');assert.equal(presentations().length,0);
    assert.deepEqual(card.dragOwner,{provider:'ollama',conversationId:'fixture-ollama-A',pet:'ai_ollama'});
    card.onDragDrop({type:'over'});
    assert.equal(card.targetSize().h,296);
    // A late receiving route must also retain the captured geometry.
    card.openFromDesktop(undefined,'upload');await flush();
    assert.equal(card.targetSize().h,296);assert.equal(presentations().length,0);
    assert.ok(260<card.targetSize().h+14);
    return {before:before.h,during:after.h,afterLateReceivingRoute:card.targetSize().h,presentationRequests:0,lowerPointStillInside:true};
  });

  await run('leave-retains-height-until-navigation-cleans-drag',async({card,effects})=>{
    card.onDragDrop({type:'enter'});
    const owner=card.dragOwner;
    card.onDragDrop({type:'leave'});
    assert.equal(card.targetSize().h,296);assert.equal(State.fileDragOver,false);
    assert.equal(card.dragOwner,owner);assert.equal(UploadSeq.isActive,true);
    assert.equal(effects.morph.at(-1),0);
    card.setView('overview');
    assert.equal(card.dragHeight,null);assert.equal(card.dragOwner,null);
    assert.equal(UploadSeq.isActive,false);assert.equal(State.fileDragOver,false);
    card.onDragDrop({type:'over'});assert.equal(card.targetSize().h,296);
    return {leaveHeight:296,sequenceIntentionallyRetainedOnLeave:true,navigationClearsOwnerAndSequence:true,newDragHeight:296};
  });

  await run('delivery-after-focus-change-uses-captured-owner',async({card,effects})=>{
    const gate=deferred();ingest=()=>gate.promise;
    card.onDragDrop({type:'enter'});
    State.setFocus('agent_codex');
    card.onDragDrop({type:'drop',paths:['/fixture/document.txt'],dropId:'fixture-drop-1'});
    await flush();
    assert.deepEqual(sent('ingest_file'),[{path:'/fixture/document.txt',provider:'ollama',conversationId:'fixture-ollama-A',dropId:'fixture-drop-1'}]);
    gate.resolve({name:'document.txt',path:'/fixture/inbox/document.txt',conversationId:'fixture-ollama-A',sequence:1});
    await flush();
    assert.equal(State.fileFor('ollama')?.name,'document.txt');
    assert.equal(State.fileFor('codex'),null);assert.equal(State.droppedFile,null);
    assert.equal(State.chatFeedback.get('ollama'),'document.txt recebido.');
    assert.equal(State.chatFeedback.has('codex'),false);
    assert.equal(State.focusId,'agent_codex');assert.equal(effects.gulp,0);
    assert.notEqual(State.view,'choose');
    return {ingestOwner:'ollama',conversation:'fixture-ollama-A',focusAfter:'agent_codex',otherPetContextUntouched:true,gulp:effects.gulp};
  });

  await run('captured-conversation-sent-even-after-same-provider-reset',async({card})=>{
    const gate=deferred();ingest=()=>gate.promise;
    card.onDragDrop({type:'enter'});
    rememberDialogue('ollama',{id:'fixture-ollama-new',messages:[]});
    currentIds.ollama='fixture-ollama-new';
    card.onDragDrop({type:'drop',paths:['/fixture/document.txt'],dropId:'fixture-drop-reset'});
    await flush();
    assert.equal(sent('ingest_file')[0].conversationId,'fixture-ollama-A');
    // Simulate the required native identity rejection; never attach to the new dialogue.
    gate.reject(new Error('fixture: conversation changed'));await flush();
    assert.equal(State.fileFor('ollama'),null);
    assert.equal(State.chatFeedback.get('ollama'),'fixture: conversation changed');
    assert.equal(State.view,'prompt');assert.equal(UploadSeq.isActive,false);
    return {capturedConversationPreserved:true,rejectedContextNotInstalled:true,currentView:State.view};
  });

  await run('async-ingest-rejection-after-focus-change-stays-with-origin',async({card,effects})=>{
    const gate=deferred();ingest=()=>gate.promise;
    card.onDragDrop({type:'over'});
    card.onDragDrop({type:'drop',paths:['/fixture/document.txt'],dropId:'fixture-drop-failure'});
    State.setFocus('agent_codex');
    const before=State.view;
    gate.reject(new Error('fixture: ingestion refused'));await flush();
    assert.equal(State.chatFeedback.get('ollama'),'fixture: ingestion refused');
    assert.equal(State.chatFeedback.has('codex'),false);
    assert.equal(State.view,before);assert.equal(effects.gulp,0);
    assert.deepEqual(notices().map(n=>({pet:n.pet,text:n.text})),[{pet:'ai_ollama',text:'fixture: ingestion refused'}]);
    return {feedbackOwner:'ollama',noticePet:'ai_ollama',currentPetViewNotChanged:true};
  });

  await run('native-rejected-after-focus-change-reports-origin',async({card})=>{
    card.onDragDrop({type:'enter'});
    State.setFocus('agent_codex');
    card.onDragDrop({type:'rejected',error:'fixture: native drop refused'});await flush();
    assert.equal(State.chatFeedback.get('ollama'),'fixture: native drop refused');
    assert.equal(State.chatFeedback.has('codex'),false);
    return {feedbackOwner:'ollama',otherProviderUntouched:true};
  });

  await run('native-rejected-cleans-drag-state',async({card,effects})=>{
    card.onDragDrop({type:'over'});
    card.onDragDrop({type:'rejected',error:'fixture: native drop refused'});await flush();
    assert.equal(State.fileDragOver,false);
    assert.equal(card.dragHeight,null);assert.equal(card.dragOwner,null);
    assert.equal(UploadSeq.isActive,false);assert.equal(effects.morph.at(-1),0);
    return {dragStateCleared:true,sequenceStopped:true};
  });

  await run('empty-drop-cleans-without-ingestion',async({card,effects})=>{
    card.onDragDrop({type:'over'});
    card.onDragDrop({type:'drop',paths:[],dropId:'fixture-empty'});await flush();
    assert.equal(sent('ingest_file').length,0);
    assert.equal(State.fileDragOver,false);assert.equal(card.dragOwner,null);assert.equal(card.dragHeight,null);
    assert.equal(UploadSeq.isActive,false);assert.equal(effects.morph.at(-1),0);
    assert.equal(State.view,'overview');
    return {ingestionCalls:0,dragStateCleared:true,view:State.view};
  });
}finally{
  UploadSeq.deactivate();Sound.setEnabled(originalSound);internals.invoke=originalInvoke;
}
