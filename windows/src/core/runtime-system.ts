import type {RuntimeGoal,RuntimeStepInput,RuntimeCriterionInput,RuntimeAdmissionReview,RuntimeBudget,PetCapabilityDescriptor} from './bridge';
export type SystemSoundAction={kind:'volume';percent:number}|{kind:'mute';muted:boolean};
const key=(s:string)=>/^audio-[a-f0-9]{16}$/.test(s);
const same=(a:unknown,b:unknown):boolean=>{const order=(v:unknown):unknown=>Array.isArray(v)?v.map(order):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,order((v as Record<string,unknown>)[k])])):v;return JSON.stringify(order(a))===JSON.stringify(order(b));};
/** A local draft and terminal predicates, never a permission or native binding. */
export function systemSoundGoalPlan(outputKey:string,actions:readonly SystemSoundAction[]){
 if(!key(outputKey)||!actions.length||actions.length>64)throw Error('Escolhe uma saída autorizada e entre um e64 passos de Som.');
 let volume:number|undefined,muted:boolean|undefined;
 const steps:RuntimeStepInput[]=actions.map(a=>{
  if(a.kind==='volume'){if(!Number.isInteger(a.percent)||a.percent<0||a.percent>100)throw Error('Indica um volume inteiro entre0 e100%.');volume=a.percent;return {description:`Som: volume ${a.percent}% na saída ${outputKey}`,postcondition:`A saída original apresenta volume ${a.percent}%.`,operation:{operationId:'system.sound.volume',version:1,arguments:{targetKey:outputKey,percent:a.percent}}};}
  if(a.kind!=='mute'||typeof a.muted!=='boolean')throw Error('Indica explicitamente o estado de silêncio.');muted=a.muted;return {description:`Som: ${a.muted?'silenciar':'ativar som'} na saída ${outputKey}`,postcondition:`A saída original apresenta silêncio ${a.muted?'ativo':'inativo'}.`,operation:{operationId:'system.sound.mute',version:1,arguments:{targetKey:outputKey,muted:a.muted}}};
 });
 const criteria:string[]=[],verification:RuntimeCriterionInput[]=[];
 if(volume!==undefined){verification.push({criterionIndex:criteria.length,predicate:{kind:'systemSoundVolume',percent:volume}});criteria.push(`A saída ${outputKey} fica com volume ${volume}%.`);}
 if(muted!==undefined){verification.push({criterionIndex:criteria.length,predicate:{kind:'systemSoundMute',muted}});criteria.push(`A saída ${outputKey} fica com silêncio ${muted?'ativo':'inativo'}.`);}
 return {criteria,steps,verification};
}
/** A finite host observation; no model, arbitrary paths or health verdict. */
export function systemHealthGoalPlan(){return {criteria:['É recolhida uma amostra válida de CPU, memória e uptime do anfitrião autorizado.'],verification:[{criterionIndex:0,predicate:{kind:'systemHealthObserved' as const}}],steps:[{description:'Consultar três contadores de saúde do anfitrião',postcondition:'Uma amostra válida; não declara o computador saudável ou seguro.',operation:{operationId:'system.health.observe',version:1,arguments:{}}}]};}
export function systemHealthFromGoal(g:RuntimeGoal){const p=systemHealthGoalPlan();return g.plans?.length===1&&g.plans[0].steps.length===1&&same(g.plans[0].steps[0].operation,p.steps[0].operation)&&same(g.spec.criteria,p.criteria)?p:null;}
export function systemHealthBaselineGoalPlan(){return {criteria:['É registada uma medição válida de CPU e memória, com origem e intervalo de pelo menos um segundo.'],verification:[{criterionIndex:0,predicate:{kind:'systemHealthBaselineMeasured' as const}}],steps:[{description:'Medir CPU e memória durante pelo menos um segundo',postcondition:'Medição verificável ou estado indeterminado, sem diagnosticar ou alterar o sistema.',operation:{operationId:'system.health.baseline',version:1,arguments:{}}}]};}
export function systemHealthBaselineFromGoal(g:RuntimeGoal){const p=systemHealthBaselineGoalPlan();return g.plans?.length===1&&g.plans[0].steps.length===1&&same(g.plans[0].steps[0].operation,p.steps[0].operation)&&same(g.spec.criteria,p.criteria)?p:null;}
export function systemInventoryGoalPlan(){return {criteria:['É recolhido um inventário válido da distribuição e kernel do anfitrião autorizado, com origem e cobertura completas.'],verification:[{criterionIndex:0,predicate:{kind:'systemHostInventoryObserved' as const}}],steps:[{description:'Consultar distribuição e kernel autorizados',postcondition:'Inventário limitado verificável, sem diagnóstico de vulnerabilidades ou estado desejado.',operation:{operationId:'system.inventory.observe',version:1,arguments:{}}}]};}
export function systemInventoryFromGoal(g:RuntimeGoal){const p=systemInventoryGoalPlan();return g.plans?.length===1&&g.plans[0].steps.length===1&&same(g.plans[0].steps[0].operation,p.steps[0].operation)&&same(g.spec.criteria,p.criteria)?p:null;}
export const systemPlanFromGoal=(g:RuntimeGoal)=>systemInventoryFromGoal(g)??systemHealthBaselineFromGoal(g)??systemHealthFromGoal(g)??systemSoundFromGoal(g);
/** Routing of the built-in literal template only. Native Review remains authority. */
export function systemSoundFromGoal(g:RuntimeGoal){
 if(!Array.isArray(g.plans)||g.plans.length!==1)return null;let outputKey='';const actions:SystemSoundAction[]=[];
 for(const step of g.plans[0].steps){const op=step.operation,a=op?.arguments as Record<string,unknown>|undefined;
  if(!op||op.version!==1||!a||typeof a!=='object'||Array.isArray(a)||typeof a.targetKey!=='string'||!key(a.targetKey)||outputKey&&outputKey!==a.targetKey)return null;outputKey=a.targetKey;
  if(op.operationId==='system.sound.volume'&&Object.keys(a).sort().join(',')==='percent,targetKey'&&Number.isInteger(a.percent)&&Number(a.percent)>=0&&Number(a.percent)<=100)actions.push({kind:'volume',percent:Number(a.percent)});
  else if(op.operationId==='system.sound.mute'&&Object.keys(a).sort().join(',')==='muted,targetKey'&&typeof a.muted==='boolean')actions.push({kind:'mute',muted:a.muted});else return null;
 }
 try{const plan=systemSoundGoalPlan(outputKey,actions);return same(plan.criteria,g.spec.criteria)?plan:null;}catch{return null;}
}
export const systemSoundGoal=(g:RuntimeGoal)=>!!g.admissions?.at(-1)?.spec.nativeSystemPlan||!!systemPlanFromGoal(g);
export function systemSoundTaskAvailable(g:RuntimeGoal,d:PetCapabilityDescriptor|undefined){return !!d&&d.lifecycle==='active'&&g.plans.flatMap(p=>p.steps).every(s=>!!s.operation&&d.capabilities.some(c=>c.id===s.operation!.operationId&&c.support==='supported'&&c.availability!=='unavailable'&&Array.isArray(c.callers)&&c.callers.includes('taskRuntime')));}
export function validateSystemReview(r:RuntimeAdmissionReview,g:RuntimeGoal,budget:RuntimeBudget,input:readonly RuntimeCriterionInput[],requestId:string,start=false){
 if(r.status!=='review')return;const p=r.ticket.proposal,n=p.nativeSystemPlan,b=p.nativeVerification,retained=r.ticket.approvalScope==='confirmOriginalAdmission';
 if(!retained&&!same(p.executionStart,start?{schemaVersion:1,mode:'whenAvailable',scope:'nativeSystemPlan'}:undefined)||p.executionStart&&!same(p.executionStart,{schemaVersion:1,mode:'whenAvailable',scope:'nativeSystemPlan'}))throw Error('O arranque de Sistema não corresponde à escolha original.');
 if(n?.schemaVersion===2||n?.schemaVersion===3){
  const inventory=n.schemaVersion===3,h=inventory?n.inventory:n.health,host=h?.host,baseline=n.steps[0]?.operation.operationId==='system.health.baseline';
  const operation=inventory?'system.inventory.observe':baseline?'system.health.baseline':'system.health.observe',predicate=inventory?'systemHostInventoryObserved':baseline?'systemHealthBaselineMeasured':'systemHealthObserved';
  if(n.output!==undefined||(inventory?n.health!==undefined:n.inventory!==undefined)||!h||h.schemaVersion!==1||h.kind!==(inventory?'systemHostInventory':'systemHostHealth')||h.petId!==g.spec.owner.petId||!/^[a-f0-9]{32}$/.test(h.grantRevision)||!host||!/^[a-f0-9]{8}(-[a-f0-9]{4}){3}-[a-f0-9]{12}$/.test(host.bootId)||![host.pidNamespace,host.mountNamespace].every(v=>Number.isSafeInteger(v)&&v>0)||n.goalId!==g.id||p.goalId!==g.id||!same(n.owner,g.spec.owner)||!same(p.owner,g.spec.owner)||p.operation!=='goalDeterministic'||p.resolvedModel||p.engineBinding||p.memoryData||p.nativePeerPlan||p.resultDisclosure||p.workDisclosure||p.funding||p.planningPolicy||p.continuation||p.budget.payment!=='runtimeOnly'||p.budget.maxModelCalls!==0||!b||b.schemaVersion!==7||b.conditions.length!==1||b.criteriaCount!==1||b.conditions[0].predicate.kind!==predicate||n.steps.length!==1||n.steps[0].operation.operationId!==operation||n.steps[0].operation.version!==1||!same(n.steps[0].operation.arguments,{})||n.expiresAt!==p.budget.deadlineAt)throw Error('A revisão de Saúde/Inventário não corresponde ao pet, anfitrião, concessão e amostra sem modelo.');
  if(retained)return;
  const steps=g.plans.flatMap(plan=>plan.steps.map(s=>({cause:{taskId:plan.taskId,planId:plan.id,stepId:s.id},operation:s.operation})));
  const bounds=(v:RuntimeBudget)=>({...v,maxTokens:v.maxTokens??null,maxCostMicros:v.maxCostMicros??null});
  if(!(inventory?systemInventoryFromGoal(g):baseline?systemHealthBaselineFromGoal(g):systemHealthFromGoal(g))||p.revision!==g.revision||p.requestId!==requestId||n.sourceRevision!==g.revision||!same(p.context,g.executionContext)||p.intent!==g.spec.intent||!same(p.criteria,g.spec.criteria)||!same(n.steps,steps)||!same(b.conditions.map(({criterionIndex,predicate})=>({criterionIndex,predicate})),input)||!same(bounds(p.budget),bounds(budget)))throw Error('A revisão alterou o objetivo, critérios ou orçamento de Saúde/Inventário.');
  return;
 }

 if(!n||n.schemaVersion!==1||n.goalId!==g.id||p.goalId!==g.id||n.owner.petId!==g.spec.owner.petId||n.output.petId!==g.spec.owner.petId||p.owner.petId!==g.spec.owner.petId||p.operation!=='goalDeterministic'||p.resolvedModel||p.engineBinding||p.memoryData||p.nativePeerPlan||p.resultDisclosure||p.workDisclosure||p.funding||p.planningPolicy||p.continuation||p.budget.payment!=='runtimeOnly'||p.budget.maxModelCalls!==0||!b||b.schemaVersion!==5||b.conditions.length!==p.criteria.length||b.criteriaCount!==p.criteria.length||!key(n.output.outputKey)||!/^[a-f0-9]{32}$/.test(n.output.grantRevision)||!Number.isInteger(n.output.incarnation.coreCookie)||n.output.incarnation.coreCookie<0||n.output.incarnation.coreCookie>0xffffffff||!Number.isInteger(n.output.incarnation.nodeId)||n.output.incarnation.nodeId<=0||!/^\d{1,24}$/.test(n.output.incarnation.nodeSerial)||n.expiresAt!==p.budget.deadlineAt)throw Error('A revisão de Sistema não corresponde a uma tarefa local sem modelo, concessão e saída exatas.');
 if(retained)return;
 const steps=g.plans.flatMap(plan=>plan.steps.map(s=>({cause:{taskId:plan.taskId,planId:plan.id,stepId:s.id},operation:s.operation})));
 const bounds=(b:RuntimeBudget)=>({...b,maxTokens:b.maxTokens??null,maxCostMicros:b.maxCostMicros??null});
 if(p.revision!==g.revision||p.requestId!==requestId||n.sourceRevision!==g.revision||!same(n.owner,g.spec.owner)||!same(p.owner,g.spec.owner)||!same(p.context,g.executionContext)||p.intent!==g.spec.intent||!same(p.criteria,g.spec.criteria)||!same(n.steps,steps)||n.steps.some(s=>(s.operation.arguments as Record<string,unknown>).targetKey!==n.output.outputKey)||!same(b.conditions.map(({criterionIndex,predicate})=>({criterionIndex,predicate})),input)||!same(bounds(p.budget),bounds(budget)))throw Error('A revisão alterou o objetivo, operações, critérios, saída ou orçamento escolhidos.');
}
