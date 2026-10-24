import test from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {approvalActions} from '../src/views/approval-actions.ts';
installFakeDom();
const choice=(id,scope,allow=true,extra={})=>({id,label:`Native ${scope}`,scope,allow,...extra});
test('native scopes are explicit and buttons send only their local choice ID',()=>{
 const sent=[];const view=approvalActions({kind:'permission',nativeChoices:[choice('local-session','session',true,{nativeResponse:{decision:'injected'}}),choice('local-policy','policy',false,{detail:'{"host":"<script>fixture</script>","action":"deny"}'})]},(...args)=>sent.push(args));
 const buttons=view.el.find('button');assert.equal(buttons.length,2);assert.equal(buttons[0].dataset.scope,'session');assert.equal(buttons[1].dataset.scope,'policy');assert.match(view.el.textContent,/<script>fixture<\/script>/);assert.equal(view.el.find('SCRIPT').length,0);assert.deepEqual(sent,[]);
 buttons[1].fire('click');assert.deepEqual(sent,[[false,'local-policy']]);
});
test('an extended-only native list never gains a generic allow-once action',()=>{
 const view=approvalActions({kind:'permission',nativeChoices:[choice('remember','nativeRemembered')]},()=>{});assert.equal(view.el.find('button').length,1);assert.match(view.el.textContent,/memoriza/);assert.ok(!view.el.textContent.includes('Allow once'));
});
test('malformed empty duplicate or unknown-scope lists cannot fall back to generic approval',()=>{
 for(const nativeChoices of[[],null,[choice('same','once'),choice('same','session')],[choice('unknown','_future')],[{id:'missing',label:'Missing scope',allow:true}]]){const sent=[];const view=approvalActions({kind:'permission',nativeChoices},(...a)=>sent.push(a));assert.equal(view.el.find('button').length,0);assert.deepEqual(sent,[]);assert.match(view.el.textContent,/não estão disponíveis/);}
});
test('one pending response disables every native choice until explicit retry',()=>{
 const sent=[];let view;view=approvalActions({kind:'permission',nativeChoices:[choice('once','once'),choice('session','session')]},(...a)=>{sent.push(a);view.setDisabled(true);});const buttons=view.el.find('button');buttons[0].fire('click');buttons[1].fire('click');assert.deepEqual(sent,[[true,'once']]);assert.ok(buttons.every(b=>b.disabled));view.setDisabled(false);buttons[1].fire('click');assert.deepEqual(sent,[[true,'once'],[true,'session']]);
});
test('legacy local decisions remain plain booleans without a native choice or wire value',()=>{
 const sent=[];const view=approvalActions({kind:'permission',approvalScope:'once'},(...a)=>sent.push(a));view.el.find('button')[0].fire('click');assert.deepEqual(sent,[[true,undefined]]);
});
