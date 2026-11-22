import {test} from 'node:test';
import assert from 'node:assert/strict';
import {RequestDestinations} from '../src/core/request-destinations.ts';
const target=(pet='ai_ollama',cid='cid-A')=>({petId:pet,provider:pet==='ai_ollama'?'ollama':pet,origin:'mochi',sessionId:cid,conversationId:cid});
test('routing retains the canonical logical owner independently of the CLI backend',()=>{
 const routes=new RequestDestinations(),native=target('integration_github','github-A');
 const saved=routes.add({id:'github-request',provider:'codex',target:native});
 native.sessionId='mutated';assert.equal(saved.target.sessionId,'github-A');assert.equal(saved.target.provider,'integration_github');
 assert.equal(routes.forPet('agent_codex'),null);assert.equal(routes.forPet('integration_github'),saved);assert.equal(Object.isFrozen(saved.target),true);
});
test('simultaneous conversations stay distinct and a reused request ID cannot change destination',()=>{
 const routes=new RequestDestinations(),a=routes.add({id:'A',target:target()}),b=routes.add({id:'B',target:target('ai_ollama','cid-B')});
 assert.equal(routes.add({id:'A',target:target('ai_ollama','cid-C')}),null);assert.equal(routes.current(a),true);assert.equal(routes.current(b),true);
 routes.end('A');assert.equal(routes.current(a),false);assert.equal(routes.forPet('ai_ollama'),b);
});
test('ended requests cannot return through a late pending read, including after tombstone eviction',()=>{
 const routes=new RequestDestinations(),request={id:'old',target:target()},epoch=routes.beginRead();
 routes.end('old');for(let i=0;i<300;i++)routes.end('other-'+i);
 assert.equal(routes.retired('old'),false,'test crosses the bounded tombstone window');
 assert.equal(routes.applyRead(epoch,[request]),false);assert.equal(routes.forPet('ai_ollama'),null);
});
test('a snapshot racing a newer event is discarded; a fresh snapshot can retire absent routes',()=>{
 const routes=new RequestDestinations(),epoch=routes.beginRead(),newer=routes.add({id:'new',target:target()});
 assert.equal(routes.applyRead(epoch,[]),false);assert.equal(routes.current(newer),true);
 assert.equal(routes.applyRead(routes.beginRead(),[]),true);assert.equal(routes.current(newer),false);
});
test('routing memory is bounded and an evicted entry cannot authorize a retry',()=>{
 const routes=new RequestDestinations(),first=routes.add({id:'first',target:target()});
 for(let i=0;i<200;i++)routes.add({id:'new-'+i,target:target('ai_ollama','cid-'+i)});
 assert.equal(routes.size,128);assert.equal(routes.current(first),false);
});
