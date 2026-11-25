/**
 * Persistent key-value storage for opaque save strings.
 *
 * The game only stores strings the core produced (run save, profile) and hands
 * them back to the core for validation on load. This module never parses them.
 *
 * The web build sits on `localStorage`; the native apps sit on Capacitor
 * Preferences (UserDefaults on iOS, SharedPreferences on Android). The choice
 * is made once at start (`chooseStore`). Nothing is migrated between them:
 * they never share a device profile.
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
  /** Codex entries found in this run and not yet opened (UI state only). */
  codexUnread: 'all-alone.codex-unread',
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

/**
 * The part of Capacitor Preferences the game uses. Declared here so this
 * module (and its tests) never load Capacitor.
 */
export interface PreferencesLike {
  get(options: { key: string }): Promise<{ value: string | null }>;
  set(options: { key: string; value: string }): Promise<void>;
  remove(options: { key: string }): Promise<void>;
}

/** A store over Capacitor Preferences (native apps). */
export function preferencesStore(prefs: PreferencesLike): KeyValueStore {
  return {
    async get(key) {
      try {
        const { value } = await prefs.get({ key });
        return typeof value === 'string' ? value : null;
      } catch {
        throw new StorageError('unavailable', 'The save could not be read.');
      }
    },
    async set(key, value) {
      try {
        await prefs.set({ key, value });
      } catch {
        throw new StorageError('failed', 'The game could not be saved.');
      }
    },
    async remove(key) {
      try {
        await prefs.remove({ key });
      } catch {
        throw new StorageError('failed', 'The save could not be removed.');
      }
    },
  };
}

/**
 * A store whose backing is loaded on first use (the native plugin code is
 * only fetched inside the apps). A failed load reads as unavailable storage.
 */
export function lazyStore(load: () => Promise<KeyValueStore>): KeyValueStore {
  let pending: Promise<KeyValueStore> | null = null;
  const store = (): Promise<KeyValueStore> =>
    (pending ??= load().catch(() => {
      throw new StorageError('unavailable', 'Storage could not be opened.');
    }));
  return {
    get: async (key) => (await store()).get(key),
    set: async (key, value) => (await store()).set(key, value),
    remove: async (key) => (await store()).remove(key),
  };
}

/** Native apps keep saves in Preferences; the web keeps them in localStorage. */
export function chooseStore(opts: {
  native: boolean;
  preferences: () => Promise<PreferencesLike>;
  web?: () => KeyValueStore;
}): KeyValueStore {
  if (opts.native) return lazyStore(async () => preferencesStore(await opts.preferences()));
  return (opts.web ?? webStore)();
}

/** Upper bound for the UI-only unread list as stored (bytes of JSON). */
export const MAX_UNREAD_BYTES = 8 * 1024;
const UNREAD_ID = /^[a-z0-9_]{1,40}$/;

/**
 * Parse the stored unread-codex list. Storage is untrusted: anything that is
 * not a short JSON array of plain ids yields an empty or filtered list.
 */
export function parseUnread(text: string | null): string[] {
  if (text === null || text.length > MAX_UNREAD_BYTES) return [];
  let v: unknown;
  try {
    v = JSON.parse(text);
  } catch {
    return [];
  }
  if (!Array.isArray(v)) return [];
  const out: string[] = [];
  for (const x of v.slice(0, 200)) {
    if (typeof x === 'string' && UNREAD_ID.test(x) && !out.includes(x)) out.push(x);
  }
  return out;
}
