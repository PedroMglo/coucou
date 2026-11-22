import type {PetAttachment,DocumentBatch} from './bridge';
import {documentBatchMessage} from './document-collection';
interface Drop {paths:string[];dropId?:string;providers:string[]}
interface Receipt {pet:string;provider:string;files:PetAttachment[];message:string}
interface Transport {
 recipient:(pet:string)=>{provider:string;name:string}|null;
 providers:()=>string[];
 ingest:(paths:string[],provider:string,dropId?:string)=>Promise<DocumentBatch>;
 deliver:(pet:string,files:PetAttachment[],message:string)=>Promise<void>;
 cancel:(provider:string,dropId:string)=>Promise<unknown>;
}
/** A native batch has an explicit recipient. Choosing a pet never starts a task. */
export class PetDrop {
 private drop:Drop|null=null;private receipt:Receipt|null=null;private version=0;
 busy=false;message='';
 get waiting(){return !!this.drop;}
 get retryable(){return !!this.receipt&&!this.busy;}
 constructor(private transport:Transport,private changed:()=>void){}
 private discard(drop:Drop){if(drop.dropId)void Promise.allSettled(drop.providers.map(p=>this.transport.cancel(p,drop.dropId!)));}
 cancel(){const drop=this.drop;this.version++;this.drop=null;this.receipt=null;this.busy=false;this.message='';if(drop)this.discard(drop);this.changed();}
 expire(){this.cancel();this.message='Esta entrega expirou. Volta a arrastar os anexos para o pet.';this.changed();}
 async start(paths:string[],dropId?:string,pet?:string){
  const providers=[...new Set(this.transport.providers())];
  if(this.busy){if(dropId)this.discard({paths:[...paths],dropId,providers});this.message='A entrega anterior ainda está em curso. Volta a arrastar os novos anexos quando terminar.';this.changed();return;}
  this.cancel();
  if(!paths.length||paths.length>32||!dropId){if(dropId)this.discard({paths:[...paths],dropId,providers});this.message='Entrega entre um e 32 anexos através de uma entrega nativa.';this.changed();return;}
  this.drop={paths:[...paths],dropId,providers};this.message='Escolhe o pet que deve receber estes anexos.';this.changed();
  if(pet)await this.choose(pet);
 }
 async choose(pet:string):Promise<boolean>{
  if(!this.drop)return false;if(this.busy)return true;
  if(this.receipt&&this.receipt.pet!==pet){this.message='Os anexos já foram recebidos pelo pet escolhido. Tenta abrir as ações novamente ou cancela esta entrega.';this.changed();return true;}
  const recipient=this.transport.recipient(pet);
  if(!recipient){this.message='Este pet não recebe documentos. Escolhe outro pet com conversa.';this.changed();return true;}
  const drop=this.drop,ticket=this.version;this.busy=true;this.message=`A entregar a ${recipient.name}…`;this.changed();
  try{
   let receipt=this.receipt;
   if(!receipt){const batch=await this.transport.ingest(drop.paths,recipient.provider,drop.dropId);if(ticket!==this.version)return true;
    if(batch.attachments.some(file=>file.provider!==recipient.provider))throw Error('Um anexo recebido não pertence ao pet escolhido.');
    if(!batch.attachments.length)throw Error(documentBatchMessage(batch));
    receipt={pet,provider:recipient.provider,files:batch.attachments.map(a=>({...a})),message:documentBatchMessage(batch)};this.receipt=receipt;
   }
   await this.transport.deliver(receipt.pet,receipt.files,receipt.message);if(ticket!==this.version)return true;
   this.drop=null;this.receipt=null;this.message=receipt.message;
  }catch(error){if(ticket!==this.version)return true;this.message=String(error).replace(/^Error:\s*/,'');if(!this.receipt){this.discard(drop);this.drop=null;this.message+=' Volta a arrastar os anexos.';}}
  finally{if(ticket===this.version){this.busy=false;this.changed();}}
  return true;
 }
 async retry(){if(this.receipt)await this.choose(this.receipt.pet);}
}
