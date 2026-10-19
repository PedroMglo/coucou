// Hook events from Claude Code and every other agent → island state.
// Port of HookServer.processEvent / processPermissionRequest from the macOS app.
// Difference from macOS: no terminal filter. On Windows the hook fires from any
// terminal (Windows Terminal, VS Code, PowerShell…) and all of them are handled.
// The relay has already mapped every agent's events and fields onto Claude
// Code's (hook/src/normalize.rs), so one handler serves them all.

import { Bridge, onEvent,emitToWindow } from "../core/bridge";
import { buildFileDiff, fileName, makeDiffStep, toOneLine } from "../core/diff";
import { Sound } from "../core/sound";
import { State, type AskedQuestion, type AgentTask } from "../core/state";
import {sessionKey,sessionLabel} from '../core/pet-sessions';
import { pillDefinition } from "../core/pills";
import { APPROVAL_AGENTS, agentColor, agentName, validateAgent } from "./agents";
import type { Island } from "./island";
import { parseClaudePlan, restorePlanUsage } from "../core/plan";
import { setClaudePlanUsage, storedClaudePlanUsage } from "../views/usage";
import { N_, t } from "../i18n/i18n";
import { taskPet } from "../mochi/tasks";

const CLAUDE_ID = "integration_claude";
const CURSOR_ID = "agent_cursor";

/** Clears the approval card if no decision was made before the hook gave up. */
let pendingTimeout: number | null = null;

interface ApprovalHandoff {requestId:string;sessionId:string;agent?:string;provider?:string|null;reason?:string}
/** The relay has stopped accepting a UI answer. Keep the waiting session and
 * take the user to its terminal; never invent a CLI decision. */
export function handoffPendingApproval(island:Island,event:ApprovalHandoff):boolean{
  const request=State.pendingApproval;
  if(!request||request.requestId!==event.requestId||request.sessionId!==event.sessionId)return false;
  if(pendingTimeout!==null)window.clearTimeout(pendingTimeout);pendingTimeout=null;
  const session=State.petSessions.entries.get(sessionKey(request.pillId,request.backend??'claude',request.sessionId));
  const task=session??State.tasks.find(t=>t.id===request.pillId&&t.sessionId===request.sessionId);
  dropPendingCard(island);
  if(task){task.decisionInTerminal=true;task.state=request.questions?'question':'approval';}
  State.projectSessions(request.pillId);State.notify();return true;
}

/** Takes the approval or question card down and gives the island back. */
export function dropPendingCard(island: Island): void {
  if (!State.pendingApproval) return;
  const editor=State.endApproval();
  island.dropPin();
  if(editor&&State.settings.companionMode)island.alert(editor);
  else if (State.view === "approval" || State.view === "question") {
    island.setView(State.defaultView());
  }
  State.notify();
}

/** The return to idle that Stop arms, per pill, so the next turn can cancel it. */
const stopTimers = new Map<string, number>();

function cancelStopTimer(id: string): boolean {
  const timer = stopTimers.get(id);
  if (timer == null) return false;
  window.clearTimeout(timer);
  stopTimers.delete(id);
  return true;
}

/** Events after which a pending permission request of the same session is moot. */
const TURN_OVER = new Set(["Stop", "StopFailure", "UserPromptSubmit", "SessionEnd", "Interrupt"]);

interface HookPayload {
  hook_event_name?: string;
  request_id?: string;
  session_id?: string;
  cwd?: string;
  message?: string;
  /** UserPromptSubmit carries `prompt`; `message` belongs to Notification/Stop. */
  prompt?: string;
  /** Stop: the turn's final answer (Markdown) — Claude Code's, Hermes's, Codex's. */
  last_assistant_message?: string;
  tool_name?: string;
  tool_input?: Record<string, unknown>;
  /** Set by the relay when an edit was too big to forward whole (> 256 KB). */
  coucou_diff_truncated?: boolean;
  /** Optional agent tag: lowercase, digits and hyphens, ≤ 24 chars. */
  coucou_agent?: string;
  /** App-owned mapping for an exact live native session, never set by the relay. */
  coucou_pet_provider?: string;
  coucou_assistant_gate?: boolean;
  /** Hermes: where the session runs (telegram, discord…; "cli" in a terminal). */
  platform?: string;
  /** "cursor" when Claude Code runs in Cursor's terminal (set by the relay). */
  term_editor?: string;
  /** StatusLine (the plan usage relay): Claude Code's 5-hour and weekly limits. */
  rate_limits?: unknown;
}

