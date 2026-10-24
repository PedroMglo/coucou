import {Bridge,onEvent,type AgentRequest,type DecisionTarget} from '../core/bridge';
import {sameSession,type SessionTarget} from '../core/session-chat';
import {h} from './dom';
import {commandReview} from './command-review';
import {approvalActions} from './approval-actions';
import {approvalQuestions} from './approval-questions';

/** Public native decisions are keyed by request ID and immutable session. A
 * second surface can observe them; only one human answer is accepted by Rust. */
interface RequestTransport{pending:()=>Promise<AgentRequest[]>;answer:(id:string,answer:Record<string,unknown>,target?:DecisionTarget)=>Promise<void>;visible?:(request:AgentRequest)=>boolean;accept?:(request:AgentRequest)=>boolean}
export function buildAgentRequests(selected:()=>SessionTarget|null,changed:()=>void,transport:RequestTransport={pending:()=>Bridge.cliAgentPending(),answer:(id,answer,target)=>Bridge.cliAgentAnswer(id,answer,target)}){
 let alive=true;const unsubscribers:Array<()=>void>=[];
 const observe=(event:string,handler:(value:any)=>void)=>void onEvent(event,handler).then(fn=>{if(alive)unsubscribers.push(fn);else fn();});
 const el=h('div',{class:'cli-requests',style:'max-height:230px;overflow:auto'});
 const cards=new Map<string,{el:HTMLElement;request:AgentRequest}>();
 const ended=new Set<string>();
 function sync(){for(const card of cards.values()){const goal=card.request.goalTarget;card.el.hidden=transport.visible?!transport.visible(card.request):goal?selected()?.petId!==goal.petId:!sameSession(card.request.target,selected());}}
 function add(request:AgentRequest){
  if(!alive||transport.accept&&!transport.accept(request)||cards.has(request.id)||ended.has(request.id)||(!request.target&&!request.goalTarget))return;
  const origin=h('p',{class:'session-origin',text:request.origin??request.target?.sessionId??request.goalTarget?.goalId??''});
  const card=h('div',{class:'cli-request',style:'padding:10px;border:1px solid #F5A524;border-radius:10px;margin:6px 0'},h('strong',{text:request.kind==='question'?'Precisa da tua resposta':'Precisa da tua aprovação'}),origin);
  if(request.goalTarget&&request.voiceCode)card.append(h('p',{text:`Voz: rever receita ${request.voiceCode} · aprovar receita ${request.voiceCode} · recusar receita ${request.voiceCode}`}));
  const questions=approvalQuestions(request.kind==='question'?request:{questions:[],detail:''});
  if(request.kind==='question')card.append(questions.el);
  else card.append(commandReview(request.detail,request.provider==='ollama'&&request.approvalScope==='once'));
  const error=h('p',{class:'chat-feedback',role:'status'});
  const actions=approvalActions(request,answer);
  function answer(yes:boolean,nativeChoice?:string){
   actions.setDisabled(true);questions.setDisabled(true);
   const answers=questions.answers(yes);
   void transport.answer(request.id,{allow:yes,answers,...(nativeChoice?{nativeChoice}:{})},request.goalTarget??request.target).then(()=>{card.remove();cards.delete(request.id);changed();})
    .catch(e=>{error.textContent=String(e).replace(/^Error:\s*/,'');actions.setDisabled(false);questions.setDisabled(false);changed();});
  }
  card.append(actions.el,error);cards.set(request.id,{el:card,request});el.append(card);sync();changed();
 }
 observe('agent-request',add);
 observe('agent-request-ended',({id}:{id:string})=>{ended.add(id);if(ended.size>256)ended.delete(ended.values().next().value!);cards.get(id)?.el.remove();cards.delete(id);changed();});
 const refresh=()=>transport.pending().then(requests=>requests.forEach(add)).catch(()=>{});void refresh();
 return {el,sync,refresh,dispose(){alive=false;for(const fn of unsubscribers)fn();unsubscribers.length=0;}};
}
