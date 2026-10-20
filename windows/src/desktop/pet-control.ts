import {ControlNavigation,controlRoutes} from '../core/control-navigation';
import type {Route,ControlOpenedEvent} from '../core/control-contract';
import {PetObservability} from '../core/pet-observability-resource';
import {petObservation} from '../core/pet-observability';
import {scopedSessionSummary} from '../core/scoped-session-summary';
import {PetPreferences} from '../mochi/pet-preferences';
import '../style.css';import './pet-control.css';
import {Bridge,onEvent,emitToWindow} from '../core/bridge';
import {FrameLoop} from '../core/frame-loop';
import {Tracked} from '../core/anim';
import {IslandStateMachine} from '../island/fsm';
import {PET_EVENTS,type CompanionsSnapshot,type PetIdentity} from '../mochi/companions';
import {createMiniBot,pruneMiniBots,syncMiniBotStates,tickMiniBots} from '../mochi/minibots';
import {petsPage} from '../hub/pages/pets';
import type {HubContext} from '../hub/shell';
import type {AgentTask} from '../core/state';
import {createControlHome} from './pet-control-home';
import {systemCenter} from './system-center';
import {petGallery} from './pet-gallery';
import {controlDocuments} from './control-documents';
import {initializeRequestDestinations,requestDestinations,type RequestDestination} from '../core/request-destinations';
import type {SessionTarget} from '../core/session-chat';
interface Context{pet?:string;section?:string}
const controlNavigation=new ControlNavigation({context:()=>Bridge.petControlNavigationContext(),navigate:request=>Bridge.petControlNavigate(request)});
const surface=document.getElementById('pet-control')!,residents=document.getElementById('residents')!,compact=document.getElementById('compact')!,grid=document.getElementById('compact-pets')!,manager=document.getElementById('manager')!;
const buttons=new Map<string,HTMLButtonElement>(),minis=new Map<string,HTMLElement>();let pets:PetIdentity[]=[],moving=false,enabled=false,petsEnabled=false,systemEnabled=false,lastShape='',ctx:HubContext|undefined,current:Context|undefined;
function residentInfo(p:PetIdentity,b:HTMLButtonElement){
 const info=scopedSessionSummary(p.id==='integration_spotify'?null:petObservation(PetObservability.snapshot,p.id),!!PetObservability.error,PetPreferences.config?.pets[p.id]?.provider),count=b.querySelector<HTMLElement>('.count')!;
 count.textContent=info.badge;count.hidden=!info.badge||p.id==='integration_spotify';count.dataset.stale=String(info.stale);
 const state=p.attention?'Precisa de atenção':p.busy?'A trabalhar':p.state==='sleeping'?'Em descanso':'Disponível',description=p.id==='integration_spotify'?'Música e som do PC':info.detail;
 b.title=`${p.name} · ${state} · ${description}\nClique: selecionar pet · botão direito: sessões`;b.setAttribute('aria-label',`${p.name} · ${state} · ${description}`);
}
PetObservability.subscribe(()=>{for(const p of pets){const button=buttons.get(p.id);if(button)residentInfo(p,button);}});
let nativeHome:Awaited<ReturnType<typeof createControlHome>>|undefined,managementOpen=false,detailOpen=false,systemPage=false,explicitOpen=false;
let documentDrop:ReturnType<typeof controlDocuments>|undefined;
let keyboardAllowed=false,pointerInside:boolean|null=null;
initializeRequestDestinations();
let failedRequest:RequestDestination|null=null,requestErrorId='';
const requestFeedback=document.createElement('aside'),requestMessage=document.createElement('p'),requestRetry=document.createElement('button'),requestDismiss=document.createElement('button');
requestFeedback.id='center-request-error';requestFeedback.hidden=true;requestFeedback.setAttribute('role','alert');requestRetry.type=requestDismiss.type='button';requestRetry.textContent='Tentar abrir pedido';requestDismiss.textContent='Fechar aviso';requestFeedback.append(requestMessage,requestRetry,requestDismiss);surface.append(requestFeedback);
const clearRequestError=()=>{failedRequest=null;requestErrorId='';requestFeedback.hidden=true;if(fsm.state==='home')expandGeometry();};
requestDismiss.onclick=clearRequestError;
requestRetry.onclick=async()=>{
 const destination=failedRequest;if(!destination||!requestDestinations.current(destination)){clearRequestError();return;}
 requestRetry.disabled=true;
 try{const shown=await Bridge.petChatOpen(destination.target.petId,undefined,false,destination.target,destination.id);if(shown&&failedRequest===destination&&requestDestinations.current(destination))clearRequestError();}
 catch(error){if(failedRequest===destination&&requestDestinations.current(destination))requestMessage.textContent=String(error).replace(/^Error:\s*/,'');}
 finally{requestRetry.disabled=false;}
};
let navigateManagement:((section:string,pet?:string)=>void)|undefined;
const areaButtons=new Map<string,HTMLButtonElement>();
const navigation=document.createElement('nav');navigation.id='center-navigation';navigation.setAttribute('aria-label','Áreas do centro dos pets');
const areas=[['overview','Todos'],['sessions','Sessões'],['tasks','Tarefas'],['providers','Providers'],['models','Modelos'],['profiles','Perfis']] as const;
const icons:Record<string,string>={overview:'M3 10 12 3l9 7M5 9v12h5v-7h4v7h5V9',sessions:'M4 4h16v12H9l-5 4V4',tasks:'m4 6 2 2 3-4M12 6h8m-16 7 2 2 3-4m3 2h8M4 20h16',providers:'M12 3v6m0 6v6M3 12h6m6 0h6M9 9h6v6H9V9',models:'M4 5h16v14H4V5m4 4h8m-8 6h8',profiles:'M4 7h16M4 17h16M8 4v6m8 4v6'};
function selectArea(area:string){for(const[id,b]of areaButtons){b.setAttribute('aria-current',id===area?'page':'false');b.dataset.active=String(id===area);}}
for(const[id,label]of areas){const b=document.createElement('button');b.type='button';b.dataset.area=id;b.innerHTML=`<svg viewBox="0 0 24 24" aria-hidden="true"><path d="${icons[id]}"/></svg><span>${label}</span>`;b.onclick=()=>{const before=areas.findIndex(([key])=>areaButtons.get(key)?.dataset.active==='true');if(id==='overview')overview();else void management({section:id});if(!matchMedia('(prefers-reduced-motion: reduce)').matches)requestAnimationFrame(()=>{const pane=id==='overview'?gallery.el:manager;pane.animate?.([{opacity:.65,transform:`translateX(${areas.findIndex(([key])=>key===id)>before?10:-10}px)`},{opacity:1,transform:'translateX(0)'}],{duration:170,easing:'ease-out'});});};areaButtons.set(id,b);navigation.append(b);}
surface.insertBefore(navigation,document.getElementById('control-header'));selectArea('overview');
const pages=document.createElement('nav');pages.id='center-pages';pages.setAttribute('aria-label','Páginas da ilha');
const petPage=document.createElement('button'),systemPageButton=document.createElement('button');petPage.type=systemPageButton.type='button';petPage.dataset.page='pets';systemPageButton.dataset.page='system';petPage.textContent='Pets';systemPageButton.textContent='Sistema';petPage.onclick=()=>showPage(false);systemPageButton.onclick=()=>showPage(true);pages.append(petPage,systemPageButton);surface.prepend(pages);
const appSettings=document.createElement('button');appSettings.id='center-app-settings';appSettings.type='button';appSettings.title='Definições da aplicação';appSettings.setAttribute('aria-label',appSettings.title);appSettings.textContent='⚙';appSettings.onclick=()=>void Bridge.openSettingsWindow();pages.append(appSettings);
const systemDisabled=document.createElement('section');systemDisabled.id='system-disabled';systemDisabled.hidden=true;
const disabledTitle=document.createElement('h2'),disabledText=document.createElement('p'),disabledSettings=document.createElement('button');disabledTitle.textContent='Centro do sistema desativado';disabledText.textContent='Esta página não está ativa nesta configuração. Os controlos nativos do sistema continuam disponíveis.';disabledSettings.textContent='Abrir definições da aplicação';disabledSettings.onclick=()=>void Bridge.openSettingsWindow();systemDisabled.append(disabledTitle,disabledText,disabledSettings);surface.append(systemDisabled);
function showPage(system:boolean,accepted=false){if(!accepted){void navigateTo(system?'system':'home',()=>showPage(system,true));return;}if(pages.contains(document.activeElement))(system?systemPageButton:petPage).focus();systemPage=system;revealCenter();expandGeometry();void emitToWindow('pet-control','pet-control-visible',!system&&managementOpen);if(!matchMedia('(prefers-reduced-motion: reduce)').matches)requestAnimationFrame(()=>{const pane=system?surface.querySelector('.system-center-actions'):managementOpen?manager:detailOpen?nativeHome?.el:gallery.el;pane?.animate?.([{opacity:.5,transform:`translateX(${system?14:-14}px)`},{opacity:1,transform:'translateX(0)'}],{duration:200,easing:'ease-out'});});}
function stepPage(direction:number){if(direction>0&&!systemPage&&!systemPageButton.hidden)showPage(true);else if(direction<0&&systemPage)showPage(false);}
pages.onkeydown=e=>{if(e.key==='ArrowLeft'||e.key==='ArrowRight'){e.preventDefault();stepPage(e.key==='ArrowRight'?1:-1);(systemPage?systemPageButton:petPage).focus();}};

