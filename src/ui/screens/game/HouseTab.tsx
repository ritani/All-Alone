import type { ComponentChildren } from 'preact';
import type { Action, ActionView, GroundsView, StationView, View } from '../../../core';
import { ActionButton, ActionList } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Rule } from '../../components/Rule';
import { groupActions, groupName } from '../../format';

/** Groups drawn on the House tab, in this order. Others have their own tab. */
export const HOUSE_GROUPS = ['ferry', 'house', 'survive', 'night'];
const OTHER_TABS = new Set(['station', 'craft', 'trip', 'food']);

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

function Station({ s, build, onAct }: { s: StationView; build: ActionView | undefined; onAct: (a: Action) => void }) {
  return (
    <li class="station">
      <div class="station__head">
        <Icon name={s.icon} size={24} class="station__icon" />
        <div class="station__title">
          <p class="station__name">{s.name}</p>
          <p class="station__level label">
            {s.level_name}
            {s.status ? ` · ${s.status}` : ''}
          </p>
        </div>
        <span class="station__lv num" aria-label={`Level ${s.level} of 3`}>
          {s.level}/3
        </span>
      </div>
      <Hairline value={s.level} max={3} steps label={`${s.name} level`} />
      <p class="station__effect">{s.effect}</p>
      {s.next && (
        <div class="station__next">
          <p class="station__next-name label">Next: {s.next.name}</p>
          <p class="station__effect station__effect--dim">{s.next.effect}</p>
          <Costs costs={s.next.cost} />
          {s.next.workbench > 0 && <p class="station__req label">Workbench {s.next.workbench}</p>}
          {build && <ActionButton a={build} onAct={onAct} />}
        </div>
      )}
    </li>
  );
}

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

/** One of the grounds: a name, its state, a line on timing, and its own actions. */
function Ground({
  icon,
  name,
  state,
  hint,
  actions,
  onAct,
  children,
}: {
  icon: string;
  name: string;
  state: string;
  hint?: string | undefined;
  actions: ActionView[];
  onAct: (a: Action) => void;
  children?: ComponentChildren;
}) {
  return (
    <div class="ground">
      <div class="figure-line">
        <span class="label">
          <Icon name={icon} size={16} /> {name}
        </span>
        <span class="ground__state">{state}</span>
      </div>
      {children}
      {hint && <p class="ground__hint">{hint}</p>}
      {actions.length > 0 && <ActionList actions={actions} onAct={onAct} />}
    </div>
  );
}

function Grounds({ g, actions, onAct }: { g: GroundsView; actions: ActionView[]; onAct: (a: Action) => void }) {
  const of = (k: string) => actions.filter((a) => FOOD_TYPES[a.action.type] === k);
  const snares = of('snares');
  const line = of('line');
  const garden = of('garden');
  const bank = of('bank');
  return (
    <section class="section">
      <Rule>Grounds</Rule>
      <div class="ground">
        <div class="figure-line">
          <span class="label">
            <Icon name="barricade" size={16} /> Barricade
          </span>
          <span class="num">
            {g.barricade}/{g.barricade_max}
          </span>
        </div>
        <Hairline
          value={g.barricade}
          max={g.barricade_max}
          label="Barricade"
          danger={g.barricade_max > 0 && g.barricade / g.barricade_max < 0.25}
        />
      </div>
      <div class="figure-line">
        <span class="label">
          <Icon name="stove" size={16} /> Fire
        </span>
        <span>{g.fire_until ? `Burning until ${g.fire_until}` : 'Out'}</span>
      </div>
      {(g.snares_set > 0 || snares.length > 0) && (
        <Ground
          icon="trap"
          name="Trap line"
          state={g.snares_set === 0 ? 'Not set' : `${g.snares_set} set${g.snared > 0 ? `, ${g.snared} caught` : ''}`}
          hint={g.snares_set === 0 ? 'Set them by day. Walk the line after a night or two.' : 'Walk the line by day. Bait and weather decide the catch.'}
          actions={snares}
          onAct={onAct}
        />
      )}
      {(g.line_set || line.length > 0) && (
        <Ground
          icon="fish"
          name="Night line"
          state={!g.line_set ? 'Not set' : `${g.line_baited ? 'Baited' : 'Unbaited'}${g.hooked > 0 ? `, ${g.hooked} on it` : ''}`}
          hint={g.line_set ? 'Haul it in after a night. Bait helps.' : 'Set it from the lock gate; it fishes overnight.'}
          actions={line}
          onAct={onAct}
        />
      )}
      {g.plots_total > 0 && (
        <Ground
          icon="garden"
          name="Garden"
          state={`${g.plots.length}/${g.plots_total} sown${g.tended_today ? ', watered today' : ''}`}
          hint={g.plots.length > 0 && !g.tended_today ? 'Water it each day it does not rain.' : undefined}
          actions={garden}
          onAct={onAct}
        >
          {g.plots.map((p, i) => (
            <div key={i} class="plot">
              <span class="plot__name">{p.name}</span>
              <Hairline value={p.growth} max={p.need} label={`${p.name} growth`} />
              <span class="plot__state label">{p.ready ? 'Ready' : `${p.growth}/${p.need}`}</span>
            </div>
          ))}
        </Ground>
      )}
      {bank.length > 0 && <Ground icon="herb" name="The bank" state="" actions={bank} onAct={onAct} />}
    </section>
  );
}

export function HouseTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const mine = view.actions.filter((a) => !OTHER_TABS.has(a.group));
  // Eating and drinking: show what can be done; the store tab lists the rest.
  const visible = mine.filter((a) => a.group !== 'survive' || a.enabled);
  const groups = groupActions(visible, HOUSE_GROUPS);
  const builds = new Map(
    view.actions.filter((a) => a.action.type === 'build').map((a) => [(a.action as { station: string }).station, a]),
  );
  return (
    <>
      {groups.map(([g, list]) => (
        <section key={g} class="section">
          <Rule>{groupName(g)}</Rule>
          <ActionList actions={list} onAct={onAct} />
        </section>
      ))}
      <Grounds g={view.grounds} actions={view.actions.filter((a) => a.group === 'food')} onAct={onAct} />
      <section class="section">
        <Rule>Stations</Rule>
        <ul class="stations">
          {view.stations.map((s) => (
            <Station key={s.id} s={s} build={builds.get(s.id)} onAct={onAct} />
          ))}
        </ul>
      </section>
    </>
  );
}
