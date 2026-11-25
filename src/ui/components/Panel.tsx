import type { ComponentChildren } from 'preact';
import { Rule } from './Rule';

export interface PanelProps {
  title?: string;
  children: ComponentChildren;
  class?: string;
}

/** A raised hairline-bordered surface, optionally headed by a Rule. */
export function Panel({ title, children, class: extra }: PanelProps) {
  return (
    <section class={extra ? `panel ${extra}` : 'panel'}>
      {title && <Rule>{title}</Rule>}
      <div class="panel__body">{children}</div>
    </section>
  );
}
