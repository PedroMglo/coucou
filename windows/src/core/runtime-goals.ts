import {systemPlanFromGoal as systemSoundFromGoal,validateSystemReview} from './runtime-system';
import {Bridge,type RuntimeGoal,type RuntimeFuturePrepared,type RuntimeReceipt,type RuntimeCreateInput,type RuntimeStepInput,type RuntimeBudget,type RuntimeAdmissionReview,type RuntimeAdmissionResult,type RuntimeCriterionInput,type RuntimeRecipeProgress,type RuntimePlanningLimits,type RuntimePlanningPolicy,type RuntimeDriverStatus,type RuntimeRecoveryPrepared,type RuntimeHistorySnapshot,type RuntimeRetirementPrepared,type PetMemorySelected,type PetMemoryGoalConsent,type RuntimeNativePlanEdit,type RuntimeNativePlanReview,type RuntimeNativePlanReference,type RuntimeNativePlanReceipt,type RuntimeNativePlanInventory} from './bridge';

export const goalRefs=(goal:RuntimeGoal)=>({petId:goal.spec.owner.petId,goalId:goal.id});
export const nativeAdmission=(goal:RuntimeGoal)=>[4,5,6,7,8,9,10,11,14,16,17,18,20,27,28].includes(latestAdmission(goal)?.spec.schemaVersion??0);
export const recursiveAdmission=(g:RuntimeGoal)=>[17,18,19,20].includes(latestAdmission(g)?.spec.schemaVersion??0);
export const resultPlanEligible=(g:RuntimeGoal)=>g.state==='paused'&&[12,13,19].includes(g.admissions?.at(-1)?.spec.schemaVersion??0)&&!!g.spec.receivedFrom?.body.delegation&&!g.nativePlanRevisions?.some(r=>[2,3].includes(r.schemaVersion))&&g.plans.some(p=>p.steps.some(s=>!s.operation));
export const resultDeliveryEligible=(g:RuntimeGoal)=>g.state==='paused'&&[12,13,19].includes(g.admissions?.at(-1)?.spec.schemaVersion??0)&&g.nativePlanRevisions?.some(r=>[2,3].includes(r.schemaVersion)&&r.targetRevision===g.revision)&&g.plans.length===1&&g.plans[0].steps.length===1&&g.plans[0].steps[0].operation?.operationId==='pet.resultReply';
export const canRequestFiniteStart=(g:RuntimeGoal)=>(g.state==='draft'||['paused','recoveryRequired'].includes(g.state)&&[12,13].includes(latestAdmission(g)?.spec.schemaVersion??0))&&['ollama','lmstudio'].includes(g.spec.owner.adapter)&&g.plans.length>0&&g.plans.every(p=>p.steps.length>0&&p.steps.every(s=>!s.operation));
export const canRequestSystemStart=(g:RuntimeGoal)=>g.state==='draft'&&!!systemSoundFromGoal(g);
export const canRequestNativeStart=(g:RuntimeGoal)=>['draft','paused','recoveryRequired'].includes(g.state)&&g.plans.length>0&&g.plans.every(p=>p.steps.length>0&&p.steps.every(s=>s.operation&&(['pet.message','pet.delegate','pet.workReport'].includes(s.operation.operationId)&&s.operation.version===1||s.operation.operationId==='pet.reply'&&[1,2].includes(s.operation.version))));
function validateExecutionStart(review:RuntimeAdmissionReview,original:RuntimeGoal,budget:RuntimeBudget,wanted:boolean,scope:'finiteResponse'|'resultDisclosure'|'nativePeerPlan',notBeforeAt?:number){
 if(review.status!=='review')return;const p=review.ticket.proposal,s=p.executionStart,retained=review.ticket.approvalScope==='confirmOriginalAdmission';
 if(!retained&&!sameData(s,wanted?{...(notBeforeAt===undefined?{}:{notBeforeAt}),schemaVersion:notBeforeAt===undefined?1:2,mode:'whenAvailable',scope}:undefined))throw Error('O arranque não corresponde à escolha original.');
 if(!s)return;const format=s.schemaVersion===1&&s.notBeforeAt===undefined||s.schemaVersion===2&&s.scope==='finiteResponse'&&Number.isSafeInteger(s.notBeforeAt)&&(s.notBeforeAt??0)>0&&(s.notBeforeAt??Infinity)<p.budget.deadlineAt;if(!format||s.mode!=='whenAvailable'||s.scope!==scope||p.goalId!==original.id||p.owner.petId!==original.spec.owner.petId||!retained&&(p.revision!==original.revision||!sameData(p.budget,budget))||s.scope==='finiteResponse'&&(p.operation!=='conversationRespond'||p.budget.maxModelCalls!==1||p.budget.maxToolCalls!==0||p.planningPolicy)||s.scope==='resultDisclosure'&&(p.operation!=='goalDeterministic'||!p.resultDisclosure||p.budget.maxModelCalls!==0||p.budget.maxToolCalls!==1)||s.scope==='nativePeerPlan'&&(p.operation!=='goalDeterministic'||!p.nativePeerPlan||p.nativePeerPlan.consent!=='boundedNativePeerPlan'||p.memoryData||p.engineBinding||p.resolvedModel||p.budget.payment!=='runtimeOnly'||p.workDisclosure||p.resultDisclosure||p.planningPolicy))throw Error('O âmbito de execução automática mudou; revê os dados originais.');
}
function validateNativePolicy(review:RuntimeAdmissionReview,original:RuntimeGoal,budget:RuntimeBudget,wanted:boolean){
 validateExecutionStart(review,original,budget,wanted,'nativePeerPlan');if(review.status!=='review')return;
 const p=review.ticket.proposal,n=p.nativePeerPlan;if(!wanted&&review.ticket.approvalScope!=='confirmOriginalAdmission'){if(n)throw Error('A revisão manual não pediu divulgação automática.');return;}if(!n)return;
 const covered=p.continuation?.native?.receipts??[];
 const steps=original.plans.flatMap(plan=>plan.steps.filter(step=>!covered.some(r=>r.planId===plan.id&&r.stepId===step.id)).map(step=>({cause:{taskId:plan.taskId,planId:plan.id,stepId:step.id},operation:step.operation})));
 if(n.schemaVersion!==1||n.goalId!==original.id||!sameData(n.owner,original.spec.owner)||!sameData(n.steps.map(({cause,operation})=>({cause,operation})),steps)||n.steps.some(s=>s.recipient.petId!==(s.operation.arguments as Record<string,unknown>).toPetId)||n.expiresAt!==p.budget.deadlineAt||n.sourceRevision!==p.revision)throw Error('O plano, dados e destinatários não correspondem ao trabalho revisto.');
}
export const latestAdmission=(goal:RuntimeGoal)=>goal.admissions?.at(-1);
export const uncertainReceipt=(receipt:RuntimeReceipt)=>receipt.status==='uncertain';
const errorText=(e:unknown)=>String(e).replace(/^Error:\s*/,'');
const sameData=(a:unknown,b:unknown):boolean=>{const order=(v:unknown):unknown=>Array.isArray(v)?v.map(order):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,order((v as Record<string,unknown>)[k])])):v;return JSON.stringify(order(a))===JSON.stringify(order(b));};
/** UI routing only: the native capture/F2, caller and original source remain
 * authoritative. A received identity without its exact selection is no data grant. */
