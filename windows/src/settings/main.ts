import {buildShortcutReconciliation} from './shortcuts-reconciliation';
import {SettingsRequestIntent} from './request-intent';
// The Coucou window (settings.html): Flow dictation, the Notetaker and, under
// Notch, every setting of the island — the place where anything that writes to
// disk is confirmed. The window's frame and the Flow pages live in ../hub; the
// island's settings are drawn here, one Notch page at a time.

import "./settings.css";
import {parseRecapPrefs,parseRecapReceipt,recapReceiptSuccess,recapFailureMessage} from "../recap/recap";
import {buildIslandAvailability} from "./island-availability";
import {buildAutostartSetting} from './autostart-setting';
import {buildAutostartReview} from './autostart-review';
import {buildShortcutReview} from './shortcuts-review';
import {ShortcutPreferenceWriter} from '../core/shortcuts-settings';
import {buildSettingsIoStatus} from './io-status';
import {islandAvailabilityValues,type IslandAvailabilityValues} from "../core/island-availability";
import {OllamaModelSelection} from "../core/ollama-selection";
import { Bridge, onEvent, type HookPreview, type HookStatus, type ShortcutsReport } from "../core/bridge";
import { CUSTOM_SERVER_KEY, providerDef, urlExposure, withModel } from "../core/providers";
import {
  ISLAND_SHORTCUTS, SHORTCUTS, SHORTCUT_TEXT, activeKeys, displayKeys, duplicates, effective,
  recordPress, type Binding,
} from "../core/shortcuts";
import { DEFAULT_SETTINGS, type Settings } from "../core/state";
import {PetRosterPreferences,type PetRosterRequest} from '../core/pet-roster-settings';
import { SOUND_NAMES } from "../core/sound";
import {
  MAX_DECLARED, PILL_CATEGORIES, availablePills, isComingSoon, mainPillChoices,
  sanitizeDeclared, type PillDefinition,
} from "../core/pills";
import { h, clear } from "../views/dom";
import { startHub, type PageId,type HubContext } from "../hub/shell";
import { FLOW_PAGE_RENDERERS } from "../hub/pages";
import { agentsSection } from "./agents";
import { phoneSection } from "./phone";
import { colorDot } from "./colors";
import { renderDiff, statusDot } from "./parts";
import {secretControls} from './secret-controls';
import {
  LANGUAGES, N_, isRtl, onLanguageChange, resolveLanguage, setLanguage, systemLanguages, t, tn,
} from "../i18n/i18n";

/** Where secrets.rs keeps the keys on this OS. */
const KEY_STORE = navigator.userAgent.includes("Windows")
  ? "Windows Credential Manager"
  : "Secret Service (GNOME Keyring, KWallet)";

let settings: Settings = { ...DEFAULT_SETTINGS };
const storageStatus=buildSettingsIoStatus(()=>Bridge.settingsIoStatus(),{prepare:()=>Bridge.settingsReviewPrepare(),apply:request=>Bridge.settingsReviewApply(request),cancel:request=>Bridge.settingsReviewCancel(request)});
let rosterSaving=false;
const rosterPreferences=new PetRosterPreferences({save:request=>Bridge.petRosterSettingsPatch(request),adopt:roster=>{settings={...settings,...roster};for(const redraw of declaredViews)redraw();}});
async function changeRoster(request:PetRosterRequest){
 if(rosterSaving)return;
 rosterSaving=true;for(const redraw of declaredViews)redraw();
 try{await rosterPreferences.change(request);}catch(error){Bridge.reportSettingsError(error);}
 finally{rosterSaving=false;for(const redraw of declaredViews)redraw();}
}
let version = "";
let islandAvailability:ReturnType<typeof buildIslandAvailability>|null=null;
let autostartSetting:ReturnType<typeof buildAutostartSetting>|ReturnType<typeof buildAutostartReview>|null=null;
let observedIslandAvailability:IslandAvailabilityValues|null=null;
let settingsObservation=0,settingsListenerReady=false;
let settingsSubscription:Promise<boolean>|null=null;
function availabilityControls(){
  islandAvailability??=buildIslandAvailability({initial:observedIslandAvailability,save:value=>Bridge.petCenterSettingsPatch(value),read:async()=>{if(!await ensureSettingsSubscription())throw Error("Preferences unavailable");const boot=await Bridge.boot();if(!boot)throw Error("Preferences unavailable");return boot.settings;},adopt:flags=>{observedIslandAvailability={...flags};settings={...settings,...flags};}});
  islandAvailability.render();return islandAvailability.el;
}
function autostartControls(){
 if(navigator.userAgent.includes('Linux')){
  autostartSetting??=buildAutostartReview({status:()=>Bridge.autostartReviewStatus(),prepare:request=>Bridge.autostartReviewPrepare(request),apply:request=>Bridge.autostartReviewApply(request),cancel:request=>Bridge.autostartReviewCancel(request)},{initial:settings.autostart,adopt:value=>{settings={...settings,autostart:value};}});
 }else autostartSetting??=buildAutostartSetting({initial:settings.autostart,save:async value=>{const receipt=await Bridge.autostartSettingsPatch({enabled:value});if(receipt.registration==='unconfirmed')throw Error(receipt.reason??'O registo de arranque não foi confirmado.');return receipt;},read:async()=>{if(!await ensureSettingsSubscription())throw Error('As preferências estão indisponíveis.');const boot=await Bridge.boot();if(!boot)throw Error('As preferências estão indisponíveis.');return boot.settings.autostart;},adopt:value=>{settings={...settings,autostart:value};}});
 autostartSetting.render();return autostartSetting.el;
}

const windowRoot = document.getElementById("hub")!;
/** Where the Notch page being shown draws its sections. */
let root: HTMLElement = windowRoot;
let notchContext:HubContext|undefined;
let hub: { redraw(): void; current(): PageId } | null = null;

/** One section lifetime, including A→leave→A and a hub redraw of the same page. */
function settingsSectionCurrent(section:HTMLElement){
 const targetRoot=root,targetPage=notchPage,targetContext=notchContext;
 const lifetime=new SettingsRequestIntent(()=>root===targetRoot&&notchPage===targetPage&&notchContext===targetContext&&section.isConnected);
 const ticket=lifetime.begin(),previousLeave=targetContext?.onLeave;
 if(targetContext)targetContext.onLeave=()=>{lifetime.dispose();previousLeave?.();};
 return ()=>lifetime.current(ticket);
}

async function save() {
  try{await Bridge.saveSettings(settings);}finally{void storageStatus.refresh();}
}

/** A colour was picked for a pill's Mochi (see ./colors.ts): the island follows. */
function pickColor(next: Record<string, string>) {
  settings.pillColors = next;
  void save().catch(Bridge.reportSettingsError);
}

// ── Reusable bits ─────────────────────────────────────────────────────────────

function toggle(on: boolean, onChange: (v: boolean) => void): HTMLElement {
  const el = h("button", { class: on ? "switch on" : "switch", "aria-pressed": on });
  el.addEventListener("click", () => {
    const next = !el.classList.contains("on");
    el.classList.toggle("on", next);
    el.setAttribute('aria-pressed',String(next));
    onChange(next);
  });
  return el;
}

// ── Changes to Claude Code's settings.json ────────────────────────────────────

/** One kind of change to ~/.claude/settings.json, with the words that go with it. */
interface Change {
  preview: (install: boolean) => Promise<HookPreview | null>;
  apply: (install: boolean, fingerprint: string) => Promise<string | null>;
  installText: string;
  removeText: string;
  installButton: string;
  removeButton: string;
  /** The note once written; `backup` is "" when there was no file to back up. */
  done: (backup: string) => string;
}

const HOOKS_CHANGE: Change = {
  preview: Bridge.hooksPreview,
  apply: Bridge.hooksApply,
  get installText() { return t("This is exactly what will change in your settings.json. Your own hooks are left untouched."); },
  get removeText() { return t("This removes Coucou's entries only. Your own hooks are left untouched."); },
  get installButton() { return t("Back up and write"); },
  get removeButton() { return t("Back up and remove"); },
  done: (backup) => backup
    ? t("Done. Previous settings saved as {backup}. Open a new Claude Code session to pick the hooks up.", { backup })
    : t("Done. Open a new Claude Code session to pick the hooks up."),
};

const STATUS_LINE_CHANGE: Change = {
  preview: Bridge.statusLinePreview,
  apply: Bridge.statusLineApply,
  get installText() { return t("This is exactly what will change: only the status line. If you already have one it keeps working, Coucou's relay runs it for you."); },
  get removeText() { return t("This puts your previous status line back, or removes the entry if there was none."); },
  get installButton() { return t("Back up and write"); },
  get removeButton() { return t("Back up and remove"); },
  done: (backup) => backup
    ? t("Done. Previous settings saved as {backup}. The numbers appear after the next reply of a Claude Code session.", { backup })
    : t("Done. The numbers appear after the next reply of a Claude Code session."),
};

/**
 * Shows the diff of a change in `body` and writes it only after an explicit
 * click, and only if settings.json still matches the diff that was shown.
 * `back` redraws the section; `applied` runs a moment after a successful write.
 */
