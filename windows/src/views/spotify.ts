// The Spotify pill and card — DOM ports of SpotifyPill / SpotifyCardView
// (SpotifyViews.swift) and the shared now-playing pieces (NowPlayingViews.swift
// and MusicControlButton). Sizes, colours and wording are the Mac's.
//
// What they show comes from src-tauri/src/spotify.rs (MPRIS on Linux, the
// system media controls on Windows) through
// island/spotify.ts. Controls show pending/failed operations; playback values
// follow the observed player state.

import { h, svg, clear, dot } from "./dom";
import { ICONS } from "./icons";
import { Bridge, type SpotifyAction } from "../core/bridge";
import { State, type AgentTask } from "../core/state";
import {
  SPOTIFY_GREEN, SPOTIFY_ID, Spotify, currentArtwork, formatTime, isAd, spotifyPosition, volumeLevel,
  type SpotifyTrack, recentMusic, spotifyItemUri,
} from "../core/spotify";
import { createMiniBot } from "../mochi/minibots";
import { pillDefinition } from "../core/pills";
import { N_, t } from "../i18n/i18n";


function lighter(hex: string, amount: number): string {
  const v = parseInt(hex.replace("#", ""), 16);
  const c = [(v >> 16) & 255, (v >> 8) & 255, v & 255].map((x) => Math.min(255, Math.round(x + amount * 255)));
  return `rgb(${c[0]},${c[1]},${c[2]})`;
}

// ── Controls (shared by the pill and the card) ───────────────────────────────

/** Commands remain pending until the backend replies; only a player read changes playback state. */
const pendingControls = new Map<SpotifyAction, number>();
let controlFeedback = "";
let controlGeneration = 0;
let playPending = false;

async function control(action: SpotifyAction, value?: number): Promise<boolean> {
  const generation = ++controlGeneration;
  pendingControls.set(action, (pendingControls.get(action) ?? 0) + 1);
  controlFeedback = "";
  State.notify();
  try {
    const accepted = await Bridge.spotifyControl(action, value);
    if (accepted !== true && generation === controlGeneration) controlFeedback = accepted === false ? "O Spotify não aceitou o comando. Podes tentar novamente." : "Não foi possível comunicar com o Spotify. Podes tentar novamente.";
    try {
      const observed = Spotify.state;
      const latest = await Bridge.spotifyRefresh();
      if (latest && generation === controlGeneration && Spotify.state === observed) Spotify.state = latest;
      else if (!latest && accepted && generation === controlGeneration) controlFeedback = "Comando enviado; ainda não foi possível confirmar o estado do Spotify.";
    } catch {
      if (accepted && generation === controlGeneration) controlFeedback = "Comando enviado; ainda não foi possível confirmar o estado do Spotify.";
    }
    return accepted === true;
  } catch {
    if (generation === controlGeneration) controlFeedback = "Não foi possível comunicar com o Spotify. Podes tentar novamente.";
    return false;
  } finally {
    const remaining = (pendingControls.get(action) ?? 1) - 1;
    if (remaining > 0) pendingControls.set(action, remaining); else pendingControls.delete(action);
    State.notify();
  }
}

export async function togglePlay() {
  if (playPending) return;
  playPending = true;
  try { await control(Spotify.state.running ? "playPause" : "play"); }
  finally { playPending = false; State.notify(); }
}

function seek(seconds: number) {
  const { running, track } = Spotify.state;
  if (!running || !track || isAd(track)) return;
  const target = Math.min(Math.max(0, seconds), Math.max(0, track.duration - 1));
  void control("seek", target);
}

function setFlag(which: "shuffle" | "repeat", on: boolean) {
  if (!Spotify.state.running || pendingControls.has(which)) return;
  void control(which, on ? 1 : 0);
}

// Coalesce rapid volume changes, with one request in flight and a 120 ms minimum gap.
let pendingVolume: number | null = null;
let inFlightVolume: number | null = null;
let volumeSending = false;
function setVolume(value: number) {
  if (!Spotify.state.running) return;
  const v = Math.min(100, Math.max(0, Math.round(value)));
  if (pendingVolume === v || inFlightVolume === v) return;
  pendingVolume = v;
  if (volumeSending) return;
  volumeSending = true;
  void (async () => {
    try {
      while (pendingVolume != null) {
        const next: number = pendingVolume;
        pendingVolume = null;
        inFlightVolume = next;
        const started = Date.now();
        await control("volume", next);
        const wait = 120 - (Date.now() - started);
        if (wait > 0) await new Promise(resolve => window.setTimeout(resolve, wait));
      }
    } finally { inFlightVolume = null; volumeSending = false; }
  })();
}

