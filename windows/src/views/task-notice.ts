import {h} from './dom';
import type {SessionTarget} from '../core/session-chat';
import {taskNoticeFor,type TaskNotice} from '../core/task-notice';
interface Snapshot{busy:boolean;notice?:string;taskNotice?:TaskNotice}
/** The interrupted request stays visibly distinct from assistant messages. */
export function buildTaskNotice(){
 const title=h('strong',{role:'status','aria-live':'polite'}),message=h('p');
 const label=h('span',{text:'Pedido original'}),query=h('pre',{class:'task-notice-query'});
 const el=h('section',{class:'task-notice','aria-label':'Estado do pedido'},title,message,label,query);el.hidden=true;
 return {el,sync(target:SessionTarget|null,snapshot:Snapshot|null,pending=false){
  const value=taskNoticeFor(target,snapshot?.taskNotice,pending||snapshot?.busy);
  el.hidden=!value;el.dataset.taskId=value?.taskId??'';
  title.textContent=value?({stopped:'Pedido interrompido',interrupted:'Execução interrompida',failed:'O pedido não terminou'}[value.phase]):'';
  query.textContent=value?.query??'';label.hidden=query.hidden=!query.textContent;
  message.textContent=value&&!snapshot?.notice?value.message:'';message.hidden=!message.textContent;
 }};
}
