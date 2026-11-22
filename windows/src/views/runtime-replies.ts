import type {RuntimeMessageEnvelope,RuntimeMessageReceipt,RuntimeMessageTicket,RuntimeOperationProposal,RuntimeReceivedMessageReference,RuntimeStepInput} from '../core/bridge';
import {h} from './dom';

export interface ReplyDraftInput {petId:string;requestId:string;intent:string;criteria:string[];steps:RuntimeStepInput[]}
interface ReplyData {petId:string;receipt:RuntimeMessageReceipt;source:RuntimeReceivedMessageReference;text:string;expiresAt:number}
const token=(s:unknown)=>typeof s==='string'&&/^[0-9a-f]{32}$/.test(s);
const hash=(s:unknown)=>typeof s==='string'&&/^[0-9a-f]{64}$/.test(s);
const sameRef=(a:RuntimeReceivedMessageReference|undefined,b:RuntimeReceivedMessageReference|undefined)=>!!a&&!!b&&a.decisionId===b.decisionId&&a.messageId===b.messageId&&a.scopeHash===b.scopeHash;
const ref=(r:RuntimeReceivedMessageReference):RuntimeReceivedMessageReference=>({decisionId:r.decisionId,messageId:r.messageId,scopeHash:r.scopeHash});
/** Native inbox references are addresses, never proofs or provider context.
 * The backend resolves the canonical source again at review and publication. */
export function replyGoalPlan(input:ReplyData){
 const e=input.receipt?.scope?.envelope,r=input.source;
 if(!e||e.schemaVersion!==1||e.replyTo||!['message','request'].includes(e.kind)||e.providerUseAuthorized!==false||e.recipient.petId!==input.petId||e.origin.petId===input.petId||e.correlationId!==e.messageId||!token(r?.decisionId)||!token(r?.messageId)||!hash(r?.scopeHash)||r.messageId!==e.messageId||r.decisionId!==e.origin.decisionId)throw Error('Escolhe uma mensagem original recebida pelo pet, com referência canónica atual.');
 if(!input.text.trim()||new TextEncoder().encode(input.text).length>2048||[...input.text].some(c=>c!=='\n'&&c!=='\t'&&/\p{Cc}/u.test(c)))throw Error('Indica uma resposta literal até 2048 bytes.');
 if(!Number.isSafeInteger(input.expiresAt)||input.expiresAt<=Date.now()||input.expiresAt>e.deadlineAt)throw Error('O prazo da resposta tem de ser futuro e não pode prolongar o da mensagem original.');
 return {criteria:['A mensagem fica persistida para o destinatário indicado; não autoriza execução.'],steps:[{description:'Responder à mensagem local recebida',postcondition:'Resposta persistida na inbox; o pedido original não foi aceite ou executado.',operation:{operationId:'pet.reply',version:1,arguments:{toPetId:e.origin.petId,source:ref(r),text:input.text,expiresAt:input.expiresAt}}}] satisfies RuntimeStepInput[]};
}
export function validateReplyReview(petId:string,operation:RuntimeOperationProposal,ticket:RuntimeMessageTicket){
 const p=ticket.proposal,e=p.envelope,parent=p.receivedMessage,args=operation.arguments as Record<string,unknown>,source=args.source as RuntimeReceivedMessageReference;
 const nested=operation.version===2&&parent?.schemaVersion===7;
 // Funded replies keep reply5 while the authenticated request can carry
 // selected/iterative DATA. Match each supported source and its DATA version;
 // provenance is not new authority, and recursive reply8 remains distinct.
 const plannedVersion=({12:2,14:3,16:4} as Record<number,number>)[parent?.schemaVersion??0];
 const parentCompatible=!!parent&&(nested?parent.schemaVersion===7&&!parent.plannedData:operation.version===2?parent.kind==='delegation'&&parent.delegation?.schemaVersion===1&&(parent.schemaVersion===4?!parent.plannedData:plannedVersion!==undefined&&parent.plannedData?.schemaVersion===plannedVersion):parent.schemaVersion===1&&!parent.plannedData);
 if(operation.operationId!=='pet.reply'||![1,2].includes(operation.version)||!parent||!sameRef(source,e.replyTo)||e.schemaVersion!==(nested?8:operation.version===2?5:2)||e.kind!=='reply'||e.origin.petId!==petId||e.providerUseAuthorized!==false||p.destinationExecutionAuthorized!==false||e.recipient.petId!==args.toPetId||e.text!==args.text||e.deadlineAt!==args.expiresAt||!parentCompatible||(nested?!parent.replyTo||parent.delegation?.schemaVersion!==2:!!parent.replyTo)||parent.recipient.petId!==petId||parent.origin.petId!==e.recipient.petId||parent.messageId!==source.messageId||parent.origin.decisionId!==source.decisionId||!nested&&parent.correlationId!==parent.messageId||e.correlationId!==(nested?parent.correlationId:parent.messageId)||e.createdAt<parent.createdAt||e.deadlineAt>parent.deadlineAt||parent.providerUseAuthorized!==false)throw Error('A revisão não corresponde à resposta e à mensagem original capturadas.');
}
export function sameDelegation(a:RuntimeMessageEnvelope['delegation'],b:RuntimeMessageEnvelope['delegation']){const wire=(d:typeof a)=>d?[d.schemaVersion,d.subgoal,d.criteria,d.allowance.modelCalls,d.allowance.toolCalls,d.allowance.outputBytes,d.maxDepth,d.ancestors]:null;return JSON.stringify(wire(a))===JSON.stringify(wire(b));}
function wire(e:RuntimeMessageEnvelope){return [e.schemaVersion,e.messageId,e.origin.petId,e.origin.provider,e.origin.adapter,e.origin.conversationId,e.origin.goalId,e.origin.decisionId,e.origin.leaseId,e.origin.controllerId,e.recipient.petId,e.recipient.name,e.recipient.configurationHash,e.correlationId,e.cause.taskId,e.cause.planId,e.cause.stepId,e.kind,e.text,e.createdAt,e.deadlineAt,e.providerUseAuthorized,e.replyTo?.decisionId,e.replyTo?.messageId,e.replyTo?.scopeHash];}
function derivedWire(d:RuntimeMessageEnvelope['derivedResult']){if(!d)return null;const r=d.result,o=d.origin,v=o.inferenceDestination;return[d.schemaVersion,d.contextHash,r.schemaVersion,r.goalId,r.petId,r.decisionId,r.requestId,r.attemptId,r.receiptHash,r.byteStart,r.byteEnd,r.totalBytes,o.provider,o.adapter,o.model,o.inputHash,o.intentHash,o.completedAt,o.recordedAt,v.kind,v.provider,'endpoint'in v?v.endpoint:null,'model'in v?v.model:null];}
export function sameReplyEnvelope(a:RuntimeMessageEnvelope,b:RuntimeMessageEnvelope){try{return JSON.stringify(wire(a))===JSON.stringify(wire(b))&&JSON.stringify(a.plannedData??null)===JSON.stringify(b.plannedData??null)&&sameDelegation(a.delegation,b.delegation)&&JSON.stringify(derivedWire(a.derivedResult))===JSON.stringify(derivedWire(b.derivedResult));}catch{return false;}}