async function reviewChange(
  body: HTMLElement,
  change: Change,
  install: boolean,
  back: () => void,
  applied: () => void,
) {
  let preview;
  try {
    preview = await change.preview(install);
  } catch (err) {
    // An unreadable or invalid settings.json stops here rather than being
    // treated as empty and written over.
    clear(body);
    body.append(
      h("div", { class: "notice err", text: String(err).replace(/^Error:\s*/, "") }),
      h("div", { class: "row" }, h("button", { text: t("Back"), onclick: back })),
    );
    return;
  }
  if (!preview) return;
  clear(body);
  body.append(
    h("div", { class: "hint", text: install ? change.installText : change.removeText }),
    renderDiff(preview.diff),
    h("div", { class: "row" },
      h("span", {
        class: "path",
        text: preview.backup
          ? t("Backup → {path}", { path: preview.backup })
          : t("No settings.json yet — nothing to back up."),
      }),
    ),
  );
  const confirm = h("button", {
    class: install ? "primary" : "danger",
    text: install ? change.installButton : change.removeButton,
  });
  confirm.addEventListener("click", async () => {
    confirm.disabled = true;
    try {
      const backup = await change.apply(install, preview.fingerprint);
      clear(body);
      body.append(h("div", { class: "notice ok", text: change.done(backup ?? "") }));
      window.setTimeout(applied, 2600);
    } catch (err) {
      confirm.disabled = false;
      body.append(h("div", { class: "notice err", text: t("Could not write: {error}", { error: String(err) }) }));
    }
  });
  body.append(h("div", { class: "row" }, confirm, h("button", { text: t("Cancel"), onclick: back })));
}

// ── Claude Code section ───────────────────────────────────────────────────────

function claudeSection(status: HookStatus): HTMLElement {
  const body = h("div", { style: "display:flex;flex-direction:column;gap:12px" });
  const section = h(
    "section",
    {},
    h("h2", {}, statusDot(status.installed), h("span", { text: "Claude Code" })),
    body,
  );

  const redraw = () => {
    clear(body);
    draw();
  };
  const rebuild = async () => {
    const fresh = await Bridge.hooksStatus();
    if (fresh) Object.assign(status, fresh);
    redraw();
    const head = section.querySelector("h2")!;
    clear(head);
    head.append(statusDot(status.installed), h("span", { text: "Claude Code" }));
  };

  function draw() {
    body.append(
      h("div", {
        class: "hint",
        text: status.installed
          ? t("Coucou is hooked into your Claude Code sessions. Tool calls, questions and permission requests show up in the island, and you can answer them there.")
          : t("Install the hooks to see your Claude Code sessions in the island and approve permissions without leaving what you are doing."),
      }),
      h("div", { class: "row" },
        h("label", { text: "settings.json" }),
        h("span", { class: "path", text: status.settingsPath }),
      ),
      h("div", { class: "row" },
        h("label", { text: t("Relay") }),
        h("span", { class: "path", text: status.hookPath }),
        statusDot(status.hookReady),
      ),
    );

    if (!status.hookReady) {
      body.append(h("div", {
        class: "notice warn",
        text: t("coucou-hook.exe is not in place yet. Restart Coucou; if it still fails, build it with `cargo build -p coucou-hook`."),
      }));
    }

    const actions = h("div", { class: "row" });
    const install = h("button", {
      class: "primary",
      text: status.installed ? t("Reinstall hooks…") : t("Install hooks…"),
      onclick: () => void reviewChange(body, HOOKS_CHANGE, true, redraw, () => void rebuild()),
    });
    // Writing hook commands that point at a relay which isn't there would give
    // every Claude Code session a broken hook and nothing to show for it.
    if (!status.hookReady) {
      install.disabled = true;
      install.title = t("The relay isn't installed yet.");
    }
    actions.append(install);
    if (status.installed) {
      actions.append(h("button", {
        class: "danger",
        text: t("Uninstall hooks…"),
        onclick: () => void reviewChange(body, HOOKS_CHANGE, false, redraw, () => void rebuild()),
      }));
    }
    body.append(actions);
  }

  draw();
  return section;
}

// ── Plan usage section ────────────────────────────────────────────────────────

/**
 * The 5-hour and weekly limits in the island's header. They come from Claude
 * Code's status line, so the relay has to be the status line first: turning the
 * switch on without it starts the install, and the switch only stays on once
 * that has been confirmed. A status line the user had keeps working.
 */
const PLAN_SETTINGS_TEXT = {
  get claude() { return t("Shows your Claude plan usage (5-hour and weekly limits) in the island's header. Coucou adds a status line relay in ~/.claude/settings.json. If you already have a status line, it keeps working as before. Pro and Max plans only."); },
  get claudeCli() { return t("The numbers come from the Claude Code command-line tool. The Claude desktop app does not report them, so the gauge stays empty if you only use the desktop app."); },
  get showClaude() { return t("Show in notch"); },
  get codex() { return t("Shows your Codex plan usage (weekly limit and free resets left) in the island's header. Coucou asks the Codex CLI (codex app-server) when the pill shows; nothing is installed. Codex must be signed in with ChatGPT."); },
  get showCodex() { return t("Show Codex plan in the notch"); },
};

function planSection(status: HookStatus): HTMLElement {
  const body = h("div", { style: "display:flex;flex-direction:column;gap:12px" });
  const section = h("section", {}, h("h2", {}, h("span", { text: t("Plan usage") })), body);

  const redraw = () => {
    clear(body);
    draw();
  };
  const rebuild = async () => {
    const fresh = await Bridge.hooksStatus();
    if (fresh) Object.assign(status, fresh);
    settings.planRelayInstalled = status.planRelayInstalled;
    // Cancelled or failed: a switch that was waiting for the install falls back.
    if (!status.planRelayInstalled) settings.showPlanInNotch = false;
    redraw();
  };

  function draw() {
    // The switch shows "on" while the install it asked for is being reviewed.
    const sw = toggle(settings.showPlanInNotch, (on) => {
      if (!on) {
        settings.showPlanInNotch = false;
        void save().catch(Bridge.reportSettingsError);
      } else if (status.planRelayInstalled) {
        settings.showPlanInNotch = true;
        void save().catch(Bridge.reportSettingsError);
      } else {
        // Turned on before the relay is in: install it first; it stays on once confirmed.
        void reviewChange(body, STATUS_LINE_CHANGE, true, () => void rebuild(), () => {
          settings.planRelayInstalled = true;
          settings.showPlanInNotch = true;
          void save().then(rebuild).catch(Bridge.reportSettingsError);
        });
      }
    });
    body.append(
      h("div", {
        class: "hint",
        text: PLAN_SETTINGS_TEXT.claude,
      }),
      h("div", { class: "hint", text: PLAN_SETTINGS_TEXT.claudeCli }),
      h("div", { class: "row" }, h("label", { text: PLAN_SETTINGS_TEXT.showClaude }), sw),
      h("div", { class: "row" },
        h("label", { text: t("Relay") }),
        statusDot(status.planRelayInstalled),
        h("span", { class: "hint", text: status.planRelayInstalled ? t("installed") : t("not installed") }),
        status.planRelayInstalled
          ? h("button", {
              class: "danger",
              text: t("Uninstall relay…"),
              onclick: () => void reviewChange(body, STATUS_LINE_CHANGE, false, redraw, () => void rebuild()),
            })
          : h("button", {
              class: "primary",
              text: t("Install relay…"),
              onclick: () => void reviewChange(body, STATUS_LINE_CHANGE, true, redraw, () => void rebuild()),
            }),
      ),
      // Codex: nothing to install, Coucou asks the Codex CLI when the pill shows.
      h("div", { class: "hint", text: PLAN_SETTINGS_TEXT.codex }),
      h("div", { class: "row" },
        h("label", { text: PLAN_SETTINGS_TEXT.showCodex }),
        toggle(settings.showCodexPlanInNotch, (on) => {
          settings.showCodexPlanInNotch = on;
          void save().catch(Bridge.reportSettingsError);
        }),
      ),
    );
  }

  draw();
  return section;
}

// ── Claude API section ────────────────────────────────────────────────────────

const MODELS: [string, string][] = [
  ["claude-opus-5", "Claude Opus 5"],
  ["claude-sonnet-5", "Claude Sonnet 5"],
  ["claude-haiku-4-5", "Claude Haiku 4.5"],
];

