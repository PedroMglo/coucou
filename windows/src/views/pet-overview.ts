import {buildAccountUsage} from './account-usage';
import {h} from './dom';
import {State} from '../core/state';
import {Bridge,onEvent,emitToWindow,type ConversationSummary} from '../core/bridge';
import {activeModel,providerForPill} from '../core/providers';
import {PetPreferences,profileSummary} from '../mochi/pet-preferences';
import {createMiniBot} from '../mochi/minibots';
import {buildSpotyCard} from './spoty-audio';
import type {ViewActions,ViewHost} from './views';
import type {PetUsage} from '../core/pet-usage';
import {PetCachedUsageResource,quotaProvider} from '../core/pet-cached-usage';
import {PetStationHistory,stationHistoryProvider} from '../core/station-history';
import {PetObservability,observePetObservability} from '../core/pet-observability-resource';
import {petObservation,type ObservedSession} from '../core/pet-observability';
import {buildPetObservability} from './pet-observability';
import {petName} from '../mochi/pet-preferences';
import {pillDefinition} from '../core/pills';
import {PROVIDERS} from '../core/providers';
import type {SessionTarget} from '../core/session-chat';

const states:Record<string,string>={idle:'Disponível',sleeping:'Em descanso',working:'A trabalhar',thinking:'A pensar',searching:'A pesquisar',approval:'Precisa de aprovação',question:'À espera da tua resposta',finished:'Concluído',error:'Precisa de atenção',ratelimit:'Limite atingido'};
const number=(n:number|null|undefined)=>n==null?'—':n.toLocaleString('pt-PT');

/** A pet card is a dashboard for the selected pet, rather than a cropped copy
 * of the original two-column notch. Nodes survive polling and live updates. */
