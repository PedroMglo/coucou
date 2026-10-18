import { shelterLayout, speechPlacement, speechText } from "../mochi/shelter";
import { PetDialogues, BUSY, ATTENTION, phaseFor, animationFor, type PetActivity } from "../mochi/dialogue";
import { NoticeQueue, type PetNotice } from "../mochi/notices";
import { Bridge, emitToWindow, onEvent } from "../core/bridge";
import { FlowBridge, type PillState } from "../core/flow";
import { Sound } from "../core/sound";
import { SPOTIFY_ID } from "../core/spotify";
import { BotEngine, hexToRGB } from "../mochi/engine";
import { CompanionWorld, PET_EVENTS, PET_SIZE, type CompanionsSnapshot, type PetLife } from "../mochi/companions";
import { resolveLanguage, setLanguage, systemLanguages, t } from "../i18n/i18n";
import { registerVoiceSpeech } from "../pill/voice-speech";

interface PetView { button: HTMLButtonElement; canvas: HTMLCanvasElement; label: HTMLSpanElement; engine: BotEngine; sleepKey?: string }
const world = new CompanionWorld();
const views = new Map<string, PetView>();
const root = document.getElementById("pets")!;
const menu = document.getElementById("menu")!;
const voice = document.getElementById("voice")!;
const notice = document.createElement("aside");
notice.className = "pet-notice"; notice.hidden = true;
notice.setAttribute("role","status"); notice.setAttribute("aria-live","polite");
document.body.append(notice);
const noticeQueue = new NoticeQueue();
let shownNotice: PetNotice | null = null;
let noticeTimer: number | null = null;
function dismissNotice() {
  if(noticeTimer!=null) window.clearTimeout(noticeTimer);
  noticeTimer=null; shownNotice=null; notice.hidden=true;
  world.presented=voicePet??snapshot.presented??null;
  input(performance.now()/1000,true); showNextNotice(); wake();
}
function showNextNotice() {
  if(shownNotice || !snapshot.enabled || snapshot.paused || !nativeVisible || voicePet || snapshot.presented) return;
  const next=noticeQueue.take(); if(!next)return;
  const p=world.pets.get(next.pet);
  if(p && !p.retired) world.touch(next.pet,performance.now()/1000);
  if(!p?.shown){showNextNotice();return;}
  shownNotice=next; world.presented=next.pet; world.touch(next.pet,performance.now()/1000);
  const name=document.createElement("strong");name.textContent=p.identity.name;
  const text=document.createElement("p");text.textContent=t(next.text);
  const review=document.createElement("button");review.textContent=t("Review");
  review.onclick=()=>{ const view=next.view; dismissNotice(); open(p,view); };
  const close=document.createElement("button");close.textContent=t("Dismiss");close.onclick=dismissNotice;
  notice.replaceChildren(name,text,review,close);
  positionNotice();notice.hidden=false;
  noticeTimer=window.setTimeout(dismissNotice,next.urgent?12000:7000);
  input(performance.now()/1000,true);wake();
}
function positionNotice() {
  const p=shownNotice && world.pets.get(shownNotice.pet);if(!p)return;
  const left=Math.max(8,Math.min(world.width-308,p.x-150));
  notice.style.left=`${left}px`;notice.style.top=`${(world.band?.height??p.y+30)+8}px`;
  notice.style.setProperty("--tip",`${Math.max(12,Math.min(280,p.x-left-6))}px`);
}
const shelter = document.createElement("div"); shelter.id="shelter"; shelter.hidden=true;
const home = document.createElement("button"); home.type="button"; home.className="shelter-home";
home.innerHTML='<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 11 12 3l9 8M5 10v10h14V10M9 20v-5a3 3 0 0 1 6 0v5"/></svg>';
const residents=document.createElement("div");residents.className="shelter-residents";
residents.addEventListener("wheel",event=>{if(residents.scrollWidth>residents.clientWidth){event.preventDefault();residents.scrollLeft+=event.deltaY||event.deltaX;}},{passive:false});
shelter.append(home,residents);document.body.append(shelter);
let shelterOpen=false,shelterKey="";
function refreshShelter() {
  const layout=shelterLayout(snapshot.band,snapshot.pets.length,shelterOpen);
  shelter.hidden=!layout || !snapshot.enabled || !nativeVisible;
  if(!layout)return;
  Object.assign(shelter.style,{left:`${layout.left}px`,top:`${layout.top}px`,width:`${layout.width}px`,height:`${layout.height}px`});
  home.setAttribute("aria-expanded",String(shelterOpen));
  home.title=`Abrigo dos pets · ${snapshot.pets.length} disponíveis`;
  home.setAttribute("aria-label",home.title);
  residents.hidden=!shelterOpen;
  const key=JSON.stringify(snapshot.pets.map(p=>[p.id,p.name,p.color,p.outfit,p.attention]));
  if(key===shelterKey)return;shelterKey=key;residents.replaceChildren();
  for(const identity of snapshot.pets) {
    const button=document.createElement("button");button.type="button";button.title=identity.name;button.setAttribute("aria-label",`Chamar ${identity.name}`);
    button.dataset.attention=String(identity.attention);
    const canvas=document.createElement("canvas");canvas.width=canvas.height=64;
    const engine=new BotEngine();engine.bodyColor=hexToRGB(identity.color);engine.setState(identity.attention?"approval":"sleeping",true);engine.setOutfit(identity.outfit,false);engine.update(.016);engine.draw(canvas.getContext("2d")!,64,64);
    button.append(canvas);button.onclick=()=>{ const p=world.pets.get(identity.id);if(p)open(p); };
    residents.append(button);
  }
}
function setShelter(open:boolean) {
  shelterOpen=open;refreshShelter();
  world.sync({...activeSnapshot(snapshot),presented:voicePet??snapshot.presented,motion:world.motion},window.innerWidth,performance.now()/1000);
  refreshDialogue();input(performance.now()/1000,true);wake();
}
home.onclick=()=>setShelter(!shelterOpen);
const nativeSpeech=new Map<string,{text:string;at:number;key:string}>();
const dialogues = new PetDialogues();
const decisions = new Map<string,{pet:string;phase:string}>();
let pendingFold: {pet:string;until:number}|null = null;
function decisionPhase(pet:string) { return [...decisions.values()].find(d=>d.pet===pet)?.phase; }
function foldResolvedCard() {
  const value=pendingFold; if(!value)return;
  if(performance.now()>value.until){pendingFold=null;return;}
  const p=snapshot.pets.find(p=>p.id===value.pet);
  if(decisions.size || p?.attention || ATTENTION.has(p?.state??""))return;
  pendingFold=null;
  if(snapshot.presented===value.pet) void emitToWindow("island","companions-hide-popup");
}
const dialogue = document.createElement("aside");
dialogue.className = "pet-dialogue"; dialogue.hidden = true;
dialogue.setAttribute("role", "status"); dialogue.setAttribute("aria-live", "polite");
const dialogueTitle = document.createElement("strong");
const dialogueText = document.createElement("p");
const dialogueHeard = document.createElement("p"); dialogueHeard.className="dialogue-heard";
const dialogueAction = document.createElement("button");
dialogueAction.textContent = "Rever pedido";
dialogue.append(dialogueTitle, dialogueHeard, dialogueText, dialogueAction); document.body.append(dialogue);
let dialoguePet: string | null = null;
let dialogueTimer: number | null = null;
let liveVoice: VoiceView | null = null;
const phaseLabel = (phase: string) => ({preparing:"A ligar o microfone",thinking:"A pensar",working:"A trabalhar",searching:"A pesquisar",message:"A responder",speaking:"A falar",listening:"A ouvir",approval:"Precisa da tua aprovação",question:"Precisa de uma resposta tua",error:"Precisa de atenção",ratelimit:"Limite atingido",done:"Concluído",stopped:"Interrompido",failed:"Não conseguiu terminar"}[phase] ?? "Disponível");
function activeSnapshot(s: CompanionsSnapshot): CompanionsSnapshot {
  const room=shelterLayout(s.band,s.pets.length,shelterOpen);
  return {...s,sheltered:!!room,band:room && s.band ? {...s.band,lanes:room.lanes}:s.band,pets:s.pets.map(identity=>{
    const phase=decisionPhase(identity.id) ?? phaseFor(dialogues.current(identity.id,performance.now()),identity.state);
    const busy=BUSY.has(phase),attention=ATTENTION.has(phase);
    const entry=dialogues.entries.get(identity.id);
    const reading=!!entry && ["done","stopped","failed"].includes(entry.phase) && performance.now()-entry.changedAt<9000;
    return {...identity,state:animationFor(phase,identity.state),busy:identity.busy||busy,activity:`${identity.activity}:${entry?.turn}:${entry?.changedAt}`,active:identity.active||busy||attention||reading,attention:identity.attention||attention};
  })};
}
function activityPhase(p: PetLife) {
  const value = dialogues.current(p.identity.id,performance.now());
  const native = snapshot.pets.find(identity=>identity.id===p.identity.id)?.state ?? p.identity.state;
  const attention=decisionPhase(p.identity.id) ?? (ATTENTION.has(native) ? native : null);
  if(attention)return attention;
  if(voicePet===p.identity.id && liveVoice) return value && BUSY.has(value.phase) ? value.phase : liveVoice.phase;
  return phaseFor(value,native);
}
function positionDialogue() {
  if(dialogue.hidden || !dialoguePet)return;
  const p=world.pets.get(dialoguePet);if(!p?.shown){dialogue.hidden=true;return;}
  const placement=speechPlacement(p.x,p.y,world.scale,world.band,world.width,[...world.pets.values()].filter(other=>other!==p && other.shown).map(other=>other.x));
  if(!placement){dialogue.style.visibility="hidden";return;}
  dialogue.style.visibility="visible";
  Object.assign(dialogue.style,{left:`${placement.left}px`,top:`${placement.top}px`,width:`${placement.width}px`});
  dialogue.dataset.side=placement.side;dialogue.dataset.compact=String(!!world.band);
  dialogue.style.setProperty("--speech-height",`${"height" in placement?placement.height:38}px`);
  dialogue.style.setProperty("--tip",`${placement.tip}px`);
}
function refreshDialogue() {
  const now = performance.now();
  const selected=dialogues.selected && world.pets.get(dialogues.selected);
  const recent=[...nativeSpeech.entries()].filter(([,s])=>now-s.at<=9000).sort((a,b)=>b[1].at-a[1].at)[0]?.[0];
  const owner = voicePet ?? (selected && dialogues.visible(selected.identity.id,now,false) ? selected.identity.id : null) ?? recent ?? [...world.pets.values()].find(p=>p.identity.attention)?.identity.id;
  const p = owner ? world.pets.get(owner) : undefined;
  dialogue.hidden = true; dialoguePet = null;
  if (!p?.shown || !snapshot.enabled || !nativeVisible || snapshot.paused || snapshot.presented || shelterOpen) return;
  const value = dialogues.visible(p.identity.id,now,false);
  const phase = activityPhase(p);
  const voiced = voicePet === p.identity.id ? liveVoice : null;
  const native=nativeSpeech.get(p.identity.id);
  const message = voiced?.phase === "speaking" ? voiced.answer : value?.message ?? native?.text ?? "";
  const text=speechText(phase,message,value?.messageAt??native?.at??now,now);
  if(!text)return;
  dialoguePet = p.identity.id; dialogue.hidden = false;
  dialogue.dataset.phase = phase; dialogue.style.setProperty("--pet-color",p.identity.color);
  dialogueTitle.textContent = p.identity.name;
  if (dialogueText.textContent !== text) { dialogueText.textContent = text; dialogueText.classList.remove("arriving");void dialogueText.offsetWidth;dialogueText.classList.add("arriving"); }
  dialogue.title=text;dialogueText.hidden=false;dialogueHeard.hidden=true;
  dialogueAction.hidden = !world.band && !ATTENTION.has(phase);
  dialogueAction.textContent=ATTENTION.has(phase)?"Rever pedido":`Ler mensagem de ${p.identity.name}`;
  dialogueAction.setAttribute("aria-label",dialogueAction.textContent);
  dialogue.setAttribute("aria-busy",String(BUSY.has(phase)));
  positionDialogue();
  if (dialogueTimer != null) window.clearTimeout(dialogueTimer);
  const remaining=phase==="speaking" || ATTENTION.has(phase) ? 9500 : Math.max(50,9001-(now-(value?.messageAt??native?.at??now)));
  dialogueTimer = window.setTimeout(()=>{dialogueTimer=null;world.sync({...activeSnapshot(snapshot),presented:voicePet??snapshot.presented,motion:world.motion},window.innerWidth,performance.now()/1000);refreshDialogue();input(performance.now()/1000,true);wake();},remaining);
}
dialogueAction.onclick = () => {
  const p=dialoguePet && world.pets.get(dialoguePet);if(!p)return;
  const final=nativeSpeech.get(p.identity.id);
  const native=final && performance.now()-final.at<=9000 && !dialogues.visible(p.identity.id,performance.now(),false);
  open(p,ATTENTION.has(activityPhase(p))?undefined:native?"finished":"prompt");
};
let voicePet: string | null = null;
let voiceActive = false;
interface VoiceView { phase: string; heard: string; answer: string; pet?: string; announcementId?: number; note?: boolean }
let bubble: {view: VoiceView; dictation: boolean} | null = null;
function voiceBubble(view: VoiceView | null, dictation = false) {
  bubble = view ? {view,dictation} : null;
  if (!dictation) {
    if(shelterOpen){shelterOpen=false;refreshShelter();}
    const starting=!liveVoice && view?.phase==="preparing";
    liveVoice = view;
    voice.hidden = true;
    voicePet = view ? view.pet ?? "agent_codex" : null;
    const owner=snapshot.pets.find(p=>p.id===voicePet);
    if(starting && snapshot.presented===voicePet && !decisions.size && !owner?.attention && !ATTENTION.has(owner?.state??"")) void emitToWindow("island","companions-hide-popup");
    world.presented = voicePet ?? snapshot.presented ?? null;
    if (voicePet) { world.touch(voicePet,performance.now()/1000); world.sync({...activeSnapshot(snapshot),presented:voicePet,motion:world.motion},window.innerWidth,performance.now()/1000); }
    refreshDialogue(); input(performance.now()/1000,true); wake(); return;
  }
  voice.hidden = !view;
  voice.replaceChildren();
  if (!view) { voicePet = null; showNextNotice(); world.presented = snapshot.presented ?? null; input(performance.now()/1000, true); return; }
  const owner = dictation ? "ai_ollama" : view.pet;
  const p = owner ? world.pets.get(owner) : world.pets.get("agent_codex") ?? world.pets.get("ai_ollama");
  if(p && !p.retired) world.touch(p.identity.id,performance.now()/1000);
  if (!p?.shown) { voice.hidden = true; return; }
  voicePet = p.identity.id;
  if(shownNotice) dismissNotice();
  world.presented = voicePet;
  world.touch(voicePet, performance.now()/1000);
  world.sync({...activeSnapshot(snapshot),presented:voicePet,motion:world.motion},window.innerWidth,performance.now()/1000);
  const title = document.createElement("strong");
  const phases: Record<string,string> = {listening:t("Listening"),thinking:t("Thinking…"),speaking:t("Answers"),copied:t("Dictation")};
  title.textContent = `${p.identity.name} · ${dictation ? t("Dictation") : phases[view.phase] ?? view.phase}`;
  const heard = document.createElement("p"); heard.className = "heard"; heard.textContent = view.heard;
  const answer = document.createElement("p"); answer.textContent = view.answer;
  voice.append(title, heard, answer);
  const actions = view.announcementId != null ? [[t("Close"),"dismiss"]] :
    view.note ? [[t("Finish"),"stopNote"]] :
    [[t("Cancel"), "cancel"], ...(["listening", "locked"].includes(view.phase) ? [[t("Finish"), "finish"]] : [])];
  for (const [text, action] of actions) {
    const button = document.createElement("button"); button.textContent = text;
    button.onclick = () => void FlowBridge.pillAction(action, view.announcementId);
    voice.append(button);
  }
  const left = Math.max(8, Math.min(world.width - 348, p.x - 170));
  voice.style.left = `${left}px`;
  voice.style.top = `${(world.band?.height ?? p.y + 30) + 10}px`;
  voice.style.setProperty("--tip", `${Math.max(12, Math.min(320,p.x-left-6))}px`);
  input(performance.now()/1000, true); wake();
}
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
let snapshot: CompanionsSnapshot = { enabled: false, paused: false, motion: true, pets: [] };
let timer: number | null = null;
let last = 0;
let lastShape = "";
let shapeAt = -Infinity;
let dragging: { id: string; start: number; x: number; moved: boolean } | null = null;
let suppressClick = false;
let nativeVisible = true;
let menuTimer: number | null = null;
let menuPet: string | null = null;
// Ambient behaviour stays silent. The island already owns notification sounds;
// several pets must not create several audio contexts or repeat an alert.
Sound.setEnabled(false);

