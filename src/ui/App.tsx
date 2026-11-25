import { useEffect, useMemo } from 'preact/hooks';
import { initCore } from '../core';
import { createPlatform } from '../platform';
import { popBack } from './back';
import { installDragGuard } from './drag';
import { Toasts } from './components/Toasts';
import { Ended } from './screens/Ended';
import { Game } from './screens/game/Game';
import { Intro } from './screens/Intro';
import { Kit } from './screens/Kit';
import { Fatal, HowTo, Marks, Record, Unlocks } from './screens/Profile';
import { Setup } from './screens/Setup';
import { Title } from './screens/Title';
import { StoreContext, useAppState, useStoreInstance } from './state/context';
import { Store } from './state/store';

/**
 * Dev review route: `?kit` shows the primitives sheet. Development builds
 * only: in production `import.meta.env.DEV` is false and the Kit is dropped
 * from the bundle.
 */
function wantsKit(): boolean {
  return import.meta.env.DEV && new URLSearchParams(window.location.search).has('kit');
}

function Screens() {
  const store = useStoreInstance();
  const s = useAppState();

  // Night dims the whole interface, including the page behind the column.
  useEffect(() => {
    const dark = s.screen === 'game' && (s.view?.dark === true || s.view?.phase === 'night');
    document.documentElement.classList.toggle('is-dark', dark);
    // The game frame holds still; only its panel scrolls.
    document.documentElement.classList.toggle('is-game', s.screen === 'game');
  }, [s.screen, s.view?.dark, s.view?.phase]);

  let screen;
  switch (s.screen) {
    case 'loading':
      screen = (
        <main class="frame loading" aria-busy="true">
          <p class="label">Calder&#8217;s Reach</p>
        </main>
      );
      break;
    case 'fatal':
      screen = <Fatal message={s.fatal} />;
      break;
    case 'title':
      screen = (
        <Title
          canContinue={s.hasRun}
          run={s.view}
          profile={s.profile}
          notices={s.notices}
          onDismissNotice={(i) => store.dismissNotice(i)}
          onBegin={() => store.go('setup')}
          onContinue={() => store.continueRun()}
          onNav={(to) => store.go(to)}
        />
      );
      break;
    case 'setup':
      screen = s.profile && (
        <Setup
          profile={s.profile}
          hasRun={s.hasRun}
          run={s.view}
          daily={store.dailyStatus()}
          onBack={() => store.go('title')}
          onStart={(o) => store.startRun(o)}
          onDaily={() => store.startDaily()}
          onUnlocks={() => store.go('unlocks')}
        />
      );
      break;
    case 'intro':
      screen = <Intro lines={s.intro} view={s.view} dailyCounts={s.dailyCounts} onGo={() => store.beginPlay()} />;
      break;
    case 'game':
      screen = s.view && (
        <Game
          view={s.view}
          profile={s.profile}
          fresh={s.fresh}
          seq={s.seq}
          onAct={(a) => void store.act(a)}
          onMenu={() => store.go('title')}
          unread={s.codexUnread}
          onRead={(id) => store.markRead(id)}
        />
      );
      break;
    case 'ended':
      screen = <Ended view={s.view} end={s.end} profile={s.profile} onDone={() => store.leaveEnded()} />;
      break;
    case 'marks':
      screen = s.profile && <Marks profile={s.profile} onBack={() => store.go('title')} />;
      break;
    case 'record':
      screen = s.profile && <Record profile={s.profile} onBack={() => store.go('title')} />;
      break;
    case 'unlocks':
      screen = s.profile && (
        <Unlocks profile={s.profile} onBack={() => void store.back()} onUnlock={(u) => store.unlock(u)} />
      );
      break;
    case 'howto':
      screen = <HowTo onBack={() => store.go('title')} />;
      break;
  }

  return (
    <>
      <Toasts toasts={s.toasts} onDismiss={(id) => store.dismissToast(id)} />
      {screen}
    </>
  );
}

export function App() {
  const platform = useMemo(() => createPlatform(), []);
  const store = useMemo(
    () => new Store({ storage: platform.storage, haptics: platform.haptics, init: () => initCore() }),
    [platform],
  );
  useEffect(() => installDragGuard(), []);
  useEffect(() => {
    if (import.meta.env.DEV && wantsKit()) return undefined;
    void store.boot();
    return platform.start({
      flush: () => store.flush(),
      back: () => (popBack() ? 'handled' : store.back()),
    });
  }, [store, platform]);
  if (import.meta.env.DEV && wantsKit()) return <Kit />;
  return (
    <StoreContext.Provider value={store}>
      <Screens />
    </StoreContext.Provider>
  );
}