let swipeTotal=0,swipeAt=0,swipeUntil=0;
function stepArea(direction:number){const index=areas.findIndex(([id])=>areaButtons.get(id)?.dataset.active==='true'),next=areas[Math.max(0,Math.min(areas.length-1,index+direction))];if(next&&next[0]!==areas[index]?.[0])areaButtons.get(next[0])?.click();}
surface.addEventListener('wheel',e=>{if(Math.abs(e.deltaX)<Math.abs(e.deltaY)*1.3||Math.abs(e.deltaX)<3||(e.target as HTMLElement).closest('input,textarea,select,pre,#residents,[contenteditable=true]'))return;const now=performance.now();if(now<swipeUntil)return;e.preventDefault();if(now-swipeAt>250)swipeTotal=0;swipeAt=now;swipeTotal+=e.deltaX;if(Math.abs(swipeTotal)>65){stepPage(swipeTotal>0?1:-1);swipeTotal=0;swipeUntil=now+650;}},{passive:false});
let touch:{x:number;y:number}|null=null;
surface.addEventListener('touchstart',e=>{if(!(e.target as HTMLElement).closest('input,textarea,select,#residents')&&e.touches.length===1)touch={x:e.touches[0].clientX,y:e.touches[0].clientY};else touch=null;},{passive:true});
surface.addEventListener('touchend',e=>{if(!touch||!e.changedTouches.length)return;const dx=e.changedTouches[0].clientX-touch.x,dy=e.changedTouches[0].clientY-touch.y;touch=null;if(Math.abs(dx)>75&&Math.abs(dy)<45){e.preventDefault();stepPage(dx<0?1:-1);}},{passive:false});surface.addEventListener('touchcancel',()=>{touch=null;});
navigation.addEventListener('keydown',e=>{if(e.key!=='ArrowLeft'&&e.key!=='ArrowRight')return;e.preventDefault();stepArea(e.key==='ArrowRight'?1:-1);[...areaButtons.values()].find(b=>b.dataset.active==='true')?.focus();});
const gallery=petGallery(p=>{if(documentDrop?.choose(p.id))return;if(p.attention){void emitToWindow('island',PET_EVENTS.summon,{pillId:p.id,view:'overview'});closeCenter();return;}selectPet(p.id);},p=>void management({pet:p.id,section:'sessions'}));surface.append(gallery.el);
const detailBack=document.createElement('button');detailBack.id='center-detail-back';detailBack.textContent='← Todos os pets';detailBack.onclick=()=>overview();surface.append(detailBack);
let applyingContext=false;function revealCenter(){applyingContext=true;fsm.forceHome();applyingContext=false;}
const w=new Tracked(288),height=new Tracked(32),radius=new Tracked(14),fsm=new IslandStateMachine();
const desktopCenter=systemCenter(surface,compact,()=>{if(fsm.state==='home')expandGeometry();else{w.springTo(desktopCenter.width);frames.wake();}},{navigation:controlNavigation,networkTransport:{read:request=>Bridge.petSystemNetworkRead(request),view:request=>Bridge.petSystemNetworkView(request),scan:request=>Bridge.petSystemNetworkScan(request),ticket:request=>Bridge.petSystemNetworkTicket(request),cancel:request=>Bridge.petSystemNetworkCancel(request)},navigateChild:commit=>{void navigateTo('system',commit);},collapse:closeCenter});
const frames=new FrameLoop((dt,now)=>{
 w.step(dt,now);height.step(dt,now);radius.step(dt,now);tickMiniBots(dt);
 if(!managementOpen&&detailOpen&&fsm.state==='home')nativeHome?.tick(now);
 const width=Math.max(0,Math.min(innerWidth,w.value)),h=Math.max(0,Math.min(innerHeight,height.value));
 surface.style.width=`${width}px`;surface.style.height=`${h}px`;surface.style.borderRadius=`0 0 ${radius.value}px ${radius.value}px`;
 const expanded=fsm.state==='home',keyboard=expanded&&keyboardAllowed;
 const rect=enabled?(fsm.state==='hidden'?{x:(innerWidth-240)/2,y:0,w:240,h:6}:expanded?{x:(innerWidth-width)/2,y:0,w:width,h}:{x:(innerWidth-desktopCenter.width)/2,y:0,w:desktopCenter.width,h:32}):{x:0,y:0,w:0,h:0};
 const key=JSON.stringify([Math.round(rect.x),Math.round(rect.y),Math.round(rect.w),Math.round(rect.h),expanded,keyboard]);
 if(key!==lastShape){lastShape=key;void Bridge.petControlInput(rect,expanded,keyboard);}
 return w.animating||height.animating||radius.animating||((moving||fsm.state==='home'&&!managementOpen)&&!document.hidden);
});
const task=(p:PetIdentity):AgentTask=>({id:p.id,name:p.name,color:p.color,state:p.state,source:'agent',isIntegration:true,steps:[],stepIndex:0});
fsm.homeToPetitDelay=3;
function expandGeometry(){
 surface.dataset.page=systemPage?'system':'pets';petPage.setAttribute('aria-current',systemPage?'false':'page');systemPageButton.setAttribute('aria-current',systemPage?'page':'false');
 surface.dataset.content=managementOpen?'management':detailOpen?'detail':'dashboard';
 gallery.el.hidden=systemPage||managementOpen||detailOpen;if(!gallery.el.hidden)gallery.sync(pets);detailBack.hidden=systemPage||managementOpen||!detailOpen;
 manager.hidden=systemPage||!managementOpen;document.getElementById('control-header')!.hidden=false;
 if(nativeHome)nativeHome.el.hidden=systemPage||managementOpen||!detailOpen;
 systemDisabled.hidden=!systemPage||systemEnabled;
 nativeHome?.visible(!systemPage&&!managementOpen&&detailOpen&&fsm.state==='home');
 w.springTo(Math.min(!systemPage&&managementOpen?760:640,innerWidth));height.springTo(Math.min((systemPage?systemEnabled?desktopCenter.height:260:managementOpen?640:detailOpen?640:innerWidth<500?640:490)+(documentDrop?.height??0)+(requestFeedback.hidden?0:80),innerHeight));radius.springTo(22);frames.wake();
}
fsm.onTransition=(from,to)=>{
 if(!applyingContext&&to==='home'){fsm.state=from;fsm.cancelTimers();void navigateTo(petsEnabled?'home':'system');return;}
 if(!applyingContext&&from==='home'&&to!=='home')void controlNavigation.request({kind:'collapse'});
 if(to!=='home'&&explicitOpen){
  explicitOpen=false;enabled=petsEnabled||systemEnabled;surface.hidden=!enabled;
  desktopCenter.setEnabled(systemEnabled);systemPageButton.hidden=!systemEnabled;
  if(!enabled&&to!=='hidden'){fsm.forceHidden();return;}
 }
 if(to==='hidden'&&systemEnabled){applyingContext=true;fsm.forcePetit();applyingContext=false;return;}
 const expanded=to==='home';
 if(!expanded){fsm.pinned=false;keyboardAllowed=false;}surface.dataset.mode=to;
 if(expanded)expandGeometry();
 else{w.curveTowards(desktopCenter.width);height.curveTowards(to==='hidden'?0:32);radius.curveTowards(14);}
 if(!expanded)nativeHome?.visible(false);
 void emitToWindow('pet-control','pet-control-visible',expanded&&!systemPage&&managementOpen);frames.wake();
};
async function management(value:Context={},accepted=false){
 if(!accepted){const route=controlRoutes.includes(value.section as Route)?value.section as Route:'sessions';await navigateTo(route,()=>{void management(value,true);});return;}
 const section=value.section??'sessions';systemPage=false;managementOpen=true;selectArea(section);revealCenter();expandGeometry();void emitToWindow('pet-control','pet-control-visible',true);
 if(navigateManagement){navigateManagement(section,value.pet);current=value;return;}
 if(current)return;current=value;
 ctx={petSelection:value.pet,petSection:section,initialVisible:true,visibilityEvent:'pet-control-visible'}as HubContext;
 try{await petsPage(manager,ctx,{embedded:true,bindNavigation:navigate=>{navigateManagement=navigate;},onSectionChange:selectArea,requestSection:(section,commit)=>{void navigateTo(section as Route,commit);},collapse:closeCenter});}
 catch(error){ctx?.onLeave?.();current=undefined;navigateManagement=undefined;manager.replaceChildren();const message=document.createElement('p');message.className='center-load-error';message.textContent=String(error).replace(/^Error:\s*/,'');const retry=document.createElement('button');retry.textContent='Tentar novamente';retry.onclick=()=>void management(value);manager.append(message,retry);}
}
function overview(accepted=false){if(!accepted){void navigateTo('home',()=>overview(true));return;}systemPage=false;managementOpen=false;detailOpen=false;selectArea('overview');revealCenter();expandGeometry();void emitToWindow('pet-control','pet-control-visible',false);}
compact.onclick=()=>petsEnabled?overview():showPage(true);
// A closing input region can still emit enter for its former expanded area.
function leaveSurface(){
 keyboardAllowed=false;frames.wake();
 if(documentDrop?.holding)return;
 fsm.pinned=false;
 if(fsm.state==='home')closeCenter();else fsm.mouseLeft();
 frames.wake();
}
surface.onpointerenter=e=>{if(fsm.state!=='home'&&e.clientY>32)return;pointerInside=true;fsm.mouseEntered();frames.wake();};surface.onpointerleave=()=>{pointerInside=false;leaveSurface();};
surface.addEventListener('pointerdown',e=>{
 const target=e.target as Element|null;
 const label=target?.closest('label') as HTMLLabelElement|null;
 const control=target?.closest('button,input,textarea,select,a,[role="button"]')??label?.control;
 keyboardAllowed=!!control&&!control.matches(':disabled,[aria-disabled="true"]');
 if(keyboardAllowed)fsm.userInteracted();else fsm.pinned=!!documentDrop?.holding;
 frames.wake();
});
surface.addEventListener('focusin',e=>{if((e.target as HTMLElement).matches('input,textarea,select'))fsm.pinned=true;});
surface.addEventListener('focusout',()=>{fsm.pinned=!!documentDrop?.holding;if(pointerInside===false||pointerInside===null&&!surface.matches(':hover'))leaveSurface();});
document.getElementById('wake')!.onpointerenter=()=>fsm.mouseEntered();
document.getElementById('manage')!.onclick=()=>void management({section:'profiles'});
document.getElementById('fold')!.onclick=()=>{documentDrop?.cancel();closeCenter();};pages.append(document.getElementById('fold')!);
document.getElementById('manage')!.hidden=true;
const attach=document.createElement('button');attach.id='center-attach';attach.type='button';attach.title='Anexar documento ao pet selecionado';attach.setAttribute('aria-label',attach.title);attach.innerHTML='<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>';attach.onclick=()=>{const pet=nativeHome?.selected;if(pet){void emitToWindow('island',PET_EVENTS.summon,{pillId:pet,view:'upload'});closeCenter();}};document.getElementById('control-header')!.append(attach);
document.addEventListener('keydown',e=>{if(e.key==='Escape'){e.preventDefault();documentDrop?.cancel();closeCenter();}});
nativeHome=await createControlHome(pet=>void management({pet,section:'profiles'}),()=>closeCenter(),()=>{for(const[id,button]of buttons)button.setAttribute('aria-pressed',String(id===nativeHome?.selected));frames.wake();});surface.append(nativeHome.el);
documentDrop=controlDocuments(surface,{pets:()=>pets,selected:()=>!systemPage&&!managementOpen&&detailOpen?nativeHome?.selected??null:null,gallery:()=>{if((fsm.state!=='home'||systemPage||managementOpen||detailOpen)&&pendingCommit?.route!=='home')overview();},collapse:()=>closeCenter(),hold:value=>{fsm.pinned=value||surface.contains(document.activeElement)&&!!document.activeElement?.matches('input,textarea,select');if(!value&&pointerInside===false)queueMicrotask(()=>{if(pointerInside===false)leaveSurface();});},resize:()=>{if(fsm.state==='home')expandGeometry();}});
let renderedRevision='',localNavigation=0;let pendingCommit:{serial:number;route:Route}|null=null;
const navigationFeedback=document.createElement('p');navigationFeedback.id='center-navigation-error';navigationFeedback.hidden=true;navigationFeedback.setAttribute('role','alert');surface.append(navigationFeedback);
function paintCompact(){applyingContext=true;fsm.forcePetit();applyingContext=false;keyboardAllowed=false;frames.wake();}
function closeCenter(){localNavigation++;pendingCommit=null;paintCompact();void controlNavigation.request({kind:'collapse'});}
async function navigateTo(route:Route,commit?:()=>void){
 if(!controlRoutes.includes(route))return;
 const serial=++localNavigation;pendingCommit=commit?{serial,route}:null;
 const kind=controlNavigation.context?.mode.kind==='expanded'?'navigate':'expand';
 const result=await controlNavigation.request({kind,route});
 if(serial!==localNavigation)return;pendingCommit=null;
 if(result.accepted&&commit){renderedRevision=result.context!.revision;prepareExpanded();desktopCenter.resetPage();commit();}
 else renderContext();
}
function prepareExpanded(){explicitOpen=true;enabled=true;surface.hidden=false;desktopCenter.setEnabled(systemEnabled);systemPageButton.hidden=false;}

