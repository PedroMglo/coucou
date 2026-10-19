import {Bridge,emitToWindow,type ConversationView} from "./bridge";
import {providerForPill} from "./providers";
import {Sound} from "./sound";
import {State} from "./state";

const resetting = new Map<string,Promise<boolean>>();
const ids=new Map<string,string>();
let nextMessage=1_000_000;
export function dialogueId(provider:string):string|undefined {return ids.get(provider);}
export function rememberDialogue(provider:string,view:ConversationView) {ids.set(provider,view.id);}
const BUSY = "Este pet já está a executar uma tarefa. Abre a tarefa para a acompanhar ou interromper.";

/** A fresh dialogue is acknowledged by Rust before any visible history is reset.
 * Native voice/recovery jobs are authoritative; task records belong to the pet.
 */
function changeDialogue(provider:string,restore?:string):Promise<boolean> {
  const existing=resetting.get(provider);if(existing)return existing;
  const fail=(error:unknown)=>{
    const message=String(error).replace(/^Error:\s*/,"");
    State.chatFeedback.set(provider,message);Sound.play("error");
    const pet=State.tasks.find(p=>providerForPill(p.id)===provider)?.id;
    if(pet && State.settings.companionMode)void emitToWindow("companions","companions-notice",{key:`reset:${provider}:${Date.now()}`,pet,text:message,urgent:false,view:"prompt"});
    State.notify();return false;
  };
  if(State.chatBusyProviders.has(provider))return Promise.resolve(fail(BUSY));
  State.chatResettingProviders.add(provider);State.chatFeedback.delete(provider);State.notify();
  const result=Promise.resolve().then(()=>restore?Bridge.chatRestoreConversation(provider,restore):Bridge.chatReset(provider)).then(view=>{
    if(view?.id)rememberDialogue(provider,view);else ids.delete(provider);
    State.replaceChatHistory(provider,(view?.messages??[]).map(m=>({...m,id:nextMessage++})));
    State.clearDialogueContext(provider);
    return true;
  }).catch(fail).finally(()=>{
    resetting.delete(provider);State.chatResettingProviders.delete(provider);State.notify();
  });
  resetting.set(provider,result);return result;
}

export function resetDialogue(provider=State.chatProvider):Promise<boolean> {return changeDialogue(provider);}
export function restoreDialogue(provider:string,id:string):Promise<boolean> {return changeDialogue(provider,id);}