// Events can precede the snapshot that introduces their owner. Keep that
// exact identity briefly; never substitute a different integration's pet.
let pendingPresentation: { id: string; view: string; until: number } | null = null;
function presentPending() {
  const e = pendingPresentation;
  if (!e) return;
  if (performance.now() > e.until) { pendingPresentation = null; return; }
  const p = world.pets.get(e.id);
  if(p && !p.retired) world.touch(e.id,performance.now()/1000);
  if (p?.shown && nativeVisible) open(p, e.view);
}

function open(p: PetLife, view?: string) {
  pendingPresentation = null;
  if(shelterOpen){shelterOpen=false;refreshShelter();}
  if(shownNotice) { if(noticeTimer!=null)window.clearTimeout(noticeTimer);noticeTimer=null;shownNotice=null;notice.hidden=true; }
  closeMenu();
  world.touch(p.identity.id, performance.now() / 1000);
  world.sync({...activeSnapshot(snapshot),presented:p.identity.id,motion:world.motion},window.innerWidth,performance.now()/1000);
  void Bridge.companionsOpen(p.identity.id, p.x, p.y, view ?? (p.identity.id === "ai_ollama" ? "prompt" : undefined));
}

function closeMenu() {
  const wasOpen = !menu.hidden;
  menuPet = null;
  if (menuTimer != null) window.clearTimeout(menuTimer);
  menuTimer = null;
  menu.hidden = true;
  if (wasOpen) world.hovered = null;
}
function showMenu(p: PetLife) {
  closeMenu();
  menu.replaceChildren();
  const add = (label: string, action: () => void) => {
    const b = document.createElement("button");
    b.textContent = label;
    b.addEventListener("click", () => { closeMenu(); action(); wake(); });
    menu.append(b);
  };
  add(t("Open"), () => open(p));
  if (p.identity.id === "ai_ollama") {
    add(t("Chat"), () => open(p, "prompt"));
    add(t("Talk to Mochi"), () => void FlowBridge.pillAction("voice"));
    add(t("Dictation"), () => void FlowBridge.pillAction("dictate"));
  }
  if (p.identity.id === SPOTIFY_ID) add(t("Play / pause"), () => void Bridge.spotifyControl("playPause"));
  add(t("Pet"), () => world.touch(p.identity.id, performance.now() / 1000));
  add(snapshot.paused ? t("Resume companions") : t("Pause companions"), () => {
    void emitToWindow("island", PET_EVENTS.action, { action: "pause" });
  });
  add(t("Settings"), () => void Bridge.openSettingsWindow());
  menu.style.left = `${Math.max(0, Math.min(world.width - 170, p.x - 76))}px`;
  menu.style.top = `${p.y + 40}px`;
  menu.hidden = false;
  menuPet = p.identity.id;
  menuTimer = window.setTimeout(closeMenu, 8000);
  world.hovered = p.identity.id;
  wake();
}

