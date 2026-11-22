import type {SessionLookupRequest,SessionLookupResult,SessionLookupEvidence} from './session-lookup-contract';
import type {SessionTarget} from './session-chat';
import {observationScope,observationDate} from './pet-observability';

export const lookupKey=(request:SessionLookupRequest)=>JSON.stringify([request.provider,request.nativeSessionId]);
const providerId=(value:unknown):value is string=>typeof value==='string'&&/^[a-z0-9_-]{1,64}$/.test(value);
const nativeId=(value:unknown):value is string=>typeof value==='string'&&/^[A-Za-z0-9_.:-]{1,128}$/.test(value);
const uuid=(value:string)=>/^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$/.test(value);
const cid=(value:unknown):value is string=>typeof value==='string'&&/^[A-Za-z0-9-]{1,128}$/.test(value);
const petId=(value:unknown):value is string=>typeof value==='string'&&/^[a-z0-9_-]{1,128}$/.test(value);
const record=(value:unknown):value is Record<string,unknown>=>!!value&&typeof value==='object'&&!Array.isArray(value);
const stamp=(value:unknown):value is number=>typeof value==='number'&&Number.isSafeInteger(value)&&value>0&&value<=8.64e12;
const nullable=(value:unknown,predicate:(value:unknown)=>boolean)=>value===null||predicate(value);
const code=(value:unknown)=>typeof value==='string'&&value.length<=128&&!/[\u0000-\u001f\u007f]/.test(value);
export function validLookupRequest(value:SessionLookupRequest):boolean{
 return providerId(value?.provider)&&nativeId(value?.nativeSessionId)&&(!['codex','copilot'].includes(value.provider)||uuid(value.nativeSessionId));
}
function evidence(value:unknown):value is SessionLookupEvidence{
 return record(value)&&code(value.scope)&&['complete','partial','unavailable','notSupported'].includes(String(value.coverage))&&nullable(value.observedAt,stamp)&&typeof value.stale==='boolean'&&nullable(value.reason,code);
}
const incompatible=()=>new Error('Não foi possível confirmar a identificação desta sessão. Tenta atualizar a lista.');

/** Checks the echoed namespace and copies only navigation fields. Native
 * freshness is evidence, not an additional authority or a liveness claim. */
export function validatedLookup(value:unknown,request:SessionLookupRequest):{result:SessionLookupResult;target:SessionTarget|null}{
 if(!validLookupRequest(request)||!record(value)||value.schemaVersion!==1||!stamp(value.generatedAt)||!record(value.request)||
  value.request.provider!==request.provider||value.request.nativeSessionId!==request.nativeSessionId||
  !['resolved','ambiguous','unavailable','notFound','notSupported','invalidRequest'].includes(String(value.state))||
  !nullable(value.reason,code)||!nullable(value.owner,providerId)||!nullable(value.conversationId,cid)||
  !['mochi','external','unknown'].includes(String(value.origin))||!evidence(value.library)||!evidence(value.native))throw incompatible();
 const hint=value.retryAfterMs;
 if(hint!==null&&(!(typeof hint==='number'&&Number.isSafeInteger(hint)&&hint>=1&&hint<=10000)||value.state!=='unavailable'||!['nativeLookupBusy','nativeLookupThrottled'].includes(String(value.reason))))throw incompatible();
 let target:SessionTarget|null=null;
 if(value.state==='resolved'){
  if(!record(value.open)||!petId(value.open.petId)||value.library.scope!=='mochiLibrary'||value.library.coverage!=='complete'||value.library.stale)throw incompatible();
  const open=value.open;
  if(value.origin==='mochi'&&open.kind==='mochi'&&providerId(open.provider)&&cid(open.conversationId)&&open.provider===value.owner&&open.conversationId===value.conversationId){
   target=Object.freeze({petId:open.petId as string,provider:open.provider,origin:'mochi',sessionId:open.conversationId,conversationId:open.conversationId});
  }else if(value.origin==='external'&&open.kind==='external'&&open.provider===request.provider&&open.sessionId===request.nativeSessionId&&value.owner===null&&value.conversationId===null&&open.petId===({codex:'agent_codex',copilot:'agent_copilot'} as Record<string,string>)[request.provider]&&value.native.scope===({codex:'codexDaemon',copilot:'copilotLiveEndpoints'} as Record<string,string>)[request.provider]&&['complete','partial'].includes(value.native.coverage)){
   target=Object.freeze({petId:open.petId as string,provider:request.provider,origin:'external',sessionId:request.nativeSessionId});
  }else throw incompatible();
 }else if(value.open!==null)throw incompatible();
 if(value.state==='notFound'){
  const expected=({codex:'codexDaemon',copilot:'copilotLiveEndpoints'} as Record<string,string>)[request.provider];
  if(!expected||value.native.scope!==expected||value.native.coverage!=='complete'||value.native.stale||!stamp(value.native.observedAt)||value.generatedAt<value.native.observedAt||value.generatedAt-value.native.observedAt>15||value.library.scope!=='mochiLibrary'||value.library.coverage!=='complete'||value.library.stale)throw incompatible();
 }
 // Private retained metadata is a detached copy; no result fields are spread
 // into an execution, terminal or approval request.
 return {result:structuredClone(value) as unknown as SessionLookupResult,target};
}

