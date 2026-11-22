import {Bridge} from './bridge';
import {State,type Settings} from './state';

/** All Ollama selectors in this window share one intent counter. A slow reply
 * cannot open a chat, close a picker or overwrite the user's newer choice. */
export class OllamaSelection {
 private intent=0;
 private generation=0;
 private model:string|undefined;
 pending=false;
 applySettings(settings:Settings&{ollamaSelectionGeneration?:number}):Settings{
  const generation=settings.ollamaSelectionGeneration,model=settings.chatModels?.ollama;
  if(typeof generation==='number'&&Number.isSafeInteger(generation)&&generation>this.generation&&typeof model==='string'){
   this.generation=generation;this.model=model;
  }
  return this.model===undefined?settings:{...settings,chatModels:{...settings.chatModels,ollama:this.model}};
 }
 async select(model:string){
  const ticket=++this.intent,selectionId=globalThis.crypto?.randomUUID?.()??`model-${Date.now()}-${ticket}`;
  const before=State.settings.chatModels.ollama;
  this.pending=true;State.notify();
  try{
   const ack=await Bridge.petModelSelect(model,selectionId);
   if(ticket!==this.intent)return false;
   if(ack?.provider!=='ollama'||ack.model!==model||ack.selectionId!==selectionId||!Number.isSafeInteger(ack.generation)||ack.generation<1)throw new Error('A escolha do modelo não foi confirmada. Atualiza a lista e tenta novamente.');
   if(ack.generation<this.generation||ack.generation===this.generation&&this.model!==ack.model)return false;
   // During a rolling frontend update, an already observed native setting can
   // lack generation metadata. Do not replace a different newer observation.
   if(!this.generation&&State.settings.chatModels.ollama!==before&&State.settings.chatModels.ollama!==ack.model)return false;
   State.settings=this.applySettings({...State.settings,chatModels:{...State.settings.chatModels,ollama:ack.model},ollamaSelectionGeneration:ack.generation});
   return true;
  }catch(error){if(ticket!==this.intent)return false;throw error;}
  finally{if(ticket===this.intent){this.pending=false;State.notify();}}
 }
}
export const OllamaModelSelection=new OllamaSelection();
