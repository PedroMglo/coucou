import type {RuntimeGoal} from '../core/bridge';
import {RuntimeGoalActions,goalStateLabel,uncertainReceipt} from '../core/runtime-goals';
import {h} from './dom';
/** A stable native-user form; refreshing another goal never redirects a choice. */
export function buildRestoredFutureReview(initial:RuntimeGoal,actions:RuntimeGoalActions,run:(operation:()=>Promise<unknown>)=>unknown){
 let goal=initial,available=false,token='',requestId=`restore-future-${initial.id}-${initial.revision}`;
 const intent=h('textarea',{'aria-label':'Intenção do trabalho futuro',rows:3,maxlength:65536}) as HTMLTextAreaElement;intent.value=initial.spec.intent;
 const criteria=h('textarea',{'aria-label':'Critérios do trabalho futuro',rows:3,maxlength:131104}) as HTMLTextAreaElement;criteria.value=initial.spec.criteria.join('\n');
 const prepare=h('button',{type:'button',text:'Rever trabalho futuro deste histórico',onclick:()=>run(()=>actions.prepareRestoredFuture(goal,intent.value.trim(),criteria.value.split('\n').map(s=>s.trim()).filter(Boolean),requestId))}) as HTMLButtonElement;
 const form=h('div',{},h('label',{},'Intenção futura selecionada',intent),h('label',{},'Critérios futuros selecionados',criteria),prepare),review=h('div'),el=h('section',{'aria-label':'Trabalho futuro de origem restaurada'},form,review);
 let acknowledge:HTMLInputElement|undefined,confirm:HTMLButtonElement|undefined,decline:HTMLButtonElement|undefined;
 function draw(){const state=actions.futureReviews.get(initial.id),busy=actions.pending.has(initial.id);el.hidden=!available&&!state;form.hidden=!available||!!state;prepare.disabled=busy;
  if(state?.status==='review'){
   const ticket=state.ticket,p=ticket.proposal;
   if(token!==ticket.token){token=ticket.token;acknowledge=h('input',{type:'checkbox','aria-label':'Reconhecer estado anterior desconhecido'}) as HTMLInputElement;
    confirm=h('button',{type:'button',text:'Criar apenas o rascunho futuro',onclick:()=>run(()=>actions.confirmRestoredFuture(goal,true,acknowledge?.checked===true))}) as HTMLButtonElement;
    decline=h('button',{type:'button',text:'Recusar criação futura',onclick:()=>run(()=>actions.confirmRestoredFuture(goal,false,false))}) as HTMLButtonElement;
    acknowledge.addEventListener('change',draw);
    review.replaceChildren(h('p',{text:'Revê a origem e o destino deste novo rascunho. Não será copiada a conversa, sessão, memória, anexos, planos, orçamento ou autorização anteriores. A execução exige contexto e consentimento próprios.'}),h('strong',{text:p.future.intent}),h('ul',{},...p.future.criteria.map(text=>h('li',{text}))),h('p',{text:`Pet: ${p.future.owner.petId} · motor configurado: ${p.future.owner.provider}/${p.future.owner.adapter} · modelo indicado: ${p.future.owner.modelHint??'por resolver na admissão'}. O motor não será invocado para guardar estes dados.`}),h('details',{},h('summary',{text:'Origem e criação revistas por completo'}),h('pre',{text:JSON.stringify(p,null,2)})),h('label',{},acknowledge,'Reconheço que o consumo e os efeitos após o snapshot são desconhecidos e que o processo original pode continuar noutra origem.'),confirm,decline,h('button',{type:'button',text:'Consultar criação ou obter revisão nova',onclick:()=>run(()=>actions.reconcileRestoredFuture(goal))}));
   }
   const decided=actions.futureDecisionStarted(ticket.token);if(decided&&acknowledge)acknowledge.checked=true;if(acknowledge)acknowledge.disabled=busy||decided;
   if(confirm){confirm.textContent=decided?'Confirmar criação original':'Criar apenas o rascunho futuro';confirm.disabled=busy||acknowledge?.checked!==true;}if(decline){decline.disabled=busy;decline.hidden=false;decline.textContent=decided?'Recusar se a escolha ainda não foi recebida':'Recusar criação futura';}
  }else if(state?.status==='existing'){
   token='';const record=state.receipt.record;review.replaceChildren(h('p',{text:uncertainReceipt(state.receipt)?'A criação continua pendente de confirmação; nenhum trabalho será iniciado.':`Trabalho futuro guardado: ${record.id} · ${goalStateLabel(record.state)}. Usa o seu cartão para preparar contexto e rever limites próprios.`}));
   if(uncertainReceipt(state.receipt)){const origin=record.spec.restoredFrom;review.append(h('button',{type:'button',text:'Confirmar dados da criação existente',onclick:()=>{if(!origin)return;run(()=>actions.prepareRestoredFuture({...goal,id:origin.body.source.goalId,revision:origin.body.source.revision,spec:{...goal.spec,owner:{...goal.spec.owner,petId:origin.body.source.petId}}},record.spec.intent,record.spec.criteria,origin.body.clientRequestId));}}));}
   else review.append(h('button',{type:'button',text:'Rever outro trabalho futuro',onclick:()=>{actions.forgetRestoredFuture(initial.id);requestId=crypto.randomUUID();draw();}}));
  }else{token='';review.replaceChildren();}
 }
 return {el,update(next:RuntimeGoal,canPrepare:boolean){goal=next;available=canPrepare;draw();}};
}
