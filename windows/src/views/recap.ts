// Weekly recap card and its share panel — port of WeeklyRecapCardView and
// RecapSharePanelView (WeeklyRecapView.swift). The Mac shares from a separate
// panel; here the card flips to a share mode inside the island, because a
// window created after launch comes up blank in WebView2 (see lib.rs).
//
// Nothing animates in this view: the image is drawn once, when share mode
// opens or the privacy toggle changes.

import { h, clear } from "./dom";
import { Bridge } from "../core/bridge";
import { washRGBA } from "../core/layout";
import { Recap } from "../recap/recap";
import { renderShareImage, toPngBlob, toPngDataUrl } from "../recap/share";
import {
  dayKey, formatCount, formatDuration, weekRangeLabel, type WeeklySummary,
} from "../recap/summary";
import type { ViewActions, ViewHost } from "./views";
import { onLanguageChange, t, tl, type Msg } from "../i18n/i18n";

/** User-visible strings, in the current language (src/i18n). */
const T = {
  get title() { return t("Weekly recap"); },
  get noActivity() { return t("No activity last week"); },
  get noActivitySub() { return t("Coucou counts your agent sessions as they happen — check back next Monday."); },
  get coding() { return t("coding"); },
  get session() { return t("session"); },
  get sessions() { return t("sessions"); },
  get file() { return t("file"); },
  get files() { return t("files"); },
  get lines() { return t("lines"); },
  get commands() { return t("commands"); },
  get topAgent() { return t("Top agent"); },
  get topProject() { return t("Top project"); },
  get busiestDay() { return t("Busiest day"); },
  get longest() { return t("Longest session"); },
  get permissions() { return t("Permissions"); },
  allowedDenied: (allowed: number, denied: number) => t("{allowed} allowed, {denied} denied", { allowed, denied }),
  get questions() { return t("Questions"); },
  get shareImage() { return t("Share image"); },
  get ok() { return t("OK"); },
  get saving() { return t("Saving…"); },
  savedAs: (name: string) => t("Saved as {name}", { name }),
  get copied() { return t("Copied — paste it anywhere."); },
  get copyUnavailable() { return t("Copying images isn't available here — use Save image."); },
};

function btn(label: string | Msg, kind: "primary" | "secondary", onClick: () => void): HTMLButtonElement {
  return h("button", { class: `btn ${kind}`, onclick: onClick }, h("span", { text: label }));
}

function chip(value: string, label: string): HTMLElement {
  return h("div", { class: "recap-chip" },
    h("div", { class: "v", text: value }),
    h("div", { class: "l", text: label }),
  );
}

/** "Label value · Label value" — values in white, like the who-row. */
function metaLine(pairs: [string, string][]): HTMLElement | null {
  if (pairs.length === 0) return null;
  const el = h("div", { class: "recap-meta" });
  pairs.forEach(([label, value], i) => {
    if (i > 0) el.append(h("span", { class: "sep", text: "·" }));
    el.append(h("span", { text: label }), h("span", { class: "n", text: value }));
  });
  return el;
}

const plural = (n: number, one: string, many: string) => (n === 1 ? one : many);