function apiSection(hasKey: boolean|null): HTMLElement {
  const section=h("section",{}),viewCurrent=settingsSectionCurrent(section),intent=new SettingsRequestIntent(viewCurrent);
  const dot = statusDot(hasKey===true);
  if(hasKey===null)dot.style.background="#94a3b8";
  const state = h("span", { class: "hint", text: hasKey===null?"Estado da chave por confirmar.":hasKey ? t("Key saved in the {store}.", { store: KEY_STORE }) : t("No key yet — the chat needs one.") });

  const field = h("input", {
    type: "password",
    placeholder: hasKey ? `••••••••••••  ${t("(stored)")}` : "sk-ant-...",
    style: "flex:1 1 auto;min-width:0",
    autocomplete: "off",
    spellcheck: "false",
  }) as HTMLInputElement;

  const saveBtn = h("button", { class: "primary", text: t("Save key") });
  const clearBtn = h("button", { class: "danger", text: t("Remove") });
  const feedback = h("div", {});

  let pending:'save'|'remove'|null=null,draft=0;
  feedback.setAttribute('role','status');feedback.setAttribute('aria-live','polite');
  function controls(){saveBtn.disabled=pending!==null||!field.value.trim();clearBtn.disabled=pending!==null;saveBtn.setAttribute('aria-busy',String(pending==='save'));clearBtn.setAttribute('aria-busy',String(pending==='remove'));}
  field.addEventListener('input',()=>{draft++;controls();});
  async function refresh(ticket:number){
    const present=await Bridge.secretPresent("anthropic-api-key");if(!intent.current(ticket))return;
    if(typeof present!=='boolean'){state.textContent='Estado da chave por confirmar.';dot.style.background='#94a3b8';return;}
    hasKey=present;dot.style.background=present?'#22c55e':'#f4505e';
    state.textContent=present?t("Key saved in the {store}.",{store:KEY_STORE}):t("No key yet — the chat needs one.");
    field.placeholder=present?`••••••••••••  ${t("(stored)")}`:"sk-ant-...";clearBtn.style.display=present?'':'none';
  }
  async function apply(remove:boolean){
    const value=field.value.trim();if(pending||!viewCurrent()||!remove&&!value)return;
    const ticket=intent.begin(),savedDraft=draft;pending=remove?'remove':'save';controls();clear(feedback);feedback.append(h('div',{class:'hint',text:remove?'A remover…':'A guardar…'}));
    try{
      if(remove)await Bridge.secretClear("anthropic-api-key");else await Bridge.secretSet("anthropic-api-key",value);
      if(!intent.current(ticket))return;
      hasKey=!remove;if(!remove&&draft===savedDraft)field.value='';
      clearBtn.style.display=hasKey?'':'none';
      clear(feedback);feedback.append(h('div',{class:'notice ok',text:remove?t("Key removed."):draft!==savedDraft?"A gravação anterior da chave foi concluída.":t("Key saved in the {store}.",{store:KEY_STORE})}));
      if(field.value)feedback.append(h('div',{class:'hint',text:'O rascunho neste campo continua por guardar.'}));
      await refresh(ticket);
    }catch{
      if(intent.current(ticket)){clear(feedback);feedback.append(h('div',{class:'notice err',text:remove?'Não foi possível confirmar a remoção da chave.':'Não foi possível confirmar a gravação da chave enviada.'}));}
    }finally{if(intent.current(ticket)){pending=null;controls();}}
  }
  saveBtn.onclick=()=>void apply(false);clearBtn.onclick=()=>void apply(true);
  field.addEventListener('keydown',event=>{if(event.key==='Enter'&&!event.isComposing){event.preventDefault();void apply(false);}});controls();

  const model = h("select", {}) as HTMLSelectElement;
  for (const [id, label] of MODELS) model.append(h("option", { value: id, text: label }));
  if (!MODELS.some(([id]) => id === settings.model)) {
    model.append(h("option", { value: settings.model, text: settings.model }));
  }
  model.value = settings.model;
  model.addEventListener("change", () => {
    settings.model = model.value;
    void save().catch(Bridge.reportSettingsError);
  });

  clearBtn.style.display = hasKey ? "" : "none";

  section.append(
    h("h2", {}, dot, h("span", { text: "Claude" })),
    state,
    h("div", { class: "row" }, h("label", { text: t("API key") }), field, saveBtn, clearBtn),
    h("div", { class: "row" }, h("label", { text: t("Model") }), model),
    feedback,
  );
  return section;
}

// ── Active pills section ──────────────────────────────────────────────────────

/**
 * The tools you use (Mac 0.1.1–0.1.2): pick the main workspace tool, which is
 * always on and takes no slot, and declare the agents and chat providers you
 * want as pills. Services are declared in Integrations below, next to their keys.
 */
function activePillsSection(connected: Record<string, boolean>): HTMLElement {
  const slots = h("div", { class: "hint" });
  const main = h("select", {}) as HTMLSelectElement;
  for (const def of mainPillChoices()) main.append(h("option", { value: def.id, text: def.name }));
  main.addEventListener("change", () => {
    const id=main.value;main.value=settings.mainPill;
    void changeRoster({action:'setMain',id});
  });
  const groups = h("div", { style: "display:flex;flex-direction:column;gap:12px" });

  /** Why a pill would show nothing yet, as on the Mac's row. */
  function hint(def: PillDefinition): string | null {
    if (isComingSoon(def.id)) return t("Coming soon");
    if (def.connect.kind === "hooks" && !connected[def.id]) return t("Hooks not installed");
    if (def.connect.kind === "key" && !connected[def.id]) return t("Key not configured");
    if (def.connect.kind === "server" && !settings[def.connect.field]) return t("Not connected");
    return null;
  }

  function row(def: PillDefinition): HTMLElement {
    const isMain = def.id === settings.mainPill;
    const on = settings.activeIntegrations.includes(def.id);
    const full = !isMain && !on && settings.activeIntegrations.length >= MAX_ACTIVE;
    const el = h("div", { class: full ? "pill-row full" : "pill-row" },
      colorDot(def, "width:10px;height:10px", () => settings.pillColors, pickColor),
      h("span", { class: "name", text: def.name }),
    );
    if (isMain) {
      el.append(h("span", { class: "state", text: t("Main") }));
      return el;
    }
    const why = hint(def);
    el.append(h("span", { class: "state", text: why ?? "" }));
    const sw = h("button", { class: on ? "switch on" : "switch",'aria-label':`${def.name} · Mostrar pet`,'aria-pressed':on }) as HTMLButtonElement;
    sw.disabled = full||rosterSaving;
    sw.addEventListener("click", () => {
      void changeRoster({action:'setActive',id:def.id,enabled:!settings.activeIntegrations.includes(def.id)});
    });
    el.append(sw);
    return el;
  }

  function draw() {
    const used = settings.activeIntegrations.length;
    slots.textContent = t("{used}/{max} slots in use — the main tool doesn't take one.", { used, max: MAX_ACTIVE });
    slots.classList.toggle("full", used >= MAX_ACTIVE);
    main.value = settings.mainPill;
    main.disabled=rosterSaving;
    clear(groups);
    for (const cat of PILL_CATEGORIES) {
      if (cat.id === "service") continue;
      const pills = availablePills().filter((p) => p.category === cat.id);
      if (pills.length === 0) continue;
      groups.append(h("div", { class: "pill-group" }, h("h3", { text: t(cat.title) }), ...pills.map(row)));
    }
  }
  declaredViews.push(draw);
  draw();

  return h(
    "section",
    {},
    h("h2", {}, h("span", { text: t("Active pills") })),
    h("div", { class: "hint", text: t("Choose the tools you use. Coucou only shows what you declare here.") }),
    slots,
    h("div", { class: "row" }, h("label", { text: t("Main tool") }), main),
    groups,
  );
}

// ── Chat providers section ────────────────────────────────────────────────────

const CHAT_STRINGS = {
  get providersTitle() { return t("Chat providers"); },
  get providersHint() { return t("Chat with Google AI, OpenAI or OpenRouter instead of Claude: add a key here, then click the model name above the chat box to switch provider and model. Keys stay in the system keychain. These providers get no web search and no tools: they can answer, never act on this computer."); },
  get stored() { return `••••••••  ${t("(stored)")}`; },
  get save() { return t("Save"); },
  get remove() { return t("Remove"); },
  get localTitle() { return t("Local models"); },
  get localHint() { return t("Chat with a model you run yourself: Ollama or LM Studio (leave the address empty for the usual one on this computer), or any server that speaks the OpenAI API, such as vLLM or llama.cpp. Once connected, pick it above the chat box."); },
  get connect() { return t("Connect"); },
  get connecting() { return t("Connecting…"); },
  get disconnect() { return t("Disconnect"); },
  get useInChat() { return t("Use in chat"); },
  get inUse() { return t("In use"); },
  get keyOptional() { return t("API key (optional)"); },
  get localOnly() { return t("Nothing leaves your PC: the server runs on this computer."); },
  get remote() { return t("This address is another machine: what you ask is sent to it."); },
  get remoteHttp() { return t("This address is another machine, over plain http: what you ask travels unencrypted."); },
  get keyOverHttp() { return t("Warning: the key would be sent unencrypted (http://) to another machine. Use https://, or a server on this computer."); },
  get invalid() { return t("Not a valid http:// or https:// address."); },
  noModels: (name: string) => t("No models yet. Download one in {name} first.", { name }),
  models: (n: number) => tn("{count} model", "{count} models", n),
};

interface CloudDef {
  id: "google" | "openai" | "openrouter";
  name: string;
  placeholder: string;
  where: string;
}

const CLOUD: CloudDef[] = [
  { id: "google", name: "Google AI", placeholder: "AIza…", where: "aistudio.google.com" },
  { id: "openai", name: "OpenAI", placeholder: "sk-…", where: "platform.openai.com" },
  { id: "openrouter", name: "OpenRouter", placeholder: "sk-or-…", where: "openrouter.ai/keys" },
];

