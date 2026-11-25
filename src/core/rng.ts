/**
 * Seeded pseudo-random number generator (sfc32, seeded through splitmix32).
 *
 * The whole generator state is four unsigned 32-bit integers, so it can live in
 * GameState and round-trip through JSON. Same seed + same calls = same numbers.
 */

/** Serialisable generator state: four uint32 words. */
export type RngState = readonly [number, number, number, number];

const UINT32 = 0x1_0000_0000;

function isUint32(n: unknown): n is number {
  return typeof n === 'number' && Number.isInteger(n) && n >= 0 && n < UINT32;
}

/** True when `value` is a structurally valid RngState (for save validation). */
export function isRngState(value: unknown): value is RngState {
  return (
    Array.isArray(value) &&
    value.length === 4 &&
    value.every(isUint32) &&
    value.some((n) => n !== 0)
  );
}

/** FNV-1a hash, so a text seed ("calder") can be used as well as a number. */
function hashString(text: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

/** Expand one 32-bit seed into a well-mixed initial state. */
function seedState(seed: number): [number, number, number, number] {
  let x = seed >>> 0;
  const next = (): number => {
    x = (x + 0x9e3779b9) >>> 0;
    let z = x;
    z = Math.imul(z ^ (z >>> 16), 0x85ebca6b);
    z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35);
    return (z ^ (z >>> 16)) >>> 0;
  };
  const s: [number, number, number, number] = [next(), next(), next(), next()];
  if (s.every((n) => n === 0)) s[3] = 1;
  return s;
}

export class Rng {
  private a: number;
  private b: number;
  private c: number;
  private d: number;

  private constructor(state: RngState) {
    [this.a, this.b, this.c, this.d] = state;
  }

  /** New generator from a numeric or text seed. */
  static fromSeed(seed: number | string): Rng {
    if (typeof seed === 'number' && !Number.isFinite(seed)) {
      throw new RangeError('Rng seed must be finite');
    }
    const n = typeof seed === 'string' ? hashString(seed) : Math.trunc(seed);
    const rng = new Rng(seedState(n));
    // Discard the first outputs so nearby seeds diverge immediately.
    for (let i = 0; i < 12; i++) rng.uint32();
    return rng;
  }

  /** Resume a generator from a saved state. Throws on an invalid state. */
  static fromState(state: unknown): Rng {
    if (!isRngState(state)) throw new TypeError('Invalid RngState');
    return new Rng(state);
  }

  /** A copy of the current state, safe to store and serialise. */
  getState(): RngState {
    return [this.a, this.b, this.c, this.d];
  }

  /** Next unsigned 32-bit integer. */
  uint32(): number {
    const t = (((this.a + this.b) >>> 0) + this.d) >>> 0;
    this.d = (this.d + 1) >>> 0;
    this.a = (this.b ^ (this.b >>> 9)) >>> 0;
    this.b = (this.c + (this.c << 3)) >>> 0;
    this.c = ((this.c << 21) | (this.c >>> 11)) >>> 0;
    this.c = (this.c + t) >>> 0;
    return t;
  }

  /** Float in [0, 1). */
  next(): number {
    return this.uint32() / UINT32;
  }

  /** Integer in [min, max], both inclusive. */
  int(min: number, max: number): number {
    if (!Number.isSafeInteger(min) || !Number.isSafeInteger(max) || min > max) {
      throw new RangeError(`Rng.int: bad range [${min}, ${max}]`);
    }
    const span = max - min + 1;
    if (span > UINT32) throw new RangeError('Rng.int: range too large');
    // Rejection sampling keeps the distribution exactly uniform.
    const limit = UINT32 - (UINT32 % span);
    let x = this.uint32();
    while (x >= limit) x = this.uint32();
    return min + (x % span);
  }

  /** True with probability p (clamped to [0, 1]). Always consumes one draw. */
  chance(p: number): boolean {
    if (Number.isNaN(p)) throw new RangeError('Rng.chance: p is NaN');
    return this.next() < p;
  }

  /** One element of a non-empty array, uniformly. */
  pick<T>(items: readonly T[]): T {
    if (items.length === 0) throw new RangeError('Rng.pick: empty array');
    return items[this.int(0, items.length - 1)] as T;
  }

  /**
   * One value chosen in proportion to its weight.
   * Weights must be finite and >= 0, with a positive total.
   */
  weighted<T>(entries: readonly (readonly [value: T, weight: number])[]): T {
    let total = 0;
    for (const [, w] of entries) {
      if (!Number.isFinite(w) || w < 0) throw new RangeError('Rng.weighted: bad weight');
      total += w;
    }
    if (total <= 0) throw new RangeError('Rng.weighted: total weight must be positive');
    let roll = this.next() * total;
    let last: T | undefined;
    for (const [value, w] of entries) {
      if (w === 0) continue;
      if (roll < w) return value;
      roll -= w;
      last = value;
    }
    // Floating-point remainder: fall back to the last positive-weight entry.
    return last as T;
  }
}
