import {isServiceProvider,petName} from "./pet-preferences";
export interface AssistantTask {
  id:string;provider:string;backend:string;model:string;query:string;cwd:string;
  phase:string;progress:string;result:string;started:number;updated:number;finished:number|null;
  resumes?:string|null;resumedBy?:string|null;
  sessionId?:string|null;origin?:"external";petId?:string;petName?:string;
}
export const taskPet=(provider:string)=>isServiceProvider(provider)?provider:({codex:"agent_codex",copilot:"agent_copilot",antigravity:"agent_antigravity","gemini-cli":"agent_gemini","claude-cli":"integration_claude"}[provider]??null);
export const taskProvider=(pet:string)=>isServiceProvider(pet)?pet:({agent_codex:"codex",agent_copilot:"copilot",agent_antigravity:"antigravity",agent_gemini:"gemini-cli",integration_claude:"claude-cli"}[pet]??null);
export const taskName=(provider:string)=>isServiceProvider(provider)?petName(provider,provider.replace("integration_","")):({codex:"Codex",copilot:"Copilot",antigravity:"Gravity","gemini-cli":"Gemini","claude-cli":"Claude"}[provider]??provider);
export const taskLabel=(phase:string)=>({thinking:"A pensar",working:"A executar",searching:"A pesquisar",message:"A responder",approval:"À espera da tua aprovação",question:"À espera da tua resposta",stopping:"A interromper",done:"Resposta pronta",failed:"Não concluída",stopped:"Interrompida",interrupted:"Estado anterior por confirmar",resumed:"Retomada numa nova execução",idle:"Em espera no terminal",finished:"Concluída no terminal"}[phase]??"Estado por confirmar");
export const taskRunning=(task:AssistantTask)=>["thinking","working","searching","message","approval","question","stopping"].includes(task.phase);
export const taskRecoverable=(task:AssistantTask)=>task.origin!=="external" && ["failed","stopped","interrupted"].includes(task.phase);
export const taskCanStop=(task:AssistantTask)=>task.origin!=="external" && taskRunning(task);
export function mergeTasks(owned:readonly AssistantTask[],external:readonly AssistantTask[]) {
  return [...owned,...external.filter(t=>!owned.some(o=>!!t.sessionId && o.sessionId===t.sessionId && o.backend===t.backend))];
}
export function orderedTasks(tasks:readonly AssistantTask[],provider:string|null=null) {
  return tasks.filter(t=>!provider || t.provider===provider).slice().sort((a,b)=>Number(taskRunning(b))-Number(taskRunning(a)) || b.started-a.started);
}
