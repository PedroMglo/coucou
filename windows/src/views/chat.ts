import {PetPreferences,profileModels,profileProvider,profileLabel,profileSummary} from "../mochi/pet-preferences";
// Chat view — DOM port of PromptView / ChatBubble / TypingDotsView /
// ModelPickerView from IslandViewContent.swift.
//
// Answers are rendered as Markdown (markdown.ts). A local model streams its
// answer: Rust sends `chat-delta` events with the text visible so far. The
// model name above the text field opens the picker: provider chips, then the
// models of the chosen provider, asked for only once it is picked.

import { h, svg, clear } from "./dom";
import {buildTaskNotice} from "./task-notice";
import {buildRequestHistory} from "./request-history";
import { ICONS } from "./icons";
import { renderMarkdown } from "./markdown";
import { Bridge, emitToWindow, onEvent, type ChatContext, type ModelInfo,type ChatTurnState,type ConversationView } from "../core/bridge";
import {
  activeModel, isCliProvider, pickModel, providerDef, providerForPill, visibleProviders, withModel, type ProviderDef,
} from "../core/providers";
import { Sound } from "../core/sound";
import { State, type ChatMessage } from "../core/state";
import type { ViewHost } from "./views";
import { N_, t, tl } from "../i18n/i18n";
import {petCapabilities} from "../mochi/capabilities";
import {dialogueId,rememberDialogue,resetDialogue,restoreDialogue} from "../core/chat-dialogue";
import {SessionChat,sessionTargetKey,sessionOrigin,type SessionTarget} from "../core/session-chat";
import {buildAgentRequests} from './agent-requests';
import {OllamaModelSelection} from '../core/ollama-selection';
import {commandReview} from './command-review';
import {LocalCapabilities,type LocalCapabilityEvent} from '../core/local-capabilities';

const localCapabilities=new LocalCapabilities();let watchingCapabilities=false;
function watchCapabilities(){if(watchingCapabilities)return;watchingCapabilities=true;void onEvent<LocalCapabilityEvent>('local-execution-capability',e=>{if(e.provider==='ollama'){localCapabilities.observe(e.conversationId,e.turn,e.capability);State.notify();}});}
function capabilityText(id:string,selected:string){const pet=PetPreferences.config?.pets.ai_ollama;const mode=selected==='@fast'?'fast':selected==='@reasoning'?'reasoning':pet?.mode??'auto';const models=!pet?[]:mode==='auto'?[pet.fast.model,pet.reasoning.model]:[pet[mode].model];return localCapabilities.label(id,selected,models,pet?.localExecution?.mode==='ask');}

const STRINGS = {
  placeholderFirst: N_("Ask me anything…"),
  placeholderNext: N_("Continue…"),
  send: N_("Send"),
  switchModel: N_("Switch provider or model"),
  noModel: N_("Choose a model"),
  loading: N_("Loading models…"),
  noKey: N_("No API key — add it in Settings."),
  openSettings: N_("Open Settings"),
};

let nextId = 1;

function bubble(message: ChatMessage): HTMLElement {
  if (message.role === "user") {
    return h(
      "div",
      { class: "chat-row user" },
      h("div", { class: "bubble", text: message.content }),
    );
  }
  const reply = h("div", { class: "reply" });
  renderMarkdown(reply, message.content);
  return h("div", { class: "chat-row" }, reply);
}

function typingDots(): HTMLElement {
  return h(
    "div",
    { class: "chat-row" },
    h("div", { class: "typing" }, h("i"), h("i"), h("i")),
  );
}

/** The coloured chip showing what the question is about (a dropped file). */
function contextChip(label: string): HTMLElement {
  const chip = h("div", { class: "chip" }, h("i", { class: "chip-dot" }), h("span", { text: label }));
  requestAnimationFrame(() => chip.classList.add("settled"));
  return chip;
}

function saveSettings() {
  void Bridge.saveSettings(State.settings).catch(Bridge.reportSettingsError);
}

// ── Model picker ──────────────────────────────────────────────────────────────

interface Picker {
  el: HTMLElement;
  open(): void;
  close(): void;
  readonly isOpen: boolean;
}

