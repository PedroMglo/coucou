import test from 'node:test';import assert from 'node:assert/strict';
import {PetInferenceEditor} from '../src/desktop/pet-inference-editor.ts';import {PetSettings} from '../src/desktop/pet-settings.ts';
import {PetFactory} from '../src/core/pet-factory.ts';import {PetPreferences,profileSummary} from '../src/mochi/pet-preferences.ts';import {internals} from './tauri.mjs';import {installFakeDom} from './fakedom.mjs';
const clone=v=>structuredClone(v);
function fixture(){
 installFakeDom();let doc={config:PetFactory.configuration(),path:'/private-fixture/pets.json',revision:'native-before'},n=0,active=true,generalDirty=false;const calls=[],saved=[];
 const hooks={active:()=>active,canStart:()=>active,generalDirty:()=>generalDirty,saved:d=>saved.push(clone(d)),busy:()=>{}};
 const invoke=async(command,args)=>{
  calls.push({command,args:clone(args)});if(command==='pet_configuration')return clone(doc);if(command==='pet_roster')return Object.keys(doc.config.pets);
  if(command==='pet_inference_set'){
   const request=args.request;assert.equal(request.expectedRevision,doc.revision);const next=clone(doc);next.config.version=2;
   const pet=next.config.pets[request.petId];if(request.update.kind==='set')pet.inference=clone(request.update.choice);else delete pet.inference;
   pet.inferenceRevision=(++n).toString(16).padStart(32,'0');next.revision=`native-after-${n}`;doc=next;return clone(doc);
  }
  if(command==='pet_configuration_save'){doc={...doc,config:clone(args.config),revision:`general-${++n}`};return clone(doc);}
  throw Error(`Unexpected IO: ${command}`);
 };
 return {hooks,calls,saved,invoke,get doc(){return clone(doc);},set active(v){active=v;},set generalDirty(v){generalDirty=v;}};
}
function fill(editor,backend='ollama',model='synthetic-model'){
 editor.backend.value=backend;editor.endpoint.value='http://127.0.0.1:18471';editor.model.value=model;editor.credential.value='none';
}
test('opening and reviewing a choice has no IO; confirm sends the exact original pet, revision and complete selection',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{
  const editor=new PetInferenceEditor(f.hooks),before=f.doc;editor.show(before,'ai_lmstudio');assert.equal(editor.backend.value,'');assert.equal(editor.endpoint.value,'');assert.equal(f.calls.length,0);
  fill(editor,'ollama','synthetic<model>');editor.prepare(false);assert.equal(f.calls.length,0);assert.match(editor.details.textContent,/ai_lmstudio/);assert.match(editor.details.textContent,/synthetic<model>/);
  await editor.apply();assert.deepEqual(f.calls,[{command:'pet_inference_set',args:{request:{petId:'ai_lmstudio',expectedRevision:'native-before',update:{kind:'set',choice:{schemaVersion:1,backend:'ollama',endpoint:'http://127.0.0.1:18471',model:'synthetic<model>',credential:'none'}}}}}]);
  assert.equal(f.saved.length,1);assert.equal(editor.dirty,false);assert.deepEqual(before.config.pets.ai_ollama,f.doc.config.pets.ai_ollama);assert.equal(f.doc.config.pets.ai_lmstudio.provider,'');assert.deepEqual(before.config.pets.ai_lmstudio.permissions,f.doc.config.pets.ai_lmstudio.permissions);
 }finally{internals.invoke=original;}
});
test('argument edits and A -> B -> A pet switches retire review while drafts remain owned by each pet',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{
  const editor=new PetInferenceEditor(f.hooks);editor.show(f.doc,'ai_ollama');fill(editor);editor.prepare(false);editor.model.value='changed-without-event';await editor.apply();assert.equal(f.calls.length,0);assert.equal(editor.review,null);
  editor.prepare(false);editor.show(f.doc,'ai_lmstudio');assert.equal(editor.model.value,'');fill(editor,'lmstudio','second');editor.show(f.doc,'ai_ollama');assert.equal(editor.model.value,'changed-without-event');await editor.apply();assert.equal(f.calls.length,0);
  editor.show(f.doc,'ai_lmstudio');assert.equal(editor.model.value,'second');assert.equal(editor.backend.value,'lmstudio');assert.equal(editor.dirty,true);
  editor.prepare(false);f.active=false;await editor.apply();assert.equal(f.calls.length,0);
 }finally{internals.invoke=original;}
});
test('general drafts, missing selection and native rejection cannot become an implicit clear or a successful save',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{
  const editor=new PetInferenceEditor(f.hooks);editor.show(f.doc,'ai_ollama');editor.prepare(false);await editor.apply();assert.equal(f.calls.length,0);
  fill(editor);f.generalDirty=true;editor.prepare(false);assert.equal(editor.review,null);f.generalDirty=false;editor.prepare(false);
  internals.invoke=async(command,args)=>{f.calls.push({command,args:clone(args)});throw Error('native revision changed');};await editor.apply();assert.equal(f.saved.length,0);assert.equal(f.calls.length,1);assert.equal(editor.dirty,true);assert.equal(editor.review,null);assert.match(editor.status.textContent,/revision changed/);
  await editor.apply();assert.equal(f.calls.length,1,'retry requires a new review');
 }finally{internals.invoke=original;}
});
test('an ACK with the old inference revision is refused without projecting false success',async()=>{
 const original=internals.invoke,f=fixture();internals.invoke=f.invoke;
 try{
  const editor=new PetInferenceEditor(f.hooks);editor.show(f.doc,'ai_ollama');fill(editor);editor.prepare(false);await editor.apply();const previous=f.doc;f.saved.length=0;
  editor.show(previous,'ai_ollama');fill(editor,'ollama','new-model');editor.prepare(false);internals.invoke=async()=>{const stale=clone(previous);stale.config.pets.ai_ollama.inference.model='new-model';return stale;};await editor.apply();
  assert.equal(f.saved.length,0);assert.match(editor.status.textContent,/não foi confirmada/);assert.equal(editor.dirty,true);
 }finally{internals.invoke=original;}
});
test('the existing pet management view saves and clears only the reviewed binding and retains pending drafts on reload',async()=>{
 const original=internals.invoke,previous=PetPreferences.config,f=fixture();internals.invoke=f.invoke;
 try{
  const view=new PetSettings();await view.load('ai_lmstudio');const reads=f.calls.length;const before=f.doc;fill(view.inference);view.inference.prepare(false);assert.equal(f.calls.length,reads);await view.inference.apply();
  assert.equal(view.profiles.hidden,true);assert.equal(view.engineGroup.hidden,true);assert.equal(PetPreferences.config.pets.ai_lmstudio.inference.model,'synthetic-model');
  assert.equal(profileSummary('lmstudio','old-global-model'),'Ollama · synthetic-model');
  const firstRevision=f.doc.config.pets.ai_lmstudio.inferenceRevision;view.inference.prepare(true);assert.match(view.inference.details.textContent,/configuração anterior do pet/);await view.inference.apply();
  assert.equal(f.doc.config.version,2);assert.equal(f.doc.config.pets.ai_lmstudio.inference,undefined);assert.notEqual(f.doc.config.pets.ai_lmstudio.inferenceRevision,firstRevision);assert.equal(f.doc.config.pets.ai_lmstudio.provider,'');
  assert.deepEqual(f.doc.config.pets.ai_ollama,before.config.pets.ai_ollama);fill(view.inference,'lmstudio','pending');assert.equal(view.inference.dirty,true);const count=f.calls.length;view.reload.onclick();assert.equal(view.discard.hidden,false);assert.equal(f.calls.length,count);assert.equal(view.inference.model.value,'pending');
  assert.deepEqual(f.calls.filter(c=>!['pet_configuration','pet_roster','pet_inference_set'].includes(c.command)),[]);
 }finally{internals.invoke=original;PetPreferences.update(previous);}
});
