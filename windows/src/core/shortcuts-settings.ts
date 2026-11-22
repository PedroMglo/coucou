import type {Binding} from './shortcuts';
import {SHORTCUTS} from './shortcuts';
import {settingsIoStatus,type SettingsIoStatus} from './settings-io';
export type ShortcutBindings=Record<string,Binding>;
export interface ShortcutPatch{changes:Record<string,{before:Binding|null;after:Binding|null}>}
export interface ShortcutPatchReceipt{schemaVersion:1;bindings:ShortcutBindings;changed:boolean;newRegistrationRequiresReview:boolean;status:SettingsIoStatus}
export function parseShortcutPatchReceipt(value:unknown):ShortcutPatchReceipt{
 if(!value||typeof value!=='object'||Array.isArray(value))throw Error('As preferências dos atalhos não foram confirmadas.');const v=value as Record<string,unknown>;
 if(v.schemaVersion!==1||typeof v.changed!=='boolean'||typeof v.newRegistrationRequiresReview!=='boolean'||!v.bindings||typeof v.bindings!=='object'||Array.isArray(v.bindings)||Object.keys(v.bindings).length>64)throw Error('Resposta de atalhos inválida.');
 const bindings:ShortcutBindings=Object.create(null);
 for(const[id,raw]of Object.entries(v.bindings)){if(id.length>128||!raw||typeof raw!=='object'||Array.isArray(raw))throw Error('Atalhos inválidos.');const b=raw as Record<string,unknown>;if(typeof b.keys!=='string'||b.keys.length>256||typeof b.enabled!=='boolean')throw Error('Atalho inválido.');bindings[id]={keys:b.keys,enabled:b.enabled};}
 const status=settingsIoStatus(v.status);if(v.changed&&(!v.newRegistrationRequiresReview||!status.suspendedOperations?.includes('shortcuts')))throw Error('A pausa do novo registo não foi confirmada.');
 return {schemaVersion:1,bindings,changed:v.changed,newRegistrationRequiresReview:v.newRegistrationRequiresReview,status};
}
export class ShortcutPreferenceWriter{
 busy=false;
 constructor(private actions:{current():ShortcutBindings;observation():number;alive():boolean;save(request:ShortcutPatch):Promise<ShortcutPatchReceipt>;adopt(bindings:ShortcutBindings):void}){}
 async change(id:string,after:Binding|null){if(!SHORTCUTS.some(d=>d.id===id))throw Error('Atalho desconhecido.');await this.write({[id]:after});}
 async reset(){const current=this.actions.current(),changes:Record<string,null>={};for(const d of SHORTCUTS)if(Object.hasOwn(current,d.id))changes[d.id]=null;if(Object.keys(changes).length)await this.write(changes);}
 private async write(changes:Record<string,Binding|null>){
  if(this.busy)throw Error('A gravação anterior dos atalhos ainda está em curso.');if(!this.actions.alive())throw Error('A janela dos atalhos foi fechada.');
  const observation=this.actions.observation(),current=this.actions.current(),request:ShortcutPatch={changes:{}};
  const copy=(v:Binding|null):Binding|null=>v===null?null:{keys:v.keys,enabled:v.enabled};
  for(const[id,after]of Object.entries(changes))request.changes[id]={before:Object.hasOwn(current,id)?copy(current[id]):null,after:copy(after)};
  this.busy=true;try{const receipt=await this.actions.save(request);if(!receipt.status.newEffectsAllowed)throw Error('A origem das preferências mudou após a gravação. Consulta o estado antes de repetir.');if(this.actions.alive()&&this.actions.observation()===observation)this.actions.adopt(receipt.bindings);}finally{this.busy=false;}
 }
}
