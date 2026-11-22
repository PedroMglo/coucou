const invalid=()=>Error('A revisão nativa dos atalhos não foi confirmada.');
function obj(v:unknown):Record<string,unknown>{if(!v||typeof v!=='object'||Array.isArray(v))throw invalid();return v as Record<string,unknown>;}
function str(v:unknown,max=1024):string{if(typeof v!=='string'||v.length>max)throw invalid();return v;}
function hex(v:unknown,n=64):string{const s=str(v,n);if(s.length!==n||!/^[0-9a-f]+$/.test(s))throw invalid();return s;}
function nullable(v:unknown,max=1024):string|null{return v===null?null:str(v,max);}
export interface ShortcutNativeProcessIdentity{pid:number;uid:number;startTicks:string;bootId:string;namespaceDevice:string;namespaceInode:string}
export interface ShortcutNativeEnvironment{display:string|null;waylandDisplay:string|null;sessionType:string|null}
export function parseShortcutNativeProcessIdentity(value:unknown):ShortcutNativeProcessIdentity{
 const owner=obj(value);for(const key of ['pid','uid'])if(!Number.isSafeInteger(owner[key])||Number(owner[key])<0||Number(owner[key])>0xffffffff||key==='pid'&&owner[key]===0)throw invalid();
 for(const key of ['startTicks','namespaceDevice','namespaceInode']){const n=str(owner[key],20);if(!/^(0|[1-9][0-9]*)$/.test(n)||BigInt(n)>18446744073709551615n||key!=='namespaceDevice'&&BigInt(n)===0n)throw invalid();}
 if(!/^[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}$/.test(str(owner.bootId,36)))throw invalid();
 return {pid:Number(owner.pid),uid:Number(owner.uid),startTicks:String(owner.startTicks),bootId:String(owner.bootId),namespaceDevice:String(owner.namespaceDevice),namespaceInode:String(owner.namespaceInode)};
}
export function parseShortcutNativeEnvironment(value:unknown):ShortcutNativeEnvironment{
 const environment=obj(value);for(const key of ['display','waylandDisplay','sessionType']){const text=nullable(environment[key],256);if(text!==null&&/[\x00-\x1f\x7f]/.test(text))throw invalid();}
 return {display:environment.display as string|null,waylandDisplay:environment.waylandDisplay as string|null,sessionType:environment.sessionType as string|null};
}
export interface ShortcutNativeSubject{instance:string;owner:ShortcutNativeProcessIdentity|null;environment:ShortcutNativeEnvironment;kind:'plugin'|'portal';portal:{busId:string;peer:string;client:string;sessions:string[]}|null}
export function parseShortcutNativeSubject(value:unknown):ShortcutNativeSubject{
 const v=obj(value);if(v.kind!=='plugin'&&v.kind!=='portal')throw invalid();const instance=hex(v.instance,32),owner=v.owner===null?null:parseShortcutNativeProcessIdentity(v.owner),environment=parseShortcutNativeEnvironment(v.environment);let portal:ShortcutNativeSubject['portal']=null;
 if(v.portal!==null){const p=obj(v.portal);if(v.kind!=='portal'||!/^:[0-9]+\.[0-9]+$/.test(str(p.peer,128))||!/^:[0-9]+\.[0-9]+$/.test(str(p.client,128))||!Array.isArray(p.sessions)||p.sessions.length>2)throw invalid();const prefix=`/org/freedesktop/portal/desktop/session/${String(p.client).slice(1).replaceAll('.','_')}/`;const sessions=p.sessions.map(s=>{const path=str(s,512);if(!/^\/[A-Za-z0-9_/]+$/.test(path)||!path.startsWith(prefix))throw invalid();return path;});if(new Set(sessions).size!==sessions.length)throw invalid();portal={busId:hex(p.busId,32),peer:String(p.peer),client:String(p.client),sessions};}
 return {instance,owner,environment,kind:v.kind,portal};
}
export function parseShortcutEffectSlot(value:unknown):Record<string,unknown>{
 const v=obj(value);if(v.component!=='shortcuts'||!['prepared','dispatched','verified','unconfirmed','notDispatched','reconciledObservation'].includes(str(v.phase,32)))throw invalid();
 const b=obj(v.binding),binding:Record<string,string>={};for(const key of ['profile','configuration','resource','plan'])binding[key]=hex(b[key]);
 const ref=(value:unknown)=>{const r=obj(value);return {id:hex(r.id,32),sha256:hex(r.sha256)};};const intent=ref(v.intent),outcome=v.outcome===undefined?undefined:ref(v.outcome);
 if((v.phase==='prepared'||v.phase==='dispatched')===(outcome!==undefined))throw invalid();
 return {id:hex(v.id,32),component:'shortcuts',phase:v.phase,binding,owner:hex(v.owner),intent,...(outcome===undefined?{}:{outcome})};
}