export function lookupEvidence(result:SessionLookupResult):string{
 const source=result.origin==='mochi'?result.library:result.native;
 return `${source.scope==='mochiLibrary'?'Biblioteca do Coucou':observationScope(source.scope)} · ${observationDate(source.observedAt)}${source.stale?' · Dados desatualizados':''}${source.coverage==='partial'?' · Cobertura parcial':''}`;
}
export function lookupMessage(result:SessionLookupResult):string{
 if(result.state==='resolved')return result.origin==='mochi'?'Conversa do pet identificada.':'Sessão externa identificada. Isto não confirma um turno em curso.';
 if(result.state==='ambiguous')return 'Não foi possível identificar uma única conversa para esta sessão.';
 if(result.state==='notFound')return 'Esta sessão não foi encontrada na fonte consultada.';
 if(result.state==='notSupported')return 'Este provider não disponibiliza esta consulta.';
 if(result.state==='invalidRequest')return 'A identificação desta sessão não permite abrir a conversa.';
 return ({petDisabled:'O pet responsável está desativado. Ativa-o para abrir esta conversa.',petBindingUnavailable:'Esta conversa pertence a um pet que já não está disponível.',libraryOwnershipIncomplete:'As identidades das conversas estão indisponíveis. Tenta novamente.',nativeLookupBusy:'Já está a ser consultado este provider. Tenta novamente mais tarde.',nativeLookupThrottled:'Este provider foi consultado há pouco. Aguarda antes de repetir.',nativeInventoryTimeout:'A consulta desta sessão demorou demasiado. Tenta novamente.'} as Record<string,string>)[result.reason??'']??'Não foi possível confirmar a ligação desta sessão ao chat. Tenta novamente.';
}

export interface LookupNavigationTransport{
 lookup(request:SessionLookupRequest):Promise<unknown>;
 open(target:SessionTarget):Promise<boolean>;
}
export interface LookupNavigationState{
 phase:'locating'|'opening'|'result';opened?:boolean;message:string;result?:SessionLookupResult;retryAt:number|null;
}
/** One explicit user intent. Local invalidation can prevent an obsolete lookup
 * from dispatching open; it cannot undo an open already accepted natively. */
export class SessionLookupNavigation{
 private epoch=0;private ticket=0;private alive=true;
 private pending=new Map<string,symbol>();
 private states=new Map<string,LookupNavigationState>();
 constructor(private transport:LookupNavigationTransport,private options:{
  current(request:SessionLookupRequest):boolean;
  unavailable(target:SessionTarget):string;
  changed():void;
  presented():void;
  clock?:()=>number;
 }){}
 private now(){return this.options.clock?.()??performance.now();}
 state(request:SessionLookupRequest){return this.states.get(lookupKey(request));}
 isPending(request:SessionLookupRequest){return this.pending.has(lookupKey(request));}
 retryWait(request:SessionLookupRequest){return Math.max(0,Math.ceil(((this.state(request)?.retryAt??0)-this.now())/1000));}
 get hasRetryWait(){return [...this.states.values()].some(s=>s.retryAt!==null&&s.retryAt>this.now());}
 invalidate(){this.epoch++;this.ticket++;for(const key of this.states.keys())if(!this.pending.has(key))this.states.delete(key);this.options.changed();}
 dispose(){this.alive=false;this.invalidate();}
 private publish(key:string,state:LookupNavigationState){
  this.states.set(key,state);
  // Retain only small results for explicitly clicked rows, never an inventory.
  if(this.states.size>64)for(const old of this.states.keys()){if(old!==key&&!this.pending.has(old))this.states.delete(old);if(this.states.size<=64)break;}
  this.options.changed();
 }
 async open(input:SessionLookupRequest):Promise<void>{
  if(!this.alive||!validLookupRequest(input)||this.isPending(input)||this.retryWait(input)>0||!this.options.current(input))return;
  const request=Object.freeze({provider:input.provider,nativeSessionId:input.nativeSessionId}),key=lookupKey(request),identity=Symbol(),epoch=this.epoch,ticket=++this.ticket;
  const current=()=>this.alive&&epoch===this.epoch&&ticket===this.ticket&&this.options.current(request);
  this.pending.set(key,identity);this.publish(key,{phase:'locating',message:'A localizar a conversa…',retryAt:null});
  try{
   if(!current())return;
   let reply:unknown;
   try{reply=await this.transport.lookup(request);}catch{if(current())this.publish(key,{phase:'result',message:'Não foi possível localizar esta sessão. Tenta novamente.',retryAt:null});return;}
   if(!current())return;
   let checked:ReturnType<typeof validatedLookup>;
   try{checked=validatedLookup(reply,request);}catch{this.publish(key,{phase:'result',message:incompatible().message,retryAt:null});return;}
   const {result,target}=checked;
   if(!target){this.publish(key,{phase:'result',result,message:lookupMessage(result),retryAt:result.retryAfterMs===null?null:this.now()+result.retryAfterMs});return;}
   const unavailable=this.options.unavailable(target);
   if(unavailable){this.publish(key,{phase:'result',result,message:unavailable,retryAt:null});return;}
   // No await between the final intent check and dispatch.
   if(!current())return;
   this.publish(key,{phase:'opening',result,message:'A abrir a conversa…',retryAt:null});
   if(!current())return;
   try{
    const shown=await this.transport.open(target);
    if(current()){
     this.publish(key,{phase:'result',opened:shown===true,result,message:shown===true?lookupMessage(result):'A conversa não abriu. Tenta novamente.',retryAt:null});
     if(shown===true&&!this.options.unavailable(target)&&current())this.options.presented();
    }
   }catch{if(current())this.publish(key,{phase:'result',result,message:'Não foi possível abrir a conversa. Tenta novamente.',retryAt:null});}
  }finally{
   if(this.pending.get(key)===identity){this.pending.delete(key);if(!current())this.states.delete(key);this.options.changed();}
  }
 }
}
