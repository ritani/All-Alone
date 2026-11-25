// Generated from the Rust core by `UPDATE_TYPES=1 cargo test --features ts ts_types`.
// Do not edit by hand.
/* eslint-disable */

export type GameConfig = { seed: number, difficulty: Difficulty, background: string, keepsake: string | null, 
/**
 * The daily river's date ("YYYY-MM-DD"), if this is a daily run.
 */
daily: string | null, 
/**
 * Marks the profile already holds, so the run only announces new ones.
 */
held_marks: Array<string>, };

export type GameError = { code: string, message: string, };

export type Action = { "type": "use", item: string, } | { "type": "build", station: Station, } | { "type": "craft", recipe: string, } | { "type": "repair_barricade" } | { "type": "feed_stove" } | { "type": "draw_water" } | { "type": "forage" } | { "type": "rest" } | { "type": "listen_radio" } | { "type": "go_to_bed" } | { "type": "plan_trip", place: string, } | { "type": "set_snares" } | { "type": "check_snares" } | { "type": "set_line" } | { "type": "check_line" } | { "type": "plant", seed: string, } | { "type": "tend_garden" } | { "type": "harvest" } | { "type": "feed_dog" } | { "type": "install_part", part: string, } | { "type": "cast_off" } | { "type": "walk_onto_ice" } | { "type": "pack", item: string, qty: number, } | { "type": "unpack", item: string, qty: number, } | { "type": "take_dog", take: boolean, } | { "type": "depart" } | { "type": "cancel_trip" } | { "type": "walk", hurry: boolean, } | { "type": "turn_back" } | { "type": "enter", quiet: boolean, } | { "type": "back" } | { "type": "leave" } | { "type": "search", careful: boolean, } | { "type": "light_up" } | { "type": "pick_lock" } | { "type": "force_lock" } | { "type": "use_key" } | { "type": "take", item: string, qty: number, } | { "type": "take_all" } | { "type": "drop", item: string, qty: number, } | { "type": "ambush" } | { "type": "trade" } | { "type": "strike" } | { "type": "shoot" } | { "type": "shove" } | { "type": "back_away" } | { "type": "hold_still" } | { "type": "flee" } | { "type": "throw", item: string, } | { "type": "choose", option: number, } | { "type": "sell", item: string, qty: number, } | { "type": "buy", item: string, } | { "type": "end_trade" } | { "type": "sleep", watch: boolean, } | { "type": "get_up" } | { "type": "rise" };

export type GameEvent = { "type": "found", item: string, qty: number, } | { "type": "lost", item: string, qty: number, } | { "type": "hurt", amount: number, } | { "type": "wounded", bleeding: boolean, infected: boolean, } | { "type": "sick" } | { "type": "vital_low", vital: VitalKind, } | { "type": "kill", kind: Creature | null, } | { "type": "shot" } | { "type": "noise", amount: number, total: number, } | { "type": "encounter_started" } | { "type": "encounter_over" } | { "type": "event_started", id: string, } | { "type": "codex", id: string, kind: CodexKind, } | { "type": "place_found", place: string, } | { "type": "built", station: Station, level: number, } | { "type": "crafted", recipe: string, } | { "type": "skill_up", skill: Skill, level: number, } | { "type": "night_visit", visitors: number, breached: boolean, } | { "type": "dawn", day: number, } | { "type": "dusk" } | { "type": "freeze" } | { "type": "dog_joined" } | { "type": "dog_lost", dead: boolean, } | { "type": "part_installed", part: string, } | { "type": "mark", id: string, new: boolean, } | { "type": "run_ended", kind: EndKind, };

export type Outcome = { journal: Array<Entry>, events: Array<GameEvent>, };

export type Entry = { day: number, 
/**
 * "HH:MM".
 */
time: string, text: string, tone: Tone, };

export type Tone = "plain" | "find" | "danger" | "quiet";

