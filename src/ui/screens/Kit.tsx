import { useState } from 'preact/hooks';
import type { ActionView, Entry } from '../../core';
import { ActionList } from '../components/ActionButton';
import { Button } from '../components/Button';
import { Hairline } from '../components/Hairline';
import { Sheet } from '../components/Sheet';
import { Stepper } from '../components/Stepper';
import { TabBar } from '../components/TabBar';
import { Toasts } from '../components/Toasts';
import { Journal } from './game/Journal';
import { Icon } from '../components/Icon';
import { LockHouseArt } from '../components/LockHouseArt';
import { Meter } from '../components/Meter';
import { Panel } from '../components/Panel';
import { Rule } from '../components/Rule';
import { ICON_NAMES } from '../icons/paths';
import { EndArt } from '../components/EndArt';
import { PLACE_ART_IDS, PlaceArt } from '../components/PlaceArt';
import { Distance } from './game/Expedition';
import { ChoiceCard, Choices, MoveRow } from './game/Parts';

/**
 * Temporary review sheet for the visual primitives. Open with `?kit`.
 * Remove or hide behind a dev flag once the real screens exist.
 */
const SWATCHES = ['field', 'panel', 'line', 'dim', 'faint', 'danger'] as const;

function act(p: Partial<ActionView> & { label: string }): ActionView {
  return {
    action: { type: 'rest' },
    group: 'house',
    icon: 'rest',
    minutes: 0,
    noise: 0,
    odds: null,
    enabled: true,
    reason: null,
    danger: false,
    max_qty: 1,
    ...p,
  };
}

const SAMPLE_ACTIONS: ActionView[] = [
  act({ label: 'Draw water from the lock', icon: 'water', minutes: 15, noise: 1 }),
  act({ label: 'Search carefully', icon: 'pack', minutes: 60, odds: 80 }),
  act({ label: 'Force the door', icon: 'lock', minutes: 5, noise: 12, danger: true }),
  act({ label: 'Pack ship\u2019s biscuit', icon: 'food', action: { type: 'pack', item: 'biscuits', qty: 1 }, max_qty: 7 }),
  act({ label: 'Build: iron stove', icon: 'stove', enabled: false, reason: 'Needs scrap metal 0/3 and machine parts 0/1.' }),
];

const SAMPLE_JOURNAL: Entry[] = [
  { day: 3, time: '06:00', text: 'Day 3. The sky is low and grey.', tone: 'quiet' },
  { day: 3, time: '06:17', text: 'You haul in the night line. A fish.', tone: 'find' },
  { day: 3, time: '06:32', text: 'You fill 2 measures from the lock. Brown water. Boil it.', tone: 'plain' },
  { day: 3, time: '06:32', text: 'Something moves on the roof.', tone: 'danger' },
];

