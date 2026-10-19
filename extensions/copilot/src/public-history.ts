import type { SessionEvent } from "@github/copilot-sdk";
import { BridgeError, MAX_HISTORY_BYTES, record, type Message } from "./protocol.js";

const MAX_LIVE_MESSAGES = 4096;
const PUBLIC_PHASES = new Set(["answer", "response", "final", "final_answer", "commentary"]);
const INCOMPLETE_PUBLIC_MESSAGE = Symbol("incomplete public message");

function identity(value: unknown): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= 256
    && !/[\u0000-\u001f]/.test(value);
}

function publicPhase(phase: unknown): boolean {
  return phase === undefined || (typeof phase === "string" && PUBLIC_PHASES.has(phase));
}

function root(event: SessionEvent): boolean {
  return !event.agentId && !record(event.data)?.parentToolCallId;
}

/** Extract just the displayed root conversation. Never copy an event or its raw data. */
export function publicMessage(event: SessionEvent): Message | undefined {
  if (!root(event)) return;
  if (event.type === "user.message") {
    const { content, messageId, source, isAutopilotContinuation } = event.data;
    // Source tags can mark hidden skills, system notifications and inter-agent prompts.
    // This is a visibility filter, not an assertion of authenticated human authorship.
    if ((source !== undefined && source !== "user") || isAutopilotContinuation
        || typeof content !== "string") return;
    const id = identity(messageId) ? messageId : event.id;
    if (identity(id)) return { id, role: "user", content };
  }
  if (event.type === "assistant.message") {
    const { content, messageId, phase } = event.data;
    if (typeof content === "string" && content.length > 0
        && identity(messageId) && publicPhase(phase)) {
      return { id: messageId, role: "assistant", content };
    }
  }
  return;
}

export interface HistoryState {
  messages: Message[];
  turn?: string;
  model?: string;
  notice?: string;
}

/** In-memory overlay for public events received while a history RPC is in flight. */
export class PublicHistory {
  private readonly live = new Map<string, Message>();
  private readonly phases = new Map<string, unknown>();
  private readonly lost = new Set<string>();
  private readonly streamGaps = new Set<string>();
  private liveBytes = 0;
  private retentionOverflow = false;
  private untrackedStreamGap = false;
  private hiddenPhase = false;
  private observedTurn: string | undefined;
  private observedModel: string | undefined;
  private observedBusy: boolean | undefined;
  private revision = 0;

  get activity(): { revision: number; busy?: boolean; turn?: string } {
    return {
      revision: this.revision,
      ...(this.observedBusy !== undefined ? { busy: this.observedBusy } : {}),
      ...(this.observedTurn ? { turn: this.observedTurn } : {}),
    };
  }

