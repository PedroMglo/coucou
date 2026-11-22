export const MAX_REQUEST_BYTES = 1024 * 1024;
export const MAX_HISTORY_BYTES = 128 * 1024 * 1024;
export const MAX_CONNECTIONS = 8;
export const CAPABILITIES = Object.freeze({ history: true, send: true, cancel: false });

export interface Target {
  readonly petId: "agent_copilot";
  readonly provider: "copilot";
  readonly origin: "external";
  readonly sessionId: string;
}

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
}

export interface Snapshot {
  target: Target;
  messages: Message[];
  busy: boolean;
  turn?: string;
  capabilities: { history: boolean; send: boolean; cancel: boolean };
  notice?: string;
  model?: string;
}

export type RequestId = string | number;
export interface Request {
  id: RequestId;
  method: "history" | "send" | "cancel";
  target: Target;
  query?: string;
  turn?: string;
}

export class BridgeError extends Error {
  constructor(readonly code: string, message: string) {
    super(message);
    this.name = "BridgeError";
  }
}

export function record(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
}

export function validSessionId(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(value);
}

export function validId(value: unknown): value is RequestId {
  return (typeof value === "string" && value.length > 0 && value.length <= 128)
    || (typeof value === "number" && Number.isSafeInteger(value) && value >= 0);
}

export function validateRequest(value: unknown, target: Target): Request {
  const input = record(value);
  if (!input || !validId(input.id)) {
    throw new BridgeError("invalid_request", "O pedido não tem um identificador válido.");
  }
  if (Object.keys(input).some(key => !["id", "method", "target", "query", "turn"].includes(key))) {
    throw new BridgeError("invalid_request", "O pedido contém campos não suportados.");
  }
  const selected = record(input.target);
  if (!selected || Object.keys(selected).length !== 4
      || selected.petId !== target.petId || selected.provider !== target.provider
      || selected.origin !== target.origin || selected.sessionId !== target.sessionId) {
    throw new BridgeError("target_mismatch", "O destino não corresponde à sessão desta extensão.");
  }
  if (input.method !== "history" && input.method !== "send" && input.method !== "cancel") {
    throw new BridgeError("method_unavailable", "A extensão não suporta esse método.");
  }
  if (input.method === "send") {
    if (typeof input.query !== "string" || !input.query.trim()
        || Buffer.byteLength(input.query, "utf8") > MAX_REQUEST_BYTES || input.turn !== undefined) {
      throw new BridgeError("invalid_request", "A mensagem não tem texto válido.");
    }
    return { id: input.id, method: "send", target, query: input.query };
  }
  if (input.query !== undefined || (input.method === "history" && input.turn !== undefined)
      || (input.turn !== undefined && (typeof input.turn !== "string" || input.turn.length > 256))) {
    throw new BridgeError("invalid_request", "O pedido contém parâmetros inválidos.");
  }
  return {
    id: input.id,
    method: input.method,
    target,
    ...(typeof input.turn === "string" ? { turn: input.turn } : {}),
  };
}

/** Only reviewed errors cross the socket. Native RPC errors may contain private payloads. */
export function safeError(error: unknown): { code: string; message: string } {
  return error instanceof BridgeError
    ? { code: error.code, message: error.message }
    : { code: "session_unavailable", message: "A sessão Copilot não está disponível através desta extensão." };
}

export async function deadline<T>(promise: Promise<T>, milliseconds: number, error: BridgeError): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => { timer = setTimeout(() => reject(error), milliseconds); }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}
