import {Bridge,onEvent,type AgentRequest} from './bridge';
import {sameSession,type SessionTarget} from './session-chat';

export interface RequestDestination {readonly id:string;readonly target:SessionTarget}
/** Routing metadata only. Native request/decision authority stays in Rust. */
export class RequestDestinations {
 private pending=new Map<string,RequestDestination>();
 private ended=new Set<string>();
 private epoch=0;
 get size(){return this.pending.size;}
 beginRead(){return this.epoch;}
 applyRead(epoch:number,requests:Pick<AgentRequest,'id'|'target'>[]){
  if(epoch!==this.epoch)return false;
  const live=new Set(requests.filter(request=>request.target).map(request=>request.id));
  for(const id of this.pending.keys())if(!live.has(id))this.end(id);
  for(const request of requests)this.add(request);
  return true;
 }
 add(request:Pick<AgentRequest,'id'|'target'>):RequestDestination|null{
  if(!request.id||!request.target||this.ended.has(request.id))return null;
  const existing=this.pending.get(request.id);
  if(existing)return sameSession(existing.target,request.target)?existing:null;
  const value=Object.freeze({id:request.id,target:Object.freeze({...request.target})});
  this.pending.set(request.id,value);
  this.epoch++;
  if(this.pending.size>128)this.pending.delete(this.pending.keys().next().value!);
  return value;
 }
 end(id:string){if(!this.pending.has(id)&&this.ended.has(id))return;this.pending.delete(id);this.ended.add(id);this.epoch++;if(this.ended.size>256)this.ended.delete(this.ended.values().next().value!);}
 current(value:RequestDestination){return this.pending.get(value.id)===value;}
 get(id:string){return this.pending.get(id)??null;}
 retired(id:string){return this.ended.has(id);}
 forPet(pet:string){return [...this.pending.values()].find(value=>value.target.petId===pet)??null;}
}
export const requestDestinations=new RequestDestinations();
export async function refreshRequestDestinations(){
 for(let attempt=0;attempt<2;attempt++){
  const epoch=requestDestinations.beginRead();
  try{if(requestDestinations.applyRead(epoch,await Bridge.cliAgentPending()))return true;}catch{return false;}
 }
 return false;
}
let initialized:Promise<void>|null=null;
export function initializeRequestDestinations(){
 if(initialized)return initialized;
 initialized=(async()=>{
  await Promise.all([
   onEvent<AgentRequest>('agent-request',request=>requestDestinations.add(request)),
   onEvent<{id:string}>('agent-request-ended',request=>requestDestinations.end(request.id)),
  ]);
  // Epochs reject an old snapshot even after its tombstone was evicted.
  await refreshRequestDestinations();
 })();return initialized;
}
