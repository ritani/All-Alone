/**
 * The expedition screens: packing, the road, a place and its rooms, an
 * encounter with the Stilled, and the widow's barter. Each is drawn from the
 * phase's view section and `view.actions`; every button dispatches an action
 * the core listed.
 */
import { useState } from 'preact/hooks';
import type { ActionView, Band, FoeView, ItemView, RoomSummary, View } from '../../../core';
import { ActionButton, ActionList } from '../../components/ActionButton';
import { Button } from '../../components/Button';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Meter } from '../../components/Meter';
import { PlaceArt } from '../../components/PlaceArt';
import { Rule } from '../../components/Rule';
import { duration, titleCase } from '../../format';
import { MapDrawing } from './MapTab';
import { ChoiceCard, Choices, MoveRow, PhaseHead, itemAct, kg, pick, type OnAct } from './Parts';
import { KIND_NAMES, KIND_ORDER } from './StoreTab';

interface PhaseProps {
  view: View;
  onAct: OnAct;
}

/** Carried against the limit, as a hairline. */
function Load({ view }: { view: View }) {
  const inv = view.inventory;
  const full = inv.capacity > 0 && inv.carried / inv.capacity >= 0.95;
  return (
    <div class="load">
      <div class="figure-line">
        <span class="label">
          <Icon name="pack" size={16} /> Carrying
        </span>
        <span class="num">{inv.carry_label}</span>
      </div>
      <Hairline value={inv.carried} max={inv.capacity} label={`Carrying ${inv.carry_label}`} danger={full} />
    </div>
  );
}

/** Eating, drinking and medicine while out: folded away until wanted. */
function Provisions({ actions, onAct, open: startOpen = false }: { actions: ActionView[]; onAct: OnAct; open?: boolean }) {
  const [open, setOpen] = useState(startOpen);
  const list = actions.filter((a) => a.group === 'survive' && a.enabled);
  if (list.length === 0) return null;
  return (
    <section class="section">
      <button type="button" class="disclosure" aria-expanded={open} onClick={() => setOpen(!open)}>
        <Icon name="food" size={20} />
        <span class="disclosure__name">Eat, drink, mend</span>
        <span class="num">{list.length}</span>
        <Icon name="down" size={16} class={open ? 'disclosure__icon disclosure__icon--open' : 'disclosure__icon'} />
      </button>
      {open && <ActionList actions={list} onAct={onAct} />}
    </section>
  );
}

function byKind(items: ItemView[]): [string, ItemView[]][] {
  const kinds = [...new Set(items.map((i) => i.kind))].sort(
    (a, b) => (KIND_ORDER.indexOf(a) + 1 || 99) - (KIND_ORDER.indexOf(b) + 1 || 99),
  );
  return kinds.map((k) => [k, items.filter((i) => i.kind === k)]);
}

// ------------------------------------------------------------------ packing

const FOLDED_KINDS = new Set(['material', 'seed']);

