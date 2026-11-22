import type {ChatContext,DroppedFile,PetAttachment,DocumentBatch} from './bridge';

/** Native item IDs and sequences are authority references. Never manufacture
 * them from paths/names or turn a malformed response into an empty selection. */
export function captureAttachments(provider:string,conversationId:string,values:unknown):PetAttachment[]{
 if(!Array.isArray(values)||values.length>32)throw Error('O conjunto de anexos recebido é inválido. Atualiza a conversa.');
 const ids=new Set<string>();
 return values.map(value=>{
  const a=value as PetAttachment;
  if(!a||typeof a.id!=='string'||!a.id||ids.has(a.id)||a.provider!==provider||a.conversationId!==conversationId||!Number.isSafeInteger(a.sequence)||a.sequence<0||typeof a.name!=='string'||typeof a.path!=='string'||!a.path||!['file','folder'].includes(a.kind)||!Number.isSafeInteger(a.size)||a.size<0||!Number.isSafeInteger(a.entries)||a.entries<0)throw Error('Um anexo não pertence a este pet e conversa ou perdeu a sua identidade. Atualiza a conversa.');
  ids.add(a.id);return {...a};
 });
}
export function attachmentContext(provider:string,conversationId:string,values:unknown):ChatContext|null{
 const files=captureAttachments(provider,conversationId,values);
 return files.length?{kind:'files',files:files.map(({id,name,path,sequence})=>({id,name,path,sequence}))}:null;
}
export function documentBatchMessage(batch:DocumentBatch):string{
 const copied=batch.attachments.length;
 const good=copied?`${copied} ${copied===1?'anexo recebido':'anexos recebidos'}.`:'';
 const failures=batch.failures.map(f=>`${f.name}: ${f.error}`).join('\n');
 return [good,failures,batch.error??''].filter(Boolean).join('\n')||'Nenhum anexo foi recebido.';
}
/** Native capture chose the CID at OS drop time. Fetching its current value
 * merely supplies the expected owner; Rust rejects a conversation switch. */
export async function receiveDocumentDrop(transport:{chatConversation:(provider:string)=>Promise<{id:string}>;petDocumentIngestDrop:(provider:string,cid:string,dropId:string)=>Promise<DocumentBatch>},provider:string,dropId:string|undefined,cid?:string):Promise<DocumentBatch>{
 if(!dropId)throw Error('A entrega não tem autorização nativa. Volta a arrastar os anexos para o pet.');
 const owner=cid??(await transport.chatConversation(provider)).id;
 const batch=await transport.petDocumentIngestDrop(provider,owner,dropId);
 if(!batch||!Array.isArray(batch.failures))throw Error('A entrega devolveu um resultado inválido. Atualiza a conversa antes de repetir.');
 return {...batch,attachments:captureAttachments(provider,owner,batch.attachments)};
}
export type DisplayFile=Pick<DroppedFile,'name'|'path'>&Partial<DroppedFile>;
