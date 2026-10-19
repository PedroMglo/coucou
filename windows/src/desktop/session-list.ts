import {sameSession,type SessionSummary,type SessionTarget} from '../core/session-chat';

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
 return all.filter((s,i)=>all.findIndex(other=>sameSession(s.target,other.target))===i);
}
