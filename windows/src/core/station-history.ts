import {Bridge} from './bridge';
import {StationMetadataResource} from './station-metadata-resource';
import type {StationHistorySnapshot,StationHistoryMetric,StationHistoryProvider} from './station-history-contract';
import type {ObservationHistorySource,ObservationCount} from './pet-observability';
export type {StationHistorySnapshot} from './station-history-contract';
const record=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
const number=(v:unknown):v is number=>typeof v==='number'&&Number.isSafeInteger(v)&&v>=0;
const stamp=(v:unknown):v is number=>number(v)&&v>0&&v<=8.64e12;
const invalid=()=>Error('O histórico local tem um formato incompatível.');
export const stationHistoryProvider=(v:string):v is 'codex'|'copilot'=>v==='codex'||v==='copilot';
function metric(v:unknown):StationHistoryMetric {
 if(!record(v)||!(v.value===null||number(v.value))||!number(v.observed)||!['completeWithinIndex','completeWithinCatalogue','partial','unavailable'].includes(String(v.coverage))||!['localIndex','localCatalogue','localMetadataUnion','archiveMetadata'].includes(String(v.scope)))throw invalid();
 if(['completeWithinIndex','completeWithinCatalogue'].includes(String(v.coverage))&&v.value===null)throw invalid();
 if(v.coverage==='unavailable'&&(v.value!==null||v.observed!==0))throw invalid();
 if(typeof v.value==='number'&&v.value<v.observed)throw invalid();
 return {value:v.value as number|null,observed:v.observed,coverage:v.coverage as StationHistoryMetric['coverage'],scope:v.scope as StationHistoryMetric['scope']};
}
/** Only reviewed aggregate metadata crosses into the presentation model. */
export function parseStationHistory(input:unknown):StationHistorySnapshot {
 if(!record(input)||input.schemaVersion!==1||input.source!=='station-management-inventory'||!['available','unavailable'].includes(String(input.status))||!stamp(input.readAt)||!(input.observedAt===null||stamp(input.observedAt)&&input.observedAt<=input.readAt)||typeof input.refreshing!=='boolean'||typeof input.stale!=='boolean'||!(input.reason===null||['notObserved','collectionFailed','invalidInventory'].includes(String(input.reason)))||!Array.isArray(input.providers)||input.providers.length>2)throw invalid();
 const providers:StationHistoryProvider[]=[],seen=new Set<string>();
 for(const p of input.providers){
  if(!record(p)||typeof p.provider!=='string'||!stationHistoryProvider(p.provider)||seen.has(p.provider)||!(p.indexedTotal===null||number(p.indexedTotal)))throw invalid();
  if(p.provider==='copilot'&&p.indexedTotal!==null)throw invalid();
  seen.add(p.provider);providers.push({provider:p.provider,indexedTotal:p.indexedTotal as number|null,known:metric(p.known),archived:metric(p.archived)});
 }
 return {schemaVersion:1,source:'station-management-inventory',status:input.status as StationHistorySnapshot['status'],observedAt:input.observedAt as number|null,readAt:input.readAt,refreshing:input.refreshing,stale:input.stale||input.status==='unavailable'||input.observedAt===null,reason:input.reason as StationHistorySnapshot['reason'],providers};
}
function count(v:StationHistoryMetric,scope:string):ObservationCount {
 return {value:v.value,observed:v.observed,scope,coverage:v.coverage==='unavailable'?'unavailable':v.coverage==='partial'?'partial':'complete',detail:v.coverage==='completeWithinIndex'?'Completo apenas dentro do índice local consultado.':v.coverage==='completeWithinCatalogue'?'Completo apenas dentro do catálogo local consultado.':'Só os registos confirmados nas fontes locais consultadas.'};
}
/** History never establishes liveness, turn state, terminal count or tokens. */
export function stationHistorySource(snapshot:StationHistorySnapshot|null,provider:string,error=false):ObservationHistorySource|null {
 if(!stationHistoryProvider(provider)||!snapshot)return null;
 const p=snapshot.providers.find(p=>p.provider===provider);if(!p)return null;
 const label=`Histórico local do ${provider==='codex'?'Codex':'Copilot'}`;
 const detail=[p.provider==='copilot'&&p.known.scope==='localCatalogue'?'Registos do catálogo local consultado.':p.indexedTotal===null?'Índice local por confirmar.':`${p.indexedTotal.toLocaleString('pt-PT')} registos no índice local.`,
  'Cobertura das fontes locais consultadas; pode existir mais histórico.',p.archived.coverage==='unavailable'?'O estado de arquivo não é disponibilizado por esta fonte.':'',
  snapshot.reason==='collectionFailed'||error?'A atualização falhou; os dados anteriores mantêm a data original.':snapshot.reason==='invalidInventory'?'Não foi possível validar a nova leitura.':''].filter(Boolean).join(' ');
 return {key:'external',label,updatedAt:snapshot.observedAt,stale:snapshot.stale||error,total:count(p.known,label),archived:count(p.archived,label),detail};
}
export const PetStationHistory=new StationMetadataResource(force=>Bridge.petStationHistory(force),parseStationHistory,'Não foi possível atualizar o histórico local. Os dados anteriores mantêm a data original.');
