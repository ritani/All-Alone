import { useEffect, useRef, useState } from 'preact/hooks';
import type { Action, ProfileView, View } from '../../../core';
import { Button } from '../../components/Button';
import { Sheet } from '../../components/Sheet';
import { TabBar, type Tab } from '../../components/TabBar';
import { CraftTab } from './CraftTab';
import { Header } from './Header';
import { HouseTab } from './HouseTab';
import { Journal } from './Journal';
import { MapTab } from './MapTab';
import { NotesTab } from './NotesTab';
import { Objectives } from './Objectives';
import { DawnPage, EventPanel, FallbackPanel, NightPanel } from './Phases';
import { BarterPanel, EncounterPanel, PackingPanel, SitePanel, TravelPanel } from './Expedition';
import { StoreTab } from './StoreTab';

type TabId = 'house' | 'store' | 'craft' | 'map' | 'notes';

export interface GameProps {
  view: View;
  profile: ProfileView | null;
  fresh: number;
  seq: number;
  onAct: (a: Action) => void;
  onMenu: () => void;
  /** Codex entries found and not yet opened. */
  unread?: readonly string[];
  onRead?: (id: string) => void;
}

function HomePanel({
  tab,
  view,
  onAct,
  unread,
  onRead,
}: {
  tab: TabId;
  view: View;
  onAct: (a: Action) => void;
  unread: readonly string[];
  onRead: (id: string) => void;
}) {
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
      return <NotesTab view={view} onAct={onAct} unread={unread} onRead={onRead} />;
  }
}

/** The game frame: header, journal, objectives, then the phase's panel. */
/** Actions that end the run: asked about once more before they are sent. */
const FINAL: Partial<Record<Action['type'], { title: string; text: string; go: string }>> = {
  cast_off: {
    title: 'Cast off',
    text: 'Leave Calder’s Reach on the Kittiwake. The run ends here, and the ledger is written.',
    go: 'Cast off',
  },
  walk_onto_ice: {
    title: 'Onto the ice',
    text: 'Walk out onto the frozen river and do not come back. The run ends here, and the ledger is written.',
    go: 'Walk out',
  },
};

export function Game({ view, fresh, seq, onAct: send, onMenu, unread = [], onRead = () => undefined }: GameProps) {
  const [tab, setTab] = useState<TabId>('house');
  const [confirm, setConfirm] = useState<Action | null>(null);
  const onAct = (a: Action) => (FINAL[a.type] ? setConfirm(a) : send(a));
  const final = confirm ? FINAL[confirm.type] : undefined;
  const panel = useRef<HTMLDivElement>(null);

  // A new tab starts at its top.
  const room = view.site?.room?.index ?? -1;
  useEffect(() => {
    if (panel.current) panel.current.scrollTop = 0;
  }, [tab, view.phase, room]);

  const home = view.phase === 'home';
  const tabs: Tab<TabId>[] = [
    { id: 'house', label: 'House', icon: 'house', flag: view.objectives.some((o) => o.urgent) },
    { id: 'store', label: 'Store', icon: 'store' },
    { id: 'craft', label: 'Craft', icon: 'workbench' },
    { id: 'map', label: 'Map', icon: 'map' },
    { id: 'notes', label: 'Notes', icon: 'journal', flag: unread.length > 0, quiet: true },
  ];

  let body;
  switch (view.phase) {
    case 'home':
      body = <HomePanel tab={tab} view={view} onAct={onAct} unread={unread} onRead={onRead} />;
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
    case 'packing':
      body = view.packing ? <PackingPanel view={view} onAct={onAct} /> : <FallbackPanel view={view} onAct={onAct} />;
      break;
    case 'travelling':
      body = view.travel ? <TravelPanel view={view} onAct={onAct} /> : <FallbackPanel view={view} onAct={onAct} />;
      break;
    case 'site':
      body = view.site ? <SitePanel view={view} onAct={onAct} /> : <FallbackPanel view={view} onAct={onAct} />;
      break;
    case 'encounter':
      body = view.encounter ? <EncounterPanel view={view} onAct={onAct} /> : <FallbackPanel view={view} onAct={onAct} />;
      break;
    case 'barter':
      body = view.barter ? <BarterPanel view={view} onAct={onAct} /> : <FallbackPanel view={view} onAct={onAct} />;
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
          {/* The dawn page and an encounter carry their own lines; the journal steps aside. */}
          {view.phase !== 'dawn' && view.phase !== 'encounter' && <Journal entries={view.journal} fresh={fresh} seq={seq} />}
          {view.phase !== 'dawn' && view.phase !== 'encounter' && <Objectives items={view.objectives} />}
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
        {confirm && final && (
          <Sheet title={final.title} onClose={() => setConfirm(null)}>
            <p class="prose">{final.text}</p>
            <div class="sheet__actions">
              <Button
                block
                onClick={() => {
                  setConfirm(null);
                  send(confirm);
                }}
              >
                {final.go}
              </Button>
              <Button block variant="quiet" onClick={() => setConfirm(null)}>
                Not yet
              </Button>
            </div>
          </Sheet>
        )}
      </div>
    </div>
  );
}
