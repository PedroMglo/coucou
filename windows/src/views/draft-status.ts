import {h} from './dom';
import {sessionTargetKey} from '../core/session-chat';
import {DRAFT_LIMITS,type DraftTarget,type SessionDrafts} from '../core/session-drafts';
import {draftRecoveryControl} from './draft-recovery';
import {draftReconcileControl} from './draft-reconcile';
/** Shared functional controls. Captured key/context/revision are rechecked on
 * every action; previews contain text only and never change a stored version. */
export function draftStatusControl(options:{resource:SessionDrafts;target:()=>DraftTarget|null;context:()=>unknown;id?:string;changed?:()=>void}){
 const el=h('section',{id:options.id??'draft-persistence',class:'composer-hint','aria-label':'Estado do rascunho desta conversa'});el.hidden=true;
 const message=h('p',{role:'status'}),actions=h('div',{class:'chip-row'});
 const retry=h('button',{type:'button',class:'picker-link',text:'Tentar guardar novamente'}),use=h('button',{type:'button',class:'picker-link',text:'Usar o guardado'}),keep=h('button',{type:'button',class:'picker-link',text:'Guardar este texto'});actions.append(retry,use,keep);
 const text=h('pre',{class:'failed-request-text'}),preview=h('details',{class:'failed-request-review'},h('summary',{text:'Rever rascunho guardado'}),text);el.append(message,actions,preview);
 const recovery=draftRecoveryControl(options);el.append(recovery.el);
 const reconcile=draftReconcileControl(options);el.append(reconcile.el);
 function sync(){
  recovery.sync();
  reconcile.sync();
  const target=options.target(),status=options.resource.status(target);el.hidden=!target||!status;if(!target||!status)return;
  el.dataset.phase=status.phase;el.setAttribute('aria-busy',String(status.busy||status.phase==='saving'));
  message.textContent=status.phase==='loading'?'A recuperar o rascunho…':status.phase==='saving'?'A guardar o rascunho…':status.phase==='restored'?'Rascunho recuperado neste dispositivo. Revê a conversa antes de enviar.':status.phase==='saved'?status.localText?'Rascunho guardado neste dispositivo.':'Sem rascunho guardado.':status.phase==='conflict'?'Há outro rascunho guardado para esta conversa. O texto desta janela foi preservado.':status.error==='limit'?status.localText.length>DRAFT_LIMITS.text?'O texto excede o limite do rascunho guardado. O texto completo mantém-se nesta janela.':'O limite do armazenamento de rascunhos foi atingido. O texto mantém-se nesta janela.':status.error==='blocked'?'O armazenamento de rascunhos está ocupado. O texto mantém-se nesta janela.':'Não foi possível guardar o rascunho. O texto mantém-se nesta janela.';
  if(status.error==='migrationConflict')message.textContent='Os rascunhos anteriores e os nativos divergem. Ambos foram preservados; a migração precisa de reconciliação antes de guardar. O texto mantém-se nesta janela.';
  if(status.error==='unsupported')message.textContent='Neste dispositivo, o texto ainda não pode ser guardado antes de escolher uma conversa. Mantém-se nesta janela; copia-o explicitamente para a conversa identificada.';
  if(status.phase==='conflict'&&status.error)message.textContent+=' Não foi possível reler o rascunho guardado. Tenta novamente.';
  retry.hidden=status.phase!=='error';use.hidden=keep.hidden=status.phase!=='conflict';use.disabled=keep.disabled=status.busy;preview.hidden=status.phase!=='conflict';text.textContent=status.storedText||'O rascunho guardado está vazio.';
  const exact={...target},key=sessionTargetKey(exact),context=options.context(),revision=status.revision;
  const current=()=>options.context()===context&&!!options.target()&&sessionTargetKey(options.target()!)===key;
  retry.onclick=()=>{if(current())void options.resource.retry(exact);};use.onclick=()=>{if(current())void options.resource.useStored(exact,revision);};keep.onclick=()=>{if(current())options.resource.keepLocal(exact,revision);};options.changed?.();
 }
 return{el,sync};
}
