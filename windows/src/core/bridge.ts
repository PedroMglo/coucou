import {parseShortcutReconciliationPreview,parseShortcutReconciliationReceipt,type ShortcutReconciliationDecision} from './shortcuts-reconciliation';
import {parseAutostartPreview,parseAutostartReceipt,parseAutostartStatus,type AutostartRequest,type AutostartDecision} from './autostart-review';
import {parseShortcutReviewPreview,parseShortcutReviewReceipt,parseShortcutReviewStatus,type ShortcutReviewKey,type ShortcutReviewDecision} from './shortcuts-review';
import {parseShortcutPatchReceipt,type ShortcutPatch} from './shortcuts-settings';
import {settingsIoStatus as parseSettingsIoStatus} from './settings-io';
import {parseSettingsReviewPreview,parseSettingsReviewReceipt,type SettingsReviewDecision} from './settings-review';
import {captureAttachments} from './document-collection';
// Thin wrapper over the Tauri commands/events. Every call is a no-op when the
// page is opened in a plain browser, so the island can be iterated on with
// `npm run dev` alone.

import { invoke } from "@tauri-apps/api/core";
import { emitTo, listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { Settings } from "./state";
import type { RecapHistory, RecapPrefs } from "../recap/summary";
import type { SpotifyState } from "./spotify";
import type {PetConfiguration,PetDocument,InferenceUpdate} from "../mochi/pet-preferences";
import {reportSettingsError} from './settings-feedback';
import {cardPresentation} from './card-presentation';
import type { AssistantTask } from "../mochi/tasks";
export interface RecapPersistenceReceipt {
 prefs:RecapPrefs;persisted:boolean;historyCleared:boolean;
 errorCode:null|'writeFailed'|'invalidWeek';
}

export const IS_TAURI =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!IS_TAURI) return null;
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    console.error(`[coucou] ${cmd} failed`, err);
    return null;
  }
}

export interface BootInfo {
  settings: Settings;
  /** Logical screen rect of the monitor the island lives on. */
  screen: { x: number; y: number; width: number; height: number; scale: number };
  version: string;
  hookPath: string;
  /** False where the OS has no global cursor (Wayland): see Island.followPageCursor. */
  cursorPoll: boolean;
}

/** Historical references are display/provenance data and grant no file access. */
export interface AttachmentTurnRecord {
 id:string;provider:string;petId:string;conversationId:string;taskId?:string;
 started:number;updated:number;messageIndex:number;messagesAdded:0|2;
 phase:'pending'|'done'|'failed'|'stopped'|'interrupted';
 attachments:Array<{id:string;name:string;kind:'file'|'folder';size:number;entries:number;sequence:number}>;
}
export interface ConversationView {id:string;messages:{role:"user"|"assistant";content:string}[];busy?:boolean;turn?:number;stream?:string;query?:string;notice?:string;taskNotice?:import('./task-notice').TaskNotice;requestHistory?:import('./request-history').RequestRecord[];localExecution?:import('./local-capabilities').LocalExecutionCapability;attachmentHistory?:AttachmentTurnRecord[]}
export interface ChatTurnState {provider:string;conversationId:string;turn:number;busy:boolean;query:string}
export interface NativeApprovalChoice {id:string;label:string;scope:'once'|'turn'|'session'|'policy'|'turnCancellation'|'nativeRemembered';allow:boolean;detail?:string|null}
export interface RuntimeNativePeerSequenceReview {roundDataProvenance?:unknown;scope:{id:string;goalId:string;petId:string;leaseId:string;controllerId:string;deadlineAt:number;dataConsent:'exactReviewedNativePeerPlan'};goalRevision:string;intent:string;criteria:string[];steps:RuntimeMessageTicket['proposal'][];preparedAt:number;expiresAt:number;criteriaVerified:false;goalComplete:false;recipientExecutionAuthorized:false;providerDataUseAuthorized:false}
interface GoalDecisionReviewBase {budget:unknown;executionStarted:false;modelPlanningGranted:false;additionalModelPlanningGranted?:false;existingGoalConsentOperation?:'conversationRespond'|'goalPlanning'|'goalDeterministic'}
export type GoalDecisionReview=GoalDecisionReviewBase&({approvalScope:'exactReviewedRecipe';recipe:Omit<RuntimeRecipeProposal,'steps'>&{steps:Omit<RuntimeStepProposal,'intent'|'criteria'>[]}}|{approvalScope:'exactReviewedNativePeerPlan';recipe:RuntimeNativePeerSequenceReview;recipientExecutionAuthorized:false;providerDataUseAuthorized:false}|{approvalScope:'exactReviewedPeerPlanningData';data:{schemaVersion:1;scope:{id:string;goalId:string;petId:string;dataConsent:'oneExactPeerPlanningInputMetadataOnly';maxUses:1;contextId:string;turnId:number};goalRevision:string;intent:string;criteria:string[];sources:unknown;destination:unknown;actualRequestBody:unknown;preparedAt:number;expiresAt:number;executionAuthorized:false;recipientExecutionAuthorized:false;newBudgetGranted:false;policyAllowsAnotherRound:false;requiresOriginalJournalAndBudgetBeforeDispatch:true};recipientExecutionAuthorized:false;newBudgetGranted:false;requiresOriginalJournalAndBudgetBeforeDispatch:true});
export interface AgentRequest {
  id:string;provider:string;kind:string;detail:string;origin?:string;
  approvalScope?:'turn'|'once'|'recipe';
  goalTarget?:GoalRecipeTarget;goalReview?:GoalDecisionReview;voiceCode?:string;
  nativeChoices?:NativeApprovalChoice[];nativeChoicesSource?:'nativeAdvertised'|'protocolFallback'|'nativePermissionProtocol'|'nativeOptions'|'nativeLegacyProtocol';
  target?:import('./session-chat').SessionTarget;
  nativeForm?:'mcp';
  questions?:AgentQuestion[]|null;
}
export interface GoalRecipeTarget {origin:'goal'|'goalPeerPlan'|'goalPeerData';petId:string;provider:string;goalId:string;contextId:string;leaseId:string;controllerId:string;recipeId:string;planId:string;revision:string}
export type DecisionTarget=import('./session-chat').SessionTarget|GoalRecipeTarget;
export interface AgentQuestion {
  id:string;question:string;isSecret?:boolean;required?:boolean;hint?:string;
  inputType?:'text'|'number'|'integer'|'boolean'|'enum'|'multi';
  options?:{label:string;value?:string}[]|null;
}
export interface ConversationSummary {id:string;provider:string;title:string;preview:string;created:number;updated:number;messageCount:number;current:boolean}
export interface ConversationPurgeReview {reviewId:string;kind:'conversation'|'recovery';provider:string;conversationId:string;title:string;messageCount:number;taskCount:number;attachmentCount:number;current:boolean;expiresInSeconds:number}
export interface ConversationPurgeRecovery {id:string;revision:string;target:{provider:string;conversationId:string;title:string;messageCount:number;taskCount:number;attachmentCount:number;current:boolean;revision:string}}
export type ConversationPurgeOutcome={kind:'conversation';receipt:{provider:string;removedConversationId:string;current:ConversationView|null;recoveryId:string|null;durable:boolean};copySelectionCleared:boolean}|{kind:'recovery';recoveryId:string;finished:boolean;restored:boolean};

/** The phone link (src-tauri/src/phone_link): how it is, never its secrets. */
export interface PhoneLinkStatus {
  enabled: boolean;
  running: boolean;
  port: number;
  host: string;
  name: string;
  clients: number;
  error: string | null;
  /** The relay connection ("Away from home Wi-Fi"). */
  relayState: PhoneRelayState;
}

export type PhoneRelayState = "off" | "connecting" | "waiting" | "linked" | "accessRefused" | "unreachable" | "roomTaken" | "tooMany";

/** The relay switch and address. The access key is write-only: only whether one is stored comes back. */
export interface PhoneRelayStatus {
  enabled: boolean;
  url: string;
  hasAccess: boolean;
  state: PhoneRelayState;
}

/** Session details for the phone: the switch (off by default). */
export interface PhoneDetailsStatus {
  enabled: boolean;
}

/** Chat from the phone: the switch and the models the phone may use ("provider/model"). */
export interface PhoneChatStatus {
  enabled: boolean;
  models: string[];
}

/** What the settings window shows to pair a phone. Only the settings window may ask. */
export interface PhoneLinkPairing {
  link: string;
  qrSvg: string;
  host: string;
  port: number;
  name: string;
  /** The relay's host when this link also carries the relay fields. */
  relay: string | null;
}

/** One session as the phone sees it; Rust stamps `updatedAt`. */
export interface PhoneLinkSession {
  pillId: string;
  agent: string;
  state: string;
  statusText: string;
  stepIndex: number;
  stepCount: number;
  /** Details, for a phone that has the capability (the user's switch). */
  steps?: string[];
  finalLine?: string;
  /** A folder name, never a path. */
  project?: string;
  color?: string;
  /** The files this session changed, for a phone that has `diffs` (Rust cuts them to a name and a count). */
  files?: PhoneLinkFile[];
}

export interface PhoneLinkFile {
  id: number;
  /** A file name; Rust keeps only its last segment. */
  name: string;
  added: number;
  removed: number;
  tooLarge?: boolean;
  isNew?: boolean;
}

/** One file's diff for the phone that asked: lines as [kind, text], kind "+", "-", " " or "@" (a hunk's first line). */
export interface PhoneLinkDiff {
  pillId: string;
  fileId: number;
  name: string;
  added: number;
  removed: number;
  tooLarge: boolean;
  gone: boolean;
  lines: [string, string][];
}

/** One plan window for the phone: used percentage (as the island has it) and when it resets (epoch ms). */
export interface PhoneLinkWindow {
  usedPct: number;
  resetsAt: number;
}

export interface PhoneLinkPlan {
  fiveHour?: PhoneLinkWindow;
  sevenDay?: PhoneLinkWindow;
  resetCredits?: number;
  planType?: string;
  updatedAt: number;
}

export interface PhoneLinkUsage {
  claude: PhoneLinkPlan | null;
  codex: PhoneLinkPlan | null;
}

/** What one service pill says, for a phone whose user allowed it (Rust cuts every text to size). */
export interface PhoneLinkService {
  id: string;
  title: string;
  headline: string;
  reason?: string;
  items: { label: string; detail: string }[];
}

export interface PhoneServicesStatus {
  /** The services allowed for the phone. */
  services: string[];
}

export interface PhoneUsageStatus {
  enabled: boolean;
}

export interface PhoneDiffsStatus {
  enabled: boolean;
}

export interface PhoneLinkApproval {
  requestId: string;
  sessionId: string;
  pillId: string;
  tool: string;
  command: string;
}

/** The question Claude Code is waiting on, for a phone that may answer it (phone_link/hub.rs). */
export interface PhoneLinkQuestion {
  requestId: string;
  sessionId: string;
  pillId: string;
  questions: { question: string; options: { label: string; description: string }[]; multiSelect: boolean }[];
}

export interface PhoneAnswersStatus {
  enabled: boolean;
}

