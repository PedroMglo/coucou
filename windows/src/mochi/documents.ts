import {providerForPill,isCliProvider} from '../core/providers';
import type {PetConfiguration} from './pet-preferences';
export function documentActions(config:PetConfiguration|null,pet:string,kind:'file'|'folder'='file',name=''){
 const p=config?.pets[pet],provider=providerForPill(pet),permissions=p?.permissions;
 const binary=/\.(?:png|jpe?g|gif|webp|mp[34]|wav|ogg|zip|docx?|xlsx?|pptx?|exe|bin)$/i.test(name);
 const localUnsupported=!!provider && ['ollama','lmstudio'].includes(provider) && kind==='file' && binary;
 return{
  analysis:!!permissions?.documents && !!provider && !localUnsupported && (kind==='file' || isCliProvider(provider) || ['ollama','lmstudio'].includes(provider)),
  email:!!permissions?.emailDrafts && kind==='file',
 };
}
