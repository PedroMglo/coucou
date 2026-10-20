/** Bounded view of workstation quotas. Pure parsing: no provider/cache IO. */
export interface ManagedUsageWindow {
 label:string;used:number|null;resetAt:number|null;resetLabel:string|null;
 remaining:number|null;entitlement:number|null;additional:boolean;
}
export interface ManagedUsageQuota {
 account?:string;plan?:string;note?:string;unlimited:string[];updated:number|null;
 stale:boolean;blocked:boolean;windows:ManagedUsageWindow[]|null;partial:boolean;
}
export interface ManagedUsageView {providers:Record<string,ManagedUsageQuota>;partial:boolean}
const object=(value:unknown):Record<string,unknown>|null=>value!==null&&typeof value==='object'&&!Array.isArray(value)?value as Record<string,unknown>:null;
const text=(value:unknown,max=128):string|undefined=>typeof value==='string'&&value.trim().length>0&&value.length<=max&&!/[\u0000-\u001f\u007f]/.test(value)?value.trim():undefined;
const number=(value:unknown,max=Number.MAX_SAFE_INTEGER):number|null=>typeof value==='number'&&Number.isFinite(value)&&value>=0&&value<=max?value:null;
/** Native helper numeric dates are Unix seconds; null/invalid dates stay absent. */
export function managedUsageDate(value:unknown):number|null {
 if(typeof value==='number')return Number.isSafeInteger(value)&&value>0&&value<=8_640_000_000_000?value*1000:null;
 if(typeof value!=='string'||value.length>64||!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?(?:Z|[+-]\d{2}:\d{2})$/.test(value))return null;
 const stamp=Date.parse(value);return Number.isFinite(stamp)?stamp:null;
}
export function parseManagedUsage(value:unknown):ManagedUsageView {
 const source=object(value),providers=object(source?.providers);
 if(!providers)throw new Error('Os dados de consumo não estão disponíveis.');
 const result:ManagedUsageView={providers:{},partial:false};
 const entries=Object.entries(providers);if(entries.length>32)result.partial=true;
 for(const [id,input]of entries.slice(0,32)){
  if(!/^[a-z0-9_-]{1,64}$/.test(id)||['__proto__','constructor','prototype'].includes(id)){result.partial=true;continue;}
  const q=object(input);if(!q){result.partial=true;continue;}
  const updated=managedUsageDate(q.updated),quota:ManagedUsageQuota={updated,stale:q.stale===true||updated===null,blocked:q.blocked===true,unlimited:[],windows:null,partial:false};
  for(const key of ['account','plan','note'] as const){const v=text(q[key],key==='note'?500:128);if(v)quota[key]=v;}
  if(Array.isArray(q.unlimited)){quota.unlimited=q.unlimited.slice(0,16).map(v=>text(v)).filter((v):v is string=>!!v);if(quota.unlimited.length!==q.unlimited.length)quota.partial=true;}
  if(Array.isArray(q.windows)){
   quota.windows=[];if(q.windows.length>32)quota.partial=true;
   for(const input of q.windows.slice(0,32)){
    const w=object(input),used=number(w?.used,100),resetValue=w?.reset_at??w?.reset,resetAt=managedUsageDate(resetValue),resetLabel=text(w?.reset_label)??null;
    if(!w||used===null||resetValue!=null&&resetAt===null)quota.partial=true;
    quota.windows.push({label:text(w?.label)??text(w?.name)??'Utilização',used,resetAt,resetLabel,remaining:number(w?.remaining),entitlement:number(w?.entitlement),additional:w?.additional===true});
   }
  }else quota.partial=true;
  result.providers[id]=quota;result.partial ||=quota.partial;
 }
 return result;
}
