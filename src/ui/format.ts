import type { Action, ActionView } from '../core';

/** "5 min", "1 h", "1 h 30". */
export function duration(minutes: number): string {
  if (minutes < 60) return `${minutes} min`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m === 0 ? `${h} h` : `${h} h ${String(m).padStart(2, '0')}`;
}

/** Whether an action carries a quantity the player may choose. */
export function hasQty(a: Action): a is Extract<Action, { qty: number }> {
  return 'qty' in a;
}

export function withQty(a: Action, qty: number): Action {
  return hasQty(a) ? { ...a, qty } : a;
}

export const GROUP_NAMES: Record<string, string> = {
  survive: 'Eat, drink, mend',
  house: 'The house',
  station: 'Build',
  craft: 'Craft',
  trip: 'Set out',
  food: 'Food',
  ferry: 'The ferry',
  pack: 'Pack',
  move: 'Move',
  room: 'The room',
  lock: 'The door',
  loot: 'Take and leave',
  fight: 'Fight',
  choice: 'Choose',
  trade: 'Trade',
  night: 'Night',
};

export function groupName(group: string): string {
  return GROUP_NAMES[group] ?? group.replace(/_/g, ' ');
}

/** Actions grouped by `group`, in first-seen order unless an order is given. */
export function groupActions(actions: ActionView[], order: string[] = []): [string, ActionView[]][] {
  const map = new Map<string, ActionView[]>();
  for (const g of order) map.set(g, []);
  for (const a of actions) {
    const list = map.get(a.group);
    if (list) list.push(a);
    else map.set(a.group, [a]);
  }
  return [...map.entries()].filter(([, list]) => list.length > 0);
}

/** A stable key for an action (for lists). */
export function actionKey(a: Action): string {
  return JSON.stringify(hasQty(a) ? { ...a, qty: 0 } : a);
}

export function signed(n: number): string {
  return n > 0 ? `+${n}` : String(n);
}

export function titleCase(s: string): string {
  return s.length === 0 ? s : s[0]!.toUpperCase() + s.slice(1);
}