export function PackingPanel({ view, onAct }: PhaseProps) {
  const p = view.packing;
  const inv = view.inventory;
  const acts = view.actions;
  const depart = pick(acts, 'depart');
  const cancel = pick(acts, 'cancel_trip');
  const dog = pick(acts, 'take_dog');
  const [showAll, setShowAll] = useState(false);
  if (!p) return null;
  const meta = [
    `${duration(p.minutes)} walk`,
    p.unvisited ? 'not been there' : `${p.rooms_searched}/${p.rooms} rooms searched`,
  ].join(' · ');
  const groups = byKind(inv.store);
  // Building stock and seed are rarely carried out: they fold away.
  const shown = showAll ? groups : groups.filter(([k]) => !FOLDED_KINDS.has(k));
  return (
    <div class="phase phase--packing">
      <PhaseHead kicker="Packing" title={`To ${p.name}`} focusKey={`pack-${p.place}`}>
        <p class="label phase__meta">{meta}</p>
      </PhaseHead>

      <Load view={view} />

      <section class="section" aria-label="What to bring">
        <ul class="needs">
          <Need met={p.has_light} icon="light" text={p.has_light ? 'A light, packed.' : 'No light packed.'} />
          <Need met={p.has_weapon} icon="blade" text={p.has_weapon ? 'Something to strike with.' : 'Nothing to strike with.'} />
          <Need met={p.meals > 0} icon="food" text={p.meals > 0 ? `${p.meals} ${p.meals === 1 ? 'meal' : 'meals'} packed.` : 'No food packed.'} />
          <Need met={p.water > 0} icon="water" text={p.water > 0 ? `${p.water} clean water packed.` : 'No clean water packed.'} />
        </ul>
        {p.needs.length > 0 && (
          <>
            <p class="label needs__head">What you know it calls for</p>
            <ul class="needs">
              {p.needs.map((n) => (
                <Need key={n.id} met={n.met} icon={n.icon} text={n.text} />
              ))}
            </ul>
          </>
        )}
        {p.unvisited && <p class="prose prose--dim">You do not know what is inside.</p>}
      </section>

      {dog && (
        <section class="section">
          <div class="figure-line">
            <span class="label">
              <Icon name="sigil_dog" size={16} /> The dog
            </span>
            <span>{p.take_dog ? 'Coming' : 'Staying'}</span>
          </div>
          <ActionButton a={dog} onAct={onAct} />
        </section>
      )}

      <section class="section">
        <Rule>In the pack</Rule>
        {inv.pack.length === 0 ? (
          <p class="prose prose--dim">Empty.</p>
        ) : (
          <ul class="moves">
            {inv.pack.map((it) => (
              <MoveRow key={it.id} item={it} a={itemAct(acts, 'unpack', it.id)} verb="Leave" onAct={onAct} aside={it.weight_label} />
            ))}
          </ul>
        )}
      </section>

      <section class="section">
        <Rule>In the store</Rule>
        {inv.store.length === 0 && <p class="prose prose--dim">Nothing left to take.</p>}
        {shown.map(([k, items]) => (
          <div key={k} class="moves__group">
            <p class="label moves__kind">{KIND_NAMES[k] ?? k}</p>
            <ul class="moves">
              {items.map((it) => (
                <MoveRow key={it.id} item={it} a={itemAct(acts, 'pack', it.id)} verb="Pack" onAct={onAct} aside={it.qty > 1 ? `${kg(it.weight)} each` : kg(it.weight)} />
              ))}
            </ul>
          </div>
        ))}
        {groups.length > shown.length && (
          <button type="button" class="link link--small" onClick={() => setShowAll(true)}>
            Show materials and seed too
          </button>
        )}
      </section>

      <div class="phase__go phase__go--row">
        {cancel && (
          <Button variant="quiet" onClick={() => onAct(cancel.action)} disabled={!cancel.enabled}>
            {cancel.label}
          </Button>
        )}
        {depart && (
          <Button block variant={depart.danger ? 'danger' : 'primary'} disabled={!depart.enabled} onClick={() => onAct(depart.action)}>
            {view.dark ? 'Set out in the dark' : 'Set out'}
          </Button>
        )}
      </div>
    </div>
  );
}

function Need({ met, icon, text }: { met: boolean; icon: string; text: string }) {
  return (
    <li class={met ? 'need need--met' : 'need'}>
      <Icon name={icon} size={16} />
      <span class="need__text">{text}</span>
      <Icon name={met ? 'check' : 'close'} size={16} label={met ? 'Have it' : 'Missing'} />
    </li>
  );
}

// ------------------------------------------------------------------ the road

