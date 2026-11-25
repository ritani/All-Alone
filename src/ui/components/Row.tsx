import type { ComponentChildren } from 'preact';
import { Icon } from './Icon';

export interface RowProps {
  icon?: string;
  title: ComponentChildren;
  /** Right-aligned figure (quantity, weight, level). */
  aside?: ComponentChildren;
  children?: ComponentChildren;
  dim?: boolean;
  danger?: boolean;
}

/** A list row: icon, title, an aside figure, and a body underneath. */
export function Row({ icon, title, aside, children, dim = false, danger = false }: RowProps) {
  const cls = ['row', dim ? 'row--dim' : '', danger ? 'row--danger' : ''].filter(Boolean).join(' ');
  return (
    <div class={cls}>
      <div class="row__head">
        {icon && <Icon name={icon} size={20} class="row__icon" />}
        <span class="row__title">{title}</span>
        {aside !== undefined && <span class="row__aside num">{aside}</span>}
      </div>
      {children && <div class="row__body">{children}</div>}
    </div>
  );
}
