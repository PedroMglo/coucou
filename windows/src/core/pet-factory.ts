/**
 * Two product molds with explicit per-pet extensions. Bundled presentation and
 * configuration data only: this registry never grants native execution/access.
 */
import catalog from '../../src-tauri/pets.factory.json';
import type {PetConfiguration,PetPreference,PetPermissions} from '../mochi/pet-preferences';

export type PetMold='llm'|'application';
export interface PetPresentation {
 readonly galleryAvatarSize:number;readonly overviewAvatarSize:number;
 readonly galleryCardRadius:number;readonly galleryCardGap:number;
 readonly profiles:boolean;readonly sessions:boolean;readonly metrics:boolean;
 readonly contextAction:'open'|'sessions';readonly galleryAction:string;
 readonly galleryState:string|null;readonly accessibilityRole:string|null;
 readonly permissions:readonly (keyof PetPermissions)[];readonly musicLabel:string;
}
export interface PetRouting {
 readonly chat:string|null;readonly voice:string|null;readonly execute:boolean;
 readonly taskProvider:string|null;readonly role:string|null;readonly taskName:string|null;
}
export interface PetBlueprint {
 readonly id:string;readonly mold:PetMold;readonly service:boolean;
 readonly preferences:PetPreference;readonly presentation:PetPresentation;readonly routing:PetRouting;
}
type Row=Record<string,unknown>;
const object=(v:unknown):v is Row=>!!v&&typeof v==='object'&&!Array.isArray(v);
function invalid():never{throw new Error('Invalid bundled pet factory');}
function merge(base:Row,override:Row):Row {
 const value=structuredClone(base);
 for(const[key,next]of Object.entries(override)){
  if(['__proto__','prototype','constructor'].includes(key))invalid();
  value[key]=object(next)&&object(value[key])?merge(value[key] as Row,next):structuredClone(next);
 }
 return value;
}
function freeze<T>(value:T):T {
 if(value&&typeof value==='object'){
  for(const child of Object.values(value))freeze(child);
  Object.freeze(value);
 }
 return value;
}
const strings=(value:unknown)=>Array.isArray(value)&&value.every(v=>typeof v==='string');
function presentation(value:unknown):value is PetPresentation {
 if(!object(value))return false;
 return ['galleryAvatarSize','overviewAvatarSize'].every(k=>Number.isInteger(value[k])&&Number(value[k])>=16&&Number(value[k])<=128)
  &&['galleryCardRadius','galleryCardGap'].every(k=>Number.isInteger(value[k])&&Number(value[k])>=0&&Number(value[k])<=64)
  &&['profiles','sessions','metrics'].every(k=>typeof value[k]==='boolean')
  &&typeof value.contextAction==='string'&&['open','sessions'].includes(value.contextAction)
  &&typeof value.galleryAction==='string'&&typeof value.musicLabel==='string'
  &&[value.galleryState,value.accessibilityRole].every(v=>v===null||typeof v==='string')
  &&strings(value.permissions)&&(value.permissions as string[]).every(k=>['documents','emailDrafts','music','openApps'].includes(k));
}
export function createPetFactory(input:unknown){
 if(!object(input)||input.schemaVersion!==1||!object(input.configuration)||!object(input.base)||!object(input.molds)||!object(input.extensions)||!object(input.pets))invalid();
 const source=structuredClone(input) as Row;
 const molds=source.molds as Row,extensions=source.extensions as Row,pets=source.pets as Row;
 if(Object.keys(molds).length!==2||!Object.hasOwn(molds,'llm')||!Object.hasOwn(molds,'application'))invalid();
 // Stable identity and access policy are always explicit for each pet.
 const protectedKeys=['name','aliases','provider','permissions','localExecution'];
 for(const recipe of [source.base,...Object.values(molds),...Object.values(extensions)]){
  if(!object(recipe))invalid();
  if(recipe.preferences!==undefined&&(!object(recipe.preferences)||protectedKeys.some(k=>Object.hasOwn(recipe.preferences as Row,k))))invalid();
 }
 const entries=new Map<string,PetBlueprint>();
 const routeOwners=new Map<string,string>();
 for(const[id,raw]of Object.entries(pets)){
  if(!object(raw)||(raw.mold!=='llm'&&raw.mold!=='application')||typeof raw.service!=='boolean'||!object(raw.preferences)||!object(raw.routing))invalid();
  if(!['name','aliases','provider','permissions'].every(k=>Object.hasOwn(raw.preferences as Row,k)))invalid();
  const mold=molds[raw.mold];if(!object(mold))invalid();
  let effective=merge(source.base as Row,mold);
  if(raw.extension!==undefined){
   if(typeof raw.extension!=='string'||!Object.hasOwn(extensions,raw.extension)||!object(extensions[raw.extension]))invalid();
   effective=merge(effective,extensions[raw.extension] as Row);
  }
  effective=merge(effective,{preferences:raw.preferences,...(raw.presentation===undefined?{}:{presentation:raw.presentation})});
  const p=effective.preferences,r=raw.routing;
  if(!object(p)||typeof p.name!=='string'||!strings(p.aliases)||typeof p.provider!=='string'||typeof p.mode!=='string'||!['auto','fast','reasoning'].includes(p.mode)
   ||!['fast','reasoning'].every(k=>object(p[k])&&typeof (p[k] as Row).model==='string'&&typeof (p[k] as Row).effort==='string')
   ||!object(p.permissions)||!['documents','emailDrafts','music','openApps'].every(k=>typeof (p.permissions as Row)[k]==='boolean')
   ||!presentation(effective.presentation)||!['chat','voice','taskProvider','role','taskName'].every(k=>r[k]===null||typeof r[k]==='string')||typeof r.execute!=='boolean')invalid();
  // Logical owners are unique even when service adapters share a Codex account.
  for(const channel of ['chat','voice','taskProvider'] as const){
   const route=r[channel];if(route===null)continue;
   if(typeof route!=='string'||!route||routeOwners.has(`${channel}:${route}`))invalid();
   routeOwners.set(`${channel}:${route}`,id);
  }
  entries.set(id,freeze({id,mold:raw.mold,service:raw.service,preferences:p,presentation:effective.presentation,routing:r}) as unknown as PetBlueprint);
 }
 if(entries.size===0)invalid();
 const services=freeze([...entries.values()].filter(p=>p.service).map(p=>p.id));
 return Object.freeze({
  get:(id:string):PetBlueprint|null=>entries.get(id)??null,
  services,
  configuration:():PetConfiguration=>structuredClone({...source.configuration as Row,pets:Object.fromEntries([...entries].map(([id,p])=>[id,p.preferences]))}) as unknown as PetConfiguration,
  forTaskProvider:(provider:string)=>typeof provider==='string'&&provider?[...entries.values()].find(p=>p.routing.taskProvider===provider)?.id??null:null,
  profileProvider:(provider:string)=>typeof provider==='string'&&!!provider&&[...entries.values()].some(p=>p.routing.taskProvider===provider&&p.presentation.profiles),
 });
}
export const PetFactory=createPetFactory(catalog);
export const petBlueprint=(id:string)=>PetFactory.get(id);
export const petForTaskProvider=(provider:string)=>PetFactory.forTaskProvider(provider);
export const taskProviderForPet=(id:string)=>petBlueprint(id)?.routing.taskProvider??null;
export const factoryProfileProvider=(provider:string)=>PetFactory.profileProvider(provider);
