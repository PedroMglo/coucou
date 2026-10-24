/** Loaded current app conversations only. Never a provider inventory, native
 * terminal activity, goal state or proof that a model request is in flight. */
export interface PetSessionStateSummary {
 idle:number;reserved:number;busy:number;waiting:number;stopping:number;unknown:number;observedAt:number;
}
const record=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const text=(v:unknown):v is string=>typeof v==='string'&&v.length>0&&v.length<=512&&!/[\x00-\x1f\x7f]/.test(v);
export function petSessionStatesSummary(value:unknown,petId:string):PetSessionStateSummary|null {
 if(!record(value)||value.status!=='observed'||typeof value.observedAt!=='number'||!Number.isSafeInteger(value.observedAt)||value.observedAt<=0||value.observedAt>8.64e12||!record(value.sessionStates))return null;
 const data=value.sessionStates;if(data.schemaVersion!==1||data.scope!=='loadedPetConversations'||!Array.isArray(data.rows)||data.rows.length>4096)return null;
 const result:PetSessionStateSummary={idle:0,reserved:0,busy:0,waiting:0,stopping:0,unknown:0,observedAt:value.observedAt};
 const seen=new Map<string,string>();let matched=false;
 for(const row of data.rows){
  if(!record(row)||!['owner','petId','conversationId'].every(k=>text(row[k]))||!['idle','reserved','busy','waiting','stopping','unknown'].includes(String(row.state))||!(row.turn===null||typeof row.turn==='string'&&/^[1-9][0-9]{0,19}$/.test(row.turn))||row.state==='idle'&&row.turn!==null||['reserved','busy','waiting','stopping'].includes(String(row.state))&&row.turn===null||!(row.reason===null||typeof row.reason==='string'&&/^[a-zA-Z][a-zA-Z0-9]{0,63}$/.test(row.reason))||row.state==='unknown'&&row.reason===null||row.state!=='unknown'&&row.reason!==null)return null;
  const key=JSON.stringify([row.owner,row.petId]),signature=JSON.stringify([row.conversationId,row.turn,row.state,row.reason]);
  if(seen.has(key)){if(seen.get(key)!==signature)return null;continue;}seen.set(key,signature);
  if(row.petId!==petId)continue;matched=true;result[row.state as keyof Pick<PetSessionStateSummary,'idle'|'reserved'|'busy'|'waiting'|'stopping'|'unknown'>]++;
 }
 return matched?result:null;
}
export function petSessionStatesText(value:PetSessionStateSummary|null|undefined,stale=false):string|null {
 if(!value)return null;
 const labels=[['busy','em trabalho'],['waiting','à espera de aprovação ou resposta'],['reserved','reservada'],['stopping','a terminar'],['idle','livre'],['unknown','estado por confirmar']] as const;
 const states=labels.filter(([key])=>value[key]>0).map(([key,label])=>`${value[key]} ${label}`).join(' · ');
 return `Conversa carregada do pet: ${states}${stale?' · leitura anterior':''}`;
}