/** Provider chips, then the chosen provider's models — ModelPickerView. */
function buildPicker(onChange: () => void,onSelect?:(provider:string,model:string)=>void,nativeOnly:()=>boolean=()=>false,selected?:()=>string): Picker {
  const chips = h("div", { class: "picker-chips" });
  const search = h("input", {
    type: "search", class: "picker-search", placeholder: "Pesquisar modelos…",
    "aria-label": "Pesquisar modelos por nome ou identificador", spellcheck: "false",
    style: "box-sizing:border-box;width:100%;flex:none;border:1px solid #ffffff26;border-radius:8px;padding:8px 10px;background:#ffffff0a;color:var(--ink);font:400 12px var(--font)",
  }) as HTMLInputElement;
  const list = h("div", { class: "picker-list" });
  const selectionStatus=h('div',{class:'picker-selection-status',role:'status'});selectionStatus.hidden=true;
  const el = h("div", { class: "picker" }, chips, search, h("div", { class: "picker-rule" }),selectionStatus, list);

  /** Models already asked for, by provider; a model server is asked again each time. */
  const cache = new Map<string, ModelInfo[]>();
  let isOpen = false;
  let request = 0;
  let catalog:{provider:ProviderDef;models:ModelInfo[]}|null=null;
  let returnFocus:HTMLElement|null=null;
  const fold=(text:string)=>text.normalize("NFD").replace(/\p{M}/gu,"").toLocaleLowerCase();
  search.addEventListener("input",()=>{drawMatches();onChange();});
  search.addEventListener("keydown",event=>{
    const key=(event as KeyboardEvent).key;
    event.stopPropagation();
    if(key==="Escape"){event.preventDefault();close();}
    else if(key==="Enter")event.preventDefault();
    else if(key==="ArrowDown"){event.preventDefault();list.querySelector<HTMLButtonElement>(".picker-model")?.focus();}
  });

  function drawChips() {
    clear(chips);
    for (const p of visibleProviders(State.settings).filter(p => !State.settings.companionMode || p.id === providerForPill(State.focusId ?? undefined))) {
      const on = p.id === State.chatProvider;
      const chip = h(
        "button",
        { class: on ? "picker-chip on" : "picker-chip", style: `--accent:${p.accent}` },
        h("i", { class: "picker-dot" }),
        h("span", { text: t(p.name) }),
      );
      chip.addEventListener("click", () => {
        if (p.id === State.chatProvider) return;
        State.settings = { ...State.settings, chatProvider: p.id };
        saveSettings();
        State.notify();
        Sound.play("pop");
        search.value="";
        drawChips();
        void loadModels();
        onChange();
      });
      chips.append(chip);
    }
  }

  function status(text: string, withSettings = false) {
    catalog=null;
    clear(list);
    const line = h("div", { class: "picker-status", text });
    if (withSettings) {
      line.append(
        h("button", {
          class: "picker-link",
          text: tl(STRINGS.openSettings),
          onclick: () => void Bridge.openSettingsWindow(),
        }),
      );
    }
    list.append(line);
  }

  function drawModels(p: ProviderDef, models: ModelInfo[]) {
    catalog={provider:p,models};
    drawMatches();
  }

  function drawMatches() {
    if(!catalog)return;
    const {provider:p,models}=catalog;
    clear(list);
    const terms=fold(search.value.trim()).split(/\s+/).filter(Boolean);
    const matches=models.filter(m=>{const text=fold(m.label+" "+m.id);return terms.every(term=>text.includes(term));});
    if(!matches.length){
      list.append(h("div",{class:"picker-status",text:terms.length?"Sem modelos para esta pesquisa.":"Este provider não disponibiliza modelos."}));
      return;
    }
    const current = nativeOnly()?(selected?.()??''):activeModel({ ...State.settings, chatProvider: State.chatProvider });
    for (const m of matches) {
      const on = m.id === current;
      const row = h(
        "button",
        { class: on ? "picker-model on" : "picker-model", style: `--accent:${p.accent}`, title: m.id },
        h("span", { class: "picker-model-name", text: m.label }),
        on ? svg(ICONS.check, 11, { stroke: 2.2 }) : null,
      );
      row.addEventListener("click", async () => {
        const ticket=request;
        if(!nativeOnly()){
          if(p.id==='ollama'){
            selectionStatus.hidden=false;selectionStatus.textContent=`A selecionar ${m.label}…`;
            try{if(!await OllamaModelSelection.select(m.id)){if(isOpen&&ticket===request)selectionStatus.textContent='O modelo mudou noutra janela. Volta a escolhê-lo para alterar.';return;}}
            catch(error){if(isOpen&&ticket===request){selectionStatus.textContent=String(error).replace(/^Error:\s*/,'');onChange();}return;}
          }else{
            State.settings = { ...withModel(State.settings, p.id, m.id), chatProvider: State.settings.chatProvider };
            saveSettings();
          }
        }
        if(ticket!==request||!isOpen)return;
        onSelect?.(p.id,m.id);
        Sound.play("blip");
        close();
      });
      list.append(row);
    }
    list.querySelector(".picker-model.on")?.scrollIntoView({ block: "nearest" });
  }

  async function loadModels() {
    const p = providerDef(State.chatProvider);
    const ticket = ++request;
    catalog=null;
    const cached = p.urlField ? undefined : cache.get(p.id);
    if (cached) {
      drawModels(p,profileProvider(p.id)&&!nativeOnly()?[...profileModels,...cached]:cached);
      return;
    }
    status(t(STRINGS.loading));
    // Nothing is asked of a provider that has no key yet.
    if (p.key && !(await Bridge.secretPresent(p.key))) {
      if (ticket === request) status(t(STRINGS.noKey), true);
      return;
    }
    if (ticket !== request) return;
    try {
      const offered = await Bridge.chatModels(p.id);
      const models=profileProvider(p.id)&&!nativeOnly()?[...profileModels,...offered]:offered;
      if (ticket !== request) return;
      if (!p.urlField) cache.set(p.id, offered);
      const keep = pickModel(p, models.map((m) => m.id), activeModel({ ...State.settings, chatProvider: State.chatProvider }));
      if (!nativeOnly()&&p.id!=='ollama'&&keep && keep !== activeModel({ ...State.settings, chatProvider: State.chatProvider })) {
        State.settings = { ...withModel(State.settings, p.id, keep), chatProvider: State.settings.chatProvider };
        saveSettings();
        onChange();
      }
      drawModels(p, models);
    } catch (err) {
      if (ticket === request) status(String(err).replace(/^Error:\s*/, ""), Boolean(p.key));
    }
  }

  function open() {
    const active=document.activeElement as HTMLElement|null;
    returnFocus=active&&typeof active.focus==="function"?active:null;
    search.value="";
    selectionStatus.hidden=true;selectionStatus.textContent='';
    catalog=null;
    isOpen = true;
    el.classList.add("on");
    drawChips();
    onChange();
    void loadModels();
    requestAnimationFrame(()=>{if(isOpen)search.focus();});
  }

  function close() {
    if(!isOpen)return;
    const restoreFocus=document.activeElement===search||list.contains?.(document.activeElement);
    isOpen = false;
    request++;
    catalog=null;
    el.classList.remove("on");
    onChange();
    if(restoreFocus)returnFocus?.focus();
  }

  return {
    el,
    open,
    close,
    get isOpen() {
      return isOpen;
    },
  };
}

// ── View ──────────────────────────────────────────────────────────────────────

