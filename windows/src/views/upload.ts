// Drop zone, upload progress and the "what do you want to do with it" card —
// ports of UploadView / UploadingView / ChooseView from IslandViewContent.swift.
//
// Sending a file by email is not in the Windows v1, so `choose` offers the one
// action the spec asks for: ask a question about it.

import { h, clear } from "./dom";
import { State } from "../core/state";
import type { ViewActions, ViewHost } from "./views";
import { N_, t, tl } from "../i18n/i18n";
import {Bridge} from '../core/bridge';
import {PetPreferences} from '../mochi/pet-preferences';
import {providerForPill} from '../core/providers';
import {documentActions} from '../mochi/documents';

/** Dashed rounded rect drawn as SVG so the dashes can march like on macOS. */
function dashedFrame(): SVGSVGElement {
  const ns = "http://www.w3.org/2000/svg";
  const el = document.createElementNS(ns, "svg");
  el.setAttribute("class", "drop-frame");
  el.setAttribute("preserveAspectRatio", "none");
  const rect = document.createElementNS(ns, "rect");
  rect.setAttribute("x", "0.75");
  rect.setAttribute("y", "0.75");
  rect.setAttribute("width", "calc(100% - 1.5px)");
  rect.setAttribute("height", "calc(100% - 1.5px)");
  rect.setAttribute("rx", "20");
  rect.setAttribute("fill", "none");
  rect.setAttribute("stroke-width", "1.5");
  rect.setAttribute("stroke-dasharray", "6 5");
  el.append(rect);
  return el;
}

export function buildUpload(): ViewHost {
  const frame = dashedFrame();
  const title = h("div", { class: "drop-title", text: tl("Drop your files here") });
  const tags = h(
    "div",
    { class: "drop-tags" },
    ...[N_("PDF"), N_("Images"), N_("Code"), N_("Docs")].map((chip) => h("span", { text: tl(chip) })),
  );
  const card = h(
    "div",
    { class: "card drop-card" },
    frame,
    h("div", { class: "drop-body" }, title, tags),
  );
  const el = h("div", { class: "view" }, card);

  return {
    el,
    sync() {
      card.classList.toggle("over", State.fileDragOver);
    },
  };
}

export function buildUploading(): ViewHost {
  const label = h("span", { class: "up-name" });
  const percent = h("span", { class: "up-pct" });
  const fill = h("div", { class: "up-fill" });
  const glow = h("div", { class: "up-glow" });
  const card = h(
    "div",
    { class: "card up-card" },
    h("div", { class: "up-row" }, label, percent),
    h("div", { class: "up-track" }, fill, glow),
  );
  const el = h("div", { class: "view" }, card);

  return {
    el,
    sync() {
      const done = State.uploadProgress >= 0.999;
      const pct = Math.round(State.uploadProgress * 100);
      label.textContent = done
        ? `✓  ${State.droppedFile?.name ?? t("File")}`
        : t("Uploading {name}", { name: State.droppedFile?.name ?? t("file") });
      label.classList.toggle("done", done);
      percent.textContent = done ? "" : `${pct} %`;
      const w = State.uploadProgress * 526;
      fill.style.width = `${w}px`;
      glow.style.transform = `translateX(${Math.max(0, w - 14)}px)`;
      glow.style.opacity = State.uploadProgress > 0.01 ? "1" : "0";
      card.classList.toggle("done", done);
    },
  };
}

