import {sessionLabel} from '../core/pet-sessions';
import {sessionOrigin,type SessionSummary,type SessionTarget} from '../core/session-chat';
import type {PetIdentity} from './companions';

type NativeSession=Readonly<NonNullable<PetIdentity['session']>>;
export interface NativeSpeech {
 readonly pet:string;readonly text:string;readonly at:number;readonly key:string;
 readonly origin:string;readonly session?:NativeSession;
}
export function nativeSessionOrigin(session?:NativeSession){
 return session?sessionLabel({sessionId:session.id,sessionCwd:session.cwd,sessionTitle:session.title,sessionOwned:session.owned}):'';
}
/** A decision and its CTA share one origin; older public speech cannot label it. */
export function petDialogueOrigin(attention:boolean,current:NativeSession|undefined,decision:SessionTarget|undefined,activity:string|undefined,speech:NativeSpeech|undefined){
 return attention?(decision?sessionOrigin(decision):nativeSessionOrigin(current)):(activity??speech?.origin??nativeSessionOrigin(current));
}
export class NativePetSpeech {
 readonly entries=new Map<string,NativeSpeech>();
 observe(pet:PetIdentity,now:number){
  if(!pet.message)return;
  const session=pet.session?Object.freeze({...pet.session}):undefined;
  const key=JSON.stringify([pet.message,pet.activity,session?.id,session?.backend,session?.owned]);
  if(this.entries.get(pet.id)?.key===key)return;
  this.entries.set(pet.id,Object.freeze({pet:pet.id,text:pet.message,at:now,key,origin:nativeSessionOrigin(session),session}));
 }
 current(entry:NativeSpeech,now:number){return this.entries.get(entry.pet)===entry&&now-entry.at<=9000;}
}
/** Native IDs are not Mochi conversation IDs. Only the backend's canonical
 * list may connect a captured source to an openable conversation. */
export function nativeSpeechTarget(speech:NativeSpeech,rows:readonly SessionSummary[]):Readonly<SessionTarget>|undefined {
 const source=speech.session;if(!source?.id||!source.backend||source.owned)return;
 const matches=rows.filter(({target:t})=>t.petId===speech.pet&&t.provider===source.backend&&t.sessionId===source.id&&t.origin==='external'&&t.conversationId===undefined);
 if(matches.length!==1)return;
 return Object.freeze({...matches[0].target});
}