function create(p: PetLife): PetView {
  const button = document.createElement("button");
  button.className = "pet";
  button.hidden = true;
  const canvas = document.createElement("canvas");
  const label = document.createElement("span");
  button.append(canvas, label);
  root.append(button);
  const engine = new BotEngine();
  engine.particleOverhang = 0;
  button.addEventListener("click", () => {
    if (suppressClick) { suppressClick = false; return; }
    if (snapshot.presented === p.identity.id) {
      pendingPresentation = null;
      void emitToWindow("island", "companions-hide-popup");
    } else open(p);
  });
  button.addEventListener("pointerenter", () => { world.hovered = p.identity.id; wake(); });
  button.addEventListener("pointerleave", () => { if (menu.hidden && !dragging) { world.hovered = null; wake(); } });
  button.addEventListener("contextmenu", e => { e.preventDefault(); showMenu(p); });
  button.addEventListener("pointerdown", e => {
    if (e.button !== 0) return;
    dragging = { id: p.identity.id, start: e.clientX, x: p.x, moved: false };
    button.setPointerCapture(e.pointerId);
  });
  button.addEventListener("pointermove", e => {
    if (!dragging || dragging.id !== p.identity.id) return;
    const delta = e.clientX - dragging.start;
    if (Math.abs(delta) > 4) dragging.moved = true;
    if (!dragging.moved) return;
    let [left, right] = world.bounds(p);
    // Keep both bodies clickable when a user drags one into another.
    for (const other of world.pets.values()) {
      if (other === p || !other.shown || other.y !== p.y) continue;
      if (p.x < other.x) right = Math.min(right, other.x - world.spacing);
      else left = Math.max(left, other.x + world.spacing);
    }
    p.x = Math.max(left, Math.min(right, dragging.x + delta));
    wake();
  });
  const endDrag = () => {
    suppressClick = dragging?.moved ?? false;
    dragging = null;
    if (menu.hidden) world.hovered = null;
  };
  button.addEventListener("pointerup", endDrag);
  button.addEventListener("pointercancel", endDrag);
  return { button, canvas, label, engine };
}