/** Stable request IDs survive retries; these draft APIs never dispatch work. */
function runtimeBudget(b:RuntimeBudget):RuntimeBudget{return {deadlineAt:b.deadlineAt,maxModelCalls:b.maxModelCalls,maxToolCalls:b.maxToolCalls,maxOutputBytes:b.maxOutputBytes,maxTokens:b.maxTokens??null,maxCostMicros:b.maxCostMicros??null,payment:b.payment};}
function runtimeDraftAction(action:RuntimeDraftAction):RuntimeDraftAction{
 switch(action.kind){
  case 'proposeTask':return {kind:action.kind,description:action.description};
  case 'proposePlan':return {kind:action.kind,taskId:action.taskId,steps:action.steps.map(step=>({description:step.description,postcondition:step.postcondition,...(step.operation===undefined?{}:{operation:step.operation===null?null:{operationId:step.operation.operationId,version:step.operation.version,arguments:structuredClone(step.operation.arguments)}})}))};
  case 'pause':case 'resumeDraft':case 'cancel':return {kind:action.kind};
  default:throw Error('Esta operação não está disponível no repositório de objetivos.');
 }
}
export const Bridge = {
  petModelSelect:(model:string,selectionId:string)=>callOrThrow<{provider:string;model:string;selectionId:string;generation:number}>('pet_model_select',{model,selectionId}),
  petFocusSession:(provider:string,id:string)=>callOrThrow<{message?:string}>('pet_focus_session',{provider,id}),
  boot: () => call<BootInfo>("boot"),
  companionsPresence: () => call<{running:string[];activity:Record<string,number>}>("companions_presence"),
  /** The system's languages as the webview sees them, for Rust's own texts (i18n.rs). */
  setSystemLanguages: (languages: string[]) => call<void>("set_system_languages", { languages }),

  saveSettings: (settings: Settings) => callOrThrow<void>("save_settings", { settings }),
  settingsIoStatus:async()=>parseSettingsIoStatus(await callOrThrow<unknown>('settings_io_status')),
  settingsReviewPrepare:async()=>parseSettingsReviewPreview(await callOrThrow<unknown>('settings_review_prepare')),
  settingsReviewApply:async(request:SettingsReviewDecision)=>parseSettingsReviewReceipt(await callOrThrow<unknown>('settings_review_apply',{request})),
  settingsReviewCancel:(request:{token:string;fingerprint:string})=>callOrThrow<void>('settings_review_cancel',{request}),
  autostartReviewStatus:async()=>parseAutostartStatus(await callOrThrow<unknown>('pet_autostart_review_status')),
  shortcutsReconcilePrepare:async()=>parseShortcutReconciliationPreview(await callOrThrow<unknown>('shortcuts_reconcile_prepare')),
  shortcutsReconcileInspect:async(request:ShortcutReviewKey)=>parseShortcutReconciliationPreview(await callOrThrow<unknown>('shortcuts_reconcile_inspect',{request:{token:request.token,fingerprint:request.fingerprint}})),
  shortcutsReconcileApply:async(request:ShortcutReconciliationDecision)=>parseShortcutReconciliationReceipt(await callOrThrow<unknown>('shortcuts_reconcile_apply',{request:{token:request.token,fingerprint:request.fingerprint,previewSchemaVersion:request.previewSchemaVersion,reviewedObservation:request.reviewedObservation,understandPastUncertainty:request.understandPastUncertainty,keepRegistrationSuspended:request.keepRegistrationSuspended}})),
  shortcutsReviewPrepare:async()=>parseShortcutReviewPreview(await callOrThrow<unknown>('shortcuts_review_prepare')),
  shortcutsReviewPrepareCurrentScope:async()=>parseShortcutReviewPreview(await callOrThrow<unknown>('shortcuts_review_prepare_current_scope')),
  shortcutsSettingsPatch:async(request:ShortcutPatch)=>parseShortcutPatchReceipt(await callOrThrow<unknown>('shortcuts_settings_patch',{request:{changes:Object.fromEntries(Object.entries(request.changes).map(([id,c])=>[id,{before:c.before===null?null:{keys:c.before.keys,enabled:c.before.enabled},after:c.after===null?null:{keys:c.after.keys,enabled:c.after.enabled}}]))}})),
  shortcutsReviewInspect:async(request:ShortcutReviewKey)=>parseShortcutReviewPreview(await callOrThrow<unknown>('shortcuts_review_inspect',{request:{token:request.token,fingerprint:request.fingerprint}})),
  shortcutsReviewCancel:(request:ShortcutReviewKey)=>callOrThrow<void>('shortcuts_review_cancel',{request:{token:request.token,fingerprint:request.fingerprint}}),
  shortcutsReviewApply:async(request:ShortcutReviewDecision)=>parseShortcutReviewReceipt(await callOrThrow<unknown>('shortcuts_review_apply',{request:{token:request.token,fingerprint:request.fingerprint,reviewedPlan:request.reviewedPlan,understandNativeEffects:request.understandNativeEffects,keepOtherComponentsSuspended:request.keepOtherComponentsSuspended,...(request.previewSchemaVersion===undefined?{}:{previewSchemaVersion:request.previewSchemaVersion}),...(request.acknowledgeRetainedUncertainty===undefined?{}:{acknowledgeRetainedUncertainty:request.acknowledgeRetainedUncertainty})}})),
  shortcutsReviewStatus:async()=>parseShortcutReviewStatus(await callOrThrow<unknown>('shortcuts_review_status')),
  autostartReviewPrepare:async(request:AutostartRequest)=>parseAutostartPreview(await callOrThrow<unknown>('pet_autostart_review_prepare',{request:(request.action==='reconcileArchivedObservation'||request.action==='reconcileLegacyObservation')?{action:request.action,archivedEffectId:request.archivedEffectId}:request.action!=='reconcileObservation'?{action:request.action,enabled:request.enabled}:{action:request.action}})),
  autostartReviewApply:async(request:AutostartDecision)=>parseAutostartReceipt(await callOrThrow<unknown>('pet_autostart_review_apply',{request:{token:request.token,fingerprint:request.fingerprint,reviewedPlan:request.reviewedPlan,understandEffect:request.understandEffect,keepOtherComponentsSuspended:request.keepOtherComponentsSuspended,...(request.acknowledgeExternalUncertainty===undefined?{}:{acknowledgeExternalUncertainty:request.acknowledgeExternalUncertainty}),...(request.acknowledgeLegacyUncertainty===undefined?{}:{acknowledgeLegacyUncertainty:request.acknowledgeLegacyUncertainty})}})),
  autostartReviewCancel:(request:{token:string;fingerprint:string})=>callOrThrow<void>('pet_autostart_review_cancel',{request:{token:request.token,fingerprint:request.fingerprint}}),
  autostartSettingsPatch:(patch:{enabled:boolean})=>callOrThrow<AutostartPreferenceReceipt>('pet_autostart_settings_patch',{patch:{enabled:patch.enabled}}),
  petRosterSettingsPatch:(request:import('./pet-roster-settings').PetRosterRequest)=>callOrThrow<import('./pet-roster-settings').PetRosterValues>('pet_roster_settings_patch',{request:request.action==='setMain'?{action:request.action,id:request.id}:{action:request.action,id:request.id,enabled:request.enabled}}),
  petCenterSettingsPatch:(request:import('./island-availability').IslandAvailabilityPatch)=>callOrThrow<import('./island-availability').IslandAvailabilityValues>('pet_center_settings_patch',{request:{...(request.petControlBar!==undefined?{petControlBar:request.petControlBar}:{}),...(request.petSystemCenter!==undefined?{petSystemCenter:request.petSystemCenter}:{})}}),
  reportSettingsError,

  cliAgentAnswer: (id: string, answer: Record<string, unknown>,target?:DecisionTarget) => invoke<void>("cli_agent_answer", { id, answer,...(target?{target}:{}) }),
  cliAgentPending:()=>callOrThrow<AgentRequest[]>("cli_agent_pending"),

  /** Shrink the window down to the invisible wake strip (hidden) or back to full. */
  setCollapsed: (collapsed: boolean) => call<void>("set_collapsed", { collapsed }),

  /**
   * Pushes the island shape in window coordinates. Rust flips click-through from
   * its own cursor poll, so the flag is never a frame behind a click.
   */
  setIslandRect: (x: number, y: number, width: number, height: number) =>
    call<void>("set_island_rect", { x, y, width, height }),

  /** Give the window keyboard focus (chat field) and take it away again. */
  focusWindow: (focused: boolean) => call<void>("focus_window", { focused }),

  reposition: () => call<void>("reposition"),
  fitZoom: (dpr: number) => call<boolean>("fit_zoom", { dpr }),

  /** Displays the island can be pinned to: `key` is what `settings.screen` stores. */
  listMonitors: () => call<{ key: string; label: string }[]>("list_monitors"),

  openUrl: (url: string) => call<void>("open_url", { url }),

  /** Which of these sounds have a file of the user's own in the sounds folder. */
  customSounds: (names: readonly string[]) => call<string[]>("custom_sounds", { names }),
  /** The user's file for one sound, as raw bytes. */
  customSound: (name: string) => call<ArrayBuffer>("custom_sound", { name }),
  revealSoundsFolder: () => call<void>("reveal_sounds_folder"),
  /** Tells the island to read the sounds folder again ("sounds-changed"). */
  reloadSounds: () => call<void>("reload_sounds"),

  /** "Open terminal" → opens the folder in VS Code when `code` is on PATH. */
  openInVSCode: (path: string | null) => call<boolean>("open_in_vscode", { path }),

  /**
   * "Open terminal": the window the session runs in when Rust found it
   * (Windows; Linux on X11 and KDE Plasma), else the folder in VS Code.
   */
  openSession: (sessionId: string | null, path: string | null) =>
    call<boolean>("open_session", { sessionId, path }),

  /** The Claude desktop app, for the Claude Desktop pill (Windows only). */
  openClaudeDesktop: () => call<boolean>("open_claude_desktop"),

  /** The diff card's ↗: an existing file, in VS Code; never launched by its type. */
  openFileInVSCode: (path: string) => call<boolean>("open_file_in_vscode", { path }),

  quit: () => call<void>("quit_app"),

  hubRequestedPage:()=>callOrThrow<string|null>("hub_requested_page"),
  openSettingsWindow: (page?:string) => call<void>("open_settings_window", {page}),
  petManagement: <T>(action:string,provider="",identifier="",keepAlive="10m") => callOrThrow<T>("pet_management",{request:{action,provider,identifier,keepAlive}}),

  /** Writes to %LOCALAPPDATA%\Coucou\coucou.log, next to the Rust lines. */
  log: (message: string) => call<void>("log_line", { message }),

  // ── Claude Code hooks ─────────────────────────────────────────────────────
  hooksStatus: () => call<HookStatus>("hooks_status"),
  /** Pill ID → whether that agent's hooks reach Coucou (read-only, Mac #183). */
  agentHooksStatus: () => call<Record<string, boolean>>("agent_hooks_status"),
  /** Diff to show before anything is written. `install: false` previews removal. */
  hooksPreview: (install: boolean) => callOrThrow<HookPreview>("hooks_preview", { install }),
  /**
   * Writes ~/.claude/settings.json — only ever after an explicit click, and only
   * when the file still matches the preview the user looked at.
   */
  hooksApply: (install: boolean, fingerprint: string) =>
    callOrThrow<string>("hooks_apply", { install, fingerprint }),

  // ── Other agents (Gemini CLI, Codex, Cursor…) ─────────────────────────────
  agentHooksList: () => call<AgentHookStatus[]>("agent_hooks_list"),
  /** Diff to show before anything is written. `install: false` previews removal. */
  agentHooksPreview: (agent: string, install: boolean) =>
    callOrThrow<AgentHookPlan>("agent_hooks_preview", { agent, install }),
  /**
   * Writes the agent's config — only ever after an explicit click, and only when
   * it still matches the preview the user looked at. Returns the backups taken.
   */
  agentHooksApply: (agent: string, install: boolean, fingerprint: string) =>
    callOrThrow<string>("agent_hooks_apply", { agent, install, fingerprint }),
  // ── Plan usage: the status line relay, installed apart from the hooks ──────
  /** Diff of the status line change. `install: false` previews taking the relay out. */
  statusLinePreview: (install: boolean) => callOrThrow<HookPreview>("status_line_preview", { install }),
  /** Same rules as hooksApply: an explicit click, and only for the diff that was shown. */
  statusLineApply: (install: boolean, fingerprint: string) =>
    callOrThrow<string>("status_line_apply", { install, fingerprint }),
  /**
   * Asks the Codex CLI (`codex app-server`) for its plan limits, as Codex's
   * /status does. The raw `account/rateLimits/read` result, or null when Codex
   * is missing, not signed in or slow (15 s).
   */
  codexPlanUsage: () => call<unknown>("codex_plan_usage"),

  relayExternalApprovalStatus:()=>callOrThrow<import('./relay-approvals').RelayApprovalStatus>('relay_external_approval_status'),
  relayExternalApprovalSet:(request:{mode:import('./relay-approvals').RelayApprovalMode;revision:string})=>callOrThrow<import('./relay-approvals').RelayApprovalStatus>('relay_external_approval_set',{request}),
  approvalDecision: (requestId: string, decision: "allow" | "deny") =>
    callOrThrow<boolean>("approval_decision", { requestId, decision }),
  approvalPresented:({requestId,sessionId,petId}:import('./approval-presentation').ApprovalContext)=>callOrThrow<boolean>('approval_presented',{requestId,sessionId,petId}),
  prepareApprovalSurface:async(rect:{x:number;y:number;w:number;h:number},current:()=>boolean)=>{
    if(!current())return false;
    await callOrThrow<void>('set_island_rect',{x:rect.x,y:rect.y,width:rect.w,height:rect.h});
    if(!current())return false;
    await callOrThrow<void>('set_collapsed',{collapsed:false});
    return current();
  },
  approvalHandoff:(requestId:string)=>callOrThrow<void>('approval_decline',{requestId}),
  /** "The card is up" — until this lands the relay only waits a moment. */
  approvalAck: (requestId: string) => call<void>("approval_ack", { requestId }),
  /** Answers a question Claude Code asked: question text → chosen label. */
  approvalAnswer: (requestId: string, answers: Record<string, string | string[]>) =>
    callOrThrow<boolean>("approval_answer", { requestId, answers }),

  /** "Nobody can act on this" — Claude Code asks in the terminal right away. */
  approvalDecline: (requestId: string) => call<void>("approval_decline", { requestId }),

  // ── Phone link (Coucou for Android) ───────────────────────────────────────
  phoneLinkStatus: () => call<PhoneLinkStatus>("phone_link_status"),
  /** Only ever from the switch in Settings. Throws with the reason if it cannot start. */
  phoneLinkSetEnabled: (enabled: boolean) => callOrThrow<PhoneLinkStatus>("phone_link_set_enabled", { enabled }),
  phoneLinkPairing: () => callOrThrow<PhoneLinkPairing>("phone_link_pairing"),
  /** A new code: the phone that had the old one is disconnected. */
  phoneLinkNewPairing: () => callOrThrow<PhoneLinkPairing>("phone_link_new_pairing"),
  phoneDetailsStatus: () => call<PhoneDetailsStatus>("phone_details_status"),
  /** Off by default; only from the switch in Settings. Phones are reconnected so they learn about it. */
  phoneDetailsSetEnabled: (enabled: boolean) => callOrThrow<PhoneDetailsStatus>("phone_details_set_enabled", { enabled }),
  phoneAnswersStatus: () => call<PhoneAnswersStatus>("phone_answers_status"),
  /** Off by default; only from the switch in Settings. Phones are reconnected so they learn about it. */
  phoneAnswersSetEnabled: (enabled: boolean) => callOrThrow<PhoneAnswersStatus>("phone_answers_set_enabled", { enabled }),
  phoneDiffsStatus: () => call<PhoneDiffsStatus>("phone_diffs_status"),
  /** Off by default; only from the switch in Settings. Phones are reconnected so they learn about it. */
  phoneDiffsSetEnabled: (enabled: boolean) => callOrThrow<PhoneDiffsStatus>("phone_diffs_set_enabled", { enabled }),
  /** The answer to a phone's request for a file's diff (the "phone-link-getdiff" event); `conn` says which phone. */
  phoneLinkSendDiff: (conn: number, diff: PhoneLinkDiff) => call<void>("phone_link_send_diff", { conn, diff }),
  phoneServicesStatus: () => call<PhoneServicesStatus>("phone_services_status"),
  /** None by default; only from the ticks in Settings. Phones are reconnected so they learn about it. */
  phoneServicesSet: (services: string[]) => callOrThrow<PhoneServicesStatus>("phone_services_set", { services }),
  phoneLinkPublishServices: (services: PhoneLinkService[]) => call<void>("phone_link_publish_services", { services }),
  phoneRelayStatus: () => call<PhoneRelayStatus>("phone_relay_status"),
  /** Off by default; needs a valid wss:// address and a stored access key. */
  phoneRelaySet: (enabled: boolean, url: string) => callOrThrow<PhoneRelayStatus>("phone_relay_set", { enabled, url }),
  /** Write-only: stored in the OS keystore, never sent back. An empty value clears it. */
  phoneRelaySetAccess: (access: string) => callOrThrow<PhoneRelayStatus>("phone_relay_set_access", { access }),
  phoneUsageStatus: () => call<PhoneUsageStatus>("phone_usage_status"),
  /** Off by default; only from the switch in Settings. Phones are reconnected so they learn about it. */
  phoneUsageSetEnabled: (enabled: boolean) => callOrThrow<PhoneUsageStatus>("phone_usage_set_enabled", { enabled }),
  phoneLinkPublishUsage: (usage: PhoneLinkUsage) => call<void>("phone_link_publish_usage", { usage }),
  phoneChatStatus: () => call<PhoneChatStatus>("phone_chat_status"),
  /** Off by default; only from the switch in Settings. Phones are reconnected so they learn about it. */
  phoneChatSetEnabled: (enabled: boolean) => callOrThrow<PhoneChatStatus>("phone_chat_set_enabled", { enabled }),
  phoneChatSetModels: (models: string[]) => callOrThrow<PhoneChatStatus>("phone_chat_set_models", { models }),
  phoneLinkPublish: (sessions: PhoneLinkSession[], approval: PhoneLinkApproval | null) =>
    call<void>("phone_link_publish", { sessions, approval }),
  phoneLinkPublishQuestion: (question: PhoneLinkQuestion | null) => call<void>("phone_link_publish_question", { question }),
  /** What Mochi wears ("auto" or an outfit), for phones that asked for it. */
  phoneLinkPublishPrefs: (outfit: string) => call<void>("phone_link_publish_prefs", { outfit }),

  // ── Chat, files, secrets ──────────────────────────────────────────────────
  /** One chat turn. The API key and any file bytes never leave Rust. */
  chatSend: (query: string, context: ChatContext | null, provider?: string,conversationId?:string,selectedModel?:string,documentSequence?:number,memoryDecision?:PetMemoryDecision) =>
    callOrThrow<{ text: string }>("chat_send", { query, context:structuredClone(context), provider,...(conversationId===undefined?{}:{conversationId}),...(selectedModel===undefined?{}:{selectedModel}),...(documentSequence===undefined?{}:{documentSequence}),...(memoryDecision?{memoryDecision:{token:memoryDecision.token,consent:memoryDecision.consent}}:{}) }),
  chatReset: (provider?: string) => callOrThrow<ConversationView|null>("chat_reset", {provider}),
  chatCancel: (provider?:string,conversationId?:string,turn?:number) => callOrThrow<boolean>("chat_cancel",{provider,...(conversationId===undefined?{}:{conversationId}),...(turn===undefined?{}:{turn})}),
  chatHistory: (provider:string) => call<{role:"user"|"assistant";content:string}[]>("chat_history",{provider}),
  chatConversation:(provider:string)=>callOrThrow<ConversationView>("chat_conversation",{provider}),
  chatReadConversation:(provider:string,id:string)=>callOrThrow<ConversationView>("chat_read_conversation",{provider,id}),
  chatConversations:(provider:string,query="")=>callOrThrow<ConversationSummary[]>("chat_conversations",{provider,query}),
  chatPurgeReview:(provider:string,conversationId:string)=>callOrThrow<ConversationPurgeReview>('chat_purge_review',{provider,conversationId}),
  chatPurgeCommit:(provider:string,conversationId:string,reviewId:string,consent:'deleteLocalConversation'|'deleteRecoveryCopy'|'restoreRecoveryConversation')=>callOrThrow<ConversationPurgeOutcome>('chat_purge_commit',{provider,conversationId,reviewId,consent}),
  chatPurgeCancel:(provider:string,conversationId:string,reviewId:string)=>callOrThrow<void>('chat_purge_cancel',{provider,conversationId,reviewId}),
  chatPurgeRecoveries:()=>callOrThrow<ConversationPurgeRecovery[]>('chat_purge_recoveries'),
  chatPurgeRecoveryReview:(recoveryId:string)=>callOrThrow<ConversationPurgeReview>('chat_purge_recovery_review',{recoveryId}),
  sessionChatList:(petId:string)=>callOrThrow<import('./session-chat').SessionSummary[]>('session_chat_list',{petId}),
  petObservability:()=>callOrThrow<import('./pet-observability-contract').PetObservabilitySnapshot>('pet_observability'),
  petSessionLookup:(request:import('./session-lookup-contract').SessionLookupRequest)=>callOrThrow<import('./session-lookup-contract').SessionLookupResult>('pet_session_lookup',{request:{provider:request.provider,nativeSessionId:request.nativeSessionId}}),
  petCachedUsage:()=>invoke<import('./pet-cached-usage-contract').PetCachedUsage>('pet_cached_usage'),
  petStationHistory:(force=false)=>invoke<import('./station-history-contract').StationHistorySnapshot>('pet_station_history',{force}),
  petUsage:(provider:string,conversationId?:string)=>callOrThrow<import('./pet-usage').PetUsage|null>('pet_usage',{provider,...(conversationId?{conversationId}:{})}),
  chatRestoreConversation:(provider:string,id:string)=>callOrThrow<ConversationView>("chat_restore_conversation",{provider,id}),
  assistantTasks: () => call<AssistantTask[]>("assistant_tasks"),
  voiceLearnAlias: (provider:string) => callOrThrow<string>("voice_learn_alias",{provider}),
  voiceSessionToggle: () => callOrThrow<void>("voice_session_toggle"),
  petChatOpen:async(pet:string,session?:{id:string;backend:string;title:string;cwd:string;steps:string[];finalLine:string},showSessions=false,target?:import('./session-chat').SessionTarget,requestId?:string)=>{
    const sourceContext=cardPresentation();
    const shown=await callOrThrow<boolean>("pet_chat_open",{request:{pet,session,showSessions,...(target?{target:{...target}}:{}),...(requestId?{requestId}:{})}});
    if(shown!==true)return false;
    await emitToWindow('island','pet-chat-presented',{pet,...(target?{target:{...target}}:{}),...(requestId?{requestId}:{}),...(sourceContext?{sourceContext}:{})});
    return true;
  },
  petAccountUsage:()=>callOrThrow<import('./account-usage-contract').CodexAccountUsage>('pet_account_usage',{}),
  petAccountUsageRefresh:()=>callOrThrow<import('./account-usage-contract').CodexAccountUsage>('pet_account_usage_refresh',{}),
  petControlNavigationContext:()=>callOrThrow<import('./control-contract').ControlContext>('pet_control_navigation_context',{}),
  petControlNavigate:(request:import('./control-contract').NavigationRequest)=>callOrThrow<import('./control-contract').NavigationReceipt>('pet_control_navigate',{request}),
  petSystemNetworkRead:(request:import('./control-contract').NetworkReadRequest)=>callOrThrow<import('./control-contract').NetworkTicket>('pet_system_network_read',{request}),
  petSystemNetworkView:(request:import('./control-contract').ContextPrecondition)=>callOrThrow<import('./control-contract').NetworkView>('pet_system_network_view',{request}),
  petSystemNetworkScan:(request:import('./control-contract').NetworkScanRequest)=>callOrThrow<import('./control-contract').NetworkTicket>('pet_system_network_scan',{request}),
  petSystemNetworkEnableWifi:(request:import('./control-contract').NetworkEnableWifiRequest)=>callOrThrow<import('./control-contract').NetworkTicket>('pet_system_network_enable_wifi',{request}),
  petSystemNetworkTicket:(request:import('./control-contract').NetworkTicketRequest)=>callOrThrow<import('./control-contract').NetworkTicket>('pet_system_network_ticket',{request}),
  petSystemNetworkCancel:(request:import('./control-contract').NetworkTicketRequest)=>callOrThrow<import('./control-contract').NetworkTicket>('pet_system_network_cancel',{request}),
  petControlContext:()=>call<{pet?:string;section?:string}>("pet_control_context"),
  petControlInput:(rect:{x:number;y:number;w:number;h:number},expanded:boolean,keyboard=false)=>call<void>("pet_control_input",{rect,expanded,keyboard}),
  petControlConfigure:()=>call<void>("pet_control_configure"),
  petSystemPanel:(panel:import('./system-center').SystemPanel)=>callOrThrow<void>('pet_system_panel',{panel}),
  petAudioSnapshot:()=>callOrThrow<import('./system-audio').AudioSnapshot>('pet_audio_snapshot'),
  petAudioControl:(request:import('./system-audio').AudioRequest)=>callOrThrow<import('./system-audio').AudioReceipt>('pet_audio_control',{request}),
  petHostInventory:(petId:string)=>callOrThrow<import('./system-inventory').SystemHostInventory>('pet_host_inventory',{petId}),
  petHealthSnapshot:(petId:string)=>callOrThrow<import('./system-health').SystemHealthSnapshot>('pet_health_snapshot',{petId}),
  petHealthBaseline:(petId:string)=>callOrThrow<import('./system-health').SystemHealthBaseline>('pet_health_baseline',{petId}),
  petSoundSnapshot:(petId:string)=>callOrThrow<import('./system-audio').SystemAudioSnapshot>('pet_sound_snapshot',{petId}),
  petSystemAccessSet:(petId:string,expectedRevision:string,grants:import('../mochi/pet-preferences').SystemGrant[])=>callOrThrow<PetDocument>('pet_system_access_set',{request:{petId,expectedRevision,grants}}),
  petSystemAudioSnapshot:()=>callOrThrow<import('./system-audio').SystemAudioSnapshot>('pet_system_audio_snapshot'),
  petSystemAudioControl:(request:import('./system-audio').AudioRequest)=>callOrThrow<import('./system-audio').AudioReceipt<import('./system-audio').SystemAudioSnapshot>>('pet_system_audio_control',{request}),
  petSystemStatus:()=>callOrThrow<import('./system-center').SystemCenterStatus>('pet_system_status'),
  petSystemTray:(id:string,action:'activate'|'menu')=>callOrThrow<void>('pet_system_tray',{id,action}),
  petControlOpen:(pet?:string)=>callOrThrow<void>("pet_control_open",{pet}),
  petPanelOpen:(x:number,y:number,pet?:string,section?:string)=>callOrThrow<void>("pet_panel_open",{x,y,pet,section}),
  petPanelHide:()=>callOrThrow<void>("pet_panel_hide"),
  voiceSessionStart: (provider:string) => callOrThrow<void>("voice_session_start",{provider}),
  assistantSetWorkspace:(provider:string,path:string)=>callOrThrow<string>("assistant_set_workspace",{provider,path}),
  assistantCancelTask: (id:string) => callOrThrow<boolean>("assistant_cancel_task",{id}),
  assistantResumeTask: (id:string,cwd?:string) => callOrThrow<{text:string}>("assistant_resume_task",{id,...(cwd===undefined?{}:{cwd})}),
  /**
   * The models a provider offers, for the picker in the chat view. Rust asks
   * the provider only when it has a key (or a server address).
   */
  chatModels: (provider: string) => callOrThrow<ModelInfo[]>("chat_models", { provider }),
  /** Settings → Local models → Connect: does the server answer, and with which models? */
  /** The custom server's key, bound to the address it is entered for. */
  localSetKey: (url: string, key: string,provider?:'lmstudio') => provider!==undefined&&provider!=='lmstudio'?Promise.reject(Error('Servidor de credenciais desconhecido.')):callOrThrow<void>("local_set_key", { url, key,...(provider==='lmstudio'?{provider}:{}) }),

  localConnect: (provider: "ollama" | "lmstudio" | "custom", url: string) =>
    callOrThrow<LocalServer>("local_connect", { provider, url }),
  /** Copies a dropped file into the inbox. */
  ingestFile: (path: string,provider?:string,conversationId?:string,dropId?:string) => callOrThrow<DroppedFile>("ingest_file", { path,provider,...(conversationId===undefined?{}:{conversationId}),...(dropId===undefined?{}:{dropId}) }),
  petDocumentCancelDrop:(provider:string,dropId:string)=>callOrThrow<void>('pet_document_cancel_drop',{provider,dropId}),
  petDocumentContext:(provider:string,conversationId:string)=>callOrThrow<DroppedFile|null>('pet_document_context',{provider,conversationId}),
  petDocumentContexts:(provider:string,conversationId:string)=>callOrThrow<PetAttachment[]>('pet_document_contexts',{provider,conversationId}).then(files=>captureAttachments(provider,conversationId,files)),
  petDocumentIngestDrop:(provider:string,conversationId:string,dropId:string)=>callOrThrow<DocumentBatch>('pet_document_ingest_drop',{provider,conversationId,dropId}),
  petDocumentRemove:(provider:string,conversationId:string,attachmentId:string)=>callOrThrow<void>('pet_document_remove',{provider,conversationId,attachmentId}),
  petDocumentInventory:(provider:string,conversationId:string)=>callOrThrow<DocumentInventory>('pet_document_inventory',{provider,conversationId}),
  petDocumentReview:(provider:string,conversationId:string,attachmentIds:string[],expectedRevision:string)=>callOrThrow<DocumentReview>('pet_document_review',{provider,conversationId,attachmentIds,expectedRevision}),
  petDocumentRenew:(provider:string,conversationId:string,reviewId:string)=>callOrThrow<PetAttachment[]>('pet_document_renew',{provider,conversationId,reviewId,consent:'readSelectedCopies'}).then(files=>captureAttachments(provider,conversationId,files)),
  petDocumentReviewCancel:(provider:string,conversationId:string,reviewId:string)=>callOrThrow<void>('pet_document_review_cancel',{provider,conversationId,reviewId}),
  petDocumentRetentionInventory:()=>callOrThrow<DocumentRetentionInventory>('pet_document_retention_inventory'),
  petDocumentPurgeReview:(provider:string,conversationId:string,attachmentIds:string[],expectedRevision:string)=>callOrThrow<DocumentPurgeReview>('pet_document_purge_review',{provider,conversationId,attachmentIds,expectedRevision}),
  petDocumentPurgeRecoveryReview:(recoveryId:string)=>callOrThrow<DocumentPurgeReview>('pet_document_purge_recovery_review',{recoveryId}),
  petDocumentPurgeCommit:(provider:string,conversationId:string,reviewId:string,consent:DocumentPurgeConsent)=>callOrThrow<DocumentPurgeReceipt>('pet_document_purge_commit',{provider,conversationId,reviewId,consent}),
  petDocumentPurgeCancel:(provider:string,conversationId:string,reviewId:string)=>callOrThrow<void>('pet_document_purge_cancel',{provider,conversationId,reviewId}),
  petDocumentClear:(provider:string,conversationId:string,sequence:number)=>callOrThrow<void>('pet_document_clear',{provider,conversationId,sequence}),
  petMailDraft:(provider:string,path:string,recipient:string,subject:string,body:string)=>callOrThrow<string>("pet_mail_draft",{provider,path,recipient,subject,body}),
  /** Only ever tells you whether a key exists — never its value. */
  secretPresent: (key: string) => call<boolean>("secret_present", { key }),
  secretSet: (key: string, value: string) => callOrThrow<void>("secret_set", { key, value }),
  secretClear: (key: string) => callOrThrow<void>("secret_clear", { key }),

  // ── Integrations ──────────────────────────────────────────────────────────
  refreshIntegration: (id: string) => call<void>("refresh_integration", { id }),
  /** The GitHub card is on screen: refetch that part if it is stale. */
  githubRefresh: (section: "pulse" | "activity") => call<void>("github_refresh", { section }),
  /** Opens the configured n8n instance in the browser. */
  openN8n: () => call<void>("open_n8n"),

  /** Tray → Pause. Stops the integration pollers, not just the island. */
  setPaused: (paused: boolean) => call<void>("set_paused", { paused }),

  // ── Global shortcuts ──────────────────────────────────────────────────────
  /** How each global shortcut went when Rust last registered them. */
  shortcutsStatus: () => call<ShortcutsReport>("shortcuts_status"),
  /** Lets go of every global shortcut while Settings records a new one. */
  shortcutsSuspend: (suspended: boolean) => call<void>("shortcuts_suspend", { suspended }),

  // ── Weekly recap ──────────────────────────────────────────────────────────
  /** Turns and decisions from `since` (Unix seconds) on, plus the recap prefs. */
  recapHistory: (since: number) => call<RecapHistory>("recap_history", { since: Math.floor(since) }),
  recapPrefs: () => callOrThrow<RecapPrefs>("recap_prefs"),
  recapSetEnabled: (enabled: boolean) => callOrThrow<RecapPersistenceReceipt>("recap_set_enabled", { enabled }),
  recapSetHideProjects: (hide: boolean) => callOrThrow<RecapPersistenceReceipt>("recap_set_hide_projects", { hide }),
  /** `week` is the Monday (YYYY-MM-DD) the recap opened on its own for. */
  recapMarkShown: (week: string) => callOrThrow<RecapPersistenceReceipt>("recap_mark_shown", { week }),
  recapClear: () => callOrThrow<RecapPersistenceReceipt>("recap_clear"),
  /** Writes the PNG (a data URL) into Pictures or Downloads; returns its path. */
  recapSavePng: (data: string, week: string) => callOrThrow<string>("recap_save_png", { data, week }),
  /** Opens the folder of the image saved last. */
  recapRevealSaved: () => call<void>("recap_reveal_saved"),

  // ── Mochi on the desktop (src-tauri/src/desktop.rs) ───────────────────────
  companionsConfigure: (enabled: boolean, count: number) => call<{ width: number; height: number; top: number; band: { height: number; lanes: [number, number][] } | null }>("companions_configure", { enabled, count }),
  companionsAnchor: (x: number, y: number) => call<void>("companions_anchor", {x,y}),
  companionsInput: (rects: number[][]) => call<void>("companions_input", { rects }),
  petConfiguration: () => callOrThrow<PetDocument>("pet_configuration"),
  petCapabilities:(petId?:string)=>callOrThrow<PetCapabilitiesSnapshot>('pet_capabilities_snapshot',{petId}),
  petMemoryContextPrepare:(input:PetMemoryPrepare)=>callOrThrow<PetMemoryTicket>('pet_memory_context_prepare',{provider:input.provider,conversationId:input.conversationId,query:input.query,selected:input.selected.map(({id,revision})=>({id,revision})),context:structuredClone(input.context),...(input.selectedModel===undefined?{}:{selectedModel:input.selectedModel}),...(input.documentSequence===undefined?{}:{documentSequence:input.documentSequence})}),
  petMemoryContextCancel:(token:string)=>callOrThrow<void>('pet_memory_context_cancel',{token}),
  petSourceConfiguration:()=>callOrThrow<PetSourceDocument>('pet_source_configuration'),
  petSourceConfigurationSave:(config:PetSourceRegistry,expectedRevision:string,consent:'metadataOnly'|'persistentCalendarRead')=>callOrThrow<PetSourceDocument>('pet_source_configuration_save',{config:structuredClone(config),expectedRevision,consent}),
  petMemoryList:(request:PetMemoryQuery)=>callOrThrow<PetMemoryPage>('pet_memory_list',{request:structuredClone(request)}),
  petMemoryMutate:(owner:PetMemoryOwner,expectedRevision:string,mutation:PetMemoryMutation)=>callOrThrow<PetMemoryReceipt>('pet_memory_mutate',{owner:structuredClone(owner),expectedRevision,mutation:structuredClone(mutation)}),
  petRuntimeRuleProbeRecoveryPrepare:(input:{petId:string;ruleId:string;revision:number})=>callOrThrow<RuntimeRuleRecoveryTicket>('pet_runtime_rule_probe_recovery_prepare',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision}}),
  petRuntimeRuleProbeRecoveryConfirm:(input:{petId:string;ruleId:string;revision:number;token:string;approve:boolean;consent?:RuntimeRuleRecoveryConsent})=>callOrThrow<RuntimeRuleAuthorityConfirmation>('pet_runtime_rule_probe_recovery_confirm',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,token:input.token,approve:input.approve,consent:input.consent??null}}),
  petRuntimeRuleIntentRetirementPrepare:(input:{petId:string;ruleId:string;revision:number})=>callOrThrow<RuntimeRuleIntentRetirementTicket>('pet_runtime_rule_intent_retirement_prepare',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision}}),
  petRuntimeRuleIntentRetirementConfirm:(input:{petId:string;ruleId:string;revision:number;token:string;approve:boolean;consent?:RuntimeRuleIntentRetirementConsent})=>callOrThrow<RuntimeRuleAuthorityConfirmation>('pet_runtime_rule_intent_retirement_confirm',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,token:input.token,approve:input.approve,consent:input.consent??null}}),
  petRuntimeRuleRevisionPrepare:(input:{petId:string;ruleId:string;revision:number;mode:RuntimeRuleRevisionMode;automatic?:boolean})=>callOrThrow<RuntimeRuleRevisionTicket>('pet_runtime_rule_revision_prepare',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,mode:input.mode,automatic:input.automatic??false}}),
  petRuntimeRuleRevisionConfirm:(input:{petId:string;ruleId:string;revision:number;mode:RuntimeRuleRevisionMode;token:string;approve:boolean;consent?:RuntimeRuleRevisionConsent})=>callOrThrow<RuntimeRuleAuthorityConfirmation>('pet_runtime_rule_revision_confirm',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,mode:input.mode,token:input.token,approve:input.approve,consent:input.consent??null}}),
  petRuntimeRuleSuccessionPrepare:(input:{petId:string;ruleId:string;revision:number})=>callOrThrow<RuntimeRuleSuccessionTicket>('pet_runtime_rule_succession_prepare',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision}}),
  petRuntimeRuleSuccessionConfirm:(input:{petId:string;ruleId:string;revision:number;token:string;approve:boolean;consent?:RuntimeRuleSuccessionConsent})=>callOrThrow<RuntimeRuleAuthorityConfirmation>('pet_runtime_rule_succession_confirm',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,token:input.token,approve:input.approve,consent:input.consent??null}}),
  petRuntimeRuleAuthorityPrepare:(input:{petId:string;ruleId:string;revision:number;automatic?:boolean})=>callOrThrow<RuntimeRuleAuthorityTicket>('pet_runtime_rule_authority_prepare',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,...(input.automatic===true?{automatic:true}:{})}}),
  petRuntimeRuleAuthorityConfirm:(input:{petId:string;ruleId:string;revision:number;token:string;approve:boolean;consent?:RuntimeRuleScope})=>callOrThrow<RuntimeRuleAuthorityConfirmation>('pet_runtime_rule_authority_confirm',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision,token:input.token,approve:input.approve,consent:input.consent??null}}),
  petRuntimeRuleAuthorityStatus:(input:{petId:string;ruleId:string;revision:number})=>callOrThrow<RuntimeRuleAuthorityStatus>('pet_runtime_rule_authority_status',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision}}),
  petRuntimeRuleOccurrenceAdmit:(input:{petId:string;ruleId:string;revision:number})=>callOrThrow<{goalId:string;ruleId:string;occurrenceId:string;queued:boolean;existingAdmission:boolean}>('pet_runtime_rule_occurrence_admit',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision}}),
  petRuntimeRuleAuthorityRevoke:(input:{petId:string;ruleId:string;decisionId?:string})=>callOrThrow<RuntimeRuleAuthorityStatus>('pet_runtime_rule_authority_revoke',{input:{petId:input.petId,ruleId:input.ruleId,decisionId:input.decisionId??null}}),
  petRuntimeRules:(petId:string)=>callOrThrow<RuntimeRuleDocument>('pet_runtime_rules',{petId}),
  petRuntimeRuleSave:(input:{petId:string;expectedRevision:string;requestId:string;ruleId?:string;definition:RuntimeRuleDefinition})=>callOrThrow<RuntimeRuleDocument>('pet_runtime_rule_save',{input:{petId:input.petId,expectedRevision:input.expectedRevision,requestId:input.requestId,ruleId:input.ruleId??null,definition:structuredClone(input.definition)}}),
  petRuntimeRulePreview:(input:{petId:string;ruleId:string;revision:number})=>callOrThrow<RuntimeRulePreview>('pet_runtime_rule_preview',{input:{petId:input.petId,ruleId:input.ruleId,revision:input.revision}}),
  petRuntimeCreate:(input:RuntimeCreateInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_create',{input:{petId:input.petId,requestId:input.requestId,intent:input.intent,criteria:[...input.criteria],conversationId:input.conversationId??null}}),
  petRuntimeCreateDeterministic:(input:RuntimeCreateInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_create_deterministic',{input:{petId:input.petId,requestId:input.requestId,intent:input.intent,criteria:[...input.criteria],conversationId:input.conversationId??null}}),
  petRuntimeGoalPending:()=>callOrThrow<AgentRequest[]>('pet_runtime_goal_pending'),
  petRuntimeGoalAnswer:(id:string,approve:boolean,target:GoalRecipeTarget)=>callOrThrow<void>('pet_runtime_goal_answer',{input:{id,approve,target:structuredClone(target)}}),
  petRuntimeRefreshReadiness:(petId:string,goalId:string)=>callOrThrow<PetCapabilityDescriptor>('pet_runtime_refresh_readiness',{petId,goalId}),
  petRuntimeList:(petId?:string)=>callOrThrow<RuntimeListing>('pet_runtime_list',{petId}),
  petRuntimePrepareContext:(input:RuntimePrepareContextInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_prepare_context',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,mode:input.mode,...(input.selectedMemory?{selectedMemory:input.selectedMemory.map(({id,revision})=>({id,revision}))}:{}),...(input.includeReceivedRequest===true?{includeReceivedRequest:true}:{}),...(input.selectedCalendars?{selectedCalendars:{...(input.selectedCalendars.sourceScope===undefined?{}:{sourceScope:input.selectedCalendars.sourceScope}),selected:input.selectedCalendars.selected.map(({id,revision})=>({id,revision})),query:{...input.selectedCalendars.query}}}:{})}}),
  petRuntimeContextRead:(petId:string,goalId:string)=>callOrThrow<RuntimeContextView>('pet_runtime_context_read',{petId,goalId}),
  petRuntimeClaim:(input:RuntimeClaimInput)=>callOrThrow<RuntimeClaimResult>('pet_runtime_claim',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId}}),
  petRuntimeClaimStatus:(petId:string,goalId:string)=>callOrThrow<RuntimeLeaseStatus>('pet_runtime_claim_status',{petId,goalId}),
  petRuntimeReleaseClaim:(petId:string,goalId:string,leaseId:string)=>callOrThrow<RuntimeReceipt>('pet_runtime_release_claim',{input:{petId,goalId,leaseId}}),
  petRuntimeRespondOnce:(input:RuntimeRespondInput)=>callOrThrow<RuntimeRespondResult>('pet_runtime_respond_once',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,requestId:input.requestId}}),
  petRuntimePendingMessages:(input:RuntimeMessageIntentsInput)=>callOrThrow<RuntimeMessageIntentInventory>('pet_runtime_pending_messages',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId}}),
  petRuntimePrepareMessageRetirement:(input:RuntimeMessageRetireInput)=>callOrThrow<RuntimeMessageRetirePrepared>('pet_runtime_prepare_message_retirement',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,decisionId:input.decisionId,messageId:input.messageId,requestId:input.requestId}}),
  petRuntimeConfirmMessageRetirement:(input:RuntimeMessageRetireConfirmInput)=>callOrThrow<RuntimeMessageRetireConfirmed>('pet_runtime_confirm_message_retirement',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,decisionId:input.decisionId,token:input.token,approve:input.approve}}),
  petRuntimePrepareMessage:(input:RuntimeStepPrepareInput)=>callOrThrow<RuntimeMessageTicket>('pet_runtime_prepare_message',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,planId:input.planId,stepId:input.stepId,requestId:input.requestId}}),
  petRuntimeConfirmMessage:(input:RuntimeStepConfirmInput)=>callOrThrow<RuntimeMessageConfirmResult>('pet_runtime_confirm_message',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,token:input.token,approve:input.approve}}),
  petRuntimeMessageResult:(input:RuntimeStepReadInput)=>callOrThrow<RuntimeMessageReceipt|null>('pet_runtime_message_result',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,planId:input.planId,stepId:input.stepId}}),
  petRuntimeMessageInbox:(input:RuntimeMessageInboxInput)=>callOrThrow<RuntimeMessageInbox>('pet_runtime_message_inbox',{input:{petId:input.petId,after:input.after??null,limit:input.limit,...(input.includeHistorical?{includeHistorical:true}:{}),...(input.includeReplySources?{includeReplySources:true}:{}),...(input.includeFundingSources?{includeFundingSources:true}:{})}}),
  petRuntimePrepareStep:(input:RuntimeStepPrepareInput)=>callOrThrow<RuntimeStepTicket>('pet_runtime_prepare_step',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,planId:input.planId,stepId:input.stepId,requestId:input.requestId}}),
  petRuntimeConfirmStep:(input:RuntimeStepConfirmInput)=>callOrThrow<RuntimeStepConfirmResult>('pet_runtime_confirm_step',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,token:input.token,approve:input.approve}}),
  petRuntimeRunStep:(input:RuntimeStepRunInput)=>callOrThrow<RuntimeStepView>('pet_runtime_run_step',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,grantId:input.grantId,planId:input.planId,stepId:input.stepId}}),
  petRuntimeStepResult:(input:RuntimeStepReadInput)=>callOrThrow<RuntimeStepView|null>('pet_runtime_step_result',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,planId:input.planId,stepId:input.stepId}}),
  petRuntimeRecoverStep:(input:RuntimeStepRecoverInput)=>callOrThrow<RuntimeStepView>('pet_runtime_recover_step',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,planId:input.planId,stepId:input.stepId,revision:input.revision,grantId:input.grantId}}),
  petRuntimePrepareRecipe:(input:RuntimeRecipePrepareInput)=>callOrThrow<RuntimeRecipeTicket>('pet_runtime_prepare_recipe',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,planId:input.planId,requestId:input.requestId,...(input.criteria===undefined?{}:{criteria:input.criteria.map(c=>({criterionIndex:c.criterionIndex,predicate:structuredClone(c.predicate)}))})}}),
  petRuntimeRequestRecipe:(input:RuntimeRecipePrepareInput)=>callOrThrow<RuntimeRecipeConfirmation>('pet_runtime_request_recipe',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,planId:input.planId,requestId:input.requestId,...(input.criteria===undefined?{}:{criteria:input.criteria.map(c=>({criterionIndex:c.criterionIndex,predicate:structuredClone(c.predicate)}))})}}),
  petRuntimeConfirmRecipe:(input:RuntimeRecipeConfirmInput)=>callOrThrow<RuntimeRecipeConfirmation>('pet_runtime_confirm_recipe',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,token:input.token,approve:input.approve}}),
  petRuntimeRunRecipe:(input:RuntimeRecipeRunInput)=>callOrThrow<RuntimeRecipeResult>('pet_runtime_run_recipe',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,recipeId:input.recipeId}}),
  petRuntimeRecipeProgress:(input:RuntimeRecipeReadInput)=>callOrThrow<RuntimeRecipeProgress|null>('pet_runtime_recipe_progress',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,planId:input.planId}}),
  petRuntimeCompleteRecipe:(input:RuntimeRecipeCompleteInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_complete_recipe',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,planId:input.planId,recipeId:input.recipeId,revision:input.revision}}),
  petRuntimeResult:(input:RuntimeResultInput)=>callOrThrow<RuntimeResult>('pet_runtime_result',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,requestId:input.requestId}}),
  petRuntimeAdmissionStatus:(petId:string,goalId:string)=>callOrThrow<RuntimeProvenanceStatus>('pet_runtime_admission_status',{petId,goalId}),
  petRuntimePreparePlanning:(input:RuntimePlanningInput)=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_planning',{input:{...(input.catalog?{catalog:input.catalog}:{}),petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),limits:{maxRounds:input.limits.maxRounds,maxSteps:input.limits.maxSteps,maxOutputBytes:input.limits.maxOutputBytes,noProgressLimit:input.limits.noProgressLimit},...(input.start==='whenAvailable'?{start:'whenAvailable'}:{})}}),
  petRuntimeStartDriver:(input:RuntimeDriverInput)=>callOrThrow<RuntimeDriverStatus>('pet_runtime_start_driver',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId}}),
  petRuntimeDriverStatus:(input:RuntimeDriverInput)=>callOrThrow<RuntimeDriverStatus>('pet_runtime_driver_status',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId}}),
  petRuntimePrepareContinuation:(input:RuntimeContinuationInput)=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_continuation',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),limits:input.limits?{maxRounds:input.limits.maxRounds,maxSteps:input.limits.maxSteps,maxOutputBytes:input.limits.maxOutputBytes,noProgressLimit:input.limits.noProgressLimit}:null,...(input.reviewUncertainEffects===true?{reviewUncertainEffects:true}:{}),...(input.start==='whenAvailable'?{start:'whenAvailable'}:{})}}),
  petRuntimePrepareAdmission:(input:RuntimeAdmissionInput&{notBeforeAt?:number})=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_admission',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),...(input.start==='whenAvailable'?{start:'whenAvailable'}:{}),...(input.notBeforeAt===undefined?{}:{notBeforeAt:input.notBeforeAt})}}),
  petRuntimePrepareDeterministicContinuation:(input:RuntimeAdmissionInput)=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_deterministic_continuation',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),...(input.start?{start:input.start}:{}),limits:null}}),
  petRuntimePrepareResultDisclosure:(input:RuntimeAdmissionInput)=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_result_disclosure',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),...(input.start==='whenAvailable'?{start:'whenAvailable'}:{})}}),
  petRuntimePrepareSystem:(input:RuntimeAdmissionInput&{criteria:RuntimeCriterionInput[]})=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_system',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),criteria:input.criteria,...(input.start?{start:input.start}:{})}}),
  petRuntimePrepareDeterministic:(input:RuntimeAdmissionInput)=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_deterministic',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget),...(input.start?{start:input.start}:{})}}),
  petRuntimeNativePlanRevisions:(input:{petId:string;goalId:string;revision:string;cursor?:string|null})=>callOrThrow<RuntimeNativePlanInventory>('pet_runtime_native_plan_revisions',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,cursor:input.cursor??null}}),
  petRuntimePrepareNativePlanRevision:(input:{petId:string;goalId:string;revision:string;requestId:string;inputs:RuntimeNativePlanEdit[]})=>callOrThrow<RuntimeNativePlanReview>('pet_runtime_prepare_native_plan_revision',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,inputs:input.inputs}}),
  petRuntimeConfirmNativePlanRevision:(input:{petId:string;goalId:string;revision:string;token:string;approve:boolean})=>callOrThrow<RuntimeNativePlanConfirmed>('pet_runtime_confirm_native_plan_revision',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,token:input.token,approve:input.approve}}),
  petRuntimeAckNativePlanRevision:(input:{petId:string;goalId:string;revision:string;reviewId:string})=>callOrThrow<RuntimeNativePlanReceipt>('pet_runtime_ack_native_plan_revision',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,reviewId:input.reviewId}}),
  petRuntimeAbandonNativePlanRevision:(input:{petId:string;goalId:string;revision:string;reviewId:string})=>callOrThrow<{reference:RuntimeNativePlanReference;withdrawn:true;executionAuthorized:false;budgetRefund:false}>('pet_runtime_abandon_native_plan_revision',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,reviewId:input.reviewId}}),
  petRuntimeCloseDeterministic:(input:{petId:string;goalId:string;leaseId:string;revision:string})=>callOrThrow<{receipt:RuntimeReceipt;confirmed:boolean;executorRecreated:false}>('pet_runtime_close_deterministic',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision}}),
  petRuntimeVerifyDeterministic:(input:{petId:string;goalId:string;leaseId:string;revision:string})=>callOrThrow<RuntimeNativeProgress>('pet_runtime_verify_deterministic',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision}}),
  petRuntimeConfirmAdmission:(input:RuntimeAdmissionConfirm)=>callOrThrow<RuntimeAdmissionResult>('pet_runtime_confirm_admission',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,token:input.token,approve:input.approve,...(input.memoryConsent?{memoryConsent:input.memoryConsent}:{}),...(input.disclosureConsent?{disclosureConsent:input.disclosureConsent}:{})}}),
  petRuntimePeerFeedback:(input:RuntimeDelegationInput)=>callOrThrow<RuntimePeerFeedback>('pet_runtime_peer_feedback',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId}}),
  petRuntimeDelegations:(input:RuntimeDelegationInput)=>callOrThrow<RuntimeDelegationView>('pet_runtime_delegations',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId}}),
  petRuntimePrepareDelegationWithdrawal:(input:RuntimeDelegationInput&{messageId:string})=>callOrThrow<RuntimeDelegationWithdrawalTicket>('pet_runtime_prepare_delegation_withdrawal',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,messageId:input.messageId}}),
  petRuntimeConfirmDelegationWithdrawal:(input:RuntimeDelegationInput&{token:string;approve:boolean})=>callOrThrow<RuntimeDelegationWithdrawalResult>('pet_runtime_confirm_delegation_withdrawal',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,token:input.token,approve:input.approve}}),
  petRuntimePrepareDelegationBranch:(input:RuntimeDelegationInput&{messageId:string;action:'cancel'|'reconcile'})=>callOrThrow<RuntimeDelegationBranchTicket>('pet_runtime_prepare_delegation_branch',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,messageId:input.messageId,action:input.action}}),
  petRuntimeConfirmDelegationBranch:(input:RuntimeDelegationInput&{token:string;approve:boolean})=>callOrThrow<RuntimeDelegationBranchResult>('pet_runtime_confirm_delegation_branch',{input:{petId:input.petId,goalId:input.goalId,decisionId:input.decisionId,token:input.token,approve:input.approve}}),
  petRuntimeWorkReportDraft:(input:RuntimeReceivedWorkInput&{expiresAt:number;includeAttempts:boolean;autonomous?:boolean})=>callOrThrow<RuntimeWorkReportDraft>('pet_runtime_work_report_draft',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},expiresAt:input.expiresAt,includeAttempts:input.includeAttempts,...(input.autonomous?{autonomous:true}:{})}}),
  petRuntimeReceivedWork:(input:RuntimeReceivedWorkInput)=>callOrThrow<RuntimeReceivedWork>('pet_runtime_received_work',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash}}}),
  petRuntimePrepareFundedGoal:(input:RuntimeReceivedGoalPrepareInput)=>callOrThrow<RuntimeReceivedGoalPrepared>('pet_runtime_prepare_funded_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},requestId:input.requestId,intent:input.intent,criteria:[...input.criteria]}}),
  petRuntimeConfirmFundedGoal:(input:RuntimeReceivedGoalConfirmInput)=>callOrThrow<RuntimeReceivedGoalConfirmed>('pet_runtime_confirm_funded_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},token:input.token,approve:input.approve}}),
  petRuntimePrepareRecursiveFundedGoal:(input:RuntimeReceivedGoalPrepareInput)=>callOrThrow<RuntimeReceivedGoalPrepared>('pet_runtime_prepare_recursive_funded_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},requestId:input.requestId,intent:input.intent,criteria:[...input.criteria]}}),
  petRuntimeConfirmRecursiveFundedGoal:(input:RuntimeReceivedGoalConfirmInput)=>callOrThrow<RuntimeReceivedGoalConfirmed>('pet_runtime_confirm_recursive_funded_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},token:input.token,approve:input.approve}}),
  petRuntimePrepareReceivedGoal:(input:RuntimeReceivedGoalPrepareInput)=>callOrThrow<RuntimeReceivedGoalPrepared>('pet_runtime_prepare_received_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},requestId:input.requestId,intent:input.intent,criteria:[...input.criteria]}}),
  petRuntimeConfirmReceivedGoal:(input:RuntimeReceivedGoalConfirmInput)=>callOrThrow<RuntimeReceivedGoalConfirmed>('pet_runtime_confirm_received_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},token:input.token,approve:input.approve}}),
  petRuntimePrepareReceivedResultGoal:(input:RuntimeReceivedGoalPrepareInput)=>callOrThrow<RuntimeReceivedGoalPrepared>('pet_runtime_prepare_received_result_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},requestId:input.requestId,intent:input.intent,criteria:[...input.criteria]}}),
  petRuntimeConfirmReceivedResultGoal:(input:RuntimeReceivedGoalConfirmInput)=>callOrThrow<RuntimeReceivedGoalConfirmed>('pet_runtime_confirm_received_result_goal',{input:{petId:input.petId,source:{decisionId:input.source.decisionId,messageId:input.source.messageId,scopeHash:input.source.scopeHash},token:input.token,approve:input.approve}}),
  petRuntimePrepareRestoredFuture:(input:RuntimeFuturePrepareInput)=>callOrThrow<RuntimeFuturePrepared>('pet_runtime_prepare_restored_future',{input:structuredClone(input)}),
  petRuntimeConfirmRestoredFuture:(input:RuntimeFutureConfirmInput)=>callOrThrow<RuntimeFutureConfirmed>('pet_runtime_confirm_restored_future',{input:structuredClone(input)}),
  petRuntimeHistory:(input:RuntimeHistoryInput)=>callOrThrow<RuntimeHistorySnapshot>('pet_runtime_history',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision}}),
  petRuntimePrepareRetirement:(input:RuntimeRetirementPrepareInput)=>callOrThrow<RuntimeRetirementPrepared>('pet_runtime_prepare_retirement',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,decisionId:input.decisionId,requestId:input.requestId}}),
  petRuntimeConfirmRetirement:(input:RuntimeRetirementConfirmInput)=>callOrThrow<RuntimeRetirementConfirmed>('pet_runtime_confirm_retirement',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,decisionId:input.decisionId,token:input.token,approve:input.approve}}),
  petRuntimePreparePeerResume:(input:RuntimeRecoveryPrepareInput)=>callOrThrow<RuntimeRecoveryPrepared>('pet_runtime_prepare_peer_resume',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId}}),
  petRuntimePrepareRecovery:(input:RuntimeRecoveryPrepareInput)=>callOrThrow<RuntimeRecoveryPrepared>('pet_runtime_prepare_recovery',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId}}),
  petRuntimeConfirmRecovery:(input:RuntimeRecoveryConfirmInput)=>callOrThrow<RuntimeRecoveryConfirmed>('pet_runtime_confirm_recovery',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,token:input.token,approve:input.approve}}),
  petRuntimeRead:(petId:string,goalId:string)=>callOrThrow<RuntimeGoal>('pet_runtime_read',{petId,goalId}),
  petRuntimeMutate:(input:RuntimeMutationInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_mutate',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,action:runtimeDraftAction(input.action)}}),
  petRuntimeImportLegacy:(input:RuntimeImportInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_import_legacy',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,taskId:input.taskId}}),

  petTargetCapabilities:(target:import('./session-chat').SessionTarget)=>callOrThrow<NativeTargetCapabilities>('pet_capabilities_target',{target}),
  petRoster:()=>callOrThrow<string[]>("pet_roster"),
  petConfigurationSave: (config:PetConfiguration,expectedRevision:string) => callOrThrow<PetDocument>("pet_configuration_save",{config,expectedRevision}),
  petInferenceSet: (petId:string,expectedRevision:string,update:InferenceUpdate) => callOrThrow<PetDocument>("pet_inference_set",{request:{petId,expectedRevision,update}}),
  companionsOpen: (pillId: string, x: number, y: number, view?: string, manual=false) => call<void>("companions_open", { pillId, x, y, view, manual }),
  desktopInfo: () => call<DesktopInfo>("desktop_mochi_info"),
  /** Dragged out of the island: (x, y) is the pointer in island-window coordinates. */
  desktopPickUp: (x: number, y: number) => call<boolean>("desktop_mochi_pick_up", { x, y }),
  /** Linux: the pointer moved during that drag (Windows carries him from Rust). */
  desktopCarry: (x: number, y: number) => call<void>("desktop_mochi_carry", { x, y }),
  desktopCarryEnd: (x: number, y: number) => call<void>("desktop_mochi_carry_end", { x, y }),
  /** A drag started on the desktop Mochi; resolves to his top-left corner. */
  desktopDragBegin: () => call<[number, number] | null>("desktop_mochi_drag_begin"),
  /** X11: top-left corner, physical pixels. */
  desktopDragMove: (x: number, y: number) => call<void>("desktop_mochi_drag_move", { x, y }),
  desktopDragEnd: (x: number, y: number) => call<void>("desktop_mochi_drag_end", { x, y }),
  /** From the island to his spot. False: no spot on any connected display.
   *  `anywhere`: the first-visit corner then, rather than staying home. */
  desktopFlyOut: (anywhere = false) => call<boolean>("desktop_mochi_fly_out", { anywhere }),
  /** To the island, then hidden. `forget`: he lives in the island again. */
  desktopFlyHome: (forget: boolean) => call<boolean>("desktop_mochi_fly_home", { forget }),
  /** Asleep, the cursor poll stops. */
  desktopSetAsleep: (asleep: boolean) => call<void>("desktop_mochi_set_asleep", { asleep }),

  // ── Spotify (src-tauri/src/spotify.rs) ────────────────────────────────────
  /** Reads the player again (the card came on screen) and reports it. */
  spotifyPlayItem: (uri: string) => call<boolean>("spotify_play_item",{uri}),
  spotifyRefresh: () => call<SpotifyState | null>("spotify_refresh"),
  /** True when Spotify took it. `value`: seconds for seek, 0/1 for shuffle and repeat, 0…100 for volume. */
  spotifyControl: (action: SpotifyAction, value?: number) =>
    call<boolean>("spotify_control", { action, value: value ?? null }),
  /** Brings Spotify forward or starts it; without it, its download page. */
  spotifyOpen: () => call<boolean>("spotify_open"),
  /** Whether there is a Spotify to launch (Settings). */
  spotifyInstalled: () => call<boolean>("spotify_installed"),
};

export type SpotifyAction = "play" | "playPause" | "next" | "previous" | "seek" | "shuffle" | "repeat" | "volume";

export type ShortcutStatus =
  | "active" | "off" | "inUse" | "duplicate" | "invalid"
  | "typesCharacter" | "unsupported" | "notPorted"
  /** Wayland: handed to the desktop's portal, no answer yet / left out by it. */
  | "pending" | "refused" | "preferencesUnconfirmed";

export interface ShortcutsReport {
  /** `trigger`: on Wayland, the keys the desktop says run it, in its words. */
  actions: { id: string; status: ShortcutStatus; typed?: string; trigger?: string }[];
  /** "wayland" or "no-display" when no key grab can be made. */
  blocked: string | null;
  /** `<executable> --shortcut`: append an action id for a desktop shortcut. */
  command: string;
  /** Wayland: the shortcuts went to the desktop's GlobalShortcuts portal. */
  portal?: "pending" | "active" | null;
}

/** How the desktop Mochi's window works here (platform::DesktopMode). */
export type DesktopMode = "poll" | "window" | "layer" | "off";

export interface DesktopInfo {
  mode: DesktopMode;
  /** He was on the desktop when the app last quit. */
  onDesktop: boolean;
}

/** An event for one window only (island ⇄ desktop Mochi). Never throws. */
export async function emitToWindow(label: string, event: string, payload?: unknown) {
  if (!IS_TAURI) return;
  try {
    await emitTo(label, event, payload);
  } catch (err) {
    console.error(`[coucou] emit ${event} failed`, err);
  }
}

/** Required deliveries must retain their receipt until the transport accepts it. */
export async function emitToWindowOrThrow(label:string,event:string,payload?:unknown){
  if(IS_TAURI)await emitTo(label,event,payload);
}

export interface IntegrationUpdate {
  id: string;
  data: Record<string, unknown>;
  error: string | null;
  event: { success: boolean; label: string; detail: string | null } | null;
}

export type ChatContext =
  | { kind: "file"; name: string; path: string }
  | { kind:"files"; files:Array<{id:string;name:string;path:string;sequence:number}> }
  | { kind: "window"; appName: string; title: string; url?: string };

export interface ModelInfo {
  id: string;
  label: string;
}

export interface LocalServer {
  /** The address as it is stored. */
  url: string;
  models: string[];
  /** The address is this machine. */
  loopback: boolean;
}

export interface DroppedFile {
  id?:string;
  provider?:string;
  conversationId?:string;
  sequence?:number;
  kind:"file"|"folder";
  entries:number;
  name: string;
  path: string;
  size: number;
}

export interface PetAttachment extends DroppedFile {id:string;provider:string;conversationId:string;sequence:number}
/** Passive metadata. A revision or ready capability is never an execution grant. */
export interface PetCapability {
 id:string;support:'supported'|'unsupported'|'unknown';availability:'ready'|'unavailable'|'unknown';reason:string|null;
 source:Array<'bundledManifest'|'configuration'|'providerCatalog'|'runtimeObservation'>;observedAt:number|null;expiresAt:number|null;modelBinding:string|null;
 decision:'none'|'nativeApproval'|'exactCommandApproval'|'scopedGrant';callers:Array<'nativeUser'|'taskRuntime'|'providerModel'>;
 limits:{maxItems:number|null;maxBytes:number|null;maxCalls:number|null;timeoutMs:number|null};limitations:string[];
}
export interface PetCapabilityDescriptor {
 schemaVersion:1;petId:string;kind:'llm'|'service'|'observer';name:string;lifecycle:'active'|'inactive'|'retired';
 binding:{engineBinding?:RuntimeEngineBinding;adapter:string|null;provider:string|null;model:string|null;profile:string|null};revision:string;observedAt:number|null;configurationAvailable:boolean;
 capabilities:PetCapability[];concurrency:{scope:'provider'|null;maxActive:number|null};
}
export interface PetCapabilitiesSnapshot {schemaVersion:1;revision:string;observedAt:number;configurationAvailable:boolean;pets:PetCapabilityDescriptor[]}
export interface NativeTargetCapabilities {
 schemaVersion:1;target:import('./session-chat').SessionTarget;revision:string;
 observedAt:number|null;expiresAt:number|null;busy:boolean|null;turn:string|null;model:string|null;capabilities:PetCapability[];
}
export interface DocumentBatch {attachments:PetAttachment[];failures:Array<{index:number;name:string;error:string}>;error:string|null}
export interface DocumentRecord {id:string;name:string;kind:'file'|'folder';size:number;entries:number;selected:boolean;authorized:boolean;purging?:boolean}
export interface DocumentInventory {revision:string;records:DocumentRecord[]}
export interface DocumentReview {reviewId:string;items:Array<Pick<DocumentRecord,'id'|'name'|'kind'|'size'|'entries'>>;expiresInSeconds:number}
export type DocumentPurgePhase='Prepared'|'Marked'|'Moved'|'Committed'|'Restoring'|'Restored'|'Cleaned';
export type DocumentPurgeConsent='deleteOwnedCopies'|'finishCopyPurge'|'restoreOwnedCopies'|'clearCopyRecovery';
export interface DocumentCopyRecovery {id:string;provider:string;conversationId:string;revision:string;phase:DocumentPurgePhase;items:DocumentReview['items']}
export interface DocumentRetentionOwner {provider:string;conversationId:string;revision:string;busy:boolean;items:Array<Omit<DocumentRecord,'authorized'> & {purging:boolean;historyReferences:number}>}
export interface DocumentRetentionInventory {owners:DocumentRetentionOwner[];recoveries:{recoveries:DocumentCopyRecovery[];unavailable:string[]}}
export interface DocumentPurgeReview {reviewId:string;provider:string;conversationId:string;items:DocumentReview['items'];recovery:DocumentCopyRecovery|null;expiresInSeconds:number}
export interface DocumentPurgeReceipt {provider:string;conversationId:string;operation:DocumentPurgeConsent;complete:boolean;recoveryId:string|null;error:string|null}

export interface HookStatus {
  installed: boolean;
  /** Coucou's status line relay (plan usage) is the status line in settings.json. */
  planRelayInstalled: boolean;
  settingsPath: string;
  hookPath: string;
  hookReady: boolean;
}

/** One agent other than Claude Code, as agents.rs reports it. */
export interface AgentHookStatus {
  /** The `--agent` name; its pill is `agent_<id>`. */
  id: string;
  name: string;
  installed: boolean;
  /** The file (or files, one per line) Coucou writes. */
  path: string;
  hookReady: boolean;
  /** The island can allow or deny this agent's permission requests. */
  approvals: boolean;
  /** What to do once it is written. */
  note: string;
}

export interface AgentHookPlan {
  diff: string;
  /** Where each existing file is copied first, one per line; "" when none. */
  backup: string;
  path: string;
  fingerprint: string;
}

export interface HookPreview {
  diff: string;
  backup: string;
  settingsPath: string;
  /** Hand back to hooksApply so only the reviewed diff is ever written. */
  fingerprint: string;
}

/** Same as `call`, but surfaces the error so the UI can show what went wrong. */
async function callOrThrow<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!IS_TAURI) throw new Error("not running inside Coucou");
  return invoke<T>(cmd, args);
}

