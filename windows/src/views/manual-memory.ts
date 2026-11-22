import {Bridge,type PetMemoryPrepare,type PetMemorySelected,type PetMemoryTicket,type PetMemoryDecision} from '../core/bridge';
import {buildMemoryControls} from './memory-controls';import {manualMemoryReview} from './memory-review';import {h} from './dom';
import {buildMemoryProjects} from './memory-projects';
export interface ManualMemoryTarget {petId:string;provider:string;conversationId:string|null}
type Transport=Pick<typeof Bridge,'petMemoryContextPrepare'|'petMemoryContextCancel'|'petMemoryList'|'petMemoryMutate'>&Partial<Pick<typeof Bridge,'petMemoryProjectList'|'petMemoryProjectMutate'>>;
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
/** One native review, rendered as data. A manual turn/session data decision is
 * never an execution grant and never authorizes a goal or another owner. */
export function buildManualMemory(options:{changed?:()=>void;transport?:Transport}={}){
 const changed=options.changed??(()=>{}),transport=options.transport??Bridge;
 let target:ManualMemoryTarget|null=null,fingerprint='',epoch=0,preparing=false,ticket:PetMemoryTicket|null=null,settle:((value:PetMemoryDecision|null)=>void)|null=null,timer:ReturnType<typeof setTimeout>|undefined;
 const status=h('p',{role:'status'}),review=h('div');
 function finish(value:PetMemoryDecision|null){const resolve=settle;settle=null;ticket=null;preparing=false;if(timer!==undefined)clearTimeout(timer);timer=undefined;review.replaceChildren();layout();resolve?.(value);changed();}
 function discard(token:string){void transport.petMemoryContextCancel(token).catch(()=>{/* Cache cleanup is bounded by native expiry; never dispatch or retry. */});}
 function invalidate(){epoch++;if(ticket)discard(ticket.token);finish(null);}
 const pet=buildMemoryControls({changed,purpose:'manual',title:'Recordações do pet',selectionChanged:invalidate,transport});
 const conversation=buildMemoryControls({changed,purpose:'manual',title:'Recordações desta conversa',selectionChanged:invalidate,transport});
 const projects=buildMemoryProjects({changed,allowSelection:true,selectionChanged:invalidate,transport:{...Bridge,...transport}});pet.el.append(projects.el);
 const el=h('section',{'aria-label':'Memória selecionada para o chat',style:'min-height:0;max-height:260px;overflow:auto;flex:0 1 auto'},pet.el,conversation.el,status,review);el.hidden=true;
 function layout(){pet.el.hidden=!!settle;conversation.el.hidden=!!settle||!target?.conversationId;}
 function current(ticketEpoch:number,valid:()=>boolean){try{return ticketEpoch===epoch&&valid();}catch{return false;}}
 const capture=()=>structuredClone([...pet.capture(),...conversation.capture(),...(projects.capture()?.selected??[])]);
 return {el,capture,get pending(){return preparing||!!settle;},invalidate,
  sync(next:ManualMemoryTarget|null,inputFingerprint:string){if(!same(next,target)||inputFingerprint!==fingerprint)invalidate();target=structuredClone(next);fingerprint=inputFingerprint;el.hidden=!target;pet.sync(target?{petId:target.petId,scope:{kind:'pet'}}:null,!!target);conversation.sync(target?.conversationId?{petId:target.petId,scope:{kind:'conversation',conversationId:target.conversationId}}:null,!!target?.conversationId);projects.sync(target?{petId:target.petId,conversationId:target.conversationId}:null);layout();},
  dispose(){invalidate();projects.dispose();pet.dispose();conversation.dispose();},
  clearSelection(){pet.clearSelection();conversation.clearSelection();projects.clearSelection();},
  async request(input:PetMemoryPrepare,selected:PetMemorySelected[],valid:()=>boolean):Promise<PetMemoryDecision|null|undefined>{
   if(!selected.length)return undefined;if(preparing||settle)return null;
   if(selected.length>16||new Set(selected.map(r=>r.id)).size!==selected.length){status.textContent='Seleciona no máximo 16 recordações distintas no conjunto dos âmbitos selecionados.';changed();return null;}
   const project=projects.capture(),projectChoice=project&&selected.some(r=>project.selected.some(p=>p.id===r.id))?project.project:undefined;
   const original=structuredClone({...input,...(projectChoice?{projectSelection:projectChoice}:{})}),selection=structuredClone(selected),owner=structuredClone(target),stamp=++epoch;preparing=true;status.textContent='A preparar a revisão nativa. Nenhum pedido foi enviado ao modelo.';changed();
   try{
    const prepared=await transport.petMemoryContextPrepare({...original,selected:selection});
    if(!current(stamp,valid)||!same(owner,target)){discard(prepared.token);return null;}
    const p=prepared.proposal;
    if(p.schemaVersion!==(projectChoice?2:1)||!!p.project!==!!projectChoice||p.project&&(p.project.id!==projectChoice?.id||p.project.revision!==projectChoice.revision||p.project.petId!==owner?.petId||!p.project.enabled||!p.project.conversationIds.includes(original.conversationId))||p.petId!==owner?.petId||p.conversationId!==original.conversationId||p.destination.provider!==original.provider||p.question!==original.query||p.dispatchCoverage?.schemaVersion!==1||!p.requestContextHash?.match(/^[a-f0-9]{64}$/)||p.records.length!==selection.length||!selection.every(r=>p.records.some(e=>e.id===r.id&&e.revision===r.revision&&e.owner.petId===p.petId&&(e.owner.scope.kind==='pet'||e.owner.scope.kind==='conversation'&&e.owner.scope.conversationId===p.conversationId||e.owner.scope.kind==='project'&&e.owner.scope.projectId===p.project?.id)))||p.requiredConsent!==(p.destination.kind==='nativeSession'?'nativeSessionContext':'thisRequest')||!Number.isFinite(p.expiresAt)||p.expiresAt<=Date.now()){
     discard(prepared.token);throw Error('A revisão nativa não corresponde ao pedido, aos dados ou à versão esperados. Nenhum envio foi autorizado.');
    }
    ticket=structuredClone(prepared);status.textContent='Revê os dados completos e o âmbito. A seleção não autoriza o envio.';
    return await new Promise<PetMemoryDecision|null>(resolve=>{
     settle=resolve;const consent=h('input',{type:'checkbox','aria-label':'Autorizar dados no âmbito deste pedido'}) as HTMLInputElement;
     const approve=h('button',{type:'button',text:'Autorizar dados e enviar',disabled:true,onclick:()=>{if(approve.disabled||!consent.checked)return;if(!current(stamp,valid)){status.textContent='O pedido mudou. Revê novamente antes de enviar.';invalidate();return;}const decision={token:prepared.token,consent:p.requiredConsent};pet.el.open=false;conversation.el.open=false;el.scrollTop=0;status.textContent='Revisão confirmada. O envio mantém as verificações nativas de dados e destino.';finish(decision);}}) as HTMLButtonElement;
     consent.addEventListener('change',()=>{approve.disabled=!consent.checked;});
     review.replaceChildren(manualMemoryReview(p,original.context),h('label',{},consent,p.requiredConsent==='nativeSessionContext'?'Autorizo estes dados como contexto desta sessão nativa, nos limites apresentados.':'Autorizo estes dados apenas neste pedido manual e destino revisto.'),approve,h('button',{type:'button',text:'Cancelar revisão de dados',onclick:()=>{status.textContent='Revisão cancelada. O pedido e os anexos foram conservados.';invalidate();}}));
     layout();timer=setTimeout(()=>{status.textContent='A revisão expirou. Prepara outra revisão antes de enviar.';invalidate();},Math.max(1,p.expiresAt-Date.now()));changed();
    });
   }catch(e){if(stamp===epoch){status.textContent=String(e).replace(/^Error:\s*/,'');changed();}return null;}
   finally{if(stamp===epoch){preparing=false;changed();}}
  },
 };
}
