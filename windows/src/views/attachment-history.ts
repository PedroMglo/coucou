import type { AttachmentTurnRecord } from '../core/bridge';
import { sessionTargetKey, type SessionTarget } from '../core/session-chat';
import { h } from './dom';

const phases: Record<AttachmentTurnRecord['phase'], string> = {
  pending: 'Pedido em curso',
  done: 'Pedido concluído',
  failed: 'Pedido falhado',
  stopped: 'Pedido cancelado',
  interrupted: 'Pedido interrompido',
};

const coverageLabels = {
  full: 'Texto preparado na íntegra',
  partial: 'Texto preparado parcialmente',
  encoded: 'Ficheiro preparado para envio',
  unavailable: 'Conteúdo indisponível na preparação',
};

type Message = { role: 'user' | 'assistant' };

/** Read-only projection of the native record, separate from the next-send collection. */
export function buildAttachmentHistory(onHeightChange: () => void) {
  let owner = '', digest = '', revision = 0;
  let lastRecords: readonly AttachmentTurnRecord[] | undefined;
  let lastMessages: readonly Message[] | undefined;
  let entries = new Map<number, HTMLElement[]>();
  let cards = new Map<string, { signature: string; element: HTMLDetailsElement }>();

  function card(record: AttachmentTurnRecord, expanded: boolean) {
    const count = record.attachments.length;
    const label = record.messagesAdded === 2 ? 'Anexos deste pedido' : 'Anexos de um pedido sem mensagens guardadas';
    const files = h('ul');
    const preparation = new Map(record.preparation?.map(({ id, coverage }) => [id, coverage]));
    for (const file of record.attachments) {
      const row = h('li', {},
        h('span', { text: `${file.name} · ${file.kind === 'folder' ? 'Pasta' : 'Ficheiro'}` }),
        h('span', {}, 'Cópia ', h('code', { text: file.id })),
      );
      const coverage = preparation.get(file.id);
      if (coverage) row.append(h('span', { text: coverageLabels[coverage] }));
      files.append(row);
    }
    const element = h('details', { class: 'chat-attachment-history' },
      h('summary', { text: `${label} (${count})` }),
      h('p', { text: `${phases[record.phase]}. Cópias associadas ao pedido. Leitura pelo modelo por confirmar.` }),
      files,
    );
    element.dataset.attachmentTurnId = record.id;
    element.open = expanded;
    element.addEventListener('toggle', onHeightChange);
    return element;
  }

  return {
    sync(target: SessionTarget | null, records: readonly AttachmentTurnRecord[] | undefined, messages: readonly Message[]) {
      const key = target?.origin === 'mochi' && target.sessionId && target.sessionId === target.conversationId
        ? sessionTargetKey(target) : '';
      if (owner === key && lastRecords === records && lastMessages === messages) return false;
      if (owner !== key) cards.clear();
      owner = key;
      lastRecords = records;
      lastMessages = messages;

      const nextEntries = new Map<number, HTMLElement[]>();
      const nextCards = new Map<string, { signature: string; element: HTMLDetailsElement }>();
      if (key && target) {
        for (const record of records ?? []) {
          if (record.petId !== target.petId || record.provider !== target.provider || record.conversationId !== target.sessionId) continue;
          const index = record.messageIndex;
          if (!Number.isSafeInteger(index) || index < 0 || index > messages.length || !record.attachments.length) continue;
          if (record.messagesAdded === 2) {
            if (messages[index]?.role !== 'user' || messages[index + 1]?.role !== 'assistant') continue;
          } else if (record.messagesAdded !== 0) continue;
          if (nextCards.has(record.id)) continue;

          const signature = JSON.stringify(record);
          const previous = cards.get(record.id);
          const entry = previous?.signature === signature
            ? previous : { signature, element: card(record, previous?.element.open ?? false) };
          nextCards.set(record.id, entry);
          const group = nextEntries.get(index);
          if (group) group.push(entry.element);
          else nextEntries.set(index, [entry.element]);
        }
      }
      const nextDigest = JSON.stringify([key, Array.from(nextCards.values(), entry => entry.signature)]);
      entries = nextEntries;
      cards = nextCards;
      if (nextDigest === digest) return false;
      digest = nextDigest;
      revision++;
      return true;
    },
    get signature() { return revision; },
    get hasEntries() { return cards.size > 0; },
    before(index: number) { return entries.get(index) ?? []; },
  };
}