async function openSpotify() {
  controlFeedback = ""; State.notify();
  try {
    if (!await Bridge.spotifyOpen()) {
      controlFeedback = "Não foi possível abrir o Spotify.";
      State.notify();
    }
  } catch { controlFeedback = "Não foi possível abrir o Spotify."; State.notify(); }
}

// ── Pill (overview right column) ──────────────────────────────────────────────

export interface SpotifyPillHost {
  el: HTMLElement;
  sync(): void;
}

/** MusicControlButton: a 20 px round button in the pill's colour. */
function pillButton(color: string, icon: string, onClick: () => void): HTMLButtonElement {
  const b = h("button", { class: "np-pill-btn" }, svg(icon, 8));
  b.style.borderColor = `${color}24`;
  b.addEventListener("click", (e) => {
    e.stopPropagation();
    onClick();
  });
  b.addEventListener("mouseenter", () => {
    b.style.background = `${color}2e`;
    b.style.borderColor = `${color}8c`;
    b.style.color = lighter(color, 0.3);
    b.style.boxShadow = `0 0 6px ${color}59`;
  });
  b.addEventListener("mouseleave", () => {
    b.style.background = "";
    b.style.borderColor = `${color}24`;
    b.style.color = "";
    b.style.boxShadow = "";
  });
  return b;
}

/**
 * SpotifyPill: the agent pill's look, with a mini Mochi that dances while
 * Spotify plays, and play/pause + next on hover when a track is loaded.
 */
export function buildSpotifyPill(task: AgentTask, onTap: () => void): SpotifyPillHost {
  const color = task.color;
  const mini = createMiniBot(task, 24, () => State.spotifyPlaying);
  const label = h("span", { class: "lbl", text: task.name });
  const playBtn = pillButton(color, ICONS.play, togglePlay);
  const nextBtn = pillButton(color, ICONS.forward, () => void control("next"));
  const controls = h("div", { class: "np-pill-controls" }, playBtn, nextBtn);
  const pill = h("div", { class: "pill np-pill", onclick: onTap }, mini, label, controls);
  pill.style.borderColor = `${color}24`;
  let hovered = false;
  let playing: boolean | null = null;

  const sync = () => {
    const s = Spotify.state;
    const current = State.tasks.find((x) => x.id === task.id);
    const name = current?.name ?? task.name;
    if (label.textContent !== name) label.textContent = name;
    const show = hovered && s.track != null;
    pill.classList.toggle("controls", show);
    if (playing !== s.playing) {
      playing = s.playing;
      clear(playBtn);
      playBtn.append(svg(s.playing ? ICONS.pause : ICONS.play, 8));
    }
    playBtn.title = s.playing ? t("Pause") : t("Play");
    playBtn.setAttribute("aria-label", playBtn.title);
    playBtn.disabled = playPending;
    nextBtn.title = controlFeedback || t("Next");
    nextBtn.setAttribute("aria-label", t("Next"));
    nextBtn.disabled = pendingControls.has("next");
  };

  pill.addEventListener("mouseenter", () => {
    hovered = true;
    pill.style.background = `${color}2e`;
    pill.style.borderColor = `${color}8c`;
    pill.style.boxShadow = `0 2px 10px ${color}59`;
    label.style.color = lighter(color, 0.3);
    sync();
  });
  pill.addEventListener("mouseleave", () => {
    hovered = false;
    pill.style.background = "";
    pill.style.borderColor = `${color}24`;
    pill.style.boxShadow = "";
    label.style.color = "";
    sync();
  });
  sync();
  return { el: pill, sync };
}

// ── Card (overview left card) ─────────────────────────────────────────────────

/**
 * NowPlayingBar: a grey track whose filled part turns green and grows a knob
 * under the pointer; drag or click anywhere on it.
 */
