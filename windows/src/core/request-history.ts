import type {SessionTarget} from './session-chat';
export interface RequestRecord {
 readonly taskId:string;readonly conversationId:string;readonly provider:string;readonly petId:string;
 readonly sequence:number;readonly query:string;readonly phase:'pending'|'done'|'stopped'|'failed'|'interrupted';
 readonly started:number;readonly updated:number;readonly source:'captured'|'taskJournal';
 readonly messageIndex?:number;readonly messagesAdded?:0|2;
}
export interface RequestPlacement {record:RequestRecord;index:number|null}
/** Public view metadata grants no execution authority and never changes a target. */
export function requestHistoryFor(target:SessionTarget|null,records:readonly RequestRecord[]|null|undefined,messageCount:number):RequestPlacement[]{
 if(!target||target.origin!=='mochi'||!target.sessionId||target.conversationId!==target.sessionId||!Array.isArray(records))return [];
 const found=new Map<string,RequestRecord>();
 for(const r of records){
  if(!r||r.petId!==target.petId||r.provider!==target.provider||r.conversationId!==target.sessionId||typeof r.taskId!=='string'||!r.taskId||typeof r.query!=='string'||!Number.isSafeInteger(r.sequence)||r.sequence<1||!Number.isSafeInteger(r.started)||r.started<0||!Number.isSafeInteger(r.updated)||r.updated<0||!['pending','done','stopped','failed','interrupted'].includes(r.phase)||!['captured','taskJournal'].includes(r.source))continue;
  const before=found.get(r.taskId);if(!before||r.updated>before.updated)found.set(r.taskId,r);
 }
 return [...found.values()].sort((a,b)=>a.sequence-b.sequence||a.taskId.localeCompare(b.taskId)).map(record=>{
  const i=record.messageIndex,n=record.messagesAdded;
  const anchored=record.source==='captured'&&Number.isSafeInteger(i)&&i!>=0&&(n===0||n===2)&&i!+n<=messageCount;
  return {record,index:anchored?i!:null};
 });
}
export function containsPendingRequest(rows:readonly RequestPlacement[],query:string){return !!query&&rows.some(({record:r})=>r.phase==='pending'&&r.query===query);}
