import { randomUUID } from "node:crypto";
import { constants, lstatSync, unlinkSync, type BigIntStats } from "node:fs";
import { chmod, lstat, mkdir, open, realpath, rename } from "node:fs/promises";
import { createConnection, createServer, type Server, type Socket } from "node:net";
import { isAbsolute, join, resolve } from "node:path";
import { BridgeError, CAPABILITIES, record, type Target } from "./protocol.js";

const MAX_METADATA_BYTES = 16 * 1024;
const STALE_PROBE_MS = 1_000;

export interface Metadata {
  provider: "copilot";
  sessionId: string;
  cwd: string;
  pid: number;
  parentPid: number;
  capabilities: typeof CAPABILITIES;
}

export interface Endpoint {
  server: Server;
  socketPath: string;
  metadataPath: string;
  cleanup(): void;
  markUpdated(): Promise<void>;
}

function sameFile(first: BigIntStats | undefined, second: BigIntStats): boolean {
  return first !== undefined && first.dev === second.dev && first.ino === second.ino;
}

function removeOwn(path: string, expected: BigIntStats | undefined): void {
  if (!expected) return;
  try {
    if (sameFile(expected, lstatSync(path, { bigint: true }))) unlinkSync(path);
  } catch {
    // Never remove a replacement or expose filesystem details through logs.
  }
}

async function privateDirectory(path: string, uid: number, create: boolean): Promise<void> {
  if (create) {
    try { await mkdir(path, { mode: 0o700 }); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error; }
  }
  const stat = await lstat(path);
  if (!stat.isDirectory() || stat.isSymbolicLink() || stat.uid !== uid
      || (stat.mode & 0o7777) !== 0o700 || await realpath(path) !== path) {
    throw new BridgeError("insecure_runtime", "A pasta de execução tem de ser privada, real e pertencer ao utilizador.");
  }
}

async function requireAbsent(path: string): Promise<void> {
  try { await lstat(path); }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
  throw new BridgeError("endpoint_exists", "Já existe uma ponte ou um ficheiro de execução para esta sessão.");
}

function deadProcess(pid: number): boolean {
  try { process.kill(pid, 0); return false; }
  catch (error) { return (error as NodeJS.ErrnoException).code === "ESRCH"; }
}

function privateFile(stat: BigIntStats, uid: number): boolean {
  return stat.uid === BigInt(uid) && (stat.mode & 0o7777n) === 0o600n;
}

async function refusedSocket(path: string): Promise<boolean> {
  return new Promise(resolveProbe => {
    let settled = false;
    const probe = createConnection(path);
    const timer = setTimeout(() => finish(false), STALE_PROBE_MS);
    const finish = (refused: boolean) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      probe.destroy();
      resolveProbe(refused);
    };
    // No request or application data is sent, even if a live server answers.
    probe.once("connect", () => finish(false));
    probe.once("error", error => finish((error as NodeJS.ErrnoException).code === "ECONNREFUSED"));
  });
}

/** Reclaim a crashed extension only with all ownership and liveness checks intact. */
async function reclaimStale(socketPath: string, metadataPath: string, target: Target, uid: number): Promise<void> {
  try {
    const [socketStat, metadataStat] = await Promise.all([
      lstat(socketPath, { bigint: true }), lstat(metadataPath, { bigint: true }),
    ]);
    if (!socketStat.isSocket() || !privateFile(socketStat, uid)
        || !metadataStat.isFile() || !privateFile(metadataStat, uid) || metadataStat.nlink !== 1n
        || metadataStat.size <= 0n || metadataStat.size > BigInt(MAX_METADATA_BYTES)) return;
    const file = await open(metadataPath, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
    let metadata: Record<string, unknown> | undefined;
    try {
      const opened = await file.stat({ bigint: true });
      if (!sameFile(metadataStat, opened) || !opened.isFile() || !privateFile(opened, uid)) return;
      // Bound the read even if another process grows the same inode after stat.
      const buffer = Buffer.alloc(MAX_METADATA_BYTES + 1);
      let length = 0;
      while (length < buffer.length) {
        const { bytesRead } = await file.read(buffer, length, buffer.length - length, length);
        if (bytesRead === 0) break;
        length += bytesRead;
      }
      if (length > MAX_METADATA_BYTES) return;
      metadata = record(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(buffer.subarray(0, length))));
    } finally {
      await file.close();
    }
    const capabilities = record(metadata?.capabilities);
    if (!metadata || metadata.provider !== "copilot" || metadata.sessionId !== target.sessionId
        || Object.keys(metadata).some(key => !["provider", "sessionId", "cwd", "pid", "parentPid", "capabilities"].includes(key))
        || typeof metadata.cwd !== "string" || !isAbsolute(metadata.cwd)
        || !Number.isSafeInteger(metadata.pid) || typeof metadata.pid !== "number" || metadata.pid <= 1
        || !Number.isSafeInteger(metadata.parentPid) || typeof metadata.parentPid !== "number" || metadata.parentPid <= 1
        || !capabilities || typeof capabilities.history !== "boolean" || typeof capabilities.send !== "boolean"
        || capabilities.cancel !== false || !deadProcess(metadata.pid)
        || !await refusedSocket(socketPath)) return;

    // Keep the final check/removal synchronous, with no local await between the
    // inode comparison and unlink. A live, replaced, ambiguous or inaccessible
    // endpoint is always preserved. PID reuse is treated as live.
    const currentSocket = lstatSync(socketPath, { bigint: true });
    const currentMetadata = lstatSync(metadataPath, { bigint: true });
    if (!sameFile(socketStat, currentSocket) || !sameFile(metadataStat, currentMetadata)
        || !currentSocket.isSocket() || !privateFile(currentSocket, uid)
        || !currentMetadata.isFile() || !privateFile(currentMetadata, uid)
        || currentMetadata.nlink !== 1n || !deadProcess(metadata.pid)) return;
    unlinkSync(socketPath);
    // A replacement sidecar must survive even if it appeared after socket removal.
    if (sameFile(metadataStat, lstatSync(metadataPath, { bigint: true }))) unlinkSync(metadataPath);
  } catch {
    // Missing pairs, malformed data, permissions and inconclusive probes are not
    // evidence of a stale owner. The following requireAbsent checks fail closed.
  }
}

