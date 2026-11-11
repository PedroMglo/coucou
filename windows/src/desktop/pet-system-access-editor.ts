import {Bridge} from '../core/bridge';
import {petBlueprint} from '../core/pet-factory';
import type {PetDocument,SystemGrant} from '../mochi/pet-preferences';
const el=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const n=document.createElement(tag);if(text)n.textContent=text;return n;};
type Hooks={active:()=>boolean;canStart:()=>boolean;generalDirty:()=>boolean;busy:(value:boolean)=>void;saved:(doc:PetDocument)=>void};
type Selection={observe:boolean;volume:boolean;mute:boolean;health:boolean;inventory:boolean};
type Review={petId:string;revision:string;intent:number;fields:string;grants:SystemGrant[];previous?:string};
const key=(value:unknown)=>JSON.stringify(value);
/** Grant management only; hardware settings remain in the native desktop. */
export class PetSystemAccessEditor{
 readonly el=el('fieldset');private inputs=new Map<keyof Selection,HTMLInputElement>();private review=el('div');private details=el('pre');private status=el('p');
 private prepareButton=el('button','Rever acessos ao Sistema');private confirm=el('button','Confirmar acessos ao Sistema');private inspect=el('button','Consultar saídas autorizadas');private inspectHealth=el('button','Consultar saúde autorizada');private inspectBaseline=el('button','Medir CPU e memória (1 s)');private inspectInventory=el('button','Consultar distribuição e kernel autorizados');
 private doc:PetDocument|null=null;private id='';private intent=0;private saving=false;private parentDisabled=false;private choice:Review|null=null;private drafts=new Map<string,Selection>();
 constructor(private hooks:Hooks){
  this.el.append(el('legend','Sistema deste pet · Som, Saúde e Inventário'));
  this.el.append(el('p','Estas concessões abrangem as saídas de Som do computador. Entradas, aplicações e outras componentes do Sistema têm acessos próprios. Saúde tem acesso separado apenas a contadores globais de CPU, memória e uptime. Inventário tem concessão própria só para distribuição e kernel; não inclui processos, serviços ou logs. Guardar não altera dispositivos nem inicia inferência.'));
  for(const[field,label]of [['observe','Observar saídas de Som'],['volume','Ajustar o volume das saídas (0–100%)'],['mute','Silenciar ou reativar as saídas'],['health','Observar CPU, memória e uptime'],['inventory','Observar distribuição e kernel']] as const){const input=el('input');input.type='checkbox';input.setAttribute('aria-label',label);this.inputs.set(field,input);const row=el('label');row.append(el('span',label),input);this.el.append(row);}
  this.prepareButton.type=this.confirm.type=this.inspect.type=this.inspectHealth.type=this.inspectBaseline.type=this.inspectInventory.type='button';this.prepareButton.onclick=()=>this.prepare();this.confirm.onclick=()=>void this.apply();this.inspect.onclick=()=>void this.observe();this.inspectHealth.onclick=()=>void this.observeHealth();this.inspectBaseline.onclick=()=>void this.observeBaseline();this.inspectInventory.onclick=()=>void this.observeInventory();
  this.review.hidden=true;this.review.setAttribute('role','group');this.review.setAttribute('aria-label','Revisão do acesso ao Sistema deste pet');this.review.append(this.details,this.confirm);this.status.setAttribute('role','status');this.el.append(this.prepareButton,this.inspect,this.inspectHealth,this.inspectBaseline,this.inspectInventory,this.review,this.status);
  for(const kind of ['input','change'])this.el.addEventListener(kind,event=>{event.stopPropagation();this.remember();this.invalidate();this.status.textContent='Concessões com alterações por guardar';});
 }
 private fields():Selection{return {observe:this.inputs.get('observe')!.checked,volume:this.inputs.get('volume')!.checked,mute:this.inputs.get('mute')!.checked,health:this.inputs.get('health')!.checked,inventory:this.inputs.get('inventory')!.checked};}
 private selected(doc:PetDocument,id:string):Selection{const grants=doc.config.pets[id]?.systemAccess?.grants??[];return {observe:grants.some(g=>g.resource.kind==='audioOutputs'&&g.operation==='observe'),volume:grants.some(g=>g.resource.kind==='audioOutputs'&&g.operation==='volume'),mute:grants.some(g=>g.resource.kind==='audioOutputs'&&g.operation==='mute'),health:grants.some(g=>g.subplugin==='health'&&g.resource.kind==='hostHealth'&&g.operation==='observe'),inventory:grants.some(g=>g.subplugin==='inventory'&&g.resource.kind==='hostComponents'&&g.operation==='observe')};}
 private remember(){if(!this.doc||!this.id)return;const fields=this.fields();if(key(fields)===key(this.selected(this.doc,this.id)))this.drafts.delete(this.id);else this.drafts.set(this.id,fields);}
 get dirty(){this.remember();return this.drafts.size>0;}
 reset(){this.drafts.clear();this.doc=null;this.id='';this.invalidate();}
 invalidate(){this.choice=null;this.review.hidden=true;this.confirm.disabled=true;this.intent++;}
 setParentDisabled(value:boolean){this.parentDisabled=value;this.sync();}
 private sync(){this.el.disabled=this.parentDisabled||this.saving;this.prepareButton.disabled=this.inspect.disabled=this.inspectHealth.disabled=this.inspectBaseline.disabled=this.inspectInventory.disabled=this.parentDisabled||this.saving;this.confirm.disabled=this.parentDisabled||this.saving||!this.choice;}
 show(doc:PetDocument,id:string){this.remember();this.invalidate();this.doc=structuredClone(doc);this.id=id;this.el.hidden=petBlueprint(id)?.runtime.kind==='observer'||petBlueprint(id)?.retired===true;const value=this.drafts.get(id)??this.selected(doc,id);for(const[field,input]of this.inputs)input.checked=value[field];this.status.textContent=this.drafts.has(id)?'Concessões com alterações por guardar':doc.config.pets[id]?.systemAccess?(doc.config.pets[id].systemAccess!.enabled?'Concessões próprias guardadas':'Concessões suspensas. Revê e confirma para permitir novos usos'):'Sem concessões próprias nesta rota. O acesso legado de música conserva o contrato anterior.';this.sync();}
 private prepare(){
  this.invalidate();if(this.el.hidden||!this.doc||!this.hooks.canStart()||this.saving||this.parentDisabled)return;
  if(this.hooks.generalDirty()){this.status.textContent='Guarda primeiro as restantes preferências e escolhas de motor.';return;}
  this.remember();const fields=this.fields();if((fields.volume||fields.mute)&&!fields.observe){this.status.textContent='Autoriza a observação das saídas necessárias à operação ou retira volume e silêncio.';return;}
  // Editing the broad output scope never discards exact-resource grants.
  const grants=(this.doc.config.pets[this.id]?.systemAccess?.grants??[]).filter(g=>g.resource.kind!=='audioOutputs'&&g.resource.kind!=='hostHealth'&&g.resource.kind!=='hostComponents');for(const operation of ['observe','volume','mute']as const)if(fields[operation])grants.push({subplugin:'sound',operation,resource:{kind:'audioOutputs'}});if(fields.health)grants.push({subplugin:'health',operation:'observe',resource:{kind:'hostHealth'}});if(fields.inventory)grants.push({subplugin:'inventory',operation:'observe',resource:{kind:'hostComponents'}});
  this.choice={petId:this.id,revision:this.doc.revision,intent:this.intent,fields:key(fields),grants:structuredClone(grants),previous:this.doc.config.pets[this.id]?.systemAccess?.revision};
  const describe=(items:SystemGrant[])=>items.length?items.map(g=>`${g.operation==='observe'?'Observar':g.operation==='volume'?'Volume 0–100%':'Silêncio'} · ${g.resource.kind==='audioOutputs'?'todas as saídas de Som':g.resource.kind==='hostHealth'?'Saúde · contadores globais CPU/memória/uptime':g.resource.kind==='hostComponents'?'Inventário · distribuição e kernel (sem processos/serviços/logs)':g.resource.key}`).join('\n'):'Sem concessões nesta rota';
  this.details.textContent=`Pet: ${this.doc.config.pets[this.id].name} (${this.id})\n\nAtual:\n${describe(this.doc.config.pets[this.id]?.systemAccess?.grants??[])}\n\nDepois:\n${describe(grants)}\n\nOutros pets conservam os seus acessos. Pedidos com a revisão anterior deixam de poder iniciar novas operações. Esta escolha não desfaz efeitos anteriores.`;this.review.hidden=false;this.status.textContent='Revê o pet, as operações e os recursos antes de confirmar.';this.sync();
 }
 private async apply(){
  const c=this.choice;if(this.el.hidden||!c||this.saving||this.parentDisabled||!this.doc||!this.hooks.canStart()||this.hooks.generalDirty()||c.petId!==this.id||c.revision!==this.doc.revision||c.intent!==this.intent||c.fields!==key(this.fields())){this.invalidate();return;}
  this.saving=true;this.hooks.busy(true);this.sync();
  try{const doc=await Bridge.petSystemAccessSet(c.petId,c.revision,structuredClone(c.grants));const policy=doc.config.pets[c.petId]?.systemAccess;if(doc.config.version!==2||!policy||policy.schemaVersion!==1||policy.enabled!==true||!/^[0-9a-f]{32}$/.test(policy.revision)||policy.revision===c.previous||key(policy.grants)!==key(c.grants))throw Error('A concessão não foi confirmada. Recarrega antes de repetir.');this.drafts.delete(c.petId);this.invalidate();this.doc=structuredClone(doc);this.hooks.saved(doc);if(this.hooks.active()&&this.id===c.petId)this.status.textContent='Concessão própria guardada. Nenhuma definição do computador foi alterada.';}
  catch(error){this.invalidate();this.status.textContent=String(error).replace(/^Error:\s*/,'');}finally{this.saving=false;this.hooks.busy(false);this.sync();}
 }
 private async observeInventory(){
  if(this.el.hidden||!this.doc||this.saving||this.parentDisabled||!this.hooks.canStart())return;
  const id=this.id,intent=this.intent;this.saving=true;this.sync();
  try{
   const result=await Bridge.petHostInventory(id);if(!this.hooks.active()||id!==this.id||intent!==this.intent)return;
   if(result.petId!==id||result.kind!=='systemHostInventory'||result.inferenceUsed!==false||result.desiredState!=='unknownNotCompared')throw Error('O inventário não corresponde ao pet e âmbito revistos.');
   const d=result.distribution,k=result.kernelType,release=result.kernelRelease;
   this.status.textContent=['Pet: '+id+' · '+new Date(result.observedAt).toLocaleString(),
    d.value?'Distribuição: '+(d.value.prettyName??d.value.id)+' · ID '+d.value.id+' · versão '+(d.value.versionId??'não fornecida'):'Distribuição indisponível: '+(d.error??'sem evidência'),
    'Origem: '+(d.path??d.source)+' · gestão observada externa.',
    k.value&&release.value?'Kernel em execução: '+k.value+' '+release.value:'Kernel com cobertura parcial: '+(k.value??k.error)+' · '+(release.value??release.error),
    'Kernel dinâmico observado. Estado desejado/sysworkspace não comparado. Não é diagnóstico de vulnerabilidade nem autorização de alterações.'].join('\n');
  }catch(error){if(id===this.id&&intent===this.intent)this.status.textContent=String(error).replace(/^Error:\s*/,'');}
  finally{this.saving=false;this.sync();}
 }
 private async observeBaseline(){
  if(this.el.hidden||!this.doc||this.saving||this.parentDisabled||!this.hooks.canStart())return;
  const id=this.id,intent=this.intent;this.saving=true;this.sync();this.status.textContent='A recolher duas amostras autorizadas; intervalo mínimo de1s…';
  try{
   const result=await Bridge.petHealthBaseline(id);if(!this.hooks.active()||id!==this.id||intent!==this.intent)return;
   if(result.before.petId!==id||result.after.petId!==id||result.before.grantRevision!==result.after.grantRevision||result.inferenceUsed!==false||result.intervalMs<1000||result.elapsedMs>=6000)throw Error('A medição não corresponde ao pet, revisão e intervalo revistos.');
   const cpu=result.cpuBusyBasisPoints,mem=result.memoryAvailableDeltaBytes;
   this.status.textContent=['Pet: '+id+' · '+new Date(result.after.observedAt).toLocaleString(),
    cpu===null?'CPU: indeterminado; faltam contadores ou um intervalo consistente.':'CPU agregada estimada: '+(cpu/100).toFixed(2)+'% (idle/iowait excluídos).',
    mem===null?'Variação de memória: indeterminada.':'Variação da memória disponível: '+(mem/1048576).toFixed(0)+' MiB.',
    'Intervalo mínimo: '+result.intervalMs+'ms · recolha total: '+result.elapsedMs+'ms.',
    'Comparação curta de contadores globais; não diagnostica, otimiza ou altera o sistema.'].join('\n');
  }catch(error){if(id===this.id&&intent===this.intent)this.status.textContent=String(error).replace(/^Error:\s*/,'');}
  finally{this.saving=false;this.sync();}
 }
 private async observeHealth(){
  if(this.el.hidden||!this.doc||this.saving||this.parentDisabled||!this.hooks.canStart())return;
  const id=this.id,intent=this.intent;this.saving=true;this.sync();
  try{
   const result=await Bridge.petHealthSnapshot(id);if(!this.hooks.active()||id!==this.id||intent!==this.intent)return;
   if(result.petId!==id||result.inferenceUsed!==false)throw Error('A observação não corresponde ao pet revisto.');
   const mem=result.memory.value,cpu=result.cpu.value,uptime=result.uptime.value;
   this.status.textContent=['Pet: '+id+' · '+new Date(result.observedAt).toLocaleString(),
    mem?'Memória disponível: '+(mem.availableBytes/1048576).toFixed(0)+' de '+(mem.totalBytes/1048576).toFixed(0)+' MiB':'Memória indisponível: '+(result.memory.error??'sem amostra'),
    cpu?'CPU: '+cpu.totalTicks+' ticks cumulativos ('+cpu.idleTicks+' idle); requer baseline para calcular utilização':'CPU indisponível: '+(result.cpu.error??'sem amostra'),
    uptime?'Uptime: '+Math.floor(uptime.seconds)+' s':'Uptime indisponível: '+(result.uptime.error??'sem amostra'),
    ...result.findings.map(f=>f.id+': '+f.recommendation),
    'Amostra sequencial de contadores globais. Não é um inventário de processos, segurança ou estado desejado.'].join('\n');
  }catch(error){if(id===this.id&&intent===this.intent)this.status.textContent=String(error).replace(/^Error:\s*/,'');}
  finally{this.saving=false;this.sync();}
 }
 private async observe(){
  if(this.el.hidden||!this.doc||this.saving||this.parentDisabled||!this.hooks.canStart())return;const id=this.id,intent=this.intent;this.saving=true;this.sync();
  try{const result=await Bridge.petSoundSnapshot(id);if(!this.hooks.active()||id!==this.id||intent!==this.intent)return;this.status.textContent=result.available?(result.outputs.length?result.outputs.map(n=>n.label).join(' · '):'Nenhuma saída no âmbito autorizado.'):(result.error??'As saídas de Som estão indisponíveis.');}
  catch(error){if(id===this.id&&intent===this.intent)this.status.textContent=String(error).replace(/^Error:\s*/,'');}finally{this.saving=false;this.sync();}
 }
}
