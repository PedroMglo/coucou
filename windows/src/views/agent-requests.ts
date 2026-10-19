import {Bridge,onEvent,type AgentRequest} from '../core/bridge';
import {sameSession,type SessionTarget} from '../core/session-chat';
import {h} from './dom';

/** Public native decisions are keyed by request ID and immutable session. A
 * second surface can observe them; only one human answer is accepted by Rust. */
export function buildAgentRequests(selected:()=>SessionTarget|null,changed:()=>void){
 const el=h('div',{class:'cli-requests',style:'max-height:230px;overflow:auto'});
 const cards=new Map<string,{el:HTMLElement;request:AgentRequest}>();
 const ended=new Set<string>();
 function sync(){for(const card of cards.values())card.el.hidden=!sameSession(card.request.target,selected());}
 function add(request:AgentRequest){
  if(cards.has(request.id)||ended.has(request.id)||!request.target)return;
  const origin=h('p',{class:'session-origin',text:request.origin??request.target.sessionId});
  const card=h('div',{class:'cli-request',style:'padding:10px;border:1px solid #F5A524;border-radius:10px;margin:6px 0'},h('strong',{text:request.kind==='question'?'Precisa da tua resposta':'Precisa da tua aprovação'}),origin);
  const fields=new Map<string,HTMLInputElement>();
  if(request.kind==='question')for(const question of request.questions??[]){
   const field=h('input',{class:'chat-input',type:question.isSecret?'password':'text',autocomplete:'off','aria-label':question.question}) as HTMLInputElement;
   fields.set(question.id,field);card.append(h('label',{text:question.question}),field);
   for(const option of question.options??[])card.append(h('button',{text:option.label,onclick:()=>{field.value=option.label;}}));
  }else card.append(h('pre',{text:request.detail,style:'white-space:pre-wrap;max-height:120px;overflow:auto'}));
  const error=h('p',{class:'chat-feedback',role:'status'});
  const allow=h('button',{class:'cli-allow',text:request.kind==='question'?'Enviar':request.approvalScope==='turn'?'Permitir neste turno':'Permitir uma vez'}),deny=h('button',{class:'cli-deny',text:'Recusar'});
  function answer(yes:boolean){
   allow.disabled=deny.disabled=true;
   const answers=Object.fromEntries([...fields].map(([id,input])=>[id,yes?[input.value]:[]]));
   void Bridge.cliAgentAnswer(request.id,{allow:yes,answers},request.target).then(()=>{card.remove();cards.delete(request.id);changed();})
    .catch(e=>{error.textContent=String(e).replace(/^Error:\s*/,'');allow.disabled=deny.disabled=false;changed();});
  }
  allow.onclick=()=>answer(true);deny.onclick=()=>answer(false);card.append(allow,deny,error);cards.set(request.id,{el:card,request});el.append(card);sync();changed();
 }
 void onEvent<AgentRequest>('agent-request',add);
 void onEvent<{id:string}>('agent-request-ended',({id})=>{ended.add(id);if(ended.size>256)ended.delete(ended.values().next().value!);cards.get(id)?.el.remove();cards.delete(id);changed();});
 void Bridge.cliAgentPending().then(requests=>requests.forEach(add)).catch(()=>{});
 return {el,sync};
}
