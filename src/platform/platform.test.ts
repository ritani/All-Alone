import { describe, expect, it, vi } from 'vitest';
import { nativeHaptics, type HapticsLike } from './haptics';
import { detect } from './index';
import { installLifecycle, type NativeApp, type PageLike } from './lifecycle';
import {
  MAX_UNREAD_BYTES,
  StorageError,
  chooseStore,
  memoryStore,
  parseUnread,
  preferencesStore,
  type PreferencesLike,
} from './storage';

/** An in-memory stand-in for Capacitor Preferences. */
function fakePreferences(fail = false): PreferencesLike & { data: Map<string, string> } {
  const data = new Map<string, string>();
  const maybe = () => (fail ? Promise.reject(new Error('bridge down')) : Promise.resolve());
  return {
    data,
    get: async ({ key }) => {
      await maybe();
      return { value: data.get(key) ?? null };
    },
    set: async ({ key, value }) => {
      await maybe();
      data.set(key, value);
    },
    remove: async ({ key }) => {
      await maybe();
      data.delete(key);
    },
  };
}

class FakeTarget {
  visibilityState = 'visible';
  private readonly fns = new Map<string, Set<() => void>>();
  addEventListener(type: string, fn: () => void) {
    if (!this.fns.has(type)) this.fns.set(type, new Set());
    this.fns.get(type)?.add(fn);
  }
  removeEventListener(type: string, fn: () => void) {
    this.fns.get(type)?.delete(fn);
  }
  fire(type: string) {
    for (const fn of this.fns.get(type) ?? []) fn();
  }
  count() {
    return [...this.fns.values()].reduce((n, s) => n + s.size, 0);
  }
}

function fakeApp() {
  const on = { pause: new Set<() => void>(), background: new Set<() => void>(), back: new Set<() => void>() };
  const sub = (set: Set<() => void>) => (fn: () => void) => {
    set.add(fn);
    return () => set.delete(fn);
  };
  const app: NativeApp & { exits: number } = {
    exits: 0,
    onPause: sub(on.pause),
    onBackground: sub(on.background),
    onBack: sub(on.back),
    exit: () => {
      app.exits += 1;
    },
  };
  const fire = (k: keyof typeof on) => on[k].forEach((f) => f());
  return { app, on, fire };
}

const tick = () => new Promise((r) => setTimeout(r, 0));

describe('storage selection', () => {
  it('uses Preferences on native and never touches the web store', async () => {
    const prefs = fakePreferences();
    const web = vi.fn(() => memoryStore());
    const store = chooseStore({ native: true, preferences: () => Promise.resolve(prefs), web });
    await store.set('k', 'v');
    expect(await store.get('k')).toBe('v');
    expect(prefs.data.get('k')).toBe('v');
    await store.remove('k');
    expect(await store.get('k')).toBeNull();
    expect(web).not.toHaveBeenCalled();
  });

  it('uses the web store off native and never loads Preferences', async () => {
    const mem = memoryStore();
    const preferences = vi.fn(() => Promise.resolve(fakePreferences()));
    const store = chooseStore({ native: false, preferences, web: () => mem });
    await store.set('k', 'v');
    expect(mem.data.get('k')).toBe('v');
    expect(preferences).not.toHaveBeenCalled();
  });

  it('maps Preferences failures to StorageError', async () => {
    const store = preferencesStore(fakePreferences(true));
    await expect(store.get('k')).rejects.toBeInstanceOf(StorageError);
    await expect(store.set('k', 'v')).rejects.toMatchObject({ code: 'failed' });
  });

  it('reads a plugin that cannot load as unavailable storage', async () => {
    const store = chooseStore({ native: true, preferences: () => Promise.reject(new Error('no plugin')) });
    await expect(store.get('k')).rejects.toMatchObject({ code: 'unavailable' });
  });

  it('detects the native shell only from a real Capacitor bridge', () => {
    expect(detect({})).toEqual({ name: 'web', native: false });
    expect(detect({ Capacitor: { isNativePlatform: () => false, getPlatform: () => 'web' } }).native).toBe(false);
    expect(detect({ Capacitor: { isNativePlatform: () => true, getPlatform: () => 'ios' } })).toEqual({
      name: 'ios',
      native: true,
    });
    const throwing = {
      isNativePlatform: () => {
        throw new Error('x');
      },
    };
    expect(detect({ Capacitor: throwing }).native).toBe(false);
  });
});

