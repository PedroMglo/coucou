import {petObservation,observationOrigin,type ObservedSession} from '../core/pet-observability';
import type {PetObservabilitySnapshot} from '../core/pet-observability-contract';
import {sessionTargetKey,type SessionTarget} from '../core/session-chat';
import type {ManagedSession} from './management';

/** The helper list stays complete. Only an unambiguous canonical observation
 * may supply a chat route; a missing row in its bounded selection proves nothing. */
export function managedSessionLinks(snapshot:PetObservabilitySnapshot|null,unavailable=false){
 const unknown={target:null,origin:'Origem por apurar',reason:'A ligação desta sessão ao chat ainda não está confirmada. Atualiza a informação antes de abrir.'};
 const groups=new Map<string,{session:ObservedSession;stale:boolean}[]>();
 if(snapshot?.schemaVersion===1)for(const pet of snapshot.pets){
  const observation=petObservation(snapshot,pet.petId);if(!observation)continue;
  for(const session of observation.sessions){const key=JSON.stringify([session.backend,session.id]),group=groups.get(key)??[];group.push({session,stale:observation.stale});groups.set(key,group);}
 }
 return (row:Pick<ManagedSession,'provider'|'id'>):{target:SessionTarget|null;origin:string;reason:string}=>{
 const matches=groups.get(JSON.stringify([row.provider,row.id]));if(!matches?.length)return unknown;
 if(unavailable||matches.some(m=>m.stale))return {...unknown,reason:'A ligação ao chat está desatualizada. Atualiza a informação antes de abrir.'};
 if(matches.some(m=>!m.session.target))return {...unknown,reason:'Não foi possível confirmar o proprietário desta sessão. A conversa permanece indisponível.'};
 const targets=new Map(matches.map(m=>[sessionTargetKey(m.session.target!),m.session]));
 if(targets.size!==1)return {...unknown,reason:'Esta sessão tem mais de um proprietário possível. A conversa permanece indisponível.'};
 const session=[...targets.values()][0];return {target:session.target,origin:observationOrigin(session.origin),reason:''};
 };
}
export const managedSessionLink=(snapshot:PetObservabilitySnapshot|null,row:Pick<ManagedSession,'provider'|'id'>,unavailable=false)=>managedSessionLinks(snapshot,unavailable)(row);
