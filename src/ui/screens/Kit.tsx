import { Button } from '../components/Button';
import { Icon } from '../components/Icon';
import { LockHouseArt } from '../components/LockHouseArt';
import { Meter } from '../components/Meter';
import { Panel } from '../components/Panel';
import { Rule } from '../components/Rule';
import { ICON_NAMES } from '../icons/paths';

/**
 * Temporary review sheet for the visual primitives. Open with `?kit`.
 * Remove or hide behind a dev flag once the real screens exist.
 */
const SWATCHES = ['field', 'panel', 'line', 'dim', 'faint', 'danger'] as const;

export function Kit() {
  return (
    <main class="frame kit">
      <header class="kit__head">
        <h1 class="wordmark wordmark--small">ALL ALONE</h1>
        <p class="label prose--dim">Kit. Primitives for review.</p>
      </header>

      <section class="kit__section">
        <Rule>Palette</Rule>
        <ul class="kit__swatches">
          {SWATCHES.map((s) => (
            <li key={s} class="kit__swatch">
              <span class={`swatch swatch--${s}`} />
              <span class="label">{s}</span>
            </li>
          ))}
        </ul>
      </section>

      <section class="kit__section">
        <Rule>Icons</Rule>
        <ul class="kit__icons">
          {ICON_NAMES.map((name) => (
            <li key={name} class="kit__icon">
              <Icon name={name} size={32} />
              <span class="label prose--dim">{name}</span>
            </li>
          ))}
        </ul>
        <ul class="kit__icon-sizes">
          {([16, 20, 24, 32, 48] as const).map((size) => (
            <li key={size}>
              <Icon name="house" size={size} label={`House, ${size} pixels`} />
            </li>
          ))}
        </ul>
      </section>

      <section class="kit__section">
        <Rule>Type</Rule>
        <p class="prose">
          You force the back door of the chandlery. The sound carries down the empty street. Inside, the
          shelves are mostly bare.
        </p>
        <p class="prose prose--dim">The radio gives you nothing but the hiss tonight.</p>
        <p class="num">DAY 03  07:40  NOISE 12</p>
      </section>

      <section class="kit__section">
        <Rule>Buttons</Rule>
        <div class="kit__stack">
          <Button block icon="map">Set out</Button>
          <Button block variant="quiet" icon="rest">Sleep until dawn</Button>
          <Button block variant="danger" icon="noise">Force the lock</Button>
          <Button block disabled icon="lock">Locked</Button>
        </div>
      </section>

      <section class="kit__section">
        <Panel title="Vitals">
          <Meter label="Health" icon="health" value={82} />
          <Meter label="Satiety" icon="food" value={54} />
          <Meter label="Hydration" icon="water" value={37} />
          <Meter label="Rest" icon="rest" value={61} />
          <Meter label="Nerve" icon="nerve" value={18} />
          <Meter label="Warmth" icon="warmth" value={0} />
        </Panel>
      </section>

      <section class="kit__section">
        <Rule>Rule</Rule>
        <Rule />
      </section>

      <section class="kit__section">
        <Rule>Illustration</Rule>
        <LockHouseArt />
      </section>

      <p class="kit__foot">
        <a href="./">Back to title</a>
      </p>
    </main>
  );
}
