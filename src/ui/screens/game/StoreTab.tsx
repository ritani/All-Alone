import { useState } from 'preact/hooks';
import type { Action, ActionView, ItemView, View } from '../../../core';
import { ActionList } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Fold, ListRow } from '../../components/ListRow';
import { Sheet } from '../../components/Sheet';

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

/** Items grouped by kind, in the store's order. */
export function byKind(items: ItemView[]): [string, ItemView[]][] {
  const kinds = [...new Set(items.map((i) => i.kind))].sort(
    (a, b) => (KIND_ORDER.indexOf(a) + 1 || 99) - (KIND_ORDER.indexOf(b) + 1 || 99),
  );
  return kinds.map((k) => [k, items.filter((i) => i.kind === k)]);
}

/** One item in full: what it does, what it is, and what can be done with it. */
export function ItemSheet({
  item,
  actions,
  onAct,
  onClose,
}: {
  item: ItemView;
  actions: ActionView[];
  onAct: (a: Action) => void;
  onClose: () => void;
}) {
  const mine = itemActions(actions, item.id);
  return (
    <Sheet title={item.name} onClose={onClose}>
      <p class="item__figures">
        <Icon name={item.icon} size={20} />
        <span class="num">x{item.qty}</span>
        <span class="num">{item.weight_label}</span>
        {item.uses_left !== null && <span class="num">{item.uses_left} uses left</span>}
      </p>
      {item.detail && <p class="item__detail num">{item.detail}</p>}
      <p class="prose prose--dim">{item.desc}</p>
      {mine.length > 0 && <ActionList actions={mine} onAct={onAct} />}
    </Sheet>
  );
}

/** Compact item rows: name, how many, weight. A row opens the item's sheet. */
export function ItemList({ items, onOpen }: { items: ItemView[]; onOpen?: ((item: ItemView) => void) | undefined }) {
  return (
    <ul class="lrows">
      {items.map((it) => (
        <li key={it.id}>
          <ListRow
            icon={it.icon}
            title={
              <>
                {it.name}
                {it.qty > 1 && <span class="num lrow__qty"> x{it.qty}</span>}
              </>
            }
            aside={<span class="num">{it.weight_label}</span>}
            onClick={onOpen ? () => onOpen(it) : undefined}
          />
        </li>
      ))}
    </ul>
  );
}

/** Which groups the player has opened, kept across tab changes. */
const opened = new Set<string>(['food', 'drink', 'medicine']);

export function StoreTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const inv = view.inventory;
  const [, redraw] = useState(0);
  const [item, setItem] = useState<string | null>(null);
  const toggle = (k: string) => {
    if (opened.has(k)) opened.delete(k);
    else opened.add(k);
    redraw((n) => n + 1);
  };
  const groups: [string, string, ItemView[]][] = [
    ...(inv.worn.length > 0 ? [['worn', 'Worn', inv.worn] as [string, string, ItemView[]]] : []),
    ...(inv.pack.length > 0 ? [['pack', 'In the pack', inv.pack] as [string, string, ItemView[]]] : []),
    ...byKind(inv.store).map(([k, list]) => [k, KIND_NAMES[k] ?? k, list] as [string, string, ItemView[]]),
  ];
  const all = [...inv.worn, ...inv.pack, ...inv.store];
  const shown = item ? all.find((i) => i.id === item) : undefined;
  return (
    <>
      <section class="section store__figures">
        <div class="figure">
          <span class="figure__name">
            <Icon name="water" size={16} /> Water
          </span>
          <span class="num">
            {inv.water}/{inv.water_capacity}
          </span>
          <Hairline value={inv.water} max={inv.water_capacity} label="Water held" />
        </div>
        <div class="figure">
          <span class="figure__name">
            <Icon name="pack" size={16} /> Carry
          </span>
          <span class="num">{inv.carry_label}</span>
        </div>
      </section>
      {inv.store.length === 0 && <p class="prose prose--dim">The shelves are bare.</p>}
      <section class="section folds">
        {groups.map(([k, name, list]) => (
          <Fold
            key={k}
            title={name}
            count={<span class="num">{list.reduce((n, i) => n + i.qty, 0)}</span>}
            open={opened.has(k)}
            onToggle={() => toggle(k)}
          >
            <ItemList items={list} onOpen={(it) => setItem(it.id)} />
          </Fold>
        ))}
      </section>
      {shown && <ItemSheet item={shown} actions={view.actions} onAct={onAct} onClose={() => setItem(null)} />}
    </>
  );
}