function chatProvidersSection(
  present: Record<string, boolean>,
  keyChanged: (key: string, on: boolean) => void,
): HTMLElement {
  const list = h("div", { style: "display:flex;flex-direction:column;gap:8px" });
  for (const def of CLOUD) {
    const p = providerDef(def.id);
    const key = p.key!;
    const fields=secretControls(key,present[key]??false,def.placeholder,on=>{present[key]=on;keyChanged(key,on);});
    list.append(
      h("div", { class: "row" },
        h("label", {},
          h("i", { class: "dot", style: `background:${p.accent};margin-right:8px` }),
          h("span", { text: def.name }),
        ),
        fields.input, fields.save, fields.remove, fields.dot,
      ),
      h("div", { class: "hint", style: "margin:-4px 0 0 144px", text: t("Key from {site}", { site: def.where }) }),
      fields.feedback,
    );
  }
  for (const id of ["codex", "antigravity"] as const) {
    const provider = providerDef(id);
    const status = h("div", { class: "hint", text: t("Connect {provider}", {provider:provider.name}) });
    const connect = h("button", { text: t("Connect {provider}", { provider: provider.name }) }) as HTMLButtonElement;
    connect.addEventListener("click", async () => {
      connect.disabled = true;
      status.textContent = t("Connecting…");
      try {
        const models = await Bridge.chatModels(id);
        if (!models.length) throw new Error(t("The CLI returned no available models."));
        const model = models.find((m) => m.id === settings.chatModels[id]) ?? models[0];
        settings = {...withModel(settings,id,model.id),chatProvider:settings.chatProvider};
        await save();
        status.textContent = t("{provider} connected.", { provider: provider.name });
      } catch (err) { status.textContent = String(err).replace(/^Error:\s*/, ""); }
      finally { connect.disabled = false; }
    });
    list.append(h("div", { class: "row" }, h("label", { text: provider.name }), connect), status);
  }
  const workspace = h("input", { type: "text", value: settings.cliWorkspace, placeholder: t("Home folder"), style: "flex:1" }) as HTMLInputElement;
  workspace.addEventListener("change", () => { settings.cliWorkspace = workspace.value.trim(); void save().catch(Bridge.reportSettingsError); });
  list.append(h("div", { class: "row" }, h("label", { text: t("Working folder") }), workspace));
  const fullAccess = h("input", { type: "checkbox", checked: settings.cliFullAccess }) as HTMLInputElement;
  fullAccess.addEventListener("change", () => { settings.cliFullAccess = fullAccess.checked; void save().catch(Bridge.reportSettingsError); });
  list.append(h("label", { class: "row" }, fullAccess, h("span", { text: t("Full access in new Codex/Gravity conversations") })),
    h("div", { class: "hint", text: t("Full access runs commands and file changes with your user account. It applies only to new Codex/Gravity conversations. Existing sessions keep their native policy; choose a new conversation after changing this option or the working folder. Copilot keeps its native permissions.") }));
  return h(
    "section",
    {},
    h("h2", {}, h("span", { text: CHAT_STRINGS.providersTitle })),
    h("div", { class: "hint", text: CHAT_STRINGS.providersHint }),
    list,
  );
}

// ── Local models section ──────────────────────────────────────────────────────

type LocalId = "ollama" | "lmstudio" | "custom";

/** Redraws the local models section after a change made elsewhere (the island). */
let localRedraw: (() => void) | null = null;

const LOCAL: Record<LocalId, { name: string; usual: string }> = {
  ollama: { name: "Ollama", usual: "http://127.0.0.1:11434" },
  lmstudio: { name: "LM Studio", usual: "http://127.0.0.1:1234" },
  // No usual address: any server that speaks the OpenAI API.
  custom: { name: N_("OpenAI-compatible"), usual: "" },
};

/** What an address means for the user's data, as a hint line. */
function exposureNotice(url: string, withKey: boolean): HTMLElement | null {
  switch (urlExposure(url)) {
    case "local":
      return h("div", { class: "hint", text: CHAT_STRINGS.localOnly });
    case "remote":
      return h("div", { class: "hint", text: CHAT_STRINGS.remote });
    case "remote-http":
      return withKey
        ? h("div", { class: "notice warn", text: CHAT_STRINGS.keyOverHttp })
        : h("div", { class: "hint", text: CHAT_STRINGS.remoteHttp });
    case "invalid":
      return url.trim() ? h("div", { class: "notice err", text: CHAT_STRINGS.invalid }) : null;
  }
}

function localSection(customKey: boolean,lmstudioKey=false): HTMLElement {
  const body = h("div", { style: "display:flex;flex-direction:column;gap:14px" });
  const section = h(
    "section",
    {},
    h("h2", {}, h("span", { text: CHAT_STRINGS.localTitle })),
    h("div", { class: "hint", text: CHAT_STRINGS.localHint }),
    body,
  );
  const viewCurrent=settingsSectionCurrent(section);
  const blocks=new Map<LocalId,{el:HTMLElement;key:string}>();
  const drafts=new WeakMap<HTMLElement,{dirty:()=>boolean;changed:()=>void}>();
  const redraw = () => {
    for(const id of Object.keys(LOCAL) as LocalId[]){
      const url=settings[providerDef(id).urlField!],key=JSON.stringify([url,url!==""&&settings.chatProvider===id]);
      const previous=blocks.get(id);if(previous?.key===key)continue;
      const draft=previous&&drafts.get(previous.el);if(draft?.dirty()){draft.changed();continue;}
      const el=serverBlock(id);blocks.set(id,{el,key});
      if(previous)previous.el.replaceWith(el);else body.append(el);
    }
  };

  function serverBlock(id: LocalId): HTMLElement {
    const def = LOCAL[id];
    const p = providerDef(id);
    const field = p.urlField!;
    const connected = settings[field] !== "";
    const status = h("div", {});
    const exposure = h("div", {});
    const label = h("label", {},
      h("i", { class: "dot", style: `background:${p.accent};margin-right:8px` }),
      h("span", { text: t(def.name) }),
    );
    const block = h("div", { style: "display:flex;flex-direction:column;gap:6px" });

    if (connected) {
      const inUse = settings.chatProvider === id;
      const use = h("button", { class: inUse ? "" : "primary", text: inUse ? CHAT_STRINGS.inUse : CHAT_STRINGS.useInChat });
      use.disabled = inUse;
      use.addEventListener("click", () => {
        settings.chatProvider = id;
        void save().then(redraw).catch(Bridge.reportSettingsError);
      });
      const disconnect = h("button", { class: "danger", text: CHAT_STRINGS.disconnect });
      disconnect.addEventListener("click", async () => {
        settings[field] = "";
        if (settings.chatProvider === id) settings.chatProvider = "anthropic";
        if (["custom","lmstudio"].includes(id)) {
          await Bridge.secretClear(id==="lmstudio"?"lmstudio-local-api-key":CUSTOM_SERVER_KEY).catch(() => {});
          if(id==="lmstudio")lmstudioKey=false;else customKey=false;
        }
        try{await save();redraw();}catch(error){Bridge.reportSettingsError(error);}
      });
      block.append(
        h("div", { class: "row" }, label, h("span", { class: "path", text: settings[field] }), statusDot(true), use, disconnect),
        status,
      );
      exposure.append(exposureNotice(settings[field], id==="lmstudio"?lmstudioKey:id==="custom"&&customKey) ?? "");
      block.append(exposure);
      return block;
    }

    const input = h("input", {
      type: "text",
      placeholder: def.usual || "https://llm.example.com",
      style: "flex:1 1 auto;min-width:0",
      spellcheck: "false",
      autocomplete: "off",
    }) as HTMLInputElement;
    // A custom server may want a key; it goes to the keychain, never to settings.json.
    const key = h("input", {
      type: "password",
      "aria-label": `${CHAT_STRINGS.keyOptional} · ${t(def.name)}`,
      placeholder: (id==="lmstudio"?lmstudioKey:customKey) ? CHAT_STRINGS.stored : CHAT_STRINGS.keyOptional,
      style: "flex:1 1 auto;min-width:0",
      autocomplete: "off",
      spellcheck: "false",
    }) as HTMLInputElement;
    const connect = h("button", { class: "primary", text: CHAT_STRINGS.connect });
    const intent=new SettingsRequestIntent(viewCurrent);
    let pending:number|null=null,edited=false;
    drafts.set(block,{dirty:()=>edited,changed:()=>{if(pending===null&&!status.textContent)status.append(h("div",{class:"hint",text:"A configuração mudou. O rascunho neste campo continua por guardar."}));}});
    const changed=()=>{edited=true;intent.invalidate();pending=null;connect.disabled=false;connect.removeAttribute('aria-busy');clear(status);};
    input.addEventListener('input',changed);key.addEventListener('input',changed);

    const showExposure = () => {
      clear(exposure);
      const withKey = ["custom","lmstudio"].includes(id) && ((id==="lmstudio"?lmstudioKey:customKey) || key.value.trim() !== "");
      const notice = exposureNotice(input.value || def.usual, withKey);
      if (notice) exposure.append(notice);
    };
    input.addEventListener("input", showExposure);
    key.addEventListener("input", showExposure);

    connect.addEventListener("click", async () => {
      if(pending!==null||!viewCurrent()||!block.isConnected)return;
      const ticket=intent.begin(),address=input.value,secret=key.value.trim();pending=ticket;connect.disabled=true;connect.setAttribute('aria-busy','true');
      let savedTarget='';
      const current=()=>pending===ticket&&intent.current(ticket);
      const attached=()=>current()&&block.isConnected;
      clear(status);status.append(h("div",{class:"hint",text:CHAT_STRINGS.connecting}));
      try{
        if(["custom","lmstudio"].includes(id)&&secret){
          // Bind the already-authorized key write to the address captured by this gesture.
          await Bridge.localSetKey(address||def.usual,secret,id==="lmstudio"?"lmstudio":undefined);
          if(!attached())return;key.value="";if(id==="lmstudio")lmstudioKey=true;else customKey=true;
        }
        const server=await Bridge.localConnect(id,address);
        if(!attached())return;
        if(!server.models.length){clear(status);status.append(h("div",{class:"notice err",text:CHAT_STRINGS.noModels(t(def.name))}));}
        else{
          const model=server.models.includes(settings.chatModels[id]??"")?settings.chatModels[id]:server.models[0];
          const beforeModel=settings.chatModels[id],chatModels=id==="ollama"?{...settings.chatModels}:{...settings.chatModels,[id]:model};
          savedTarget=server.url;await Bridge.saveSettings({...settings,[field]:server.url,chatModels});if(!current())return;
          // An observed ACK may have redrawn this block. Do not overwrite a subsequent
          // disconnect/target or model choice while finishing that dispatched write.
          if(!block.isConnected&&(settings[field]!==server.url||settings.chatModels[id]!==chatModels[id]))return;
          if(id==="ollama"&&settings.chatModels.ollama!==beforeModel)return;
          settings={...settings,[field]:server.url,chatModels:id==="ollama"?settings.chatModels:{...settings.chatModels,[id]:model}};
          if(id==="ollama"){
            const selected=await OllamaModelSelection.select(model);if(!current()||settings[field]!==server.url)return;
            if(!selected)throw new Error('O modelo mudou noutra janela. Volta a escolhê-lo para alterar.');
            settings.chatModels={...settings.chatModels,ollama:model};
          }
          edited=false;redraw();return;
        }
      }catch(error){if(attached()){clear(status);status.append(h("div",{class:"notice err",text:String(error).replace(/^Error:\s*/,"")}));}else if(current()&&savedTarget&&settings[field]===savedTarget)Bridge.reportSettingsError(error);}
      finally{if(attached()){pending=null;connect.disabled=false;connect.removeAttribute('aria-busy');}}
    });

    block.append(h("div", { class: "row" }, label, input, connect));
    if (["custom","lmstudio"].includes(id)) block.append(h("div", { class: "row" }, h("label", { text: "" }), key));
    block.append(status, exposure);
    showExposure();
    return block;
  }

  redraw();
  localRedraw = redraw;
  return section;
}

