import type {AgentTask,ApprovalInfo} from './state';
import type {ManagedSession} from '../mochi/management';
import type {AssistantTask} from '../mochi/tasks';
export const petForBackend=(backend:string)=>({codex:'agent_codex',copilot:'agent_copilot',claude:'integration_claude',gemini:'agent_gemini',antigravity:'agent_antigravity',ollama:'ai_ollama',opencode:'agent_opencode'} as Record<string,string>)[backend];

/** One provider has one pet, but each native session has its own public state. */
export interface PetSession extends AgentTask {
  sessionId:string;
  sessionBackend:string;
  sessionTitle:string;
  sessionOwned:boolean;
  updated:number;
  ended:boolean;
  decisionInTerminal?:boolean;
  hookAt?:number;
}
export const sessionKey=(pet:string,backend:string,id:string)=>JSON.stringify([pet,backend,id]);
export function sessionLabel(s:{sessionId?:string|null;sessionTitle?:string;sessionOwned?:boolean;sessionCwd?:string|null}) {
  const folder=(s.sessionCwd??'').replace(/[\\/]+$/,'').split(/[\\/]/).pop();
  return `${s.sessionOwned?'Mochi':'Terminal'}${s.sessionId?' #'+s.sessionId.slice(0,8):''} · ${s.sessionTitle||folder||'Sessão'}`;
}
export class PetSessions {
  readonly entries=new Map<string,PetSession>();
  private selected=new Map<string,string>();
  observe(base:AgentTask,backend:string,id:string,cwd:string,owned:boolean,now=Date.now()):PetSession {
    const key=sessionKey(base.id,backend,id);let s=this.entries.get(key);
    if(!s){
      s={...base,steps:[],stepIndex:0,stepSeq:0,state:'idle',pillBadge:null,finalLine:null,
        sessionId:id,sessionBackend:backend,sessionCwd:cwd,sessionTitle:'',sessionOwned:owned,decisionInTerminal:false,updated:now,ended:false};
      this.entries.set(key,s);
    }
    if(cwd)s.sessionCwd=cwd;
    s.sessionOwned=owned;s.updated=now;s.hookAt=now;s.ended=false;
    // Retain active sessions. Only old finished sessions may be evicted.
    if(this.entries.size>128)for(const [k,v]of this.entries){if(this.entries.size<=128)break;if(v.ended)this.entries.delete(k);}
    return s;
  }
  merge(rows:ManagedSession[],tasks:AssistantTask[],base:(pet:string)=>AgentTask|undefined,now=Date.now()) {
    for(const row of rows){
      const job=tasks.find(t=>t.sessionId===row.id&&t.backend===row.provider);
      const pet=job?(petForBackend(job.provider)??(job.provider.startsWith('integration_')?job.provider:undefined)):petForBackend(row.provider);
      const owner=pet?base(pet):undefined;if(!owner)continue;
      const key=sessionKey(owner.id,row.provider,row.id),before=this.entries.get(key);
      const active=['working','attention','idle','running'].includes(row.state);
      if(!active&&!before)continue;
      const s=before??this.observe(owner,row.provider,row.id,row.cwd,!!job,now);
      s.sessionCwd=row.cwd||s.sessionCwd;s.sessionTitle=row.name||s.sessionTitle;
      if(job)s.sessionOwned=true;
      if(!before||!s.hookAt||now-s.hookAt>30000){
        s.ended=!active;s.state=row.state==='working'?'working':row.state==='attention'?'approval':'idle';
      }
      if(!before)s.hookAt=undefined;
    }
  }
  list(pet:string):PetSession[]{return [...this.entries.values()].filter(s=>s.id===pet&&!s.ended).sort((a,b)=>b.updated-a.updated);}
  choose(pet:string,backend:string,id:string):boolean {
    const key=sessionKey(pet,backend,id),s=this.entries.get(key);if(!s||s.ended)return false;
    this.selected.set(pet,key);return true;
  }
  current(pet:string,pending:ApprovalInfo|null):PetSession|undefined {
    const list=this.list(pet);
    // The decision always names its own immutable origin, even if another
    // session produces a message while the card is open.
    const approval=pending?.pillId===pet?list.find(s=>s.sessionId===pending.sessionId&&(!pending.backend||s.sessionBackend===pending.backend)):undefined;
    if(approval)return approval;
    const selected=this.selected.get(pet),chosen=selected?this.entries.get(selected):undefined;
    if(chosen&&!chosen.ended)return chosen;
    const priority=(s:PetSession)=>['approval','question','error','ratelimit'].includes(s.state)?3:['thinking','working','searching'].includes(s.state)?2:s.state==='finished'?1:0;
    return list.sort((a,b)=>priority(b)-priority(a)||b.updated-a.updated)[0];
  }
  project(task:AgentTask,pending:ApprovalInfo|null):boolean {
    const s=this.current(task.id,pending);
    if(s){Object.assign(task,s,{steps:[...s.steps]});return true;}
    return false;
  }
}
