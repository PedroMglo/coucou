import {taskPet} from "../mochi/tasks";
// « OK Coucou » asks the island what only it knows (src-tauri/src/voice): the
// pills, to add, remove or pick the main one, and the user's own data for
// Claude to answer from. Port of the Mac's VoiceTools pills and service_info.

import { invoke } from "@tauri-apps/api/core";
import { Bridge, IS_TAURI, onEvent } from "../core/bridge";
import { MAX_DECLARED, availablePills, pillDefinition, sanitizeDeclared } from "../core/pills";
import {PetRosterPreferences} from '../core/pet-roster-settings';
import { Spotify } from "../core/spotify";
import { State } from "../core/state";
import { serviceCards } from "./phone-services";

interface Ask {
  id: number;
  kind: "pills" | "info";
  input: { action?: string; names?: string[] };
}

const norm = (s: string) =>
  s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]/g, "");

/** The pill a spoken name means: exact, then the start, then anywhere in it. */
export function findPill(name: string): string | null {
  const wanted = norm(name);
  if (!wanted) return null;
  const pills = availablePills(State.os).filter((p) => p.support !== "soon");
  const hit =
    pills.find((p) => norm(p.name) === wanted || norm(p.id.replace(/^(integration|agent)_/, "")) === wanted) ??
    pills.find((p) => norm(p.name).startsWith(wanted)) ??
    pills.find((p) => norm(p.name).includes(wanted) || wanted.includes(norm(p.name)));
  return hit?.id ?? null;
}

function describe(): string {
  const d = sanitizeDeclared(State.settings, State.os);
  const name = (id: string) => pillDefinition(id)?.name ?? id;
  const all = availablePills(State.os).filter((p) => p.support !== "soon").map((p) => p.name);
  return JSON.stringify({
    main: name(d.mainPill),
    shown: d.activeIntegrations.map(name),
    maxBesidesMain: MAX_DECLARED,
    available: all,
  });
}

const rosterPreferences=new PetRosterPreferences({save:request=>Bridge.petRosterSettingsPatch(request),adopt:roster=>{
 const previousMain=State.mainPillId;State.settings={...State.settings,...roster};State.loadIntegrationTasks();if(State.mainPillId!==previousMain)State.setFocus(State.mainPillId);
}});

export async function pills(input: Ask["input"]): Promise<string> {
  const action = input.action ?? "list";
  if (action === "list") return describe();
  if(!['set_main','add','remove'].includes(action))throw Error('Unknown pill action');
  const names = input.names ?? [];
  const done: string[] = [];
  const failed: string[] = [];
  for (const spoken of names) {
    const id = findPill(spoken);
    if (!id) {
      failed.push(`${spoken}: unknown pill`);
      continue;
    }
    const label = pillDefinition(id)?.name ?? id;
    if (action === "set_main") {
      try{await rosterPreferences.change({action:'setMain',id});done.push(label);}catch(error){failed.push(`${label}: ${String(error)}`);}
      continue;
    }
    try{await rosterPreferences.change({action:'setActive',id,enabled:action==='add'});done.push(label);}catch(error){failed.push(`${label}: ${String(error)}`);}
  }
  return JSON.stringify({ done, failed, now: JSON.parse(describe()) });
}

function info(): string {
  const sessions = State.tasks
    .filter((t) => !t.isIntegration || t.state !== "idle" || t.steps.length > 0)
    .map((t) => ({ pill: t.name, state: t.state, doing: t.steps[t.stepIndex] ?? t.steps[t.steps.length - 1] ?? null }));
  const plan = (p: typeof State.planUsage) =>
    p && {
      fiveHourPercent: p.fiveHour?.usedPct ?? null,
      weekPercent: p.sevenDay?.usedPct ?? null,
    };
  const s = Spotify.state;
  return JSON.stringify({
    pills: JSON.parse(describe()),
    sessions,
    claudePlan: plan(State.planUsage),
    codexPlan: plan(State.codexPlanUsage),
    spotify: s.running && s.track ? { playing: s.playing, title: s.track.title, artist: s.track.artist } : null,
    services: serviceCards().map((c) => ({ service: c.title, headline: c.headline, about: c.reason, items: c.items })),
  });
}

export function registerVoiceHandlers() {
  if (!IS_TAURI) return;
  void onEvent('settings-changed',settings=>rosterPreferences.observe(settings));
  void onEvent<string>("voice-pet", (provider) => {
    const pill = taskPet(provider) ?? (provider === "ollama" ? "ai_ollama" : null);
    if (!pill) return;
    const definition = pillDefinition(pill);
    if (definition) State.upsertExternalAgent(pill, definition.name, definition.color);
    State.setFocus(pill);
  });
  void onEvent<Ask>("voice-ask", async (ask) => {
    let text: string;
    try {
      text = ask.kind === "pills" ? await pills(ask.input ?? {}) : info();
    } catch (err) {
      text = `error: ${String(err)}`;
    }
    void invoke("voice_done", { id: ask.id, text }).catch(() => {});
  });
}
