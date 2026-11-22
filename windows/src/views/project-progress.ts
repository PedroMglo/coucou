import{projectDecisionDisplayIntent}from'../core/project-decision';
import {Bridge,type PetMemoryProject,type RuntimeGoal,type RuntimeListing,type RuntimeHistorySnapshot} from '../core/bridge';
import {readPersistedGoalResponse,goalStateLabel} from '../core/runtime-goals';
import {h} from './dom';
type Transport=Pick<typeof Bridge,'petRuntimeList'|'petRuntimeRead'|'petRuntimeHistory'|'petRuntimeResult'>;
type Summary=RuntimeListing['goals'][number];
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
const errorText=(e:unknown)=>String(e).replace(/^Error:\s*/,'');
const date=(at:number)=>Number.isSafeInteger(at)&&at>0?new Date(at).toLocaleString():'data indisponível';
// The native summary uses ContextBinding::project(): progress observations own
// a project snapshot even when their optional pinned notes are empty.
const capturedProject=(goal:RuntimeGoal)=>{
 const context=goal.executionContext;
 if(context?.schemaVersion===12){
  const project=context.projectProgress?.selection.project;
  return project&&(!context.memory||context.memory.schemaVersion===2&&same(context.memory.project,project))?project:undefined;
 }
 return !context?.projectProgress&&context?.memory?.schemaVersion===2?context.memory.project:undefined;
};
const linked=(goal:RuntimeGoal,project:PetMemoryProject,captured:PetMemoryProject|undefined)=>goal.spec.owner.petId===project.petId&&captured?.id===project.id&&captured.petId===project.petId;
/** Explicit reads of existing runtime state. This view has no scheduler,
 * execution controls, progress store, percentage or implicit data/model use. */
