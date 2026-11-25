import type { Ledger } from '../../core';

/** The figures of one ended run, then its score lines. */
export function LedgerTable({ ledger: l }: { ledger: Ledger }) {
  const rows: [string, string | number][] = [
    ['Days', l.days],
    ['Nights survived', l.nights],
    ['Places found', `${l.places_found}/${l.places_total}`],
    ['Noise made', l.noise_total],
    ['Killed', l.kills],
    ['Shots fired', l.shots],
    ['Walked', `${(l.metres / 1000).toFixed(1)} km`],
    ['Ferry parts fitted', `${l.parts}/4`],
    ['Station levels', l.stations],
    ['Codex entries', l.codex.length],
    ['The dog', l.dog ? 'With you' : 'No'],
  ];
  return (
    <>
      <dl class="table">
        {rows.map(([k, v]) => (
          <div key={k} class="table__row">
            <dt>{k}</dt>
            <dd class="num">{v}</dd>
          </div>
        ))}
      </dl>
      <p class="label table__caption">Score</p>
      <dl class="table table--score">
        {l.lines.map((s) => (
          <div key={s.label} class="table__row">
            <dt>{s.label}</dt>
            <dd class="num">{s.points}</dd>
          </div>
        ))}
        <div class="table__row table__row--total">
          <dt>Score</dt>
          <dd class="num">{l.score}</dd>
        </div>
      </dl>
    </>
  );
}
