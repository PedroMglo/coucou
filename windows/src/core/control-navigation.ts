import type {ControlContext,Route,NavigationIntent,NavigationReceipt,NavigationRequest,ControlOpenedEvent,ControlCollapseEvent} from './control-contract';
export const controlRoutes:readonly Route[]=['home','system','tasks','sessions','providers','models','profiles'];
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
export function revision(v:unknown):v is string{return typeof v==='string'&&/^(0|[1-9][0-9]{0,19})$/.test(v)&&BigInt(v)<=18446744073709551615n;}
export function controlContext(v:unknown):ControlContext|null{
 if(!object(v)||v.schemaVersion!==1||!revision(v.revision)||v.authorityContract!=='nativeIdentityAndAcceptedIntent'||!object(v.mode))return null;
 if(!['pending','nativeAvailable','nativeVisible','nativeMapped','documentReady','systemEnabled','systemEligible'].every(k=>typeof v[k]==='boolean'))return null;
 const m=v.mode;if(m.kind!=='inactive'&&m.kind!=='compact'&&!(m.kind==='expanded'&&controlRoutes.includes(m.route as Route)))return null;
 if(v.systemEligible&&!(m.kind==='expanded'&&m.route==='system'&&!v.pending&&v.nativeAvailable&&v.nativeVisible&&v.nativeMapped&&v.documentReady&&v.systemEnabled))return null;
 return structuredClone(v) as unknown as ControlContext;
}
function sameContext(a:ControlContext,b:ControlContext){
 return a.revision===b.revision&&a.mode.kind===b.mode.kind&&(a.mode.kind!=='expanded'||b.mode.kind==='expanded'&&a.mode.route===b.mode.route)&&(['pending','nativeAvailable','nativeVisible','nativeMapped','documentReady','systemEnabled','systemEligible'] as const).every(k=>a[k]===b[k]);
}
export function failureContext(error:unknown):ControlContext|null{
 if(typeof error==='string'){try{error=JSON.parse(error);}catch{return null;}}
 return object(error)&&error.kind==='binding'?controlContext(error.context):null;
}
export interface NavigationTransport {context():Promise<ControlContext>;navigate(request:NavigationRequest):Promise<NavigationReceipt>}
export interface NavigationResult {accepted:boolean;context:ControlContext|null}
interface Queued {serial:number;intent:NavigationIntent;resolve:(v:NavigationResult)=>void}
/** Accepted native intent is not DOM visibility, gesture, attention or permission. */
export class ControlNavigation {
 context:ControlContext|null=null;error='';busy=true;
 private booted=false;private bootstrap:Promise<void>|null=null;private buffered:ControlContext|null=null;
 private listeners=new Set<()=>void>();private serial=0;private readonly prefix=`fe47-${Math.random().toString(36).slice(2,12)}`;private running=false;private queued:Queued|null=null;
 private opened:ControlOpenedEvent|null=null;private openedRevision='';
 onOpened:((event:ControlOpenedEvent)=>void)|null=null;
 constructor(private transport:NavigationTransport){}
 subscribe(fn:()=>void){this.listeners.add(fn);return()=>{this.listeners.delete(fn);};}
 private changed(){for(const fn of this.listeners)fn();}
 get systemEligible(){return !this.busy&&!this.error&&this.context?.systemEligible===true;}
 private accept(raw:unknown){
  const next=controlContext(raw);if(!next)return false;
  if(!this.booted){if(!this.buffered||BigInt(next.revision)>BigInt(this.buffered.revision))this.buffered=next;return false;}
  if(this.context&&BigInt(next.revision)<=BigInt(this.context.revision))return false;
  this.context=next;this.error='';this.changed();return true;
 }
 observe(raw:unknown){this.accept(raw);}
 receiveOpened(raw:unknown){
  if(!object(raw)||!revision(raw.navigationRevision)||!controlContext(raw.context)||raw.navigationRevision!==(raw.context as ControlContext).revision||!(raw.pet===null||typeof raw.pet==='string'&&raw.pet.length<=128)||!(raw.section===null||controlRoutes.includes(raw.section as Route)))return;
  const value=raw as unknown as ControlOpenedEvent;this.accept(value.context);
  if(!this.booted){this.opened=value;return;}
  if(!this.busy&&!this.context?.pending&&this.openedRevision!==value.navigationRevision&&this.context?.revision===value.navigationRevision&&sameContext(this.context,value.context)&&this.context.mode.kind==='expanded'&&(value.section===null||value.section===this.context.mode.route)){this.openedRevision=value.navigationRevision;this.onOpened?.({...value,context:this.context});}
 }
 receiveCollapse(raw:unknown){
  if(!object(raw)||!revision(raw.navigationRevision)||!controlContext(raw.context)||raw.navigationRevision!==(raw.context as ControlContext).revision||(raw.context as ControlContext).mode.kind==='expanded')return;
  this.accept((raw as unknown as ControlCollapseEvent).context);
 }
 start(){
  if(this.bootstrap)return this.bootstrap;
  this.bootstrap=(async()=>{
   try{const raw=await this.transport.context();const initial=controlContext(raw);if(!initial)throw new Error('invalid');this.buffered=this.buffered&&BigInt(this.buffered.revision)>BigInt(initial.revision)?this.buffered:initial;}
   catch{this.error='Não foi possível confirmar a navegação. Tenta abrir novamente.';}
   this.booted=true;this.context=this.buffered;this.buffered=null;this.busy=this.running||!!this.queued;this.changed();
   const opened=this.opened;this.opened=null;if(opened)this.receiveOpened(opened);
  })();return this.bootstrap;
 }
 async request(intent:NavigationIntent):Promise<NavigationResult>{
  const serial=++this.serial;this.busy=true;this.error='';this.changed();
  return new Promise(resolve=>{
   this.queued?.resolve({accepted:false,context:this.context});this.queued={serial,intent:structuredClone(intent),resolve};void this.drain();
  });
 }
 private async drain(){
  if(this.running)return;this.running=true;
  await this.start();
  while(this.queued){
   const job=this.queued;this.queued=null;this.busy=true;this.changed();let accepted=false;
   if(!this.context){try{this.accept(await this.transport.context());}catch{/* An explicit user action may re-query context; it never replays navigation. */}}
   const context=this.context;
   if(context){
    const intentId=`${this.prefix}-${job.serial}`;
    try{
     const receipt=await this.transport.navigate({expectedRevision:context.revision,intentId,intent:job.intent});
     if(!object(receipt)||receipt.intentId!==intentId||typeof receipt.applied!=='boolean'||!controlContext(receipt.context))throw new Error('invalid');
     this.accept(receipt.context);
     const mode=this.context?.mode;
     accepted=receipt.applied&&!!this.context&&sameContext(this.context,receipt.context)&&(job.intent.kind==='collapse'?mode?.kind!=='expanded':mode?.kind==='expanded'&&mode.route===job.intent.route)&&!receipt.context.pending;
    }catch(error){this.accept(failureContext(error));if(job.serial===this.serial)this.error='A navegação não foi confirmada. Tenta novamente.';}
   }else this.error='Centro indisponível. Tenta abrir novamente.';
   const latest=job.serial===this.serial;this.busy=!!this.queued;job.resolve({accepted:latest&&accepted,context:this.context});this.changed();
  }
  this.running=false;
 }
}