export function TravelPanel({ view, onAct }: PhaseProps) {
  const t = view.travel;
  const walk = pick(view.actions, 'walk', (a) => !a.hurry);
  const hurry = pick(view.actions, 'walk', (a) => a.hurry);
  const back = pick(view.actions, 'turn_back');
  if (!t) return null;
  const light = view.dark || view.daylight_left === null ? 'Dark now' : `${duration(view.daylight_left)} of light left`;
  const arrive = (at: string, dark: boolean) => [
    { text: `${t.homeward ? 'Home' : 'There'} at ${at}` },
    { text: dark ? 'After dark' : 'In the light', danger: dark },
  ];
  return (
    <div class="phase phase--travelling">
      <PhaseHead kicker="On the road" title={t.homeward ? `Home from ${t.name}` : `To ${t.name}`} focusKey={`road-${t.place}-${t.homeward}`}>
        <p class="label phase__meta">
          {view.clock} · {light}
        </p>
      </PhaseHead>
      <figure class="road">
        <MapDrawing
          nodes={view.map.nodes}
          paths={view.map.paths}
          selected={t.place}
          onSelect={() => undefined}
          route={t.homeward ? [t.place, 'home'] : ['home', t.place]}
        />
        <figcaption class="visually-hidden">
          The way {t.homeward ? `home from ${t.name}` : `from the lock house to ${t.name}`}.
        </figcaption>
      </figure>
      <Choices label="How to go">
        <ChoiceCard a={walk} onAct={onAct} title="Walk" lines={arrive(t.arrive_at, t.arrive_dark)} />
        <ChoiceCard a={hurry} onAct={onAct} title="Hurry" lines={arrive(t.hurry_arrive_at, t.hurry_arrive_dark)} />
      </Choices>
      <p class="prose prose--dim road__note">
        Hurrying is quicker and costs you, and it carries. Anything on the road happens on the way.
      </p>
      {back && <ActionList actions={[back]} onAct={onAct} />}
      <Provisions actions={view.actions} onAct={onAct} />
    </div>
  );
}

// ------------------------------------------------------------------ a place

const SEARCH_NAME = { none: 'Not searched', partial: 'Partly searched', full: 'Searched through' } as const;

function RouteStop({ r }: { r: RoomSummary }) {
  const unseen = !r.entered && r.name === 'Further in';
  const cls = ['stop', r.here ? 'stop--here' : '', r.entered ? 'stop--entered' : '', unseen ? 'stop--unseen' : '']
    .filter(Boolean)
    .join(' ');
  const flags: { icon: string; text: string; danger?: boolean }[] = [];
  if (r.locked) flags.push({ icon: 'lock', text: 'Locked' });
  if (r.dark) flags.push({ icon: 'light', text: 'Dark' });
  if (r.flooded) flags.push({ icon: 'water', text: 'Flooded' });
  if (r.search === 'full') flags.push({ icon: 'check', text: 'Searched' });
  else if (r.search === 'partial') flags.push({ icon: 'pack', text: 'Part searched' });
  return (
    <li class={cls} aria-current={r.here ? 'location' : undefined}>
      <span class="stop__mark" aria-hidden="true" />
      <span class="stop__name">{r.name}</span>
      <span class="stop__flags">
        {flags.map((f) => (
          <span key={f.text} class="stop__flag" title={f.text}>
            <Icon name={f.icon} size={16} label={f.text} />
          </span>
        ))}
      </span>
    </li>
  );
}

