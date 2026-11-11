import {Bridge,type PetSourceRegistry,type RuntimeCalendarRequest} from '../core/bridge';
import {h} from './dom';
type Owner={petId:string;provider:string};
type Transport=Pick<typeof Bridge,'chatConversations'|'petSourceConfiguration'>;
const same=(a:unknown,b:unknown)=>JSON.stringify(a)===JSON.stringify(b);
const PET_SCOPE='#pet';
/** Explicit NativeUser metadata selection. No file, model, grant or enrollment
 * is read/created by this panel; actual data and destination are reviewed later. */
export function buildGoalCalendars(options:{transport?:Transport;changed?:()=>void}={}){
 const transport=options.transport??Bridge,changed=options.changed??(()=>{});let owner:Owner|null=null,epoch=0,busy=false,book:PetSourceRegistry|null=null;
 const selected=new Set<string>(),status=h('p',{role:'status'}),sources=h('div');
 const conversation=h('select',{'aria-label':'Conversa de origem dos calendários do objetivo'})as HTMLSelectElement;
 const start=h('input',{type:'text','aria-label':'Início do recorte de calendários',placeholder:'2026-11-09T00:00:00+00:00',maxlength:64})as HTMLInputElement;
 const end=h('input',{type:'text','aria-label':'Fim do recorte de calendários',placeholder:'2026-11-10T00:00:00+00:00',maxlength:64})as HTMLInputElement;
 const zone=h('input',{type:'text','aria-label':'Fuso IANA dos calendários do objetivo',placeholder:'Europe/Lisbon',maxlength:128})as HTMLInputElement;
 const limit=h('input',{type:'number','aria-label':'Máximo de eventos por calendário no objetivo',min:1,max:128,value:32})as HTMLInputElement;limit.value='32';
 const load=h('button',{type:'button',text:'Consultar fontes de calendário do pet',onclick:()=>void read()})as HTMLButtonElement;
 const el=h('details',{'aria-label':'Calendários para este objetivo'},h('summary',{text:'Selecionar calendários para o objetivo'}),h('p',{text:'Escolhe fontes próprias deste pet ou inscritas numa conversa sua, e um intervalo explícito. O objetivo recebe um recorte num contexto próprio. Depois de guardar, revê o conteúdo e o destino antes de autorizar a resposta.'}),load,status,h('label',{},'Conversa de origem',conversation),sources,h('label',{},'Início RFC3339 (com offset)',start),h('label',{},'Fim RFC3339 (com offset)',end),h('label',{},'Fuso IANA',zone),h('label',{},'Eventos por calendário',limit),h('p',{text:'Até oito fontes, com limite conjunto de dados. Consultar esta lista lê apenas metadados. Guardar o objetivo lê o recorte escolhido, sem copiar o histórico nem ativar inferência. Não autoriza ferramentas, partilha, regras ou sincronização de conta.'}));el.hidden=true;
 function controls(){load.disabled=busy;conversation.disabled=busy||!book;for(const input of[start,end,zone,limit])input.disabled=busy||!book;}
 function draw(){sources.replaceChildren();selected.clear();if(book&&owner)for(const source of book.sources.filter(s=>s.enabled&&s.owners.some(o=>o.petId===owner!.petId&&(conversation.value===PET_SCOPE?o.scope==='pet':o.scope===undefined&&o.conversationId===conversation.value)))){
  const check=h('input',{type:'checkbox','aria-label':`Usar calendário ${source.name} ${source.id} no objetivo`})as HTMLInputElement;
  check.addEventListener('change',()=>{if(check.checked&&selected.size>=8){check.checked=false;status.textContent='Escolhe no máximo oito calendários para este recorte.';}else if(check.checked)selected.add(source.id);else selected.delete(source.id);changed();});sources.append(h('label',{},check,`${source.name} · ID ${source.id} · revisão ${source.revision}`));
 }controls();changed();}
 conversation.addEventListener('change',draw);
 async function read(){if(!owner||busy)return;const stamp=epoch,target={...owner};busy=true;book=null;selected.clear();sources.replaceChildren();controls();status.textContent='A consultar conversas e fontes inscritas, sem abrir calendários.';changed();
  try{const[document,rows]=await Promise.all([transport.petSourceConfiguration(),transport.chatConversations(target.provider).catch(()=>null)]);if(stamp!==epoch||!same(target,owner))return;
   if(![1,2,3].includes(document.config?.version)||!Array.isArray(document.config.sources)||document.config.sources.length>64)throw Error('O catálogo de fontes tem um formato incompatível.');
   book=structuredClone(document.config);conversation.replaceChildren(h('option',{value:'',text:'Sem calendários selecionados'}));
   if(book.sources.some(s=>s.enabled&&s.owners.some(o=>o.petId===target.petId&&o.scope==='pet')))conversation.append(h('option',{value:PET_SCOPE,text:'Fontes próprias deste pet · sem conversa'}));
   for(const row of rows??[])if(book.sources.some(s=>s.enabled&&s.owners.some(o=>o.petId===target.petId&&o.scope===undefined&&o.conversationId===row.id)))conversation.append(h('option',{value:row.id,text:`${row.title||'Conversa'} · #${row.id}`}));conversation.value='';
   status.textContent=(rows===null?'Conversas indisponíveis; as fontes próprias do pet continuam selecionáveis. ':'')+'Seleciona o âmbito de origem, as referências e o intervalo. A autorização de utilização pertence ao objetivo e será revista separadamente.';
  }catch(error){if(stamp===epoch){book=null;status.textContent=String(error);conversation.replaceChildren();}}
  finally{if(stamp===epoch){busy=false;draw();}}
 }
 function capture():{conversationId:string|null;request:RuntimeCalendarRequest}|undefined{
  if(busy)throw Error('Aguarda a consulta dos metadados dos calendários.');if(!selected.size)return;
  if(!owner||!book||!conversation.value)throw Error('Seleciona o âmbito de origem dos calendários.');
  const a=start.value.trim(),b=end.value.trim(),tz=zone.value.trim(),count=Number(limit.value);
  const timestamp=(s:string)=>s.length<=64&&/T.*(?:Z|[+-]\d\d:\d\d)$/.test(s)&&Number.isFinite(Date.parse(s));
  if(!timestamp(a)||!timestamp(b)||Date.parse(a)>=Date.parse(b)||Date.parse(b)-Date.parse(a)>367*86400000)throw Error('Indica início e fim RFC3339 com offset e intervalo válido até367 dias.');
  if(!tz||tz.length>128||!/^[-+A-Za-z0-9_/]+$/.test(tz)||tz.split('/').some(p=>!p||p==='.'||p==='..'))throw Error('Indica um fuso IANA explícito.');
  if(!limit.value.trim()||!Number.isInteger(count)||count<1||count>128)throw Error('O limite de eventos tem de ser um inteiro entre1 e128.');
  const picked=book.sources.filter(s=>selected.has(s.id)&&s.enabled&&s.owners.some(o=>o.petId===owner!.petId&&(conversation.value===PET_SCOPE?o.scope==='pet':o.scope===undefined&&o.conversationId===conversation.value)));
  if(picked.length!==selected.size||picked.some(s=>!Number.isSafeInteger(s.revision)||s.revision<1))throw Error('As referências dos calendários mudaram. Consulta novamente as fontes.');
  return {conversationId:conversation.value===PET_SCOPE?null:conversation.value,request:{...(conversation.value===PET_SCOPE?{sourceScope:'pet' as const}:{}),selected:picked.map(({id,revision})=>({id,revision})),query:{start:a,end:b,timeZone:tz,limit:count}}};
 }
 return {el,capture,matches(expected:ReturnType<typeof capture>){try{return same(capture(),expected);}catch{return false;}},clear(){selected.clear();for(const check of sources.querySelectorAll<HTMLInputElement>('input'))check.checked=false;},sync(next:Owner|null){if(same(owner,next))return;epoch++;owner=next?{...next}:null;busy=false;book=null;selected.clear();sources.replaceChildren();conversation.replaceChildren();start.value=end.value=zone.value='';status.textContent='';el.hidden=!owner;controls();}};
}
