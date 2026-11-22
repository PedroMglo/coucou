import type { Coverage, OpenTarget, SessionRow } from './pet-observability-contract';

/** provider is the physical backend. Codex ID is Thread.id, not tree sessionId. */
export type SessionLookupRequest = { provider: string; nativeSessionId: string };
export type SessionLookupState = 'resolved' | 'ambiguous' | 'unavailable' |
  'notFound' | 'notSupported' | 'invalidRequest';
export type SessionLookupEvidence = {
  scope: string; coverage: Coverage; observedAt: number | null;
  stale: boolean; reason: string | null;
};
/** A resolved destination authorizes navigation only; no liveness/execute claim.
 * Positive native evidence may be stale. open.provider for Mochi is the logical
 * owner (which may differ from the physical backend and the pet ID).
 */
export type SessionLookupResult = {
  schemaVersion: 1; generatedAt: number; request: SessionLookupRequest;
  state: SessionLookupState; reason: string | null;
  /** Advisory manual retry delay for lookup throttling, not a completion SLA. */
  retryAfterMs: number | null;
  owner: string | null; conversationId: string | null; origin: SessionRow['origin'];
  open: OpenTarget | null;
  library: SessionLookupEvidence; native: SessionLookupEvidence;
};
