import { useState } from 'preact/hooks';
import type { Action, RecipeView, View } from '../../../core';
import { ActionButton, Chips } from '../../components/ActionButton';
import { Icon } from '../../components/Icon';
import { Fold, ListRow } from '../../components/ListRow';
import { Rule } from '../../components/Rule';
import { Sheet } from '../../components/Sheet';
import { Costs } from './HouseTab';

/** The first thing that stands in the way, in a few words. */
function missing(r: RecipeView, reason: string | null | undefined): string {
  if (reason) return reason;
  const short = r.inputs.filter((c) => c.have < c.need).map((c) => `${c.name} ${c.have}/${c.need}`);
  return short.length > 0 ? `Needs ${short.join(', ')}.` : '';
}

function RecipeSheet({ r, view, onAct, onClose }: { r: RecipeView; view: View; onAct: (a: Action) => void; onClose: () => void }) {
  const act = view.actions.find((a) => a.action.type === 'craft' && a.action.recipe === r.id);
  const station = r.station ? view.stations.find((s) => s.id === r.station) : undefined;
  return (
    <Sheet title={r.name} onClose={onClose}>
      <Rule level={3}>Takes</Rule>
      <Costs costs={r.inputs} />
      <Rule level={3}>Makes</Rule>
      <ul class="costs">
        {r.outputs.map((o) => (
          <li key={o.item} class="cost">
            <Icon name={o.icon} size={16} />
            <span class="cost__name">{o.name}</span>
            <span class="cost__qty num">x{o.need}</span>
          </li>
        ))}
      </ul>
      {(station || r.fire) && (
        <p class="station__req">
          {station && (
            <>
              <Icon name={station.icon} size={16} /> {station.name} {r.station_level}
            </>
          )}
          {r.fire && (
            <>
              <Icon name="stove" size={16} /> A fire
            </>
          )}
        </p>
      )}
      {act ? <ActionButton a={act} onAct={onAct} /> : <Chips minutes={r.minutes} noise={0} odds={null} />}
    </Sheet>
  );
}

let notYetOpen = false;

export function CraftTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const [all, setAll] = useState(notYetOpen);
  const [open, setOpen] = useState<string | null>(null);
  const ready = view.recipes.filter((r) => r.can_craft);
  const rest = view.recipes.filter((r) => !r.can_craft);
  const act = (r: RecipeView) => view.actions.find((a) => a.action.type === 'craft' && a.action.recipe === r.id);
  const shown = open ? view.recipes.find((r) => r.id === open) : undefined;
  return (
    <>
      <section class="section">
        <Rule>Can make now</Rule>
        {ready.length === 0 ? (
          <p class="prose prose--dim">Nothing, with what is here.</p>
        ) : (
          <ul class="lrows">
            {ready.map((r) => {
              const a = act(r);
              return (
                <li key={r.id}>
                  <ListRow
                    icon={r.icon}
                    title={r.name}
                    sub={<Chips minutes={a?.minutes ?? r.minutes} noise={a?.noise ?? 0} odds={null} />}
                    onClick={() => setOpen(r.id)}
                  />
                </li>
              );
            })}
          </ul>
        )}
      </section>
      {rest.length > 0 && (
        <section class="section">
          <Fold
            title="Not yet"
            count={<span class="num">{rest.length}</span>}
            open={all}
            onToggle={() => {
              notYetOpen = !all;
              setAll(!all);
            }}
          >
            <ul class="lrows">
              {rest.map((r) => (
                <li key={r.id}>
                  <ListRow icon={r.icon} title={r.name} sub={missing(r, act(r)?.reason ?? r.reason)} dim onClick={() => setOpen(r.id)} />
                </li>
              ))}
            </ul>
          </Fold>
        </section>
      )}
      {shown && <RecipeSheet r={shown} view={view} onAct={onAct} onClose={() => setOpen(null)} />}
    </>
  );
}
