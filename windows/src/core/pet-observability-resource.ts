import {Bridge,onEvent} from './bridge';
import type {PetObservabilitySnapshot} from './pet-observability-contract';
import {validPetCapabilities} from './pet-capabilities';
type Snapshot=PetObservabilitySnapshot;

/** One read per webview, shared by every card. Slow reads cannot block local
 * navigation; a failed refresh retains its original measurement timestamps. */
export class PetObservabilityResource {
 snapshot:Snapshot|null=null;
 loading=false;
 error='';
 private lastAttempt=-Infinity;
 private pending:Promise<void>|null=null;
 private again=false;
 private evidenceEpoch=0;
 private listeners=new Set<()=>void>();
 constructor(private load:()=>Promise<Snapshot>,private clock=()=>Date.now()){}
 subscribe(fn:()=>void){this.listeners.add(fn);return()=>this.listeners.delete(fn);}
 private changed(){for(const fn of this.listeners)fn();}
 invalidate(){this.lastAttempt=-Infinity;this.evidenceEpoch++;if(this.loading)this.again=true;if(this.snapshot?.petCapabilities){this.snapshot={...this.snapshot,petCapabilities:null};this.changed();}}
 refresh(force=false):Promise<void>{
  if(this.pending)return this.pending;
  if(!force&&this.clock()-this.lastAttempt<5000)return Promise.resolve();
  this.lastAttempt=this.clock();this.loading=true;const epoch=this.evidenceEpoch;
  this.pending=Promise.resolve().then(()=>this.load()).then(value=>{
   if(value?.schemaVersion!==1||!Array.isArray(value.pets)||!Array.isArray(value.providers))throw Error('A informação das sessões não tem um formato compatível.');
   if(value.petCapabilities!=null&&!validPetCapabilities(value.petCapabilities))throw Error('As capacidades não têm um formato compatível.');
   this.snapshot=epoch===this.evidenceEpoch||value.petCapabilities==null?value:{...value,petCapabilities:null};this.error='';
  }).catch(()=>{if(this.snapshot?.petCapabilities)this.snapshot={...this.snapshot,petCapabilities:null};this.error='Não foi possível atualizar as sessões e o consumo. Tenta Atualizar.';}).finally(()=>{
   this.pending=null;this.loading=false;const again=this.again;this.again=false;this.changed();if(again)void this.refresh(true);
  });
  this.changed();return this.pending;
 }
}
export const PetObservability=new PetObservabilityResource(()=>Bridge.petObservability());

const observers=new Set<()=>boolean>();
/** Each webview installs one event stream and shares one short refresh burst. */
export function observePetObservability(visible:()=>boolean,changed:()=>void){
 observers.add(visible);const unsubscribe=PetObservability.subscribe(changed);
 return()=>{observers.delete(visible);unsubscribe();};
}
let refreshTimer:number|null=null;
for(const event of ['pet-observability-changed','pet-configuration-changed','chat-turn-state','assistant-history-changed','assistant-tasks-changed']){
 void onEvent(event,()=>{
  if(![...observers].some(visible=>visible())){PetObservability.invalidate();return;}
  if(refreshTimer!==null)window.clearTimeout(refreshTimer);
  refreshTimer=window.setTimeout(()=>{refreshTimer=null;PetObservability.invalidate();if([...observers].some(visible=>visible()))void PetObservability.refresh(true);},80);
 });
}