export type BridgeEvent =
  | { name: "cursor"; payload: { x: number; y: number } }
  | { name: "tray"; payload: string }
  | { name: "hook"; payload: Record<string, unknown> }
  | { name: "screen-changed"; payload: null };

export interface DragDropPayload {
  type: "enter" | "over" | "drop" | "leave" | "rejected";
  error?:string;
  dropId?:string;
  paths?: string[];
  position?: {x:number;y:number};
}

interface WebView2Bridge {
  postMessageWithAdditionalObjects(message: unknown, objects: ArrayLike<unknown>): void;
}

/**
 * Files dragged onto the island. Only reaches us when the window takes the mouse.
 *
 * On Windows WebView2 takes the drop itself, as in Edge — Tauri's drop handling
 * never sees drags from the classic Explorer folder view (see
 * src-tauri/src/webview_drop.rs). Enter/over/leave come straight from the page;
 * the drop hands the File objects to Rust, which answers with their real paths
 * as a `file-drag` event. Elsewhere (Linux) Tauri's own drag events still do it.
 */
export async function onDragDrop(handler: (e: DragDropPayload) => void) {
  if (!IS_TAURI) return () => {};
  // Defence in depth: native drops deliver real paths; the page must never
  // allow the browser to navigate to a PDF/image/link dropped on a surface.
  const preventNavigation=(e:DragEvent)=>e.preventDefault();
  window.addEventListener("dragover",preventNavigation);
  window.addEventListener("drop",preventNavigation);
  const webview = (window as unknown as { chrome?: { webview?: WebView2Bridge } }).chrome?.webview;
  if (!webview) {
    const nativeFiles=await getCurrentWebview().listen<DragDropPayload>('pet-file-drag',event=>handler(event.payload));
    const nativeRejected=await getCurrentWebview().listen<string>('pet-file-rejected',event=>handler({type:'rejected',error:event.payload}));
    const unlisten=await getCurrentWebview().onDragDropEvent((event) => {
      handler(event.payload as DragDropPayload);
    });
    return ()=>{nativeFiles();nativeRejected();unlisten();window.removeEventListener("dragover",preventNavigation);window.removeEventListener("drop",preventNavigation);};
  }
  const hasFiles = (e: DragEvent) => e.dataTransfer?.types.includes("Files") ?? false;
  // dragenter/dragleave fire for every element crossed; only the outermost pair counts.
  let depth = 0;

  const onEnter = (e: DragEvent) => {
    if (!hasFiles(e)) return;
    e.preventDefault();
    if (depth++ === 0) handler({ type: "enter" });
  };
  const onOver = (e: DragEvent) => {
    if (!hasFiles(e)) return;
    // Without this WebView2 refuses the drop — or navigates to the file.
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
    handler({ type: "over" });
  };
  const onLeave = (e: DragEvent) => {
    if (!hasFiles(e) || depth === 0) return;
    if (--depth === 0) handler({ type: "leave" });
  };
  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    depth = 0;
    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) {
      handler({ type: "drop", paths: [] });
      return;
    }
    webview.postMessageWithAdditionalObjects("coucou-file-drop", files);
  };

  window.addEventListener("dragenter", onEnter);
  window.addEventListener("dragover", onOver);
  window.addEventListener("dragleave", onLeave);
  window.addEventListener("drop", onDrop);
  const unlisten = await listen<DragDropPayload>("file-drag", (e) => handler(e.payload));
  return () => {
    window.removeEventListener("dragenter", onEnter);
    window.removeEventListener("dragover", onOver);
    window.removeEventListener("dragleave", onLeave);
    window.removeEventListener("drop", onDrop);
    unlisten();
    window.removeEventListener("dragover",preventNavigation);
    window.removeEventListener("drop",preventNavigation);
  };
}

