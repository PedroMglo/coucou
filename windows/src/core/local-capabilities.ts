export interface LocalExecutionCapability {model:string;tools:boolean}
export interface LocalCapabilityEvent {provider:string;conversationId:string;turn:number;capability:LocalExecutionCapability}
/** Ephemeral metadata belongs to a verified model and an exact conversation. */
export class LocalCapabilities {
 private values=new Map<string,{turn:number;capability:LocalExecutionCapability|null}>();
 observe(id:string,turn:number,capability?:LocalExecutionCapability|null){
  if(!id||!Number.isSafeInteger(turn)||turn<0)return;const previous=this.values.get(id);
  if(previous&&(previous.turn>turn||previous.turn===turn&&!capability))return;
  this.values.set(id,{turn,capability:capability?{...capability}:null});if(this.values.size>128)this.values.delete(this.values.keys().next().value!);
 }
 label(id:string,selected:string,profiles:string[],ask:boolean){
  const cap=this.values.get(id)?.capability;if(!cap)return '';
  const profile=selected.startsWith('@');if(profile?!profiles.includes(cap.model):selected!==cap.model)return '';
  const action=cap.tools?(ask?'Ações por aprovação':'Ações desativadas'):'Só conversa';return profile?`Último modelo: ${cap.model} · ${action}`:`${cap.model} · ${action}`;
 }
}