const PROJECT_ALIASES: Record<string, string> = {
  "notch-buddy": "Notch Buddy",
  notchbuddy: "Notch Buddy",
  notch_buddy: "Notch Buddy",
};

function aliasProjectName(name: string): string {
  return PROJECT_ALIASES[name.toLowerCase()] ?? name;
}

function lastPathComponent(p: string): string {
  const cleaned = p.replace(/[\\/]+$/, "");
  const idx = Math.max(cleaned.lastIndexOf("\\"), cleaned.lastIndexOf("/"));
  return idx >= 0 ? cleaned.slice(idx + 1) : cleaned;
}

/**
 * localizedStep() — same labels as the macOS app, in English and shown in the
 * interface language (src/i18n) when the step is recorded.
 */
const TOOL_LABELS: Record<string, string> = {
  Bash: N_("Runs"),
  Read: N_("Reads"),
  Write: N_("Writes"),
  Edit: N_("Edits"),
  Glob: N_("Searches"),
  Grep: N_("Searches"),
  WebSearch: N_("Searches the web"),
  WebFetch: N_("Fetches"),
  TodoWrite: N_("Tasks"),
  Task: N_("Agent"),
  LS: N_("Lists"),
  MultiEdit: N_("Edits"),
  NotebookEdit: N_("Notebook"),
  PowerShell: N_("Runs"),
  // Antigravity's tools (#298).
  run_command: N_("Runs"),
  view_file: N_("Reads"),
  write_to_file: N_("Writes"),
  replace_file_content: N_("Edits"),
  read_url_content: N_("Fetches"),
  search_web: N_("Searches the web"),
};

function stepLabel(tool: string, input: Record<string, unknown>): string {
  const label = TOOL_LABELS[tool] ? t(TOOL_LABELS[tool]) : tool;
  const str = (k: string) => (typeof input[k] === "string" ? (input[k] as string) : null);
  const cmd = str("command");
  if (cmd) return `${label} · ${cmd.slice(0, 40)}`;
  const path = str("path");
  if (path) return `${label} · ${lastPathComponent(path)}`;
  const file = str("file_path");
  if (file) return `${label} · ${lastPathComponent(file)}`;
  const query = str("query");
  if (query) return `${label} · ${query.slice(0, 40)}`;
  return label;
}

/**
 * What the Allow button actually authorises. Approving "Write" tells you nothing
 * — approving `Write · C:\…\.env` tells you everything, and the difference is
 * the whole point of approving from the island rather than blind.
 *
 * Ordered by how specific the field is, so an unfamiliar tool still shows
 * whatever identifying string it carries instead of falling back to its name.
 */
const APPROVAL_FIELDS = [
  "command", // Bash, PowerShell
  "file_path", // Write, Edit, MultiEdit, NotebookEdit
  "path", // Read, LS
  "url", // WebFetch
  "query", // WebSearch
  "pattern", // Glob, Grep
  "prompt", // Task
] as const;

function approvalTarget(tool: string, input: Record<string, unknown>): string {
  for (const field of APPROVAL_FIELDS) {
    const value = input[field];
    if (typeof value === "string" && value.trim()) {
      return `${tool} · ${value.trim()}`;
    }
  }
  return tool;
}

/**
 * The questions of an AskUserQuestion call, if the island can show all of them
 * as options to pick from. Anything it cannot is left to the terminal.
 */
