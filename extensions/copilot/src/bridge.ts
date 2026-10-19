import type { CopilotSession, SessionEvent } from "@github/copilot-sdk";
import type { Socket } from "node:net";
import {
  BridgeError, CAPABILITIES, deadline, MAX_CONNECTIONS, MAX_HISTORY_BYTES,
  MAX_REQUEST_BYTES, record, safeError, validId, validateRequest,
  type Request, type RequestId, type Snapshot, type Target,
} from "./protocol.js";
import { PublicHistory, publicMessage } from "./public-history.js";
import { listenPrivate, type Endpoint } from "./runtime.js";

const RPC_TIMEOUT_MS = 30_000;
const MAX_SEND_IDS = 4096;
const CANCEL_NOTICE = "As aprovações continuam no terminal Copilot. O SDK não permite cancelar um turno exato nesta ponte; use o terminal que detém a sessão.";

interface Connection {
  socket: Socket;
  fragments: Buffer[];
  fragmentBytes: number;
  pending: boolean;
  writing: boolean;
  dirty: boolean;
  outputTimer: NodeJS.Timeout | undefined;
}

interface PendingSend {
  messageId?: string;
  unknown: boolean;
}

/** A bridge instance owns one immutable SDK handle and never resumes/replays elsewhere. */
export class SessionBridge {
  private readonly history = new PublicHistory();
  private readonly connections = new Set<Connection>();
  private readonly admittedRequestIds = new Set<RequestId>();
  private readonly observedUserIds = new Set<string>();
  private readonly sentMessageIds = new Set<string>();
  private endpoint: Endpoint | undefined;
  private unsubscribe: (() => void) | undefined;
  private heartbeat: NodeJS.Timeout | undefined;
  private notificationTimer: NodeJS.Timeout | undefined;
  private historyRead: Promise<Snapshot> | undefined;
  private sendInFlight = false;
  private pendingSend: PendingSend | undefined;
  private stopped = false;
  private lastNativeBusy: boolean | undefined;
  private stateNotice: string | undefined;

  constructor(
    private readonly session: CopilotSession,
    readonly target: Target,
    private readonly cwd: string,
    private readonly parentPid: number,
    private readonly onStop: (error?: BridgeError) => void,
  ) {}

  async start(): Promise<void> {
    this.checkTarget();
    // Subscribe first, so public events cannot fall between the initial read and watching.
    this.unsubscribe = this.session.on(event => this.onEvent(event));
    await this.snapshot();
    this.checkTarget();
    this.endpoint = await listenPrivate(this.target, this.cwd, this.parentPid, socket => this.accept(socket));
    this.endpoint.server.on("error", () => this.stop(new BridgeError("endpoint_failed", "A ponte local foi desligada.")));
    this.heartbeat = setInterval(() => {
      try { this.checkTarget(); }
      catch { this.stop(new BridgeError("session_closed", "A sessão que alojava a extensão foi fechada.")); }
    }, 1_000);
    this.heartbeat.unref();
  }

  stop(error?: BridgeError): void {
    if (this.stopped) return;
    this.stopped = true;
    if (this.notificationTimer) clearTimeout(this.notificationTimer);
    if (this.heartbeat) clearInterval(this.heartbeat);
    this.unsubscribe?.();
    this.unsubscribe = undefined;
    for (const connection of this.connections) connection.socket.destroy();
    this.connections.clear();
    this.endpoint?.server.close();
    this.endpoint?.cleanup();
    this.onStop(error);
  }

  cleanupOnExit(): void { this.endpoint?.cleanup(); }

  private checkTarget(): void {
    if (this.stopped || process.env.SESSION_ID !== this.target.sessionId
        || this.session.sessionId !== this.target.sessionId || process.ppid !== this.parentPid) {
      throw new BridgeError("session_closed", "A sessão que alojava a extensão já não corresponde ao destino.");
    }
    try { process.kill(this.parentPid, 0); }
    catch { throw new BridgeError("session_closed", "O processo que alojava a extensão foi fechado."); }
  }

