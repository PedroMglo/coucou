// Flow → Settings: the key, where the pill is, the microphone, the voice
// model, the Notetaker's summaries, the window and the history. Port of
// FlowHubSettings.swift.

import { h, clear } from "../../views/dom";
import { t } from "../../i18n/i18n";
import { Bridge, onEvent } from "../../core/bridge";
import { FlowBridge, flowKeyId, type Edge, type FlowKey, type ModelsStatus } from "../../core/flow";
import { comboFromKeys } from "../keys";
import type { HubContext } from "../shell";
import { pageSave } from "../save";
import { anchor, button, card, divider, keyKeycap, keycap, keyLabel, label, number, row, segmented, select, toggle } from "../ui";

/** Dictation and pet conversations share this choice; model files stay installed. */
export async function recognitionControl(ctx:HubContext,onChanged:()=>void):Promise<HTMLElement>{
  const boot=await Bridge.boot();
  const active=ctx.status.recognition;
  const note=h('div',{class:'hub-note',text:`Reconhecimento ativo: ${active?.engine==='whisper'?'Whisper multilingue':'Parakeet multilingue'}${active&&!active.ready?' · modelo indisponível':''}. O áudio é processado neste computador, para ditado e conversa com os pets.`});
  if(!boot)return note;
  const feedback=h('div',{class:'hub-note'});
  const picker=select<'auto'|'whisper'|'parakeet'>([['auto','Automático'],['whisper','Whisper multilingue'],['parakeet','Parakeet multilingue']],boot.settings.voiceAsrEngine??'auto',engine=>{
    picker.disabled=true;
    void Bridge.boot().then(async current=>{
      if(!current)throw Error('Não foi possível ler as definições.');
      await Bridge.saveSettings({...current.settings,voiceAsrEngine:engine});
      await ctx.refresh();onChanged();
    }).catch(error=>{feedback.textContent=String(error).replace(/^Error:\s*/,'');picker.disabled=false;});
  });
  return h('div',{},row('Motor de reconhecimento','Automático mantém o Whisper se já estiver configurado; caso contrário usa o Parakeet. Alternar preserva os modelos instalados.',picker),note,feedback);
}

/** Download / progress / installed, for the voice model. Redraws itself. */
export function modelsControl(ctx: HubContext, onReady?: () => void): HTMLElement {
  const el = h("div", { style: "display:flex;align-items:center;gap:8px" });
  let confirming = false;
  const draw = (models: ModelsStatus) => {
    clear(el);
    const m = models.parakeet;
    if (m.state === "missing") {
      el.append(button(t("Download"), () => void FlowBridge.download(), "primary"));
    } else if (m.state === "downloading") {
      const bar = h("div", { class: "hub-progress" }, h("i", { style: `width:${Math.round(m.fraction * 100)}%` }));
      el.append(bar, h("span", { class: "hub-note", text: `${Math.round(m.fraction * 100)} %` }));
    } else if (m.state === "failed") {
      el.append(h("span", { class: "hub-note", text: m.error, title: m.error, style: "max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap" }));
      el.append(button(t("Retry"), () => void FlowBridge.download()));
    } else if (confirming) {
      el.append(
        button(t("Remove the models"), () => {
          confirming = false;
          void FlowBridge.removeModels();
        }, "danger", true),
        button(t("Cancel"), () => {
          confirming = false;
          draw(ctx.status.models);
        }, "", true),
      );
    } else {
      const ok = h("button", { class: "hub-link", style: "color:var(--hub-green)", text: `✓ ${t("Installed")}`, title: t("Remove the models") });
      ok.addEventListener("click", () => {
        confirming = true;
        draw(ctx.status.models);
      });
      el.append(ok);
    }
  };
  draw(ctx.status.models);
  void onEvent<ModelsStatus>("speech-models", (models) => {
    if (!el.isConnected) return;
    const wasReady = ctx.status.models.parakeet.state === "ready";
    ctx.status.models = models;
    draw(models);
    if (!wasReady && models.parakeet.state === "ready") onReady?.();
  });
  return el;
}

