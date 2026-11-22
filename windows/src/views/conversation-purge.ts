import {Bridge,type ConversationPurgeReview,type ConversationPurgeOutcome} from '../core/bridge';
import {h,clear} from './dom';
type Transport=Pick<typeof Bridge,'chatPurgeReview'|'chatPurgeCommit'|'chatPurgeCancel'|'chatPurgeRecoveryReview'>;
/** Explicit native review. Every continuation owns its original row/window. */
export function conversationPurge(options:{provider:string;conversationId:string;recoveryId?:string;current:()=>boolean;locked?:()=>boolean;done:(result:ConversationPurgeOutcome)=>void|Promise<void>;changed:()=>void;transport?:Transport}){
 const api=options.transport??Bridge,el=h('div'),status=h('p',{class:'sub',role:'status'}),details=h('div');
 let alive=true,busy=false,generation=0,review:ConversationPurgeReview|null=null,message='';
 const valid=(ticket:number)=>alive&&options.current()&&ticket===generation;
 const cancel=(r:ConversationPurgeReview|null)=>{if(r?.reviewId)void api.chatPurgeCancel(options.provider,options.conversationId,r.reviewId).catch(()=>{});};
 const permitted=()=>alive&&options.current()&&!busy&&!options.locked?.();
 function draw(){
  clear(el);clear(details);const open=h('button',{type:'button',class:'picker-link',text:options.recoveryId?'Rever recuperação…':'Eliminar histórico local…',onclick:()=>void prepare()});open.disabled=!permitted();el.append(open,status,details);status.textContent=message;status.hidden=!message;
  if(!review)return;const r=review;
  details.append(h('p',{text:r.title}),h('p',{class:'sub',text:`${r.messageCount} mensagens · ${r.taskCount} tarefas · ${r.attachmentCount} anexos referidos`}),h('p',{class:'sub',text:r.kind==='conversation'?'Eliminar apenas esta conversa da biblioteca Coucou? As cópias dos anexos e o histórico do fornecedor ficam conservados.':'Esta cópia permite recuperar a conversa no arquivo. Limpar a recuperação mantém a biblioteca atual.'}));
  const button=(label:string,consent:'deleteLocalConversation'|'deleteRecoveryCopy'|'restoreRecoveryConversation')=>{const b=h('button',{type:'button',class:'picker-link',text:label,onclick:()=>void confirm(consent,r)});b.disabled=!permitted();details.append(b);};
  if(r.kind==='conversation')button('Confirmar eliminação desta conversa','deleteLocalConversation');else{button('Restaurar conversa da recuperação','restoreRecoveryConversation');button('Eliminar apenas a cópia de recuperação','deleteRecoveryCopy');}
  const dismiss=h('button',{type:'button',class:'picker-link',text:'Cancelar revisão',onclick:()=>{if(!permitted())return;generation++;cancel(review);review=null;message='';draw();options.changed();}});dismiss.disabled=!permitted();details.append(dismiss);
 }
 async function prepare(){
  if(!permitted())return;const ticket=++generation;cancel(review);review=null;busy=true;message='A rever a operação…';draw();options.changed();
  try{const result=options.recoveryId?await api.chatPurgeRecoveryReview(options.recoveryId):await api.chatPurgeReview(options.provider,options.conversationId);
   if(!valid(ticket)){cancel(result);return;}
   if(!result||result.provider!==options.provider||result.conversationId!==options.conversationId||result.kind!==(options.recoveryId?'recovery':'conversation')||typeof result.reviewId!=='string'||!result.reviewId||typeof result.title!=='string'||result.title.length>256||![result.messageCount,result.taskCount,result.attachmentCount].every(n=>Number.isSafeInteger(n)&&n>=0)||!Number.isSafeInteger(result.expiresInSeconds)||result.expiresInSeconds<1||result.expiresInSeconds>120){cancel(result);throw Error('A revisão não corresponde à conversa escolhida.');}
   review=result;message='A revisão expira em dois minutos. Confirma a operação pretendida.';
  }catch(e){if(valid(ticket))message=String(e).replace(/^Error:\s*/,'');}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 async function confirm(consent:'deleteLocalConversation'|'deleteRecoveryCopy'|'restoreRecoveryConversation',original:ConversationPurgeReview){
  if(!permitted()||review!==original)return;const ticket=++generation;review=null;busy=true;message='A concluir a operação revista…';draw();options.changed();
  try{const result=await api.chatPurgeCommit(options.provider,options.conversationId,original.reviewId,consent);
   if(!valid(ticket))return;
   if(result?.kind==='conversation'&&original.kind==='conversation'&&result.receipt&&result.receipt.provider===options.provider&&result.receipt.removedConversationId===options.conversationId&&typeof result.receipt.durable==='boolean'&&typeof result.copySelectionCleared==='boolean'){message=result.receipt.recoveryId?'Histórico removido; existe recuperação por confirmar.':'Histórico local eliminado. As cópias dos anexos continuam guardadas.';}
   else if(result?.kind==='recovery'&&original.kind==='recovery'&&result.recoveryId===options.recoveryId&&typeof result.finished==='boolean'&&typeof result.restored==='boolean'){message=result.restored?'Conversa recuperada no arquivo.':'Cópia de recuperação eliminada; biblioteca atual mantida.';if(!result.finished)message+=' A limpeza da recuperação está pendente.';}
   else throw Error('A confirmação não corresponde à operação revista.');
   await options.done(result);
  }catch(e){if(valid(ticket))message=String(e).replace(/^Error:\s*/,'');}finally{if(valid(ticket)){busy=false;draw();options.changed();}}
 }
 draw();return{el,dispose(){alive=false;generation++;cancel(review);review=null;},refresh(){draw();}};
}
