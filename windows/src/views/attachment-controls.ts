import {Bridge,type DocumentInventory,type DocumentReview} from '../core/bridge';
import type {DisplayFile} from '../core/document-collection';
import {h,clear} from './dom';
export interface AttachmentOwner{provider:string;conversationId:string}
const key=(o:AttachmentOwner|null)=>o?JSON.stringify([o.provider,o.conversationId]):'';
const errorText=(e:unknown)=>String(e).replace(/^Error:\s*/,'');
type Transport=Pick<typeof Bridge,'petDocumentRemove'|'petDocumentInventory'|'petDocumentReview'|'petDocumentRenew'|'petDocumentReviewCancel'>;
/** Functional collection/renewal controls shared by island and floating chats.
 * Every async action owns its original pet/CID and may not retarget a new view. */
export function attachmentControls(options:{refresh:(owner:AttachmentOwner)=>Promise<void>;activate?:(owner:AttachmentOwner)=>Promise<void>;actions?:(file:DisplayFile,owner:AttachmentOwner|null)=>void;changed:()=>void;transport?:Transport}){
 const api=options.transport??Bridge;
 const chips=h('div',{class:'chip-row'}),status=h('p',{class:'sub',role:'status'}),stored=h('div'),reviewed=h('div');
 const open=h('button',{type:'button',class:'picker-link',text:'Rever anexos guardados',onclick:()=>void load()});
 const el=h('section',{class:'chat-attachments',style:'max-height:160px;overflow:auto;flex-shrink:0;min-width:0','aria-label':'Anexos desta conversa'},chips,open,status,stored,reviewed);
 let owner:AttachmentOwner|null=null,files:DisplayFile[]=[],signature='',generation=0,locked=false,busy=false;
 let controls:Array<[HTMLButtonElement|HTMLInputElement,boolean]>=[];
 const track=<T extends HTMLButtonElement|HTMLInputElement>(node:T,blocked=false)=>{controls.push([node,blocked]);node.disabled=busy||locked||blocked;return node;};
 const updateDisabled=()=>{for(const [node,blocked]of controls)node.disabled=busy||locked||blocked;};
 let inventory:DocumentInventory|null=null,review:DocumentReview|null=null,selected=new Set<string>(),message='',limit=50;
 function valid(o:AttachmentOwner,ticket:number){return ticket===generation&&key(o)===key(owner);}
 function cancel(o:AttachmentOwner|null,value:DocumentReview|null){if(o&&value)void api.petDocumentReviewCancel(o.provider,o.conversationId,value.reviewId).catch(()=>{});}
 function draw(){
  clear(chips);clear(stored);clear(reviewed);controls=[];const drawnAt=generation,drawnOwner=owner?{...owner}:null;const guard=(work:()=>void)=>{if(drawnAt===generation&&key(drawnOwner)===key(owner))work();};
  for(const file of files){
   const row=h('div',{class:'chip settled',style:'max-width:100%;flex-wrap:wrap'},h('span',{text:file.name,style:'overflow-wrap:anywhere;min-width:0'}));if(file.id)row.dataset.attachmentId=file.id;
   if(file.id){const remove=h('button',{type:'button',class:'picker-link',text:'Retirar','aria-label':`Retirar ${file.name}`,onclick:()=>guard(()=>void removeFile(file.id!))});track(remove);row.append(remove);}
   if(options.actions){const action=h('button',{type:'button',class:'picker-link',text:'Ações do documento',onclick:()=>guard(()=>{if(!busy&&!locked)options.actions?.(file,drawnOwner);})});track(action);row.append(action);}
   chips.append(row);
  }
  track(open,!owner);open.hidden=!owner;status.textContent=message;status.hidden=!message;el.hidden=!owner&&!files.length;
  if(inventory){
   stored.append(h('p',{class:'sub',text:'As cópias guardadas precisam de autorização de leitura após reiniciar. Escolhe os itens a rever.'}));
   for(const record of inventory.records.slice(0,limit)){
    const input=h('input',{type:'checkbox','aria-label':`Selecionar ${record.name}`});input.checked=selected.has(record.id);track(input,record.authorized||inventory.records.some(r=>r.purging));
    input.addEventListener('change',()=>guard(()=>{if(busy||locked||record.authorized||inventory?.records.some(r=>r.purging))return;if(input.checked)selected.add(record.id);else selected.delete(record.id);}));
    stored.append(h('label',{class:'chip-row'},input,h('span',{text:record.name}),h('span',{class:'sub',text:record.purging?'Retenção por resolver':record.authorized?'Leitura autorizada':record.selected?'A aguardar autorização':'Guardado no histórico'})));
   }
   if(limit<inventory.records.length)stored.append(h('button',{type:'button',class:'picker-link',text:'Mostrar mais anexos',onclick:()=>guard(()=>{limit+=50;draw();options.changed();})}));
   const prepare=h('button',{type:'button',class:'picker-link',text:'Rever seleção',onclick:()=>guard(()=>void prepareReview())});track(prepare,inventory.records.some(r=>r.purging));stored.append(prepare);
  }
  if(review){
   reviewed.append(h('p',{text:'Permitir que este pet leia estas cópias nesta conversa?'}));
   for(const item of review.items)reviewed.append(h('p',{class:'sub',text:`${item.name} · ${item.kind==='folder'?'pasta':'ficheiro'} · ${item.size} bytes`}));
   const allow=h('button',{type:'button',class:'picker-link',text:'Autorizar leitura destes anexos',onclick:()=>guard(()=>void renew())}),dismiss=h('button',{type:'button',class:'picker-link',text:'Cancelar revisão',onclick:()=>guard(()=>{if(busy||locked)return;cancel(owner,review);review=null;generation++;busy=false;message='';draw();options.changed();})});
   track(allow);track(dismiss);reviewed.append(allow,dismiss);
  }
 }
 async function removeFile(id:string){
  if(!owner||busy||locked)return;const o={...owner},ticket=++generation;busy=true;message='A retirar o anexo…';cancel(o,review);review=null;inventory=null;draw();options.changed();
  try{await api.petDocumentRemove(o.provider,o.conversationId,id);if(valid(o,ticket))message='Anexo retirado. A cópia continua guardada no histórico.';}
  catch(e){if(valid(o,ticket))message=errorText(e);}
  finally{if(key(o)===key(owner))await options.refresh(o).catch(e=>{if(valid(o,ticket))message=errorText(e);});if(valid(o,ticket)){busy=false;draw();options.changed();}}
 }
 async function load(){
  if(!owner||busy||locked)return;const o={...owner},ticket=++generation;busy=true;message='A ler o registo de anexos…';cancel(o,review);review=null;inventory=null;draw();options.changed();
  try{const result=await api.petDocumentInventory(o.provider,o.conversationId);if(!valid(o,ticket))return;
   if(!result||typeof result.revision!=='string'||!Array.isArray(result.records)||result.records.length>1024||result.records.some(r=>typeof r.id!=='string'||typeof r.name!=='string'))throw Error('O registo de anexos recebido é inválido.');
   inventory=result;selected=new Set(result.records.filter(r=>r.selected&&!r.authorized&&!r.purging).map(r=>r.id));limit=50;message=result.records.some(r=>r.purging)?'Há uma operação de retenção por resolver nesta conversa.':result.records.length?'':'Esta conversa ainda não tem anexos guardados.';
  }catch(e){if(valid(o,ticket))message=errorText(e);}finally{if(valid(o,ticket)){busy=false;draw();options.changed();}}
 }
 async function prepareReview(){
  if(!owner||!inventory||busy||locked||inventory.records.some(r=>r.purging))return;
  const ids=[...selected];if(!ids.length||ids.length>32){message='Escolhe entre um e 32 anexos.';draw();options.changed();return;}
  const o={...owner},revision=inventory.revision,ticket=++generation;busy=true;message='A verificar as cópias escolhidas…';cancel(o,review);review=null;draw();options.changed();
  try{
   await options.activate?.(o);if(!valid(o,ticket))return;
   const result=await api.petDocumentReview(o.provider,o.conversationId,ids,revision);
   if(!valid(o,ticket)){cancel(o,result);return;}
   if(!result||typeof result.reviewId!=='string'||!result.reviewId||!Array.isArray(result.items)||result.items.length!==ids.length||new Set(result.items.map(i=>i.id)).size!==ids.length||result.items.some(i=>!ids.includes(i.id))){cancel(o,result);throw Error('A revisão não corresponde aos anexos escolhidos. Volta a carregar o registo.');}
   review=result;message='A revisão expira em dois minutos. A autorização aplica-se a estas cópias.';
  }catch(e){if(valid(o,ticket))message=errorText(e);}finally{if(valid(o,ticket)){busy=false;draw();options.changed();}}
 }
 async function renew(){
  if(!owner||!review||busy||locked)return;const o={...owner},id=review.reviewId,ticket=++generation;busy=true;message='A autorizar a leitura dos anexos…';draw();options.changed();
  try{await api.petDocumentRenew(o.provider,o.conversationId,id);if(valid(o,ticket)){review=null;inventory=null;message='Leitura autorizada para os anexos escolhidos.';}}
  catch(e){if(valid(o,ticket)){cancel(o,review);review=null;inventory=null;message=errorText(e);}}
  finally{if(key(o)===key(owner))await options.refresh(o).catch(e=>{if(valid(o,ticket))message=errorText(e);});if(valid(o,ticket)){busy=false;draw();options.changed();}}
 }
 return {el,sync(next:AttachmentOwner|null,values:DisplayFile[],disabled=false){
  const nextSignature=JSON.stringify([key(next),values.map(f=>[f.id,f.name,f.path,f.sequence])]);
  const dirty=nextSignature!==signature;
  if(nextSignature!==signature){cancel(owner,review);owner=next?{...next}:null;files=values.map(f=>({...f}));signature=nextSignature;generation++;busy=false;inventory=null;review=null;selected.clear();message='';}
  locked=disabled;if(dirty)draw();else updateDisabled();
 },dispose(){cancel(owner,review);owner=null;generation++;}};
}