  observe(event: SessionEvent): boolean {
    if (!root(event)) return false;
    if (event.type === "assistant.message_start") {
      const { messageId, phase } = event.data;
      if (!identity(messageId)) return false;
      if (!publicPhase(phase)) {
        this.hiddenPhase = true;
        this.dropAssistant(messageId);
      }
      if (this.phases.size >= MAX_LIVE_MESSAGES && !this.phases.has(messageId)) {
        // Only stream-classification metadata is missing. A later canonical
        // message can still supply its complete, explicitly public content.
        this.noteStreamGap(messageId);
        return true;
      } else {
        if (this.phases.get(messageId) !== INCOMPLETE_PUBLIC_MESSAGE) this.phases.set(messageId, phase);
      }
      return false;
    }
    if (event.type === "assistant.message_delta") {
      const { messageId, deltaContent } = event.data;
      if (!identity(messageId) || typeof deltaContent !== "string") return false;
      // An absent start is different from an observed start whose phase is
      // explicitly undefined (the native legacy public-message shape).
      if (!this.phases.has(messageId)) {
        this.noteStreamGap(messageId);
        return true;
      }
      if (this.untrackedStreamGap || this.streamGaps.has(messageId)
          || !publicPhase(this.phases.get(messageId))) return false;
      const previous = this.live.get(messageId);
      const bytes = Buffer.byteLength(deltaContent, "utf8");
      if (!this.canStore(messageId, bytes)) {
        // Never resume at a later suffix after dropping a public delta.
        this.phases.set(messageId, INCOMPLETE_PUBLIC_MESSAGE);
        return true;
      }
      this.live.set(messageId, {
        id: messageId, role: "assistant", content: (previous?.content ?? "") + deltaContent,
      });
      this.liveBytes += bytes;
      return true;
    }
    const message = publicMessage(event);
    if (message) {
      this.store(message);
      if (message.role === "user") {
        this.revision++;
        this.observedBusy = true;
      }
      if (event.type === "assistant.message") this.phases.delete(event.data.messageId);
      return true;
    }
    if (event.type === "assistant.message") {
      this.phases.delete(event.data.messageId);
      if (typeof event.data.content === "string" && identity(event.data.messageId)) {
        this.streamGaps.delete(event.data.messageId);
        this.lost.delete(event.data.messageId);
      }
      this.dropAssistant(event.data.messageId);
      if (!publicPhase(event.data.phase)) this.hiddenPhase = true;
      return true;
    }
    if (event.type === "assistant.turn_start") {
      this.revision++;
      this.observedBusy = true;
      this.observedTurn = identity(event.data.turnId) ? event.data.turnId : undefined;
      if (identity(event.data.model)) this.observedModel = event.data.model;
      return true;
    }
    if (event.type === "session.idle") {
      this.phases.clear();
      this.revision++;
      this.observedBusy = false;
      this.observedTurn = undefined;
      return true;
    }
    if (event.type === "assistant.turn_end") {
      if (!this.observedTurn || this.observedTurn === event.data.turnId) {
        this.phases.clear();
        this.observedTurn = undefined;
      }
      return true;
    }
    if (event.type === "session.snapshot_rewind") {
      // A native rewind owns the canonical log. A cached old message must not
      // reappear after the host removed it deliberately. Context compaction or
      // BasicTruncator events alone do not remove the public transcript.
      this.live.clear();
      this.phases.clear();
      this.lost.clear();
      this.streamGaps.clear();
      this.liveBytes = 0;
      this.retentionOverflow = false;
      this.untrackedStreamGap = false;
      this.observedTurn = undefined;
      return true;
    }
    return event.type === "assistant.idle"
      || event.type === "session.compaction_complete" || event.type === "session.mode_changed"
      || event.type === "session.model_change" || event.type === "permission.requested"
      || event.type === "pending_messages.modified" || event.type === "session.truncation"
      || event.type === "session.context_cleared";
  }

  rememberSent(id: string, content: string): void {
    if (identity(id)) this.store({ id, role: "user", content });
  }

  forgetDraft(id: string): void {
    const draft = this.live.get(id);
    if (draft?.role === "user") {
      this.live.delete(id);
      this.liveBytes -= Buffer.byteLength(draft.content, "utf8");
      this.lost.delete(id);
    }
  }

