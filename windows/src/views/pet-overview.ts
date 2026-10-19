import {h} from './dom';
import {State} from '../core/state';
import {Bridge,onEvent,emitToWindow,type ConversationSummary} from '../core/bridge';
import {activeModel,providerForPill} from '../core/providers';
import {PetPreferences,profileSummary} from '../mochi/pet-preferences';
import {createMiniBot} from '../mochi/minibots';
import {buildSpotifyCard} from './spotify';
import type {ViewActions,ViewHost} from './views';
import type {PetUsage} from '../core/pet-usage';

const states:Record<string,string>={idle:'Disponível',sleeping:'Em descanso',working:'A trabalhar',thinking:'A pensar',searching:'A pesquisar',approval:'Precisa de aprovação',question:'À espera da tua resposta',finished:'Concluído',error:'Precisa de atenção',ratelimit:'Limite atingido'};
const number=(n:number|null|undefined)=>n==null?'—':n.toLocaleString('pt-PT');

/** A pet card is a dashboard for the selected pet, rather than a cropped copy
 * of the original two-column notch. Nodes survive polling and live updates. */
export function buildPetOverview(actions:Pick<ViewActions,'setFocus'|'openTarget'|'blip'|'openSettingsWindow'|'openPetSettings'>):ViewHost {
 const avatar=h('div',{class:'pet-summary-avatar'}),name=h('strong'),status=h('span',{class:'pet-summary-status'});
 const model=h('p',{class:'pet-summary-model'}),activity=h('p',{class:'pet-summary-activity'});
 const usage=h('div',{class:'pet-summary-usage'}),sessions=h('div',{class:'pet-summary-sessions'});
 const error=h('p',{class:'pet-summary-error',role:'status'});
 const chat=h('button',{class:'pet-summary-primary',text:'Conversar',onclick:()=>{
  const task=State.focusTask;if(!task)return;
  if(State.settings.companionMode&&['approval','question'].includes(task.state))void emitToWindow('companions','companions-present',{id:task.id});
  else void Bridge.petChatOpen(task.id).catch(e=>{error.textContent=String(e);});
 }});
 const history=h('button',{text:'Conversas e sessões',onclick:()=>{
  const task=State.focusTask;if(task)void Bridge.petChatOpen(task.id,undefined,true).catch(e=>{error.textContent=String(e);});
 }});
 const settings=h('button',{text:'Configurar pet',onclick:()=>{const id=State.focusId;if(id){if(actions.openPetSettings)actions.openPetSettings(id);else void Bridge.openSettingsWindow('pets:'+id);}}});
 const refresh=h('button',{text:'Atualizar',title:'Atualizar informação deste pet',onclick:()=>{lastRead=0;sync();}});
 const identity=h('div',{class:'pet-summary-identity'},avatar,h('div',{},h('div',{class:'pet-summary-heading'},name,status),model));
 const content=h('div',{class:'pet-summary-content'},identity,activity,usage,sessions,error,h('div',{class:'pet-summary-actions'},chat,history,settings,refresh));
 const music=buildSpotifyCard(),musicWrap=h('div',{class:'pet-summary-music'},music.el);
 const el=h('div',{class:'view pet-overview'},content,musicWrap);
 let owner='',provider='',lastRead=0,reading=false,version=0,rows:ConversationSummary[]|null=null,metrics:PetUsage|null=null,readError='';
 function draw(){
  const task=State.focusTask;if(!task)return;
  name.textContent=task.name;status.textContent=states[task.state]??task.state;status.dataset.state=task.state;
  chat.textContent=['approval','question'].includes(task.state)?'Rever pedido':'Conversar';
  model.textContent=provider?profileSummary(provider,activeModel({...State.settings,chatProvider:provider as typeof State.chatProvider}))||'Modelo definido no pet':'Integração';
  model.title=model.textContent;
  const attachment=provider?State.fileFor(provider):null;
  activity.textContent=attachment?`Documento: ${attachment.name}`:task.sessionId?`${task.sessionOwned?'Conversa do pet':'Terminal'} · ${task.sessionTitle||task.sessionCwd||'#'+task.sessionId.slice(0,8)}`:task.finalLine||task.steps.at(-1)||'Pronto para uma conversa. Arrasta um documento para o pet ou para este cartão.';
  activity.title=activity.textContent;
  const token=metrics?.measured&&metrics.totalTokens!=null;
  usage.replaceChildren();
  if(provider==='ollama'){
   for(const [label,value]of [['Tokens de entrada',number(metrics?.inputTokens)],['Tokens gerados',number(metrics?.outputTokens)],['Tempo',metrics?.durationMs==null?'—':`${(metrics.durationMs/1000).toLocaleString('pt-PT',{maximumFractionDigits:1})} s`]])usage.append(h('div',{},h('strong',{text:value}),h('span',{text:label})));
   usage.title=token?`${number(metrics!.totalTokens)} tokens · ${metrics!.scope}${metrics?.model?' · '+metrics.model:''}`:'Sem medição disponível para esta conversa. As contagens aparecem após uma resposta medida pelo Ollama.';
  }else{
   usage.append(h('div',{},h('strong',{text:rows?String(rows.length):'—'}),h('span',{text:'Conversas guardadas'})),h('div',{},h('strong',{text:String(State.petSessions.list(task.id).length)}),h('span',{text:'Sessões de terminal'})));
  }
  sessions.replaceChildren();
  const external=State.petSessions.current(task.id,State.pendingApproval);
  if(external)sessions.append(h('button',{class:'pet-summary-session',text:`↗ ${external.sessionTitle||external.sessionCwd||'Sessão de terminal'} · #${external.sessionId.slice(0,8)}`,onclick:()=>actions.openTarget()}));
  if(rows?.length){const current=rows.find(r=>r.current)??rows[0];const detail=provider==='ollama'&&metrics?.measured?`Última resposta · ${metrics.model||'Ollama'}${token?' · '+number(metrics.totalTokens)+' tokens':''}`:`${current.title||'Conversa atual'} · ${current.messageCount} mensagens`;sessions.append(h('span',{text:detail,title:detail}));}
  else if(provider==='ollama')sessions.append(h('span',{text:reading?'A atualizar informação…':'Os tokens são medidos pelo Ollama após cada resposta.'}));
  error.textContent=readError;error.hidden=!readError;chat.hidden=history.hidden=!provider;refresh.disabled=reading;
 }
 async function read(){
  if(reading||!provider)return;reading=true;lastRead=Date.now();const ticket=version,p=provider;draw();
  const results=await Promise.allSettled([
   Bridge.chatConversations(p).then(async value=>{
    if(ticket!==version)return;rows=value;draw();
    const id=value.find(row=>row.current)?.id;
    if(!id)return;const measured=await Bridge.petUsage(p,id);
    if(ticket===version&&rows?.find(row=>row.current)?.id===id){metrics=measured;draw();}
   }),
  ]);
  if(ticket!==version)return;
  reading=false;
  readError=results[0].status==='rejected'?'Não foi possível atualizar as conversas. Tenta Atualizar.':'';
  draw();
 }
 function sync(){
  const task=State.focusTask;if(!task)return;
  if(task.id!==owner){owner=task.id;provider=providerForPill(owner)??'';version++;reading=false;rows=null;metrics=null;lastRead=0;readError='';avatar.replaceChildren(createMiniBot(task,44));}
  const spotify=task.id==='integration_spotify';musicWrap.hidden=!spotify;content.hidden=spotify;
  if(spotify){music.sync();return;}
  draw();if(State.mode==='expanded'&&State.view==='overview'&&Date.now()-lastRead>5000)void read();
 }
 void onEvent<string>('assistant-history-changed',value=>{
  if(value!==provider)return;
  version++;reading=false;lastRead=0;rows=null;metrics=null;
  if(State.view==='overview')sync();
 });
 void onEvent<{provider:string}>('pet-usage-changed',value=>{if(value.provider!==provider)return;version++;reading=false;lastRead=0;metrics=null;if(State.view==='overview')sync();});
 PetPreferences.subscribe(sync);
 return {el,sync};
}
