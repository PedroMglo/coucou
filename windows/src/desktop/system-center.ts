import {Bridge,emitToWindow} from '../core/bridge';

export interface SystemCenterStatus {
 available:boolean;updatedAt:number|null;
 media:{title:string;artist:string;playing:boolean}|null;
 weather:{label:string;temperature:number;unit:string}|null;
 notifications:number|null;
}

/** One entry for the pets and the existing desktop functions. DMS continues
 * to own its calendar, media controls, forecast and system panels. */
export function systemCenter(surface:HTMLElement,compact:HTMLElement,changed:()=>void){
 const clock=document.createElement('span');clock.className='system-center-clock';
 const time=document.createElement('time'),date=document.createElement('span');clock.append(time,date);compact.insertBefore(clock,compact.lastElementChild);
 const tools=document.createElement('section');tools.className='system-center-actions';tools.hidden=true;tools.setAttribute('aria-label','Centro do sistema');
 const feedback=document.createElement('p');feedback.className='system-center-feedback';feedback.hidden=true;feedback.setAttribute('role','status');
 const calendar=document.createElement('button');calendar.className='system-center-calendar';calendar.type='button';calendar.setAttribute('aria-label','Abrir calendário');
 const expandedClock=document.createElement('time'),expandedDate=document.createElement('span');calendar.append(expandedClock,expandedDate);
 const shortcuts=document.createElement('div');shortcuts.className='system-center-shortcuts';
 const cards=document.createElement('div');cards.className='system-center-cards';
 function action(panel:'calendar'|'media'|'weather'|'notifications'|'quick-settings',button:HTMLButtonElement){
  button.onclick=async()=>{
   button.disabled=true;button.setAttribute('aria-busy','true');feedback.hidden=true;changed();
   try{await Bridge.petSystemPanel(panel);await emitToWindow('pet-control','pet-control-collapse',{});}
   catch(error){feedback.textContent=String(error).replace(/^Error:\s*/,'');feedback.hidden=false;changed();}
   finally{button.disabled=false;button.removeAttribute('aria-busy');}
  };
 }
 action('calendar',calendar);
 const media=document.createElement('button');media.className='system-center-media';media.type='button';media.setAttribute('aria-label','Abrir música e controlos de reprodução');
 const mediaLabel=document.createElement('span'),mediaTitle=document.createElement('strong'),mediaArtist=document.createElement('small');mediaLabel.textContent='Música';mediaTitle.textContent='Ver reprodução';mediaArtist.textContent='Faixas e controlos de multimédia';media.append(mediaLabel,mediaTitle,mediaArtist);action('media',media);
 const weather=document.createElement('button');weather.className='system-center-weather';weather.type='button';weather.setAttribute('aria-label','Abrir tempo e previsão');
 const weatherLabel=document.createElement('span'),weatherTitle=document.createElement('strong'),weatherDetail=document.createElement('small');weatherLabel.textContent='Tempo';weatherTitle.textContent='Ver previsão';weatherDetail.textContent='Condições e próximos dias';weather.append(weatherLabel,weatherTitle,weatherDetail);action('weather',weather);cards.append(media,weather);
 const notifications=document.createElement('button');notifications.type='button';notifications.textContent='Notificações';action('notifications',notifications);
 const quick=document.createElement('button');quick.type='button';quick.textContent='Definições rápidas';action('quick-settings',quick);shortcuts.append(notifications,quick);
 tools.append(calendar,shortcuts,cards);surface.insertBefore(tools,document.getElementById('control-header'));surface.append(feedback);
 let enabled=false,reading=false,generation=0;
 const locale=document.documentElement.lang||undefined;
 function render(){
  if(!enabled)return;const now=new Date();
  const hours=new Intl.DateTimeFormat(locale,{hour:'2-digit',minute:'2-digit',hour12:false}).format(now);
  const day=new Intl.DateTimeFormat(locale,{weekday:'short',day:'numeric',month:'short'}).format(now);
  const full=new Intl.DateTimeFormat(locale,{weekday:'long',day:'numeric',month:'long'}).format(now);
  if(time.textContent!==hours)time.textContent=hours;if(date.textContent!==day)date.textContent=day;
  expandedClock.textContent=hours;expandedDate.textContent=full;expandedClock.dateTime=time.dateTime=now.toISOString();clock.title=calendar.title=new Intl.DateTimeFormat(locale,{dateStyle:'full',timeStyle:'short'}).format(now);
 }
 async function refresh(){
  if(!enabled||reading||document.hidden)return;reading=true;const ticket=generation;
  try{
   const status=await Bridge.petSystemStatus();if(!enabled||ticket!==generation)return;
   if(status.media){mediaTitle.textContent=status.media.title||'Sem título';mediaArtist.textContent=[status.media.artist,status.media.playing?'A reproduzir':'Em pausa'].filter(Boolean).join(' · ');media.title=[status.media.title,status.media.artist].filter(Boolean).join('\n');}
   else{mediaTitle.textContent='Sem reprodução';mediaArtist.textContent='Abrir os controlos de multimédia';media.title='';}
   if(status.weather){const unit=status.weather.unit.toLowerCase();weatherTitle.textContent=`${Math.round(status.weather.temperature)}${unit==='c'||unit==='celsius'?'°C':unit==='f'||unit==='fahrenheit'?'°F':status.weather.unit||'°'}`;weatherDetail.textContent=status.weather.label;weather.title=status.weather.label;}
   else{weatherTitle.textContent='Ver previsão';weatherDetail.textContent='Abrir o tempo no DMS';weather.title='';}
   notifications.textContent=status.notifications===null?'Notificações':`Notificações · ${status.notifications}`;
  }catch{/* Keep all native panel actions available if status is unavailable. */}
  finally{reading=false;}
 }
 clock.hidden=true;window.setInterval(()=>{render();void refresh();},3000);document.addEventListener('visibilitychange',()=>{render();void refresh();});
 return {get height(){return enabled?156+(feedback.hidden?0:56):0;},get width(){return enabled?432:288;},setEnabled(value:boolean){
  if(enabled===value)return;enabled=value;generation++;surface.dataset.systemCenter=String(value);clock.hidden=!value;tools.hidden=!value;feedback.hidden=true;
  surface.setAttribute('aria-label',value?'Centro dos pets e do sistema':'Centro de controlo dos pets');compact.setAttribute('aria-label',value?'Abrir centro dos pets e do sistema':'Abrir centro de controlo dos pets');render();void refresh();changed();
 }};
}
