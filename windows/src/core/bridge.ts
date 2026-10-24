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
import type {PetConfiguration,PetDocument} from "../mochi/pet-preferences";
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
export interface AgentRequest {
  id:string;provider:string;kind:string;detail:string;origin?:string;
  approvalScope?:'turn'|'once'|'recipe';
  goalTarget?:GoalRecipeTarget;goalReview?:{recipe:RuntimeRecipeProposal;budget:unknown;approvalScope:'exactReviewedRecipe';executionStarted:false;modelPlanningGranted:false};voiceCode?:string;
  nativeChoices?:NativeApprovalChoice[];nativeChoicesSource?:'nativeAdvertised'|'protocolFallback'|'nativePermissionProtocol'|'nativeOptions'|'nativeLegacyProtocol';
  target?:import('./session-chat').SessionTarget;
  nativeForm?:'mcp';
  questions?:AgentQuestion[]|null;
}
export interface GoalRecipeTarget {origin:'goal';petId:string;provider:string;goalId:string;contextId:string;leaseId:string;controllerId:string;recipeId:string;planId:string;revision:string}
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
  chatSend: (query: string, context: ChatContext | null, provider?: string,conversationId?:string,selectedModel?:string,documentSequence?:number) =>
    callOrThrow<{ text: string }>("chat_send", { query, context, provider,...(conversationId===undefined?{}:{conversationId}),...(selectedModel===undefined?{}:{selectedModel}),...(documentSequence===undefined?{}:{documentSequence}) }),
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
  localSetKey: (url: string, key: string) => callOrThrow<void>("local_set_key", { url, key }),

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
  petRuntimeCreate:(input:RuntimeCreateInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_create',{input:{petId:input.petId,requestId:input.requestId,intent:input.intent,criteria:[...input.criteria],conversationId:input.conversationId??null}}),
  petRuntimeGoalPending:()=>callOrThrow<AgentRequest[]>('pet_runtime_goal_pending'),
  petRuntimeGoalAnswer:(id:string,approve:boolean,target:GoalRecipeTarget)=>callOrThrow<void>('pet_runtime_goal_answer',{input:{id,approve,target:structuredClone(target)}}),
  petRuntimeRefreshReadiness:(petId:string,goalId:string)=>callOrThrow<PetCapabilityDescriptor>('pet_runtime_refresh_readiness',{petId,goalId}),
  petRuntimeList:(petId?:string)=>callOrThrow<RuntimeListing>('pet_runtime_list',{petId}),
  petRuntimePrepareContext:(input:RuntimePrepareContextInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_prepare_context',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,mode:input.mode}}),
  petRuntimeContextRead:(petId:string,goalId:string)=>callOrThrow<RuntimeContextView>('pet_runtime_context_read',{petId,goalId}),
  petRuntimeClaim:(input:RuntimeClaimInput)=>callOrThrow<RuntimeClaimResult>('pet_runtime_claim',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId}}),
  petRuntimeClaimStatus:(petId:string,goalId:string)=>callOrThrow<RuntimeLeaseStatus>('pet_runtime_claim_status',{petId,goalId}),
  petRuntimeReleaseClaim:(petId:string,goalId:string,leaseId:string)=>callOrThrow<RuntimeReceipt>('pet_runtime_release_claim',{input:{petId,goalId,leaseId}}),
  petRuntimeRespondOnce:(input:RuntimeRespondInput)=>callOrThrow<RuntimeRespondResult>('pet_runtime_respond_once',{input:{petId:input.petId,goalId:input.goalId,leaseId:input.leaseId,revision:input.revision,requestId:input.requestId}}),
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
  petRuntimePrepareAdmission:(input:RuntimeAdmissionInput)=>callOrThrow<RuntimeAdmissionReview>('pet_runtime_prepare_admission',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,budget:runtimeBudget(input.budget)}}),
  petRuntimeConfirmAdmission:(input:RuntimeAdmissionConfirm)=>callOrThrow<RuntimeAdmissionResult>('pet_runtime_confirm_admission',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,token:input.token,approve:input.approve}}),
  petRuntimeRead:(petId:string,goalId:string)=>callOrThrow<RuntimeGoal>('pet_runtime_read',{petId,goalId}),
  petRuntimeMutate:(input:RuntimeMutationInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_mutate',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,action:runtimeDraftAction(input.action)}}),
  petRuntimeImportLegacy:(input:RuntimeImportInput)=>callOrThrow<RuntimeReceipt>('pet_runtime_import_legacy',{input:{petId:input.petId,goalId:input.goalId,revision:input.revision,requestId:input.requestId,taskId:input.taskId}}),

  petTargetCapabilities:(target:import('./session-chat').SessionTarget)=>callOrThrow<NativeTargetCapabilities>('pet_capabilities_target',{target}),
  petRoster:()=>callOrThrow<string[]>("pet_roster"),
  petConfigurationSave: (config:PetConfiguration,expectedRevision:string) => callOrThrow<PetDocument>("pet_configuration_save",{config,expectedRevision}),
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
  | "pending" | "refused";

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
 binding:{adapter:string|null;provider:string|null;model:string|null;profile:string|null};revision:string;observedAt:number|null;configurationAvailable:boolean;
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
 schemaVersion:1|2|3|4|5|6|7;id:string;revision:string;sequence:number;createdAt:number;updatedAt:number;executionContext?:RuntimeContextBinding;admissions?:Array<{spec:RuntimeAdmissionSpec;phase:'queued'|'paused'|'cancelled'|'leased'|'stopRequested'|'recoveryRequired'|'completed';proof?:RuntimeConsentProof;lease?:RuntimeLease;stopReason?:'pause'|'cancel'|'expired'}>;
 spec:{requestId:string;owner:{petId:string;provider:string;adapter:string;modelHint:string|null};context:{kind:'unbound'}|{kind:'mochi';conversationId:string};intent:string;criteria:string[]};
 completion?:RuntimeCompletionRef;state:RuntimeState;tasks:Array<{id:string;description:string;state:RuntimeWorkState;planId:string|null}>;
 plans:Array<{id:string;taskId:string;steps:Array<RuntimeStepInput&{id:string;state:RuntimeWorkState}>}>;
 events:Array<{sequence:number;at:number;requestId:string;fingerprint:string;kind:'created'|'taskProposed'|'planProposed'|'paused'|'draftResumed'|'cancelled'|'cancelRequested'|'legacyImported'|'contextPrepared'|'admissionQueued'|'leaseClaimed'|'stopRequested'|'leaseDrained'|'claimAbandoned'|'goalCompleted'}>;
 legacy:Array<{taskId:string;provider:string;backend:string;conversationId:string|null;phase:string;intentMayBeIncomplete:true;receipts:import('../mochi/tasks').LocalStepReceipt[]}>;
}
export interface RuntimeLease {schemaVersion:1;id:string;controllerId:string;requestId:string;decisionId:string;goalId:string;provider:string;conversationId:string;model:string;policyRevision:string;capabilityRevision:string;sourceRevision:string;claimedAt:number;deadlineAt:number}
export interface RuntimeClaimInput {petId:string;goalId:string;revision:string;requestId:string}
export interface RuntimeClaimResult {receipt:RuntimeReceipt;lease:RuntimeLease;owned:boolean;reservationOwned:boolean;executionAvailable:false}
export interface RuntimeLeaseStatus {record:RuntimeGoal;lease:RuntimeLease|null;owned:boolean;reservationOwned:boolean;source:'liveController'|'storedLease'|'none';current:boolean;workers:number|null;executionAvailable:false}
export interface RuntimeConsentProof {schemaVersion:1;keyId:string;operation:'conversationRespond';tag:string}
export interface RuntimeProvenanceStatus {verified:boolean;reason:'missing'|'unsafe'|'changed'|'invalid'|'wrongKey'|'revoked'|'expired'|'notQueued'|'capacity'|'unavailable'|'uncertain'|'busy'|'recoveryRequired'|null;executionAvailable:false}
export interface RuntimeBudget {deadlineAt:number;maxModelCalls:number;maxToolCalls:number;maxOutputBytes:number;maxTokens?:number|null;maxCostMicros?:number|null;payment:'localOnly'|'existingAccount'}
export interface RuntimeAdmissionInput {petId:string;goalId:string;revision:string;requestId:string;budget:RuntimeBudget}
export interface RuntimeAdmissionConfirm {petId:string;goalId:string;revision:string;token:string;approve:boolean}
export interface RuntimeAdmissionSpec {schemaVersion:1|2;proposalHash?:string|null;requestId:string;decisionId:string;goalId:string;sourceRevision:string;sourceSequence:number;owner:RuntimeGoal['spec']['owner'];context:RuntimeContextBinding;contextHash:string;intentHash:string;resolvedModel:string;descriptorRevision:string;policyRevision:string;evidenceHash:string;approvedAt:number;budget:RuntimeBudget}
export interface RuntimeAdmissionProposal {proposals:RuntimeReviewedProposals;operation:'conversationRespond';toolExecutionAvailable:false;goalId:string;revision:string;requestId:string;owner:RuntimeGoal['spec']['owner'];context:RuntimeContextBinding;intent:string;criteria:string[];resolvedModel:string;policyRevision:string;descriptorRevision:string;budget:RuntimeBudget;preparedAt:number;expiresAt:number}
export type RuntimeAdmissionDenial='invalid'|'ownerChanged'|'contextRequired'|'contextChanged'|'notDraft'|'inactive'|'unsupported'|'notReady'|'stale'|'modelUnresolved'|'policyChanged'|'budget'|'paymentPolicy'|'meteringUnavailable'|'expired'|'revisionChanged'|'inputLimit';
export type RuntimeAdmissionReview={status:'review';ticket:{token:string;proposal:RuntimeAdmissionProposal}}|{status:'blocked';reason:RuntimeAdmissionDenial};
export type RuntimeAdmissionResult={status:'queued';receipt:RuntimeReceipt}|{status:'declined'}|{status:'blocked';reason:RuntimeAdmissionDenial}|{status:'authorityBlocked';reason:NonNullable<RuntimeProvenanceStatus['reason']>};
export interface RuntimeReceipt {status:'durable'|'uncertain'|'unchanged';record:RuntimeGoal}
export interface RuntimeRespondInput {petId:string;goalId:string;leaseId:string;revision:string;requestId:string}
export interface RuntimeRespondResult {text:string;criteriaVerified:false;goalComplete:false}
export interface RuntimeResultInput {petId:string;goalId:string;decisionId:string;requestId:string}
export interface RuntimeModelResult {schemaVersion:1;goalId:string;petId:string;provider:string;decisionId:string;leaseId:string;requestId:string;attemptId:string;model:string;inputHash:string;intentHash:string;text:string;acceptedOutputBytes:number;completedAt:number;recordedAt:number}
export type RuntimeResult=({status:'notStarted'}|{status:'recoveryRequired';phase:'reserved'|'started'|'uncertain'|'settledReceiptMissing'}|{status:'available';receipt:RuntimeModelResult})&{criteriaVerified:false;goalComplete:false};
export interface RuntimeListing {schemaVersion:1;goals:Array<{id:string;revision:string;petId:string;state:RuntimeState;updatedAt:number;intentPreview:string;tasks:number;steps:number}>;unavailable:number;pendingArtifacts:number;complete:boolean}

