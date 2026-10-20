import {test} from 'node:test';import assert from 'node:assert/strict';
import {parseManagedUsage,managedUsageDate} from '../src/core/managed-usage.ts';
const value=(used,extra={})=>({providers:{codex:{updated:1700000000,windows:[{label:'5 horas',used,...extra}]}}});
test('known zero remains measured; null, missing, booleans and non-finite percentages stay unknown',()=>{
 assert.equal(parseManagedUsage(value(0)).providers.codex.windows[0].used,0);
 for(const x of [null,undefined,true,false,'42',NaN,Infinity,-1,101]){const v=parseManagedUsage(value(x));assert.equal(v.providers.codex.windows[0].used,null);assert.equal(v.partial,true);}
});
test('source dates are seconds or ISO and bounded without coercion or Invalid Date',()=>{
 assert.equal(managedUsageDate(1700000000),1700000000000);assert.equal(managedUsageDate('2026-10-20T01:00:00Z'),1792458000000);
 for(const x of [null,undefined,true,'invalid','1700000000',0,-1,NaN,Infinity,8_640_000_000_001])assert.equal(managedUsageDate(x),null);
 assert.equal(managedUsageDate(8_640_000_000_000),8_640_000_000_000_000);
});
test('unknown/reset dates are omitted while valid readings and native renewal labels survive',()=>{
 const q=parseManagedUsage(value(42,{reset:'bad',reset_label:'No próximo mês',remaining:null,entitlement:null})).providers.codex;
 assert.equal(q.windows[0].used,42);assert.equal(q.windows[0].resetAt,null);assert.equal(q.windows[0].resetLabel,'No próximo mês');assert.equal(q.partial,true);assert.equal(q.updated,1700000000000);
});
test('malformed provider/window cannot erase other useful provider and window readings',()=>{
 const v=parseManagedUsage({providers:{codex:{updated:null,windows:[null,{used:25}]},copilot:{updated:1700000000,windows:[{used:0}]},broken:null}});
 assert.equal(v.providers.codex.windows[0].used,null);assert.equal(v.providers.codex.windows[1].used,25);assert.equal(v.providers.copilot.windows[0].used,0);assert.equal(v.providers.codex.stale,true);assert.equal(v.providers.broken,undefined);
});
test('missing windows differ from known empty windows and missing envelope is refused',()=>{
 assert.equal(parseManagedUsage({providers:{codex:{}}}).providers.codex.windows,null);assert.deepEqual(parseManagedUsage({providers:{codex:{windows:[]}}}).providers.codex.windows,[]);
 for(const v of [null,{}, {providers:null},{providers:[]}])assert.throws(()=>parseManagedUsage(v),/não estão disponíveis/);
});
test('view strips arbitrary nested fields and bounds all arrays, identifiers and labels',()=>{
 const v=parseManagedUsage({providers:{codex:{updated:1700000000,token:'SECRET',windows:Array.from({length:33},()=>({used:42,token:'SECRET',label:'bad\nname'})),unlimited:Array(17).fill('Livre')},['bad\nprovider']:{}}});
 assert.equal(v.providers.codex.windows.length,32);assert.equal(v.providers.codex.unlimited.length,16);assert.equal(v.providers.codex.windows[0].label,'Utilização');assert.doesNotMatch(JSON.stringify(v),/SECRET|token|bad/);assert.equal(v.partial,true);
});