export function SitePanel({ view, onAct }: PhaseProps) {
  const s = view.site;
  const acts = view.actions;
  const [dropOpen, setDropOpen] = useState(false);
  if (!s) return null;
  const room = s.room;
  const quiet = pick(acts, 'enter', (a) => a.quiet);
  const loud = pick(acts, 'enter', (a) => !a.quiet);
  const careful = pick(acts, 'search', (a) => a.careful);
  const fast = pick(acts, 'search', (a) => !a.careful);
  const locks = acts.filter((a) => a.group === 'lock');
  const light = pick(acts, 'light_up');
  const takeAll = pick(acts, 'take_all');
  const ambush = pick(acts, 'ambush');
  const trade = pick(acts, 'trade');
  const moves = acts.filter((a) => a.action.type === 'back' || a.action.type === 'leave');
  const next = s.rooms.find((r) => r.index === (room ? room.index + 1 : 0));
  const outside = !room;
  // The light is for this room if it is dark and unlit, else for the dark ahead.
  const lightHere = light && room && room.dark && !room.lit;
  const lightAhead = light && !lightHere;
  return (
    <div class="phase phase--site">
      <figure class={room ? 'site__art site__art--small' : 'site__art'}>
        <PlaceArt place={s.place} />
      </figure>
      <PhaseHead
        kicker={outside ? 'Outside' : s.name}
        title={room ? room.name : s.name}
        focusKey={`site-${s.place}-${room?.index ?? -1}`}
      >
        <p class="prose site__line">{room ? room.desc : s.desc}</p>
      </PhaseHead>

      {room && (
        <p class="label site__state">
          {[
            room.dark ? (room.lit ? 'Lit' : 'Dark') : '',
            room.flooded ? 'Flooded' : '',
            SEARCH_NAME[room.search],
            `Noise here ${s.site_noise}`,
          ]
            .filter(Boolean)
            .join(' · ')}
        </p>
      )}
      {outside && (
        <p class="label site__state">
          {`Noise here ${s.site_noise}`}
          {s.widow_here ? ' · The widow is on the steps' : ''}
        </p>
      )}

      {room && room.dormant.length > 0 && (
        <section class="section">
          <Rule>Standing here</Rule>
          <ul class="dormant">
            {room.dormant.map((f, i) => (
              <li key={i} class="dormant__one">
                <Icon name={f.icon} size={20} />
                <span>{titleCase(f.name)}, standing still. It has not turned.</span>
              </li>
            ))}
          </ul>
          {ambush && <ActionButton a={ambush} onAct={onAct} />}
        </section>
      )}

      {trade && (
        <section class="section">
          <ActionButton a={trade} onAct={onAct} />
        </section>
      )}

      {room && room.pile.length > 0 && (
        <section class="section">
          <Rule>Found</Rule>
          <Load view={view} />
          <ul class="moves">
            {room.pile.map((it) => (
              <MoveRow key={it.id} item={it} a={itemAct(acts, 'take', it.id)} verb="Take" onAct={onAct} aside={it.weight_label} />
            ))}
          </ul>
          {takeAll && <ActionButton a={takeAll} onAct={onAct} />}
        </section>
      )}

      {(careful || fast || lightHere) && (
        <section class="section">
          <Rule>Search</Rule>
          {lightHere && <ActionButton a={light} onAct={onAct} />}
          <Choices label="How to search">
            <ChoiceCard a={careful} onAct={onAct} title="Carefully" lines={[{ text: 'Slow. Finds papers.' }]} />
            <ChoiceCard a={fast} onAct={onAct} title="Fast" lines={[{ text: 'Quick and loud.' }]} />
          </Choices>
        </section>
      )}

      {(quiet || loud || locks.length > 0) && (
        <section class="section">
          <Rule>{next ? `Further in: ${next.name}` : 'Further in'}</Rule>
          {lightAhead && <ActionButton a={light} onAct={onAct} />}
          {locks.length > 0 && (
            <Choices label="The locked door">
              {locks.map((a) => (
                <ChoiceCard key={a.action.type} a={a} onAct={onAct} />
              ))}
            </Choices>
          )}
          <Choices label="How to go in">
            <ChoiceCard a={quiet} onAct={onAct} title="Quietly" lines={[{ text: 'Slow. No noise.' }]} />
            <ChoiceCard a={loud} onAct={onAct} title="Fast" lines={[{ text: 'Quick. They may hear.' }]} />
          </Choices>
        </section>
      )}

      <section class="section">
        <Rule>{s.name}</Rule>
        <ol class="route" aria-label="Rooms, in order">
          <li class={outside ? 'stop stop--here stop--entered' : 'stop stop--entered'} aria-current={outside ? 'location' : undefined}>
            <span class="stop__mark" aria-hidden="true" />
            <span class="stop__name">Outside</span>
            <span class="stop__flags" />
          </li>
          {s.rooms.map((r) => (
            <RouteStop key={r.index} r={r} />
          ))}
        </ol>
      </section>

      {room && view.inventory.pack.length > 0 && (
        <section class="section">
          <button type="button" class="disclosure" aria-expanded={dropOpen} onClick={() => setDropOpen(!dropOpen)}>
            <Icon name="pack" size={20} />
            <span class="disclosure__name">Leave something here</span>
            <span class="num">{kg(view.inventory.carried)}</span>
            <Icon name="down" size={16} class={dropOpen ? 'disclosure__icon disclosure__icon--open' : 'disclosure__icon'} />
          </button>
          {dropOpen && (
            <ul class="moves">
              {view.inventory.pack.map((it) => (
                <MoveRow key={it.id} item={it} a={itemAct(acts, 'drop', it.id)} verb="Leave" onAct={onAct} aside={it.weight_label} />
              ))}
            </ul>
          )}
        </section>
      )}

      <Provisions actions={acts} onAct={onAct} />

      {moves.length > 0 && (
        <section class="section">
          <Rule>Go</Rule>
          <Choices label="Where to go">
            {moves.map((a) => (
              <ChoiceCard key={a.action.type} a={a} onAct={onAct} />
            ))}
          </Choices>
        </section>
      )}
    </div>
  );
}

