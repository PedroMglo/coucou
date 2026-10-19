import '../style.css';import './pet-chat.css';
import {invoke} from '@tauri-apps/api/core';
import {Bridge,onEvent,onDragDrop,type ConversationView} from '../core/bridge';
import {State} from '../core/state';
import {buildPrompt} from '../views/chat';
import {PetPreferences} from '../mochi/pet-preferences';
import {SessionChat,sameSession,sessionOrigin,sessionTargetKey,type SessionTarget,type SessionSnapshot,type SessionSummary} from '../core/session-chat';
import {matchingSessions,mergeSessionLists} from './session-list';
import {OllamaModelSelection} from '../core/ollama-selection';
import type {Settings} from '../core/state';
import {managedReference,managedFor,managedDescription,canOpenManaged,type ManagedCodexSession} from '../core/managed-codex';
interface Session{id:string;backend:string;title:string;cwd:string;steps:string[];finalLine:string}
interface Context{pet:string;name:string;provider:string;session?:Session;conversationId?:string;showSessions?:boolean;target?:SessionTarget}
interface Rect{x:number;y:number;width:number;height:number}
let context:Context|null=null,rect:Rect={x:100,y:80,width:760,height:620},large=false,listing=0,opening=0;
let dragging=false,dragState:{id:number;origin:Rect;pointer:{x:number;y:number}|null}|null=null;
const node=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const e=document.createElement(tag);if(text!==undefined)e.textContent=text;return e;};
const info=document.getElementById('feedback')!;
const list=document.getElementById('session-list')!,search=document.getElementById('session-search') as HTMLInputElement;
const newSession=document.getElementById('new-session') as HTMLButtonElement;
const newCodex=document.getElementById('new-codex') as HTMLButtonElement;
const managedPanel=document.getElementById('managed-session')!,managedOpen=document.getElementById('managed-open') as HTMLButtonElement,managedRelease=document.getElementById('managed-release') as HTMLButtonElement;
let managed:ManagedCodexSession[]=[],managedAction=false,managedReading=0,creatingCodex=false;
let summaries:SessionSummary[]=[],listLoading=false;
const local=(c:Context,id:string):SessionTarget=>({petId:c.pet,provider:c.provider,origin:'mochi',sessionId:id,conversationId:id});
function fromLocal(target:SessionTarget,value:ConversationView):SessionSnapshot {
 if(value.id!==target.sessionId)throw new Error('A conversa recebida não corresponde à sessão escolhida.');
 const messages=value.messages.map((m,i)=>({id:`${value.id}:${i}`,role:m.role,content:m.content}));
 if(value.busy&&value.query)messages.push({id:`${value.id}:query`,role:'user',content:value.query});
 if(value.busy&&value.stream)messages.push({id:`${value.id}:stream`,role:'assistant',content:value.stream});
 return {target,messages,notice:value.notice,busy:!!value.busy,turn:value.turn===undefined?undefined:String(value.turn),capabilities:{history:true,send:true,cancel:true}};
}
const session=new SessionChat({
 history:async target=>{
  if(target.origin==='external')return invoke<SessionSnapshot>('session_chat_history',{target});
  const[value,attachment]=await Promise.all([Bridge.chatReadConversation(target.provider,target.sessionId),Bridge.petDocumentContext(target.provider,target.sessionId)]);
  return {...fromLocal(target,value),attachment};
 },
 send:async(target,query,model,attachment)=>{
  if(target.origin==='external'&&attachment)throw new Error('Esta ligação de terminal não recebe anexos. Entrega o documento à conversa Mochi deste pet.');
  if(target.origin==='external')return invoke('session_chat_send',{target,query,...(model?{model}:{})});
  // Reading an archive does not make it current. Replying does, with a busy
  // guard in Rust and a second identity check when the send reserves its turn.
  const current=await Bridge.chatConversation(target.provider);
  if(current.id!==target.sessionId)await Bridge.chatRestoreConversation(target.provider,target.sessionId);
  if(attachment?.conversationId&&attachment.conversationId!==target.sessionId)throw new Error('O documento pertence a outra conversa.');
  const reply=await Bridge.chatSend(query,attachment?{kind:'file',name:attachment.name,path:attachment.path}:null,target.provider,target.sessionId,model,attachment?.sequence);
  if(attachment?.sequence!==undefined)void Bridge.petDocumentClear(target.provider,target.sessionId,attachment.sequence).catch(()=>{});
  return reply;
 },
 clearAttachment:(target,attachment)=>attachment.sequence===undefined?Promise.resolve():Bridge.petDocumentClear(target.provider,target.sessionId,attachment.sequence),
 cancel:async(target,turn)=>{
  if(target.origin==='external'){await invoke('session_chat_cancel',{target,turn});return true;}
  const value=Number(turn);if(!Number.isSafeInteger(value)||value<0)throw new Error('Turno inválido.');
  return Bridge.chatCancel(target.provider,target.sessionId,value);
 },
},()=>{chat?.sync();drawIdentity();});
const chat=buildPrompt(()=>{},session);chat.el.classList.add('on');document.getElementById('conversation')!.append(chat.el);
function report(text:string){info.textContent=text;info.hidden=!text;}
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
function drawSessions(){
 const query=search.value.trim(),scroll=list.scrollTop,focused=(document.activeElement as HTMLElement|null)?.closest<HTMLButtonElement>('.session-entry')?.dataset.key;list.replaceChildren();
 list.setAttribute('aria-busy',String(listLoading));
 const refresh=document.getElementById('sessions-refresh') as HTMLButtonElement;refresh.disabled=listLoading;refresh.dataset.loading=String(listLoading);
 for(const origin of ['mochi','external'] as const){
  const rows=matchingSessions(summaries,query,origin);
  if(!rows.length)continue;
  const heading=node('h3',origin==='mochi'?'Conversas do pet':'Sessões de terminal');heading.append(node('span',String(rows.length)));heading.className='session-group';list.append(heading);
  for(const s of rows){
  const button=node('button');button.className='session-entry';button.dataset.key=sessionTargetKey(s.target);button.dataset.selected=String(sameSession(session.target,s.target));
  if(sameSession(session.target,s.target))button.setAttribute('aria-current','true');
  button.append(node('strong',s.title||'Conversa'),node('span',`${s.state} · #${s.target.sessionId.slice(0,8)}${s.model?' · '+s.model:''}`));
  if(s.cwd)button.append(node('small',s.cwd));button.title=`${s.target.sessionId}\n${s.cwd}`;
  button.onclick=()=>void choose(s.target);list.append(button);
  }
 }
 if(!list.childElementCount){
  const empty=node('div');empty.className='session-list-empty';empty.append(node('strong',listLoading?'A procurar conversas…':query?'Sem resultados':'Ainda sem conversas'),node('p',query?'Experimenta outro nome, pasta ou identificador.':'Cria uma conversa para começar a falar com este pet.'));
  if(query){const clear=node('button','Limpar pesquisa');clear.onclick=()=>{search.value='';drawSessions();search.focus();};empty.append(clear);}list.append(empty);
 }
 if(focused&&document.hasFocus())Array.from(list.querySelectorAll<HTMLButtonElement>('.session-entry')).find(b=>b.dataset.key===focused)?.focus({preventScroll:true});
 list.scrollTop=scroll;drawIdentity();
}
async function refreshSessions(){
 const c=context;if(!c||listLoading)return;const ticket=++listing;listLoading=true;drawSessions();
 const results=await Promise.allSettled([c.provider?Bridge.chatConversations(c.provider):Promise.resolve([]),invoke<SessionSummary[]>('session_chat_list',{petId:c.pet}),c.pet==='agent_codex'&&c.provider==='codex'?invoke<ManagedCodexSession[]>('session_chat_managed_list'):Promise.resolve([])]);
 if(context!==c||ticket!==listing)return;
 const own=results[0].status==='fulfilled'?results[0].value.map(s=>({target:local(c,s.id),title:s.title,cwd:'',updatedAt:s.updated,state:s.current?'Atual':'Guardada',capabilities:{history:true,send:true,cancel:true}})):null;
 const external=results[1].status==='fulfilled'?results[1].value.filter(s=>s.target.petId===c.pet&&s.target.origin==='external'):null;
 if(results[2].status==='fulfilled')managed=results[2].value.filter(s=>s.target.petId===c.pet&&s.phase!=='released');
 if(external)for(const receipt of managed)if(!external.some(s=>sameSession(s.target,receipt.target)))external.unshift({target:receipt.target,title:'Sessão Codex',cwd:receipt.cwd,updatedAt:0,state:receipt.phase==='lost'?'Ligação terminada':receipt.materialized?'Criada no Coucou':'Nova · sem mensagens',capabilities:{history:true,send:receipt.phase!=='lost',cancel:false}});
 summaries=mergeSessionLists(summaries,own,external);listLoading=false;
 const target=session.target;
 if(target&&!summaries.some(s=>sameSession(s.target,target)))summaries.unshift({target,title:c.session?.title||'Sessão selecionada',cwd:c.session?.cwd||'',updatedAt:0,state:'Selecionada',capabilities:{history:true,send:false,cancel:false}});
 report(results.filter(r=>r.status==='rejected').map(r=>String((r as PromiseRejectedResult).reason).replace(/^Error:\s*/, '')).join(' · '));drawSessions();
}
async function choose(target:SessionTarget){
 report('');
 await session.select(target);if(!sameSession(session.target,target))return;drawSessions();if(document.hasFocus())chat.focus?.();
 if(innerWidth<=480)setSidebar(false);
}
function setSidebar(show:boolean){document.getElementById('sessions')!.hidden=!show;document.getElementById('chat-layout')!.classList.toggle('sidebar-collapsed',!show);document.getElementById('sessions-toggle')!.setAttribute('aria-expanded',String(show));if(show)void refreshSessions();}
async function open(c:Context){
 if(!c.pet)return;const ticket=++opening;context=c;session.clear();summaries=[];managed=[];managedReading++;managedPanel.hidden=true;listing++;listLoading=false;search.value='';report('');
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
  if(ticket!==opening)return;await session.select(target);if(ticket!==opening)return;void refreshSessions();
 }catch(e){if(ticket===opening)report(String(e).replace(/^Error:\s*/,''));}
}
const boot=await Bridge.boot();if(boot)State.settings={...State.settings,...boot.settings,companionMode:true};await PetPreferences.init();
State.subscribe(()=>chat.sync());
await onEvent<{context:Context;rect:Rect}>('pet-chat-opened',value=>{rect=value.rect;void open(value.context);});
let refreshTimer:ReturnType<typeof setTimeout>|undefined;
function invalidate(target?:SessionTarget){
 if(target&&!sameSession(target,session.target)||refreshTimer)return;
 refreshTimer=setTimeout(()=>{refreshTimer=undefined;if(!document.hidden)void session.refresh();},80);
}
async function refreshManaged(target?:SessionTarget){
 const receipt=managedFor(managed,target??session.target);if(!receipt)return;
 const c=context,ticket=++managedReading;
 try{const next=await invoke<ManagedCodexSession>('session_chat_managed_inspect',{reference:managedReference(receipt)});if(ticket!==managedReading||context!==c)return;managed=managed.map(row=>row.handle===next.handle?next:row);drawIdentity();}
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
 if(event.paths?.length!==1){report('Entrega um documento ou uma pasta de cada vez.');return;}
 report('A receber o documento…');
 void Bridge.ingestFile(event.paths[0],target.provider,target.sessionId,event.dropId).then(async file=>{
  if(!sameSession(target,session.target))return;
  report(`${file.name} anexado a esta conversa.`);await session.refresh();
 }).catch(error=>{if(sameSession(target,session.target))report(String(error).replace(/^Error:\s*/,''));});
});
const initialRect=await invoke<Rect|null>('pet_chat_rect');if(initialRect)rect=initialRect;
const initialOpening=opening,initialContext=await invoke<Context>('pet_chat_context');
if(opening===initialOpening)await open(initialContext);
document.getElementById('close')!.onclick=()=>{stopDrag();void invoke('pet_chat_hide');};
await onEvent('pet-chat-hidden',()=>{opening++;stopDrag();session.clear();});
document.addEventListener('keydown',e=>{
 if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();setSidebar(true);search.focus();search.select();}
 else if(e.key==='Escape'&&!e.defaultPrevented){e.preventDefault();void invoke('pet_chat_hide');}
});
document.getElementById('sessions-toggle')!.onclick=()=>{
 const show=document.getElementById('sessions')!.hidden;setSidebar(show);if(show)search.focus();
};
newCodex.onclick=()=>{
 const c=context;if(c?.pet!=='agent_codex'||c.provider!=='codex'||creatingCodex)return;
 creatingCodex=true;newCodex.disabled=true;report('A criar a sessão Codex…');
 void invoke<ManagedCodexSession>('session_chat_managed_create',{query:null,openTerminal:false}).then(async receipt=>{
  if(context!==c)return;managed=[...managed.filter(row=>row.handle!==receipt.handle),receipt];report(receipt.notice??'');await choose(receipt.target);await refreshSessions();
 }).catch(error=>{if(context===c)report(String(error).replace(/^Error:\s*/,''));}).finally(()=>{creatingCodex=false;newCodex.disabled=false;});
};
async function managedCommand(command:'session_chat_managed_open'|'session_chat_managed_release'){
 const receipt=managedFor(managed,session.target),c=context;if(!receipt||managedAction)return;
 managedAction=true;report('');drawIdentity();
 try{
  const next=await invoke<ManagedCodexSession>(command,{reference:managedReference(receipt)});
  if(context!==c)return;managedReading++;managed=managed.map(row=>row.handle===next.handle?next:row).filter(row=>row.phase!=='released');
  await refreshSessions();
  if(context===c&&sameSession(receipt.target,session.target))report(next.notice??(command==='session_chat_managed_open'?'Abertura pedida ao terminal.':'Sessão retirada da gestão. O histórico e o terminal foram preservados.'));
 }catch(error){if(context===c&&sameSession(receipt.target,session.target))report(String(error).replace(/^Error:\s*/,''));}
 finally{managedAction=false;drawIdentity();}
}
managedOpen.onclick=()=>void managedCommand('session_chat_managed_open');managedRelease.onclick=()=>void managedCommand('session_chat_managed_release');
document.getElementById('sessions-close')!.onclick=()=>{setSidebar(false);document.getElementById('sessions-toggle')!.focus();};document.getElementById('sessions-refresh')!.onclick=()=>{void refreshSessions();void session.refresh();};search.oninput=drawSessions;
search.onkeydown=e=>{
 if(e.key==='ArrowDown'){e.preventDefault();list.querySelector<HTMLButtonElement>('.session-entry')?.focus();}
 else if(e.key==='Escape'){e.preventDefault();e.stopPropagation();if(search.value){search.value='';drawSessions();}else{setSidebar(false);document.getElementById('sessions-toggle')!.focus();}}
};
list.onkeydown=e=>{
 if(e.key!=='ArrowDown'&&e.key!=='ArrowUp')return;
 const buttons=Array.from(list.querySelectorAll<HTMLButtonElement>('.session-entry')),index=buttons.indexOf(document.activeElement as HTMLButtonElement);if(index<0)return;
 e.preventDefault();if(e.key==='ArrowUp'&&index===0)search.focus();else buttons[Math.min(buttons.length-1,index+(e.key==='ArrowDown'?1:-1))]?.focus();
};
newSession.onclick=()=>{
 const c=context;if(!c?.provider)return;newSession.disabled=true;report('');
 void Bridge.chatReset(c.provider).then(async view=>{if(context!==c)return;if(!view)throw new Error('Não foi possível criar a conversa.');await choose(local(c,view.id));await refreshSessions();})
  .catch(e=>{if(context===c)report(String(e).replace(/^Error:\s*/,''));}).finally(()=>{if(context===c)newSession.disabled=false;});
};
document.getElementById('opacity')!.oninput=e=>document.body.style.setProperty('--surface-opacity',String(Number((e.target as HTMLInputElement).value)/100));
let nextPlacement:Rect|null=null,placing=false;
async function place(next:Rect){nextPlacement=next;if(placing)return;placing=true;try{while(nextPlacement){const requested=nextPlacement;nextPlacement=null;const fitted=await invoke<Rect>('pet_chat_layout',{rect:requested});if(!nextPlacement)rect=fitted;}}finally{placing=false;}}
document.getElementById('size')!.onclick=()=>{stopDrag();large=!large;void place({...rect,width:large?1000:760,height:large?800:620});};
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
document.addEventListener('visibilitychange',()=>{if(!document.hidden){invalidate();void refreshSessions();}});
