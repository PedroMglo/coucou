import {Bridge,type RuntimeNativeSystemPolicy} from '../core/bridge';
import {systemSoundGoalPlan,type SystemSoundAction} from '../core/runtime-system';
import type {AudioNode} from '../core/system-audio';
import {h} from './dom';
export function buildRuntimeSystemSound(options:{pet:()=>string;active:()=>boolean;changed:()=>void}){
 let generation=0,owner='',outputs:AudioNode[]=[],disposed=false,busy=false;
 const output=h('select',{'aria-label':'Saída autorizada de Som'})as HTMLSelectElement;
 const operation=h('select',{'aria-label':'Operações da tarefa de Som'},h('option',{value:'volume',text:'Definir volume'}),h('option',{value:'mute',text:'Definir silêncio'}),h('option',{value:'both',text:'Definir volume e silêncio'}))as HTMLSelectElement;operation.value='volume';
 const volume=h('input',{type:'number',min:0,max:100,value:35,'aria-label':'Volume final de Som em percentagem'})as HTMLInputElement;
 volume.value='35';
 const muted=h('input',{type:'checkbox','aria-label':'Silêncio final de Som'})as HTMLInputElement;
 const status=h('p',{role:'status'});
 const read=h('button',{type:'button',text:'Consultar saídas autorizadas',onclick:()=>void load()})as HTMLButtonElement;
 const el=h('fieldset',{},h('legend',{text:'Tarefa de Som sem modelo'}),h('p',{text:'Usa as concessões de Sistema deste pet. As definições do computador continuam nos widgets existentes. Consultar ou guardar esta tarefa não altera o som.'}),read,h('label',{},'Saída exata',output),h('label',{},'Operações',operation),h('label',{},'Volume final',volume),h('label',{},muted,'Silêncio final ativo'),status);el.hidden=true;
 const current=(token:number,pet:string)=>!disposed&&options.active()&&token===generation&&pet===options.pet();
 async function load(){
  if(disposed||!options.active()||busy)return;const pet=options.pet(),token=++generation;busy=true;read.disabled=true;status.textContent='A consultar apenas as saídas autorizadas deste pet…';
  try{const result=await Bridge.petSoundSnapshot(pet);if(!current(token,pet))return;if(!result.available)throw Error(result.error??'As saídas de Som estão indisponíveis.');outputs=structuredClone(result.outputs.filter(n=>n.kind==='output'&&n.controllable));owner=pet;output.replaceChildren(...outputs.map(n=>h('option',{value:n.key,text:`${n.label} · ${n.key}`})));output.value=outputs[0]?.key??'';status.textContent=outputs.length?'Escolhe o estado final. O backend volta a capturar a saída exata na revisão.':'Não há saídas incluídas nas concessões deste pet. Revê as permissões de Sistema.';}
  catch(e){if(current(token,pet)){outputs=[];owner='';output.replaceChildren();status.textContent=String(e).replace(/^Error:\s*/,'');}}
  finally{if(token===generation){busy=false;read.disabled=false;options.changed();}}
 }
 function invalidate(){generation++;busy=false;read.disabled=false;owner='';outputs=[];output.replaceChildren();status.textContent='Consulta as saídas autorizadas do pet selecionado.';}
 function capture(){
  if(disposed||!options.active()||owner!==options.pet()||!outputs.some(n=>n.key===output.value))throw Error('Consulta e escolhe uma saída autorizada deste pet.');
  const actions:SystemSoundAction[]=[];
  if(operation.value!=='mute'){if(!volume.value.trim())throw Error('Indica o volume final.');actions.push({kind:'volume',percent:Number(volume.value)});}
  if(operation.value!=='volume')actions.push({kind:'mute',muted:muted.checked});
  return systemSoundGoalPlan(output.value,actions);
 }
 return {el,capture,invalidate,syncOwner(){if(owner&&owner!==options.pet())invalidate();},dispose(){disposed=true;invalidate();}};
}
export function systemAdmissionReview(p:RuntimeNativeSystemPolicy){if(p.schemaVersion===3)return h('section',{'aria-label':'Revisão exata da tarefa de Sistema'},h('h4',{text:'Inventário — distribuição e kernel, sem modelo'}),h('p',{text:`Pet: ${p.inventory.petId} · concessão Inventory: ${p.inventory.grantRevision}`}),h('p',{text:`Anfitrião: arranque ${p.inventory.host.bootId} · namespaces PID ${p.inventory.host.pidNamespace} e montagem ${p.inventory.host.mountNamespace}. A autorização não acompanha outro anfitrião.`}),h('p',{text:'Uma consulta finita da identificação da distribuição e do kernel em execução, até2s. Origem, hash, cobertura e estado desejado desconhecido ficam no recibo original. Sem processos, serviços, logs, diagnóstico, modelos ou alterações. Saúde e Som mantêm concessões próprias.'}),h('ol',{},...p.steps.map(s=>h('li',{text:`${s.operation.operationId}: ${JSON.stringify(s.operation.arguments)}`}))));if(p.schemaVersion===2)return h('section',{'aria-label':'Revisão exata da tarefa de Sistema'},h('h4',{text:'Saúde — amostra sem modelo'}),h('p',{text:`Pet: ${p.health.petId} · concessão: ${p.health.grantRevision}`}),h('p',{text:`Anfitrião: arranque ${p.health.host.bootId} · namespaces PID ${p.health.host.pidNamespace} e montagem ${p.health.host.mountNamespace}. A aprovação não acompanha outro anfitrião.`}),h('p',{text:p.steps[0]?.operation.operationId==='system.health.baseline'?'Duas amostras de CPU, memória e uptime separadas por pelo menos1s, até6s no total. Resultado medido ou indeterminado; a verificação usa o recibo original. Sem modelos, diagnóstico ou alterações.':'Uma consulta finita de CPU, memória e uptime, com verificação independente. Não autoriza modelos, diagnóstico de segurança, alterações ou outras concessões.'}),h('ol',{},...p.steps.map(s=>h('li',{text:`${s.operation.operationId}: ${JSON.stringify(s.operation.arguments)}`}))));return h('section',{'aria-label':'Revisão exata da tarefa de Sistema'},h('h4',{text:'Sistema — tarefa sem modelo'}),h('p',{text:`Pet: ${p.output.petId} · saída: ${p.output.outputKey} · concessão: ${p.output.grantRevision}`}),h('p',{text:`Dispositivo original: core ${p.output.incarnation.coreCookie}, nó ${p.output.incarnation.nodeId}, serial ${p.output.incarnation.nodeSerial}. A autorização não acompanha um dispositivo que o substitua.`}),h('ol',{},...p.steps.map(s=>h('li',{text:`${s.operation.operationId}: ${JSON.stringify(s.operation.arguments)}`}))),h('p',{text:'Autorizar admite só este objetivo e estes passos dentro dos limites apresentados. Não ativa outras permissões nem inicia um modelo. O arranque é manual, salvo a opção explícita de executar este plano finito quando houver recurso.'}));}
