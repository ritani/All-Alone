import type { ComponentChildren } from 'preact';
import { useEffect, useRef } from 'preact/hooks';
import { Icon } from './Icon';

export interface PageProps {
  title: string;
  /** Small tracked line above the title. */
  kicker?: string | undefined;
  onBack?: (() => void) | undefined;
  backLabel?: string;
  children: ComponentChildren;
  class?: string;
}

/** Moves keyboard and screen-reader focus to a heading when a screen appears. */
export function useFocusHeading<T extends HTMLElement>(dep: unknown = null) {
  const ref = useRef<T>(null);
  useEffect(() => {
    window.scrollTo(0, 0);
    ref.current?.focus({ preventScroll: true });
  }, [dep]);
  return ref;
}

/** A plain screen: back link, heading, content. Focus lands on the heading. */
export function Page({ title, kicker, onBack, backLabel = 'Back', children, class: extra }: PageProps) {
  const h = useFocusHeading<HTMLHeadingElement>(title);
  return (
    <main class={extra ? `frame page fade-in ${extra}` : 'frame page fade-in'}>
      <header class="page__head">
        {onBack && (
          <button type="button" class="page__back" onClick={onBack}>
            <Icon name="back" size={20} />
            <span class="label">{backLabel}</span>
          </button>
        )}
        {kicker && <p class="label page__kicker">{kicker}</p>}
        <h1 class="page__title" tabIndex={-1} ref={h}>
          {title}
        </h1>
      </header>
      {children}
    </main>
  );
}