  private onEvent(event: SessionEvent): void {
    if (this.stopped) return;
    try { this.checkTarget(); }
    catch { this.stop(new BridgeError("session_closed", "A sessão que alojava a extensão foi fechada.")); return; }
    if (!event.agentId && event.type === "session.shutdown") {
      this.stop();
      return;
    }
    const message = publicMessage(event);
    if (message?.role === "user") {
      if (this.observedUserIds.size >= 4096) this.observedUserIds.clear();
      this.observedUserIds.add(message.id);
      this.sentMessageIds.delete(message.id);
      if (this.pendingSend?.messageId === message.id) this.pendingSend = undefined;
    }
    if (this.history.observe(event)) this.invalidate();
    if (!event.agentId && (event.type === "session.idle" || event.type === "pending_messages.modified"
        || event.type === "session.truncation" || event.type === "session.snapshot_rewind")) {
      // An accepted draft can be removed in the TUI before its user.message event.
      // Reconcile through SDK reads; never abort or replay the draft to resolve it.
      if (this.pendingSend?.messageId) void this.reconcilePending().catch(() => {});
    }
  }

  private async reconcilePending(): Promise<void> {
    const expected = this.pendingSend;
    if (!expected?.messageId) return;
    const [status, queue] = await deadline(
      Promise.all([this.session.rpc.metadata.isProcessing(), this.session.rpc.queue.pendingItems()]),
      RPC_TIMEOUT_MS, new BridgeError("status_timeout", "Não foi possível confirmar a fila da sessão."),
    );
    this.checkTarget();
    if (this.pendingSend !== expected || status.processing
        || queue.items.some(item => item.messageId === expected.messageId)
        || queue.withdrawableTurnMessageId === expected.messageId) return;
    const events = await deadline(this.session.getEvents(), RPC_TIMEOUT_MS,
      new BridgeError("history_timeout", "Não foi possível confirmar a mensagem na sessão."));
    this.checkTarget();
    if (this.pendingSend !== expected) return;
    // Either it is now persisted, or the owner removed it while the session became idle.
    const persisted = events.some(event => publicMessage(event)?.id === expected.messageId);
    this.pendingSend = undefined;
    if (!persisted) {
      this.history.forgetDraft(expected.messageId);
      this.sentMessageIds.delete(expected.messageId);
      this.stateNotice = "A mensagem aceite já não está na fila nem no histórico nativo; pode ter sido removida no terminal.";
    }
    this.invalidate();
  }

  private snapshot(): Promise<Snapshot> {
    if (this.historyRead) return this.historyRead;
    const read = this.readSnapshot();
    this.historyRead = read;
    void read.finally(() => { if (this.historyRead === read) this.historyRead = undefined; }).catch(() => {});
    return read;
  }

  private async readSnapshot(): Promise<Snapshot> {
    this.checkTarget();
    const before = this.history.activity.revision;
    const [events, native] = await Promise.all([
      deadline(this.session.getEvents(), RPC_TIMEOUT_MS,
        new BridgeError("history_timeout", "O SDK não devolveu o histórico dentro do prazo.")),
      deadline(this.session.rpc.metadata.isProcessing(), RPC_TIMEOUT_MS,
        new BridgeError("status_timeout", "O SDK não confirmou o estado da sessão dentro do prazo."))
        .catch(() => undefined),
    ]);
    this.checkTarget();
    const projected = this.history.project(events);
    const activity = this.history.activity;
    if (native && typeof native.processing === "boolean") {
      this.lastNativeBusy = activity.revision !== before && activity.busy !== undefined
        ? activity.busy : native.processing;
    } else {
      this.lastNativeBusy = activity.busy;
    }
    for (const event of events) {
      const message = publicMessage(event);
      if (message?.role === "user" && this.pendingSend?.messageId === message.id) {
        this.pendingSend = undefined;
        this.sentMessageIds.delete(message.id);
      }
    }
    const busy = this.sendInFlight || this.pendingSend !== undefined || (this.lastNativeBusy ?? projected.turn !== undefined);
    const notices = [
      CANCEL_NOTICE,
      projected.notice,
      this.stateNotice,
      ...(this.session.transcriptRecovery ? ["O runtime recuperou um registo de sessão danificado; só os eventos conservados estão disponíveis."] : []),
      ...(native ? [] : ["O SDK não confirmou o estado atual; o indicador de atividade usa os eventos observados."]),
      ...(this.pendingSend?.unknown ? ["A entrega da última mensagem está por confirmar; não a reenvie automaticamente."] : []),
    ].filter((notice): notice is string => Boolean(notice));
    return {
      target: this.target,
      messages: projected.messages,
      busy,
      ...(busy && !this.pendingSend && projected.turn ? { turn: projected.turn } : {}),
      capabilities: { ...CAPABILITIES, send: !this.pendingSend?.unknown },
      notice: notices.join(" "),
      ...(projected.model ? { model: projected.model } : {}),
    };
  }

