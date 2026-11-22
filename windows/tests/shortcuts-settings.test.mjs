import test from 'node:test';import assert from 'node:assert/strict';
import {ShortcutPreferenceWriter,parseShortcutPatchReceipt} from '../src/core/shortcuts-settings.ts';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
const binding={keys:'Ctrl+Alt+Space',enabled:true};
function receipt(bindings={openChat:binding}){return {schemaVersion:1,bindings,changed:true,newRegistrationRequiresReview:true,status:{schemaVersion:1,phase:'ready',newEffectsAllowed:true,message:'',suspendedOperations:['shortcuts']}};}
test('shortcut data receipt refuses unsupported collections and success without the persistent barrier',()=>{
 assert.equal(parseShortcutPatchReceipt(receipt()).newRegistrationRequiresReview,true);
 for(const v of [{...receipt(),newRegistrationRequiresReview:false},{...receipt(),status:{...receipt().status,suspendedOperations:[]}},{...receipt(),bindings:{openChat:{keys:'a'.repeat(257),enabled:true}}}])assert.throws(()=>parseShortcutPatchReceipt(v));
 const bindings=JSON.parse('{"__proto__":{"keys":"Super+P","enabled":true}}');assert.equal(Object.getPrototypeOf(parseShortcutPatchReceipt(receipt(bindings)).bindings),null);
});
test('field writer captures per-action before/after and never forwards a complete general settings snapshot',async()=>{
 let current={legacyUnknown:{keys:'Super+P',enabled:true}},observation=0;const sent=[];
 const writer=new ShortcutPreferenceWriter({current:()=>current,observation:()=>observation,alive:()=>true,save:async p=>{sent.push(p);return receipt({...current,openChat:binding});},adopt:b=>{current=b;}});
 await writer.change('openChat',binding);assert.deepEqual(sent,[{changes:{openChat:{before:null,after:binding}}}]);assert.equal(current.legacyUnknown.keys,'Super+P');
 await writer.reset();assert.deepEqual(sent[1],{changes:{openChat:{before:binding,after:null}}});assert.equal(Object.hasOwn(sent[1].changes,'legacyUnknown'),false);
});
test('field writer has no retry and late ACK cannot overwrite a newer observation or a closed view',async()=>{
 for(const close of [false,true]){let done,current={},observation=0,alive=true,adopts=0;const pending=new Promise(r=>{done=r;});const writer=new ShortcutPreferenceWriter({current:()=>current,observation:()=>observation,alive:()=>alive,save:()=>pending,adopt:()=>{adopts++;}});const task=writer.change('openChat',binding);await assert.rejects(writer.change('openChat',{...binding,enabled:false}));if(close)alive=false;else observation++;done(receipt());await task;assert.equal(adopts,0);assert.equal(writer.busy,false);}
 let calls=0;const writer=new ShortcutPreferenceWriter({current:()=>({}),observation:()=>0,alive:()=>true,save:async()=>{calls++;throw Error('ACK lost; inspect');},adopt:()=>assert.fail()});await assert.rejects(writer.change('openChat',binding));assert.equal(calls,1);
});
test('shortcut bridge strips caller, destination and unrelated fields from data changes',async()=>{
 const previous=internals.invoke,sent=[];internals.invoke=async(command,args)=>{sent.push({command,args});return receipt();};
 try{await Bridge.shortcutsSettingsPatch({caller:'providerModel',autostart:true,changes:{openChat:{before:null,after:{...binding,target:'foreign'},unrelated:true}}});assert.deepEqual(sent,[{command:'shortcuts_settings_patch',args:{request:{changes:{openChat:{before:null,after:binding}}}}}]);}finally{internals.invoke=previous;}
});
