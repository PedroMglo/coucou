export interface ApprovalContext {
 readonly requestId:string;readonly sessionId:string;readonly petId:string;
 readonly view:'approval'|'question';
}
interface Candidate{requestId:string;sessionId:string;pillId:string;presentationReady?:boolean}
export const approvalContext=(request:Candidate,view:ApprovalContext['view']):ApprovalContext=>Object.freeze({requestId:request.requestId,sessionId:request.sessionId,petId:request.pillId,view});
export const sameApproval=(context:ApprovalContext,request:Candidate|null|undefined)=>!!request&&!!context.requestId&&request.requestId===context.requestId&&request.sessionId===context.sessionId&&request.pillId===context.petId;
export const canAnswerApproval=(context:ApprovalContext,request:Candidate|null|undefined,visible:boolean)=>sameApproval(context,request)&&request?.presentationReady===true&&visible;
interface Host {
 current(context:ApprovalContext):boolean;
 visible(context:ApprovalContext):boolean;
 prepare(context:ApprovalContext):Promise<boolean>;
 acknowledge(context:ApprovalContext):Promise<boolean>;
 ready(context:ApprovalContext):void;
 handoff(context:ApprovalContext):Promise<void>|void;
}
/** ACK is presentation metadata, never a permission decision. Each await is
 * fenced by the initiating immutable request; a late A never acts on B. */
export async function presentApproval(context:ApprovalContext,host:Host):Promise<boolean>{
 let shown=false;
 try{
  if(host.current(context)&&await host.prepare(context)&&host.current(context)&&host.visible(context)){
   const accepted=await host.acknowledge(context);
   if(accepted===true&&host.current(context)&&host.visible(context)){host.ready(context);shown=true;}
  }
 }catch{ /* A failed presentation hands off instead of extending a blind wait. */ }
 if(!shown)await host.handoff(context);
 return shown;
}
