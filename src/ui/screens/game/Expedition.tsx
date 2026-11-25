/**
 * The expedition screens: packing, the road, a place and its rooms, an
 * encounter with the Stilled, and the widow's barter. Each is drawn from the
 * phase's view section and `view.actions`; every button dispatches an action
 * the core listed.
 */
import { useState } from 'preact/hooks';
import type { ActionView, Band, FoeView, ItemView, RoomSummary, View } from '../../../core';
import { ActionButton } from '../../components/ActionButton';
import { Button } from '../../components/Button';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Meter } from '../../components/Meter';
import { PlaceArt } from '../../components/PlaceArt';
import { Fold, ListRow } from '../../components/ListRow';
import { Rule } from '../../components/Rule';
import { Sheet } from '../../components/Sheet';
import { duration, titleCase, withQty } from '../../format';
import { Provisions } from './HouseTab';
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
    <div class="figure load">
      <span class="figure__name">
        <Icon name="pack" size={16} /> Carrying
      </span>
      <span class="num">{inv.carry_label}</span>
      <Hairline value={inv.carried} max={inv.capacity} label={`Carrying ${inv.carry_label}`} danger={full} />
    </div>
  );
}

// ------------------------------------------------------------------ packing

/** Kinds that are usually carried out: their groups start open. */
const CARRIED_KINDS = ['food', 'drink', 'medicine', 'light', 'weapon', 'ammo'];
const packOpen = new Set<string>(CARRIED_KINDS);

interface PackLine {
  id: string;
  name: string;
  icon: string;
  kind: string;
  /** Tenths of a kilogram, each. */
  weight: number;
  detail: string;
  desc: string;
  packed: number;
  stored: number;
}

/** Everything that could go in the pack, store and pack together, one line per item. */
function packLines(store: ItemView[], pack: ItemView[]): PackLine[] {
  const by = new Map<string, PackLine>();
  const add = (it: ItemView, where: 'packed' | 'stored') => {
    const line = by.get(it.id) ?? {
      id: it.id,
      name: it.name,
      icon: it.icon,
      kind: it.kind,
      weight: it.weight,
      detail: it.detail,
      desc: it.desc,
      packed: 0,
      stored: 0,
    };
    line[where] += it.qty;
    by.set(it.id, line);
  };
  pack.forEach((it) => add(it, 'packed'));
  store.forEach((it) => add(it, 'stored'));
  return [...by.values()];
}

/** One thing to take or leave: the packed count between minus and plus. */
function PackRow({ line, acts, onAct, onOpen }: { line: PackLine; acts: ActionView[]; onAct: OnAct; onOpen: () => void }) {
  const add = itemAct(acts, 'pack', line.id);
  const less = itemAct(acts, 'unpack', line.id);
  const canAdd = add?.enabled === true;
  const canLess = less?.enabled === true;
  const why = !canAdd && line.stored > 0 ? add?.reason : null;
  return (
    <li class={line.packed > 0 ? 'prow prow--in' : 'prow'}>
      <button type="button" class="prow__item" onClick={onOpen}>
        <Icon name={line.icon} size={20} class="prow__icon" />
        <span class="prow__main">
          <span class="prow__name">{line.name}</span>
          <span class="prow__meta">
            <span class="num">{kg(line.weight)}</span>
            {line.stored > 0 && <span class="num"> · {line.stored} in store</span>}
            {why && <span> · {why}</span>}
          </span>
        </span>
      </button>
      <span class="stepper" role="group" aria-label={`${line.name}: ${line.packed} packed`}>
        <button
          type="button"
          class="stepper__btn"
          aria-label={`Leave one ${line.name}`}
          disabled={!canLess}
          onClick={() => less && onAct(withQty(less.action, 1))}
        >
          <Icon name="minus" size={16} />
        </button>
        <output class="stepper__value num" aria-live="polite">
          {line.packed}
        </output>
        <button
          type="button"
          class="stepper__btn"
          aria-label={`Pack one ${line.name}`}
          disabled={!canAdd}
          onClick={() => add && onAct(withQty(add.action, 1))}
        >
          <Icon name="plus" size={16} />
        </button>
      </span>
    </li>
  );
}

