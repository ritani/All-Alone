/**
 * Typed wrapper over the Rust game core (compiled to WebAssembly in ./pkg).
 *
 * Every rule lives in Rust. This module only moves JSON across the boundary
 * and gives it types (./types.ts, generated from the Rust serde types).
 * Call `initCore()` once before anything else.
 */
import init, {
  Game,
  core_version,
  daily_setup,
  initSync,
  intro,
  profile_apply_run_end,
  profile_check,
  profile_daily,
  profile_daily_status,
  profile_new,
  profile_run_config,
  profile_unlock,
  profile_view,
} from './pkg/all_alone_core.js';
import type {
  Action,
  DailySetup,
  DailyStatus,
  Difficulty,
  GameConfig,
  GameError,
  Ledger,
  Outcome,
  ProfileView,
  RunEndReport,
  Unlock,
  View,
} from './types';

export type * from './types';

/** An error from the core: `code` is stable, `message` is for people. */
export class CoreError extends Error {
  readonly code: string;
  constructor(code: string, message: string) {
    super(message);
    this.name = 'CoreError';
    this.code = code;
  }
}

let ready = false;
let broken = false;

function isGameError(v: unknown): v is GameError {
  return typeof v === 'object' && v !== null && typeof (v as GameError).code === 'string' && typeof (v as GameError).message === 'string';
}

/** Run a core call; turn thrown JSON errors into CoreError, traps into a fatal error. */
function call<T>(f: () => T): T {
  if (!ready) throw new CoreError('not_ready', 'initCore() has not finished.');
  if (broken) throw new CoreError('core_broken', 'The game core stopped after an internal error. Reload.');
  try {
    return f();
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      // A Rust panic aborts the instance; nothing in it can be trusted after this.
      broken = true;
      throw new CoreError('core_broken', 'The game core stopped after an internal error. Reload.');
    }
    if (e instanceof Error) {
      let parsed: unknown;
      try {
        parsed = JSON.parse(e.message);
      } catch {
        parsed = undefined;
      }
      if (isGameError(parsed)) throw new CoreError(parsed.code, parsed.message);
      throw new CoreError('error', e.message);
    }
    throw e;
  }
}

function json<T>(text: string): T {
  return JSON.parse(text) as T;
}

/** Load the wasm module (browser: fetched from the bundled asset URL). */
export async function initCore(source?: Parameters<typeof init>[0]): Promise<void> {
  if (ready) return;
  await (source === undefined ? init() : init(source));
  ready = true;
}

/** Load the wasm module from bytes you already have (tests, workers). */
export function initCoreSync(bytes: BufferSource | WebAssembly.Module): void {
  if (ready) return;
  initSync({ module: bytes });
  ready = true;
}

/** One run of the game. Free it with `free()` when replaced. */
export class Run {
  private constructor(private readonly game: Game) {}

  /** Start a new run. */
  static start(config: GameConfig): Run {
    return call(() => new Run(new Game(JSON.stringify(config))));
  }

  /** Resume from a save string; throws CoreError('bad_save') if it is not valid. */
  static load(save: string): Run {
    return call(() => new Run(Game.load(save)));
  }

  /** The only way to change the game. Invalid actions throw CoreError('not_available'). */
  dispatch(action: Action): Outcome {
    return call(() => json<Outcome>(this.game.dispatch(JSON.stringify(action))));
  }

  /** Give the run up: it ends with the cause "Gave up" and a ledger, like any other end. */
  forfeit(): Outcome {
    return call(() => json<Outcome>(this.game.forfeit()));
  }

  view(): View {
    return call(() => json<View>(this.game.view()));
  }

  /** An opaque string to store; hand it back to `Run.load`. */
  save(): string {
    return call(() => this.game.save());
  }

  ledger(): Ledger | null {
    const text = call(() => this.game.ledger());
    return text === '' ? null : json<Ledger>(text);
  }

  get ended(): boolean {
    return call(() => this.game.ended());
  }

  free(): void {
    this.game.free();
  }
}

/** The persistent profile, handled as an opaque save string. */
export const profile = {
  create: (): string => call(() => profile_new()),
  /** Validates and normalises a stored profile; throws CoreError('bad_save'). */
  check: (save: string): string => call(() => profile_check(save)),
  view: (save: string): ProfileView => call(() => json<ProfileView>(profile_view(save))),
  applyRunEnd: (save: string, ledger: Ledger): { profile: string; report: RunEndReport } =>
    call(() => json<{ profile: string; report: RunEndReport }>(profile_apply_run_end(save, JSON.stringify(ledger)))),
  unlock: (save: string, unlock: Unlock): string => call(() => profile_unlock(save, JSON.stringify(unlock))),
  runConfig: (save: string, seed: number, difficulty: Difficulty, background: string, keepsake: string | null): GameConfig =>
    call(() => json<GameConfig>(profile_run_config(save, seed >>> 0, difficulty, background, keepsake))),
  /** Whether the daily river for `date` has been attempted, without starting it. */
  dailyStatus: (save: string, date: string): DailyStatus =>
    call(() => json<DailyStatus>(profile_daily_status(save, date))),
  /** Starts today's daily river: records the attempt. `counts` is false on a repeat. */
  daily: (save: string, date: string): { profile: string; config: GameConfig; counts: boolean } =>
    call(() => json<{ profile: string; config: GameConfig; counts: boolean }>(profile_daily(save, date))),
};

export function dailySetup(date: string): DailySetup {
  return call(() => json<DailySetup>(daily_setup(date)));
}

export function introLines(): string[] {
  return call(() => json<string[]>(intro()));
}

export function coreVersion(): string {
  return call(() => core_version());
}
