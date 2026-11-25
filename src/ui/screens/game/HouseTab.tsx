import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import type { Action, ActionView, GroundsView, ItemView, StationView, View } from '../../../core';
import { ActionButton, ActionList, Chips } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { ListRow, Pips } from '../../components/ListRow';
import { Rule } from '../../components/Rule';
import { Sheet } from '../../components/Sheet';

/** Groups drawn on the House tab. Others have their own tab. */
export const HOUSE_GROUPS = ['ferry', 'house', 'survive', 'night'];

export function Costs({ costs }: { costs: { item: string; name: string; icon: string; need: number; have: number }[] }) {
  return (
    <ul class="costs">
      {costs.map((c) => {
        const short = c.have < c.need;
        return (
          <li key={c.item} class={short ? 'cost cost--short' : 'cost'}>
            <Icon name={c.icon} size={16} />
            <span class="cost__name">{c.name}</span>
            <span class="cost__qty num">
              {c.have}/{c.need}
            </span>
            {short && <span class="visually-hidden">(short)</span>}
          </li>
        );
      })}
    </ul>
  );
}

// ------------------------------------------------------------------ eat, drink, mend

type Need = 'eat' | 'drink' | 'mend';
const NEEDS: { id: Need; name: string; icon: string }[] = [
  { id: 'eat', name: 'Eat', icon: 'food' },
  { id: 'drink', name: 'Drink', icon: 'water' },
  { id: 'mend', name: 'Mend', icon: 'bandage' },
];

/** Which of eat, drink or mend an item answers, by its kind. */
function needOf(a: ActionView, items: Map<string, ItemView>): Need {
  const id = 'item' in a.action ? a.action.item : '';
  const kind = items.get(id)?.kind;
  if (kind === 'food') return 'eat';
  if (kind === 'drink') return 'drink';
  return 'mend';
}

/**
 * Eat, drink, mend: three buttons on one line, each opening a short sheet of
 * what can be used now. Used at home, on the road and in a fight.
 */
export function Provisions({ view, onAct, always = false }: { view: View; onAct: (a: Action) => void; always?: boolean }) {
  const [open, setOpen] = useState<Need | null>(null);
  const inv = view.inventory;
  const items = new Map([...inv.worn, ...inv.store, ...inv.pack].map((i) => [i.id, i]));
  const list = view.actions.filter((a) => a.group === 'survive' && a.enabled);
  const by = (n: Need) => list.filter((a) => needOf(a, items) === n);
  const shown = open ? by(open) : [];
  const vital = (id: string) => view.vitals.find((v) => v.id === id);
  const hint: Record<Need, ReturnType<typeof vital>> = { eat: vital('satiety'), drink: vital('hydration'), mend: vital('health') };
  if (list.length === 0 && !always) return null;
  return (
    <div class="needs3" role="group" aria-label="Eat, drink, mend">
      {NEEDS.map((n) => {
        const count = by(n.id).length;
        const v = hint[n.id];
        const low = v && v.level !== 'ok';
        return (
          <button
            key={n.id}
            type="button"
            class={low ? 'need3 need3--low' : 'need3'}
            disabled={count === 0}
            onClick={() => setOpen(n.id)}
            aria-label={`${n.name}: ${count} ${count === 1 ? 'thing' : 'things'} to hand`}
          >
            <Icon name={n.icon} size={20} />
            <span class="need3__name">{n.name}</span>
            <span class="need3__count num">{count}</span>
          </button>
        );
      })}
      {open && (
        <Sheet title={NEEDS.find((n) => n.id === open)!.name} onClose={() => setOpen(null)}>
          {shown.length === 0 ? (
            <p class="prose prose--dim">Nothing to hand.</p>
          ) : (
            <ActionList
              actions={shown}
              onAct={(a) => {
                onAct(a);
                setOpen(null);
              }}
            />
          )}
        </Sheet>
      )}
    </div>
  );
}

// ------------------------------------------------------------------ grounds