export interface DraftPromptHost extends ViewHost {restoreDraft?:(target:SessionTarget,text:string,previous:string)=>boolean}
export function buildPrompt(onHeightChange: () => void,session?:SessionChat,onDocumentActions?:()=>void,draftStatus?:HTMLElement): DraftPromptHost {
  watchCapabilities();
  if(session)return buildSessionPrompt(onHeightChange,session,draftStatus);
  const chipRow = h("div", { class: "chip-row" });
  let documentActionsButton:HTMLButtonElement|null=null;
  const log = h("div", { class: "chat-log" });
  const input = h("textarea", {
    rows:1,
    class: "chat-input",
    placeholder: t(STRINGS.placeholderFirst),
    spellcheck: "false",
    'aria-label':'Mensagem para este pet',
    title:'Enter para enviar · Shift + Enter para mudar de linha',
  }) as HTMLTextAreaElement;
  const fitInput=()=>{input.style.height='auto';input.style.height=`${Math.min(72,Math.max(20,input.scrollHeight))}px`;};
  input.addEventListener('input',()=>{fitInput();onHeightChange();});
  const send = h("button", { class: "send-btn", title: tl(STRINGS.send) }, svg(ICONS.arrowUp, 11));
  const bar = h("div", { class: "chat-bar" }, input, send);

  const modelDot = h("i", { class: "model-dot" });
  const modelName = h("span", { class: "model-name" });
  const modelBtn = h(
    "button",
    { class: "model-btn", title: tl(STRINGS.switchModel) },
    modelDot,
    modelName,
    svg(ICONS.chevronUpDown, 9, { stroke: 2 }),
  );
  const profileBtn=h("button",{class:"picker-link chat-profile",text:"Perfis",title:"Gerir modelos e esforço deste pet",onclick:()=>void Bridge.openSettingsWindow("pets:"+(State.focusId??"agent_codex"))});
  const modelRow = h("div", { class: "model-row" }, modelBtn,profileBtn);
  const executionBadge=h('p',{class:'local-execution-capability',role:'status'});executionBadge.hidden=true;
  const folderBtn=h("button",{class:"picker-link chat-folder",text:"Pasta de trabalho"}) as HTMLButtonElement;
  const folderInput=h("input",{class:"chat-folder-input",style:"width:100%;box-sizing:border-box;background:transparent;color:inherit;border:1px solid #ffffff38;border-radius:6px;padding:6px",type:"text",placeholder:"/home/…","aria-label":"Pasta de trabalho deste pet"}) as HTMLInputElement;
  const folderSave=h("button",{class:"picker-link",text:"Aplicar"}) as HTMLButtonElement;
  const folderError=h("p",{class:"chat-folder-error",style:"font-size:11px;color:#fbbf24"});
  const folderEditor=h("div",{class:"chat-folder-editor"},folderInput,folderSave,folderError);folderEditor.hidden=true;
  let folderOwner="";
  folderBtn.addEventListener("click",()=>{folderOwner=State.chatProvider;folderInput.value=State.settings.cliWorkspaces[folderOwner]??State.settings.cliWorkspace;folderError.textContent="";folderEditor.hidden=!folderEditor.hidden;if(!folderEditor.hidden)folderInput.focus();onHeightChange();});
  folderSave.addEventListener("click",()=>{const owner=folderOwner;folderSave.disabled=true;void Bridge.assistantSetWorkspace(owner,folderInput.value.trim()).then(path=>{
    const cliWorkspaces={...State.settings.cliWorkspaces};if(path)cliWorkspaces[owner]=path;else delete cliWorkspaces[owner];
    State.settings={...State.settings,cliWorkspaces};if(owner===State.chatProvider)folderEditor.hidden=true;State.notify();
  }).catch(error=>{if(owner===State.chatProvider)folderError.textContent=String(error).replace(/^Error:\s*/,"");}).finally(()=>{folderSave.disabled=false;onHeightChange();});});
  const expandChat=h("button",{class:"picker-link chat-expand",text:"Expandir",title:"Expandir esta conversa numa janela flutuante","aria-label":"Expandir esta conversa",onclick:()=>{const pet=State.focusId;if(pet)void Bridge.petChatOpen(pet).catch(error=>{State.chatFeedback.set(State.chatProvider,String(error).replace(/^Error:\s*/,""));State.notify();});}});
  modelRow.append(folderBtn,expandChat);
  const libraryBtn=h("button",{class:"picker-link chat-library-toggle",text:"Conversas",title:"Pesquisar e restaurar conversas deste pet"});
  const newConversation=h("button",{class:"picker-link chat-library-new",text:"Nova conversa",title:"Arquivar a conversa atual e começar outra"});
  const librarySearch=h("input",{class:"chat-library-search",type:"search",placeholder:"Pesquisar conversas","aria-label":"Pesquisar conversas deste pet"}) as HTMLInputElement;
  const libraryList=h("div",{class:"chat-library-list"});
  const libraryStatus=h("p",{class:"chat-library-status","role":"status"});
  const library=h("div",{class:"chat-library",style:"max-height:210px;overflow:auto;border:1px solid #ffffff24;border-radius:8px;padding:8px"},librarySearch,newConversation,libraryStatus,libraryList);library.hidden=true;
  modelRow.append(libraryBtn);
  let libraryOwner="",libraryTicket=0;
  async function loadLibrary() {
    const owner=State.chatProvider,ticket=++libraryTicket;libraryOwner=owner;
    libraryStatus.textContent="A carregar conversas…";
    try {
      const conversations=await Bridge.chatConversations(owner,librarySearch.value.trim());
      if(ticket!==libraryTicket || owner!==State.chatProvider || library.hidden)return;
      clear(libraryList);libraryStatus.textContent=conversations?.length?"":"Nenhuma conversa encontrada.";
      for(const c of conversations??[]) {
        const row=h("button",{class:"picker-model chat-library-entry",title:`${c.messageCount} mensagens · ${new Date(c.updated).toLocaleString()}`,style:"display:block;text-align:left;width:100%;white-space:normal"},
          h("strong",{text:`${c.current?"Atual · ":""}${c.title}`}),h("span",{text:c.preview,style:"display:block;opacity:.65;font-size:11px"}));
        row.disabled=c.current || State.chatBusyProviders.has(owner) || State.chatResettingProviders.has(owner);
        row.addEventListener("click",()=>{row.disabled=true;void restoreDialogue(owner,c.id).then(ok=>{
          if(ok)drafts.delete(owner);
          if(ok && owner===State.chatProvider) {input.value="";live=null;renderedCount=-1;library.hidden=true;State.notify();}
          else if(owner===State.chatProvider)void loadLibrary();
          onHeightChange();
        });});
        libraryList.append(row);
      }
    } catch(error) {if(ticket===libraryTicket && owner===State.chatProvider)libraryStatus.textContent=String(error).replace(/^Error:\s*/,"");}
    onHeightChange();
  }
  libraryBtn.addEventListener("click",()=>{library.hidden=!library.hidden;if(!library.hidden){librarySearch.value="";void loadLibrary();}onHeightChange();});
  const sessionsBtn=h('button',{class:'picker-link chat-sessions',text:'Todas as sessões',title:'Conversas do pet e sessões de terminal na barra lateral',onclick:()=>{if(State.focusId)void Bridge.petChatOpen(State.focusId,undefined,true);}});
  modelRow.append(sessionsBtn);
  librarySearch.addEventListener("input",()=>void loadLibrary());
  newConversation.addEventListener("click",()=>{const owner=State.chatProvider;newConversation.disabled=true;void resetDialogue(owner).then(ok=>{
    if(ok)drafts.delete(owner);
    if(ok && owner===State.chatProvider){input.value="";live=null;renderedCount=-1;State.notify();}
    if(!library.hidden && owner===State.chatProvider)void loadLibrary();
  }).finally(()=>{newConversation.disabled=false;onHeightChange();});});

  const body = h("div", { class: "chat-body" });
  const requests = h("div", { class: "cli-requests", style: "max-height:190px;overflow:auto" });
  const unavailable=h("p",{class:"pet-chat-unavailable"});unavailable.hidden=true;
  const feedback=h("p",{class:"chat-feedback",style:"font-size:11px;color:#fbbf24","role":"status","aria-live":"polite"});feedback.hidden=true;
  const failedQueries=new Map<string,{query:string;id?:string}>();
  const turnSnapshots=new Map<string,ConversationView>(),taskStatus=buildTaskNotice(),requestHistory=buildRequestHistory();
  const failedText=h('pre',{class:'failed-request-text'});
  const failedReview=h('details',{class:'failed-request-review'},h('summary',{text:'Rever pedido original · confirma o histórico antes de repetir'}),failedText);failedReview.hidden=true;
  const prepared=h("button",{class:"picker-link prepared-chat"});prepared.hidden=true;
  prepared.addEventListener("click",()=>{
    const draft=State.preparedChat;if(!draft || draft.provider!==State.chatProvider || pending.has(draft.provider))return;
    input.value=draft.text;State.preparedChat=null;prepared.hidden=true;input.focus();onHeightChange();
  });
  const picker = buildPicker(() => {
    body.classList.toggle("picking", picker.isOpen);
    drawModelButton();
  });
  body.append(chipRow, log, taskStatus.el,requests, picker.el,unavailable,prepared, modelRow,executionBadge,folderEditor,library,feedback,failedReview, bar);

  const requestCards = new Map<string, HTMLElement>();
  const endedRequests=new Set<string>();
  const addRequest=(request:{ id: string; provider: string; kind: string; detail: string;origin?:string;approvalScope?:'turn'|'once';
    questions?: { id: string; question: string; isSecret?: boolean; options?: { label: string }[] }[] | null
    target?:import('../core/session-chat').SessionTarget
  }) => {
    if(request.target?.origin==='external')return;
    if (requestCards.has(request.id)||endedRequests.has(request.id)) return;
    const card = h("div", { class: "cli-request", style: "padding:10px;border:1px solid #F5A524;border-radius:10px;margin:6px 0" },
      h("strong", { text: t("{provider} needs your answer", { provider: isCliProvider(request.provider.toLowerCase())?providerDef(request.provider.toLowerCase()).name:request.provider }) }));
    card.append(h('p',{class:'session-origin',text:request.origin??'Mochi · conversa própria'}));
    const fields = new Map<string, HTMLInputElement>();
    if (request.kind === "question") {
      for (const question of request.questions ?? []) {
        const input = h("input", { class: "chat-input", type: question.isSecret ? "password" : "text", autocomplete: "off" }) as HTMLInputElement;
        fields.set(question.id, input);
        card.append(h("label", { text: question.question }), input);
        for (const option of question.options ?? []) {
          card.append(h("button", { text: option.label, onclick: () => { input.value = option.label; } }));
        }
      }
    } else { card.append(commandReview(request.detail,request.provider==='ollama'&&request.approvalScope==='once')); }
    const reply = (allow: boolean) => {
      allowButton.disabled=denyButton.disabled=true;requestError.textContent='';
      const answers = Object.fromEntries([...fields].map(([id, input]) => [id, allow ? [input.value] : []]));
      void Bridge.cliAgentAnswer(request.id, { allow, answers },request.target).then(() => {
        card.remove(); requestCards.delete(request.id);
      }).catch(error=>{requestError.textContent=String(error).replace(/^Error:\s*/,'');allowButton.disabled=denyButton.disabled=false;onHeightChange();});
    };
    const allowButton=h("button", { class: "cli-allow", text: request.kind === "question" ? t("Send") : request.approvalScope==='turn'?'Permitir neste turno':t("Allow once"), onclick: () => reply(true) });
    const denyButton=h("button", { class: "cli-deny", text: t("Deny"), onclick: () => reply(false) });
    const requestError=h('p',{class:'chat-feedback',role:'status'});card.append(allowButton,denyButton,requestError);
    requestCards.set(request.id, card);
    card.dataset.provider = request.provider.toLowerCase();
    card.dataset.conversationId=request.target?.conversationId??(request.target?.origin==='mochi'?request.target.sessionId:'');
    card.hidden = card.dataset.provider !== State.chatProvider || !!card.dataset.conversationId&&card.dataset.conversationId!==dialogueId(State.chatProvider);
    requests.append(card);
    onHeightChange();
  };
  void onEvent('agent-request',addRequest);
  void Bridge.cliAgentPending().then(requests=>requests.forEach(addRequest)).catch(error=>console.error('Não foi possível recuperar os pedidos pendentes.',error));
  void onEvent<{ id: string }>("agent-request-ended", ({ id }) => {
    endedRequests.add(id);if(endedRequests.size>256)endedRequests.delete(endedRequests.values().next().value!);requestCards.get(id)?.remove(); requestCards.delete(id);
  });

  const el = h("div", { class: "view" }, h("div", { class: "card wash chat-card" }, body));
  (el.querySelector(".card") as HTMLElement).style.setProperty("--wash", "rgba(99,102,241,0.5)");

  const pending = new Map<string,{text:string}>();
  const observed=new Map<string,{id:string;turn:number;text:string;query:string}>();
  const busy=(provider:string)=>pending.has(provider)||observed.has(provider);
  let renderedCount = -1;
  let renderedProvider = "";
  const drafts = new Map<string, string>();
  PetPreferences.subscribe(()=>{drawModelButton();onHeightChange();});
  const historyRequests=new Map<string,number>();
  async function refreshHistory(provider:string) {
    if(pending.has(provider) || State.chatResettingProviders.has(provider))return;
    const ticket=(historyRequests.get(provider)??0)+1;historyRequests.set(provider,ticket);
    const before=State.chatHistoryFor(provider),count=before.length;
    let snapshot;
    try {snapshot=await Bridge.chatConversation(provider);} catch(error) {
      State.chatFeedback.set(provider,String(error).replace(/^Error:\s*/,""));if(provider===State.chatProvider)State.notify();return;
    }
    const messages=snapshot?.messages;
    if(!messages || historyRequests.get(provider)!==ticket || State.chatHistoryFor(provider)!==before || before.length!==count || pending.has(provider) || State.chatResettingProviders.has(provider))return;
    if(dialogueId(provider) && dialogueId(provider)!==snapshot.id) {
      State.clearDialogueContext(provider);drafts.delete(provider);
      if(provider===State.chatProvider)input.value="";
    }
    rememberDialogue(provider,snapshot);
    if(provider==='ollama')localCapabilities.observe(snapshot.id,snapshot.turn??0,snapshot.localExecution);
    const running=observed.get(provider);
    if(snapshot.turn!==undefined && running && running.id===snapshot.id && running.turn>snapshot.turn)return;
    const previousNotice=turnSnapshots.get(provider)?.notice;
    if(snapshot.notice)State.chatFeedback.set(provider,snapshot.notice);
    else if(previousNotice&&State.chatFeedback.get(provider)===previousNotice)State.chatFeedback.delete(provider);
    turnSnapshots.set(provider,snapshot);
    if(snapshot.busy && snapshot.turn!==undefined) {
      observed.set(provider,{id:snapshot.id,turn:snapshot.turn,text:snapshot.stream??"",query:snapshot.query??""});State.chatBusyProviders.add(provider);
    } else if(snapshot.busy===false) {observed.delete(provider);State.chatBusyProviders.delete(provider);}
    State.replaceChatHistory(provider,messages.filter(m=>["user","assistant"].includes(m.role) && typeof m.content==="string").map(m=>({...m,id:nextId++})));
    if(provider===State.chatProvider){renderedCount=-1;State.notify();onHeightChange();}
  }
  void onEvent<string>("assistant-history-changed",provider=>void refreshHistory(provider));
  // A local model answers token by token: where its text so far is shown.
  let live: HTMLElement | null = null;

  function drawModelButton() {
    const p = providerDef(State.chatProvider);
    modelDot.style.background = p.accent;
    modelName.textContent = profileLabel(activeModel({ ...State.settings, chatProvider: State.chatProvider })) || (isCliProvider(p.id) ? p.name : t(STRINGS.noModel));
    const selected=activeModel({...State.settings,chatProvider:State.chatProvider});
    executionBadge.textContent=State.chatProvider==='ollama'?capabilityText(dialogueId('ollama')??'',selected):'';executionBadge.hidden=!executionBadge.textContent;
    const summary=profileSummary(p.id,selected)||(isCliProvider(p.id)?p.name:t(STRINGS.noModel));
    modelName.textContent=summary;
    modelBtn.title = `${p.name} · ${summary} · ${t(STRINGS.switchModel)}`;
    modelBtn.setAttribute("aria-label",`Modelo: ${p.name} · ${summary}`);
    modelBtn.classList.toggle("open", picker.isOpen);
    const busyHere = busy(State.chatProvider);
    const resetting=State.chatResettingProviders.has(State.chatProvider);
    modelBtn.disabled = busyHere || resetting;
    const observedTurn=observed.get(State.chatProvider);
    const stoppable = (isCliProvider(State.chatProvider) || State.chatProvider==="ollama") && !!observedTurn?.id && observedTurn.turn!==undefined;
    send.title = busyHere && stoppable ? t("Stop") : t(STRINGS.send);
    send.disabled = resetting || (busyHere && !stoppable);
    send.textContent = busyHere && stoppable ? "■" : "↑";
    libraryBtn.disabled=newConversation.disabled=busyHere || resetting;
  }

  modelBtn.addEventListener("click", () => {
    if (pending.has(State.chatProvider)) return;
    if (picker.isOpen) picker.close();
    else picker.open();
  });

  function drawLive(text:string) {
    if (!text) return;
    if (!live) {
      live = h("div", { class: "reply" });
      log.querySelector(".typing")?.parentElement?.remove();
      log.append(h("div", { class: "chat-row" }, live));
    }
    const follow=!requestHistory.hasEntries||log.scrollHeight-log.scrollTop-log.clientHeight<56;
    renderMarkdown(live, text);
    if(follow)log.scrollTop = log.scrollHeight;
  }
  void onEvent<ChatTurnState>("chat-turn-state",event=>{
    const provider=event.provider,known=dialogueId(provider),old=observed.get(provider);
    if(known && known!==event.conversationId) {void refreshHistory(provider);return;}
    if(old && old.id===event.conversationId && old.turn>event.turn)return;
    if(event.busy) {
      observed.set(provider,{id:event.conversationId,turn:event.turn,text:old?.turn===event.turn?old.text:"",query:event.query??""});
      State.chatBusyProviders.add(provider);
    } else {
      observed.delete(provider);
      if(!pending.has(provider))State.chatBusyProviders.delete(provider);
    }
    if(provider===State.chatProvider && !pending.has(provider)){live=null;renderedCount=-1;}
    State.notify();onHeightChange();
    if(!event.busy)void refreshHistory(provider);
  });
  const documentReads=new Map<string,number>();
  void onEvent<{provider:string|null;conversationId:string|null}>('pet-document-context-changed',async event=>{
    if(!event.provider){State.clearAllDialogueContexts();State.notify();return;}
    const provider=event.provider,ticket=(documentReads.get(provider)??0)+1;documentReads.set(provider,ticket);
    if(!event.conversationId){State.clearDialogueContext(provider);State.notify();return;}
    try{
      const current=await Bridge.chatConversation(provider);
      const file=await Bridge.petDocumentContext(provider,current.id);
      if(documentReads.get(provider)!==ticket)return;
      if(file)State.setFileContext(provider,file);else State.clearDialogueContext(provider);
      State.notify();
    }catch{/* The receiving surface reports copy errors without changing another owner. */}
  });
  void onEvent<string | {provider:string;text:string;conversationId?:string;turn?:number}>("chat-delta", (delta) => {
    // Old unscoped events are accepted only when there is exactly one owner.
    const provider=typeof delta === "string" ? (pending.size===1 ? pending.keys().next().value : null) : delta.provider;
    const text=typeof delta === "string" ? delta : delta.text;
    const turn=provider ? pending.get(provider) : null;
    if(!provider || !text)return;
    const watching=observed.get(provider);
    if(typeof delta!=="string" && delta.conversationId) {
      if(dialogueId(provider) && dialogueId(provider)!==delta.conversationId)return;
      if(!watching || watching.id!==delta.conversationId || watching.turn!==delta.turn)return;
      watching.text=text;
    } else if(!turn)return;
    if(turn)turn.text=text;
    if(provider===State.chatProvider)drawLive(text);
  });

  async function submit() {
    if(State.settings.companionMode && !providerForPill(State.focusId??undefined))return;
    const query = input.value.trim();
    if (!query || busy(State.chatProvider) || State.chatResettingProviders.has(State.chatProvider)||(State.chatProvider==='ollama'&&OllamaModelSelection.pending)) return;
    if (picker.isOpen) picker.close();
    input.value = "";
    const sendingProvider = State.chatProvider;
    const expectedModel=sendingProvider==='ollama'?activeModel({...State.settings,chatProvider:'ollama'}):undefined;
    failedQueries.delete(sendingProvider);
    State.chatFeedback.delete(sendingProvider);
    pending.set(sendingProvider,{text:""});
    State.chatBusyProviders.add(sendingProvider);
    const owner = State.tasks.find(t => providerForPill(t.id) === sendingProvider)?.id;
    const notifyReply = (text: string) => {
      if(owner && State.settings.companionMode && (State.mode !== "expanded" || State.focusId !== owner)) void emitToWindow("companions","companions-notice",{key:`chat:${sendingProvider}:${nextId}`,pet:owner,text,urgent:false,view:"prompt"});
    };
    const history = State.chatHistory;
    drawModelButton();
    Sound.play("send");

    history.push({ id: nextId++, role: "user", content: query });
    State.stateOverride = "thinking";
    State.notify();
    onHeightChange();

    const file = State.fileFor(sendingProvider);

    try {
      if(!dialogueId(sendingProvider)) {
        const snapshot=await Bridge.chatConversation(sendingProvider);
        if(snapshot?.id)rememberDialogue(sendingProvider,snapshot);
      }
      if(file?.conversationId&&file.conversationId!==dialogueId(sendingProvider))throw new Error('O documento pertence a outra conversa. Volta a entregá-lo ao pet.');
      const context:ChatContext|null=file?{kind:'file',name:file.name,path:file.path}:null;
      const reply = await Bridge.chatSend(query, context, sendingProvider,dialogueId(sendingProvider),expectedModel,file?.sequence);
      history.push({ id: nextId++, role: "assistant", content: reply.text });
      if(file&&State.fileFor(sendingProvider)===file){State.clearDialogueContext(sendingProvider);if(file.sequence!==undefined&&file.conversationId)void Bridge.petDocumentClear(sendingProvider,file.conversationId,file.sequence).catch(()=>{});}
      notifyReply("Your answer is ready.");
      State.stateOverride = pending.size>1 ? "thinking" : null;
      Sound.play("finish");
    } catch (err) {
      State.stateOverride = pending.size>1 ? "thinking" : null;
      const message = String(err).replace(/^Error:\s*/, "");
      failedQueries.set(sendingProvider,{query,id:dialogueId(sendingProvider)});
      if (message !== "The CLI request was cancelled.") {
        history.push({id:nextId++,role:"assistant",content:message});
        if(State.chatProvider === sendingProvider) { State.noteMessage = message; State.view = "note"; }
        notifyReply("The chat could not finish. Open it to see the error.");
        Sound.play("error");
      }
    } finally {
      pending.delete(sendingProvider);
      if(!observed.has(sendingProvider))State.chatBusyProviders.delete(sendingProvider);
      if(State.chatProvider===sendingProvider){live = null;renderedCount = -1;} // only replace the owner's stream
      drawModelButton();
      State.notify();
      onHeightChange();
      void refreshHistory(sendingProvider);
      if(document.hasFocus?.()&&State.mode === "expanded" && State.view === "prompt" && State.chatProvider === sendingProvider) input.focus();
    }
  }

  send.addEventListener("click", () => {
    const watching=observed.get(State.chatProvider);
    if (busy(State.chatProvider)) {
      const provider=State.chatProvider;
      if(watching?.id&&watching.turn!==undefined){
        send.disabled=true;
        void Bridge.chatCancel(provider,watching.id,watching.turn).then(async confirmed=>{
          await refreshHistory(provider);
          if(confirmed!==true&&observed.get(provider)?.id===watching.id&&observed.get(provider)?.turn===watching.turn)State.chatFeedback.set(provider,'Não foi possível confirmar a interrupção deste turno. Atualiza a conversa antes de tentar novamente.');
        }).catch(error=>{State.chatFeedback.set(provider,String(error).replace(/^Error:\s*/,''));}).finally(()=>{State.notify();drawModelButton();});
      }
    }
    else void submit();
  });
  input.addEventListener("keydown", (e) => {
    const key = (e as KeyboardEvent).key;
    if (key === "Enter"&&!e.shiftKey&&!e.isComposing) {
      e.preventDefault();
      void submit();
    } else if (key === "Escape" && picker.isOpen) {
      e.preventDefault();
      picker.close();
    }
    e.stopPropagation(); // Escape closes the island, not the chat
  });

  return {
    el,
    sync() {
      const noChat=State.settings.companionMode && !providerForPill(State.focusId??undefined);
      const snapshot=turnSnapshots.get(State.chatProvider),cid=dialogueId(State.chatProvider),pet=State.focusId;
      const target=pet&&cid&&providerForPill(pet)===State.chatProvider?{petId:pet,provider:State.chatProvider,origin:'mochi' as const,sessionId:cid,conversationId:cid}:null;
      const historyChanged=requestHistory.sync(noChat?null:target,snapshot?.requestHistory,State.chatHistory.length);
      body.classList.toggle('has-request-history',requestHistory.hasEntries);
      if(historyChanged){renderedCount=-1;live=null;}
      const noticeWasHidden=taskStatus.el.hidden;
      taskStatus.sync(noChat?null:target,snapshot?{...snapshot,taskNotice:requestHistory.coversTask(snapshot.taskNotice?.taskId)?undefined:snapshot.taskNotice,busy:!!snapshot.busy}:null,busy(State.chatProvider));
      if(noticeWasHidden!==taskStatus.el.hidden)onHeightChange();
      const message=State.chatFeedback.get(State.chatProvider)??"";feedback.hidden=!message || !!noChat;feedback.textContent=message;
      const failed=failedQueries.get(State.chatProvider);failedReview.hidden=!failed||failed.id!==dialogueId(State.chatProvider);failedText.textContent=failed?.query??'';
      bar.hidden=modelRow.hidden=log.hidden=!!noChat;unavailable.hidden=!noChat;
      if(noChat)unavailable.textContent=petCapabilities(State.focusId??"").role;
      profileBtn.hidden=!profileProvider(State.chatProvider) || !!noChat;
      libraryBtn.hidden=!!noChat;
      sessionsBtn.hidden=!!noChat||!State.focusId||document.body.classList.contains('floating-chat');
      expandChat.hidden=!!noChat || !State.focusId || document.body.classList.contains("floating-chat");
      if(noChat || (libraryOwner && libraryOwner!==State.chatProvider)){library.hidden=true;libraryTicket++;}
      folderBtn.hidden=!isCliProvider(State.chatProvider) || !!noChat;
      const path=State.settings.cliWorkspaces[State.chatProvider]??State.settings.cliWorkspace;
      folderBtn.textContent=`Pasta: ${path.split(/[\\/]/).filter(Boolean).at(-1)??"pessoal"}`;folderBtn.title=path||"Pasta pessoal · usa as instruções da tua conta";
      folderBtn.disabled=busy(State.chatProvider) || State.chatResettingProviders.has(State.chatProvider);
      if(folderOwner!==State.chatProvider || noChat)folderEditor.hidden=true;
      if (renderedProvider !== State.chatProvider) {
        if (renderedProvider) drafts.set(renderedProvider, input.value);
        input.value = drafts.get(State.chatProvider) ?? "";
        if (State.settings.companionMode && picker.isOpen) picker.close();
        renderedProvider = State.chatProvider; renderedCount = -1; live = null;
        void refreshHistory(renderedProvider);
      }
      for (const card of requestCards.values()) card.hidden = card.dataset.provider !== State.chatProvider || !!card.dataset.conversationId&&card.dataset.conversationId!==dialogueId(State.chatProvider);
      const file = State.droppedFile;
      const wantChip = file?.name ?? "";
      if (chipRow.dataset.label !== wantChip) {
        chipRow.dataset.label = wantChip;
        clear(chipRow);
        documentActionsButton=null;
        if (wantChip) chipRow.append(contextChip(wantChip));
        if(wantChip&&onDocumentActions){
          documentActionsButton=h('button',{class:'picker-link',text:'Ações do documento','aria-label':'Ações para o documento anexado',onclick:()=>{if(State.pendingApproval?.pillId!==State.focusId)onDocumentActions();}});
          chipRow.append(documentActionsButton);
        }
      }
      if(documentActionsButton)documentActionsButton.disabled=State.pendingApproval?.pillId===State.focusId;

      const thinking = busy(State.chatProvider);
      const draft=State.preparedChat?.provider===State.chatProvider?State.preparedChat:null;
      prepared.hidden=!draft;prepared.disabled=thinking;
      if(draft){
        if(!thinking && !input.value.trim()){input.value=draft.text;State.preparedChat=null;prepared.hidden=true;}
        else prepared.textContent=`Usar pedido preparado de ${draft.source}`;
      }
      const count = State.chatHistory.length + (thinking ? 0.5 : 0);
      if (count !== renderedCount && !live) {
        renderedCount = count;
        const follow=!requestHistory.hasEntries||log.scrollHeight-log.scrollTop-log.clientHeight<56,previousTop=log.scrollTop;
        clear(log);
        for(let i=0;i<=State.chatHistory.length;i++){
          log.append(...requestHistory.before(i));if(i<State.chatHistory.length)log.append(bubble(State.chatHistory[i]));
        }
        log.append(...requestHistory.unplaced());
        if (thinking) {
          const watching=observed.get(State.chatProvider);
          if(!pending.has(State.chatProvider) && watching?.query&&!requestHistory.coversPending(watching.query))log.append(bubble({id:0,role:"user",content:watching.query}));
          log.append(typingDots());drawLive(pending.get(State.chatProvider)?.text || watching?.text || "");
        }
        log.scrollTop = follow?log.scrollHeight:previousTop;
      }

      // Leaving the chat folds the picker away.
      if (State.view !== "prompt" && picker.isOpen) picker.close();
      drawModelButton();

      input.placeholder = t(State.chatHistory.length === 0 ? STRINGS.placeholderFirst : STRINGS.placeholderNext);
      input.disabled = State.chatResettingProviders.has(State.chatProvider)||(State.chatProvider==='ollama'&&OllamaModelSelection.pending);
      fitInput();if(historyChanged)onHeightChange();
    },
    focus() {
      input.focus();
      input.select();
    },
  };
}