/** A new manual request: the backend captures its current last-text revision. */
export function lastTextCopyControl(ctx:HubContext):HTMLElement{
  const feedback=h('div',{class:'hub-note'});
  let pending=false,disposed=false;
  const supported=typeof ctx.status.lastTextAvailable==='boolean';
  const copy=button('Copiar último texto',()=>{
    if(pending||!supported||disposed)return;pending=true;
    copy.disabled=true;feedback.textContent='A copiar…';
    void FlowBridge.copyLast().then(ok=>{
      pending=false;if(disposed||!el.isConnected)return;
      feedback.textContent=ok===true?'Último texto copiado.':'A cópia não foi confirmada. O texto pode ter mudado, não estar disponível ou o pedido anterior ainda estar a terminar.';
      copy.disabled=false;
    });
  },'',true);
  copy.disabled=!supported||ctx.status.lastTextAvailable!==true;
  feedback.textContent=!supported?'Esta versão do backend ainda não suporta a cópia do último texto.':ctx.status.prefs.keepDays===0?'O último texto fica apenas em memória nesta sessão; não se cria histórico.':'Usa a retenção escolhida. Limpar o histórico retira também o último texto disponível.';
  const el=h('div',{},row('Copiar o texto mais recente disponível','Também quando a colagem automática foi impedida para preservar o clipboard.',copy),feedback);
  // The shell builds pages off-DOM before attaching them. Leaving the page,
  // rather than temporary disconnection during construction, owns cleanup.
  let stopListening:(()=>void)|undefined;
  const previous=ctx.onLeave;
  ctx.onLeave=()=>{disposed=true;stopListening?.();stopListening=undefined;previous?.();};
  void onEvent('flow-history',()=>{if(!disposed&&el.isConnected&&supported&&!pending)copy.disabled=false;}).then(unlisten=>{
    if(disposed){unlisten();return;}
    stopListening=unlisten;
  });
  return el;
}

