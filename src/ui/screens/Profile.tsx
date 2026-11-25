/**
 * Profile screens: marks, the ledger of runs, remnants and unlocks, and the
 * short guide. Everything shown comes from the core's ProfileView.
 */
import { useState } from 'preact/hooks';
import type { Ledger, ProfileView, Unlock } from '../../core';
import { Button } from '../components/Button';
import { Hairline } from '../components/Hairline';
import { Icon } from '../components/Icon';
import { Page } from '../components/Page';
import { Rule } from '../components/Rule';
import { LedgerTable } from './LedgerTable';

const ROMAN = ['', 'I', 'II', 'III', 'IV', 'V'];

export function Marks({ profile, onBack }: { profile: ProfileView; onBack: () => void }) {
  const tiers = [...new Set(profile.marks.map((m) => m.tier))].sort((a, b) => a - b);
  return (
    <Page title="Marks" kicker={`${profile.marks_earned} of ${profile.marks_total}`} onBack={onBack}>
      <Hairline value={profile.marks_earned} max={profile.marks_total} label="Marks earned" />
      {tiers.map((tier) => (
        <section key={tier} class="section">
          <Rule>{`Tier ${ROMAN[tier] ?? tier}`}</Rule>
          <ul class="marks">
            {profile.marks
              .filter((m) => m.tier === tier)
              .map((m) => (
                <li key={m.id} class={m.earned ? 'mark mark--earned' : 'mark'}>
                  <Icon name={m.earned || !m.hidden ? m.icon : 'sigil_eye'} size={32} class="mark__sigil" />
                  <div class="mark__body">
                    <p class="mark__name">{m.name}</p>
                    <p class="mark__text">{m.text}</p>
                  </div>
                  <span class="visually-hidden">{m.earned ? 'Earned.' : 'Not yet earned.'}</span>
                </li>
              ))}
          </ul>
        </section>
      ))}
    </Page>
  );
}

function endName(l: Ledger): string {
  return l.end === 'departure' ? 'Left' : l.end === 'freeze_out' ? 'Frozen in' : 'Dead';
}

export function Record({ profile, onBack }: { profile: ProfileView; onBack: () => void }) {
  const [open, setOpen] = useState<number | null>(null);
  const t = profile.totals;
  const bgName = (id: string) => profile.backgrounds.find((b) => b.id === id)?.name ?? id;
  const totals: [string, string | number][] = [
    ['Runs', t.runs],
    ['Deaths', t.deaths],
    ['Departures', t.departures],
    ['Frozen in', t.freeze_outs],
    ['Nights survived', t.nights],
    ['Longest run', `${t.longest} days`],
    ['Best score', t.best_score],
    ['Killed', t.kills],
    ['Shots fired', t.shots],
    ['Noise made', t.noise],
    ['Walked', `${(t.metres / 1000).toFixed(1)} km`],
    ['Remnants held', profile.remnants],
  ];
  return (
    <Page title="Ledger" kicker="What is left of you" onBack={onBack}>
      <section class="section">
        <Rule>Best runs</Rule>
        {profile.best.length === 0 ? (
          <p class="prose prose--dim">No run has ended yet.</p>
        ) : (
          <ol class="best">
            {profile.best.map((l, i) => (
              <li key={i} class="best__item">
                <button
                  type="button"
                  class="best__row"
                  aria-expanded={open === i}
                  onClick={() => setOpen(open === i ? null : i)}
                >
                  <span class="best__rank num">{i + 1}</span>
                  <span class="best__what">
                    <span class="best__cause">
                      {endName(l)}. {l.cause}.
                    </span>
                    <span class="best__meta label">
                      {l.days} days · {bgName(l.background)}
                      {l.daily ? ` · daily ${l.daily}` : ''}
                    </span>
                  </span>
                  <span class="best__score num">{l.score}</span>
                </button>
                {open === i && (
                  <div class="best__detail">
                    <p class="prose prose--dim">{l.text}</p>
                    <LedgerTable ledger={l} />
                  </div>
                )}
              </li>
            ))}
          </ol>
        )}
      </section>

      <section class="section">
        <Rule>Codex, ever found</Rule>
        <div class="figure-line">
          <span class="label">Entries</span>
          <span class="num">
            {profile.codex_found}/{profile.codex_total}
          </span>
        </div>
        <Hairline value={profile.codex_found} max={profile.codex_total} label="Codex entries ever found" />
      </section>

      <section class="section">
        <Rule>The daily river</Rule>
        <div class="figure-line">
          <span class="label">Streak</span>
          <span class="num">
            {profile.daily_streak} (best {profile.daily_best_streak})
          </span>
        </div>
        {profile.daily_results.length > 0 && (
          <dl class="table">
            {profile.daily_results
              .slice(-10)
              .reverse()
              .map((r) => (
                <div key={r.date} class="table__row">
                  <dt class="num">{r.date}</dt>
                  <dd class="num">
                    {r.days} d · {r.score}
                  </dd>
                </div>
              ))}
          </dl>
        )}
      </section>

      <section class="section">
        <Rule>All runs</Rule>
        <dl class="table">
          {totals.map(([k, v]) => (
            <div key={k} class="table__row">
              <dt>{k}</dt>
              <dd class="num">{v}</dd>
            </div>
          ))}
        </dl>
      </section>
    </Page>
  );
}

