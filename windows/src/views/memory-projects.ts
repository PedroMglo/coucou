import {buildProjectProgress} from './project-progress';
import {Bridge,type PetMemoryProject,type PetMemoryProjectsPage,type PetMemoryProjectMutation} from '../core/bridge';
import {buildMemoryControls} from './memory-controls';import {h} from './dom';
type Target={petId:string;conversationId:string|null};
type Transport=Pick<typeof Bridge,'petMemoryProjectList'|'petMemoryProjectMutate'|'petMemoryList'|'petMemoryMutate'>&Partial<Pick<typeof Bridge,'petRuntimeList'|'petRuntimeRead'|'petRuntimeHistory'|'petRuntimeResult'>>;
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
/** Explicit project choice; association is data scope, never model consent. */
export function buildMemoryProjects(options:{purpose?:'goal'|'manual'|'rule';changed?:()=>void;allowSelection?:boolean;selectionChanged?:()=>void;transport?:Transport;ids?:()=>string}={}){
 const transport=options.transport??Bridge,changed=options.changed??(()=>{});
 let target:Target|null=null,page:PetMemoryProjectsPage|null=null,selectedId='',epoch=0,busy=false,dirty=true,alive=true,uncertain:PetMemoryProjectMutation|null=null;
 const progress=buildProjectProgress({changed,transport:{...Bridge,...transport}});
 const status=h('p',{role:'status'}),facts=h('p'),metadata=h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere'});
 const select=h('select',{'aria-label':'Projeto de memória'})as HTMLSelectElement;
 const name=h('input',{'aria-label':'Nome do novo projeto',maxlength:160})as HTMLInputElement;
 const associate=h('input',{type:'checkbox','aria-label':'Associar a conversa atual ao novo projeto'})as HTMLInputElement;
 const rename=h('input',{'aria-label':'Novo nome do projeto selecionado',maxlength:160})as HTMLInputElement;
 const notes=buildMemoryControls({changed,purpose:options.purpose??'manual',title:'Recordações do projeto selecionado',transport,selectionChanged:options.selectionChanged,catalogChanged:()=>{dirty=true;options.selectionChanged?.();tell('As recordações mudaram o catálogo. Consulta os projetos antes de editar os seus dados.');draw();}});
 const load=h('button',{type:'button',text:'Consultar projetos',onclick:()=>void read()})as HTMLButtonElement;
 const create=h('button',{type:'button',text:'Guardar novo projeto',onclick:()=>{
  if(!name.value.trim()){tell('Indica um nome para o projeto.');return;}
  void mutate({operation:'create',requestId:(options.ids??(()=>crypto.randomUUID()))(),name:name.value,conversationIds:associate.checked&&target?.conversationId?[target.conversationId]:[]});
 }})as HTMLButtonElement;
 const saveName=h('button',{type:'button',text:'Guardar nome do projeto',onclick:()=>{const p=current();if(p)void mutate({operation:'rename',id:p.id,expectedProjectRevision:p.revision,name:rename.value});}})as HTMLButtonElement;
 const attach=h('button',{type:'button',text:'Associar esta conversa ao projeto',onclick:()=>{const p=current();if(p&&target?.conversationId)void mutate({operation:'setConversation',id:p.id,expectedProjectRevision:p.revision,conversationId:target.conversationId,attached:true});}})as HTMLButtonElement;
 const detach=h('button',{type:'button',text:'Retirar esta conversa do projeto',onclick:()=>{detach.hidden=true;confirmDetach.hidden=false;}})as HTMLButtonElement;
 const confirmDetach=h('button',{type:'button',text:'Confirmar retirada desta conversa',hidden:true,onclick:()=>{const p=current();if(p&&target?.conversationId)void mutate({operation:'setConversation',id:p.id,expectedProjectRevision:p.revision,conversationId:target.conversationId,attached:false});}})as HTMLButtonElement;
 const archive=h('button',{type:'button',text:'Arquivar projeto',onclick:()=>{archive.hidden=true;confirmArchive.hidden=false;}})as HTMLButtonElement;
 const confirmArchive=h('button',{type:'button',text:'Confirmar alteração de disponibilidade do projeto',hidden:true,onclick:()=>{const p=current();if(p)void mutate({operation:'setEnabled',id:p.id,expectedProjectRevision:p.revision,enabled:!p.enabled});}})as HTMLButtonElement;
 const el=h('details',{class:'memory-projects','aria-label':'Projetos e recordações deste pet'},h('summary',{text:'Projetos e recordações entre conversas'}),h('p',{text:'Um projeto tem identidade própria e associações explícitas. Uma pasta ou o nome de uma conversa não o cria. Guardar não inicia trabalho nem envia dados a modelos.'}),load,status,h('fieldset',{},h('legend',{text:'Novo projeto'}),h('label',{},'Nome',name),h('label',{},associate,'Associar explicitamente a conversa atual'),create),h('label',{},'Projeto consultado',select),facts,h('details',{},h('summary',{text:'Identidade e associações do projeto'}),metadata),h('label',{},'Nome revisto',rename),saveName,attach,detach,confirmDetach,archive,confirmArchive,h('p',{text:options.purpose==='rule'?'Seleciona apenas notas deste projeto para rever uma rotina limitada. O consentimento próprio autoriza repetir as mesmas revisões, dentro do saldo e prazo apresentados. Alterar projeto, associação ou notas exige nova revisão.':options.purpose==='goal'?'Selecionar notas deste projeto permite rever os dados apenas no objetivo limitado criado com esta conversa de origem, sem copiar o seu histórico. A associação não autoriza execução ou partilha. Alterações ao projeto ou notas invalidam usos posteriores.':'Selecionar notas de um projeto disponível e associado permite rever os dados antes deste pedido manual. A associação não autoriza envio. Alterar o projeto ou retirar a associação impede novos usos da revisão anterior. Objetivos autónomos exigem revisão própria.'}),notes.el,progress.el);
 function tell(text:string){status.textContent=text;changed();}
 function current():PetMemoryProject|undefined{return page?.items.find(p=>p.id===selectedId);}
 function resetConfirmations(){confirmDetach.hidden=confirmArchive.hidden=true;detach.hidden=archive.hidden=false;}
 function draw(){
  const p=current(),locked=busy||dirty||!target||!page,member=!!target?.conversationId&&!!p?.conversationIds.includes(target.conversationId);
  load.disabled=busy||!target;create.disabled=name.disabled=locked;associate.disabled=locked||!target?.conversationId;select.disabled=busy||!page;rename.disabled=saveName.disabled=locked||!p;
  attach.disabled=locked||!p||!target?.conversationId||member;detach.disabled=confirmDetach.disabled=locked||!p||!member;archive.disabled=confirmArchive.disabled=locked||!p;archive.textContent=p?.enabled?'Arquivar projeto':'Reativar projeto';
  facts.textContent=p?`${p.name} · revisão ${p.revision} · ${p.enabled?'disponível':'arquivado'} · ${p.conversationIds.length} conversas associadas${member?' · esta conversa está associada':''}. Arquivar conserva notas e associações.`:'Seleciona um projeto consultado para gerir as suas recordações.';
  metadata.textContent=p?JSON.stringify(p,null,2):'';progress.sync(!locked&&p?p:null);notes.el.hidden=!p;notes.sync(p&&target?{petId:target.petId,scope:{kind:'project',projectId:p.id}}:null,!!options.allowSelection&&!locked&&!!p?.enabled&&member);changed();
 }
 function renderList(){select.replaceChildren(...(page?.items??[]).map(p=>h('option',{value:p.id,text:p.name+(p.enabled?'':' (arquivado)')})));if(!page?.items.some(p=>p.id===selectedId))selectedId=page?.items[0]?.id??'';select.value=selectedId;rename.value=current()?.name??'';}
 select.addEventListener('change',()=>{options.selectionChanged?.();selectedId=select.value;rename.value=current()?.name??'';resetConfirmations();draw();});
 async function read(){
  if(!alive||busy||!target)return;const original=structuredClone(target),stamp=epoch;options.selectionChanged?.();busy=true;draw();
  try{const result=await transport.petMemoryProjectList(original.petId);if(!alive||stamp!==epoch)return;
   if(result.schemaVersion!==1||result.petId!==original.petId||result.executionAuthorized!==false||result.items.some(p=>p.petId!==original.petId))throw Error('O catálogo não corresponde ao pet original.');
   page=structuredClone(result);dirty=false;
   if(uncertain?.operation==='create'){const request=uncertain.requestId,p=page.items.find(p=>p.creationRequestId===request);if(p){selectedId=p.id;name.value='';associate.checked=false;tell('O projeto da escolha original foi encontrado. O estado atual foi consultado, sem nova criação.');}else tell('Catálogo consultado. A criação anterior não está presente; revê a escolha antes de guardar novamente.');}
   else tell('Projetos consultados. As associações não autorizam modelos ou execução.');uncertain=null;renderList();resetConfirmations();
  }catch(e){if(alive&&stamp===epoch){dirty=true;tell(String(e));}}finally{if(alive&&stamp===epoch){busy=false;draw();}}
 }
 async function mutate(input:PetMemoryProjectMutation){
  if(!alive||busy||dirty||!target||!page)return;const original=structuredClone(target),mutation=structuredClone(input),revision=page.revision,stamp=epoch;options.selectionChanged?.();busy=true;draw();
  try{const result=await transport.petMemoryProjectMutate(original.petId,revision,mutation);if(!alive||stamp!==epoch)return;
   const p=result.project,matches=mutation.operation==='create'?p.creationRequestId===mutation.requestId&&p.name===mutation.name&&same(p.conversationIds,mutation.conversationIds):p.id===mutation.id&&p.revision===mutation.expectedProjectRevision+1&&(mutation.operation==='rename'?p.name===mutation.name:mutation.operation==='setEnabled'?p.enabled===mutation.enabled:p.conversationIds.includes(mutation.conversationId)===mutation.attached);
   if(result.schemaVersion!==1||result.petId!==original.petId||result.executionAuthorized!==false||p.petId!==original.petId||!matches)throw Error('O recibo não corresponde ao projeto e escolha originais.');
   selectedId=p.id;dirty=true;uncertain=null;name.value='';associate.checked=false;resetConfirmations();tell('Projeto guardado. Consulta o catálogo para confirmar o estado atual e gerir as recordações.');
  }catch(e){if(alive&&stamp===epoch){uncertain=mutation;dirty=true;tell(`${String(e)} Consulta os projetos antes de repetir; a gravação pode ter ocorrido.`);}}finally{if(alive&&stamp===epoch){busy=false;draw();}}
 }
 draw();return {el,captureMetadata(){const p=current();return options.allowSelection&&!busy&&!dirty&&p?.enabled&&target?.conversationId&&p.conversationIds.includes(target.conversationId)?{project:{id:p.id,revision:p.revision},selected:notes.capture()}:null;},capture(){const p=current(),selected=notes.capture();return options.allowSelection&&!busy&&!dirty&&p?.enabled&&target?.conversationId&&p.conversationIds.includes(target.conversationId)&&selected.length?{project:{id:p.id,revision:p.revision},selected}:null;},clearSelection(){notes.clearSelection();},sync(next:Target|null){if(same(next,target))return;options.selectionChanged?.();epoch++;target=structuredClone(next);page=null;selectedId='';busy=false;dirty=true;uncertain=null;name.value=rename.value='';associate.checked=false;status.textContent='Consulta explicitamente os projetos deste pet.';resetConfirmations();renderList();draw();},dispose(){alive=false;epoch++;progress.dispose();notes.dispose();}};
}
