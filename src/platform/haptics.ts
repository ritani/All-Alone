/**
 * Haptic feedback. Quiet by design: a tick for marks and skill gains, a
 * heavier pulse for harm. The web uses `navigator.vibrate` where it exists;
 * the native apps use the Capacitor Haptics plugin (`nativeHaptics`).
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

/** The part of Capacitor Haptics the game uses (values match its enums). */
export interface HapticsLike {
  impact(options: { style: 'LIGHT' | 'MEDIUM' | 'HEAVY' }): Promise<void>;
  notification(options: { type: 'SUCCESS' | 'WARNING' | 'ERROR' }): Promise<void>;
}

/** Native haptics: a light tap, a warning, a heavy blow. Failures are ignored. */
export function nativeHaptics(load: () => Promise<HapticsLike>): Haptics {
  let pending: Promise<HapticsLike> | null = null;
  return {
    pulse(kind) {
      pending ??= load();
      pending
        .then((h) => {
          switch (kind) {
            case 'tick':
              return h.impact({ style: 'LIGHT' });
            case 'warn':
              return h.notification({ type: 'WARNING' });
            case 'harm':
              return h.impact({ style: 'HEAVY' });
          }
        })
        .catch(() => undefined);
    },
  };
}
