import {Bridge,emitToWindow} from '../core/bridge';
import {PetPreferences,profileProvider,type PetDocument,type PetPreference,type PetPermissions} from '../mochi/pet-preferences';
import {OllamaModelSelection} from '../core/ollama-selection';
const element=<K extends keyof HTMLElementTagNameMap>(tag:K,text?:string)=>{const el=document.createElement(tag);if(text)el.textContent=text;return el;};
/** Edits preferences only. Opening, listing models or saving never starts a task. */
export class PetSettings {
 readonly el=element('section');private body=element('div');private error=element('p');private path=element('p');
 private pet=element('select');private name=element('input');private aliases=element('input');private mode=element('select');
 private models=[element('input'),element('input')];private efforts=[element('select'),element('select')];
 private choices=element('datalist');private profiles=element('div');private save=element('button','Guardar');private catalog=element('button','Atualizar modelos da conta');
 private reload=element('button','Recarregar ficheiro');
 private status=element('p');private document:PetDocument|null=null;private selected='';private ticket=0;private saving=false;private loading=false;
 private grants=new Map<keyof PetPermissions,HTMLInputElement>();
 get loaded(){return this.document!==null;}
 selectPet(id:string){if(!this.document){void this.load(id);return;}if(this.document.config.pets[id]){this.keep();this.choose(id);}}
 constructor(){
  this.el.className='assistant-prefs';this.el.hidden=true;this.body.className='assistant-prefs-body';this.error.className='assistant-task-error';this.error.hidden=true;this.path.className='assistant-task-context';
  this.error.setAttribute('role','alert');this.status.setAttribute('role','status');this.status.className='pet-save-status';this.save.className='pet-primary-action';
  this.body.addEventListener('input',()=>this.dirty());this.body.addEventListener('change',()=>this.dirty());
  this.pet.setAttribute('aria-label','Pet a configurar');this.pet.onchange=()=>{this.keep();this.choose(this.pet.value);};
  const field=(label:string,input:HTMLElement)=>{const l=element('label');l.append(element('span',label),input);return l;};
  this.name.maxLength=40;this.name.setAttribute('aria-label','Nome do pet');this.aliases.setAttribute('aria-label','Nomes de voz do pet');
  for(const [value,label]of [['auto','Automático'],['fast','Rápido'],['reasoning','Raciocínio elevado']]){const o=element('option',label);o.value=value;this.mode.append(o);}
  this.mode.setAttribute('aria-label','Modo predefinido do pet');
  const identity=element('fieldset');identity.append(element('legend','Identidade'),field('Pet',this.pet),field('Nome apresentado',this.name),field('Nomes de voz, separados por vírgulas',this.aliases));this.body.append(identity);
  this.profiles.append(element('h2','Como responde'),field('Voz e pedidos escritos',this.mode));
  this.choices.id='pet-profile-models';
  for(let i=0;i<2;i++){
   const label=i?'Raciocínio elevado':'Rápido';this.models[i].setAttribute('aria-label',`Modelo ${label}`);this.models[i].setAttribute('list',this.choices.id);this.models[i].maxLength=256;
   for(const effort of ['', 'low','medium','high','xhigh','max']){const o=element('option',effort||'Esforço nativo');o.value=effort;this.efforts[i].append(o);}
   this.efforts[i].setAttribute('aria-label',`Esforço ${label}`);this.profiles.append(field(`Modelo · ${label}`,this.models[i]),field(`Esforço · ${label}`,this.efforts[i]));
  }
  this.catalog.onclick=()=>void this.loadModels();const use=element('button','Usar este perfil no chat');use.onclick=async()=>{
   const selected=this.selected;if(!await this.store())return;
   if(selected==='ai_ollama'){
    use.disabled=true;this.status.textContent='A aplicar o perfil…';
    try{this.status.textContent=await OllamaModelSelection.select('@default')?'Perfil aplicado ao chat':'Perfil guardado; foi mantida uma escolha de modelo mais recente.';}catch(error){this.status.textContent='Perfil guardado; não foi aplicado ao chat.';this.failure(error);}finally{use.disabled=false;}
   }else void emitToWindow('island','pet-use-profile',{provider:({agent_codex:'codex',agent_copilot:'copilot',agent_antigravity:'antigravity',agent_gemini:'gemini-cli',integration_claude:'claude-cli'} as Record<string,string>)[selected]??selected});
  };
  const modelActions=element('div');modelActions.className='pet-profile-actions';modelActions.append(this.catalog,use);
  this.profiles.append(modelActions,this.choices,element('p','Automático escolhe entre rapidez e raciocínio conforme o pedido. O chat usa este perfil, a menos que escolhas outro modelo nessa conversa.'));
  const learn=element('button','Aprender nome por voz');learn.onclick=async()=>{
   this.keep();const id=this.selected,p=this.document?.config.pets[id];if(!p || !profileProvider(p.provider))return;
   const provider=({agent_codex:'codex',agent_copilot:'copilot',agent_antigravity:'antigravity',agent_gemini:'gemini-cli',integration_claude:'claude-cli',ai_ollama:'ollama'} as Record<string,string>)[id]??id;
   learn.disabled=true;this.error.hidden=true;
   try{const spelling=await Bridge.voiceLearnAlias(provider);if(spelling){p.aliases=[...new Set([...p.aliases,spelling])];if(this.selected===id)this.aliases.value=p.aliases.join(', ');this.dirty();}}
   catch(error){this.failure(error);}finally{learn.disabled=false;}
  };this.profiles.append(learn);
  const permissions=element('fieldset');permissions.append(element('legend','Acessos deste pet'));
  for(const [key,label]of Object.entries({documents:'Receber e ler documentos arrastados',emailDrafts:'Preparar rascunhos de email',music:'Controlar música',openApps:'Abrir aplicações'})){
   const input=element('input');input.type='checkbox';input.setAttribute('aria-label',label);this.grants.set(key as keyof PetPermissions,input);permissions.append(field(label,input));
  }
  permissions.append(element('p','Estes acessos controlam as integrações do Mochi. As ferramentas e permissões do CLI continuam a ser geridas pelo próprio Codex/Gravity. O email abre como rascunho para reveres e enviares.'));
  this.body.append(this.profiles,permissions,this.path);const footer=element('footer');this.reload.onclick=()=>void this.load();this.save.onclick=()=>void this.store();footer.append(this.reload,this.save);
  footer.className='pet-save-actions';footer.prepend(this.status);this.el.append(this.body,this.error,footer);
 }
 private dirty(){if(this.document&&!this.loading&&!this.saving){this.status.textContent='Alterações por guardar';this.el.dataset.dirty='true';}}
 async load(petId?:string){
  if(this.loading||this.saving)return;this.selected=petId??this.selected;this.loading=true;this.error.hidden=true;this.status.textContent='A carregar os perfis…';this.el.setAttribute('aria-busy','true');
  this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=true);
  try{
   const doc=await Bridge.petConfiguration();if(!doc)throw new Error('A configuração dos pets não está disponível.');
   const roster=await Bridge.petRoster();const configured=Object.entries(doc.config.pets).filter(([id])=>roster?.includes(id));
   if(!configured.length)throw new Error('Não há pets configurados para apresentar.');
   this.document=structuredClone(doc);this.pet.replaceChildren();
   for(const [id,p]of configured){const o=element('option',p.name);o.value=id;this.pet.append(o);}
   this.path.textContent=doc.path;this.path.title=doc.path;
   const wanted=petId??this.selected;this.choose(configured.some(([id])=>id===wanted)?wanted:configured[0][0]);this.status.textContent='';this.el.dataset.dirty='false';
  }catch(error){this.failure(error);this.status.textContent='Não foi possível carregar os perfis.';}finally{this.loading=false;this.el.setAttribute('aria-busy','false');this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=!this.document&&el!==this.reload);}
 }
 private keep(){
  const p=this.document?.config.pets[this.selected];if(!p)return;
  p.name=this.name.value.trim();p.aliases=this.aliases.value.split(',').map(x=>x.trim()).filter(Boolean);p.mode=this.mode.value as PetPreference['mode'];
  p.fast={model:this.models[0].value.trim(),effort:this.efforts[0].value};p.reasoning={model:this.models[1].value.trim(),effort:this.efforts[1].value};
  p.permissions=Object.fromEntries([...this.grants].map(([key,input])=>[key,input.checked])) as unknown as PetPermissions;
 }
 private choose(id:string){
  const p=this.document?.config.pets[id];if(!p)return;this.selected=id;this.pet.value=id;this.name.value=p.name;this.aliases.value=p.aliases.join(', ');this.mode.value=p.mode;
  this.models[0].value=p.fast.model;this.models[1].value=p.reasoning.model;this.efforts[0].value=p.fast.effort;this.efforts[1].value=p.reasoning.effort;
  for(const [key,input]of this.grants)input.checked=p.permissions?.[key]??false;
  this.profiles.hidden=!profileProvider(p.provider);for(const effort of this.efforts)effort.parentElement!.hidden=!['codex','copilot'].includes(p.provider);this.choices.replaceChildren();this.ticket++;this.catalog.disabled=false;
 }
 private failure(error:unknown){this.error.textContent=String(error).replace(/^Error:\s*/,'');this.error.hidden=false;}
 private async loadModels(){
  const provider=this.document?.config.pets[this.selected]?.provider;if(!provider)return;const ticket=++this.ticket;this.catalog.disabled=true;this.error.hidden=true;
  try{const models=await Bridge.chatModels(provider);if(ticket!==this.ticket)return;this.choices.replaceChildren();for(const m of models??[]){const o=element('option');o.value=m.id;o.label=m.label;this.choices.append(o);}if(!models?.length)throw new Error('A conta não devolveu modelos disponíveis.');}
  catch(error){if(ticket===this.ticket)this.failure(error);}finally{if(ticket===this.ticket)this.catalog.disabled=false;}
 }
 private async store(){
  if(!this.document || this.saving||this.loading)return false;this.keep();this.saving=true;this.status.textContent='A guardar…';this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=true);this.error.hidden=true;
  try{const doc=await Bridge.petConfigurationSave(this.document.config,this.document.revision);if(doc){this.document=structuredClone(doc);PetPreferences.update(doc.config);this.pet.options[this.pet.selectedIndex].textContent=this.name.value.trim();this.status.textContent='Alterações guardadas';this.el.dataset.dirty='false';return true;}throw new Error('Não foi possível guardar as alterações.');}
  catch(error){this.status.textContent='Alterações por guardar';this.failure(error);return false;}finally{this.saving=false;this.el.querySelectorAll<HTMLInputElement|HTMLButtonElement|HTMLSelectElement>('input,select,button').forEach(el=>el.disabled=false);}
 }
}
