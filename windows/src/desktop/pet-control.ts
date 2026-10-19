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
interface Context{pet?:string;section?:string}
const surface=document.getElementById('pet-control')!,residents=document.getElementById('residents')!,compact=document.getElementById('compact')!,grid=document.getElementById('compact-pets')!,manager=document.getElementById('manager')!;
const buttons=new Map<string,HTMLButtonElement>(),minis=new Map<string,HTMLElement>();let pets:PetIdentity[]=[],moving=false,enabled=false,lastShape='',ctx:HubContext|undefined,current:Context|undefined;
let nativeHome:Awaited<ReturnType<typeof createControlHome>>|undefined,managementOpen=false,detailOpen=false,systemPage=false;
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
function showPage(system:boolean){systemPage=system;revealCenter();expandGeometry();void emitToWindow('pet-control','pet-control-visible',!system&&managementOpen);if(!matchMedia('(prefers-reduced-motion: reduce)').matches)requestAnimationFrame(()=>{const pane=system?surface.querySelector('.system-center-actions'):managementOpen?manager:detailOpen?nativeHome?.el:gallery.el;pane?.animate?.([{opacity:.5,transform:`translateX(${system?14:-14}px)`},{opacity:1,transform:'translateX(0)'}],{duration:200,easing:'ease-out'});});}
function stepPage(direction:number){if(direction>0&&!systemPage&&!systemPageButton.hidden)showPage(true);else if(direction<0&&systemPage)showPage(false);}
pages.onkeydown=e=>{if(e.key==='ArrowLeft'||e.key==='ArrowRight'){e.preventDefault();stepPage(e.key==='ArrowRight'?1:-1);(systemPage?systemPageButton:petPage).focus();}};