function input(now: number, force = false) {
  if (!force && now - shapeAt < 0.1) return;
  const rects: [number, number, number, number][] = [];
  if(!shelter.hidden && snapshot.enabled){const r=shelter.getBoundingClientRect();rects.push([r.x,r.y,r.width,r.height]);}
  if (snapshot.enabled) for (const p of world.pets.values()) {
    if (!p.shown || p.visibility === "leaving" || p.opacity <= 0.05) continue;
    const scale = world.scale;
    // Disc rows around the body, plus its own label. Empty habitat never takes
    // input. A small margin accommodates the movement between region updates.
    for (let y = -22; y < 22; y += 4) {
      const dx = Math.sqrt(23 * 23 - y * y) + 3;
      rects.push([p.x - dx * scale, p.y + (y - 2) * scale, dx * 2 * scale, 8 * scale]);
    }
    if (!world.band) rects.push([p.x - 30, p.y + 19, 60, 23]);
  }
  if (!menu.hidden && snapshot.enabled) {
    const r = menu.getBoundingClientRect();
    rects.push([r.x, r.y, r.width, r.height]);
  }
  if (!notice.hidden && snapshot.enabled) { const r=notice.getBoundingClientRect();rects.push([r.x,r.y,r.width,r.height]); }
  if (!dialogue.hidden && dialogue.style.visibility!=="hidden" && snapshot.enabled) { const r=dialogue.getBoundingClientRect();rects.push([r.x,r.y,r.width,r.height]); }
  if (!voice.hidden && snapshot.enabled) { const r = voice.getBoundingClientRect(); rects.push([r.x,r.y,r.width,r.height]); }
  const key = JSON.stringify(rects.map(r => r.map(Math.round)));
  shapeAt = now;
  if (key === lastShape) return;
  lastShape = key;
  void Bridge.companionsInput(JSON.parse(key));
}

