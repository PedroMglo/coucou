/** Pure policy over already collected usage. Quotas never alter task/approval state. */
import type {ManagedUsage} from './management';
import type {PetNotice} from './notices';

export interface PetQuota {
  provider:string;
  status:'ok'|'warning'|'blocked'|'stale'|'unknown';
  text:string;
  usedPercent?:number;
  /** Epoch milliseconds; the workstation helper supplies epoch seconds. */
  updatedAt?:number;
  resetsAt?:number;
}

const MAX_AGE=5*60_000;
const COOLDOWN=15*60_000;
const ACCOUNTS=[
  {provider:'codex',pet:'agent_codex',name:'Codex'},
  {provider:'copilot',pet:'agent_copilot',name:'Copilot'},
  {provider:'claude',pet:'integration_claude',name:'Claude'},
  {provider:'antigravity',pet:'agent_antigravity',name:'Antigravity'},
] as const;
// These services use the Codex account. Music-only pets and arbitrary
// integration_* names do not inherit a quota merely through their prefix.
const CODEX_SERVICES=new Set([
  'integration_resend','integration_n8n','integration_vercel','integration_github',
  'integration_notion','integration_calcom','integration_stripe','integration_spotify',
]);
type Account=typeof ACCOUNTS[number];
type Status=PetQuota['status'];
type RecordValue=Record<string,unknown>;
const object=(value:unknown):RecordValue|undefined=>value!==null&&typeof value==='object'&&!Array.isArray(value)?value as RecordValue:undefined;
const finite=(value:unknown):value is number=>typeof value==='number'&&Number.isFinite(value);
const own=(record:RecordValue|undefined,key:string):unknown=>record&&Object.prototype.hasOwnProperty.call(record,key)?record[key]:undefined;
const seconds=(value:unknown):number|undefined=>finite(value)&&value>0&&value<=8.64e12?value*1000:undefined;
function resetTime(value:unknown):number|undefined {
  if(typeof value==='number')return seconds(value);
  if(typeof value!=='string'||value.length>64||!/^\d{4}-\d\d-\d\dT\d\d:\d\d(?::\d\d(?:\.\d{1,9})?)?(?:Z|[+-]\d\d:\d\d)$/.test(value))return;
  const parsed=Date.parse(value);
  return Number.isFinite(parsed)&&parsed>0?parsed:undefined;
}
function accountForPet(pet:string):Account|undefined {
  return ACCOUNTS.find(account=>account.pet===pet)??(CODEX_SERVICES.has(pet)?ACCOUNTS[0]:undefined);
}

interface Window {used:number;reset?:number;key:string}
interface Reading {quota:PetQuota;windowKey:string}
function read(account:Account,usage:ManagedUsage|undefined,now:number):Reading {
  const raw=object(own(object(own(object(usage),'providers')),account.provider));
  const quota:PetQuota={provider:account.provider,status:'unknown',text:`${account.name}: utilização atual desconhecida.`};
  if(!raw||!finite(now)||now<=0)return {quota,windowKey:'unknown'};
  const updated=seconds(raw.updated);
  if(updated!==undefined&&updated<=now)quota.updatedAt=updated;
  const windows:Window[]=[];
  let expired=false,invalid=false;
  if(raw.windows!==undefined&&!Array.isArray(raw.windows))invalid=true;
  if(Array.isArray(raw.windows)) {
    if(raw.windows.length>32)invalid=true;
    for(const value of raw.windows.slice(0,32)) {
      const window=object(value);
      if(!window){invalid=true;continue;}
      const resetValue=window.reset_at??window.reset;
      const reset=resetValue===undefined?undefined:resetTime(resetValue);
      if(resetValue!==undefined&&reset===undefined){invalid=true;continue;}
      if(reset!==undefined&&reset<=now)expired=true;
      if(!finite(window.used)||window.used<0||window.used>100){invalid=true;continue;}
      const label=typeof window.label==='string'?window.label:typeof window.name==='string'?window.name:'';
      const duration=finite(window.duration)&&window.duration>=0?window.duration:null;
      // Poll time and percentage are deliberately absent from this identity.
      const key=JSON.stringify([label.slice(0,128),duration,reset??null]);
      windows.push({used:window.used,reset,key});
    }
  }
  const windowKey=JSON.stringify(windows.map(window=>window.key).sort());
  if(raw.stale===true||expired||(updated!==undefined&&(updated>now||now-updated>MAX_AGE))) {
    quota.status='stale';
    quota.text=`${account.name}: dados de utilização desatualizados; a disponibilidade atual é desconhecida.`;
    return {quota,windowKey};
  }
  // A fresh explicit flag can report a block even without a percentage. Neither
  // a full included window nor an unlimited model list proves account access.
  if(updated===undefined)return {quota,windowKey};
  if(raw.blocked===true) {
    quota.status='blocked';
    quota.text=`${account.name}: o fornecedor indica um bloqueio de utilização. Revê a quota da conta.`;
  }
  const dominant=windows.reduce<Window|undefined>((best,window)=>!best||window.used>best.used||(window.used===best.used&&(window.reset??Infinity)<(best.reset??Infinity))?window:best,undefined);
  if(dominant&&(quota.status==='blocked'||!invalid||dominant.used>=85)) {
    quota.usedPercent=dominant.used;
    if(dominant.reset!==undefined)quota.resetsAt=dominant.reset;
    if(quota.status!=='blocked') {
      quota.status=dominant.used>=85?'warning':'ok';
      const percent=Number(dominant.used.toFixed(1));
      quota.text=quota.status==='warning'
        ?`${account.name}: utilização medida em ${percent}%. Confirma a disponibilidade na conta; este valor não confirma um bloqueio.`
        :`${account.name}: utilização medida em ${percent}%.`;
    }
  } else if(quota.status!=='blocked'&&Array.isArray(raw.unlimited)&&raw.unlimited.some(model=>typeof model==='string'&&model.length>0)) {
    quota.text=`${account.name}: há modelos sem limite indicado; a disponibilidade dos restantes não está confirmada.`;
  }
  return {quota,windowKey};
}

