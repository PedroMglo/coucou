import {petSessionStatesSummary,type PetSessionStateSummary} from './pet-session-states';
import {inventoryFeedback,tokenFeedback,hasInventoryEvidence,type TokenFeedback} from './pet-observability-feedback';
import type {SessionTarget} from './session-chat';
import type {PetCapabilityDescriptor} from './bridge';
import {capabilityDescriptorFor} from './pet-capabilities';
import type {Coverage,Inventory,SessionRow,PetObservabilitySnapshot,UsageSummary,UsageDimension} from './pet-observability-contract';
import {parseProcessCensus,livePetWorkSummary,type CensusPresentation,type LiveWorkSummary} from './process-census';
/** Public presentation model. Missing measurements are never inferred from
 * hooks, history length or the last measured turn. */
export type ObservationCoverage='complete'|'partial'|'unavailable';
export interface ObservationCount {value:number|null;coverage:ObservationCoverage;scope:string;detail?:string;observed?:number}
export interface ObservedSession {
 key:string;id:string;backend:string;title:string;cwd:string;model?:string;
 origin:'pet'|'external'|'unknown';owner:string;archived:boolean|null;state:string;updatedAt?:number;canOpen:boolean;target:SessionTarget|null;
}
export interface ProviderTokenUsage {
 inputTokens:string|null;outputTokens:string|null;totalTokens:string|null;
 since:number|null;updatedAt:number|null;coverage:ObservationCoverage;scope:string;detail:string;
}
export interface ObservationHistorySource {
 key:'pet'|'external';label:string;updatedAt:number|null;stale:boolean;
 total:ObservationCount;archived:ObservationCount;detail:string;
}
export interface PetObservation {
 capabilityDescriptor:PetCapabilityDescriptor|null;
 petId:string;provider:string;providerName:string;updatedAt:number|null;stale:boolean;refreshing:boolean;
 active:ObservationCount;total:ObservationCount;archived:ObservationCount;
 working:ObservationCount;waiting:ObservationCount;
 sessions:ObservedSession[];sessionsComplete:boolean;tokens:ProviderTokenUsage|null;notes:string[];
 library:{updatedAt:number|null;total:ObservationCount;active:ObservationCount;working:ObservationCount;waiting:ObservationCount;archived:ObservationCount};
 primaryScope:'pet'|'provider';externalStatus:string;tokenStatus:TokenFeedback;
 scopeLabel:string;scopeDetail:string;
 opened:ObservationCount;history:ObservationHistorySource[];
 sessionStates?:PetSessionStateSummary|null;
 processCensus:CensusPresentation|null;liveWork:LiveWorkSummary|null;
 nativeCoverage:Coverage|null;
}
export function observationNumber(value:unknown):number|null {
 return typeof value==='number'&&Number.isSafeInteger(value)&&value>=0?value:null;
}
export function observationCount(value:ObservationCount|undefined):string {
 const n=observationNumber(value?.value);
 if(!value||value.coverage==='unavailable')return '—';
 const observed=observationNumber(value.observed);
 if(n===null)return value.coverage==='partial'&&observed!==null&&observed>0?`≥ ${observed.toLocaleString('pt-PT')}`:'—';
 return `${value.coverage==='partial'?'≥ ':''}${n.toLocaleString('pt-PT')}`;
}
export function observationTime(value:number|null|undefined):string {
 return typeof value==='number'&&Number.isSafeInteger(value)&&value>0&&value<=8.64e12?new Date(value*1000).toLocaleTimeString('pt-PT',{hour:'2-digit',minute:'2-digit',second:'2-digit'}):'por apurar';
}
export function observationDate(value:number|null|undefined):string {
 return typeof value==='number'&&Number.isSafeInteger(value)&&value>0&&value<=8.64e12?new Date(value*1000).toLocaleString('pt-PT',{day:'numeric',month:'short',year:'numeric',hour:'2-digit',minute:'2-digit'}):'período por apurar';
}
export const observationState=(state:string)=>({working:'Em execução',running:'Em execução',thinking:'A pensar',searching:'A pesquisar',approval:'Aguarda aprovação',attention:'Precisa de ti',question:'Aguarda resposta',idle:'Sem turno em curso',saved:'Guardada',archived:'Arquivada',finished:'Concluída',error:'Precisa de atenção',unknown:'Estado por apurar',waiting:'À tua espera',busy:'Em curso',notLoaded:'Guardada',reserved:'A preparar'} as Record<string,string>)[state]??'Estado por apurar';
export const observationOrigin=(origin:ObservedSession['origin'])=>origin==='pet'?'No pet':origin==='external'?'Externa':'Origem por apurar';