export class NowPlayingBar {
  readonly el: HTMLElement;
  private fill: HTMLElement;
  private knob: HTMLElement;
  private hovering = false;
  private dragging = false;
  private fraction = 0;
  private available = true;
  private pointerId:number|null=null;
  private gestureContext:string|undefined;
  private onChange: (f: number) => void;
  private onCommit: (f: number) => void;

  constructor(onChange: (f: number) => void, onCommit: (f: number) => void,private readonly options:{label:string;step:()=>number;valueText:(fraction:number)=>string;onCancel?:()=>void;context?:()=>string}) {
    this.onChange = onChange;
    this.onCommit = onCommit;
    this.fill = h("i", { class: "np-fill" });
    this.knob = h("i", { class: "np-knob" });
    this.fill.style.background = "#C5C8CD";
    this.el = h("div", { class: "np-bar",role:"slider",tabindex:0,"aria-label":options.label,"aria-valuemin":0,"aria-valuemax":100 }, h("i", { class: "np-track" }), this.fill, this.knob);
    this.el.addEventListener("mouseenter", () => {
      this.hovering = true;
      this.paint();
    });
    this.el.addEventListener("mouseleave", () => {
      this.hovering = false;
      this.paint();
    });
    this.el.addEventListener("pointerdown", (e) => {
      if (!this.enabled || this.dragging || e.button !== 0) return;
      e.stopPropagation();
      this.dragging = true;this.pointerId=e.pointerId;this.gestureContext=this.options.context?.();this.el.focus({preventScroll:true});
      try {
        this.el.setPointerCapture(e.pointerId);
      } catch {
        /* capture is a nicety */
      }
      this.fraction=this.at(e.clientX);this.onChange(this.fraction);
      this.paint();
    });
    this.el.addEventListener("pointermove", (e) => {
      if(!this.dragging||this.pointerId!==e.pointerId)return;
      if(this.options.context?.()!==this.gestureContext){this.cancel();return;}
      if(this.enabled){this.fraction=this.at(e.clientX);this.onChange(this.fraction);this.paint();}
    });
    const end = (e: PointerEvent) => {
      if (!this.dragging||this.pointerId!==e.pointerId) return;
      if(this.options.context?.()!==this.gestureContext){this.cancel();return;}
      this.dragging = false;this.pointerId=null;
      try { this.el.releasePointerCapture(e.pointerId); } catch { /* capture already released */ }
      this.fraction=this.at(e.clientX);if(this.enabled)this.onCommit(this.fraction);
      this.paint();
    };
    this.el.addEventListener("pointerup", end);
    const cancel=(e:PointerEvent)=>{if(this.pointerId===e.pointerId)this.cancel();};
    this.el.addEventListener("pointercancel",cancel);
    this.el.addEventListener("lostpointercapture",cancel);
    this.el.addEventListener("keydown",e=>{
      if(!this.enabled||e.altKey||e.ctrlKey||e.metaKey)return;
      if(e.key==='Escape'&&this.dragging){e.preventDefault();e.stopPropagation();this.cancel();return;}
      if(this.dragging)return;
      const step=this.options.step();let next:number;
      switch(e.key){case 'Home':next=0;break;case 'End':next=1;break;
       case 'ArrowRight':case 'ArrowUp':next=this.fraction+step;break;
       case 'ArrowLeft':case 'ArrowDown':next=this.fraction-step;break;
       case 'PageUp':next=this.fraction+step*5;break;case 'PageDown':next=this.fraction-step*5;break;
       default:return;}
      e.preventDefault();e.stopPropagation();next=Math.max(0,Math.min(1,next));
      if(next===this.fraction)return;this.fraction=next;this.paint();this.onChange(next);this.onCommit(next);
    });this.paint();
  }

  get enabled(){return this.available;}
  set enabled(value:boolean){this.available=value;this.el.tabIndex=value?0:-1;this.el.setAttribute('aria-disabled',String(!value));if(!value)this.cancel();this.paint();}
  private cancel(){if(!this.dragging)return;const id=this.pointerId;this.dragging=false;this.pointerId=null;if(id!==null)try{this.el.releasePointerCapture(id);}catch{/* capture already released */}this.options.onCancel?.();this.paint();}

  get active(): boolean {
    return this.enabled && (this.hovering || this.dragging);
  }

  private at(clientX: number): number {
    const r = this.el.getBoundingClientRect();
    return Math.min(1, Math.max(0, (clientX - r.left) / Math.max(1, r.width)));
  }