function selectPet(pet:string){void navigateTo('home',()=>{nativeHome?.select(pet);detailOpen=true;managementOpen=false;systemPage=false;selectArea('overview');revealCenter();expandGeometry();});}
function renderContext(){
 navigationFeedback.textContent=controlNavigation.error;navigationFeedback.hidden=!controlNavigation.error;surface.setAttribute('aria-busy',String(controlNavigation.busy));
 const context=controlNavigation.context;if(!context||controlNavigation.busy||context.pending||renderedRevision===context.revision)return;
 if(context.mode.kind==='expanded'&&pendingCommit?.route===context.mode.route)return;
 renderedRevision=context.revision;
 if(context.mode.kind!=='expanded'){paintCompact();return;}
 prepareExpanded();
 const route=context.mode.route;desktopCenter.resetPage();
 if(route==='home')overview(true);else if(route==='system')showPage(true,true);else void management({section:route,pet:current?.pet},true);
}
controlNavigation.subscribe(renderContext);
controlNavigation.onOpened=(event:ControlOpenedEvent)=>{if(event.pet&&event.context.mode.kind==='expanded'){if(event.context.mode.route==='home'){nativeHome?.select(event.pet);detailOpen=true;managementOpen=false;systemPage=false;selectArea('overview');revealCenter();expandGeometry();}else if(event.context.mode.route!=='system')void management({pet:event.pet,section:event.context.mode.route},true);}};
await onEvent('pet-control-navigation-context',value=>controlNavigation.observe(value));
await onEvent('pet-control-opened',value=>controlNavigation.receiveOpened(value));
await onEvent('pet-control-collapse',value=>controlNavigation.receiveCollapse(value));
await onEvent<{id:string;target?:SessionTarget;message:string}>('pet-control-request-error',value=>{
 if(requestDestinations.retired(value.id))return;
 const destination=value.target?requestDestinations.add({id:value.id,target:value.target}):null;
 if(value.target&&!destination)return;
 failedRequest=destination;requestErrorId=value.id;requestMessage.textContent=value.message;requestRetry.hidden=!destination;requestRetry.disabled=false;requestFeedback.hidden=false;if(fsm.state==='home')expandGeometry();
});
await onEvent<{id:string}>('agent-request-ended',({id})=>{if(requestErrorId===id)clearRequestError();});
await onEvent<boolean>('pet-control-pointer-inside',inside=>{
 // The native observer exists only while expanded. A queued observation
 // belongs to that old region and must never undo an explicit collapse.
 pointerInside=inside;if(inside){if(fsm.state==='home')fsm.mouseEntered();}else leaveSurface();
 frames.wake();
});
await onEvent<CompanionsSnapshot>(PET_EVENTS.state,s=>{
 const previousPets=petsEnabled,previousSystem=systemEnabled;
 petsEnabled=s.enabled&&s.control?.enabled!==false;systemEnabled=!!s.control?.systemCenter;enabled=petsEnabled||systemEnabled||explicitOpen;surface.hidden=!enabled;grid.hidden=!petsEnabled;pets=s.catalog??s.pets;moving=petsEnabled&&s.motion&&!s.paused;
 desktopCenter.setEnabled(systemEnabled);systemPageButton.hidden=!systemEnabled&&!explicitOpen;
 nativeHome?.sync(s);gallery.sync(pets);
 for(const[id,button]of buttons)button.setAttribute('aria-pressed',String(id===nativeHome?.selected));
 fsm.homeToPetitDelay=s.control?.autoCloseInterval??3;fsm.openOnHover=s.control?.openOnHover??false;
 const wanted=new Set(pets.map(p=>p.id));
 for(const[id,button]of buttons)if(!wanted.has(id)){button.remove();buttons.delete(id);minis.get(id)?.remove();minis.delete(id);}
 for(const p of pets){
  let b=buttons.get(p.id);
  if(!b){
   b=document.createElement('button');b.dataset.petId=p.id;b.append(createMiniBot(task(p),16,()=>pets.some(v=>v.id===p.id&&v.dancing)));
   const name=document.createElement('span');name.className='name';const count=document.createElement('span');count.className='count';const status=document.createElement('span');status.className='status';b.append(name,count,status);
   b.onclick=()=>{if(managementOpen)void management({section:current?.section??'sessions',pet:p.id});else selectPet(p.id);};
   b.oncontextmenu=e=>{e.preventDefault();void management({pet:p.id,section:'sessions'});};buttons.set(p.id,b);
   const mini=createMiniBot(task(p),14,()=>pets.some(v=>v.id===p.id&&v.dancing));minis.set(p.id,mini);grid.append(mini);
  }
  b.querySelector('.name')!.textContent=p.name;residentInfo(p,b);
  b.dataset.attention=String(p.attention);b.dataset.busy=String(p.busy);b.style.setProperty('--pet-color',p.color);
  (b.querySelector('.status')as HTMLElement).dataset.active=String(p.active);
  b.setAttribute('aria-pressed',String(p.id===nativeHome?.selected));
  residents.append(b);
 }
 const attention=pets.filter(p=>p.attention).length,busy=pets.filter(p=>p.busy).length;
 document.getElementById('summary')!.textContent=!petsEnabled?'Sistema':attention?`${attention} a aguardar`:busy?`${busy} a trabalhar`:'Pets';
 pruneMiniBots();syncMiniBotStates(pets.map(task));if(enabled&&fsm.state==='hidden')fsm.reveal();if(fsm.state==='home'&&(previousPets!==petsEnabled||previousSystem!==systemEnabled))expandGeometry();frames.wake();
});
document.addEventListener('visibilitychange',()=>{if(!document.hidden)frames.wake();});
window.addEventListener('resize',()=>{if(fsm.state==='home')expandGeometry();else frames.wake();});
paintCompact();await controlNavigation.start();await emitToWindow('island',PET_EVENTS.ready,{});