  project(events: SessionEvent[]): HistoryState {
    if (!Array.isArray(events)) {
      throw new BridgeError("invalid_history", "O SDK não devolveu um histórico válido.");
    }
    const messages: Message[] = [];
    const indexes = new Map<string, number>();
    let bytes = 0;
    let turn: string | undefined;
    let model: string | undefined;
    const append = (message: Message) => {
      const index = indexes.get(message.id);
      const old = index === undefined ? undefined : messages[index];
      bytes += Buffer.byteLength(message.content, "utf8") - (old ? Buffer.byteLength(old.content, "utf8") : 0);
      if (bytes > MAX_HISTORY_BYTES || messages.length >= 1_000_000) {
        throw new BridgeError("history_too_large", "O histórico público excede o limite de 128 MiB; não foi truncado.");
      }
      if (index === undefined) {
        indexes.set(message.id, messages.length);
        messages.push(message);
      } else {
        messages[index] = message;
      }
    };
    for (const event of events) {
      const message = publicMessage(event);
      if (message) {
        append(message);
        this.lost.delete(message.id);
        this.streamGaps.delete(message.id);
      } else if (root(event) && event.type === "assistant.message"
          && identity(event.data.messageId) && typeof event.data.content === "string") {
        // A canonical private or empty final message also proves that no
        // public text for this ID should be reconstructed from earlier deltas.
        this.dropAssistant(event.data.messageId);
        this.lost.delete(event.data.messageId);
        this.streamGaps.delete(event.data.messageId);
      }
      if (root(event) && event.type === "assistant.turn_start") {
        turn = identity(event.data.turnId) ? event.data.turnId : undefined;
        if (identity(event.data.model)) model = event.data.model;
      } else if (root(event) && event.type === "assistant.turn_end" && event.data.turnId === turn) {
        turn = undefined;
      }
    }
    if (this.lost.size > 0) {
      throw new BridgeError("live_history_overflow", "A retenção em direto atingiu o limite de 4096 mensagens ou 128 MiB; o SDK ainda não confirmou todo o texto público em falta.");
    }
    for (const [id, message] of this.live) {
      if (indexes.has(id)) {
        // Persisted complete messages win over a live partial or queued draft.
        this.live.delete(id);
        this.liveBytes -= Buffer.byteLength(message.content, "utf8");
      } else {
        append({ ...message });
      }
    }
    const selectedTurn = this.observedTurn ?? turn;
    const selectedModel = this.observedModel ?? model;
    const notices = [
      ...(this.hiddenPhase ? ["Foram omitidas fases privadas ou não reconhecidas do modelo."] : []),
      ...(this.streamGaps.size > 0 || this.untrackedStreamGap
        ? ["Foi omitido texto em direto cujo início público não foi observado; apenas mensagens canónicas validadas podem completá-lo."] : []),
      ...(this.retentionOverflow
        ? ["A atualização em direto excedeu o limite de retenção de 4096 identificadores; não foi possível verificar todos os fragmentos omitidos."] : []),
    ];
    return {
      messages,
      ...(selectedTurn ? { turn: selectedTurn } : {}),
      ...(selectedModel ? { model: selectedModel } : {}),
      ...(notices.length > 0 ? { notice: notices.join(" ") } : {}),
    };
  }

  private canStore(id: string, bytes: number): boolean {
    if (this.liveBytes + bytes <= MAX_HISTORY_BYTES
        && (this.live.has(id) || this.live.size < MAX_LIVE_MESSAGES)) return true;
    if (!this.lost.has(id)) {
      if (this.lost.size < MAX_LIVE_MESSAGES) this.lost.add(id);
      else this.retentionOverflow = true;
    }
    return false;
  }

  private store(message: Message): void {
    const old = this.live.get(message.id);
    const oldBytes = old ? Buffer.byteLength(old.content, "utf8") : 0;
    const newBytes = Buffer.byteLength(message.content, "utf8");
    if (!this.canStore(message.id, newBytes - oldBytes)) return;
    this.live.set(message.id, message);
    this.liveBytes += newBytes - oldBytes;
    this.lost.delete(message.id);
    this.streamGaps.delete(message.id);
  }

  private noteStreamGap(id: string): void {
    if (this.streamGaps.has(id)) return;
    if (this.streamGaps.size < MAX_LIVE_MESSAGES) this.streamGaps.add(id);
    else this.untrackedStreamGap = true;
  }

  private dropAssistant(id: string): void {
    const previous = this.live.get(id);
    if (previous?.role !== "assistant") return;
    this.live.delete(id);
    this.liveBytes -= Buffer.byteLength(previous.content, "utf8");
  }
}