function askedQuestions(tool: string, input: Record<string, unknown>): AskedQuestion[] | null {
  if (tool !== "AskUserQuestion" || !Array.isArray(input.questions)) return null;
  const out: AskedQuestion[] = [];
  for (const raw of input.questions as Record<string, unknown>[]) {
    const question = typeof raw?.question === "string" ? raw.question : "";
    const options = (Array.isArray(raw?.options) ? (raw.options as Record<string, unknown>[]) : [])
      .filter((o) => typeof o?.label === "string" && o.label)
      .map((o) => ({
        label: o.label as string,
        description: typeof o.description === "string" ? o.description : "",
      }));
    // A question cut short by the relay would be answered under the wrong text.
    if (!question || question.endsWith("…") || options.length < 2) return null;
    out.push({ question, options, multiSelect: raw.multiSelect === true });
  }
  return out.length > 0 ? out : null;
}

/** The Claude Code session's pill, named after its project for the session. */
function upsert(id: string, projectName: string, cwd: string, sessionId: string) {
  const t = State.upsertWorkspacePill(id, projectName, cwd);
  if (t && sessionId) t.sessionId = sessionId;
}

/** The session is over: the pill goes back as it was, or away if it was only there for it. */
function clearSession(id: string) {
  const t = State.tasks.find((x) => x.id === id);
  if (!t) return;
  if (!State.isKept(id)) {
    State.removeTask(id);
    return;
  }
  t.state = "idle";
  t.steps = [];
  t.stepIndex = 0;
  delete t.stepSeq;
  t.name = pillDefinition(id)?.name ?? t.name;
  t.pillBadge = null;
  t.sessionId = null;
  t.finalLine = null;
}

/**
 * PostToolUse of Edit / MultiEdit / Write → a diff stored for the pill and a
 * ticker step with its +N −M counts. Nothing is kept for a pill that does not
 * exist, so a stray event cannot grow memory. An edit the relay had to cut
 * would give wrong counts: the PreToolUse step ("Edits · file") stands alone.
 */
function recordDiff(agentId: string, payload: HookPayload, target?:AgentTask, append?:(step:string)=>void) {
  if (payload.coucou_diff_truncated) return;
  if (!State.tasks.some((t) => t.id === agentId)) return;
  const diff = buildFileDiff(payload.tool_name ?? "", payload.tool_input ?? {});
  if (!diff) return;
  const key=target?.sessionId?sessionKey(agentId,target.sessionBackend??'',target.sessionId):agentId;
  const id = State.appendSessionDiff(key, diff);
  if(append)append(makeDiffStep(fileName(diff.path),diff.added,diff.removed,id));
  else State.appendStep(agentId, makeDiffStep(fileName(diff.path), diff.added, diff.removed, id));
}

export function registerHookHandlers(island: Island) {
  // The last plan numbers seen survive a restart, as on the Mac.
  State.planUsage ??= restorePlanUsage(storedClaudePlanUsage());
  void onEvent<HookPayload>("hook", (payload) => handleHook(island, payload));
  void onEvent<ApprovalHandoff>('approval-handoff',event=>handoffPendingApproval(island,event));
}

