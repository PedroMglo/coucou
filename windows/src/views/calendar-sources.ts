import {Bridge,type PetSourceOwner,type PetSourceDocument,type PetSourceRegistry} from '../core/bridge';
import {h} from './dom';
type Transport=Pick<typeof Bridge,'petSourceConfiguration'|'petSourceConfigurationSave'>;
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
const ownerSame=(a:PetSourceOwner,b:PetSourceOwner)=>a.petId===b.petId&&a.scope===b.scope&&a.conversationId===b.conversationId;
const owns=(owners:PetSourceOwner[],owner:PetSourceOwner)=>owners.some(o=>ownerSame(o,owner!));
const bytes=(s:string)=>new TextEncoder().encode(s).length;
const pathValid=(s:string)=>s.startsWith('/')&&bytes(s)<=4096&&!/\p{Cc}/u.test(s)&&!/\/(\.|\.\.)(\/|$)/.test(s)&&/\.ics$/i.test(s);
function documentValid(d:PetSourceDocument){return !!d&&(d.revision==='missing'||/^[a-f0-9]{64}$/.test(d.revision));}
/** Native-user enrollment metadata only. Rendering/loading never opens ICS,
 * queries a model or enrolls a source. One exact owner/revision survives IO. */
export function buildCalendarSources(options:{changed?:()=>void;transport?:Transport;id?:()=>string;canStart?:()=>boolean}={}){
 const allowed=()=>options.canStart?.()??true;
 const transport=options.transport??Bridge,changed=options.changed??(()=>{}),id=options.id??(()=>crypto.randomUUID());
 let owner:PetSourceOwner|null=null,epoch=0,busy=false,doc:PetSourceDocument|null=null,draft:PetSourceRegistry|null=null,review:PetSourceRegistry|null=null;
 const status=h('p',{role:'status'}),rows=h('div'),proposal=h('div'),name=h('input',{'aria-label':'Nome do calendário persistente',type:'text',maxlength:160})as HTMLInputElement,path=h('input',{'aria-label':'Caminho do ficheiro ICS',type:'text',maxlength:4096})as HTMLInputElement;
 const consent=h('input',{type:'checkbox','aria-label':'Autorizar leitura persistente destes calendários'})as HTMLInputElement;
 const load=h('button',{type:'button',text:'Consultar calendários desta conversa',onclick:()=>void read()})as HTMLButtonElement;
 const add=h('button',{type:'button',text:'Adicionar calendário à seleção',onclick:()=>{
  if(!allowed()||busy||!draft||!owner)return;const title=name.value.trim(),file=path.value.trim();
  if(!title||bytes(title)>160||/\p{Cc}/u.test(title)||!pathValid(file)){status.textContent='Indica um nome e um caminho absoluto para um ficheiro .ics.';changed();return;}
  if(draft.sources.length>=64||draft.sources.filter(s=>s.enabled&&owns(s.owners,owner!)).length>=32){status.textContent='A seleção atingiu o limite de calendários.';changed();return;}
  const key=id();if(!/^[A-Za-z0-9_-]{1,64}$/.test(key)||draft.sources.some(s=>s.id===key)){status.textContent='Não foi possível criar uma referência distinta para o calendário.';changed();return;}
  draft.sources.push({id:key,revision:0,kind:'ics',name:title,path:file,enabled:true,owners:[structuredClone(owner)]});name.value='';path.value='';invalidate();draw();
 }})as HTMLButtonElement;
 const prepare=h('button',{type:'button',text:'Rever seleção persistente',onclick:()=>{
  if(!allowed()||busy||!draft||!doc||!owner)return;const next=structuredClone(draft),counts=new Map<string,number>();
  for(const s of next.sources.filter(s=>s.enabled))for(const o of s.owners){const key=JSON.stringify(o);counts.set(key,(counts.get(key)??0)+1);}
  if(next.sources.some(s=>s.owners.some(o=>o.scope==='pet')))next.version=3;else if(next.version===1&&[...counts.values()].some(n=>n>1))next.version=2;
  review=next;consent.checked=false;
  proposal.replaceChildren(h('p',{text:`Pet: ${owner.petId} · âmbito: ${owner.scope==='pet'?'fontes próprias do pet':'conversa '+owner.conversationId}. A seleção torna estas fontes elegíveis no âmbito revisto; não autoriza escrita, sincronização de conta ou trabalho autónomo.`}),h('ul',{},...next.sources.filter(s=>owns(s.owners,owner!)||doc!.config.sources.some(old=>old.id===s.id&&owns(old.owners,owner!))).map(s=>h('li',{text:`${s.enabled&&owns(s.owners,owner!)?'Ler':'Retirar acesso'}: ${s.name} · ${s.path} · referência ${s.id}`}))),h('p',{text:'O pet também precisa de permissão para documentos. Vários calendários partilham os limites de consulta do turno. O conteúdo dos calendários é dado não fiável.'}),h('label',{},consent,'Autorizo a leitura persistente das fontes apresentadas apenas nos âmbitos revistos.'),save);
  layout();changed();
 }})as HTMLButtonElement;
 const save=h('button',{type:'button',text:'Guardar seleção revista',disabled:true,onclick:()=>void commit()})as HTMLButtonElement;
 consent.addEventListener('change',()=>{save.disabled=busy||!review||needsConsent(review)&&!consent.checked;});
 const form=h('fieldset',{},h('legend',{text:'Adicionar uma fonte explícita'}),h('label',{},'Nome',name),h('label',{},'Ficheiro ICS',path),add);
 const summary=h('summary',{text:'Calendários desta conversa'});
 const el=h('details',{'aria-label':'Calendários persistentes desta conversa'},summary,h('p',{text:'Consulta e gere exportações ICS escolhidas por ti. Carregar a lista não abre os ficheiros.'}),load,status,rows,form,prepare,proposal);el.hidden=true;
 function invalidate(){review=null;consent.checked=false;proposal.replaceChildren();}
 function needsConsent(next:PetSourceRegistry){return !!doc&&next.sources.some(s=>s.enabled&&s.owners.some(o=>!doc!.config.sources.some(before=>before.id===s.id&&before.enabled&&before.kind===s.kind&&before.path===s.path&&owns(before.owners,o))));}
 function layout(){load.disabled=busy;add.disabled=busy||!draft;name.disabled=path.disabled=busy||!draft;prepare.disabled=busy||!draft||!!review;save.disabled=busy||!review||needsConsent(review)&&!consent.checked;}
 function draw(){rows.replaceChildren();if(draft&&owner)for(const s of draft.sources){
  const original=doc?.config.sources.find(v=>v.id===s.id);if(!owns(s.owners,owner)&&!original?.owners.some(o=>ownerSame(o,owner!)))continue;
  const check=h('input',{type:'checkbox','aria-label':`Selecionar calendário ${s.name} ${s.id}`,disabled:busy})as HTMLInputElement;check.checked=s.enabled&&owns(s.owners,owner);
  check.addEventListener('change',()=>{if(!allowed()||busy||!draft||!owner)return;
   if(check.checked){
    if(!s.enabled&&s.owners.some(o=>!ownerSame(o,owner!))){
     const key=id();if(draft.sources.length>=64||!/^[A-Za-z0-9_-]{1,64}$/.test(key)||draft.sources.some(v=>v.id===key)){check.checked=false;status.textContent='Não foi possível criar uma associação própria sem alterar os outros donos.';changed();return;}
     draft.sources.push({...structuredClone(s),id:key,revision:0,enabled:true,owners:[structuredClone(owner)]});
    }else{if(!owns(s.owners,owner))s.owners.push(structuredClone(owner));s.enabled=true;}
   }
   else{s.owners=s.owners.filter(o=>!ownerSame(o,owner!));if(!s.owners.length)s.enabled=false;}
   invalidate();draw();
  });rows.append(h('label',{},check,`${s.name} · ${s.path} · ${s.id}`));
 }layout();changed();}
 function validResponse(value:PetSourceDocument){if(!documentValid(value)||![1,2,3].includes(value.config?.version)||!Number.isSafeInteger(value.config?.generation)||value.config.generation<0||!Array.isArray(value.config.sources)||value.config.sources.length>64)throw Error('O catálogo de calendários está indisponível ou tem um formato incompatível.');}
 async function read(){if(!allowed()||!owner||busy)return;const stamp=epoch,target=structuredClone(owner);busy=true;invalidate();layout();status.textContent='A consultar apenas os metadados das fontes.';changed();
  try{const value=await transport.petSourceConfiguration();if(stamp!==epoch||!same(target,owner))return;validResponse(value);doc=structuredClone(value);draft=structuredClone(value.config);status.textContent='Revê as fontes antes de autorizar a sua leitura persistente.';}
  catch(error){if(stamp===epoch){doc=null;draft=null;status.textContent=String(error);}}
  finally{if(stamp===epoch){busy=false;draw();}}
 }
 async function commit(){if(!allowed()||busy||!owner||!doc||!review||needsConsent(review)&&!consent.checked)return;
  const target=structuredClone(owner),next=structuredClone(review),revision=doc.revision,choice=needsConsent(next)?'persistentCalendarRead' as const:'metadataOnly' as const,stamp=epoch;busy=true;layout();changed();
  try{const value=await transport.petSourceConfigurationSave(next,revision,choice);if(stamp!==epoch||!same(owner,target))return;validResponse(value);doc=structuredClone(value);draft=structuredClone(value.config);invalidate();status.textContent='Seleção guardada para o âmbito revisto. Novos pedidos continuam sujeitos às suas próprias permissões.';}
  catch(error){if(stamp===epoch){invalidate();doc=null;draft=null;status.textContent=`${String(error)} Volta a consultar o catálogo antes de guardar; não foi repetida a gravação.`;}}
  finally{if(stamp===epoch){busy=false;draw();}}
 }
 return {el,sync(next:PetSourceOwner|null){if(same(next,owner))return;epoch++;owner=next?structuredClone(next):null;busy=false;doc=null;draft=null;name.value=path.value='';status.textContent='';invalidate();el.hidden=!owner;const pet=owner?.scope==='pet';el.setAttribute('aria-label',pet?'Calendários persistentes deste pet':'Calendários persistentes desta conversa');summary.textContent=pet?'Calendários próprios deste pet':'Calendários desta conversa';load.textContent=pet?'Consultar calendários deste pet':'Consultar calendários desta conversa';draw();}};
}
