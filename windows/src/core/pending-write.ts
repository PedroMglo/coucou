export interface WriteState {pending:boolean;error:string;retryable:boolean}
/** Serialise complete preference snapshots so an older acknowledgement cannot
 * overtake a newer edit. Keep the latest failed snapshot available for retry. */
export class PendingWrite<T> {
 private queue:{value:T;version:number;resolve:(ok:boolean)=>void}[]=[];
 private running=false;private version=0;
 private failure:{value:T;version:number;message:string}|null=null;
 constructor(private transport:(value:T)=>Promise<unknown>,private changed:(state:WriteState)=>void){}
 private publish(){this.changed({pending:this.running||this.queue.length>0,error:this.failure?.message??'',retryable:!!this.failure&&this.failure.version===this.version});}
 write(value:T):Promise<boolean>{
  const snapshot=structuredClone(value),version=++this.version;this.failure=null;
  const result=new Promise<boolean>(resolve=>this.queue.push({value:snapshot,version,resolve}));
  void this.flush();this.publish();return result;
 }
 retry():Promise<boolean>{const failure=this.failure;return failure&&failure.version===this.version?this.write(failure.value):Promise.resolve(false);}
 private async flush(){
  if(this.running)return;this.running=true;
  try{while(this.queue.length){
   const job=this.queue.shift()!;let ok=false;
   try{await this.transport(job.value);ok=true;if(job.version===this.version)this.failure=null;}
   catch(error){if(job.version===this.version)this.failure={value:job.value,version:job.version,message:String(error).replace(/^Error:\s*/,'')};}
   job.resolve(ok);
  }}finally{this.running=false;this.publish();}
 }
}
