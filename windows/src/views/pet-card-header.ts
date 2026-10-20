import {Bridge} from '../core/bridge';
import {State} from '../core/state';
import {providerForPill} from '../core/providers';
import {dialogueId} from '../core/chat-dialogue';
import {h,svg} from './dom';
import {ICONS} from './icons';
import type {ViewActions,ViewHost} from './views';

/** Context for one pet/request. The shared centre owns home and management. */
export function buildPetCardHeader(actions:ViewActions):ViewHost {
 let pending=false;
 const owner=h('strong',{class:'pet-card-owner'}),error=h('span',{class:'pet-card-header-error',role:'status'});
 const button=(label:string,icon:string,run:()=>void)=>h('button',{type:'button',title:label,'aria-label':label,onclick:run},svg(icon,14));
 const context=()=>JSON.stringify([State.focusId,State.view,State.chatProvider,dialogueId(State.chatProvider),State.focusTask?.sessionId,State.fileFor(State.chatProvider)?.sequence,State.lastActivity]);
 const open=async(run:()=>Promise<unknown>)=>{
  if(pending)return;pending=true;for(const button of [chat,sessions,profile])button.disabled=true;
  const original=context();
  error.textContent='';
  try{const shown=await run();if(shown!==false&&State.mode==='expanded'&&context()===original)actions.collapse();}catch(cause){if(context()===original)error.textContent=String(cause).replace(/^Error:\s*/,'');}
  finally{pending=false;for(const button of [chat,sessions,profile])button.disabled=false;}
 };
 const chat=button('Conversar com este pet',ICONS.bubble,()=>{
  const task=State.focusTask;if(!task)return;
  const session=State.view!=='prompt'&&task.sessionId&&task.sessionBackend?{id:task.sessionId,backend:task.sessionBackend,title:task.sessionTitle??'',cwd:task.sessionCwd??'',steps:task.steps.slice(-64),finalLine:task.finalLine??''}:undefined;
  void open(()=>Bridge.petChatOpen(task.id,session));
 });
 const sessions=button('Sessões deste pet',ICONS.stack,()=>void open(()=>Bridge.petPanelOpen(360,50,State.focusId??undefined,'sessions')));
 const attach=button('Anexar documento a este pet',ICONS.plus,()=>actions.setView('upload'));
 const profile=button('Configurar este pet',ICONS.gear,()=>void open(()=>Bridge.petPanelOpen(360,50,State.focusId??undefined,'profiles')));
 const close=button('Fechar cartão',ICONS.xmark,()=>actions.collapse());
 const el=h('div',{id:'header',class:'pet-card-header'},owner,error,h('div',{class:'header-actions'},chat,sessions,attach,profile,close));
 return{el,sync(){
  const task=State.focusTask,pending=State.pendingApproval?.pillId===task?.id;
  owner.textContent=task?.name??'Pedido do pet';owner.style.color=task?.color??'';
  owner.title=task?.sessionTitle||task?.sessionCwd||'';
  chat.hidden=attach.hidden=!task||!providerForPill(task.id)||pending;
  sessions.hidden=!task||task.id==='integration_spotify';profile.hidden=!task;
 }};
}
