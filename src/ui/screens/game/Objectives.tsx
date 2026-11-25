import { useState } from 'preact/hooks';
import type { Objective } from '../../../core';
import { Icon } from '../../components/Icon';

/** What matters next. Urgent ones first and in the danger accent; the rest fold away. */
export function Objectives({ items }: { items: Objective[] }) {
  const [open, setOpen] = useState(false);
  if (items.length === 0) return null;
  const sorted = [...items].sort((a, b) => Number(b.urgent) - Number(a.urgent));
  const shown = open ? sorted : sorted.slice(0, 2);
  return (
    <section class={open ? 'objectives objectives--open' : 'objectives'} aria-label="Objectives">
      <ul class="objectives__list">
        {shown.map((o) => (
          <li key={o.id} class={o.urgent ? 'objective objective--urgent' : 'objective'}>
            <Icon name={o.icon} size={16} class="objective__icon" />
            <span class="objective__text">{o.text}</span>
          </li>
        ))}
      </ul>
      {sorted.length > 1 && (
        <button type="button" class="objectives__more label" aria-expanded={open} onClick={() => setOpen(!open)}>
          {open ? 'Fewer' : `All ${sorted.length}`}
        </button>
      )}
    </section>
  );
}