// ------------------------------------------------------------------ an encounter

const BAND_X: Record<Band, number> = { far: 4, near: 50, close: 96 };
const BAND_NAME: Record<Band, string> = { far: 'Far', near: 'Near', close: 'Close' };

/** Far, near, close as a hairline track with a marker where it stands. */
export function Distance({ band, name }: { band: Band; name: string }) {
  const x = BAND_X[band];
  return (
    <svg
      class={band === 'close' ? 'track track--close' : 'track'}
      viewBox="0 0 100 14"
      preserveAspectRatio="none"
      role="img"
      aria-label={`${name}: ${BAND_NAME[band].toLowerCase()}`}
    >
      <line class="track__line" x1="0" y1="7" x2="100" y2="7" vector-effect="non-scaling-stroke" />
      {[0, 50, 100].map((t) => (
        <line key={t} class="track__tick" x1={t} y1="4" x2={t} y2="10" vector-effect="non-scaling-stroke" />
      ))}
      <line class="track__mark" x1={x} y1="0" x2={x} y2="14" vector-effect="non-scaling-stroke" />
      <line class="track__mark" x1={x - 3} y1="1" x2={x + 3} y2="1" vector-effect="non-scaling-stroke" />
      <line class="track__mark" x1={x - 3} y1="13" x2={x + 3} y2="13" vector-effect="non-scaling-stroke" />
    </svg>
  );
}

function Foe({ f }: { f: FoeView }) {
  const close = f.band === 'close';
  return (
    <li class={close ? 'stilled stilled--close' : 'stilled'}>
      <div class="stilled__head">
        <Icon name={f.icon} size={24} />
        <span class="stilled__name">{titleCase(f.name)}</span>
        <span class="label stilled__band">{BAND_NAME[f.band]}</span>
      </div>
      <Distance band={f.band} name={f.name} />
      <div class="stilled__scale label" aria-hidden="true">
        <span>Far</span>
        <span>Near</span>
        <span>Close</span>
      </div>
      <div class="stilled__health">
        <span class="label">Left in it</span>
        <Hairline value={f.health} max={100} label={`${f.name}: ${f.health}% left in it`} danger={close} />
      </div>
    </li>
  );
}

const FIGHT_ORDER = ['strike', 'shoot', 'shove', 'throw', 'back_away', 'hold_still', 'flee'];

