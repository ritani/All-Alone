import { describe, expect, it } from 'vitest';
import { Rng, isRngState } from './rng';

describe('Rng', () => {
  it('is deterministic for the same seed', () => {
    const a = Rng.fromSeed(1234);
    const b = Rng.fromSeed(1234);
    const xs = Array.from({ length: 50 }, () => a.uint32());
    const ys = Array.from({ length: 50 }, () => b.uint32());
    expect(xs).toEqual(ys);
  });

  it('diverges for different seeds, including adjacent ones', () => {
    const a = Rng.fromSeed(1);
    const b = Rng.fromSeed(2);
    const xs = Array.from({ length: 10 }, () => a.uint32());
    const ys = Array.from({ length: 10 }, () => b.uint32());
    expect(xs).not.toEqual(ys);
  });

  it('accepts text seeds deterministically', () => {
    expect(Rng.fromSeed('calder').next()).toBe(Rng.fromSeed('calder').next());
    expect(Rng.fromSeed('calder').next()).not.toBe(Rng.fromSeed('reach').next());
  });

  it('pins a known sequence so the algorithm cannot drift silently', () => {
    const r = Rng.fromSeed(42);
    const seq = Array.from({ length: 4 }, () => r.uint32());
    expect(seq).toEqual([1028872839, 2516511472, 400437680, 853279530]);
  });

  it('round-trips its state through JSON', () => {
    const a = Rng.fromSeed(99);
    for (let i = 0; i < 17; i++) a.next();
    const saved = JSON.parse(JSON.stringify(a.getState())) as unknown;
    const b = Rng.fromState(saved);
    for (let i = 0; i < 100; i++) expect(b.uint32()).toBe(a.uint32());
  });

  it('getState returns a copy', () => {
    const r = Rng.fromSeed(5);
    const s = r.getState();
    r.next();
    expect(r.getState()).not.toEqual(s);
  });

  it('rejects invalid states', () => {
    for (const bad of [null, [], [1, 2, 3], [0, 0, 0, 0], [1, 2, 3, -1], [1, 2, 3, 2 ** 32], [1, 2, 3, 1.5], ['1', 2, 3, 4], { 0: 1 }]) {
      expect(isRngState(bad)).toBe(false);
      expect(() => Rng.fromState(bad)).toThrow(TypeError);
    }
    expect(isRngState([1, 2, 3, 4])).toBe(true);
  });

  it('rejects non-finite seeds', () => {
    expect(() => Rng.fromSeed(Number.NaN)).toThrow(RangeError);
    expect(() => Rng.fromSeed(Infinity)).toThrow(RangeError);
  });

  it('next() stays in [0, 1)', () => {
    const r = Rng.fromSeed(7);
    for (let i = 0; i < 10_000; i++) {
      const x = r.next();
      expect(x).toBeGreaterThanOrEqual(0);
      expect(x).toBeLessThan(1);
    }
  });

  it('int() covers the inclusive range roughly uniformly', () => {
    const r = Rng.fromSeed(8);
    const counts = new Map<number, number>();
    const n = 60_000;
    for (let i = 0; i < n; i++) {
      const v = r.int(-2, 3);
      counts.set(v, (counts.get(v) ?? 0) + 1);
    }
    expect([...counts.keys()].sort((x, y) => x - y)).toEqual([-2, -1, 0, 1, 2, 3]);
    for (const c of counts.values()) expect(Math.abs(c - n / 6)).toBeLessThan(n / 6 * 0.05);
    expect(r.int(4, 4)).toBe(4);
  });

  it('int() rejects bad ranges', () => {
    const r = Rng.fromSeed(1);
    expect(() => r.int(3, 2)).toThrow(RangeError);
    expect(() => r.int(0.5, 2)).toThrow(RangeError);
    expect(() => r.int(0, 2 ** 33)).toThrow(RangeError);
  });

  it('chance() respects the edges and the rate', () => {
    const r = Rng.fromSeed(9);
    for (let i = 0; i < 1000; i++) {
      expect(r.chance(0)).toBe(false);
      expect(r.chance(1)).toBe(true);
    }
    let hits = 0;
    for (let i = 0; i < 20_000; i++) if (r.chance(0.3)) hits++;
    expect(hits / 20_000).toBeCloseTo(0.3, 1);
    expect(() => r.chance(Number.NaN)).toThrow(RangeError);
  });

  it('pick() returns members and rejects empty arrays', () => {
    const r = Rng.fromSeed(10);
    const items = ['lamp', 'rope', 'tin'] as const;
    const seen = new Set<string>();
    for (let i = 0; i < 300; i++) seen.add(r.pick(items));
    expect(seen).toEqual(new Set(items));
    expect(() => r.pick([])).toThrow(RangeError);
  });

  it('weighted() follows the weights and never picks weight 0', () => {
    const r = Rng.fromSeed(11);
    const counts = { a: 0, b: 0, c: 0 };
    const n = 40_000;
    for (let i = 0; i < n; i++) {
      counts[r.weighted([['a', 1], ['b', 3], ['c', 0]] as const)]++;
    }
    expect(counts.c).toBe(0);
    expect(counts.a / n).toBeCloseTo(0.25, 1);
    expect(counts.b / n).toBeCloseTo(0.75, 1);
  });

  it('weighted() rejects bad weights', () => {
    const r = Rng.fromSeed(12);
    expect(() => r.weighted([])).toThrow(RangeError);
    expect(() => r.weighted([['a', 0]])).toThrow(RangeError);
    expect(() => r.weighted([['a', -1], ['b', 2]])).toThrow(RangeError);
    expect(() => r.weighted([['a', Number.NaN]])).toThrow(RangeError);
    expect(() => r.weighted([['a', Infinity]])).toThrow(RangeError);
  });
});
