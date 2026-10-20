import type {ControlNavigation} from '../core/control-navigation';
import {failureContext} from '../core/control-navigation';
import {SystemNetwork,type NetworkTransport} from '../core/system-network';
import {networkPage} from './system-network';
import {Bridge} from '../core/bridge';
import {systemDetails,type SystemCenterStatus,type SystemPanel,type SystemTrayItem} from '../core/system-center';
import {IslandSystemAudio} from '../core/system-audio';
import {buildSpotyCard} from '../views/spoty-audio';
export type {SystemCenterStatus} from '../core/system-center';
const make=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const el=document.createElement(tag);if(text!==undefined)el.textContent=text;return el;};
const icons:Record<string,string>={wifi:'M3 8a15 15 0 0 1 18 0M6 12a10 10 0 0 1 12 0m-9 4a5 5 0 0 1 6 0m-3 4h.01',bluetooth:'m7 7 10 10-5 4V3l5 4L7 17',audio:'M11 5 6 9H3v6h3l5 4V5m4 3a6 6 0 0 1 0 8m3-11a10 10 0 0 1 0 14',battery:'M3 7h16v10H3V7m18 3v4M6 10h7v4H6v-4',clipboard:'M8 5H5v16h14V5h-3M8 3h8v4H8V3m0 9h8m-8 4h5',disks:'M4 4h16v16H4V4m0 10h16M7 17h.01M10 17h.01',recording:'M4 4h16v13H4V4m5 17h6m-3-4v4M9 10a3 3 0 1 0 6 0 3 3 0 1 0-6 0',tray:'M4 4h6v6H4V4m10 0h6v6h-6V4M4 14h6v6H4v-6m10 0h6v6h-6v-6'};