function draw(p: PetLife, v: PetView, dt: number, now: number) {
  const { engine, canvas, button, label } = v;
  button.hidden = !p.shown;
  if (!p.shown) { v.sleepKey = undefined; return; }
  button.disabled = p.visibility === "leaving";
  button.style.opacity = String(p.opacity);
  const scale = world.scale * (0.35 + 0.65 * p.opacity);
  button.style.transform = `translate(${p.x - 32 * scale}px, ${p.y - 32 * scale + (1 - p.opacity) * 8 * world.scale}px) scale(${scale})`;
  button.style.transformOrigin = "top left";
  const phase = activityPhase(p);
  const busy = !snapshot.paused && BUSY.has(phase);
  const sleeping = p.action === "sleep" && !busy && !ATTENTION.has(phase);
  const sleepKey = sleeping ? JSON.stringify([p.x,p.y,world.scale,p.identity.color,p.identity.outfit]) : undefined;
  if (sleeping && v.sleepKey === sleepKey) return;
  v.sleepKey = sleepKey;
  engine.setState(sleeping ? "sleeping" : animationFor(phase,p.identity.state), sleeping);
  engine.bodyColor = hexToRGB(p.identity.color);
  engine.setOutfit(p.identity.outfit, false);
  engine.setDancing(p.action === "dance" || (world.motion && p.identity.dancing && p.action === "walk"));
  engine.lookX = ["walk", "wave"].includes(p.action) ? p.direction * 0.65 : p.action === "look" ? Math.sin(now * 0.4 + p.seed) * 0.8 : 0;
  if (p.emote) {
    if (world.motion && !snapshot.paused) {
      if (p.action === "wave") engine.greet();
      engine.triggerEmote(p.emote, 1.5);
    }
    p.emote = null;
  }
  if (world.motion || sleeping) engine.update(dt);
  const dpr = Math.min(2, window.devicePixelRatio || 1);
  const size = Math.round(PET_SIZE * dpr);
  if (canvas.width !== size || canvas.height !== size) canvas.width = canvas.height = size;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, PET_SIZE, PET_SIZE);
  ctx.save();
  if (p.action === "walk") {
    const stride = Math.sin(now * 11) * 2;
    // Two little feet and a gait complement the engine's face and breathing.
    ctx.fillStyle = p.identity.color;
    for (const side of [-1, 1]) {
      ctx.beginPath();
      ctx.ellipse(32 + side * 10 + side * stride, 50, 5, 2.5, 0, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.translate(0, -Math.abs(Math.sin(now * 11)) * 2);
    ctx.translate(32, 32); ctx.rotate(stride * 0.012); ctx.translate(-32, -32);
  }
  if (p.action === "wave") {
    // Partners face each other, lift a hand and share a small synchronized hop.
    const phase = (1.8 - Math.max(0, p.until - now)) / 1.8;
    ctx.translate(0, -Math.sin(Math.PI * phase) * 2);
    ctx.strokeStyle = p.identity.color;
    ctx.lineWidth = 4; ctx.lineCap = "round";
    const side = p.direction;
    ctx.beginPath(); ctx.moveTo(32 + side * 19, 34);
    ctx.quadraticCurveTo(32 + side * 28, 26, 32 + side * (26 + Math.sin(now * 12) * 2), 19);
    ctx.stroke();
  }
  engine.applyDance(ctx, PET_SIZE, PET_SIZE);
  engine.draw(ctx, PET_SIZE, PET_SIZE);
  if(phase==="speaking" || phase==="message") {
    ctx.fillStyle="#182032";ctx.beginPath();ctx.ellipse(32,40,3.5,world.motion?1.5+Math.abs(Math.sin(now*12))*2:2,0,0,Math.PI*2);ctx.fill();
  } else if(phase==="thinking") {
    ctx.strokeStyle="#182032";ctx.lineWidth=1.6;ctx.lineCap="round";
    ctx.beginPath();ctx.moveTo(24,22);ctx.lineTo(28,20.5);ctx.stroke();
    ctx.fillStyle=p.identity.color;ctx.beginPath();ctx.ellipse(41,42,3.5,2.5,-.5,0,Math.PI*2);ctx.fill();
  } else if(phase==="listening") {
    ctx.strokeStyle="#93c5fd";ctx.lineWidth=1.4;ctx.lineCap="round";
    for(const side of [-1,1]) {const pulse=world.motion?Math.sin(now*3)*1.2:0;ctx.beginPath();ctx.ellipse(32+side*19,32,2+pulse*.2,6+pulse,0,-Math.PI/2,Math.PI/2);ctx.stroke();}
  }
  if (busy && world.motion && !snapshot.paused) {
    ctx.strokeStyle = p.identity.color; ctx.lineWidth = 2; ctx.lineCap = "round";
    for(let i=0;i<3;i++) { const angle=now*2.6+i*Math.PI*2/3;ctx.beginPath();ctx.arc(32,32,23,angle,angle+0.42);ctx.stroke(); }
  }
  if (p.identity.attention || ATTENTION.has(phase)) {
    ctx.fillStyle = "#fbbf24";
    ctx.beginPath(); ctx.arc(52, 11, 5, 0, Math.PI*2); ctx.fill();
  }
  ctx.restore();
  button.style.clipPath = world.band ? "circle(25px at 32px 32px)" : "none";
  label.hidden = !!world.band;
  button.dataset.attention = String(p.identity.attention);
  const description = `${p.identity.name} · ${phaseLabel(phase)}`;
  if (button.title !== description) { button.title = description; button.setAttribute("aria-label", description); }
  if (label.textContent !== p.identity.name) label.textContent = p.identity.name;
}

function wake() {
  if (timer != null || !snapshot.enabled || !nativeVisible || document.hidden) return;
  timer = window.setTimeout(frame, 0);
}
function frame() {
  timer = null;
  if (!snapshot.enabled || !nativeVisible || document.hidden) return;
  const now = performance.now() / 1000;
  const dt = Math.min(0.1, Math.max(0, now - last));
  last = now;
  try {
    world.tick(dt, now);
    for (const [id, v] of views) if (!world.pets.has(id)) { v.button.remove(); views.delete(id); }
    for (const p of world.pets.values()) draw(p, views.get(p.identity.id)!, dt, now);
    positionDialogue();
    input(now);
  } catch (error) {
    // A drawing failure must not leave invisible input regions over the apps.
    console.error("[coucou] companions frame", error);
    void Bridge.companionsInput([]);
    lastShape = "";
    return;
  }
  if (world.needsFrames || (!snapshot.paused && [...world.pets.values()].some(p=>p.shown && BUSY.has(activityPhase(p))))) {
    const moving = [...world.pets.values()].some(p => p.shown && (p.visibility !== "visible" || BUSY.has(activityPhase(p)) || ["walk","wave","dance","celebrate"].includes(p.action)));
    timer = window.setTimeout(frame, !world.motion ? 1000 : moving ? 1000 / 24 : 250);
  }
}

async function start() {
  await registerVoiceSpeech("companions");
  await onEvent<{id:string;provider:string;kind:string}>("agent-request", request=>{
    const pet=({codex:"agent_codex",antigravity:"agent_antigravity",copilot:"agent_copilot"} as Record<string,string>)[request.provider.toLowerCase()];
    if(pet)decisions.set(request.id,{pet,phase:request.kind==="question"?"question":"approval"});
    refreshDialogue();wake();
  });
  await onEvent<{id:string}>("agent-request-ended", request=>{
    const decision=decisions.get(request.id);decisions.delete(request.id);
    if(decision)pendingFold={pet:decision.pet,until:performance.now()+3000};
    refreshDialogue();wake();
  });
  await onEvent<PetActivity>("pet-activity", value => {
    const before=dialogues.entries.get(value.pet);
    if (!dialogues.update(value,performance.now())) return;
    if(!before || before.turn!==value.turn) pendingFold={pet:value.pet,until:performance.now()+3000};
    foldResolvedCard();
    const p=world.pets.get(value.pet); if(p) world.touch(value.pet,performance.now()/1000);
    world.sync({...activeSnapshot(snapshot),presented:voicePet??snapshot.presented,motion:world.motion},window.innerWidth,performance.now()/1000);
    refreshDialogue(); input(performance.now()/1000,true); wake();
  }, "companions");
  await onEvent<PetNotice>("companions-notice", n=>{noticeQueue.push(n);showNextNotice();});
  await onEvent<VoiceView | null>("companions-voice", view => { voiceActive = !!view; voiceBubble(view); });
  await onEvent<PillState>("companions-flow", state => {
    if (voiceActive) return;
    const active = ["listening", "locked", "thinking", "copied", "failed", "pastedAgain"].includes(state.phase);
    const feedback = state.phase === "copied" ? t("Copied · press Ctrl+V to paste") :
      state.phase === "failed" ? t("Microphone unavailable") :
      state.phase === "pastedAgain" ? t("Last dictation pasted") : null;
    if (active || state.announcement || state.noteStartedAt) {
      voiceBubble({phase: state.phase, heard: "", announcementId: state.announcement?.id, note: !!state.noteStartedAt, answer: state.announcement?.text ?? feedback ??
        (state.phase === "thinking" ? t("Thinking…") : state.noteStartedAt ? t("Notetaker") : t("Listening"))}, true);
    } else voiceBubble(null, true);
  });
  await onEvent<{ id: string; view: string }>("companions-present", e => {
    if (!e.id) return;
    pendingPresentation = { ...e, until: performance.now() + 10000 };
    presentPending();
  });
  await onEvent<boolean>("companions-visible", visible => {
    nativeVisible = visible;
    if (!visible && timer != null) { window.clearTimeout(timer); timer = null; }
    if (visible) { void emitToWindow("island",PET_EVENTS.ready);presentPending();showNextNotice();wake(); }
    refreshShelter();
  });
  await onEvent<CompanionsSnapshot>(PET_EVENTS.state, s => {
    snapshot = s;
    if(s.presented)shelterOpen=false;
    for(const pet of s.pets) if(pet.message) {
      const key=JSON.stringify([pet.message,pet.activity]);
      if(nativeSpeech.get(pet.id)?.key!==key)nativeSpeech.set(pet.id,{text:pet.message,at:performance.now(),key});
    }
    refreshShelter();
    setLanguage(resolveLanguage(s.language ?? "", systemLanguages()));
    if (!s.enabled || (menuPet && !s.pets.some(p=>p.id===menuPet))) closeMenu();
    const oldPet = s.presented ? world.pets.get(s.presented) : null;
    const oldPosition = oldPet ? [oldPet.x, oldPet.y] : null;
    world.sync({ ...activeSnapshot(s), presented: voicePet ?? s.presented ?? shownNotice?.pet, motion: s.motion && !reducedMotion.matches }, window.innerWidth, performance.now() / 1000);
    for (const [id, v] of views) if (!world.pets.has(id)) { v.button.remove(); views.delete(id); }
    for (const p of world.pets.values()) if (!views.has(p.identity.id)) views.set(p.identity.id, create(p));
    if (shownNotice && (s.paused || !s.enabled || s.presented)) dismissNotice();
    presentPending();
    positionNotice();
    foldResolvedCard();
    refreshDialogue();
    showNextNotice();
    if (bubble) voiceBubble(bubble.view,bubble.dictation);
    else if (oldPet?.shown && oldPosition && (oldPet.x !== oldPosition[0] || oldPet.y !== oldPosition[1])) {
      void Bridge.companionsAnchor(oldPet.x,oldPet.y);
    }
    if (timer != null) window.clearTimeout(timer);
    timer = null;
    if (!s.enabled) { root.hidden = true; input(performance.now() / 1000, true); }
    else { root.hidden = false; last = performance.now() / 1000; wake(); }
  });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      if (timer != null) window.clearTimeout(timer);
      timer = null;
    } else wake();
  });
  document.addEventListener("pointerdown", e => { if (e.target === document.body) { closeMenu(); wake(); } });
  reducedMotion.addEventListener("change", () => { world.motion = snapshot.motion && !reducedMotion.matches; wake(); });
  window.addEventListener("resize", () => {
    world.sync({ ...activeSnapshot(snapshot), motion: snapshot.motion && !reducedMotion.matches }, window.innerWidth, performance.now() / 1000);
    wake();
  });
  await emitToWindow("island", PET_EVENTS.ready);
}
void start();