export async function onEvent<T>(name: string, handler: (payload: T) => void, webview?: string) {
  if (!IS_TAURI) return () => {};
  return listen<T>(name, (e) => handler(e.payload), webview ? { target: { kind: "Webview", label: webview } } : undefined);
}

/** RUN-01 v1 stores proposals and recovery references. It has no execution grant. */
export type RuntimeState='draft'|'paused'|'cancelled'|'recoveryRequired'|'queued'|'leased'|'stopping'|'completed';
export type RuntimeWorkState='proposed'|'cancelled'|'recoveryRequired';
export interface RuntimeOperationProposal {operationId:string;version:number;arguments:unknown}
export interface RuntimeStepInput {description:string;postcondition:string;operation?:RuntimeOperationProposal|null}
export interface RuntimeReviewedProposals {tasks:Array<{id:string;description:string;planId:string|null}>;plans:Array<{id:string;taskId:string;steps:Array<RuntimeStepInput&{id:string}>}>}
export type RuntimeDraftAction={kind:'proposeTask';description:string}|{kind:'proposePlan';taskId:string;steps:RuntimeStepInput[]}|{kind:'pause'|'resumeDraft'|'cancel'};
export interface RuntimeCreateInput {petId:string;requestId:string;intent:string;criteria:string[];conversationId?:string|null}
export interface RuntimeMutationInput {petId:string;goalId:string;revision:string;requestId:string;action:RuntimeDraftAction}
export interface RuntimeImportInput {petId:string;goalId:string;revision:string;requestId:string;taskId:string}
export interface RuntimeGoal {
 nativePlanRevisions?:RuntimeNativePlanReference[];
 nativeDrains?:RuntimeNativeDrainReference[];
 schemaVersion:1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|37|38|39|40|41|42|43|44|45|46|47|48|49|50|51|52|53|54|55|56|57|58|59|60|61|62|63|64|65|66;id:string;revision:string;sequence:number;createdAt:number;updatedAt:number;executionContext?:RuntimeContextBinding;admissions?:Array<{spec:RuntimeAdmissionSpec;phase:'queued'|'paused'|'cancelled'|'leased'|'stopRequested'|'recoveryRequired'|'completed';proof?:RuntimeConsentProof;lease?:RuntimeLease;stopReason?:'pause'|'cancel'|'expired'}>;
 spec:{receivedFrom?:RuntimeReceivedGoalOrigin;restoredFrom?:RuntimeRestoredFutureOrigin;requestId:string;owner:{petId:string;provider:string;adapter:string;modelHint:string|null};context:{kind:'unbound'}|{kind:'mochi';conversationId:string};intent:string;criteria:string[]};
 completion?:RuntimeCompletionRef;state:RuntimeState;tasks:Array<{id:string;description:string;state:RuntimeWorkState;planId:string|null}>;
 plans:Array<{id:string;taskId:string;steps:Array<RuntimeStepInput&{id:string;state:RuntimeWorkState}>}>;
 events:Array<{sequence:number;at:number;requestId:string;fingerprint:string;kind:'created'|'taskProposed'|'planProposed'|'paused'|'draftResumed'|'cancelled'|'cancelRequested'|'legacyImported'|'contextPrepared'|'admissionQueued'|'leaseClaimed'|'stopRequested'|'leaseDrained'|'claimAbandoned'|'goalCompleted'|'planGenerated'|'lostOwnerClosed'|'unclaimedOwnerClosed'|'nativeDrained'|'nativePlanRevised'|'ownerResumed'}>;
 legacy:Array<{taskId:string;provider:string;backend:string;conversationId:string|null;phase:string;intentMayBeIncomplete:true;receipts:import('../mochi/tasks').LocalStepReceipt[]}>;
}
export interface RuntimeLease {schemaVersion:1|2;id:string;controllerId:string;requestId:string;decisionId:string;goalId:string;provider:string;conversationId:string;model:string;policyRevision:string;capabilityRevision:string;sourceRevision:string;claimedAt:number;deadlineAt:number}
export interface RuntimeClaimInput {petId:string;goalId:string;revision:string;requestId:string}
export interface RuntimeClaimResult {receipt:RuntimeReceipt;lease:RuntimeLease;owned:boolean;reservationOwned:boolean;executionAvailable:false}
export interface RuntimeOwnerObservation {state:'originalProcessLive'|'ownerAbsent'|'identityMismatch'|'unavailable'|'legacyUnknown'|'notClaimed'|'closed';reason:string|null;backendPid:number|null;capturedAt:number|null;source:'nativeGrant'|'authenticatedRuntimeOwner';executionAvailable:false;recoveryAvailable:false}
export interface RuntimeLeaseStatus {record:RuntimeGoal;lease:RuntimeLease|null;owned:boolean;reservationOwned:boolean;source:'liveController'|'storedLease'|'queuedRequest'|'none';current:boolean;workers:number|null;executionAvailable:false;nativeOwner?:RuntimeOwnerObservation;requested?:{phase:'queued'|'scheduled'|'waitingResource'|'validating'|'confirmingClaim'|'recoveryRequired'|'stopped';reason:string|null;notBeforeAt?:number;resourceWait?:{position:number;interactiveBypasses:number;maxInteractiveBypasses:number}}}
export interface RuntimeConsentProof {schemaVersion:1|2|3;keyId:string;operation:'conversationRespond'|'goalPlanning'|'goalDeterministic';tag:string}
export interface RuntimeProvenanceStatus {verified:boolean;reason:'missing'|'unsafe'|'changed'|'invalid'|'wrongKey'|'revoked'|'expired'|'notQueued'|'capacity'|'unavailable'|'uncertain'|'busy'|'recoveryRequired'|null;executionAvailable:false}
export interface RuntimeBudget {deadlineAt:number;maxModelCalls:number;maxToolCalls:number;maxOutputBytes:number;maxTokens?:number|null;maxCostMicros?:number|null;payment:'localOnly'|'existingAccount'|'runtimeOnly'}
export interface RuntimeNativePeerPolicy {schemaVersion:1;goalId:string;owner:RuntimeGoal['spec']['owner'];sourceRevision:string;sourceSequence:number;specHash:string;contextHash:string;proposalHash:string;preparedAt:number;expiresAt:number;consent:'boundedNativePeerPlan';steps:Array<{cause:{taskId:string;planId:string;stepId:string};operation:NonNullable<RuntimeStepInput['operation']>;recipient:{petId:string;name:string;configurationHash:string};workReport?:unknown}>;sources:unknown[]}
export interface RuntimeExecutionStart {notBeforeAt?:number;schemaVersion:1|2;mode:'whenAvailable';scope:'finiteResponse'|'resultDisclosure'|'nativePeerPlan'|'nativeSystemPlan'}
export interface RuntimeAdmissionInput {start?:'whenAvailable';petId:string;goalId:string;revision:string;requestId:string;budget:RuntimeBudget}
export interface RuntimeAdmissionConfirm {disclosureConsent?:'boundedWorkDisclosure'|'boundedResultDisclosure'|'boundedNativePeerPlan';memoryConsent?:PetMemoryGoalConsent;petId:string;goalId:string;revision:string;token:string;approve:boolean}
export interface RuntimeEngineBinding {schemaVersion:1;endpoint:string;modelKey:string;instanceId:string|null;modelHash:string}
interface RuntimeNativeSystemCommon {goalId:string;owner:RuntimeGoal['spec']['owner'];sourceRevision:string;sourceSequence:number;specHash:string;contextHash:string;proposalHash:string;preparedAt:number;expiresAt:number;steps:Array<{cause:{taskId:string;planId:string;stepId:string};operation:NonNullable<RuntimeStepInput['operation']>}>}
export interface RuntimeHealthHost {bootId:string;pidNamespace:number;mountNamespace:number}
export interface RuntimeHealthBinding {schemaVersion:1;kind:'systemHostHealth';petId:string;grantRevision:string;host:RuntimeHealthHost}
export interface RuntimeInventoryBinding{schemaVersion:1;kind:'systemHostInventory';petId:string;grantRevision:string;host:RuntimeHealthHost}
export type RuntimeNativeSystemPolicy=RuntimeNativeSystemCommon&({schemaVersion:1;output:{petId:string;grantRevision:string;outputKey:string;incarnation:{coreCookie:number;nodeId:number;nodeSerial:string}};health?:never;inventory?:never}|{schemaVersion:2;health:RuntimeHealthBinding;output?:never;inventory?:never}|{schemaVersion:3;inventory:RuntimeInventoryBinding;health?:never;output?:never});
export interface RuntimeAdmissionSpec {nativeSystemPlan?:RuntimeNativeSystemPolicy;nativePeerPlan?:RuntimeNativePeerPolicy;executionStart?:RuntimeExecutionStart;resultDisclosure?:RuntimeResultDisclosurePolicy;funding?:RuntimeFunding;workDisclosure?:RuntimeWorkDisclosurePolicy;nativeVerification?:RuntimeCriteriaBinding;memoryData?:PetMemoryGoalGrant;engineBinding?:RuntimeEngineBinding;schemaVersion:1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28;continuation?:RuntimeContinuationReference;planningPolicy?:RuntimePlanningPolicy|null;proposalHash?:string|null;requestId:string;decisionId:string;goalId:string;sourceRevision:string;sourceSequence:number;owner:RuntimeGoal['spec']['owner'];context:RuntimeContextBinding;contextHash:string;intentHash:string;resolvedModel:string;descriptorRevision:string;policyRevision:string;evidenceHash:string;approvedAt:number;budget:RuntimeBudget}
export interface RuntimeAdmissionProposal {nativeSystemPlan?:RuntimeNativeSystemPolicy;nativePeerPlan?:RuntimeNativePeerPolicy;executionStart?:RuntimeExecutionStart;resultDisclosure?:RuntimeResultDisclosurePolicy;resultData?:RuntimeResultDisclosurePreview;funding?:RuntimeFunding;workDisclosure?:RuntimeWorkDisclosurePolicy;nativeVerification?:RuntimeCriteriaBinding;restoredFrom?:RuntimeRestoredFutureOrigin;memoryData?:PetMemoryGoalPreview;engineBinding?:RuntimeEngineBinding;continuation?:RuntimeContinuationReference;proposals:RuntimeReviewedProposals;planningPolicy?:RuntimePlanningPolicy|null;operation:'conversationRespond'|'goalPlanning'|'goalDeterministic';toolExecutionAvailable:false;goalId:string;revision:string;requestId:string;owner:RuntimeGoal['spec']['owner'];context:RuntimeContextBinding;intent:string;criteria:string[];resolvedModel:string;policyRevision:string;descriptorRevision:string;budget:RuntimeBudget;preparedAt:number;expiresAt:number}
export type RuntimeAdmissionDenial='invalid'|'ownerChanged'|'contextRequired'|'contextChanged'|'notDraft'|'inactive'|'unsupported'|'notReady'|'stale'|'modelUnresolved'|'policyChanged'|'budget'|'paymentPolicy'|'meteringUnavailable'|'expired'|'revisionChanged'|'inputLimit'|'recoveryNotClosed'|'historyIncomplete'|'unreconciledEffects'|'dataConsentRequired';
export type RuntimeAdmissionReview={status:'review';ticket:{token:string;proposal:RuntimeAdmissionProposal;approvalScope?:'admitGoal'|'confirmOriginalAdmission'}}|{status:'blocked';reason:RuntimeAdmissionDenial};
export type RuntimeAdmissionResult={status:'queued';receipt:RuntimeReceipt}|{status:'declined'}|{status:'blocked';reason:RuntimeAdmissionDenial;retryable?:boolean}|{status:'authorityBlocked';reason:NonNullable<RuntimeProvenanceStatus['reason']>;retryable?:boolean};
export interface RuntimeReceipt {status:'durable'|'uncertain'|'unchanged';record:RuntimeGoal}
export interface RuntimeRespondInput {petId:string;goalId:string;leaseId:string;revision:string;requestId:string}
export interface RuntimeRespondResult {text:string;criteriaVerified:false;goalComplete:false}
export interface RuntimeResultInput {petId:string;goalId:string;decisionId:string;requestId:string}
export interface RuntimeModelResult {schemaVersion:1;goalId:string;petId:string;provider:string;decisionId:string;leaseId:string;requestId:string;attemptId:string;model:string;inputHash:string;intentHash:string;text:string;acceptedOutputBytes:number;completedAt:number;recordedAt:number}
export type RuntimeResult=({status:'notStarted'}|{status:'recoveryRequired';phase:'reserved'|'started'|'uncertain'|'settledReceiptMissing'}|{status:'available';receipt:RuntimeModelResult})&{historicalSource?:RuntimeMessageInbox['items'][number]['historicalSource'];receiptHash?:string;criteriaVerified:false;goalComplete:false};
export interface RuntimeListing {schemaVersion:1;goals:Array<{id:string;revision:string;petId:string;historicalSource?:boolean;state:RuntimeState;updatedAt:number;intentPreview:string;tasks:number;steps:number}>;unavailable:number;pendingArtifacts:number;complete:boolean}

