import {buildAccountUsage} from './account-usage';
import {h} from './dom';
import {State,type AgentTask} from '../core/state';
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
import {Ticker} from './ticker';
import {buildDiffCard} from './diff';
import {parseDiffStep} from '../core/diff';
import {renderIntegrationCard,type IntegrationCardDetail,type IntegrationCardHooks} from './integrations';
import {petRetired} from '../core/pet-factory';
import {petCapabilities} from '../mochi/capabilities';
import {language} from '../i18n/i18n';

const states:Record<string,string>={idle:'Disponível',sleeping:'Em descanso',working:'A trabalhar',thinking:'A pensar',searching:'A pesquisar',approval:'Precisa de aprovação',question:'À espera da tua resposta',finished:'Concluído',error:'Precisa de atenção',ratelimit:'Limite atingido'};
const number=(n:number|null|undefined)=>n==null?'—':n.toLocaleString('pt-PT');

/** A pet card is a dashboard for the selected pet, rather than a cropped copy
 * of the original two-column notch. Nodes survive polling and live updates. */
export function buildPetOverview(actions:Pick<ViewActions,'setFocus'|'openTarget'|'blip'|'openSettingsWindow'|'openPetSettings'>&{openDiff?:(task:AgentTask,id:number)=>void|Promise<void>},_knownSessions?:(pet:string)=>number|undefined):ViewHost {
 const avatar=h('div',{class:'pet-summary-avatar'}),name=h('strong'),status=h('span',{class:'pet-summary-status'});
 const model=h('p',{class:'pet-summary-model'}),activity=h('p',{class:'pet-summary-activity'});
 let activityContext='',activeDiffId:number|null=null,diffKey='';
 const closeDiff=()=>{activeDiffId=null;diffKey='';diffBody.replaceChildren();State.notify();};
 const latestDiff=(task:AgentTask|null)=>{if(!task)return null;const own=State.diffsFor(task.id).at(-1);if(own?.id!=null)return own.id;if(actions.openDiff){for(let i=task.steps.length-1;i>=0;i--){const step=parseDiffStep(task.steps[i]);if(step)return step.diffId;}}return null;};
 const openDiff=(id:number)=>{const task=State.focusTask;if(!task||task.id!==owner||!Number.isSafeInteger(id)||id<0)return;if(!State.findDiff(task.id,id)){if(actions.openDiff){const key=JSON.stringify([task.id,task.sessionBackend,task.sessionId]);void Promise.resolve(actions.openDiff({...task,steps:[...task.steps]},id)).catch(e=>{if(key===activityContext){actionError=String(e);draw();}});}else{actionError='A alteração já não está disponível nesta sessão.';draw();}return;}activeDiffId=id;actions.blip();State.notify();};
 const ticker=new Ticker(openDiff),diffBody=h('div',{class:'pet-summary-diff'});
 const lastEdit=h('button',{type:'button',text:'Ver última alteração',onclick:()=>{const id=latestDiff(State.focusTask);if(id!=null)openDiff(id);}});
 const live=h('section',{class:'pet-summary-live','aria-label':'Atividade da sessão'},ticker.el,lastEdit,diffBody);
 const usage=h('div',{class:'pet-summary-usage'}),sessions=h('div',{class:'pet-summary-sessions'});
 const error=h('p',{class:'pet-summary-error',role:'status'});
 const chat=h('button',{class:'pet-summary-primary',text:'Conversar'});
 const voice=h('button',{type:'button',class:'pet-summary-voice',text:'Conversar por voz',hidden:true,onclick:()=>void openVoice()});
 const history=h('button',{text:'Conversas',title:'Abrir a lista de conversas e sessões no chat expansivo'});
 const settings=h('button',{text:'Configurar pet',onclick:()=>{const id=State.focusId;if(id){if(actions.openPetSettings)actions.openPetSettings(id);else void Bridge.openSettingsWindow('pets:'+id);}}});
 const refresh=h('button',{text:'↻','aria-label':'Atualizar informação do pet',title:'Atualizar sessões e consumo deste pet',onclick:()=>{diagnostics.open=true;lastRead=0;void PetObservability.refresh(true);const p=PetPreferences.config?.pets[owner]?.provider??provider;if(quotaProvider(p))void PetCachedUsageResource.refresh(true);if(stationHistoryProvider(p))void PetStationHistory.refresh(true);sync();}});
 const identity=h('div',{class:'pet-summary-identity'},avatar,h('div',{},h('div',{class:'pet-summary-heading'},name,status),model));
 const diagnostics=h('details',{class:'pet-summary-diagnostics'},h('summary',{text:'Sessões, consumo e diagnóstico'}))as HTMLDetailsElement;
 const account=buildAccountUsage({compact:true,visible:()=>diagnostics.open&&State.mode==='expanded'&&State.view==='overview'&&State.focusId===owner&&!document.hidden});
 const observation=buildPetObservability({sessions:()=>void openPanel('sessions'),library:()=>void openChat(owner,undefined,true),provider:()=>void openPanel('providers'),open:session=>void openObserved(session)},account.el);
 diagnostics.append(observation.el);
 const lastTurn=h('details',{class:'pet-summary-last-turn'},h('summary',{text:'Último turno desta conversa'}),usage,sessions);
 const integration=h('section',{class:'pet-summary-integration','aria-label':'Atividade da integração'});
 const content=h('div',{class:'pet-summary-content'},identity,activity,integration,live,h('div',{class:'pet-summary-actions'},chat,voice,history,settings,refresh),error,diagnostics,lastTurn);
 const music=buildSpotyCard(),musicWrap=h('div',{class:'pet-summary-music'},music.el);
 const el=h('div',{class:'view pet-overview'},content,musicWrap);
 let owner='',provider='',lastRead=0,reading=false,version=0,rows:ConversationSummary[]|null=null,metrics:PetUsage|null=null,readError='',usageError='',actionError='',opening=false,openingTicket=0;
 let integrationKey='';
 let integrationDetail:IntegrationCardDetail|null=null;
 const integrationHooks:IntegrationCardHooks={
  get detail(){return integrationDetail;},
  openDetail(deploymentId){integrationDetail={deploymentId};integrationKey='';actions.blip();sync();State.notify();},
  closeDetail(){integrationDetail=null;integrationKey='';actions.blip();sync();State.notify();},
  openSettings(){void Bridge.openSettingsWindow('integrations');},
 };
 function syncIntegration(task:AgentTask){
  const shown=pillDefinition(task.id)?.category==='service'&&!petRetired(task.id);
  integration.hidden=!shown;
  if(!shown){if(integrationKey)integration.replaceChildren();integrationKey='';return false;}
  const info=State.integrations[task.id],native=info?.activity;
  // A selected assistant session shares the pet identity, but its output is
  // never an n8n workflow result. Use only the existing native notification.
  const source:AgentTask=task.id==='integration_n8n'?{...task,state:native?(native.success?'finished':'error'):'idle',steps:native?[native.label,...(native.detail?[native.detail]:[])]:[]}:task;
  const key=JSON.stringify([language(),task.id,task.color,integrationDetail,info,source.state,source.steps]);
  if(key!==integrationKey){integrationKey=key;integration.replaceChildren(renderIntegrationCard(source,integrationHooks));}
  return true;
 }
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
 async function openVoice(){
  const pet=owner,target=petCapabilities(pet).voice;
  if(State.focusId!==pet||!provider||!target||opening||State.mode!=='expanded'||State.view!=='overview')return;
  const ticket=++openingTicket;opening=true;actionError='';draw();
  try{await Bridge.voiceSessionStart(target);}
  catch(e){if(ticket===openingTicket&&owner===pet)actionError=String(e).replace(/^Error:\s*/,'');}
  finally{if(ticket===openingTicket&&owner===pet){opening=false;draw();}}
 }
 async function openObserved(session:ObservedSession){
  if(!session.canOpen||!session.target||State.focusId!==owner)return;
  await openChat(owner,undefined,false,Object.freeze({...session.target}));
 }
 function draw(){
  const task=State.focusTask;if(!task||task.id!==owner)return;
  const key=JSON.stringify([task.id,task.sessionBackend,task.sessionId]);
  if(key!==activityContext||State.view!=='overview'||State.mode!=='expanded'){activityContext=key;activeDiffId=null;}
  const diff=activeDiffId==null?null:State.findDiff(task.id,activeDiffId);
  if(!diff)activeDiffId=null;
  live.hidden=!task.steps.length&&!diff;ticker.el.hidden=!!diff;lastEdit.hidden=!!diff||latestDiff(task)===null;diffBody.hidden=!diff;
  ticker.sync(task);
  const nextDiff=diff?JSON.stringify([key,diff.id]):'';
  if(nextDiff!==diffKey){diffKey=nextDiff;diffBody.replaceChildren();if(diff)diffBody.append(buildDiffCard(diff,{dismiss:closeDiff,open:path=>void Bridge.openFileInVSCode(path)}));}
  const observed=petObservation(PetObservability.snapshot,owner,id=>petName(id,pillDefinition(id)?.name??id));
  if(observed)observed.providerName=String(PROVIDERS.find(p=>p.id===observed.provider)?.name??observed.provider);
  observation.sync(observed,PetObservability.loading,PetObservability.error,opening,{provider:observed?.provider??PetPreferences.config?.pets[owner]?.provider??'',snapshot:PetCachedUsageResource.snapshot,loading:PetCachedUsageResource.loading,error:PetCachedUsageResource.error},{snapshot:PetStationHistory.snapshot,loading:PetStationHistory.loading,error:PetStationHistory.error});
  diagnostics.hidden=lastTurn.hidden=refresh.hidden=!provider;
  const accountProvider=provider?(PetPreferences.config?.pets[owner]?.provider??observed?.provider??''):'';account.setContext(accountProvider,!!provider&&diagnostics.open&&State.mode==='expanded'&&State.view==='overview',owner);
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
  voice.hidden=!provider||!petCapabilities(pet).voice;voice.disabled=opening;
  voice.title=`Conversar por voz com ${task.name}`;voice.setAttribute('aria-label',voice.title);
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
  const service=syncIntegration(task);activity.hidden=service;if(service)live.hidden=true;
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
  if(task.id!==owner){owner=task.id;provider=providerForPill(owner)??'';diagnostics.open=lastTurn.open=false;integrationDetail=null;integrationKey='';version++;reading=false;rows=null;metrics=null;lastRead=0;readError=usageError=actionError='';opening=false;openingTicket++;avatar.replaceChildren(createMiniBot(task,44));}
  const spotify=task.id==='integration_spotify';musicWrap.hidden=!spotify;content.hidden=spotify;
  if(spotify){account.setContext('',false);music.sync();return;}
  draw();if(visible()){void PetObservability.refresh();const p=PetPreferences.config?.pets[owner]?.provider??provider;if(quotaProvider(p))void PetCachedUsageResource.refresh();if(stationHistoryProvider(p))void PetStationHistory.refresh();}if(State.mode==='expanded'&&State.view==='overview'&&(diagnostics.open||lastTurn.open)&&Date.now()-lastRead>5000)void read();
 }
 void onEvent<string>('assistant-history-changed',value=>{
  if(value!==provider)return;
  version++;reading=false;lastRead=0;rows=null;metrics=null;
  if(State.view==='overview')sync();
 });
 void onEvent<{provider:string}>('pet-usage-changed',value=>{if(value.provider!==provider)return;version++;reading=false;lastRead=0;metrics=null;if(State.view==='overview')sync();});
 const visible=()=>diagnostics.open&&!!provider&&State.focusId!=='integration_spotify'&&!document.hidden&&State.mode==='expanded'&&State.view==='overview'&&(State.settings.companionMode||State.petCardsOnly);
 diagnostics.addEventListener('toggle',sync);lastTurn.addEventListener('toggle',sync);
 observePetObservability(visible,()=>{if(State.focusId===owner)draw();});
 window.setInterval(()=>{if(visible()){void PetObservability.refresh();if(stationHistoryProvider(PetPreferences.config?.pets[owner]?.provider??provider))void PetStationHistory.refresh();}},5000);
 PetCachedUsageResource.subscribe(()=>{if(visible()&&State.focusId===owner)draw();});
 PetStationHistory.subscribe(()=>{if(visible()&&State.focusId===owner)draw();});
 void onEvent<{schemaVersion:number}>('pet-station-history-changed',value=>{
  if(value?.schemaVersion===1)void PetStationHistory.invalidate(visible()&&stationHistoryProvider(PetPreferences.config?.pets[owner]?.provider??provider));
 });
 PetPreferences.subscribe(sync);
 State.subscribe(()=>{if(State.view!=='overview'||State.mode!=='expanded')activeDiffId=null;});
 // Escape first leaves the diff, as in the original overview. A folded card
 // or another session cannot keep a previous session's selected edit.
 window.addEventListener('keydown',e=>{if(e.key==='Escape'&&activeDiffId!=null&&State.mode==='expanded'&&State.view==='overview'&&!el.hidden&&el.isConnected&&el.classList.contains('on')){e.stopImmediatePropagation();e.preventDefault();closeDiff();}},true);
 return {el,sync,tick(now){if(live.hidden||activeDiffId!=null)return false;ticker.tick(now);return ticker.animating;},command(command,diffId){if(command!=='toggleDiff')return;if(diffId!==undefined)openDiff(diffId);else if(activeDiffId!=null)closeDiff();else{const id=latestDiff(State.focusTask);if(id!=null)openDiff(id);}}};
}
