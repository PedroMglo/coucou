import {sessionTargetKey,type SessionTarget} from './session-chat';
import {SessionDrafts,draftTarget,parseStoredDraft,type DraftStorage,type DraftTarget,type StoredDraft,type UnboundDraft} from './session-drafts';
export interface InlineIdentity {petId:string;provider:string;conversationId:string|null}
export interface InlineCapture {target:SessionTarget;key:string;text:string;edit:number}
const pending=(id:InlineIdentity):UnboundDraft=>({petId:id.petId,provider:id.provider,origin:'unbound',sessionId:'inline-composer'});
const exact=(id:InlineIdentity):SessionTarget|null=>id.conversationId?{petId:id.petId,provider:id.provider,origin:'mochi',sessionId:id.conversationId,conversationId:id.conversationId}:null;
/** Coordinates the inline producer only. The same SessionDrafts/native CAS
 * persists data; this object grants no send, model, session or memory authority. */
export class InlineDrafts {
 readonly resource:SessionDrafts;
 target:DraftTarget|null=null;candidate:SessionTarget|null=null;identity:InlineIdentity|null=null;
 epoch=0;recovery:StoredDraft|null=null;recovering=false;recoveryError=false;
 private edits=new Map<string,number>();private recoveryTicket=0;
 constructor(private storage:DraftStorage,private changed:()=>void=()=>{},restore?:(target:DraftTarget,text:string,previous:string)=>boolean,published?:(key:string)=>void){
  this.resource=new SessionDrafts(storage,{changed:()=>this.changed(),restore:(target,text,previous)=>!!this.target&&sessionTargetKey(target)===sessionTargetKey(this.target)&&(restore?.(target,text,previous)??true),published:key=>{published?.(key);if(this.identity&&key===sessionTargetKey(pending(this.identity)))void this.refreshRecovery();}});
 }
 get key(){return this.target?sessionTargetKey(this.target):'';}
 get text(){return this.resource.map.get(this.key)??'';}
 get needsChoice(){return this.target?.origin==='unbound'&&!!this.candidate;}
 get sendable(){return this.target?.origin==='mochi'&&!!this.candidate&&sessionTargetKey(this.target)===sessionTargetKey(this.candidate);}
 select(identity:InlineIdentity|null){
  const before=this.identity;if(JSON.stringify(before)===JSON.stringify(identity))return false;
  const ownerChanged=before?.petId!==identity?.petId||before?.provider!==identity?.provider;
  if(this.target)void this.resource.flush(this.target);
  this.identity=identity?{...identity}:null;this.epoch++;this.candidate=identity?exact(identity):null;
  if(!identity){this.target=null;this.recovery=null;this.recoveryTicket++;}
  else if(ownerChanged){this.target=this.candidate??pending(identity);this.recovery=null;void this.refreshRecovery();}
  else if(!(this.target?.origin==='unbound'&&this.text.length)){this.target=this.candidate??pending(identity);}
  this.resource.activate(this.target,this.epoch);this.changed();return true;
 }
 edit(target:DraftTarget,value:string){
  const t=draftTarget(target),key=sessionTargetKey(t);if((this.resource.map.get(key)??'')!==value)this.edits.set(key,(this.edits.get(key)??0)+1);
  this.resource.map.set(key,value);
 }
 capture():InlineCapture|null{
  if(!this.sendable||this.target?.origin!=='mochi')return null;
  return{target:{...this.target},key:this.key,text:this.text,edit:this.edits.get(this.key)??0};
 }
 complete(capture:InlineCapture){
  if((this.edits.get(capture.key)??0)!==capture.edit||(this.resource.map.get(capture.key)??'')!==capture.text)return false;
  this.edit(capture.target,'');return true;
 }
 /** Deliberate copy/open after CID resolution. Source remains durable. */
 async choose(copy:boolean){
  const source=this.target,destination=this.candidate,epoch=this.epoch,text=this.text;
  if(source?.origin!=='unbound'||!destination)return false;const key=this.key,edit=this.edits.get(key)??0;
  await this.resource.flush(source);const status=this.resource.status(source);
  // Compatibility storage may explicitly support only bound drafts. Only an
  // explicit copy can leave that RAM-only slot; it is never marked durable.
  const unsupported=status?.error==='unsupported';
  if(epoch!==this.epoch||key!==this.key||text!==this.text||edit!==(this.edits.get(key)??0)||status?.busy||status?.error&&!(unsupported&&copy)||status?.phase==='conflict')return false;
  this.target={...destination};this.epoch++;this.resource.activate(this.target,this.epoch);
  if(copy&&text)this.edit(this.target,text);void this.refreshRecovery();this.changed();return true;
 }
 async refreshRecovery(){
  const id=this.identity;if(!id)return;const ticket=++this.recoveryTicket;this.recovering=true;this.recoveryError=false;this.changed();
  try{const raw=await this.storage.read(pending(id));if(ticket!==this.recoveryTicket)return;this.recovery=raw===null?null:parseStoredDraft(raw,pending(id));}
  catch(error){if(ticket===this.recoveryTicket)this.recoveryError=!(error&&typeof error==='object'&&'code'in error&&error.code==='unsupported');}
  finally{if(ticket===this.recoveryTicket){this.recovering=false;this.changed();}}
 }
 /** Review is identity/data only; never overwrite a current composer silently. */
 async reviewRecovery(expectedRevision:string){
  if(!this.identity||!this.recovery?.text||this.recovery.revision!==expectedRevision||this.target?.origin==='unbound')return false;
  const epoch=this.epoch,key=this.key,text=this.text;if(this.target)await this.resource.flush(this.target);
  const status=this.resource.status(this.target);
  if(epoch!==this.epoch||key!==this.key||text!==this.text||status?.error||status?.busy||status?.phase==='conflict')return false;
  this.target=pending(this.identity);this.epoch++;this.resource.activate(this.target,this.epoch);this.changed();return true;
 }
 async flush(){await this.resource.flushAll();}
 dispose(){this.recoveryTicket++;this.resource.dispose();}
}