export const canPlanReceivedData=(g:RuntimeGoal):boolean=>{const body=g.spec.receivedFrom?.body,context=g.executionContext,selection=context?.receivedRequest;return !!body&&(body.delegation?[2,4,7].includes(body.schemaVersion)&&body.delegation.allowance.modelCalls>0&&body.delegation.allowance.toolCalls>0:[1,3,5,6].includes(body.schemaVersion))&&context?.mode==='dedicated'&&!context.copiedMessages&&!!selection&&selection.schemaVersion===body.schemaVersion&&selection.source.recipientPetId===g.spec.owner.petId&&sameData(selection.source,body.source);};
const planningCatalogs:RuntimePlanningPolicy['catalog'][]=['spotifyTransportV1','peerCorrespondenceV1','peerCorrespondenceMemoryV1','peerDelegationV1','peerDelegationIterationV1'];
function validatePlanningReview(review:RuntimeAdmissionReview,original:RuntimeGoal,budget:RuntimeBudget,limits:RuntimePlanningLimits,requestId:string,start:boolean,catalog:RuntimePlanningPolicy['catalog']){
 if(review.status!=='review')return;const p=review.ticket.proposal,policy=p.planningPolicy,retained=review.ticket.approvalScope==='confirmOriginalAdmission';
 if(!policy||!planningCatalogs.includes(policy.catalog)||p.operation!=='goalPlanning'||p.goalId!==original.id||p.owner?.petId!==original.spec.owner.petId||p.toolExecutionAvailable!==false||!p.budget||!p.resolvedModel||p.executionStart||p.nativePeerPlan||p.resultDisclosure||p.workDisclosure)throw Error('A revisão de planeamento não corresponde ao objetivo e ao catálogo originais.');
 const schema=policy.catalog==='peerDelegationIterationV1'?(policy.start?4:3):(policy.start?2:1);
 if(policy.schemaVersion!==schema||policy.start!==undefined&&policy.start!=='whenAvailable'||!policy.limits)throw Error('A política de planeamento recebida é incompatível.');
 const funded=original.spec.receivedFrom?.body.delegation,selection=original.executionContext?.receivedRequest;
 if(p.funding||funded&&catalog==='peerDelegationIterationV1'&&canPlanReceivedData(original)){const f=p.funding;if(!funded||!canPlanReceivedData(original)||catalog!=='peerDelegationIterationV1'||!f||f.schemaVersion!==9||f.previous?.length||!sameData(f.source,original.spec.receivedFrom?.body.source)||!sameData(f.allowance,funded.allowance)||f.ownOriginHash!==selection?.ownOriginHash||p.budget.maxModelCalls>f.allowance.modelCalls||p.budget.maxToolCalls>f.allowance.toolCalls||p.budget.maxOutputBytes>f.allowance.outputBytes)throw Error('O planeamento não corresponde à reserva, origem e orçamento próprios da delegação.');}
 if(catalog==='peerDelegationIterationV1'&&canPlanReceivedData(original)&&(!p.memoryData||!sameData(p.memoryData.receivedRequest?.selection,original.executionContext?.receivedRequest)||p.memoryData.requiredConsent!=='boundedGoalContext'||p.memoryData.maxInputBytes!==262144||p.memoryData.destination.kind!=='server'||p.memoryData.destination.provider!==original.spec.owner.adapter||p.memoryData.destination.model!==p.resolvedModel||p.budget.deadlineAt>original.spec.receivedFrom!.body.source.deadlineAt))throw Error('Falta a revisão própria dos dados recebidos, destino/modelo ou prazo original.');
 // This explicit scope acknowledges the original approved transaction. It is
 // not a new review of the current form, destination or limits.
 if(retained)return;
 const bounds=(b:RuntimeBudget)=>({...b,maxTokens:b.maxTokens??null,maxCostMicros:b.maxCostMicros??null});
 if(p.revision!==original.revision||p.requestId!==requestId||!sameData(p.owner,original.spec.owner)||!sameData(p.context,original.executionContext)||p.intent!==original.spec.intent||!sameData(p.criteria,original.spec.criteria)||!sameData(bounds(p.budget),bounds(budget))||policy.catalog!==catalog||!sameData(policy.limits,limits)||policy.start!==(start?'whenAvailable':undefined))throw Error('A revisão de planeamento mudou o catálogo, os dados, os limites ou o arranque escolhidos.');
}
export const goalStateLabel=(state:RuntimeGoal['state'])=>({draft:'Rascunho',paused:'Pausado',cancelled:'Cancelado',recoveryRequired:'Requer recuperação',queued:'Autorizado na fila',leased:'Sessão reservada',stopping:'A interromper',completed:'Concluído com verificação nativa'})[state];
const receiptGoal=(receipt:RuntimeReceipt)=>{if(uncertainReceipt(receipt))throw Error('A gravação precisa de confirmação. Atualiza o objetivo antes de continuar; nenhuma ação seguinte foi enviada.');return receipt.record;};
const stableRequest=(prefix:string,goal:RuntimeGoal)=>`${prefix}-${latestAdmission(goal)?.spec.decisionId??goal.id}`;
export type GoalTransport=Pick<typeof Bridge,'petRuntimeRead'|'petRuntimePrepareRestoredFuture'|'petRuntimeConfirmRestoredFuture'|'petRuntimeHistory'|'petRuntimePrepareRetirement'|'petRuntimeConfirmRetirement'|'petRuntimePrepareDeterministicContinuation'|'petRuntimePrepareContinuation'|'petRuntimePrepareRecovery'|'petRuntimePreparePeerResume'|'petRuntimeConfirmRecovery'|'petRuntimePreparePlanning'|'petRuntimeStartDriver'|'petRuntimeDriverStatus'|'petRuntimeCreate'|'petRuntimeCreateDeterministic'|'petRuntimeNativePlanRevisions'|'petRuntimePrepareNativePlanRevision'|'petRuntimeConfirmNativePlanRevision'|'petRuntimeAckNativePlanRevision'|'petRuntimeAbandonNativePlanRevision'|'petRuntimeCloseDeterministic'|'petRuntimeVerifyDeterministic'|'petRuntimePrepareResultDisclosure'|'petRuntimePrepareSystem'|'petRuntimePrepareDeterministic'|'petRuntimeMutate'|'petRuntimePrepareContext'|'petRuntimeRefreshReadiness'|'petRuntimePrepareAdmission'|'petRuntimeConfirmAdmission'|'petRuntimeClaim'|'petRuntimeClaimStatus'|'petRuntimeReleaseClaim'|'petRuntimeRespondOnce'|'petRuntimeRequestRecipe'|'petRuntimeRunRecipe'|'petRuntimeRecipeProgress'|'petRuntimeResult'|'petRuntimeCompleteRecipe'|'petRuntimeRecoverStep'>;
/** User-driven native transactions. Opening/refreshing this controller performs
 * no IO. Failed/uncertain actions never schedule another write or effect. */
