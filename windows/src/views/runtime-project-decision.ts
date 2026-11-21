import {Bridge,type RuntimeGoal,type RuntimeProjectDecisionInput,type RuntimeProjectDecisionView,type RuntimeProjectDecisionTicket,type RuntimeProjectDecisionAnswer,type RuntimeProjectDecisionReport,type PetMemoryEntry} from '../core/bridge';
import {PROJECT_DECISION_PROMPT} from '../core/project-decision';
import {h} from './dom';
type Transport=Pick<typeof Bridge,'petRuntimeProjectDecisionRead'|'petRuntimeProjectDecisionPrepare'|'petRuntimeProjectDecisionConfirm'|'petRuntimeProjectDecisionCancel'>;
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
const error=(e:unknown)=>{
 const raw=String(e).replace(/^Error:\s*/,'');
 return({projectDecisionReviewEnded:'A revisão terminou. Consulta o pedido e prepara outra revisão.',projectDecisionSourceChanged:'A origem ou os dados revistos mudaram. Consulta o objetivo e o projeto antes de preparar outra resposta.',projectDecisionSourceUnavailable:'Não foi possível confirmar a origem e o recibo. Consulta o objetivo antes de continuar.',projectDecisionResponseUnavailable:'Ainda não existe uma resposta confirmada para este pedido. Não foi pedida nova inferência.',projectDecisionReportInvalid:'O relatório não contém um pedido de decisão no formato revisto. Podes consultar a resposta persistida.',projectDecisionReportLimit:'O relatório excede os limites do pedido de decisão. A resposta persistida foi conservada.',projectDecisionAnswerInvalid:'Escolhe uma opção válida ou escreve uma resposta de até1024 bytes.',projectDecisionNoteLimit:'A nota completa excede o limite disponível. Revê a resposta antes de guardar.',projectDecisionOwnerChanged:'O perfil do pet mudou. Consulta novamente o objetivo.',projectDecisionNotRecordable:'Este pedido já não está disponível para registar uma resposta.',Conflict:'A origem ou a memória mudou, ou já existe outra resposta. Consulta a nota antes de repetir.',Busy:'Outro pedido está a utilizar estes dados. Consulta o estado antes de tentar novamente.'}as Record<string,string>)[raw]??raw;
};
const answerText=(answer:RuntimeProjectDecisionAnswer,report:RuntimeProjectDecisionReport)=>answer.kind==='decline'?'Recusar decidir':answer.kind==='text'?answer.text:report.question?.options[answer.index]??'Opção indisponível';
export function buildRuntimeProjectDecision(options:{transport?:Transport;changed?:()=>void}={}){
 const api=options.transport??Bridge,changed=options.changed??(()=>{});
 let source:RuntimeProjectDecisionInput|null=null,key='',epoch=0,alive=true,busy=false,view:RuntimeProjectDecisionView|null=null,ticket:RuntimeProjectDecisionTicket|null=null;
 const status=h('p',{role:'status'}),body=h('div'),reviewFacts=h('div'),preview=h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere'});
 const choice=h('select',{'aria-label':'Resposta ao pedido do projeto'})as HTMLSelectElement;
 const free=h('textarea',{'aria-label':'Resposta escrita ao projeto',rows:3,maxlength:1024})as HTMLTextAreaElement;
 const consent=h('input',{type:'checkbox','aria-label':'Guardar esta resposta como nota deste projeto'})as HTMLInputElement;
 const read=h('button',{type:'button',text:'Consultar pedido de decisão',onclick:()=>void consult()})as HTMLButtonElement;
 const prepare=h('button',{type:'button',text:'Rever resposta ao pedido',onclick:()=>void review()})as HTMLButtonElement;
 const confirm=h('button',{type:'button',text:'Guardar resposta revista no projeto',onclick:()=>void save()})as HTMLButtonElement;
 const cancel=h('button',{type:'button',text:'Cancelar revisão da resposta',onclick:()=>{clearTicket();draw();}})as HTMLButtonElement;
 const answerForm=h('div',{},choice,free,prepare);
 const reviewForm=h('div',{},h('p',{text:'Revê a origem, pergunta e resposta completas antes de guardar. A nota não executa uma decisão nem autoriza uso futuro por modelos.'}),reviewFacts,h('details',{},h('summary',{text:'Nota completa e referências de origem'}),preview),h('label',{},consent,'Guardar esta resposta como nota deste projeto'),confirm,cancel);
 const el=h('details',{'aria-label':'Decisão do acompanhamento de projeto'},h('summary',{text:'Pedido de decisão do projeto'}),h('p',{text:'A pergunta provém do relatório persistido. Uma escolha ou resposta é informação do projeto; ações e uso por modelos mantêm a sua revisão própria.'}),read,status,body,answerForm,reviewForm);el.hidden=true;
 function clearTicket(){const old=ticket;ticket=null;consent.checked=false;reviewFacts.replaceChildren();preview.textContent='';if(old)void api.petRuntimeProjectDecisionCancel(old.token).catch(()=>{});}
 function showNote(note:PetMemoryEntry,v:RuntimeProjectDecisionView){
  let answer:string|null=null;
  try{
   const data=JSON.parse(note.content),a=data.answer as RuntimeProjectDecisionAnswer;
   if(data.schemaVersion===1&&data.kind==='projectDecisionData'&&same(data.source,v.source)&&same(data.report,v.report)&&a&&(a.kind==='decline'||a.kind==='text'&&typeof a.text==='string'||a.kind==='choice'&&Number.isSafeInteger(a.index)&&a.index>=0&&a.index<(v.report.question?.options.length??0)))answer=answerText(a,v.report);
  }catch{}
  body.append(h('p',{text:answer===null?'Existe uma nota editada para esta origem. Consulta o conteúdo integral.':note.revision>1?'Registo atual da nota editada:':'Resposta registada na nota do projeto:'}),...(answer===null?[]:[h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere',text:answer})]),h('details',{},h('summary',{text:'Consultar nota completa e origem'}),h('p',{text:'Esta nota é um registo explícito do utilizador e pode ser editada. Não substitui o recibo original nem concede execução.'}),h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere',text:note.content})));
 }
 function invalidate(){epoch++;busy=false;clearTicket();view=null;body.replaceChildren();choice.replaceChildren();free.value='';status.textContent='';draw();}
 function current(stamp:number){return alive&&stamp===epoch;}
 function draw(){read.disabled=busy||!source;prepare.disabled=busy||!view?.canRecord||!choice.value;free.hidden=choice.value!=='text';answerForm.hidden=!view?.canRecord;reviewForm.hidden=!ticket;confirm.disabled=busy||!ticket||!consent.checked;cancel.disabled=busy;changed();}
 choice.addEventListener('change',()=>{clearTicket();draw();});free.addEventListener('input',()=>{clearTicket();draw();});consent.addEventListener('change',draw);
 async function consult(){
  if(!alive||busy||!source)return;const original=structuredClone(source),stamp=++epoch;busy=true;clearTicket();view=null;body.replaceChildren();status.textContent='A consultar a pergunta e a origem nativas…';draw();
  try{
   const v=await api.petRuntimeProjectDecisionRead(original);if(!current(stamp))return;
   if(v.schemaVersion!==1||typeof v.projectName!=='string'||!v.projectName||v.source.petId!==original.petId||v.source.goalId!==original.goalId||v.source.decisionId!==original.decisionId||v.executionAuthorized!==false||v.modelUseAuthorized!==false||!v.source.projectId||!Number.isSafeInteger(v.source.projectRevision)||v.source.projectRevision<1||!/^[a-f0-9]{64}$/.test(v.source.receiptHash)||v.report.schemaVersion!==1)throw Error('A pergunta não corresponde ao objetivo e ao recibo consultados.');
   const q=v.report.question;if(q&&(!q.text||!Array.isArray(q.options)||q.options.length<2||q.options.length>4||new Set(q.options).size!==q.options.length)||v.canRecord&&(v.historical||!q||!!v.note))throw Error('A disponibilidade da pergunta não foi confirmada.');
   view=structuredClone(v);choice.replaceChildren(h('option',{value:'',text:'Escolhe uma resposta'}),...(q?.options??[]).map((text,i)=>h('option',{value:String(i),text})),h('option',{value:'text',text:'Escrever outra resposta'}),h('option',{value:'decline',text:'Recusar decidir'}));choice.value='';
   body.replaceChildren(h('p',{text:v.report.summary}),h('p',{text:`Projeto ${v.projectName} · revisão ${v.source.projectRevision}. Relatório observado em ${new Date(v.source.progressObservedAt).toLocaleString()}; resposta guardada em ${new Date(v.source.responseRecordedAt).toLocaleString()}.`}),...(q?[h('h5',{text:q.text})]:[]));
   if(v.note)showNote(v.note,v);
   status.textContent=v.historical?'Histórico restaurado: consulta disponível; esta origem não permite registar uma nova resposta.':v.note?'Resposta do projeto já registada.':!q?'O relatório não pediu uma decisão.':v.canRecord?'Escolhe uma resposta e revê a nota antes de guardar.':'Esta origem não está disponível para registar uma resposta.';
  }catch(e){if(current(stamp))status.textContent=`Pedido indisponível: ${error(e)}`;}
  finally{if(current(stamp)){busy=false;draw();}}
 }
 async function review(){
  if(!alive||busy||!source||!view?.canRecord)return;
  const answer:RuntimeProjectDecisionAnswer=choice.value==='decline'?{kind:'decline'}:choice.value==='text'?{kind:'text',text:free.value}:{kind:'choice',index:Number(choice.value)};
  if(!choice.value||answer.kind==='text'&&!answer.text.trim()){status.textContent='Escolhe uma opção, escreve a resposta ou recusa decidir.';return;}
  const original=structuredClone(source),observed=structuredClone(view),stamp=epoch;busy=true;clearTicket();status.textContent='A preparar a revisão exata da nota…';draw();
  try{
   const t=await api.petRuntimeProjectDecisionPrepare({source:original,answer});
   if(!current(stamp)){void api.petRuntimeProjectDecisionCancel(t.token).catch(()=>{});return;}
   if(!t.token||!t.noteId||t.projectName!==observed.projectName||t.executionAuthorized!==false||t.modelUseAuthorized!==false||!same(t.data.source,observed.source)||!same(t.data.report,observed.report)||!same(t.data.answer,answer)||t.data.kind!=='projectDecisionData'||!same(JSON.parse(t.noteContent),t.data)){void api.petRuntimeProjectDecisionCancel(t.token).catch(()=>{});throw Error('A nota revista não corresponde à pergunta e resposta escolhidas.');}
   ticket=structuredClone(t);reviewFacts.replaceChildren(h('p',{text:`Guardar em ${t.projectName}, revisão ${t.data.source.projectRevision}.`}),h('p',{text:t.data.report.question?.text??''}),h('strong',{text:'A tua resposta:'}),h('pre',{style:'white-space:pre-wrap;overflow-wrap:anywhere',text:answerText(t.data.answer,t.data.report)}));preview.textContent=`${t.noteTitle}\n${t.noteContent}`;status.textContent='Nota preparada. Confirma separadamente a sua gravação no projeto.';
  }catch(e){if(current(stamp))status.textContent=`Revisão indisponível: ${error(e)}`;}
  finally{if(current(stamp)){busy=false;draw();}}
 }
 async function save(){
  if(!alive||busy||!ticket||!consent.checked)return;const t=structuredClone(ticket),stamp=epoch;busy=true;status.textContent='A guardar a resposta revista…';draw();
  try{
   const receipt=await api.petRuntimeProjectDecisionConfirm(t.token);if(!current(stamp))return;
   if(receipt.schemaVersion!==1||receipt.id!==t.noteId||receipt.item?.id!==t.noteId||receipt.item.content!==t.noteContent||receipt.owner.petId!==t.data.source.petId||receipt.owner.scope.kind!=='project'||receipt.owner.scope.projectId!==t.data.source.projectId)throw Error('A gravação não confirmou a nota exata. Consulta o pedido antes de repetir.');
   if(view){view.note=structuredClone(receipt.item);view.canRecord=false;showNote(receipt.item,view);}clearTicket();status.textContent='Resposta guardada como dados do projeto. Não foi iniciada execução ou inferência.';
  }catch(e){if(current(stamp))status.textContent=`Gravação não confirmada: ${error(e)} A revisão original foi conservada; consulta a nota antes de preparar outra resposta.`;}
  finally{if(current(stamp)){busy=false;draw();}}
 }
 draw();return{el,invalidate,update(goal:RuntimeGoal,historical=false){const row=goal.admissions?.at(-1);const next=goal.spec.owner.adapter==='ollama'&&goal.executionContext?.schemaVersion===12&&row?.spec.schemaVersion===32&&goal.spec.intent.endsWith(PROJECT_DECISION_PROMPT)?{petId:goal.spec.owner.petId,goalId:goal.id,revision:goal.revision,decisionId:row.spec.decisionId}:null;const nextKey=JSON.stringify([next,historical]);if(nextKey!==key){key=nextKey;source=next;invalidate();}el.hidden=!next;},dispose(){invalidate();alive=false;}};
}