// ── Integrations section ──────────────────────────────────────────────────────

interface IntegrationDef {
  id: string;
  name: string;
  color: string;
  /** Credential Manager keys, in the order they are shown. */
  fields: { key: string; label: string; placeholder: string; secret: boolean }[];
  /** What the key needs, shown under its field. */
  hint?: string;
}

const INTEGRATIONS: IntegrationDef[] = [
  { id: "integration_stripe", name: "Stripe", color: "#0570DE",
    fields: [{ key: "stripe-api-key", label: N_("Secret key"), placeholder: "sk_live_…", secret: true }] },
  { id: "integration_github", name: "GitHub", color: "#F4505E",
    fields: [{ key: "github-token", label: N_("Token"), placeholder: "ghp_…", secret: true }],
    hint: N_("Classic token with the repo scope, or fine-grained with read access to Pull requests, Commit statuses and Actions.") },
  { id: "integration_vercel", name: "Vercel", color: "#7C5CFF",
    fields: [{ key: "vercel-token", label: N_("Token"), placeholder: "…", secret: true }] },
  { id: "integration_n8n", name: "n8n", color: "#F29B38",
    fields: [
      { key: "n8n-url", label: N_("Instance URL"), placeholder: "https://n8n.example.com", secret: false },
      { key: "n8n-api-key", label: N_("API key"), placeholder: "…", secret: true },
    ] },
  { id: "integration_resend", name: "Resend", color: "#22C55E",
    fields: [{ key: "resend-api-key", label: N_("API key"), placeholder: "re_…", secret: true }] },
  { id: "integration_notion", name: "Notion", color: "#8C8C8C",
    fields: [{ key: "notion-api-key", label: N_("Integration token"), placeholder: "ntn_…", secret: true }] },
  { id: "integration_calcom", name: "Cal.com", color: "#C9956A",
    fields: [{ key: "calcom-api-key", label: N_("API key"), placeholder: "cal_…", secret: true }] },
  // Nothing to enter: Spotify is read from the OS (src-tauri/src/spotify.rs).
  { id: "integration_spotify", name: "Spoty · Música e som", color: "#1DB954", fields: [] },
];

const MAX_ACTIVE = MAX_DECLARED;

/** Everything that shows the declared pills, redrawn when any of them changes. */
const declaredViews: (() => void)[] = [];

function integrationsSection(present: Record<string, boolean>): HTMLElement {
  const note = h("div", { class: "hint" });
  const list = h("div", { style: "display:flex;flex-direction:column;gap:14px" });

  function updateNote() {
    const used = settings.activeIntegrations.length;
    note.textContent = t("Pick up to {max} pills to show next to Mochi — {used}/{max} in use. Keys are stored in the {store}, never on disk.", { max: MAX_ACTIVE, used, store: KEY_STORE });
  }
  declaredViews.push(updateNote);

  const offered = new Set(availablePills().map((p) => p.id));
  for (const def of INTEGRATIONS.filter((d) => offered.has(d.id))) {
    const active = settings.activeIntegrations.includes(def.id);
    const sw = h("button", { class: active ? "switch on" : "switch",'aria-label':`${def.name} · Mostrar pet`,'aria-pressed':active });
    const redraw=()=>{const on=settings.activeIntegrations.includes(def.id);sw.classList.toggle('on',on);sw.setAttribute('aria-pressed',String(on));sw.disabled=rosterSaving||(!on&&settings.activeIntegrations.length>=MAX_ACTIVE);};
    declaredViews.push(redraw);redraw();
    sw.addEventListener("click", () => {void changeRoster({action:'setActive',id:def.id,enabled:!settings.activeIntegrations.includes(def.id)});});

    const rows = h("div", { style: "display:flex;flex-direction:column;gap:6px;flex:1 1 auto;min-width:0" });
    for (const field of def.fields) {
      const fields=secretControls(field.key,present[field.key]??false,field.placeholder,on=>{present[field.key]=on;});
      fields.input.type=field.secret?'password':'text';
      rows.append(
        h("div", { class: "row" },
          h("label", { style: "min-width:104px", text: t(field.label) }),
          fields.input, fields.save, fields.remove, fields.dot,
        ),fields.feedback,
      );
    }

    if (def.hint) rows.append(h("div", { class: "hint", text: t(def.hint) }));
    if (def.id === "integration_spotify") {
      // As on the Mac's row: said only when there is no Spotify to launch.
      const hint = h("div", { class: "hint", style: "padding-top:5px" });
      rows.append(hint);
      void Bridge.spotifyInstalled().then((ok) => {
        hint.textContent = ok === false ? "Spotify não instalado. Os controlos de som do PC continuam disponíveis." : "Música, dispositivos e mistura de som por aplicação.";
      });
    }

    list.append(
      h("div", { style: "display:flex;gap:12px;align-items:flex-start" },
        h("div", { style: "display:flex;align-items:center;gap:8px;min-width:132px;padding-top:4px" },
          sw,
          colorDot(def, "", () => settings.pillColors, pickColor),
          h("span", { style: "font-size:12.5px", text: def.name }),
        ),
        rows,
      ),
    );
  }

  updateNote();
  return h("section", {}, h("h2", {}, h("span", { text: t("Integrations") })), note, list);
}

// ── General section ───────────────────────────────────────────────────────────

