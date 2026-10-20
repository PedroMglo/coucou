import type {DroppedFile} from './bridge';

interface Drop {path:string;dropId?:string;providers:string[]}
interface Receipt {pet:string;provider:string;file:DroppedFile}
interface Transport {
 recipient:(pet:string)=>{provider:string;name:string}|null;
 providers:()=>string[];
 ingest:(path:string,provider:string,dropId?:string)=>Promise<DroppedFile>;
 deliver:(pet:string,file:DroppedFile)=>Promise<void>;
 cancel:(provider:string,dropId:string)=>Promise<unknown>;
}
/** A desktop drop has an explicit recipient. Choosing a pet never starts a task. */
export class PetDrop {
 private drop:Drop|null=null;private receipt:Receipt|null=null;private version=0;
 busy=false;message='';
 get waiting(){return !!this.drop;}
 get retryable(){return !!this.receipt&&!this.busy;}
 constructor(private transport:Transport,private changed:()=>void){}
 private discard(drop:Drop){if(drop.dropId)void Promise.allSettled(drop.providers.map(p=>this.transport.cancel(p,drop.dropId!)));}
 cancel(){const drop=this.drop;this.version++;this.drop=null;this.receipt=null;this.busy=false;this.message='';if(drop)this.discard(drop);this.changed();}
 expire(){this.cancel();this.message='Esta entrega expirou. Volta a arrastar o documento para o pet.';this.changed();}
 async start(paths:string[],dropId?:string,pet?:string){
  const providers=[...new Set(this.transport.providers())];
  if(this.busy){if(dropId)this.discard({path:paths[0]??'',dropId,providers});return;}
  this.cancel();
  if(paths.length!==1){if(dropId)this.discard({path:'',dropId,providers});this.message='Entrega um documento ou uma pasta de cada vez.';this.changed();return;}
  this.drop={path:paths[0],dropId,providers};this.message='Escolhe o pet que deve receber este documento.';this.changed();
  if(pet)await this.choose(pet);
 }
 async choose(pet:string):Promise<boolean>{
  if(!this.drop)return false;if(this.busy)return true;
  if(this.receipt&&this.receipt.pet!==pet){this.message='O documento já foi recebido pelo pet escolhido. Tenta abrir as ações novamente ou cancela esta entrega.';this.changed();return true;}
  const recipient=this.transport.recipient(pet);
  if(!recipient){this.message='Este pet não recebe documentos. Escolhe outro pet com conversa.';this.changed();return true;}
  const drop=this.drop,ticket=this.version;this.busy=true;this.message=`A entregar a ${recipient.name}…`;this.changed();
  try{
   let receipt=this.receipt;
   if(!receipt){const file=await this.transport.ingest(drop.path,recipient.provider,drop.dropId);if(ticket!==this.version)return true;
    if(file.provider&&file.provider!==recipient.provider)throw Error('O documento recebido não pertence ao pet escolhido.');
    receipt={pet,provider:recipient.provider,file};this.receipt=receipt;
   }
   await this.transport.deliver(receipt.pet,receipt.file);if(ticket!==this.version)return true;
   this.drop=null;this.receipt=null;this.message='';
  }catch(error){if(ticket!==this.version)return true;this.message=String(error).replace(/^Error:\s*/,'');if(!this.receipt){this.discard(drop);this.drop=null;this.message+=' Volta a arrastar o documento.';}}
  finally{if(ticket===this.version){this.busy=false;this.changed();}}
  return true;
 }
 async retry(){if(this.receipt)await this.choose(this.receipt.pet);}
}
