import type {PetCapabilityDescriptor,RuntimeGoal,RuntimeNativePlanEdit,RuntimeNativePlanInventory,RuntimeStepInput} from '../core/bridge';
import {nativeAdmission,recursiveAdmission,resultPlanEligible,RuntimeGoalActions} from '../core/runtime-goals';
import {h} from './dom';
import {resultDisclosureReview} from './result-disclosure-review';
interface Options{actions:RuntimeGoalActions;run:(operation:()=>Promise<unknown>)=>Promise<unknown>;ids?:()=>string;changed?:()=>void}
type Editor={operationId:'pet.message'|'pet.reply';cause:RuntimeNativePlanEdit['cause'];original:RuntimeStepInput;description:HTMLInputElement;postcondition:HTMLTextAreaElement;recipient:HTMLSelectElement;kind:HTMLSelectElement;text:HTMLTextAreaElement;deadline:HTMLInputElement;el:HTMLElement};
const iso=(at:number)=>{try{return new Date(at).toISOString();}catch{return String(at);}};
const scope=(g:RuntimeGoal)=>`${g.spec.owner.petId}/${g.id}/${g.revision}`;
const states={staged:'Intenção assinada por publicar',sealed:'Plano assinado por publicar',published:'Plano publicado',withdrawalPending:'Retirada assinada por confirmar',abandoned:'Intenção abandonada',blocked:'Requer recuperação'};
/** Functional native data review. Opening the row does not list, sign, prepare
 * providers or renew any deadline. Backend signature/Frame checks decide. */
