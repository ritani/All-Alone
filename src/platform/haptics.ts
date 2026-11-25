/**
 * Haptic feedback. Quiet by design: a tick for marks and skill gains, a
 * heavier pulse for harm. M5 adds a Capacitor Haptics implementation behind
 * the same interface.
 */
export type HapticKind = 'tick' | 'warn' | 'harm';

export interface Haptics {
  pulse(kind: HapticKind): void;
}

export const noHaptics: Haptics = { pulse: () => undefined };

const PATTERNS: Record<HapticKind, number | number[]> = {
  tick: 8,
  warn: [12, 60, 12],
  harm: 30,
};

/** `navigator.vibrate` where it exists (Android browsers); a no-op elsewhere. */
export function webHaptics(): Haptics {
  return {
    pulse(kind) {
      try {
        if (typeof navigator === 'undefined' || typeof navigator.vibrate !== 'function') return;
        // Browsers refuse (and log) vibration before the first tap; do not ask until then.
        const active = (navigator as Navigator & { userActivation?: { hasBeenActive: boolean } }).userActivation;
        if (active && !active.hasBeenActive) return;
        navigator.vibrate(PATTERNS[kind]);
      } catch {
        // Vibration is a nicety; never let it break play.
      }
    },
  };
}
