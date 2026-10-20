import {Bridge,onEvent} from './bridge';
import type {CodexAccountUsage,CodexAccountSummary} from './account-usage-contract';
export type {CodexAccountUsage,CodexAccountSummary} from './account-usage-contract';
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const fields=['lifetimeTokens','peakDailyTokens','longestRunningTurnSec','currentStreakDays','longestStreakDays'] as const;
const failures=['notObserved','nativeUnavailable','notSignedIn','invalidData','clockUnavailable','unsupportedPlatform'];
const instance=(v:unknown):v is string=>typeof v==='string'&&/^[0-9a-f]{32}$/.test(v);
const revision=(v:unknown):v is string=>typeof v==='string'&&/^(0|[1-9][0-9]{0,19})$/.test(v)&&BigInt(v)<=18446744073709551615n;
const metric=(v:unknown):v is string|null=>v===null||typeof v==='string'&&/^(0|[1-9][0-9]{0,18})$/.test(v)&&BigInt(v)<=9223372036854775807n;
const time=(v:unknown):v is number|null=>v===null||typeof v==='number'&&Number.isSafeInteger(v)&&v>0;
export function parseAccountUsage(raw:unknown):CodexAccountUsage{
 if(!object(raw)||raw.schemaVersion!==1||raw.provider!=='codex'||raw.source!=='codexAppServerAccountUsage'||raw.scope!=='lastObservedCodexAccount'||!(raw.sourceInstance===null||instance(raw.sourceInstance))||!revision(raw.revision)||!['available','unavailable'].includes(String(raw.status))||!time(raw.observedAt)||!time(raw.readAt)||typeof raw.stale!=='boolean'||typeof raw.refreshing!=='boolean'||!(raw.reason===null||failures.includes(String(raw.reason))))throw Error('Observação da conta com formato incompatível.');
 let summary:CodexAccountSummary|null=null;
 if(raw.summary!==null){if(!object(raw.summary)||!fields.every(k=>metric((raw.summary as Record<string,unknown>)[k])))throw Error('Métricas da conta incompatíveis.');summary=Object.fromEntries(fields.map(k=>[k,(raw.summary as Record<string,unknown>)[k]])) as unknown as CodexAccountSummary;}
 if(raw.status==='available'&&(!summary||!raw.sourceInstance||raw.observedAt===null||raw.refreshing||raw.reason!==null&&raw.reason!=='clockUnavailable')||raw.status==='unavailable'&&(summary!==null||raw.observedAt!==null)||raw.sourceInstance===null&&(summary!==null||raw.refreshing))throw Error('Estado da conta incompatível.');
 return {schemaVersion:1,provider:'codex',source:'codexAppServerAccountUsage',sourceInstance:raw.sourceInstance as string|null,revision:raw.revision,scope:'lastObservedCodexAccount',status:raw.status as CodexAccountUsage['status'],observedAt:raw.observedAt,readAt:raw.readAt,stale:raw.stale,refreshing:raw.refreshing,summary,reason:raw.reason as CodexAccountUsage['reason']};
}
function validHint(raw:unknown){return object(raw)&&raw.schemaVersion===1&&raw.provider==='codex'&&raw.source==='codexAppServerAccountUsage'&&instance(raw.sourceInstance)&&revision(raw.revision);}
function sameStored(a:CodexAccountUsage,b:CodexAccountUsage){return a.status===b.status&&a.refreshing===b.refreshing&&a.observedAt===b.observedAt&&JSON.stringify(a.summary)===JSON.stringify(b.summary)&&(a.reason===b.reason||a.reason==='clockUnavailable'||b.reason==='clockUnavailable');}
export function accountNumber(v:string|null|undefined,locale='pt-PT'){return metric(v)&&v!==null?BigInt(v).toLocaleString(locale):'—';}
export function accountDuration(v:string|null|undefined,locale='pt-PT'){
 if(!metric(v)||v===null)return '—';const sec=BigInt(v),days=sec/86400n,hours=(sec%86400n)/3600n,minutes=(sec%3600n)/60n,seconds=sec%60n;
 return `${days?days.toLocaleString(locale)+' d · ':''}${hours.toString().padStart(2,'0')}:${minutes.toString().padStart(2,'0')}:${seconds.toString().padStart(2,'0')}`;
}
export function accountDate(v:number|null|undefined){return typeof v==='number'&&Number.isSafeInteger(v)&&v>0&&v<=8.64e12?new Date(v*1000).toLocaleString('pt-PT',{day:'numeric',month:'short',year:'numeric',hour:'2-digit',minute:'2-digit',second:'2-digit'}):null;}
export function accountReason(v:CodexAccountUsage['reason']|undefined){return ({notObserved:'Ainda não há uma observação da atividade da conta.',nativeUnavailable:'O leitor nativo do Codex não está disponível nesta consulta.',notSignedIn:'Esta fonte requer a sessão nativa do Codex com autenticação ChatGPT.',invalidData:'O Codex não forneceu métricas válidas nesta consulta.',clockUnavailable:'O relógio local não permite confirmar a data ou a idade da observação.',unsupportedPlatform:'Esta fonte de atividade não está disponível nesta plataforma.'} as Record<string,string>)[v??'']??'';}
export interface AccountTransport {query():Promise<unknown>;refresh():Promise<unknown>}
interface Attempt {id:number;epoch:number;source:string;base:string;active:()=>boolean;reconciled:boolean}
/** Native cached QUERY alone publishes state. ACK/hint never publish or set a floor. */
export class AccountUsageResource {
 private native:CodexAccountUsage|null=null;private producer:string|null=null;private binding=0;private retired=new Set<string>();
 private readEpoch=0;private readId=0;private pendingRead:Promise<void>|null=null;private hintQueued=false;
 private attemptId=0;private attempt:Attempt|null=null;private masked=true;private listeners=new Set<()=>void>();
 querying=false;requesting=false;error='';feedback='';
 constructor(private transport:AccountTransport){}
 subscribe(fn:()=>void){this.listeners.add(fn);return()=>{this.listeners.delete(fn);};}
 private changed(){for(const fn of this.listeners)fn();}
 get snapshot(){return this.masked||this.requesting&&this.attempt&&this.native?.sourceInstance===this.attempt.source&&BigInt(this.native.revision)<=BigInt(this.attempt.base)?null:this.native;}
 get refreshing(){return this.native?.refreshing===true;}
 get canRefresh(){return !this.masked&&!this.error&&!this.requesting&&!this.refreshing&&this.producer!==null&&this.native?.sourceInstance===this.producer&&this.native.reason!=='unsupportedPlatform';}
 query(force=false):Promise<void>{
  if(this.pendingRead&&!force)return this.pendingRead;
  const id=++this.readId,epoch=this.readEpoch,binding=this.binding;this.querying=true;this.changed();
  const current=()=>id===this.readId&&epoch===this.readEpoch&&binding===this.binding;
  const task=Promise.resolve().then(()=>this.transport.query()).then(raw=>{
   if(!current())return;let next=parseAccountUsage(raw);
   if(next.sourceInstance&&this.retired.has(next.sourceInstance))return;
   if(next.sourceInstance!==this.producer){
    if(this.producer)this.retired.add(this.producer);this.producer=next.sourceInstance;this.binding++;
    this.attemptId++;this.attempt=null;this.requesting=false;this.feedback='';
   }else if(this.native){
    const order=BigInt(next.revision)-BigInt(this.native.revision);if(order<0n)return;
    if(order===0n){if(!sameStored(this.native,next))throw Error('Observações contraditórias.');next={...next,stale:next.stale||this.native.stale};}
   }
   this.native=next;this.masked=false;this.error='';
   const attempt=this.attempt;
   if(attempt&&attempt.epoch===this.binding&&attempt.source===this.producer&&BigInt(next.revision)>BigInt(attempt.base)&&!next.refreshing){this.requesting=false;attempt.reconciled=true;this.feedback='';}
   this.changed();
  }).catch(()=>{if(current()){this.masked=true;this.error='Não foi possível confirmar a observação guardada da conta.';this.changed();}}).finally(()=>{if(id===this.readId){this.querying=false;this.pendingRead=null;this.changed();}});
  this.pendingRead=task;return task;
 }
 private reconcile(){this.readEpoch++;this.masked=true;this.changed();return this.query(true);}
 hint(raw:unknown){
  if(!validHint(raw)||this.hintQueued)return;this.hintQueued=true;this.readEpoch++;this.masked=true;this.changed();
  // Payload ordinals/producer are deliberately NOT applied or stored.
  queueMicrotask(()=>{this.hintQueued=false;void this.query(true);});
 }
 async refresh(active:()=>boolean=()=>true){
  if(!active()||!this.canRefresh||!this.producer||!this.native)return;
  const attempt:Attempt={id:++this.attemptId,epoch:this.binding,source:this.producer,base:this.native.revision,active,reconciled:false};this.attempt=attempt;
  this.readEpoch++;this.masked=true;this.requesting=true;this.feedback='';this.error='';this.changed();let failure:unknown=null;
  try{await this.transport.refresh();}catch(error){failure=error;}
  // A superseded owner/runtime can neither clear a newer operation nor rebind it.
  if(attempt.id!==this.attemptId||attempt.epoch!==this.binding||attempt.source!==this.producer)return;
  this.requesting=false;
  if(attempt.active()&&!attempt.reconciled&&failure!==null){let code=failure;if(typeof code==='string'){try{code=JSON.parse(code);}catch{/* Native enum may already be plain text. */}}
   this.feedback=code==='busy'?'O pedido não foi iniciado porque já havia uma consulta em curso.':code==='cooldown'?'O pedido não foi iniciado porque a consulta anterior é recente.':code==='unavailable'?'Não foi possível iniciar a consulta nesta execução.':'A resposta da consulta não foi confirmada. O estado guardado será relido.';
  }
  this.attempt=null;
  // Completion fences reads STARTED DURING the refresh, not only older reads.
  // ACK contents (including instance, revision and summary) are never published.
  await this.reconcile();
 }
}
export const CodexAccountActivity=new AccountUsageResource({query:()=>Bridge.petAccountUsage(),refresh:()=>Bridge.petAccountUsageRefresh()});
let listening:Promise<void>|null=null;
export function listenAccountActivity(){
 if(!listening)listening=onEvent('pet-account-usage-changed',raw=>CodexAccountActivity.hint(raw)).then(()=>{},()=>{listening=null;});
 return listening;
}
