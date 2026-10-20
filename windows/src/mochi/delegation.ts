import {petRetired} from "../core/pet-factory";
import {pillDefinition} from "../core/pills";
import {petCapabilities} from "./capabilities";

/** Editable requests only. Selecting a service never starts an AI or PC job. */
export function delegatedRequest(id:string):{source:string;text:string}|null {
  if(petRetired(id))return null;
  const pet=pillDefinition(id);
  if(!pet || petCapabilities(id).execute || !["service","workspace","agent"].includes(pet.category))return null;
  const requests:Record<string,string>={
    integration_github:"Revê os pull requests e o estado do CI do GitHub ligado. Indica falhas e o que precisa da minha atenção.",
    integration_vercel:"Revê as implementações recentes da Vercel ligada e identifica falhas que precisem de atenção.",
    integration_n8n:"Revê o estado e as execuções dos fluxos n8n ligados. Identifica falhas e propõe os próximos passos.",
    integration_resend:"Revê os dados de entrega de email disponíveis na Resend e identifica problemas.",
    integration_notion:"Revê os dados disponíveis da Notion ligada e ajuda-me a identificar tarefas e próximos passos.",
    integration_calcom:"Revê os compromissos disponíveis do Cal.com ligado e ajuda-me a preparar o dia.",
    integration_stripe:"Resume os dados de pagamentos disponíveis na Stripe ligada e identifica falhas que precisem de atenção.",
    integration_spotify:"Consulta a música atual e as faixas recentes disponíveis do Spotify e ajuda-me a escolher o que ouvir.",
  };
  const request=requests[id]??`Ajuda-me a acompanhar a sessão de ${pet.name} no computador. Verifica o estado e os pedidos pendentes realmente disponíveis.`;
  return {source:pet.name,text:`${request} Usa as ferramentas do Mochi para consultar os dados disponíveis e distingue dados verificados de informação indisponível. Este pedido é de consulta e preparação: apresenta o resultado antes de executar alterações.`};
}
