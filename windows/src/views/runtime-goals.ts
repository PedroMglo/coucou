import {Bridge,onEvent,type PetCapabilityDescriptor,type RuntimeGoal,type RuntimeBudget} from '../core/bridge';
import {RuntimeGoalActions,goalStateLabel,latestAdmission,musicGoalPlan,templateCriteria,type MusicAction} from '../core/runtime-goals';
import {buildAgentRequests} from './agent-requests';
import {h} from './dom';
const requestId=()=>crypto.randomUUID();
const message=(error:unknown)=>{const raw=String(error).replace(/^Error:\s*/,'');return ({conflict:'O objetivo ou a autorização mudou. Atualiza o estado e revê o trabalho antes de continuar.',invalidInput:'Revê os campos e o modelo local selecionado no perfil do pet.',busy:'O recurso está ocupado. Consulta o estado antes de tentar novamente.',storageUnavailable:'O armazenamento está indisponível. Consulta os recibos antes de repetir.',limitReached:'Um limite desta operação foi atingido. Revê o orçamento e o estado.',unsupportedPlatform:'Este runtime de objetivos requer Linux.'}as Record<string,string>)[raw]??raw;};
const stateText=(status:string)=>({notStarted:'Ainda não há uma tentativa de resposta.',recoveryRequired:'O recibo requer recuperação; não foi pedida nova inferência.',declined:'A autorização foi recusada.',ended:'O pedido terminou sem autorização.',blocked:'A autorização não está disponível.',authorityBlocked:'A autoridade nativa recusou a autorização.',granted:'Receita autorizada. A execução ainda não começou.',queued:'Objetivo autorizado na fila.',goalCompleted:'Objetivo concluído com verificação nativa.',verified:'Passos verificados; consulta os critérios do objetivo.',criteriaUnmet:'O estado final não satisfaz os critérios.',criteriaIncomplete:'Falta verificar critérios.',invalidated:'A execução perdeu a autorização ou o alvo.',available:'Resposta persistida consultada.'})[status]??status;
const eligiblePlans=(goal:RuntimeGoal)=>goal.plans.filter(p=>p.steps.length>0&&p.steps.every(s=>s.operation?.operationId==='spotify.control'&&s.operation.version===1));
interface Options{initialPet?:string;changed?:()=>void;openPet?:(pet:string)=>void;ids?:()=>string;actions?:RuntimeGoalActions}
/** Functional supervision only. Passive reads do not prepare providers, approve
 * work, execute tools or reconstruct a controller after a restart. */