export interface RuntimePrepareContextInput {petId:string;goalId:string;revision:string;requestId:string;mode:'dedicated'|'snapshot';selectedMemory?:PetMemorySelected[];selectedCalendars?:RuntimeCalendarRequest;includeReceivedRequest?:boolean}
export interface RuntimeCalendarQuery{start:string;end:string;timeZone:string;limit:number}
export interface RuntimeCalendarRequest{sourceScope?:'pet';selected:Array<{id:string;revision:number}>;query:RuntimeCalendarQuery}
export interface RuntimeCalendarSelection{schemaVersion:1|2;sourceScope?:'pet';goalId:string;petId:string;provider:string;sourceConversationId?:string;query:RuntimeCalendarQuery;capturedAt:number;references:Array<{id:string;revision:number;binding:string;provenanceId:string;name:string;file:{stamp:unknown;contentHash:string}|null}>;dataHash:string}
export interface RuntimeContextBinding {calendars?:{selection:RuntimeCalendarSelection;data:unknown};receivedRequest?:RuntimeReceivedContextSelection;schemaVersion:1|2|3|4|5|6|7|8|9|10|11;memory?:{schemaVersion:1;references:PetMemoryReference[]};goalId:string;petId:string;provider:string;conversationId:string;requestId:string;mode:'dedicated'|'snapshot';sourceConversationId:string|null;sourceMessages:number;copiedMessages:number;seedHash:string;preparedAt:number}
export interface RuntimeContextView {binding:RuntimeContextBinding;messages:Array<{role:'user'|'assistant';content:string}>;messageCount:number;truncated:boolean}

