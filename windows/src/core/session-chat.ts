import type {PetAttachment,NativeTargetCapabilities} from './bridge';
/** A conversation surface is pinned to an explicit origin, never the last pet
 * or terminal that happened to emit an event. Late reads and sends cannot move it. */
export interface SessionTarget {
  readonly petId:string;
  readonly provider:string;
  readonly origin:'mochi'|'external';
  readonly sessionId:string;
  readonly conversationId?:string;
}
export interface SessionCapabilities {history:boolean;send:boolean;cancel:boolean}
export interface SessionMessage {id:string;role:'user'|'assistant';content:string}
export interface SessionAttachment {name:string;path:string;kind?:'file'|'folder';provider?:string;conversationId?:string;sequence?:number}
export interface SessionSnapshot {
  target:SessionTarget;messages:SessionMessage[];busy:boolean;turn?:string;
  capabilities:SessionCapabilities;notice?:string;taskNotice?:import('./task-notice').TaskNotice;requestHistory?:import('./request-history').RequestRecord[];model?:string;
  attachmentHistory?:import('./bridge').AttachmentTurnRecord[];
  attachments?:PetAttachment[];
  attachment?:SessionAttachment|null;
  localExecution?:import('./local-capabilities').LocalExecutionCapability;
  capabilityDescriptor?:NativeTargetCapabilities;
}
export interface SessionSummary {
  target:SessionTarget;title:string;cwd:string;updatedAt:number;state:string;
  model?:string;capabilities:SessionCapabilities;
}
export interface SessionTransport {
  history(target:SessionTarget):Promise<SessionSnapshot>;
  send(target:SessionTarget,query:string,model?:string,attachments?:PetAttachment[]):Promise<unknown>;
  cancel(target:SessionTarget,turn:string):Promise<boolean>;
  removeAttachment?(target:SessionTarget,attachment:PetAttachment):Promise<unknown>;
  clearAttachment?(target:SessionTarget,attachment:SessionAttachment):Promise<unknown>;
}
export const sessionTargetKey=(t:SessionTarget)=>JSON.stringify([t.petId,t.provider,t.origin,t.sessionId,t.conversationId??'']);
export const sameSession=(a:SessionTarget|null|undefined,b:SessionTarget|null|undefined)=>!!a&&!!b&&sessionTargetKey(a)===sessionTargetKey(b);
export const sessionOrigin=(t:SessionTarget)=>`${t.origin==='mochi'?'Mochi':'Terminal'} #${t.sessionId.slice(0,8)}`;
const message=(e:unknown)=>String(e).replace(/^Error:\s*/,'');

/** Metadata can only narrow the controls supplied by the native snapshot.
 * The backend still revalidates the actual operation and its original target. */
export function nativeSessionReady(snapshot:SessionSnapshot,id:'session.history'|'session.send'|'session.interrupt',now=Date.now()):boolean {
  const d=snapshot.capabilityDescriptor;
  if(!d)return true; // Older transports and Mochi retain their existing checks.
  if(snapshot.target.origin!=='external'||d.schemaVersion!==1||!sameSession(d.target,snapshot.target)||
    typeof d.revision!=='string'||!d.revision||d.revision.length>512||d.busy!==snapshot.busy||
    (d.turn??undefined)!==snapshot.turn||(d.model??undefined)!==snapshot.model)return false;
  const at=d.observedAt,expires=d.expiresAt;
  if(!Number.isSafeInteger(at)||!Number.isSafeInteger(expires)||at===null||expires===null||at<=0||
    expires<=at||expires-at>15_000||!Number.isSafeInteger(now)||now<at||now>=expires||
    !Array.isArray(d.capabilities)||d.capabilities.length>5)return false;
  const matches=d.capabilities.filter(c=>c.id===id);
  return matches.length===1&&matches[0].support==='supported'&&matches[0].availability==='ready'&&
    matches[0].observedAt===at&&matches[0].expiresAt===expires;
}
function constrainNative(snapshot:SessionSnapshot,invalidated=false):SessionSnapshot {
  if(!snapshot.capabilityDescriptor&&!invalidated)return snapshot;
  const c=snapshot.capabilities;
  return {...snapshot,capabilities:{history:c.history&&!invalidated&&nativeSessionReady(snapshot,'session.history'),
    send:c.send&&!invalidated&&nativeSessionReady(snapshot,'session.send'),cancel:c.cancel&&!invalidated&&nativeSessionReady(snapshot,'session.interrupt')}};
}

