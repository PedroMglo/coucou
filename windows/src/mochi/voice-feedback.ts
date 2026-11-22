import type {Dialogue} from './dialogue';
export interface VoiceView {phase:string;heard:string;answer:string;pet?:string|null;announcementId?:number;note?:boolean}
const labels:Record<string,string>={preparing:'A ligar o microfone',listening:'A ouvir',transcribing:'A transcrever',received:'Pedido recebido',thinking:'A preparar resposta',working:'A executar',searching:'A pesquisar',message:'A responder',speaking:'Resposta por voz',approval:'Precisa da tua aprovação',question:'Precisa de uma resposta tua',failed:'Não conseguiu terminar',stopped:'Interrompido'};
/** App-owned progress describes this conversation. External approvals stay in their own card. */
export function voiceFeedback(view:VoiceView,activity:Dialogue|undefined,name:string,receivedAt=0) {
 if(activity && activity.changedAt<receivedAt)activity=undefined;
 const waiting=view.phase==='received';
 const phase=waiting && activity && !['done','stopped','failed'].includes(activity.phase) ? activity.phase : view.phase;
 const title=`${view.pet ? name+' · ' : ''}Mochi · ${labels[phase]??phase}`;
 const text=view.phase==='speaking' ? view.answer : waiting ? (activity?.message || `Ouvi: ${view.heard}`) : view.phase==='transcribing' ? 'Estou a converter a tua fala em texto.' : view.phase==='preparing' ? 'Espera pelo sinal de escuta.' : view.phase==='listening' ? (view.pet ? 'Fala comigo · Super termina a escuta' : 'Diz o nome do pet e o teu pedido.') : view.answer;
 return {phase,title,text};
}
