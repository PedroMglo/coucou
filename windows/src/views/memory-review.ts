import {type PetMemoryEntry,type PetMemoryGoalPreview,type PetMemoryManualPreview,type ChatContext} from '../core/bridge';
import {h} from './dom';

const at=(value:number|null)=>value===null?'sem validade definida':new Date(value).toLocaleString();
/** Records are untrusted data. Never interpret their content as markup or as
 * an approval, including when it resembles commands or a policy document. */
export function memoryRecord(entry:PetMemoryEntry){
 return h('article',{'aria-label':`Recordação ${entry.title}`},h('strong',{text:entry.title}),
  h('p',{text:`${entry.kind==='preference'?'Preferência':'Nota'} · ${entry.sensitivity==='sensitive'?'sensível':'normal'} · origem: utilizador explícito · pet ${entry.owner.petId} · ${entry.owner.scope.kind==='pet'?'âmbito do pet':'conversa '+entry.owner.scope.conversationId}`}),
  h('p',{text:`ID ${entry.id} · revisão ${entry.revision} · observada ${at(entry.observedAt)} · validade: ${at(entry.expiresAt)}`}),
  h('pre',{text:entry.content,style:'white-space:pre-wrap;overflow-wrap:anywhere'}));
}
export function goalMemoryReview(data:PetMemoryGoalPreview){
 const destination=data.destination;
 return h('section',{'aria-label':'Dados selecionados para este objetivo'},
  h('h4',{text:'Revisão dos dados do objetivo'}),
  h('p',{text:`Pet ${data.petId} · objetivo ${data.goalId} · origem: ${data.sourceConversationId?'conversa '+data.sourceConversationId:'recordações do próprio pet'}`}),
  h('p',{text:destination.kind==='nativeSession'?`Sessão nativa ${destination.provider} · modelo ${destination.model} · pasta ${destination.workspace}`:`Destino ${destination.endpoint} · fornecedor ${destination.provider} · modelo ${destination.model}`}),
  h('p',{text:`Âmbito: apenas este objetivo e os limites da autorização de trabalho apresentada. Os dados podem ser usados em várias chamadas autorizadas, até ${data.maxInputBytes} B de entrada por pedido. Não autoriza ferramentas, outros objetivos ou partilha com outros pets.`}),
  h('p',{text:'Alterar ou eliminar uma recordação impede novos usos da revisão anterior. Não retira dados que o destino já tenha recebido.'}),
  ...data.records.map(memoryRecord));
}

export function manualMemoryReview(data:PetMemoryManualPreview,context:ChatContext|null){
 const d=data.destination,c=data.dispatchCoverage;
 return h('section',{'aria-label':'Revisão dos dados deste pedido'},h('h4',{text:'Rever dados antes de enviar'}),
  h('p',{text:`Pet ${data.petId} · conversa ${data.conversationId} · fornecedor ${d.provider} · modelo revisto ${d.model}`}),
  h('pre',{text:data.question,style:'white-space:pre-wrap;overflow-wrap:anywhere'}),
  ...(d.kind==='nativeSession'?[h('p',{text:`Sessão nativa ${d.adapter} · pasta ${d.workspace} · ID ${d.sessionId??'ainda não atribuído'}. A conta nativa e o routing interno não foram observados.`})]:[h('p',{text:c.kind==='knownApiEndpointSet'?`Base API revista: ${d.endpoint}. O protocolo pode usar um dos destinos abaixo, incluindo calendário quando configurado.`:`Destino nativo revisto: ${d.endpoint}`})]),
  h('ul',{},...c.endpoints.map(endpoint=>h('li',{text:endpoint}))),
  h('p',{text:'O modelo pedido, os destinos nativos e a credencial ficam vinculados à revisão. Aliases, routing e ferramentas internos do fornecedor não são garantias de contenção. A credencial não é apresentada.'}),
  h('p',{text:data.requiredConsent==='nativeSessionContext'?'Âmbito: contexto da sessão nativa. Depois de entregue, a sessão e os sistemas do fornecedor podem conservar estes dados; cancelar ou eliminar impede novos usos no Coucou, sem apagar entregas anteriores. Não autoriza objetivos autónomos.':'Âmbito: apenas este pedido manual. Não autoriza rondas autónomas, outros objetivos, ferramentas ou partilha com outros pets. Dados já recebidos pelo destino não são retirados ao cancelar ou eliminar.'}),
  ...(context?[h('details',{},h('summary',{text:'Referências de anexos/contexto deste pedido'}),h('pre',{text:JSON.stringify(context,null,2)}))]:[]),
  h('p',{text:`Revisão válida até ${new Date(data.expiresAt).toLocaleString()}. Anexos/contexto vinculados à revisão ${data.requestContextHash}.`}),
  ...data.records.map(memoryRecord));
}
