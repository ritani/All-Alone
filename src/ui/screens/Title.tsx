import { Button } from '../components/Button';
import { LockHouseArt } from '../components/LockHouseArt';

export interface TitleProps {
  onBegin?: (() => void) | undefined;
  onContinue?: (() => void) | undefined;
  /** True when an autosave exists. Not wired until the save system lands. */
  canContinue?: boolean;
}

export function Title({ onBegin, onContinue, canContinue = false }: TitleProps) {
  return (
    <main class="frame title fade-in">
      <header class="title__head">
        <h1 class="wordmark">ALL ALONE</h1>
        <p class="label title__place">Calder&#8217;s Reach</p>
      </header>

      <figure class="title__art">
        <LockHouseArt />
      </figure>

      <div class="title__prose prose">
        <p>The town stopped in the night. Some of it moves again after dark, and it comes to sound.</p>
        <p class="prose--dim">The ferry on the far slip could carry one person downriver, if it ran. The river will freeze.</p>
      </div>

      <nav class="title__actions" aria-label="Start">
        <Button block onClick={onBegin}>
          Begin
        </Button>
        <Button block variant="quiet" disabled={!canContinue} onClick={onContinue}>
          Continue
        </Button>
      </nav>
    </main>
  );
}
