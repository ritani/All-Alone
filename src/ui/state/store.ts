/**
 * The app store: owns the Run, the current View, the profile and the queue of
 * things to tell the player. Framework-free so it can be tested headless.
 *
 * Every change to the game goes through `act(action)`, which dispatches to the
 * core, refreshes the view, autosaves, and turns typed events into toasts and
 * haptics. The store never re-implements a rule; it only moves strings the
 * core produced to storage and back.
 */
import {
  CoreError,
  Run,
  introLines,
  profile as coreProfile,
  type Action,
  type DailyStatus,
  type Difficulty,
  type GameEvent,
  type Ledger,
  type Outcome,
  type ProfileView,
  type RunEndReport,
  type Unlock,
  type View,
} from '../../core';
import { noHaptics, type Haptics } from '../../platform/haptics';
import { KEYS, StorageError, memoryStore, parseUnread, type KeyValueStore } from '../../platform/storage';

export type Screen =
  | 'loading'
  | 'title'
  | 'setup'
  | 'intro'
  | 'game'
  | 'ended'
  | 'marks'
  | 'record'
  | 'unlocks'
  | 'howto'
  | 'fatal';

/** Upper bound for a rejected save kept aside (characters); real saves are well under 1 MiB. */
const MAX_KEPT_ASIDE = 1024 * 1024;

export type ToastTone = 'quiet' | 'notice' | 'danger';

export interface Toast {
  id: number;
  title: string;
  text: string | null;
  icon: string;
  tone: ToastTone;
}

export interface EndState {
  ledger: Ledger | null;
  report: RunEndReport | null;
  /** Ledger mark ids the profile did not hold before this run (from the report). */
  newMarks: string[];
}

export interface AppState {
  screen: Screen;
  profile: ProfileView | null;
  view: View | null;
  /** A valid, unfinished run is loaded and can be continued. */
  hasRun: boolean;
  /** Intro prose from the core, shown when a run begins. */
  intro: string[];
  /** For a daily river: whether this attempt counts for the day. */
  dailyCounts: boolean | null;
  /** How many of the newest journal entries arrived with the last action. */
  fresh: number;
  /** Increments on every action, so the view can key its fades. */
  seq: number;
  toasts: Toast[];
  /** Messages that stay until dismissed: rejected saves, storage trouble. */
  notices: string[];
  end: EndState | null;
  fatal: string | null;
  /** Codex entries found in this run and not yet opened. */
  codexUnread: string[];
}

export interface StoreDeps {
  storage: KeyValueStore;
  haptics?: Haptics;
  /** Loads the core (once). Tests pass a no-op after `initCoreSync`. */
  init?: () => Promise<void>;
  /** A fresh run seed. */
  seed?: () => number;
  /** The device's local date, "YYYY-MM-DD". */
  today?: () => string;
  /** How long a toast stays, in ms (0 keeps it until dismissed). */
  toastMs?: number;
}

export type Listener = (state: AppState) => void;

