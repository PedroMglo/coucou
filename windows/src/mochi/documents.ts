import {providerForPill} from '../core/providers';
import {petRetired} from '../core/pet-factory';
import policy from '../../src-tauri/documents.policy.json';
import type {PetConfiguration} from './pet-preferences';
export function documentActions(config:PetConfiguration|null,pet:string,kind:'file'|'folder'='file',name=''){
 const p=config?.pets[pet],provider=providerForPill(pet),permissions=p?.permissions;
 const route=provider?(policy.routes as Record<string,string>)[provider]:undefined;
 // Match Path::extension in the backend: a leading dot is not an extension.
 const basename=name.slice(name.lastIndexOf('/')+1),dot=basename.lastIndexOf('.'),ext=dot>0?basename.slice(dot+1).toLowerCase():'';
 const binary=policy.unsupportedBinary.includes(ext),image=policy.images.includes(ext);
 const supported=!!route && (kind==='folder'||route==='native'||!binary&&(route!=='local'||!image));
 const active=!!provider&&!petRetired(pet);
 return{
  analysis:active && !!permissions?.documents && supported,
  email:active && !!permissions?.emailDrafts && kind==='file',
 };
}