/** Step receipts are observations, never reconstructible execution permits. */
export interface RuntimeStepPrepareInput {petId:string;goalId:string;leaseId:string;revision:string;planId:string;stepId:string;requestId:string}
export interface RuntimeStepConfirmInput {petId:string;goalId:string;leaseId:string;revision:string;token:string;approve:boolean}
export interface RuntimeStepRunInput {petId:string;goalId:string;leaseId:string;grantId:string;planId:string;stepId:string}
export interface RuntimeStepReadInput {petId:string;goalId:string;decisionId:string;planId:string;stepId:string}
export interface RuntimeStepRecoverInput extends RuntimeStepReadInput {revision:string;grantId:string}
export type RuntimePhysicalBinding={schemaVersion:1;busId:string;owner:string;uid:number;pid:number;identity:string}|{schemaVersion:1;kind:'systemSoundOutput';petId:string;grantRevision:string;outputKey:string}|RuntimeHealthBinding|RuntimeInventoryBinding;
export interface RuntimeStepScope {schemaVersion:1|2|3|4|5;grantId:string;requestId:string;goalId:string;petId:string;provider:string;decisionId:string;leaseId:string;controllerId:string;mainSpecHash:string;contextHash:string;proposalHash:string;policyRevision:string;planId:string;stepId:string;operationId:'spotify.control'|'system.sound.volume'|'system.sound.mute'|'system.health.observe'|'system.health.baseline'|'system.inventory.observe';version:1;argumentsHash:string;resource:RuntimePhysicalBinding;approvedAt:number;deadlineAt:number;maxOutputBytes:number}
export interface RuntimeStepProposal {scope:RuntimeStepScope;goalRevision:string;intent:string;criteria:string[];arguments:unknown;postcondition:string;preparedAt:number;expiresAt:number;maxUses:1;goalCriteriaVerified:false}
export interface RuntimeStepTicket {token:string;proposal:RuntimeStepProposal}
export type RuntimeStepConfirmResult={status:'granted';scope:RuntimeStepScope}|{status:'declined'}|{status:'blocked';reason:NonNullable<RuntimeProvenanceStatus['reason']>};
export type RuntimeStepSnapshot={playbackStatus:'Playing'|'Paused'|'Stopped';volume:number;observedAt:number}|{kind:'systemSoundOutput';outputKey:string;volumePercent:number|null;muted:boolean|null;observedAt:number}|RuntimeHealthEvidence|import('./system-health').SystemHealthBaseline|import('./system-inventory').SystemHostInventory;
export interface RuntimeHealthEvidence {kind:'systemHostHealth';petId:string;grantRevision:string;host:RuntimeHealthHost;sampleStartedAt:number;observedAt:number;sampleDurationMs:number;memory:{totalBytes:number;availableBytes:number}|null;cpu:{totalTicks:number;idleTicks:number}|null;uptime:{seconds:number}|null}
export interface RuntimeStepReceipt {observation:{delivery:'notDispatched'|'accepted'|'uncertain';before:RuntimeStepSnapshot|null;after:RuntimeStepSnapshot|null;effectVerified:boolean;scopeCurrent:boolean;failure:'invalidAddress'|'unavailable'|'changed'|'invalidState'|'cancelled'|'deadline'|'invalidAction'|'notAuthorized'|'transport'|null};finishedAt:number;criteriaVerified:false;goalComplete:false}
export interface RuntimeStepRecovery {status:'prepared'|'reserved'|'resultMissing'|'resultRecorded'|'finished';budgetPhase:'reserved'|'started'|'settled'|'uncertain'|null;recordedEffectVerified:boolean|null;reconciliationAvailable:boolean;accountingFrozen?:boolean;recordedResult?:RuntimeStepReceipt}
export interface RuntimeStepView {scope:RuntimeStepScope;phase:'approved'|'reserved'|'started'|'finished';result:RuntimeStepReceipt|null;recovery:RuntimeStepRecovery;executionAvailable:false}
export interface RuntimeRecipePrepareInput {petId:string;goalId:string;leaseId:string;revision:string;planId:string;requestId:string;criteria?:RuntimeCriterionInput[]}
export type RuntimeRecipeConfirmInput=RuntimeStepConfirmInput;
export interface RuntimeRecipeRunInput {petId:string;goalId:string;leaseId:string;revision:string;recipeId:string}
export interface RuntimeRecipeReadInput {petId:string;goalId:string;decisionId:string;planId:string}
export interface RuntimeRecipeScope {schemaVersion:1|2|3|4|5|6;id:string;requestId:string;goalId:string;petId:string;provider:string;decisionId:string;leaseId:string;controllerId:string;planId:string;proposalHash:string;approvedAt:number;deadlineAt:number;steps:RuntimeStepScope[];verification?:RuntimeCriteriaBinding}
export interface RuntimeRecipeProposal {scope:RuntimeRecipeScope;goalRevision:string;intent:string;criteria:string[];steps:RuntimeStepProposal[];preparedAt:number;expiresAt:number;criteriaVerified:false;goalComplete:false}
export interface RuntimeRecipeTicket {token:string;proposal:RuntimeRecipeProposal}
export type RuntimeRecipeConfirmation={status:'granted';scope:RuntimeRecipeScope}|{status:'declined'}|{status:'ended'}|{status:'blocked';reason:NonNullable<RuntimeProvenanceStatus['reason']>};
export interface RuntimeRecipeProgress {scope:RuntimeRecipeScope;phase:'reviewed'|'partial'|'verified'|'recoveryRequired';verifiedSteps:number;totalSteps:number;steps:Array<{stepId:string;phase:RuntimeStepView['phase']|null;recovery:RuntimeStepRecovery|null;result:RuntimeStepReceipt|null}>;verification:RuntimeVerification|null;criteriaVerified:boolean;goalComplete:boolean;executionAvailable:false}
export interface RuntimeRecipeResult {outcome:'verified'|'blocked'|'invalidated'|'recoveryRequired'|'criteriaUnmet'|'criteriaIncomplete'|'goalCompleted';progress:RuntimeRecipeProgress;completion:'durable'|'uncertain'|'unchanged'|null}

export type RuntimeCriterionPredicate={kind:'spotifyPlayback';status:'playing'|'paused'|'stopped'}|{kind:'spotifyVolume';percent:number}|{kind:'petMessagesPersisted'}|{kind:'systemSoundVolume';percent:number}|{kind:'systemSoundMute';muted:boolean}|{kind:'systemHealthObserved'}|{kind:'systemHealthBaselineMeasured'}|{kind:'systemHostInventoryObserved'};
export interface RuntimeCriterionInput {criterionIndex:number;predicate:RuntimeCriterionPredicate}
export interface RuntimeCriteriaBinding {schemaVersion:1|2|3|4|5|6|7;specHash:string;criteriaCount:number;maxObservations:1;conditions:Array<RuntimeCriterionInput&{criterionHash:string}>}
export interface RuntimeCompletionRef {schemaVersion:1|2;source?:{kind:'nativeMessages';verificationId:string};goalId:string;petId:string;recipeId?:string;decisionId:string;leaseId:string;planId?:string;sourceRevision:string;targetRevision:string;reportHash:string;observedAt:number}
export interface RuntimeVerification {schemaVersion:1|2|3|4;reference:RuntimeCompletionRef;recipeHash:string;specHash:string;sourceSequence:number;sourceUpdatedAt:number;sourceRecordHash:string;snapshot:RuntimeStepSnapshot|null;scopeCurrent:boolean;recordedAt:number;results:Array<{criterionIndex:number;criterionHash:string;status:'met'|'unmet'|'unknown'}>;criteriaVerified:boolean}
export interface RuntimeRecipeCompleteInput extends RuntimeRecipeReadInput {recipeId:string;revision:string}

export interface RuntimePlanningLimits {maxRounds:number;maxSteps:number;maxOutputBytes:number;noProgressLimit:number}
export interface RuntimePlanningPolicy {schemaVersion:1|2|3|4;catalog:'spotifyTransportV1'|'peerCorrespondenceV1'|'peerCorrespondenceMemoryV1'|'peerDelegationV1'|'peerDelegationIterationV1';limits:RuntimePlanningLimits;start?:'whenAvailable'}
export interface RuntimePlanningInput extends RuntimeAdmissionInput {catalog?:RuntimePlanningPolicy['catalog'];limits:RuntimePlanningLimits;start?:'whenAvailable'}
export interface RuntimeContinuationInput extends RuntimeAdmissionInput {limits?:RuntimePlanningLimits|null;reviewUncertainEffects?:boolean;start?:'whenAvailable'}
export interface RuntimeRecoveryReference {schemaVersion:1;id:string;requestId:string;goalId:string;petId:string;decisionId:string;leaseId:string;sourceGoalRevision:string;sourceGoalHash:string;sourceSequence:number;targetGoalRevision:string;targetSequence:number;targetState:RuntimeState;sourceGrantRevision:string;sourceGrantHash:string;preparedAt:number;approvedAt:number;unclaimed:boolean}
export interface RuntimeUncertainEffect {decisionId:string;attemptId:string;grantId:string;planId:string;stepId:string;scopeHash:string;operation:RuntimeOperationProposal;targetLabel:string}
export interface RuntimeNativeReceiptRef {sourceDecisionId?:string;planId:string;stepId:string;messageId:string;recipientPetId:string;hash:string}
export interface RuntimeNativeDrainReference {schemaVersion:1;id:string;requestId:string;goalId:string;petId:string;decisionId:string;leaseId:string;sourceRevision:string;sourceSequence:number;sourceGoalHash:string;targetRevision:string;targetSequence:number;targetState:'paused'|'recoveryRequired'|'cancelled';confirmed:boolean;reason:'pause'|'cancel'|'expired'|null;observedAt:number}
export interface RuntimeContinuationReference {schemaVersion:1|2|3|4|5|6|7|8|9|10;funded?:{goalId:string;petId:string;decisionId:string;closedAt:number};planRevision?:RuntimeNativePlanReference;drain?:RuntimeNativeDrainReference;native?:{schemaVersion:1;proposalHash:string;receipts:RuntimeNativeReceiptRef[]};uncertainEffects?:RuntimeUncertainEffect[];sourceRevision:string;sourceSequence:number;sourceGoalHash:string;closure?:RuntimeRecoveryReference;historyHash:string;evidenceFence:string;previous:Array<{nativeSchema?:4|5|6|7|8|11|16;decisionId:string;specHash:string;rowHash:string;grantHash:string;budget:RuntimeBudget;resolvedModel:string}>;unused:Array<{decisionId:string;grantHash:string}>;charge:{modelCalls:number;toolCalls:number;outputBytes:number;unknownModelResults:number;unknownToolResults:number};historyContinuityVerified:false;additionalAllowance:boolean;reviewedAt:number}
export interface RuntimeDriverInput {petId:string;goalId:string;leaseId:string}
export interface RuntimeDriverStatus extends RuntimeDriverInput {controllerId:string;phase:'idle'|'planning'|'publishing'|'awaitingApproval'|'waitingResource'|'awaitingVerification'|'closing'|'registering'|'executing'|'feedback'|'completed'|'declined'|'approvalEnded'|'stopped'|'limited'|'recoveryRequired';round:number;active:boolean;retainedApproval:boolean;retainedClosure?:boolean;reason:string|null;notBeforeAt?:number;resourceWait?:{position:number;interactiveBypasses:number;maxInteractiveBypasses:number}}

