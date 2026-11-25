import { useState } from 'preact/hooks';
import type { Action, ActionView, ItemView, View } from '../../../core';
import { ActionButton } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Rule } from '../../components/Rule';

export const KIND_ORDER = [
  'food',
  'drink',
  'medicine',
  'part',
  'weapon',
  'ammo',
  'light',
  'tool',
  'bag',
  'clothing',
  'fuel',
  'material',
  'trap',
  'bait',
  'seed',
  'key',
  'comfort',
];
export const KIND_NAMES: Record<string, string> = {
  food: 'Food',
  drink: 'Drink',
  medicine: 'Medicine',
  part: 'Ferry parts',
  weapon: 'Weapons',
  ammo: 'Rounds',
  light: 'Light',
  tool: 'Tools',
  bag: 'Bags',
  clothing: 'Clothing',
  fuel: 'Fuel',
  material: 'Materials',
  trap: 'Traps and lines',
  bait: 'Bait',
  seed: 'Seed',
  key: 'Keys',
  comfort: 'Comforts',
};

/** Actions that name this item (use, drop, sell, pack ...). */
export function itemActions(actions: ActionView[], id: string): ActionView[] {
  return actions.filter((a) => 'item' in a.action && a.action.item === id);
}

function ItemRow({ item, actions, onAct }: { item: ItemView; actions: ActionView[]; onAct: (a: Action) => void }) {
  const [open, setOpen] = useState(false);
  const mine = itemActions(actions, item.id);
  return (
    <li class="item">
      <button type="button" class="item__row" aria-expanded={open} onClick={() => setOpen(!open)}>
        <Icon name={item.icon} size={20} class="item__icon" />
        <span class="item__name">
          {item.name}
          {item.qty > 1 && <span class="num item__qty"> x{item.qty}</span>}
        </span>
        <span class="item__weight num">{item.weight_label}</span>
      </button>
      <p class="item__detail">
        {item.detail}
        {item.uses_left !== null ? ` · ${item.uses_left} uses left` : ''}
      </p>
      {open && <p class="item__desc prose prose--dim">{item.desc}</p>}
      {mine.length > 0 && (
        <ul class="acts acts--inline">
          {mine.map((a, i) => (
            <li key={i}>
              <ActionButton a={a} onAct={onAct} plain />
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}

export function ItemList({ items, actions, onAct }: { items: ItemView[]; actions: ActionView[]; onAct: (a: Action) => void }) {
  return (
    <ul class="items">
      {items.map((it) => (
        <ItemRow key={it.id} item={it} actions={actions} onAct={onAct} />
      ))}
    </ul>
  );
}

export function StoreTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const inv = view.inventory;
  const kinds = [...new Set(inv.store.map((i) => i.kind))].sort(
    (a, b) => (KIND_ORDER.indexOf(a) + 1 || 99) - (KIND_ORDER.indexOf(b) + 1 || 99),
  );
  return (
    <>
      <section class="section">
        <div class="figure-line">
          <span class="label">
            <Icon name="water" size={16} /> Water held
          </span>
          <span class="num">
            {inv.water}/{inv.water_capacity}
          </span>
        </div>
        <Hairline value={inv.water} max={inv.water_capacity} label="Water held" />
        <div class="figure-line">
          <span class="label">
            <Icon name="pack" size={16} /> You can carry
          </span>
          <span class="num">{inv.carry_label}</span>
        </div>
      </section>
      {inv.worn.length > 0 && (
        <section class="section">
          <Rule>Worn</Rule>
          <ItemList items={inv.worn} actions={view.actions} onAct={onAct} />
        </section>
      )}
      {inv.pack.length > 0 && (
        <section class="section">
          <Rule>In the pack</Rule>
          <ItemList items={inv.pack} actions={view.actions} onAct={onAct} />
        </section>
      )}
      {inv.store.length === 0 && <p class="prose prose--dim">The shelves are bare.</p>}
      {kinds.map((k) => (
        <section key={k} class="section">
          <Rule>{KIND_NAMES[k] ?? k}</Rule>
          <ItemList items={inv.store.filter((i) => i.kind === k)} actions={view.actions} onAct={onAct} />
        </section>
      ))}
    </>
  );
}
