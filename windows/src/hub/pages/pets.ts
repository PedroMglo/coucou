import {buildAccountUsage} from '../../views/account-usage';
import {Bridge,emitToWindow,onEvent} from '../../core/bridge';
import {PET_EVENTS} from '../../mochi/companions';
import {parseManagedUsage,type ManagedUsageView} from '../../core/managed-usage';
import {PetSettings} from '../../desktop/pet-settings';
import {petCapabilities} from '../../mochi/capabilities';
import {PetPreferences} from '../../mochi/pet-preferences';
import {taskLabel,taskName,taskPet,taskRecoverable,taskCanStop,type AssistantTask} from '../../mochi/tasks';
import {taskExecution} from '../../views/task-execution';
import {activeSession,bytesLabel,inventoryProvider,sessionTree,uniqueManagedModels,matchesSearch,type ManagedInventory,type ManagedDetail,type ManagedResult} from '../../mochi/management';
import type {HubContext} from '../shell';
import {OllamaModelSelection} from '../../core/ollama-selection';
import {PetObservability,observePetObservability} from '../../core/pet-observability-resource';
import {managedSessionLinks} from '../../mochi/management-observation';
import {SessionLookupNavigation,lookupKey,lookupEvidence,validLookupRequest} from '../../core/session-lookup';
import type {SessionLookupRequest} from '../../core/session-lookup-contract';
import './pets.css';
const node=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const el=document.createElement(tag);if(text!=null)el.textContent=text;return el;};
const button=(text:string,action:()=>void)=>{const el=node('button',text);el.type='button';el.onclick=action;return el;};
const select=(label:string,options:[string,string][],value:string,change:(v:string)=>void)=>{const el=node('select');el.setAttribute('aria-label',label);for(const [id,text]of options){const o=node('option',text);o.value=id;el.append(o);}el.value=value;el.onchange=()=>change(el.value);return el;};
export interface PetsPageOptions {
 embedded?:boolean;
 bindNavigation?:(navigate:(section:string,pet?:string)=>void)=>void;
 onSectionChange?:(section:string)=>void;
 requestSection?:(section:string,commit:()=>void)=>void;
 collapse?:()=>void;
}
export async function petsPage(page:HTMLElement,ctx:HubContext,options:PetsPageOptions={}){
 const root=node('section');root.className='pets-manager';root.setAttribute('aria-label','Gestão dos pets');
 const favorites=node('div');favorites.className='pet-panel-favorites';
 const title=node('header'),nav=node('nav'),filters=node('div'),content=node('div'),message=node('p'),confirmation=node('div'),summary=node('p');
 const selectedProvider=(pet:string)=>inventoryProvider(pet,PetPreferences.config?.pets[pet]?.provider);
 let autoProvider=!!ctx.petSelection,providerFilter:HTMLSelectElement|null=null;
 let alive=true,visible=ctx.initialVisible??true,loading=false,busy=false,tab=ctx.petSection??'tasks',provider=ctx.petSelection?selectedProvider(ctx.petSelection):'',query='',state='all',modelState='all',keep='10m',detail:ManagedDetail|null=null;
 let inventory:ManagedInventory|null=null,usage:ManagedUsageView={providers:{},partial:false},tasks:AssistantTask[]=[],lastUsage=0,lastKey='',modelQuery='';
 let navigation=0,usageGeneration=0,usageLoading=false,usageFromCache=true;
 const collapsed=new Set<string>(),expanded=new Set<string>(),resuming=new Set<string>();
 const folderDrafts=new Map<string,string>(),editingFolders=new Set<string>();
 const account=buildAccountUsage({compact:false,visible:()=>alive&&visible&&tab==='providers'&&!detail&&!document.hidden});
 const syncAccount=()=>account.setContext(provider===''||provider==='codex'?'codex':'',alive&&visible&&tab==='providers'&&!detail,provider);
 const prefs=new PetSettings();prefs.el.hidden=false;prefs.setActive(visible&&tab==='profiles');
 const request=<T>(action:string,p='',id='')=>Bridge.petManagement<T>(action,p,id,keep);
 const report=(text:string,error=false)=>{message.textContent=text;message.dataset.error=String(error);};
 const lookupRows=new Map<string,{request:SessionLookupRequest;open:HTMLButtonElement;note:HTMLElement;subtitle:HTMLElement;base:string;origin:string}>();
 let retryTimer:number|null=null;
 const lookup=new SessionLookupNavigation({lookup:request=>Bridge.petSessionLookup(request),open:target=>Bridge.petChatOpen(target.petId,undefined,false,target)},{
  current:request=>alive&&visible&&tab==='sessions'&&!detail&&!document.hidden&&lookupRows.has(lookupKey(request))&&!!inventory?.sessions.some(row=>row.provider===request.provider&&row.id===request.nativeSessionId),
  unavailable:target=>{
   if(target.petId==='integration_spotify')return 'O Spoty controla música e som. Esta conversa antiga não tem chat disponível.';
   if(!PetPreferences.active(target.petId))return 'O pet responsável por esta sessão está desativado.';
   const capability=petCapabilities(target.petId).chat;
   return !capability?'Este pet não tem chat disponível.':capability!==target.provider?'A conversa já não corresponde ao provider deste pet. Atualiza e tenta novamente.':'';
  },changed:()=>syncLookupControls(),presented:()=>{report('');void Bridge.petPanelHide().catch(()=>{});},
 });
 function advanceNavigation(){navigation++;lookup.invalidate();report('');}
 function syncLookupControls(){
  if(retryTimer!==null){window.clearTimeout(retryTimer);retryTimer=null;}
  if(!alive||!visible||tab!=='sessions'||detail)return;
  for(const row of lookupRows.values()){
   const state=lookup.state(row.request),pending=lookup.isPending(row.request),wait=lookup.retryWait(row.request),valid=validLookupRequest(row.request);
   row.open.textContent=state?.phase==='locating'?'A localizar…':state?.phase==='opening'?'A abrir…':wait?`Tentar novamente · ${wait} s`:state&&!state.opened?'Tentar novamente':'Abrir conversa';
   row.open.disabled=pending||wait>0||!valid;row.open.setAttribute('aria-busy',String(pending));
   const origin=state?.result?.origin==='mochi'?'No pet':state?.result?.origin==='external'?'Externa':row.origin;
   row.subtitle.textContent=`${origin} · ${row.base}`;
   const text=!valid?'A identificação desta sessão não permite abrir a conversa.':state?`${state.message}${state.result?' '+lookupEvidence(state.result):''}${wait?` Voltar a tentar em cerca de ${wait} s.`:''}`:'';
   if(row.note.textContent!==text)row.note.textContent=text;
   row.note.hidden=!text;row.open.title=text;
  }
  if(lookup.hasRetryWait)retryTimer=window.setTimeout(()=>{retryTimer=null;syncLookupControls();},1000);
 }
 const openPet=(pet:string)=>{void Bridge.petPanelHide().catch(()=>{});options.collapse?.();void emitToWindow('island',PET_EVENTS.summon,{pillId:pet});};
 let modelChoice=0;
 const converseWithModel=async(model:string)=>{
  const choice=++modelChoice;report(`A selecionar ${model}…`);
  try{const selected=await OllamaModelSelection.select(model);if(alive&&choice===modelChoice){if(selected){report(`Modelo selecionado: ${model}`);openPet('ai_ollama');}else report('O modelo mudou noutra janela. A escolha mais recente foi mantida.');}}
  catch(error){if(alive&&choice===modelChoice)report(String(error).replace(/^Error:\s*/,''),true);}
 };
 const setTab=(value:string,accepted=false)=>{if(!accepted&&options.requestSection){options.requestSection(value,()=>setTab(value,true));return;}advanceNavigation();tab=value;prefs.setActive(visible&&value==='profiles');options.onSectionChange?.(value);detail=null;confirmation.replaceChildren();lastKey='';draw();if(value==='profiles'&&!prefs.loaded)void prefs.load(ctx.petSelection);void refresh();};
 options.bindNavigation?.((section,pet)=>{ctx.petSelection=pet;autoProvider=!!pet;provider=pet?selectedProvider(pet):'';if(pet)prefs.selectPet(pet);setTab(section,true);});
 const refreshButton=button('Atualizar',()=>void refresh(true));title.append(node('h1','Os teus pets'),refreshButton);
 nav.setAttribute('aria-label','Áreas de gestão dos pets');
 for(const [id,label]of [['tasks','Tarefas'],['sessions','Sessões'],['providers','Providers e quotas'],['models','Modelos locais'],['profiles','Nomes e perfis']]){const b=button(label,()=>setTab(id));b.dataset.tab=id;nav.append(b);}
 content.className='pets-management-content';filters.className='pets-management-filters';confirmation.className='pets-management-confirmation';confirmation.setAttribute('role','alert');message.setAttribute('role','status');summary.className='pets-management-summary';
 root.append(title,favorites,nav,summary,filters,confirmation,message,content);page.append(root);
 if(options.embedded){root.dataset.embedded='true';nav.hidden=true;}
 let confirmationFocus:HTMLElement|null=null;
 const cancelConfirmation=()=>{confirmation.replaceChildren();if(document.hasFocus())confirmationFocus?.focus();};
 confirmation.addEventListener('keydown',event=>{if(event.key==='Escape'){event.preventDefault();event.stopPropagation();cancelConfirmation();}});
 async function run(action:string,p:string,id='',confirmed=false){
  if(busy)return;advanceNavigation();
  if(!confirmed&&['interrupt','archive','unarchive','unload','service-stop'].includes(action)){
   const text=action==='unload'?'Libertar a memória deste modelo? As conversas de terminal associadas serão fechadas; o ficheiro fica no disco.':action==='service-stop'?'Parar o Ollama? Pedidos ativos serão interrompidos e os modelos descarregados.':`Confirmar ${action==='interrupt'?'a interrupção':action==='archive'?'o arquivo':'o restauro'} desta sessão?`;
   confirmationFocus=document.activeElement as HTMLElement|null;
   confirmation.replaceChildren(node('p',`${text} ${id}`),button('Cancelar',cancelConfirmation),button('Confirmar',()=>{confirmation.replaceChildren();void run(action,p,id,true);}));confirmation.querySelector('button')?.focus();return;
  }
  confirmation.replaceChildren();busy=true;root.dataset.busy='true';report('A executar…');
  const origin=navigation;
  try{const result=await request<ManagedResult>(action,p,id);if(!alive)return;report(result.message??'');if(result.detail&&origin===navigation){detail=result.detail;lastKey='';draw();}}
  catch(error){if(alive)report(String(error).replace(/^Error:\s*/,''),true);}
  finally{busy=false;root.dataset.busy='false';if(alive)await refresh();}
 }
 function quota(p:string){
  const q=usage.providers[p];if(!q)return ['codex','claude','copilot'].includes(p)?node('p',usageLoading?'A ler utilização guardada…':'Utilização guardada indisponível. Podes consultar a conta em Atualizar consumo.'):null;
  const el=node('div');el.className='pets-quota';
  el.append(node('p',[usageFromCache?'Quotas guardadas':'Quotas consultadas',q.account,q.plan,q.unlimited.length?'Sem limite: '+q.unlimited.join(', '):'',q.note,q.updated!==null?'Atualizado '+new Date(q.updated).toLocaleString():'Data da fonte indisponível',q.stale?'Dados desatualizados':'',q.blocked?'Utilização incluída bloqueada':''].filter(Boolean).join(' · ')));
  if(q.windows===null)el.append(node('p','As janelas de consumo não estão disponíveis.'));
  for(const w of q.windows??[]){
   const line=node('div'),reset=w.resetLabel??(w.resetAt!==null?new Date(w.resetAt).toLocaleString():null);
   line.append(node('span',`${w.label} · ${w.used===null?'Utilização desconhecida':w.used.toFixed(1)+'%'}${reset?' · renova '+reset:''}`));
   if(w.remaining!==null)line.append(node('span',` · saldo ${w.remaining}${w.entitlement!==null?' / '+w.entitlement:''}${w.additional?' · utilização adicional autorizada':''}`));
   if(w.used!==null){const bar=node('progress');bar.max=100;bar.value=w.used;bar.setAttribute('aria-label',w.label);line.append(bar);}el.append(line);
  }
  if(q.partial)el.append(node('p','Alguns dados de consumo não estão disponíveis.'));return el;
 }
 function card(title:string,subtitle=''){const el=node('article');el.append(node('h2',title));if(subtitle)el.append(node('p',subtitle));return el;}
 function actions(el:HTMLElement,items:[string,()=>void,boolean?,boolean?][]){const row=node('footer');for(const [label,action,show,disabled]of items)if(show!==false){const b=button(label,action);b.disabled=!!disabled;if(disabled)b.setAttribute('aria-busy','true');row.append(b);}el.append(row);}
 const unsupportedProvider=()=>!!inventory&&!!provider&&!inventory.providers.some(p=>p.id===provider);
 const unavailableText=()=>`O inventário de ${taskName(provider)} não está disponível nesta vista. O histórico no Coucou continua no cartão do pet. Podes escolher outra fonte no filtro.`;
 function syncProviderOptions(){
  if(!providerFilter)return;
  const values:[string,string][]=[['','Todos os providers'],...(inventory?.providers??[]).map(p=>[p.id,p.name] as [string,string])];
  if(provider&&!values.some(([id])=>id===provider))values.push([provider,`${taskName(provider)} · ${inventory?'inventário indisponível':'a confirmar'}`]);
  if(JSON.stringify([...providerFilter.options].map(o=>[o.value,o.textContent]))!==JSON.stringify(values)){
   providerFilter.replaceChildren();for(const [id,text]of values){const option=node('option',text);option.value=id;providerFilter.append(option);}
  }
  providerFilter.value=provider;
 }
 function draw(controls=true){
  syncAccount();
  if(options.embedded)title.querySelector('h1')!.textContent=({tasks:'Tarefas',sessions:'Sessões',providers:'Providers e utilização',models:'Modelos locais',profiles:'Nomes, perfis e acessos'} as Record<string,string>)[tab]??'Os teus pets';
  for(const b of nav.querySelectorAll('button')){b.setAttribute('aria-pressed',String(b.dataset.tab===tab));b.setAttribute('aria-selected',String(b.dataset.tab===tab));}
  if(controls){filters.replaceChildren();providerFilter=null;}
  summary.textContent=inventory?`Inventário da estação · ${inventory.summary.sessions??'—'} sessões sinalizadas (inclui sessões sem turno) · ${inventory.summary.agents??'—'} agentes detetados · ${inventory.summary.loaded??'—'} modelos em memória${inventory.notes?.length?' · '+inventory.notes.join(' · '):''}`:'O inventário consulta metadados; não lê as conversas.';
  if(tab==='profiles'){summary.textContent='Personaliza cada pet, os modelos que usa e os acessos que lhe dás.';if(prefs.el.parentElement!==content)content.replaceChildren(prefs.el);return;}
  if((tab==='sessions'||tab==='providers')&&unsupportedProvider())summary.textContent=`Fonte selecionada · ${taskName(provider)} · Inventário indisponível`;
  if(controls&&tab!=='models'){
   providerFilter=select('Filtrar provider',[],provider,value=>{advanceNavigation();autoProvider=false;provider=value;detail=null;confirmation.replaceChildren();lastKey='';draw();});filters.append(providerFilter);
  }
  syncProviderOptions();
  if(controls&&tab==='sessions'){
   filters.append(select('Filtrar sessões',[['all','Todas'],['active','Sinalizadas (inclui sem turno)'],['saved','Guardadas'],['archived','Arquivadas']],state,value=>{advanceNavigation();state=value;detail=null;confirmation.replaceChildren();lastKey='';draw();}));
   const search=node('input');search.placeholder='Pesquisar sessão, modelo ou projeto';search.setAttribute('aria-label','Pesquisar sessões');search.value=query;search.oninput=()=>{advanceNavigation();query=search.value;detail=null;confirmation.replaceChildren();drawContent();};filters.append(search);
  }
  if(controls&&tab==='models'){
   filters.append(select('Estado dos modelos',[['all','Todos'],['sessions','Com sessões'],['memory','Em memória'],['disk','Só no disco']],modelState,value=>{advanceNavigation();modelState=value;detail=null;confirmation.replaceChildren();lastKey='';draw();}),select('Retenção na memória',[['5m','5 minutos'],['10m','10 minutos'],['30m','30 minutos'],['1h','1 hora']],keep,value=>keep=value));
   const search=node('input');search.placeholder='Pesquisar nome, alias ou tamanho';search.setAttribute('aria-label','Pesquisar modelos locais');search.value=modelQuery;search.oninput=()=>{advanceNavigation();modelQuery=search.value;detail=null;confirmation.replaceChildren();drawContent();};filters.append(search);
  }
  drawContent();
 }
 async function resumeTask(t:AssistantTask,cwd?:string){
  if(resuming.has(t.id))return;resuming.add(t.id);lastKey='';drawContent();report('A verificar e retomar a tarefa…');
  try{await Bridge.assistantResumeTask(t.id,cwd);}catch(error){report(String(error),true);}finally{resuming.delete(t.id);lastKey='';await refresh();}
 }
 async function stopTask(t:AssistantTask){
  if(busy)return;busy=true;root.dataset.busy='true';report('A interromper a tarefa…');
  try{const stopped=await Bridge.assistantCancelTask(t.id);if(alive)report(stopped?'Interrupção pedida.':'Esta tarefa já não está a executar.');}
  catch(error){if(alive)report(String(error).replace(/^Error:\s*/,''),true);}
  finally{busy=false;root.dataset.busy='false';if(alive)await refresh();}
 }
 async function openManagedSession(p:string,id:string){await lookup.open({provider:p,nativeSessionId:id});}
 function drawContent(){
  const focused=account.el.contains(document.activeElement)?document.activeElement as HTMLElement:null;
  try{
  if(content.contains(document.activeElement)&&document.activeElement?.tagName==='INPUT')return;
  const key=JSON.stringify([tab,provider,query,state,modelState,modelQuery,detail,tasks,inventory?{...inventory,updated:0}:null,usage,[...collapsed],[...expanded]]);if(key===lastKey)return;lastKey=key;
  const scroll=content.scrollTop;lookupRows.clear();content.replaceChildren();
  if(detail){
   const d=detail,el=card(d.title);el.append(button('Voltar à lista',()=>{advanceNavigation();detail=null;lastKey='';draw();}));
   const fields=node('dl');for(const f of d.fields){fields.append(node('dt',f.label),node('dd',f.bytes!==undefined?bytesLabel(f.bytes):f.value??'—'));}el.append(fields);
   const q=quota(d.provider);if(q)el.append(q);
   actions(el,[['Retomar no terminal',()=>void run('open',d.provider,d.id),d.kind==='session'&&!d.archived],['Conversar neste pet',()=>void converseWithModel(d.id),d.kind==='model'&&d.chat],['Carregar na memória',()=>void run('load','ollama',d.id),d.kind==='model'&&!d.loaded],['Libertar memória',()=>void run('unload','ollama',d.id),d.kind==='model'&&!!d.loaded],['Editar configuração',()=>void run('edit-config',d.provider),d.kind!=='model'&&d.editable],['Configuração do projeto',()=>void run('project-config',d.provider,d.id),!!d.project_config],['Abrir pasta',()=>void run('folder',d.provider,d.id),!!d.cwd],['Consumo da conta',()=>void run('usage-page',d.provider),['codex','claude','copilot'].includes(d.provider)],['Documentação',()=>void run('docs',d.provider)]]);
   for(const child of d.clients??d.children??[]) {const row=card(child.name,child.state);row.append(button('Abrir no terminal',()=>void run('open',child.provider??d.provider,child.id)));el.append(row);}content.append(el);
  }else if(tab==='tasks'){
   const selected=tasks.filter(t=>!provider||t.backend===provider||t.provider===provider||provider==='gemini'&&t.provider==='gemini-cli'||provider==='claude'&&t.provider==='claude-cli');
   for(const t of selected){const el=card(t.query,`${taskName(t.provider)} · ${taskLabel(t.phase)} · ${t.model}${t.localReasoningEffort?' · '+t.localReasoningEffort:''} · ${t.cwd}`);if(t.progress)el.append(node('p',t.progress));const execution=taskExecution();execution.sync(t);el.append(execution.el);if(t.result){const details=node('details');details.append(node('summary','Ler resultado'),node('pre',t.result));el.append(details);}
    actions(el,[['Rever conversa / decisão',()=>{const pet=taskPet(t.provider);if(pet)openPet(pet);}],['Interromper',()=>void stopTask(t),taskCanStop(t)],[resuming.has(t.id)?'A retomar…':'Verificar e retomar',()=>void resumeTask(t),taskRecoverable(t)&&!resuming.has(t.id)],['Alterar pasta',()=>{editingFolders.has(t.id)?editingFolders.delete(t.id):editingFolders.add(t.id);lastKey='';drawContent();},taskRecoverable(t)&&!resuming.has(t.id)]]);
    if(editingFolders.has(t.id)&&taskRecoverable(t)){
     const folder=node('input');folder.value=folderDrafts.get(t.id)??t.cwd;folder.setAttribute('aria-label','Pasta para retomar a tarefa');folder.oninput=()=>folderDrafts.set(t.id,folder.value);folder.onblur=()=>{window.setTimeout(()=>{if(alive){lastKey='';drawContent();}},300);};
     el.append(folder,button('Verificar e retomar nesta pasta',()=>void resumeTask(t,folder.value.trim())));
    }
    content.append(el);}
   if(!selected.length)content.append(node('p','Os pedidos feitos aos pets aparecem aqui. As sessões externas estão em Sessões.'));
  }else if(tab==='sessions'){
   const providers=(inventory?.providers??[]).filter(p=>!provider||p.id===provider),linkFor=managedSessionLinks(PetObservability.snapshot,!!PetObservability.error);
   for(const entry of sessionTree(inventory?.sessions??[],providers,state,query,collapsed,expanded)){
    if(entry.kind==='provider'){const id=entry.provider.id;content.append(button(`${entry.opened?'▾':'▸'} ${entry.provider.name} · ${entry.count}`,()=>{advanceNavigation();collapsed.has(id)?collapsed.delete(id):collapsed.add(id);drawContent();}));continue;}
    const {row:r}=entry,job=tasks.find(t=>t.backend===r.provider&&t.sessionId===r.id),link=linkFor(r);
    const el=card(r.name,`${link.origin} · #${r.id.slice(0,8)} · ${r.state} · ${r.model||'Modelo não publicado'} · ${r.cwd}`);el.style.marginLeft=`${entry.depth*18}px`;el.dataset.context=String(entry.context);el.dataset.session=r.id;el.dataset.provider=r.provider;el.title=`${r.id}\n${r.cwd}`;
    if(entry.children)el.prepend(button(`${entry.opened?'▾':'▸'} ${entry.children} subagentes`,()=>{advanceNavigation();const id=`${r.provider}:${r.id}`;expanded.has(id)?expanded.delete(id):expanded.add(id);drawContent();}));
    if(r.agents_active!=null)el.append(node('p',`${r.agents_exact?'':'≥ '}${r.agents_active} agentes detetados · ${r.agents_total??'—'} registados${r.tokens!=null?' · '+r.tokens+' tokens':''}`));
    const open=button('Abrir conversa',()=>void openManagedSession(r.provider,r.id)),note=node('p');note.setAttribute('role','status');note.className='pets-session-lookup';note.hidden=true;
    const request={provider:r.provider,nativeSessionId:r.id};lookupRows.set(lookupKey(request),{request,open,note,subtitle:el.querySelector('p')!,base:`#${r.id.slice(0,8)} · ${r.state} · ${r.model||'Modelo não publicado'} · ${r.cwd}`,origin:link.origin});el.append(note);
    actions(el,[['Abrir terminal',()=>void run('focus',r.provider,r.id),activeSession(r)&&!job],['Retomar no terminal',()=>void run('open',r.provider,r.id),!activeSession(r)&&!r.archived],['Detalhes',()=>void run('settings',r.provider,r.id)],['Interromper',()=>void run('interrupt',r.provider,r.id),activeSession(r)],['Arquivar',()=>void run('archive',r.provider,r.id),r.provider==='codex'&&!activeSession(r)&&!r.archived],['Restaurar',()=>void run('unarchive',r.provider,r.id),r.provider==='codex'&&r.archived]]);el.querySelector('footer')!.prepend(open);content.append(el);
   }
   if(!content.childElementCount)content.append(node('p',unsupportedProvider()?unavailableText():'Nenhuma sessão neste filtro.'));
  }else if(tab==='providers'){
   if((provider===''||provider==='codex')&&!detail)content.append(account.el);
   for(const p of inventory?.providers??[]){if(provider&&p.id!==provider)continue;const el=card(p.name,`${p.detail} · ${p.active??'—'} sessões sinalizadas · ${p.agents_exact?'':'≥ '}${p.agents??'—'} agentes detetados · ${p.history??'—'} guardadas`),q=quota(p.id);if(q)el.append(q);
    actions(el,[['Detalhes e opções',()=>void run('settings',p.id)],['Nova sessão no terminal',()=>void run('new',p.id),'ollama lmstudio'.split(' ').includes(p.id)?false:p.installed],['Conversar com Ollama',()=>openPet('ai_ollama'),p.id==='ollama'],[p.available?'Parar motor':'Iniciar motor',()=>void run(p.available?'service-stop':'service-start','ollama'),p.id==='ollama'&&p.installed],['Atualizar quotas',()=>void refreshUsage(true),['codex','claude','copilot'].includes(p.id),usageLoading]]);content.append(el);}
   if(!content.childElementCount&&unsupportedProvider())content.append(node('p',unavailableText()));
  }else if(tab==='models'){
   const r=inventory?.resources??{};content.append(node('p',`RAM disponível ${bytesLabel(r.ram_available)} / ${bytesLabel(r.ram_total)} · modelos ${bytesLabel(r.model_memory)} · GPU ${bytesLabel(r.model_vram)} · disco ${bytesLabel(r.disk)}`));
   const models=uniqueManagedModels(inventory?.models??[]).filter(m=>(modelState==='all'||modelState==='sessions'&&m.clients.length>0||modelState==='memory'&&m.loaded||modelState==='disk'&&!m.loaded)&&matchesSearch(modelQuery,m.name,m.parameters,m.quantization,...m.aliases??[]));
   for(const m of models){const el=card(m.name,`${m.loaded?'Em memória':'Só no disco'} · ${m.parameters??''} · ${m.quantization??''} · ${bytesLabel(m.size)} disco · ${bytesLabel(m.memory)} RAM/GPU · ${m.clients.length} sessões detetadas`);if(m.aliases?.length)el.append(node('p','Aliases: '+m.aliases.join(', ')));
    actions(el,[['Conversar neste pet',()=>void converseWithModel(m.name),m.chat],['Carregar na memória',()=>void run('load','ollama',m.name),!m.loaded],['Libertar memória',()=>void run('unload','ollama',m.name),m.loaded],['Detalhes e opções',()=>void run('model-settings','ollama',m.name)]]);content.append(el);}
   if(!models.length)content.append(node('p',inventory?.summary.models==null?'O motor Ollama está indisponível. Os modelos e a memória são desconhecidos.':'Nenhum modelo neste filtro.'));
  }
  content.scrollTop=scroll;syncLookupControls();
  }finally{if(focused?.isConnected&&account.el.isConnected&&!account.el.hidden&&document.hasFocus()&&focused!==document.activeElement)focused.focus({preventScroll:true});}
 }
 async function refreshUsage(force=false){
  if(!alive||usageLoading)return;
  const generation=++usageGeneration;usageLoading=true;lastKey='';draw(false);
  try{
   let value:unknown;
   if(force)value=await request<unknown>('usage-refresh');
   else{const cached=await Bridge.petCachedUsage();if(cached?.schemaVersion!==1||cached.source!=='ia-barra-local-cache'||cached.status!=='available')throw new Error('Utilização guardada indisponível.');value=cached;}
   const next=parseManagedUsage(value);
   if(!alive||generation!==usageGeneration)return;
   usage=next;usageFromCache=!force;lastUsage=Date.now();
  }catch(error){
   if(!alive||generation!==usageGeneration)return;
   for(const q of Object.values(usage.providers))q.stale=true;lastUsage=Date.now();
   if(force)report(String(error).replace(/^Error:\s*/,''),true);
  }finally{
   if(alive&&generation===usageGeneration){usageLoading=false;lastKey='';draw(false);}
  }
 }
 async function refresh(force=false){
  if(!alive||!visible||loading||busy||document.hidden)return;loading=true;refreshButton.disabled=true;
  if(tab==='sessions')void PetObservability.refresh(force);
  try{
   const first=!inventory;const [next,jobs]=await Promise.all([request<ManagedInventory>('list'),Bridge.assistantTasks()]);if(!alive)return;inventory=next;tasks=jobs??[];
   draw(first);
   if(force||Date.now()-lastUsage>120000)void refreshUsage();
  }catch(error){if(alive)report(String(error).replace(/^Error:\s*/,''),true);}finally{loading=false;refreshButton.disabled=false;}
 }
 const unobserve=observePetObservability(()=>alive&&visible&&tab==='sessions'&&!document.hidden,()=>{if(alive&&visible&&tab==='sessions'){lastKey='';draw(false);}});
 let unlisten=()=>{},unprefs=()=>{};const timer=window.setInterval(()=>void refresh(),5000);ctx.onLeave=()=>{alive=false;account.dispose();prefs.setActive(false);window.clearInterval(timer);unlisten();unprefs();unobserve();lookup.dispose();if(retryTimer!==null)window.clearTimeout(retryTimer);};
 unlisten=await onEvent<boolean>(ctx.visibilityEvent??'hub-visible',value=>{visible=value;syncAccount();prefs.setActive(value&&tab==='profiles');if(!value){lookup.invalidate();}else{syncLookupControls();void refresh();}});if(!alive){unlisten();return;}
 await PetPreferences.init();
 const drawFavorites=()=>{
  favorites.replaceChildren();for(const [id,p]of Object.entries(PetPreferences.config?.pets??{})){if(!PetPreferences.active(id))continue;const b=button(p.name,()=>openPet(id));b.title=`Abrir ${p.name}`;favorites.append(b);}
  if(autoProvider&&ctx.petSelection){const next=selectedProvider(ctx.petSelection);if(next!==provider){advanceNavigation();provider=next;detail=null;confirmation.replaceChildren();lastKey='';draw();}}
 };
 unprefs=PetPreferences.subscribe(drawFavorites);drawFavorites();
 draw();if(tab==='profiles')void prefs.load(ctx.petSelection);void refresh();
}
