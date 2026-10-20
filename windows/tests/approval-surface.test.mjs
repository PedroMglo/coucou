import {test} from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals,calls,sent} from './tauri.mjs';
const rect={x:40,y:0,w:640,h:184};
for(const reason of ['closed','new-request'])test(`a delayed old rectangle cannot open the surface after ${reason}`,async()=>{
 const old=internals.invoke;let finish,current=true;calls.length=0;
 internals.invoke=(command,args)=>{calls.push([command,args]);return command==='set_island_rect'?new Promise(resolve=>finish=resolve):Promise.resolve(null);};
 try{const opening=Bridge.prepareApprovalSurface(rect,()=>current);assert.equal(sent('set_island_rect').length,1);current=false;finish();assert.equal(await opening,false);assert.deepEqual(sent('set_collapsed'),[]);assert.deepEqual(sent('approval_presented'),[]);}
 finally{internals.invoke=old;}
});
test('a refused rectangle cannot claim presentation or open the window',async()=>{
 const old=internals.invoke;calls.length=0;internals.invoke=(command,args)=>{calls.push([command,args]);return Promise.reject(Error('native rejected'));};
 try{await assert.rejects(Bridge.prepareApprovalSurface(rect,()=>true),/native rejected/);assert.deepEqual(sent('set_collapsed'),[]);assert.deepEqual(sent('approval_presented'),[]);}
 finally{internals.invoke=old;}
});
