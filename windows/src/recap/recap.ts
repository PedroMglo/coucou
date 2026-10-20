// Weekly recap: metadata reads and causal preference receipts. No timers beyond
// the existing Monday presentation delay; no automatic replay of failed writes.
import {Bridge,IS_TAURI,type RecapPersistenceReceipt} from '../core/bridge';
import type {IslandViewName} from '../core/layout';
import {State} from '../core/state';
import {DEFAULT_PREFS,dayKey,mondayOf,previousWeekStart,sampleHistory,shouldAutoShow,summarize,type RecapPrefs,type WeeklySummary} from './summary';
export interface RecapHost {alert(view:IslandViewName):void}
const QUIET_VIEWS:ReadonlySet<IslandViewName>=new Set(['overview','empty','settings','greeting']);
const SHOW_DELAY_MS=1500;
export function parseRecapPrefs(value:unknown):RecapPrefs {
 const p=value as Partial<RecapPrefs>|null;
 if(!p||Array.isArray(p)||Object.keys(p).length!==3||Object.keys(p).some(k=>!['enabled','hideProjects','lastShownWeek'].includes(k))||typeof p.enabled!=='boolean'||typeof p.hideProjects!=='boolean'||typeof p.lastShownWeek!=='string'||p.lastShownWeek.length>128||/[\u0000-\u001f\u007f]/.test(p.lastShownWeek))throw new Error('O estado do resumo não foi confirmado.');
 return {enabled:p.enabled,hideProjects:p.hideProjects,lastShownWeek:p.lastShownWeek};
}
export function parseRecapReceipt(value:unknown):RecapPersistenceReceipt {
 const r=value as Partial<RecapPersistenceReceipt>|null;
 if(!r||Array.isArray(r)||Object.keys(r).length!==4||Object.keys(r).some(k=>!['prefs','persisted','historyCleared','errorCode'].includes(k))||typeof r.persisted!=='boolean'||typeof r.historyCleared!=='boolean'||r.errorCode!==null&&r.errorCode!=='writeFailed'&&r.errorCode!=='invalidWeek'||r.persisted&&r.errorCode!==null||!r.persisted&&r.errorCode===null||r.historyCleared&&!r.persisted)throw new Error('A gravação do resumo não foi confirmada.');
 return {prefs:parseRecapPrefs(r.prefs),persisted:r.persisted,historyCleared:r.historyCleared,errorCode:r.errorCode!};
}
export type RecapOperation='enabled'|'hide'|'clear'|'shown';
export function recapReceiptSuccess(op:RecapOperation,r:RecapPersistenceReceipt,value?:boolean|string){
 return r.persisted&&r.errorCode===null&&(op==='clear'?r.historyCleared:op==='enabled'?r.prefs.enabled===value:op==='hide'?r.prefs.hideProjects===value:r.prefs.lastShownWeek===value);
}
export function recapFailureMessage(op:RecapOperation,r:RecapPersistenceReceipt,value?:boolean|string){
 if(op==='clear')return r.persisted?'A resposta não confirmou a limpeza do histórico.':'Não foi possível guardar a limpeza. O histórico foi preservado.';
 if(op==='enabled'&&value===false&&!r.prefs.enabled)return 'Registo desativado nesta sessão; não foi possível guardar em disco.';
 if(op==='hide'&&value===true&&r.prefs.hideProjects)return 'Nomes ocultos nesta sessão; não foi possível guardar em disco.';
 if(op==='shown')return 'Resumo apresentado; não foi possível guardar que já foi mostrado.';
 return 'Não foi possível guardar a preferência. Foi mantido o estado confirmado pelo sistema.';
}
export class RecapController {
 summary:WeeklySummary|null=null;prefs:RecapPrefs={...DEFAULT_PREFS};version=0;
 prefsKnown=false;hidePending=false;notice='';noticeFrom:'read'|'hide'|'shown'|null=null;retryHide:boolean|null=null;
 private checking=false;private scheduled=false;private prefIntent=0;private readIntent=0;private loadIntent=0;
 private presentedWeeks=new Set<string>();private listeners=new Set<()=>void>();
 subscribe(listener:()=>void){this.listeners.add(listener);return()=>this.listeners.delete(listener);}
 private changed(){for(const listener of this.listeners)listener();}
 private settlePreferenceWrite(intent:number){
  // Reads started during a write share its start stamp. Fence them again when
  // that write settles, without invalidating a newer user intent's observations.
  if(intent!==this.prefIntent)return;
  this.prefIntent++;this.readIntent++;
 }
 async readPreferences(){
  const read=++this.readIntent,intent=this.prefIntent;
  try{const prefs=parseRecapPrefs(await Bridge.recapPrefs());if(read!==this.readIntent||intent!==this.prefIntent||this.hidePending)return false;this.prefs=prefs;this.prefsKnown=true;if(this.retryHide===null){this.notice='';this.noticeFrom=null;}this.changed();return true;}
  catch{if(read===this.readIntent&&intent===this.prefIntent&&!this.hidePending){this.prefsKnown=false;this.noticeFrom='read';this.notice='Não foi possível confirmar as preferências do resumo. Atualiza antes de exportar.';this.changed();}return false;}
 }
 async load(now=new Date()):Promise<WeeklySummary|null>{
  const ticket=++this.loadIntent,read=++this.readIntent,intent=this.prefIntent,start=previousWeekStart(now);
  const history=IS_TAURI?await Bridge.recapHistory(start.getTime()/1000):sampleHistory(start);
  if(ticket!==this.loadIntent)return this.summary;
  if(history&&intent===this.prefIntent&&read===this.readIntent&&!this.hidePending){try{this.prefs=parseRecapPrefs(history.prefs);this.prefsKnown=true;}catch{this.prefsKnown=false;}}
  this.summary=history?summarize(history,start):null;this.version++;this.changed();return this.summary;
 }
 async open(host:RecapHost){await this.load();host.alert('recap');}
 async check(host:RecapHost,now=new Date()){
  const week=dayKey(mondayOf(now));
  if(this.hidePending||this.checking||this.scheduled||this.presentedWeeks.has(week)||!shouldAutoShow(now,'')||!this.quiet())return;
  this.checking=true;const intent=this.prefIntent;
  try{
   const prefs=parseRecapPrefs(await Bridge.recapPrefs());
   if(intent!==this.prefIntent||!prefs.enabled||!shouldAutoShow(now,prefs.lastShownWeek))return;
   if(!await this.load(now)||intent!==this.prefIntent)return;
   this.scheduled=true;
   window.setTimeout(()=>{void this.present(host,now,week,intent);},SHOW_DELAY_MS);
  }catch{if(intent===this.prefIntent){this.noticeFrom='shown';this.notice='Não foi possível confirmar as preferências do resumo.';this.changed();}}
  finally{this.checking=false;}
 }
 private async present(host:RecapHost,now:Date,week:string,intent:number){
  try{
   if(intent!==this.prefIntent||this.hidePending||!this.quiet())return;
   const prefs=parseRecapPrefs(await Bridge.recapPrefs());
   if(intent!==this.prefIntent||this.hidePending||!this.quiet()||!prefs.enabled||!shouldAutoShow(now,prefs.lastShownWeek))return;
   this.prefs=prefs;this.prefsKnown=true;this.presentedWeeks.add(week);host.alert('recap');
   if(intent!==this.prefIntent||this.hidePending)return;
   const operation=++this.prefIntent;
   try{const receipt=parseRecapReceipt(await Bridge.recapMarkShown(week));if(operation!==this.prefIntent)return;this.prefs=receipt.prefs;this.prefsKnown=true;this.noticeFrom='shown';this.notice=recapReceiptSuccess('shown',receipt,week)?'':recapFailureMessage('shown',receipt,week);}
   catch{if(operation===this.prefIntent){this.noticeFrom='shown';this.notice='Resumo apresentado; não foi possível confirmar que ficou registado como mostrado.';}}
   finally{this.settlePreferenceWrite(operation);}
  }catch{if(intent===this.prefIntent){this.noticeFrom='shown';this.notice='Não foi possível confirmar as preferências do resumo.';}}
  finally{this.scheduled=false;this.changed();}
 }
 private quiet(){return !State.paused&&!State.pendingApproval&&(State.mode!=='expanded'||QUIET_VIEWS.has(State.view));}
 async setHideProjects(hide:boolean):Promise<RecapPersistenceReceipt|null>{
  if(this.hidePending)return null;const intent=++this.prefIntent;this.hidePending=true;this.retryHide=null;this.notice='';this.noticeFrom='hide';this.changed();
  try{
   const receipt=parseRecapReceipt(await Bridge.recapSetHideProjects(hide));if(intent!==this.prefIntent)return null;
   this.prefs=receipt.prefs;this.prefsKnown=true;
   if(!recapReceiptSuccess('hide',receipt,hide)){this.retryHide=hide;this.notice=recapFailureMessage('hide',receipt,hide);}return receipt;
  }catch{if(intent===this.prefIntent){this.prefsKnown=false;this.retryHide=hide;this.notice='Não foi possível confirmar a gravação. Atualiza o estado antes de exportar.';}return null;}
  finally{this.settlePreferenceWrite(intent);this.hidePending=false;this.changed();}
 }
}
export const Recap=new RecapController();
