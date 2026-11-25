/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import { Run, initCoreSync, profile, type View } from '../../core';
import { StorageError, KEYS, memoryStore, type KeyValueStore } from '../../platform/storage';
import { Store } from './store';

beforeAll(() => {
  initCoreSync(readFileSync(new URL('../../core/pkg/all_alone_core_bg.wasm', import.meta.url)));
});

function makeStore(storage: KeyValueStore, seed = 3): Store {
  return new Store({ storage, seed: () => seed, today: () => '2026-11-25', toastMs: 0 });
}

/** A small deterministic picker over enabled actions. */
function picker(seed: number) {
  let x = seed >>> 0 || 1;
  return (view: View) => {
    const enabled = view.actions.filter((a) => a.enabled);
    x = (Math.imul(x, 1103515245) + 12345) >>> 0;
    return enabled[x % Math.max(1, enabled.length)]?.action ?? null;
  };
}

async function playToEnd(store: Store, seed: number, limit = 20000): Promise<void> {
  const pick = picker(seed);
  for (let i = 0; i < limit; i++) {
    const s = store.get();
    if (s.screen !== 'game' || !s.view) return;
    const a = pick(s.view);
    if (!a) return;
    await store.act(a);
  }
}

describe('store', () => {
  it('boots a fresh profile and reaches the title', async () => {
    const storage = memoryStore();
    const store = makeStore(storage);
    await store.boot();
    await store.flush();
    expect(store.get().screen).toBe('title');
    expect(store.get().hasRun).toBe(false);
    expect(storage.data.get(KEYS.profile)).toBeTruthy();
    expect(store.get().profile?.backgrounds.some((b) => b.unlocked)).toBe(true);
  });

  it('autosaves after every action', async () => {
    const storage = memoryStore();
    const store = makeStore(storage);
    await store.boot();
    store.startRun({ difficulty: 'still', background: 'clerk', keepsake: null });
    expect(store.get().screen).toBe('intro');
    expect(store.get().intro.length).toBeGreaterThan(0);
    store.beginPlay();
    await store.flush();
    const first = storage.data.get(KEYS.run);
    expect(first).toBeTruthy();

    const view = store.get().view;
    const water = view?.actions.find((a) => a.action.type === 'draw_water' && a.enabled);
    expect(water).toBeTruthy();
    await store.act(water!.action);
    const second = storage.data.get(KEYS.run);
    expect(second).toBeTruthy();
    expect(second).not.toBe(first);
    expect(store.get().fresh).toBeGreaterThan(0);
    // The stored save is the current game.
    const reloaded = Run.load(second!);
    expect(reloaded.view().clock).toBe(store.get().view?.clock);
    reloaded.free();

    // A second store continues from it.
    const again = makeStore(storage);
    await again.boot();
    expect(again.get().hasRun).toBe(true);
    again.continueRun();
    expect(again.get().screen).toBe('game');
    expect(again.get().view?.clock).toBe(store.get().view?.clock);
  });

  it('reports an unavailable action without changing the game', async () => {
    const storage = memoryStore();
    const store = makeStore(storage);
    await store.boot();
    store.startRun({ difficulty: 'still', background: 'clerk', keepsake: null });
    store.beginPlay();
    await store.flush();
    const before = storage.data.get(KEYS.run);
    await store.act({ type: 'cast_off' });
    expect(store.get().toasts.length).toBe(1);
    expect(storage.data.get(KEYS.run)).toBe(before);
    expect(store.get().screen).toBe('game');
  });

  it('keeps rejected saves aside and never half-loads them', async () => {
    const storage = memoryStore({ [KEYS.run]: '{"format":"nope"}', [KEYS.profile]: 'garbage' });
    const store = makeStore(storage);
    await store.boot();
    await store.flush();
    const s = store.get();
    expect(s.screen).toBe('title');
    expect(s.hasRun).toBe(false);
    expect(s.notices.length).toBe(2);
    expect(storage.data.get(KEYS.runRejected)).toBe('{"format":"nope"}');
    expect(storage.data.get(KEYS.profileRejected)).toBe('garbage');
    expect(storage.data.has(KEYS.run)).toBe(false);
    // A fresh, valid profile replaced the bad one.
    expect(() => profile.check(storage.data.get(KEYS.profile)!)).not.toThrow();
  });

  it('applies the run end to the profile exactly once', async () => {
    const storage = memoryStore();
    const store = makeStore(storage, 11);
    await store.boot();
    store.startRun({ difficulty: 'dead_of_winter', background: 'clerk', keepsake: null });
    store.beginPlay();
    await playToEnd(store, 11);
    await store.flush();

    const s = store.get();
    expect(s.screen).toBe('ended');
    expect(s.view?.phase).toBe('ended');
    expect(s.end?.ledger).toBeTruthy();
    expect(s.end?.report).toBeTruthy();
    expect(storage.data.has(KEYS.run)).toBe(false);
    const stored = storage.data.get(KEYS.profile)!;
    expect(profile.view(stored).totals.runs).toBe(1);
    expect(s.profile?.totals.runs).toBe(1);

    // Further actions are ignored once the run is over.
    await store.act({ type: 'rise' });
    expect(profile.view(storage.data.get(KEYS.profile)!).totals.runs).toBe(1);

    store.leaveEnded();
    expect(store.get().screen).toBe('title');
    expect(store.get().hasRun).toBe(false);
  });

  it('enters an ended run left in storage once, then drops it', async () => {
    // Make an ended run save, as if the game closed before the ledger was kept.
    const storage = memoryStore();
    const setup = makeStore(storage, 11);
    await setup.boot();
    await setup.flush();
    const p = storage.data.get(KEYS.profile)!;
    const run = Run.start(profile.runConfig(p, 11, 'dead_of_winter', 'clerk', null));
    const pick = picker(11);
    for (let i = 0; i < 20000 && !run.ended; i++) {
      const a = pick(run.view());
      if (!a) break;
      run.dispatch(a);
    }
    expect(run.ended).toBe(true);
    const endedSave = run.save();
    run.free();

    storage.data.set(KEYS.run, endedSave);
    const a = makeStore(storage);
    await a.boot();
    await a.flush();
    expect(a.get().hasRun).toBe(false);
    expect(a.get().notices.length).toBe(1);
    expect(storage.data.has(KEYS.run)).toBe(false);
    expect(profile.view(storage.data.get(KEYS.profile)!).totals.runs).toBe(1);

    // The same save again (say, restored from a backup) is not counted twice.
    storage.data.set(KEYS.run, endedSave);
    const b = makeStore(storage);
    await b.boot();
    await b.flush();
    expect(profile.view(storage.data.get(KEYS.profile)!).totals.runs).toBe(1);
    expect(storage.data.has(KEYS.run)).toBe(false);
  });

  it('records the daily river attempt in the profile', async () => {
    const storage = memoryStore();
    const store = makeStore(storage);
    await store.boot();
    store.startDaily();
    expect(store.get().dailyCounts).toBe(true);
    expect(store.get().view?.run.daily).toBe('2026-11-25');
    store.startDaily();
    expect(store.get().dailyCounts).toBe(false);
  });

  it('surfaces storage failures', async () => {
    const base = memoryStore();
    const full: KeyValueStore = {
      get: (k) => base.get(k),
      remove: (k) => base.remove(k),
      set: () => Promise.reject(new StorageError('quota', 'full')),
    };
    const store = makeStore(full);
    await store.boot();
    await store.flush();
    expect(store.get().notices.some((n) => n.includes('out of room'))).toBe(true);
  });
});
