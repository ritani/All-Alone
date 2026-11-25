/**
 * Phase panels: the choice event, the night, the dawn page, and a generic
 * fallback that renders any phase plainly (M4 designs packing, travel, site,
 * encounter and barter properly).
 */
import type { ComponentChildren } from 'preact';
import type { Action, ActionView, View } from '../../../core';
import { ActionList } from '../../components/ActionButton';
import { Button } from '../../components/Button';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { ListRow } from '../../components/ListRow';
import { useFocusHeading } from '../../components/Page';
import { Rule } from '../../components/Rule';
import { duration, groupActions, groupName } from '../../format';
import { Provisions } from './HouseTab';
import { ItemList } from './StoreTab';

interface PhaseProps {
  view: View;
  onAct: (a: Action) => void;
}

function PanelHead({ kicker, title, icon }: { kicker?: string; title: string; icon?: string }) {
  const h = useFocusHeading<HTMLHeadingElement>(title);
  return (
    <header class="phase__head">
      {kicker && <p class="label phase__kicker">{kicker}</p>}
      <h2 class="phase__title" tabIndex={-1} ref={h}>
        {icon && <Icon name={icon} size={24} />}
        <span>{title}</span>
      </h2>
    </header>
  );
}

function Groups({ actions, onAct, order }: { actions: ActionView[]; onAct: (a: Action) => void; order?: string[] }) {
  return (
    <>
      {groupActions(actions, order).map(([g, list]) => (
        <section key={g} class="section">
          <Rule>{groupName(g)}</Rule>
          <ActionList actions={list} onAct={onAct} />
        </section>
      ))}
    </>
  );
}

// ------------------------------------------------------------------ event

export function EventPanel({ view, onAct }: PhaseProps) {
  const ev = view.event;
  const choices = view.actions.filter((a) => a.group === 'choice');
  const rest = view.actions.filter((a) => a.group !== 'choice' && a.group !== 'survive');
  return (
    <div class="phase phase--event">
      <PanelHead kicker="Something happens" title={ev?.title ?? 'A choice'} />
      {ev && (
        <div class="event__text prose">
          {ev.text.split('\n').map((p, i) => (
            <p key={i}>{p}</p>
          ))}
        </div>
      )}
      <ActionList actions={choices} onAct={onAct} plain />
      {rest.length > 0 && <Groups actions={rest} onAct={onAct} />}
      <Provisions view={view} onAct={onAct} />
    </div>
  );
}

// ------------------------------------------------------------------ night

export function NightPanel({ view, onAct }: PhaseProps) {
  const night = view.actions.filter((a) => a.group === 'night');
  const rest = view.actions.filter((a) => a.group !== 'night' && a.group !== 'survive' && a.enabled);
  const g = view.grounds;
  return (
    <div class="phase phase--night">
      <PanelHead kicker={`Night ${view.run.nights + 1}`} title="The door is barred" />
      <ul class="lrows">
        <li>
          <ListRow
            icon="barricade"
            title="Barricade"
            aside={
              <span class="num">
                {g.barricade}/{g.barricade_max}
              </span>
            }
            danger={g.barricade_max > 0 && g.barricade / g.barricade_max < 0.25}
          />
        </li>
        <li>
          <ListRow icon="stove" title="Fire" aside={g.fire_until ? `Until ${g.fire_until}` : 'Out'} />
        </li>
      </ul>
      <ActionList actions={night} onAct={onAct} />
      {rest.length > 0 && <Groups actions={rest} onAct={onAct} />}
      <Provisions view={view} onAct={onAct} />
    </div>
  );
}

// ------------------------------------------------------------------ dawn