const FOOD_TYPES: Record<string, string> = {
  set_snares: 'snares',
  check_snares: 'snares',
  set_line: 'line',
  check_line: 'line',
  plant: 'garden',
  tend_garden: 'garden',
  harvest: 'garden',
  forage: 'bank',
};

interface Ground {
  id: string;
  icon: string;
  name: string;
  state: ComponentChildren;
  hint?: string | undefined;
  actions: ActionView[];
  body?: ComponentChildren;
  danger?: boolean;
}

function grounds(g: GroundsView, actions: ActionView[]): Ground[] {
  const of = (k: string) => actions.filter((a) => FOOD_TYPES[a.action.type] === k);
  const out: Ground[] = [];
  const low = g.barricade_max > 0 && g.barricade / g.barricade_max < 0.25;
  out.push({
    id: 'barricade',
    icon: 'barricade',
    name: 'Barricade',
    state: (
      <span class="num">
        {g.barricade}/{g.barricade_max}
      </span>
    ),
    actions: actions.filter((a) => a.action.type === 'repair_barricade'),
    danger: low,
  });
  out.push({ id: 'fire', icon: 'stove', name: 'Fire', state: g.fire_until ? `Until ${g.fire_until}` : 'Out', actions: [] });
  const snares = of('snares');
  if (g.snares_set > 0 || snares.length > 0)
    out.push({
      id: 'snares',
      icon: 'trap',
      name: 'Trap line',
      state: g.snares_set === 0 ? 'Not set' : `${g.snares_set} set${g.snared > 0 ? `, ${g.snared} caught` : ''}`,
      hint: g.snares_set === 0 ? 'Set them by day. Walk the line after a night or two.' : 'Walk the line by day. Bait and weather decide the catch.',
      actions: snares,
    });
  const line = of('line');
  if (g.line_set || line.length > 0)
    out.push({
      id: 'line',
      icon: 'fish',
      name: 'Night line',
      state: !g.line_set ? 'Not set' : `${g.line_baited ? 'Baited' : 'Set'}${g.hooked > 0 ? `, ${g.hooked} on it` : ''}`,
      hint: g.line_set ? 'Haul it in after a night. Bait helps.' : 'Set it from the lock gate; it fishes overnight.',
      actions: line,
    });
  if (g.plots_total > 0)
    out.push({
      id: 'garden',
      icon: 'garden',
      name: 'Garden',
      state: `${g.plots.length}/${g.plots_total} sown${g.tended_today ? ', watered' : ''}`,
      hint: g.plots.length > 0 && !g.tended_today ? 'Water it each day it does not rain.' : undefined,
      actions: of('garden'),
      body:
        g.plots.length > 0 ? (
          <ul class="plots">
            {g.plots.map((p, i) => (
              <li key={i} class="plot">
                <span class="plot__name">{p.name}</span>
                <Hairline value={p.growth} max={p.need} label={`${p.name} growth`} />
                <span class="plot__state num">{p.ready ? 'Ready' : `${p.growth}/${p.need}`}</span>
              </li>
            ))}
          </ul>
        ) : undefined,
    });
  const bank = of('bank');
  if (bank.length > 0) out.push({ id: 'bank', icon: 'herb', name: 'The bank', state: '', actions: bank });
  return out;
}

