import { Button } from '../components/Button';
import { Icon } from '../components/Icon';
import { LockHouseArt } from '../components/LockHouseArt';
import { useFocusHeading } from '../components/Page';
import type { View } from '../../core';
import type { ProfileView } from '../../core';

export interface TitleProps {
  onBegin?: (() => void) | undefined;
  onContinue?: (() => void) | undefined;
  /** True when a valid run save exists. */
  canContinue?: boolean;
  /** The run in progress, for the Continue line. */
  run?: View | null;
  profile?: ProfileView | null;
  notices?: string[];
  onDismissNotice?: (i: number) => void;
  onNav?: (to: 'marks' | 'record' | 'unlocks' | 'howto') => void;
}

export function Title({
  onBegin,
  onContinue,
  canContinue = false,
  run,
  profile,
  notices = [],
  onDismissNotice,
  onNav,
}: TitleProps) {
  const h = useFocusHeading<HTMLHeadingElement>();
  return (
    <main class="frame title fade-in">
      <header class="title__head">
        <h1 class="wordmark" tabIndex={-1} ref={h}>
          ALL ALONE
        </h1>
        <p class="label title__place">Calder&#8217;s Reach</p>
      </header>

      <figure class="title__art">
        <LockHouseArt />
      </figure>

      {notices.length > 0 && (
        <ul class="notices" aria-label="Notices">
          {notices.map((n, i) => (
            <li key={n} class="notice">
              <p>{n}</p>
              {onDismissNotice && (
                <button type="button" class="notice__close" aria-label="Dismiss notice" onClick={() => onDismissNotice(i)}>
                  <Icon name="close" size={16} />
                </button>
              )}
            </li>
          ))}
        </ul>
      )}

      <div class="title__prose prose">
        <p>The town stopped in the night. Some of it moves again after dark, and it comes to sound.</p>
        <p class="prose--dim">The ferry on the far slip could carry one person downriver, if it ran. The river will freeze.</p>
      </div>

      <nav class="title__actions" aria-label="Start">
        {canContinue && (
          <Button block onClick={onContinue}>
            Continue
          </Button>
        )}
        {canContinue && run && (
          <p class="title__run num">
            Day {run.day}, {run.clock}. {run.run.background_name}. {run.run.difficulty_name}.
          </p>
        )}
        <Button block variant={canContinue ? 'quiet' : 'primary'} onClick={onBegin}>
          {canContinue ? 'Begin again' : 'Begin'}
        </Button>
        {onNav && (
          <ul class="title__links">
            <li>
              <button type="button" class="link" onClick={() => onNav('marks')}>
                Marks
                {profile && (
                  <span class="num link__aside">
                    {profile.marks_earned}/{profile.marks_total}
                  </span>
                )}
              </button>
            </li>
            <li>
              <button type="button" class="link" onClick={() => onNav('record')}>
                Ledger
                {profile && profile.totals.runs > 0 && <span class="num link__aside">{profile.totals.runs}</span>}
              </button>
            </li>
            <li>
              <button type="button" class="link" onClick={() => onNav('unlocks')}>
                Remnants
                {profile && <span class="num link__aside">{profile.remnants}</span>}
              </button>
            </li>
            <li>
              <button type="button" class="link" onClick={() => onNav('howto')}>
                How to survive
              </button>
            </li>
          </ul>
        )}
      </nav>
    </main>
  );
}