export function DawnPage({ view, onAct }: PhaseProps) {
  const d = view.dawn;
  const rise = view.actions.find((a) => a.action.type === 'rise');
  const others = view.actions.filter((a) => a.action.type !== 'rise' && a.group !== 'survive' && a.enabled);
  const h = useFocusHeading<HTMLHeadingElement>(d?.night ?? 0);
  if (!d) return <FallbackPanel view={view} onAct={onAct} />;
  const [first, ...restOfHeadline] = d.headline.split('. ');
  const harmed = d.breached || d.visitors > 0;
  return (
    <div class={harmed ? 'dawn dawn--harmed' : 'dawn'}>
      <p class="label dawn__kicker">Day {d.day}</p>
      <h2 class="dawn__title" tabIndex={-1} ref={h}>
        {first}.
      </h2>
      <p class={harmed ? 'dawn__headline dawn__headline--danger' : 'dawn__headline'}>{restOfHeadline.join('. ')}</p>
      <div class="dawn__lines prose">
        {d.lines.map((l, i) => (
          <p key={i}>{l}</p>
        ))}
      </div>
      <dl class="table dawn__table">
        <div class="table__row">
          <dt>Noise yesterday</dt>
          <dd class="num">{d.noise}</dd>
        </div>
        <div class="table__row">
          <dt>Came to the house</dt>
          <dd class="num">{d.visitors}</dd>
        </div>
        <div class="table__row">
          <dt>Today</dt>
          <dd>
            <Icon name={view.weather.today.icon} size={16} /> {view.weather.today.name}
          </dd>
        </div>
        {view.weather.forecast && (
          <div class="table__row">
            <dt>The radio says, tomorrow</dt>
            <dd>
              <Icon name={view.weather.forecast.icon} size={16} /> {view.weather.forecast.name}
            </dd>
          </div>
        )}
      </dl>
      {rise && (
        <div class="phase__go dawn__go">
          <Button block onClick={() => onAct(rise.action)} disabled={!rise.enabled}>
            {rise.label}
          </Button>
        </div>
      )}
      {others.length > 0 && <Groups actions={others} onAct={onAct} />}
      <Provisions view={view} onAct={onAct} />
    </div>
  );
}

// ------------------------------------------------------------------ fallback

function Facts({ rows }: { rows: [string, ComponentChildren][] }) {
  return (
    <dl class="table">
      {rows.map(([k, v]) => (
        <div key={k} class="table__row">
          <dt>{k}</dt>
          <dd>{v}</dd>
        </div>
      ))}
    </dl>
  );
}

const PHASE_TITLES: Record<string, string> = {
  packing: 'Packing',
  travelling: 'On the road',
  site: 'On site',
  encounter: 'They are here',
  barter: 'The widow',
  event: 'A choice',
  night: 'Night',
  dawn: 'Dawn',
  home: 'The lock house',
  ended: 'The end',
};