export function EncounterPanel({ view, onAct }: PhaseProps) {
  const e = view.encounter;
  if (!e) return null;
  const fight = view.actions
    .filter((a) => a.group === 'fight')
    .sort((a, b) => FIGHT_ORDER.indexOf(a.action.type) - FIGHT_ORDER.indexOf(b.action.type));
  const meds = view.actions.filter((a) => a.group === 'survive' && a.enabled);
  const health = view.vitals.find((v) => v.id === 'health');
  const nerve = view.vitals.find((v) => v.id === 'nerve');
  const kicker = [`Turn ${e.turn + 1}`, e.dark ? 'Dark' : '', e.gripped ? 'It has hold of you' : ''].filter(Boolean).join(' · ');
  // The last few lines of the journal, newest last: what just happened.
  const log = view.journal.slice(-3).map((j) => j.text);
  return (
    <div class={e.foes.some((f) => f.band === 'close') ? 'phase phase--encounter is-close' : 'phase phase--encounter'}>
      <PhaseHead kicker={kicker} title={e.at_home ? 'At the door' : 'They are here'} focusKey={`enc-${e.at_home}`} />

      {log.length > 0 && (
        <div class="enc__log prose" aria-live="polite">
          {log.map((l, i) => (
            <p key={`${view.journal.length}-${i}`} class={i === log.length - 1 ? 'enc__line enc__line--last' : 'enc__line'}>
              {l}
            </p>
          ))}
        </div>
      )}

      <ul class="stilled-list" aria-label="The Stilled">
        {e.foes.map((f, i) => (
          <Foe key={i} f={f} />
        ))}
      </ul>

      <div class="fight" role="group" aria-label="What you do">
        {fight.map((a, i) => (
          <ChoiceCard key={`${a.action.type}-${i}`} a={a} onAct={onAct} />
        ))}
      </div>

      <section class="enc__you" aria-label="You">
        <div class="enc__kit">
          <span class="label">
            <Icon name="blade" size={16} /> {e.weapon}
          </span>
          {e.gun && (
            <span class={e.ammo === 0 ? 'label danger-text' : 'label'}>
              <Icon name="gun" size={16} /> {e.gun}, {e.ammo} {e.ammo === 1 ? 'round' : 'rounds'}
            </span>
          )}
        </div>
        {health && <Meter label="Health" value={health.value} icon="health" danger={15} />}
        {nerve && <Meter label="Nerve" value={nerve.value} icon="nerve" danger={15} />}
      </section>

      {meds.length > 0 && (
        <section class="section">
          <Rule>Mend</Rule>
          <ActionList actions={meds} onAct={onAct} />
        </section>
      )}
    </div>
  );
}

// ------------------------------------------------------------------ barter

export function BarterPanel({ view, onAct }: PhaseProps) {
  const b = view.barter;
  const end = pick(view.actions, 'end_trade');
  if (!b) return null;
  return (
    <div class="phase phase--barter">
      <PhaseHead kicker="The weir steps" title="The widow" icon="sigil_scale" focusKey="barter" />
      <p class="prose barter__line">{b.line}</p>

      <div class="slate" aria-live="polite">
        <span class="label">On the slate</span>
        <span class="num slate__figure">{b.credit}</span>
      </div>

      <section class="section">
        <Rule>She offers</Rule>
        {b.offers.length === 0 ? (
          <p class="prose prose--dim">Her cart is empty.</p>
        ) : (
          <ul class="moves">
            {b.offers.map((o) => (
              <MoveRow
                key={o.item}
                item={{ id: o.item, name: o.name, icon: o.icon, qty: o.qty }}
                a={itemAct(view.actions, 'buy', o.item)}
                verb="Buy"
                onAct={onAct}
                hideReason={b.credit < o.price}
                aside={o.qty > 1 ? `${o.price} each` : o.price}
              />
            ))}
          </ul>
        )}
      </section>

      <section class="section">
        <Rule>She would take</Rule>
        {b.sell.length === 0 ? (
          <p class="prose prose--dim">Nothing you carry interests her.</p>
        ) : (
          <ul class="moves">
            {b.sell.map((o) => (
              <MoveRow
                key={o.item}
                item={{ id: o.item, name: o.name, icon: o.icon, qty: o.qty }}
                a={itemAct(view.actions, 'sell', o.item)}
                verb="Sell"
                onAct={onAct}
                aside={`${o.price} each`}
              />
            ))}
          </ul>
        )}
      </section>

      <Load view={view} />

      {end && (
        <div class="phase__go">
          <Button block variant={b.credit > 0 ? 'danger' : 'primary'} onClick={() => onAct(end.action)}>
            {b.credit > 0 ? `Done. ${b.credit} on the slate is lost` : 'Done trading'}
          </Button>
        </div>
      )}
    </div>
  );
}
