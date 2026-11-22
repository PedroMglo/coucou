import {test} from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
test('lookup bridge sends only the explicit readonly request namespace and propagates native refusal',async()=>{
 const saved=internals.invoke,calls=[],request={provider:'codex',nativeSessionId:'00000000-0000-4000-8000-000000000001',model:'forbidden',openTerminal:true};let fail=false;
 try{internals.invoke=async(command,args)=>{calls.push({command,args});if(fail)throw Error('Janela não autorizada');return{schemaVersion:1};};
 assert.deepEqual(await Bridge.petSessionLookup(request),{schemaVersion:1});assert.deepEqual(calls[0],{command:'pet_session_lookup',args:{request:{provider:request.provider,nativeSessionId:request.nativeSessionId}}});fail=true;await assert.rejects(Bridge.petSessionLookup(request),/Janela não autorizada/);assert.ok(calls.every(c=>c.command==='pet_session_lookup'));
 }finally{internals.invoke=saved;}
});
