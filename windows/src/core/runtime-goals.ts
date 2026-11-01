import {Bridge,type RuntimeGoal,type RuntimeFuturePrepared,type RuntimeReceipt,type RuntimeCreateInput,type RuntimeStepInput,type RuntimeBudget,type RuntimeAdmissionReview,type RuntimeAdmissionResult,type RuntimeCriterionInput,type RuntimeRecipeProgress,type RuntimePlanningLimits,type RuntimeDriverStatus,type RuntimeRecoveryPrepared,type RuntimeHistorySnapshot,type RuntimeRetirementPrepared,type PetMemorySelected,type PetMemoryGoalConsent} from './bridge';

export const goalRefs=(goal:RuntimeGoal)=>({petId:goal.spec.owner.petId,goalId:goal.id});
export const latestAdmission=(goal:RuntimeGoal)=>goal.admissions?.at(-1);
export const uncertainReceipt=(receipt:RuntimeReceipt)=>receipt.status==='uncertain';
const errorText=(e:unknown)=>String(e).replace(/^Error:\s*/,'');
export const goalStateLabel=(state:RuntimeGoal['state'])=>({draft:'Rascunho',paused:'Pausado',cancelled:'Cancelado',recoveryRequired:'Requer recuperação',queued:'Autorizado na fila',leased:'Sessão reservada',stopping:'A interromper',completed:'Concluído com verificação nativa'})[state];
const receiptGoal=(receipt:RuntimeReceipt)=>{if(uncertainReceipt(receipt))throw Error('A gravação precisa de confirmação. Atualiza o objetivo antes de continuar; nenhuma ação seguinte foi enviada.');return receipt.record;};
const stableRequest=(prefix:string,goal:RuntimeGoal)=>`${prefix}-${latestAdmission(goal)?.spec.decisionId??goal.id}`;
export type GoalTransport=Pick<typeof Bridge,'petRuntimePrepareRestoredFuture'|'petRuntimeConfirmRestoredFuture'|'petRuntimeHistory'|'petRuntimePrepareRetirement'|'petRuntimeConfirmRetirement'|'petRuntimePrepareContinuation'|'petRuntimePrepareRecovery'|'petRuntimeConfirmRecovery'|'petRuntimePreparePlanning'|'petRuntimeStartDriver'|'petRuntimeDriverStatus'|'petRuntimeCreate'|'petRuntimeMutate'|'petRuntimePrepareContext'|'petRuntimeRefreshReadiness'|'petRuntimePrepareAdmission'|'petRuntimeConfirmAdmission'|'petRuntimeClaim'|'petRuntimeClaimStatus'|'petRuntimeReleaseClaim'|'petRuntimeRespondOnce'|'petRuntimeRequestRecipe'|'petRuntimeRunRecipe'|'petRuntimeRecipeProgress'|'petRuntimeResult'|'petRuntimeCompleteRecipe'|'petRuntimeRecoverStep'>;
/** User-driven native transactions. Opening/refreshing this controller performs
 * no IO. Failed/uncertain actions never schedule another write or effect. */
