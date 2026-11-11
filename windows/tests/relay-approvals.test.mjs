import {installFakeDom} from './fakedom.mjs';
import {test} from 'node:test';import assert from 'node:assert/strict';
import {parseRelayApprovalStatus} from '../src/core/relay-approvals.ts';
import {buildRelayApprovalRouting} from '../src/settings/relay-approvals.ts';
installFakeDom();
const value=(mode='terminal',revision='1')=>({mode,revision,scope:'processOnly',developmentBuild:true,restartMode:'terminal'});
const flush=async()=>{for(let i=0;i<10;i++)await Promise.resolve();};
test('unknown routing and persistent/grant-shaped responses are not confirmed modes',()=>{
 for(const bad of [null,{}, {...value(),mode:'allow'},{...value(),revision:'01'},{...value(),revision:1},{...value(),revision:'18446744073709551616'},{...value(),scope:'persistent'}])assert.throws(()=>parseRelayApprovalStatus(bad));
 assert.deepEqual(parseRelayApprovalStatus(value()),value());
});
test('the control uses confirmed revision, rejects double submission and never displays optimistic success',async()=>{
 let finish,writes=0;const f=buildRelayApprovalRouting({read:async()=>value('coucou'),change:request=>{writes++;assert.deepEqual(request,{mode:'terminal',revision:'1'});return new Promise(resolve=>finish=resolve);}});await flush();
 const buttons=f.el.querySelectorAll('button');buttons[0].onclick();buttons[0].onclick();assert.equal(writes,1);assert.match(f.el.find('p')[1].textContent,/no Coucou/);finish(value('terminal','2'));await flush();assert.match(f.el.find('p')[1].textContent,/no terminal/);
});
test('an uncertain change disables mutation until explicit readback and never retries the write',async()=>{
 let writes=0;const f=buildRelayApprovalRouting({read:async()=>value(),change:async()=>{writes++;throw Error('ACK perdido');}});await flush();const buttons=f.el.querySelectorAll('button');buttons[1].onclick();await flush();assert.equal(writes,1);assert.match(f.el.find('p')[2].textContent,/ACK perdido/);assert.equal(buttons[0].disabled,true);assert.equal(buttons[1].disabled,true);buttons[2].onclick();await flush();assert.equal(buttons[1].disabled,false);assert.equal(writes,1);
});
