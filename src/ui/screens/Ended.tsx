import type { ProfileView, View } from '../../core';
import { Button } from '../components/Button';
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

/** The end of a run: cause, the ledger, score, remnants, new marks and unlocks. */
export function Ended({ view, end, profile, onDone }: EndedProps) {
  const ending = view?.ending ?? null;
  const ledger = end?.ledger ?? view?.ledger ?? null;
  const report = end?.report ?? null;
  const kind = ending?.kind ?? ledger?.end ?? 'death';
  const marks = profile?.marks ?? [];
  const newMarks = (report?.new_marks ?? []).map((id) => marks.find((m) => m.id === id)).filter((m) => m !== undefined);
  const unlocked = (report?.unlocked_backgrounds ?? [])
    .map((id) => profile?.backgrounds.find((b) => b.id === id))
    .filter((b) => b !== undefined);

  return (
    <Page title={HEADINGS[kind]} kicker={ledger ? `Day ${ledger.days}` : undefined} class={`ended ended--${kind}`}>
      {(ending ?? ledger) && (
        <div class="ended__cause">
          <p class="ended__why label">{ending?.cause ?? ledger?.cause}</p>
          <p class="prose ended__text">{ending?.text ?? ledger?.text}</p>
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