export type Phase = "home" | "packing" | "travelling" | "site" | "encounter" | "event" | "barter" | "night" | "dawn" | "ended";

export type EndKind = "death" | "departure" | "freeze_out";

export type DogState = "unmet" | "following" | "companion" | "lost" | "dead";

export type SearchState = "none" | "partial" | "full";

export type VitalKind = "health" | "satiety" | "hydration" | "rest" | "nerve" | "warmth";

export type Skill = "quiet_step" | "lockwork" | "mending" | "marksmanship" | "foraging" | "field_medicine";

export type Station = "stove" | "cot" | "workbench" | "barrel" | "garden" | "barricade" | "radio" | "medicine" | "store";

export type Weather = "clear" | "overcast" | "rain" | "fog" | "frost" | "snow";

export type Creature = "stander" | "listener" | "runner" | "drowned" | "hollow" | "small";

export type Band = "far" | "near" | "close";

export type Difficulty = "quiet" | "still" | "dead_of_winter";

export type CodexKind = "note" | "photograph" | "radio" | "field_guide";

export type DawnSummary = { night: number, day: number, 
/**
 * The first line: "Night 6. Noise 14. Nothing came."
 */
headline: string, lines: Array<string>, noise: number, visitors: number, breached: boolean, weather: Weather, forecast: Weather | null, };

export type Objective = { id: string, text: string, 
/**
 * Shown in the danger accent.
 */
urgent: boolean, icon: string, };

export type View = { phase: Phase, day: number, clock: string, minute_of_day: number, dark: boolean, 
/**
 * Minutes of daylight left, while it is light.
 */
daylight_left: number | null, season: SeasonView, weather: WeatherView, vitals: Array<VitalView>, conditions: Array<ConditionView>, noise: NoiseView, 
/**
 * Where you are, in words.
 */
location: string, actions: Array<ActionView>, objectives: Array<Objective>, inventory: InventoryView, stations: Array<StationView>, recipes: Array<RecipeView>, grounds: GroundsView, map: MapView, codex: CodexView, skills: Array<SkillView>, dog: DogView | null, ferry: FerryView, packing: PackingView | null, travel: TravelView | null, site: SiteView | null, encounter: EncounterView | null, event: EventView | null, barter: BarterView | null, dawn: DawnSummary | null, ending: EndingView | null, ledger: Ledger | null, 
/**
 * Most recent last; at most `JOURNAL_IN_VIEW` entries.
 */
journal: Array<Entry>, 
/**
 * Marks earned this run.
 */
marks: Array<string>, run: RunInfo, };

export type ActionView = { 
/**
 * Send this back to `dispatch` unchanged (quantity actions may change `qty` up to `max_qty`).
 */
action: Action, label: string, 
/**
 * "survive", "house", "station", "craft", "trip", "food", "ferry", "pack", "move", "room", "lock", "loot", "fight", "choice", "trade", "night".
 */
group: string, icon: string, minutes: number, noise: number, odds: number | null, enabled: boolean, reason: string | null, 
/**
 * Risky or irreversible: draw in the danger accent.
 */
danger: boolean, max_qty: number, };

export type VitalView = { id: VitalKind, name: string, value: number, icon: string, 
/**
 * "ok", "low" (< 35) or "critical" (< 15).
 */
level: string, };

export type ConditionView = { id: string, name: string, detail: string, icon: string, };

export type WeatherInfo = { id: Weather, name: string, icon: string, effect: string, };

export type WeatherView = { today: WeatherInfo, 
/**
 * The radio's guess at tomorrow (may be wrong).
 */
forecast: WeatherInfo | null, forecast_accuracy: number, };

export type SeasonView = { temperature: number, frozen: boolean, 
/**
 * Days until the freeze, once the radio has said.
 */
freeze_in: number | null, note: string, };

export type NoiseView = { value: number, label: string, 
/**
 * Percent chance something comes to the house tonight, at today's noise.
 */
visit_chance: number, };

