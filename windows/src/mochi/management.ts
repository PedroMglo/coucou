/** Metadata supplied by the workstation helper. No chat items or credentials. */
export interface ManagedSession {provider:string;id:string;name:string;cwd:string;model:string;state:string;parent:string;updated:number;archived:boolean;can_resume?:boolean;agents_active?:number;agents_total?:number;agents_exact?:boolean;tokens?:number}
export interface ManagedProvider {id:string;name:string;installed:boolean;available:boolean;active:number;agents:number;agents_exact:boolean;history:number|null;detail:string}
export interface ManagedModel {name:string;digest:string;loaded:boolean;chat:boolean;embedding:boolean;capabilities_known?:boolean;parameters:string;quantization:string;size:number;memory:number;vram:number;clients:{provider:string;id:string;name:string;state:string}[];aliases?:string[]}
export interface ManagedInventory {updated:number;providers:ManagedProvider[];sessions:ManagedSession[];models:ManagedModel[];resources:Record<string,number|null>;summary:Record<string,number|null>;notes:string[]}
export interface ManagedQuota {name?:string;account?:string;plan?:string;unlimited?:string[];note?:string;stale?:boolean;blocked?:boolean;updated?:number;windows?:{label?:string;name?:string;used:number;reset?:number|string;reset_at?:number;reset_label?:string;additional?:boolean;entitlement?:number;duration?:number;remaining?:number}[]}
export interface ManagedUsage {providers:Record<string,ManagedQuota>}
export interface ManagedDetail {kind:string;provider:string;id:string;title:string;fields:{label:string;value?:string;bytes?:number}[];cwd?:string;editable?:boolean;project_config?:string;chat?:boolean;loaded?:boolean;archived?:boolean;clients?:{provider:string;id:string;name:string;state:string}[];children?:{provider?:string;id:string;name:string;state:string}[]}
export interface ManagedResult {message?:string;detail?:ManagedDetail}
export const activeSession=(row:ManagedSession)=>['working','attention','idle','running'].includes(row.state);
export function matchesSearch(query:string,...values:(string|undefined)[]){
 const fold=(value:string)=>value.normalize('NFD').replace(/\p{M}/gu,'').toLowerCase();
 const text=fold(values.filter(Boolean).join(' '));return fold(query).trim().split(/\s+/).filter(Boolean).every(word=>text.includes(word));
}
export const bytesLabel=(n:number|null|undefined)=>n==null?'—':n<1024?`${n} B`:`${(n/1024**Math.min(4,Math.floor(Math.log(n)/Math.log(1024)))).toFixed(1)} ${['B','KiB','MiB','GiB','TiB'][Math.min(4,Math.floor(Math.log(n)/Math.log(1024)))]}`;
export function uniqueManagedModels(models:ManagedModel[]){const unique=new Map<string,ManagedModel>();for(const m of models){const key=m.digest||m.name,old=unique.get(key);if(old)old.aliases!.push(m.name);else unique.set(key,{...m,aliases:[],clients:m.clients??[]});}return [...unique.values()].sort((a,b)=>Number(b.clients.length>0)-Number(a.clients.length>0)||Number(b.loaded)-Number(a.loaded)||a.name.localeCompare(b.name));}
export type SessionEntry={kind:'provider';provider:ManagedProvider;count:number;opened:boolean}|{kind:'session';row:ManagedSession;depth:number;children:number;opened:boolean;context:boolean};
export function sessionTree(sessions:ManagedSession[],providers:ManagedProvider[],filter:string,query:string,collapsed:Set<string>,expanded:Set<string>):SessionEntry[]{
 const result:SessionEntry[]=[],needle=query.trim().toLowerCase();
 for(const provider of providers){
  const rows=sessions.filter(s=>s.provider===provider.id),byId=new Map(rows.map(s=>[s.id,s])),children=new Map<string,ManagedSession[]>();
  for(const row of rows){if(!children.has(row.parent))children.set(row.parent,[]);children.get(row.parent)!.push(row);}
  const matches=(row:ManagedSession)=>(filter==='all'||filter==='active'&&activeSession(row)||filter==='saved'&&row.state==='saved'||filter==='archived'&&row.archived)&&matchesSearch(query,row.name,row.id,row.cwd,row.model,provider.name);
  const relevant=new Set<string>();for(const row of rows.filter(matches)){let item:ManagedSession|undefined=row;const seen=new Set<string>();while(item&&!seen.has(item.id)){seen.add(item.id);relevant.add(item.id);item=byId.get(item.parent);}}
  if(!relevant.size)continue;const opened=!!needle||!collapsed.has(provider.id);result.push({kind:'provider',provider,count:rows.filter(matches).length,opened});if(!opened)continue;
  const visited=new Set<string>();
  const visit=(row:ManagedSession,depth:number)=>{if(visited.has(row.id)||!relevant.has(row.id))return;visited.add(row.id);const descendants=(children.get(row.id)??[]).filter(s=>relevant.has(s.id)&&s.id!==row.id),opened=!!needle||expanded.has(`${provider.id}:${row.id}`);result.push({kind:'session',row,depth:Math.min(depth,6),children:descendants.length,opened,context:!matches(row)});if(opened)for(const child of descendants)visit(child,depth+1);};
  for(const row of rows)if(!row.parent||!byId.has(row.parent)||row.parent===row.id)visit(row,0);
  for(const row of rows){if(visited.has(row.id)||!relevant.has(row.id))continue;let parent=byId.get(row.parent);const seen=new Set([row.id]);let hidden=false;while(parent&&!seen.has(parent.id)){if(visited.has(parent.id)){hidden=true;break;}seen.add(parent.id);parent=byId.get(parent.parent);}if(!hidden)visit(row,0);}
 }
 return result;
}
/** A logical service owner does not identify its physical provider. Unknown
 * selections stay explicit rather than becoming Codex or All providers. */
export function inventoryProvider(pet:string,binding?:string|null):string {
 if(binding&&/^[a-z0-9_-]{1,64}$/.test(binding))return ({'gemini-cli':'gemini','claude-cli':'claude'} as Record<string,string>)[binding]??binding;
 return ({agent_codex:'codex',agent_copilot:'copilot',agent_antigravity:'antigravity',agent_gemini:'gemini',integration_claude:'claude',ai_ollama:'ollama',agent_opencode:'opencode',ai_lmstudio:'lmstudio'} as Record<string,string>)[pet]??pet;
}
