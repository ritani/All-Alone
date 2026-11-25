import { Button } from '../components/Button';
import { useFocusHeading } from '../components/Page';
import type { View } from '../../core';

export interface IntroProps {
  lines: string[];
  view: View | null;
  dailyCounts: boolean | null;
  onGo: () => void;
}

/** The core's opening prose, line by line, before the first morning. */
export function Intro({ lines, view, dailyCounts, onGo }: IntroProps) {
  const h = useFocusHeading<HTMLHeadingElement>();
  return (
    <main class="frame intro">
      <h1 class="visually-hidden" tabIndex={-1} ref={h}>
        Calder&#8217;s Reach
      </h1>
      <div class="intro__prose prose">
        {lines.map((l, i) => (
          <p key={i} class={`intro__line intro__line--${Math.min(i, 7)}`}>
            {l}
          </p>
        ))}
      </div>
      <div class="intro__foot intro__line intro__line--8">
        {view && (
          <p class="label intro__who">
            {view.run.background_name}. {view.run.difficulty_name}.
            {view.run.daily ? ` The daily river, ${view.run.daily}.` : ''}
          </p>
        )}
        {dailyCounts === false && (
          <p class="intro__note">You have taken today&#8217;s river before. This attempt will not count.</p>
        )}
        <Button block onClick={onGo}>
          Go on
        </Button>
      </div>
    </main>
  );
}
