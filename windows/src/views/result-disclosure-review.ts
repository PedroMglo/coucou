import type {RuntimeResultDisclosurePreview} from '../core/bridge';
import {h} from './dom';
/** Native authenticated DATA shown as text. It never approves or dispatches. */
export function resultDisclosureReview(p:RuntimeResultDisclosurePreview,disclosure=false){
 const r=p.arguments.result,o=p.origin;
 return h('section',{'aria-label':'Origem e âmbito do resultado'},h('h4',{text:'Origem e âmbito do resultado'}),
  h('p',{text:`Origem: ${p.petId} · motor ${o.inferenceDestination.provider} · adaptador ${o.adapter} · modelo pedido ${o.model} · origem ${o.provider}. Os internos do servidor não são observados por este recibo. Resultado registado em ${new Date(o.recordedAt).toISOString()}.`}),
  h('pre',{text:JSON.stringify({destinoOriginalDeInferencia:o.inferenceDestination,inputHash:o.inputHash,intentHash:o.intentHash,completedAt:o.completedAt,recordedAt:o.recordedAt,decisionId:r.decisionId,attemptId:r.attemptId,recibo:r.receiptHash},null,2)}),
  h('p',{text:`Excerto exato: bytes UTF-8 [${r.byteStart}, ${r.byteEnd}) de ${r.totalBytes}. Não representa cobertura integral das fontes nem verificação dos critérios.`}),h('pre',{text:p.arguments.text}),
  h('p',{text:`Destinatário local: ${p.recipient.name} (${p.recipient.petId}) · configuração ${p.recipient.configurationHash}. Âmbito previsto: uma resposta à origem da delegação; prazo ${new Date(p.arguments.expiresAt).toISOString()}.`}),
  h('p',{text:'Dependências: todas as fontes originais selecionadas têm de continuar atuais. Revisão, dono, conteúdo e validade serão novamente verificados; reativar uma fonte não recupera uma revisão já invalidada.'}),
  h('pre',{text:JSON.stringify({pedidoRecebido:{messageId:p.receivedMessage.messageId,origin:p.receivedMessage.origin.petId,deadlineAt:p.receivedMessage.deadlineAt},memorias:p.memoryReferences,contextHash:p.contextHash,causa:p.cause,plano:p.proposalHash},null,2)}),
  h('p',{text:disclosure?'Esta autorização permite uma única entrega local do excerto e proveniência revistos, dentro da reserva original. Não autoriza inferência, encaminhamento, execução pelo destinatário nem certifica os critérios do objetivo.':'Esta confirmação guarda apenas o plano. A divulgação continua bloqueada até existir consentimento próprio e um sucessor financiado. Receber o resultado não autoriza inferência, encaminhamento nem execução pelo destinatário.'}));
}