  set(fraction: number) {
    if (this.dragging) return;
    this.fraction = Math.min(1, Math.max(0, fraction));
    this.paint();
  }

  private paint() {
    this.el.setAttribute("aria-valuenow",String(Math.round(this.fraction*100)));
    this.el.setAttribute("aria-valuetext",this.options.valueText(this.fraction));
    const active = this.active;
    this.el.classList.toggle("active", active);
    const pct = `${this.fraction * 100}%`;
    this.fill.style.width = pct;
    this.fill.style.background = active ? SPOTIFY_GREEN : "#C5C8CD";
    this.knob.style.left = `clamp(0px, calc(${pct} - 4.5px), calc(100% - 9px))`;
  }
}

/** NowPlayingIconButton: an icon that brightens under the pointer. */
function iconButton(icon: string, size: number, stroke: number, onClick: () => void): HTMLButtonElement {
  const b = h("button", { class: "np-icon" }, svg(icon, size, stroke ? { stroke } : {}));
  b.addEventListener("click", onClick);
  return b;
}

export interface SpotifyCardHost {
  el: HTMLElement;
  sync(): void;
}

/** SpotifyCardView: now playing, or the idle card (not playing / not installed). */
export function buildSpotifyCard(): SpotifyCardHost {
  const green = SPOTIFY_GREEN;

  // Now playing: artwork row, progress row, controls row.
  const art = h("button", { class: "np-art", type: "button", "aria-label": t("Open Spotify") });
  const artImg = h("img", { alt: "", draggable: "false" }) as HTMLImageElement;
  const artNote = svg(ICONS.musicNote, 16);
  artNote.style.color = `${green}b3`;
  art.append(artNote, artImg);
  art.addEventListener("click", () => void openSpotify());
  const title = h("span", { class: "np-title" });
  const subtitle = h("div", { class: "np-sub" });
  const head = h("div", { class: "np-head" },
    art,
    h("div", { class: "np-text" }, h("div", { class: "np-title-row" }, dot(green, 6), title), subtitle),
  );

  let dragFraction: number | null = null;
  const elapsed = h("span", { class: "np-time elapsed" });
  const remaining = h("span", { class: "np-time remaining" });
  const progress = new NowPlayingBar(
    (f) => {
      dragFraction = f;
      paintProgress();
    },
    (f) => {
      const d = Spotify.state.track?.duration ?? 0;
      dragFraction = null;
      seek(f * d);
    },
    {label:'Posição da reprodução',step:()=>5/Math.max(1,Spotify.state.track?.duration??1),valueText:f=>formatTime(f*(Spotify.state.track?.duration??0)),onCancel:()=>{dragFraction=null;paintProgress();},context:()=>Spotify.state.track?.id??''},
  );
  const progressRow = h("div", { class: "np-progress" }, elapsed, progress.el, remaining);

  const shuffle = iconButton(ICONS.shuffle, 10, 2.2, () => setFlag("shuffle", !Spotify.state.shuffle));
  const prev = iconButton(ICONS.backward, 11, 0, () => void control("previous"));
  const next = iconButton(ICONS.forward, 11, 0, () => void control("next"));
  const repeat = iconButton(ICONS.repeat, 10, 2.2, () => setFlag("repeat", !Spotify.state.repeat));
  prev.style.color = "#C5C8CD";
  next.style.color = "#C5C8CD";
  const play = h("button", { class: "np-play", onclick: togglePlay });
  const volIcon = h("span", { class: "np-vol-icon" });
  const volume = new NowPlayingBar(
    (f) => setVolume(f * 100),
    (f) => setVolume(f * 100),
    {label:'Volume do Spotify',step:()=>.05,valueText:f=>`${Math.round(f*100)}%`},
  );
  const volumeBox = h("div", { class: "np-volume" }, volIcon, volume.el);
  const controls = h("div", { class: "np-controls" },
    h("div", { class: "np-buttons" }, shuffle, prev, play, next, repeat),
    h("div", { class: "grow" }),
    volumeBox,
  );
  const playingEl = h("div", { class: "np-card" }, head, progressRow, controls);

  // Idle: the same layout as the other idle cards.
  const idleDot = dot("#22C55E", 5);
  const idleText = h("span");
  const idleAction = h("button", { class: "link-btn", style: `color:${green}d9` });
  idleAction.addEventListener("click", () => void (Spotify.state.installed ? togglePlay() : openSpotify()));
  const openApp = h("button", {class:"link-btn",text:t("Open Spotify"),onclick:()=>void openSpotify()});
  const idleSub = h("span");
  const idleEl = h("div", { class: "int-card" },
    h("div", { class: "int-head" }, dot(green, 7), h("b", { text: "Spotify" }), idleSub),
    h("div", { class: "int-status" }, idleDot, idleText),
    h("div", { class: "int-actions" }, idleAction, openApp),
  );

  const searchInput=h("input",{type:"search",class:"chat-input",placeholder:t("Recent music or Spotify link"),"aria-label":t("Recent music or Spotify link")}) as HTMLInputElement;
  const matches=h("div",{class:"np-search-results",style:"display:grid;gap:4px;margin:6px 0;max-height:100px;overflow:auto"});
  matches.setAttribute("role","status");matches.setAttribute("aria-live","polite");
  let searching=false;
  let searchGeneration=0;
  const playItem=async (uri:string)=>{
    if(searching)return;searching=true;const generation=searchGeneration;matches.setAttribute("aria-busy","true");
    try { const ok=await Bridge.spotifyPlayItem(uri);if(generation===searchGeneration)matches.textContent=ok?t("Sent to Spotify"):t("Spotify could not play this item."); }
    catch { if(generation===searchGeneration)matches.textContent=t("Spotify could not play this item."); }
    finally {searching=false;matches.setAttribute("aria-busy","false");}
  };
  const searchRecent=()=>{
    searchGeneration++;clear(matches);const text=searchInput.value.trim();if(!text)return;
    const uri=spotifyItemUri(text);
    if(uri){matches.append(h("button",{class:"link-btn",text:t("Play this link"),onclick:()=>void playItem(uri)}));return;}
    const tracks=recentMusic.search(text);
    for(const track of tracks)matches.append(h("button",{class:"link-btn",text:`${track.title} · ${track.artist}`,onclick:()=>void playItem(track.id)}));
    if(!tracks.length)matches.append(h("span",{text:t("No matches in recent music.")}));
    matches.append(h("button",{class:"link-btn",text:t("Search catalog in Spotify ↗"),onclick:()=>void Bridge.openUrl(`https://open.spotify.com/search/${encodeURIComponent(text)}`)}));
  };
  searchInput.addEventListener("input",searchRecent);
  searchInput.addEventListener("keydown",e=>{if(e.key==="Enter"){e.preventDefault();searchRecent();}e.stopPropagation();});
  const quiet = h("input",{type:"checkbox"}) as HTMLInputElement;
  quiet.addEventListener("change",()=>{State.settings={...State.settings,spotifyQuietAds:quiet.checked};void Bridge.saveSettings(State.settings).catch(Bridge.reportSettingsError);State.notify();});
  const extras=h("div",{class:"np-options"},searchInput,matches,h("label",{class:"np-quiet"},quiet,h("span",{text:t("Lower the volume during identified ads")})));
  const feedback = h("p", { class: "np-feedback", role: "status", "aria-live": "polite" });
  feedback.hidden = true;
  const el = h("div", { class: "np-root" });
  let showing: "playing" | "idle" | null = null;
  let shownTrack: SpotifyTrack | null = null;
  let playIcon: boolean | null = null;
  let volIconLevel = -1;
  let timer: number | null = null;

  function paintProgress() {
    const s = Spotify.state;
    const d = s.track?.duration ?? 0;
    const pos = dragFraction != null ? dragFraction * d : spotifyPosition(s, Date.now());
    const f = dragFraction ?? (d > 0 ? Math.min(1, Math.max(0, pos / d)) : 0);
    progress.set(f);
    elapsed.textContent = formatTime(f * d);
    remaining.textContent = d > 0 ? `-${formatTime(Math.max(0, d - f * d))}` : "";
  }

  /** The position runs on its own clock, twice a second, only while it plays on screen. */
  function syncTimer() {
    const run = Spotify.state.playing && showing === "playing" && el.isConnected &&
      State.mode === "expanded" && State.view === "overview";
    if (run && timer == null) {
      timer = window.setInterval(() => {
        if (!(Spotify.state.playing && el.isConnected && State.mode === "expanded" && State.view === "overview")) {
          if (timer != null) window.clearInterval(timer);
          timer = null;
          return;
        }
        paintProgress();
      }, 500);
    } else if (!run && timer != null) {
      window.clearInterval(timer);
      timer = null;
    }
  }

  function syncPlaying(track: SpotifyTrack) {
    const s = Spotify.state;
    if (shownTrack !== track) {
      shownTrack = track;
      title.textContent = isAd(track) ? t("Advertisement") : track.title;
      const sub = [track.artist, track.album].filter((x) => x).join(" · ");
      subtitle.textContent = sub;
      subtitle.style.display = sub ? "" : "none";
    } else if (isAd(track)) {
      title.textContent = t("Advertisement");
    }
    art.title = track.album ? t("{0} — open Spotify", { 0: track.album }) : t("Open Spotify");
    const cover = currentArtwork(s);
    if (cover) {
      if (artImg.getAttribute("src") !== cover) artImg.setAttribute("src", cover);
      art.classList.add("has-art");
    } else {
      art.classList.remove("has-art");
      artImg.removeAttribute("src");
    }

    progress.enabled = s.running && !isAd(track) && track.duration > 0;
    paintProgress();

    shuffle.style.color = s.shuffle ? green : "#6B7079";
    shuffle.title = s.shuffle ? t("Shuffle on") : t("Shuffle off");
    shuffle.setAttribute("aria-label", shuffle.title);shuffle.setAttribute("aria-pressed",String(s.shuffle));shuffle.disabled = !s.running || pendingControls.has("shuffle");
    repeat.style.color = s.repeat ? green : "#6B7079";
    repeat.title = s.repeat ? t("Repeat on") : t("Repeat off");
    repeat.setAttribute("aria-label", repeat.title);repeat.setAttribute("aria-pressed",String(s.repeat));repeat.disabled = !s.running || pendingControls.has("repeat");
    prev.title = t("Previous");prev.setAttribute("aria-label",prev.title);prev.disabled=!s.running||pendingControls.has("previous");
    next.title = t("Next");next.setAttribute("aria-label",next.title);next.disabled=!s.running||pendingControls.has("next");
    if (playIcon !== s.playing) {
      playIcon = s.playing;
      clear(play);
      const icon = svg(s.playing ? ICONS.pause : ICONS.play, 9);
      if (!s.playing) icon.style.transform = "translateX(1px)";
      play.append(icon);
    }
    play.title = s.playing ? t("Pause") : t("Play");play.setAttribute("aria-label",play.title);play.disabled=playPending;

    const level = volumeLevel(s.volume);
    if (level !== volIconLevel) {
      volIconLevel = level;
      clear(volIcon);
      volIcon.append(svg([ICONS.volume0, ICONS.volume1, ICONS.volume2, ICONS.volume3][level], 10, { stroke: 2.2 }));
    }
    volume.enabled = s.running;
    volume.set(s.volume / 100);
    volumeBox.title = t("Volume {0}%", { 0: s.volume });
  }

  function syncIdle() {
    const installed = Spotify.state.installed || Spotify.state.running;
    idleDot.style.background = installed ? "#22C55E" : "#F4505E";
    idleText.textContent = installed ? t("Not playing") : t("Spotify not installed");
    idleAction.textContent = installed ? t("Play") : t("Get Spotify");
    if (playPending) idleAction.setAttribute("disabled",""); else idleAction.removeAttribute("disabled");
    openApp.hidden = !installed;
    idleSub.textContent = t(pillDefinition(SPOTIFY_ID)?.subtitle ?? N_("Integration"));
  }

  return {
    el,
    sync() {
      quiet.checked=State.settings.spotifyQuietAds;
      const pending = pendingControls.size > 0;
      feedback.textContent = controlFeedback || (pending ? "A enviar comando ao Spotify…" : "");
      feedback.hidden = !feedback.textContent;
      playingEl.setAttribute("aria-busy",String(pending));
      const track = Spotify.state.track;
      const want = track ? "playing" : "idle";
      if (want !== showing) {
        showing = want;
        clear(el);
        el.append(want === "playing" ? playingEl : idleEl,feedback,extras);
        shownTrack = null;
      }
      if (track) syncPlaying(track);
      else syncIdle();
      syncTimer();
    },
  };
}
