import {petSessionStatesText} from './pet-session-states';
import {observationCount,observationDate,type PetObservation} from './pet-observability';
import {processCensusSummary} from './process-census';

export interface ScopedSessionSummary {
 kind:'clients'|'library'|'unknown';label:string;badge:string;summary:string;detail:string;stale:boolean;
}
/** Compact views use the same measured sources as the pet dashboard. The old
 * mixed, retained hook registry has no count authority here. */
export function scopedSessionSummary(value:PetObservation|null,error=false,expectedProvider?:string):ScopedSessionSummary {
 const switched=!!value&&!!expectedProvider&&value.provider!==expectedProvider;
 const census=processCensusSummary(!switched&&value?.processCensus?{...value.processCensus,stale:value.processCensus.stale||error}:null);
 const library=value?.history.find(s=>s.key==='pet'),local=observationCount(library?.total),hasLocal=local!=='—';
 const localStale=error||!!library?.stale,clientStale=error||!!value?.processCensus?.stale;
 const localText=hasLocal?`Coucou deste pet: ${local} ${(library?.total.value??library?.total.observed)===1?'registo':'registos'}${localStale?' · leitura anterior':''}`:'Biblioteca deste pet por confirmar';
 const censusText=census?`${census.count} ${census.unit==='terminals'?(census.count==='1'?'terminal':'terminais'):(census.count==='1'?'cliente':'clientes')} · ${census.caption}`:'Terminais por confirmar';
 const stale=!!census&&clientStale||localStale;
 const stateText=petSessionStatesText(!switched?value?.sessionStates:null,error||!!value?.stale);
 const detail=[stateText?`${stateText}. Este estado refere-se à conversa carregada na app; não às sessões externas nem aos objetivos autónomos. Trabalho indica um turno aberto na app; não confirma inferência em curso.`:'',census?.detail??(switched?'O provider do pet mudou; os terminais estão por confirmar.':'Ainda não há contagem confirmada dos clientes CLI deste provider.'),
  library?`${localText} · ${library.label} · ${library.updatedAt?observationDate(library.updatedAt):'Data por confirmar'}.`:'',
  error?'A atualização falhou; as leituras anteriores mantêm a data original.':'',
  'Clientes CLI, turnos em execução e conversas guardadas são contagens diferentes.'].filter(Boolean).join(' ');
 return {kind:census?'clients':hasLocal?'library':'unknown',label:census?`${census.count} CLI${clientStale?' · anterior':''}`:hasLocal?`${local} no Coucou${localStale?' · anterior':''}`:'Sessões',badge:census?`${census.count} CLI`:'',summary:[censusText,localText,stateText].filter(Boolean).join('\n'),detail,stale};
}