export type ItemView = { id: string, name: string, desc: string, icon: string, qty: number, 
/**
 * Tenths of a kilogram, per unit.
 */
weight: number, 
/**
 * Total weight, e.g. "1.2 kg".
 */
weight_label: string, value: number, 
/**
 * "food", "medicine", "material", "weapon", "ammo", "tool", "light", "fuel", "bag", "clothing", "part", "key", "seed", "bait", "comfort", "trap".
 */
kind: string, 
/**
 * One line of numbers: "Satiety +34", "Damage 13-19".
 */
detail: string, uses_left: number | null, };

export type InventoryView = { store: Array<ItemView>, pack: Array<ItemView>, worn: Array<ItemView>, carried: number, capacity: number, carry_label: string, water: number, water_capacity: number, };

export type CostView = { item: string, name: string, icon: string, need: number, have: number, };

export type NextLevelView = { level: number, name: string, effect: string, cost: Array<CostView>, workbench: number, minutes: number, noise: number, can_build: boolean, reason: string | null, };

export type StationView = { id: Station, name: string, icon: string, level: number, level_name: string, effect: string, 
/**
 * Live state: "Burning until 21:30", "Holds 40 of 50".
 */
status: string | null, next: NextLevelView | null, };

export type RecipeView = { id: string, name: string, icon: string, inputs: Array<CostView>, outputs: Array<CostView>, station: Station | null, station_level: number, fire: boolean, minutes: number, can_craft: boolean, reason: string | null, };

export type MapNode = { 
/**
 * "home" or a place id.
 */
id: string, name: string, desc: string, icon: string, x: number, y: number, minutes: number, visited: boolean, rooms: number, rooms_searched: number, notes: Array<string>, };

export type MapView = { nodes: Array<MapNode>, paths: Array<[string, string]>, 
/**
 * Known places / all places.
 */
known: number, total: number, };

export type CodexEntryView = { id: string, title: string, text: string, };

export type CodexSection = { kind: CodexKind, name: string, icon: string, found: number, total: number, entries: Array<CodexEntryView>, };

export type CodexView = { sections: Array<CodexSection>, found: number, total: number, };

export type SkillView = { id: Skill, name: string, icon: string, level: number, xp: number, 
/**
 * Practice needed for the next level; null at level 5.
 */
next: number | null, effect: string, };

export type DogView = { state: DogState, health: number, fed: number, with_you: boolean, line: string, };

export type FerryPartView = { id: string, name: string, icon: string, installed: boolean, in_store: boolean, };

export type FerryView = { parts: Array<FerryPartView>, installed: number, ready: boolean, frozen: boolean, };

export type PlotView = { crop: string, name: string, growth: number, need: number, ready: boolean, };

export type GroundsView = { snares_set: number, snared: number, line_set: boolean, line_baited: boolean, hooked: number, plots: Array<PlotView>, plots_total: number, tended_today: boolean, fire_until: string | null, barricade: number, barricade_max: number, };

export type FoeView = { kind: Creature, name: string, band: Band, 
/**
 * Percent of its health left.
 */
health: number, icon: string, };

export type RoomSummary = { index: number, 
/**
 * Room name once seen (the next door is always seen); "Further in" otherwise.
 */
name: string, entered: boolean, here: boolean, dark: boolean, flooded: boolean, locked: boolean, search: SearchState, };

export type RoomView = { index: number, name: string, desc: string, dark: boolean, lit: boolean, flooded: boolean, search: SearchState, pile: Array<ItemView>, 
/**
 * Still ones standing here, not yet woken.
 */
dormant: Array<FoeView>, };

export type SiteView = { place: string, name: string, desc: string, icon: string, rooms: Array<RoomSummary>, room: RoomView | null, site_noise: number, widow_here: boolean, };

export type EncounterView = { foes: Array<FoeView>, turn: number, gripped: boolean, at_home: boolean, dark: boolean, 
/**
 * What happened in the last exchange.
 */
log: Array<string>, };

