import {h} from './dom';
import {quotaProvider,quotaReset,type PetCachedUsage} from '../core/pet-cached-usage';
import {stationHistoryProvider,stationHistorySource,type StationHistorySnapshot} from '../core/station-history';
import {processCensusSummary} from '../core/process-census';
import {observationCount,observationDate,observationState,observationOrigin,observationTokens,type PetObservation,type ObservedSession} from '../core/pet-observability';

/** Read-only dashboard. Opening a session keeps its immutable native identity. */
export function buildPetObservability(actions:{sessions:()=>void;library?:()=>void;provider:()=>void;open:(session:ObservedSession)=>void},activity?:HTMLElement) {
 let localLibraryMain=false;
 const library=()=>void (actions.library??actions.sessions)();
 const freshness=h('span',{class:'pet-observation-freshness'});
 const headingTitle=h('h3',{text:'Sessões'}),heading=h('div',{class:'pet-observation-heading'},headingTitle,freshness),source=h('p',{class:'pet-observation-scope'});
 const counters=h('div',{class:'pet-observation-counts'});
 const metrics=new Map<string,{button:HTMLButtonElement;value:HTMLElement;caption:HTMLElement;observed:HTMLElement}>();
 for(const[key,label]of [['opened','Sessões abertas'],['total','Histórico conhecido'],['archived','Arquivadas']]){
  const value=h('strong',{text:'—'}),caption=h('span',{text:label}),observed=h('small'),button=h('button',{type:'button',class:'pet-observation-metric',onclick:()=>{if(localLibraryMain&&key!=='opened')library();else actions.sessions();}},value,caption,observed);
  button.dataset.metric=key;counters.append(button);metrics.set(key,{button,value,caption,observed});
 }
 const breakdown=h('p',{class:'pet-observation-breakdown'});
 const censusDetail=h('p',{class:'pet-observation-scope'}),censusInfo=h('details',{class:'pet-observation-source-details'},h('summary',{text:'Sobre a deteção CLI'}),censusDetail);
 const history=h('section',{class:'pet-observation-history','aria-label':'Biblioteca deste pet no Coucou'}),historyNote=h('p',{class:'pet-observation-scope',text:'Só as conversas guardadas neste pet. Podem sobrepor-se ao histórico externo; os valores não são somados.'}),historyList=h('div',{class:'pet-observation-history-list'});history.append(historyList,historyNote);
 const histories=new Map<string,{button:HTMLButtonElement;label:HTMLElement;counts:HTMLElement;detail:HTMLElement}>();
 const list=h('div',{class:'pet-observation-list'}),empty=h('p',{class:'pet-observation-empty'});
 const more=h('button',{type:'button',class:'pet-observation-link',text:'Gerir sessões e histórico →',onclick:()=>{if(localLibraryMain)library();else actions.sessions();}});
 const usage=h('section',{class:'pet-observation-consumption','aria-label':'Consumo do provider'});
 const providerButton=h('button',{type:'button',class:'pet-observation-link',text:'Provider e quotas →',onclick:actions.provider});
 const usageHeading=h('div',{class:'pet-observation-heading'},h('h3',{text:'Consumo do provider'}),providerButton);
 const tokens=h('strong',{class:'pet-observation-token-value',text:'—'}),unit=h('span',{text:'tokens medidos'});
 const tokenLine=h('div',{class:'pet-observation-token-line'},tokens,unit);
 const tokenBreakdown=h('p'),scope=h('p',{class:'pet-observation-scope'});
 const tokenStatus=h('p',{class:'pet-observation-token-status'}),external=h('p',{class:'pet-observation-external',role:'status'});
 const quota=h('section',{class:'pet-observation-quota','aria-label':'Quota da conta'}),quotaTitle=h('h3',{text:'Quota da conta'}),quotaNote=h('p'),quotaWindows=h('div');quota.append(quotaTitle,quotaNote,quotaWindows);
 usage.append(usageHeading,tokenLine,tokenStatus,tokenBreakdown,scope,quota);
 const note=h('p',{class:'pet-observation-note'}),notice=h('p',{class:'pet-observation-notice',role:'status'});
 const el=h('section',{class:'pet-observation','aria-label':'Observabilidade deste pet'},heading,counters,source,breakdown,...(activity?[activity]:[]),censusInfo,external,list,empty,history,more,usage,note,notice);
 const rows=new Map<string,{button:HTMLButtonElement;title:HTMLElement;meta:HTMLElement;badge:HTMLElement;session:ObservedSession}>();
 let lastKey='';
 return {el,sync(value:PetObservation|null,loading=false,error='',pending=false,cached?:{provider:string;snapshot:PetCachedUsage|null;loading:boolean;error:string},station?:{snapshot:StationHistorySnapshot|null;loading:boolean;error:string}){
  const historyState=stationHistoryProvider(value?.provider??'')?station:undefined;
  const key=JSON.stringify([value,loading,error,pending,cached,historyState]);if(key===lastKey)return;lastKey=key;
  const census=processCensusSummary(value?.processCensus?{...value.processCensus,stale:value.processCensus.stale||!!error}:null);
  headingTitle.textContent=value?`Sessões · ${value.providerName}`:'Sessões';source.textContent=census?.summary??(value?.processCensus?.coverage==='notSupported'?'A contagem de clientes abertos deste provider ainda não está disponível. As conversas do pet e o histórico são apresentados separadamente.':value?.processCensus?.coverage==='unavailable'?'Não foi possível confirmar os clientes abertos deste provider. Isto não significa zero sessões.':value?.opened.detail??'A contagem de sessões abertas no pet e no terminal ainda não está confirmada.');
  censusInfo.hidden=!census;censusDetail.textContent=census?.detail??'';
  // Unsupported external history is distinct from failed supported history.
  // Only this branch can promote the known pet library, with explicit captions.
  const localHistory=value?.nativeCoverage==='notSupported'&&!stationHistoryProvider(value.provider);
  localLibraryMain=!!localHistory;
  const nativeHistory=localHistory?value?.history.find(s=>s.key==='pet'):stationHistorySource(historyState?.snapshot??null,value?.provider??'',!!historyState?.error)??value?.history.find(s=>s.key==='external');
  history.hidden=!!localHistory;
  external.textContent=nativeHistory?[nativeHistory.label,nativeHistory.updatedAt?`${nativeHistory.stale?'Leitura anterior de':'Consulta de'} ${observationDate(nativeHistory.updatedAt)}`:'Consulta por confirmar',nativeHistory.detail].join(' · '):'O histórico externo ainda está por confirmar. Os valores não são um total do computador.';
  if(historyState?.loading)external.textContent+=' A atualizar o histórico…';
  if(historyState?.error&&!external.textContent.includes(historyState.error))external.textContent+=' '+historyState.error;
  const observedAt=census?value?.processCensus?.observedAt:value?.updatedAt,stale=!!(census?value?.processCensus?.stale:value?.stale)||!!error,refreshing=loading||!!value?.refreshing||!!value?.processCensus?.refreshing;el.dataset.stale=String(stale);el.setAttribute('aria-busy',String(refreshing||!!historyState?.loading));
  freshness.textContent=refreshing?stale&&observedAt?`A atualizar · leitura de ${observationDate(observedAt)}`:'A atualizar…':observedAt?`${stale?'Dados anteriores de':'Consulta de'} ${observationDate(observedAt)}`:'Por apurar';
  freshness.title=observedAt?`Última consulta: ${observationDate(observedAt)}`:'Ainda não foi possível consultar o estado das sessões.';
  for(const[key,m]of metrics){
   const count=key==='opened'?value?.opened:nativeHistory?.[key as 'total'|'archived'];m.value.textContent=observationCount(count);m.button.disabled=pending;
   if(key==='opened'&&census)m.value.textContent=census.count;
   m.caption.textContent=key==='opened'?census?(census.unit==='terminals'?'Terminais detetados':'Clientes CLI'):'Sessões abertas':key==='total'?localHistory?'Histórico no Coucou':'Histórico conhecido':localHistory?'Arquivo no Coucou':'Arquivadas';
   m.observed.hidden=false;m.observed.textContent=key==='opened'?census?.caption??'Pet e terminal':`${count?.coverage==='unavailable'?'Por confirmar':localHistory?'Só neste pet':'Nesta fonte'}${count?.coverage==='partial'?' · parcial':''}${nativeHistory?.stale?' · leitura anterior':''}`;
   m.button.title=key==='opened'&&census?census.detail:[count?.scope,count?.detail,count?.coverage==='partial'?'Contagem parcial; podem existir mais sessões.':''].filter(Boolean).join(' · ')||'Contagem por apurar. Abrir gestão de sessões.';
   m.button.setAttribute('aria-label',`${m.caption.textContent}: ${m.value.textContent}${m.observed.hidden?'':' · '+m.observed.textContent}. ${localHistory&&key!=='opened'?'Consultar esta biblioteca no chat':'Abrir gestão de sessões'}`);
  }
  const work=value?.liveWork;
  breakdown.textContent=work?`Neste pet · Turnos ativos: ${work.active.toLocaleString('pt-PT')} · À tua espera: ${work.waiting.toLocaleString('pt-PT')} · A preparar: ${work.reserved.toLocaleString('pt-PT')} · Consulta de ${observationDate(work.observedAt)}`:value?`${observationCount(value.working)} a executar · ${observationCount(value.waiting)} à tua espera · Turnos observados em ${value.scopeLabel}. Não contam sessões abertas.`:'Os turnos em execução e os pedidos à tua espera ainda estão por confirmar.';
  breakdown.title=value?.scopeDetail??'';
  const historySources=value?.history.filter(s=>s.key==='pet')??[],historyKeys=new Set<string>(historySources.map(s=>s.key));
  for(const[key,row]of histories)if(!historyKeys.has(key)){row.button.remove();histories.delete(key);}
  for(const [index,s]of historySources.entries()){
   let row=histories.get(s.key);
   if(!row){const label=h('strong'),counts=h('span'),detail=h('small'),button=h('button',{type:'button',class:'pet-observation-history-source',onclick:library},label,counts,detail);button.dataset.historySource=s.key;row={button,label,counts,detail};histories.set(s.key,row);}
   row.label.textContent=s.label;row.counts.textContent=`${observationCount(s.total)} registos · ${observationCount(s.archived)} em arquivo`;
   row.detail.textContent=s.updatedAt?`${s.stale?'Leitura anterior de':'Consulta de'} ${observationDate(s.updatedAt)}`:'Consulta por confirmar';
   row.button.disabled=pending;row.button.dataset.stale=String(s.stale);row.button.setAttribute('aria-label',`${s.label}: ${row.counts.textContent}. ${row.detail.textContent}. Consultar esta biblioteca no chat`);
   if(historyList.children[index]!==row.button)historyList.insertBefore(row.button,historyList.children[index]??null);
  }
  const visible=value?.sessions.slice(0,3)??[],keys=new Set(visible.map(s=>s.key));
  for(const[key,row]of rows)if(!keys.has(key)){row.button.remove();rows.delete(key);}
  for(const [index,session]of visible.entries()){
   const key=session.key;let row=rows.get(key);
   if(!row){
    const title=h('strong'),meta=h('span',{class:'pet-observation-session-meta'}),badge=h('span',{class:'pet-observation-session-state'});
    const button=h('button',{type:'button',class:'pet-observation-session'},h('span',{class:'pet-observation-session-copy'},title,meta),badge);button.dataset.session=session.id;button.dataset.sessionKey=session.key;
    row={button,title,meta,badge,session};rows.set(key,row);button.onclick=()=>actions.open(rows.get(key)!.session);
   }
   row.session=session;row.title.textContent=session.title||`Sessão #${session.id.slice(0,8)}`;
   const folder=session.cwd.replace(/[\\/]+$/,'').split(/[\\/]/).pop();
   row.meta.textContent=[session.origin==='pet'&&session.owner?`Pet ${session.owner}`:observationOrigin(session.origin),folder,session.archived===true?'Em arquivo':'',session.model,`#${session.id.slice(0,8)}`].filter(Boolean).join(' · ');
   row.badge.textContent=observationState(session.state);row.button.disabled=pending||!session.canOpen||!session.id||!session.backend;
   row.button.dataset.state=session.state;row.button.title=[session.cwd,session.id,session.updatedAt?`Última atividade: ${observationDate(session.updatedAt)}`:'',row.button.disabled?'Abertura desta sessão indisponível. Consulta a gestão.':'Abrir esta sessão no chat expansivo'].filter(Boolean).join('\n');
   row.button.setAttribute('aria-label',`${row.title.textContent} · ${row.meta.textContent} · ${row.badge.textContent} · Abrir chat`);
   if(list.children[index]!==row.button)list.insertBefore(row.button,list.children[index]??null);
  }
  more.disabled=providerButton.disabled=pending;
  empty.hidden=visible.length>0||!!value;empty.textContent=loading&&!value?'A consultar sessões…':'Ainda não há sessões com detalhe disponível nesta consulta. Isto não confirma que não existam sessões abertas.';
  more.textContent=localHistory?'Ver histórico no chat →':visible.length&&value&&!value.sessionsComplete?'Ver restantes e gerir sessões →':'Gerir sessões e histórico →';
  const measured=value?.tokens,total=observationTokens(measured?.totalTokens),feedback=value?.tokenStatus;
  usage.dataset.status=feedback?.status??'unknown';tokenStatus.textContent=feedback?.title??'Medições por confirmar';
  tokens.textContent=measured?.coverage==='unavailable'?'—':total;
  unit.textContent=!measured||total==='—'?'total por apurar':measured.coverage==='partial'?'tokens medidos · parcial':'tokens medidos';
  const input=observationTokens(measured?.inputTokens),output=observationTokens(measured?.outputTokens);
  tokenBreakdown.hidden=input==='—'&&output==='—';
  tokenBreakdown.textContent=[input==='—'?'':`${input} de entrada`,output==='—'?'':`${output} gerados`].filter(Boolean).join(' · ');
  scope.textContent=measured?[measured.since?`Desde ${observationDate(measured.since)}`:'Período por apurar',measured.updatedAt?`Última medição ${observationDate(measured.updatedAt)}`:'',measured.scope,measured.detail].filter(Boolean).join(' · '):[feedback?.since?`Registo desde ${observationDate(feedback.since)}`:'',feedback?.detail??'Consumo acumulado por apurar. A última resposta não representa o total do provider.'].filter(Boolean).join(' · ');
  const provider=quotaProvider(cached?.provider??value?.provider??'');quota.hidden=!provider;quota.dataset.provider=provider??'';quotaWindows.replaceChildren();
  if(provider){
   quotaTitle.textContent=`Quota da conta · ${provider}`;const snapshot=cached?.snapshot,q=snapshot?.providers[provider];
   quota.dataset.stale=String(!!cached?.error||q?.stale!==false);
   quotaNote.textContent=cached?.loading&&!snapshot?'A ler a quota guardada…':!snapshot?'Ainda não há quota confirmada nesta vista.':snapshot.status==='notSupported'?'A quota guardada não está disponível nesta plataforma.':snapshot.status==='unavailable'?'Não foi possível consultar a quota guardada. Isto não significa saldo zero.':!q?'Não há quota guardada para este provider. Isto não significa utilização zero nem quota ilimitada.':[q.available===false?'Quota da conta indisponível nesta observação.':'Quota da conta guardada localmente.',q.updated?`Medição de ${observationDate(q.updated)}`:'Data da medição por apurar.',q.stale?'Dados desatualizados.':'',q.blocked?'Utilização incluída bloqueada.':''].filter(Boolean).join(' · ');
   if(cached?.error)quotaNote.textContent+=' '+cached.error;
   if(q){for(const w of q.windows??[]){const reset=quotaReset(w.reset_at??w.reset),line=h('div',{class:'pet-observation-quota-window'}),label=[w.label,w.name].filter((v,i,a)=>v&&a.indexOf(v)===i).join(' · ')||'Período por apurar';
    const bar=h('progress',{max:100,value:w.used,'aria-label':`${label}: ${w.used.toLocaleString('pt-PT')}% da quota utilizada`});line.append(h('span',{text:`${label} · ${w.used.toLocaleString('pt-PT',{maximumFractionDigits:1})}% utilizada`}),bar,h('small',{text:reset?`Renova ${reset}`:'Renovação por apurar'}));quotaWindows.append(line);}
    if(q.unlimited?.length)quotaWindows.append(h('p',{text:`Sem limite comunicado para: ${q.unlimited.join(', ')}.`}));
    if(!q.windows?.length&&!q.unlimited?.length)quotaWindows.append(h('p',{text:'Sem limites numéricos confirmados nesta observação.'}));
   }
   quotaWindows.append(h('p',{class:'pet-observation-scope',text:'Origem: quota da conta no registo local. Percentagens de quota não são tokens. Consulta ou atualiza em Provider e quotas.'}));
  }
  note.hidden=!value?.notes.length;note.textContent=value?.notes.join(' · ')??'';
  notice.hidden=!error;notice.textContent=error?`${error}${value?' Os valores anteriores ficam visíveis com a data da consulta.':''}`:'';
 }};
}
