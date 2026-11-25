/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import { CoreError, Run, dailySetup, initCoreSync, introLines, profile } from './index';
import type { Action, GameConfig, View } from './types';

beforeAll(() => {
  initCoreSync(readFileSync(new URL('./pkg/all_alone_core_bg.wasm', import.meta.url)));
});

const config: GameConfig = { seed: 42, difficulty: 'still', background: 'clerk', keepsake: null, daily: null, held_marks: [] };

function first(view: View, pred: (a: Action) => boolean): Action {
  const found = view.actions.find((a) => a.enabled && pred(a.action));
  if (!found) throw new Error('no such action');
  return found.action;
}

describe('core via wasm', () => {
  it('plays a few actions through the typed wrapper', () => {
    const run = Run.start({ ...config });
    let view = run.view();
    expect(view.phase).toBe('home');
    expect(view.day).toBe(1);
    expect(view.vitals.map((v) => v.id)).toEqual(['health', 'satiety', 'hydration', 'rest', 'nerve', 'warmth']);
    expect(view.objectives.length).toBeGreaterThan(0);
    expect(introLines().length).toBeGreaterThan(2);

    const out = run.dispatch(first(view, (a) => a.type === 'draw_water'));
    expect(out.journal.length).toBeGreaterThan(0);

    run.dispatch(first(run.view(), (a) => a.type === 'plan_trip' && a.place === 'cottages'));
    view = run.view();
    expect(view.phase).toBe('packing');
    expect(view.packing?.name).toBe('Towpath Cottages');
    run.dispatch({ type: 'pack', item: 'candle', qty: 1 });
    run.dispatch({ type: 'depart' });
    run.dispatch({ type: 'walk', hurry: false });
    view = run.view();
    expect(['site', 'event', 'encounter']).toContain(view.phase);

    // Invalid actions are typed errors, and change nothing.
    const before = run.save();
    expect(() => run.dispatch({ type: 'cast_off' })).toThrow(CoreError);
    try {
      run.dispatch({ type: 'cast_off' });
    } catch (e) {
      expect((e as CoreError).code).toBe('not_available');
    }
    expect(run.save()).toBe(before);

    // Saves round-trip; garbage is rejected.
    const again = Run.load(before);
    expect(again.save()).toBe(before);
    expect(() => Run.load(before.slice(0, 200))).toThrow(CoreError);
    run.free();
    again.free();
  });

  it('keeps a profile and a daily river', () => {
    const p = profile.create();
    const pv = profile.view(p);
    expect(pv.backgrounds.find((b) => b.id === 'clerk')?.unlocked).toBe(true);
    expect(pv.marks_total).toBeGreaterThanOrEqual(40);
    const { profile: p2, config: daily, counts } = profile.daily(p, '2026-11-25');
    expect(counts).toBe(true);
    expect(daily.seed).toBe(dailySetup('2026-11-25').seed);
    expect(profile.daily(p2, '2026-11-25').counts).toBe(false);
    expect(() => profile.check('{}')).toThrow(CoreError);
    expect(() => profile.runConfig(p, 1, 'still', 'nurse', null)).toThrow(CoreError);
  });
});
