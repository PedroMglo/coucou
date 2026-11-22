import {test} from 'node:test';import assert from 'node:assert/strict';
import {sessionTree,uniqueManagedModels,bytesLabel,matchesSearch} from '../src/mochi/management.ts';
const provider={id:'codex',name:'Codex'};
const row=(id,parent='',state='saved')=>({id,parent,state,provider:'codex',name:id,model:'luna',cwd:'/tmp',archived:state==='archived'});
test('management search ignores accents and matches words across title, model and folder',()=>{
 assert.equal(matchesSearch('codigo LUNA','Código do projeto','luna','/tmp'),true);
 assert.equal(matchesSearch('luna ausente','Código do projeto','luna','/tmp'),false);
 const tree=sessionTree([{...row('one'),name:'Código',cwd:'/tmp/projeto'}],[provider],'all','CODIGO projeto',new Set(),new Set());
 assert.equal(tree.filter(e=>e.kind==='session').length,1);
});
test('session tree keeps parents of search hits, nested agents, archive filters and cyclic records without duplicating',()=>{
 const rows=[row('root'),row('child','root'),row('target','child'),row('archived','','archived'),row('cycle-a','cycle-b'),row('cycle-b','cycle-a')];
 let tree=sessionTree(rows,[provider],'all','target',new Set(['codex']),new Set());assert.deepEqual(tree.filter(e=>e.kind==='session').map(e=>e.row.id),['root','child','target']);assert.equal(tree[1].context,true);
 tree=sessionTree(rows,[provider],'archived','',new Set(),new Set());assert.deepEqual(tree.filter(e=>e.kind==='session').map(e=>e.row.id),['archived']);
 tree=sessionTree(rows,[provider],'all','',new Set(),new Set(['codex:cycle-a','codex:cycle-b']));const ids=tree.filter(e=>e.kind==='session').map(e=>e.row.id);assert.equal(new Set(ids).size,ids.length);assert.ok(ids.includes('cycle-a'));assert.ok(ids.includes('cycle-b'));assert.ok(!ids.includes('child'));
});
test('local model aliases share one memory/disk entry; unavailable values stay unknown',()=>{
 const a={name:'model:latest',digest:'abc',loaded:true,clients:[]};const models=uniqueManagedModels([a,{...a,name:'model:other'},{...a,name:'other',digest:'def',loaded:false}]);assert.equal(models.length,2);assert.deepEqual(models[0].aliases,['model:other']);assert.equal(bytesLabel(null),'—');assert.equal(bytesLabel(0),'0 B');assert.equal(bytesLabel(1024**3),'1.0 GiB');
});