export function buildNativePlanRevision(options:Options){
 const actions=options.actions,ids=options.ids??(()=>crypto.randomUUID());let goal:RuntimeGoal|null=null,disabled=false,control=false,descriptors:PetCapabilityDescriptor[]=[],editorScope='',dirty=false,editors:Editor[]=[],request='';
 const notice=h('p',{role:'status'}),facts=h('div'),fields=h('fieldset',{},h('legend',{text:'Passos pendentes do plano'})),preview=h('div'),pending=h('div');
 const load=h('button',{type:'button',text:'Consultar revisões do plano',onclick:()=>{if(!goal||load.disabled)return;const original=structuredClone(goal);void options.run(()=>actions.readNativePlans(original));}})as HTMLButtonElement;
 const reset=h('button',{type:'button',text:'Recarregar passos pendentes',onclick:()=>{if(!goal||reset.disabled)return;dirty=false;editorScope='';request=ids();render();}})as HTMLButtonElement;
 const prepare=h('button',{type:'button',text:'Rever alterações do plano',onclick:()=>{
  if(!goal||prepare.disabled)return;const original=structuredClone(goal);
  try{const inputs=collect();if(!inputs.length)throw Error('Altera pelo menos um passo pendente.');const captured=request||(request=ids());void options.run(()=>actions.prepareNativePlan(original,inputs,captured));}catch(e){notice.textContent=String(e).replace(/^Error:\s*/,'');}
 }})as HTMLButtonElement;
 const resultStep=h('select',{'aria-label':'Passo para proposta de resultado'})as HTMLSelectElement;
 const byteStart=h('input',{type:'number',value:0,min:0,max:4194303,'aria-label':'Início do excerto em bytes UTF-8'})as HTMLInputElement,byteEnd=h('input',{type:'number',value:2048,min:1,max:4194304,'aria-label':'Fim do excerto em bytes UTF-8'})as HTMLInputElement;byteStart.value='0';byteEnd.value='2048';let resultScope='';
 const resultPrepare=h('button',{type:'button',text:'Rever proposta com resultado próprio',onclick:()=>{
  if(!goal||resultPrepare.disabled)return;const original=structuredClone(goal);try{const cause=JSON.parse(resultStep.value)as RuntimeNativePlanEdit['cause'];const captured=request||(request=ids());void options.run(()=>actions.prepareResultPlan(original,cause,Number(byteStart.value),Number(byteEnd.value),captured));}catch(e){notice.textContent=String(e).replace(/^Error:\s*/,'');}
 }})as HTMLButtonElement;
 const resultFields=h('fieldset',{},h('legend',{text:'Selecionar resultado próprio para uma proposta'}),h('p',{text:'Apenas dados do plano: não envia o resultado nem concede divulgação. O excerto é verificado contra o recibo autenticado. Destino e origem ficam na delegação original. Valores em bytes UTF-8, até 2048 bytes; a revisão mostra a cobertura e o texto exato.'}),resultStep,h('label',{},'Início em bytes',byteStart),h('label',{},'Fim em bytes',byteEnd),resultPrepare)as HTMLFieldSetElement;
 const el=h('details',{'aria-label':'Revisão de passos pendentes'},h('summary',{text:'Planos e revisões nativas'}),h('p',{text:'Rever o plano altera apenas dados. Executar continua a exigir a revisão separada do orçamento e as autorizações das operações. Prazos não são renovados automaticamente.'}),load,notice,facts,reset,fields,prepare,resultFields,preview,pending);
 function inventory():RuntimeNativePlanInventory|undefined{const value=goal&&actions.nativePlanInventories.get(goal.id);return value&&scope(goal!)===`${value.petId}/${value.goalId}/${value.revision}`?value:undefined;}
 function buildEditors(g:RuntimeGoal,view:RuntimeNativePlanInventory){
  editors=[];fields.replaceChildren(h('legend',{text:'Passos pendentes do plano'}));
  for(const plan of g.plans)for(const step of plan.steps){
   if(view.coverage?.receipts.some(r=>r.planId===plan.id&&r.stepId===step.id))continue;
   const original:RuntimeStepInput={description:step.description,postcondition:step.postcondition,...(step.operation?{operation:structuredClone(step.operation)}:{})},args=step.operation?.arguments as Record<string,unknown>|undefined;
   if(!['pet.message','pet.reply'].includes(step.operation?.operationId??'')||step.operation?.version!==1||!args){notice.textContent='Este plano não pertence ao catálogo literal suportado por este editor.';continue;}
   const description=h('input',{type:'text',value:step.description,maxLength:4096,'aria-label':`Descrição ${step.id}`})as HTMLInputElement;
   const postcondition=h('textarea',{value:step.postcondition,rows:2,maxLength:4096,'aria-label':`Pós-condição ${step.id}`})as HTMLTextAreaElement;
   description.value=step.description;postcondition.value=step.postcondition;
   const recipient=h('select',{'aria-label':`Destinatário ${step.id}`})as HTMLSelectElement;
   const isReply=step.operation.operationId==='pet.reply';
   const choices=descriptors.filter(d=>d.petId!==g.spec.owner.petId);for(const d of choices)recipient.append(h('option',{value:d.petId,text:`${d.name}${d.lifecycle!=='active'?' · inativo':''}`}));
   if(!choices.some(d=>d.petId===args.toPetId))recipient.append(h('option',{value:String(args.toPetId),text:String(args.toPetId)}));recipient.value=String(args.toPetId);
   const kind=h('select',{'aria-label':`Tipo ${step.id}`},h('option',{value:'message',text:'Mensagem'}),h('option',{value:'request',text:'Pedido (dados; sem executar)'}))as HTMLSelectElement;kind.value=String(args.kind);if(isReply){kind.replaceChildren(h('option',{value:'reply',text:'Resposta à origem capturada'}));kind.value='reply';kind.disabled=true;recipient.disabled=true;}
   const text=h('textarea',{value:String(args.text??''),rows:3,maxLength:2048,'aria-label':`Texto ${step.id}`})as HTMLTextAreaElement;
   text.value=String(args.text??'');
   const deadline=h('input',{type:'text',value:iso(Number(args.expiresAt)),'aria-label':`Prazo UTC ${step.id}`})as HTMLInputElement;
   deadline.value=iso(Number(args.expiresAt));
   const renew=h('button',{type:'button',disabled:isReply,text:`Definir prazo de 5 minutos para ${step.id}`,onclick:()=>{if(renew.disabled)return;deadline.value=iso(Date.now()+300_000);dirty=true;}})as HTMLButtonElement;
   const box=h('div',{},h('p',{text:`Causa fixa: ${plan.taskId} / ${plan.id} / ${step.id}`}),h('label',{},'Descrição',description),h('label',{},'Pós-condição',postcondition),h('label',{},'Destinatário',recipient),h('label',{},'Tipo',kind),h('label',{},'Texto',text),h('label',{},'Prazo UTC (ISO)',deadline),h('p',{text:`Prazo original: ${iso(Number(args.expiresAt))}. Receber dados não autoriza uso por um modelo nem execução pelo destinatário.`}),renew);
   for(const input of [description,postcondition,recipient,kind,text,deadline]){input.addEventListener('input',()=>{dirty=true;});input.addEventListener('change',()=>{dirty=true;});}
   if(isReply)box.append(h('p',{text:'A origem recebida e o destinatário da resposta ficam fixos neste editor. Alterar o prazo não renova a autorização da origem; a entrega volta a verificá-la.'}),h('pre',{text:JSON.stringify(args.source,null,2)}));
   editors.push({operationId:step.operation.operationId as 'pet.message'|'pet.reply',cause:{taskId:plan.taskId,planId:plan.id,stepId:step.id},original,description,postcondition,recipient,kind,text,deadline,el:box});fields.append(box);
  }
  editorScope=scope(g);dirty=false;
 }
 function collect():RuntimeNativePlanEdit[]{
  if(!goal||editorScope!==scope(goal))throw Error('Os dados mudaram. Recarrega os passos antes de rever.');const output:RuntimeNativePlanEdit[]=[];
  for(const e of editors){
   const expiresAt=Date.parse(e.deadline.value);if(!/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{1,3})?Z$/.test(e.deadline.value)||!Number.isFinite(expiresAt)||expiresAt<=Date.now())throw Error(`Indica um prazo UTC futuro para ${e.cause.stepId}.`);
   const after:RuntimeStepInput={description:e.description.value,postcondition:e.postcondition.value,operation:{operationId:e.operationId,version:1,arguments:{expiresAt,...(e.operationId==='pet.reply'?{source:structuredClone((e.original.operation!.arguments as Record<string,unknown>).source)}:{kind:e.kind.value}),text:e.text.value,toPetId:e.recipient.value}}};
   const old=e.original.operation?.arguments as Record<string,unknown>;if(after.description!==e.original.description||after.postcondition!==e.original.postcondition||expiresAt!==old.expiresAt||e.operationId!=='pet.reply'&&e.kind.value!==old.kind||e.text.value!==old.text||e.recipient.value!==old.toPetId)output.push({cause:structuredClone(e.cause),after});
  }return output;
 }
 function render(){
  if(!goal)return;const current=goal,view=inventory(),review=actions.nativePlanReviews.get(current.id),canEdit=['paused','recoveryRequired'].includes(current.state);notice.textContent='';
  const derived=resultPlanEligible(current),native=nativeAdmission(current),hasDerived=current.nativePlanRevisions?.some(r=>[2,3].includes(r.schemaVersion));resultFields.hidden=!derived;resultFields.disabled=disabled||!!review;resultPrepare.disabled=resultFields.disabled;if(derived&&resultScope!==scope(current)){resultStep.replaceChildren(...current.plans.flatMap(p=>p.steps.filter(s=>!s.operation).map(s=>h('option',{value:JSON.stringify({taskId:p.taskId,planId:p.id,stepId:s.id}),text:s.description}))));resultStep.value=resultStep.querySelector('option')?.value??'';resultScope=scope(current);request=ids();}
  facts.replaceChildren();if(view?.coverage){facts.append(h('p',{text:`${view.coverage.receipts.length} passos cobertos por recibos originais. São leitura e não são reenviados por esta revisão.`}),h('ul',{},...view.coverage.receipts.map(r=>h('li',{text:`${r.planId} / ${r.stepId} · mensagem ${r.messageId} · destinatário ${r.recipientPetId} · ${r.hash}`}))));}
  if(!native){notice.textContent=hasDerived?'Proposta de resultado guardada. A divulgação efetiva ainda exige consentimento próprio e admissão sucessora; nenhuma entrega foi autorizada.':'Seleciona o excerto do resultado próprio. Guardar a proposta não autoriza enviar ou verificar os critérios.';}else if(!view){notice.textContent='Consulta primeiro os recibos e as revisões deste objetivo.';}else if(!view.factCoverageComplete){notice.textContent='Há trabalho ou informação por reconciliar. Consulta os estados abaixo antes de preparar outra revisão.';}
  if(view?.coverage&&view.factCoverageComplete&&canEdit&&(!editorScope||editorScope!==scope(current)&&!dirty))buildEditors(current,view);
  if(editorScope&&editorScope!==scope(current)&&dirty)notice.textContent='O objetivo mudou. O rascunho anterior foi conservado; recarrega os passos antes de preparar uma revisão nova.';
  fields.hidden=!native||!canEdit;fields.disabled=disabled||!!review||!view?.factCoverageComplete||editorScope!==scope(current);
  reset.disabled=disabled||!!review||!view?.factCoverageComplete||!canEdit;load.disabled=disabled;prepare.disabled=fields.disabled||!editors.length;prepare.hidden=!native||!canEdit;reset.hidden=!native;
  preview.replaceChildren();if(review){const p=review.proposal,original=structuredClone(current),published=current.nativePlanRevisions?.some(r=>r.id===p.changes.id);
   preview.append(h('h4',{text:'Revisão capturada do plano'}),h('p',{text:`Pet ${p.petId} · objetivo ${p.goalId} · revisão ${p.revision} · conversa ${p.conversationId}`}),h('p',{text:p.intent}),h('ul',{},...p.criteria.map(c=>h('li',{text:c}))),h('p',{text:`Válida até ${iso(p.expiresAt)}. Só dados; sem reserva de orçamento nem execução. Depois desta confirmação, revê a continuação separadamente.`}),h('p',{text:`Plano original ${p.changes.sourceProposalHash} · plano revisto ${p.changes.targetProposalHash}`}));
   for(const patch of p.changes.patches)preview.append(h('p',{text:`Causa ${patch.cause.taskId} / ${patch.cause.planId} / ${patch.cause.stepId}`}),h('pre',{text:JSON.stringify({antes:patch.before,depois:patch.after},null,2)}),h('p',{text:`Novo prazo UTC: ${iso(Number((patch.after.operation?.arguments as Record<string,unknown>)?.expiresAt))}`}));
   if(p.resultData)preview.append(resultDisclosureReview(p.resultData));
   const yes=h('button',{type:'button',text:published?'Confirmar recibo do plano':'Confirmar plano (sem executar)',disabled:disabled||!published&&(Date.now()>=p.expiresAt||current.revision!==p.revision),onclick:()=>{if(!yes.disabled)void options.run(()=>actions.confirmNativePlan(original,true));}})as HTMLButtonElement;
   const no=h('button',{type:'button',text:'Recusar revisão do plano',disabled:disabled,onclick:()=>{if(!no.disabled)void options.run(()=>actions.confirmNativePlan(original,false));}})as HTMLButtonElement;preview.append(yes,no);
  }
  pending.replaceChildren();for(const item of view?.items??[]){const original=structuredClone(current),ref=structuredClone(item.reference),box=h('div',{},h('p',{text:`${states[item.state]} · ${ref.id}${item.reason?' · '+item.reason:''}`}),h('pre',{text:JSON.stringify(ref.patches,null,2)}));
   if(item.canAck){const b=h('button',{type:'button',text:`Confirmar plano publicado ${ref.id}`,disabled:control,onclick:()=>{if(!b.disabled)void options.run(()=>actions.ackNativePlan(original,ref));}})as HTMLButtonElement;box.append(b);}
   if(item.canAbandon){const b=h('button',{type:'button',text:`Abandonar intenção ${ref.id}`,disabled:control,onclick:()=>{if(!b.disabled)void options.run(()=>actions.abandonNativePlan(original,ref));}})as HTMLButtonElement;box.append(h('p',{text:'Abandonar impede publicar esta intenção. Não elimina recibos, não devolve orçamento e não desfaz efeitos anteriores.'}),b);}pending.append(box);
  }
 }
 return {el,update(value:RuntimeGoal,busy:boolean,historical:boolean,pets:PetCapabilityDescriptor[]=[]){goal=value;disabled=busy;control=actions.controls.has(value.id);descriptors=pets;el.hidden=historical||recursiveAdmission(value)&&!resultPlanEligible(value)&&!value.nativePlanRevisions?.some(r=>r.schemaVersion===3)||!(nativeAdmission(value)||resultPlanEligible(value)||value.nativePlanRevisions?.some(r=>[2,3].includes(r.schemaVersion)))||!['paused','recoveryRequired','cancelled','completed'].includes(value.state);if(!el.hidden)render();},dispose(){goal=null;editors=[];}};
}
