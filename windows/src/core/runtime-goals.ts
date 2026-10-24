import {Bridge,type RuntimeGoal,type RuntimeReceipt,type RuntimeCreateInput,type RuntimeStepInput,type RuntimeBudget,type RuntimeAdmissionReview,type RuntimeAdmissionResult,type RuntimeCriterionInput,type RuntimeRecipeProgress} from './bridge';

export const goalRefs=(goal:RuntimeGoal)=>({petId:goal.spec.owner.petId,goalId:goal.id});
export const latestAdmission=(goal:RuntimeGoal)=>goal.admissions?.at(-1);
export const uncertainReceipt=(receipt:RuntimeReceipt)=>receipt.status==='uncertain';
const errorText=(e:unknown)=>String(e).replace(/^Error:\s*/,'');
export const goalStateLabel=(state:RuntimeGoal['state'])=>({draft:'Rascunho',paused:'Pausado',cancelled:'Cancelado',recoveryRequired:'Requer recuperação',queued:'Autorizado na fila',leased:'Sessão reservada',stopping:'A interromper',completed:'Concluído com verificação nativa'})[state];
const receiptGoal=(receipt:RuntimeReceipt)=>{if(uncertainReceipt(receipt))throw Error('A gravação precisa de confirmação. Atualiza o objetivo antes de continuar; nenhuma ação seguinte foi enviada.');return receipt.record;};
const stableRequest=(prefix:string,goal:RuntimeGoal)=>`${prefix}-${latestAdmission(goal)?.spec.decisionId??goal.id}`;
export type GoalTransport=Pick<typeof Bridge,'petRuntimeCreate'|'petRuntimeMutate'|'petRuntimePrepareContext'|'petRuntimeRefreshReadiness'|'petRuntimePrepareAdmission'|'petRuntimeConfirmAdmission'|'petRuntimeClaim'|'petRuntimeClaimStatus'|'petRuntimeReleaseClaim'|'petRuntimeRespondOnce'|'petRuntimeRequestRecipe'|'petRuntimeRunRecipe'|'petRuntimeRecipeProgress'|'petRuntimeResult'|'petRuntimeCompleteRecipe'|'petRuntimeRecoverStep'>;
/** User-driven native transactions. Opening/refreshing this controller performs
 * no IO. Failed/uncertain actions never schedule another write or effect. */
export class RuntimeGoalActions{
 readonly pending=new Set<string>();readonly controls=new Set<string>();readonly errors=new Map<string,string>();
 readonly reviews=new Map<string,RuntimeAdmissionReview>();readonly responses=new Map<string,string>();readonly recipes=new Map<string,RuntimeRecipeProgress>();
 private transport:GoalTransport;private changed:()=>void;
 constructor(changed:()=>void=()=>{},transport:GoalTransport=Bridge){this.changed=changed;this.transport=transport;}
 private async action<T>(key:string,operation:()=>Promise<T>,control=false):Promise<T>{
  const group=control?this.controls:this.pending;if(group.has(key))throw Error('Este pedido já está em curso.');group.add(key);this.errors.delete(key);
  try{this.changed();return await operation();}catch(e){this.errors.set(key,errorText(e));throw e;}finally{group.delete(key);this.changed();}
 }
 async create(input:RuntimeCreateInput,steps:RuntimeStepInput[]):Promise<RuntimeGoal>{
  const original=structuredClone(input),proposed=structuredClone(steps);
  return this.action('create',async()=>{
   let goal=receiptGoal(await this.transport.petRuntimeCreate(original));
   goal=receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-task`,action:{kind:'proposeTask',description:proposed.some(s=>s.operation)?'Executar a sequência proposta':'Preparar uma resposta ao objetivo'}}));
   const task=goal.tasks.at(-1);if(!task)throw Error('A tarefa não foi confirmada. Atualiza o objetivo.');
   goal=receiptGoal(await this.transport.petRuntimeMutate({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-plan`,action:{kind:'proposePlan',taskId:task.id,steps:proposed}}));
   return receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(goal),revision:goal.revision,requestId:`${original.requestId}-context`,mode:'dedicated'}));
  });
 }
 async prepareContext(goal:RuntimeGoal,requestId:string){const original=structuredClone(goal);return this.action(goal.id,async()=>receiptGoal(await this.transport.petRuntimePrepareContext({...goalRefs(original),revision:original.revision,requestId,mode:'dedicated'})));}
 private async readiness(goal:RuntimeGoal){if(goal.spec.owner.adapter==='ollama')await this.transport.petRuntimeRefreshReadiness(goal.spec.owner.petId,goal.id);}
 async prepareAdmission(goal:RuntimeGoal,budget:RuntimeBudget,requestId:string){
  const original=structuredClone(goal),limits=structuredClone(budget);
  return this.action(goal.id,async()=>{await this.readiness(original);const review=await this.transport.petRuntimePrepareAdmission({...goalRefs(original),revision:original.revision,requestId,budget:limits});this.reviews.set(original.id,review);return review;});
 }
 async confirmAdmission(goal:RuntimeGoal,approve:boolean):Promise<RuntimeAdmissionResult>{
  const review=this.reviews.get(goal.id);if(review?.status!=='review')throw Error('Prepara uma revisão antes de decidir.');const ticket=structuredClone(review.ticket),p=ticket.proposal;
  return this.action(goal.id,async()=>{if(approve)await this.readiness(goal);let result:RuntimeAdmissionResult;try{result=await this.transport.petRuntimeConfirmAdmission({petId:p.owner.petId,goalId:p.goalId,revision:p.revision,token:ticket.token,approve});}finally{if(!approve)this.reviews.delete(goal.id);}this.reviews.delete(goal.id);if(result.status==='queued')receiptGoal(result.receipt);return result;});
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
 async readRecipe(goal:RuntimeGoal,planId:string){const row=latestAdmission(goal);if(!row)throw Error('Não há uma autorização associada a este objetivo.');const result=await this.transport.petRuntimeRecipeProgress({...goalRefs(goal),decisionId:row.spec.decisionId,planId});if(result)this.recipes.set(goal.id,result);else this.recipes.delete(goal.id);this.changed();return result;}
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