let swipeTotal=0,swipeAt=0,swipeUntil=0;
function stepArea(direction:number){const index=areas.findIndex(([id])=>areaButtons.get(id)?.dataset.active==='true'),next=areas[Math.max(0,Math.min(areas.length-1,index+direction))];if(next&&next[0]!==areas[index]?.[0])areaButtons.get(next[0])?.click();}
surface.addEventListener('wheel',e=>{if(Math.abs(e.deltaX)<Math.abs(e.deltaY)*1.3||Math.abs(e.deltaX)<3||(e.target as HTMLElement).closest('input,textarea,select,pre,#residents,[contenteditable=true]'))return;const now=performance.now();if(now<swipeUntil)return;e.preventDefault();if(now-swipeAt>250)swipeTotal=0;swipeAt=now;swipeTotal+=e.deltaX;if(Math.abs(swipeTotal)>65){stepPage(swipeTotal>0?1:-1);swipeTotal=0;swipeUntil=now+650;}},{passive:false});
let touch:{x:number;y:number}|null=null;
surface.addEventListener('touchstart',e=>{if(!(e.target as HTMLElement).closest('input,textarea,select,#residents')&&e.touches.length===1)touch={x:e.touches[0].clientX,y:e.touches[0].clientY};else touch=null;},{passive:true});
surface.addEventListener('touchend',e=>{if(!touch||!e.changedTouches.length)return;const dx=e.changedTouches[0].clientX-touch.x,dy=e.changedTouches[0].clientY-touch.y;touch=null;if(Math.abs(dx)>75&&Math.abs(dy)<45){e.preventDefault();stepPage(dx<0?1:-1);}},{passive:false});surface.addEventListener('touchcancel',()=>{touch=null;});
navigation.addEventListener('keydown',e=>{if(e.key!=='ArrowLeft'&&e.key!=='ArrowRight')return;e.preventDefault();stepArea(e.key==='ArrowRight'?1:-1);[...areaButtons.values()].find(b=>b.dataset.active==='true')?.focus();});
const gallery=petGallery(p=>{if(p.attention){void emitToWindow('companions','companions-present',{id:p.id});fsm.forcePetit();return;}nativeHome?.select(p.id);detailOpen=true;managementOpen=false;selectArea('overview');revealCenter();expandGeometry();},p=>void management({pet:p.id,section:'sessions'}));surface.append(gallery.el);
const detailBack=document.createElement('button');detailBack.id='center-detail-back';detailBack.textContent='← Todos os pets';detailBack.onclick=()=>overview();surface.append(detailBack);
let explicitNavigation=false;function revealCenter(){explicitNavigation=true;fsm.forceHome();explicitNavigation=false;}
const w=new Tracked(288),height=new Tracked(32),radius=new Tracked(14),fsm=new IslandStateMachine();
const desktopCenter=systemCenter(surface,compact,()=>{if(fsm.state==='home')expandGeometry();else{w.springTo(desktopCenter.width);frames.wake();}});
const frames=new FrameLoop((dt,now)=>{
 w.step(dt,now);height.step(dt,now);radius.step(dt,now);tickMiniBots(dt);
 if(!managementOpen&&detailOpen&&fsm.state==='home')nativeHome?.tick(now);
 const width=Math.max(0,Math.min(innerWidth,w.value)),h=Math.max(0,Math.min(innerHeight,height.value));
 surface.style.width=`${width}px`;surface.style.height=`${h}px`;surface.style.borderRadius=`0 0 ${radius.value}px ${radius.value}px`;
 const rect=enabled?(fsm.state==='hidden'?{x:(innerWidth-240)/2,y:0,w:240,h:6}:{x:(innerWidth-width)/2,y:0,w:width,h}):{x:0,y:0,w:0,h:0};
 const expanded=fsm.state==='home',key=JSON.stringify([Math.round(rect.x),Math.round(rect.y),Math.round(rect.w),Math.round(rect.h),expanded]);
 if(key!==lastShape){lastShape=key;void Bridge.petControlInput(rect,expanded);}
 return w.animating||height.animating||radius.animating||((moving||fsm.state==='home'&&!managementOpen)&&!document.hidden);
});
const task=(p:PetIdentity):AgentTask=>({id:p.id,name:p.name,color:p.color,state:p.state,source:'agent',isIntegration:true,steps:[],stepIndex:0});
fsm.homeToPetitDelay=3;
function expandGeometry(){
 surface.dataset.page=systemPage?'system':'pets';petPage.setAttribute('aria-current',systemPage?'false':'page');systemPageButton.setAttribute('aria-current',systemPage?'page':'false');
 surface.dataset.content=managementOpen?'management':detailOpen?'detail':'dashboard';
 gallery.el.hidden=systemPage||managementOpen||detailOpen;detailBack.hidden=systemPage||managementOpen||!detailOpen;
 manager.hidden=systemPage||!managementOpen;document.getElementById('control-header')!.hidden=false;
 if(nativeHome)nativeHome.el.hidden=systemPage||managementOpen||!detailOpen;
 nativeHome?.visible(!systemPage&&!managementOpen&&detailOpen&&fsm.state==='home');
 w.springTo(Math.min(!systemPage&&managementOpen?760:640,innerWidth));height.springTo(Math.min(systemPage?376+Math.max(0,desktopCenter.height-156):managementOpen?640:detailOpen?476:innerWidth<500?640:490,innerHeight));radius.springTo(22);frames.wake();
}
fsm.onTransition=(_from,to)=>{
 const expanded=to==='home';
 if(expanded&&!explicitNavigation){systemPage=false;managementOpen=false;detailOpen=false;selectArea('overview');}if(!expanded)fsm.pinned=false;surface.dataset.mode=to;
 if(expanded)expandGeometry();
 else{w.curveTowards(desktopCenter.width);height.curveTowards(to==='hidden'?0:32);radius.curveTowards(14);}
 if(!expanded)nativeHome?.visible(false);
 void emitToWindow('pet-control','pet-control-visible',expanded&&!systemPage&&managementOpen);frames.wake();
};
async function management(value:Context={}){
 const section=value.section??'sessions';systemPage=false;managementOpen=true;selectArea(section);revealCenter();expandGeometry();void emitToWindow('pet-control','pet-control-visible',true);
 if(navigateManagement){navigateManagement(section,value.pet);current=value;return;}
 if(current)return;current=value;
 ctx={petSelection:value.pet,petSection:section,initialVisible:true,visibilityEvent:'pet-control-visible'}as HubContext;
 try{await petsPage(manager,ctx,{embedded:true,bindNavigation:navigate=>{navigateManagement=navigate;},onSectionChange:selectArea});}
 catch(error){ctx?.onLeave?.();current=undefined;navigateManagement=undefined;manager.replaceChildren();const message=document.createElement('p');message.className='center-load-error';message.textContent=String(error).replace(/^Error:\s*/,'');const retry=document.createElement('button');retry.textContent='Tentar novamente';retry.onclick=()=>void management(value);manager.append(message,retry);}
}
function overview(){systemPage=false;managementOpen=false;detailOpen=false;selectArea('overview');revealCenter();expandGeometry();void emitToWindow('pet-control','pet-control-visible',false);}
compact.onclick=overview;
surface.onpointerenter=()=>{fsm.mouseEntered();frames.wake();};surface.onpointerleave=()=>{fsm.mouseLeft();frames.wake();};
surface.addEventListener('pointerdown',()=>fsm.userInteracted());
surface.addEventListener('focusin',e=>{if((e.target as HTMLElement).matches('input,textarea,select'))fsm.pinned=true;});
surface.addEventListener('focusout',()=>{fsm.pinned=false;if(!surface.matches(':hover'))fsm.mouseLeft();});
document.getElementById('wake')!.onpointerenter=()=>fsm.mouseEntered();
document.getElementById('manage')!.onclick=()=>void management({section:'profiles'});
document.getElementById('fold')!.onclick=()=>fsm.forcePetit();pages.append(document.getElementById('fold')!);
document.getElementById('manage')!.hidden=true;
const attach=document.createElement('button');attach.id='center-attach';attach.type='button';attach.title='Anexar documento ao pet selecionado';attach.setAttribute('aria-label',attach.title);attach.innerHTML='<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>';attach.onclick=()=>{const pet=nativeHome?.selected;if(pet){void emitToWindow('island',PET_EVENTS.summon,{pillId:pet,view:'upload'});fsm.forcePetit();}};document.getElementById('control-header')!.append(attach);
document.addEventListener('keydown',e=>{if(e.key==='Escape'){e.preventDefault();fsm.forcePetit();}});
nativeHome=await createControlHome(pet=>void management({pet,section:'profiles'}),()=>fsm.forcePetit(),()=>frames.wake());surface.append(nativeHome.el);
await onEvent<Context>('pet-control-opened',value=>void management(value));
await onEvent('pet-control-collapse',()=>fsm.forcePetit());
await onEvent<boolean>('pet-control-pointer-inside',inside=>{
 if(inside)fsm.mouseEntered();else fsm.mouseLeft();
 frames.wake();
});
await onEvent<CompanionsSnapshot>(PET_EVENTS.state,s=>{
 enabled=s.enabled;surface.hidden=!enabled;pets=s.catalog??s.pets;moving=s.enabled&&s.motion&&!s.paused;
 desktopCenter.setEnabled(!!s.control?.systemCenter);systemPageButton.hidden=!s.control?.systemCenter;
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
   b.onclick=()=>{nativeHome?.select(p.id);for(const[id,button]of buttons)button.setAttribute('aria-pressed',String(id===p.id));if(managementOpen)navigateManagement?.(current?.section??'sessions',p.id);else{detailOpen=true;revealCenter();expandGeometry();}};
   b.oncontextmenu=e=>{e.preventDefault();void management({pet:p.id,section:'sessions'});};buttons.set(p.id,b);
   const mini=createMiniBot(task(p),14,()=>pets.some(v=>v.id===p.id&&v.dancing));minis.set(p.id,mini);grid.append(mini);
  }
  b.querySelector('.name')!.textContent=p.name;b.querySelector('.count')!.textContent=p.sessions?String(p.sessions):'';
  b.dataset.attention=String(p.attention);b.dataset.busy=String(p.busy);b.style.setProperty('--pet-color',p.color);
  (b.querySelector('.status')as HTMLElement).dataset.active=String(p.active);
  b.setAttribute('aria-pressed',String(p.id===nativeHome?.selected));
  const state=p.attention?'Precisa de atenção':p.busy?'A trabalhar':p.active?'Disponível · sessão ativa':'Disponível';
  b.title=`${p.name} · ${state} · ${p.sessions??0} sessões\nClique: selecionar pet · botão direito: sessões`;
  b.setAttribute('aria-label',`${p.name} · ${state} · ${p.sessions??0} sessões`);residents.append(b);
 }
 const attention=pets.filter(p=>p.attention).length,busy=pets.filter(p=>p.busy).length;
 document.getElementById('summary')!.textContent=attention?`${attention} a aguardar`:busy?`${busy} a trabalhar`:'Pets';
 pruneMiniBots();syncMiniBotStates(pets.map(task));if(enabled&&fsm.state==='hidden')fsm.reveal();frames.wake();
});
document.addEventListener('visibilitychange',()=>{if(!document.hidden)frames.wake();});
window.addEventListener('resize',()=>{if(fsm.state==='home')expandGeometry();else frames.wake();});
fsm.forcePetit();await emitToWindow('island',PET_EVENTS.ready,{});
const initial=await Bridge.petControlContext();if(initial)void management(initial);