export interface RuntimeHistoryInput{petId:string;goalId:string;revision:string}
export interface RuntimeHistoryCharge{modelCalls:number;toolCalls:number;outputBytes:number;unknownModelResults:number;unknownToolResults:number}
export interface RuntimeRestoredHistorySource{schemaVersion:1;historyHash:string;sourceFingerprint:string;sourceGoalHash:string;historicalRelatedGrants:number;unattributedHistoricalGrants:number;ownerAbsenceProven:false;postSnapshotStateUnknown:true;executionAuthorityGranted:false}
export interface RuntimeHistorySnapshot{schemaVersion:1|2;scope:'allRecordedAdmissions'|'restoredSnapshotOnly';restoredSource?:RuntimeRestoredHistorySource;goalId:string;petId:string;goalRevision:string;goalState:RuntimeState;sourceHash:string;historyContinuityVerified:false;historyCoverage:string;authenticatedSubtotal:RuntimeHistoryCharge;cumulativeCharge:RuntimeHistoryCharge|null;scanComplete:boolean;sourceConsistent:boolean;accountingComplete:boolean;pendingArtifacts:number;unlinkedGrants:Array<{decisionId:string;hash:string;phase:string;updatedAt:number;leaseId:string|null;revokedWithoutIo:boolean;retirement?:RuntimeRetirementReference}>;executionAvailable:false;continuationAvailable:false;tokenUsage:number|null;costMicros:number|null}
export interface RuntimeRetirementReference{schemaVersion:1|2|3;id:string;requestId:string;goalId:string;petId:string;decisionId:string;sourceGoalRevision:string;sourceGoalHash:string;sourceSequence:number;sourceGrantRevision:string;sourceGrantHash:string;sourceGrantSchema:1|4|10|19|48;sourceGrantUpdatedAt:number;inventoryHash:string;artifactFence:string;profile:{directory:{device:number;inode:number};keyId:string};preparedAt:number;approvedAt:number;historyContinuityVerified:false}
export interface RuntimeRetirementPrepareInput extends RuntimeHistoryInput{decisionId:string;requestId:string}
export interface RuntimeRetirementConfirmInput extends RuntimeHistoryInput{decisionId:string;token:string;approve:boolean}
export interface RuntimeRetirementProposal{nativeSystemPlanHash?:string;goalId:string;petId:string;revision:string;requestId:string;decisionId:string;intent:string;criteria:string[];approvalScope:'retireUnlinkedAdmission'|'confirmOriginalRetirement';originalOperation:'conversationRespond'|'goalPlanning'|'goalDeterministic';originalSpecHash:string;originalApprovedAt:number;sourceGrantRevision:string;sourceGrantHash:string;authenticatedSubtotal:RuntimeHistoryCharge;linkedAccountingComplete:boolean;historyContinuityVerified:false;targetUsageKnownZero:true;sameGoalUnlinked:number;matchingStagedImages:number;preparedAt:number;expiresAt:number;executionAvailable:false}
export interface RuntimeRetirementTicket{token:string;proposal:RuntimeRetirementProposal}
export interface RuntimeRetirementReceipt{status:RuntimeReceipt['status'];record:RuntimeGoal;reference:RuntimeRetirementReference;executionAvailable:false;newAdmissionCreated:false}
export type RuntimeRetirementPrepared={status:'review';ticket:RuntimeRetirementTicket}|{status:'retired';receipt:RuntimeRetirementReceipt}|{status:'blocked';reason:string;retryable:boolean};
export type RuntimeRetirementConfirmed={status:'retired';receipt:RuntimeRetirementReceipt}|{status:'declined'}|{status:'blocked';reason:string;retryable:boolean;retained:boolean};
export interface RuntimeRecoveryPrepareInput{petId:string;goalId:string;revision:string;requestId:string}
export interface RuntimeRecoveryConfirmInput{petId:string;goalId:string;revision:string;token:string;approve:boolean}
export interface RuntimePeerPlanningSnapshot{goalId:string;petId:string;goalRevision:string;goalHash:string;decisionId:string;leaseId:string;closureId:string;specHash:string;grantHash:string;historyHash:string;receiptFence:string;originalBudget:RuntimeBudget;originalPolicy:Omit<RuntimePlanningPolicy,'schemaVersion'|'catalog'>&{schemaVersion:3|4;catalog:'peerDelegationIterationV1'};model:string;lastReviewedDestination:PetMemoryDestination|null;remaining:RuntimeDelegationAllowance;delegatedHeld:RuntimeDelegationAllowance;charge:RuntimeHistoryCharge;journalHash:string;roundsObserved:number;outcome:'canonicalPlan'|'unpublishedPlan'|'uncertainModelResult'|'noExecutablePlan';receipts:RuntimeNativeReceiptRef[];pendingSteps:Array<{taskId:string;planId:string;stepId:string}>;observedAt:number;executionAvailable:false;modelUseAuthorized:false;additionalAllowance:false}
export interface RuntimeRecoveryProposal{peerResumption?:RuntimePeerPlanningSnapshot;goalId:string;petId:string;revision:string;requestId:string;intent:string;criteria:string[];provider:string;leaseId:string;approvalScope:'closeLostExecution'|'ackRecordedClosure'|'resumeOriginalPeerExecution'|'ackRecordedPeerResume';resultingState:RuntimeState;snapshot:{accounting:{modelCallsUsed:number;toolCallsUsed:number;outputBytesCharged:number;unknownAttempts:number}|null;budget:RuntimeBudget|null};preparedAt:number;expiresAt:number;executionAvailable:false}
export interface RuntimeRecoveryTicket{token:string;proposal:RuntimeRecoveryProposal}
export type RuntimeRecoveryPrepared={status:'review';ticket:RuntimeRecoveryTicket}|{status:'closed';receipt:RuntimeReceipt}|{status:'acknowledged';receipt:RuntimeReceipt;executionAvailable:false}|{status:'blocked';reason:string;retryable:boolean};
export type RuntimeRecoveryConfirmed={status:'closed';receipt:RuntimeReceipt}|{status:'resumed';receipt:RuntimeReceipt;lease:RuntimeLease;owned:boolean;executionAvailable:false}|{status:'acknowledgementPending';receipt:RuntimeReceipt}|{status:'declined'}|{status:'blocked';reason:string;retryable:boolean;retained?:boolean};

/** Explicit native-user records. None of these DTOs confers a model tool or
 * execution authority; only the original native data review can grant use. */
export interface PetMemoryOwner {petId:string;scope:{kind:'pet'}|{kind:'conversation';conversationId:string}}
export interface PetMemorySelected {id:string;revision:number}
export interface PetMemoryInput {kind:'note'|'preference';title:string;content:string;sensitivity:'normal'|'sensitive';expiresAt:number|null}
export interface PetMemoryEntry extends PetMemoryInput {id:string;revision:number;owner:PetMemoryOwner;provenance:'explicitUser';retention:'untilDeleted';createdAt:number;updatedAt:number;observedAt:number}
export interface PetMemoryReference {id:string;revision:number;owner:PetMemoryOwner;contentHash:string;expiresAt:number|null}
export interface PetMemoryQuery {owner:PetMemoryOwner;query:string;limit:number;includeExpired:boolean;offset?:number;expectedRevision?:string|null}
export interface PetMemoryPage {schemaVersion:1;owner:PetMemoryOwner;revision:string;items:Array<{entry:PetMemoryEntry;available:boolean}>;hasMore:boolean;nextOffset:number|null}
export type PetMemoryMutation={operation:'create';input:PetMemoryInput}|{operation:'update';id:string;expectedItemRevision:number;input:PetMemoryInput}|{operation:'delete';id:string;expectedItemRevision:number};
export interface PetMemoryReceipt {schemaVersion:1;owner:PetMemoryOwner;revision:string;id:string;item:PetMemoryEntry|null}
export type PetMemoryDestination={kind:'server'|'api';provider:string;endpoint:string;model:string}|{kind:'nativeSession';provider:string;adapter:string;model:string;workspace:string;sessionId:string|null};
export type PetMemoryGoalConsent='boundedGoalContext';
export interface PetMemoryGoalGrant {calendarSelection?:RuntimeCalendarSelection;receivedRequest?:RuntimeReceivedContextSelection;schemaVersion:1|2|3|4|5|6|7|8|9|10;consent:PetMemoryGoalConsent;destination:PetMemoryDestination;maxInputBytes:number}
export interface PetMemoryGoalPreview {calendarSelection?:RuntimeCalendarSelection;receivedRequest?:RuntimeReceivedContextPreview;composedMessage?:unknown;schemaVersion:1|2|3|4|5|6|7|8|9|10;goalId:string;petId:string;sourceConversationId:string|null;destination:PetMemoryDestination;records:PetMemoryEntry[];requiredConsent:PetMemoryGoalConsent;maxInputBytes:number}

export type PetMemoryManualConsent='thisRequest'|'nativeSessionContext';
export interface PetMemoryDecision {token:string;consent:PetMemoryManualConsent}
export interface PetMemoryPrepare {provider:string;conversationId:string;query:string;selectedModel?:string;selected:PetMemorySelected[];context:ChatContext|null;documentSequence?:number}
export interface PetMemoryDispatchCoverage {schemaVersion:1;kind:'exactNativeEndpoint'|'knownApiEndpointSet'|'nativeSessionProtocol';endpoints:string[];providerInternalsObserved:false;modelSemantics:string;toolsContainmentObserved:false}
export interface PetMemoryManualPreview {schemaVersion:1;petId:string;conversationId:string;destination:PetMemoryDestination;question:string;records:PetMemoryEntry[];preparedAt:number;expiresAt:number;requiredConsent:PetMemoryManualConsent;nativeAccountCoverage:string;dispatchCoverage:PetMemoryDispatchCoverage;requestContextHash:string}
export interface PetMemoryTicket {token:string;proposal:PetMemoryManualPreview}

export interface RuntimeRestoredFutureSource{petId:string;goalId:string;revision:string;goalHash:string;historyHash:string;sourceFingerprint:string;snapshotHash:string;historicalRelatedGrants:number;unattributedHistoricalGrants:number}
export interface RuntimeRestoredFutureOrigin{body:{schemaVersion:1;purpose:'restoredFutureDraftDataOnly';keyId:string;reviewId:string;clientRequestId:string;preparedAt:number;approvedAt:number;expiresAt:number;source:RuntimeRestoredFutureSource;targetGoalId:string;baseSpecHash:string;unknownStateAccepted:true;executionAuthorityGranted:false;accountingKnown:false;ownerAbsenceProven:false};tag:string}
export interface RuntimeFuturePrepareInput{petId:string;sourceGoalId:string;sourceRevision:string;requestId:string;intent:string;criteria:string[]}
export interface RuntimeFutureConfirmInput{petId:string;sourceGoalId:string;sourceRevision:string;token:string;approve:boolean;acknowledgeUnknownState:boolean}
export interface RuntimeFutureProposal{schemaVersion:1;purpose:'reviewRestoredFutureDraftDataOnly';reviewId:string;clientRequestId:string;source:RuntimeRestoredFutureSource;sourceOwner:RuntimeGoal['spec']['owner'];sourceIntent:string;sourceCriteria:string[];future:RuntimeGoal['spec'];targetGoalId:string;preparedAt:number;expiresAt:number;contextCopied:false;historicalBudgetReused:false;executionAuthorityGranted:false;postSnapshotStateUnknown:true;ownerAbsenceProven:false}
export type RuntimeFuturePrepared={status:'review';ticket:{token:string;proposal:RuntimeFutureProposal};pendingCreation?:RuntimeReceipt}|{status:'existing';receipt:RuntimeReceipt;executionAuthorityGranted:false};
export type RuntimeFutureConfirmed={status:'created';receipt:RuntimeReceipt;executionAuthorityGranted:false}|{status:'declined'};

/** Confirms preference data and the executor response, not the next login. */
export interface AutostartPreferenceReceipt{schemaVersion:1;enabled:boolean;change:'changed'|'unchanged';registration:'executorAccepted'|'unconfirmed'|'notAttempted';reason?:string}

export interface RuntimeReceivedMessageReference{decisionId:string;messageId:string;scopeHash:string}
export interface RuntimeGeneratedPlanReference {schemaVersion:1;decisionId:string;roundIndex:number;output:{schemaVersion:1;attemptId:string;bodyHash:string;kind:'plan';recordedAt:number};source:{revision:string;sequence:number;updatedAt:number;recordHash:string;policyHash:string;mainSpecHash:string;leaseId:string;controllerId:string};taskId:string;planId:string;targetRevision:string;targetSequence:number;publishedAt:number;proposalHash:string}
export interface RuntimePeerRoundDataConsent {schemaVersion:1;id:string;goalHash:string;reviewHash:string;leaseId:string;controllerId:string;contextId:string;turnId:number;inputHash:string;source:{schemaVersion:1;petId:string;goalId:string;decisionId:string;destination:PetMemoryDestination;exactHash:string;projectionHash:string;informationHash:string;expiresAt:number};approvedAt:number;expiresAt:number}
export interface RuntimePlannedPeerData {roundData?:RuntimePeerRoundDataConsent;schemaVersion:1|2|3|4;source:{receivedRequest?:RuntimeReceivedContextSelection;schemaVersion:1|2|3|4;goalId:string;petId:string;conversationId:string;admissionHash:string;contextHash:string;destination:PetMemoryDestination;references:PetMemoryReference[]};generatedPlan:RuntimeGeneratedPlanReference}
export interface RuntimeMessageEnvelope{plannedData?:RuntimePlannedPeerData;derivedResult?:{schemaVersion:1|2;result:RuntimeDerivedResultReference;origin:RuntimeResultDisclosurePreview['origin'];contextHash:string};delegation?:{schemaVersion:1|2;subgoal:string;criteria:string[];allowance:RuntimeDelegationAllowance;maxDepth:number;ancestors:string[]};workReport?:RuntimeWorkReport;schemaVersion:1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16;replyTo?:RuntimeReceivedMessageReference;messageId:string;origin:{petId:string;provider:string;adapter:string;conversationId:string;goalId:string;decisionId:string;leaseId:string;controllerId:string};recipient:{petId:string;name:string;configurationHash:string};correlationId:string;cause:{taskId:string;planId:string;stepId:string};kind:'message'|'request'|'reply'|'workEvent'|'delegation';text:string;createdAt:number;deadlineAt:number;providerUseAuthorized:false}
export interface RuntimeMessageReceipt{scope:{generatedPlan?:RuntimeGeneratedPlanReference;envelope:RuntimeMessageEnvelope;requestId:string;goalRevision:string;mainSpecHash:string;proposalHash:string};stage:'received';receivedAt:number;attemptId:string}
export interface RuntimeMessageTicket{token:string;proposal:{receivedMessage?:RuntimeMessageEnvelope;envelope:RuntimeMessageEnvelope;goalRevision:string;intent:string;criteria:string[];expiresAt:number;destinationExecutionAuthorized:false}}
export type RuntimeMessageConfirmResult={status:'received';receipt:RuntimeMessageReceipt}|{status:'declined'}|{status:'blocked';reason:string;retained:boolean};
export interface RuntimeMessageInboxInput{petId:string;after?:string|null;limit:number;includeHistorical?:boolean;includeReplySources?:boolean;includeFundingSources?:boolean}
export interface RuntimeMessageInbox{schemaVersion:1|2|3|4|5|6|7|8;petId:string;items:Array<{resultSource?:RuntimeReceivedMessageReference;fundingSource?:RuntimeReceivedMessageReference;replySource?:RuntimeReceivedMessageReference;receipt:RuntimeMessageReceipt;available:boolean;historicalSource?:{schemaVersion:1;historyHash:string;sourceProfileFingerprint:string;sourceFileHash:string;snapshotOnly:true;postSnapshotStateUnknown:true;executionAuthorized:false}}>;unavailable:number;pendingArtifacts:number;scanComplete:boolean;hasMore:boolean;nextGrant:string|null;executionAuthorized:false}

export interface RuntimeMessageIntentsInput{petId:string;goalId:string;decisionId:string}
export interface RuntimeMessageIntentInventory{goalRevision:string;decisionId:string;entries:Array<{message:RuntimeMessageEnvelope;retirementRecorded:boolean}>;pendingArtifacts:number;scanComplete:boolean}
export interface RuntimeMessageRetireInput extends RuntimeMessageIntentsInput{revision:string;messageId:string;requestId:string}
export interface RuntimeMessageRetireConfirmInput extends RuntimeMessageIntentsInput{revision:string;token:string;approve:boolean}
export interface RuntimeMessageRetireProposal{goalId:string;petId:string;decisionId:string;revision:string;message:RuntimeMessageEnvelope;artifacts:Array<{name:string;hash:string;stampHash:string}>;sourceGrantHash:string;preparedAt:number;expiresAt:number;authorityReductionOnly:true;budgetRefund:false;deliveryAuthorized:false}
export interface RuntimeMessageRetireTicket{token:string;proposal:RuntimeMessageRetireProposal}
export interface RuntimeMessageRetireReceipt{reference:{schemaVersion:2;id:string;requestId:string;scope:RuntimeMessageReceipt['scope'];sourceGrantHash:string;files:RuntimeMessageRetireProposal['artifacts'];preparedAt:number;retiredAt:number};cleanupComplete:boolean;messageDelivered:false;budgetRefund:false;executionAuthorized:false}
export type RuntimeMessageRetirePrepared={status:'review';ticket:RuntimeMessageRetireTicket}|{status:'retired';receipt:RuntimeMessageRetireReceipt};
export type RuntimeMessageRetireConfirmed={status:'retired';receipt:RuntimeMessageRetireReceipt}|{status:'declined'}|{status:'blocked';reason:string;retained:boolean};

export interface RuntimeNativeProgress{closurePending?:string;report:{reference:RuntimeCompletionRef;criteriaVerified:boolean;expectedMessages:number;receipts:RuntimeNativeReceiptRef[];results:Array<{criterionIndex:number;criterionHash:string;status:'met'|'unmet'|'unknown'}>};completion:RuntimeReceipt|null;recipientExecutionVerified:false}


export interface RuntimeNativeCarry {schemaVersion:1;proposalHash:string;receipts:RuntimeNativeReceiptRef[]}
export interface RuntimeNativePlanEdit {cause:{taskId:string;planId:string;stepId:string};after:RuntimeStepInput}
export interface RuntimeDerivedResultReference{schemaVersion:1;goalId:string;petId:string;decisionId:string;requestId:string;attemptId:string;receiptHash:string;byteStart:number;byteEnd:number;totalBytes:number}
export interface RuntimeDerivedResultReply{toPetId:string;source:RuntimeReceivedMessageReference;result:RuntimeDerivedResultReference;text:string;expiresAt:number}
export interface RuntimeNativePlanReference {schemaVersion:1|2|3;id:string;requestId:string;goalId:string;petId:string;decisionId:string;sourceRevision:string;sourceSequence:number;sourceGoalHash:string;sourceProposalHash:string;targetRevision:string;targetSequence:number;targetState:'paused'|'recoveryRequired';targetProposalHash:string;reviewedAt:number;patches:Array<RuntimeNativePlanEdit&{before:RuntimeStepInput}>}
export interface RuntimeResultDisclosurePolicy {schemaVersion:1|2;consent:'boundedResultDisclosure';goalId:string;petId:string;sourceRevision:string;intentHash:string;contextHash:string;proposalHash:string;cause:RuntimeNativePlanEdit['cause'];arguments:RuntimeDerivedResultReply;origin:RuntimeResultDisclosurePreview['origin'];recipient:RuntimeMessageEnvelope['recipient'];memoryReferences:PetMemoryReference[];source:RuntimeReceivedGoalSource}
export interface RuntimeResultDisclosurePreview {schemaVersion:1|2;goalId:string;petId:string;revision:string;intentHash:string;contextHash:string;proposalHash:string;cause:RuntimeNativePlanEdit['cause'];arguments:RuntimeDerivedResultReply;origin:{provider:string;adapter:string;model:string;inferenceDestination:PetMemoryDestination;inputHash:string;intentHash:string;completedAt:number;recordedAt:number};recipient:RuntimeMessageEnvelope['recipient'];receivedMessage:RuntimeMessageEnvelope;memoryReferences:PetMemoryReference[];dependencyPolicy:'allOriginalSelectedSourcesMustRemainCurrent';useScope:'oneLocalReplyToDelegationSource';requiredConsent:'boundedResultDisclosure';providerUseAuthorized:false;sharingAuthorized:false;expiresAt:number}
export interface RuntimeNativePlanProposal {resultData?:RuntimeResultDisclosurePreview;goalId:string;petId:string;conversationId:string;revision:string;intent:string;criteria:string[];before:RuntimeReviewedProposals;after:RuntimeReviewedProposals;changes:RuntimeNativePlanReference;coverage?:RuntimeNativeCarry;historyHash:string;preparedAt:number;expiresAt:number;executionAuthorized:false;budgetReserved:false}
export interface RuntimeNativePlanReview {token:string;proposal:RuntimeNativePlanProposal}
export interface RuntimeNativePlanReceipt {receipt:RuntimeReceipt;reference:RuntimeNativePlanReference;acknowledged:boolean;executionAuthorized:false;newAdmissionCreated:false}
export type RuntimeNativePlanConfirmed={status:'revised';receipt:RuntimeNativePlanReceipt}|{status:'declined'}|{status:'blocked';reason:string;retained:boolean};
export interface RuntimeNativePlanInventory {schemaVersion:1;goalId:string;petId:string;revision:string;sourceHash:string;observedAt:number;coverage:RuntimeNativeCarry|null;factCoverageComplete:boolean;items:Array<{reference:RuntimeNativePlanReference;state:'staged'|'sealed'|'published'|'withdrawalPending'|'abandoned'|'blocked';pendingArtifacts:boolean;canAck:boolean;canAbandon:boolean;reason:RuntimeProvenanceStatus['reason']}>;hasMore:boolean;nextCursor:string|null;executionAuthorized:false;repairPerformed:false}

