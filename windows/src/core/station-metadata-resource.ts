/** Shared read-only metadata lifecycle. No timers and no constructor IO. */
export class StationMetadataResource<T> {
 snapshot:T|null=null;loading=false;error='';
 private pending:Promise<void>|null=null;private followup:Promise<void>|null=null;private invalidated=false;private last=-Infinity;private listeners=new Set<()=>void>();
 constructor(private load:(force:boolean)=>Promise<unknown>,private parse:(value:unknown)=>T,private failure:string,private now=()=>Date.now()){}
 subscribe(listener:()=>void){this.listeners.add(listener);return()=>this.listeners.delete(listener);}
 private changed(){for(const listener of this.listeners)listener();}
 /** A native completion event invalidates only the local cache. Re-read with
  * force=false: the native collector cooldown remains authoritative. */
 invalidate(read=true):Promise<void>{
  this.invalidated=true;this.last=-Infinity;
  if(!read)return Promise.resolve();
  if(!this.pending)return this.refresh();
  if(!this.followup)this.followup=this.pending.then(()=>{this.followup=null;return this.invalidated?this.refresh():undefined;});
  return this.followup;
 }
 refresh(force=false):Promise<void>{
  if(this.pending)return this.invalidated?this.invalidate():this.pending;
  if(!this.invalidated&&this.now()-this.last<(force?2000:15000))return Promise.resolve();
  this.invalidated=false;this.loading=true;
  this.pending=Promise.resolve().then(()=>this.load(force)).then(value=>{this.snapshot=this.parse(value);this.error='';}).catch(()=>{this.error=this.failure;}).finally(()=>{this.last=this.invalidated?-Infinity:this.now();this.loading=false;this.pending=null;this.changed();});
  this.changed();return this.pending;
 }
}
