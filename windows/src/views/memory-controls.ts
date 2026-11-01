import {Bridge,type PetMemoryOwner,type PetMemoryEntry,type PetMemoryInput,type PetMemorySelected,type PetMemoryPage,type PetMemoryMutation} from '../core/bridge';
import {h} from './dom';
import {memoryRecord} from './memory-review';

type Transport=Pick<typeof Bridge,'petMemoryList'|'petMemoryMutate'>;
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
/** Explicit native-user catalogue. Opening the containing view and changing
 * its owner perform no IO. Selection pins revisions, never mutable records. */
export function buildMemoryControls(options:{changed?:()=>void;selectionChanged?:()=>void;purpose?:'goal'|'manual';title?:string;transport?:Transport}={}){
 const transport=options.transport??Bridge,changed=options.changed??(()=>{});
 let owner:PetMemoryOwner|null=null,selectable=false,alive=true,epoch=0,busy=false,page:PetMemoryPage|null=null,entries:PetMemoryPage['items']=[],editing:PetMemoryEntry|null=null,refreshRequired=true,pageQuery='';
 const selected=new Map<string,PetMemorySelected>();
 const status=h('p',{role:'status'}),records=h('div'),choices=h('div'),query=h('input',{'aria-label':'Pesquisar recordações',maxlength:256}) as HTMLInputElement;
 const title=h('input',{'aria-label':'Título da recordação',maxlength:160}) as HTMLInputElement;
 const content=h('textarea',{'aria-label':'Conteúdo da recordação',rows:4}) as HTMLTextAreaElement;
 const sensitivity=h('select',{'aria-label':'Sensibilidade da recordação'},h('option',{value:'normal',text:'Normal'}),h('option',{value:'sensitive',text:'Sensível'})) as HTMLSelectElement;sensitivity.value='normal';
 const kind=h('select',{'aria-label':'Tipo de recordação'},h('option',{value:'note',text:'Nota'}),h('option',{value:'preference',text:'Preferência'})) as HTMLSelectElement;kind.value='note';
 const expiry=h('input',{'aria-label':'Validade da recordação em ISO 8601',placeholder:'Opcional, por exemplo 2026-12-01T10:00:00Z'}) as HTMLInputElement;
 const load=h('button',{type:'button',text:'Consultar recordações',onclick:()=>void read(false)}) as HTMLButtonElement;
 const more=h('button',{type:'button',text:'Mais recordações',onclick:()=>void read(true)}) as HTMLButtonElement;
 const save=h('button',{type:'button',text:'Guardar recordação',onclick:()=>void write()}) as HTMLButtonElement;
 const editor=h('fieldset',{},h('legend',{text:'Recordação explícita do utilizador'}),title,kind,content,sensitivity,expiry,save,h('button',{type:'button',text:'Limpar edição',onclick:()=>resetEditor()}));
 const el=h('details',{'aria-label':'Memória do pet'},h('summary',{text:options.title??'Recordações e seleção de contexto'}),h('p',{text:'Consultar ou guardar recordações não as envia a um modelo. A utilização exige uma revisão de dados própria. A seleção conserva os IDs e revisões escolhidos.'}),query,load,status,choices,records,more,editor);
 function tell(text:string){status.textContent=text;changed();}
 function resetEditor(){editing=null;title.value='';content.value='';expiry.value='';kind.value='note';sensitivity.value='normal';save.textContent='Guardar recordação';}
 function draw(){
  load.disabled=busy||!owner;more.hidden=!page?.hasMore;more.disabled=busy||!owner;save.disabled=busy||!owner||refreshRequired;
  editor.disabled=busy||!owner||refreshRequired;query.disabled=busy||!owner;
  choices.replaceChildren(h('p',{text:selectable?`${selected.size}/16 recordações selecionadas. Revê as revisões antes de ${options.purpose==='manual'?'enviar o pedido':'guardar o objetivo'}.`:`Este destino não suporta seleção de memória ${options.purpose==='manual'?'nesta conversa':'por um objetivo'}. A gestão das recordações continua disponível.`}));
  for(const reference of selected.values())choices.append(h('p',{},`${reference.id} · revisão ${reference.revision}`,h('button',{type:'button',text:'Retirar da seleção',onclick:()=>{selected.delete(reference.id);options.selectionChanged?.();draw();changed();}})));
  records.replaceChildren();
  for(const view of entries){const entry=view.entry,check=h('input',{type:'checkbox','aria-label':`Selecionar recordação ${entry.title} revisão ${entry.revision}`}) as HTMLInputElement;
   check.checked=selected.get(entry.id)?.revision===entry.revision;check.disabled=busy||!selectable||!view.available;
   check.addEventListener('change',()=>{if(check.disabled)return;if(check.checked){if(!selected.has(entry.id)&&selected.size>=16){check.checked=false;tell('O limite é 16 recordações.');return;}selected.set(entry.id,{id:entry.id,revision:entry.revision});}else selected.delete(entry.id);options.selectionChanged?.();draw();changed();});
   const remove=h('button',{type:'button',text:'Eliminar recordação',disabled:busy||refreshRequired,onclick:()=>{remove.hidden=true;confirm.hidden=false;}}) as HTMLButtonElement;
   const confirm=h('button',{type:'button',text:'Confirmar eliminação desta revisão',hidden:true,disabled:busy||refreshRequired,onclick:()=>void mutate({operation:'delete',id:entry.id,expectedItemRevision:entry.revision})}) as HTMLButtonElement;
   records.append(h('div',{},memoryRecord(entry),...(!view.available?[h('p',{text:'Recordação expirada: não pode ser selecionada para novo uso.'})]:[]),h('label',{},check,'Selecionar esta revisão'),h('button',{type:'button',text:'Editar recordação',disabled:busy||refreshRequired,onclick:()=>{editing=structuredClone(entry);title.value=entry.title;content.value=entry.content;kind.value=entry.kind;sensitivity.value=entry.sensitivity;expiry.value=entry.expiresAt===null?'':new Date(entry.expiresAt).toISOString();save.textContent='Guardar nova revisão';}}),remove,confirm));
  }changed();
 }
 async function read(append:boolean){
  if(!alive||busy||!owner||append&&(!page?.hasMore||page.nextOffset===null))return;
  if(append&&query.value!==pageQuery){tell('A pesquisa mudou. Consulta novamente antes de carregar outra página.');return;}
  const target=structuredClone(owner),ticket=epoch,search=query.value,previous=page;busy=true;draw();
  try{const reply=await transport.petMemoryList({owner:target,query:search,limit:50,includeExpired:true,...(append?{offset:previous!.nextOffset!,expectedRevision:previous!.revision}:{})});
   if(!alive||ticket!==epoch)return;if(!same(reply.owner,target)||append&&reply.revision!==previous!.revision)throw Error('A resposta não corresponde ao âmbito e revisão pedidos.');
   page=reply;pageQuery=search;entries=append?[...entries,...reply.items]:reply.items;refreshRequired=false;tell('Catálogo consultado. As revisões já selecionadas não foram substituídas.');
  }catch(e){if(alive&&ticket===epoch){refreshRequired=true;tell(String(e));}}finally{if(alive&&ticket===epoch){busy=false;draw();}}
 }
 async function mutate(mutation:PetMemoryMutation){
  if(!alive||busy||!owner||!page||refreshRequired)return;
  const target=structuredClone(owner),revision=page.revision,original=structuredClone(mutation),ticket=epoch;busy=true;draw();
  try{const receipt=await transport.petMemoryMutate(target,revision,original);if(!alive||ticket!==epoch)return;if(!same(receipt.owner,target))throw Error('O recibo não corresponde ao âmbito original.');
   // Never replace an already selected revision with the result of an edit.
   resetEditor();refreshRequired=true;tell('Gravação confirmada. Consulta o catálogo para rever a nova revisão.');
  }catch(e){if(alive&&ticket===epoch){refreshRequired=true;tell(`${String(e)} Consulta o catálogo antes de repetir; a gravação pode ter ocorrido.`);}}
  finally{if(alive&&ticket===epoch){busy=false;draw();}}
 }
 async function write(){
  const expiresAt=expiry.value.trim()?Date.parse(expiry.value):null;
  if(!title.value.trim()||!content.value.trim()||expiresAt!==null&&(!Number.isFinite(expiresAt)||expiresAt<=Date.now())){tell('Indica título, conteúdo e uma validade futura válida, ou deixa a validade vazia.');return;}
  const input:PetMemoryInput={title:title.value,content:content.value,kind:kind.value==='preference'?'preference':'note',sensitivity:sensitivity.value==='sensitive'?'sensitive':'normal',expiresAt};
  await mutate(editing?{operation:'update',id:editing.id,expectedItemRevision:editing.revision,input}:{operation:'create',input});
 }
 draw();return {el,sync(next:PetMemoryOwner|null,allowSelection:boolean){if(!same(next,owner)){epoch++;owner=structuredClone(next);busy=false;page=null;entries=[];selected.clear();refreshRequired=true;resetEditor();status.textContent='Consulta explicitamente o catálogo deste pet.';}selectable=allowSelection;if(!selectable)selected.clear();draw();},capture(){return structuredClone([...selected.values()]);},clearSelection(){selected.clear();options.selectionChanged?.();draw();},dispose(){alive=false;epoch++;}};
}
