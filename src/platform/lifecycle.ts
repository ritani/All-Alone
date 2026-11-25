/**
 * App lifecycle: make sure saves land before the app is put away, and give the
 * Android back button a sensible meaning.
 *
 * - Web: `visibilitychange` (to hidden) and `pagehide` flush pending writes.
 * - Native: the App plugin's `pause` and `appStateChange` (inactive) do too.
 * - Android back: the UI decides (`back()`): close a sheet, go back a screen,
 *   and only from the title does the app exit. A run is never dropped: it is
 *   autosaved after every action and flushed on the way out.
 *
 * Capacitor itself is reached through `NativeApp`, which `native.ts` builds;
 * this module and its tests never load it.
 */

export type BackResult = 'handled' | 'exit';

export interface LifecycleHooks {
  /** Resolves when every pending save has been written. */
  flush(): Promise<void>;
  /** The Android back button (and nothing else) was pressed. */
  back(): BackResult;
}

/** The part of the Capacitor App plugin the game uses. */
export interface NativeApp {
  onPause(fn: () => void): () => void;
  onBackground(fn: () => void): () => void;
  onBack(fn: () => void): () => void;
  exit(): void;
}

interface Listens {
  addEventListener(type: string, fn: () => void): void;
  removeEventListener(type: string, fn: () => void): void;
}

export interface PageLike extends Listens {
  readonly visibilityState: string;
}

export interface LifecycleEnv {
  native?: NativeApp | null;
  /** `document` on the web. */
  page?: PageLike | null;
  /** `window` on the web. */
  win?: Listens | null;
}

/** Install the lifecycle handlers. Returns a function that removes them. */
export function installLifecycle(hooks: LifecycleHooks, env: LifecycleEnv): () => void {
  const undo: (() => void)[] = [];
  const flush = () => {
    hooks.flush().catch(() => undefined);
  };

  const page = env.page;
  if (page) {
    const onVisibility = () => {
      if (page.visibilityState === 'hidden') flush();
    };
    page.addEventListener('visibilitychange', onVisibility);
    undo.push(() => page.removeEventListener('visibilitychange', onVisibility));
  }
  const win = env.win;
  if (win) {
    win.addEventListener('pagehide', flush);
    undo.push(() => win.removeEventListener('pagehide', flush));
  }

  const app = env.native;
  if (app) {
    undo.push(app.onPause(flush));
    undo.push(app.onBackground(flush));
    undo.push(
      app.onBack(() => {
        if (hooks.back() !== 'exit') return;
        // Leave only once the saves are down.
        hooks
          .flush()
          .catch(() => undefined)
          .then(() => app.exit())
          .catch(() => undefined);
      }),
    );
  }

  return () => {
    for (const f of undo.splice(0)) f();
  };
}
