/**
 * Persistent key-value storage for opaque save strings.
 *
 * The game only stores strings the core produced (run save, profile) and hands
 * them back to the core for validation on load. This module never parses them.
 *
 * The web implementation sits on `localStorage`. M5 adds a Capacitor
 * Preferences implementation behind the same interface.
 */

export type StorageErrorCode = 'quota' | 'unavailable' | 'failed';

export class StorageError extends Error {
  readonly code: StorageErrorCode;
  constructor(code: StorageErrorCode, message: string) {
    super(message);
    this.name = 'StorageError';
    this.code = code;
  }
}

export interface KeyValueStore {
  /** The stored string, or null when there is none. Throws StorageError if storage cannot be read. */
  get(key: string): Promise<string | null>;
  /** Throws StorageError('quota') when the device is out of room. */
  set(key: string, value: string): Promise<void>;
  remove(key: string): Promise<void>;
}

/** Storage keys. One run slot, one profile, and a place to keep rejected saves aside. */
export const KEYS = {
  run: 'all-alone.run',
  profile: 'all-alone.profile',
  runRejected: 'all-alone.run.rejected',
  profileRejected: 'all-alone.profile.rejected',
} as const;

function isQuotaError(e: unknown): boolean {
  if (!(e instanceof DOMException)) return false;
  return (
    e.name === 'QuotaExceededError' ||
    e.name === 'NS_ERROR_DOM_QUOTA_REACHED' ||
    e.code === 22 ||
    e.code === 1014
  );
}

/** A store over a Web Storage object (localStorage by default). */
export function webStore(backing?: Storage): KeyValueStore {
  const area = (): Storage => {
    if (backing) return backing;
    try {
      // Accessing localStorage itself can throw (privacy modes, sandboxed frames).
      const ls = window.localStorage;
      if (!ls) throw new Error('no localStorage');
      return ls;
    } catch {
      throw new StorageError('unavailable', 'This browser will not let the game keep a save.');
    }
  };
  return {
    get(key) {
      try {
        return Promise.resolve(area().getItem(key));
      } catch (e) {
        return Promise.reject(e instanceof StorageError ? e : new StorageError('unavailable', 'The save could not be read.'));
      }
    },
    set(key, value) {
      try {
        area().setItem(key, value);
        return Promise.resolve();
      } catch (e) {
        if (e instanceof StorageError) return Promise.reject(e);
        if (isQuotaError(e)) {
          return Promise.reject(new StorageError('quota', 'The device is out of room for saves.'));
        }
        return Promise.reject(new StorageError('failed', 'The game could not be saved.'));
      }
    },
    remove(key) {
      try {
        area().removeItem(key);
        return Promise.resolve();
      } catch (e) {
        return Promise.reject(e instanceof StorageError ? e : new StorageError('failed', 'The save could not be removed.'));
      }
    },
  };
}

/** An in-memory store: tests, and the fallback when the device refuses storage. */
export function memoryStore(initial: Record<string, string> = {}): KeyValueStore & { readonly data: Map<string, string> } {
  const data = new Map(Object.entries(initial));
  return {
    data,
    get: (key) => Promise.resolve(data.get(key) ?? null),
    set: (key, value) => {
      data.set(key, value);
      return Promise.resolve();
    },
    remove: (key) => {
      data.delete(key);
      return Promise.resolve();
    },
  };
}
