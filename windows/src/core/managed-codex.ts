import {sameSession,type SessionTarget} from './session-chat';

export interface ManagedCodexSession {
 target:SessionTarget;handle:string;generation:number;
 phase:'draft'|'ready'|'active'|'needsNativeClient'|'lost'|'released';
 turn:string|null;materialized:boolean;terminalLaunched:boolean;
 cwd:string;model:string|null;notice:string|null;
}
/** Only a native creation receipt grants these actions. An ordinary external
 * session ID cannot be promoted to a managed session by the frontend. */
export function managedReference(receipt:ManagedCodexSession){
 return {target:{...receipt.target},handle:receipt.handle,generation:receipt.generation};
}
export function managedFor(rows:ManagedCodexSession[],target:SessionTarget|null){
 return rows.find(row=>sameSession(row.target,target)&&row.phase!=='released');
}
export function managedDescription(receipt:ManagedCodexSession){
 if(receipt.phase==='lost')return 'A ligação original terminou. Esta sessão já não é gerida pelo Coucou.';
 if(receipt.phase==='needsNativeClient')return 'Esta sessão precisa de atenção no Codex nativo.';
 if(!receipt.materialized)return 'Sessão Codex criada. Escreve a primeira mensagem para poder abri-la no terminal.';
 return receipt.terminalLaunched?'Foi pedida a abertura no terminal. Podes continuar a conversa aqui.':'A mesma conversa pode continuar aqui ou no terminal.';
}
export const canOpenManaged=(receipt:ManagedCodexSession)=>receipt.materialized&&!['lost','released'].includes(receipt.phase);
