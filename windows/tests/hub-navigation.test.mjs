import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createHubNavigation} from '../src/hub/navigation.ts';
test('a destination retained before startup is applied after the listener can read it',async()=>{
 const pages=[];const navigate=createHubNavigation(async()=>'pets:agent_system_guardian',()=>0,async p=>pages.push(p));
 await navigate();assert.deepEqual(pages,['pets:agent_system_guardian']);
});
test('a delayed startup destination cannot replace a subsequent user choice',async()=>{
 let finish,revision=0;const pages=[];const navigate=createHubNavigation(()=>new Promise(r=>finish=r),()=>revision,async p=>pages.push(p));
 const pending=navigate();revision++;finish('pets');await pending;assert.deepEqual(pages,[]);
});
test('out-of-order native reads preserve the newer destination',async()=>{
 const replies=[],pages=[];const navigate=createHubNavigation(()=>new Promise(r=>replies.push(r)),()=>0,async p=>pages.push(p));
 const older=navigate(),newer=navigate();replies[1]('notes');await newer;replies[0]('pets');await older;assert.deepEqual(pages,['notes']);
});
test('failed lookup cannot invent or apply a destination',async()=>{
 const pages=[];const navigate=createHubNavigation(async()=>{throw Error('unavailable');},()=>0,async p=>pages.push(p));
 await assert.rejects(navigate(),/unavailable/);assert.deepEqual(pages,[]);
});
