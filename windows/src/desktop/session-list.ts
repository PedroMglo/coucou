import {sameSession,sessionTargetKey,type SessionSummary,type SessionTarget} from '../core/session-chat';

const fold=(text:string)=>text.normalize('NFD').replace(/\p{M}/gu,'').toLocaleLowerCase();
/** Search the public session labels, keeping each result tied to its origin. */
export function matchingSessions(summaries:SessionSummary[],query:string,origin:SessionTarget['origin']){
 const terms=fold(query.trim()).split(/\s+/).filter(Boolean);
 return summaries.filter(s=>s.target.origin===origin&&terms.every(term=>fold([s.title,s.cwd,s.target.sessionId,s.model??''].join(' ')).includes(term)))
  .sort((a,b)=>b.updatedAt-a.updatedAt||a.title.localeCompare(b.title));
}
/** A failed refresh keeps the last known list rather than inventing an empty one. */
export function mergeSessionLists(previous:SessionSummary[],own:SessionSummary[]|null,external:SessionSummary[]|null){
 const all=[...(own??previous.filter(s=>s.target.origin==='mochi')),...(external??previous.filter(s=>s.target.origin==='external'))];
 const seen=new Set<string>();return all.filter(s=>{const key=sessionTargetKey(s.target);if(seen.has(key))return false;seen.add(key);return true;});
}
/** Keep selection visible without rendering an entire terminal archive at once. */
export function visibleSessions(rows:SessionSummary[],limit:number,selected:SessionTarget|null){
 const shown=rows.slice(0,Math.max(1,limit)),current=rows.find(s=>sameSession(s.target,selected));
 if(current&&!shown.includes(current))return [current,...shown.slice(0,-1)];return shown;
}
