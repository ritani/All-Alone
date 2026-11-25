import { useState } from 'preact/hooks';
import type { Action, MapNode, View } from '../../../core';
import { ActionButton } from '../../components/ActionButton';
import { Icon } from '../../components/Icon';
import { iconPath } from '../../icons/paths';
import { Rule } from '../../components/Rule';
import { duration } from '../../format';

/** The valley as a node map: paths in hairline, places as line icons. */
export function MapDrawing({
  nodes,
  paths,
  selected,
  onSelect,
}: {
  nodes: MapNode[];
  paths: [string, string][];
  selected: string | null;
  onSelect: (id: string) => void;
}) {
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
  return (
    <svg class="map" viewBox="0 0 100 64" aria-hidden="true">
      <g class="map__paths">
        {paths.map(([a, b]) => {
          const p = at.get(a);
          const q = at.get(b);
          if (!p || !q) return null;
          return <line key={`${a}-${b}`} x1={X(p.x)} y1={Y(p.y)} x2={X(q.x)} y2={Y(q.y)} vector-effect="non-scaling-stroke" />;
        })}
      </g>
      {nodes.map((n) => {
        const cls = ['map__node', n.visited ? 'map__node--visited' : '', n.id === selected ? 'map__node--on' : '']
          .filter(Boolean)
          .join(' ');
        return (
          <g key={n.id} class={cls} transform={`translate(${X(n.x) - 3} ${Y(n.y) - 3})`} onClick={() => onSelect(n.id)}>
            <rect class="map__hit" x="-1.5" y="-1.5" width="9" height="9" />
            <path d={iconPath(n.icon)} transform="scale(0.25)" vector-effect="non-scaling-stroke" />
            <text class="map__label" x="3" y="9.6">
              {n.name}
            </text>
          </g>
        );
      })}
    </svg>
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
        <div class="figure-line">
          <span class="label">Places known</span>
          <span class="num">
            {view.map.known}/{view.map.total}
          </span>
        </div>
        <MapDrawing nodes={view.map.nodes} paths={view.map.paths} selected={selected} onSelect={setSelected} />
      </section>
      {node && node.id !== 'home' && (
        <section class="section place" aria-live="polite">
          <Rule>{node.name}</Rule>
          <p class="prose">{node.desc}</p>
          <p class="place__meta label">
            {duration(node.minutes)} walk · {node.rooms_searched}/{node.rooms} rooms searched
          </p>
          {node.notes.map((t) => (
            <p key={t} class="place__note">
              {t}
            </p>
          ))}
          {tripFor(node.id) && <ActionButton a={tripFor(node.id)!} onAct={onAct} />}
        </section>
      )}
      <section class="section">
        <Rule>Places</Rule>
        <ul class="places">
          {others.map((n) => (
            <li key={n.id}>
              <button
                type="button"
                class={n.id === selected ? 'placerow placerow--on' : 'placerow'}
                aria-pressed={n.id === selected}
                onClick={() => setSelected(n.id)}
              >
                <Icon name={n.icon} size={20} />
                <span class="placerow__name">{n.name}</span>
                <span class="placerow__meta num">
                  {duration(n.minutes)}
                  {n.rooms > 0 ? ` · ${n.rooms_searched}/${n.rooms}` : ''}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}
