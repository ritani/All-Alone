import { useEffect, useRef, useState } from 'preact/hooks';
import type { Action, ProfileView, View } from '../../../core';
import { TabBar, type Tab } from '../../components/TabBar';
import { CraftTab } from './CraftTab';
import { Header } from './Header';
import { HouseTab } from './HouseTab';
import { Journal } from './Journal';
import { MapTab } from './MapTab';
import { NotesTab } from './NotesTab';
import { Objectives } from './Objectives';
import { DawnPage, EventPanel, FallbackPanel, NightPanel } from './Phases';
import { StoreTab } from './StoreTab';

type TabId = 'house' | 'store' | 'craft' | 'map' | 'notes';

export interface GameProps {
  view: View;
  profile: ProfileView | null;
  fresh: number;
  seq: number;
  onAct: (a: Action) => void;
  onMenu: () => void;
}

function HomePanel({ tab, view, profile, onAct }: { tab: TabId; view: View; profile: ProfileView | null; onAct: (a: Action) => void }) {
  switch (tab) {
    case 'house':
      return <HouseTab view={view} onAct={onAct} />;
    case 'store':
      return <StoreTab view={view} onAct={onAct} />;
    case 'craft':
      return <CraftTab view={view} onAct={onAct} />;
    case 'map':
      return <MapTab view={view} onAct={onAct} />;
    case 'notes':
      return <NotesTab view={view} profile={profile} onAct={onAct} />;
  }
}

/** The game frame: header, journal, objectives, then the phase's panel. */
export function Game({ view, profile, fresh, seq, onAct, onMenu }: GameProps) {
  const [tab, setTab] = useState<TabId>('house');
  const panel = useRef<HTMLDivElement>(null);

  // A new tab starts at its top.
  useEffect(() => {
    if (panel.current) panel.current.scrollTop = 0;
  }, [tab, view.phase]);

  const home = view.phase === 'home';
  const tabs: Tab<TabId>[] = [
    { id: 'house', label: 'House', icon: 'house', flag: view.objectives.some((o) => o.urgent) },
    { id: 'store', label: 'Store', icon: 'store' },
    { id: 'craft', label: 'Craft', icon: 'workbench' },
    { id: 'map', label: 'Map', icon: 'map' },
    { id: 'notes', label: 'Notes', icon: 'journal' },
  ];

  let body;
  switch (view.phase) {
    case 'home':
      body = <HomePanel tab={tab} view={view} profile={profile} onAct={onAct} />;
      break;
    case 'event':
      body = <EventPanel view={view} onAct={onAct} />;
      break;
    case 'night':
      body = <NightPanel view={view} onAct={onAct} />;
      break;
    case 'dawn':
      body = <DawnPage view={view} onAct={onAct} />;
      break;
    default:
      body = <FallbackPanel view={view} onAct={onAct} />;
  }

  const cls = ['game', `game--${view.phase}`, view.dark ? 'is-dark' : '', view.phase === 'night' ? 'is-night' : '']
    .filter(Boolean)
    .join(' ');

  return (
    <div class={cls}>
      <div class="game__col">
        <div class="game__bar">
          <button type="button" class="game__menu" onClick={onMenu} aria-label="Leave to the title">
            <span class="wordmark wordmark--tiny">ALL ALONE</span>
          </button>
          <span class="game__where label">{view.location}</span>
        </div>
        <Header view={view} />
        <main class="game__main" aria-label="The game">
          {view.phase !== 'dawn' && <Journal entries={view.journal} fresh={fresh} seq={seq} />}
          {view.phase !== 'dawn' && <Objectives items={view.objectives} />}
          <div
            class={home ? 'game__panel' : 'game__panel game__panel--phase'}
            ref={panel}
            id="home-panel"
            role={home ? 'tabpanel' : undefined}
            aria-labelledby={home ? `home-tab-${tab}` : undefined}
            tabIndex={-1}
          >
            {body}
          </div>
        </main>
        {home && <TabBar tabs={tabs} current={tab} onChange={setTab} label="The lock house" idBase="home" />}
      </div>
    </div>
  );
}
