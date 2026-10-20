import {Bridge} from './bridge';

export type AudioKind='output'|'input'|'playback'|'recording'|'card';
export interface AudioChoice {id:string;label:string;available:boolean}
export interface AudioNode {
 key:string;kind:AudioKind;label:string;description:string;volume:number|null;muted:boolean|null;
 balance:number|null;isDefault:boolean;deviceKey:string|null;monitor:boolean;controllable:boolean;
 canVolume:boolean;canBalance:boolean;active:boolean;options:AudioChoice[];activeOption:string|null;
}
export interface AudioDeviceSnapshot {
 available:boolean;observedAt:number|null;outputs:AudioNode[];inputs:AudioNode[];
 cards:AudioNode[];error:string|null;
}
export interface AudioSnapshot extends AudioDeviceSnapshot {playback:AudioNode[];recording:AudioNode[]}
/** Stream fields are absent, not an observation of zero applications. */
export interface SystemAudioSnapshot extends AudioDeviceSnapshot {scope:'systemDevices'}
export interface AudioRequest {
 targetKey:string;action:'volume'|'mute'|'default'|'route'|'port'|'profile'|'balance';
 value?:number;muted?:boolean;destinationKey?:string;option?:string;
}
export interface AudioReceipt<T extends AudioDeviceSnapshot=AudioSnapshot> {snapshot:T;confirmed:boolean;warning:string|null}
export interface AudioTransport<T extends AudioDeviceSnapshot=AudioSnapshot> {snapshot():Promise<T>;control(request:AudioRequest):Promise<AudioReceipt<T>>}
const message=(error:unknown)=>String(error).replace(/^Error:\s*/,'');

/** Observations are the only playback/device state. A late poll cannot replace
 * a newer command receipt; failed reads keep the last values visibly stale. */
export class AudioController<T extends AudioDeviceSnapshot=AudioSnapshot> {
 snapshot:T|null=null;
 loading=false;pending=false;stale=true;error='';warning='';
 private epoch=0;private lastRead=0;private listeners=new Set<()=>void>();
 constructor(private transport:AudioTransport<T>){}
 subscribe(listener:()=>void){this.listeners.add(listener);return()=>this.listeners.delete(listener);}
 private notify(){for(const listener of this.listeners)listener();}
 private accept(snapshot:T){
  if(this.snapshot?.observedAt&&snapshot.observedAt&&snapshot.observedAt<this.snapshot.observedAt)return;
  if(snapshot.available){this.snapshot=snapshot;this.stale=false;this.error='';}
  else{this.stale=true;this.error=snapshot.error||'O serviço de som não está disponível.';}
 }
 /** Stop publishing a hidden surface's in-flight reads/receipts. A command
  * already accepted by the native engine is not cancelled or rolled back. */
 invalidate(){this.epoch++;this.loading=false;this.stale=true;this.lastRead=0;this.notify();}
 async refresh(force=false){
  if(this.loading||this.pending||!force&&Date.now()-this.lastRead<1800)return;
  this.lastRead=Date.now();this.loading=true;const epoch=this.epoch;this.notify();
  try{const snapshot=await this.transport.snapshot();if(epoch===this.epoch)this.accept(snapshot);}
  catch(error){if(epoch===this.epoch){this.stale=true;this.error=message(error);}}
  finally{if(epoch===this.epoch){this.loading=false;this.notify();}}
 }
 async control(request:AudioRequest){
  if(this.pending||this.stale)return false;
  // Invalidate an already running poll before sending. Its finally must not
  // clear the pending state or install values from before this command.
  const epoch=++this.epoch;this.loading=false;this.pending=true;this.error=this.warning='';this.notify();
  try{
   const receipt=await this.transport.control({...request});if(epoch!==this.epoch)return false;this.accept(receipt.snapshot);
   this.warning=receipt.warning||(!receipt.confirmed?'O estado ainda não confirmou o pedido. Atualiza antes de repetir.':'');
   return receipt.confirmed;
  }catch(error){if(epoch===this.epoch){this.stale=true;this.error=message(error);}return false;}
  finally{this.pending=false;this.lastRead=0;this.notify();}
 }
}
export const SystemAudio=new AudioController({snapshot:()=>Bridge.petAudioSnapshot(),control:request=>Bridge.petAudioControl(request)});
export function systemDeviceSnapshot(value:SystemAudioSnapshot):SystemAudioSnapshot {
 if(value?.scope!=='systemDevices'||!Array.isArray(value.outputs)||!Array.isArray(value.inputs)||!Array.isArray(value.cards)||
  typeof value.available!=='boolean'||value.observedAt!==null&&(!Number.isSafeInteger(value.observedAt)||value.observedAt<=0||value.observedAt>8_640_000_000_000_000))throw new Error('A leitura do som do sistema não está disponível.');
 if(value.outputs.some(n=>n.kind!=='output')||value.inputs.some(n=>n.kind!=='input')||value.cards.some(n=>n.kind!=='card'))throw new Error('A leitura do som do sistema não corresponde aos dispositivos.');
 return {scope:'systemDevices',available:value.available,observedAt:value.observedAt,outputs:value.outputs,inputs:value.inputs,cards:value.cards,error:value.error};
}
/** This controller has no Spoty permissions, music state or stream catalogue. */
export const IslandSystemAudio=new AudioController<SystemAudioSnapshot>({
 snapshot:async()=>systemDeviceSnapshot(await Bridge.petSystemAudioSnapshot()),
 control:async request=>{const receipt=await Bridge.petSystemAudioControl(request);return {...receipt,snapshot:systemDeviceSnapshot(receipt.snapshot)};},
});
