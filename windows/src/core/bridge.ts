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

export interface ConversationView {id:string;messages:{role:"user"|"assistant";content:string}[];busy?:boolean;turn?:number;stream?:string;query?:string;notice?:string;taskNotice?:import('./task-notice').TaskNotice;requestHistory?:import('./request-history').RequestRecord[];localExecution?:import('./local-capabilities').LocalExecutionCapability}
export interface ChatTurnState {provider:string;conversationId:string;turn:number;busy:boolean;query:string}
export interface AgentRequest {
  id:string;provider:string;kind:string;detail:string;origin?:string;
  approvalScope?:'turn'|'once';
  target?:import('./session-chat').SessionTarget;
  questions?:{id:string;question:string;isSecret?:boolean;options?:{label:string}[]}[]|null;
}
export interface ConversationSummary {id:string;provider:string;title:string;preview:string;created:number;updated:number;messageCount:number;current:boolean}

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

export const Bridge = {
  petModelSelect:(model:string,selectionId:string)=>callOrThrow<{provider:string;model:string;selectionId:string;generation:number}>('pet_model_select',{model,selectionId}),
  petFocusSession:(provider:string,id:string)=>callOrThrow<{message?:string}>('pet_focus_session',{provider,id}),
  boot: () => call<BootInfo>("boot"),
  companionsPresence: () => call<{running:string[];activity:Record<string,number>}>("companions_presence"),
  /** The system's languages as the webview sees them, for Rust's own texts (i18n.rs). */
  setSystemLanguages: (languages: string[]) => call<void>("set_system_languages", { languages }),

  saveSettings: (settings: Settings) => callOrThrow<void>("save_settings", { settings }),
  petCenterSettingsPatch:(request:import('./island-availability').IslandAvailabilityPatch)=>callOrThrow<import('./island-availability').IslandAvailabilityValues>('pet_center_settings_patch',{request:{...(request.petControlBar!==undefined?{petControlBar:request.petControlBar}:{}),...(request.petSystemCenter!==undefined?{petSystemCenter:request.petSystemCenter}:{})}}),
  reportSettingsError,

  cliAgentAnswer: (id: string, answer: Record<string, unknown>,target?:import('./session-chat').SessionTarget) => invoke<void>("cli_agent_answer", { id, answer,...(target?{target}:{}) }),
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
  provider?:string;
  conversationId?:string;
  sequence?:number;
  kind:"file"|"folder";
  entries:number;
  name: string;
  path: string;
  size: number;
}

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
