import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DesktopLink} from '../src/island/desktop.ts';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {calls,internals,subscriptions} from './tauri.mjs';
import {Sound} from '../src/core/sound.ts';
import {Island} from '../src/island/island.ts';

test('pet-card runtime configures the real shared lifecycle without singleton initialization or waking pets',async()=>{
 const invoke=internals.invoke,interval=window.setInterval,sound=Sound.enabled;
 window.setInterval=()=>0;
 internals.invoke=async(cmd,args)=>{
  if(cmd==='companions_configure'){calls.push([cmd,args]);return{width:1440,height:900,top:0,band:null};}
  if(cmd==='pet_management'){calls.push([cmd,args]);return{sessions:[],providers:[],models:[],summary:{}};}
  return invoke(cmd,args);
 };
 try{
  Sound.setEnabled(false);State.petCardsOnly=true;
  State.settings={...DEFAULT_SETTINGS,companionMode:false,petControlBar:false,petSystemCenter:false,chatProvider:'anthropic'};
  State.tasks=[{id:'ai_ollama',name:'Ollama',state:'idle',color:'#eab308',steps:[],stepIndex:0}];State.focusId='ai_ollama';
  const link=new DesktopLink({openFromDesktop(){},reveal(){},wardrobeFromDesktop(){},dizzyFromDesktop(){}});
  await link.init();await link.launch();
  assert.equal(link.controller,undefined);assert.equal(link.canPickUp(),false);
  assert.ok(calls.some(([cmd])=>cmd==='pet_control_configure'));
  assert.ok(calls.some(([cmd])=>cmd==='pet_management'),'session metadata remains available with pets off');
  assert.equal(calls.some(([cmd])=>cmd==='desktop_info'||cmd.startsWith('desktop_fly')),false);
  assert.equal(subscriptions.some(({event})=>event.startsWith('desktop-mochi-')),false);
  assert.ok(calls.filter(([cmd])=>cmd==='companions_configure').every(([,args])=>args.enabled===false));
  assert.equal(State.chatProvider,'ollama');assert.equal(State.settings.chatProvider,'anthropic');
  assert.equal(calls.some(([cmd])=>cmd==='save_settings'),false);
 }finally{internals.invoke=invoke;window.setInterval=interval;Sound.setEnabled(sound);State.petCardsOnly=false;}
});

test('expired decision synchronization never reopens a contextual card',()=>{
 const saved={mode:State.mode,view:State.view,pending:State.pendingApproval};
 try{
  for(const mode of ['hidden','expanded'])for(const view of ['approval','question']){
   State.mode=mode;State.view=view;State.pendingApproval=null;
   const card=Object.create(Island.prototype);card.presentation={cardsOnly:true};let reopened=false;
   card.setView=()=>{reopened=true;State.mode='expanded';};
   card.collapse=()=>{State.mode='hidden';};
   card.syncDom();
   assert.equal(reopened,false);assert.equal(State.mode,'hidden');assert.notEqual(State.view,view);
  }
 }finally{State.mode=saved.mode;State.view=saved.view;State.pendingApproval=saved.pending;}
});


test('pet overview fits the native viewport and retains a scrollable margin',()=>{
 const saved={height:window.innerHeight,mode:State.mode,view:State.view,settings:State.settings,focus:State.focusId};
 try{
  State.settings={...DEFAULT_SETTINGS,companionMode:true};State.mode='expanded';State.view='overview';State.focusId='ai_ollama';
  const card=Object.create(Island.prototype);card.presentation={cardsOnly:true};card.views=new Map();
  for(const [viewport,expected]of [[560,520],[900,520],[428,404],[320,296]]){window.innerHeight=viewport;const size=card.targetSize();assert.equal(size.w,640);assert.equal(size.h,expected);assert.ok(size.h+24<=viewport);}
  window.innerHeight=undefined;assert.equal(card.targetSize().h,296);
 }finally{window.innerHeight=saved.height;State.mode=saved.mode;State.view=saved.view;State.settings=saved.settings;State.focusId=saved.focus;}
});
test('Spoty audio has the overview viewport while decision dimensions stay intact',()=>{
 const saved={height:window.innerHeight,mode:State.mode,view:State.view,settings:State.settings,focus:State.focusId,pending:State.pendingApproval};
 try{
  window.innerHeight=560;State.settings={...DEFAULT_SETTINGS,companionMode:true};State.mode='expanded';State.view='overview';State.focusId='integration_spotify';State.pendingApproval=null;
  const card=Object.create(Island.prototype);card.presentation={cardsOnly:true};card.views=new Map();assert.equal(card.targetSize().h,520);
  window.innerHeight=360;assert.equal(card.targetSize().h,336);window.innerHeight=560;
  State.focusId='ai_ollama';for(const view of ['approval','question']){State.view=view;window.innerHeight=320;const baseline=card.targetSize().h;window.innerHeight=560;assert.equal(card.targetSize().h,baseline);}
 }finally{window.innerHeight=saved.height;State.mode=saved.mode;State.view=saved.view;State.settings=saved.settings;State.focusId=saved.focus;State.pendingApproval=saved.pending;}
});
