import type {ControlContext,NetworkView,NetworkSnapshot,NetworkTicket,NetworkCollection,WifiNetwork,NetworkReadRequest,NetworkScanRequest,NetworkEnableWifiRequest,NetworkTicketRequest,ContextPrecondition} from './control-contract';
const obj=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const text=(v:unknown,max=4096):v is string=>typeof v==='string'&&v.length<=max;
const enableScope=(v:unknown)=>v===null||text(v,128)&&!/[\u0000-\u001f\u007f-\u009f]/.test(v)&&new TextEncoder().encode(v).length<=128;
const flag=(v:unknown)=>v===null||typeof v==='boolean';
const signal=(v:unknown)=>v===null||typeof v==='number'&&Number.isFinite(v)&&v>=0&&v<=100;
const named=(v:unknown)=>obj(v)&&text(v.name);
function collection(v:unknown,row:(v:unknown)=>boolean){return v===null||obj(v)&&Array.isArray(v.entries)&&v.entries.length<=4096&&v.entries.every(row)&&typeof v.complete==='boolean'&&Number.isSafeInteger(v.observedRows)&&Number(v.observedRows)>=v.entries.length&&Number.isSafeInteger(v.rejectedRows)&&Number(v.rejectedRows)>=0;}
export function networkView(raw:unknown):NetworkView|null{
 if(!obj(raw)||(raw.schemaVersion!==1&&raw.schemaVersion!==2)||raw.source!=='dmsNetwork'||!(raw.generation===null||text(raw.generation,128))||typeof raw.stale!=='boolean'||typeof raw.refreshing!=='boolean'||!(raw.scanScope===null||text(raw.scanScope,128))||!obj(raw.capabilities)||!['read','scan'].every(k=>typeof (raw.capabilities as Record<string,unknown>)[k]==='boolean')||!['connect','radio','credentials'].every(k=>(raw.capabilities as Record<string,unknown>)[k]===false)||!Array.isArray(raw.targets)||raw.targets.length>4096||!raw.targets.every(t=>obj(t)&&text(t.handle,128)&&typeof t.kind==='string'&&['wifiDevice','accessPoint','vpnProfile'].includes(t.kind)&&text(t.label))||!(raw.reason===null||text(raw.reason,64)))return null;
 if(raw.schemaVersion===2&&(!enableScope(raw.enableWifiScope)||typeof raw.capabilities.enableWifi!=='boolean'))return null;
 if(raw.data!==null){
  const d=raw.data;if(!obj(d)||d.schemaVersion!==1||d.source!=='dmsNetwork'||d.readOnly!==true||typeof d.observedAt!=='number'||!Number.isFinite(d.observedAt)||d.observedAt<0||d.observedAt>8640000000000000||!obj(d.wifi)||!obj(d.hotspot))return null;
  const w=d.wifi,h=d.hotspot;
  if(!['enabled','connected','connecting'].every(k=>flag(w[k]))||!(w.currentName===null||text(w.currentName))||!signal(w.signal)||!['ethernetConnected','cellularConnected','cellularEnabled','cellularHardwareEnabled'].every(k=>flag(d[k]))||!['supported','available','configured','enabled','activating','secured'].every(k=>flag(h[k]))||!(h.name===null||text(h.name)))return null;
  const wifi=(v:unknown)=>named(v)&&obj(v)&&signal(v.signal)&&['secured','saved','connected','outOfRange'].every(k=>flag(v[k]));
  const vpn=(v:unknown)=>named(v)&&obj(v)&&(v.state===null||text(v.state,128));
  if(!collection(w.nearby,wifi)||!collection(w.saved,wifi)||!collection(w.devices,named)||!collection(d.vpnProfiles,vpn)||!collection(d.activeVpns,vpn)||!Array.isArray(d.warnings)||d.warnings.length>128||!d.warnings.every(x=>text(x,512)))return null;
 }
 const copy=structuredClone(raw) as unknown as Record<string,unknown>;
 if(raw.schemaVersion===1){delete copy.enableWifiScope;delete (copy.capabilities as Record<string,unknown>).enableWifi;}
 return copy as unknown as NetworkView;
}
const phases=['queued','running','observed','acknowledged','cancelled','unknown','failed'];
export function networkTicket(raw:unknown):NetworkTicket|null{
 if(!obj(raw)||!text(raw.id,128)||!raw.id||typeof raw.kind!=='string'||!['read','scan','enableWifi'].includes(raw.kind)||typeof raw.phase!=='string'||!phases.includes(raw.phase)||!(raw.reason===null||text(raw.reason,64))||raw.effectConfirmed!==false||typeof raw.refreshRecommended!=='boolean'||raw.kind==='enableWifi'&&raw.phase==='observed')return null;
 return {id:raw.id,kind:raw.kind,phase:raw.phase,reason:raw.reason,effectConfirmed:false,refreshRecommended:raw.refreshRecommended} as NetworkTicket;
}
export function networkReason(reason:string|null|undefined){
 if(!reason)return '';
 if(['staleContext','transitionPending','superseded','revoked','hidden','systemDisabled','documentInactive','closed','unauthorized','nativeCallerMismatch'].includes(reason))return 'O contexto mudou. Abre novamente Redes para consultar o estado atual.';
 if(['busy','capacity'].includes(reason))return 'O sistema está ocupado. Podes tentar novamente.';
 if(['cancelledBeforeDispatch','abandonedBeforeDispatch'].includes(reason))return 'Pedido cancelado antes de ser enviado.';
 if(['cancelledAfterDispatch','alreadyDispatched','unknownAfterDispatch'].includes(reason))return 'O pedido já pode ter sido enviado. O efeito no sistema não está confirmado.';
 if(['sourceChanged','staleTarget','wrongTarget','cacheExpired','notObserved','notFound'].includes(reason))return 'É necessária uma nova leitura do estado da rede.';
 if(['unsupported','nativeUnavailable','unavailable'].includes(reason))return 'A leitura de redes não está disponível neste sistema.';
 if(reason==='deadline')return 'O estado não foi confirmado a tempo. Podes atualizar manualmente.';
 return 'O sistema não confirmou o pedido. Podes atualizar manualmente.';
}
export const networkFlag=(v:boolean|null|undefined,yes='Ligado',no='Desligado')=>v===true?yes:v===false?no:'Por confirmar';
export function collectionSummary<T>(v:NetworkCollection<T>|null|undefined){return !v?'Leitura indisponível':!v.complete||v.rejectedRows>0?`${v.entries.length} apresentados · leitura parcial${v.rejectedRows?` · ${v.rejectedRows} não interpretados`:''}`:v.entries.length?`${v.entries.length} apresentados`:'Nenhum observado';}
export interface NetworkTransport {
 read(request:NetworkReadRequest):Promise<NetworkTicket>;view(request:ContextPrecondition):Promise<NetworkView>;
 scan(request:NetworkScanRequest):Promise<NetworkTicket>;enableWifi?(request:NetworkEnableWifiRequest):Promise<NetworkTicket>;ticket(request:NetworkTicketRequest):Promise<NetworkTicket>;cancel(request:NetworkTicketRequest):Promise<NetworkTicket>;
}
function enableNonce(){try{const value=globalThis.crypto.randomUUID().replace(/-/g,'');return /^[a-f0-9]{32}$/.test(value)?value:null;}catch{return null;}}
export class SystemNetwork {
 view:NetworkView|null=null;ticket:NetworkTicket|null=null;busy=false;cancelling=false;message='';stale=true;
 private active=false;private epoch=0;private work=0;private contextRevision='';private timer:ReturnType<typeof setTimeout>|null=null;private seq=0;private polls=0;
 private pendingEnable=false;private enableReadbackIssued=false;private nonce=enableNonce();
 private listeners=new Set<()=>void>();
 constructor(private transport:NetworkTransport,private context:()=>ControlContext|null,private failure:(e:unknown)=>void=()=>{}){}
 subscribe(fn:()=>void){this.listeners.add(fn);return()=>{this.listeners.delete(fn);};}
 private changed(){for(const fn of this.listeners)fn();}
 private clearTimer(){if(this.timer!==null)clearTimeout(this.timer);this.timer=null;}
 private current(epoch:number,revision:string,work=this.work){const c=this.context();return this.active&&work===this.work&&epoch===this.epoch&&c?.systemEligible===true&&c.pending===false&&c.revision===revision;}
 setActive(active:boolean){
  const c=this.context(),next=active&&c?.systemEligible===true&&c.pending===false;
  if(this.active===next&&(!next||this.contextRevision===c!.revision))return;
  this.epoch++;this.clearTimer();this.active=next;this.contextRevision=next?c!.revision:'';this.busy=false;this.cancelling=false;this.ticket=null;this.stale=true;this.message='';this.pendingEnable=false;this.enableReadbackIssued=false;this.changed();
  if(next)void this.refresh(false);
 }
 private error(e:unknown,current:()=>boolean){
  this.failure(e);if(!current())return;let v=e;if(typeof v==='string'){try{v=JSON.parse(v);}catch{v=null;}}
  this.message=networkReason(obj(v)&&text(v.code,64)?v.code:'nativeRejected');
  if(this.pendingEnable)this.message+=' O pedido de ativação continua por confirmar. Atualiza o estado manualmente.';
  this.stale=true;this.changed();
 }
 get available(){const c=this.context();return this.active&&c?.systemEligible===true&&c.pending===false&&c.revision===this.contextRevision;}
 get canScan(){return this.available&&!this.busy&&!this.pendingTicket&&!this.pendingEnable&&!this.stale&&!this.view?.stale&&!this.view?.refreshing&&this.view?.capabilities.scan===true&&!!this.view.scanScope;}
 get canEnableWifi(){const v=this.view;return this.available&&!!this.nonce&&typeof this.transport.enableWifi==='function'&&!this.busy&&!this.pendingTicket&&!this.pendingEnable&&!this.stale&&v?.schemaVersion===2&&!v.stale&&!v.refreshing&&v.capabilities.enableWifi===true&&!!v.enableWifiScope&&v.data?.wifi.enabled===false;}
 get enableUnconfirmed(){return this.pendingEnable;}
 get enableUnavailable(){
  if(this.pendingEnable)return 'O pedido anterior ainda está por confirmar. Atualiza o estado.';
  if(this.busy||this.pendingTicket)return 'Aguarda o pedido atual.';
  if(!this.available||this.stale||this.view?.stale||this.view?.refreshing)return 'É necessária uma leitura atual do estado da rede.';
  if(this.view?.data?.wifi.enabled===true)return 'Já está ativado na configuração de software observada.';
  if(this.view?.data?.wifi.enabled!==false)return 'O estado de software está por confirmar. Atualiza o estado.';
  return 'A ativação não está disponível nesta observação.';
 }
 get pendingTicket(){return !!this.ticket&&['queued','running'].includes(this.ticket.phase);}
 async refresh(force=true){
  const c=this.context();if(!this.available||this.busy||!c)return;
  const epoch=this.epoch,rev=c.revision,work=++this.work,previous=this.ticket;this.clearTimer();this.busy=true;this.polls=0;
  // Keep the cancellable enable handle if local polling ended before native work.
  if(previous?.kind==='enableWifi'&&this.pendingTicket){this.message='A consultar o estado do pedido de ativação…';this.changed();await this.poll(previous,epoch,rev,work);return;}
  this.ticket=null;this.message=this.pendingEnable?'Pedido de ativação por confirmar · a obter uma nova leitura de software…':'A obter o estado das redes…';this.changed();
  try{const t=networkTicket(await this.transport.read({expectedRevision:rev,force}));if(!this.current(epoch,rev,work))return;if(!t||t.kind!=='read')throw new Error('invalid');await this.track(t,epoch,rev,work);}
  catch(e){if(this.current(epoch,rev,work)){this.busy=false;this.error(e,()=>this.current(epoch,rev,work));}}
 }
 async scan(){
  const c=this.context();if(!this.canScan||!c)return;
  const epoch=this.epoch,rev=c.revision,work=++this.work,scope=this.view!.scanScope!;this.busy=true;this.ticket=null;this.polls=0;this.message='A pedir uma pesquisa global de redes…';this.changed();
  try{const t=networkTicket(await this.transport.scan({expectedRevision:rev,scope,intentId:`scan47-${++this.seq}`}));if(!this.current(epoch,rev,work))return;if(!t||t.kind!=='scan')throw new Error('invalid');await this.track(t,epoch,rev,work);}
  catch(e){if(this.current(epoch,rev,work)){this.busy=false;this.error(e,()=>this.current(epoch,rev,work));}}
 }
 async enableWifi(){
  const c=this.context(),v=this.view;if(!this.canEnableWifi||!c||v?.schemaVersion!==2||!v.enableWifiScope||!this.transport.enableWifi)return;
  const epoch=this.epoch,rev=c.revision,work=++this.work,scope=v.enableWifiScope,intentId=`enwifi-${this.nonce}-${++this.seq}`;
  this.clearTimer();this.busy=true;this.ticket=null;this.polls=0;this.pendingEnable=true;this.enableReadbackIssued=false;this.stale=true;this.message='A pedir a ativação da configuração Wi-Fi…';this.changed();
  try{const t=networkTicket(await this.transport.enableWifi({expectedRevision:rev,scope,intentId}));if(!this.current(epoch,rev,work))return;if(!t||t.kind!=='enableWifi')throw new Error('invalid');await this.track(t,epoch,rev,work);}
  catch(e){if(this.current(epoch,rev,work)){this.busy=false;this.error(e,()=>this.current(epoch,rev,work));}}
 }
 private async track(t:NetworkTicket,epoch:number,rev:string,work:number){
  if(!this.current(epoch,rev,work))return;this.ticket=t;
  if(t.phase==='queued'||t.phase==='running'){
   this.busy=true;this.message=t.kind==='enableWifi'?'Ativação pendente · estado de software por confirmar.':t.kind==='scan'?'Pesquisa pendente · ainda sem resultado confirmado.':this.pendingEnable?'Ativação pedida · nova leitura de software pendente…':'Leitura pendente…';this.changed();
   if(++this.polls>40){this.busy=false;this.stale=true;this.message='O estado continua por confirmar. Atualiza manualmente para consultar o pedido.';this.changed();return;}
   this.clearTimer();this.timer=setTimeout(()=>{this.timer=null;void this.poll(t,epoch,rev,work);},500);return;
  }
  if(t.kind==='enableWifi'){
   this.busy=false;this.stale=true;
   this.message=t.reason?networkReason(t.reason):t.phase==='acknowledged'?'Pedido de ativação aceite; estado de software por confirmar.':'O estado do pedido de ativação não foi confirmado. Atualiza manualmente.';
   this.changed();
   // Backend contract: only this ACK is published after worker join/active.take.
   // Unknown/Cancelled/Failed do NOT prove I/O drained and never auto-read.
   if(t.phase==='acknowledged'&&t.reason===null&&!this.enableReadbackIssued&&this.current(epoch,rev,work)){this.enableReadbackIssued=true;await this.refresh(true);}
   return;
  }
  this.message=t.reason?networkReason(t.reason):t.phase==='cancelled'?'Acompanhamento cancelado. O efeito no sistema não está confirmado.':t.phase==='unknown'||t.phase==='failed'?'O estado do pedido não foi confirmado.':t.kind==='scan'?'Pesquisa aceite pelo sistema. O resultado não está confirmado; atualiza para consultar as redes.':t.phase==='observed'?'':'O estado da rede não foi confirmado.';this.changed();
  if(!(t.kind==='read'&&t.phase==='observed'||t.kind==='scan'&&t.phase==='acknowledged')){this.busy=false;this.stale=true;this.changed();return;}
  try{const v=networkView(await this.transport.view({expectedRevision:rev}));if(!this.current(epoch,rev,work))return;if(!v)throw new Error('invalid');this.view=v;this.stale=v.stale;this.busy=false;
   if(t.kind==='read'&&!v.stale&&!v.refreshing){this.pendingEnable=false;}
   this.changed();}
  catch(e){if(this.current(epoch,rev,work)){this.busy=false;this.error(e,()=>this.current(epoch,rev,work));}}
 }
 private async poll(t:NetworkTicket,epoch:number,rev:string,work:number){
  if(!this.current(epoch,rev,work))return;
  try{const next=networkTicket(await this.transport.ticket({expectedRevision:rev,ticketId:t.id}));if(!this.current(epoch,rev,work))return;if(!next||next.id!==t.id||next.kind!==t.kind)throw new Error('invalid');await this.track(next,epoch,rev,work);}
  catch(e){if(this.current(epoch,rev,work)){this.busy=false;this.error(e,()=>this.current(epoch,rev,work));}}
 }
 async cancel(){
  const t=this.ticket,c=this.context();if(!this.available||this.cancelling||!t||!['queued','running'].includes(t.phase)||!c)return;
  const epoch=this.epoch,rev=c.revision,work=++this.work;this.clearTimer();this.busy=true;this.cancelling=true;this.message='A pedir cancelamento…';this.changed();
  try{const next=networkTicket(await this.transport.cancel({expectedRevision:rev,ticketId:t.id}));if(!this.current(epoch,rev,work))return;if(!next||next.id!==t.id||next.kind!==t.kind)throw new Error('invalid');this.cancelling=false;await this.track(next,epoch,rev,work);}
  catch(e){if(this.current(epoch,rev,work)){this.busy=false;this.error(e,()=>this.current(epoch,rev,work));}}
  finally{if(this.current(epoch,rev,work)){this.cancelling=false;this.changed();}}
 }
 dispose(){this.active=false;this.epoch++;this.clearTimer();this.listeners.clear();}
}
export type {NetworkSnapshot,WifiNetwork};
