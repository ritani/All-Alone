import type { ProfileView, View } from '../../core';
import { Button } from '../components/Button';
import { EndArt } from '../components/EndArt';
import { Icon } from '../components/Icon';
import { Page } from '../components/Page';
import { Rule } from '../components/Rule';
import type { EndState } from '../state/store';
import { LedgerTable } from './LedgerTable';

export interface EndedProps {
  view: View | null;
  end: EndState | null;
  profile: ProfileView | null;
  onDone: () => void;
}

const HEADINGS = { death: 'Dead', departure: 'Downriver', freeze_out: 'Frozen in' } as const;

/** A last line under the ending, by kind: the run is over, the voice stays level. */
const CODA = {
  death: 'Calder’s Reach is as quiet as it was.',
  departure: 'Two days to the estuary. You do not look back until the town is gone.',
  freeze_out: 'The river will open again in the spring. You will not be here to see it.',
} as const;

/** The end of a run: cause, the ledger, score, remnants, new marks and unlocks. */
export function Ended({ view, end, profile, onDone }: EndedProps) {
  const ending = view?.ending ?? null;
  const ledger = end?.ledger ?? view?.ledger ?? null;
  const report = end?.report ?? null;
  const kind = ending?.kind ?? ledger?.end ?? 'death';
  const cause = ending?.cause ?? ledger?.cause ?? '';
  const gaveUp = kind === 'death' && cause === 'Gave up';
  const marks = profile?.marks ?? [];
  const newMarks = (report?.new_marks ?? []).map((id) => marks.find((m) => m.id === id)).filter((m) => m !== undefined);
  const unlocked = (report?.unlocked_backgrounds ?? [])
    .map((id) => profile?.backgrounds.find((b) => b.id === id))
    .filter((b) => b !== undefined);

  return (
    <Page
      title={gaveUp ? 'Given up' : HEADINGS[kind]}
      kicker={ledger ? `Day ${ledger.days}${ledger.nights > 0 ? `, ${ledger.nights} ${ledger.nights === 1 ? 'night' : 'nights'}` : ''}` : undefined}
      class={`ended ended--${kind}`}
    >
      {kind !== 'death' && (
        <figure class="ended__art">
          <EndArt kind={kind} />
        </figure>
      )}
      {(ending ?? ledger) && (
        <div class="ended__cause">
          <p class="ended__why label">{cause}</p>
          <p class="prose ended__text">{ending?.text ?? ledger?.text}</p>
          {!gaveUp && <p class="prose prose--dim ended__coda">{CODA[kind]}</p>}
        </div>
      )}

      {ledger && (
        <section class="section">
          <Rule>Ledger</Rule>
          <LedgerTable ledger={ledger} />
        </section>
      )}

      {report && (
        <section class="section">
          <Rule>Kept</Rule>
          <dl class="table">
            <div class="table__row">
              <dt>Remnants</dt>
              <dd class="num">+{report.remnants}</dd>
            </div>
            {report.rank !== null && (
              <div class="table__row">
                <dt>Among your best runs</dt>
                <dd class="num">{report.rank}</dd>
              </div>
            )}
            {ledger?.daily && (
              <div class="table__row">
                <dt>The daily river</dt>
                <dd>{report.daily_counted ? 'Counted' : 'Not counted'}</dd>
              </div>
            )}
          </dl>
        </section>
      )}

      {newMarks.length > 0 && (
        <section class="section">
          <Rule>New marks</Rule>
          <ul class="marks">
            {newMarks.map((m) => (
              <li key={m.id} class="mark mark--earned">
                <Icon name={m.icon} size={32} class="mark__sigil" />
                <div class="mark__body">
                  <p class="mark__name">{m.name}</p>
                  <p class="mark__text">{m.text}</p>
                </div>
              </li>
            ))}
          </ul>
        </section>
      )}

      {unlocked.length > 0 && (
        <section class="section">
          <Rule>Now open</Rule>
          <ul class="rows">
            {unlocked.map((b) => (
              <li key={b.id} class="unlock unlock--open">
                <Icon name={b.icon} size={24} />
                <div class="unlock__body">
                  <p class="unlock__name">{b.name}</p>
                  <p class="unlock__text">{b.desc}</p>
                </div>
              </li>
            ))}
          </ul>
        </section>
      )}

      <div class="page__actions">
        <Button block onClick={onDone}>
          Back to the title
        </Button>
      </div>
    </Page>
  );
}
