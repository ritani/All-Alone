import { useState } from 'preact/hooks';
import type { Action, ActionView } from '../../core';
import { duration, hasQty, withQty } from '../format';
import { Icon } from './Icon';
import { Stepper } from './Stepper';

export interface ActionButtonProps {
  a: ActionView;
  onAct: (action: Action) => void;
  /** Hide the icon (dense lists). */
  plain?: boolean;
}

/** The cost line under an action, as icons and numbers: time, noise, odds. */
export function Chips({ minutes, noise, odds }: { minutes: number; noise: number; odds: number | null }) {
  const parts: { key: string; icon: string | null; text: string; label: string; loud?: boolean }[] = [];
  if (minutes > 0) parts.push({ key: 'm', icon: 'clock', text: duration(minutes), label: 'takes' });
  if (noise > 0) parts.push({ key: 'n', icon: 'noise', text: String(noise), label: 'noise', loud: noise >= 6 });
  if (odds !== null) parts.push({ key: 'o', icon: null, text: `${odds}%`, label: 'odds' });
  if (parts.length === 0) return null;
  return (
    <span class="chips">
      {parts.map((p) => (
        <span key={p.key} class={p.loud ? 'chip chip--loud' : 'chip'}>
          {p.icon && <Icon name={p.icon} size={16} class="chip__icon" />}
          <span class="visually-hidden">{p.label} </span>
          {p.text}
        </span>
      ))}
    </span>
  );
}

/**
 * One action from `view.actions`: label, icon, cost chips, and the reason it
 * cannot be done. Quantity actions get a stepper when more than one is possible.
 */
export function ActionButton({ a, onAct, plain = false }: ActionButtonProps) {
  const qtyAction = hasQty(a.action) && a.enabled && a.max_qty > 1;
  const [qty, setQty] = useState(1);
  const shown = Math.min(qty, Math.max(1, a.max_qty));
  const cls = ['act', a.danger ? 'act--danger' : '', a.enabled ? '' : 'act--off', qtyAction ? 'act--qty' : '']
    .filter(Boolean)
    .join(' ');
  const button = (
    <button
      type="button"
      class="act__btn"
      disabled={!a.enabled}
      onClick={() => onAct(qtyAction ? withQty(a.action, shown) : a.action)}
    >
      {!plain && <Icon name={a.icon} size={20} class="act__icon" />}
      <span class="act__body">
        <span class="act__label">
          {a.label}
          {qtyAction && shown > 1 ? <span class="num act__times"> x{shown}</span> : null}
        </span>
        {a.enabled ? (
          <Chips minutes={a.minutes} noise={a.noise} odds={a.odds} />
        ) : (
          a.reason && <span class="act__reason">{a.reason}</span>
        )}
      </span>
    </button>
  );
  if (!qtyAction) return <div class={cls}>{button}</div>;
  return (
    <div class={cls}>
      {button}
      <Stepper value={shown} min={1} max={a.max_qty} onChange={setQty} label={`How many: ${a.label}`} />
    </div>
  );
}

/** A list of actions; `grid` sets them two to a row as compact tiles. */
export function ActionList({
  actions,
  onAct,
  plain,
  grid = false,
}: {
  actions: ActionView[];
  onAct: (a: Action) => void;
  plain?: boolean;
  grid?: boolean;
}) {
  return (
    <ul class={grid ? 'acts acts--grid' : 'acts'}>
      {actions.map((a, i) => (
        <li key={`${a.label}-${i}`}>
          <ActionButton a={a} onAct={onAct} plain={plain ?? false} />
        </li>
      ))}
    </ul>
  );
}