export function Kit() {
  const [qty, setQty] = useState(2);
  const [tab, setTab] = useState<'house' | 'store' | 'craft' | 'map' | 'notes'>('house');
  const [sheet, setSheet] = useState(false);
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
        <Rule>Actions</Rule>
        <ActionList actions={SAMPLE_ACTIONS} onAct={() => undefined} />
      </section>

      <section class="kit__section">
        <Rule>Choices, side by side</Rule>
        <Choices label="How to go in">
          <ChoiceCard
            a={act({ label: 'Go in quietly', icon: 'house', minutes: 25 })}
            title="Quietly"
            lines={[{ text: 'Slow. No noise.' }]}
            onAct={() => undefined}
          />
          <ChoiceCard
            a={act({ label: 'Go in fast', icon: 'house', minutes: 5, noise: 4, danger: true })}
            title="Fast"
            lines={[{ text: 'After dark', danger: true }]}
            onAct={() => undefined}
          />
        </Choices>
        <Choices label="The locked door">
          <ChoiceCard a={act({ label: 'Pick the lock', icon: 'lock', minutes: 20, noise: 1, odds: 64 })} onAct={() => undefined} />
          <ChoiceCard a={act({ label: 'Use the key', icon: 'key', enabled: false, reason: 'Needs the station keys.' })} onAct={() => undefined} />
          <ChoiceCard a={act({ label: 'Force the door', icon: 'lock', minutes: 5, noise: 18, danger: true })} onAct={() => undefined} />
        </Choices>
      </section>

      <section class="kit__section">
        <Rule>Item moves</Rule>
        <ul class="moves">
          <MoveRow
            item={{ id: 'biscuits', name: 'Ship’s biscuit', icon: 'food', qty: 7, detail: 'Satiety +14' }}
            a={act({ label: 'Pack', action: { type: 'pack', item: 'biscuits', qty: 1 }, max_qty: 7 })}
            verb="Pack"
            aside="0.2 kg"
            onAct={() => undefined}
          />
          <MoveRow
            item={{ id: 'plank', name: 'Plank', icon: 'plank', qty: 3 }}
            a={act({ label: 'Pack', enabled: false, reason: 'It will not fit.' })}
            verb="Pack"
            aside="2.0 kg"
            onAct={() => undefined}
          />
        </ul>
      </section>

      <section class="kit__section">
        <Rule>Distance</Rule>
        <Distance band="far" name="A stander" />
        <Distance band="near" name="A listener" />
        <Distance band="close" name="A runner" />
      </section>

      <section class="kit__section">
        <Rule>Stepper and hairlines</Rule>
        <Stepper value={qty} min={1} max={7} onChange={setQty} label="How many" />
        <Hairline value={2} max={3} steps label="Station level" />
        <Hairline value={61} max={100} label="Rest" />
        <Hairline value={9} max={100} label="Warmth" danger />
      </section>

      <section class="kit__section">
        <Rule>Journal</Rule>
        <div class="kit__journal">
          <Journal entries={SAMPLE_JOURNAL} fresh={2} seq={1} />
        </div>
      </section>

      <section class="kit__section">
        <Rule>Toast</Rule>
        <div class="kit__toast">
          <Toasts
            toasts={[{ id: 1, title: 'Mark. First night', text: 'Survive a night at the lock house.', icon: 'sigil_moon', tone: 'quiet' }]}
            onDismiss={() => undefined}
          />
        </div>
      </section>

      <section class="kit__section">
        <Rule>Sheet</Rule>
        <Button block variant="quiet" onClick={() => setSheet(true)}>
          Open a sheet
        </Button>
        {sheet && (
          <Sheet title="Day 3, 06:32" onClose={() => setSheet(false)}>
            <p class="prose">The lock house.</p>
          </Sheet>
        )}
      </section>

      <section class="kit__section">
        <Rule>Tab bar</Rule>
        <TabBar
          idBase="kit"
          label="Kit tabs"
          current={tab}
          onChange={setTab}
          tabs={[
            { id: 'house', label: 'House', icon: 'house', flag: true },
            { id: 'store', label: 'Store', icon: 'store' },
            { id: 'craft', label: 'Craft', icon: 'workbench' },
            { id: 'map', label: 'Map', icon: 'map' },
            { id: 'notes', label: 'Notes', icon: 'journal' },
          ]}
        />
      </section>

      <section class="kit__section">
        <Rule>Rule</Rule>
        <Rule />
      </section>

      <section class="kit__section">
        <Rule>Illustration</Rule>
        <LockHouseArt />
      </section>

      <section class="kit__section">
        <Rule>Places</Rule>
        {[...PLACE_ART_IDS, 'unknown'].map((id) => (
          <figure key={id} class="kit__place">
            <PlaceArt place={id} />
            <figcaption class="label prose--dim">{id}</figcaption>
          </figure>
        ))}
      </section>

      <section class="kit__section">
        <Rule>Endings</Rule>
        <EndArt kind="departure" />
        <EndArt kind="freeze_out" />
      </section>

      <p class="kit__foot">
        <a href="./">Back to title</a>
      </p>
    </main>
  );
}
