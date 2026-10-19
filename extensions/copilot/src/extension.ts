import { joinSession } from "@github/copilot-sdk/extension";
import { SessionBridge } from "./bridge.js";
import { BridgeError, safeError, validSessionId, type Target } from "./protocol.js";

let bridge: SessionBridge | undefined;
let exiting = false;

function finish(error?: BridgeError): void {
  if (exiting) return;
  exiting = true;
  bridge?.stop();
  if (error) process.stderr.write(`[coucou-copilot] ${safeError(error).code}\n`);
  // This exits only the extension child; it never aborts or shuts down the parent session.
  process.exit(error ? 1 : 0);
}

process.once("SIGTERM", () => finish());
process.once("SIGINT", () => finish());
process.once("disconnect", () => finish());
process.stdin.once("end", () => finish());
process.stdin.once("close", () => finish());
process.once("exit", () => bridge?.cleanupOnExit());
process.once("uncaughtException", () => finish(new BridgeError("extension_failed", "A ponte foi desligada.")));
process.once("unhandledRejection", () => finish(new BridgeError("extension_failed", "A ponte foi desligada.")));

async function main(): Promise<void> {
  const sessionId = process.env.SESSION_ID;
  const parentPid = process.ppid;
  if (!sessionId || !validSessionId(sessionId) || parentPid <= 1) {
    throw new BridgeError("invalid_host", "Esta extensão deve ser iniciada pela sessão Copilot que a aloja.");
  }
  const target: Target = Object.freeze({ petId: "agent_copilot", provider: "copilot", origin: "external", sessionId });
  const cwd = process.cwd();
  // Omitting onPermissionRequest is intentional: the SDK uses no-result and does
  // not advertise requestPermission, leaving every approval with the owning TUI.
  // Request live events for this extension. Only public text deltas are projected
  // by the bridge; model, tools, permissions and session ownership stay native.
  const session = await joinSession({ streaming: true });
  if (session.sessionId !== sessionId || process.env.SESSION_ID !== sessionId || process.ppid !== parentPid) {
    throw new BridgeError("target_mismatch", "O SDK juntou uma sessão diferente da sessão capturada.");
  }
  bridge = new SessionBridge(session, target, cwd, parentPid, error => finish(error));
  await bridge.start();
}

void main().catch(error => finish(error instanceof BridgeError
  ? error : new BridgeError("startup_failed", "Não foi possível iniciar a ponte da sessão Copilot.")));