  private async send(request: Request, connected: () => boolean): Promise<{ target: Target; messageId: string }> {
    this.checkTarget();
    if (!connected()) throw new BridgeError("request_closed", "A ligação que pediu o envio já foi fechada.");
    if (this.sendInFlight || this.pendingSend) {
      throw new BridgeError("busy", "A sessão já tem trabalho ativo ou uma entrega por confirmar.");
    }
    if (this.admittedRequestIds.has(request.id)) {
      throw new BridgeError("duplicate_send", "Este pedido de envio já foi processado; não foi repetido.");
    }
    this.sendInFlight = true;
    this.invalidate();
    let admission: PendingSend | undefined;
    try {
      // This guard is local to this extension. Native TUI activity can race it; enqueue
      // remains the documented safe delivery mode and never interrupts that activity.
      const native = await deadline(this.session.rpc.metadata.isProcessing(), RPC_TIMEOUT_MS,
        new BridgeError("status_timeout", "Não foi possível confirmar se a sessão está ocupada."));
      this.checkTarget();
      if (!connected()) throw new BridgeError("request_closed", "A ligação que pediu o envio já foi fechada.");
      if (native.processing) throw new BridgeError("busy", "A sessão já tem trabalho ativo.");
      if (this.admittedRequestIds.size >= MAX_SEND_IDS) {
        const oldest = this.admittedRequestIds.values().next().value;
        if (oldest !== undefined) this.admittedRequestIds.delete(oldest);
      }
      this.admittedRequestIds.add(request.id);
      admission = { unknown: false };
      this.pendingSend = admission;
      // Do not pass model, permissions, tools, source, agentMode, displayPrompt or headers.
      const submitted = this.session.send({ prompt: request.query!, mode: "enqueue" });
      void submitted.then(messageId => {
        if (this.stopped || this.pendingSend !== admission) return;
        if (typeof messageId !== "string" || !messageId || messageId.length > 256) {
          admission!.unknown = true;
          return;
        }
        admission!.messageId = messageId;
        admission!.unknown = false;
        this.history.rememberSent(messageId, request.query!);
        this.sentMessageIds.add(messageId);
        if (this.observedUserIds.has(messageId)) {
          this.pendingSend = undefined;
          this.sentMessageIds.delete(messageId);
        }
        this.invalidate();
      }, () => {
        if (this.pendingSend === admission) {
          // A lost response is ambiguous. An error must never trigger blind replay.
          admission!.unknown = true;
          this.invalidate();
        }
      });
      let messageId: string;
      try {
        messageId = await deadline(submitted, RPC_TIMEOUT_MS,
          new BridgeError("send_status_unknown", "A entrega não foi confirmada; não reenvie esta mensagem automaticamente."));
      } catch {
        admission.unknown = true;
        throw new BridgeError("send_status_unknown", "A entrega não foi confirmada; não reenvie esta mensagem automaticamente.");
      }
      this.checkTarget();
      if (typeof messageId !== "string" || !messageId || messageId.length > 256) {
        admission.unknown = true;
        throw new BridgeError("send_status_unknown", "O SDK não confirmou o identificador da mensagem; não a reenvie automaticamente.");
      }
      return { target: this.target, messageId };
    } finally {
      this.sendInFlight = false;
      this.invalidate();
    }
  }