function handleHook(island: Island, payload: HookPayload) {
  // Account-wide numbers from the status line relay, not part of any session:
  // keep the latest, nothing else (no reveal, no sound), paused or not.
  if (payload.hook_event_name === "StatusLine") {
    const usage = parseClaudePlan(payload.rate_limits);
    if (usage) setClaudePlanUsage(usage);
    return;
  }

  if (State.paused) {
    // Silence here used to cost Claude Code nearly two minutes: the relay waited
    // for a decision from an island that had already decided not to look. Say so,
    // and the terminal takes the question immediately.
    if (payload.request_id) void Bridge.approvalDecline(payload.request_id);
    return;
  }

  const name = payload.hook_event_name ?? "";
  const cwd = payload.cwd ?? "";
  const raw = lastPathComponent(cwd);
  const projectName = aliasProjectName(raw || "Session");

  // Route to the right pill. Valid coucou_agent → dynamic "agent_<name>" pill.
  // "claude" is reserved; absent or invalid → Claude Code's own pill: Cursor's
  // when it runs in Cursor's terminal (Mac #120), VS Code's otherwise.
  const validAgent = validateAgent(payload.coucou_agent);
  const workspaceId = payload.term_editor === "cursor" ? CURSOR_ID : CLAUDE_ID;
  const ownedPet=payload.coucou_pet_provider ? taskPet(payload.coucou_pet_provider) : null;
  const agentId = ownedPet ?? (validAgent ? `agent_${validAgent}` : workspaceId);
  const isExternalAgent = validAgent !== null;
  const sessionId = payload.session_id ?? "";
  const backend=validAgent??'claude';
  const base=State.tasks.find(t=>t.id===agentId)??{id:agentId,name:isExternalAgent?agentName(validAgent!):projectName,color:agentColor(backend),state:'idle',steps:[],stepIndex:0,source:'agent',isIntegration:false} as AgentTask;
  const session=sessionId?State.petSessions.observe(base,backend,sessionId,cwd,!!ownedPet):undefined;
  if(session){
    session.name=isExternalAgent?(pillDefinition(agentId)?.name??agentName(validAgent!)):projectName;
    const prompt=payload.prompt??(name==='UserPromptSubmit'?payload.message:undefined);
    if(prompt&&!session.sessionTitle)session.sessionTitle=toOneLine(prompt).slice(0,80);
  }
  const timerKey=session?sessionKey(agentId,backend,sessionId):agentId;
  const currentTask=()=>session??State.tasks.find(t=>t.id===agentId);
  const publish=()=>{if(session)State.projectSessions(agentId);else State.notify();};
  const updateTask=(_id:string,state:AgentTask['state'])=>{if(session)session.state=state;else State.updateTask(agentId,state);};
  const appendStep=(_id:string,step:string)=>{
    if(!session){State.appendStep(agentId,step);return;}
    session.stepSeq=(session.stepSeq??0)+1;session.steps.push(step);if(session.steps.length>20)session.steps.shift();session.stepIndex=session.steps.length-1;
  };
  const setBadge=(_id:string,badge:AgentTask['pillBadge'])=>{if(session)session.pillBadge=badge;else State.setPillBadge(agentId,badge??null);};
  const clearFinal=()=>{const task=currentTask();if(task)task.finalLine=null;};

  /** Alerts force the island open; work events only reveal the compact island. */
  const surface = (view: Parameters<Island["alert"]>[0], isAlert: boolean) => {
    publish();
    if (State.settings.companionMode) {
      // Completion is the pet's speech. Decisions/errors retain their original card.
      if (isAlert && view !== "finished") { State.setFocus(agentId); island.alert(view); }
      else State.notify();
      return;
    }
    if (State.mode === "expanded") {
      if (isAlert) island.setView(view);
    } else if (isAlert) {
      island.alert(view);
    } else if (State.mode === "hidden") {
      island.reveal();
    }
  };

  /** Ensure the agent pill exists (no-op for Claude Code). */
  const ensurePill = () => {
    if (isExternalAgent && agentId!==CLAUDE_ID) {
      State.upsertExternalAgent(agentId, pillDefinition(agentId)?.name ?? agentName(validAgent!), pillDefinition(agentId)?.color ?? agentColor(validAgent!));
      const t = State.tasks.find((x) => x.id === agentId);
      if (!session && t && cwd) t.sessionCwd = cwd;
    } else {
      upsert(agentId, projectName, session?'':cwd, session?'':sessionId);
    }
    const task=State.tasks.find(t=>t.id===agentId);
    if(task && session)State.petSessions.project(task,State.pendingApproval);
  };

  /**
   * Called where a handler is about to replace `finished` with a newer state:
   * the timer Stop armed would otherwise put the pill back to idle over it. The
   * badge that timer was going to clear goes now.
   */
  const supersedeStop = () => {
    if (cancelStopTimer(timerKey)) setBadge(agentId, null);
    if(session)cancelStopTimer(agentId); // An earlier unlabelled legacy event.
    if(session)session.decisionInTerminal=false;
  };
  if(session)ensurePill();

  // The turn that asked for a permission is over — answered in the terminal,
  // interrupted, or a new prompt — so the card would be lying. It goes, and the
  // relay is released without a decision. Same rule as the Mac.
  const pending = State.pendingApproval;
  if (
    pending &&
    TURN_OVER.has(name) &&
    pending.pillId === agentId &&
    pending.sessionId === (payload.session_id ?? "")
  ) {
    if (pendingTimeout != null) window.clearTimeout(pendingTimeout);
    pendingTimeout = null;
    if (pending.requestId) void Bridge.approvalDecline(pending.requestId);
    dropPendingCard(island);
  }

  // Read after the card above is dropped: its pill may have handed the front
  // back to the pill you were on.
  const focused = State.focusId === agentId;

  switch (name) {
    case "SessionStart":
      ensurePill();
      clearFinal();
      // Hermes through its gateway says where the session comes from.
      if (validAgent === "hermes" && payload.platform && payload.platform !== "cli") {
        appendStep(agentId, payload.platform.charAt(0).toUpperCase() + payload.platform.slice(1));
      }
      surface("overview", false);
      Sound.play("work");
      break;

    case "UserPromptSubmit": {
      ensurePill();
      supersedeStop();
      clearFinal();
      updateTask(agentId, "thinking");
      // The field is `prompt`; reading `message` meant this step was always blank.
      const asked = payload.prompt ?? payload.message;
      if (asked) appendStep(agentId, asked.slice(0, 60));
      surface("overview", false);
      break;
    }

    case "PreToolUse": {
      ensurePill();
      supersedeStop();
      clearFinal();
      updateTask(agentId, "working");
      const tool = payload.tool_name ?? "Tool";
      appendStep(agentId, stepLabel(tool, payload.tool_input ?? {}));
      surface("overview", false);
      break;
    }

    case "PostToolUse":
      supersedeStop();
      // The question was answered in the terminal: the card would be lying.
      if (
        payload.tool_name === "AskUserQuestion" &&
        State.pendingApproval?.questions &&
        State.pendingApproval.sessionId === (payload.session_id ?? "")
      ) {
        if (pendingTimeout != null) window.clearTimeout(pendingTimeout);
        pendingTimeout = null;
        dropPendingCard(island);
      }
      updateTask(agentId, "working");
      recordDiff(agentId, payload, session, step=>appendStep(agentId,step));
      break;

    case "PostToolUseFailure":
      supersedeStop();
      updateTask(agentId, "working");
      appendStep(agentId, t("⚠ failed"));
      break;

    case "Notification": {
      const message = payload.message ?? "";
      const lower = message.toLowerCase();
      if (lower.includes("rate limit") || lower.includes("limite d")) {
        supersedeStop();
        updateTask(agentId, "ratelimit");
        Sound.play("rate");
      } else if (message.endsWith("?")) {
        supersedeStop();
        updateTask(agentId, "question");
        appendStep(agentId, message);
      }
      break;
    }

    case "Stop": {
      updateTask(agentId, "finished");
      // Claude Code puts the turn's answer in the Stop payload itself, so there is
      // no transcript to read (the relay does not even forward its path). Other
      // agents report their last words the same way (Hermes, Codex) or as
      // `message`.
      const finalText = toOneLine(payload.last_assistant_message ?? payload.message ?? "");
      if (finalText) {
        appendStep(agentId, finalText);
        const t = currentTask();
        if (t) t.finalLine = finalText;
        if(session&&State.settings.companionMode)void emitToWindow('companions','companions-notice',{
          key:`session:${timerKey}:${session.stepSeq}`,pet:agentId,text:finalText,urgent:false,view:'finished',
          origin:sessionLabel(session),session:{id:sessionId,backend,cwd:session.sessionCwd,title:session.sessionTitle,owned:session.sessionOwned},
        });
      }
      Sound.play("finish");
      // A card waiting for an answer is never covered by another alert.
      if ((focused || State.settings.companionMode) && !State.pendingApproval) surface("finished", true);
      else setBadge(agentId, "finished");
      cancelStopTimer(timerKey);
      stopTimers.set(
        timerKey,
        window.setTimeout(() => {
          stopTimers.delete(timerKey);
          const task = currentTask();
          if (!session && isExternalAgent && !(State.settings.companionMode && task?.sessionId)) {
            State.removeTask(agentId);
          } else {
            updateTask(agentId, "idle");
            setBadge(agentId, null);
          }
          publish();
        }, 5200),
      );
      break;
    }

    case "Interrupt":
      // Codex: the user stopped the turn. Back to idle, nothing to celebrate.
      supersedeStop();
      updateTask(agentId, "idle");
      setBadge(agentId, null);
      break;

    case "StopFailure":
      supersedeStop();
      updateTask(agentId, "error");
      Sound.play("error");
      if ((focused || State.settings.companionMode) && !State.pendingApproval) surface("error", true);
      else setBadge(agentId, "error");
      break;

    case "SessionEnd":
      // Nothing left for the timer to do, and it must not outlive the session: a
      // pill recreated within 5.2 s would be removed by it.
      cancelStopTimer(timerKey);
      State.clearSessionDiffs(timerKey);
      if(session){session.ended=true;session.state="idle";if(!State.petSessions.list(agentId).length)clearSession(agentId);}
      else clearSession(agentId);
      break;

    case "SubagentStart":
      appendStep(agentId, t("+ subagent"));
      break;

    case "SubagentStop":
      appendStep(agentId, t("• subagent done"));
      break;

    case "PermissionRequest": {
      // Only Claude Code and the agents the relay can answer for (Codex, Copilot
      // CLI, Muse Code — same as the Mac) get a card. Anyone else's request is
      // declined at once, so the agent asks in its own terminal.
      const gravityGate=validAgent==="antigravity" && payload.coucou_assistant_gate===true && !!ownedPet;
      if (isExternalAgent && !APPROVAL_AGENTS.has(validAgent!) && !gravityGate) {
        if (payload.request_id) void Bridge.approvalDecline(payload.request_id);
        if(session){session.state='approval';session.decisionInTerminal=true;}
        break;
      }

      const requestId = payload.request_id ?? "";
      // One card, one request. A second one must never quietly replace the first
      // — that would leave a human staring at request B while request A waits for
      // a decision nobody can give. Hand it straight back to the terminal.
      if (State.pendingApproval && State.pendingApproval.requestId !== requestId) {
        if (requestId) void Bridge.approvalDecline(requestId);
        if(session){session.state='approval';session.decisionInTerminal=true;}
        break;
      }
      ensurePill();
      supersedeStop();
      if (pendingTimeout != null) window.clearTimeout(pendingTimeout);
      const tool = payload.tool_name ?? "Tool";
      const input = payload.tool_input ?? {};
      // Claude Code asking a question is not a permission to grant: the island
      // shows the options and sends back the one that was picked. Only Claude
      // Code asks questions this way.
      const questions = isExternalAgent ? null : askedQuestions(tool, input);
      const view = questions ? "question" : "approval";
      // The card always comes up, even over another pill or an island that is
      // already open: its pill comes to the front, and the one you were on
      // comes back once you answer (Mac #117, #120).
      const interrupted=State.mode==='expanded'?State.view:undefined;
      if (State.settings.companionMode && State.mode === "expanded" && State.focusId !== agentId) island.collapse();
      State.beginApproval({
        requestId,
        sessionId,
        sessionTitle:session?.sessionTitle,cwd:session?.sessionCwd??cwd,backend,owned:!!ownedPet,
        pillId: agentId,
        tool,
        command: approvalTarget(tool, input),
        ...(questions ? { questions } : {}),
      },interrupted);
      // The relay's short ack window closes in 800 ms; everything below this
      // line is synchronous, so the card really is up by the time it lands.
      if (requestId) void Bridge.approvalAck(requestId);
      updateTask(agentId, view);
      Sound.play(view);
      // Any agent's card (Claude Code, Codex, Copilot CLI, Muse Code) comes up
      // the same way: beginApproval brought its pill to the front.
      island.alert(view);
      // Coucou answers within 108 s or not at all; after that the terminal has
      // taken over and the card would be lying.
      pendingTimeout = window.setTimeout(() => {
        pendingTimeout = null;
        handoffPendingApproval(island,{requestId,sessionId,reason:'timeout'});
      }, 110_000);
      break;
    }

    default:
      break;
  }
  publish();
}