/** System controls live here; application playback belongs to its owning pet. */
export interface SystemCenterNavigation {navigation?:ControlNavigation;navigateChild?:(commit:()=>void)=>void;collapse?:()=>void;networkTransport?:NetworkTransport}
export function systemCenter(surface:HTMLElement,compact:HTMLElement,changed:()=>void,options:SystemCenterNavigation={}){
 const clock=make('span');clock.className='system-center-clock';const time=make('time'),date=make('span');clock.append(time,date);compact.insertBefore(clock,compact.lastElementChild);
 const recordingMarkers=[compact,document.getElementById('center-pages')!].map(parent=>{const el=make('span');el.className='system-recording-indicator';el.hidden=true;el.setAttribute('role','status');const dot=make('i'),label=make('span');el.append(dot,label);parent.insertBefore(el,parent.lastElementChild);return{el,label};});
 const tools=make('section');tools.className='system-center-actions system-center-full';tools.hidden=true;tools.setAttribute('aria-label','Centro do sistema');
 const feedback=make('p');feedback.className='system-center-feedback';feedback.hidden=true;feedback.setAttribute('role','alert');
 const hero=make('header');hero.className='system-center-hero';
 const calendar=make('button');calendar.className='system-center-calendar';calendar.type='button';calendar.setAttribute('aria-label','Abrir calendário');
 const expandedClock=make('time'),expandedDate=make('span');calendar.append(expandedClock,expandedDate);
 const shortcuts=make('div');shortcuts.className='system-center-shortcuts';const notifications=make('button','Notificações'),quick=make('button','Definições rápidas');notifications.type=quick.type='button';shortcuts.append(notifications,quick);hero.append(calendar,shortcuts);
 const statusLine=make('p','A obter o estado do sistema…');statusLine.className='system-center-status';statusLine.setAttribute('role','status');
 const cards=make('div');cards.className='system-center-cards';
 const weather=make('button');weather.type='button';weather.className='system-center-weather';weather.setAttribute('aria-label','Abrir tempo e previsão');
 const weatherTitle=make('strong','Tempo'),weatherDetail=make('small','Estado indisponível');weather.append(make('span','TEMPO'),weatherTitle,weatherDetail);cards.append(weather);
 const controls=make('section');controls.className='system-settings';controls.append(make('h2','Ligações e controlos'));const settingGrid=make('div');settingGrid.className='system-settings-grid';controls.append(settingGrid);
 const settingCards=new Map<string,{button:HTMLButtonElement;detail:HTMLElement}>();
 const labels:Record<string,string>={wifi:'Wi-Fi e redes',bluetooth:'Bluetooth',audio:'Som',battery:'Bateria e energia',clipboard:'Área de transferência',disks:'Discos e dispositivos',recording:'Gravação do ecrã',tray:'Ícones de aplicações'};
 for(const panel of ['wifi','bluetooth','audio','battery','clipboard','disks','recording','tray'] as const){
  const button=make('button');button.type='button';button.dataset.panel=panel;button.setAttribute('aria-label',`Abrir ${labels[panel]}`);
  const icon=make('span');icon.className='system-setting-icon';icon.setAttribute('aria-hidden','true');icon.innerHTML=`<svg viewBox="0 0 24 24"><path d="${icons[panel]}"/></svg>`;
  const text=make('span'),detail=make('small','Estado indisponível');text.append(make('strong',labels[panel]),detail);button.append(icon,text);settingGrid.append(button);settingCards.set(panel,{button,detail});
 }
 const traySection=make('section');traySection.className='system-tray-section';const trayHeading=make('h2','Aplicações em segundo plano'),trayList=make('div'),trayEmpty=make('p','A obter os ícones das aplicações…');trayList.className='system-tray-list';trayEmpty.className='system-tray-empty';traySection.append(trayHeading,trayList,trayEmpty);
 const trayRows=new Map<string,{el:HTMLElement;name:HTMLElement;open:HTMLButtonElement;menu:HTMLButtonElement;item:SystemTrayItem}>();
 const overview=make('div');overview.className='system-center-overview';overview.append(hero,statusLine,cards,controls,traySection);
 const audioPage=make('section');audioPage.className='system-audio-page';audioPage.hidden=true;audioPage.setAttribute('aria-label','Som do sistema');
 const audioBack=make('button','← Voltar ao sistema');audioBack.type='button';audioBack.className='system-audio-back';audioPage.append(audioBack);
 tools.append(overview,audioPage);surface.insertBefore(tools,document.getElementById('control-header'));surface.append(feedback);
 let enabled=false,reading=false,generation=0,actionPending=false,lastStatus:SystemCenterStatus|null=null;
 let audio:ReturnType<typeof buildSpotyCard>|null=null,audioOpen=false,audioVisible=false,networkOpen=false,networkRevision='';
 const network=options.networkTransport&&options.navigation?new SystemNetwork(options.networkTransport,()=>options.navigation!.systemEligible?options.navigation!.context:null,error=>{const context=failureContext(error);if(context)options.navigation!.observe(context);}):null;
 const networks=network?networkPage(network,()=>child('overview')):null;if(networks){networks.el.hidden=true;tools.append(networks.el);}
 const systemVisible=()=>enabled&&surface.isConnected&&!surface.hidden&&!document.hidden&&surface.dataset.mode==='home'&&surface.dataset.page==='system';
 function syncAudio(){
  const visible=audioOpen&&systemVisible();
  if(audioVisible&&!visible)IslandSystemAudio.invalidate();audioVisible=visible;audio?.sync();
  network?.setActive(networkOpen&&systemVisible()&&networkRevision===options.navigation?.context?.revision);
 }
 function openAudio(value:boolean){
  if(value&&!systemVisible())return;audioOpen=value;networkOpen=false;overview.hidden=value;audioPage.hidden=!value;if(networks)networks.el.hidden=true;tools.scrollTop=0;feedback.hidden=true;
  if(value&&!audio){audio=buildSpotyCard({devicesOnly:true,controller:IslandSystemAudio,visible:()=>audioOpen&&systemVisible()});audioPage.append(audio.el);}
  syncAudio();if(!value&&document.hasFocus())settingCards.get('audio')?.button.focus();changed();
 }
 function child(page:'overview'|'audio'|'network'){
  const commit=()=>{if(page==='network'&&networks){audioOpen=false;networkOpen=true;networkRevision=options.navigation?.context?.revision??'';overview.hidden=true;audioPage.hidden=true;networks.el.hidden=false;tools.scrollTop=0;feedback.hidden=true;syncAudio();changed();}else openAudio(page==='audio');};
  if(options.navigateChild)options.navigateChild(commit);else commit();
 }
 audioBack.onclick=()=>child('overview');
 const unobserveNavigation=options.navigation?.subscribe(syncAudio);
 const observeVisibility=new MutationObserver(syncAudio);observeVisibility.observe(surface,{attributes:true,attributeFilter:['data-mode','data-page','hidden']});
 const locale=document.documentElement.lang||undefined;
 function actionButtons(){return [...tools.querySelectorAll<HTMLButtonElement>('button')];}
 async function perform(button:HTMLButtonElement,run:()=>Promise<unknown>){
  if(actionPending)return;actionPending=true;feedback.hidden=true;button.setAttribute('aria-busy','true');for(const b of actionButtons())b.disabled=true;changed();
  try{await run();options.collapse?.();}
  catch(error){feedback.textContent=String(error).replace(/^Error:\s*/,'');feedback.hidden=false;changed();}
  finally{actionPending=false;button.removeAttribute('aria-busy');for(const b of actionButtons())b.disabled=false;}
 }
 const action=(panel:SystemPanel,button:HTMLButtonElement)=>{button.onclick=()=>void perform(button,()=>Bridge.petSystemPanel(panel));};
 action('calendar',calendar);action('weather',weather);action('notifications',notifications);action('quick-settings',quick);
 for(const[panel,c]of settingCards){if(panel==='audio')c.button.onclick=()=>child('audio');else if(panel==='wifi')c.button.onclick=()=>child('network');else action(panel as SystemPanel,c.button);}
 function syncTray(items:SystemTrayItem[]|null|undefined){
  const ids=new Set(items?.map(i=>i.id));for(const[id,row]of trayRows)if(!ids.has(id)){row.el.remove();trayRows.delete(id);}
  let index=0;for(const item of items??[]){
   let row=trayRows.get(item.id);
   if(!row){const el=make('div'),name=make('strong'),open=make('button','Abrir'),menu=make('button','Menu');el.className='system-tray-item';open.type=menu.type='button';el.append(name,open,menu);row={el,name,open,menu,item};trayRows.set(item.id,row);
    open.onclick=()=>void perform(open,()=>Bridge.petSystemTray(item.id,'activate'));menu.onclick=()=>void perform(menu,()=>Bridge.petSystemTray(item.id,'menu'));
   }
   row.item=item;row.name.textContent=item.label||'Aplicação';row.open.hidden=item.onlyMenu;row.menu.hidden=!item.hasMenu&&!item.onlyMenu;row.open.setAttribute('aria-label',`Abrir ${item.label||'aplicação'}`);row.menu.setAttribute('aria-label',`Menu de ${item.label||'aplicação'}`);row.open.disabled=row.menu.disabled=actionPending;
   if(trayList.children[index]!==row.el)trayList.insertBefore(row.el,trayList.children[index]??null);index++;
  }
  trayEmpty.hidden=!!items?.length;trayEmpty.textContent=items?'Sem ícones de aplicações ativos.':'Estado das aplicações indisponível. Podes abrir os ícones no controlo acima.';
 }
 function renderStatus(status:SystemCenterStatus|null){
  const details=systemDetails(status);for(const[panel,card]of settingCards){card.detail.textContent=details[panel as keyof typeof details];card.button.title=card.detail.textContent;}
  if(status?.available){
   if(status.weather){const unit=status.weather.unit.toLowerCase();weatherTitle.textContent=`${Math.round(status.weather.temperature)}${unit==='c'||unit==='celsius'?'°C':unit==='f'||unit==='fahrenheit'?'°F':status.weather.unit||'°'}`;weatherDetail.textContent=[status.weather.location,status.weather.label].filter(Boolean).join(' · ');}
   else{weatherTitle.textContent='Ver previsão';weatherDetail.textContent='Abrir o tempo no DMS';}
   notifications.textContent=status.notifications===null?'Notificações':`Notificações · ${status.notifications}`;
  }else{weatherTitle.textContent='Tempo';weatherDetail.textContent='Estado indisponível';notifications.textContent='Notificações';}
  weather.title=[weatherTitle.textContent,weatherDetail.textContent].join(' · ');syncTray(status?.available?status.tray:null);
  renderRecording(status);
 }
 function renderRecording(status:SystemCenterStatus|null,stale=false){
  const recording=status?.recording,active=recording?.active===true&&!stale;
  for(const marker of recordingMarkers){marker.el.hidden=!enabled||!recording?.available||recording.active===false;marker.el.dataset.state=active?'active':'unknown';marker.label.textContent=active?'A gravar':'Por confirmar';marker.el.title=active?'Gravação do ecrã ativa':'Estado da gravação do ecrã por confirmar';marker.el.setAttribute('aria-label',marker.el.title);}
 }
 function renderClock(){
  if(!enabled)return;const now=new Date(),hours=new Intl.DateTimeFormat(locale,{hour:'2-digit',minute:'2-digit',hour12:false}).format(now),day=new Intl.DateTimeFormat(locale,{weekday:'short',day:'numeric',month:'short'}).format(now),full=new Intl.DateTimeFormat(locale,{weekday:'long',day:'numeric',month:'long'}).format(now);
  if(time.textContent!==hours)time.textContent=hours;if(date.textContent!==day)date.textContent=day;expandedClock.textContent=hours;expandedDate.textContent=full;expandedClock.dateTime=time.dateTime=now.toISOString();clock.title=calendar.title=new Intl.DateTimeFormat(locale,{dateStyle:'full',timeStyle:'short'}).format(now);
 }
 async function refresh(){
  if(!enabled||reading||document.hidden)return;reading=true;const ticket=generation;
  try{const status=await Bridge.petSystemStatus();if(!enabled||ticket!==generation)return;const previous=lastStatus;renderStatus(status);if(status.available)lastStatus=status;else renderRecording(previous,true);tools.dataset.stale=String(!status.available);statusLine.textContent=status.available?'Controlos do sistema'+(status.updatedAt?' · '+new Intl.DateTimeFormat(locale,{hour:'2-digit',minute:'2-digit'}).format(new Date(status.updatedAt)):''):'Estado indisponível. Podes continuar a abrir os painéis.';}
  catch{if(!enabled||ticket!==generation)return;tools.dataset.stale='true';renderRecording(lastStatus,true);statusLine.textContent=lastStatus?'Ligação ao sistema interrompida. Os valores mostram a última leitura.':'Não foi possível obter o estado do sistema. Podes continuar a abrir os painéis.';}
  finally{reading=false;}
 }
 const visibilityChanged=()=>{renderClock();void refresh();syncAudio();};
 clock.hidden=true;const clockTimer=window.setInterval(()=>{renderClock();void refresh();},3000);document.addEventListener('visibilitychange',visibilityChanged);
 return {get height(){return enabled?640+(feedback.hidden?0:56):0;},get width(){return enabled?432:288;},setEnabled(value:boolean){
  if(enabled===value)return;enabled=value;generation++;surface.dataset.systemCenter=String(value);clock.hidden=!value;tools.hidden=!value;feedback.hidden=true;syncAudio();renderRecording(lastStatus);surface.setAttribute('aria-label',value?'Centro dos pets e do sistema':'Centro de controlo dos pets');compact.setAttribute('aria-label',value?'Abrir centro dos pets e do sistema':'Abrir centro de controlo dos pets');renderClock();void refresh();changed();
 },resetPage(){openAudio(false);},dispose(){unobserveNavigation?.();network?.dispose();networks?.dispose();observeVisibility.disconnect();document.removeEventListener('visibilitychange',visibilityChanged);window.clearInterval(clockTimer);if(audioVisible)IslandSystemAudio.invalidate();audio?.dispose();}};
}