export function PackingPanel({ view, onAct }: PhaseProps) {
  const p = view.packing;
  const inv = view.inventory;
  const acts = view.actions;
  const depart = pick(acts, 'depart');
  const cancel = pick(acts, 'cancel_trip');
  const dog = pick(acts, 'take_dog');
  const [, redraw] = useState(0);
  const [item, setItem] = useState<string | null>(null);
  if (!p) return null;
  const lines = packLines(inv.store, inv.pack);
  const kinds = [...new Set(lines.map((l) => l.kind))].sort(
    (a, b) => (KIND_ORDER.indexOf(a) + 1 || 99) - (KIND_ORDER.indexOf(b) + 1 || 99),
  );
  const toggle = (k: string) => {
    if (packOpen.has(k)) packOpen.delete(k);
    else packOpen.add(k);
    redraw((n) => n + 1);
  };
  const open = item ? lines.find((l) => l.id === item) : undefined;
  return (
    <div class="phase phase--packing">
      <PhaseHead kicker="Packing" title={`To ${p.name}`} focusKey={`pack-${p.place}`}>
        <p class="phase__meta">
          <Icon name="clock" size={16} /> <span class="num">{duration(p.minutes)}</span>
          {p.unvisited ? ' · not been there' : ` · ${p.rooms_searched}/${p.rooms} rooms searched`}
        </p>
      </PhaseHead>

      <Load view={view} />

      <ul class="needs" aria-label="What to bring">
        <Need met={p.has_light} icon="light" text="Light" />
        <Need met={p.has_weapon} icon="blade" text="Weapon" />
        <Need met={p.meals > 0} icon="food" text={p.meals > 0 ? `${p.meals} ${p.meals === 1 ? 'meal' : 'meals'}` : 'Food'} />
        <Need met={p.water > 0} icon="water" text={p.water > 0 ? `${p.water} water` : 'Water'} />
        {p.needs.map((n) => (
          <Need key={n.id} met={n.met} icon={n.icon} text={n.text} wide />
        ))}
      </ul>

      {dog && (
        <ListRow
          icon="sigil_dog"
          title="The dog"
          aside={p.take_dog ? 'Coming' : 'Staying'}
          onClick={dog.enabled ? () => onAct(dog.action) : undefined}
          label={`${dog.label}. The dog is ${p.take_dog ? 'coming' : 'staying'}.`}
        />
      )}

      <section class="section folds" aria-label="What to take">
        {lines.length === 0 && <p class="prose prose--dim">Nothing to take.</p>}
        {kinds.map((k) => {
          const list = lines.filter((l) => l.kind === k);
          const packed = list.reduce((n, l) => n + l.packed, 0);
          return (
            <Fold
              key={k}
              title={KIND_NAMES[k] ?? k}
              count={<span class="num">{packed > 0 ? `${packed} packed` : list.length}</span>}
              open={packOpen.has(k) || packed > 0}
              onToggle={() => toggle(k)}
            >
              <ul class="prows">
                {list.map((l) => (
                  <PackRow key={l.id} line={l} acts={acts} onAct={onAct} onOpen={() => setItem(l.id)} />
                ))}
              </ul>
            </Fold>
          );
        })}
      </section>

      {open && (
        <Sheet title={open.name} onClose={() => setItem(null)}>
          {open.detail && <p class="item__detail num">{open.detail}</p>}
          <p class="prose prose--dim">{open.desc}</p>
          <ul class="moves">
            {open.stored > 0 && (
              <MoveRow
                item={{ id: open.id, name: 'From the store', icon: 'store', qty: open.stored }}
                a={itemAct(acts, 'pack', open.id)}
                verb="Pack"
                onAct={onAct}
                aside={`${kg(open.weight)} each`}
              />
            )}
            {open.packed > 0 && (
              <MoveRow
                item={{ id: open.id, name: 'In the pack', icon: 'pack', qty: open.packed }}
                a={itemAct(acts, 'unpack', open.id)}
                verb="Leave"
                onAct={onAct}
              />
            )}
          </ul>
        </Sheet>
      )}

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

function Need({ met, icon, text, wide = false }: { met: boolean; icon: string; text: string; wide?: boolean }) {
  const cls = ['need', met ? 'need--met' : '', wide ? 'need--wide' : ''].filter(Boolean).join(' ');
  return (
    <li class={cls}>
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
        <p class="phase__meta">
          <span class="num">{view.clock}</span> · {light}
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
      {back && <ActionButton a={back} onAct={onAct} />}
      <Provisions view={view} onAct={onAct} />
    </div>
  );
}

// ------------------------------------------------------------------ a place

const SEARCH_NAME = { none: 'Not searched', partial: 'Part searched', full: 'Searched' } as const;

function stopFlags(r: RoomSummary): string[] {
  const flags: string[] = [];
  if (r.locked) flags.push('locked');
  if (r.dark) flags.push('dark');
  if (r.flooded) flags.push('flooded');
  if (r.search === 'full') flags.push('searched');
  else if (r.search === 'partial') flags.push('part searched');
  return flags;
}

/**
 * The rooms as a short route across the page: outside on the left, each room
 * a square, where you are a square in a square. Names live in the list's
 * accessible text; the line under it names where you are and what is next.
 */
function Route({ rooms, here }: { rooms: RoomSummary[]; here: number | null }) {
  const stops = [{ index: -1, name: 'Outside', entered: true, here: here === null, dark: false, flooded: false, locked: false, search: 'none' as const }, ...rooms];
  return (
    <ol class="route" aria-label="Rooms, in order">
      {stops.map((r) => {
        const unseen = !r.entered && r.name === 'Further in';
        const cls = ['stop', r.here ? 'stop--here' : '', r.entered ? 'stop--entered' : '', unseen ? 'stop--unseen' : '', r.locked ? 'stop--locked' : '']
          .filter(Boolean)
          .join(' ');
        const flags = r.index >= 0 ? stopFlags(r) : [];
        return (
          <li key={r.index} class={cls} aria-current={r.here ? 'location' : undefined} title={r.name}>
            <span class="stop__mark" aria-hidden="true" />
            <span class="visually-hidden">
              {r.name}
              {flags.length > 0 ? `, ${flags.join(', ')}` : ''}
            </span>
          </li>
        );
      })}
    </ol>
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
  const state = room
    ? [room.dark ? (room.lit ? 'Lit' : 'Dark') : '', room.flooded ? 'Flooded' : '', SEARCH_NAME[room.search]].filter(Boolean)
    : [s.widow_here ? 'The widow is on the steps' : ''].filter(Boolean);
  const entered = s.rooms.filter((r) => r.entered).length;
  return (
    <div class="phase phase--site">
      <figure class={room ? 'site__art site__art--small' : 'site__art'}>
        <PlaceArt place={s.place} />
      </figure>
      <PhaseHead kicker={outside ? 'Outside' : s.name} title={room ? room.name : s.name} focusKey={`site-${s.place}-${room?.index ?? -1}`}>
        <p class="prose site__line">{room ? room.desc : s.desc}</p>
      </PhaseHead>

      <div class="site__where">
        <Route rooms={s.rooms} here={room ? room.index : null} />
        <p class="site__state">
          <span>
            <Icon name="noise" size={16} /> <span class="visually-hidden">Noise here </span>
            <span class="num">{s.site_noise}</span>
          </span>
          {state.map((t) => (
            <span key={t}> · {t}</span>
          ))}
          <span class="num site__count">
            {' '}
            · {entered}/{s.rooms.length} rooms
          </span>
        </p>
      </div>

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

      {trade && <ActionButton a={trade} onAct={onAct} />}

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
            <ChoiceCard a={careful} onAct={onAct} title="Carefully" lines={[{ text: 'Finds papers.' }]} />
            <ChoiceCard a={fast} onAct={onAct} title="Fast" />
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
            <ChoiceCard a={quiet} onAct={onAct} title="Quietly" />
            <ChoiceCard a={loud} onAct={onAct} title="Fast" />
          </Choices>
        </section>
      )}

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

      <div class="lrows">
        <Provisions view={view} onAct={onAct} />
        {room && view.inventory.pack.length > 0 && (
          <ListRow
            icon="pack"
            title="Leave something here"
            aside={<span class="num">{kg(view.inventory.carried)}</span>}
            onClick={() => setDropOpen(true)}
          />
        )}
      </div>
      {dropOpen && room && (
        <Sheet title="Leave something here" onClose={() => setDropOpen(false)}>
          <Load view={view} />
          <ul class="moves">
            {view.inventory.pack.map((it) => (
              <MoveRow key={it.id} item={it} a={itemAct(acts, 'drop', it.id)} verb="Leave" onAct={onAct} aside={it.weight_label} />
            ))}
          </ul>
        </Sheet>
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
        <span class="stilled__health">
          <Hairline value={f.health} max={100} label={`${f.name}: ${f.health}% left in it`} danger={close} />
        </span>
        <span class="stilled__band">{BAND_NAME[f.band]}</span>
      </div>
      <Distance band={f.band} name={f.name} />
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
        <div class="enc__meters">
          {health && <Meter label="Health" value={health.value} icon="health" danger={15} />}
          {nerve && <Meter label="Nerve" value={nerve.value} icon="nerve" danger={15} />}
        </div>
        <p class="enc__kit">
          <span>
            <Icon name="blade" size={16} /> {e.weapon}
          </span>
          {e.gun && (
            <span class={e.ammo === 0 ? 'danger-text' : undefined}>
              <Icon name="gun" size={16} /> {e.gun}, <span class="num">{e.ammo}</span> {e.ammo === 1 ? 'round' : 'rounds'}
            </span>
          )}
        </p>
      </section>

      <Provisions view={view} onAct={onAct} />
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
        <span class="slate__name">On the slate</span>
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