function Grounds({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const [open, setOpen] = useState<string | null>(null);
  const food = view.actions.filter((a) => a.group === 'food');
  const list = grounds(view.grounds, food);
  const g = list.find((x) => x.id === open);
  return (
    <section class="section">
      <Rule>Grounds</Rule>
      <ul class="lrows">
        {list.map((x) => {
          // The barricade is mended from the list above; its row is state only.
          const choices = x.id === 'barricade' ? [] : x.actions;
          const ready = choices.some((a) => a.enabled);
          return (
            <li key={x.id}>
              <ListRow
                icon={x.icon}
                title={x.name}
                aside={x.state}
                danger={x.danger ?? false}
                flag={ready ? 'Something to do' : undefined}
                onClick={choices.length > 0 || x.body ? () => setOpen(x.id) : undefined}
              />
            </li>
          );
        })}
      </ul>
      {g && (
        <Sheet title={g.name} onClose={() => setOpen(null)}>
          <p class="sheet__state">{g.state}</p>
          {g.body}
          {g.hint && <p class="sheet__hint">{g.hint}</p>}
          {g.actions.length > 0 && <ActionList actions={g.actions} onAct={onAct} />}
        </Sheet>
      )}
    </section>
  );
}

// ------------------------------------------------------------------ stations

function StationTile({ s, onOpen }: { s: StationView; onOpen: () => void }) {
  const ready = s.next?.can_build === true;
  return (
    <li>
      <button
        type="button"
        class={s.level > 0 ? 'stile stile--built' : 'stile'}
        onClick={onOpen}
        aria-label={`${s.name}, level ${s.level} of 3${ready ? ', can be built up now' : ''}`}
      >
        {ready && <span class="stile__flag" aria-hidden="true" />}
        <Icon name={s.icon} size={24} class="stile__icon" />
        <span class="stile__name">{s.name}</span>
        <Pips value={s.level} max={3} label={`Level ${s.level} of 3`} />
      </button>
    </li>
  );
}

export function StationSheet({ s, build, onAct, onClose }: { s: StationView; build: ActionView | undefined; onAct: (a: Action) => void; onClose: () => void }) {
  return (
    <Sheet title={s.name} onClose={onClose}>
      <div class="station__now">
        <Icon name={s.icon} size={32} class="station__icon" />
        <div>
          <p class="station__level">
            {s.level_name} <span class="num">· {s.level}/3</span>
          </p>
          {s.status && <p class="station__status">{s.status}</p>}
        </div>
      </div>
      <p class="station__effect">{s.effect}</p>
      {s.next ? (
        <div class="station__next">
          <Rule level={3}>Next: {s.next.name}</Rule>
          <p class="station__effect station__effect--dim">{s.next.effect}</p>
          <Costs costs={s.next.cost} />
          {s.next.workbench > 0 && (
            <p class="station__req">
              <Icon name="workbench" size={16} /> Workbench {s.next.workbench}
            </p>
          )}
          {build ? (
            <ActionButton a={build} onAct={onAct} />
          ) : (
            <Chips minutes={s.next.minutes} noise={s.next.noise} odds={null} />
          )}
        </div>
      ) : (
        <p class="station__effect station__effect--dim">Built as far as it goes.</p>
      )}
    </Sheet>
  );
}

function Stations({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const [open, setOpen] = useState<string | null>(null);
  const builds = new Map(
    view.actions.filter((a) => a.action.type === 'build').map((a) => [(a.action as { station: string }).station, a]),
  );
  const s = view.stations.find((x) => x.id === open);
  return (
    <section class="section">
      <Rule>Stations</Rule>
      <ul class="stiles">
        {view.stations.map((st) => (
          <StationTile key={st.id} s={st} onOpen={() => setOpen(st.id)} />
        ))}
      </ul>
      {s && <StationSheet s={s} build={builds.get(s.id)} onAct={onAct} onClose={() => setOpen(null)} />}
    </section>
  );
}

// ------------------------------------------------------------------ the tab

export function HouseTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const ferry = view.actions.filter((a) => a.group === 'ferry');
  // What can be done right now in the house, the door last, and what cannot yet after it.
  const all = [...view.actions.filter((a) => a.group === 'house'), ...view.actions.filter((a) => a.group === 'night')];
  const now = [...all.filter((a) => a.enabled), ...all.filter((a) => !a.enabled)];
  return (
    <>
      {ferry.length > 0 && (
        <section class="section">
          <Rule>The ferry</Rule>
          <ActionList actions={ferry} onAct={onAct} />
        </section>
      )}
      <section class="section" aria-label="The house">
        <ActionList actions={now} onAct={onAct} grid />
        <Provisions view={view} onAct={onAct} always />
      </section>
      <Grounds view={view} onAct={onAct} />
      <Stations view={view} onAct={onAct} />
    </>
  );
}