export interface RuntimePrepareContextInput {petId:string;goalId:string;revision:string;requestId:string;mode:'dedicated'|'snapshot'}
export interface RuntimeContextBinding {schemaVersion:1;goalId:string;petId:string;provider:string;conversationId:string;requestId:string;mode:'dedicated'|'snapshot';sourceConversationId:string|null;sourceMessages:number;copiedMessages:number;seedHash:string;preparedAt:number}
export interface RuntimeContextView {binding:RuntimeContextBinding;messages:Array<{role:'user'|'assistant';content:string}>;messageCount:number;truncated:boolean}

/** Step receipts are observations, never reconstructible execution permits. */
export interface RuntimeStepPrepareInput {petId:string;goalId:string;leaseId:string;revision:string;planId:string;stepId:string;requestId:string}
export interface RuntimeStepConfirmInput {petId:string;goalId:string;leaseId:string;revision:string;token:string;approve:boolean}
export interface RuntimeStepRunInput {petId:string;goalId:string;leaseId:string;grantId:string;planId:string;stepId:string}
export interface RuntimeStepReadInput {petId:string;goalId:string;decisionId:string;planId:string;stepId:string}
export interface RuntimeStepRecoverInput extends RuntimeStepReadInput {revision:string;grantId:string}
export interface RuntimeStepScope {schemaVersion:1;grantId:string;requestId:string;goalId:string;petId:string;provider:string;decisionId:string;leaseId:string;controllerId:string;mainSpecHash:string;contextHash:string;proposalHash:string;policyRevision:string;planId:string;stepId:string;operationId:'spotify.control';version:1;argumentsHash:string;resource:{schemaVersion:1;busId:string;owner:string;uid:number;pid:number;identity:string};approvedAt:number;deadlineAt:number;maxOutputBytes:number}
export interface RuntimeStepProposal {scope:RuntimeStepScope;goalRevision:string;intent:string;criteria:string[];arguments:unknown;postcondition:string;preparedAt:number;expiresAt:number;maxUses:1;goalCriteriaVerified:false}
export interface RuntimeStepTicket {token:string;proposal:RuntimeStepProposal}
export type RuntimeStepConfirmResult={status:'granted';scope:RuntimeStepScope}|{status:'declined'}|{status:'blocked';reason:NonNullable<RuntimeProvenanceStatus['reason']>};
export interface RuntimeStepSnapshot {playbackStatus:'Playing'|'Paused'|'Stopped';volume:number;observedAt:number}
export interface RuntimeStepReceipt {observation:{delivery:'notDispatched'|'accepted'|'uncertain';before:RuntimeStepSnapshot|null;after:RuntimeStepSnapshot|null;effectVerified:boolean;scopeCurrent:boolean;failure:'invalidAddress'|'unavailable'|'changed'|'invalidState'|'cancelled'|'deadline'|'invalidAction'|'notAuthorized'|'transport'|null};finishedAt:number;criteriaVerified:false;goalComplete:false}
export interface RuntimeStepRecovery {status:'prepared'|'reserved'|'resultMissing'|'resultRecorded'|'finished';budgetPhase:'reserved'|'started'|'settled'|'uncertain'|null;recordedEffectVerified:boolean|null;reconciliationAvailable:boolean}
export interface RuntimeStepView {scope:RuntimeStepScope;phase:'approved'|'reserved'|'started'|'finished';result:RuntimeStepReceipt|null;recovery:RuntimeStepRecovery;executionAvailable:false}
export interface RuntimeRecipePrepareInput {petId:string;goalId:string;leaseId:string;revision:string;planId:string;requestId:string;criteria?:RuntimeCriterionInput[]}
export type RuntimeRecipeConfirmInput=RuntimeStepConfirmInput;
export interface RuntimeRecipeRunInput {petId:string;goalId:string;leaseId:string;revision:string;recipeId:string}
export interface RuntimeRecipeReadInput {petId:string;goalId:string;decisionId:string;planId:string}
export interface RuntimeRecipeScope {schemaVersion:1|2;id:string;requestId:string;goalId:string;petId:string;provider:string;decisionId:string;leaseId:string;controllerId:string;planId:string;proposalHash:string;approvedAt:number;deadlineAt:number;steps:RuntimeStepScope[];verification?:RuntimeCriteriaBinding}
export interface RuntimeRecipeProposal {scope:RuntimeRecipeScope;goalRevision:string;intent:string;criteria:string[];steps:RuntimeStepProposal[];preparedAt:number;expiresAt:number;criteriaVerified:false;goalComplete:false}
export interface RuntimeRecipeTicket {token:string;proposal:RuntimeRecipeProposal}
export type RuntimeRecipeConfirmation={status:'granted';scope:RuntimeRecipeScope}|{status:'declined'}|{status:'ended'}|{status:'blocked';reason:NonNullable<RuntimeProvenanceStatus['reason']>};
export interface RuntimeRecipeProgress {scope:RuntimeRecipeScope;phase:'reviewed'|'partial'|'verified'|'recoveryRequired';verifiedSteps:number;totalSteps:number;steps:Array<{stepId:string;phase:RuntimeStepView['phase']|null;recovery:RuntimeStepRecovery|null;result:RuntimeStepReceipt|null}>;verification:RuntimeVerification|null;criteriaVerified:boolean;goalComplete:boolean;executionAvailable:false}
export interface RuntimeRecipeResult {outcome:'verified'|'blocked'|'invalidated'|'recoveryRequired'|'criteriaUnmet'|'criteriaIncomplete'|'goalCompleted';progress:RuntimeRecipeProgress;completion:'durable'|'uncertain'|'unchanged'|null}

export type RuntimeCriterionPredicate={kind:'spotifyPlayback';status:'playing'|'paused'|'stopped'}|{kind:'spotifyVolume';percent:number};
export interface RuntimeCriterionInput {criterionIndex:number;predicate:RuntimeCriterionPredicate}
export interface RuntimeCriteriaBinding {schemaVersion:1;specHash:string;criteriaCount:number;maxObservations:1;conditions:Array<RuntimeCriterionInput&{criterionHash:string}>}
export interface RuntimeCompletionRef {schemaVersion:1;goalId:string;petId:string;recipeId:string;decisionId:string;leaseId:string;planId:string;sourceRevision:string;targetRevision:string;reportHash:string;observedAt:number}
export interface RuntimeVerification {schemaVersion:1;reference:RuntimeCompletionRef;recipeHash:string;specHash:string;sourceSequence:number;sourceUpdatedAt:number;sourceRecordHash:string;snapshot:RuntimeStepSnapshot|null;scopeCurrent:boolean;recordedAt:number;results:Array<{criterionIndex:number;criterionHash:string;status:'met'|'unmet'|'unknown'}>;criteriaVerified:boolean}
export interface RuntimeRecipeCompleteInput extends RuntimeRecipeReadInput {recipeId:string;revision:string}
