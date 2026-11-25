import { useEffect, useRef } from 'preact/hooks';
import type { Entry } from '../../../core';

export interface JournalProps {
  entries: Entry[];
  /** How many of the newest entries are new. */
  fresh: number;
  seq: number;
}

/**
 * The journal: the game's main voice. Newest last; new lines fade in, one
 * after another. The list scrolls itself to the newest line after each action.
 */
export function Journal({ entries, fresh, seq }: JournalProps) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [seq, entries.length]);

  const firstFresh = entries.length - Math.min(fresh, entries.length);
  return (
    <div class="journal" ref={ref} role="log" aria-live="polite" aria-relevant="additions" aria-label="Journal" tabIndex={0}>
      <ol class="journal__list">
      {entries.map((e, i) => {
        const prev = entries[i - 1];
        const newDay = !prev || prev.day !== e.day;
        const newTime = newDay || prev.time !== e.time;
        const isFresh = i >= firstFresh;
        const cls = [
          'entry',
          `entry--${e.tone}`,
          isFresh ? `entry--fresh entry--d${Math.min(i - firstFresh, 7)}` : '',
          newTime ? 'entry--time' : '',
        ]
          .filter(Boolean)
          .join(' ');
        return (
          <li key={isFresh ? `${seq}-${i}` : `${e.day}-${e.time}-${i}-${entries.length}`} class={cls}>
            {newDay && <span class="entry__day label">Day {e.day}</span>}
            <span class={newTime ? 'entry__time num' : 'entry__time entry__time--same num'} aria-hidden={!newTime}>
              {e.time}
            </span>
            <span class="entry__text">{e.text}</span>
          </li>
        );
      })}
      </ol>
    </div>
  );
}