function generalSection(): HTMLElement {
  const volume = h("input", {
    type: "range", min: "0", max: "0.2", step: "0.005",
    value: String(settings.soundVolume),
  }) as HTMLInputElement;
  volume.addEventListener("input", () => {
    settings.soundVolume = Number(volume.value);
    void save().catch(Bridge.reportSettingsError);
  });

  // Your own sounds: the folder, a reload, and how many are replaced.
  const customCount = h("span", { class: "hint" });
  const countCustom = () =>
    void Bridge.customSounds(SOUND_NAMES).then((own) => {
      const n = own?.length ?? 0;
      customCount.textContent = n > 0 ? t("{0} custom", { 0: n }) : "";
    });
  countCustom();
  const soundsFolder = h("button", { text: t("Open sounds folder"), onclick: () => void Bridge.revealSoundsFolder() });
  const reloadSounds = h("button", {
    text: t("Reload sounds"),
    onclick: () => void Bridge.reloadSounds().then(() => window.setTimeout(countCustom, 300)),
  });

  const autoClose = h("input", {
    type: "number", min: "5", max: "120", step: "1",
    value: String(Math.round(settings.autoCloseInterval)),
    style: "width:72px",
  }) as HTMLInputElement;
  autoClose.addEventListener("change", () => {
    settings.autoCloseInterval = Math.max(5, Math.min(120, Number(autoClose.value) || 15));
    autoClose.value = String(settings.autoCloseInterval);
    void save().catch(Bridge.reportSettingsError);
  });

  const companionClearance = h("input", {
    type: "number", min: "0", max: "240", step: "8",
    value: String(settings.companionTopClearance), style: "width:72px",
  }) as HTMLInputElement;
  companionClearance.addEventListener("change", () => {
    const value = Number(companionClearance.value);
    settings.companionTopClearance = Number.isFinite(value) ? Math.max(0, Math.min(240, value)) : 64;
    companionClearance.value = String(settings.companionTopClearance);
    void save().catch(Bridge.reportSettingsError);
  });
  const screen = h("select", {}) as HTMLSelectElement;
  screen.append(
    h("option", { value: "primary", text: t("Main display") }),
    h("option", { value: "cursor", text: t("Display under the cursor") }),
  );
  screen.value = settings.screen;
  void Bridge.listMonitors().then((list) => {
    for (const m of list ?? []) screen.append(h("option", { value: m.key, text: m.label }));
    // Set again now the option exists. A display saved under an older key (moved,
    // resized, or saved before names were kept) is shown by its place or its
    // name; one that is gone shows as the main one.
    const saved = settings.screen;
    const [place, name] = saved.split("|");
    const keys = (list ?? []).map((m) => m.key);
    screen.value =
      keys.find((k) => k === saved) ??
      (saved.startsWith("at:") ? keys.find((k) => k.split("|")[0] === place) : undefined) ??
      (name ? keys.find((k) => k.split("|")[1] === name) : undefined) ??
      saved;
    if (!screen.value) screen.value = "primary";
  });
  screen.addEventListener("change", () => {
    settings.screen = screen.value;
    void save().catch(Bridge.reportSettingsError);
  });

  return h(
    "section",
    {},
    h("h2", {}, h("span", { text: t("General") })),
    h("div", { class: "row" },
      h("label", { text: t("Sound") }),
      toggle(settings.soundEnabled, (v) => { settings.soundEnabled = v; void save().catch(Bridge.reportSettingsError); }),
      volume,
    ),
    h("div", { class: "row" }, soundsFolder, reloadSounds, customCount),
    h("div", { class: "hint", text: t("Drop a file named like one of Mochi's sounds (finish.wav, approval.mp3, greet.m4a…) in the sounds folder to replace it, then Reload.") }),
    h("div", { class: "row" },
      h("label", { text: t("Open on hover") }),
      toggle(settings.openOnHover, (v) => { settings.openOnHover = v; void save().catch(Bridge.reportSettingsError); }),
    ),
    h("div", { class: "hint", text: t("Hovering the island opens it; it folds again shortly after the pointer leaves. Click inside to keep it open.") }),
    h("div", { class: "row" },
      h("label", { text: t("Auto-close") }),
      autoClose,
      h("span", { class: "hint", text: t("seconds after you leave the island") }),
    ),
    h("div", { class: "row" },
      h("label", { text: t("Desktop companion") }),
      toggle(settings.companionMode, (v) => { settings.companionMode = v; void save().catch(Bridge.reportSettingsError); }),
    ),
    h("div", { class: "hint", text: t("Each active integration has its own companion. Click a pet for its controls; right-click for actions. Restart Coucou after changing companion mode.") }),
    availabilityControls(),
    h("div", { class: "row" },
      h("label", { text: t("Companion movement") }),
      toggle(settings.companionMotion, v => { settings.companionMotion = v; void save().catch(Bridge.reportSettingsError); }),
    ),
    h("div", { class: "row" },
      h("label", { text: t("Use the free space in the top bar") }),
      toggle(settings.companionBarMode, v => { settings.companionBarMode = v; void save().catch(Bridge.reportSettingsError); }),
    ),
    h("div", { class: "hint", text: t("Live bar geometry keeps pets away from controls and windows. Without free space, use the tray or shortcuts.") }),
    h("div", { class: "row", style: settings.companionBarMode ? "display:none" : "" },
      h("label", { text: t("Keep clear of the top bar") }), companionClearance,
      h("span", { class: "hint", text: t("pixels") }),
    ),
    h("div", { class: "hint", style: settings.companionBarMode ? "display:none" : "", text: t("Pets stay in a narrow strip below this margin. Increase it if they cover a bar control. Movement also respects reduced motion.") }),
    h("div", { class: "row" },
      h("label", { text: t("Island lives on") }),
      screen,
    ),
    autostartControls(),
    ...recapRows(notchContext),
    languageRow(),
  );
}

/**
 * Settings → General → Language, as on the Mac: "System" follows the
 * system's language when Coucou has it (else English), or one of the ten.
 * Both windows and the tray switch in place, without a restart.
 */
function languageRow(): HTMLElement {
  const select = h("select", {}) as HTMLSelectElement;
  select.append(h("option", { value: "", text: t("System") }));
  for (const { code, name } of LANGUAGES) select.append(h("option", { value: code, text: name, lang: code }));
  select.value = LANGUAGES.some((l) => l.code === settings.language) ? settings.language : "";
  select.addEventListener("change", () => {
    settings.language = select.value;
    void save().catch(Bridge.reportSettingsError);
    applyLanguage();
  });
  return h("div", { class: "row" }, h("label", { text: t("Language") }), select);
}

// ── Shortcuts section ─────────────────────────────────────────────────────────

const SHORTCUTS_UI = {
  get title() { return t("Shortcuts"); },
  get hint() { return t("Work from any app. Click a shortcut to change it, then press the new keys — Esc cancels, Backspace removes it."); },
  get global() { return t("From anywhere"); },
  get island() { return t("In the open island"); },
  get recording() { return t("Press keys…"); },
  get none() { return t("None"); },
  get reset() { return t("Reset to defaults"); },
  get inUse() { return t("In use by another app"); },
  get duplicate() { return t("Used twice"); },
  get invalid() { return t("Not a valid shortcut"); },
  get unavailable() { return t("Not available"); },
  types: (ch: string) => t("Types “{char}”", { char: ch }),
  typesNote: (keys: string, ch: string) =>
    t("{keys} types “{char}” on your keyboard, so it can't be a shortcut. Pick another key.", { keys, char: ch }),
  get needsModifier() { return t("Hold Ctrl, Alt or the Windows key with it."); },
  get unsupportedKey() { return t("That key can't be used in a shortcut."); },
  get wayland() {
    return t("Your Wayland desktop doesn't let apps listen for keys outside their own windows. Add the shortcuts in your system's keyboard settings instead, with these commands:");
  },
  get noDisplay() { return t("No display server was found, so global shortcuts are off."); },
  get portalPending() {
    return t("Asking your desktop to register the shortcuts. It may show its own window to confirm them.");
  },
  get portalActive() {
    return t("These shortcuts are registered with your desktop. It may ask you to confirm them or to pick other keys; when it says which keys it uses, they show next to each shortcut.");
  },
  get portalFallback() {
    return t("If one doesn't work, you can also add it in your system's keyboard settings with these commands:");
  },
  desktopKeys: (keys: string) => t("Desktop: {keys}", { keys }),
  get refused() { return t("Not set by your desktop"); },
};