export function buildChoose(actions: ViewActions): ViewHost {
  const title = h("div", { class: "title" });
  const sub = h("div", { class: "sub", text: tl("What do you want to do with it?") });
  const email=h('button',{class:'btn secondary',text:'Preparar email',onclick:()=>actions.setView('mail')});
  const analyze=h('button',{class:'btn primary',text:'Analisar / conversar',onclick:()=>actions.setView('prompt')});
  const row = h(
    "div",
    { class: "actions" },
    analyze,
    email,
    h("button", {
      class: "btn secondary",
      text: tl("Cancel"),
      onclick: () => actions.cancelDrop(),
    }),
  );
  const el = h(
    "div",
    { class: "view" },
    h(
      "div",
      { class: "card" },
      h("div", { class: "stack", style: "padding:0 18px 0 98px" }, title, sub, row),
    ),
  );

  return {
    el,
    sync() {
      const id=Object.keys(PetPreferences.config?.pets??{}).find(id=>providerForPill(id)===State.chatProvider);
      const available=documentActions(PetPreferences.config,id??'',State.droppedFile?.kind??'file',State.droppedFile?.name??'');
      email.hidden=!available.email;analyze.hidden=!available.analysis;
      sub.textContent=available.email || available.analysis?'Escolhe o que este pet deve fazer.': 'Este pet não tem ações disponíveis para este documento. Revê os seus acessos na gestão dos pets.';
      clear(title);
      title.append(
        h("b", { text: State.droppedFile?.name ?? t("file") }),
        document.createTextNode(State.droppedFile?.kind==='folder'?' está pronta.':' está pronto.'),
      );
    },
  };
}

/** Explicit draft only: no inference or send action is invoked here. */
export function buildMail(actions:ViewActions):ViewHost{
 const drafts=new Map<string,{to:string;subject:string;body:string;status:string}>(),busy=new Set<string>();let owner='';
 const to=h('input',{placeholder:'Destinatário (podes preencher depois)'});to.setAttribute('aria-label','Destinatário do email');
 const subject=h('input',{placeholder:'Assunto'});subject.setAttribute('aria-label','Assunto do email');
 const body=h('textarea',{placeholder:'Mensagem'});body.setAttribute('aria-label','Mensagem do email');
 const status=h('p',{class:'sub',role:'status'}),attachment=h('p',{class:'sub'});
 const send=h('button',{class:'btn primary',text:'Abrir rascunho com anexo',onclick:async()=>{
  const provider=owner,file=State.fileFor(provider);
  if(!provider||provider!==State.chatProvider||State.pendingApproval?.pillId===State.focusId){status.textContent='Volta ao pet deste documento para continuar o rascunho.';return;}
  if(!file){status.textContent='O anexo já não está disponível. Entrega novamente o documento a este pet.';return;}
  if(busy.has(provider))return;busy.add(provider);keep();
  send.disabled=true;status.textContent='A preparar o rascunho…';
  let message='';try{message=await Bridge.petMailDraft(provider,file.path,to.value,subject.value,body.value);}catch(error){message=String(error).replace(/^Error:\s*/,'');}finally{busy.delete(provider);const draft=drafts.get(provider);if(draft)draft.status=message;if(owner===provider){status.textContent=message;send.disabled=false;}}
 }});
 function keep(){if(owner)drafts.set(owner,{to:to.value,subject:subject.value,body:body.value,status:status.textContent??''});}
 for(const field of [to,subject,body])field.addEventListener('input',keep);
 const el=h('div',{class:'view'},h('div',{class:'card'},h('div',{class:'pet-mail-form'},attachment,to,subject,body,status,h('div',{class:'actions'},send,h('button',{class:'btn secondary',text:'Voltar',onclick:()=>actions.setView('choose')})))));
 return{el,focus:()=>subject.focus(),sync(){
  // Hidden views are synced as well. An unrelated approval must not replace
  // the form's owner or move its private fields to a different pet.
  if(State.pendingApproval?.pillId===State.focusId||(['approval','question'].includes(State.view)))return;
  if(owner!==State.chatProvider){keep();owner=State.chatProvider;const draft=drafts.get(owner);to.value=draft?.to??'';subject.value=draft?.subject??'';body.value=draft?.body??'';status.textContent=draft?.status??'';}
  attachment.textContent=State.droppedFile?`Anexo: ${State.droppedFile.name}`:'Entrega primeiro um documento ao pet.';send.disabled=!State.droppedFile || State.droppedFile.kind==='folder' || busy.has(owner);
 }};
}
