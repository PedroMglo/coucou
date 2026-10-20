import {h} from './dom';
import {State} from '../core/state';
import {SystemAudio,type AudioNode,type AudioRequest,type AudioController,type AudioDeviceSnapshot,type AudioSnapshot} from '../core/system-audio';
import {buildSpotifyCard} from './spotify';

type Tab='music'|'sound'|'apps';
const percent=(value:number|null)=>value===null?'Volume desconhecido':`${Math.round(value)}%`;

/** The same music/audio surface is used by the physical pet and its island. */
export function buildSpotyCard(options:{devicesOnly?:boolean;controller?:AudioController<AudioDeviceSnapshot>;visible?:()=>boolean}={}){
 const devicesOnly=options.devicesOnly===true,controller=options.controller??SystemAudio;
 const music=devicesOnly?null:buildSpotifyCard();
 const el=h('section',{class:'spoty'+(devicesOnly?' system-device-audio':''),'aria-label':devicesOnly?'Som do sistema':'Spoty · Música e som'});
 const heading=h('header',{class:'spoty-heading'},h('strong',{text:devicesOnly?'Som do sistema':'Spoty'}),h('span',{text:devicesOnly?'Saídas, microfones e dispositivos':'Música e som do PC'}));
 const tabs=h('div',{class:'spoty-tabs',role:'tablist','aria-label':'Controlos do Spoty'});
 const buttons=new Map<Tab,HTMLButtonElement>();let selected:Tab=devicesOnly?'sound':'music';
 const musicPanel=music?h('div',{id:'spoty-music',role:'tabpanel','aria-labelledby':'spoty-tab-music'},music.el):null;
 const audioPanel=h('div',{class:'spoty-audio-panel'});
 const status=h('p',{class:'spoty-status',role:'status','aria-live':'polite'});
 const refresh=h('button',{type:'button',class:'spoty-refresh',text:'Atualizar',onclick:()=>void controller.refresh(true)});
 const boost=h('input',{type:'checkbox','aria-label':'Permitir amplificação até 150%'}) as HTMLInputElement;
 const boostLabel=h('label',{class:'spoty-boost'},boost,h('span',{text:'Amplificar até 150% · pode distorcer'}));
 const toolbar=h('div',{class:'spoty-toolbar'},status,refresh);
 const groups=h('div',{class:'spoty-groups'}),empty=h('p',{class:'spoty-empty'});
 audioPanel.append(toolbar,boostLabel);if(devicesOnly)audioPanel.append(h('p',{class:'system-audio-effect-note',text:'Alterar o microfone predefinido ou o perfil pode mudar as captações das aplicações e interromper som Bluetooth.'}));audioPanel.append(groups,empty);el.append(heading);if(!devicesOnly)el.append(tabs);if(musicPanel)el.append(musicPanel);el.append(audioPanel);
 let poll:number|null=null;
 const rows=new Map<string,ReturnType<typeof row>>();
 const sections=new Map<string,{el:HTMLElement;list:HTMLElement}>();
 for(const[id,title]of (devicesOnly?[]:[['music','Música'],['sound','Som'],['apps','Aplicações']]) as readonly (readonly [Tab,string])[]){
  const button=h('button',{type:'button',id:`spoty-tab-${id}`,role:'tab',text:title,'aria-controls':id==='music'?'spoty-music':'spoty-audio'}) as HTMLButtonElement;
  button.onclick=()=>select(id);buttons.set(id,button);tabs.append(button);
  button.onkeydown=e=>{
   const order=[...buttons.keys()],index=order.indexOf(selected);let next:Tab|undefined;
   if(e.key==='ArrowRight')next=order[(index+1)%order.length];else if(e.key==='ArrowLeft')next=order[(index+order.length-1)%order.length];else if(e.key==='Home')next=order[0];else if(e.key==='End')next=order.at(-1);
   if(next){e.preventDefault();e.stopPropagation();select(next);buttons.get(next)!.focus();}
  };
 }
 audioPanel.id=devicesOnly?'system-device-audio':'spoty-audio';audioPanel.setAttribute('role',devicesOnly?'region':'tabpanel');if(devicesOnly)audioPanel.setAttribute('aria-label','Dispositivos de som');
 function visible(){if(options.visible)return el.isConnected&&!document.hidden&&options.visible()&&el.getClientRects().length>0;return el.isConnected&&!document.hidden&&State.focusId==='integration_spotify'&&State.mode==='expanded'&&State.view==='overview'&&el.getClientRects().length>0;}
 function select(tab:Tab){
  selected=tab;for(const[id,button]of buttons){button.setAttribute('aria-selected',String(id===tab));button.tabIndex=id===tab?0:-1;}
  if(musicPanel)musicPanel.hidden=tab!=='music';audioPanel.hidden=tab==='music';
  // Keep music drafts in the same nodes, with no playback-position timer while
  // its tab is hidden. Detaching never issues a player command.
  if(music&&musicPanel){if(tab==='music')musicPanel.append(music.el);else music.el.remove();}
  if(!devicesOnly)audioPanel.setAttribute('aria-labelledby',`spoty-tab-${tab}`);music?.sync();render();schedule();
  if(tab!=='music'&&visible())void controller.refresh(true);
 }
 function schedule(){
  if(poll!==null){window.clearTimeout(poll);poll=null;}
  if(!visible()||selected==='music')return;
  poll=window.setTimeout(()=>{poll=null;if(visible()&&selected!=='music'){void controller.refresh();schedule();}},2000);
 }
 function section(id:string,title:string,index:number){
  let group=sections.get(id);
  if(!group){const list=h('div'),el=h('section',{class:'spoty-group'},h('h3',{text:title}),list);group={el,list};sections.set(id,group);}
  if(groups.children[index]!==group.el)groups.insertBefore(group.el,groups.children[index]??null);return group;
 }
 function send(node:AudioNode,action:AudioRequest['action'],values:Partial<AudioRequest>={}){
  if(devicesOnly&&!visible()||!node.controllable||controller.pending||controller.stale)return;
  void controller.control({targetKey:node.key,action,...values});
 }
 function row(initial:AudioNode){
  let node=initial;
  const title=h('strong'),detail=h('span',{class:'spoty-node-detail'});
  const mute=h('button',{type:'button',class:'spoty-mute'}) as HTMLButtonElement;
  const volume=h('input',{type:'range',min:'0',max:'100',step:'1'}) as HTMLInputElement;
  const level=h('output',{class:'spoty-level'});
  const defaultButton=h('button',{type:'button',class:'spoty-default'}) as HTMLButtonElement;
  const options=h('select') as HTMLSelectElement,route=devicesOnly?null:h('select') as HTMLSelectElement;
  const balance=h('input',{type:'range',min:'-1',max:'1',step:'0.05'}) as HTMLInputElement;
  const balanceLabel=h('label',{class:'spoty-balance'},h('span',{text:'Esquerda'}),balance,h('span',{text:'Direita'}));
  const optionLabel=h('label',{class:'spoty-select'},h('span',{text:node.kind==='card'?'Perfil':'Porta'}),options);
  const routeLabel=route?h('label',{class:'spoty-select'},h('span',{text:node.kind==='playback'?'Saída desta aplicação':'Entrada desta aplicação'}),route):null;
  const volumeRow=h('div',{class:'spoty-volume'},mute,volume,level);
  const body=h('article',{class:'spoty-node','data-audio-key':node.key},h('div',{class:'spoty-node-heading'},h('div',{},title,detail),defaultButton),volumeRow,...(routeLabel?[routeLabel]:[]),optionLabel,balanceLabel);
  mute.onclick=()=>send(node,'mute',{muted:!node.muted});
  defaultButton.onclick=()=>send(node,'default');
  volume.oninput=()=>{level.textContent=`${volume.value}%`;level.title='Volume escolhido · aguardando confirmação';};
  let adjustingVolume=false,adjustingBalance=false,cancelledVolume=false,cancelledBalance=false;
  volume.onpointerdown=()=>{adjustingVolume=true;cancelledVolume=false;};
  volume.onpointercancel=()=>{adjustingVolume=false;cancelledVolume=true;render();};
  volume.onpointerup=()=>{adjustingVolume=false;};
  volume.onchange=()=>{adjustingVolume=false;if(cancelledVolume){cancelledVolume=false;render();return;}send(node,'volume',{value:Number(volume.value)});};
  balance.onpointerdown=()=>{adjustingBalance=true;cancelledBalance=false;};
  balance.onpointercancel=()=>{adjustingBalance=false;cancelledBalance=true;render();};
  balance.onpointerup=()=>{adjustingBalance=false;};
  balance.onchange=()=>{adjustingBalance=false;if(cancelledBalance){cancelledBalance=false;render();return;}send(node,'balance',{value:Number(balance.value)});};
  options.onchange=()=>send(node,node.kind==='card'?'profile':'port',{option:options.value});
  if(route)route.onchange=()=>send(node,'route',{destinationKey:route.value});
  const fill=(select:HTMLSelectElement,choices:{id:string;label:string;available:boolean}[],value:string|null)=>{
   const signature=JSON.stringify(choices);
   if(select.dataset.choices!==signature){select.replaceChildren(...choices.map(choice=>{const option=h('option',{value:choice.id,text:choice.label||choice.id}) as HTMLOptionElement;option.disabled=!choice.available;return option;}));select.dataset.choices=signature;}
   select.value=value??'';if(!value||!choices.some(c=>c.id===value))select.selectedIndex=-1;
  };
  return {el:body,sync(value:AudioNode){
   node=value;const disabled=controller.pending||controller.stale||!node.controllable;
   title.textContent=node.label;title.title=node.label;
   detail.textContent=[node.description!==node.label?node.description:'',node.isDefault?'Predefinido':'',node.monitor?'Monitor do som do PC':node.kind==='recording'?'A aplicação está a captar áudio':''].filter(Boolean).join(' · ');detail.title=detail.textContent;
   defaultButton.hidden=!['output','input'].includes(node.kind)||node.monitor;
   defaultButton.textContent=node.isDefault?'Predefinido':'Usar por defeito';defaultButton.disabled=disabled||node.isDefault;
   defaultButton.setAttribute('aria-label',`${node.kind==='input'?'Usar microfone':'Usar saída'} ${node.label} por defeito`);
   mute.textContent=node.muted===null?'Som por confirmar':node.muted?'Repor som':'Silenciar';mute.setAttribute('aria-label',node.muted===null?`Estado do som de ${node.label} por confirmar`:`${node.muted?'Repor som de':'Silenciar'} ${node.label}`);mute.setAttribute('aria-pressed',node.muted===null?'mixed':String(node.muted));mute.disabled=disabled||node.muted===null;
   volumeRow.hidden=node.kind==='card';volume.disabled=disabled||!node.canVolume||devicesOnly&&node.volume===null;volume.max=boost.checked?'150':'100';
   if(!adjustingVolume)volume.value=String(Math.min(Number(volume.max),node.volume??0));
   volume.setAttribute('aria-label',`Volume · ${node.label}`);volume.setAttribute('aria-valuetext',percent(node.volume));
   if(!adjustingVolume)level.textContent=percent(node.volume);level.title=node.volume!==null&&node.volume>100?'O volume observado está amplificado.':'Volume observado';
   optionLabel.hidden=node.options.length<2;options.disabled=disabled;options.setAttribute('aria-label',`${node.kind==='card'?'Perfil':'Porta'} · ${node.label}`);fill(options,node.options,node.activeOption);
   balanceLabel.hidden=!node.canBalance;balance.disabled=disabled||devicesOnly&&node.balance===null;balance.setAttribute('aria-label',`Balanço · ${node.label}`);if(!adjustingBalance)balance.value=String(node.balance??0);
   balance.setAttribute('aria-valuetext',node.balance===null?'Desconhecido':Math.abs(node.balance)<.01?'Centrado':`${Math.round(Math.abs(node.balance)*100)}% à ${node.balance<0?'esquerda':'direita'}`);
   if(route&&routeLabel){const devices=(node.kind==='playback'?controller.snapshot?.outputs:controller.snapshot?.inputs)??[];
   routeLabel.hidden=!['playback','recording'].includes(node.kind);route.disabled=disabled;
   route.setAttribute('aria-label',`${node.kind==='playback'?'Saída':'Entrada'} · ${node.label}`);
   fill(route,devices.filter(d=>!d.monitor||d.key===node.deviceKey).map(d=>({id:d.key,label:d.label,available:d.controllable&&!d.monitor})),node.deviceKey);}
  }};
 }
 function render(){
  if(selected==='music')return;
  const snapshot=controller.snapshot;
  status.textContent=(controller.error?`${controller.error}${snapshot?' Valores da última leitura.':''}`:'')||controller.warning||(controller.pending?'A aplicar e confirmar…':controller.loading?'A atualizar…':snapshot?.observedAt?`Atualizado às ${new Intl.DateTimeFormat('pt-PT',{hour:'2-digit',minute:'2-digit',second:'2-digit'}).format(snapshot.observedAt)}`:snapshot?'Data da leitura indisponível.':'A obter os dispositivos de som…');
  if(devicesOnly&&controller.stale&&snapshot&&!controller.error)status.textContent+=' · Valores da última leitura, por confirmar.';
  el.dataset.stale=String(controller.stale);status.dataset.error=String(!!controller.error);audioPanel.setAttribute('aria-busy',String(controller.pending||controller.loading));refresh.disabled=controller.loading||controller.pending;
  boostLabel.hidden=selected==='apps';boost.disabled=controller.pending;
  for(const group of sections.values())group.el.hidden=true;
  const data=selected==='sound'?[['outputs','Saídas',snapshot?.outputs],['inputs','Microfones',snapshot?.inputs.filter(n=>!n.monitor)],['cards','Perfis dos dispositivos',snapshot?.cards]] as const:[['playback','Aplicações com som',(snapshot as AudioSnapshot|null)?.playback],['recording','Aplicações a captar áudio',(snapshot as AudioSnapshot|null)?.recording]] as const;
  const wanted=new Set<string>();let count=0,sectionIndex=0;
  for(const[id,title,nodes]of data){
   if(!nodes?.length)continue;const group=section(id,title,sectionIndex++);group.el.hidden=false;
   for(const node of nodes){count++;wanted.add(node.key);let item=rows.get(node.key);if(!item){item=row(node);rows.set(node.key,item);}if(item.el.parentElement!==group.list)group.list.append(item.el);item.sync(node);}
  }
  for(const[key,item]of rows)if(!wanted.has(key)){item.el.remove();rows.delete(key);}
  empty.hidden=count>0;empty.textContent=controller.stale?'Os dispositivos de som não estão disponíveis. Tenta Atualizar.':selected==='apps'?'As aplicações aparecem aqui enquanto reproduzem ou captam som.':'Não há dispositivos de som disponíveis.';
 }
 boost.onchange=render;
 const unsubscribe=controller.subscribe(()=>{if(visible()&&selected!=='music')render();});
 const visibilityChanged=()=>{if(visible()&&selected!=='music')void controller.refresh();schedule();};
 document.addEventListener('visibilitychange',visibilityChanged);
 function sync(){music?.sync();if(visible()&&selected!=='music'){render();void controller.refresh();}schedule();}
 function dispose(){if(poll!==null)window.clearTimeout(poll);poll=null;unsubscribe();document.removeEventListener('visibilitychange',visibilityChanged);music?.el.remove();}
 select(devicesOnly?'sound':'music');return {el,sync,dispose};
}