export function buildRuntimeGoals(options:Options={}){
 const changed=options.changed??(()=>{}),ids=options.ids??requestId;let alive=true,active=false,loading=false,again=false,epoch=0,goals:RuntimeGoal[]=[],descriptors:PetCapabilityDescriptor[]=[],draftId=ids();
 const title=h('h2',{text:'Objetivos persistentes'}),feedback=h('p',{role:'status'}),items=h('div'),form=h('details',{},h('summary',{text:'Criar objetivo'}));
 const pet=h('select',{'aria-label':'Pet responsável pelo objetivo'}) as HTMLSelectElement;
 const kind=h('select',{'aria-label':'Tipo de trabalho'},h('option',{value:'text',text:'Preparar resposta'}),h('option',{value:'music',text:'Sequência de música'})) as HTMLSelectElement;
 kind.value='text';
 const intent=h('textarea',{'aria-label':'Objetivo',placeholder:'O que queres alcançar?',rows:3}) as HTMLTextAreaElement;
 const criteria=h('textarea',{'aria-label':'Critérios de conclusão',placeholder:'Um critério de conclusão por linha',rows:3}) as HTMLTextAreaElement;
 const music=h('div'),sequence=h('ol'),preview=h('p'),musicAction=h('select',{'aria-label':'Passo de música'},h('option',{value:'pause',text:'Pausar'}),h('option',{value:'play',text:'Reproduzir'}),h('option',{value:'set_volume',text:'Volume do leitor'})) as HTMLSelectElement;
 musicAction.value='pause';const volume=h('input',{type:'number',min:0,max:100,value:30,'aria-label':'Volume do leitor em percentagem'}) as HTMLInputElement;
 let musicSteps:MusicAction[]=[{action:'pause'}];
 const setFeedback=(text:string)=>{if(alive){feedback.textContent=text;changed();}};
 const actions=options.actions??new RuntimeGoalActions(()=>{if(alive){draw();create.disabled=actions.pending.has('create');changed();}});
 function syncMusic(){music.hidden=kind.value!=='music';criteria.hidden=kind.value==='music';sequence.replaceChildren();for(const [i,step]of musicSteps.entries())sequence.append(h('li',{},h('span',{text:step.action==='set_volume'?`Volume do leitor: ${step.volume}%`:step.action==='play'?'Reproduzir':'Pausar'}),h('button',{type:'button',text:'Remover',onclick:()=>{musicSteps.splice(i,1);syncMusic();}})));try{preview.textContent='Critérios de conclusão: '+musicGoalPlan(musicSteps).criteria.join(' ');}catch(e){preview.textContent=message(e);}changed();}
 music.append(h('p',{text:'Escolhe a sequência completa. O leitor será capturado na revisão nativa; o som do sistema não é alterado por estes passos.'}),musicAction,volume,h('button',{type:'button',text:'Adicionar passo',onclick:()=>{if(musicSteps.length>=8){setFeedback('O limite desta receita é oito passos.');return;}try{if(musicAction.value==='set_volume'&&!volume.value.trim())throw Error('Indica o volume do leitor.');const next:MusicAction=musicAction.value==='set_volume'?{action:'set_volume',volume:Number(volume.value)}:{action:musicAction.value==='play'?'play':'pause'};musicGoalPlan([...musicSteps,next]);musicSteps.push(next);syncMusic();}catch(e){setFeedback(message(e));}}}),sequence,preview);
 kind.addEventListener('change',syncMusic);
 const create=h('button',{type:'button',text:'Guardar objetivo',onclick:()=>void createGoal()}) as HTMLButtonElement;
 form.append(h('label',{},'Pet responsável',pet),h('label',{},'Objetivo',intent),h('label',{},'Tipo de trabalho',kind),h('label',{},'Critérios de conclusão',criteria),music,h('p',{text:'Guardar prepara dados e um contexto privado. Não inicia um modelo, não lê anexos da conversa e não executa ações.'}),create);
 const refreshButton=h('button',{type:'button',text:'Atualizar objetivos',onclick:()=>void refresh()});
 const decisions=buildAgentRequests(()=>null,changed,{pending:()=>Bridge.petRuntimeGoalPending(),answer:(id,answer,target)=>target?.origin==='goal'?Bridge.petRuntimeGoalAnswer(id,answer.allow===true,target):Promise.reject(Error('Este controlo só responde a objetivos.')),visible:request=>!!request.goalTarget,accept:request=>!!request.goalTarget});
 const el=h('section',{class:'runtime-goals','aria-label':'Objetivos persistentes'},title,form,refreshButton,feedback,decisions.el,items);
 function syncPets(){const previous=pet.value;pet.replaceChildren();for(const d of descriptors){const option=h('option',{value:d.petId,text:`${d.name}${d.lifecycle!=='active'?' · inativo':''}${!d.binding.adapter?' · sem executor configurado':''}`}) as HTMLOptionElement;pet.append(option);}pet.value=descriptors.some(d=>d.petId===previous)?previous:descriptors.some(d=>d.petId===options.initialPet)?options.initialPet!:descriptors.find(d=>d.lifecycle==='active'&&d.capabilities.some(c=>c.id==='goals.draft'&&c.support==='supported'))?.petId??descriptors[0]?.petId??'';}
 async function createGoal(){
  const selected=descriptors.find(d=>d.petId===pet.value);if(!selected||selected.lifecycle!=='active'||!selected.binding.adapter){setFeedback('Escolhe um pet ativo com um executor configurado.');return;}
  const originalIntent=intent.value.trim();if(!originalIntent){setFeedback('Descreve o objetivo.');return;}
  try{const plan=kind.value==='music'?musicGoalPlan(musicSteps):{criteria:criteria.value.split('\n').map(s=>s.trim()).filter(Boolean),steps:[{description:'Preparar uma resposta ao objetivo',postcondition:'Resposta persistida; critérios ainda por verificar.'}]};if(!plan.criteria.length)throw Error('Indica pelo menos um critério de conclusão.');
   const result=await actions.create({petId:selected.petId,requestId:draftId,intent:originalIntent,criteria:plan.criteria,conversationId:null},plan.steps);draftId=ids();intent.value='';criteria.value='';setFeedback(`Objetivo guardado para ${selected.name}. Revê a autorização de trabalho antes de executar.`);await refresh();return result;
  }catch(e){setFeedback(message(e));await refresh();}
 }
 type Row={el:HTMLElement;goal:RuntimeGoal;state:HTMLElement;criteria:HTMLElement;plans:HTMLSelectElement;error:HTMLElement;review:HTMLElement;reviewToken:string;result:HTMLElement;progress:HTMLElement;recovery:HTMLElement;buttons:Map<string,HTMLButtonElement>;budget:HTMLFieldSetElement;minutes:HTMLInputElement;calls:HTMLInputElement;tools:HTMLInputElement;output:HTMLInputElement;admissionId:string;claimId:string};
 const rows=new Map<string,Row>();const records=new Map<string,RuntimeGoal>();let unavailable=new Set<string>();
 async function run<T>(row:Row,operation:()=>Promise<T>){try{const result=await operation();if(result&&typeof result==='object'&&'status'in result)setFeedback(stateText(String(result.status))+('reason'in result?` · ${result.reason}`:''));return result;}catch(e){actions.errors.set(row.goal.id,message(e));setFeedback(message(e));}finally{await refresh();}}
 function numeric(label:string,value:number,min:number,max:number){return h('input',{type:'number',value,min,max,'aria-label':label}) as HTMLInputElement;}
 function rowFor(goal:RuntimeGoal):Row{
  const heading=h('h3',{text:goal.spec.intent}),state=h('p'),criteria=h('ul'),plans=h('select',{'aria-label':'Plano de execução'}) as HTMLSelectElement;
  const error=h('p',{role:'status'}),review=h('div'),result=h('pre',{style:'white-space:pre-wrap'}),progress=h('p'),recovery=h('div'),buttons=new Map<string,HTMLButtonElement>();
  const minutes=numeric('Prazo em minutos',10,1,1440),calls=numeric('Máximo de chamadas ao modelo',1,1,64),tools=numeric('Máximo de chamadas a ferramentas',goal.plans.flatMap(p=>p.steps).filter(s=>s.operation).length,0,128),output=numeric('Máximo de saída em KiB',64,2,16384);
  const budget=h('fieldset',{},h('legend',{text:'Limites da autorização'}),h('label',{},'Prazo em minutos',minutes),h('label',{},'Chamadas ao modelo',calls),h('label',{},'Chamadas a ferramentas',tools),h('label',{},'Saída em KiB',output)) as HTMLFieldSetElement;
  const footer=h('div'),el=h('article',{},heading,state,criteria,budget,plans,review,progress,recovery,result,error,footer);
  const row:Row={el,goal,state,criteria,plans,error,review,reviewToken:'',result,progress,recovery,buttons,budget,minutes,calls,tools,output,admissionId:ids(),claimId:ids()};
  function button(id:string,text:string,fn:()=>Promise<unknown>){const b=h('button',{type:'button',text,onclick:()=>{if(!b.disabled)void run(row,fn);}}) as HTMLButtonElement;buttons.set(id,b);footer.append(b);}
  button('context','Preparar contexto privado',()=>actions.prepareContext(row.goal,ids()));
  button('admission','Rever autorização de trabalho',async()=>{const budget:RuntimeBudget={deadlineAt:Date.now()+Number(minutes.value)*60_000,maxModelCalls:Number(calls.value),maxToolCalls:Number(tools.value),maxOutputBytes:Number(output.value)*1024,maxTokens:null,maxCostMicros:null,payment:row.goal.spec.owner.adapter==='ollama'?'localOnly':'existingAccount'};return actions.prepareAdmission(row.goal,budget,row.admissionId);});
  button('claim','Reservar sessão para este objetivo',()=>actions.claim(row.goal,row.claimId));
  button('respond','Pedir resposta ao objetivo',()=>actions.respond(row.goal));
  button('read-response','Consultar resposta persistida',()=>actions.readResponse(row.goal));
  button('recipe','Rever receita',async()=>{const plan=plans.value;if(!plan)throw Error('Escolhe um plano.');const reply=await actions.requestRecipe(row.goal,plan,templateCriteria(row.goal));if(reply.status==='granted')await actions.readRecipe(row.goal,plan);return reply;});
  button('progress','Consultar progresso da receita',()=>actions.readRecipe(row.goal,plans.value));
  button('run','Executar receita autorizada',async()=>{const current=actions.recipes.get(row.goal.id);if(!current||current.scope.planId!==plans.value)throw Error('Consulta a receita deste plano antes de executar.');const reply=await actions.runRecipe(row.goal,current);setFeedback(stateText(reply.outcome));return reply;});
  button('complete','Confirmar conclusão nativa',()=>actions.completeRecipe(row.goal,actions.recipes.get(row.goal.id)!));
  button('session','Consultar estado da sessão',async()=>{const s=await Bridge.petRuntimeClaimStatus(row.goal.spec.owner.petId,row.goal.id);setFeedback(`${s.source==='liveController'?'Controlador nativo presente':s.source==='storedLease'?'Só existe a reserva guardada':'Sem controlador'} · workers: ${s.workers===null?'desconhecidos':s.workers}.`);return s;});
  button('pause','Pausar / interromper',()=>actions.stop(row.goal,'pause',ids()));
  button('cancel','Cancelar objetivo',()=>actions.stop(row.goal,'cancel',ids()));
  button('release','Pedir libertação da sessão',()=>actions.release(row.goal));
  button('resume','Reabrir rascunho',()=>actions.resumeDraft(row.goal,ids()));
  if(options.openPet)footer.append(h('button',{type:'button',text:'Abrir chat do pet',onclick:()=>options.openPet?.(row.goal.spec.owner.petId)}));
  return row;
 }
 function draw(){
  if(!alive)return;const currentIds=new Set(goals.map(g=>g.id));for(const[id,row]of rows)if(!currentIds.has(id)){row.el.remove();rows.delete(id);}
  for(const goal of goals){let row=rows.get(goal.id);if(!row){row=rowFor(goal);rows.set(goal.id,row);}row.goal=goal;row.el.dataset.goalId=goal.id;
   const petName=descriptors.find(d=>d.petId===goal.spec.owner.petId)?.name??goal.spec.owner.petId;row.state.textContent=`${petName} · ${unavailable.has(goal.id)?'último estado conhecido: ':''}${goalStateLabel(goal.state)}${unavailable.has(goal.id)?' · fonte atual indisponível':''}`;row.criteria.replaceChildren(...goal.spec.criteria.map(text=>h('li',{text})));
   const previous=row.plans.value,plans=eligiblePlans(goal);if(JSON.stringify([...row.plans.querySelectorAll('option')].map(o=>o.getAttribute('value')))!==JSON.stringify(plans.map(p=>p.id))){row.plans.replaceChildren(...plans.map((p,i)=>h('option',{value:p.id,text:`Plano ${i+1} · ${p.steps.map(s=>s.description).join(' → ')}`})));row.plans.value=plans.some(p=>p.id===previous)?previous:plans[0]?.id??'';}
   const busy=actions.pending.has(goal.id),control=actions.controls.has(goal.id),lease=latestAdmission(goal)?.lease,recipe=actions.recipes.get(goal.id),sameRecipe=recipe?.scope.planId===row.plans.value;
   row.budget.hidden=goal.state!=='draft';row.plans.hidden=!plans.length;row.plans.disabled=busy;
   for(const [id,b]of row.buttons){b.disabled=id==='pause'||id==='cancel'||id==='release'?control:busy;b.hidden=!({context:goal.state==='draft'&&!goal.executionContext,admission:goal.state==='draft'&&!!goal.executionContext&&actions.reviews.get(goal.id)?.status!=='review',claim:goal.state==='queued',respond:goal.state==='leased'&&goal.spec.owner.provider==='ollama'&&goal.spec.owner.adapter==='ollama','read-response':!!latestAdmission(goal),recipe:goal.state==='leased'&&plans.length>0,progress:!!latestAdmission(goal)&&plans.length>0,run:goal.state==='leased'&&!!sameRecipe&&recipe?.phase!=='recoveryRequired',complete:!!sameRecipe&&!!recipe?.criteriaVerified&&!recipe.goalComplete,session:!!lease,pause:['draft','queued','leased'].includes(goal.state),cancel:!['cancelled','completed'].includes(goal.state),release:!!lease&&!['completed','cancelled'].includes(goal.state),resume:['paused','recoveryRequired'].includes(goal.state)}as Record<string,boolean>)[id];}
   row.error.textContent=actions.errors.get(goal.id)??'';row.result.textContent=actions.responses.get(goal.id)??'';
   row.progress.textContent=recipe?`${recipe.verifiedSteps}/${recipe.totalSteps} passos verificados · ${recipe.criteriaVerified?'critérios verificados':'critérios ainda não verificados'} · ${recipe.goalComplete?'conclusão nativa confirmada':'conclusão ainda não confirmada'}`:'';
   row.recovery.replaceChildren();for(const step of recipe?.steps??[]){if(step.recovery?.reconciliationAvailable){const s=step;row.recovery.append(h('button',{type:'button',text:`Reconciliar passo ${s.stepId} sem repetir`,onclick:()=>void run(row!,async()=>{await actions.recoverStep(row!.goal,recipe!,s.stepId);return actions.readRecipe(row!.goal,recipe!.scope.planId);})}));}}
   const review=actions.reviews.get(goal.id);if(review?.status==='review'){
    if(row.reviewToken!==review.ticket.token){row.reviewToken=review.ticket.token;const p=review.ticket.proposal;row.review.replaceChildren(h('p',{text:'Esta autorização reserva os limites apresentados. As operações propostas exigem ainda a revisão própria da receita.'}),h('strong',{text:p.intent}),h('ul',{},...p.criteria.map(text=>h('li',{text}))),h('p',{text:`Modelo: ${p.resolvedModel} · até ${p.budget.maxModelCalls} chamadas ao modelo, ${p.budget.maxToolCalls} a ferramentas e ${p.budget.maxOutputBytes} B de saída · prazo ${new Date(p.budget.deadlineAt).toLocaleString()}`}),h('details',{},h('summary',{text:'Revisão completa'}),h('pre',{text:JSON.stringify(p,null,2)})),h('button',{type:'button',text:'Autorizar este trabalho',onclick:()=>void run(row!,()=>actions.confirmAdmission(row!.goal,true))}),h('button',{type:'button',text:'Recusar este trabalho',onclick:()=>void run(row!,()=>actions.confirmAdmission(row!.goal,false))}));}
    for(const b of row.review.querySelectorAll('button'))(b as HTMLButtonElement).disabled=busy;
   }else{row.reviewToken='';row.review.replaceChildren();if(review?.status==='blocked')row.review.append(h('p',{text:`A autorização está indisponível: ${review.reason}. Não foi iniciada execução.`}));}
   if(row.el.parentElement!==items)items.append(row.el);
  }changed();
 }
 async function refresh(){
  if(!alive||!active)return;if(loading){again=true;return;}loading=true;
  try{do{again=false;const ticket=++epoch;const [listing,snapshot]=await Promise.all([Bridge.petRuntimeList(),Bridge.petCapabilities()]);if(!alive||ticket!==epoch)return;const loaded:Array<PromiseSettledResult<RuntimeGoal>>=[];for(let start=0;start<listing.goals.length;start+=4){if(!alive||ticket!==epoch)return;loaded.push(...await Promise.allSettled(listing.goals.slice(start,start+4).map(summary=>records.get(summary.id)?.revision===summary.revision?Promise.resolve(records.get(summary.id)!):Bridge.petRuntimeRead(summary.petId,summary.id))));}
    if(!alive||ticket!==epoch)return;unavailable=new Set(records.keys());const seen=new Set(listing.goals.map(g=>g.id));
    loaded.forEach((value,i)=>{const expected=listing.goals[i];if(value.status==='fulfilled'&&value.value.id===expected.id&&value.value.spec.owner.petId===expected.petId){records.set(expected.id,value.value);unavailable.delete(expected.id);}});
    if(listing.complete)for(const id of records.keys())if(!seen.has(id)){records.delete(id);unavailable.delete(id);}
    goals=[...records.values()];descriptors=snapshot.pets;syncPets();draw();if(!listing.complete||loaded.some(v=>v.status==='rejected'))setFeedback(`A listagem está incompleta: ${listing.unavailable} objetivos indisponíveis. Nenhum foi considerado terminado.`);}while(again&&alive&&active);
  }catch(e){setFeedback(message(e));}finally{loading=false;}
 }
 let unsubscribe=()=>{};void onEvent('pet-runtime-progress',()=>{if(active)void refresh();}).then(fn=>{if(alive)unsubscribe=fn;else fn();});syncMusic();
 return {el,actions,refresh,setActive(value:boolean){active=value;el.hidden=!value;if(value)void refresh();},dispose(){alive=false;epoch++;unsubscribe();decisions.dispose();},get goals(){return goals;}};
}
