import {Bridge} from '../core/bridge';
import {petBlueprint} from '../core/pet-factory';
import type {InferenceChoice,InferenceUpdate,PetDocument} from '../mochi/pet-preferences';
const el=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const n=document.createElement(tag);if(text)n.textContent=text;return n;};
type Draft={backend:string;endpoint:string;model:string;credential:string};
type Review={petId:string;revision:string;update:InferenceUpdate;fields:string;intent:number};
type Hooks={active:()=>boolean;canStart:()=>boolean;generalDirty:()=>boolean;saved:(doc:PetDocument)=>void;busy:(value:boolean)=>void};
const fromChoice=(c?:InferenceChoice):Draft=>({backend:c?.backend??'',endpoint:c?.endpoint??'',model:c?.model??'',credential:c?.credential??''});
const key=(d:Draft)=>JSON.stringify([d.backend,d.endpoint,d.model,d.credential]);
const choiceKey=(c?:InferenceChoice)=>c?JSON.stringify([c.schemaVersion,c.backend,c.endpoint,c.model,c.credential]):'none';
/** Native field/revision management. Opening/reviewing never contacts a model. */
export class PetInferenceEditor{
 readonly el=el('fieldset');private backend=el('select');private endpoint=el('input');private model=el('input');private credential=el('select');
 private reviewButton=el('button','Rever alteração do motor');private clearButton=el('button','Rever remoção da escolha explícita');private confirm=el('button','Confirmar alteração do motor');
 private reviewPanel=el('div');private details=el('pre');private status=el('p');private drafts=new Map<string,Draft>();
 private doc:PetDocument|null=null;private id='';private intent=0;private review:Review|null=null;private saving=false;private parentDisabled=false;
 constructor(private hooks:Hooks){
  this.el.hidden=true;this.el.append(el('legend','Motor de inferência deste pet'));
  const field=(label:string,input:HTMLElement)=>{const row=el('label');row.append(el('span',label),input);this.el.append(row);input.setAttribute('aria-label',label);};
  this.endpoint.type='url';this.endpoint.maxLength=4096;this.endpoint.placeholder='http://127.0.0.1:11434';this.endpoint.autocomplete='off';this.model.maxLength=256;this.model.autocomplete='off';
  for(const[value,label]of [['','Escolher referência de credencial'],['none','Sem credencial'],['configuredLmstudioEndpoint','Credencial LM Studio deste endereço exato']]){const o=el('option',label);o.value=value;this.credential.append(o);}
  field('Motor',this.backend);field('Servidor explícito',this.endpoint);field('Modelo concreto',this.model);field('Credencial',this.credential);
  this.el.append(el('p','A escolha pertence apenas a este pet. Guardar não inicia inferência ou tarefas e não concede acessos. O suporte e a disponibilidade do modelo são verificados ao utilizá-lo.'));
  const actions=el('div');actions.className='pet-profile-actions';this.reviewButton.type=this.clearButton.type=this.confirm.type='button';
  this.reviewButton.onclick=()=>this.prepare(false);this.clearButton.onclick=()=>this.prepare(true);this.confirm.onclick=()=>void this.apply();actions.append(this.reviewButton,this.clearButton);this.el.append(actions);
  this.reviewPanel.className='pets-management-confirmation';this.reviewPanel.hidden=true;this.reviewPanel.setAttribute('role','group');this.reviewPanel.setAttribute('aria-label','Revisão do motor do pet');this.reviewPanel.append(this.details,this.confirm);this.status.setAttribute('role','status');this.el.append(this.reviewPanel,this.status);
  for(const kind of ['input','change'])this.el.addEventListener(kind,event=>{event.stopPropagation();this.remember();this.invalidate();this.status.textContent='Motor com alterações por guardar';});
 }
 private fields():Draft{return {backend:this.backend.value,endpoint:this.endpoint.value,model:this.model.value,credential:this.credential.value};}
 private remember(){if(!this.doc||!this.id||this.el.hidden)return;const d=this.fields();if(key(d)===key(fromChoice(this.doc.config.pets[this.id]?.inference)))this.drafts.delete(this.id);else this.drafts.set(this.id,d);}
 get dirty(){this.remember();return this.drafts.size>0;}
 reset(){this.drafts.clear();this.doc=null;this.id='';this.invalidate();}
 invalidate(){this.review=null;this.reviewPanel.hidden=true;this.confirm.disabled=true;this.intent++;}
 setParentDisabled(value:boolean){this.parentDisabled=value;this.sync();}
 private sync(){
  this.el.disabled=this.parentDisabled||this.saving;this.reviewButton.disabled=this.parentDisabled||this.saving;this.clearButton.disabled=this.parentDisabled||this.saving||!this.doc?.config.pets[this.id]?.inference;
  this.confirm.disabled=this.parentDisabled||this.saving||!this.review;
 }
 show(doc:PetDocument,id:string){
  this.remember();this.invalidate();this.doc=structuredClone(doc);this.id=id;const adapters=petBlueprint(id)?.runtime.inferenceAdapters??[];
  this.el.hidden=adapters.length===0;if(this.el.hidden)return;
  this.backend.replaceChildren();for(const[value,label]of [['','Escolher motor'],...adapters.map(a=>[a,a==='ollama'?'Ollama':'LM Studio'])]){const option=el('option',label);option.value=value;this.backend.append(option);}
  const draft=this.drafts.get(id)??fromChoice(doc.config.pets[id]?.inference);this.backend.value=draft.backend;this.endpoint.value=draft.endpoint;this.model.value=draft.model;this.credential.value=draft.credential;
  this.status.textContent=this.drafts.has(id)?'Motor com alterações por guardar':doc.config.pets[id]?.inference?'Escolha explícita guardada':'Configuração anterior preservada';this.sync();
 }
 private prepare(clear:boolean){
  this.invalidate();if(!this.doc||!this.hooks.canStart()||this.saving||this.parentDisabled)return;
  if(this.hooks.generalDirty()){this.status.textContent='Guarda primeiro as restantes preferências antes de rever o motor.';return;}
  this.remember();const d=this.fields();let update:InferenceUpdate;
  if(clear){if(!this.doc.config.pets[this.id]?.inference)return;update={kind:'clear'};}
  else{
   if(!(petBlueprint(this.id)?.runtime.inferenceAdapters??[]).some(a=>a===d.backend)||!d.endpoint.trim()||!d.model.trim()||!['none','configuredLmstudioEndpoint'].includes(d.credential)){
    this.status.textContent='Escolhe explicitamente motor, servidor, modelo e referência de credencial.';return;
   }
   update={kind:'set',choice:{schemaVersion:1,backend:d.backend as InferenceChoice['backend'],endpoint:d.endpoint.trim(),model:d.model.trim(),credential:d.credential as InferenceChoice['credential']}};
  }
  this.review={petId:this.id,revision:this.doc.revision,update,fields:key(d),intent:this.intent};
  const format=(c?:InferenceChoice)=>c?`${c.backend}\nServidor: ${c.endpoint}\nModelo: ${c.model}\nCredencial: ${c.credential==='none'?'sem credencial':'LM Studio, vinculada ao endereço exato'}`:'configuração anterior do pet';
  this.details.textContent=`Pet: ${this.doc.config.pets[this.id].name} (${this.id})\n\nAtual: ${format(this.doc.config.pets[this.id].inference)}\n\nDepois: ${format(update.kind==='set'?update.choice:undefined)}\n\nRevisões antigas deixam de iniciar novos envios. A alteração não desfaz envios anteriores. As permissões mantêm-se próprias do pet.`;
  this.reviewPanel.hidden=false;this.status.textContent='Revê o pet, destino, modelo e credencial antes de confirmar.';this.sync();
 }
 private async apply(){
  const review=this.review;
  if(!review||this.saving||this.parentDisabled||!this.hooks.canStart()||this.hooks.generalDirty()||!this.doc||review.intent!==this.intent||review.petId!==this.id||review.revision!==this.doc.revision||review.fields!==key(this.fields())){this.invalidate();return;}
  const previous=this.doc.config.pets[review.petId]?.inferenceRevision;this.saving=true;this.hooks.busy(true);this.sync();this.status.textContent='A guardar a escolha revista…';
  try{
   const doc=await Bridge.petInferenceSet(review.petId,review.revision,structuredClone(review.update));const pet=doc?.config.pets[review.petId];
   if(!doc||doc.config.version!==2||!pet?.inferenceRevision||!/^[0-9a-f]{32}$/.test(pet.inferenceRevision)||pet.inferenceRevision===previous||choiceKey(pet.inference)!==choiceKey(review.update.kind==='set'?review.update.choice:undefined))throw Error('A alteração não foi confirmada. Recarrega o perfil antes de repetir.');
   const current=this.hooks.active()&&this.id===review.petId&&this.intent===review.intent;this.drafts.delete(review.petId);this.invalidate();this.doc=structuredClone(doc);
   if(this.id===review.petId){const d=fromChoice(pet.inference);this.backend.value=d.backend;this.endpoint.value=d.endpoint;this.model.value=d.model;this.credential.value=d.credential;}
   this.hooks.saved(doc);if(current)this.status.textContent='Escolha do motor guardada. Nenhuma inferência ou tarefa foi iniciada.';
  }catch(error){this.invalidate();this.status.textContent=String(error).replace(/^Error:\s*/,'');}
  finally{this.saving=false;this.hooks.busy(false);this.sync();}
 }
}
