/**
 * Small pieces shared by the expedition screens: a phase heading, side-by-side
 * choice cards, and an item row that moves a quantity from one place to
 * another (pack, leave, take, sell). Everything they show comes from the
 * core's `ActionView`s; nothing here decides a rule.
 */
import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import type { Action, ActionView, ItemView } from '../../../core';
import { Chips } from '../../components/ActionButton';
import { Icon } from '../../components/Icon';
import { useFocusHeading } from '../../components/Page';
import { Stepper } from '../../components/Stepper';
import { withQty } from '../../format';

export type OnAct = (a: Action) => void;

/** The first action of a type (and matching `pred`). */
export function pick<T extends Action['type']>(
  actions: ActionView[],
  type: T,
  pred: (a: Extract<Action, { type: T }>) => boolean = () => true,
): ActionView | undefined {
  return actions.find((a) => a.action.type === type && pred(a.action as Extract<Action, { type: T }>));
}

/** Actions that name this item, of a type. */
export function itemAct(actions: ActionView[], type: Action['type'], item: string): ActionView | undefined {
  return actions.find((a) => a.action.type === type && 'item' in a.action && a.action.item === item);
}

/** The heading of a phase screen. Focus lands on it when `focusKey` changes. */
export function PhaseHead({
  kicker,
  title,
  icon,
  focusKey,
  children,
}: {
  kicker?: string | undefined;
  title: string;
  icon?: string | undefined;
  focusKey?: unknown;
  children?: ComponentChildren;
}) {
  const h = useFocusHeading<HTMLHeadingElement>(focusKey ?? title);
  return (
    <header class="phase__head">
      {kicker && <p class="label phase__kicker">{kicker}</p>}
      <h2 class="phase__title" tabIndex={-1} ref={h}>
        {icon && <Icon name={icon} size={24} />}
        <span>{title}</span>
      </h2>
      {children}
    </header>
  );
}

/**
 * One of two or three choices drawn side by side (walk or hurry, quiet or
 * loud). The card is the button; its cost and what follows are listed in it.
 */
export function ChoiceCard({
  a,
  onAct,
  title,
  lines = [],
}: {
  a: ActionView | undefined;
  onAct: OnAct;
  title?: string;
  lines?: { text: string; danger?: boolean }[];
}) {
  if (!a) return null;
  const cls = ['choice-card', a.danger ? 'choice-card--danger' : '', a.enabled ? '' : 'choice-card--off']
    .filter(Boolean)
    .join(' ');
  return (
    <button type="button" class={cls} disabled={!a.enabled} onClick={() => onAct(a.action)}>
      <span class="choice-card__head">
        <Icon name={a.icon} size={20} class="choice-card__icon" />
        <span class="choice-card__title">{title ?? a.label}</span>
      </span>
      {a.enabled ? (
        <Chips minutes={a.minutes} noise={a.noise} odds={a.odds} />
      ) : (
        a.reason && <span class="choice-card__reason">{a.reason}</span>
      )}
      {a.enabled &&
        lines.map((l) => (
          <span key={l.text} class={l.danger ? 'choice-card__line choice-card__line--danger' : 'choice-card__line'}>
            {l.text}
          </span>
        ))}
    </button>
  );
}

/** A pair (or three) of choice cards in a row. */
export function Choices({ children, label }: { children: ComponentChildren; label: string }) {
  return (
    <div class="choices" role="group" aria-label={label}>
      {children}
    </div>
  );
}

/**
 * An item with a quantity stepper and a verb: "Pack", "Leave", "Take", "Sell".
 * `aside` is a figure on the right (weight, price).
 */
export function MoveRow({
  item,
  a,
  verb,
  onAct,
  aside,
  note,
  hideReason = false,
}: {
  item: Pick<ItemView, 'id' | 'name' | 'icon' | 'qty'> & { detail?: string };
  a: ActionView | undefined;
  verb: string;
  onAct: OnAct;
  aside?: ComponentChildren;
  note?: string | undefined;
  /** The reason it cannot be done is plain from the screen (the slate is empty). */
  hideReason?: boolean;
}) {
  const [qty, setQty] = useState(1);
  const max = a && a.enabled ? Math.max(1, a.max_qty) : 1;
  const shown = Math.min(qty, max);
  const can = a?.enabled === true;
  const sub = [aside, note ?? item.detail].filter((x) => x !== undefined && x !== null && x !== '');
  return (
    <li class={can ? 'move' : 'move move--off'}>
      <Icon name={item.icon} size={20} class="move__icon" />
      <span class="move__main">
        <span class="move__name">
          {item.name}
          {item.qty > 1 && <span class="num move__qty"> x{item.qty}</span>}
        </span>
        {sub.length > 0 && (
          <span class="move__detail">
            {sub.map((x, i) => (
              <span key={i} class={i === 0 && aside !== undefined ? 'num' : undefined}>
                {i > 0 ? ' · ' : ''}
                {x}
              </span>
            ))}
          </span>
        )}
        {a && !can && a.reason && !hideReason && <span class="move__reason">{a.reason}</span>}
      </span>
      {a && (
        <span class="move__do">
          {can && max > 1 && (
            <Stepper value={shown} min={1} max={max} onChange={setQty} label={`How many: ${item.name}`} />
          )}
          <button
            type="button"
            class="move__btn"
            disabled={!can}
            onClick={() => a && onAct(withQty(a.action, shown))}
            aria-label={`${verb} ${shown > 1 ? `${shown} x ` : ''}${item.name}`}
          >
            {verb}
          </button>
        </span>
      )}
    </li>
  );
}

/** "0.4 kg" from tenths of a kilogram. */
export function kg(tenths: number): string {
  return `${Math.floor(tenths / 10)}.${tenths % 10} kg`;
}
