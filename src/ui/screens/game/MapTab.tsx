import { useEffect, useRef, useState } from 'preact/hooks';
import type { Action, MapNode, View } from '../../../core';
import { ActionButton } from '../../components/ActionButton';
import { Icon } from '../../components/Icon';
import { ListRow } from '../../components/ListRow';
import { Rule } from '../../components/Rule';
import { Sheet } from '../../components/Sheet';
import { iconPath } from '../../icons/paths';
import { duration } from '../../format';

/** Map labels are drawn at this size on screen, however wide the map is. */
const LABEL_PX = 12.5;

/** Drawing units per CSS pixel for an SVG 100 units wide (labels keep their size). */
function useUnitsPerPx() {
  const ref = useRef<SVGSVGElement>(null);
  const [k, setK] = useState(100 / 358);
  useEffect(() => {
    const el = ref.current;
    if (!el) return undefined;
    // The drawing keeps its aspect inside its box: its screen matrix gives the real scale.
    const measure = () => {
      const m = el.getScreenCTM();
      const scale = m ? Math.hypot(m.a, m.b) : 0;
      if (scale > 0) setK(1 / scale);
    };
    measure();
    if (typeof ResizeObserver === 'undefined') return undefined;
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);
  return [ref, k] as const;
}

/** The valley as a node map: paths in hairline, places as line icons. */
export function MapDrawing({
  nodes,
  paths,
  selected,
  onSelect,
  route,
}: {
  nodes: MapNode[];
  paths: [string, string][];
  selected: string | null;
  onSelect: (id: string) => void;
  /** Draw the way between two nodes, from the first to the second. */
  route?: [string, string] | undefined;
}) {
  const [svg, unit] = useUnitsPerPx();
  const at = new Map(nodes.map((n) => [n.id, n]));
  // Map coordinates are 0..100. Fit the known places into a 100 x 64 drawing,
  // zoomed in while few are known, but never so far that the walk looks short.
  const xs = nodes.map((n) => n.x);
  const ys = nodes.map((n) => n.y);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  const k = Math.min(76 / Math.max(maxX - minX, 1), 44 / Math.max(maxY - minY, 1), 2.4);
  const X = (x: number) => 50 + (x - (minX + maxX) / 2) * k;
  const Y = (y: number) => 30 + (y - (minY + maxY) / 2) * k;
  const fs = LABEL_PX * unit;
  // Labels are large enough to read, so only the place in question carries one;
  // the list under the map names the rest.
  const named = (id: string) => id === selected || (route !== undefined && id !== 'home' && route.includes(id));
  return (
    <svg class="map" viewBox="0 0 100 64" aria-hidden="true" ref={svg}>
      <g class="map__paths">
        {paths.map(([a, b]) => {
          const p = at.get(a);
          const q = at.get(b);
          if (!p || !q) return null;
          return <line key={`${a}-${b}`} x1={X(p.x)} y1={Y(p.y)} x2={X(q.x)} y2={Y(q.y)} vector-effect="non-scaling-stroke" />;
        })}
      </g>
      {route &&
        (() => {
          const p = at.get(route[0]);
          const q = at.get(route[1]);
          if (!p || !q) return null;
          return (
            <g class="map__route">
              <line x1={X(p.x)} y1={Y(p.y)} x2={X(q.x)} y2={Y(q.y)} vector-effect="non-scaling-stroke" />
            </g>
          );
        })()}
      {nodes.map((n) => {
        const cls = ['map__node', n.visited ? 'map__node--visited' : '', n.id === selected ? 'map__node--on' : '']
          .filter(Boolean)
          .join(' ');
        const x = X(n.x);
        // Keep a label inside the drawing near the edges.
        const anchor = x < 18 ? 'start' : x > 82 ? 'end' : 'middle';
        const lx = anchor === 'start' ? -1.5 : anchor === 'end' ? 7.5 : 3;
        const below = Y(n.y) < 52;
        return (
          <g key={n.id} class={cls} transform={`translate(${x - 3} ${Y(n.y) - 3})`} onClick={() => onSelect(n.id)}>
            <rect class="map__hit" x="-1.5" y="-1.5" width="9" height="9" />
            <path d={iconPath(n.icon)} transform="scale(0.25)" vector-effect="non-scaling-stroke" />
            {named(n.id) && (
              <text class="map__label" x={lx} y={below ? 7.5 + fs : -1.5} font-size={fs} text-anchor={anchor} stroke-width={fs * 0.3}>
                {n.name}
              </text>
            )}
          </g>
        );
      })}
    </svg>
  );
}

/** A place in full, with the way to go there. */
function PlaceSheet({ node, trip, onAct, onClose }: { node: MapNode; trip: ReturnType<View['actions']['find']>; onAct: (a: Action) => void; onClose: () => void }) {
  return (
    <Sheet title={node.name} onClose={onClose}>
      <p class="place__figures">
        <span>
          <Icon name="clock" size={16} /> <span class="num">{duration(node.minutes)}</span> walk
        </span>
        {node.rooms > 0 && (
          <span>
            <Icon name="check" size={16} /> <span class="num">{node.rooms_searched}/{node.rooms}</span> rooms searched
          </span>
        )}
        {!node.visited && <span>Not been there</span>}
      </p>
      <p class="prose">{node.desc}</p>
      {node.notes.map((t) => (
        <p key={t} class="place__note">
          {t}
        </p>
      ))}
      {trip && <ActionButton a={trip} onAct={onAct} />}
    </Sheet>
  );
}

export function MapTab({ view, onAct }: { view: View; onAct: (a: Action) => void }) {
  const [selected, setSelected] = useState<string | null>(null);
  const node = selected ? view.map.nodes.find((n) => n.id === selected) : undefined;
  const trips = view.actions.filter((a) => a.action.type === 'plan_trip');
  const tripFor = (id: string) => trips.find((a) => a.action.type === 'plan_trip' && a.action.place === id);
  const others = view.map.nodes.filter((n) => n.id !== 'home');
  return (
    <>
      <section class="section">
        <MapDrawing nodes={view.map.nodes} paths={view.map.paths} selected={selected} onSelect={(id) => id !== 'home' && setSelected(id)} />
      </section>
      <section class="section">
        <Rule>
          Places{' '}
          <span class="num rule__count">
            {view.map.known}/{view.map.total}
          </span>
        </Rule>
        <ul class="lrows">
          {others.map((n) => (
            <li key={n.id}>
              <ListRow
                icon={n.icon}
                title={n.name}
                aside={
                  <span class="num">
                    {duration(n.minutes)}
                    {n.rooms > 0 ? ` · ${n.rooms_searched}/${n.rooms}` : ''}
                  </span>
                }
                dim={!n.visited}
                onClick={() => setSelected(n.id)}
              />
            </li>
          ))}
        </ul>
      </section>
      {node && node.id !== 'home' && <PlaceSheet node={node} trip={tripFor(node.id)} onAct={onAct} onClose={() => setSelected(null)} />}
    </>
  );
}
