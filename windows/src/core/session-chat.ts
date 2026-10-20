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
  attachment?:SessionAttachment|null;
  localExecution?:import('./local-capabilities').LocalExecutionCapability;
}
export interface SessionSummary {
  target:SessionTarget;title:string;cwd:string;updatedAt:number;state:string;
  model?:string;capabilities:SessionCapabilities;
}
export interface SessionTransport {
  history(target:SessionTarget):Promise<SessionSnapshot>;
  send(target:SessionTarget,query:string,model?:string,attachment?:SessionAttachment|null):Promise<unknown>;
  cancel(target:SessionTarget,turn:string):Promise<boolean>;
  clearAttachment?(target:SessionTarget,attachment:SessionAttachment):Promise<unknown>;
}
export const sessionTargetKey=(t:SessionTarget)=>JSON.stringify([t.petId,t.provider,t.origin,t.sessionId,t.conversationId??'']);
export const sameSession=(a:SessionTarget|null|undefined,b:SessionTarget|null|undefined)=>!!a&&!!b&&sessionTargetKey(a)===sessionTargetKey(b);
export const sessionOrigin=(t:SessionTarget)=>`${t.origin==='mochi'?'Mochi':'Terminal'} #${t.sessionId.slice(0,8)}`;
const message=(e:unknown)=>String(e).replace(/^Error:\s*/,'');

export class SessionChat {
  target:SessionTarget|null=null;
  snapshot:SessionSnapshot|null=null;
  loading=false;
  error='';
  private reads=0;
  private readKey='';
  private again=false;
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
  async refresh(){
    const target=this.target;if(!target)return;
    const key=sessionTargetKey(target);
    if(this.loading&&this.readKey===key){this.again=true;return;}
    const ticket=++this.reads;this.readKey=key;this.loading=true;this.changed();
    try{
      const snapshot=await this.transport.history(target);
      if(ticket!==this.reads||!sameSession(target,this.target))return;
      if(!sameSession(target,snapshot.target))throw new Error('O histórico recebido pertence a outra sessão.');
      const pending=this.pending.get(key);
      if(pending&&snapshot.messages.some(m=>m.role==='user'&&m.content===pending.query&&!pending.messageIds.has(m.id)))pending.acknowledged=true;
      this.snapshot=snapshot;this.error='';
    }catch(e){if(ticket===this.reads&&sameSession(target,this.target))this.error=message(e);}
    finally{if(ticket===this.reads){this.loading=false;this.changed();if(this.again){this.again=false;void this.refresh();}}}
  }
  async send(text:string,expectedModel?:string):Promise<boolean>{
    const target=this.target,snapshot=this.snapshot,query=text.trim();
    if(!target||!snapshot||!query||this.error||snapshot.busy||!snapshot.capabilities.send||this.sending)return false;
    const key=sessionTargetKey(target),model=expectedModel??this.models.get(key);this.failures.delete(key);this.pending.set(key,{query,messageIds:new Set(snapshot.messages.map(m=>m.id)),acknowledged:false});this.changed();
    let ok=false,error='';
    try{await this.transport.send(target,query,model,snapshot.attachment);ok=true;}
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
  async clearAttachment(){
    const target=this.target,attachment=this.snapshot?.attachment;if(!target||!attachment||!this.transport.clearAttachment)return;
    try{await this.transport.clearAttachment(target,attachment);if(sameSession(target,this.target))await this.refresh();}
    catch(e){if(sameSession(target,this.target)){this.error=message(e);this.changed();}}
  }
  async cancel(){
    const target=this.target,snapshot=this.snapshot;
    if(!target||!snapshot?.busy||!snapshot.turn||!snapshot.capabilities.cancel)return;
    const turn=snapshot.turn;
    try{
      const confirmed=await this.transport.cancel(target,turn);
      if(!sameSession(target,this.target))return;
      await this.refresh();
      if(sameSession(target,this.target)&&confirmed!==true&&this.snapshot?.busy&&this.snapshot.turn===turn){this.error='Não foi possível confirmar a interrupção deste turno. Atualiza a conversa antes de tentar novamente.';this.changed();}
    }
    catch(e){if(sameSession(target,this.target)){this.error=message(e);this.changed();}}
  }
}