/** The same chat surface with an explicit session binding. It deliberately
 * shares the pet's bubbles, Markdown, cards and composer rather than showing a
 * terminal activity log in place of a conversation. */
function buildSessionPrompt(onHeightChange:()=>void,session:SessionChat,draftStatus?:HTMLElement):DraftPromptHost {
  const log=h('div',{class:'chat-log','aria-label':'Conversa da sessão','aria-live':'off'});
  const feedback=h('p',{class:'chat-feedback','role':'status','aria-live':'polite'});
  const input=h('textarea',{rows:1,class:'chat-input',placeholder:t(STRINGS.placeholderNext),spellcheck:'false','aria-label':'Mensagem para esta sessão','aria-describedby':'session-composer-hint'});
  const send=h('button',{class:'send-btn',title:tl(STRINGS.send)},svg(ICONS.arrowUp,11));
  const model=h('span',{class:'model-name'});
  const modelButton=h('button',{class:'model-btn'},h('i',{class:'model-dot'}),model,svg(ICONS.chevronUpDown,9,{stroke:2}));
  const profiles=h('button',{class:'picker-link chat-profile',text:'Perfis',onclick:()=>{
    const pet=session.target?.petId;if(pet)void Bridge.openSettingsWindow('pets:'+pet);
  }});
  const modelRow=h('div',{class:'model-row'},modelButton,profiles);
  const executionBadge=h('p',{class:'local-execution-capability',role:'status'});executionBadge.hidden=true;
  const attachments=h('div',{class:'chips'});
  const taskStatus=buildTaskNotice(),requestHistory=buildRequestHistory();
  const failedText=h('pre',{class:'failed-request-text'});
  const failedReview=h('details',{class:'failed-request-review'},h('summary',{text:'Rever pedido original · confirma o histórico antes de repetir'}),failedText);failedReview.hidden=true;
  const decisions=buildAgentRequests(()=>session.target,onHeightChange);
  const picker=buildPicker(()=>{body.classList.toggle('picking',picker.isOpen);onHeightChange();},(provider,model)=>{if(provider!=='ollama')session.chooseModel(model);},()=>session.target?.origin==='external',()=>session.model||session.snapshot?.model||'');
  const hint=h('p',{id:'session-composer-hint',class:'composer-hint',text:'Enter para enviar · Shift + Enter para mudar de linha'});
  const body=h('div',{class:'chat-body'},attachments,log,taskStatus.el,decisions.el,picker.el,modelRow,executionBadge,feedback,failedReview,...(draftStatus?[draftStatus]:[]),h('div',{class:'chat-bar'},input,send),hint);
  modelButton.addEventListener('click',()=>{if(picker.isOpen)picker.close();else picker.open();});
  const el=h('div',{class:'view'},h('div',{class:'card wash chat-card'},body));
  (el.querySelector('.card') as HTMLElement).style.setProperty('--wash','rgba(99,102,241,0.5)');
  let rendered='',owner='';
  const messageNodes=new Map<string,{signature:string;el:HTMLElement}>();
  let emptyNode:HTMLElement|null=null,dots:HTMLElement|null=null;
  const fitInput=()=>{input.style.height='auto';input.style.height=`${Math.min(128,Math.max(20,input.scrollHeight-14))}px`;};
  PetPreferences.subscribe(()=>{sync();onHeightChange();});
  input.addEventListener('input',()=>{if(session.target)session.drafts.set(sessionTargetKey(session.target),input.value);fitInput();sync();});
  async function submit(){
    const target=session.target,query=input.value.trim();if(!target||!query||session.sending||session.snapshot?.busy||!session.snapshot?.capabilities.send||session.error||(target.provider==='ollama'&&OllamaModelSelection.pending))return;
    const key=sessionTargetKey(target);
    input.value='';session.drafts.set(key,'');fitInput();
    await session.send(query,target.provider==='ollama'?activeModel({...State.settings,chatProvider:'ollama'}):undefined);
    sync();
  }
  send.addEventListener('click',()=>{if(session.snapshot?.busy)void session.cancel();else void submit();});
  input.addEventListener('keydown',e=>{
    if(e.key==='Enter'&&!e.shiftKey&&!e.isComposing){e.preventDefault();void submit();}
    if(e.key!=='Escape'&&!((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'))e.stopPropagation();
  });
  function sync(){
    decisions.sync();
    const target=session.target,snapshot=session.snapshot,key=target?sessionTargetKey(target):'';
    if(target?.provider==='ollama'&&target.origin==='mochi'&&snapshot)localCapabilities.observe(target.sessionId,Number(snapshot.turn??0),snapshot.localExecution);
    executionBadge.textContent=target?.provider==='ollama'&&target.origin==='mochi'?capabilityText(target.sessionId,activeModel({...State.settings,chatProvider:'ollama'})):'';executionBadge.hidden=!executionBadge.textContent;
    // Ollama's native selection is shared by text and voice. An old session
    // override must never advertise or send a model preceding the current ACK.
    if(target?.provider==='ollama')session.models.delete(key);
    if(key!==owner){if(owner)session.drafts.set(owner,input.value);owner=key;input.value=session.drafts.get(key)??'';fitInput();rendered='';messageNodes.clear();emptyNode=null;dots=null;clear(log);if(picker.isOpen)picker.close();}
    const waiting=session.sending||!!snapshot?.busy;
    requestHistory.sync(target,snapshot?.requestHistory,snapshot?.messages.length??0);
    taskStatus.sync(target,snapshot?{...snapshot,taskNotice:requestHistory.coversTask(snapshot.taskNotice?.taskId)?undefined:snapshot.taskNotice}:null,session.sending);
    const attachment=snapshot?.attachment,label=attachment?`${attachment.name}:${attachment.sequence??''}`:'';
    if(attachments.dataset.label!==label){attachments.dataset.label=label;clear(attachments);if(attachment){attachments.append(contextChip(attachment.name),h('button',{class:'picker-link',text:'Retirar anexo',onclick:()=>void session.clearAttachment()}));}}
    failedReview.hidden=!session.failure;failedText.textContent=session.failure?.query??'';
    const messages=snapshot?.messages??[];
    // Preserve scroll position during live updates if the user is reading older
    // messages. Only a new selection or a reader already at the end follows it.
    const follow=log.scrollHeight-log.scrollTop-log.clientHeight<56||log.dataset.owner!==key;
    const emptyState=taskStatus.el.hidden&&!requestHistory.hasEntries&&!messages.length&&!waiting&&!session.query?(session.error?'error':session.loading?'loading':snapshot?'empty':'opening'):'';
    const signature=JSON.stringify([key,messages,waiting,session.query,emptyState,requestHistory.signature]);
    if(signature!==rendered){
      rendered=signature;
      const desired:HTMLElement[]=[],wanted=new Set<string>();
      const showMessage=(id:string,m:{role:'user'|'assistant';content:string})=>{
        wanted.add(id);const signature=JSON.stringify([m.role,m.content]);let cached=messageNodes.get(id);
        if(!cached||cached.signature!==signature){const next={signature,el:bubble({...m,id:0})};cached?.el.replaceWith(next.el);cached=next;messageNodes.set(id,cached);}
        desired.push(cached.el);
      };
      for(let i=0;i<=messages.length;i++){desired.push(...requestHistory.before(i));if(i<messages.length)showMessage(messages[i].id,messages[i]);}
      desired.push(...requestHistory.unplaced());
      if(emptyState){
        const title=emptyState==='error'?'Não foi possível abrir a conversa':emptyState==='loading'||emptyState==='opening'?'A abrir a conversa…':target?.origin==='external'?'Estás nesta sessão de terminal':'Começa uma conversa';
        const description=emptyState==='error'?'Usa Atualizar sessões para tentar novamente.':emptyState==='loading'||emptyState==='opening'?'O histórico vai aparecer aqui.':target?.origin==='external'?'As mensagens escritas aqui pertencem apenas à sessão selecionada.':'Escreve o que precisas. Podes voltar a esta conversa na lista de sessões.';
        if(!emptyNode||emptyNode.dataset.state!==emptyState){emptyNode=h('div',{class:'session-empty'},h('strong',{text:title}),h('p',{text:description}));emptyNode.dataset.state=emptyState;}
        desired.push(emptyNode);
      }
      if(session.query&&!snapshot?.busy&&!requestHistory.coversPending(session.query))showMessage('__pending_query__',{role:'user',content:session.query});
      if(waiting){dots??=typingDots();desired.push(dots);}
      for(const [id,cached]of messageNodes)if(!wanted.has(id)){cached.el.remove();messageNodes.delete(id);}
      const retained=new Set(desired);for(const child of Array.from(log.children))if(!retained.has(child as HTMLElement))child.remove();
      desired.forEach((child,index)=>{if(log.children[index]!==child)log.insertBefore(child,log.children[index]??null);});
      if(follow)log.scrollTop=log.scrollHeight;log.dataset.owner=key;
    }
    const notices=[session.error,snapshot?.notice].filter((value,index,all)=>value&&all.indexOf(value)===index);
    const text=notices.join('\n\n')||(session.loading&&!snapshot?'A carregar a conversa…':waiting?'A trabalhar nesta sessão.':'');
    feedback.hidden=!text;feedback.textContent=text;
    input.disabled=!snapshot?.capabilities.send;
    input.placeholder=snapshot&&!snapshot.capabilities.send?'Esta sessão permite apenas leitura.':target?.origin==='external'?'Mensagem para esta sessão de terminal…':'Escreve uma mensagem…';
    const stoppable=!!snapshot?.busy&&snapshot.capabilities.cancel&&!!snapshot.turn;
    send.disabled=waiting?!stoppable:input.disabled||!input.value.trim()||!!session.error||(target?.provider==='ollama'&&OllamaModelSelection.pending);
    send.textContent=waiting?'■':'↑';send.title=waiting?'Interromper esta sessão':t(STRINGS.send);
    send.setAttribute('aria-label',send.title);
    const provider=target?.provider??'';
    model.textContent=provider==='ollama'?profileSummary('ollama',activeModel({...State.settings,chatProvider:'ollama'})):profileLabel(session.model??'')||session.model||snapshot?.model||(target?.origin==='mochi'?profileSummary(provider,activeModel({...State.settings,chatProvider:provider as typeof State.chatProvider})):null)||'Modelo da sessão';
    model.title=target?`${sessionOrigin(target)} · ${model.textContent}`:'';
    const nativeModel=target?.origin==='external'&&provider!=='codex';
    modelButton.disabled=waiting||!snapshot?.capabilities.send||nativeModel;modelButton.title=`${nativeModel?'Modelo escolhido no terminal':'Modelo para o próximo pedido'} · ${model.textContent}`;modelButton.setAttribute('aria-label',modelButton.title);
    profiles.disabled=waiting;profiles.hidden=target?.origin==='external'||!profileProvider(provider);
    onHeightChange();
  }
  if(typeof ResizeObserver!=='undefined'){
    let width=0;new ResizeObserver(entries=>{const next=entries[0]?.contentRect.width;if(next!==undefined&&next!==width){width=next;fitInput();}}).observe(input);
  }
  return {el,sync,focus:()=>{fitInput();input.focus();},restoreDraft:(target,text,previous)=>{
    const key=sessionTargetKey(target);if(key!==owner||!session.target||sessionTargetKey(session.target)!==key||input.value!==previous)return false;
    session.drafts.set(key,text);input.value=text;fitInput();sync();return true;
  }};
}
