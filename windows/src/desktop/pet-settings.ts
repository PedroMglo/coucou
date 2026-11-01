import {Bridge,emitToWindowOrThrow} from '../core/bridge';
import {PetPreferences,profileProvider,type PetDocument,type PetPreference,type PetPermissions} from '../mochi/pet-preferences';
import {petBlueprint,taskProviderForPet} from '../core/pet-factory';
import {OllamaModelSelection} from '../core/ollama-selection';
const element=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const el=document.createElement(tag);if(text)el.textContent=text;return el;};
/** Edits preferences only. Opening, listing models or saving never starts a task. */
export class PetSettings {
 readonly el=element('section');private body=element('div');private error=element('p');private path=element('p');
 private pet=element('select');private name=element('input');private aliases=element('input');private mode=element('select');
 private models=[element('input'),element('input')];private efforts=[element('select'),element('select')];private execution=element('select');private executionGroup=element('fieldset');
 private engine=element('input');private engineGroup=element('fieldset');
 private choices=element('datalist');private profiles=element('div');private save=element('button','Guardar');private catalog=element('button','Atualizar modelos da conta');
 private reload=element('button','Recarregar ficheiro');
 private discard=element('div');private use=element('button','Usar este perfil no chat');private active=true;private intent=0;private requested='';private applying=false;
 private status=element('p');private document:PetDocument|null=null;private selected='';private ticket=0;private saving=false;private loading=false;
 private grants=new Map<keyof PetPermissions,HTMLInputElement>();
 get loaded(){return this.document!==null;}
 selectPet(id:string){this.requested=id;this.intent++;this.closeDiscard();if(!this.document){void this.load(id);return;}if(this.document.config.pets[id]){this.keep();this.choose(id);}}
 setActive(active:boolean){if(this.active===active)return;this.active=active;this.intent++;if(!active)this.closeDiscard();}
 constructor(){
  this.el.className='assistant-prefs';this.el.hidden=true;this.body.className='assistant-prefs-body';this.error.className='assistant-task-error';this.error.hidden=true;this.path.className='assistant-task-context';
  this.error.setAttribute('role','alert');this.status.setAttribute('role','status');this.status.className='pet-save-status';this.save.className='pet-primary-action';
  this.body.addEventListener('input',event=>{if(event.target!==this.pet)this.dirty();});this.body.addEventListener('change',event=>{if(event.target!==this.pet)this.dirty();});
  this.pet.setAttribute('aria-label','Pet a configurar');this.pet.onchange=()=>{this.keep();this.choose(this.pet.value);};
  const field=(label:string,input:HTMLElement)=>{const l=element('label');l.append(element('span',label),input);return l;};
  this.name.maxLength=40;this.name.setAttribute('aria-label','Nome do pet');this.aliases.setAttribute('aria-label','Nomes de voz do pet');
  for(const [value,label]of [['auto','Automático'],['fast','Rápido'],['reasoning','Raciocínio elevado']]){const o=element('option',label);o.value=value;this.mode.append(o);}
  this.mode.setAttribute('aria-label','Modo predefinido do pet');
  const identity=element('fieldset');identity.append(element('legend','Identidade'),field('Pet',this.pet),field('Nome apresentado',this.name),field('Nomes de voz, separados por vírgulas',this.aliases));this.body.append(identity);
  this.engine.type='checkbox';this.engine.setAttribute('aria-label','Associar este pet ao LM Studio');this.engineGroup.append(element('legend','Motor deste pet'),field('Usar o servidor LM Studio configurado',this.engine),element('p','A associação é guardada apenas ao escolher Guardar. O servidor e o modelo precisam de confirmação nativa; guardar não inicia inferência nem tarefas.'));this.body.append(this.engineGroup);this.engine.onchange=()=>{this.keep();this.choose(this.selected);this.dirty();};
  this.profiles.append(element('h2','Como responde'),field('Voz e pedidos escritos',this.mode));
  this.choices.id='pet-profile-models';
  for(let i=0;i<2;i++){
   const label=i?'Raciocínio elevado':'Rápido';this.models[i].setAttribute('aria-label',`Modelo ${label}`);this.models[i].setAttribute('list',this.choices.id);this.models[i].maxLength=256;
   for(const effort of ['', 'low','medium','high','xhigh','max']){const o=element('option',effort||'Esforço nativo');o.value=effort;this.efforts[i].append(o);}
   this.efforts[i].setAttribute('aria-label',`Esforço ${label}`);this.profiles.append(field(`Modelo · ${label}`,this.models[i]),field(`Esforço · ${label}`,this.efforts[i]));
  }
  this.catalog.onclick=()=>void this.loadModels();const use=this.use;use.onclick=async()=>{
   if(this.applying)return;const selected=this.selected,intent=this.intent;this.applying=true;
   const current=()=>this.active&&!document.hidden&&selected===this.selected&&intent===this.intent;
   try{
    if(!await this.store()||!current())return;use.disabled=true;
    if(selected==='ai_ollama'){
     this.status.textContent='A aplicar o perfil…';
     try{const applied=await OllamaModelSelection.select('@default');if(current())this.status.textContent=applied?'Perfil aplicado ao chat':'Perfil guardado; foi mantida uma escolha de modelo mais recente.';}
     catch(error){if(current()){this.status.textContent='Perfil guardado; não foi aplicado ao chat.';this.failure(error);}}
    }else{
     try{await emitToWindowOrThrow('island','pet-use-profile',{provider:taskProviderForPet(selected)??selected});if(current())this.status.textContent='Perfil guardado; pedido de aplicação enviado ao chat.';}
     catch{if(current()){this.status.textContent='Perfil guardado; não foi possível enviar o pedido ao chat.';this.failure('O chat não confirmou a receção do pedido. Podes voltar a tentar.');}}
    }
   }finally{this.applying=false;use.disabled=this.loading||this.saving||!this.document;}
  };
  const modelActions=element('div');modelActions.className='pet-profile-actions';modelActions.append(this.catalog,use);
  this.profiles.append(modelActions,this.choices,element('p','Automático escolhe entre rapidez e raciocínio conforme o pedido. O chat usa este perfil, a menos que escolhas outro modelo nessa conversa.'));
  const learn=element('button','Aprender nome por voz');learn.onclick=async()=>{
   this.keep();const id=this.selected,p=this.document?.config.pets[id];if(!p || !profileProvider(p.provider))return;
   const provider=taskProviderForPet(id)??id;
   learn.disabled=true;this.error.hidden=true;
   try{const spelling=await Bridge.voiceLearnAlias(provider);if(spelling){p.aliases=[...new Set([...p.aliases,spelling])];if(this.selected===id)this.aliases.value=p.aliases.join(', ');this.dirty();}}
   catch(error){this.failure(error);}finally{learn.disabled=false;}
  };this.profiles.append(learn);
  this.execution.setAttribute('aria-label','Execução de tarefas locais');for(const[value,label]of [['disabled','Só conversa'],['ask','Pedir autorização para cada ação']]){const option=element('option',label);option.value=value;this.execution.append(option);}
  this.executionGroup.append(element('legend','Tarefas no computador'),field('Execução local',this.execution),element('p','Cada comando precisa da tua aprovação. O pedido mostra o programa, os argumentos e a pasta de trabalho. O modelo também precisa de suportar ferramentas.'));this.body.append(this.executionGroup);
  const permissions=element('fieldset');permissions.append(element('legend','Acessos deste pet'));
  for(const [key,label]of Object.entries({documents:'Receber e ler documentos arrastados',emailDrafts:'Preparar rascunhos de email',music:'Controlar música',openApps:'Abrir aplicações'})){
   const input=element('input');input.type='checkbox';input.setAttribute('aria-label',label);this.grants.set(key as keyof PetPermissions,input);permissions.append(field(label,input));
  }
  permissions.append(element('p','Estes acessos controlam as integrações do Mochi. As ferramentas e permissões do CLI continuam a ser geridas pelo próprio Codex/Gravity. O email abre como rascunho para reveres e enviares.'));
  this.body.append(this.profiles,permissions,this.path);const footer=element('footer');this.reload.onclick=()=>{if(this.el.dataset.dirty==='true'){this.discard.hidden=false;this.discard.querySelector<HTMLButtonElement>('button')?.focus();}else void this.load();};this.save.onclick=()=>void this.store();footer.append(this.reload,this.save);
  this.discard.hidden=true;this.discard.className='pet-discard-confirmation pets-management-confirmation';this.discard.setAttribute('role','group');this.discard.setAttribute('aria-label','Descartar alterações dos perfis');
  this.discard.append(element('p','Recarregar vai descartar todas as alterações por guardar nos perfis. Queres mantê-las?'));
  const keep=element('button','Manter alterações'),discard=element('button','Descartar e recarregar'),discardActions=element('div');keep.type=discard.type='button';keep.onclick=()=>this.closeDiscard(true);discard.onclick=()=>{this.closeDiscard();void this.load();};discardActions.className='pet-profile-actions';discardActions.append(keep,discard);this.discard.append(discardActions);
  this.discard.onkeydown=event=>{if(event.key==='Escape'){event.preventDefault();event.stopPropagation();this.closeDiscard(true);}};
  footer.className='pet-save-actions';footer.prepend(this.status);this.el.append(this.body,this.error,this.discard,footer);
 }
 private closeDiscard(focus=false){this.discard.hidden=true;if(focus&&document.hasFocus())this.reload.focus();}
 private dirty(){if(this.document&&!this.loading&&!this.saving){this.status.textContent='Alterações por guardar';this.el.dataset.dirty='true';}}
 async load(petId?:string){
  if(petId){this.requested=petId;this.intent++;}if(this.loading||this.saving)return;this.closeDiscard();this.loading=true;this.error.hidden=true;this.status.textContent='A carregar os perfis…';this.el.setAttribute('aria-busy','true');
  this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=true);
  try{
   const doc=await Bridge.petConfiguration();if(!doc)throw new Error('A configuração dos pets não está disponível.');
   const roster=await Bridge.petRoster();const configured=Object.entries(doc.config.pets).filter(([id])=>roster?.includes(id));
   if(!configured.length)throw new Error('Não há pets configurados para apresentar.');
   this.document=structuredClone(doc);this.pet.replaceChildren();
   for(const [id,p]of configured){const o=element('option',p.name);o.value=id;this.pet.append(o);}
   this.path.textContent=doc.path;this.path.title=doc.path;
   const wanted=this.requested||this.selected;this.choose(configured.some(([id])=>id===wanted)?wanted:configured[0][0]);this.status.textContent='';this.el.dataset.dirty='false';
  }catch(error){this.failure(error);this.status.textContent='Não foi possível carregar os perfis.';}finally{this.loading=false;this.el.setAttribute('aria-busy','false');this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=!this.document&&el!==this.reload);}
 }
 private keep(){
  const p=this.document?.config.pets[this.selected];if(!p)return;
  if(this.selected==='ai_lmstudio')p.provider=this.engine.checked?'lmstudio':'';
  p.name=this.name.value.trim();p.aliases=this.aliases.value.split(',').map(x=>x.trim()).filter(Boolean);p.mode=this.mode.value as PetPreference['mode'];
  p.fast={model:this.models[0].value.trim(),effort:this.efforts[0].value};p.reasoning={model:this.models[1].value.trim(),effort:this.efforts[1].value};
  p.permissions=Object.fromEntries([...this.grants].map(([key,input])=>[key,input.checked])) as unknown as PetPermissions;
  if(p.provider==='ollama')p.localExecution={...p.localExecution,mode:this.execution.value==='ask'?'ask':'disabled'};
 }
 private choose(id:string){
  const p=this.document?.config.pets[id];if(!p)return;this.intent++;this.closeDiscard();this.status.textContent=this.el.dataset.dirty==='true'?'Alterações por guardar':'';this.error.hidden=true;this.requested=id;this.selected=id;this.pet.value=id;this.name.value=p.name;this.aliases.value=p.aliases.join(', ');this.mode.value=p.mode;
  this.engineGroup.hidden=id!=='ai_lmstudio';this.engine.checked=p.provider==='lmstudio';
  this.models[0].value=p.fast.model;this.models[1].value=p.reasoning.model;this.efforts[0].value=p.fast.effort;this.efforts[1].value=p.reasoning.effort;this.executionGroup.hidden=p.provider!=='ollama';this.execution.value=p.localExecution?.mode??'disabled';
  for(const [key,input]of this.grants)input.checked=p.permissions?.[key]??false;
  const presentation=petBlueprint(id)?.presentation;
  this.el.dataset.petMold=petBlueprint(id)?.mold??'unknown';
  this.profiles.hidden=!presentation?.profiles||!profileProvider(p.provider);
  for(const[key,input]of this.grants){input.parentElement!.hidden=!(presentation?.permissions.includes(key)??true);if(key==='music'){const label=presentation?.musicLabel??'Controlar música';input.setAttribute('aria-label',label);input.parentElement!.firstElementChild!.textContent=label;}}for(const effort of this.efforts)effort.parentElement!.hidden=!['codex','copilot'].includes(p.provider);this.choices.replaceChildren();this.ticket++;this.catalog.disabled=false;
 }
 private failure(error:unknown){this.error.textContent=String(error).replace(/^Error:\s*/,'');this.error.hidden=false;}
 private async loadModels(){
  const provider=this.document?.config.pets[this.selected]?.provider;if(!provider)return;const ticket=++this.ticket;this.catalog.disabled=true;this.error.hidden=true;
  try{const models=await Bridge.chatModels(provider);if(ticket!==this.ticket)return;this.choices.replaceChildren();for(const m of models??[]){const o=element('option');o.value=m.id;o.label=m.label;this.choices.append(o);}if(!models?.length)throw new Error('A conta não devolveu modelos disponíveis.');}
  catch(error){if(ticket===this.ticket)this.failure(error);}finally{if(ticket===this.ticket)this.catalog.disabled=false;}
 }
 private async store(){
  if(!this.document || this.saving||this.loading)return false;const selected=this.selected,intent=this.intent,current=()=>this.active&&selected===this.selected&&intent===this.intent;this.keep();this.saving=true;this.status.textContent='A guardar…';this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=true);this.error.hidden=true;
  try{const doc=await Bridge.petConfigurationSave(this.document.config,this.document.revision);if(doc){this.document=structuredClone(doc);PetPreferences.update(doc.config);for(const option of this.pet.options){const p=doc.config.pets[option.value];if(p)option.textContent=p.name;}if(current())this.status.textContent='Alterações guardadas';this.el.dataset.dirty='false';this.closeDiscard();return true;}throw new Error('Não foi possível guardar as alterações.');}
  catch(error){if(current()){this.status.textContent='Alterações por guardar';this.failure(error);}return false;}finally{this.saving=false;this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=false);}
 }
}