export function buildProjectProgress(options:{changed?:()=>void;transport?:Transport}={}){
 const transport=options.transport??Bridge,changed=options.changed??(()=>{});
 let project:PetMemoryProject|null=null,epoch=0,alive=true,loading=false;
 const status=h('p',{role:'status'}),items=h('div');
 const load=h('button',{type:'button',text:'Consultar objetivos do projeto',onclick:()=>void read()})as HTMLButtonElement;
 const el=h('details',{'aria-label':'Progresso do projeto'},h('summary',{text:'Objetivos e progresso deste projeto'}),h('p',{text:'Inclui apenas objetivos cujo contexto foi revisto com este projeto e pet. Uma conversa associada ou uma resposta guardada não prova conclusão. A consulta não executa trabalho nem envia dados a modelos.'}),load,status,items);el.hidden=true;
 function draw(){el.hidden=!project;load.disabled=!project||loading;changed();}
 function valid(stamp:number){return alive&&stamp===epoch;}
 function row(summary:Summary,p:PetMemoryProject,stamp:number){
  const details=h('div'),message=h('p',{role:'status'}),answer=h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere'});
  let goal:RuntimeGoal|null=null,busy=false;
  const inspect=h('button',{type:'button',text:'Consultar progresso deste objetivo',onclick:()=>void inspectGoal()})as HTMLButtonElement;
  const response=h('button',{type:'button',text:'Consultar resposta persistida',hidden:true,onclick:()=>void readResponse()})as HTMLButtonElement;
  const article=h('article',{'aria-label':`Objetivo do projeto ${summary.id}`},h('h4',{text:projectDecisionDisplayIntent(summary.intentPreview)}),h('p',{text:`${summary.historicalSource?'Histórico restaurado · estado declarado no snapshot: ':'Estado na listagem: '}${goalStateLabel(summary.state)} · atualizado em ${date(summary.updatedAt)}.`}),h('p',{text:`Contexto capturado com a revisão ${summary.project!.revision}; projeto consultado na revisão ${p.revision}${p.enabled?'':' · arquivado'}. Alterações não atualizam o contexto nem renovam autorizações.`}),inspect,response,message,details,answer);
  function pending(value:boolean){busy=value;inspect.disabled=response.disabled=value;changed();}
  async function inspectGoal(){
   if(!valid(stamp)||busy)return;pending(true);goal=null;response.hidden=true;answer.textContent='';details.replaceChildren();message.textContent='A consultar objetivo e provas da mesma revisão…';
   try{
    const g=await transport.petRuntimeRead(p.petId,summary.id);if(!valid(stamp))return;
    const captured=capturedProject(g);
    if(g.id!==summary.id||g.revision!==summary.revision||!linked(g,p,captured)||captured?.revision!==summary.project?.revision)throw Error('O objetivo mudou ou não pertence ao projeto original. Consulta novamente a listagem.');
    const history=await transport.petRuntimeHistory({petId:p.petId,goalId:g.id,revision:g.revision});if(!valid(stamp))return;
    if(![1,2].includes(history.schemaVersion)||!['allRecordedAdmissions','restoredSnapshotOnly'].includes(history.scope)||history.goalId!==g.id||history.petId!==p.petId||history.goalRevision!==g.revision||history.goalState!==g.state||history.executionAvailable!==false||history.continuationAvailable!==false)throw Error('O histórico não corresponde ao objetivo e à revisão consultados.');
    const historical=history.scope==='restoredSnapshotOnly';if(historical&&(!history.restoredSource||history.restoredSource.executionAuthorityGranted!==false||history.restoredSource.postSnapshotStateUnknown!==true))throw Error('A origem do histórico restaurado não foi confirmada.');
    goal=structuredClone(g);const criteria=h('ul');criteria.append(...g.spec.criteria.map(text=>h('li',{text})));
    const source=g.executionContext?.sourceConversationId;
    details.replaceChildren(h('h4',{text:projectDecisionDisplayIntent(g.spec.intent)}),h('p',{text:`${historical?'Estado declarado no histórico: ':'Estado registado: '}${goalStateLabel(g.state)}. Observação das provas: ${date(history.observedAt)}.`}),h('h5',{text:'Critérios originais de conclusão'}),criteria,h('p',{text:g.completion?'Existe uma referência de verificação nativa; consulta o comprovativo completo abaixo.':'Não há referência de conclusão nativa neste objetivo. Uma resposta não verifica estes critérios.'}),h('p',{text:`Conversa de origem: ${source??'não indicada'}${source&&!p.conversationIds.includes(source)?' · associação atual retirada':''}. A associação atual não altera o contexto capturado.`}),historyFacts(history),h('details',{},h('summary',{text:'Comprovativos e referências exatas'}),h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere',text:JSON.stringify({goalId:g.id,goalRevision:g.revision,project:captured,completion:g.completion??null,history},null,2)})));
    response.hidden=!g.admissions?.length;message.textContent=historical?'Histórico consultado; o estado posterior ao snapshot é desconhecido e não há execução restaurada.':'Progresso consultado. Consulta novamente para observar alterações; não foi iniciado trabalho.';
   }catch(e){if(valid(stamp)){goal=null;details.replaceChildren();message.textContent=`Progresso indisponível: ${errorText(e)}`;}}
   finally{if(valid(stamp))pending(false);}
  }
  async function readResponse(){
   if(!valid(stamp)||busy||!goal)return;const original=structuredClone(goal);pending(true);answer.textContent='';message.textContent='A consultar o recibo da resposta…';
   try{
    const {result,text}=await readPersistedGoalResponse(original,transport);if(!valid(stamp))return;
    if(result.status==='available'){
     if(result.receipt.petId!==p.petId||result.receipt.goalId!==original.id||!original.admissions?.some(a=>a.spec.decisionId===result.receipt.decisionId))throw Error('A resposta não pertence ao objetivo consultado.');
     answer.textContent=text??'';message.textContent='Resposta persistida consultada. Os critérios de conclusão mantêm a sua verificação própria.';
    }else message.textContent=result.status==='notStarted'?'Não há uma resposta persistida nesta admissão.':'A tentativa tem resultado desconhecido ou recibo por recuperar. Não foi pedida nova inferência.';
   }catch(e){if(valid(stamp))message.textContent=`Resposta indisponível: ${errorText(e)}`;}
   finally{if(valid(stamp))pending(false);}
  }
  return article;
 }
 async function read(){
  if(!project||loading||!alive)return;const p=structuredClone(project),stamp=++epoch;loading=true;items.replaceChildren();status.textContent='A consultar objetivos persistidos…';draw();
  try{
   const page=await transport.petRuntimeList(p.petId);if(!valid(stamp))return;
   if(page.schemaVersion!==1||!Array.isArray(page.goals)||page.goals.length>256||!Number.isSafeInteger(page.unavailable)||page.unavailable<0||!Number.isSafeInteger(page.pendingArtifacts)||page.pendingArtifacts<0||typeof page.complete!=='boolean'||page.complete&&(page.unavailable>0||page.pendingArtifacts>0)||page.goals.some(g=>g.petId!==p.petId||!Object.hasOwn(g,'project')||g.project!==null&&(!g.project?.id||!Number.isSafeInteger(g.project.revision)||g.project.revision<1)))throw Error('A listagem não confirma os projetos e o pet nesta versão. Nenhum objetivo foi associado por nome ou conversa.');
   const selected=page.goals.filter(g=>g.project?.id===p.id);if(new Set(selected.map(g=>g.id)).size!==selected.length)throw Error('A listagem contém identidades repetidas. Consulta novamente.');
   status.textContent=`${selected.length} objetivos deste projeto identificados nesta consulta.${page.complete?'':' Consulta incompleta: '+page.unavailable+' registos indisponíveis e '+page.pendingArtifacts+' artefactos pendentes; podem faltar objetivos.'}${selected.length?'':' Sem associação explícita demonstrada nos registos consultados.'}`;
   items.replaceChildren(...selected.map(g=>row(g,p,stamp)));
  }catch(e){if(valid(stamp))status.textContent=`Consulta indisponível: ${errorText(e)}`;}
  finally{if(valid(stamp)){loading=false;draw();}}
 }
 draw();return{el,sync(next:PetMemoryProject|null){if(same(next,project))return;epoch++;project=structuredClone(next);loading=false;status.textContent='';items.replaceChildren();el.open=false;draw();},dispose(){alive=false;epoch++;}};
}
function historyFacts(history:RuntimeHistorySnapshot){
 if(history.scope==='restoredSnapshotOnly')return h('p',{text:'Contabilidade atual e resultados posteriores desconhecidos. Os dados históricos foram preservados; os valores vazios não significam consumo zero. Não existe saldo ou autorização recuperados.'});
 const total=history.cumulativeCharge??history.authenticatedSubtotal;
 return h('p',{text:`${history.accountingComplete&&history.cumulativeCharge?'Consumo comprovado no âmbito dos registos consultados':'Subtotal autenticado; contabilidade incompleta'}: ${total.modelCalls} chamadas de modelo, ${total.toolCalls} de ferramentas e ${total.outputBytes} B debitados. Resultados desconhecidos: ${total.unknownModelResults+total.unknownToolResults}.${history.outcomesComplete?'':' Há resultados em falta ou não confirmados.'} Esta consulta não demonstra continuidade integral do histórico nem renova saldo.`});
}