export function buildPetOverview(actions:Pick<ViewActions,'setFocus'|'openTarget'|'blip'|'openSettingsWindow'|'openPetSettings'>,_knownSessions?:(pet:string)=>number|undefined):ViewHost {
 const avatar=h('div',{class:'pet-summary-avatar'}),name=h('strong'),status=h('span',{class:'pet-summary-status'});
 const model=h('p',{class:'pet-summary-model'}),activity=h('p',{class:'pet-summary-activity'});
 const usage=h('div',{class:'pet-summary-usage'}),sessions=h('div',{class:'pet-summary-sessions'});
 const error=h('p',{class:'pet-summary-error',role:'status'});
 const chat=h('button',{class:'pet-summary-primary',text:'Conversar'});
 const history=h('button',{text:'Conversas',title:'Abrir a lista de conversas e sessões no chat expansivo'});
 const settings=h('button',{text:'Configurar pet',onclick:()=>{const id=State.focusId;if(id){if(actions.openPetSettings)actions.openPetSettings(id);else void Bridge.openSettingsWindow('pets:'+id);}}});
 const refresh=h('button',{text:'↻','aria-label':'Atualizar informação do pet',title:'Atualizar sessões e consumo deste pet',onclick:()=>{lastRead=0;void PetObservability.refresh(true);const p=PetPreferences.config?.pets[owner]?.provider??provider;if(quotaProvider(p))void PetCachedUsageResource.refresh(true);if(stationHistoryProvider(p))void PetStationHistory.refresh(true);sync();}});
 const identity=h('div',{class:'pet-summary-identity'},avatar,h('div',{},h('div',{class:'pet-summary-heading'},name,status),model));
 const account=buildAccountUsage({compact:true,visible:()=>State.mode==='expanded'&&State.view==='overview'&&State.focusId===owner&&!document.hidden});
 const observation=buildPetObservability({sessions:()=>void openPanel('sessions'),library:()=>void openChat(owner,undefined,true),provider:()=>void openPanel('providers'),open:session=>void openObserved(session)},account.el);
 const lastTurn=h('details',{class:'pet-summary-last-turn'},h('summary',{text:'Último turno desta conversa'}),usage,sessions);
 const content=h('div',{class:'pet-summary-content'},identity,activity,h('div',{class:'pet-summary-actions'},chat,history,settings,refresh),error,observation.el,lastTurn);
 const music=buildSpotyCard(),musicWrap=h('div',{class:'pet-summary-music'},music.el);
 const el=h('div',{class:'view pet-overview'},content,musicWrap);
 let owner='',provider='',lastRead=0,reading=false,version=0,rows:ConversationSummary[]|null=null,metrics:PetUsage|null=null,readError='',usageError='',actionError='',opening=false,openingTicket=0;
 function sessionFor(task:{sessionId?:string|null;sessionBackend?:string;sessionTitle?:string;sessionCwd?:string|null}){
  return task.sessionId&&task.sessionBackend?{id:task.sessionId,backend:task.sessionBackend,title:task.sessionTitle??'',cwd:task.sessionCwd??'',steps:[],finalLine:''}:undefined;
 }
 async function openChat(pet:string,selected:ReturnType<typeof sessionFor>,showSessions=false,target?:SessionTarget){
  if(pet!==owner||State.focusId!==pet||opening)return;
  const ticket=++openingTicket;opening=true;actionError='';draw();
  try{if(await Bridge.petChatOpen(target?.petId??pet,selected,showSessions,target)!==true)throw Error('A conversa não abriu. Tenta novamente ou consulta as sessões.');}
  catch(e){if(ticket===openingTicket&&owner===pet)actionError=String(e).replace(/^Error:\s*/,'');}
  finally{if(ticket===openingTicket&&owner===pet){opening=false;draw();}}
 }
 async function openPanel(section:string){
  const pet=owner;if(State.focusId!==pet||opening)return;const ticket=++openingTicket;opening=true;actionError='';draw();
  try{await Bridge.petPanelOpen(360,50,pet,section);}catch(e){if(ticket===openingTicket&&owner===pet)actionError=String(e).replace(/^Error:\s*/,'');}
  finally{if(ticket===openingTicket&&owner===pet){opening=false;draw();}}
 }
 async function openObserved(session:ObservedSession){
  if(!session.canOpen||!session.target||State.focusId!==owner)return;
  await openChat(owner,undefined,false,Object.freeze({...session.target}));
 }
 function draw(){
  const task=State.focusTask;if(!task||task.id!==owner)return;
  const observed=petObservation(PetObservability.snapshot,owner,id=>petName(id,pillDefinition(id)?.name??id));
  if(observed)observed.providerName=String(PROVIDERS.find(p=>p.id===observed.provider)?.name??observed.provider);
  observation.sync(observed,PetObservability.loading,PetObservability.error,opening,{provider:observed?.provider??PetPreferences.config?.pets[owner]?.provider??'',snapshot:PetCachedUsageResource.snapshot,loading:PetCachedUsageResource.loading,error:PetCachedUsageResource.error},{snapshot:PetStationHistory.snapshot,loading:PetStationHistory.loading,error:PetStationHistory.error});
  const accountProvider=PetPreferences.config?.pets[owner]?.provider??observed?.provider??'';account.setContext(accountProvider,State.mode==='expanded'&&State.view==='overview',owner);
  name.textContent=task.name;status.textContent=states[task.state]??task.state;status.dataset.state=task.state;
  const pet=task.id,selected=sessionFor(task);
  chat.textContent=['approval','question'].includes(task.state)?'Rever pedido':'Conversar';
  chat.onclick=()=>{
   if(State.focusId!==pet)return;
   const current=State.focusTask;
   if(current&&(State.settings.companionMode||State.petCardsOnly)&&['approval','question'].includes(current.state))void emitToWindow('island','companions-summon',{pillId:pet,view:'overview'});
   else void openChat(pet,selected);
  };
  history.onclick=()=>void openChat(pet,selected,true);chat.disabled=history.disabled=opening;
  model.textContent=provider?profileSummary(provider,activeModel({...State.settings,chatProvider:provider as typeof State.chatProvider}))||'Modelo definido no pet':'Integração';
  model.title=model.textContent;
  const attachment=provider?State.fileFor(provider):null;
  activity.textContent=attachment?`Documento: ${attachment.name}`:task.sessionId?`${task.sessionOwned?'Conversa do pet':'Sessão externa'} · ${task.sessionTitle||task.sessionCwd||'#'+task.sessionId.slice(0,8)}`:task.finalLine||task.steps.at(-1)||'Pronto para uma conversa. Arrasta um documento para o pet ou para este cartão.';
  activity.title=activity.textContent;
  const measured=metrics?.measured?metrics:null,token=measured?.totalTokens!=null;
  const measuredScope=metrics?.scope==='partial-turn'?'Turno interrompido · medição parcial':metrics?.scope==='last-completed-turn'?'Último turno medido':'Medição desta conversa';
  usage.replaceChildren();
  if(provider==='ollama'){
   for(const [label,value]of [['Tokens de entrada',number(measured?.inputTokens)],['Tokens gerados',number(measured?.outputTokens)],['Duração',measured?.durationMs==null?'—':`${(measured.durationMs/1000).toLocaleString('pt-PT',{maximumFractionDigits:1})} s`]])usage.append(h('div',{},h('strong',{text:value}),h('span',{text:label})));
   usage.title=token?`${number(measured!.totalTokens)} tokens · ${measuredScope}${metrics?.model?' · '+metrics.model:''}`:'Última resposta desta conversa · sem medição disponível. Valores desconhecidos não representam zero consumo.';
  }
  lastTurn.hidden=provider!=='ollama';
  sessions.replaceChildren();
  const external=State.petSessions.current(task.id,State.pendingApproval);
  if(external)sessions.append(h('button',{class:'pet-summary-session',text:`Abrir chat · ${external.sessionTitle||external.sessionCwd||'Sessão externa'} · #${external.sessionId.slice(0,8)}`,onclick:()=>void openChat(pet,sessionFor(external))}));
  if(rows?.length){const current=rows.find(r=>r.current)??rows[0];const detail=provider==='ollama'&&metrics?.measured?`${measuredScope} · ${metrics.model||'Ollama'}${token?' · '+number(metrics.totalTokens)+' tokens':''}`:`${current.title||'Conversa atual'} · ${current.messageCount} mensagens`;sessions.append(h('span',{text:detail,title:detail}));}
  else if(provider==='ollama')sessions.append(h('span',{text:reading?'A atualizar informação…':'Os tokens são medidos pelo Ollama após cada resposta.'}));
  const message=actionError||readError||usageError;error.textContent=message;error.hidden=!message;chat.hidden=history.hidden=!provider;refresh.disabled=PetObservability.loading;
 }
 async function read(){
  if(reading||!provider)return;reading=true;lastRead=Date.now();const ticket=version,p=provider;draw();
  const results=await Promise.allSettled([
   Bridge.chatConversations(p).then(async value=>{
    if(ticket!==version)return;
    const id=value.find(row=>row.current)?.id;
    // A previous conversation's counts cannot label a newly current one,
    // including while its own measurement is loading or unavailable.
    if(id!==rows?.find(row=>row.current)?.id){metrics=null;usageError='';}
    rows=value;draw();
    if(!id||p!=='ollama')return;
    try{const measured=await Bridge.petUsage(p,id);
     if(ticket===version&&rows?.find(row=>row.current)?.id===id){metrics=measured;usageError='';draw();}
    }catch{if(ticket===version&&rows?.find(row=>row.current)?.id===id){metrics=null;usageError='Não foi possível atualizar a medição desta conversa. Tenta Atualizar.';draw();}}

   }),
  ]);
  if(ticket!==version)return;
  reading=false;
  readError=results[0].status==='rejected'?'Não foi possível atualizar as conversas. Tenta Atualizar.':'';
  draw();
 }
 function sync(){
  const task=State.focusTask;if(!task)return;
  if(task.id!==owner){owner=task.id;provider=providerForPill(owner)??'';version++;reading=false;rows=null;metrics=null;lastRead=0;readError=usageError=actionError='';opening=false;openingTicket++;avatar.replaceChildren(createMiniBot(task,44));}
  const spotify=task.id==='integration_spotify';musicWrap.hidden=!spotify;content.hidden=spotify;
  if(spotify){account.setContext('',false);music.sync();return;}
  draw();if(State.mode==='expanded'&&State.view==='overview'){void PetObservability.refresh();const p=PetPreferences.config?.pets[owner]?.provider??provider;if(quotaProvider(p))void PetCachedUsageResource.refresh();if(stationHistoryProvider(p))void PetStationHistory.refresh();}if(State.mode==='expanded'&&State.view==='overview'&&Date.now()-lastRead>5000)void read();
 }
 void onEvent<string>('assistant-history-changed',value=>{
  if(value!==provider)return;
  version++;reading=false;lastRead=0;rows=null;metrics=null;
  if(State.view==='overview')sync();
 });
 void onEvent<{provider:string}>('pet-usage-changed',value=>{if(value.provider!==provider)return;version++;reading=false;lastRead=0;metrics=null;if(State.view==='overview')sync();});
 const visible=()=>!!provider&&State.focusId!=='integration_spotify'&&!document.hidden&&State.mode==='expanded'&&State.view==='overview'&&(State.settings.companionMode||State.petCardsOnly);
 observePetObservability(visible,()=>{if(State.focusId===owner)draw();});
 window.setInterval(()=>{if(visible()){void PetObservability.refresh();if(stationHistoryProvider(PetPreferences.config?.pets[owner]?.provider??provider))void PetStationHistory.refresh();}},5000);
 PetCachedUsageResource.subscribe(()=>{if(visible()&&State.focusId===owner)draw();});
 PetStationHistory.subscribe(()=>{if(visible()&&State.focusId===owner)draw();});
 void onEvent<{schemaVersion:number}>('pet-station-history-changed',value=>{
  if(value?.schemaVersion===1)void PetStationHistory.invalidate(visible()&&stationHistoryProvider(PetPreferences.config?.pets[owner]?.provider??provider));
 });
 PetPreferences.subscribe(sync);
 return {el,sync};
}
