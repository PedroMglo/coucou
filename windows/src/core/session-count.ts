/** A catalogue count describes known session records, not live jobs. Missing is not zero. */
export function knownSessionCount(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 ? value : undefined;
}

export function sessionCountLabel(value: unknown): string {
  const count = knownSessionCount(value);
  return count === undefined ? 'Contagem de sessões não disponível' : `${count} ${count === 1 ? 'sessão conhecida' : 'sessões conhecidas'}`;
}

export function compactSessionCount(value: unknown): string {
  const count = knownSessionCount(value);
  return count === undefined ? 'Sessões · —' : `${count} ${count === 1 ? 'sessão' : 'sessões'}`;
}
