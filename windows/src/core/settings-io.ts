export type SettingsIoPhase='ready'|'writing'|'changed'|'readUnavailable'|'reviewRequired'|'publicationUnconfirmed'|'unbound'|'unavailable'|'platformNotTracked';
export interface SettingsIoStatus {schemaVersion:1;phase:SettingsIoPhase;newEffectsAllowed:boolean;message:string;suspendedOperations?:string[]}
const phases:readonly string[]=['ready','writing','changed','readUnavailable','reviewRequired','publicationUnconfirmed','unbound','unavailable','platformNotTracked'];
/** Read-only public metadata; neither a ready badge nor JSON grants execution. */
export function settingsIoStatus(value:unknown):SettingsIoStatus{
 if(!value||typeof value!=='object'||Array.isArray(value))throw Error('O estado das preferências não foi confirmado.');
 const v=value as Record<string,unknown>;
 if(v.schemaVersion!==1||typeof v.phase!=='string'||!phases.includes(v.phase)||typeof v.newEffectsAllowed!=='boolean'||v.newEffectsAllowed!==(v.phase==='ready')||typeof v.message!=='string'||v.message.length>1024)throw Error('O estado das preferências não foi confirmado.');
 const pending=v.suspendedOperations;
 if(pending!==undefined&&(!Array.isArray(pending)||pending.length>6||new Set(pending).size!==pending.length||pending.some(s=>typeof s!=='string'||!['voice','mobile','autostart','shortcuts','hooks','credentials'].includes(s))))throw Error('As componentes pendentes não foram confirmadas.');
 return {schemaVersion:1,phase:v.phase as SettingsIoPhase,newEffectsAllowed:v.newEffectsAllowed,message:v.message,...(pending!==undefined?{suspendedOperations:[...pending as string[]]}:{})};
}