/** Linux pathname permissions protect this endpoint; Node exposes no public SO_PEERCRED API. */
export async function listenPrivate(
  target: Target,
  cwd: string,
  parentPid: number,
  onConnection: (socket: Socket) => void,
): Promise<Endpoint> {
  const runtime = process.env.XDG_RUNTIME_DIR;
  if (process.platform !== "linux" || !process.getuid || !runtime || !isAbsolute(runtime)) {
    throw new BridgeError("runtime_unavailable", "A ponte requer Linux e XDG_RUNTIME_DIR privado.");
  }
  const uid = process.getuid();
  const root = resolve(runtime);
  await privateDirectory(root, uid, false);
  const coucou = join(root, "coucou");
  await privateDirectory(coucou, uid, true);
  const directory = join(coucou, "copilot");
  await privateDirectory(directory, uid, true);
  const socketPath = join(directory, `${target.sessionId}.sock`);
  const metadataPath = join(directory, `${target.sessionId}.json`);
  if (Buffer.byteLength(socketPath, "utf8") >= 108) {
    throw new BridgeError("runtime_path_too_long", "O caminho do socket excede o limite Unix do Linux.");
  }
  await reclaimStale(socketPath, metadataPath, target, uid);
  await requireAbsent(socketPath);
  await requireAbsent(metadataPath);

  const server = createServer({ allowHalfOpen: true }, onConnection);
  let socketIdentity: BigIntStats | undefined;
  let metadataIdentity: BigIntStats | undefined;
  let temporary: string | undefined;
  const cleanup = () => {
    removeOwn(metadataPath, metadataIdentity);
    removeOwn(socketPath, socketIdentity);
  };
  try {
    await new Promise<void>((resolveListen, reject) => {
      const onError = (error: Error) => { server.off("listening", onListen); reject(error); };
      const onListen = () => { server.off("error", onError); resolveListen(); };
      server.once("error", onError);
      server.once("listening", onListen);
      server.listen(socketPath);
    });
    socketIdentity = await lstat(socketPath, { bigint: true });
    await chmod(socketPath, 0o600);
    const socketStat = await lstat(socketPath);
    if (!socketStat.isSocket() || socketStat.uid !== uid || (socketStat.mode & 0o7777) !== 0o600) {
      throw new BridgeError("insecure_runtime", "Não foi possível criar um socket privado.");
    }
    const metadata: Metadata = {
      provider: "copilot", sessionId: target.sessionId, cwd, pid: process.pid, parentPid,
      capabilities: CAPABILITIES,
    };
    temporary = join(directory, `.${target.sessionId}.${randomUUID()}.tmp`);
    const file = await open(temporary, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL | constants.O_NOFOLLOW, 0o600);
    try {
      await file.writeFile(`${JSON.stringify(metadata)}\n`, "utf8");
      await file.sync();
      await file.chmod(0o600);
    } finally {
      await file.close();
    }
    // The sidecar contains discovery data only. History never reaches the filesystem.
    await rename(temporary, metadataPath);
    temporary = undefined;
    metadataIdentity = await lstat(metadataPath, { bigint: true });
    return {
      server, socketPath, metadataPath, cleanup,
      async markUpdated(): Promise<void> {
        // Discovery uses mtime as updatedAt; changing it must not follow a link or
        // modify a sidecar replaced by another process. No event content is written.
        const handle = await open(metadataPath, constants.O_RDONLY | constants.O_NOFOLLOW);
        try {
          if (!sameFile(metadataIdentity, await handle.stat({ bigint: true }))) return;
          const now = new Date();
          await handle.utimes(now, now);
        } finally {
          await handle.close();
        }
      },
    };
  } catch (error) {
    server.close(() => {});
    cleanup();
    if (temporary) {
      try { unlinkSync(temporary); } catch { /* O_EXCL-created temporary only. */ }
    }
    throw error;
  }
}