  private accept(socket: Socket): void {
    if (this.stopped || this.connections.size >= MAX_CONNECTIONS) { socket.destroy(); return; }
    const connection: Connection = {
      socket, fragments: [], fragmentBytes: 0, pending: false, writing: false, dirty: false,
      outputTimer: undefined,
    };
    this.connections.add(connection);
    // A valid idle subscriber stays connected. Incomplete requests cannot occupy
    // a connection indefinitely; native RPCs have their own bounded deadline.
    socket.setTimeout(30_000, () => socket.destroy());
    socket.on("error", () => socket.destroy());
    socket.on("close", () => {
      if (connection.outputTimer) clearTimeout(connection.outputTimer);
      connection.outputTimer = undefined;
      connection.fragments = [];
      connection.fragmentBytes = 0;
      this.connections.delete(connection);
    });
    socket.on("end", () => {
      if (connection.fragmentBytes > 0) socket.destroy();
      else if (!connection.pending) socket.end();
    });
    socket.on("drain", () => {
      if (connection.outputTimer) clearTimeout(connection.outputTimer);
      connection.outputTimer = undefined;
      connection.writing = false;
      this.notify(connection);
    });
    socket.on("data", (chunk: Buffer) => this.receive(connection, chunk));
  }

  private receive(connection: Connection, chunk: Buffer): void {
    let start = 0;
    while (start < chunk.length) {
      const newline = chunk.indexOf(10, start);
      const end = newline < 0 ? chunk.length : newline;
      const fragment = chunk.subarray(start, end);
      if (connection.fragmentBytes + fragment.length > MAX_REQUEST_BYTES) {
        connection.socket.destroy(); return;
      }
      connection.fragments.push(fragment);
      connection.fragmentBytes += fragment.length;
      if (newline < 0) { connection.socket.setTimeout(10_000); break; }
      if (connection.pending || connection.writing) { connection.socket.destroy(); return; }
      const line = Buffer.concat(connection.fragments, connection.fragmentBytes);
      connection.fragments = [];
      connection.fragmentBytes = 0;
      connection.socket.setTimeout(0);
      connection.pending = true;
      void this.dispatch(connection, line);
      start = newline + 1;
    }
  }

  private async dispatch(connection: Connection, line: Buffer): Promise<void> {
    let id: RequestId | null = null;
    try {
      let raw: unknown;
      try { raw = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(line)); }
      catch { throw new BridgeError("invalid_request", "O pedido não é um objeto JSON UTF-8 válido."); }
      const maybeId = record(raw)?.id;
      if (validId(maybeId)) id = maybeId;
      const request = validateRequest(raw, this.target);
      this.checkTarget();
      let result: Snapshot | { target: Target; messageId: string };
      if (request.method === "history") result = await this.snapshot();
      else if (request.method === "send") {
        result = await this.send(request, () => !connection.socket.destroyed && connection.socket.writable);
      }
      else throw new BridgeError("cancel_unavailable", CANCEL_NOTICE);
      this.checkTarget();
      this.write(connection, { id: request.id, result });
    } catch (error) {
      this.write(connection, { id, error: safeError(error) });
    } finally {
      connection.pending = false;
      if (connection.socket.readableEnded) connection.socket.end();
      this.notify(connection);
    }
  }

  private write(connection: Connection, value: unknown): void {
    if (connection.socket.destroyed) return;
    let encoded = JSON.stringify(value);
    if (Buffer.byteLength(encoded, "utf8") + 1 > MAX_HISTORY_BYTES) {
      const id = record(value)?.id ?? null;
      encoded = JSON.stringify({ id, error: {
        code: "history_too_large", message: "O histórico público excede 128 MiB em JSON; não foi truncado.",
      } });
    }
    connection.writing = !connection.socket.write(`${encoded}\n`);
    if (connection.writing && !connection.outputTimer) {
      // Unlike an idle subscriber, a reader holding a queued frame must make
      // progress. Bound its retained output and connection slot after backpressure.
      connection.outputTimer = setTimeout(() => {
        connection.outputTimer = undefined;
        connection.socket.destroy();
      }, RPC_TIMEOUT_MS);
      connection.outputTimer.unref();
    }
  }

  private invalidate(): void {
    for (const connection of this.connections) connection.dirty = true;
    if (this.notificationTimer || this.stopped) return;
    this.notificationTimer = setTimeout(() => {
      this.notificationTimer = undefined;
      void this.endpoint?.markUpdated().catch(() => {});
      for (const connection of this.connections) this.notify(connection);
    }, 100);
  }

  private notify(connection: Connection): void {
    if (!connection.dirty || connection.pending || connection.writing || connection.socket.destroyed) return;
    connection.dirty = false;
    this.write(connection, { method: "updated", target: this.target });
  }
}
