import type {SessionTarget} from './session-chat';
export interface TaskNotice {
 readonly taskId:string;readonly conversationId:string;readonly provider:string;
 readonly phase:'stopped'|'interrupted'|'failed';readonly query:string;readonly message:string;
}
/** Display metadata alone cannot authorize execution, retries or cancellation. */
export function taskNoticeFor(target:SessionTarget|null,notice:TaskNotice|null|undefined,busy=false):TaskNotice|undefined {
 if(busy||!target||target.origin!=='mochi'||!target.sessionId||target.conversationId!==target.sessionId||!notice?.taskId)return;
 if(notice.provider!==target.provider||notice.conversationId!==target.sessionId||!['stopped','interrupted','failed'].includes(notice.phase))return;
 return notice;
}
