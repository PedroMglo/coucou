import type {AssistantTask,LocalStepReceipt} from '../mochi/tasks';
import {h} from './dom';
const causes:Record<string,string>={userCancel:'Interrupção pedida por ti',approvalDeadline:'O pedido de aprovação expirou',taskDeadline:'O tempo da tarefa terminou',commandDeadline:'O tempo do comando terminou',outputBudget:'Limite de saída atingido',previousDenied:'Uma ação anterior foi recusada',policyRevoked:'A permissão de execução foi desativada',spawnError:'Não foi possível iniciar o processo',crashDuringStart:'Arranque interrompido; confirma os efeitos antes de retomar',cleanupUnconfirmed:'A limpeza do processo ainda não foi confirmada',processExit:'O processo terminou'};
export function stepStatus(step:LocalStepReceipt){return ({proposed:'À espera da tua decisão',starting:'A executar',exited:`Terminou${step.exitCode===undefined?'':` · código ${step.exitCode}`}`,denied:'Recusado',expired:'Aprovação expirada',failedToStart:'Falha no arranque',timedOut:'Tempo esgotado',outputLimit:'Limite de saída atingido',cancelled:'Interrompido',skipped:'Não executado',unknown:'Resultado por confirmar'} satisfies Record<LocalStepReceipt['state'],string>)[step.state]??'Estado por confirmar';}
/** Metadata receipts expose progress without publishing command arguments or output. */
export function taskExecution(){
 const summary=h('summary'),list=h('ol'),el=h('details',{class:'task-execution'},summary,list);el.hidden=true;
 const rows=new Map<string,{el:HTMLElement;title:HTMLElement;detail:HTMLElement}>();
 return {el,sync(task:AssistantTask){
  const steps=task.localExecution??[];el.hidden=!steps.length;summary.textContent=`Execução no computador · ${steps.length} ${steps.length===1?'passo':'passos'}`;
  const ids=new Set(steps.map(s=>s.stepId));for(const[id,row]of rows)if(!ids.has(id)){row.el.remove();rows.delete(id);}
  for(const step of steps){let row=rows.get(step.stepId);if(!row){const title=h('strong'),detail=h('p'),entry=h('li',{},title,detail);row={el:entry,title,detail};rows.set(step.stepId,row);list.append(entry);}
   row.el.dataset.state=step.state;row.title.textContent=`Passo ${step.ordinal} · ${stepStatus(step)}`;
   const duration=step.startedAt!==undefined&&step.finishedAt!==undefined?`${Math.max(0,step.finishedAt-step.startedAt)/1000}s`:null;
   row.detail.textContent=[step.cause?causes[step.cause]??'Causa por confirmar':null,step.trigger&&step.trigger!==step.cause?causes[step.trigger]??null:null,`Saída: ${step.stdoutBytes} B · saída de erros: ${step.stderrBytes} B`,duration,step.truncated?'Saída limitada':null].filter(Boolean).join(' · ');
  }
 }};
}
