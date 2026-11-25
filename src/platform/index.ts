/**
 * Platform selection, done once at start. The UI talks to these interfaces
 * only; it never imports Capacitor.
 *
 * Inside the native apps Capacitor's bridge defines `window.Capacitor` before
 * any page script runs, so detection needs no import, and the plugin code is
 * loaded on demand (`./native`) only there.
 */
import { nativeHaptics, webHaptics, type Haptics } from './haptics';
import { installLifecycle, type LifecycleHooks } from './lifecycle';
import { chooseStore, type KeyValueStore } from './storage';

export interface Platform {
  /** 'web', 'ios' or 'android'. */
  name: string;
  native: boolean;
  storage: KeyValueStore;
  haptics: Haptics;
  /** Hook saves and the back button to the app's lifecycle. Returns an uninstaller. */
  start(hooks: LifecycleHooks): () => void;
}

interface CapacitorGlobal {
  isNativePlatform?: () => boolean;
  getPlatform?: () => string;
}

/** Which platform the page runs on, without loading Capacitor. */
export function detect(g: { Capacitor?: CapacitorGlobal } = globalThis as { Capacitor?: CapacitorGlobal }): {
  name: string;
  native: boolean;
} {
  try {
    const cap = g.Capacitor;
    if (cap?.isNativePlatform?.() === true) return { name: cap.getPlatform?.() ?? 'native', native: true };
  } catch {
    // Treat anything odd as the web.
  }
  return { name: 'web', native: false };
}

const loadNative = () => import('./native');

export function createPlatform(env = detect()): Platform {
  if (!env.native) {
    return {
      name: 'web',
      native: false,
      storage: chooseStore({ native: false, preferences: () => Promise.reject(new Error('web')) }),
      haptics: webHaptics(),
      start: (hooks) => installLifecycle(hooks, { page: document, win: window }),
    };
  }
  return {
    name: env.name,
    native: true,
    storage: chooseStore({ native: true, preferences: async () => (await loadNative()).preferences }),
    haptics: nativeHaptics(async () => (await loadNative()).haptics),
    start(hooks) {
      let stopped = false;
      let stopNative: (() => void) | null = null;
      // The page events still fire inside the webview; keep them as a backstop.
      const stopPage = installLifecycle(hooks, { page: document, win: window });
      loadNative()
        .then((m) => {
          void m.styleSystemBars(env.name);
          if (!stopped) stopNative = installLifecycle(hooks, { native: m.app });
        })
        .catch(() => undefined);
      return () => {
        stopped = true;
        stopPage();
        stopNative?.();
      };
    },
  };
}