export function buildRuntimeReplyDraft(options:{petId:string;receipt:RuntimeMessageReceipt;source:RuntimeReceivedMessageReference;create:(input:ReplyDraftInput)=>Promise<unknown>;ids?:()=>string}){
 const original={petId:options.petId,receipt:structuredClone(options.receipt),source:ref(options.source)},parent=original.receipt.scope.envelope;
 let captured:ReplyDraftInput|null=null,working=false,created=false;
 const text=h('textarea',{rows:3,maxLength:2048,'aria-label':`Resposta local ${parent.messageId}`})as HTMLTextAreaElement;
 const deadline=h('input',{type:'text','aria-label':`Prazo UTC da resposta ${parent.messageId}`})as HTMLInputElement;
 deadline.value=new Date(Math.min(Date.now()+300_000,parent.deadlineAt)).toISOString();
 const status=h('p',{role:'status'}),fields=h('fieldset',{},h('legend',{text:'Rascunho de objetivo próprio'}),h('label',{},'Resposta literal',text),h('label',{},'Prazo UTC (ISO)',deadline));
 const save=h('button',{type:'button',text:'Guardar objetivo de resposta',onclick:()=>void submit()})as HTMLButtonElement;
 const el=h('details',{'aria-label':`Responder à mensagem ${parent.messageId}`},h('summary',{text:'Preparar resposta com objetivo próprio'}),h('p',{text:`${original.petId} responde a ${parent.origin.petId}. Guardar cria dados e contexto próprios; não inicia trabalho, não envia resposta e não partilha dados com um modelo.`}),h('p',{text:`Origem ${parent.messageId}. O prazo não pode ultrapassar ${new Date(parent.deadlineAt).toISOString()}.`}),fields,save,status);
 function render(){fields.disabled=working||captured!==null;save.disabled=working||created;save.textContent=created?'Objetivo de resposta guardado':captured?'Confirmar criação com os dados originais':'Guardar objetivo de resposta';}
 async function submit(){
  if(save.disabled||working||created)return;
  try{
   if(!captured){const expiresAt=Date.parse(deadline.value);if(!/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{1,3})?Z$/.test(deadline.value))throw Error('Indica um prazo UTC no formato ISO.');const plan=replyGoalPlan({...original,text:text.value,expiresAt});captured={petId:original.petId,requestId:(options.ids??(()=>crypto.randomUUID()))(),intent:`Responder à mensagem ${parent.messageId} de ${parent.origin.petId}`,criteria:plan.criteria,steps:plan.steps};}
   working=true;render();await options.create(structuredClone(captured));created=true;status.textContent='Objetivo próprio guardado. Revê o orçamento, reserva a sessão e aprova a entrega exata; ainda não foi enviada uma resposta.';
  }catch(error){status.textContent=(captured?'A criação não ficou confirmada. Consulta os objetivos; repetir conserva os dados e o identificador originais. ':'')+String(error).replace(/^Error:\s*/,'');}
  finally{working=false;render();}
 }
 return {el};
}
