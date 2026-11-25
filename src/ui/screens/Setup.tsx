import { useMemo, useState } from 'preact/hooks';
import { dailySetup, type Difficulty, type ProfileView } from '../../core';
import { Button } from '../components/Button';
import { Icon } from '../components/Icon';
import { Page } from '../components/Page';
import { Rule } from '../components/Rule';
import { Sheet } from '../components/Sheet';
import { localDate } from '../state/store';
import { titleCase } from '../format';

export interface SetupProps {
  profile: ProfileView;
  hasRun: boolean;
  onBack: () => void;
  onStart: (opts: { difficulty: Difficulty; background: string; keepsake: string | null }) => void;
  onDaily: () => void;
  onUnlocks: () => void;
}

const SKILL_NAMES: Record<string, string> = {
  quiet_step: 'Quiet step',
  lockwork: 'Lockwork',
  mending: 'Mending',
  marksmanship: 'Marksmanship',
  foraging: 'Foraging',
  field_medicine: 'Field medicine',
};

/** New run: background, keepsake, difficulty; or the daily river. */
export function Setup({ profile, hasRun, onBack, onStart, onDaily, onUnlocks }: SetupProps) {
  const firstOpen = profile.backgrounds.find((b) => b.unlocked)?.id ?? 'clerk';
  const [background, setBackground] = useState(firstOpen);
  const [keepsake, setKeepsake] = useState<string | null>(null);
  const [difficulty, setDifficulty] = useState<Difficulty>('still');
  const [confirm, setConfirm] = useState<null | 'run' | 'daily'>(null);

  const today = localDate();
  const daily = useMemo(() => {
    try {
      return dailySetup(today);
    } catch {
      return null;
    }
  }, [today]);
  const dailyBg = daily ? profile.backgrounds.find((b) => b.id === daily.background) : undefined;
  const dailyDiff = daily ? profile.difficulties.find((d) => d.id === daily.difficulty) : undefined;
  const dailyDone = profile.daily_results.find((r) => r.date === today);

  const go = (what: 'run' | 'daily') => {
    if (hasRun && confirm === null) {
      setConfirm(what);
      return;
    }
    setConfirm(null);
    if (what === 'daily') onDaily();
    else onStart({ difficulty, background, keepsake });
  };

  return (
    <Page title="A new run" kicker="The lock house" onBack={onBack} class="setup">
      <fieldset class="choice-set">
        <legend class="rule rule--label">
          <span class="rule__text">Who you were</span>
        </legend>
        {profile.backgrounds.map((b) => (
          <label key={b.id} class={b.unlocked ? 'choice' : 'choice choice--locked'}>
            <input
              type="radio"
              name="background"
              class="visually-hidden"
              value={b.id}
              checked={background === b.id}
              disabled={!b.unlocked}
              onChange={() => setBackground(b.id)}
            />
            <span class="choice__head">
              <Icon name={b.unlocked ? b.icon : 'lock'} size={24} class="choice__icon" />
              <span class="choice__name">{b.name}</span>
              <span class="choice__aside label">
                {SKILL_NAMES[b.skill] ?? b.skill} {b.skill_level}
              </span>
            </span>
            <span class="choice__body">
              <span class="choice__text prose">{b.desc}</span>
              {b.unlocked ? (
                <>
                  <span class="choice__line">{b.drawback}</span>
                  <span class="choice__line choice__line--dim">Brings {b.kit.join(', ').toLowerCase()}.</span>
                </>
              ) : (
                <span class="choice__line choice__line--dim">
                  {b.unlock_mark ? `Earn the mark ${b.unlock_mark}, or spend ${b.cost} remnants.` : `Spend ${b.cost} remnants.`}
                </span>
              )}
            </span>
          </label>
        ))}
      </fieldset>

      <fieldset class="choice-set">
        <legend class="rule rule--label">
          <span class="rule__text">Something kept</span>
        </legend>
        <label class="choice choice--small">
          <input
            type="radio"
            name="keepsake"
            class="visually-hidden"
            checked={keepsake === null}
            onChange={() => setKeepsake(null)}
          />
          <span class="choice__head">
            <Icon name="close" size={20} class="choice__icon" />
            <span class="choice__name">Nothing</span>
          </span>
        </label>
        {profile.keepsakes
          .filter((k) => k.unlocked)
          .map((k) => (
            <label key={k.id} class="choice choice--small">
              <input
                type="radio"
                name="keepsake"
                class="visually-hidden"
                checked={keepsake === k.id}
                onChange={() => setKeepsake(k.id)}
              />
              <span class="choice__head">
                <Icon name={k.icon} size={20} class="choice__icon" />
                <span class="choice__name">{k.name}</span>
              </span>
              <span class="choice__body">
                <span class="choice__line choice__line--dim">{k.desc}</span>
              </span>
            </label>
          ))}
        {profile.keepsakes.some((k) => !k.unlocked) && (
          <button type="button" class="link link--small" onClick={onUnlocks}>
            {profile.keepsakes.filter((k) => !k.unlocked).length} more for remnants
            <span class="num link__aside">{profile.remnants} held</span>
          </button>
        )}
      </fieldset>

      <fieldset class="choice-set">
        <legend class="rule rule--label">
          <span class="rule__text">How hard</span>
        </legend>
        {profile.difficulties.map((d) => (
          <label key={d.id} class="choice choice--small">
            <input
              type="radio"
              name="difficulty"
              class="visually-hidden"
              checked={difficulty === d.id}
              onChange={() => setDifficulty(d.id)}
            />
            <span class="choice__head">
              <span class="choice__name">{d.name}</span>
              <span class="choice__aside num">score {d.score_pct}%</span>
            </span>
            <span class="choice__body">
              <span class="choice__line choice__line--dim">{d.blurb}</span>
            </span>
          </label>
        ))}
      </fieldset>

      <div class="setup__go">
        <Button block onClick={() => go('run')}>
          Begin
        </Button>
      </div>

      {daily && (
        <section class="daily" aria-labelledby="daily-title">
          <Rule>The daily river</Rule>
          <h2 id="daily-title" class="daily__title num">
            {today}
          </h2>
          <p class="prose prose--dim">
            One river for everyone today. {dailyBg?.name ?? titleCase(daily.background)},{' '}
            {(dailyDiff?.name ?? daily.difficulty).toLowerCase()}. The first attempt is the one that counts.
          </p>
          {dailyDone && (
            <p class="daily__done num">
              Counted: {dailyDone.days} days, score {dailyDone.score}.
            </p>
          )}
          <p class="daily__streak label">
            Streak {profile.daily_streak}. Best {profile.daily_best_streak}.
          </p>
          <Button block variant="quiet" icon="sigil_river" onClick={() => go('daily')}>
            Take the daily river
          </Button>
        </section>
      )}

      {confirm && (
        <Sheet title="A run is in progress" onClose={() => setConfirm(null)}>
          <p class="prose">Beginning again leaves the lock house as it is. That run ends here, unrecorded.</p>
          <div class="sheet__actions">
            <Button block variant="danger" onClick={() => go(confirm)}>
              End it and begin
            </Button>
            <Button block variant="quiet" onClick={() => setConfirm(null)}>
              Keep it
            </Button>
          </div>
        </Sheet>
      )}
    </Page>
  );
}
