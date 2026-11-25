import { useState } from 'preact/hooks';
import type { Objective } from '../../../core';
import { Icon } from '../../components/Icon';
import { Sheet } from '../../components/Sheet';

/** What matters next, on one line: the most urgent. Tapping it lists them all. */
export function Objectives({ items }: { items: Objective[] }) {
  const [open, setOpen] = useState(false);
  if (items.length === 0) return null;
  const sorted = [...items].sort((a, b) => Number(b.urgent) - Number(a.urgent));
  const top = sorted[0]!;
  const more = sorted.length - 1;
  return (
    <section class="objectives" aria-label="Objectives">
      <button
        type="button"
        class={top.urgent ? 'objectives__top objective--urgent' : 'objectives__top'}
        onClick={() => setOpen(true)}
        aria-label={`${top.text}${more > 0 ? ` And ${more} more: list them` : ''}`}
      >
        <Icon name={top.icon} size={16} class="objective__icon" />
        <span class="objectives__text">{top.text}</span>
        {more > 0 && <span class="objectives__more num">+{more}</span>}
      </button>
      {open && (
        <Sheet title="What matters" onClose={() => setOpen(false)}>
          <ul class="objective-list">
            {sorted.map((o) => (
              <li key={o.id} class={o.urgent ? 'objective objective--urgent' : 'objective'}>
                <Icon name={o.icon} size={20} class="objective__icon" />
                <span>{o.text}</span>
              </li>
            ))}
          </ul>
        </Sheet>
      )}
    </section>
  );
}