export function quotaForPet(petId:string,usage:ManagedUsage|undefined,now=Date.now()):PetQuota|undefined {
  const account=accountForPet(petId);
  return account?read(account,usage,now).quota:undefined;
}

interface Episode {
  status:Status;
  windowKey:string;
  number:number;
  announced:Set<Status>;
  hasKnown:boolean;
  lastAt?:number;
  blockedBypassUsed:boolean;
}

/** One bounded episode per known account, irrespective of how many pets use it. */
export class QuotaNotices {
  private accounts=new Map<string,Episode>();
  observe(usage:ManagedUsage|undefined,configuredPets:ReadonlySet<string>,now=Date.now()):PetNotice[] {
    if(!finite(now)||now<=0)return [];
    const notices:PetNotice[]=[];
    for(const account of ACCOUNTS) {
      let pet:string|undefined=configuredPets.has(account.pet)?account.pet:undefined;
      if(!pet&&account.provider==='codex')for(const candidate of configuredPets)if(CODEX_SERVICES.has(candidate)){pet=candidate;break;}
      if(!pet)continue;
      const {quota,windowKey}=read(account,usage,now);
      let episode=this.accounts.get(account.provider);
      const before=episode?.status;
      if(!episode) {
        episode={status:quota.status,windowKey,number:1,announced:new Set(),hasKnown:false,blockedBypassUsed:false};
        this.accounts.set(account.provider,episode);
      } else if(episode.windowKey!==windowKey||(before==='ok'&&quota.status!=='ok')) {
        episode.number++;
        episode.windowKey=windowKey;
        episode.announced.clear();
      }
      episode.status=quota.status;
      const unavailable=quota.status==='unknown'||quota.status==='stale';
      if(!unavailable)episode.hasKnown=true;
      const actionable=quota.status==='warning'||quota.status==='blocked'||(unavailable&&episode.hasKnown);
      if(!actionable||episode.announced.has(quota.status))continue;
      const cooling=episode.lastAt!==undefined&&now-episode.lastAt<COOLDOWN;
      const escalation=quota.status==='blocked'&&before!==undefined&&before!=='blocked'&&!episode.blockedBypassUsed;
      if(cooling&&!escalation)continue;
      if(cooling)episode.blockedBypassUsed=true;
      else episode.blockedBypassUsed=quota.status==='blocked';
      episode.lastAt=now;
      episode.announced.add(quota.status);
      notices.push({key:`quota:${account.provider}:${episode.number}:${quota.status}:${windowKey}`,pet,text:quota.text,urgent:false,view:'overview'});
    }
    return notices;
  }
}