/** Token ledgers serialize decimal strings to preserve every digit. */
export function observationTokens(value:unknown):string {
 return typeof value==='string'&&/^(0|[1-9]\d{0,127})$/.test(value)?BigInt(value).toLocaleString('pt-PT'):'—';
}
export const observationScope=(scope:string)=>({mochiLibrary:'Biblioteca deste pet no Coucou',codexDaemon:'Inventário local do Codex',copilotLiveEndpoints:'Sessões detetadas do Copilot',providerGlobal:'Inventário do provider'} as Record<string,string>)[scope]??'Inventário consultado';
function countFrom(inventory:Inventory|undefined,key:'total'|'archived'|'confirmedActive'|'working'|'waiting'):ObservationCount {
 const c=inventory?.counts?.[key],n=observationNumber(c?.value),valid=inventory?.coverage==='complete'||inventory?.coverage==='partial';
 return {value:valid?n:null,observed:valid?observationNumber(c?.observed)??undefined:undefined,
  coverage:!valid?'unavailable':n===null||inventory?.coverage==='partial'?'partial':'complete',
  scope:inventory?observationScope(inventory.scope):'Inventário por apurar',detail:inventory?.coverage==='partial'?'Cobertura parcial desta fonte.':undefined};
}
function exactTarget(row:SessionRow):SessionTarget|null {
 const target=row.open;if(!target||!target.petId||!target.provider||row.petId!==target.petId)return null;
 if(target.kind==='mochi'&&row.origin==='mochi'&&target.conversationId&&target.conversationId===row.conversationId)
  return {petId:target.petId,provider:target.provider,origin:'mochi',sessionId:target.conversationId,conversationId:target.conversationId};
 if(target.kind==='external'&&row.origin==='external'&&target.sessionId&&target.sessionId===row.sessionId&&target.provider===row.provider)
  return {petId:target.petId,provider:target.provider,origin:'external',sessionId:target.sessionId};
 return null;
}
function sessionFrom(row:SessionRow,name:(pet:string)=>string):ObservedSession {
 const target=exactTarget(row),id=row.sessionId||row.conversationId||row.key;
 return {key:target?JSON.stringify([target.petId,target.provider,target.origin,target.sessionId]):row.key,id,backend:row.provider,title:row.title,cwd:row.cwd??'',origin:row.origin==='mochi'?'pet':row.origin==='external'?'external':'unknown',
  owner:row.petId?name(row.petId):'',archived:row.archived,state:row.state,updatedAt:row.updatedAt??undefined,canOpen:!!target,target};
}
function measuredTokens(dimension:UsageDimension):string|null {
 const attempts=(observationNumber(dimension?.knownAttempts)??0)+(observationNumber(dimension?.partialAttempts)??0);
 return dimension?.coverage!=='unavailable'&&attempts>0&&observationTokens(dimension?.knownTokens)!=='—'?dimension.knownTokens:null;
}
function tokensFrom(usage:UsageSummary|undefined,provider:string):ProviderTokenUsage|null {
 if(!usage||usage.scope!=='mochi-observed-attempts'||usage.provider!==provider||usage.owner!==null||usage.status!=='observed')return null;
 const total=usage.total,input=usage.input,output=usage.output;
 const parts=[`${observationNumber(usage.attempts?.tracked)??'—'} pedidos registados`];
 if(total.partialAttempts)parts.push(`${total.partialAttempts} medições parciais`);
 const absent=(observationNumber(total.missingAttempts)??0)+(observationNumber(total.unknownAttempts)??0);
 if(absent)parts.push(`${absent} sem contadores de total`);
 if(usage.attempts.pending)parts.push(`${usage.attempts.pending} por concluir`);
 parts.push('Não inclui outras aplicações, histórico anterior ou pedidos não acompanhados.');
 const persistence=usage.persistence?.state;
 if(persistence==='memoryOnly')parts.push('Registo apenas em memória.');
 else if(persistence==='dirty')parts.push('Há medições por guardar.');
 else if(persistence==='error')parts.push('Falhou a gravação; podem perder-se medições ao reiniciar.');
 else if(persistence==='unknown')parts.push('Confirmação da gravação por apurar.');
 return {inputTokens:measuredTokens(input),outputTokens:measuredTokens(output),totalTokens:measuredTokens(total),
  since:usage.since,updatedAt:usage.lastObservation,coverage:total.coverage==='completeWithinTrackedAttempts'?'complete':'partial',
  scope:'Pedidos registados pelo Coucou',detail:parts.join(' · ')};
}
/** Native and Mochi sources overlap. They are displayed separately and their
 * counters are never added. A bounded list cannot become an inventory total. */