export async function flowSettingsPage(page: HTMLElement, ctx: HubContext) {
  const status = await ctx.refresh();
  const prefs = status.prefs;
  page.append(ctx.hero("flowsettings", t("Settings"), null));

  // ── Keys ──
  const keys: [FlowKey, string][] = [
    ["right_ctrl", t("Right {0}", { 0: "Ctrl" })],
    ["right_alt", t("Right {0}", { 0: "Alt" })],
    ["right_shift", t("Right {0}", { 0: "Shift" })],
    ["left_ctrl", t("Left {0}", { 0: "Ctrl" })],
    ["left_alt", t("Left {0}", { 0: "Alt" })],
    ["left_shift", t("Left {0}", { 0: "Shift" })],
    ["left_ctrl_super", status.platform === "windows" ? "Ctrl + Win" : "Ctrl + Super"],
    ["custom", t("A shortcut of your own…")],
  ];
  // A shortcut of one's own: its keys, and the button that records new ones.
  const recordRow = prefs.key === "custom" && status.keys !== "wayland"
    ? [divider(), row(t("Your shortcut"), t("Hold the keys together to dictate. Its letter no longer reaches the app in front while you hold them."),
      shortcutRecorder(prefs.keyCombo, status.platform, (combo) => void ctx.setPrefs((p) => {
        p.key = "custom";
        p.keyCombo = combo;
      }).then(() => ctx.navigate("flowsettings"))))]
    : [];
  const edges: [Edge, string][] = [["bottom", t("Bottom")], ["bottom-left", t("Bottom-left")], ["bottom-right", t("Bottom-right")], ["left", t("Left")], ["right", t("Right")]];
  page.append(label(t("Keys")));
  const keysCard = card(
    row(t("Hold a key to dictate anywhere"), null, toggle(prefs.enabled, (on) => void ctx.setPrefs((p) => (p.enabled = on)))),
    divider(),
    status.keys === "wayland"
      ? row(t("Hold to dictate"), null, keyKeycap("wayland", status.platform))
      : row(t("Hold to dictate"), prefs.key === "right_alt" ? t("On a keyboard with AltGr, the right Alt types characters: pick another key.") : null,
        select(keys, prefs.key, (key) => void ctx.setPrefs((p) => (p.key = key)).then(() => ctx.navigate("flowsettings")))),
    ...recordRow,
    divider(),
    row(t("Where the pill is"), null, segmented(edges, prefs.edge, (edge) => void ctx.setPrefs((p) => (p.edge = edge)))),
    divider(),
    row(
      t("Hide the pill until I dictate"),
      t("It shows only while you dictate, record a call or get a message. The dictation key still works."),
      toggle(prefs.hidePill, (on) => void ctx.setPrefs((p) => (p.hidePill = on))),
    ),
    divider(),
    row(t("Hands-free"),
      prefs.oneTapHandsFree ? t("Tap once, speak as long as you like, tap again to paste.") : t("Tap twice, speak as long as you like, tap once to paste."),
      keyKeycap(status.keys === "wayland" ? "wayland" : flowKeyId(prefs), status.platform, !prefs.oneTapHandsFree)),
    divider(),
    row(t("Hands-free with one press"), null, toggle(prefs.oneTapHandsFree, (on) => void ctx.setPrefs((p) => (p.oneTapHandsFree = on)).then(() => ctx.navigate("flowsettings")))),
    divider(),
    row(t("Paste the last dictation again"), t("When the words went to the wrong place."), button(t("Paste again"), () => void FlowBridge.pasteAgain(), "", true)),
    divider(),
    lastTextCopyControl(ctx),
    divider(),
    row(t("Cancel a dictation"), null, keycap(t("Esc"))),
  );
  if (status.keys === "wayland") {
    keysCard.append(h("div", {
      class: "hub-warn",
      text: t("On Wayland, the desktop holds the dictation key for Coucou: Ctrl+Alt+V, or the keys you picked when it asked. The first paste asks once to let Coucou type for you."),
    }));
  } else if (status.keys !== "ok") {
    keysCard.append(h("div", { class: "hub-warn", text: t("Coucou can't watch the keyboard here: use the pill's microphone to dictate.") }));
  }
  page.append(anchor("flow.keys", keysCard));

  // ── Microphone ──
  page.append(label(t("Microphone")));
  const mics: [string, string][] = [["", t("System default")], ...status.mics.map((m): [string, string] => [m.name, m.name])];
  const missingMic = prefs.mic !== "" && !mics.some(([id]) => id === prefs.mic);
  if (missingMic) mics.push([prefs.mic, `${prefs.mic} · ${t("Not available")}`]);
  const microphone = select(mics, prefs.mic, (mic) => void ctx.setPrefs((p) => (p.mic = mic)));
  const savedOption = microphone.selectedOptions?.item(0);
  if (missingMic && savedOption) savedOption.disabled = true;
  const micCard = card(row(t("Microphone"), t("The one Coucou listens to for dictation and calls."), microphone));
  // Silence while you speak (flow/hush.rs): Windows mutes each app on its own.
  if (status.platform === "windows") {
    micCard.append(
      divider(),
      row(t("Mute the computer while you speak"),
        t("Videos, music and calls go quiet while Coucou listens to you, then come back. Coucou's own sounds keep playing."),
        toggle(prefs.muteOthers, (on) => void ctx.setPrefs((p) => (p.muteOthers = on)))),
    );
  }
  page.append(anchor("flow.permission", micCard));

  // ── Recognition ──
  page.append(label(t("Recognition")));
  page.append(anchor("flow.recognition", card(
    await recognitionControl(ctx,()=>ctx.navigate('flowsettings')),
    divider(),
    row(t("Parakeet, on your computer"), t("The most accurate, offline, 25 languages. Downloaded once (about {0} MB).", { 0: number(status.models.downloadMb) }), modelsControl(ctx)),
    h("div", { class: "hub-note", text: 'Parakeet reconhece português e inglês localmente. O segundo modelo distingue intervenientes nas gravações; não altera a voz das respostas dos pets.' }),
  )));

  // ── Notetaker ──
  page.append(label(t("Notetaker")));
  page.append(anchor("flow.summary", card(
    row(t("Summaries"), null, select([["claude", t("Claude (your key)")], ["off", t("No summary")]], prefs.summaryEngine, (engine) => void ctx.setPrefs((p) => (p.summaryEngine = engine)))),
    h("div", { class: "hub-note", text: t("With Claude, the transcript (never the audio) goes to Anthropic with your key from Notch → Chat.") }),
    divider(),
    h("div", { class: "hub-note", text: t("Everything stays on this computer. Tell the people in the call that you are recording.") }),
  )));

  // ── App ──
  page.append(label(t("App")));
  page.append(anchor("flow.app", card(
    row(t("Open this window when you open Coucou"), t("Not when Coucou starts with the computer."), toggle(prefs.openAtLaunch, (on) => void ctx.setPrefs((p) => (p.openAtLaunch = on)))),
  )));

  // ── Privacy ──
  page.append(label(t("Privacy")));
  const keep: [string, string][] = [["0", t("Don't keep")], ["7", t("7 days")], ["30", t("30 days")], ["365", t("1 year")], ["-1", t("Forever")]];
  const confirmRow = h("div", { class: "hub-row" });
  const privacy = card(
    row(t("Keep the history"), null, select(keep, String(prefs.keepDays), (v) => void ctx.setPrefs((p) => (p.keepDays = Number(v))))),
    divider(),
    confirmRow,
    h("div", { class: "hub-note", text: t("The history, the statistics and your notes stay on this computer, in Coucou's folder. Dictations are never saved as audio; a call's audio is deleted as soon as its transcript is written.") }),
  );
  const saveErase = pageSave(privacy);
  const erase = () => {
    clear(confirmRow);
    confirmRow.append(
      button(t("Erase the history"), () => void saveErase(() => FlowBridge.clear(false)), "", true),
      button(t("Erase everything"), () => {
        clear(confirmRow);
        confirmRow.append(
          h("span", { class: "hub-note", style: "flex:1", text: t("Erase the history and the statistics?") }),
          button(t("Erase everything"), () => {
            void saveErase(async () => {
              await FlowBridge.clear(true);
              erase();
            });
          }, "danger", true),
          button(t("Cancel"), erase, "", true),
        );
      }, "", true),
    );
  };
  erase();
  page.append(anchor("flow.privacy", privacy));
}