/** Renders whatever the phase's view section offers, plainly, then every action grouped. */
export function FallbackPanel({ view, onAct }: PhaseProps) {
  const body: ComponentChildren[] = [];
  let title = PHASE_TITLES[view.phase] ?? view.phase;
  let kicker: string | undefined;

  if (view.packing) {
    const p = view.packing;
    title = `To ${p.name}`;
    kicker = 'Packing';
    body.push(
      <Facts
        key="facts"
        rows={[
          ['Walk', duration(p.minutes)],
          ['Light', p.has_light ? 'Yes' : 'None'],
          ['Weapon', p.has_weapon ? 'Yes' : 'None'],
          ['Meals packed', p.meals],
          ['Water packed', p.water],
          ['The dog', p.take_dog ? 'Coming' : 'Staying'],
          ['Carrying', view.inventory.carry_label],
        ]}
      />,
    );
    if (view.inventory.pack.length > 0) {
      body.push(
        <section key="pack" class="section">
          <Rule>In the pack</Rule>
          <ItemList items={view.inventory.pack} />
        </section>,
      );
    }
  }

  if (view.travel) {
    const t = view.travel;
    title = t.homeward ? 'Going home' : `To ${t.name}`;
    kicker = 'On the road';
    body.push(
      <Facts
        key="travel"
        rows={[
          ['Walking', duration(t.minutes)],
          ['Hurrying', duration(t.hurry_minutes)],
          ['Arrive', t.arrive_dark ? <span class="danger-text">After dark</span> : 'In the light'],
        ]}
      />,
    );
  }

  if (view.site) {
    const s = view.site;
    title = s.name;
    kicker = 'On site';
    body.push(
      <p key="desc" class="prose">
        {s.desc}
      </p>,
      <section key="rooms" class="section">
        <Rule>Rooms</Rule>
        <ol class="rooms">
          {s.rooms.map((r) => (
            <li key={r.index} class={r.here ? 'room room--here' : r.entered ? 'room' : 'room room--dim'}>
              <span class="room__name">{r.name}</span>
              <span class="room__flags">
                {[
                  r.here ? 'here' : '',
                  r.dark ? 'dark' : '',
                  r.locked ? 'locked' : '',
                  r.flooded ? 'flooded' : '',
                  r.search === 'full' ? 'searched' : r.search === 'partial' ? 'part searched' : '',
                ]
                  .filter(Boolean)
                  .join(' · ')}
              </span>
            </li>
          ))}
        </ol>
        <p class="meta">Noise here {s.site_noise}{s.widow_here ? ' · the widow is here' : ''}</p>
      </section>,
    );
    if (s.room) {
      const r = s.room;
      body.push(
        <section key="room" class="section">
          <Rule>{r.name}</Rule>
          <p class="prose">{r.desc}</p>
          <p class="meta">
            {[r.dark ? (r.lit ? 'lit' : 'dark') : '', r.flooded ? 'flooded' : '', `search: ${r.search}`]
              .filter(Boolean)
              .join(' · ')}
          </p>
          {r.dormant.length > 0 && (
            <ul class="foes">
              {r.dormant.map((f, i) => (
                <li key={i} class="foe foe--still">
                  <Icon name={f.icon} size={20} />
                  <span>{f.name}, standing still</span>
                </li>
              ))}
            </ul>
          )}
          {r.pile.length > 0 && (
            <>
              <p class="meta">Here</p>
              <ItemList items={r.pile} />
            </>
          )}
        </section>,
      );
    }
  }

  if (view.encounter) {
    // The fight comes first, above whatever the site says.
    const before = body.length;
    const e = view.encounter;
    title = e.at_home ? 'At the door' : 'They are here';
    kicker = `Turn ${e.turn}${e.dark ? ' · dark' : ''}${e.gripped ? ' · gripped' : ''}`;
    body.push(
      <ul key="foes" class="foes">
        {e.foes.map((f, i) => (
          <li key={i} class={f.band === 'close' ? 'foe foe--close' : 'foe'}>
            <Icon name={f.icon} size={20} />
            <span class="foe__name">{f.name}</span>
            <span class="foe__band">{f.band}</span>
            <Hairline value={f.health} max={100} label={`${f.name} health`} />
          </li>
        ))}
      </ul>,
    );
    if (e.log.length > 0) {
      body.push(
        <div key="log" class="prose encounter__log">
          {e.log.map((l, i) => (
            <p key={i}>{l}</p>
          ))}
        </div>,
      );
    }
    body.unshift(...body.splice(before));
  }

  if (view.barter) {
    const b = view.barter;
    title = 'The widow';
    kicker = `Credit ${b.credit} · trades ${b.trades}`;
    body.push(
      <section key="offers" class="section">
        <Rule>She offers</Rule>
        <ul class="rows">
          {b.offers.map((o) => (
            <li key={o.item} class="figure-line">
              <span>
                <Icon name={o.icon} size={16} /> {o.name} x{o.qty}
              </span>
              <span class="num">{o.price}</span>
            </li>
          ))}
        </ul>
      </section>,
      <section key="sell" class="section">
        <Rule>She would take</Rule>
        <ul class="rows">
          {b.sell.map((o) => (
            <li key={o.item} class="figure-line">
              <span>
                <Icon name={o.icon} size={16} /> {o.name} x{o.qty}
              </span>
              <span class="num">{o.price}</span>
            </li>
          ))}
        </ul>
      </section>,
    );
  }

  if (view.event) {
    title = view.event.title;
    body.push(
      <p key="ev" class="prose">
        {view.event.text}
      </p>,
    );
  }

  return (
    <div class={`phase phase--${view.phase}`}>
      <PanelHead kicker={kicker ?? view.location} title={title} />
      {body}
      <Groups actions={view.actions.filter((a) => a.enabled || a.group !== 'survive')} onAct={onAct} />
    </div>
  );
}
