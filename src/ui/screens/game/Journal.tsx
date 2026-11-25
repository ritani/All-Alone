import { useState } from 'preact/hooks';
import type { Entry } from '../../../core';
import { Icon } from '../../components/Icon';
import { Sheet } from '../../components/Sheet';

export interface JournalProps {
  entries: Entry[];
  /** How many of the newest entries are new. */
  fresh: number;
  seq: number;
}

/** Lines the strip shows: the newest, and up to two before it. */
const STRIP_LINES = 3;

function entryClass(e: Entry, isFresh: boolean, delay: number, extra = ''): string {
  return ['entry', `entry--${e.tone}`, isFresh ? `entry--fresh entry--d${Math.min(delay, 7)}` : '', extra]
    .filter(Boolean)
    .join(' ');
}

/**
 * The whole journal, by day, newest last. It does not scroll itself: it sits
 * in the journal sheet, whose body is the one scroller.
 */
export function Journal({ entries, fresh, seq }: JournalProps) {
  const firstFresh = entries.length - Math.min(fresh, entries.length);
  return (
    <ol class="journal">
      {entries.map((e, i) => {
        const prev = entries[i - 1];
        const newDay = !prev || prev.day !== e.day;
        const newTime = newDay || prev.time !== e.time;
        const isFresh = i >= firstFresh;
        return (
          <li key={isFresh ? `${seq}-${i}` : `${e.day}-${e.time}-${i}-${entries.length}`} class={entryClass(e, isFresh, i - firstFresh)}>
            {newDay && <span class="entry__day label">Day {e.day}</span>}
            <span class={newTime ? 'entry__time num' : 'entry__time entry__time--same num'} aria-hidden={!newTime}>
              {e.time}
            </span>
            <span class="entry__text">{e.text}</span>
          </li>
        );
      })}
    </ol>
  );
}

/**
 * The journal on the game frame: only the newest lines, the latest in full
 * voice and the ones before it fading. It never scrolls; tapping it opens the
 * whole journal in a sheet.
 */
export function JournalStrip({ entries, fresh, seq }: JournalProps) {
  const [open, setOpen] = useState(false);
  const shown = entries.slice(-STRIP_LINES);
  const offset = entries.length - shown.length;
  const firstFresh = entries.length - Math.min(fresh, entries.length);
  return (
    <section class="jstrip" aria-label="Journal">
      <div role="log" aria-live="polite" aria-relevant="additions">
      <ol class="jstrip__list">
        {shown.map((e, j) => {
          const i = offset + j;
          const isFresh = i >= firstFresh;
          const last = j === shown.length - 1;
          const prev = entries[i - 1];
          const sameTime = j > 0 && prev !== undefined && prev.day === e.day && prev.time === e.time;
          return (
            <li
              key={isFresh ? `${seq}-${i}` : `${e.day}-${e.time}-${i}`}
              class={entryClass(e, isFresh, i - firstFresh, last ? 'entry--last' : isFresh ? 'entry--new' : 'entry--old')}
            >
              <span class={sameTime ? 'entry__time entry__time--same num' : 'entry__time num'} aria-hidden={sameTime}>
                {e.time}
              </span>
              <span class="entry__text">{e.text}</span>
            </li>
          );
        })}
      </ol>
      </div>
      <button type="button" class="jstrip__open" onClick={() => setOpen(true)} aria-label="Open the journal">
        <Icon name="journal" size={16} />
      </button>
      {open && (
        <Sheet
          title="Journal"
          tall
          onClose={() => setOpen(false)}
          bodyRef={(el) => {
            if (el && !el.dataset.end) {
              el.dataset.end = '1';
              el.scrollTop = el.scrollHeight;
            }
          }}
        >
          <Journal entries={entries} fresh={0} seq={seq} />
        </Sheet>
      )}
    </section>
  );
}
