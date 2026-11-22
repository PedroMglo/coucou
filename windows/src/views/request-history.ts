import {h} from './dom';
import {requestHistoryFor,containsPendingRequest,type RequestRecord,type RequestPlacement} from '../core/request-history';
import type {SessionTarget} from '../core/session-chat';
const phases={pending:'Pedido registado',done:'Pedido concluído',stopped:'Pedido interrompido',failed:'O pedido não terminou',interrupted:'Execução interrompida'};
/** History is factual view data, distinct from model messages and live decisions. */
export function buildRequestHistory(){
 let rows:RequestPlacement[]=[],digest='',revision=0,previousRecords:readonly RequestRecord[]|null|undefined,previousTarget='',previousCount=-1;const nodes=new Map<string,{signature:string;el:HTMLElement}>();
 const legacy=h('section',{class:'request-history-unplaced','aria-label':'Pedidos com posição desconhecida'});
 const heading=h('p',{text:'Histórico recuperado · posição relativa às mensagens desconhecida'});
 function card(value:RequestPlacement){
  const r=value.record,key=r.taskId,signature=JSON.stringify(value);let cached=nodes.get(key);
  if(cached?.signature===signature)return cached.el;
  const title=h('strong',{text:phases[r.phase]});
  const origin=h('span',{class:'request-history-provenance',text:r.source==='taskJournal'?'Registo recuperado · o texto antigo pode estar incompleto':value.index===null?'Posição no histórico por confirmar':'Pedido registado nesta conversa'});
  const query=h('pre',{class:'task-notice-query request-history-query',text:r.query});
  const el=h('section',{class:'task-notice request-history-record','aria-label':phases[r.phase]},title,origin);
  el.dataset.taskId=r.taskId;el.dataset.phase=r.phase;
  if(r.messagesAdded===2&&value.index!==null)el.append(h('details',{},h('summary',{text:'Consultar pedido original'}),query));
  else el.append(h('span',{text:'Pedido original'}),query);
  cached={signature,el};nodes.set(key,cached);return el;
 }
 return {
  sync(target:SessionTarget|null,records:readonly RequestRecord[]|null|undefined,messageCount:number){
   const targetKey=JSON.stringify(target);
   if(records===previousRecords&&targetKey===previousTarget&&messageCount===previousCount)return false;
   previousRecords=records;previousTarget=targetKey;previousCount=messageCount;
   rows=requestHistoryFor(target,records,messageCount);const next=JSON.stringify([target,rows]),changed=next!==digest;digest=next;if(changed)revision++;
   const wanted=new Set(rows.map(v=>v.record.taskId));for(const key of nodes.keys())if(!wanted.has(key))nodes.delete(key);
   return changed;
  },
  get signature(){return revision;},get hasEntries(){return rows.length>0;},
  coversTask(taskId:string|undefined){return !!taskId&&rows.some(v=>v.record.taskId===taskId);},
  coversPending(query:string){return containsPendingRequest(rows,query);},
  before(index:number){return rows.filter(v=>v.index===index).map(card);},
  unplaced(){const unplaced=rows.filter(v=>v.index===null);if(!unplaced.length)return [];
   const desired=[heading,...unplaced.map(card)],retained=new Set(desired);for(const node of Array.from(legacy.children))if(!retained.has(node as HTMLElement))node.remove();
   desired.forEach((node,i)=>{if(legacy.children[i]!==node)legacy.insertBefore(node,legacy.children[i]??null);});return [legacy];
  },
 };
}
