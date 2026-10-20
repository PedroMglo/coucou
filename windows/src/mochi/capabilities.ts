import {isServiceProvider,petName} from "./pet-preferences";
import {providerForPill,isCliProvider,type ProviderId} from "../core/providers";
import {pillDefinition} from "../core/pills";

export interface PetCapabilities {
  chat:ProviderId|null;voice:ProviderId|null;execute:boolean;role:string;
}
/** Product actions follow implemented bridges, never a name or an AI-looking icon. */
export function petCapabilities(id:string):PetCapabilities {
  if(id==='integration_spotify')return {chat:null,voice:'integration_spotify',execute:false,role:'Música e som do PC · controlos de áudio'};
  const chat=providerForPill(id),def=pillDefinition(id);
  if(chat && isServiceProvider(chat))return {chat,voice:chat,execute:true,role:`${petName(id,def?.name??id)} · ações na app/serviço · conta Codex`};
  if(chat && isCliProvider(chat))return {chat,voice:chat,execute:true,role:
    chat==="copilot"?"Tarefas no PC · conta GitHub Copilot":chat==="codex"?"Tarefas no PC · conta Codex":chat==="antigravity"?"Tarefas no PC · conta Gravity":
    `${chat==="gemini-cli"?"Gemini":"Claude"} via conta Gravity · tarefas no PC`};
  if(chat==="ollama")return {chat,voice:chat,execute:false,role:"Conversa local · ações limitadas do Mochi"};
  if(chat==="lmstudio")return {chat,voice:null,execute:false,role:"Conversa no servidor local"};
  if(chat)return {chat,voice:null,execute:false,role:"Chat por API · requer chave própria"};
  if(def?.category==="workspace" || def?.category==="agent")return {chat:null,voice:null,execute:false,role:"Acompanha sessões do CLI · sem chat integrado"};
  if(def?.category==="service")return {chat:null,voice:null,execute:false,role:"Dados e estado do serviço ligado"};
  return {chat:null,voice:null,execute:false,role:"Integração sem conversa disponível"};
}
