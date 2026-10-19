/** A pet asking for a decision must reopen that decision instead of its home.
 * Native chat decisions live in the conversation; hook decisions have cards. */
export function petOpenView(phase:string,requested?:string,chatDecision=false):string{
 if(requested&&requested!=='overview')return requested;
 if(chatDecision)return 'prompt';
 if(phase==='approval'||phase==='question')return phase;
 return requested??'overview';
}
