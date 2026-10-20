/** The control island uses Coucou's header and the selected pet's dashboard.
 * It observes the main window's public state, without starting hook controllers
 * or another chat/approval owner. Management keeps its existing complete UI. */
import {Bridge,onEvent,emitToWindow} from '../core/bridge';
import {State,type AgentTask} from '../core/state';
import {Spotify} from '../core/spotify';
import {Sound} from '../core/sound';
import {buildHeader} from '../views/views';
import {buildPetOverview} from '../views/pet-overview';
import {createMiniBot,pruneMiniBots} from '../mochi/minibots';
import {PetPreferences} from '../mochi/pet-preferences';
import {PET_EVENTS,type CompanionsSnapshot,type PetIdentity} from '../mochi/companions';
import {OllamaModelSelection} from '../core/ollama-selection';

export async function createControlHome(manage:(pet?:string)=>void,collapse:()=>void,wake:()=>void){
 State.petCardsOnly=true;
 const boot=await Bridge.boot();let nativeCompanionMode=boot?.settings.companionMode??State.settings.companionMode;State.settings={...State.settings,...boot?.settings,companionMode:true};await PetPreferences.init();
 State.mode='expanded';State.view='overview';
 const el=document.createElement('section');el.id='native-control-home';
 const avatar=document.createElement('div');avatar.id='native-control-avatar';let avatarOwner='';
 const openChat=()=>{
  const task=State.focusTask;if(!task||task.id==='integration_spotify')return;
  const s=task.sessionId&&task.sessionBackend?{id:task.sessionId,backend:task.sessionBackend,title:task.sessionTitle??'',cwd:task.sessionCwd??'',steps:[],finalLine:''}:undefined;
  void Bridge.petChatOpen(task.id,s);
 };
 const header=buildHeader({setView:view=>{
  if(view==='settings')manage();
  else if(view==='upload'){const task=State.focusTask;if(task)void emitToWindow('island',PET_EVENTS.summon,{pillId:task.id,view:'upload'});collapse();}
  else if(view!=='overview'&&view!=='empty')openChat();
 },blip:()=>Sound.play('blip'),collapse,toggleSound:()=>{
  State.settings={...State.settings,soundEnabled:!State.settings.soundEnabled};Sound.setEnabled(State.settings.soundEnabled);void Bridge.saveSettings({...State.settings,companionMode:nativeCompanionMode}).catch(Bridge.reportSettingsError);header.sync();
 }});
 header.el.querySelector('.header-settings')!.setAttribute('aria-label','Gerir pets, sessões e modelos');
 const overview=buildPetOverview({setFocus:id=>{State.setFocus(id);syncViews();wake();},openTarget:openChat,blip:()=>Sound.play('blip'),openSettingsWindow:manage,openPetSettings:manage});
 overview.el.classList.add('on');const views=document.createElement('div');views.className='native-control-views';views.append(overview.el,avatar);el.append(header.el,views);
 function syncViews(){
  header.sync();overview.sync();const task=State.focusTask;
  if(task&&task.id!==avatarOwner){avatarOwner=task.id;avatar.replaceChildren(createMiniBot(task,58,()=>Spotify.state.playing&&State.focusId==='integration_spotify'));pruneMiniBots();}
  avatar.hidden=!task;
 }
 State.subscribe(syncViews);
 function task(p:PetIdentity):AgentTask{return {id:p.id,name:p.name,color:p.color,state:p.state,source:'agent',isIntegration:true,steps:p.message?[p.message]:[],stepIndex:0,
  sessionId:p.session?.id,sessionBackend:p.session?.backend,sessionCwd:p.session?.cwd,sessionTitle:p.session?.title,sessionOwned:p.session?.owned,decisionInTerminal:p.session?.decisionInTerminal,finalLine:p.session?.result};}
 await onEvent<typeof State.settings>('settings-changed',settings=>{nativeCompanionMode=settings.companionMode;State.settings=OllamaModelSelection.applySettings({...settings,companionMode:true});syncViews();wake();});
 return {el,get selected(){return State.focusId;},select(id:string){if(State.tasks.some(p=>p.id===id)){State.setFocus(id);syncViews();wake();}},sync(snapshot:CompanionsSnapshot){
  const pets=snapshot.catalog??snapshot.pets;
  State.tasks=pets.map(p=>{const live=snapshot.control?.tasks?.find(t=>t.id===p.id);return live?{...live,name:p.name,color:p.color}:task(p);});
  if(!State.focusId||!State.tasks.some(t=>t.id===State.focusId))State.setFocus(snapshot.control?.focusId&&State.tasks.some(t=>t.id===snapshot.control!.focusId)?snapshot.control.focusId:State.tasks[0]?.id??null);
  if(snapshot.control?.integrations)State.integrations=snapshot.control.integrations;
  if(snapshot.control?.spotify)Spotify.state=snapshot.control.spotify;
  syncViews();
 },visible(visible:boolean){State.mode=visible?'expanded':'compact';State.notify();},tick(now:number){return overview.tick?.(now);}};
}