export type OptionView = { index: number, label: string, enabled: boolean, reason: string | null, odds: number | null, minutes: number, };

export type EventView = { id: string, title: string, text: string, options: Array<OptionView>, };

export type TravelView = { place: string, name: string, homeward: boolean, minutes: number, hurry_minutes: number, 
/**
 * You will arrive after dusk if you walk.
 */
arrive_dark: boolean, };

export type PackingView = { place: string, name: string, minutes: number, take_dog: boolean, has_light: boolean, has_weapon: boolean, meals: number, water: number, };

export type OfferView = { item: string, name: string, icon: string, qty: number, price: number, };

export type SellView = { item: string, name: string, icon: string, qty: number, price: number, };

export type BarterView = { credit: number, offers: Array<OfferView>, sell: Array<SellView>, trades: number, };

export type EndingView = { kind: EndKind, cause: string, text: string, };

export type RunInfo = { seed: number, difficulty: Difficulty, difficulty_name: string, background: string, background_name: string, keepsake: string | null, daily: string | null, nights: number, kills: number, };

export type Ledger = { seed: number, difficulty: Difficulty, background: string, daily: string | null, end: EndKind, cause: string, text: string, days: number, nights: number, places_found: number, places_total: number, noise_total: number, kills: number, shots: number, metres: number, parts: number, stations: number, dog: boolean, codex: Array<string>, marks: Array<string>, lines: Array<ScoreLine>, score: number, remnants: number, };

export type ScoreLine = { label: string, points: number, };

export type Totals = { runs: number, deaths: number, departures: number, freeze_outs: number, nights: number, kills: number, shots: number, noise: number, metres: number, best_score: number, longest: number, };

export type DailyResult = { date: string, score: number, end: EndKind, days: number, };

export type DailyRecord = { 
/**
 * Dates attempted, oldest first (capped).
 */
attempted: Array<string>, 
/**
 * The counted result per date (the first finished attempt).
 */
results: Array<DailyResult>, best_streak: number, };

export type Profile = { 
/**
 * Mark id -> the run number it was first earned in.
 */
marks: { [key in string]: number }, backgrounds: Array<string>, keepsakes: Array<string>, codex: Array<string>, remnants: number, remnants_earned: number, totals: Totals, 
/**
 * The ten best ledgers, best first.
 */
best: Array<Ledger>, daily: DailyRecord, 
/**
 * Fingerprint of the last ledger applied, so one run is not counted twice.
 */
last_ledger: number, };

export type RunEndReport = { new_marks: Array<string>, remnants: number, unlocked_backgrounds: Array<string>, daily_counted: boolean, 
/**
 * Position in the best ten, 1-based, if it made it.
 */
rank: number | null, };

export type Unlock = { "type": "background", id: string, } | { "type": "keepsake", id: string, };

export type DailySetup = { date: string, seed: number, difficulty: Difficulty, background: string, };

export type MarkView = { id: string, 
/**
 * "???" while hidden and unearned.
 */
name: string, text: string, tier: number, hidden: boolean, earned: boolean, icon: string, };

export type BackgroundView = { id: string, name: string, desc: string, drawback: string, skill: Skill, skill_level: number, kit: Array<string>, unlocked: boolean, cost: number, 
/**
 * The mark that unlocks it for free, by name.
 */
unlock_mark: string | null, icon: string, };

export type KeepsakeView = { id: string, name: string, desc: string, unlocked: boolean, cost: number, icon: string, };

export type DifficultyView = { id: Difficulty, name: string, blurb: string, score_pct: number, };

export type ProfileView = { remnants: number, marks: Array<MarkView>, marks_earned: number, marks_total: number, backgrounds: Array<BackgroundView>, keepsakes: Array<KeepsakeView>, difficulties: Array<DifficultyView>, codex_found: number, codex_total: number, totals: Totals, best: Array<Ledger>, daily_streak: number, daily_best_streak: number, daily_results: Array<DailyResult>, };
