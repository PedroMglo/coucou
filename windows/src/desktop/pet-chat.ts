import {attachmentContext,receiveDocumentDrop,documentBatchMessage} from '../core/document-collection';
import {IndexedDraftStorage,SessionDrafts,draftBroadcast,DRAFT_LIMITS} from '../core/session-drafts';
import {requestHistoryFor,containsPendingRequest} from '../core/request-history';
import '../style.css';import './pet-chat.css';
import {invoke} from '@tauri-apps/api/core';
import {Bridge,onEvent,onDragDrop,type ConversationView} from '../core/bridge';
import {State} from '../core/state';
import {buildPrompt} from '../views/chat';
import {PetPreferences} from '../mochi/pet-preferences';
import {SessionChat,sameSession,sessionOrigin,sessionTargetKey,type SessionTarget,type SessionSnapshot,type SessionSummary} from '../core/session-chat';
import {matchingSessions,mergeSessionLists,visibleSessions} from './session-list';
import {SessionListLoading,type ListSource} from './session-list-loading';
import {OllamaModelSelection} from '../core/ollama-selection';
import type {Settings} from '../core/state';
import {managedReference,managedFor,managedDescription,canOpenManaged,type ManagedCodexSession} from '../core/managed-codex';
interface Session{id:string;backend:string;title:string;cwd:string;steps:string[];finalLine:string}
interface Context{pet:string;name:string;provider:string;session?:Session;conversationId?:string;showSessions?:boolean;target?:SessionTarget}
interface Rect{x:number;y:number;width:number;height:number}
let context:Context|null=null,rect:Rect={x:100,y:80,width:760,height:620},large=false,opening=0,selectionIntent=0;
function acceptRect(value:Rect){rect=value;large=rect.width>760||rect.height>620;}
let dragging=false,dragState:{id:number;origin:Rect;pointer:{x:number;y:number}|null}|null=null;
const node=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const e=document.createElement(tag);if(text!==undefined)e.textContent=text;return e;};
const info=document.getElementById('feedback')!;
// Sidebar actions must report errors even when the narrow layout hides chat-main.
document.getElementById('chat-layout')!.before(info);
const list=document.getElementById('session-list')!,search=document.getElementById('session-search') as HTMLInputElement;
const newSession=document.getElementById('new-session') as HTMLButtonElement;
const newCodex=document.getElementById('new-codex') as HTMLButtonElement;
const managedPanel=document.getElementById('managed-session')!,managedOpen=document.getElementById('managed-open') as HTMLButtonElement,managedRelease=document.getElementById('managed-release') as HTMLButtonElement;
let managed:ManagedCodexSession[]=[],managedAction=false,managedReading=0,creatingCodex=false;
let summaries:SessionSummary[]=[];
let sessionLimits={mochi:50,external:50};
const local=(c:{pet:string;provider:string},id:string):SessionTarget=>({petId:c.pet,provider:c.provider,origin:'mochi',sessionId:id,conversationId:id});
function fromLocal(target:SessionTarget,value:ConversationView):SessionSnapshot {
 if(value.id!==target.sessionId)throw new Error('A conversa recebida não corresponde à sessão escolhida.');
 const messages=value.messages.map((m,i)=>({id:`${value.id}:${i}`,role:m.role,content:m.content}));
 if(value.busy&&value.query&&!containsPendingRequest(requestHistoryFor(target,value.requestHistory,value.messages.length),value.query))messages.push({id:`${value.id}:query`,role:'user',content:value.query});
 if(value.busy&&value.stream)messages.push({id:`${value.id}:stream`,role:'assistant',content:value.stream});
 return {target,messages,notice:value.notice,taskNotice:value.taskNotice,requestHistory:value.requestHistory,attachmentHistory:value.attachmentHistory,localExecution:value.localExecution,busy:!!value.busy,turn:value.turn===undefined?undefined:String(value.turn),capabilities:{history:true,send:true,cancel:true}};
}
const draftStatus=node('section');draftStatus.id='draft-persistence';draftStatus.className='composer-hint';draftStatus.setAttribute('aria-label','Estado do rascunho desta conversa');draftStatus.hidden=true;
const draftMessage=node('p');draftMessage.setAttribute('role','status');
const draftActions=node('div');draftActions.className='chip-row';
const draftRetry=node('button','Tentar guardar novamente'),draftUse=node('button','Usar o guardado'),draftKeep=node('button','Guardar este texto');
for(const b of [draftRetry,draftUse,draftKeep]){b.type='button';b.className='picker-link';draftActions.append(b);}
const draftPreview=node('details');draftPreview.className='failed-request-review';const draftSummary=node('summary','Rever rascunho guardado'),draftText=node('pre');draftText.className='failed-request-text';draftPreview.append(draftSummary,draftText);draftStatus.append(draftMessage,draftActions,draftPreview);
let draftChannel:{publish:(key:string)=>void;close:()=>void}|null=null;
const draftStorage=new IndexedDraftStorage();
const drafts=new SessionDrafts(draftStorage,{changed:()=>drawDraftStatus(),restore:(target,text,previous)=>chat.restoreDraft?.(target,text,previous)??false,published:key=>draftChannel?.publish(key)});
draftChannel=draftBroadcast(drafts);
const session=new SessionChat({
 history:async target=>{
  if(target.origin==='external')return invoke<SessionSnapshot>('session_chat_history',{target});
  const[value,attachments]=await Promise.all([Bridge.chatReadConversation(target.provider,target.sessionId),Bridge.petDocumentContexts(target.provider,target.sessionId)]);
  return {...fromLocal(target,value),attachments};
 },
 send:async(target,query,model,attachments=[])=>{
  if(target.origin==='external'&&attachments.length)throw new Error('Esta ligação de terminal não recebe anexos. Entrega o documento à conversa Mochi deste pet.');
  if(target.origin==='external')return invoke('session_chat_send',{target,query,...(model?{model}:{})});
  // Reading an archive does not make it current. Replying does, with a busy
  // guard in Rust and a second identity check when the send reserves its turn.
  const current=await Bridge.chatConversation(target.provider);
  if(current.id!==target.sessionId)await Bridge.chatRestoreConversation(target.provider,target.sessionId);
  return Bridge.chatSend(query,attachmentContext(target.provider,target.sessionId,attachments),target.provider,target.sessionId,model);
 },
 removeAttachment:(target,attachment)=>Bridge.petDocumentRemove(target.provider,target.sessionId,attachment.id),
 cancel:async(target,turn)=>{
  if(target.origin==='external'){await invoke('session_chat_cancel',{target,turn});return true;}
  const value=Number(turn);if(!Number.isSafeInteger(value)||value<0)throw new Error('Turno inválido.');
  return Bridge.chatCancel(target.provider,target.sessionId,value);
 },
},()=>{chat?.sync();drafts.activate(session.target,selectionIntent);drawDraftStatus();drawIdentity();},drafts.map);
const chat=buildPrompt(()=>{},session,undefined,draftStatus);chat.el.classList.add('on');document.getElementById('conversation')!.append(chat.el);
function report(text:string){info.textContent=text;info.hidden=!text;}
function drawDraftStatus(){
 const target=session?.target,status=drafts.status(target);draftStatus.hidden=!target||!status;if(!target||!status)return;
 draftStatus.dataset.phase=status.phase;draftStatus.setAttribute('aria-busy',String(status.busy||status.phase==='saving'));
 draftMessage.textContent=status.phase==='loading'?'A recuperar o rascunho…':status.phase==='saving'?'A guardar o rascunho…':status.phase==='restored'?'Rascunho recuperado neste dispositivo. Revê a conversa antes de enviar.':status.phase==='saved'?status.localText?'Rascunho guardado neste dispositivo.':'Sem rascunho guardado.':status.phase==='conflict'?'Há outro rascunho guardado para esta conversa. O texto desta janela foi preservado.':status.error==='limit'?status.localText.length>DRAFT_LIMITS.text?'O texto excede o limite do rascunho guardado. O texto completo mantém-se nesta janela.':'O limite do armazenamento de rascunhos foi atingido. O texto mantém-se nesta janela.':status.error==='blocked'?'O armazenamento de rascunhos está ocupado. O texto mantém-se nesta janela.':'Não foi possível guardar o rascunho. O texto mantém-se nesta janela.';
 draftRetry.hidden=status.phase!=='error';draftUse.hidden=draftKeep.hidden=status.phase!=='conflict';
 draftUse.disabled=draftKeep.disabled=status.busy;
 if(status.phase==='conflict'&&status.error)draftMessage.textContent+=' Não foi possível reler o rascunho guardado. Tenta novamente.';
 draftPreview.hidden=status.phase!=='conflict';draftText.textContent=status.storedText||'O rascunho guardado está vazio.';
 const exact=Object.freeze({...target}),intent=selectionIntent,ticket=opening,revision=status.revision;
 const current=()=>intent===selectionIntent&&ticket===opening&&sameSession(exact,session.target);
 draftRetry.onclick=()=>{if(current())void drafts.retry(exact);};draftUse.onclick=()=>{if(current())void drafts.useStored(exact,revision);};draftKeep.onclick=()=>{if(current())drafts.keepLocal(exact,revision);};
}

