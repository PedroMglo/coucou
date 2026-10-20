// Readonly IPC contract. All timestamps are Unix seconds. No inferred counters.
import type {ProcessCensus,SnapshotLiveWorkExtension} from './process-census-contract';
export type Coverage = 'complete' | 'partial' | 'unavailable' | 'notSupported';
export type Activity = 'working' | 'waiting' | 'busy' | 'reserved' | 'idle' | 'notLoaded' | 'error' | 'unknown';
export type Count = {
  value: number | null;
  // Count of known matching rows, NOT a timestamp. Do not add across scopes.
  observed: number;
};
export type OpenTarget =
  | { kind: 'mochi'; petId: string; provider: string; conversationId: string }
  | { kind: 'external'; petId: string; provider: string; sessionId: string };
export type SessionRow = {
  key: string; provider: string; petId: string | null;
  origin: 'mochi' | 'external' | 'unknown';
  sessionId: string | null; conversationId: string | null;
  title: string; cwd: string | null; state: Activity;
  archived: boolean | null; updatedAt: number | null;
  observedAt: number | null; open: OpenTarget | null;
};
export type Inventory = {
  // complete means complete for THIS source, not all accounts or all clients.
  scope: 'mochiLibrary' | 'codexDaemon' | 'copilotLiveEndpoints' | 'providerGlobal' | string;
  coverage: Coverage; observedAt: number | null; stale: boolean;
  refreshing: boolean; reason: string | null;
  counts: {
    total: Count; archived: Count; confirmedActive: Count;
    working: Count; waiting: Count; reserved: Count; idle: Count; unknown: number;
  };
  sessions: SessionRow[]; listed: number; limit: number; truncated: boolean;
};
export type UsageStatus = 'unavailable' | 'noObservations' | 'observed';
export type UsageCoverage =
  | 'unavailable' | 'partial' | 'completeWithinTrackedAttempts';
export type UsagePersistenceState =
  | 'unavailable' | 'memoryOnly' | 'dirty' | 'durable' | 'error' | 'unknown';
export type UsageIssue =
  | 'invalidInput' | 'invalidData' | 'limit' | 'conflict' | 'busy' | 'notFound'
  | 'io' | 'unsupportedPlatform' | 'durabilityUnknown' | 'lockCleanup';

export interface UsageDimension {
  knownTokens: string | null; // Exact unsigned decimal, never Number(...).
  knownAttempts: number;      // Final snapshot with this dimension present.
  partialAttempts: number;    // Partial snapshot with this dimension present.
  missingAttempts: number;    // Snapshot exists; this dimension is absent.
  unknownAttempts: number;    // No snapshot at all.
  derivedAttempts: number;    // Subset of known/partial; total=input+output.
  coverage: UsageCoverage;
}
export interface UsageSummary {
  status: UsageStatus;
  scope: 'mochi-observed-attempts';
  provider: string | null;
  owner: string | null;       // null = all registered owners for this provider.
  since: number | null;      // Unix SECONDS; explicit ledger observation start.
  lastObservation: number | null; // Unix SECONDS.
  freshness: UsageStatus;    // Observation presence only; no freshness threshold.
  input: UsageDimension;
  output: UsageDimension;
  total: UsageDimension;
  attempts: {
    tracked: number;
    pending: number;
    completed: number;
    failed: number;
    cancelled: number;
    interrupted: number;
  };
  coverage: {
    status: UsageCoverage;
    historicalBeforeSince: 'notCovered';
    outsideMochi: 'notCovered';
    untrackedMochi: 'notCovered';
  };
  persistence: {
    state: UsagePersistenceState;
    revision: string | null; // Last CONFIRMED disk revision, not UI generation.
    lastCommittedAt: number | null; // Unix SECONDS.
    issue: UsageIssue | null; // Static code, never a path, prompt or raw error.
  };
}

export type PetObservability<U = UsageSummary> = {
  petId: string; provider: string; enabled: boolean;
  mochi: Inventory; usage: U;
};
export type ProviderObservability<U = UsageSummary> = {
  provider: string; petIds: string[]; native: Inventory; usage: U;
  processCensus?:ProcessCensus|null;
};
export type PetObservabilitySnapshot<U = UsageSummary> = {
  schemaVersion: 1; generatedAt: number;
  pets: PetObservability<U>[]; providers: ProviderObservability<U>[];
  livePetWork?:SnapshotLiveWorkExtension['livePetWork'];
};

// confirmedActive = known in-flight turn (working, waiting or busy).
// Busy does not distinguish work from a human wait. Idle does not prove that a
// terminal UI is open. External may be CLI, editor or another native client.
// Mochi library and native inventory overlap: NEVER add their counts.
// For value=null, observed=N is only a known subtotal; null never becomes zero.
// Stale values retain their old observedAt; do not style them as live certainty.
// open is exact navigation only, never an execution/approval authorization.
