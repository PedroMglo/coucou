import {PetSettings} from "./pet-settings";
import {PetPreferences,petName} from "../mochi/pet-preferences";
import {Bridge,onEvent} from "../core/bridge";
import {mergeTasks,orderedTasks,taskCanStop,taskLabel,taskName,taskPet,taskProvider,taskRecoverable,taskRunning,type AssistantTask} from "../mochi/tasks";
import type {PetIdentity} from "../mochi/companions";

interface Row {el:HTMLElement;title:HTMLElement;status:HTMLElement;context:HTMLElement;progress:HTMLElement;result:HTMLElement;stop:HTMLButtonElement;resume:HTMLButtonElement;review:HTMLButtonElement;terminal:HTMLButtonElement;folder:HTMLElement;folderToggle:HTMLButtonElement;folderApply:HTMLButtonElement}
/** Explicit supervision surface. No model is started by opening or refreshing it. */
export class TaskBoard {
  readonly el=document.createElement("aside");
  private list=document.createElement("div");
  private title=document.createElement("strong");
  private preferences=new PetSettings();
  private configuring=false;
  private error=document.createElement("p");
  private rows=new Map<string,Row>();
  private tasks:AssistantTask[]=[];
  private external:AssistantTask[]=[];
  private get allTasks(){return mergeTasks(this.tasks,this.external);}
  private filter:string|null=null;
  private pending=false;
  private dirty=false;
  private resuming=new Set<string>();
  private visible=true;
  private hovered=false;
  private openPet:(pet:string)=>void;
  private changed:()=>void;
  constructor(openPet:(pet:string)=>void,changed:()=>void) {
    this.openPet=openPet;this.changed=changed;
    this.el.className="assistant-tasks";this.el.hidden=true;this.el.setAttribute("aria-label","Centro de tarefas");
    const header=document.createElement("header");const close=document.createElement("button");close.textContent="×";close.title="Fechar tarefas";close.setAttribute("aria-label","Fechar tarefas");close.onclick=()=>this.hide();
    const all=document.createElement("button");all.textContent="Todos";all.onclick=()=>{this.filter=null;this.draw();};
    const configure=document.createElement("button");configure.textContent="Configurar pets";configure.onclick=()=>{this.configuring=!this.configuring;this.preferences.el.hidden=!this.configuring;this.list.hidden=this.configuring;configure.textContent=this.configuring?"Voltar às tarefas":"Configurar pets";if(this.configuring)void this.preferences.load();this.changed();};
    const manage=document.createElement("button");manage.textContent="Gestão dos pets";manage.onclick=()=>void Bridge.openSettingsWindow("pets");
    header.append(this.title,all,configure,manage,close);this.list.className="assistant-task-list";
    this.error.className="assistant-task-error";this.error.hidden=true;
    this.list.addEventListener("pointerenter",()=>{this.hovered=true;});
    this.list.addEventListener("pointerleave",()=>{this.hovered=false;this.draw();});
    this.el.append(header,this.list,this.preferences.el,this.error);document.body.append(this.el);
  }
  get hidden(){return this.el.hidden;}
  get activeCount(){return this.allTasks.filter(taskRunning).length;}
  get needsAttention(){return this.allTasks.some(t=>["approval","question","interrupted","failed","error","ratelimit"].includes(t.phase));}
  supports(pet:string){return !!taskProvider(pet) || this.external.some(t=>t.petId===pet);}
  setExternal(pets:readonly PetIdentity[]) {
    const previous=new Map(this.external.map(t=>[t.id,t]));
    this.external=pets.filter(p=>p.session && !p.session.owned).map(p=>{
      const id=`external:${p.id}:${p.session!.id}`,old=previous.get(id),provider=taskProvider(p.id)??p.id;
      return {id,origin:"external",petId:p.id,petName:p.name,sessionId:p.session!.id,provider,backend:p.session!.backend??provider,model:"",cwd:p.session!.cwd,query:`Sessão no terminal · ${p.name}`,phase:p.state,progress:"",result:p.session!.result,started:old?.started??Date.now(),updated:Date.now(),finished:null};
    });
    if(!this.hidden)this.draw();
  }
  async init(){await PetPreferences.init();await onEvent("assistant-tasks-changed",()=>void this.refresh());await this.refresh();}
  setVisible(visible:boolean){this.visible=visible;if(!visible)this.hide();}
  toggle(pet?:string) {
    const provider=pet?(taskProvider(pet)??pet):null;
    if(!this.el.hidden && this.filter===provider){this.hide();return;}
    if(!this.visible)return;
    this.filter=provider;this.el.hidden=false;this.error.hidden=true;this.draw();void this.refresh();this.changed();
  }
  hide(){this.el.hidden=true;this.changed();}
  position(x:number,top:number,width:number) {
    this.el.style.left=`${Math.max(8,Math.min(width-428,x-210))}px`;
    this.el.style.top=`${top}px`;this.el.style.setProperty("--task-top",`${top}px`);
  }
  private async refresh() {
    if(this.pending){this.dirty=true;return;}this.pending=true;
    try {do {
      this.dirty=false;const tasks=await Bridge.assistantTasks();
      if(tasks){this.tasks=tasks;if(!this.hidden)this.draw();this.changed();}
    }while(this.dirty);} finally {this.pending=false;}
  }
  private async resume(id:string,cwd?:string) {
    if(this.resuming.has(id))return;
    this.resuming.add(id);this.error.hidden=true;this.draw();
    try {await Bridge.assistantResumeTask(id,cwd);}catch(error){this.error.textContent=String(error).replace(/^Error:\s*/,"");this.error.hidden=false;}
    finally {this.resuming.delete(id);await this.refresh();}
  }
  private row(task:AssistantTask):Row {
    const el=document.createElement("article");const title=document.createElement("h3"),status=document.createElement("p"),context=document.createElement("p"),progress=document.createElement("p");
    const details=document.createElement("details"),summary=document.createElement("summary"),result=document.createElement("pre");summary.textContent="Ler resultado";details.append(summary,result);
    status.className="assistant-task-status";context.className="assistant-task-context";progress.className="assistant-task-progress";
    const actions=document.createElement("footer");const stop=document.createElement("button"),resume=document.createElement("button"),review=document.createElement("button"),terminal=document.createElement("button");
    stop.textContent="Interromper";resume.textContent="Verificar e retomar";review.textContent="Rever decisão";
    stop.onclick=()=>{
      stop.disabled=true;this.error.hidden=true;
      void Bridge.assistantCancelTask(task.id).then(stopped=>{if(!stopped){this.error.textContent='Esta tarefa já não está a executar.';this.error.hidden=false;}})
       .catch(error=>{this.error.textContent=String(error).replace(/^Error:\s*/,'');this.error.hidden=false;})
       .finally(()=>{stop.disabled=false;void this.refresh();});
    };
    resume.onclick=()=>void this.resume(task.id);
    const folderToggle=document.createElement("button");folderToggle.textContent="Alterar pasta";
    const folder=document.createElement("div"),folderInput=document.createElement("input"),folderApply=document.createElement("button");folder.className="assistant-task-folder";folder.hidden=true;
    folderInput.value=task.cwd;folderInput.setAttribute("aria-label","Pasta para retomar a tarefa");folderInput.autocomplete="off";
    folderApply.textContent="Verificar e retomar nesta pasta";folderApply.onclick=()=>void this.resume(task.id,folderInput.value.trim());
    folderToggle.onclick=()=>{folder.hidden=!folder.hidden;if(!folder.hidden)folderInput.focus();this.changed();};folder.append(folderInput,folderApply);
    review.onclick=()=>{const pet=task.petId??taskPet(task.provider);if(pet){this.hide();this.openPet(pet);}};
    terminal.textContent="Abrir este terminal";terminal.onclick=()=>{
      if(!task.sessionId){this.error.textContent='Esta tarefa não tem um terminal associado.';this.error.hidden=false;return;}
      terminal.disabled=true;this.error.hidden=true;
      void Bridge.petFocusSession(task.backend,task.sessionId).catch(error=>{this.error.textContent=String(error).replace(/^Error:\s*/,'');this.error.hidden=false;}).finally(()=>terminal.disabled=false);
    };
    actions.append(stop,resume,folderToggle,review,terminal);el.append(title,status,context,progress,details,actions,folder);
    return {el,title,status,context,progress,result,stop,resume,review,terminal,folder,folderToggle,folderApply};
  }
  private draw() {
    this.title.textContent=this.filter?`Tarefas · ${taskName(this.filter)}`:"Centro de tarefas";
    const tasks=orderedTasks(this.allTasks,this.filter),ids=new Set(tasks.map(t=>t.id));
    for(const [id,row] of this.rows)if(!this.hovered && !ids.has(id)){row.el.remove();this.rows.delete(id);}
    let empty=this.list.querySelector(".assistant-task-empty");
    if(!tasks.length){if(!empty){empty=document.createElement("p");empty.className="assistant-task-empty";empty.textContent="Os pedidos aos assistentes aparecem aqui, com o seu progresso e resultado.";this.list.append(empty);}return;}else empty?.remove();
    for(const task of tasks) {
      const row=this.rows.get(task.id)??this.row(task);this.rows.set(task.id,row);
      row.title.textContent=task.query;row.title.title=task.query;
      row.status.textContent=`${task.petName??petName(taskPet(task.provider)??"",taskName(task.provider))} · ${taskLabel(task.phase)}`;
      row.el.dataset.phase=task.phase;
      const folder=task.cwd.split(/[\\/]/).filter(Boolean).at(-1)??task.cwd;
      row.context.textContent=[folder,task.origin==="external"?"CLI externo":task.backend!==task.provider?`via ${taskName(task.backend)}`:"",task.origin!=="external"?(task.model||"modelo definido na CLI"):""].filter(Boolean).join(" · ");row.context.title=task.cwd;
      row.progress.textContent=task.progress;row.progress.hidden=!task.progress || task.phase==="done";
      row.result.textContent=task.result;row.result.parentElement!.hidden=!task.result;
      row.stop.hidden=!taskCanStop(task);row.stop.disabled=task.phase==="stopping";
      row.terminal.hidden=task.origin!=="external";
      row.resume.hidden=!taskRecoverable(task);row.resume.disabled=this.resuming.has(task.id);
      row.folderToggle.hidden=!taskRecoverable(task);row.folderToggle.disabled=row.folderApply.disabled=this.resuming.has(task.id);
      if(!taskRecoverable(task))row.folder.hidden=true;
      row.resume.textContent=this.resuming.has(task.id)?"A retomar…":"Verificar e retomar";
      row.review.hidden=!["approval","question"].includes(task.phase);
      if(!this.hovered || row.el.parentElement!==this.list)this.list.append(row.el);
    }
  }
}