export class RuntimeGoalActions{
 readonly pending=new Set<string>();readonly controls=new Set<string>();readonly errors=new Map<string,string>();
 readonly drivers=new Map<string,RuntimeDriverStatus>();
 readonly nativePlanReviews=new Map<string,RuntimeNativePlanReview>();readonly nativePlanInventories=new Map<string,RuntimeNativePlanInventory>();
 readonly recoveryReviews=new Map<string,RuntimeRecoveryPrepared>();
 readonly futureReviews=new Map<string,RuntimeFuturePrepared>();private futureChoices=new Set<string>();
 readonly histories=new Map<string,RuntimeHistorySnapshot>();readonly retirementReviews=new Map<string,RuntimeRetirementPrepared>();
 readonly reviews=new Map<string,RuntimeAdmissionReview>();readonly responses=new Map<string,string>();readonly recipes=new Map<string,RuntimeRecipeProgress>();
 private transport:GoalTransport;private changed:()=>void;private admissionChoices=new Map<string,PetMemoryGoalConsent|undefined>();private disclosureChoices=new Map<string,string|undefined>();
 constructor(changed:()=>void=()=>{},transport:GoalTransport=Bridge){this.changed=changed;this.transport=transport;}
 private async action<T>(key:string,operation:()=>Promise<T>,control=false):Promise<T>{
  const group=control?this.controls:this.pending;if(group.has(key))throw Error('Este pedido já está em curso.');group.add(key);this.errors.delete(key);
  try{this.changed();return await operation();}catch(e){this.errors.set(key,errorText(e));throw e;}finally{group.delete(key);this.changed();}
 }
 async create(input:RuntimeCreateInput,steps:RuntimeStepInput[],selectedMemory?:PetMemorySelected[],execution:'inference'|'deterministic'='inference',selectedCalendars?:import('./bridge').RuntimeCalendarRequest):Promise<RuntimeGoal>{
  const original=structuredClone(input),proposed=structuredClone(steps),memory=structuredClone(selectedMemory),calendars=structuredClone(selectedCalendars);
  if(calendars&&(execution!=='inference'||(calendars.sourceScope==='pet'?!!original.conversationId:!original.conversationId)))throw Error('Seleciona o âmbito de origem dos calendários para este objetivo.');
  if(execution==='deterministic'&&memory?.length)throw Error('O trabalho literal sem inferência não aceita memória selecionada.');
  return this.action('create',async()=>{
   let goal=receiptGoal(await (execution==='deterministic'?this.transport.petRuntimeCreateDeterministic(original):this.transport.petRuntimeCreate(original)));
   goal=receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-task`,action:{kind:'proposeTask',description:proposed.some(s=>s.operation)?'Executar a sequência proposta':'Preparar uma resposta ao objetivo'}}));
   const task=goal.tasks.at(-1);if(!task)throw Error('A tarefa não foi confirmada. Atualiza o objetivo.');
   goal=receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-plan`,action:{kind:'proposePlan',taskId:task.id,steps:proposed}}));
   return receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-context`,mode:'dedicated',...(memory?.length?{selectedMemory:memory}:{}),...(calendars?{selectedCalendars:calendars}:{})}));
  });
 }
 /** Prepare own draft DATA through the normal CAS/context APIs. A stable
  * request/event binds the task; a last array item is not a causal reference. */
 async prepareReceivedPlan(goal:RuntimeGoal,steps:RuntimeStepInput[],includeReceivedRequest=false,selectedMemory?:PetMemorySelected[]):Promise<RuntimeGoal>{
  const original=structuredClone(goal),proposed=structuredClone(steps),memory=structuredClone(selectedMemory),prefix=`ownplan-${original.id}`;
  if(original.spec.receivedFrom?.body.schemaVersion===4&&proposed.some(s=>!s.operation)&&(!proposed.every(s=>!s.operation)||!includeReceivedRequest||!['ollama','lmstudio'].includes(original.spec.owner.adapter)))throw Error('A resposta recursiva exige seleção explícita da fonte, plano sem ferramentas e o motor local configurado.');
  if([3,5,6].includes(original.spec.receivedFrom?.body.schemaVersion??0)&&proposed.some(s=>!!s.operation))throw Error('O resultado recebido não concede encaminhamento nem ferramentas.');if(original.spec.receivedFrom?.body.delegation||[3,5,6].includes(original.spec.receivedFrom?.body.schemaVersion??0)){const response=proposed.every(s=>!s.operation);if(response&&(!['ollama','lmstudio'].includes(original.spec.owner.adapter)||!includeReceivedRequest))throw Error('Seleciona explicitamente o pedido delegado para a resposta com o motor local configurado.');if(!response&&(includeReceivedRequest||memory?.length))throw Error('A resposta nativa literal não usa contexto de modelo.');}if(!original.spec.receivedFrom||!proposed.length)throw Error('Escolhe um objetivo recebido e um plano próprio.');
  return this.action(original.id,async()=>{
   const current=(g:RuntimeGoal)=>{if(g.id!==original.id||!sameData(g.spec,original.spec)||g.state!=='draft'||g.admissions?.length)throw Error('O objetivo mudou; nenhum passo seguinte foi preparado.');return g;};
   let g=current(await this.transport.petRuntimeRead(original.spec.owner.petId,original.id));
   const ownTask=()=>{const event=g.events.find(e=>e.kind==='taskProposed'&&e.requestId===`${prefix}-task`);return event?g.tasks.find(t=>t.id===`task-${event.sequence}`):undefined;};
   let task=ownTask();if(!task){if(g.tasks.length||g.plans.length)throw Error('O rascunho já tem trabalho de outra preparação.');g=current(receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(g),revision:g.revision,requestId:`${prefix}-task`,action:{kind:'proposeTask',description:'Preparar o plano próprio revisto'}})));task=ownTask();}
   if(!task||g.tasks.length!==1||g.tasks[0].id!==task.id)throw Error('A tarefa própria não ficou isolada e confirmada.');
   const event=g.events.find(e=>e.kind==='planProposed'&&e.requestId===`${prefix}-plan`),plan=event?g.plans.find(p=>p.id===`plan-${event.sequence}`&&p.taskId===task!.id):undefined;
   if(plan){if(g.plans.length!==1||task.planId!==plan.id||!sameData(plan.steps.map(({id:_id,state:_state,...step})=>step),proposed))throw Error('O plano persistido corresponde a outra escolha. Conserva e revê os dados originais.');}
   else{if(g.plans.length||task.planId)throw Error('A tarefa já pertence a outro plano.');g=current(receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(g),revision:g.revision,requestId:`${prefix}-plan`,action:{kind:'proposePlan',taskId:task.id,steps:proposed}})));}
   const planEvent=g.events.find(e=>e.kind==='planProposed'&&e.requestId===`${prefix}-plan`),confirmed=planEvent?g.plans.find(p=>p.id===`plan-${planEvent.sequence}`&&p.taskId===task!.id):undefined;
   if(!confirmed||g.plans.length!==1||!sameData(confirmed.steps.map(({id:_id,state:_state,...step})=>step),proposed))throw Error('O plano próprio não ficou confirmado; nenhum contexto seguinte foi preparado.');
   if(g.executionContext){if(!!g.executionContext.receivedRequest!==includeReceivedRequest)throw Error('O contexto já preparado usa outra seleção de dados; não foi substituído.');if(memory!==undefined&&!sameData(g.executionContext.memory?.references.map(({id,revision})=>({id,revision}))??[],memory))throw Error('As recordações do contexto já preparado não foram substituídas.');return g;}
   return current(receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(g),revision:g.revision,requestId:`${prefix}-context`,mode:'dedicated',...(includeReceivedRequest?{includeReceivedRequest:true}:{}),...(memory?.length?{selectedMemory:memory}:{})})));
  });
 }
 futureDecisionStarted(token:string){return this.futureChoices.has(token);}
 forgetRestoredFuture(goalId:string){const state=this.futureReviews.get(goalId);if(state?.status!=='existing'||uncertainReceipt(state.receipt))throw Error('Confirma primeiro a criação anterior.');this.futureReviews.delete(goalId);this.changed();}
 async prepareRestoredFuture(goal:RuntimeGoal,intent:string,criteria:string[],requestId:string){
  const original=structuredClone(goal),selected=structuredClone(criteria);return this.action(original.id,async()=>{
   const reply=await this.transport.petRuntimePrepareRestoredFuture({petId:original.spec.owner.petId,sourceGoalId:original.id,sourceRevision:original.revision,requestId,intent,criteria:selected});this.futureReviews.set(original.id,reply);if(reply.status==='existing')receiptGoal(reply.receipt);return reply;
  });
 }
 async reconcileRestoredFuture(goal:RuntimeGoal){
  const state=this.futureReviews.get(goal.id);if(state?.status!=='review')throw Error('Não existe uma revisão original para consultar.');const ticket=structuredClone(state.ticket),p=ticket.proposal;
  return this.action(p.source.goalId,async()=>{const reply=await this.transport.petRuntimePrepareRestoredFuture({petId:p.source.petId,sourceGoalId:p.source.goalId,sourceRevision:p.source.revision,requestId:p.clientRequestId,intent:p.future.intent,criteria:structuredClone(p.future.criteria)});this.futureReviews.set(p.source.goalId,reply);if(reply.status==='existing'||reply.ticket.token!==ticket.token)this.futureChoices.delete(ticket.token);if(reply.status==='existing')receiptGoal(reply.receipt);return reply;});
 }
 async confirmRestoredFuture(goal:RuntimeGoal,approve:boolean,acknowledgeUnknownState:boolean){
  const state=this.futureReviews.get(goal.id);if(state?.status!=='review')throw Error('Revê primeiro o trabalho futuro.');const ticket=structuredClone(state.ticket),p=ticket.proposal;
  if(approve&&!acknowledgeUnknownState)throw Error('Reconhece explicitamente que o estado e o consumo anteriores são desconhecidos.');
  return this.action(p.source.goalId,async()=>{if(approve)this.futureChoices.add(ticket.token);const reply=await this.transport.petRuntimeConfirmRestoredFuture({petId:p.source.petId,sourceGoalId:p.source.goalId,sourceRevision:p.source.revision,token:ticket.token,approve,acknowledgeUnknownState:approve&&acknowledgeUnknownState});
   if(reply.status==='created'){if(uncertainReceipt(reply.receipt)){this.futureReviews.set(p.source.goalId,{...state,pendingCreation:reply.receipt});throw Error('A criação precisa de confirmação. Repete a escolha original; não prepares outro rascunho.');}this.futureReviews.set(p.source.goalId,{status:'existing',receipt:reply.receipt,executionAuthorityGranted:false});this.futureChoices.delete(ticket.token);}else{this.futureReviews.delete(p.source.goalId);this.futureChoices.delete(ticket.token);}return reply;
  });
 }
 async readHistory(goal:RuntimeGoal){const original=structuredClone(goal);return this.action(goal.id,async()=>{const history=await this.transport.petRuntimeHistory({...goalRefs(original),revision:original.revision});this.histories.set(original.id,history);return history;});}
 async prepareRetirement(goal:RuntimeGoal,decisionId:string,requestId:string){
  const original=structuredClone(goal);return this.action(goal.id,async()=>{const reply=await this.transport.petRuntimePrepareRetirement({...goalRefs(original),revision:original.revision,decisionId,requestId});this.retirementReviews.set(original.id,reply);if(reply.status==='retired'){if(uncertainReceipt(reply.receipt))throw Error('A retirada precisa de confirmação. Volta a consultar esta operação.');this.histories.delete(original.id);}return reply;});
 }
 async confirmRetirement(goal:RuntimeGoal,approve:boolean){
  const review=this.retirementReviews.get(goal.id);if(review?.status!=='review')throw Error('Revê a retirada da autorização antes de confirmar.');const ticket=structuredClone(review.ticket),p=ticket.proposal;
  return this.action(goal.id,async()=>{const reply=await this.transport.petRuntimeConfirmRetirement({petId:p.petId,goalId:p.goalId,revision:p.revision,decisionId:p.decisionId,token:ticket.token,approve});if(reply.status==='retired'){if(uncertainReceipt(reply.receipt))throw Error('A retirada precisa de confirmação. Repete a confirmação original.');this.retirementReviews.delete(goal.id);this.histories.delete(goal.id);}else if(reply.status==='declined'||reply.status==='blocked'&&!reply.retained)this.retirementReviews.delete(goal.id);return reply;});
 }
 async prepareRecovery(goal:RuntimeGoal,requestId=`close-lost-${goal.revision}`){
  const original=structuredClone(goal);return this.action(goal.id,async()=>{const reply=await this.transport.petRuntimePrepareRecovery({...goalRefs(original),revision:original.revision,requestId});if(reply.status==='closed'&&uncertainReceipt(reply.receipt)){this.recoveryReviews.delete(original.id);throw Error('O fecho precisa de confirmação. Volta a consultar esta operação.');}this.recoveryReviews.set(original.id,reply);return reply;});
 }
 async preparePeerResume(goal:RuntimeGoal,requestId=`resume-owner-${goal.revision}`){
  const original=structuredClone(goal);return this.action(original.id,async()=>{
   const reply=await this.transport.petRuntimePreparePeerResume({...goalRefs(original),revision:original.revision,requestId});
   if(reply.status==='review'){
    const p=reply.ticket.proposal,s=p.peerResumption;
    if(!s||p.approvalScope==='resumeOriginalPeerExecution'&&p.revision!==original.revision||p.goalId!==original.id||p.petId!==original.spec.owner.petId||s.goalId!==p.goalId||s.petId!==p.petId||s.goalRevision!==p.revision||s.additionalAllowance!==false||s.modelUseAuthorized!==false||p.resultingState!=='leased'||!['resumeOriginalPeerExecution','ackRecordedPeerResume'].includes(p.approvalScope))throw Error('A revisão de retoma não corresponde ao objetivo original.');
   }
   this.recoveryReviews.set(original.id,reply);if(reply.status==='acknowledged'&&uncertainReceipt(reply.receipt))throw Error('O registo de retoma precisa de confirmação. Volta a consultar esta operação.');return reply;
  });
 }
 async confirmRecovery(goal:RuntimeGoal,approve:boolean){
  const review=this.recoveryReviews.get(goal.id);if(review?.status!=='review')throw Error('Revê a operação de recuperação antes de confirmar.');const ticket=structuredClone(review.ticket);
  return this.action(goal.id,async()=>{
   // Native passive resumption checks identity/configuration and the original
   // authority; neither the first choice nor an ACK needs provider readiness.
   const reply=await this.transport.petRuntimeConfirmRecovery({petId:ticket.proposal.petId,goalId:ticket.proposal.goalId,revision:ticket.proposal.revision,token:ticket.token,approve});
   if(reply.status==='acknowledgementPending')throw Error('A retoma precisa de confirmação. Repete a confirmação original; nenhum trabalho foi iniciado.');
   if(reply.status==='resumed'){if(uncertainReceipt(reply.receipt))throw Error('A retoma precisa de confirmação.');this.recoveryReviews.delete(goal.id);this.drivers.delete(goal.id);}
   else if(reply.status==='closed'){if(uncertainReceipt(reply.receipt))throw Error('O fecho precisa de confirmação. Repete esta confirmação; o trabalho não será reiniciado.');this.recoveryReviews.delete(goal.id);}
   else if(reply.status==='declined'||reply.status==='blocked'&&reply.retained===false)this.recoveryReviews.delete(goal.id);return reply;});
 }
 async prepareContext(goal:RuntimeGoal,requestId:string,selectedMemory?:PetMemorySelected[]){const original=structuredClone(goal),memory=structuredClone(selectedMemory);return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(original),revision:original.revision,requestId,mode:'dedicated',...(memory?.length?{selectedMemory:memory}:{})})));}
 private async readiness(goal:RuntimeGoal,requestedStart=false){if(nativeAdmission(goal))return;if(['ollama','lmstudio'].includes(goal.spec.owner.adapter)){try{await this.transport.petRuntimeRefreshReadiness(goal.spec.owner.petId,goal.id);}catch(error){if(!requestedStart||String(error).replace(/^Error:\s*/,'')!=='busy')throw error;}}}
 async prepareSystem(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string,criteria?:RuntimeCriterionInput[],start=false){
  const original=structuredClone(goal),limits=structuredClone(budget),bindings=structuredClone(criteria??systemSoundFromGoal(original)?.verification);
  if(!bindings?.length)throw Error('Revê os critérios tipados desta tarefa de Sistema.');
  return this.action(original.id,async()=>{const review=await this.transport.petRuntimePrepareSystem({...goalRefs(original),revision:original.revision,requestId,budget:limits,criteria:bindings,...(start?{start:'whenAvailable' as const}:{})});validateSystemReview(review,original,limits,bindings,requestId,start);this.reviews.set(original.id,review);return review;});
 }
 async prepareDeterministic(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string,start=false){
  const original=structuredClone(goal),limits=structuredClone(budget);
  return this.action(goal.id,async()=>{if(start&&!canRequestNativeStart(original))throw Error('O plano nativo não suporta este arranque.');const review=await this.transport.petRuntimePrepareDeterministic({...goalRefs(original),revision:original.revision,requestId,budget:limits,...(start?{start:'whenAvailable' as const}:{})});validateNativePolicy(review,original,limits,start);this.reviews.set(original.id,review);return review;});
 }
 async prepareResultDisclosure(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string,start=false){
  const original=structuredClone(goal),bounds=structuredClone(budget);if(!resultDeliveryEligible(original)||bounds.maxModelCalls!==0||bounds.maxToolCalls!==1||bounds.payment!=='runtimeOnly'||bounds.deadlineAt!==Number((original.plans[0].steps[0].operation!.arguments as Record<string,unknown>).expiresAt))throw Error('Revê um resultado próprio e seleciona uma única entrega sem inferência.');
  return this.action(original.id,async()=>{const result=await this.transport.petRuntimePrepareResultDisclosure({...goalRefs(original),revision:original.revision,requestId,budget:bounds,...(start?{start:'whenAvailable' as const}:{})});validateExecutionStart(result,original,bounds,start,'resultDisclosure');
   if(result.status==='review'){const p=result.ticket.proposal,d=p.resultDisclosure,v=p.resultData,c=p.continuation,step=original.plans[0].steps[0];if(!d||!v||!c||c.schemaVersion!==(original.spec.receivedFrom?.body.schemaVersion===4?10:7)||c.additionalAllowance!==false||p.operation!=='goalDeterministic'||p.owner.petId!==original.spec.owner.petId||p.goalId!==original.id||p.revision!==original.revision||!sameData(p.context,original.executionContext)||p.intent!==original.spec.intent||!sameData(p.criteria,original.spec.criteria)||d.schemaVersion!==(original.spec.receivedFrom?.body.schemaVersion===4?2:1)||v.schemaVersion!==d.schemaVersion||p.funding?.schemaVersion!==(d.schemaVersion===2?8:3)||d.sourceRevision!==original.revision||d.consent!=='boundedResultDisclosure'||!sameData(d.arguments,step.operation!.arguments)||!sameData(d.recipient,v.recipient)||!sameData(d.origin,v.origin)||!sameData(d.arguments,v.arguments)||!sameData(d.memoryReferences,original.executionContext?.memory?.references??[])||!sameData(d.source,original.spec.receivedFrom!.body.source)||p.memoryData||p.workDisclosure||p.engineBinding||p.resolvedModel||!sameData(p.budget,bounds))throw Error('A concessão não corresponde ao resultado, destinatário e orçamento originais.');}
   this.reviews.set(original.id,result);return result;});
 }
 async prepareAdmission(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string,start=false,notBeforeAt?:number){
  if(notBeforeAt!==undefined&&(!start||!Number.isSafeInteger(notBeforeAt)||notBeforeAt<=0||notBeforeAt>=budget.deadlineAt))throw Error('A hora UTC exige arranque pedido e tem de anteceder o prazo original.');
  if(recursiveAdmission(goal))throw Error('Esta admissão recursiva não pode ser renovada por nova autorização de trabalho.');
  const original=structuredClone(goal),limits=structuredClone(budget);
  if(start&&(!canRequestFiniteStart(original)||limits.maxModelCalls!==1||limits.maxToolCalls!==0||limits.payment!=='localOnly'))throw Error('O arranque finito exige uma resposta local e zero ferramentas.');
  return this.action(goal.id,async()=>{await this.readiness(original,start);const review=await this.transport.petRuntimePrepareAdmission({...goalRefs(original),revision:original.revision,requestId,budget:limits,...(start?{start:'whenAvailable' as const}:{}),...(notBeforeAt===undefined?{}:{notBeforeAt})});validateExecutionStart(review,original,limits,start,'finiteResponse',notBeforeAt);this.reviews.set(original.id,review);return review;});
 }
 async preparePlanning(goal:RuntimeGoal,budget:RuntimeBudget,limits:RuntimePlanningLimits,requestId:string,start=false,catalog:RuntimePlanningPolicy['catalog']='spotifyTransportV1'){
  if((goal.spec.receivedFrom?.body.schemaVersion===4||recursiveAdmission(goal))&&!(goal.state==='draft'&&catalog==='peerDelegationIterationV1'&&canPlanReceivedData(goal)))throw Error('Este pedido recursivo exige seleção exata, consentimento próprio e catálogo de planeamento compatível.');
  const original=structuredClone(goal),bounds=structuredClone(budget),planning=structuredClone(limits);
  const funded=original.spec.receivedFrom?.body.delegation;
  if(funded&&catalog==='peerDelegationIterationV1'&&canPlanReceivedData(original)&&(bounds.maxModelCalls>funded.allowance.modelCalls||bounds.maxToolCalls>funded.allowance.toolCalls||bounds.maxOutputBytes>funded.allowance.outputBytes))throw Error('O planeamento excede a reserva original da delegação.');
  if(catalog==='peerDelegationIterationV1'&&(!['ollama','lmstudio'].includes(original.spec.owner.adapter)||bounds.payment!=='localOnly'||!Number.isInteger(planning.maxRounds)||planning.maxRounds<(funded?1:2)||planning.maxRounds>8||!Number.isInteger(bounds.maxModelCalls)||planning.maxRounds>bounds.maxModelCalls||!Number.isInteger(planning.maxSteps)||planning.maxSteps<1||planning.maxSteps>8||planning.maxSteps>bounds.maxToolCalls||!Number.isInteger(planning.maxOutputBytes)||planning.maxOutputBytes<2048||planning.maxOutputBytes>65536||planning.maxOutputBytes>bounds.maxOutputBytes||!Number.isInteger(planning.noProgressLimit)||planning.noProgressLimit<1||planning.noProgressLimit>planning.maxRounds||bounds.maxTokens!=null||bounds.maxCostMicros!=null||original.executionContext?.mode!=='dedicated'||original.executionContext?.copiedMessages||original.executionContext?.memory&&!original.executionContext.memory.references.length||original.executionContext?.receivedRequest&&!canPlanReceivedData(original)||!!original.spec.receivedFrom?.body.delegation&&!canPlanReceivedData(original)||[3,5,6].includes(original.spec.receivedFrom?.body.schemaVersion??0)&&!canPlanReceivedData(original)||!!original.spec.receivedFrom&&bounds.deadlineAt>original.spec.receivedFrom.body.source.deadlineAt))throw Error('A delegação iterativa exige rondas próprias limitadas pelo orçamento total (1 a 8 para trabalho financiado, 2 a 8 nos restantes casos), motor local e contexto dedicado; memória/dados recebidos exigem seleção exata e consentimento R43 próprio; cada entrega mantém a revisão e proveniência nativas.');
  if(catalog==='peerDelegationV1'&&(planning.maxRounds!==1||bounds.maxModelCalls<1||original.executionContext?.mode!=='dedicated'||original.executionContext?.copiedMessages||original.executionContext?.memory||original.executionContext?.receivedRequest))throw Error('A delegação proposta exige uma ronda própria, orçamento total partilhado e contexto dedicado sem memória ou dados recebidos.');
  if(catalog==='peerCorrespondenceMemoryV1'&&(bounds.maxModelCalls!==1||planning.maxRounds!==1||original.executionContext?.mode!=='dedicated'||!original.executionContext?.memory?.references.length||original.executionContext?.copiedMessages||original.executionContext?.receivedRequest))throw Error('A correspondência com memória exige uma ronda local, contexto dedicado e memória selecionada. Contexto recebido ou histórico copiado não autorizam encaminhamento.');
  if(catalog==='peerCorrespondenceV1'&&(bounds.maxModelCalls!==1||planning.maxRounds!==1||original.executionContext?.copiedMessages||original.executionContext?.memory||original.executionContext?.receivedRequest))throw Error('A correspondência usa uma ronda local e contexto dedicado. Dados de memória ou recebidos exigem a extensão de proveniência ainda pendente.');
  return this.action(goal.id,async()=>{await this.readiness(original,start);const review=await this.transport.petRuntimePreparePlanning({...goalRefs(original),revision:original.revision,requestId,budget:bounds,limits:planning,catalog,...(start?{start:'whenAvailable' as const}:{})});validatePlanningReview(review,original,bounds,planning,requestId,start,catalog);this.reviews.set(original.id,review);return review;});
 }
 async prepareDeterministicContinuation(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string,start=false){
  const original=structuredClone(goal),bounds=structuredClone(budget);if(!nativeAdmission(original))throw Error('Este objetivo não usa a execução determinística.');if(recursiveAdmission(original))throw Error('A continuação recursiva ainda exige um contrato próprio; o saldo e o prazo não são renovados.');
  return this.action(original.id,async()=>{if(start&&!canRequestNativeStart(original))throw Error('A continuação não suporta este arranque.');const review=await this.transport.petRuntimePrepareDeterministicContinuation({...goalRefs(original),revision:original.revision,requestId,budget:bounds,...(start?{start:'whenAvailable' as const}:{})});validateNativePolicy(review,original,bounds,start);this.reviews.set(original.id,review);return review;});
 }
 async prepareContinuation(goal:RuntimeGoal,budget:RuntimeBudget,limits:RuntimePlanningLimits|null,requestId:string,reviewUncertainEffects=false,start=false){
  if(goal.spec.receivedFrom?.body.schemaVersion===4||recursiveAdmission(goal))throw Error('Este pedido recursivo ainda não autoriza inferência, planeamento ou continuação.');
  const original=structuredClone(goal),bounds=structuredClone(budget),planning=structuredClone(limits);
  if(start&&!planning&&(!canRequestFiniteStart(original)||bounds.maxModelCalls!==1||bounds.maxToolCalls!==0||bounds.payment!=='localOnly'))throw Error('A continuação finita exige uma resposta local dentro da reserva original.');
  return this.action(goal.id,async()=>{await this.readiness(original,start);const review=await this.transport.petRuntimePrepareContinuation({...goalRefs(original),revision:original.revision,requestId,budget:bounds,limits:planning,...(reviewUncertainEffects?{reviewUncertainEffects:true}:{}),...(start?{start:'whenAvailable' as const}:{})});if(!planning)validateExecutionStart(review,original,bounds,start,'finiteResponse');this.reviews.set(original.id,review);return review;});
 }
 async startDriver(goal:RuntimeGoal){
  const original=structuredClone(goal),row=latestAdmission(original),lease=row?.lease;if(!lease||!(row.spec.executionStart?.scope==='nativeSystemPlan'&&row.spec.nativeSystemPlan)&&![12,13,15,19,21,22,23].includes(row.spec.schemaVersion)&&row.spec.executionStart?.scope!=='finiteResponse'&&![4,5,6,7,8,10,11,16,17,18].includes(row.spec.schemaVersion)&&!(row.spec.executionStart?.scope==='nativePeerPlan'&&row.spec.nativePeerPlan)&&!([14,20].includes(row.spec.schemaVersion)&&row.spec.resultDisclosure)&&!(row.spec.schemaVersion===9&&row.spec.workDisclosure)&&(![3,24,25,26].includes(row.spec.schemaVersion)||!row.spec.planningPolicy))throw Error('Revê e autoriza primeiro o planeamento limitado deste objetivo.');
  return this.action(goal.id,async()=>{await this.liveOwner(original,lease.id);const status=await this.transport.petRuntimeStartDriver({...goalRefs(original),leaseId:lease.id});this.drivers.set(original.id,status);return status;});
 }
 async retryClosure(goal:RuntimeGoal){
  const original=structuredClone(goal),lease=latestAdmission(original)?.lease;if(!lease)throw Error('O objetivo não tem uma referência de sessão.');
  return this.action(original.id,async()=>{const input={...goalRefs(original),leaseId:lease.id};const current=await this.transport.petRuntimeDriverStatus(input);if(!current.retainedClosure)throw Error('Não existe uma prova nativa de fecho retida.');const status=await this.transport.petRuntimeStartDriver(input);this.drivers.set(original.id,status);return status;});
 }
 async readDriver(goal:RuntimeGoal){const lease=latestAdmission(goal)?.lease;if(!lease)throw Error('O objetivo ainda não tem uma sessão reservada.');const status=await this.transport.petRuntimeDriverStatus({...goalRefs(goal),leaseId:lease.id});this.drivers.set(goal.id,status);this.changed();return status;}
 async confirmAdmission(goal:RuntimeGoal,approve:boolean,memoryConsent?:PetMemoryGoalConsent,disclosureConsent?:'boundedWorkDisclosure'|'boundedResultDisclosure'|'boundedNativePeerPlan'):Promise<RuntimeAdmissionResult>{
  const review=this.reviews.get(goal.id);if(review?.status!=='review')throw Error('Prepara uma revisão antes de decidir.');const ticket=structuredClone(review.ticket),p=ticket.proposal;const cleanup=ticket.approvalScope==='confirmOriginalAdmission';
  // Validate the separate, explicit data scope before metadata or authority IO.
  // A retained acknowledgement reuses this choice; it cannot renew its scope.
  if(approve&&!cleanup&&(p.nativePeerPlan?disclosureConsent!=='boundedNativePeerPlan':p.resultDisclosure?disclosureConsent!=='boundedResultDisclosure':p.workDisclosure?disclosureConsent!=='boundedWorkDisclosure':disclosureConsent!==undefined))throw Error('Revê e autoriza explicitamente a divulgação futura limitada de metadados.');
  if(approve&&cleanup&&disclosureConsent!==undefined)throw Error('A confirmação original não aceita nova divulgação.');
  if(approve&&!cleanup&&this.disclosureChoices.has(ticket.token)&&this.disclosureChoices.get(ticket.token)!==disclosureConsent)throw Error('Conserva o consentimento original de divulgação.');
  if(approve&&!cleanup&&(p.memoryData?memoryConsent!==p.memoryData.requiredConsent:memoryConsent!==undefined))throw Error('Revê e autoriza explicitamente o âmbito de utilização dos dados deste objetivo.');
  if(approve&&cleanup&&memoryConsent!==undefined)throw Error('A confirmação pendente conserva o consentimento original; não aceita um âmbito novo.');
  if(approve&&!cleanup&&this.admissionChoices.has(ticket.token)&&this.admissionChoices.get(ticket.token)!==memoryConsent)throw Error('A confirmação tem de conservar o consentimento original.');
  return this.action(p.goalId,async()=>{
   if(approve&&!this.admissionChoices.has(ticket.token)){
    if(!cleanup&&p.operation!=='goalDeterministic'&&['draft','recoveryRequired'].includes(goal.state)&&['ollama','lmstudio'].includes(p.owner.adapter)){try{await this.transport.petRuntimeRefreshReadiness(p.owner.petId,p.goalId);}catch(error){if(!(p.executionStart?.mode==='whenAvailable'||p.planningPolicy?.start==='whenAvailable')||errorText(error)!=='busy')throw error;}}
    this.admissionChoices.set(ticket.token,memoryConsent);this.disclosureChoices.set(ticket.token,disclosureConsent);
   }
   const result=await this.transport.petRuntimeConfirmAdmission({petId:p.owner.petId,goalId:p.goalId,revision:p.revision,token:ticket.token,approve,...(approve&&memoryConsent?{memoryConsent}:{}),...(approve&&disclosureConsent?{disclosureConsent}:{})});
   const retained=(result.status==='queued'&&uncertainReceipt(result.receipt))||((result.status==='blocked'||result.status==='authorityBlocked')&&result.retryable===true)||(result.status==='blocked'&&result.reason==='dataConsentRequired');
   if(!retained){this.reviews.delete(p.goalId);this.admissionChoices.delete(ticket.token);this.disclosureChoices.delete(ticket.token);}if(result.status==='queued')receiptGoal(result.receipt);return result;
  });
 }

 async readNativePlans(goal:RuntimeGoal){
  const original=structuredClone(goal);return this.action(original.id,async()=>{
   this.nativePlanInventories.delete(original.id);let cursor:string|null=null,first:RuntimeNativePlanInventory|undefined;const ids=new Set<string>();
   for(let page=0;page<64;page++){
    const data=await this.transport.petRuntimeNativePlanRevisions({...goalRefs(original),revision:original.revision,cursor});
    if(data.schemaVersion!==1||data.goalId!==original.id||data.petId!==original.spec.owner.petId||data.revision!==original.revision||data.executionAuthorized!==false||data.repairPerformed!==false||!Array.isArray(data.items)||data.items.some(i=>i.reference.goalId!==original.id||i.reference.petId!==original.spec.owner.petId)||first&&first.sourceHash!==data.sourceHash)throw Error('A consulta não corresponde à fonte original. Atualiza antes de rever.');
    if(!first)first=structuredClone(data);else first.items.push(...structuredClone(data.items));
    for(const item of data.items){if(ids.has(item.reference.id))throw Error('A consulta repetiu uma revisão. Atualiza a fonte.');ids.add(item.reference.id);}
    if(new TextEncoder().encode(JSON.stringify(first)).length>4*1024*1024)throw Error('A consulta atingiu o limite de dados.');
    if(!data.hasMore){first.hasMore=false;first.nextCursor=null;this.nativePlanInventories.set(original.id,first);return first;}
    if(!data.nextCursor||data.nextCursor===cursor)throw Error('A continuação da consulta é inválida.');cursor=data.nextCursor;
   }
   throw Error('A consulta atingiu o limite de páginas.');
  });
 }
 async prepareNativePlan(goal:RuntimeGoal,inputs:RuntimeNativePlanEdit[],requestId:string){
  const original=structuredClone(goal),edits=structuredClone(inputs);if(recursiveAdmission(original)&&!resultPlanEligible(original))throw Error('A revisão de plano recursivo ainda exige contrato próprio.');if(this.nativePlanReviews.has(original.id))throw Error('Decide ou confirma a revisão anterior antes de preparar outra.');if(!(nativeAdmission(original)&&['paused','recoveryRequired'].includes(original.state)||resultPlanEligible(original)&&edits.length===1&&edits[0].after.operation?.operationId==='pet.resultReply'))throw Error('Confirma primeiro o fecho nativo deste objetivo.');
  return this.action(original.id,()=>this.captureNativePlan(original,edits,requestId));
 }
 private async captureNativePlan(original:RuntimeGoal,edits:RuntimeNativePlanEdit[],requestId:string){
   const before={tasks:original.tasks.map(({id,description,planId})=>({id,description,planId})),plans:original.plans.map(({id,taskId,steps})=>({id,taskId,steps:steps.map(({id,description,postcondition,operation})=>({id,description,postcondition,...(operation?{operation:structuredClone(operation)}:{})}))}))};
   const after=structuredClone(before);for(const edit of edits){const step=after.plans.find(p=>p.id===edit.cause.planId&&p.taskId===edit.cause.taskId)?.steps.find(s=>s.id===edit.cause.stepId);if(!step)throw Error('O passo não pertence ao plano capturado.');Object.assign(step,structuredClone(edit.after));}
   const ticket=await this.transport.petRuntimePrepareNativePlanRevision({...goalRefs(original),revision:original.revision,inputs:edits,requestId}),p=ticket.proposal,r=p.changes;
   if(!sameData(p.before,before)||!sameData(p.after,after)||r.patches.some(v=>!sameData(v.before,before.plans.find(p=>p.id===v.cause.planId&&p.taskId===v.cause.taskId)?.steps.map(({id,...input})=>({id,input})).find(s=>s.id===v.cause.stepId)?.input))||p.goalId!==original.id||p.petId!==original.spec.owner.petId||p.revision!==original.revision||p.intent!==original.spec.intent||JSON.stringify(p.criteria)!==JSON.stringify(original.spec.criteria)||p.executionAuthorized!==false||p.budgetReserved!==false||r.schemaVersion!==(original.spec.receivedFrom?.body.schemaVersion===4?3:resultPlanEligible(original)?2:1)||r.goalId!==original.id||r.petId!==p.petId||r.sourceRevision!==p.revision||r.requestId!==requestId||!sameData(r.patches.map(v=>({cause:v.cause,after:v.after})),edits)||([2,3].includes(r.schemaVersion)?(!!p.coverage||!resultPlanEligible(original)||edits.length!==1||edits[0].after.operation?.operationId!=='pet.resultReply'):r.schemaVersion!==1||!p.coverage||p.coverage.proposalHash!==r.sourceProposalHash||r.patches.some(v=>p.coverage!.receipts.some(w=>w.planId===v.cause.planId&&w.stepId===v.cause.stepId))))throw Error('A revisão não corresponde aos passos capturados.');
   const data=p.resultData,edit=edits[0],args=edit?.after.operation?.arguments as import('./bridge').RuntimeDerivedResultReply|undefined;
   if([2,3].includes(r.schemaVersion)&&(!data||data.schemaVersion!==(r.schemaVersion===3?2:1)||data.goalId!==original.id||data.petId!==original.spec.owner.petId||data.revision!==original.revision||data.proposalHash!==r.targetProposalHash||!sameData(data.cause,edit.cause)||!sameData(data.arguments,args)||data.recipient.petId!==args?.toPetId||!sameData(data.memoryReferences,original.executionContext?.memory?.references??[])||data.receivedMessage.schemaVersion!==(r.schemaVersion===3?7:4)||data.receivedMessage.kind!=='delegation'||data.receivedMessage.origin.petId!==original.spec.receivedFrom?.body.source.senderPetId||data.receivedMessage.origin.decisionId!==args?.source?.decisionId||data.receivedMessage.messageId!==args?.source?.messageId||data.receivedMessage.recipient.petId!==original.spec.owner.petId||data.useScope!=='oneLocalReplyToDelegationSource'||data.dependencyPolicy!=='allOriginalSelectedSourcesMustRemainCurrent'||data.requiredConsent!=='boundedResultDisclosure'||data.providerUseAuthorized!==false||data.sharingAuthorized!==false||data.expiresAt>p.expiresAt||!data.origin.provider||!data.origin.model||data.origin.inferenceDestination.model!==data.origin.model))throw Error('A origem ou o destino da revisão de resultado não corresponde aos dados capturados.');
   this.nativePlanReviews.set(original.id,ticket);return ticket;
 }
 async prepareResultPlan(goal:RuntimeGoal,cause:RuntimeNativePlanEdit['cause'],start:number,end:number,requestId:string){
  const original=structuredClone(goal),selected=structuredClone(cause),row=latestAdmission(original),source=structuredClone(original.spec.receivedFrom?.body.source);
  if(!resultPlanEligible(original)||!row||!source||![start,end].every(Number.isSafeInteger)||start<0||end<=start||end-start>2048||this.nativePlanReviews.has(original.id))throw Error('Escolhe um resultado próprio fechado e um intervalo UTF-8 de até 2048 bytes.');
  const step=original.plans.find(p=>p.id===selected.planId&&p.taskId===selected.taskId)?.steps.find(s=>s.id===selected.stepId);if(!step||step.operation)throw Error('Escolhe um passo próprio sem operação.');
  return this.action(original.id,async()=>{
   const result=await this.transport.petRuntimeResult({...goalRefs(original),decisionId:row.spec.decisionId,requestId:`response-${row.spec.decisionId}`});
   if(result.status!=='available'||!result.receiptHash||!/^[a-f0-9]{64}$/.test(result.receiptHash))throw Error('O recibo autenticado do resultado ainda não está disponível.');const r=result.receipt;
   if(r.goalId!==original.id||r.petId!==original.spec.owner.petId||r.decisionId!==row.spec.decisionId||r.requestId!==`response-${row.spec.decisionId}`)throw Error('O resultado não corresponde ao pet, objetivo e decisão capturados.');
   const bytes=new TextEncoder().encode(r.text),byteEnd=Math.min(end,bytes.length);if(byteEnd<=start)throw Error('O intervalo não contém texto do resultado.');let text:string;try{text=new TextDecoder('utf-8',{fatal:true}).decode(bytes.slice(start,byteEnd));}catch{throw Error('Escolhe limites que não dividam um carácter UTF-8.');}
   const after:RuntimeStepInput={description:'Proposta de resposta com resultado próprio revisto',postcondition:'Ainda sem autorização de divulgação, entrega ou verificação dos critérios.',operation:{operationId:'pet.resultReply',version:1,arguments:{toPetId:source.senderPetId,source:source.reference,result:{schemaVersion:1,goalId:original.id,petId:original.spec.owner.petId,decisionId:r.decisionId,requestId:r.requestId,attemptId:r.attemptId,receiptHash:result.receiptHash,byteStart:start,byteEnd,totalBytes:bytes.length},text,expiresAt:source.deadlineAt}}};
   return this.captureNativePlan(original,[{cause:selected,after}],requestId);
  });
 }
 private nativePlanReceipt(result:RuntimeNativePlanReceipt,reference:RuntimeNativePlanReference){
  if(!sameData(result.reference,reference)||result.receipt.record.id!==reference.goalId||result.receipt.record.spec.owner.petId!==reference.petId||!result.receipt.record.nativePlanRevisions?.some(r=>sameData(r,reference))||result.executionAuthorized!==false||result.newAdmissionCreated!==false)throw Error('O recibo não corresponde à revisão original.');
  if(!result.acknowledged||uncertainReceipt(result.receipt))throw Error('A publicação do plano precisa de confirmação. Consulta a revisão pendente; nenhuma execução foi autorizada.');
 }
 async confirmNativePlan(goal:RuntimeGoal,approve:boolean){
  const saved=this.nativePlanReviews.get(goal.id);if(!saved)throw Error('Prepara uma revisão do plano antes de decidir.');const ticket=structuredClone(saved),p=ticket.proposal;
  return this.action(p.goalId,async()=>{
   const result=await this.transport.petRuntimeConfirmNativePlanRevision({petId:p.petId,goalId:p.goalId,revision:p.revision,token:ticket.token,approve});
   if(result.status==='revised'){this.nativePlanInventories.delete(p.goalId);this.nativePlanReceipt(result.receipt,p.changes);this.nativePlanReviews.delete(p.goalId);}
   else if(result.status==='declined'||!result.retained)this.nativePlanReviews.delete(p.goalId);
   return result;
  });
 }
 async ackNativePlan(goal:RuntimeGoal,reference:RuntimeNativePlanReference){
  const original=structuredClone(goal),ref=structuredClone(reference);if(ref.goalId!==original.id||ref.petId!==original.spec.owner.petId)throw Error('Escolhe uma revisão deste objetivo.');
  return this.action(original.id,async()=>{const result=await this.transport.petRuntimeAckNativePlanRevision({...goalRefs(original),revision:ref.targetRevision,reviewId:ref.id});this.nativePlanReceipt(result,ref);if(this.nativePlanReviews.get(original.id)?.proposal.changes.id===ref.id)this.nativePlanReviews.delete(original.id);this.nativePlanInventories.delete(original.id);return result;},true);
 }
 async abandonNativePlan(goal:RuntimeGoal,reference:RuntimeNativePlanReference){
  const original=structuredClone(goal),ref=structuredClone(reference);if(ref.goalId!==original.id||ref.petId!==original.spec.owner.petId)throw Error('Escolhe uma revisão deste objetivo.');
  return this.action(original.id,async()=>{const result=await this.transport.petRuntimeAbandonNativePlanRevision({...goalRefs(original),revision:original.revision,reviewId:ref.id});if(result.reference.id!==ref.id||result.reference.goalId!==original.id||result.reference.petId!==ref.petId||result.withdrawn!==true||result.executionAuthorized!==false||result.budgetRefund!==false)throw Error('A retirada não corresponde à revisão original.');if(this.nativePlanReviews.get(original.id)?.proposal.changes.id===ref.id)this.nativePlanReviews.delete(original.id);this.nativePlanInventories.delete(original.id);return result;},true);
 }

 async closeNative(goal:RuntimeGoal){const original=structuredClone(goal),row=latestAdmission(original);if(!nativeAdmission(original)||!row?.lease)throw Error('Este objetivo não tem executor nativo.');return this.action(original.id,async()=>{const result=await this.transport.petRuntimeCloseDeterministic({...goalRefs(original),revision:original.revision,leaseId:row.lease!.id});receiptGoal(result.receipt);return result;},true);}
 async verifyNative(goal:RuntimeGoal){const original=structuredClone(goal),row=latestAdmission(original);if(!row||!nativeAdmission(original)||!row.spec.nativeVerification||!row.lease)throw Error('Este objetivo não tem critérios nativos vinculados.');return this.action(goal.id,()=>this.transport.petRuntimeVerifyDeterministic({...goalRefs(original),revision:original.revision,leaseId:row.lease!.id}));}
 async claim(goal:RuntimeGoal,requestId:string){const original=structuredClone(goal);return this.action(goal.id,async()=>{
   try{await this.readiness(original);}catch(error){
    if(String(error).replace(/^Error:\s*/,'')!=='busy')throw error;
    // A prepared original claim already holds this provider slot. Metadata
    // cannot acquire it again; only resume that exact native pending request.
    const status=await this.transport.petRuntimeClaimStatus(original.spec.owner.petId,original.id),lease=status.lease,row=latestAdmission(original);
    if(status.source!=='liveController'||status.owned||!status.reservationOwned||status.record.state!=='queued'||status.record.revision!==original.revision||!lease||!row||lease.goalId!==original.id||lease.sourceRevision!==original.revision||lease.requestId!==requestId||lease.decisionId!==row?.spec.decisionId||lease.provider!==original.spec.owner.provider||lease.model!==row.spec.resolvedModel||lease.conversationId!==original.executionContext?.conversationId)throw error;
   }
   const result=await this.transport.petRuntimeClaim({...goalRefs(original),revision:original.revision,requestId});receiptGoal(result.receipt);if(!result.owned)throw Error('A posse do controlador ainda não foi confirmada. Consulta o estado antes de continuar.');if(this.drivers.get(original.id)?.leaseId!==latestAdmission(result.receipt.record)?.lease?.id)this.drivers.delete(original.id);return result;});}
 private async liveOwner(goal:RuntimeGoal,leaseId:string){
  const status=await this.transport.petRuntimeClaimStatus(goal.spec.owner.petId,goal.id);
  if(status.source!=='liveController'||!status.owned||status.lease?.id!==leaseId)throw Error('A reserva guardada não tem o controlador nativo original. Nenhum pedido de execução foi enviado; é necessária recuperação.');
  if(status.record.revision!==goal.revision)throw Error('O objetivo mudou. Atualiza e revê o trabalho antes de executar.');
 }
 async respond(goal:RuntimeGoal){
  const original=structuredClone(goal),lease=latestAdmission(original)?.lease;if(!lease)throw Error('O objetivo ainda não tem uma sessão reservada.');
  return this.action(goal.id,async()=>{await this.liveOwner(original,lease.id);const result=await this.transport.petRuntimeRespondOnce({...goalRefs(original),revision:original.revision,leaseId:lease.id,requestId:stableRequest('response',original)});this.responses.set(original.id,result.text);return result;});
 }
 async readResponse(goal:RuntimeGoal){
  const original=structuredClone(goal),row=latestAdmission(original);if(!row)throw Error('Não há uma autorização associada a este objetivo.');const ref=row.spec.resultDisclosure?.arguments.result;
  if(ref&&(ref.goalId!==original.id||ref.petId!==original.spec.owner.petId||!original.admissions?.some(r=>r.spec.decisionId===ref.decisionId&&[12,13,19].includes(r.spec.schemaVersion))))throw Error('A origem da resposta não pertence a este objetivo.');
  const result=await this.transport.petRuntimeResult({...goalRefs(original),decisionId:ref?.decisionId??row.spec.decisionId,requestId:ref?.requestId??stableRequest('response',original)});
  if(result.status==='available'){if(ref&&(result.receiptHash!==ref.receiptHash||result.receipt.goalId!==original.id||result.receipt.petId!==original.spec.owner.petId||result.receipt.decisionId!==ref.decisionId||result.receipt.requestId!==ref.requestId||result.receipt.attemptId!==ref.attemptId||new TextEncoder().encode(result.receipt.text).length!==ref.totalBytes))throw Error('A resposta não corresponde ao recibo original autenticado.');const h=result.historicalSource;if(h&&(!h.snapshotOnly||h.executionAuthorized!==false||h.postSnapshotStateUnknown!==true))throw Error('A resposta histórica não tem o âmbito de leitura esperado.');this.responses.set(original.id,(h?'Resposta histórica restaurada; não concede execução e o estado posterior ao snapshot é desconhecido.\n\n':'')+result.receipt.text);}this.changed();return result;
 }
 async requestRecipe(goal:RuntimeGoal,planId:string,criteria:RuntimeCriterionInput[]){
  const original=structuredClone(goal),bindings=structuredClone(criteria),lease=latestAdmission(original)?.lease;if(!lease)throw Error('O objetivo ainda não tem uma sessão reservada.');
  return this.action(goal.id,()=>this.transport.petRuntimeRequestRecipe({...goalRefs(original),leaseId:lease.id,revision:original.revision,planId,requestId:`recipe-${lease.id}`,criteria:bindings}));
 }
 async readRecipe(goal:RuntimeGoal,planId:string,decisionId?:string){const original=structuredClone(goal),row=decisionId?original.admissions?.find(a=>a.spec.decisionId===decisionId):latestAdmission(original);if(!row)throw Error('Não há uma autorização associada a este objetivo.');const result=await this.transport.petRuntimeRecipeProgress({...goalRefs(original),decisionId:row.spec.decisionId,planId});if(result&&(result.scope.decisionId!==row.spec.decisionId||result.scope.planId!==planId))throw Error('A receita recebida não corresponde à autorização consultada.');if(result)this.recipes.set(original.id,result);else this.recipes.delete(original.id);this.changed();return result;}
 async runRecipe(goal:RuntimeGoal,progress:RuntimeRecipeProgress){const original=structuredClone(goal),scope=structuredClone(progress.scope);return this.action(goal.id,async()=>{await this.liveOwner(original,scope.leaseId);const result=await this.transport.petRuntimeRunRecipe({...goalRefs(original),leaseId:scope.leaseId,revision:original.revision,recipeId:scope.id});this.recipes.set(original.id,result.progress);return result;});}
 async completeRecipe(goal:RuntimeGoal,progress:RuntimeRecipeProgress){const original=structuredClone(goal),scope=structuredClone(progress.scope);return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimeCompleteRecipe({...goalRefs(original),revision:original.revision,decisionId:scope.decisionId,planId:scope.planId,recipeId:scope.id})));}
 async stop(goal:RuntimeGoal,kind:'pause'|'cancel',requestId:string){const original=structuredClone(goal);return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(original),revision:original.revision,requestId,action:{kind}})),true);}
 async release(goal:RuntimeGoal){const original=structuredClone(goal),lease=latestAdmission(original)?.lease;if(!lease)throw Error('Não há uma sessão associada a este objetivo.');return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimeReleaseClaim(original.spec.owner.petId,original.id,lease.id)),true);}
 async resumeDraft(goal:RuntimeGoal,requestId:string){const original=structuredClone(goal);return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(original),revision:original.revision,requestId,action:{kind:'resumeDraft'}})));}
 async recoverStep(goal:RuntimeGoal,progress:RuntimeRecipeProgress,stepId:string){const original=structuredClone(goal),scope=structuredClone(progress.scope),step=scope.steps.find(s=>s.stepId===stepId);if(!step)throw Error('O passo não pertence à receita revista.');return this.action(goal.id,()=>this.transport.petRuntimeRecoverStep({...goalRefs(original),decisionId:scope.decisionId,planId:scope.planId,stepId,revision:original.revision,grantId:step.grantId}));}
}
export type MusicAction={action:'play'}|{action:'pause'}|{action:'set_volume';volume:number};
/** This is proposal data from explicit controls, never an execution grant. */
export function musicGoalPlan(actions:readonly MusicAction[]){
 if(!actions.length||actions.length>8)throw Error('Escolhe entre um e oito passos.');const steps:RuntimeStepInput[]=[];let status:'playing'|'paused'|undefined,volume:number|undefined;
 for(const a of actions){if(!['play','pause','set_volume'].includes(a.action))throw Error('Este passo não pertence à receita de música suportada.');if(a.action==='set_volume'){if(!Number.isInteger(a.volume)||a.volume<0||a.volume>100)throw Error('Escolhe um volume entre 0 e 100.');volume=a.volume;}else status=a.action==='play'?'playing':'paused';steps.push({description:a.action==='play'?'Reproduzir música':a.action==='pause'?'Pausar música':`Definir volume do leitor em ${a.volume}%`,postcondition:a.action==='set_volume'?`Volume do leitor observado em ${a.volume}%`:`Reprodução observada ${a.action==='play'?'a tocar':'em pausa'}`,operation:{operationId:'spotify.control',version:1,arguments:structuredClone(a)}});}
 const criteria:string[]=[],bindings:RuntimeCriterionInput[]=[];
 if(status){bindings.push({criterionIndex:criteria.length,predicate:{kind:'spotifyPlayback',status}});criteria.push(status==='playing'?'A reprodução fica a tocar.':'A reprodução fica em pausa.');}
 if(volume!==undefined){bindings.push({criterionIndex:criteria.length,predicate:{kind:'spotifyVolume',percent:volume}});criteria.push(`O volume do leitor fica em ${volume}%.`);}
 return {steps,criteria,bindings};
}
/** Only proposes predicates for literal criteria from the explicit template.
 * The native exact review still binds hashes and requires human approval. */
export function templateCriteria(goal:RuntimeGoal):RuntimeCriterionInput[]{const system=systemSoundFromGoal(goal);if(system)return system.verification;return goal.spec.criteria.flatMap<RuntimeCriterionInput>((text,criterionIndex)=>{
 if(text==='A reprodução fica a tocar.')return [{criterionIndex,predicate:{kind:'spotifyPlayback' as const,status:'playing' as const}}];
 if(text==='A reprodução fica em pausa.')return [{criterionIndex,predicate:{kind:'spotifyPlayback' as const,status:'paused' as const}}];
 const volume=/^O volume do leitor fica em (\d{1,3})%\.$/.exec(text);return volume&&Number(volume[1])<=100?[{criterionIndex,predicate:{kind:'spotifyVolume' as const,percent:Number(volume[1])}}]:[];
});}