describe('unread codex list (untrusted UI state)', () => {
  it('keeps only short plain ids from a JSON array', () => {
    expect(parseUnread(null)).toEqual([]);
    expect(parseUnread('not json')).toEqual([]);
    expect(parseUnread('{"a":1}')).toEqual([]);
    expect(parseUnread('["fg_cold","note_lowe","fg_cold",3,null,"<b>x</b>","' + 'a'.repeat(41) + '"]')).toEqual([
      'fg_cold',
      'note_lowe',
    ]);
  });

  it('refuses oversized input and caps the count', () => {
    expect(parseUnread('["a"' + ',"a"'.repeat(MAX_UNREAD_BYTES) + ']')).toEqual([]);
    const many = JSON.stringify(Array.from({ length: 500 }, (_, i) => `e${i}`));
    expect(many.length).toBeLessThan(MAX_UNREAD_BYTES);
    expect(parseUnread(many)).toHaveLength(200);
  });
});

describe('lifecycle', () => {
  it('flushes saves when the page is hidden or unloaded', () => {
    const page = new FakeTarget();
    const win = new FakeTarget();
    const flush = vi.fn(() => Promise.resolve());
    const stop = installLifecycle({ flush, back: () => 'handled' }, { page: page satisfies PageLike, win });
    page.fire('visibilitychange');
    expect(flush).not.toHaveBeenCalled();
    page.visibilityState = 'hidden';
    page.fire('visibilitychange');
    win.fire('pagehide');
    expect(flush).toHaveBeenCalledTimes(2);
    stop();
    expect(page.count() + win.count()).toBe(0);
  });

  it('flushes on native pause and background', () => {
    const { app, fire, on } = fakeApp();
    const flush = vi.fn(() => Promise.resolve());
    const stop = installLifecycle({ flush, back: () => 'handled' }, { native: app });
    fire('pause');
    fire('background');
    expect(flush).toHaveBeenCalledTimes(2);
    stop();
    expect(on.pause.size + on.background.size + on.back.size).toBe(0);
  });

  it('a flush that fails does not throw out of the handler', () => {
    const { app, fire } = fakeApp();
    installLifecycle({ flush: () => Promise.reject(new Error('disk')), back: () => 'handled' }, { native: app });
    expect(() => fire('pause')).not.toThrow();
  });

  it('back exits only when the UI says so, and only after the saves land', async () => {
    const { app, fire } = fakeApp();
    let result: 'handled' | 'exit' = 'handled';
    const order: string[] = [];
    const flush = vi.fn(async () => {
      await tick();
      order.push('flushed');
    });
    app.exit = () => {
      order.push('exit');
      app.exits += 1;
    };
    installLifecycle({ flush, back: () => result }, { native: app });
    fire('back');
    await tick();
    await tick();
    expect(app.exits).toBe(0);
    result = 'exit';
    fire('back');
    await tick();
    await tick();
    expect(app.exits).toBe(1);
    expect(order).toEqual(['flushed', 'exit']);
  });
});

describe('native haptics', () => {
  it('maps tick, warn and harm to the plugin', async () => {
    const calls: string[] = [];
    const fake: HapticsLike = {
      impact: (o) => {
        calls.push(`impact:${o.style}`);
        return Promise.resolve();
      },
      notification: (o) => {
        calls.push(`notify:${o.type}`);
        return Promise.resolve();
      },
    };
    const h = nativeHaptics(() => Promise.resolve(fake));
    h.pulse('tick');
    h.pulse('warn');
    h.pulse('harm');
    await tick();
    expect(calls).toEqual(['impact:LIGHT', 'notify:WARNING', 'impact:HEAVY']);
  });

  it('a missing plugin is silent', async () => {
    const h = nativeHaptics(() => Promise.reject(new Error('none')));
    expect(() => h.pulse('harm')).not.toThrow();
    await tick();
  });
});