function shortcutsSection(initial: ShortcutsReport | null,context?:HubContext): HTMLElement {
  let sectionAlive=true,shortcutWrite:Promise<void>=Promise.resolve();
  const shortcutWriter=navigator.userAgent.includes('Linux')?new ShortcutPreferenceWriter({current:()=>settings.shortcuts,observation:()=>settingsObservation,alive:()=>sectionAlive,save:request=>Bridge.shortcutsSettingsPatch(request),adopt:bindings=>{settings={...settings,shortcuts:bindings};}}):null;
  const nativeReview=navigator.userAgent.includes('Linux')?buildShortcutReview({prepare:()=>Bridge.shortcutsReviewPrepare(),prepareCurrentScope:()=>Bridge.shortcutsReviewPrepareCurrentScope(),inspect:key=>Bridge.shortcutsReviewInspect(key),cancel:key=>Bridge.shortcutsReviewCancel(key),apply:decision=>Bridge.shortcutsReviewApply(decision),status:()=>Bridge.shortcutsReviewStatus()}):null;
  const nativeReconciliation=navigator.userAgent.includes('Linux')?buildShortcutReconciliation({prepare:()=>Bridge.shortcutsReconcilePrepare(),inspect:key=>Bridge.shortcutsReconcileInspect(key),cancel:key=>Bridge.shortcutsReviewCancel(key),apply:decision=>Bridge.shortcutsReconcileApply(decision)}):null;
  let report = initial;
  const list = h("div", { class: "shortcut-list" });
  const feedback = h("div", {});
  const blockedNote = h("div", {});

  let stopRecording: (() => void) | null = null;
  if(context)context.onLeave=()=>{sectionAlive=false;stopRecording?.();nativeReview?.dispose();nativeReconciliation?.dispose();shortcutsListener=null;};

  function store(id: string, binding: Binding) {
    nativeReview?.invalidate();nativeReconciliation?.invalidate();
    if(shortcutWriter)shortcutWrite=shortcutWriter.change(id,binding).catch(Bridge.reportSettingsError).finally(draw);
    else{settings.shortcuts = { ...settings.shortcuts, [id]: binding };shortcutWrite=save().catch(Bridge.reportSettingsError).finally(draw);}
  }

  function tagFor(id: string, dups: Set<string>): HTMLElement | null {
    if (dups.has(id)) return h("span", { class: "tag err", text: SHORTCUTS_UI.duplicate });
    const st = report?.actions.find((a) => a.id === id);
    switch (st?.status) {
      case "inUse": return h("span", { class: "tag warn", text: SHORTCUTS_UI.inUse });
      case "duplicate": return h("span", { class: "tag err", text: SHORTCUTS_UI.duplicate });
      case "invalid": return h("span", { class: "tag err", text: SHORTCUTS_UI.invalid });
      case "typesCharacter": return h("span", { class: "tag warn", text: SHORTCUTS_UI.types(st.typed ?? "?") });
      case "unsupported": return h("span", { class: "tag", text: SHORTCUTS_UI.unavailable });
      case "refused": return h("span", { class: "tag warn", text: SHORTCUTS_UI.refused });
      case "preferencesUnconfirmed": return h("span", { class: "tag warn", text: "As preferências aguardam confirmação; o atalho não foi registado." });
      // Wayland: the desktop may run it on other keys than the ones asked for.
      case "active": return st.trigger ? h("span", { class: "tag", text: SHORTCUTS_UI.desktopKeys(st.trigger) }) : null;
      default: return null;
    }
  }

  function record(id: string, binding: Binding, button: HTMLButtonElement) {
    stopRecording?.();
    clear(feedback);
    button.classList.add("recording");
    button.textContent = SHORTCUTS_UI.recording;
    void Bridge.shortcutsSuspend(true);

    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const result = recordPress(e);
      switch (result.kind) {
        case "pending":
          return;
        case "keys":
          finish();
          store(id, { keys: result.keys, enabled: true });
          return;
        case "clear":
          finish();
          store(id, { keys: "", enabled: binding.enabled });
          return;
        case "typesCharacter":
          finish();
          feedback.append(h("div", {
            class: "notice warn",
            text: SHORTCUTS_UI.typesNote(displayKeys(result.keys), result.typed),
          }));
          return;
        case "needsModifier":
          feedback.replaceChildren(h("div", { class: "notice warn", text: SHORTCUTS_UI.needsModifier }));
          return;
        case "unsupported":
          feedback.replaceChildren(h("div", { class: "notice warn", text: SHORTCUTS_UI.unsupportedKey }));
          return;
        case "cancel":
          finish();
          return;
      }
    };
    const onBlur = () => finish();

    function finish() {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("blur", onBlur);
      stopRecording = null;
      // Takes the global shortcuts back, from what is saved by now.
      void shortcutWrite.then(()=>Bridge.shortcutsSuspend(false));
      draw();
    }
    stopRecording = finish;
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("blur", onBlur);
  }

  function draw() {
    clear(list);
    const dups = duplicates(activeKeys(settings.shortcuts));
    for (const d of SHORTCUTS) {
      if (!d.ported) continue;
      const binding = effective(d, settings.shortcuts);
      const keycap = h("button", {
        class: "keycap",
        text: binding.keys ? displayKeys(binding.keys) : SHORTCUTS_UI.none,
      }) as HTMLButtonElement;
      keycap.disabled = !binding.enabled;
      keycap.addEventListener("click", () => record(d.id, binding, keycap));
      const sw = toggle(binding.enabled, (on) => store(d.id, { keys: binding.keys, enabled: on }));
      const tag = binding.enabled ? tagFor(d.id, dups) : null;
      list.append(h("div", { class: binding.enabled ? "row shortcut" : "row shortcut off" },
        sw,
        h("span", { class: "shortcut-name", text: t(SHORTCUT_TEXT[d.id]) }),
        ...(tag ? [tag] : []),
        keycap,
      ));
    }

    clear(blockedNote);
    const commands = (command: string) => {
      const list = h("div", { class: "diff" });
      for (const d of SHORTCUTS) {
        if (d.ported) list.append(h("div", { class: "ctx", text: `${command} ${d.id}` }));
      }
      return list;
    };
    if (report?.portal) {
      // Wayland, through the desktop's GlobalShortcuts portal; the commands
      // stay as a way around a shortcut the desktop didn't take.
      const active = report.portal === "active";
      blockedNote.append(
        h("div", {
          class: active ? "notice ok" : "notice",
          text: active ? SHORTCUTS_UI.portalActive : SHORTCUTS_UI.portalPending,
        }),
        h("div", { class: "hint", text: SHORTCUTS_UI.portalFallback }),
        commands(report.command),
      );
    } else if (report?.blocked === "wayland") {
      blockedNote.append(h("div", { class: "notice warn", text: SHORTCUTS_UI.wayland }), commands(report.command));
    } else if (report?.blocked) {
      blockedNote.append(h("div", { class: "notice warn", text: SHORTCUTS_UI.noDisplay }));
    }
  }

  const islandList = h("div", { class: "shortcut-list" });
  for (const row of ISLAND_SHORTCUTS) {
    islandList.append(h("div", { class: "row shortcut" },
      h("span", { class: "shortcut-name", text: t(row.description) }),
      h("span", { class: "keycap static", text: row.keys }),
    ));
  }

  const reset = h("button", {
    text: SHORTCUTS_UI.reset,
    onclick: () => {
      stopRecording?.();
      nativeReview?.invalidate();nativeReconciliation?.invalidate();
      clear(feedback);
      if(shortcutWriter)shortcutWrite=shortcutWriter.reset().catch(Bridge.reportSettingsError).finally(draw);
      else{settings.shortcuts = {};shortcutWrite=save().catch(Bridge.reportSettingsError).finally(draw);}
      draw();
    },
  });

  // The events are listened to once (see main); only the section on screen redraws.
  shortcutsListener = {
    report(fresh) {
      report = fresh;
      if (!stopRecording) draw();
    },
    settingsChanged() {
      nativeReview?.invalidate();nativeReconciliation?.invalidate();
      if (!stopRecording) draw();
    },
  };

  draw();
  return h(
    "section",
    {},
    h("h2", {}, h("span", { text: SHORTCUTS_UI.title })),
    nativeReview?.el??h('span',{}),
    nativeReconciliation?.el??h('span',{}),
    h("div", { class: "hint", text: SHORTCUTS_UI.hint }),
    h("div", { class: "subhead", text: SHORTCUTS_UI.global }),
    list,
    blockedNote,
    feedback,
    h("div", { class: "row" }, reset),
    h("div", { class: "subhead", text: SHORTCUTS_UI.island }),
    islandList,
  );
}