export function localDate(d: Date = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function randomSeed(): number {
  const buf = new Uint32Array(1);
  crypto.getRandomValues(buf);
  return buf[0] ?? 1;
}

function initial(): AppState {
  return {
    screen: 'loading',
    profile: null,
    view: null,
    hasRun: false,
    intro: [],
    dailyCounts: null,
    fresh: 0,
    seq: 0,
    toasts: [],
    notices: [],
    end: null,
    fatal: null,
    codexUnread: [],
  };
}

export class Store {
  private state: AppState = initial();
  private readonly listeners = new Set<Listener>();
  private storage: KeyValueStore;
  private readonly haptics: Haptics;
  private readonly deps: StoreDeps;
  private run: Run | null = null;
  private profileSave = '';
  /** Set once the current run's ledger has been folded into the profile. */
  private endApplied = false;
  /** Writes are queued so they land in order. */
  private writes: Promise<void> = Promise.resolve();
  private toastSeq = 0;
  /** The unlocks page was opened from the new-run page (back returns there). */
  private cameFromSetup = false;
  private readonly timers = new Map<number, ReturnType<typeof setTimeout>>();

  constructor(deps: StoreDeps) {
    this.deps = deps;
    this.storage = deps.storage;
    this.haptics = deps.haptics ?? noHaptics;
  }

  // ------------------------------------------------------------ subscription

  get(): AppState {
    return this.state;
  }

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  private set(patch: Partial<AppState>): void {
    this.state = { ...this.state, ...patch };
    for (const fn of this.listeners) fn(this.state);
  }

  /** Resolves when every queued write has landed (or failed and been reported). */
  flush(): Promise<void> {
    return this.writes;
  }

  // ------------------------------------------------------------ boot

  async boot(): Promise<void> {
    try {
      await (this.deps.init?.() ?? Promise.resolve());
    } catch (e) {
      this.fail(e);
      return;
    }
    try {
      await this.loadProfile();
      await this.loadRun();
    } catch (e) {
      if (this.isBroken(e)) return;
      throw e;
    }
    if (this.state.screen !== 'fatal') this.set({ screen: 'title' });
  }

  private async read(key: string): Promise<string | null> {
    try {
      return await this.storage.get(key);
    } catch (e) {
      if (e instanceof StorageError) {
        // The device refuses storage: play on in memory and say so once.
        this.storage = memoryStore();
        this.notice('This device will not keep saves. Play will not be remembered after you close the game.');
        return null;
      }
      throw e;
    }
  }

  private async loadProfile(): Promise<void> {
    const stored = await this.read(KEYS.profile);
    if (stored !== null) {
      try {
        this.profileSave = coreProfile.check(stored);
        if (this.profileSave !== stored) this.persist(KEYS.profile, this.profileSave);
      } catch (e) {
        if (this.isBroken(e)) throw e;
        // Never half-load. Keep the bad string aside rather than destroy it.
        this.keepAside(KEYS.profileRejected, stored);
        this.notice(
          `Your record could not be read (${reason(e)}). It has been kept aside, and a new record started.`,
        );
        this.profileSave = coreProfile.create();
        this.persist(KEYS.profile, this.profileSave);
      }
    } else {
      this.profileSave = coreProfile.create();
      this.persist(KEYS.profile, this.profileSave);
    }
    this.refreshProfile();
  }

  private async loadRun(): Promise<void> {
    const stored = await this.read(KEYS.run);
    if (stored === null) return;
    let run: Run;
    try {
      run = Run.load(stored);
    } catch (e) {
      if (this.isBroken(e)) throw e;
      this.keepAside(KEYS.runRejected, stored);
      this.remove(KEYS.run);
      this.notice(`The saved run could not be read (${reason(e)}). It has been kept aside. Begin again.`);
      return;
    }
    this.run = run;
    this.endApplied = false;
    if (run.ended) {
      // The game closed between the end of a run and its ledger being kept.
      this.finishRun();
      this.notice('Your last run had ended. Its ledger has been entered in your record.');
      this.dropRun();
      return;
    }
    this.set({ hasRun: true, view: run.view(), fresh: 0, codexUnread: await this.loadUnread() });
  }

  private async loadUnread(): Promise<string[]> {
    // UI-only state, but still untrusted: parsed, filtered and size-capped.
    return parseUnread(await this.read(KEYS.codexUnread));
  }

  /** An unread codex entry has been opened. */
  markRead(id: string): void {
    if (!this.state.codexUnread.includes(id)) return;
    const codexUnread = this.state.codexUnread.filter((x) => x !== id);
    this.set({ codexUnread });
    this.persist(KEYS.codexUnread, JSON.stringify(codexUnread));
  }

  // ------------------------------------------------------------ navigation

  go(screen: Exclude<Screen, 'loading' | 'fatal' | 'game' | 'ended' | 'intro'>): void {
    if (screen === 'unlocks') this.cameFromSetup = this.state.screen === 'setup';
    this.refreshProfile();
    this.set({ screen });
  }

  /**
   * The Android back button, once no sheet is open: go back a screen. Never
   * ends or drops a run (it is saved after every action; leaving the game
   * goes to the title, where Continue picks it up). 'exit' only from the title.
   */
  back(): 'handled' | 'exit' {
    switch (this.state.screen) {
      case 'title':
      case 'fatal':
        return 'exit';
      case 'loading':
        return 'handled';
      case 'game':
      case 'intro':
        this.go('title');
        return 'handled';
      case 'ended':
        this.leaveEnded();
        return 'handled';
      case 'unlocks':
        this.go(this.cameFromSetup ? 'setup' : 'title');
        return 'handled';
      default:
        this.go('title');
        return 'handled';
    }
  }

  continueRun(): void {
    if (!this.run) return;
    this.set({ screen: 'game', view: this.run.view(), fresh: 0 });
  }

  /** From the intro page into the game. */
  beginPlay(): void {
    if (!this.run) return;
    this.set({ screen: 'game' });
  }

  // ------------------------------------------------------------ starting runs

  startRun(opts: { difficulty: Difficulty; background: string; keepsake: string | null }): void {
    if (this.run) this.forfeitRun();
    this.guard(() => {
      const seed = (this.deps.seed ?? randomSeed)();
      const config = coreProfile.runConfig(this.profileSave, seed, opts.difficulty, opts.background, opts.keepsake);
      this.begin(Run.start(config), null);
    });
  }

  startDaily(): void {
    if (this.run) this.forfeitRun();
    this.guard(() => {
      const date = (this.deps.today ?? localDate)();
      const { profile: next, config, counts } = coreProfile.daily(this.profileSave, date);
      this.profileSave = next;
      this.persist(KEYS.profile, next);
      this.refreshProfile();
      this.begin(Run.start(config), counts);
    });
  }

  private begin(run: Run, dailyCounts: boolean | null): void {
    this.dropRun();
    this.run = run;
    this.endApplied = false;
    this.persist(KEYS.run, run.save());
    this.persist(KEYS.codexUnread, '[]');
    this.set({
      codexUnread: [],
      screen: 'intro',
      hasRun: true,
      view: run.view(),
      intro: introLines(),
      dailyCounts,
      fresh: 0,
      seq: this.state.seq + 1,
      end: null,
    });
  }

  private dropRun(): void {
    if (this.run) {
      try {
        this.run.free();
      } catch {
        // A trapped instance cannot be freed; it is dropped either way.
      }
    }
    this.run = null;
  }

  // ------------------------------------------------------------ playing

  /**
   * The single way the UI changes the game. Dispatches, refreshes the view,
   * autosaves (queued), and turns events into toasts and haptics.
   */
  act(action: Action): Promise<void> {
    const run = this.run;
    if (!run || this.state.screen !== 'game') return this.writes;
    let outcome: Outcome;
    try {
      outcome = run.dispatch(action);
    } catch (e) {
      if (this.isBroken(e)) return this.writes;
      // Not available now, or a bad quantity. The core changed nothing.
      this.toast({ title: reason(e), text: null, icon: 'close', tone: 'notice' });
      this.safeRefresh();
      return this.writes;
    }
    let view: View;
    try {
      view = run.view();
    } catch (e) {
      this.isBroken(e);
      return this.writes;
    }
    this.set({ view, fresh: outcome.journal.length, seq: this.state.seq + 1 });
    this.handleEvents(outcome.events, view);

    let ended: boolean;
    try {
      ended = run.ended;
    } catch (e) {
      this.isBroken(e);
      return this.writes;
    }
    if (ended) {
      this.finishRun();
    } else {
      this.guard(() => this.persist(KEYS.run, run.save()));
    }
    return this.writes;
  }

  private handleEvents(events: GameEvent[], view: View): void {
    // At the end of a run the ledger page lists new marks; no toasts over it.
    const ending = events.some((e) => e.type === 'run_ended');
    for (const ev of events) {
      switch (ev.type) {
        case 'mark': {
          if (!ev.new || ending) break;
          // The core names the mark as it is earned, hidden ones included.
          this.toast({ title: `Mark. ${ev.name}`, text: ev.text, icon: ev.icon, tone: 'quiet' });
          this.haptics.pulse('tick');
          break;
        }
        case 'codex': {
          const entry = view.codex.sections.flatMap((s) => s.entries).find((e) => e.id === ev.id);
          if (!this.state.codexUnread.includes(ev.id)) {
            const codexUnread = [...this.state.codexUnread, ev.id];
            this.set({ codexUnread });
            this.persist(KEYS.codexUnread, JSON.stringify(codexUnread));
          }
          if (entry) {
            this.toast({ title: `Into the codex. ${entry.title}`, text: null, icon: entry.icon, tone: 'quiet' });
          }
          break;
        }
        case 'skill_up': {
          const s = view.skills.find((k) => k.id === ev.skill);
          this.toast({
            title: `${s?.name ?? 'Skill'}. Level ${ev.level}.`,
            text: s?.effect ?? null,
            icon: s?.icon ?? 'item',
            tone: 'quiet',
          });
          this.haptics.pulse('tick');
          break;
        }
        case 'hurt':
        case 'wounded':
          this.haptics.pulse('harm');
          break;
        case 'night_visit':
          this.haptics.pulse(ev.breached ? 'harm' : 'warn');
          break;
        case 'run_ended':
          this.haptics.pulse('warn');
          break;
        default:
          break;
      }
    }
  }

  /** Fold the ended run into the profile, exactly once, then delete the run save. */
  private finishRun(): void {
    const run = this.run;
    if (!run || this.endApplied) return;
    this.endApplied = true;
    let ledger: Ledger | null;
    let report: RunEndReport | null = null;
    try {
      ledger = run.ledger();
      if (ledger) {
        try {
          const applied = coreProfile.applyRunEnd(this.profileSave, ledger);
          this.profileSave = applied.profile;
          report = applied.report;
          this.persist(KEYS.profile, this.profileSave);
        } catch (e) {
          // `already_applied`: the profile has this run. Anything else is reported.
          if (this.isBroken(e)) return;
          if (!(e instanceof CoreError && e.code === 'already_applied')) {
            this.notice(`The ledger could not be entered in your record (${reason(e)}).`);
          }
        }
      }
    } catch (e) {
      if (this.isBroken(e)) return;
      throw e;
    }
    // The profile write is queued first, so the run save is only removed after it.
    this.remove(KEYS.run);
    this.refreshProfile();
    this.set({
      hasRun: false,
      screen: this.state.screen === 'game' ? 'ended' : this.state.screen,
      end: { ledger, report, newMarks: report?.new_marks ?? [] },
    });
  }

  /** Whether today's daily river has already been attempted (without starting it). */
  dailyStatus(): DailyStatus | null {
    if (!this.profileSave) return null;
    try {
      return coreProfile.dailyStatus(this.profileSave, (this.deps.today ?? localDate)());
    } catch (e) {
      this.isBroken(e);
      return null;
    }
  }

  /** Leave the ended page for the title. */
  leaveEnded(): void {
    this.dropRun();
    this.set({ screen: 'title', view: null, end: null, hasRun: false, fresh: 0 });
  }

  /**
   * Give up the run in progress (from the new-run page, after confirmation).
   * The core ends it properly ("Gave up"), and its ledger is entered in the
   * profile once, so beginning again never silently discards a run.
   * Returns the ledger, or null if there was no run.
   */
  forfeitRun(): Ledger | null {
    const run = this.run;
    if (!run) return null;
    let ledger: Ledger | null = null;
    this.guard(() => {
      if (!run.ended) run.forfeit();
      this.finishRun();
      ledger = this.state.end?.ledger ?? null;
    });
    this.dropRun();
    this.set({ hasRun: false, view: null, end: null });
    return ledger;
  }

  // ------------------------------------------------------------ profile

  unlock(u: Unlock): void {
    this.guard(() => {
      try {
        this.profileSave = coreProfile.unlock(this.profileSave, u);
      } catch (e) {
        if (this.isBroken(e)) return;
        this.toast({ title: reason(e), text: null, icon: 'lock', tone: 'notice' });
        return;
      }
      this.persist(KEYS.profile, this.profileSave);
      this.refreshProfile();
      this.haptics.pulse('tick');
    });
  }

  private refreshProfile(): void {
    if (!this.profileSave) return;
    this.guard(() => this.set({ profile: coreProfile.view(this.profileSave) }));
  }

  // ------------------------------------------------------------ toasts and notices

  toast(t: Omit<Toast, 'id'>): void {
    const id = ++this.toastSeq;
    // Keep at most three; older ones give way.
    const toasts = [...this.state.toasts, { ...t, id }].slice(-3);
    this.set({ toasts });
    const ms = this.deps.toastMs ?? 4500;
    if (ms > 0) this.timers.set(id, setTimeout(() => this.dismissToast(id), ms));
  }

  dismissToast(id: number): void {
    const timer = this.timers.get(id);
    if (timer !== undefined) clearTimeout(timer);
    this.timers.delete(id);
    if (!this.state.toasts.some((t) => t.id === id)) return;
    this.set({ toasts: this.state.toasts.filter((t) => t.id !== id) });
  }

  notice(text: string): void {
    if (this.state.notices.includes(text)) return;
    this.set({ notices: [...this.state.notices, text] });
  }

  dismissNotice(index: number): void {
    this.set({ notices: this.state.notices.filter((_, i) => i !== index) });
  }

  // ------------------------------------------------------------ persistence

  private persist(key: string, value: string): void {
    this.writes = this.writes.then(
      () =>
        this.storage.set(key, value).catch((e: unknown) => {
          this.storageFailed(e);
        }),
    );
  }

  /**
   * Keep a rejected save for the player (never parsed again by the game).
   * Something far larger than any real save is not worth the room: dropped.
   */
  private keepAside(key: string, value: string): void {
    if (value.length <= MAX_KEPT_ASIDE) this.persist(key, value);
    else this.remove(key);
  }

  private remove(key: string): void {
    this.writes = this.writes.then(
      () =>
        this.storage.remove(key).catch((e: unknown) => {
          this.storageFailed(e);
        }),
    );
  }

  private storageFailed(e: unknown): void {
    const msg =
      e instanceof StorageError && e.code === 'quota'
        ? 'The device is out of room. The game could not be saved.'
        : 'The game could not be saved on this device.';
    this.notice(msg);
    this.toast({ title: msg, text: null, icon: 'close', tone: 'danger' });
  }

  // ------------------------------------------------------------ errors

  /** Run `f`; a broken core goes to the fatal screen, other errors become a toast. */
  private guard(f: () => void): void {
    try {
      f();
    } catch (e) {
      if (this.isBroken(e)) return;
      this.toast({ title: reason(e), text: null, icon: 'close', tone: 'notice' });
    }
  }

  private safeRefresh(): void {
    if (!this.run) return;
    try {
      this.set({ view: this.run.view(), fresh: 0 });
    } catch (e) {
      this.isBroken(e);
    }
  }

  /** True (and the fatal screen shown) when the core has trapped. */
  private isBroken(e: unknown): boolean {
    if (e instanceof CoreError && e.code === 'core_broken') {
      this.fail(e);
      return true;
    }
    return false;
  }

  private fail(e: unknown): void {
    const message = e instanceof Error ? e.message : String(e);
    this.set({ screen: 'fatal', fatal: message });
  }
}

function reason(e: unknown): string {
  if (e instanceof CoreError || e instanceof Error) return e.message;
  return String(e);
}
