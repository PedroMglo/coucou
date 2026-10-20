import {sameSession,type SessionSummary} from '../core/session-chat';
import type {ManagedCodexSession} from '../core/managed-codex';

export type ListSource='mochi'|'external'|'managed';
export interface ListOwner {pet:string;provider:string}
interface Rows {mochi:SessionSummary[];external:SessionSummary[];managed:ManagedCodexSession[]}
export interface ListResource<T> {rows:T[];phase:'idle'|'ready'|'error'|'partial'|'notApplicable';loading:boolean;error:string|null}
export type ListResources={[K in ListSource]:ListResource<Rows[K][number]>};
export type ListLoaders={[K in ListSource]:(owner:Readonly<ListOwner>)=>Promise<Rows[K]>};
const sources:ListSource[]=['mochi','external','managed'];
const empty=<T>():ListResource<T>=>({rows:[],phase:'idle',loading:false,error:null});
const resources=():ListResources=>({mochi:empty(),external:empty(),managed:empty()});
interface Pending {promise:Promise<unknown[]>;managedUpdates:ManagedCodexSession[]}
/** Preserve newer native generations and unrelated receipts. A release only
 * removes the exact receipt generation; it does not stop an external session. */
export function mergeManagedReceipt(rows:ManagedCodexSession[],receipt:ManagedCodexSession){
 const same=rows.find(row=>row.handle===receipt.handle&&sameSession(row.target,receipt.target));
 if(same&&same.generation>receipt.generation)return rows;
 const kept=rows.filter(row=>row.handle!==receipt.handle||!sameSession(row.target,receipt.target));
 return receipt.phase==='released'?kept:[...kept,receipt];
}

/** Queries have independent lifecycles. A context change detaches publication,
 * not the native operation. Reopening an owner reuses its pending query. */
export class SessionListLoading {
 private owner:Readonly<ListOwner>|null=null;
 private epoch=0;
 private pending=new Map<string,Pending>();
 private value=resources();
 constructor(private readonly loaders:ListLoaders,private readonly changed:()=>void){}
 get state():Readonly<ListResources>{return this.value;}
 reset(owner:ListOwner|null){
  this.epoch++;this.owner=owner?{pet:owner.pet,provider:owner.provider}:null;this.value=resources();
  if(!owner?.provider)this.value.mochi.phase='notApplicable';
  if(owner?.pet!=='agent_codex'||owner.provider!=='codex')this.value.managed.phase='notApplicable';
  this.changed();
 }
 refresh(manual=false,only?:ListSource){
  // Recheck the epoch after each callback: changed() is allowed to navigate.
  const epoch=this.epoch;
  for(const source of only?[only]:sources){if(epoch!==this.epoch)return;this.start(source,manual);}
 }
 managedReceipt(receipt:ManagedCodexSession){
  const owner=this.owner;if(!owner||receipt.target.petId!==owner.pet||receipt.target.provider!==owner.provider||receipt.target.origin!=='external'||this.value.managed.phase==='notApplicable')return;
  const pending=this.pending.get(this.key('managed',owner));if(pending)pending.managedUpdates.push(receipt);
  const state=this.value.managed;state.rows=mergeManagedReceipt(state.rows,receipt);
  if(state.phase==='error')state.phase='partial';this.changed();
 }
 private key(source:ListSource,owner:Readonly<ListOwner>){return JSON.stringify([source,owner.pet,owner.provider]);}
 private start<K extends ListSource>(source:K,manual:boolean){
  const owner=this.owner,state=this.value[source];
  if(!owner||state.phase==='notApplicable'||state.loading||(!manual&&(state.phase==='error'||state.phase==='partial')))return;
  const key=this.key(source,owner),epoch=this.epoch;
  let request=this.pending.get(key);
  if(!request){
   request={promise:Promise.resolve().then(async()=>await this.loaders[source](owner)).then(rows=>{
    if(!Array.isArray(rows))throw Error('A origem não devolveu uma lista válida.');return rows;
   }),managedUpdates:[]};
   this.pending.set(key,request);
  }
  const query=request;state.loading=true;this.changed();
  void query.promise.then(rows=>{
   if(this.epoch!==epoch||this.owner!==owner)return;
   const updated=source==='managed'?query.managedUpdates.reduce(mergeManagedReceipt,rows as ManagedCodexSession[]):rows;
   this.value[source]={rows:[...updated],phase:'ready',loading:false,error:null} as ListResources[K];this.changed();
  },error=>{
   if(this.epoch!==epoch||this.owner!==owner)return;
   state.phase=state.phase==='ready'||state.phase==='partial'||state.rows.length>0?'partial':'error';state.loading=false;
   state.error=String(error).replace(/^Error:\s*/,'');this.changed();
  }).finally(()=>{if(this.pending.get(key)===query)this.pending.delete(key);});
 }
}