export function buildRecap(actions: ViewActions): ViewHost {
  const card = h("div", { class: "card wash recap-card" });
  card.style.setProperty("--wash", washRGBA("indigo"));
  card.style.overflowY="auto";card.style.overscrollBehavior="contain";
  const el = h("div", { class: "view" }, card);

  const preferenceNotice=h('div',{class:'recap-status',role:'status','aria-live':'polite',style:'display:block;white-space:normal;overflow-wrap:anywhere'});
  let viewIntent=0,syncShare:(()=>void)|null=null;
  let built = -1;
  let shareCanvas: HTMLCanvasElement | null = null;
  /** Share mode's redraw, while it is open: a language change redraws the image. */
  let redrawShare: (() => void) | null = null;

  // In place: the summary is redrawn in the new language, share mode keeps
  // its state (its labels follow by themselves) and only redraws the image.
  onLanguageChange(() => {
    if (built === -1) return;
    if (redrawShare) redrawShare();
    else showSummary();
  });

  function summaryBody(s: WeeklySummary): HTMLElement {
    const chips = h("div", { class: "recap-chips" },
      chip(formatDuration(s.totalMinutes), T.coding),
      chip(formatCount(s.sessionCount), plural(s.sessionCount, T.session, T.sessions)),
      chip(formatCount(s.filesChanged), plural(s.filesChanged, T.file, T.files)),
    );
    if (s.linesAdded + s.linesRemoved > 0) {
      chips.append(chip(`+${formatCount(s.linesAdded)} / −${formatCount(s.linesRemoved)}`, T.lines));
    }
    if (s.commandsRun > 0) chips.append(chip(formatCount(s.commandsRun), T.commands));

    const first: [string, string][] = [];
    if (s.topAgent) first.push([T.topAgent, s.topAgent]);
    if (s.topProject) first.push([T.topProject, s.topProject]);
    if (s.busiestDay) first.push([T.busiestDay, t(s.busiestDay)]);
    const second: [string, string][] = [];
    if (s.longestSessionMinutes > 1) second.push([T.longest, formatDuration(s.longestSessionMinutes)]);
    if (s.permissionsAllowed + s.permissionsDenied > 0) {
      second.push([T.permissions, T.allowedDenied(s.permissionsAllowed, s.permissionsDenied)]);
    }
    if (s.questions > 0) second.push([T.questions, formatCount(s.questions)]);

    return h("div", { class: "stack recap-stack" },
      h("div", { class: "recap-head" },
        h("span", { class: "recap-title", text: T.title }),
        h("span", { class: "grow" }),
        h("span", { class: "recap-range", text: weekRangeLabel(s) }),
      ),
      chips,
      metaLine(first),
      metaLine(second),
      h("div", { class: "actions" },
        btn(T.shareImage, "primary", () => showShare(s)),
        btn(T.ok, "secondary", () => actions.collapse()),
      ),
    );
  }

  function emptyBody(): HTMLElement {
    return h("div", { class: "stack recap-stack" },
      h("div", { class: "title", text: T.noActivity }),
      h("div", { class: "sub", text: T.noActivitySub }),
      h("div", { class: "actions" }, btn(T.ok, "secondary", () => actions.collapse())),
    );
  }

  function showSummary() {
    viewIntent++;syncShare=null;redrawShare = null;
    clear(card);
    card.append(Recap.summary ? summaryBody(Recap.summary) : emptyBody(),preferenceNotice);updateNotice();
  }

  function updateNotice(){
    const message=syncShare?(Recap.hidePending?'A guardar a preferência…':Recap.notice):Recap.noticeFrom==='shown'?Recap.notice:'';
    preferenceNotice.textContent=message;preferenceNotice.hidden=!message;preferenceNotice.style.color=message&&!Recap.hidePending?'#f2b8a6':'';preferenceNotice.setAttribute('role',message&&!Recap.hidePending?'alert':'status');
  }
  Recap.subscribe(()=>{if(syncShare)syncShare();else updateNotice();});
  function showShare(s:WeeklySummary){
    const current=++viewIntent;let saving=false,copying=false,drawnHide:boolean|null=null;
    const thumb=h('canvas',{class:'recap-thumb'}),status=h('div',{class:'recap-status'});
    const sw=h('button',{type:'button',class:'switch','aria-label':tl("Hide project names")}) as HTMLButtonElement;
    const retry=btn('Atualizar preferência','secondary',()=>{if(Recap.hidePending)return;if(!Recap.prefsKnown)void Recap.readPreferences();else if(Recap.retryHide!==null)void Recap.setHideProjects(Recap.retryHide);});
    const draw=()=>{
      drawnHide=Recap.hidePending||!Recap.prefsKnown||Recap.prefs.hideProjects||Recap.retryHide===true;
      shareCanvas=renderShareImage(s,drawnHide);const dpr=Math.min(2,window.devicePixelRatio||1);thumb.width=Math.round(64*dpr);thumb.height=Math.round(114*dpr);
      const ctx=thumb.getContext('2d');if(ctx){ctx.imageSmoothingQuality='high';ctx.drawImage(shareCanvas,0,0,thumb.width,thumb.height);}
    };
    const say=(text:string,extra?:HTMLElement)=>{clear(status);status.append(h('span',{text}));if(extra)status.append(extra);};
    const blocked=()=>Recap.hidePending||!Recap.prefsKnown||Recap.retryHide===true&&!Recap.prefs.hideProjects;
    sw.onclick=()=>{if(blocked()||saving||copying)return;clear(status);void Recap.setHideProjects(!Recap.prefs.hideProjects);};
    const save=btn(tl("Save image"),'primary',async()=>{
      if(!shareCanvas||blocked()||saving||copying)return;saving=true;sync();say(T.saving);
      try{const path=await Bridge.recapSavePng(toPngDataUrl(shareCanvas),dayKey(s.weekStart));if(current!==viewIntent)return;const name=path.split(/[\\/]/).pop()??path;say(T.savedAs(name),h('button',{class:'link-btn recap-link',text:tl("Show in folder"),onclick:()=>void Bridge.recapRevealSaved()}));}
      catch(error){if(current===viewIntent)say(String(error).replace(/^Error:\s*/,''));}
      finally{saving=false;if(current===viewIntent)sync();}
    });
    const copy=btn(tl("Copy"),'secondary',async()=>{
      if(!shareCanvas||blocked()||saving||copying)return;copying=true;sync();try{const copied=await copyImage(shareCanvas);if(current===viewIntent)say(copied?T.copied:T.copyUnavailable);}finally{copying=false;if(current===viewIntent)sync();}
    });
    function sync(){
      sw.classList.toggle('on',Recap.prefs.hideProjects);sw.setAttribute('aria-pressed',blocked()?'mixed':String(Recap.prefs.hideProjects));sw.setAttribute('aria-busy',String(Recap.hidePending));sw.title=blocked()?'Estado da preferência por confirmar.':'Preferência observada.';sw.disabled=blocked()||saving||copying;
      save.disabled=copy.disabled=blocked()||saving||copying;retry.hidden=Recap.prefsKnown&&Recap.retryHide===null;retry.disabled=Recap.hidePending||saving||copying;retry.firstElementChild!.textContent=Recap.prefsKnown?'Tentar guardar novamente':'Atualizar preferência';
      if(drawnHide!==(Recap.hidePending||!Recap.prefsKnown||Recap.prefs.hideProjects||Recap.retryHide===true))draw();updateNotice();
    }
    clear(card);card.append(h('div',{class:'stack recap-share',style:'height:auto;min-height:100%;box-sizing:border-box;align-items:flex-start;flex-wrap:wrap'},thumb,h('div',{class:'recap-share-side',style:'flex:1 1 150px'},h('div',{class:'recap-title',text:tl("Share your week")}),h('div',{class:'settings-row'},sw,h('span',{text:tl("Hide project names")})),h('div',{class:'actions',style:'flex-wrap:wrap'},save,copy,btn(tl("Back"),'secondary',showSummary),retry),status,preferenceNotice)));
    syncShare=sync;draw();sync();redrawShare=draw;
  }

  return {
    el,
    sync() {
      // Rebuilt only when the recap was (re)loaded, never on every state change:
      // that would reset share mode under the user's cursor.
      if (built === Recap.version) return;
      built = Recap.version;
      shareCanvas = null;
      showSummary();
    },
  };
}

/**
 * Puts the PNG on the clipboard through the web Clipboard API — no new
 * dependency. It needs the document focused, and the island never takes focus
 * on its own, so it is lent focus for the duration. WebKitGTK builds without
 * image clipboard support say no, and the caller offers Save instead.
 */
async function copyImage(canvas: HTMLCanvasElement): Promise<boolean> {
  const Item = (window as { ClipboardItem?: typeof ClipboardItem }).ClipboardItem;
  if (!Item || !navigator.clipboard?.write) return false;
  await Bridge.focusWindow(true);
  try {
    const blob = await toPngBlob(canvas);
    if (!blob) return false;
    await navigator.clipboard.write([new Item({ "image/png": blob })]);
    return true;
  } catch {
    return false;
  } finally {
    void Bridge.focusWindow(false);
  }
}