export function petObservation(snapshot:PetObservabilitySnapshot|null,petId:string,name:(pet:string)=>string=id=>id):PetObservation|null {
 if(snapshot?.schemaVersion!==1||!Array.isArray(snapshot.pets)||!Array.isArray(snapshot.providers))return null;
 const found=snapshot.pets.filter(p=>p.petId===petId);if(found.length!==1)return null;const pet=found[0];
 const providers=snapshot.providers.filter(p=>p.provider===pet.provider),native=providers.length===1?providers[0].native:undefined,library=pet.mochi;
 if(!library?.counts||!Array.isArray(library.sessions))return null;
 const sessions=new Map<string,ObservedSession>();
 for(const row of [...library.sessions,...(native?.sessions??[])]){
  const session=sessionFrom(row,name),key=session.target?JSON.stringify([session.target.petId,session.target.provider,session.target.origin,session.target.sessionId]):row.key;
  if(key&&!sessions.has(key))sessions.set(key,session);
 }
 const sorted=[...sessions.values()].sort((a,b)=>{const rank=(s:ObservedSession)=>['working','waiting','busy'].includes(s.state)?0:s.state==='unknown'?1:s.archived===true?3:2;return rank(a)-rank(b)||(b.updatedAt??0)-(a.updatedAt??0);});
 const primaryScope=native&&(native.coverage==='complete'||native.coverage==='partial'&&hasInventoryEvidence(native))?'provider':'pet';
 const primary=primaryScope==='provider'?native:library;
 const count=(key:'total'|'archived'|'confirmedActive'|'working'|'waiting')=>countFrom(primary,key);
 const local=(key:'total'|'archived'|'confirmedActive'|'working'|'waiting')=>countFrom(library,key);
 const notes:string[]=[];
 if(native?.counts?.unknown)notes.push(`${native.counts.unknown.toLocaleString('pt-PT')} sessões com atividade por apurar.`);
 if(native?.truncated||library.truncated)notes.push('A lista é uma seleção; as contagens incluem os registos anteriores ao limite.');
 if(library.reason==='memoryNotDurable')notes.push('A biblioteca tem alterações que ainda não foram guardadas no disco.');
 // Contract v1 has no open-session census. In-flight, idle, loaded and saved
 // rows cannot establish whether a native pet or terminal session is open.
 const opened:ObservationCount={value:null,coverage:'unavailable',scope:'Pet e terminal',detail:'A contagem de sessões abertas no pet e no terminal ainda não está confirmada.'};
 const history:ObservationHistorySource[]=[
  {key:'external',label:native?observationScope(native.scope):'Histórico externo',updatedAt:native?.observedAt??null,stale:!!native?.stale,total:countFrom(native,'total'),archived:countFrom(native,'archived'),detail:inventoryFeedback(native)},
  {key:'pet',label:'Biblioteca deste pet no Coucou',updatedAt:library.observedAt,stale:library.stale,total:local('total'),archived:local('archived'),detail:'Só as conversas guardadas neste pet. Pode sobrepor-se ao histórico externo; os valores não são somados.'},
 ];
 return {capabilityDescriptor:capabilityDescriptorFor(snapshot.petCapabilities,petId,pet.provider),petId,provider:pet.provider,providerName:pet.provider,primaryScope,externalStatus:inventoryFeedback(native),tokenStatus:tokenFeedback(providers.length===1?providers[0].usage:undefined,pet.provider),updatedAt:primary?.observedAt??null,opened,history,processCensus:providers.length===1?parseProcessCensus(providers[0].processCensus,pet.provider):null,liveWork:livePetWorkSummary(snapshot.livePetWork,petId),sessionStates:petSessionStatesSummary(snapshot.livePetWork,petId),nativeCoverage:native?.coverage??null,
  refreshing:!!primary?.refreshing,stale:!!primary?.stale,active:count('confirmedActive'),total:count('total'),archived:count('archived'),working:count('working'),waiting:count('waiting'),
  sessions:sorted,sessionsComplete:!native?.truncated&&!library.truncated,tokens:tokensFrom(providers.length===1?providers[0].usage:undefined,pet.provider),notes,
  library:{updatedAt:library.observedAt,total:local('total'),active:local('confirmedActive'),working:local('working'),waiting:local('waiting'),archived:local('archived')},
  scopeLabel:primary?observationScope(primary.scope):'Inventário por apurar',scopeDetail:primaryScope==='pet'?'Estas contagens pertencem apenas à biblioteca deste pet. Não incluem as sessões externas.':inventoryFeedback(native)};
}
