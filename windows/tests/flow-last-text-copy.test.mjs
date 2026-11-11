import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {emit} from './tauri.mjs';
import {FlowBridge} from '../src/core/flow.ts';
import {lastTextCopyControl} from '../src/hub/pages/flowsettings.ts';
installFakeDom();
const tick=()=>new Promise(resolve=>setImmediate(resolve));
function view(available){const ctx={status:{prefs:{keepDays:0},lastTextAvailable:available}};const el=lastTextCopyControl(ctx);el.isConnected=true;return{ctx,el,button:el.find('button')[0]};}
function click(button){for(const listener of button.listeners.get('click')??[])listener({});}

test('unknown backend support never calls the new native endpoint',async()=>{
 const original=FlowBridge.copyLast;let calls=0;FlowBridge.copyLast=async()=>{calls++;return true;};const v=view(undefined);
 try{await tick();assert.equal(v.button.disabled,true);click(v.button);await tick();assert.equal(calls,0);assert.match(v.el.textContent,/ainda não suporta/);}finally{v.ctx.onLeave?.();FlowBridge.copyLast=original;}
});
test('pending copy cannot be duplicated or enabled by a history event',async()=>{
 const original=FlowBridge.copyLast;let resolve;let calls=0;FlowBridge.copyLast=()=>{calls++;return new Promise(done=>resolve=done);};const v=view(true);
 try{await tick();click(v.button);assert.equal(v.button.disabled,true);emit('flow-history',null);assert.equal(v.button.disabled,true);click(v.button);assert.equal(calls,1);resolve(true);await tick();assert.match(v.el.textContent,/Último texto copiado\./);assert.equal(v.button.disabled,false);}finally{v.ctx.onLeave?.();FlowBridge.copyLast=original;}
});
test('false and unavailable native outcomes cannot publish copied',async()=>{
 const original=FlowBridge.copyLast;
 for(const result of[false,null]){FlowBridge.copyLast=async()=>result;const v=view(true);try{await tick();click(v.button);await tick();assert.match(v.el.textContent,/não foi confirmada/);assert.doesNotMatch(v.el.textContent,/Último texto copiado\./);}finally{v.ctx.onLeave?.();}}
 FlowBridge.copyLast=original;
});
test('unkept output is available after its event without claiming persisted history',async()=>{
 const v=view(false);await tick();try{assert.equal(v.button.disabled,true);assert.match(v.el.textContent,/apenas em memória/);emit('flow-history',null);assert.equal(v.button.disabled,false);}finally{v.ctx.onLeave?.();}
});
test('listener survives detached page construction and responds after mounting',async()=>{
 const v=view(false);v.el.isConnected=false;
 try{await tick();assert.equal(v.button.disabled,true);v.el.isConnected=true;emit('flow-history',null);assert.equal(v.button.disabled,false);}finally{v.ctx.onLeave?.();}
});
test('leaving before listener registration completes removes the late listener',async()=>{
 const v=view(false);v.el.isConnected=false;v.ctx.onLeave?.();await tick();v.el.isConnected=true;
 assert.throws(()=>emit('flow-history',null),/nothing listens/);assert.equal(v.button.disabled,true);
});
