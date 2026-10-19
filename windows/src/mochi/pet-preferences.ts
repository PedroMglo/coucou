import {Bridge,onEvent} from '../core/bridge';
export interface ModelProfile {model:string;effort:string}
export interface PetPermissions {documents:boolean;emailDrafts:boolean;music:boolean;openApps:boolean}
export interface PetPreference {name:string;aliases:string[];provider:string;mode:'auto'|'fast'|'reasoning';fast:ModelProfile;reasoning:ModelProfile;permissions?:PetPermissions}
export interface PetConfiguration {version:number;reasoningTerms:string[];reasoningLength:number;pets:Record<string,PetPreference>}
export interface PetDocument {config:PetConfiguration;path:string;revision:string}
export const SERVICE_PETS=["integration_resend", "integration_n8n", "integration_vercel", "integration_github", "integration_notion", "integration_calcom", "integration_stripe", "integration_spotify"] as const;
export type ServiceProvider=typeof SERVICE_PETS[number];
export const isServiceProvider=(id:string):id is ServiceProvider=>(SERVICE_PETS as readonly string[]).includes(id);
export const profileModels=[{id:'@default',label:'Perfil definido para o pet'},{id:'@auto',label:'Automático · rápido ou raciocínio'},{id:'@fast',label:'Rápido · conversa e dia a dia'},{id:'@reasoning',label:'Raciocínio elevado'}];
export const profileProvider=(provider:string)=>isServiceProvider(provider) || ['codex','copilot','antigravity','gemini-cli','claude-cli','ollama'].includes(provider);
let config:PetConfiguration|null=null,initialized=false;
let roster:readonly string[]|null=null;
const listeners=new Set<()=>void>();
export const petName=(id:string,fallback:string)=>config?.pets[id]?.name??fallback;
export const profileLabel=(model:string)=>profileModels.find(m=>m.id===model)?.label??model;
export const PetPreferences={
 get config(){return config;},
 get roster(){return roster;},
 active(id:string){return roster?.includes(id)??false;},
 subscribe(fn:()=>void){listeners.add(fn);return()=>listeners.delete(fn);},
 update(value:PetConfiguration){config=value;for(const listener of listeners)listener();},
 async init(){
  if(!initialized){initialized=true;await onEvent<PetConfiguration>('pet-configuration-changed',value=>this.update(value));await onEvent('settings-changed',async()=>{roster=await Bridge.petRoster();if(config)this.update(config);});}
  try{const document=await Bridge.petConfiguration();roster=await Bridge.petRoster();if(document)this.update(document.config);return document;}catch{return null;}
 },
};

/** Show the actual profile models/effort, including when automatic selection is enabled. */
export function profileSummary(provider:string,selected:string){
 if(!selected.startsWith('@'))return selected;
 const id=({codex:'agent_codex',copilot:'agent_copilot',antigravity:'agent_antigravity','gemini-cli':'agent_gemini','claude-cli':'integration_claude',ollama:'ai_ollama'} as Record<string,string>)[provider]??provider;
 const p=config?.pets[id];if(!p)return profileLabel(selected);
 const mode=selected==='@default'?p.mode:selected.slice(1);
 const describe=(profile:ModelProfile)=>[profile.model,profile.effort].filter(Boolean).join(' · ');
 return mode==='fast'?`Rápido · ${describe(p.fast)}`:mode==='reasoning'?`Raciocínio · ${describe(p.reasoning)}`:`Automático · ${describe(p.fast)} / ${describe(p.reasoning)}`;
}