export class RuntimeGoalActions{
 readonly pending=new Set<string>();readonly controls=new Set<string>();readonly errors=new Map<string,string>();
 readonly drivers=new Map<string,RuntimeDriverStatus>();
 readonly recoveryReviews=new Map<string,RuntimeRecoveryPrepared>();
 readonly futureReviews=new Map<string,RuntimeFuturePrepared>();private futureChoices=new Set<string>();
 readonly histories=new Map<string,RuntimeHistorySnapshot>();readonly retirementReviews=new Map<string,RuntimeRetirementPrepared>();
 readonly reviews=new Map<string,RuntimeAdmissionReview>();readonly responses=new Map<string,string>();readonly recipes=new Map<string,RuntimeRecipeProgress>();
 private transport:GoalTransport;private changed:()=>void;private admissionChoices=new Map<string,PetMemoryGoalConsent|undefined>();
 constructor(changed:()=>void=()=>{},transport:GoalTransport=Bridge){this.changed=changed;this.transport=transport;}
 private async action<T>(key:string,operation:()=>Promise<T>,control=false):Promise<T>{
  const group=control?this.controls:this.pending;if(group.has(key))throw Error('Este pedido já está em curso.');group.add(key);this.errors.delete(key);
  try{this.changed();return await operation();}catch(e){this.errors.set(key,errorText(e));throw e;}finally{group.delete(key);this.changed();}
 }
 async create(input:RuntimeCreateInput,steps:RuntimeStepInput[],selectedMemory?:PetMemorySelected[]):Promise<RuntimeGoal>{
  const original=structuredClone(input),proposed=structuredClone(steps),memory=structuredClone(selectedMemory);
  return this.action('create',async()=>{
   let goal=receiptGoal(await this.transport.petRuntimeCreate(original));
   goal=receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-task`,action:{kind:'proposeTask',description:proposed.some(s=>s.operation)?'Executar a sequência proposta':'Preparar uma resposta ao objetivo'}}));
   const task=goal.tasks.at(-1);if(!task)throw Error('A tarefa não foi confirmada. Atualiza o objetivo.');
   goal=receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-plan`,action:{kind:'proposePlan',taskId:task.id,steps:proposed}}));
   return receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-context`,mode:'dedicated',...(memory?.length?{selectedMemory:memory}:{})}));
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
 async prepareRecovery(goal:RuntimeGoal,requestId=stableRequest('close-lost',goal)){
  const original=structuredClone(goal);return this.action(goal.id,async()=>{const reply=await this.transport.petRuntimePrepareRecovery({...goalRefs(original),revision:original.revision,requestId});if(reply.status==='closed'&&uncertainReceipt(reply.receipt)){this.recoveryReviews.delete(original.id);throw Error('O fecho precisa de confirmação. Volta a consultar esta operação.');}this.recoveryReviews.set(original.id,reply);return reply;});
 }
 async confirmRecovery(goal:RuntimeGoal,approve:boolean){
  const review=this.recoveryReviews.get(goal.id);if(review?.status!=='review')throw Error('Revê o fecho da execução antiga antes de confirmar.');const ticket=structuredClone(review.ticket);
  return this.action(goal.id,async()=>{const reply=await this.transport.petRuntimeConfirmRecovery({petId:ticket.proposal.petId,goalId:ticket.proposal.goalId,revision:ticket.proposal.revision,token:ticket.token,approve});
   if(reply.status==='closed'){if(uncertainReceipt(reply.receipt))throw Error('O fecho precisa de confirmação. Repete esta confirmação; o trabalho não será reiniciado.');this.recoveryReviews.delete(goal.id);}
   else if(reply.status==='declined'||reply.status==='blocked'&&reply.retained===false)this.recoveryReviews.delete(goal.id);return reply;});
 }
 async prepareContext(goal:RuntimeGoal,requestId:string,selectedMemory?:PetMemorySelected[]){const original=structuredClone(goal),memory=structuredClone(selectedMemory);return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(original),revision:original.revision,requestId,mode:'dedicated',...(memory?.length?{selectedMemory:memory}:{})})));}
 private async readiness(goal:RuntimeGoal){if(['ollama','lmstudio'].includes(goal.spec.owner.adapter))await this.transport.petRuntimeRefreshReadiness(goal.spec.owner.petId,goal.id);}
 async prepareAdmission(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string){
  const original=structuredClone(goal),limits=structuredClone(budget);
  return this.action(goal.id,async()=>{await this.readiness(original);const review=await this.transport.petRuntimePrepareAdmission({...goalRefs(original),revision:original.revision,requestId,budget:limits});this.reviews.set(original.id,review);return review;});
 }
 async preparePlanning(goal:RuntimeGoal,budget:RuntimeBudget,limits:RuntimePlanningLimits,requestId:string){
  const original=structuredClone(goal),bounds=structuredClone(budget),planning=structuredClone(limits);
  return this.action(goal.id,async()=>{await this.readiness(original);const review=await this.transport.petRuntimePreparePlanning({...goalRefs(original),revision:original.revision,requestId,budget:bounds,limits:planning});this.reviews.set(original.id,review);return review;});
 }
 async prepareContinuation(goal:RuntimeGoal,budget:RuntimeBudget,limits:RuntimePlanningLimits|null,requestId:string,reviewUncertainEffects=false){
  const original=structuredClone(goal),bounds=structuredClone(budget),planning=structuredClone(limits);
  return this.action(goal.id,async()=>{await this.readiness(original);const review=await this.transport.petRuntimePrepareContinuation({...goalRefs(original),revision:original.revision,requestId,budget:bounds,limits:planning,...(reviewUncertainEffects?{reviewUncertainEffects:true}:{})});this.reviews.set(original.id,review);return review;});
 }
 async startDriver(goal:RuntimeGoal){
  const original=structuredClone(goal),row=latestAdmission(original),lease=row?.lease;if(!lease||row.spec.schemaVersion!==3||!row.spec.planningPolicy)throw Error('Revê e autoriza primeiro o planeamento limitado deste objetivo.');
  return this.action(goal.id,async()=>{await this.liveOwner(original,lease.id);const status=await this.transport.petRuntimeStartDriver({...goalRefs(original),leaseId:lease.id});this.drivers.set(original.id,status);return status;});
 }
 async retryClosure(goal:RuntimeGoal){
  const original=structuredClone(goal),lease=latestAdmission(original)?.lease;if(!lease)throw Error('O objetivo não tem uma referência de sessão.');
  return this.action(original.id,async()=>{const input={...goalRefs(original),leaseId:lease.id};const current=await this.transport.petRuntimeDriverStatus(input);if(!current.retainedClosure)throw Error('Não existe uma prova nativa de fecho retida.');const status=await this.transport.petRuntimeStartDriver(input);this.drivers.set(original.id,status);return status;});
 }
 async readDriver(goal:RuntimeGoal){const lease=latestAdmission(goal)?.lease;if(!lease)throw Error('O objetivo ainda não tem uma sessão reservada.');const status=await this.transport.petRuntimeDriverStatus({...goalRefs(goal),leaseId:lease.id});this.drivers.set(goal.id,status);this.changed();return status;}
 async confirmAdmission(goal:RuntimeGoal,approve:boolean,memoryConsent?:PetMemoryGoalConsent):Promise<RuntimeAdmissionResult>{
  const review=this.reviews.get(goal.id);if(review?.status!=='review')throw Error('Prepara uma revisão antes de decidir.');const ticket=structuredClone(review.ticket),p=ticket.proposal;
  // Validate the separate, explicit data scope before metadata or authority IO.
  // A retained acknowledgement reuses this choice; it cannot renew its scope.
  if(approve&&(p.memoryData?memoryConsent!==p.memoryData.requiredConsent:memoryConsent!==undefined))throw Error('Revê e autoriza explicitamente o âmbito de utilização dos dados deste objetivo.');
  if(approve&&this.admissionChoices.has(ticket.token)&&this.admissionChoices.get(ticket.token)!==memoryConsent)throw Error('A confirmação tem de conservar o consentimento original.');
  return this.action(p.goalId,async()=>{
   if(approve&&!this.admissionChoices.has(ticket.token)){
    if(['draft','recoveryRequired'].includes(goal.state)&&['ollama','lmstudio'].includes(p.owner.adapter))await this.transport.petRuntimeRefreshReadiness(p.owner.petId,p.goalId);
    this.admissionChoices.set(ticket.token,memoryConsent);
   }
   const result=await this.transport.petRuntimeConfirmAdmission({petId:p.owner.petId,goalId:p.goalId,revision:p.revision,token:ticket.token,approve,...(approve&&memoryConsent?{memoryConsent}:{})});
   const retained=(result.status==='queued'&&uncertainReceipt(result.receipt))||((result.status==='blocked'||result.status==='authorityBlocked')&&result.retryable===true)||(result.status==='blocked'&&result.reason==='dataConsentRequired');
   if(!retained){this.reviews.delete(p.goalId);this.admissionChoices.delete(ticket.token);}if(result.status==='queued')receiptGoal(result.receipt);return result;
  });
 }

 async claim(goal:RuntimeGoal,requestId:string){const original=structuredClone(goal);return this.action(goal.id,async()=>{await this.readiness(original);const result=await this.transport.petRuntimeClaim({...goalRefs(original),revision:original.revision,requestId});receiptGoal(result.receipt);if(!result.owned)throw Error('A posse do controlador ainda não foi confirmada. Consulta o estado antes de continuar.');return result;});}
 private async liveOwner(goal:RuntimeGoal,leaseId:string){
  const status=await this.transport.petRuntimeClaimStatus(goal.spec.owner.petId,goal.id);
  if(status.source!=='liveController'||!status.owned||status.lease?.id!==leaseId)throw Error('A reserva guardada não tem o controlador nativo original. Nenhum pedido de execução foi enviado; é necessária recuperação.');
  if(status.record.revision!==goal.revision)throw Error('O objetivo mudou. Atualiza e revê o trabalho antes de executar.');
 }
 async respond(goal:RuntimeGoal){
  const original=structuredClone(goal),lease=latestAdmission(original)?.lease;if(!lease)throw Error('O objetivo ainda não tem uma sessão reservada.');
  return this.action(goal.id,async()=>{await this.liveOwner(original,lease.id);const result=await this.transport.petRuntimeRespondOnce({...goalRefs(original),revision:original.revision,leaseId:lease.id,requestId:stableRequest('response',original)});this.responses.set(original.id,result.text);return result;});
 }
 async readResponse(goal:RuntimeGoal){const row=latestAdmission(goal);if(!row)throw Error('Não há uma autorização associada a este objetivo.');const result=await this.transport.petRuntimeResult({...goalRefs(goal),decisionId:row.spec.decisionId,requestId:stableRequest('response',goal)});if(result.status==='available')this.responses.set(goal.id,result.receipt.text);this.changed();return result;}
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
export function templateCriteria(goal:RuntimeGoal):RuntimeCriterionInput[]{return goal.spec.criteria.flatMap<RuntimeCriterionInput>((text,criterionIndex)=>{
 if(text==='A reprodução fica a tocar.')return [{criterionIndex,predicate:{kind:'spotifyPlayback' as const,status:'playing' as const}}];
 if(text==='A reprodução fica em pausa.')return [{criterionIndex,predicate:{kind:'spotifyPlayback' as const,status:'paused' as const}}];
 const volume=/^O volume do leitor fica em (\d{1,3})%\.$/.exec(text);return volume&&Number(volume[1])<=100?[{criterionIndex,predicate:{kind:'spotifyVolume' as const,percent:Number(volume[1])}}]:[];
});}
