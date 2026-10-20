import type {Settings} from './state';

export type IslandAvailabilityKey='petControlBar'|'petSystemCenter';
export type IslandAvailabilityValues=Pick<Settings,IslandAvailabilityKey>;
export type IslandAvailabilityPatch=Partial<IslandAvailabilityValues>;
export type IslandAvailabilityStatus='idle'|'saving'|'checking'|'saved'|'changed'|'failed'|'unconfirmed';
export function islandAvailabilityValues(input:unknown):IslandAvailabilityValues|null{
 if(!input||typeof input!=='object')return null;
 const flags=input as Partial<IslandAvailabilityValues>;
 return typeof flags.petControlBar==='boolean'&&typeof flags.petSystemCenter==='boolean'?{petControlBar:flags.petControlBar,petSystemCenter:flags.petSystemCenter}:null;
}

/** These are persisted preferences, not proof that an OS surface is visible.
 * Readback adopts only the two owned fields; newer events beat older reads.
 * Writes use the dedicated two-field patch; no whole-settings rollback or
 * assertion that an ACK/event is the only concurrent preference transaction. */
export class IslandAvailabilityPreferences{
 private confirmed:IslandAvailabilityValues={petControlBar:false,petSystemCenter:false};
 private observation=0;
 private reading=false;
 known=false;
 pending:IslandAvailabilityKey|null=null;
 status:IslandAvailabilityStatus='idle';
 constructor(private options:{
  initial:unknown;
  save(patch:IslandAvailabilityPatch):Promise<unknown>;
  read():Promise<unknown>;
  adopt(flags:IslandAvailabilityValues):void;
  changed():void;
 }){
  const initial=islandAvailabilityValues(options.initial);if(initial){this.confirmed=initial;this.known=true;}else this.status='unconfirmed';
 }
 get flags():Readonly<IslandAvailabilityValues>{return Object.freeze({...this.confirmed});}
 get busy(){return this.pending!==null||this.reading;}
 /** A full authoritative native settings event, including events from another
  * window. This local counter orders observations, not native transactions. */
 observe(settings:unknown){
  this.observation++;
  const flags=islandAvailabilityValues(settings);
  if(flags){this.accept(flags);if(!this.busy&&this.status==='unconfirmed')this.status='saved';}
  else{this.known=false;if(!this.busy)this.status='unconfirmed';}
  this.options.changed();
 }
 private accept(flags:IslandAvailabilityValues){
  this.confirmed={...flags};this.known=true;this.options.adopt({...flags});
 }
 private async reconcile():Promise<boolean>{
  const observation=this.observation;this.reading=true;this.options.changed();
  try{
   const snapshot=await this.options.read();
   if(observation===this.observation){const flags=islandAvailabilityValues(snapshot);if(flags)this.accept(flags);else this.known=false;}
  }catch{if(observation===this.observation)this.known=false;}
  finally{this.reading=false;}
  return this.known;
 }
 async change(key:IslandAvailabilityKey){
  if(this.busy||!this.known)return;
  const desired=!this.confirmed[key],patch:IslandAvailabilityPatch=Object.freeze({[key]:desired});
  this.pending=key;this.status='saving';this.options.changed();
  let failed=false;
  try{await this.options.save(patch);}catch{failed=true;}
  this.status='checking';this.options.changed();
  const known=await this.reconcile();
  this.status=!known?'unconfirmed':failed?'failed':this.confirmed[key]!==desired?'changed':'saved';
  this.pending=null;this.options.changed();
 }
 async refresh(){
  if(this.busy)return;
  this.status='checking';
  const known=await this.reconcile();this.status=known?'saved':'unconfirmed';this.options.changed();
 }
}
