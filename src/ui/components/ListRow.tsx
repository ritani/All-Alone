import type { ComponentChildren } from 'preact';
import { Icon } from './Icon';

export interface ListRowProps {
  icon?: string | undefined;
  title: ComponentChildren;
  /** One short line under the title (a state, a missing need). */
  sub?: ComponentChildren;
  /** A figure on the right (a count, a state, a weight). */
  aside?: ComponentChildren;
  /** Opens something: the row is a button with a chevron. */
  onClick?: (() => void) | undefined;
  /** A small bone mark: something here can be done now. */
  flag?: string | undefined;
  danger?: boolean;
  dim?: boolean;
  expanded?: boolean | undefined;
  /** Accessible name when the visible text is not enough. */
  label?: string | undefined;
}

/**
 * One compact line: icon, name, a figure on the right. When it opens a sheet
 * it is a button with a chevron; otherwise it is a plain line of state.
 */
export function ListRow({ icon, title, sub, aside, onClick, flag, danger = false, dim = false, expanded, label }: ListRowProps) {
  const cls = ['lrow', onClick ? 'lrow--btn' : '', danger ? 'lrow--danger' : '', dim ? 'lrow--dim' : ''].filter(Boolean).join(' ');
  const inner = (
    <>
      {icon && <Icon name={icon} size={20} class="lrow__icon" />}
      <span class="lrow__body">
        <span class="lrow__title">
          {title}
          {flag && (
            <span class="lrow__flag" role="img" aria-label={flag} title={flag} />
          )}
        </span>
        {sub !== undefined && sub !== null && sub !== '' && <span class="lrow__sub">{sub}</span>}
      </span>
      {aside !== undefined && <span class="lrow__aside">{aside}</span>}
      {onClick && (
        <Icon
          name={expanded === undefined ? 'forward' : 'down'}
          size={16}
          class={expanded ? 'lrow__chev lrow__chev--open' : 'lrow__chev'}
        />
      )}
    </>
  );
  if (!onClick) return <div class={cls}>{inner}</div>;
  return (
    <button type="button" class={cls} onClick={onClick} aria-expanded={expanded} aria-label={label}>
      {inner}
    </button>
  );
}

/** Level pips: one square per level, filled up to `value`. */
export function Pips({ value, max, label }: { value: number; max: number; label: string }) {
  return (
    <span class="pips" role="img" aria-label={label}>
      {Array.from({ length: max }, (_, i) => (
        <span key={i} class={i < value ? 'pip pip--on' : 'pip'} />
      ))}
    </span>
  );
}

/**
 * A folded group: a header row with a count that opens the list under it
 * in place. For secondary lists that should not take the screen by default.
 */
export function Fold({
  icon,
  title,
  count,
  open,
  onToggle,
  children,
}: {
  icon?: string | undefined;
  title: ComponentChildren;
  count?: ComponentChildren;
  open: boolean;
  onToggle: () => void;
  children: ComponentChildren;
}) {
  return (
    <div class={open ? 'fold fold--open' : 'fold'}>
      <ListRow icon={icon} title={title} aside={count} onClick={onToggle} expanded={open} />
      {open && <div class="fold__body">{children}</div>}
    </div>
  );
}
