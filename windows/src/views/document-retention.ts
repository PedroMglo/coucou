import {Bridge,type DocumentRetentionInventory,type DocumentRetentionOwner,type DocumentPurgeReview,type DocumentCopyRecovery,type DocumentPurgeConsent} from '../core/bridge';
import {h,clear} from './dom';
type Transport=Pick<typeof Bridge,'petDocumentRetentionInventory'|'petDocumentPurgeReview'|'petDocumentPurgeRecoveryReview'|'petDocumentPurgeCommit'|'petDocumentPurgeCancel'|'petDocumentRemove'>;
const errorText=(e:unknown)=>String(e).replace(/^Error:\s*/,'');
const phases=new Set(['Prepared','Marked','Moved','Committed','Restoring','Restored','Cleaned']);
/** Explicit copy management for the library, including CIDs with no public history. */
export function documentRetention(options:{provider:string;current:()=>boolean;changed:()=>void;title?:(cid:string)=>string|undefined;transport?:Transport}){
 const api=options.transport??Bridge,provider=options.provider;
 const open=h('button',{type:'button',class:'picker-link',text:'Gerir cópias guardadas',onclick:()=>void load()});
 const status=h('p',{class:'sub',role:'status'}),list=h('div'),preview=h('div'),el=h('section',{'aria-label':'Cópias guardadas deste pet'},open,status,list,preview);
 let active=true,generation=0,busy=false,message='',inventory:DocumentRetentionInventory|null=null,review:DocumentPurgeReview|null=null,limit=50;
 const chosen=new Map<string,Set<string>>();const valid=(ticket:number)=>active&&options.current()&&ticket===generation;
 const cancel=(r:DocumentPurgeReview|null)=>{if(r)void api.petDocumentPurgeCancel(provider,r.conversationId,r.reviewId).catch(()=>{});};
 const name=(cid:string)=>options.title?.(cid)??'Conversa sem histórico local';
 function draw(){
  clear(list);clear(preview);status.textContent=message;status.hidden=!message;open.disabled=busy||!active;
  const drawn=generation,guard=(work:()=>void)=>{if(valid(drawn)&&!busy)work();};
  if(inventory){let remaining=limit,total=0;
   list.append(h('p',{class:'sub',text:'Cópias registadas pelo Coucou. A eliminação exige uma revisão e uma confirmação separadas.'}));
   for(const owner of inventory.owners.filter(o=>o.provider===provider&&o.items.length)){
    total+=owner.items.length;if(remaining<=0)continue;const section=h('div',{},h('strong',{text:name(owner.conversationId)}));const selected=chosen.get(owner.conversationId)??new Set<string>();chosen.set(owner.conversationId,selected);
    for(const item of owner.items.slice(0,remaining)){
     const blocked=owner.busy||item.selected||item.purging||item.historyReferences>0;
     const input=h('input',{type:'checkbox','aria-label':`Eliminar cópia ${item.name}`});input.checked=selected.has(item.id);input.disabled=busy||blocked;
     input.addEventListener('change',()=>guard(()=>{if(blocked)return;if(input.checked)selected.add(item.id);else selected.delete(item.id);}));
     const state=item.purging?'Retenção por resolver':item.historyReferences?`Referenciada em ${item.historyReferences} conversa(s)` :item.selected?'Selecionada':owner.busy?'Conversa ocupada':'Sem referências no histórico local';
     const row=h('label',{class:'chip-row'},input,h('span',{text:item.name}),h('span',{class:'sub',text:state}));
     if(item.selected&&!item.purging){const remove=h('button',{type:'button',class:'picker-link',text:'Retirar da seleção',onclick:()=>guard(()=>void deselect(owner,item.id))});remove.disabled=busy||owner.busy;row.append(remove);}section.append(row);
    }remaining-=owner.items.length;
    const prepare=h('button',{type:'button',class:'picker-link',text:'Rever eliminação das cópias',onclick:()=>guard(()=>void prepareCopies(owner))});prepare.disabled=busy||owner.busy;section.append(prepare);list.append(section);
   }
   if(total>limit)list.append(h('button',{type:'button',class:'picker-link',text:'Mostrar mais cópias',onclick:()=>guard(()=>{limit+=50;draw();options.changed();})}));
   for(const recovery of inventory.recoveries.recoveries.filter(r=>r.provider===provider)){
    const button=h('button',{type:'button',class:'picker-link',text:'Rever recuperação de cópias',onclick:()=>guard(()=>void prepareRecovery(recovery))});button.disabled=busy;
    list.append(h('div',{},h('strong',{text:`Recuperação · ${name(recovery.conversationId)}`}),h('p',{class:'sub',text:recovery.items.map(i=>i.name).join(', ')}),button));
   }
   if(inventory.recoveries.unavailable.length)list.append(h('p',{class:'sub',text:`Há ${inventory.recoveries.unavailable.length} registo(s) de recuperação indisponíveis. Foram conservados.`}));
  }
  if(review){preview.append(h('strong',{text:name(review.conversationId)}));for(const item of review.items)preview.append(h('p',{class:'sub',text:`${item.name} · ${item.kind==='folder'?'pasta':'ficheiro'} · ${item.size} bytes`}));
   preview.append(h('p',{class:'sub',text:'A revisão expira em dois minutos. A operação aplica-se apenas às cópias apresentadas.'}));
   const action=(text:string,consent:DocumentPurgeConsent)=>{const button=h('button',{type:'button',class:'picker-link',text,onclick:()=>guard(()=>void confirm(consent))});button.disabled=busy;preview.append(button);};
   if(!review.recovery)action('Eliminar estas cópias','deleteOwnedCopies');else{
    const phase=review.recovery.phase;if(['Prepared','Marked','Moved','Committed'].includes(phase))action('Continuar eliminação das cópias','finishCopyPurge');
    if(['Prepared','Marked','Moved','Committed','Restoring'].includes(phase))action('Recuperar cópias completas','restoreOwnedCopies');
    if(['Prepared','Restored','Cleaned'].includes(phase))action('Limpar apenas o registo de recuperação','clearCopyRecovery');
    if(phase==='Cleaned')preview.append(h('p',{class:'sub',text:'As cópias desta operação já foram eliminadas.'}));
   }
   const close=h('button',{type:'button',class:'picker-link',text:'Cancelar revisão das cópias',onclick:()=>guard(()=>{cancel(review);review=null;generation++;draw();options.changed();})});close.disabled=busy;preview.append(close);
  }
 }
 async function load(){
  if(!active||!options.current()||busy)return;const ticket=++generation;busy=true;cancel(review);review=null;message='A ler os registos de cópias…';draw();options.changed();
  try{const result=await api.petDocumentRetentionInventory();if(!valid(ticket))return;if(!Array.isArray(result?.owners)||!Array.isArray(result.recoveries?.recoveries)||!Array.isArray(result.recoveries?.unavailable))throw Error('O inventário de cópias é inválido.');inventory=result;chosen.clear();message='';}
  catch(e){if(valid(ticket))message=errorText(e);}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 function check(result:DocumentPurgeReview,cid:string,ids:string[],recoveryId?:string){
  if(!result||result.provider!==provider||result.conversationId!==cid||typeof result.reviewId!=='string'||!result.reviewId||!Array.isArray(result.items)||result.items.length!==ids.length||new Set(result.items.map(i=>i.id)).size!==ids.length||result.items.some(i=>!ids.includes(i.id))||result.expiresInSeconds<=0||result.expiresInSeconds>120||
   (recoveryId?result.recovery?.id!==recoveryId||!phases.has(result.recovery.phase):result.recovery!==null)){cancel(result);throw Error('A revisão recebida não corresponde às cópias escolhidas.');}
 }
 async function prepareCopies(owner:DocumentRetentionOwner){
  const ids=[...(chosen.get(owner.conversationId)??[])];if(!ids.length||ids.length>32){message='Escolhe entre uma e 32 cópias sem referências no histórico local.';draw();options.changed();return;}
  const ticket=++generation;busy=true;cancel(review);review=null;message='A verificar as cópias escolhidas…';draw();options.changed();
  try{const result=await api.petDocumentPurgeReview(provider,owner.conversationId,ids,owner.revision);if(!valid(ticket)){cancel(result);return;}check(result,owner.conversationId,ids);review=result;message='Confirma a eliminação das cópias apresentadas.';}
  catch(e){if(valid(ticket))message=errorText(e);}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 async function prepareRecovery(recovery:DocumentCopyRecovery){
  const ticket=++generation;busy=true;cancel(review);review=null;message='A rever a recuperação das cópias…';draw();options.changed();
  try{const result=await api.petDocumentPurgeRecoveryReview(recovery.id);if(!valid(ticket)){cancel(result);return;}check(result,recovery.conversationId,recovery.items.map(i=>i.id),recovery.id);review=result;message='Escolhe como resolver esta operação. A recuperação exige cópias completas.';}
  catch(e){if(valid(ticket))message=errorText(e);}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 async function deselect(owner:DocumentRetentionOwner,id:string){
  const ticket=++generation;busy=true;cancel(review);review=null;draw();options.changed();
  try{await api.petDocumentRemove(provider,owner.conversationId,id);if(!valid(ticket))return;inventory=await api.petDocumentRetentionInventory();if(!valid(ticket))return;chosen.clear();message='Seleção retirada. A cópia continua guardada.';}
  catch(e){if(valid(ticket))message=errorText(e);}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 async function confirm(consent:DocumentPurgeConsent){
  if(!review)return;const original=review,ticket=++generation;busy=true;message='A aplicar a decisão sobre as cópias…';draw();options.changed();
  try{const result=await api.petDocumentPurgeCommit(provider,original.conversationId,original.reviewId,consent);if(!valid(ticket))return;review=null;
   if(result.provider!==provider||result.conversationId!==original.conversationId||result.operation!==consent||typeof result.complete!=='boolean')throw Error('O resultado da operação é inválido. Volta a carregar os registos.');
   message=result.complete?(result.recoveryId?'Operação concluída. Falta limpar o registo da recuperação.':'Operação concluída.'):(result.error??'Operação incompleta. Revê a recuperação das cópias.');
   inventory=await api.petDocumentRetentionInventory();if(!valid(ticket))return;chosen.clear();
  }catch(e){if(valid(ticket)){cancel(review);review=null;message=errorText(e);}}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 draw();return{el,dispose(){cancel(review);review=null;active=false;generation++;}};
}
