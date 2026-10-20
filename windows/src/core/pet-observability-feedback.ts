import type {Inventory,UsageSummary} from './pet-observability-contract';
export interface TokenFeedback {status:'unknown'|'unavailable'|'noObservations'|'observed';title:string;detail:string;since:number|null}
/** A partial empty collection is not evidence of an empty provider. */
export function hasInventoryEvidence(source:Inventory|undefined):boolean {
 return !!source&&(['total','archived','confirmedActive','working','waiting'] as const).some(key=>{
  const count=source.counts?.[key];return [count?.value,count?.observed].some(n=>typeof n==='number'&&Number.isSafeInteger(n)&&n>0);
 });
}
export function inventoryFeedback(source:Inventory|undefined):string {
 if(source?.coverage==='notSupported')return 'Este provider não disponibiliza inventário externo. A biblioteca deste pet continua disponível.';
 if(source?.refreshing&&source.coverage==='unavailable')return 'A consultar sessões externas. Ainda não há inventário confirmado; isto não significa zero sessões.';
 if(source?.reason==='nativePetDisabled')return 'O pet nativo deste provider está desativado. As sessões externas não foram consultadas.';
 if(source?.reason==='clientUnavailable')return 'O cliente nativo está indisponível nesta consulta. Não é possível confirmar as sessões externas.';
 if(source?.reason==='refreshFailed')return 'A consulta das sessões externas falhou. A data e os dados anteriores foram preservados, quando disponíveis.';
 if(!source||source.coverage==='unavailable')return 'As sessões externas ainda não foram confirmadas. Isto não significa que não existam.';
 return source.coverage==='partial'?hasInventoryEvidence(source)?'Inventário externo parcial; podem existir mais sessões.':'Ainda não há contagens externas confirmadas nesta fonte parcial. A biblioteca do pet é apresentada separadamente.':'Cobertura completa desta fonte; não representa todas as contas ou aplicações.';
}
export function tokenFeedback(usage:UsageSummary|undefined,provider:string):TokenFeedback {
 const unknown:TokenFeedback={status:'unknown',title:'Medições por confirmar',detail:'Ainda não há informação confirmada sobre o registo acumulado de tokens deste provider.',since:null};
 if(!usage||usage.scope!=='mochi-observed-attempts'||usage.owner!==null||usage.provider!==provider)return unknown;
 const since=typeof usage.since==='number'&&Number.isSafeInteger(usage.since)&&usage.since>0&&usage.since<=8.64e12?usage.since:null;
 if(usage.status==='unavailable')return {status:'unavailable',since,title:'Registo de tokens indisponível',detail:provider==='ollama'?'O registo acumulado do Ollama está indisponível nesta consulta. Valores desconhecidos não representam consumo zero.':'O Coucou ainda não dispõe de um registo acumulado de tokens deste provider. A quota da conta é apresentada separadamente, quando disponível.'};
 if(usage.status==='noObservations'){
  const pending=usage.attempts?.pending;
  return {status:'noObservations',since,title:'Sem medições registadas',detail:'Ainda não há contadores de tokens registados pelo Coucou neste período. Isto não confirma consumo zero.'+(typeof pending==='number'&&Number.isSafeInteger(pending)&&pending>0?` ${pending} pedidos por concluir.`:'')};
 }
 return usage.status==='observed'?{status:'observed',since,title:'Tokens registados no Coucou',detail:'Medições dos pedidos acompanhados pelo Coucou; não incluem outras aplicações nem o histórico anterior.',}:unknown;
}
