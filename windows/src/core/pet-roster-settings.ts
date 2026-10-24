import type {Declared} from './pills';

export type PetRosterRequest={action:'setMain';id:string}|{action:'setActive';id:string;enabled:boolean};
export type PetRosterValues=Declared;
function values(input:unknown):PetRosterValues{
 if(!input||typeof input!=='object')throw Error('Preferências de pets indisponíveis.');
 const value=input as Partial<PetRosterValues>;
 if(typeof value.mainPill!=='string'||!Array.isArray(value.activeIntegrations)||!value.activeIntegrations.every(id=>typeof id==='string'))throw Error('Preferências de pets indisponíveis.');
 return {mainPill:value.mainPill,activeIntegrations:[...value.activeIntegrations]};
}

/** Save one explicit intention. Native events observed during a request win
 * over its delayed response. Only the two owned fields are adopted; general
 * settings and another pet's visibility are never copied into the request. */
export class PetRosterPreferences{
 private observation=0;
 private queue:Promise<unknown>=Promise.resolve();
 constructor(private options:{save(request:PetRosterRequest):Promise<unknown>;adopt(roster:PetRosterValues):void}){}
 observe(input:unknown){this.observation++;this.options.adopt(values(input));}
 change(request:PetRosterRequest):Promise<PetRosterValues>{
  const captured=Object.freeze({...request});
  const pending=this.queue.then(async()=>{
   const observation=this.observation;
   const committed=values(await this.options.save(captured));
   if(observation===this.observation)this.options.adopt({...committed,activeIntegrations:[...committed.activeIntegrations]});
   return committed;
  });
  this.queue=pending.catch(()=>{});
  return pending;
 }
}