/** Settings → General → Weekly recap. The prefs live with the history in Rust. */
function recapRows(context?:HubContext): HTMLElement[] {
  const T={label:t("Weekly recap"),keep:t("Keep a history of my coding sessions"),clear:t("Clear history"),cleared:t("History cleared."),about:t("Counts and project names only — never commands, files or prompts. Kept on this computer for 12 weeks.")};
  const feedback=h('span',{class:'hint',role:'status','aria-live':'polite'});
  const sw=h('button',{type:'button',class:'switch','aria-label':T.label,'aria-pressed':'mixed'}) as HTMLButtonElement;
  const stateHint=h('span',{class:'hint',text:'Por confirmar'});
  const clearBtn=h('button',{type:'button',class:'danger',text:T.clear}) as HTMLButtonElement;
  const retryBtn=h('button',{type:'button',text:'Atualizar estado'}) as HTMLButtonElement;retryBtn.hidden=true;
  let alive=true,intent=0,known=false,prefs:ReturnType<typeof parseRecapPrefs>|null=null,pending:'read'|'enabled'|'clear'|null=null;
  let lastAttempt:{op:'enabled'|'clear';value?:boolean}|null=null,retry:(()=>void)|null=null;
  const previousLeave=context?.onLeave;
  if(context)context.onLeave=()=>{alive=false;intent++;previousLeave?.();};
  const say=(text:string,error=false)=>{feedback.textContent=text;feedback.className=error?'notice err':'hint';feedback.setAttribute('role',error?'alert':'status');};
  function render(){
    const observed=known&&pending===null;sw.classList.toggle('on',prefs?.enabled===true);sw.setAttribute('aria-pressed',observed?String(prefs!.enabled):'mixed');sw.setAttribute('aria-busy',String(pending!==null));sw.disabled=pending!==null||!known;
    stateHint.hidden=observed;sw.title=observed?`Estado observado: registo ${prefs!.enabled?'ativo':'desativado'}.`:'Estado do registo por confirmar.';
    clearBtn.disabled=pending!==null||!known;clearBtn.setAttribute('aria-busy',String(pending==='clear'));clearBtn.textContent=pending==='clear'?'A limpar…':T.clear;
    retryBtn.hidden=!retry;retryBtn.disabled=pending!==null;
  }
  async function read(){
    if(!alive||pending!==null)return;const ticket=++intent;pending='read';say('A carregar as preferências do resumo…');render();
    try{
      const value=parseRecapPrefs(await Bridge.recapPrefs());if(!alive||ticket!==intent)return;prefs=value;known=true;
      if(lastAttempt){say('Estado das preferências atualizado. O pedido anterior continua sem confirmação.',true);retry=lastAttempt.op==='enabled'?()=>void mutate('enabled',lastAttempt!.value):null;retryBtn.textContent='Tentar guardar novamente';}
      else{say('');retry=null;}
    }catch{if(alive&&ticket===intent){known=false;say('Não foi possível confirmar as preferências do resumo. Estado do registo desconhecido.',true);retry=()=>void read();retryBtn.textContent='Atualizar estado';}}
    finally{if(alive&&ticket===intent){pending=null;render();}}
  }
  async function mutate(op:'enabled'|'clear',value?:boolean){
    if(!alive||pending!==null||!known)return;const ticket=++intent;pending=op;retry=null;lastAttempt={op,value};say(op==='clear'?'A limpar e confirmar a gravação…':'A guardar a preferência…');render();
    try{
      const receipt=parseRecapReceipt(await (op==='clear'?Bridge.recapClear():Bridge.recapSetEnabled(value!)));if(!alive||ticket!==intent)return;
      prefs=receipt.prefs;known=true;
      if(recapReceiptSuccess(op,receipt,value)){say(op==='clear'?T.cleared:'Preferência guardada.');lastAttempt=null;}
      else{say(recapFailureMessage(op,receipt,value),true);if(op==='enabled'){retry=()=>void mutate(op,value);retryBtn.textContent='Tentar guardar novamente';}}
    }catch{if(alive&&ticket===intent){known=false;say(op==='clear'?'Não foi possível confirmar a limpeza. Atualiza o estado antes de voltar a tentar.':'Estado do registo por confirmar. Não foi possível confirmar a gravação. Atualiza o estado.',true);retry=()=>void read();retryBtn.textContent='Atualizar estado';}}
    finally{if(alive&&ticket===intent){pending=null;render();}}
  }
  sw.onclick=()=>{if(prefs)void mutate('enabled',!prefs.enabled);};clearBtn.onclick=()=>void mutate('clear');retryBtn.onclick=()=>retry?.();
  render();void read();
  return [h('div',{class:'row'},h('label',{text:T.label}),sw,stateHint,h('span',{class:'hint',text:T.keep})),h('div',{class:'row'},h('label',{}),clearBtn,retryBtn,feedback),h('div',{class:'row'},h('label',{}),h('span',{class:'hint',style:'flex:1 1 0',text:T.about}))];
}

// ── Language ──────────────────────────────────────────────────────────────────

/** The shortcuts section on screen, told about the events listened to once in main. */
let shortcutsListener: { report(fresh: ShortcutsReport): void; settingsChanged(): void } | null = null;

/**
 * Shows the language Settings asks for. A change redraws the window in place,
 * where it was scrolled to: nothing reloads, nothing is written.
 */
function applyLanguage() {
  setLanguage(resolveLanguage(settings.language, systemLanguages()));
}

function applyDirection() {
  document.documentElement.dir = isRtl() ? "rtl" : "ltr";
  document.title = "Coucou";
}

// ── Boot ──────────────────────────────────────────────────────────────────────

function receiveSettings(s:Settings){
  void storageStatus.refresh();
  settingsObservation++;
  const before = `${settings.chatProvider}|${settings.ollamaUrl}|${settings.lmstudioUrl}|${settings.customUrl}`;
  settings = OllamaModelSelection.applySettings({ ...settings, ...s });
  rosterPreferences.observe(s);
  observedIslandAvailability=islandAvailabilityValues(s);
  islandAvailability?.observe(s);
  autostartSetting?.observe(s.autostart);
  shortcutsListener?.settingsChanged();
  for(const redraw of declaredViews)redraw();
  const after = `${settings.chatProvider}|${settings.ollamaUrl}|${settings.lmstudioUrl}|${settings.customUrl}`;
  if(before!==after)localRedraw?.();
  applyLanguage();
}
async function ensureSettingsSubscription():Promise<boolean>{
  if(settingsListenerReady)return true;
  if(settingsSubscription)return settingsSubscription;
  settingsSubscription=onEvent<Settings>("settings-changed",receiveSettings).then(()=>{settingsListenerReady=true;return true;},()=>false).finally(()=>{settingsSubscription=null;});
  return settingsSubscription;
}

async function main() {
  await ensureSettingsSubscription();
  const observation=settingsObservation;
  const boot = await Bridge.boot();
  if (boot) {
    if(observation===settingsObservation){
      settings = { ...settings, ...boot.settings };
      observedIslandAvailability=settingsListenerReady?islandAvailabilityValues(boot.settings):null;
    }
    version = boot.version;
  }
  setLanguage(resolveLanguage(settings.language, systemLanguages()));
  applyDirection();
  hub = await startHub(windowRoot, {
    version,
    flowPages: FLOW_PAGE_RENDERERS,
    renderNotch: async (page, container, context) => {
      notchPage = page;
      root = container;
      notchContext=context;
      await render();
      container.append(storageStatus.el);void storageStatus.refresh();
    },
  });
  onLanguageChange(() => {
    applyDirection();
    hub?.redraw();
  });

  void onEvent<ShortcutsReport>("shortcuts-status", (fresh) => shortcutsListener?.report(fresh));

}

/** The Notch page being shown. */
let notchPage: PageId = "general";

/** Reads what the sections of the Notch page show and draws them. */
async function render() {
  const targetRoot=root,targetPage=notchPage,targetContext=notchContext;
  const status = (await Bridge.hooksStatus()) ?? {
    installed: false, planRelayInstalled: false, settingsPath: "", hookPath: "", hookReady: false,
  };
  const agents = await Bridge.agentHooksList();

  const hasKey = await Bridge.secretPresent("anthropic-api-key");
  const shortcutReport = await Bridge.shortcutsStatus();

  const keys = [
    "stripe-api-key", "github-token", "vercel-token",
    "n8n-url", "n8n-api-key", "resend-api-key", "notion-api-key", "calcom-api-key",
  ];
  const present: Record<string, boolean> = {};
  for (const k of keys) present[k] = (await Bridge.secretPresent(k)) ?? false;

  // A main pill this build can run, and no pill declared twice.
  settings = { ...settings, ...sanitizeDeclared(settings) };
  const connected: Record<string, boolean> = { ...((await Bridge.agentHooksStatus()) ?? {}) };
  for (const def of availablePills()) {
    if (def.connect.kind === "key") {
      connected[def.id] = present[def.connect.key] ?? (await Bridge.secretPresent(def.connect.key)) ?? false;
    }
  }
  /** A chat provider's key was saved or removed: its pill's row says so at once. */
  const keyChanged = (key: string, on: boolean) => {
    for (const def of availablePills()) {
      if (def.connect.kind === "key" && def.connect.key === key) connected[def.id] = on;
    }
    for (const redraw of declaredViews) redraw();
  };
  const chatKeys: Record<string, boolean> = {};
  for (const def of CLOUD) {
    const key = providerDef(def.id).key!;
    chatKeys[key] = (await Bridge.secretPresent(key)) ?? false;
  }
  const customKey = (await Bridge.secretPresent(CUSTOM_SERVER_KEY)) ?? false;
  const lmstudioKey=(await Bridge.secretPresent("lmstudio-local-api-key"))??false;

  // A slow metadata read for a page the user left must never redraw the new
  // page or discard the fields they have already started editing there.
  if(root!==targetRoot||notchPage!==targetPage)return;
  declaredViews.length = 0;
  localRedraw = null;
  shortcutsListener = null;
  clear(root);
  const sections: Partial<Record<PageId, () => HTMLElement[]>> = {
    general: () => [generalSection()],
    pills: () => [activePillsSection(connected)],
    agents: () => [claudeSection(status), agentsSection(agents), planSection(status)],
    chat: () => [apiSection(hasKey), chatProvidersSection(chatKeys, keyChanged), localSection(customKey,lmstudioKey)],
    integrations: () => [integrationsSection(present)],
    phone: () => [phoneSection(settings.phoneLink ?? false, toggle, settings)],
    shortcuts: () => [shortcutsSection(shortcutReport,targetContext)],
  };
  root.append(...(sections[notchPage]?.() ?? []));
  if (notchPage === "general") {
    root.append(h("div", {
      class: "hint",
      text: t("No telemetry. Network requests only go to the services you configure yourself."),
    }));
  }
}

void main();