export function Unlocks({
  profile,
  onBack,
  onUnlock,
}: {
  profile: ProfileView;
  onBack: () => void;
  onUnlock: (u: Unlock) => void;
}) {
  return (
    <Page title="Remnants" kicker={`${profile.remnants} held`} onBack={onBack}>
      <p class="prose prose--dim">
        Each run that ends leaves something behind. Spend it on who you were, or on what you kept. Nothing here makes
        you stronger. It only changes the start.
      </p>
      <section class="section">
        <Rule>Who you were</Rule>
        <ul class="rows">
          {profile.backgrounds.map((b) => (
            <li key={b.id} class={b.unlocked ? 'unlock unlock--open' : 'unlock'}>
              <Icon name={b.icon} size={24} />
              <div class="unlock__body">
                <p class="unlock__name">{b.name}</p>
                <p class="unlock__text">{b.unlocked ? b.desc : b.unlock_mark ? `Or earn the mark ${b.unlock_mark}.` : b.desc}</p>
              </div>
              {b.unlocked ? (
                <span class="label unlock__state">Open</span>
              ) : (
                <Button
                  variant="quiet"
                  disabled={profile.remnants < b.cost}
                  onClick={() => onUnlock({ type: 'background', id: b.id })}
                >
                  {`${b.cost}`}
                </Button>
              )}
            </li>
          ))}
        </ul>
      </section>
      <section class="section">
        <Rule>Something kept</Rule>
        <ul class="rows">
          {profile.keepsakes.map((k) => (
            <li key={k.id} class={k.unlocked ? 'unlock unlock--open' : 'unlock'}>
              <Icon name={k.icon} size={24} />
              <div class="unlock__body">
                <p class="unlock__name">{k.name}</p>
                <p class="unlock__text">{k.desc}</p>
              </div>
              {k.unlocked ? (
                <span class="label unlock__state">Kept</span>
              ) : (
                <Button
                  variant="quiet"
                  disabled={profile.remnants < k.cost}
                  onClick={() => onUnlock({ type: 'keepsake', id: k.id })}
                >
                  {`${k.cost}`}
                </Button>
              )}
            </li>
          ))}
        </ul>
      </section>
    </Page>
  );
}

const GUIDE: [string, string[]][] = [
  [
    'Time',
    [
      'Nothing happens until you act. Every action costs minutes, shown beside it.',
      'Dusk is at seven. After dark they are awake. Be home, and bar the door.',
    ],
  ],
  [
    'Noise',
    [
      'They come to sound. Forcing, breaking, hurrying and firing all add to the day’s noise.',
      'The header shows the noise you have made and the chance that something comes to the house tonight.',
      'Quiet ways take longer. That is the trade.',
    ],
  ],
  [
    'The body',
    [
      'Six things keep you alive: health, satiety, hydration, rest, nerve and warmth. Any at nothing will start to kill you.',
      'A wound that bleeds must be bound. River water must be boiled.',
    ],
  ],
  [
    'The house',
    [
      'Build and improve the stations. Each level opens new work at the bench and the stove.',
      'Keep the barricade mended. Keep a fire through the cold nights.',
    ],
  ],
  [
    'Going out',
    [
      'Choose a place on the map, pack within what you can carry, and walk. Rooms are searched quietly or fast.',
      'Some rooms are dark and need a light. Some are locked. Some have them standing in them.',
    ],
  ],
  [
    'Leaving',
    [
      'The ferry needs four parts: a hull patch, fuel, a spark coil and a drive belt. Fit them and cast off before the river freezes.',
      'Death is permanent. The game keeps itself after every action.',
    ],
  ],
];

export function HowTo({ onBack }: { onBack: () => void }) {
  return (
    <Page title="How to survive" kicker="A few things" onBack={onBack}>
      {GUIDE.map(([head, lines]) => (
        <section key={head} class="section">
          <Rule>{head}</Rule>
          <div class="prose guide">
            {lines.map((l) => (
              <p key={l}>{l}</p>
            ))}
          </div>
        </section>
      ))}
    </Page>
  );
}

export function Fatal({ message }: { message: string | null }) {
  return (
    <Page title="The game has stopped" kicker="An internal error">
      <p class="prose">
        Something inside the game failed, and it cannot go on safely. Your last saved moment is kept. Reload to carry on
        from there.
      </p>
      {message && <p class="fatal__detail num">{message}</p>}
      <div class="page__actions">
        <Button block onClick={() => window.location.reload()}>
          Reload
        </Button>
      </div>
    </Page>
  );
}
