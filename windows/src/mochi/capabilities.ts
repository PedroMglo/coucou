import {petBlueprint} from '../core/pet-factory';
import {petName} from "./pet-preferences";
import {type ProviderId} from "../core/providers";
import {pillDefinition} from "../core/pills";

export interface PetCapabilities {
  chat:ProviderId|null;voice:ProviderId|null;execute:boolean;role:string;
}
/** Product actions follow implemented bridges, never a name or an AI-looking icon. */
export function petCapabilities(id:string):PetCapabilities {
  const blueprint=petBlueprint(id),def=pillDefinition(id),routing=blueprint?.routing;
  if(routing?.role){
    const role=routing.role==='service'?`${petName(id,def?.name??id)} · ações na app/serviço · conta Codex`:routing.role;
    return {chat:routing.chat as ProviderId|null,voice:routing.voice as ProviderId|null,execute:routing.execute,role};
  }
  if(def?.category==="workspace" || def?.category==="agent")return {chat:null,voice:null,execute:false,role:"Acompanha sessões do CLI · sem chat integrado"};
  if(def?.category==="service")return {chat:null,voice:null,execute:false,role:"Dados e estado do serviço ligado"};
  return {chat:null,voice:null,execute:false,role:"Integração sem conversa disponível"};
}
