import { useState } from 'preact/hooks';
import type { Action, RecipeView, View } from '../../../core';
import { ActionButton } from '../../components/ActionButton';
import { Icon } from '../../components/Icon';
import { Rule } from '../../components/Rule';
import { duration } from '../../format';
import { Costs } from './HouseTab';

function Recipe({ r, view, onAct }: { r: RecipeView; view: View; onAct: (a: Action) => void }) {
  const act = view.actions.find((a) => a.action.type === 'craft' && a.action.recipe === r.id);
  const station = r.station ? view.stations.find((s) => s.id === r.station) : undefined;
  return (
    <li class={r.can_craft ? 'recipe' : 'recipe recipe--off'}>
      <div class="recipe__head">
        <Icon name={r.icon} size={20} class="recipe__icon" />
        <p class="recipe__name">{r.name}</p>
        <span class="recipe__time num">{duration(r.minutes)}</span>
      </div>
      <div class="recipe__flow">
        <Costs costs={r.inputs} />
        <Icon name="forward" size={16} class="recipe__arrow" />
        <ul class="costs">
          {r.outputs.map((o) => (
            <li key={o.item} class="cost">
              <Icon name={o.icon} size={16} />
              <span class="cost__name">{o.name}</span>
              <span class="cost__qty num">x{o.need}</span>
            </li>
          ))}
        </ul>
      </div>
      {(station || r.fire) && (
        <p class="recipe__req label">
          {station ? `${station.name} ${r.station_level}` : ''}
          {station && r.fire ? ' · ' : ''}
          {r.fire ? 'Needs a fire' : ''}
        </p>
      )}
      {act && <ActionButton a={act} onAct={onAct} plain />}
    </li>
  );
}

export function CraftTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const [all, setAll] = useState(false);
  const ready = view.recipes.filter((r) => r.can_craft);
  const rest = view.recipes.filter((r) => !r.can_craft);
  return (
    <>
      <section class="section">
        <Rule>Can make now</Rule>
        {ready.length === 0 ? (
          <p class="prose prose--dim">Nothing, with what is here.</p>
        ) : (
          <ul class="recipes">
            {ready.map((r) => (
              <Recipe key={r.id} r={r} view={view} onAct={onAct} />
            ))}
          </ul>
        )}
      </section>
      {rest.length > 0 && (
        <section class="section">
          <button type="button" class="disclosure" aria-expanded={all} onClick={() => setAll(!all)}>
            <span class="rule__text">Not yet</span>
            <span class="num">{rest.length}</span>
            <Icon name="down" size={16} class={all ? 'disclosure__icon disclosure__icon--open' : 'disclosure__icon'} />
          </button>
          {all && (
            <ul class="recipes">
              {rest.map((r) => (
                <Recipe key={r.id} r={r} view={view} onAct={onAct} />
              ))}
            </ul>
          )}
        </section>
      )}
    </>
  );
}
