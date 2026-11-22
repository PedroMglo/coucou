import {Bridge,emitToWindow} from '../core/bridge';
import {requestDestinations,type RequestDestination} from '../core/request-destinations';

/** Open the native target as read-only routing, without restoring or deciding it. */
export async function presentRequest(destination:RequestDestination){
 if(!requestDestinations.current(destination))return;
 try{await Bridge.petChatOpen(destination.target.petId,undefined,false,destination.target,destination.id);}
 catch(error){
  if(!requestDestinations.current(destination))return;
  await emitToWindow('pet-control','pet-control-request-error',{...destination,message:String(error).replace(/^Error:\s*/,'')});
  if(requestDestinations.current(destination))await Bridge.petPanelOpen(360,50,undefined,'home').catch(error=>Bridge.log(`request destination could not be presented: ${String(error)}`));
 }
}