function drawIdentity(){
 const target=session.target;if(!target)return;
 const summary=summaries.find(s=>sameSession(s.target,target));
 const source=document.getElementById('source')!;
 const receipt=managedFor(managed,target);
 const origin=document.getElementById('origin')!;origin.textContent=target.origin==='mochi'?'Conversa do pet':receipt?'Sessão Codex':'Terminal';origin.dataset.origin=target.origin;
 source.textContent=`${summary?.title||context?.name||target.provider} · #${target.sessionId.slice(0,8)}`;
 source.title=[target.provider,target.sessionId,summary?.cwd].filter(Boolean).join('\n');
 document.getElementById('terminal')!.hidden=target.origin!=='external'||!!receipt;
 managedPanel.hidden=!receipt;
 if(receipt){
  document.getElementById('managed-description')!.textContent=[managedDescription(receipt),receipt.notice].filter(Boolean).join(' ');
  managedOpen.disabled=managedAction||!canOpenManaged(receipt);managedOpen.textContent=receipt.terminalLaunched?'Reabrir no terminal':'Abrir no terminal';
  managedRelease.disabled=managedAction||!!receipt.turn||!!session.snapshot?.busy;
 }
 const selected=document.getElementById('selected-session')!;selected.textContent=summary?.cwd|| (target.origin==='mochi'?'Mensagens para este pet':'Mensagens para esta sessão de terminal');selected.title=summary?.cwd||sessionOrigin(target);
}
const sessionLists=new SessionListLoading({
 mochi:async owner=>(await Bridge.chatConversations(owner.provider)).map(s=>({target:local(owner,s.id),title:s.title,cwd:'',updatedAt:s.updated,state:s.current?'Atual':'Guardada',capabilities:{history:true,send:true,cancel:true}})),
 external:async owner=>(await invoke<SessionSummary[]>('session_chat_list',{petId:owner.pet})).filter(s=>s.target.petId===owner.pet&&s.target.provider===owner.provider&&s.target.origin==='external'),
 managed:async owner=>(await invoke<ManagedCodexSession[]>('session_chat_managed_list')).filter(s=>s.target.petId===owner.pet&&s.target.provider===owner.provider&&s.target.origin==='external'&&s.phase!=='released'),
},()=>{
 const state=sessionLists.state;
 managed=state.managed.rows;
 const external=[...state.external.rows];
 for(const receipt of managed)if(!external.some(s=>sameSession(s.target,receipt.target)))external.unshift({target:receipt.target,title:'Sessão Codex',cwd:receipt.cwd,updatedAt:0,state:receipt.phase==='lost'?'Ligação terminada':receipt.materialized?'Criada no Coucou':'Nova · sem mensagens',capabilities:{history:true,send:receipt.phase!=='lost',cancel:false}});
 summaries=mergeSessionLists([],state.mochi.rows,external);drawSessions();
});
function sourceNote(source:ListSource,label:string){
 const state=sessionLists.state[source];if(state.phase==='notApplicable')return;
 const note=node('div');note.className='session-source-note';note.dataset.source=source;note.dataset.loading=String(state.loading);
 note.setAttribute('role','status');note.tabIndex=-1;
 const detail=state.loading?state.phase==='idle'?'A procurar…':'A atualizar…':state.phase==='error'?'Indisponível. Não foi possível consultar.':state.phase==='partial'?'Consulta falhou. A mostrar os dados já recebidos.':state.phase==='idle'?'Ainda por consultar.':state.rows.length?`${state.rows.length} identificadas.`:'Nenhuma nesta consulta.';
 note.append(node('span',`${label} · ${detail}`));
 if(state.error){const error=node('small',state.error);note.append(error);}
 if(!state.loading&&(state.phase==='error'||state.phase==='partial')){
  const retry=node('button','Tentar novamente');retry.dataset.retry=source;retry.setAttribute('aria-label',`Tentar consultar ${label.toLowerCase()} novamente`);retry.onclick=()=>sessionLists.refresh(true,source);note.append(retry);
 }
 return note;
}
function drawSessions(){
 const query=search.value.trim(),scroll=list.scrollTop,focused=(document.activeElement as HTMLElement|null)?.closest<HTMLButtonElement>('.session-entry')?.dataset.key;
 const retryFocus=(document.activeElement as HTMLElement|null)?.closest<HTMLElement>('.session-source-note')?.dataset.source;list.replaceChildren();
 const states=sessionLists.state,loading=Object.values(states).some(s=>s.loading);
 // One pending inventory must not hide ready rows from assistive technology.
 list.setAttribute('aria-busy','false');
 const refresh=document.getElementById('sessions-refresh') as HTMLButtonElement;
 refresh.disabled=Object.values(states).every(s=>s.phase==='notApplicable'||s.loading);refresh.dataset.loading=String(loading);
 const target=session.target,rowsWithSelection=[...summaries];
 if(target&&!rowsWithSelection.some(s=>sameSession(s.target,target)))rowsWithSelection.unshift({target,title:context?.session?.title||'Sessão selecionada',cwd:context?.session?.cwd||'',updatedAt:0,state:'Selecionada · fora da lista recebida',capabilities:{history:true,send:false,cancel:false}});
 for(const origin of ['mochi','external'] as const){
  if(origin==='mochi'&&states.mochi.phase==='notApplicable')continue;
  const rows=matchingSessions(rowsWithSelection,query,origin),shown=visibleSessions(rows,sessionLimits[origin],session.target);
  const group=node('section');group.className='session-source-group';group.dataset.origin=origin;
  const heading=node('h3',origin==='mochi'?'Conversas do pet':'Sessões externas');heading.className='session-group';
  // This is the visible inventory, not a provider total or a liveness count.
  const confirmed=(origin==='mochi'?[states.mochi]:[states.external,states.managed]).every(s=>s.phase==='ready'||s.phase==='notApplicable');
  heading.append(node('span',!rows.length&&!confirmed?'Por confirmar':`${shown.length<rows.length?`${shown.length} / `:''}${rows.length} listadas`));group.append(heading);
  for(const source of origin==='mochi'?['mochi'] as const:['external','managed'] as const){const note=sourceNote(source,source==='mochi'?'Conversas':source==='external'?'Inventário externo':'Gestão Codex');if(note)group.append(note);}
  for(const s of shown){
   const button=node('button');button.className='session-entry';button.dataset.key=sessionTargetKey(s.target);button.dataset.selected=String(sameSession(session.target,s.target));
   if(sameSession(session.target,s.target))button.setAttribute('aria-current','true');
   button.append(node('strong',s.title||'Conversa'),node('span',`${s.state} · #${s.target.sessionId.slice(0,8)}${s.model?' · '+s.model:''}`));
   if(s.cwd)button.append(node('small',s.cwd));button.title=`${s.target.sessionId}\n${s.cwd}`;
   button.dataset.origin=origin;button.onclick=()=>void choose(s.target);group.append(button);
  }
  if(shown.length<rows.length){const more=node('button',`Mostrar mais · ${rows.length-shown.length} por mostrar`);more.className='session-more';more.setAttribute('aria-label',`Mostrar mais ${origin==='mochi'?'conversas do pet':'sessões externas'} · ${rows.length-shown.length} por mostrar`);more.onclick=()=>{sessionLimits[origin]+=50;drawSessions();list.querySelectorAll<HTMLButtonElement>(`.session-entry[data-origin="${origin}"]`)[shown.length]?.focus();};group.append(more);}
  if(query&&!rows.length)group.append(node('p','Sem correspondências nos dados já recebidos.'));
  list.append(group);
 }
 if(query){const clear=node('button','Limpar pesquisa');clear.className='session-more';clear.onclick=()=>{search.value='';sessionLimits={mochi:50,external:50};drawSessions();search.focus();};list.append(clear);}
 if(document.hasFocus()){
  if(focused)Array.from(list.querySelectorAll<HTMLButtonElement>('.session-entry')).find(b=>b.dataset.key===focused)?.focus({preventScroll:true});
  else if(retryFocus)(list.querySelector<HTMLElement>(`[data-retry="${retryFocus}"]`)??list.querySelector<HTMLElement>(`.session-source-note[data-source="${retryFocus}"]`))?.focus({preventScroll:true});
 }
 list.scrollTop=scroll;drawIdentity();
}
function refreshSessions(manual=false){if(context)sessionLists.refresh(manual);}
async function choose(target:SessionTarget){
 const c=context;if(!c||target.petId!==c.pet||target.provider!==c.provider){report('A conversa pedida não pertence a este pet.');return;}
 const intent=++selectionIntent;report('');
 const selecting=session.select(target);drawSessions();await selecting;if(context!==c||intent!==selectionIntent||!sameSession(session.target,target))return;drawSessions();if(document.hasFocus())chat.focus?.();
 if(innerWidth<=480)setSidebar(false);
}
function setSidebar(show:boolean){document.getElementById('sessions')!.hidden=!show;document.getElementById('chat-layout')!.classList.toggle('sidebar-collapsed',!show);document.getElementById('sessions-toggle')!.setAttribute('aria-expanded',String(show));if(show)void refreshSessions();}
async function open(c:Context){
 if(!c.pet)return;const ticket=++opening,intent=++selectionIntent;context=c;session.clear();summaries=[];managed=[];managedReading++;managedPanel.hidden=true;sessionLists.reset(c);search.value='';sessionLimits={mochi:50,external:50};report('');
 document.getElementById('owner')!.textContent=c.name;
 document.getElementById('origin')!.textContent='';document.getElementById('source')!.textContent='A abrir a conversa…';document.getElementById('selected-session')!.textContent='';document.getElementById('terminal')!.hidden=true;
 State.upsertExternalAgent(c.pet,c.name,'#94a3b8');State.setFocus(c.pet);State.view='prompt';State.mode='expanded';State.notify();
 newSession.hidden=!c.provider;newSession.disabled=false;newSession.setAttribute('aria-label',`Nova conversa com ${c.name}`);newSession.title=`Criar uma conversa com ${c.name}`;
 newCodex.hidden=c.pet!=='agent_codex'||c.provider!=='codex';newCodex.disabled=creatingCodex;
 setSidebar(c.showSessions||innerWidth>=700);
 try{
  let target:SessionTarget;
  if(c.target){if(c.target.petId!==c.pet||c.target.provider!==c.provider)throw Error('O pedido não pertence a este pet.');target={...c.target};}
  else if(c.conversationId)target=local(c,c.conversationId);
  else if(c.session)target={petId:c.pet,provider:c.session.backend,origin:'external',sessionId:c.session.id};
  else if(c.provider){const current=await Bridge.chatConversation(c.provider);target=local(c,current.id);}
  else {report('Este pet ainda não tem um provider de chat configurado.');return;}
  if(ticket!==opening||intent!==selectionIntent)return;await session.select(target);if(ticket!==opening||intent!==selectionIntent)return;void refreshSessions();
 }catch(e){if(ticket===opening&&intent===selectionIntent)report(String(e).replace(/^Error:\s*/,''));}
}
const boot=await Bridge.boot();if(boot)State.settings={...State.settings,...boot.settings,companionMode:true};await PetPreferences.init();
State.subscribe(()=>chat.sync());
PetPreferences.subscribe(()=>chat.sync());
await onEvent<{context:Context;rect:Rect}>('pet-chat-opened',value=>{acceptRect(value.rect);void open(value.context);});
let refreshTimer:ReturnType<typeof setTimeout>|undefined;
function invalidate(target?:SessionTarget){
 if(target&&!sameSession(target,session.target))return;
 session.invalidate(target);
 if(refreshTimer)return;
 refreshTimer=setTimeout(()=>{refreshTimer=undefined;if(!document.hidden)void session.refresh();},80);
}
async function refreshManaged(target?:SessionTarget){
 const receipt=managedFor(managed,target??session.target);if(!receipt)return;
 const c=context,ticket=++managedReading;
 try{const next=await invoke<ManagedCodexSession>('session_chat_managed_inspect',{reference:managedReference(receipt)});if(ticket!==managedReading||context!==c)return;sessionLists.managedReceipt(next);drawIdentity();}
 catch(error){if(ticket===managedReading&&context===c&&sameSession(receipt.target,session.target))report(String(error).replace(/^Error:\s*/,''));}
}
await onEvent<{target:SessionTarget}>('session-chat-updated',e=>{invalidate(e.target);void refreshManaged(e.target);});
await onEvent<{provider:string|null;conversationId:string|null}>('pet-document-context-changed',e=>{
 const target=session.target;if(target?.origin==='mochi'&&(!e.provider||target.provider===e.provider)&&(!e.conversationId||target.sessionId===e.conversationId))invalidate();
});
await onEvent<string>('assistant-history-changed',provider=>{if(context?.provider===provider){if(session.target?.origin==='mochi')invalidate();void refreshSessions();}});
await onEvent<{provider:string;conversationId:string}>('chat-turn-state',e=>{if(session.target?.origin==='mochi'&&session.target.provider===e.provider&&session.target.sessionId===e.conversationId)invalidate();});
await onEvent<{provider:string;conversationId?:string}>('chat-delta',e=>{if(typeof e==='object'&&session.target?.origin==='mochi'&&session.target.provider===e.provider&&session.target.sessionId===e.conversationId)invalidate();});
await onEvent<Settings>('settings-changed',settings=>{State.settings=OllamaModelSelection.applySettings({...State.settings,...settings});chat.sync();});
const dropHint=node('div');dropHint.id='chat-drop-hint';dropHint.hidden=true;dropHint.setAttribute('role','status');document.body.append(dropHint);
await onDragDrop(event=>{
 const target=session.target;
 if(event.type==='enter'||event.type==='over'){
  dropHint.textContent=target?.origin==='mochi'?'Larga o documento para o anexar a esta conversa':'Os anexos pertencem à conversa do pet. Abre uma conversa do pet para anexar.';
  dropHint.hidden=false;return;
 }
 dropHint.hidden=true;
 if(event.type==='rejected'){report(event.error||'O documento não foi recebido. Volta a tentar.');return;}
 if(event.type!=='drop')return;
 if(!target||target.origin!=='mochi'){report('Esta sessão de terminal não recebe anexos. Abre uma conversa do pet para anexar o documento.');return;}
 if(!event.paths?.length)return;
 report('A receber os anexos…');
 void receiveDocumentDrop(Bridge,target.provider,event.dropId,target.sessionId).then(async batch=>{
  if(!sameSession(target,session.target))return;
  report(documentBatchMessage(batch));await session.refresh();
 }).catch(error=>{if(sameSession(target,session.target))report(String(error).replace(/^Error:\s*/,''));});
});
const initialRect=await invoke<Rect|null>('pet_chat_rect');if(initialRect)acceptRect(initialRect);
const initialOpening=opening,initialContext=await invoke<Context>('pet_chat_context');
if(opening===initialOpening)await open(initialContext);
const closeButton=document.getElementById('close') as HTMLButtonElement;
async function hideChat(){
 if(closeButton.disabled)return;stopDrag();closeButton.disabled=true;closeButton.setAttribute('aria-busy','true');const ticket=opening;
 try{await invoke('pet_chat_hide');}
 catch(error){if(ticket===opening)report(`Não foi possível fechar a conversa. ${String(error).replace(/^Error:\s*/,'')}`);}
 finally{closeButton.disabled=false;closeButton.removeAttribute('aria-busy');}
}
closeButton.onclick=()=>void hideChat();
await onEvent('pet-chat-hidden',()=>{opening++;selectionIntent++;context=null;stopDrag();session.clear();sessionLists.reset(null);});
document.addEventListener('keydown',e=>{
 if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();setSidebar(true);search.focus();search.select();}
 else if(e.key==='Escape'&&!e.defaultPrevented){e.preventDefault();void hideChat();}
});
document.getElementById('sessions-toggle')!.onclick=()=>{
 const show=document.getElementById('sessions')!.hidden;setSidebar(show);if(show)search.focus();
};
newCodex.onclick=()=>{
 const c=context;if(c?.pet!=='agent_codex'||c.provider!=='codex'||creatingCodex)return;
 const intent=++selectionIntent;drafts.activate(session.target,intent);creatingCodex=true;newCodex.disabled=true;report('A criar a sessão Codex…');
 void invoke<ManagedCodexSession>('session_chat_managed_create',{query:null,openTerminal:false}).then(async receipt=>{
  if(context!==c)return;if(receipt.target.petId!==c.pet||receipt.target.provider!==c.provider||receipt.target.origin!=='external')throw Error('A sessão criada não pertence a este pet.');sessionLists.managedReceipt(receipt);if(intent===selectionIntent){report(receipt.notice??'');await choose(receipt.target);}refreshSessions();
 }).catch(error=>{if(context===c&&intent===selectionIntent)report(String(error).replace(/^Error:\s*/,''));}).finally(()=>{creatingCodex=false;newCodex.disabled=false;});
};
async function managedCommand(command:'session_chat_managed_open'|'session_chat_managed_release'){
 const receipt=managedFor(managed,session.target),c=context;if(!receipt||managedAction)return;
 managedAction=true;report('');drawIdentity();
 try{
  const next=await invoke<ManagedCodexSession>(command,{reference:managedReference(receipt)});
  if(context!==c)return;managedReading++;sessionLists.managedReceipt(next);
  refreshSessions();
  if(context===c&&sameSession(receipt.target,session.target))report(next.notice??(command==='session_chat_managed_open'?'Abertura pedida ao terminal.':'Sessão retirada da gestão. O histórico e o terminal foram preservados.'));
 }catch(error){if(context===c&&sameSession(receipt.target,session.target))report(String(error).replace(/^Error:\s*/,''));}
 finally{managedAction=false;drawIdentity();}
}
managedOpen.onclick=()=>void managedCommand('session_chat_managed_open');managedRelease.onclick=()=>void managedCommand('session_chat_managed_release');
document.getElementById('sessions-close')!.onclick=()=>{setSidebar(false);document.getElementById('sessions-toggle')!.focus();};document.getElementById('sessions-refresh')!.onclick=()=>{refreshSessions(true);void session.refresh();};search.oninput=()=>{sessionLimits={mochi:50,external:50};drawSessions();};
search.onkeydown=e=>{
 if(e.key==='ArrowDown'){e.preventDefault();list.querySelector<HTMLButtonElement>('.session-entry')?.focus();}
 else if(e.key==='Escape'){e.preventDefault();e.stopPropagation();if(search.value){search.value='';sessionLimits={mochi:50,external:50};drawSessions();}else{setSidebar(false);document.getElementById('sessions-toggle')!.focus();}}
};
list.onkeydown=e=>{
 if(e.key!=='ArrowDown'&&e.key!=='ArrowUp')return;
 const buttons=Array.from(list.querySelectorAll<HTMLButtonElement>('.session-entry')),index=buttons.indexOf(document.activeElement as HTMLButtonElement);if(index<0)return;
 e.preventDefault();if(e.key==='ArrowUp'&&index===0)search.focus();else buttons[Math.min(buttons.length-1,index+(e.key==='ArrowDown'?1:-1))]?.focus();
};
newSession.onclick=()=>{
 const c=context;if(!c?.provider||newSession.disabled)return;const intent=++selectionIntent;drafts.activate(session.target,intent);newSession.disabled=true;report('');
 void Bridge.chatReset(c.provider).then(async view=>{if(context!==c)return;if(!view)throw new Error('Não foi possível criar a conversa.');if(intent===selectionIntent)await choose(local(c,view.id));refreshSessions(true);})
  .catch(e=>{if(context===c&&intent===selectionIntent)report(String(e).replace(/^Error:\s*/,''));}).finally(()=>{if(context===c)newSession.disabled=false;});
};
document.getElementById('opacity')!.oninput=e=>document.body.style.setProperty('--surface-opacity',String(Number((e.target as HTMLInputElement).value)/100));
let nextPlacement:Rect|null=null,placement:Promise<void>|null=null;
function place(next:Rect):Promise<void>{
 nextPlacement=next;
 if(!placement)placement=drainPlacement();
 return placement;
}
async function drainPlacement(){
 try{while(nextPlacement){const requested=nextPlacement;nextPlacement=null;acceptRect(await invoke<Rect>('pet_chat_layout',{rect:requested}));}}
 catch(error){nextPlacement=null;stopDrag();report(`Não foi possível ajustar a conversa. ${String(error).replace(/^Error:\s*/,'')}`);}
 finally{placement=null;}
}
const sizeButton=document.getElementById('size') as HTMLButtonElement;
sizeButton.onclick=async()=>{
 if(sizeButton.disabled)return;stopDrag();sizeButton.disabled=true;sizeButton.setAttribute('aria-busy','true');report('');
 try{await place({...rect,width:large?760:1000,height:large?620:800});}
 finally{sizeButton.disabled=false;sizeButton.removeAttribute('aria-busy');}
};
const handle=document.getElementById('handle')!;
function stopDrag(){
 if(!dragging)return;dragging=false;const active=dragState;dragState=null;handle.dataset.dragging='false';
 if(active&&handle.hasPointerCapture(active.id))handle.releasePointerCapture(active.id);
}
async function dragLoop(active:NonNullable<typeof dragState>){
 if(dragState!==active)return;
 try{
  const point=await invoke<{x:number;y:number}>('pet_chat_pointer');if(dragState!==active)return;
  if(!active.pointer)active.pointer=point;
  else await place({...active.origin,x:active.origin.x+point.x-active.pointer.x,y:active.origin.y+point.y-active.pointer.y});
 }catch(error){if(dragState===active){stopDrag();report(String(error).replace(/^Error:\s*/,''));}return;}
 if(dragState===active)setTimeout(()=>void dragLoop(active),24);
}
handle.onpointerdown=e=>{if(e.button!==0||(e.target as HTMLElement).closest('button,input,label'))return;e.preventDefault();stopDrag();dragging=true;dragState={id:e.pointerId,origin:{...rect},pointer:null};handle.dataset.dragging='true';handle.setPointerCapture(e.pointerId);void dragLoop(dragState);};
handle.onpointermove=e=>{if(dragging&&!(e.buttons&1))stopDrag();};
handle.onpointerup=handle.onpointercancel=handle.onlostpointercapture=stopDrag;
document.addEventListener('pointerup',stopDrag,true);window.addEventListener('blur',stopDrag);
document.getElementById('terminal')!.onclick=()=>{const target=session.target;if(target?.origin==='external')void Bridge.petFocusSession(target.provider,target.sessionId).catch(e=>report(String(e).replace(/^Error:\s*/,'')));};
setInterval(()=>{if(!document.hidden&&session.target&&!session.loading)void session.refresh();},2000);
setInterval(()=>{if(!document.hidden)void refreshSessions();},30000);
document.addEventListener('visibilitychange',()=>{if(!document.hidden){invalidate();void refreshSessions();drafts.hint(session.target?sessionTargetKey(session.target):'');}else if(session.target)void drafts.flush(session.target);});
window.addEventListener('pagehide',()=>{if(session.target)void drafts.flush(session.target);});