export interface RuntimeReceivedGoalSource{reference:RuntimeReceivedMessageReference;senderPetId:string;senderGoalId:string;senderConversationId:string;recipientPetId:string;receiptHash:string;envelopeHash:string;receivedAt:number;deadlineAt:number}
export interface RuntimeReceivedGoalOrigin{body:{plannedDependencies?:RuntimePlannedPeerData;delegation?:RuntimeMessageEnvelope['delegation'];schemaVersion:1|2|3|4|5|6|7;purpose:'receivedRequestOwnGoalDataOnly'|'fundedDelegationOwnGoalDataOnly'|'receivedResultOwnGoalDataOnly'|'receivedRecursiveResultOwnGoalDataOnly'|'recursiveFundedDelegationOwnGoalDataOnly'|'receivedPlannedPeerOwnGoalDataOnly'|'derivedFundedDelegationOwnGoalDataOnly';keyId:string;reviewId:string;clientRequestId:string;actorWindow:string;source:RuntimeReceivedGoalSource;targetGoalId:string;baseSpecHash:string;preparedAt:number;approvedAt:number;expiresAt:number;contextCopied:false;executionAuthorityGranted:false};tag:string}
export interface RuntimeReceivedGoalPrepareInput{petId:string;source:RuntimeReceivedMessageReference;requestId:string;intent:string;criteria:string[]}
export interface RuntimeReceivedGoalConfirmInput{petId:string;source:RuntimeReceivedMessageReference;token:string;approve:boolean}
export interface RuntimeReceivedGoalProposal{schemaVersion:1|2|3|4|5|6|7;reviewId:string;clientRequestId:string;source:RuntimeReceivedGoalSource;receivedMessage:RuntimeMessageEnvelope;future:RuntimeGoal['spec'];targetGoalId:string;preparedAt:number;expiresAt:number;contextCopied:false;executionAuthorityGranted:false;budgetReserved:false}
export interface RuntimeReceivedGoalTicket{token:string;proposal:RuntimeReceivedGoalProposal}
export type RuntimeReceivedGoalPrepared={status:'review';ticket:RuntimeReceivedGoalTicket}|{status:'existing';receipt:RuntimeReceipt;executionAuthorityGranted:false};
export type RuntimeReceivedGoalConfirmed={status:'created';receipt:RuntimeReceipt;executionAuthorityGranted:false}|{status:'declined'};

export interface RuntimeReceivedContextSelection {schemaVersion:1|2|3|4|5|6|7;ownOriginHash:string;source:RuntimeReceivedGoalSource}
export interface RuntimeReceivedContextPreview {selection:RuntimeReceivedContextSelection;receivedMessage:RuntimeMessageEnvelope;modelMessage:unknown}

export interface RuntimeReceivedWorkInput{petId:string;goalId:string;revision:string;source:RuntimeReceivedMessageReference}
export interface RuntimeReceivedWork{schemaVersion:1;scope:'ownGoalLinkedToReceivedRequest';petId:string;goalId:string;goalRevision:string;goalHash:string;source:Omit<RuntimeReceivedGoalSource,'senderConversationId'>;originHash:string;observedAt:number;state:'proposed'|'acceptedQueued'|'sessionReserved'|'attemptInFlight'|'waiting'|'awaitingVerification'|'stopping'|'paused'|'cancelled'|'completed'|'recoveryRequired'|'deadlineElapsed'|'historical'|'unavailable';admission:{decisionId:string;approvedAt:number;deadlineAt:number;grantRevision:string;grantHash:string}|null;ownerState:'originalProcessLive'|'ownerAbsent'|'identityMismatch'|'unavailable'|'legacyUnknown'|'notClaimed'|'closed'|null;attempts:{reserved:number;started:number;settled:number;uncertain:number;unknownOutcomes:number}|null;evidenceComplete:boolean;criteriaVerified:boolean;requestWithinDeadline:boolean;executionAuthorized:false}

export interface RuntimeWorkReport{schemaVersion:1;subjectPetId:string;subjectGoalId:string;subjectRevision:string;observedAt:number;state:RuntimeReceivedWork['state'];attempts:RuntimeReceivedWork['attempts'];evidenceComplete:boolean;criteriaVerified:boolean}
export interface RuntimeWorkDisclosureArguments{toPetId:string;subjectGoalId:string;subjectSpecHash:string;source:RuntimeReceivedMessageReference;expiresAt:number;includeAttempts:boolean;states:RuntimeWorkReport['state'][]}
export interface RuntimeWorkDisclosurePolicy{schemaVersion:1;planId:string;stepId:string;arguments:RuntimeWorkDisclosureArguments;recipient:RuntimeMessageEnvelope['recipient'];proposalHash:string}
export interface RuntimeWorkReportDraft{futureDisclosure?:RuntimeWorkDisclosureArguments;schemaVersion:1;petId:string;subjectGoalId:string;subjectRevision:string;source:RuntimeReceivedMessageReference;toPetId:string;expiresAt:number;includeAttempts:boolean;intent:string;criteria:string[];steps:RuntimeStepInput[];preview:RuntimeWorkReport;executionAuthorized:false;budgetReserved:false}

export interface RuntimeDelegationAllowance{modelCalls:number;toolCalls:number;outputBytes:number}
export interface RuntimeDelegationBinding{goalId:string;petId:string;decisionId:string;admissionHash:string;boundAt:number}
export interface RuntimeDelegationSettlement{at:number;reviewId:string;childGrantHash:string;childRevision:string;consumed:RuntimeDelegationAllowance;unknown:RuntimeDelegationAllowance;closure:'originalNativeClosure'|'revokedNeverClaimed'}
export interface RuntimeDelegationReservation{schemaVersion:1|2|3|4|5|6|7|8;messageId:string;scopeHash:string;recipientPetId:string;allowance:RuntimeDelegationAllowance;deadlineAt:number;reservedAt:number;state:{phase:'pending'}|{phase:'bound';binding:RuntimeDelegationBinding}|{phase:'withdrawn';at:number;reviewId:string}|{phase:'cancelRequested';binding:RuntimeDelegationBinding;cancellation:{at:number;reviewId:string}}|{phase:'settled';binding:RuntimeDelegationBinding;cancellation:{at:number;reviewId:string}|null;settlement:RuntimeDelegationSettlement}}
export interface RuntimeDelegationInput{petId:string;goalId:string;decisionId:string}
export interface RuntimePeerFeedback extends RuntimeDelegationInput{schemaVersion:1;scope:'nativeUserExactDelegationOutcomeEvidence';parentGoalRevision:string;parentGoalHash:string;parentGrantHash:string;observedAt:number;items:Array<{messageId:string;scopeHash:string;recipientPetId:string;status:'notAccepted'|'withdrawn'|'observed'|'unavailable';binding:{goalId:string;petId:string;decisionId:string;admissionHash:string;boundAt:number}|null;child:null|{goalId:string;petId:string;goalRevision:string;goalHash:string;grantHash:string;state:RuntimeReceivedWork['state'];requestWithinDeadline:boolean;evidenceComplete:boolean;criteriaVerified:boolean;goalComplete:boolean;modelAttempts:Array<{attemptId:string;phase:string;chargedOutputBytes:number;outcomeKnown:boolean;resultEvidence:Array<{kind:'modelResult';reference:string;status:'verified'|'missing'|'invalid'|'unavailable';hash:string|null;reason:string|null;outcomeKnown:boolean;criteriaVerified:boolean}>}>};reason:string|null}>;evidenceComplete:boolean;executionAuthorized:false;modelUseAuthorized:false}
export interface RuntimeDelegationView extends RuntimeDelegationInput{schemaVersion:2;grantHash:string;observedAt:number;historical:boolean;reservations:RuntimeDelegationReservation[];held:RuntimeDelegationAllowance|null;consumed:RuntimeDelegationAllowance|null;unknown:RuntimeDelegationAllowance|null;remaining:RuntimeDelegationAllowance|null;executionAuthorized:false;recipientAdmissionSupport:'nativeDeterministicRequiresOwnReview';recipientAdmissionAvailable:false;limitation:string}
export interface RuntimeDelegationWithdrawalTicket{token:string;proposal:RuntimeDelegationInput&{schemaVersion:1;reservation:RuntimeDelegationReservation;subgoal:string;recipient:string;sourceGrantHash:string;reviewId:string;expiresAt:number;scope:'withdrawOnlyUnboundDelegationReservation'}}
export type RuntimeDelegationWithdrawalResult={status:'withdrawn';view:RuntimeDelegationView}|{status:'declined'}|{status:'blocked';reason:string;retained:boolean};
export interface RuntimeDelegationBranchTicket{token:string;proposal:RuntimeDelegationInput&{schemaVersion:1|2;action:'cancel'|'reconcile';reservation:RuntimeDelegationReservation;sourceGrantHash:string;childGrantHash:string;childRevision:string;childPhase:string;consumed:RuntimeDelegationAllowance|null;unknown:RuntimeDelegationAllowance|null;reviewId:string;expiresAt:number;scope:'exactBoundChildOnly;noNewExecutionAuthority;unknownConsumptionRetained'|'exactBoundSubtree;persistIntentThenStop;noNewAuthority;descendantUnknownConsumptionRetained'}}
export type RuntimeDelegationBranchResult={status:'updated';view:RuntimeDelegationView;stopProjection:string|null}|{status:'declined'}|{status:'blocked';reason:string;retained:boolean};

export interface RuntimeFunding{schemaVersion:1|2|3|4|5|6|7|8|9;previous?:RuntimeContinuationReference['previous'];source:RuntimeReceivedGoalSource;allowance:RuntimeDelegationAllowance;ownOriginHash:string}


export type PetSourceOwner = {petId:string;conversationId:string;scope?:never}|{petId:string;scope:'pet';conversationId?:never}
export interface PetCalendarSource {id:string;revision:number;kind:'ics';name:string;path:string;enabled:boolean;owners:PetSourceOwner[]}
export interface PetSourceRegistry {version:1|2|3;generation:number;sources:PetCalendarSource[]}
export interface PetSourceDocument {config:PetSourceRegistry;revision:string}


export interface RuntimeRuleSchedule{kind:'daily';firstDate:string;days:number;localTime:string;timeZone:string;weekdays:number[];overlap:'earlier'|'later'|'skip'|'decision';gap:'skip'|'decision'}
export interface RuntimeRuleDefinition{name:string;intent:string;criteria:string[];schedule:RuntimeRuleSchedule;missed:'skip'|'coalesce'|'decision';graceSeconds:number;dataScope:'none';limits:{maxOccurrences:number;maxModelCalls:number;maxToolCalls:number;maxOutputBytes:number;occurrenceOutputBytes:number;occurrenceTimeoutSeconds:number}}
export interface RuntimeRule{id:string;revision:number;owner:{petId:string;provider:string;adapter:string;modelHint?:string|null};state:'draft';createdAt:number;updatedAt:number;definition:RuntimeRuleDefinition}
export interface RuntimeRuleDocument{schemaVersion:1;revision:string;rules:RuntimeRule[];executionAvailable:false}
export interface RuntimeRulePreview{rule:RuntimeRule;schedule:{schemaVersion:1;schedule:RuntimeRuleSchedule;timeZoneDataVersion:string;resolverRevision:string;slots:Array<{localDateTime:string;resolution:'ordinary'|'earlier'|'later'|'skippedGap'|'skippedOverlap'|'decisionGap'|'decisionOverlap';at:number|null;offsetSeconds:number|null}>;executionAuthorized:false};observedAt:number;backlog:{selectedAt:number|null;skipped:number;coalesced:number;needsDecision:number};executionAvailable:false}


export type RuntimeRuleScope='boundedRuleRecurrenceWithoutSelectedData'|'boundedAutomaticRuleRecurrenceWithoutSelectedData';
export interface RuntimeRulePolicy{dispatch?:{schemaVersion:1;maxMetadataProbes:3;metadataTimeoutMillis:6000;retryDelayMillis:30000};schemaVersion:1|2;scope:RuntimeRuleScope;rule:RuntimeRule;payment:'localOnly';destination:{kind:'server';provider:string;endpoint:string;model:string};model:string;engineBinding:unknown;policyRevision:string;descriptorSemantics:string;schedule:RuntimeRulePreview['schedule'];preparedAt:number;validUntil:number}
export interface RuntimeRuleAuthorityTicket{token:string;proposal:RuntimeRulePolicy;expiresAt:number;approvalScope:RuntimeRuleScope;dispatchAvailable:false}
export interface RuntimeRuleReservation{readiness?:Array<{id:string;scopeHash:string;preparedAt:number;expiresAt:number;finishedAt:number|null;observed:boolean|null}>;admission?:{schemaVersion:1;goalId:string;decisionId:string;sourceRevision:string;sourceSequence:number;approvedAt:number;descriptorRevision:string;evidenceHash:string;specHash:string};occurrenceId:string;slotIndex:number;at:number;deadlineAt:number;reservedAt:number;modelCalls:1;toolCalls:0;outputBytes:number}
export interface RuntimeRuleAuthorityStatus{lineage?:{rootDecisionId:string;predecessorDecisionId:string;generation:number;originalApprovedAt:number;inheritedOccurrences:number;inheritedOutputBytes:number;allocation?:{rootDecisionId:string;approvedAt:number;validUntil:number;maxOccurrences:number;committedOccurrences:number;remainingOccurrences:number;cumulativeCommittedOccurrences:number;cumulativeCommittedOutputBytes:number;futureAfter:number}};observedAt?:number;schemaVersion:1;ruleId:string;decisionId:string|null;phase:'notAuthorized'|'reconciliationRequired'|'restoredSnapshotOnly'|'revoked'|'authorizedNotDispatched'|'configurationChanged'|'authorizedScheduled'|'readinessPendingOrUncertain'|'expired'|'authorityTimeUncertain';policy:RuntimeRulePolicy|null;committedOccurrences:number;remainingOccurrences:number|null;committedModelCalls:number;committedOutputBytes:number;allocations:RuntimeRuleReservation[];pendingArtifacts:number;snapshotOnly:boolean;executionAvailable:false;dispatchAvailable:boolean;automaticStatus?:{observedAt:number;phase:string;reason:string|null}}
export type RuntimeRuleAuthorityConfirmation={status:'declined'}|{status:'recorded';authority:RuntimeRuleAuthorityStatus};

export type RuntimeRuleRecoveryConsent='reconcileInterruptedRuleReadinessWithoutReplayingIo';
export interface RuntimeRuleRecoveryTicket{token:string;expiresAt:number;approvalScope:RuntimeRuleRecoveryConsent;executionAuthorityGranted:false;proposal:{schemaVersion:1;ownerWitness?:'sameBackendOperationEnded'|'originalProcessAbsent';sourceHash:string;ruleId:string;petId:string;occurrenceId:string;probeId:string;parentDecisionId:string;parentRevision:string;pendingIntent:boolean;originalCompletionAvailable:boolean;originalDeadlineAt:number;metadataProbesSpent:number;metadataProbesLimit:3;model:string;destination:{kind:'server';provider:string;endpoint:string;model:string};selectedSignedSource:unknown;canonicalHash:string;stagedHash:string|null;executionAuthorityGranted:false;ioWillReplay:false}}

export type RuntimeRuleIntentRetirementConsent='reconcileRevokedRuleMetadataIntentsWithoutNewAuthority';
export interface RuntimeRuleIntentRetirementTicket{token:string;expiresAt:number;approvalScope:RuntimeRuleIntentRetirementConsent;executionAuthorityGranted:false;proposal:{schemaVersion:1;sourceHash:string;ruleId:string;petId:string;parentDecisionId:string;parentRevision:string;canonicalHash:string;pendingIntents:number;intentDigests:{name:string;bodyHash:string}[];recordedChoice:boolean;committedOccurrences:number;unknownReadinessProbes:number;originalRevokedAt:number;originalValidUntil:number;model:string;destination:{kind:'server';provider:string;endpoint:string;model:string};selectedSignedSource:unknown;executionAuthorityGranted:false;workerEndProven:false;resourcesReleased:false;ioWillReplay:false}}

export type RuntimeRuleSuccessionConsent='continueRevokedRuleWithOriginalRemainingAllocationAndDeadline';
export interface RuntimeRuleSuccessionTicket{token:string;expiresAt:number;approvalScope:RuntimeRuleSuccessionConsent;executionAuthorityGranted:false;proposal:{schemaVersion:1;sourceHash:string;ruleId:string;petId:string;predecessorDecisionId:string;rootDecisionId:string;generation:number;recordedChoice:boolean;originalApprovedAt:number;originalValidUntil:number;committedOccurrences:number;remainingOccurrences:number;committedOutputBytes:number;unknownReadinessProbes:number;originalPolicy:RuntimeRulePolicy;signedChain:unknown[];stagedChoice:unknown;oldReferencesRemainRevoked:true;freshBudgetGranted:false;deadlineRenewed:false;workerEndProven:false;executionAuthorityGranted:false}}

export type RuntimeRuleRevisionMode='carryRemaining'|'renewFiniteAllocation';
export type RuntimeRuleRevisionConsent='reviseRuleWithinRemainingAllocationAndDeadline'|'renewRuleWithReviewedFiniteAllocation';
export interface RuntimeRuleRevisionTicket{token:string;expiresAt:number;approvalScope:RuntimeRuleRevisionConsent;executionAuthorityGranted:false;proposal:{schemaVersion:1;sourceHash:string;ruleId:string;petId:string;mode:RuntimeRuleRevisionMode;proposedPolicy:RuntimeRulePolicy;previousPolicy:RuntimeRulePolicy;predecessorDecisionId:string;rootDecisionId:string;generation:number;recordedChoice:boolean;allocationRootDecisionId:string;originalAllocationValidUntil:number;currentAllocationCommittedOccurrences:number;currentAllocationRemainingOccurrences:number;proposedAvailableOccurrences:number;oldUnusedAuthorityRetired:boolean;oldUnusedOccurrencesRetired:number;cumulativeCommittedOccurrences:number;eligibleFutureSlotsAtPreparation:number;futureOnlyAtApproval:true;oldReferencesRemainRevoked:true;workerEndProven:false;executionAuthorityGranted:false;signedChain:unknown[];stagedChoice:unknown}}