export class SessionChat {
  target:SessionTarget|null=null;
  snapshot:SessionSnapshot|null=null;
  loading=false;
  error='';
  private reads=0;
  private readKey='';
  private again=false;
  private evidenceEpoch=0;
  private pending=new Map<string,{query:string;messageIds:Set<string>;acknowledged:boolean}>();
  readonly drafts:Map<string,string>;
  readonly models=new Map<string,string>();
  readonly failures=new Map<string,{query:string;message:string}>();
  private transport:SessionTransport;
  private changed:()=>void;
  constructor(transport:SessionTransport,changed:()=>void,drafts:Map<string,string>=new Map()){this.transport=transport;this.changed=changed;this.drafts=drafts;}
  get sending(){return !!this.target&&this.pending.has(sessionTargetKey(this.target));}
  get query(){const pending=this.target&&this.pending.get(sessionTargetKey(this.target));return pending&&!pending.acknowledged?pending.query:'';}
  get model(){return this.target?this.models.get(sessionTargetKey(this.target)):undefined;}
  get failure(){return this.target?this.failures.get(sessionTargetKey(this.target)):undefined;}
  chooseModel(model:string){if(this.target)this.models.set(sessionTargetKey(this.target),model);this.changed();}
  async select(target:SessionTarget){
    this.target=Object.freeze({...target});this.snapshot=null;this.error='';this.reads++;this.loading=false;this.again=false;
    await this.refresh();
  }
  clear(){this.reads++;this.target=null;this.snapshot=null;this.loading=false;this.again=false;this.error='';this.changed();}
  invalidate(target?:SessionTarget){
    if(!this.target||target&&!sameSession(target,this.target)||this.target.origin!=='external')return;
    this.evidenceEpoch++;
    if(this.snapshot)this.snapshot=constrainNative(this.snapshot,true);
    if(this.loading)this.again=true;
    this.changed();
  }
  async refresh(){
    const target=this.target;if(!target)return;
    const key=sessionTargetKey(target);
    if(this.loading&&this.readKey===key){this.again=true;return;}
    const ticket=++this.reads,epoch=this.evidenceEpoch;this.readKey=key;this.loading=true;this.changed();
    try{
      const snapshot=await this.transport.history(target);
      if(ticket!==this.reads||!sameSession(target,this.target))return;
      if(!sameSession(target,snapshot.target))throw new Error('O histórico recebido pertence a outra sessão.');
      const pending=this.pending.get(key);
      if(pending&&snapshot.messages.some(m=>m.role==='user'&&m.content===pending.query&&!pending.messageIds.has(m.id)))pending.acknowledged=true;
      this.snapshot=constrainNative(snapshot,target.origin==='external'&&epoch!==this.evidenceEpoch);this.error='';
    }catch(e){if(ticket===this.reads&&sameSession(target,this.target))this.error=message(e);}
    finally{if(ticket===this.reads){this.loading=false;this.changed();if(this.again){this.again=false;void this.refresh();}}}
  }
  async send(text:string,expectedModel?:string):Promise<boolean>{
    const target=this.target,snapshot=this.snapshot,query=text.trim();
    if(!target||!snapshot||!query||this.error||snapshot.busy||!snapshot.capabilities.send||!nativeSessionReady(snapshot,'session.send')||this.sending)return false;
    const attachments=(snapshot.attachments??[]).map(a=>({...a}));
    const key=sessionTargetKey(target),model=expectedModel??this.models.get(key);this.failures.delete(key);this.pending.set(key,{query,messageIds:new Set(snapshot.messages.map(m=>m.id)),acknowledged:false});this.changed();
    this.invalidate(target);
    let ok=false,error='';
    try{await this.transport.send(target,query,model,attachments);ok=true;}
    catch(e){error=message(e);if(sameSession(target,this.target))this.error=error;}
    finally{
      if(sameSession(target,this.target)){
        await this.refresh();if(!ok&&sameSession(target,this.target))this.error=error;
      }
      const acknowledged=this.pending.get(key)?.acknowledged;
      this.pending.delete(key);
      if(!ok&&!acknowledged)this.failures.set(key,{query,message:error});
      if(sameSession(target,this.target))this.changed();
    }
    return ok;
  }
  async removeAttachment(id:string){
    const target=this.target,attachment=this.snapshot?.attachments?.find(a=>a.id===id);if(!target||!attachment||!this.transport.removeAttachment)return;
    try{await this.transport.removeAttachment(target,{...attachment});}
    catch(e){if(sameSession(target,this.target)){this.error=message(e);this.changed();}}
    finally{if(sameSession(target,this.target))await this.refresh();}
  }
  async clearAttachment(){
    const target=this.target,attachment=this.snapshot?.attachment;if(!target||!attachment||!this.transport.clearAttachment)return;
    try{await this.transport.clearAttachment(target,attachment);if(sameSession(target,this.target))await this.refresh();}
    catch(e){if(sameSession(target,this.target)){this.error=message(e);this.changed();}}
  }
  async cancel(){
    const target=this.target,snapshot=this.snapshot;
    if(!target||!snapshot?.busy||!snapshot.turn||!snapshot.capabilities.cancel||!nativeSessionReady(snapshot,'session.interrupt'))return;
    const turn=snapshot.turn;
    this.invalidate(target);
    try{
      const confirmed=await this.transport.cancel(target,turn);
      if(!sameSession(target,this.target))return;
      await this.refresh();
      if(sameSession(target,this.target)&&confirmed!==true&&this.snapshot?.busy&&this.snapshot.turn===turn){this.error='Não foi possível confirmar a interrupção deste turno. Atualiza a conversa antes de tentar novamente.';this.changed();}
    }
    catch(e){if(sameSession(target,this.target)){this.error=message(e);this.changed();}}
  }
}
