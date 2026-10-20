import type {ProcessCensus} from './process-census-contract';
import {observationDate} from './pet-observability';
export interface CensusPresentation {
 provider:string;coverage:ProcessCensus['coverage'];observedAt:number|null;stale:boolean;refreshing:boolean;
 counts:ProcessCensus['counts'];reason:string|null;
}
const record=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const number=(v:unknown):v is number=>typeof v==='number'&&Number.isSafeInteger(v)&&v>=0;
const stamp=(v:unknown):v is number=>number(v)&&v>0&&v<=8.64e12;
/** Display only aggregate verified-client counts. PIDs, CWD, source environment
 * and native identity strings are neither rendered nor promoted to authority. */
export function parseProcessCensus(value:unknown,provider:string):CensusPresentation|null {
 if(!record(value)||value.schemaVersion!==1||value.provider!==provider||value.scope!=='localProcesses'||!['completeWithinUserProcessScan','partial','unavailable','notSupported'].includes(String(value.coverage))||!(value.observedAt===null||stamp(value.observedAt))||typeof value.stale!=='boolean'||typeof value.refreshing!=='boolean'||value.turns!=='notObservedByProcessCensus'||!record(value.counts))return null;
 const keys=['clientsObserved','terminalClientsObserved','foregroundClientsObserved','identifiedNativeSessionsObserved','clientsWithUnknownSession','clientsWithConflictingSession'] as const;
 const raw=value.counts;if(!keys.every(key=>number(raw[key])))return null;
 const c=value.counts as unknown as ProcessCensus['counts'];
 if(!(c.distinctNativeSessions===null||number(c.distinctNativeSessions))||keys.some(key=>c[key]>c.clientsObserved)||c.foregroundClientsObserved>c.terminalClientsObserved||c.clientsWithUnknownSession+c.clientsWithConflictingSession>c.clientsObserved||(c.clientsWithUnknownSession+c.clientsWithConflictingSession>0&&c.distinctNativeSessions!==null)||c.distinctNativeSessions!==null&&c.distinctNativeSessions>c.clientsObserved)return null;
 return {provider,coverage:value.coverage as ProcessCensus['coverage'],observedAt:value.observedAt as number|null,stale:value.stale||value.observedAt===null,refreshing:value.refreshing,counts:Object.fromEntries([...keys.map(key=>[key,c[key]]),['distinctNativeSessions',c.distinctNativeSessions]]) as unknown as ProcessCensus['counts'],reason:typeof value.reason==='string'&&/^[a-zA-Z][a-zA-Z0-9]{0,63}$/.test(value.reason)?value.reason:null};
}
/** Four verified interactive clients are useful even without a native UUID.
 * They are labelled clients/terminals, never four unique working sessions. */
export function processCensusSummary(value:CensusPresentation|null):{count:string;caption:string;summary:string;detail:string}|null {
 if(!value||!['completeWithinUserProcessScan','partial'].includes(value.coverage)||value.observedAt===null)return null;
 const c=value.counts,number=(n:number)=>n.toLocaleString('pt-PT'),parts=[`${number(c.terminalClientsObserved)} clientes CLI com terminal observados${value.observedAt?` em ${observationDate(value.observedAt)}`:''}.`];
 if(c.clientsObserved>c.terminalClientsObserved)parts.push(`${number(c.clientsObserved-c.terminalClientsObserved)} outros clientes sem terminal.`);
 if(c.distinctNativeSessions!==null)parts.push(`${number(c.distinctNativeSessions)} identidades de sessão distintas nestes clientes.`);
 else parts.push('O número de sessões distintas ainda está por confirmar.');
 if(c.clientsWithUnknownSession)parts.push(`${number(c.clientsWithUnknownSession)} clientes sem identidade de sessão confirmada.`);
 if(c.clientsWithConflictingSession)parts.push(`${number(c.clientsWithConflictingSession)} clientes com identidade em conflito.`);
 if(value.stale)parts.push('Leitura anterior; pode já ter mudado.');
 parts.push('Não inclui conversas guardadas, não conta os pets e não mede turnos em execução.');
 const provider=({codex:'Codex',copilot:'Copilot',antigravity:'Antigravity',ollama:'Ollama'} as Record<string,string>)[value.provider]??value.provider;
 return {count:number(c.terminalClientsObserved),caption:`CLI ${provider}${value.coverage==='partial'?' · parcial':''}${value.stale?' · leitura anterior':''}`,summary:c.distinctNativeSessions===null?'Clientes observados deste provider. Sessões distintas e turnos no terminal por confirmar.':`${number(c.distinctNativeSessions)} sessões distintas identificadas nestes clientes. Turnos no terminal por confirmar.`,detail:`Provider ${provider} · `+parts.join(' ')};
}
export interface LiveWorkSummary {active:number;waiting:number;reserved:number;observedAt:number}
/** Exact pet ownership; no addition to a potentially aliased terminal session. */
export function livePetWorkSummary(value:unknown,petId:string):LiveWorkSummary|null {
 if(!record(value)||value.status!=='observed'||!stamp(value.observedAt)||!Array.isArray(value.rows)||value.rows.length>4096)return null;
 const result:LiveWorkSummary={active:0,waiting:0,reserved:0,observedAt:value.observedAt},seen=new Set<string>();
 const text=(v:unknown):v is string=>typeof v==='string'&&v.length>0&&v.length<=512&&!/[\x00-\x1f\x7f]/.test(v);
 for(const row of value.rows){
  if(!record(row)||!['owner','petId','conversationId','turn'].every(key=>text(row[key]))||!['reserved','busy','waiting'].includes(String(row.phase)))return null;
  if(row.petId!==petId)continue;
  const key=JSON.stringify([row.owner,row.petId,row.conversationId,row.turn]);if(seen.has(key))continue;seen.add(key);
  if(row.phase==='busy')result.active++;else if(row.phase==='waiting')result.waiting++;else result.reserved++;
 }
 return result;
}