/**
 * The keys of a shortcut of one's own, and "Record": press the keys together
 * (Alt + Shift + F, Left Ctrl…), let go, and they are kept. Esc cancels.
 */
function shortcutRecorder(combo: string[], platform: "windows" | "linux", onDone: (combo: string[]) => void): HTMLElement {
  const cap = keycap(combo.length ? keyLabel(`custom:${combo.join("+")}`, platform) : "—");
  const record = button(t("Record"), () => start(), "", true);
  const el = h("div", { style: "display:flex;align-items:center;gap:8px" }, cap, record);
  let held = new Set<string>();
  let seen: string[] = [];
  const stop = () => {
    window.removeEventListener("keydown", down, true);
    window.removeEventListener("keyup", up, true);
    record.textContent = t("Record");
    record.classList.remove("on");
  };
  const down = (e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.code === "Escape") {
      stop();
      cap.textContent = combo.length ? keyLabel(`custom:${combo.join("+")}`, platform) : "—";
      return;
    }
    held.add(e.code);
    seen = comboFromKeys([...held]);
    cap.textContent = seen.length ? keyLabel(`custom:${seen.join("+")}`, platform) : "…";
  };
  // Done once every key is up: what was held together is the shortcut.
  const up = (e: KeyboardEvent) => {
    e.preventDefault();
    held.delete(e.code);
    if (held.size > 0) return;
    stop();
    if (seen.length) onDone(seen);
  };
  function start() {
    held = new Set();
    seen = [];
    cap.textContent = t("Press the keys…");
    record.textContent = t("Cancel");
    record.classList.add("on");
    window.addEventListener("keydown", down, true);
    window.addEventListener("keyup", up, true);
  }
  return el;
}
