import {h} from './dom';
import {CodexAccountActivity,listenAccountActivity,accountNumber,accountDuration,accountDate,accountReason,type AccountUsageResource} from '../core/account-usage';
export function buildAccountUsage(options:{resource?:AccountUsageResource;compact?:boolean;visible?:()=>boolean}={}){
 const resource=options.resource??CodexAccountActivity;
 const el=h('section',{class:'account-usage','aria-label':'Atividade da conta Codex observada'});el.hidden=true;
 const heading=h('div',{class:'account-usage-heading'},h('h3',{text:'Conta Codex · última observação'}));
 const status=h('p',{class:'account-usage-status',role:'status','aria-live':'polite'}),date=h('p',{class:'account-usage-date'});
 const metrics=h('dl',{class:'account-usage-metrics'}),extra=h('dl',{class:'account-usage-metrics'}),details=h('details',{class:'account-usage-details'},h('summary',{text:'Mais atividade comunicada'}),extra);
 const rows=new Map<string,HTMLElement>();
 for(const[key,label]of [['lifetimeTokens','Tokens comunicados pela conta'],['peakDailyTokens','Pico diário comunicado'],['longestRunningTurnSec','Maior duração de um turno'],['currentStreakDays','Dias seguidos na observação'],['longestStreakDays','Maior sequência de dias']]){
  const value=h('dd',{text:'—'}),row=h('div',{},h('dt',{text:label}),value);value.dataset.accountMetric=key;rows.set(key,value);(options.compact!==false&&rows.size>2?extra:metrics).append(row);
 }
 const refresh=h('button',{type:'button',text:'Consultar atividade da conta'}),query=h('button',{type:'button',text:'Consultar estado guardado'});
 refresh.title='Consultar estatísticas pela sessão nativa do Codex; não inicia um chat.';query.title='Reler só a observação guardada, sem consultar a conta.';
 const actions=h('div',{class:'account-usage-actions'},refresh,query),feedback=h('p',{class:'account-usage-feedback',role:'status'});
 const note=h('p',{class:'account-usage-note',text:'A conta pode ter mudado desde esta observação. Estes valores não são o uso deste pet, as quotas ou o registo de pedidos do Coucou; não são somados.'});
 el.append(heading,status,date,metrics);if(options.compact!==false)el.append(details);el.append(actions,feedback,note);
 let provider='',context='',contextEpoch=0,requestEpoch=-1,enabled=false,active=false,disposed=false;
 const allowed=()=>!disposed&&enabled&&provider==='codex'&&!document.hidden&&(options.visible?.()??true);
 function render(){
  const show=allowed();el.hidden=!show;if(!show)return;
  const snapshot=resource.snapshot,summary=snapshot?.summary;el.dataset.status=snapshot?.status??'unknown';el.dataset.stale=String(snapshot?.stale!==false);el.setAttribute('aria-busy',String(resource.querying||resource.requesting||resource.refreshing));
  status.textContent=resource.error||(resource.requesting?'A pedir a consulta da atividade…':snapshot?.refreshing?'A consultar a atividade da conta…':accountReason(snapshot?.reason)||(!snapshot?'Observação guardada por confirmar.':snapshot.summary?'Estatísticas comunicadas pela conta observada.':'Contadores por apurar.'));
  const observed=accountDate(snapshot?.observedAt);date.textContent=observed?`Observação local de ${observed}${snapshot?.stale?' · dados anteriores':''}`:'Data da observação por confirmar';
  for(const[key,node]of rows){const raw=summary?.[key as keyof typeof summary]??null;node.textContent=key==='longestRunningTurnSec'?accountDuration(raw):accountNumber(raw);node.title=raw===null?'Valor não comunicado pela fonte.':`${raw}${key==='longestRunningTurnSec'?' segundos':key.includes('Tokens')?' tokens':' dias'}`;}
  refresh.disabled=!resource.canRefresh;refresh.setAttribute('aria-busy',String(resource.requesting));
  query.hidden=snapshot?.status==='available'&&!resource.error&&!resource.requesting&&!snapshot.refreshing;query.setAttribute('aria-busy',String(resource.querying));
  const localFeedback=requestEpoch===contextEpoch?resource.feedback:'';feedback.hidden=!localFeedback;feedback.textContent=localFeedback;
 }
 function lifecycle(){
  const next=allowed();if(next&&!active){active=true;if(resource===CodexAccountActivity)void listenAccountActivity()?.then(()=>{if(allowed())void resource.query();});else void resource.query();}
  else if(!next){if(active)contextEpoch++;active=false;}render();
 }
 refresh.onclick=()=>{if(allowed()&&el.isConnected){const epoch=contextEpoch;requestEpoch=epoch;void resource.refresh(()=>allowed()&&el.isConnected&&epoch===contextEpoch);}};query.onclick=()=>{if(allowed()&&el.isConnected)void resource.query(true);};
 const unsubscribe=resource.subscribe(render);document.addEventListener('visibilitychange',lifecycle);
 return{el,setContext(physicalProvider:string,visible:boolean,owner=physicalProvider){if(provider!==physicalProvider||enabled!==visible||context!==owner)contextEpoch++;provider=physicalProvider;enabled=visible;context=owner;lifecycle();},dispose(){disposed=true;active=false;unsubscribe();document.removeEventListener('visibilitychange',lifecycle);el.hidden=true;}};
}
