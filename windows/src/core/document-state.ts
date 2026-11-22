import {Bridge,type PetAttachment} from './bridge';
import {State} from './state';
import {dialogueId,rememberDialogue} from './chat-dialogue';
const reads=new Map<string,symbol>();
/** One read generation per provider across every receiving surface in this
 * webview. A late drop/event cannot replace a newer canonical collection. */
export async function refreshDocumentState(provider:string,expectedId?:string):Promise<{files:PetAttachment[];applied:boolean}>{
 const ticket=Symbol();reads.set(provider,ticket);
 const current=await Bridge.chatConversation(provider);
 if(expectedId&&current.id!==expectedId)throw Error('A conversa mudou durante a entrega. Os anexos não foram enviados para outra conversa.');
 const files=await Bridge.petDocumentContexts(provider,current.id);
 if(reads.get(provider)!==ticket||dialogueId(provider)&&dialogueId(provider)!==current.id)return {files,applied:false};
 if(!dialogueId(provider))rememberDialogue(provider,current);
 State.setFilesContext(provider,files);State.notify();return {files,applied:true};
}
export function invalidateDocumentState(provider?:string){
 if(provider){reads.set(provider,Symbol());State.clearDialogueContext(provider);}else{reads.clear();State.clearAllDialogueContexts();}State.notify();
}
