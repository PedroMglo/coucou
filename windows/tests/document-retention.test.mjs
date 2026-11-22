import {test} from 'node:test';import assert from 'node:assert/strict';import {installFakeDom} from './fakedom.mjs';
installFakeDom();const {documentRetention}=await import('../src/views/document-retention.ts');
const item=id=>({id,name:`${id}.txt`,kind:'file',size:3,entries:1,selected:false,purging:false,historyReferences:0});
const owner=(cid='old-cid')=>({provider:'ollama',conversationId:cid,revision:'registry-revision',busy:false,items:[item('copy-a')]});
const recovery=(phase='Committed')=>({id:'recovery-1',provider:'ollama',conversationId:'old-cid',revision:'wal-revision',phase,items:[item('copy-a')]});
const inventory=()=>({owners:[owner()],recoveries:{recoveries:[],unavailable:[]}});
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));const deferred=()=>{let resolve;const promise=new Promise(r=>resolve=r);return{promise,resolve}};
const button=(g,text)=>g.panel.el.find('button').find(b=>b.textContent===text);
function setup(overrides={},options={}){
 const calls=[];const api={petDocumentRetentionInventory:async()=>{calls.push(['inventory']);return inventory();},petDocumentPurgeReview:async(p,c,ids,revision)=>{calls.push(['review',p,c,ids,revision]);return{reviewId:'native-review',provider:p,conversationId:c,items:ids.map(item),recovery:null,expiresInSeconds:110};},petDocumentPurgeRecoveryReview:async id=>{calls.push(['recovery-review',id]);return{reviewId:'recovery-review',provider:'ollama',conversationId:'old-cid',items:[item('copy-a')],recovery:recovery(),expiresInSeconds:110};},petDocumentPurgeCommit:async(p,c,id,consent)=>{calls.push(['commit',p,c,id,consent]);return{provider:p,conversationId:c,operation:consent,complete:true,recoveryId:null,error:null};},petDocumentPurgeCancel:async(...args)=>{calls.push(['cancel',...args]);},petDocumentRemove:async(...args)=>{calls.push(['remove',...args]);},...overrides};
 const panel=documentRetention({provider:'ollama',current:()=>true,changed:()=>{},transport:api,...options});return{panel,calls};
}
async function load(g){button(g,'Gerir cópias guardadas').fire('click');await tick();}
async function review(g){const input=g.panel.el.find('input')[0];input.checked=true;input.fire('change');button(g,'Rever eliminação das cópias').fire('click');await tick();}
test('copy listing and review cannot delete before a separate original-owner confirmation',async()=>{
 const g=setup();await load(g);assert.equal(g.calls.some(c=>c[0]==='commit'),false);await review(g);assert.deepEqual(g.calls.find(c=>c[0]==='review'),['review','ollama','old-cid',['copy-a'],'registry-revision']);assert.equal(g.calls.some(c=>c[0]==='commit'),false);button(g,'Eliminar estas cópias').fire('click');await tick();assert.deepEqual(g.calls.find(c=>c[0]==='commit'),['commit','ollama','old-cid','native-review','deleteOwnedCopies']);
});
test('referenced selected busy and pending records are not offered as purge choices',async()=>{
 const data=inventory();data.owners[0].items=[item('a'),{...item('b'),historyReferences:2},{...item('c'),selected:true},{...item('d'),purging:true}];data.owners.push({...owner('busy'),busy:true});const g=setup({petDocumentRetentionInventory:async()=>data});await load(g);assert.deepEqual(g.panel.el.find('input').map(i=>i.disabled),[false,true,true,true,true]);button(g,'Retirar da seleção').fire('click');await tick();assert.deepEqual(g.calls.find(c=>c[0]==='remove'),['remove','ollama','old-cid','c']);assert.equal(g.calls.some(c=>c[0]==='commit'),false);
});
test('late preview is cancelled after disposal and cannot create a confirmation for a new view',async()=>{
 const gate=deferred(),g=setup({petDocumentPurgeReview:()=>gate.promise});await load(g);const input=g.panel.el.find('input')[0];input.checked=true;input.fire('change');button(g,'Rever eliminação das cópias').fire('click');g.panel.dispose();gate.resolve({reviewId:'late-review',provider:'ollama',conversationId:'old-cid',items:[item('copy-a')],recovery:null,expiresInSeconds:90});await tick();assert.deepEqual(g.calls.find(c=>c[0]==='cancel'),['cancel','ollama','old-cid','late-review']);assert.equal(button(g,'Eliminar estas cópias'),undefined);assert.equal(g.calls.some(c=>c[0]==='commit'),false);
});
test('old controls cannot commit after provider scope changed and disposal cancels exact token',async()=>{
 let current=true;const g=setup({}, {current:()=>current});await load(g);await review(g);const old=button(g,'Eliminar estas cópias');current=false;old.fire('click');await tick();assert.equal(g.calls.some(c=>c[0]==='commit'),false);g.panel.dispose();assert.deepEqual(g.calls.find(c=>c[0]==='cancel'),['cancel','ollama','old-cid','native-review']);
});
test('mismatched native scope cannot create a destructive button',async()=>{
 const g=setup({petDocumentPurgeReview:async()=>({reviewId:'wrong',provider:'codex',conversationId:'old-cid',items:[item('copy-a')],recovery:null,expiresInSeconds:100})});await load(g);await review(g);assert.equal(button(g,'Eliminar estas cópias'),undefined);assert.equal(g.calls.some(c=>c[0]==='commit'),false);assert(g.calls.some(c=>c[0]==='cancel'));
});
test('recovery needs explicit review and partial restore stays incomplete with its recovery',async()=>{
 const data={owners:[],recoveries:{recoveries:[recovery()],unavailable:[]}},g=setup({petDocumentRetentionInventory:async()=>data,petDocumentPurgeCommit:async(...args)=>{g.calls.push(['commit',...args]);return{provider:'ollama',conversationId:'old-cid',operation:'restoreOwnedCopies',complete:false,recoveryId:'recovery-1',error:'Cópia parcialmente eliminada; recuperação conservada.'};}});await load(g);assert.equal(g.calls.some(c=>c[0]==='commit'),false);button(g,'Rever recuperação de cópias').fire('click');await tick();assert.equal(g.calls.some(c=>c[0]==='commit'),false);button(g,'Recuperar cópias completas').fire('click');await tick();assert.deepEqual(g.calls.find(c=>c[0]==='commit'),['commit','ollama','old-cid','recovery-review','restoreOwnedCopies']);assert(g.panel.el.find('p').some(p=>p.textContent.includes('parcialmente eliminada')));assert(button(g,'Rever recuperação de cópias'));
});
test('completed deletion offers metadata cleanup and never a promise to restore removed files',async()=>{
 const r=recovery('Cleaned'),data={owners:[],recoveries:{recoveries:[r],unavailable:[]}},g=setup({petDocumentRetentionInventory:async()=>data,petDocumentPurgeRecoveryReview:async()=>({reviewId:'done',provider:'ollama',conversationId:'old-cid',items:r.items,recovery:r,expiresInSeconds:100})});await load(g);button(g,'Rever recuperação de cópias').fire('click');await tick();assert.equal(button(g,'Recuperar cópias completas'),undefined);assert.equal(button(g,'Continuar eliminação das cópias'),undefined);button(g,'Limpar apenas o registo de recuperação').fire('click');await tick();assert.deepEqual(g.calls.find(c=>c[0]==='commit'),['commit','ollama','old-cid','done','clearCopyRecovery']);
});
