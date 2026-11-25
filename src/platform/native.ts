/**
 * Capacitor-backed implementations. Loaded with a dynamic import only inside
 * the native apps, so the web build never fetches or runs plugin code.
 */
import { App } from '@capacitor/app';
import { Haptics } from '@capacitor/haptics';
import { Preferences } from '@capacitor/preferences';
import { StatusBar, Style } from '@capacitor/status-bar';
import type { HapticsLike } from './haptics';
import type { NativeApp } from './lifecycle';
import type { PreferencesLike } from './storage';

// Plugins are Proxies that answer every property, `then` included, so one
// must never be resolved from a promise directly (it would never settle).
// Plain wrapper objects are safe to hand through async code.
export const preferences: PreferencesLike = {
  get: (o) => Preferences.get(o),
  set: (o) => Preferences.set(o),
  remove: (o) => Preferences.remove(o),
};

export const haptics: HapticsLike = {
  impact: (o) => Haptics.impact(o as Parameters<typeof Haptics.impact>[0]),
  notification: (o) => Haptics.notification(o as Parameters<typeof Haptics.notification>[0]),
};

/** Wrap a plugin listener (registered asynchronously) as a sync unsubscribe. */
function listen(add: () => Promise<{ remove: () => Promise<void> }>): () => void {
  const handle = add();
  return () => {
    handle.then((h) => h.remove()).catch(() => undefined);
  };
}

export const app: NativeApp = {
  onPause: (fn) => listen(() => App.addListener('pause', fn)),
  onBackground: (fn) =>
    listen(() =>
      App.addListener('appStateChange', (s) => {
        if (!s.isActive) fn();
      }),
    ),
  onBack: (fn) => listen(() => App.addListener('backButton', () => fn())),
  exit: () => {
    App.exitApp().catch(() => undefined);
  },
};

/** Light status bar text over the dark field. */
export async function styleSystemBars(platform: string): Promise<void> {
  try {
    await StatusBar.setStyle({ style: Style.Dark });
    if (platform === 'ios') await StatusBar.setOverlaysWebView({ overlay: true });
  } catch {
    // Cosmetic; never block start-up.
  }
}
