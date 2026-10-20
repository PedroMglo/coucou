import {Bridge,onDragDrop,emitToWindowOrThrow} from '../core/bridge';
import {PetDrop} from '../core/pet-drop';
import {petCapabilities} from '../mochi/capabilities';
import {PetPreferences} from '../mochi/pet-preferences';
import type {PetIdentity} from '../mochi/companions';

export function controlDocuments(surface:HTMLElement,options:{pets:()=>PetIdentity[];selected:()=>string|null;gallery:()=>void;collapse:()=>void;hold:(value:boolean)=>void;resize:()=>void}){
 const banner=document.createElement('aside');banner.id='center-document';banner.hidden=true;banner.setAttribute('aria-label','Entrega de documento');
 const message=document.createElement('p');message.setAttribute('role','status');
 const retry=document.createElement('button');retry.textContent='Tentar novamente';retry.hidden=true;
 const cancel=document.createElement('button');cancel.textContent='Cancelar entrega';banner.append(message,retry,cancel);surface.append(banner);
 let over=false,hovered:string|null=null,timer:ReturnType<typeof setTimeout>|undefined;
 const recipient=(id:string)=>{const pet=options.pets().find(p=>p.id===id),provider=pet&&petCapabilities(id).chat;return pet&&provider&&PetPreferences.config?.pets[id]?.permissions?.documents?{provider,name:pet.name}:null;};
 const drop=new PetDrop({recipient,providers:()=>options.pets().flatMap(p=>{const r=recipient(p.id);return r?[r.provider]:[];}),
  ingest:(path,provider,dropId)=>Bridge.ingestFile(path,provider,undefined,dropId),
  deliver:async(pet,file)=>{await emitToWindowOrThrow('island','pet-document',{pet,file});options.collapse();},cancel:Bridge.petDocumentCancelDrop,
 },render);
 function render(){
  const target=hovered&&recipient(hovered);
  message.textContent=drop.message||(over?(target?`Larga o documento para ${target.name}.`:'Larga o documento e escolhe o pet que o deve receber.'):'');
  banner.hidden=!message.textContent;retry.hidden=!drop.retryable;cancel.textContent=drop.waiting?'Cancelar entrega':'Fechar';cancel.disabled=false;
  if(drop.waiting&&!timer)timer=setTimeout(()=>{timer=undefined;drop.expire();},110000);
  if(!drop.waiting){clearTimeout(timer);timer=undefined;}
  banner.setAttribute('aria-busy',String(drop.busy));options.hold(over||drop.waiting);options.resize();
 }
 cancel.onclick=()=>{over=false;hovered=null;clearTimeout(timer);drop.cancel();};retry.onclick=()=>void drop.retry();
 void onDragDrop(event=>{
  if(event.type==='rejected'){over=false;drop.cancel();drop.message=event.error||'O documento não foi recebido. Volta a tentar.';render();return;}
  if(event.position){const scale=window.devicePixelRatio||1,el=document.elementFromPoint(event.position.x/scale,event.position.y/scale);hovered=el?.closest<HTMLElement>('[data-pet]')?.dataset.pet??el?.closest<HTMLElement>('[data-pet-id]')?.dataset.petId??options.selected();}
  else hovered=options.selected();
  if(event.type==='enter'||event.type==='over'){over=true;if(!options.selected())options.gallery();render();return;}
  if(event.type==='leave'){over=false;hovered=null;render();return;}
  if(event.type!=='drop')return;over=false;const pet=hovered??undefined;hovered=null;
  options.gallery();void drop.start(event.paths??[],event.dropId,pet);
 });
 return {get height(){return banner.hidden?0:58;},get holding(){return over||drop.waiting;},choose:(pet:string)=>{if(!drop.waiting)return false;void drop.choose(pet);return true;},cancel:()=>{over=false;hovered=null;clearTimeout(timer);drop.cancel();}};
}
