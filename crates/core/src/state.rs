//! The whole state of one run, and `new_game`.
//!
//! Everything here is plain data that round-trips through a save exactly.
//! Vitals and similar meters are held in thousandths (0..=100_000) so that
//! small per-minute changes accumulate without floating point.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::content::backgrounds;
use crate::content::{Band, Creature, Difficulty, Skill, Slot, Station, VitalKind, Weather};
use crate::meta::Ledger;
use crate::rng::Rng;
use crate::systems;
use crate::{GameError, Journal};

/// Thousandths per vital point.
pub const MILLI: i32 = 1000;
pub const VITAL_MAX: i32 = 100 * MILLI;
pub const DAY: u32 = 1440;
pub const DAWN: u32 = 6 * 60;
pub const DUSK: u32 = 19 * 60;
pub const START_MINUTE: u32 = 7 * 60;
/// The longest run the rules allow before the save is considered corrupt.
pub const MAX_DAY: u32 = 400;

pub type Inv = BTreeMap<String, u32>;

/// What the UI sends to start a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub seed: u32,
    pub difficulty: Difficulty,
    pub background: String,
    #[serde(default)]
    pub keepsake: Option<String>,
    /// The daily river's date ("YYYY-MM-DD"), if this is a daily run.
    #[serde(default)]
    pub daily: Option<String>,
    /// Marks the profile already holds, so the run only announces new ones.
    #[serde(default)]
    pub held_marks: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vitals {
    pub health: i32,
    pub satiety: i32,
    pub hydration: i32,
    pub rest: i32,
    pub nerve: i32,
    pub warmth: i32,
}

impl Vitals {
    pub fn get(&self, k: VitalKind) -> i32 {
        match k {
            VitalKind::Health => self.health,
            VitalKind::Satiety => self.satiety,
            VitalKind::Hydration => self.hydration,
            VitalKind::Rest => self.rest,
            VitalKind::Nerve => self.nerve,
            VitalKind::Warmth => self.warmth,
        }
    }
    pub fn slot(&mut self, k: VitalKind) -> &mut i32 {
        match k {
            VitalKind::Health => &mut self.health,
            VitalKind::Satiety => &mut self.satiety,
            VitalKind::Hydration => &mut self.hydration,
            VitalKind::Rest => &mut self.rest,
            VitalKind::Nerve => &mut self.nerve,
            VitalKind::Warmth => &mut self.warmth,
        }
    }
    /// Add (milli) and clamp to range.
    pub fn add(&mut self, k: VitalKind, milli: i32) {
        let v = self.slot(k);
        *v = v.saturating_add(milli).clamp(0, VITAL_MAX);
    }
    /// Whole points 0..=100 for display and rules.
    pub fn pts(&self, k: VitalKind) -> i32 {
        (self.get(k) + MILLI / 2) / MILLI
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum SearchState {
    None,
    Partial,
    Full,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foe {
    pub kind: Creature,
    pub health: i32,
    pub band: Band,
    /// Turns since it last moved.
    pub wait: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomState {
    pub entered: bool,
    pub unlocked: bool,
    /// Lit for the current visit.
    pub lit: bool,
    pub search: SearchState,
    /// Loot rolled but not yet found.
    pub hidden: Inv,
    /// Found and lying here, not taken (what did not fit stays behind).
    pub pile: Inv,
    /// Whether the occupants have been decided.
    pub rolled: bool,
    pub foes: Vec<Foe>,
    /// Day the room was last restocked with a wanderer.
    pub restocked: u32,
}

impl RoomState {
    pub fn new() -> Self {
        RoomState {
            entered: false,
            unlocked: false,
            lit: false,
            search: SearchState::None,
            hidden: Inv::new(),
            pile: Inv::new(),
            rolled: false,
            foes: Vec::new(),
            restocked: 0,
        }
    }
}

impl Default for RoomState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceState {
    pub known: bool,
    pub visited: bool,
    pub rooms: Vec<RoomState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trip {
    pub place: String,
    /// `None` while outside, at the place's door.
    pub room: Option<usize>,
    pub dog: bool,
    pub departed: u32,
    /// Noise made during this visit.
    pub site_noise: u32,
}

/// Where play continues after an interruption (an event or an encounter).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resume {
    Home,
    Site,
    /// Finish the walk out and arrive.
    Arrive,
    /// Finish the walk home and arrive.
    ArriveHome,
    /// Carry on with the night.
    Night,
    /// Back to the dawn summary's aftermath: the house.
    Rise,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    pub foes: Vec<Foe>,
    pub turn: u32,
    pub at_home: bool,
    /// Room the foes came from, if any (survivors return there).
    pub room: Option<usize>,
    /// You are held and cannot back away or flee this turn.
    pub gripped: bool,
    pub then: Resume,
    /// Lines from the latest exchange, newest last.
    pub log: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingEvent {
    pub id: String,
    pub then: Resume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// At the lock house.
    Home,
    /// Choosing what to carry before setting out.
    Packing,
    /// On the road, out or back.
    Travelling,
    /// At a place: outside it or in one of its rooms.
    Site,
    Encounter,
    Event,
    /// Trading with the widow at the weir.
    Barter,
    /// Choosing how to spend the night.
    Night,
    /// The dawn summary.
    Dawn,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum DogState {
    /// Not met yet.
    Unmet,
    /// Fed once; may stay.
    Following,
    Companion,
    Lost,
    Dead,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dog {
    pub state: DogState,
    /// 0..=100.
    pub health: i32,
    /// 0..=100; falls each day, filled by feeding.
    pub fed: i32,
    pub days_hungry: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Traps {
    pub snares: u32,
    /// Rabbits waiting in the snares.
    pub snared: u32,
    pub line: bool,
    pub baited: bool,
    /// Fish waiting on the line.
    pub hooked: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plot {
    pub crop: String,
    pub growth: u32,
    pub need: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct Offer {
    pub item: String,
    pub qty: u32,
    pub price: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Widow {
    pub met: bool,
    /// Her day pattern: she comes when (day + offset) % 3 == 0.
    pub offset: u32,
    pub stock: Vec<Offer>,
    pub credit: u32,
    /// Units sold to her, by category.
    pub sold: BTreeMap<String, u32>,
    /// Items events have added to her offers (offered every visit until bought).
    pub extra: Vec<String>,
    pub trades: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Plain,
    /// Something found or gained.
    Find,
    /// Something that hurt or threatened.
    Danger,
    /// Quiet, reflective: dawn, radio, notes.
    Quiet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub day: u32,
    /// "HH:MM".
    pub time: String,
    pub text: String,
    pub tone: Tone,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub nights: u32,
    pub kills: u32,
    pub shots: u32,
    pub noise_total: u32,
    pub metres: u32,
    pub quiet_streak: u32,
    pub best_quiet_streak: u32,
    pub quietest_day: u32,
    pub trades: u32,
    pub fish: u32,
    pub snared: u32,
    pub harvests: u32,
    pub treated: u32,
    pub breaches_survived: u32,
    pub night_return: bool,
    pub trips: u32,
    pub rooms_searched: u32,
    pub crafted: u32,
    pub built: u32,
    pub events: u32,
    pub dog_joined: bool,
    pub dog_lost: bool,
    pub saw_freeze: bool,
    pub walked_ice: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NightLog {
    pub bed: u32,
    pub watch: bool,
    /// 0: before the visit hour; 1: visit resolved.
    pub stage: u8,
    pub noise: u32,
    pub visitors: u32,
    pub damage: u32,
    pub breached: bool,
    pub rest_before: i32,
    pub warmth_low: bool,
    pub event: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum EndKind {
    Death,
    Departure,
    FreezeOut,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ending {
    pub kind: EndKind,
    pub cause: String,
    pub text: String,
}

/// The dawn summary shown after a night at home.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct DawnSummary {
    pub night: u32,
    pub day: u32,
    /// The first line: "Night 6. Noise 14. Nothing came."
    pub headline: String,
    pub lines: Vec<String>,
    pub noise: u32,
    pub visitors: u32,
    pub breached: bool,
    pub weather: Weather,
    pub forecast: Option<Weather>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameState {
    pub seed: u32,
    pub rng: Rng,
    pub difficulty: Difficulty,
    pub background: String,
    pub keepsake: Option<String>,
    pub daily: Option<String>,
    pub held_marks: BTreeSet<String>,

    pub minute: u32,
    pub phase: Phase,
    pub vitals: Vitals,
    /// 0..=3.
    pub bleeding: u8,
    /// Thousandths, 0..=100_000. Above zero means the wound is infected.
    pub infection: i32,
    pub antiseptic_until: u32,
    pub sick_until: u32,
    /// What last hurt you, for the cause of death.
    pub harm: String,

    /// Today's noise, 0..=100.
    pub noise: u32,
    pub weather: Weather,
    pub tomorrow: Weather,
    /// The radio's guess at tomorrow, if you listened today.
    pub forecast: Option<Weather>,
    /// Today's shift to the seasonal temperature, degrees.
    pub temp_offset: i32,
    pub freeze_day: u32,
    pub frozen: bool,
    /// The freeze date has been heard on the radio.
    pub freeze_known: bool,

    pub store: Inv,
    pub pack: Inv,
    pub worn: BTreeMap<Slot, String>,
    /// Uses spent on the current unit of a wearing item.
    pub wear: BTreeMap<String, u32>,
    pub stations: BTreeMap<Station, u8>,
    pub barricade: u32,
    pub fire_until: u32,
    pub skills: BTreeMap<Skill, u32>,

    pub places: BTreeMap<String, PlaceState>,
    pub trip: Option<Trip>,
    /// Destination while packing or travelling.
    pub dest: Option<String>,
    /// Travelling toward home (true) or away (false).
    pub homeward: bool,
    pub encounter: Option<Encounter>,
    pub event: Option<PendingEvent>,
    pub scheduled: Vec<(String, u32)>,
    pub seen_events: BTreeSet<String>,
    pub night: Option<NightLog>,
    /// Day whose small-hours visit has been resolved.
    pub visit_day: u32,
    pub house_breached: bool,
    pub dawn: Option<DawnSummary>,

    pub dog: Dog,
    pub traps: Traps,
    pub garden: Vec<Plot>,
    /// Plots were watered (or rained on) today.
    pub tended: bool,
    pub widow: Widow,
    pub radio_day: u32,

    pub codex: BTreeSet<String>,
    pub flags: BTreeSet<String>,
    pub ferry: BTreeSet<String>,

    pub journal: Vec<Entry>,
    pub stats: Stats,
    pub marks: BTreeSet<String>,
    pub ending: Option<Ending>,
    pub ledger: Option<Ledger>,
}

pub const JOURNAL_CAP: usize = 400;

impl GameState {
    pub fn day(&self) -> u32 {
        self.minute / DAY + 1
    }
    pub fn time_of_day(&self) -> u32 {
        self.minute % DAY
    }
    pub fn is_dark(&self) -> bool {
        let t = self.time_of_day();
        !(DAWN..DUSK).contains(&t)
    }
    pub fn level(&self, s: Station) -> u8 {
        self.stations.get(&s).copied().unwrap_or(0)
    }
    pub fn skill(&self, s: Skill) -> u8 {
        systems::skills::level(self.skills.get(&s).copied().unwrap_or(0))
    }
    pub fn at_home(&self) -> bool {
        self.trip.is_none()
    }
    pub fn ended(&self) -> bool {
        self.phase == Phase::Ended
    }
    /// What can be used right now: the pack when out, the store at home.
    pub fn inv(&self) -> &Inv {
        if self.at_home() { &self.store } else { &self.pack }
    }
    pub fn inv_mut(&mut self) -> &mut Inv {
        if self.at_home() {
            &mut self.store
        } else {
            &mut self.pack
        }
    }
    pub fn count(&self, item: &str) -> u32 {
        self.inv().get(item).copied().unwrap_or(0)
    }
    pub fn has_flag(&self, f: &str) -> bool {
        self.flags.contains(f)
    }
    pub fn place(&self, id: &str) -> Option<&PlaceState> {
        self.places.get(id)
    }
}

pub fn inv_add(inv: &mut Inv, item: &str, n: u32) {
    if n == 0 {
        return;
    }
    let e = inv.entry(item.to_string()).or_insert(0);
    *e = e.saturating_add(n).min(9_999);
}

/// Remove up to `n`; returns how many were removed.
pub fn inv_take(inv: &mut Inv, item: &str, n: u32) -> u32 {
    let Some(have) = inv.get_mut(item) else {
        return 0;
    };
    let took = n.min(*have);
    *have -= took;
    if *have == 0 {
        inv.remove(item);
    }
    took
}

pub fn inv_count(inv: &Inv, item: &str) -> u32 {
    inv.get(item).copied().unwrap_or(0)
}

/// Start a new run. Fails if the background or keepsake is unknown.
pub fn new_game(config: &GameConfig) -> Result<GameState, GameError> {
    let bg = backgrounds::get(&config.background).ok_or_else(|| GameError::new("bad_config", "Unknown background."))?;
    let keepsake = match &config.keepsake {
        Some(k) => Some(backgrounds::keepsake(k).ok_or_else(|| GameError::new("bad_config", "Unknown keepsake."))?),
        None => None,
    };
    if let Some(date) = &config.daily
        && crate::meta::parse_date(date).is_none()
    {
        return Err(GameError::new("bad_config", "The daily date must be YYYY-MM-DD."));
    }
    let mut held = BTreeSet::new();
    for m in &config.held_marks {
        if crate::content::marks::get(m).is_none() {
            return Err(GameError::new("bad_config", "Unknown mark in held_marks."));
        }
        held.insert(m.clone());
    }

    let mut rng = Rng::from_seed(config.seed);
    let freeze_day = config.difficulty.freeze_day() + rng.below(3);
    let offset = rng.below(3);

    let mut places = BTreeMap::new();
    for p in crate::content::places::PLACES {
        places.insert(
            p.id.to_string(),
            PlaceState {
                known: p.known_at_start,
                visited: false,
                rooms: p.rooms.iter().map(|_| RoomState::new()).collect(),
            },
        );
    }

    let mut store = Inv::new();
    for (id, n) in backgrounds::START_KIT.iter().chain(bg.kit) {
        inv_add(&mut store, id, *n);
    }
    let mut skills = BTreeMap::new();
    for s in Skill::ALL {
        skills.insert(*s, 0);
    }
    skills.insert(bg.skill, systems::skills::xp_for(bg.skill_level));

    let mut stations = BTreeMap::new();
    for s in Station::ALL {
        stations.insert(*s, 0u8);
    }

    let mut s = GameState {
        seed: config.seed,
        rng,
        difficulty: config.difficulty,
        background: bg.id.to_string(),
        keepsake: keepsake.map(|k| k.id.to_string()),
        daily: config.daily.clone(),
        held_marks: held,
        minute: START_MINUTE,
        phase: Phase::Home,
        vitals: Vitals {
            health: VITAL_MAX,
            satiety: 72 * MILLI,
            hydration: 70 * MILLI,
            rest: 80 * MILLI,
            nerve: 55 * MILLI,
            warmth: 85 * MILLI,
        },
        bleeding: 0,
        infection: 0,
        antiseptic_until: 0,
        sick_until: 0,
        harm: "exhaustion".into(),
        noise: 0,
        weather: Weather::Overcast,
        tomorrow: Weather::Overcast,
        forecast: None,
        temp_offset: 0,
        freeze_day,
        frozen: false,
        freeze_known: false,
        store,
        pack: Inv::new(),
        worn: BTreeMap::new(),
        wear: BTreeMap::new(),
        stations,
        barricade: crate::content::stations::barricade_max(0),
        fire_until: 0,
        skills,
        places,
        trip: None,
        dest: None,
        homeward: false,
        encounter: None,
        event: None,
        scheduled: Vec::new(),
        seen_events: BTreeSet::new(),
        night: None,
        visit_day: 0,
        house_breached: false,
        dawn: None,
        dog: Dog {
            state: DogState::Unmet,
            health: 100,
            fed: 60,
            days_hungry: 0,
        },
        traps: Traps {
            snares: 0,
            snared: 0,
            line: false,
            baited: false,
            hooked: 0,
        },
        garden: Vec::new(),
        tended: false,
        widow: Widow {
            met: false,
            offset,
            stock: Vec::new(),
            credit: 0,
            sold: BTreeMap::new(),
            extra: Vec::new(),
            trades: 0,
        },
        radio_day: 0,
        codex: BTreeSet::new(),
        flags: BTreeSet::new(),
        ferry: BTreeSet::new(),
        journal: Vec::new(),
        stats: Stats {
            quietest_day: 100,
            ..Stats::default()
        },
        marks: BTreeSet::new(),
        ending: None,
        ledger: None,
    };

    // The first day's weather, and tomorrow's.
    s.weather = systems::weather::roll(&mut s.rng, 1, Weather::Overcast);
    s.tomorrow = systems::weather::roll(&mut s.rng, 2, s.weather);
    s.temp_offset = s.rng.range(-2, 2);

    let mut j = Journal::default();
    if let Some(k) = keepsake {
        for (id, n) in k.items {
            inv_add(&mut s.store, id, *n);
        }
        for p in k.reveals {
            if let Some(ps) = s.places.get_mut(*p) {
                ps.known = true;
            }
        }
    }
    // Put on any clothing you start with.
    let clothes: Vec<String> = s
        .store
        .keys()
        .filter(|id| {
            matches!(
                crate::content::items::get(id).map(|d| d.kind),
                Some(crate::content::items::ItemKind::Clothing { .. })
            )
        })
        .cloned()
        .collect();
    for c in clothes {
        systems::inventory::wear(&mut s, &c, &mut j);
    }
    j.journal.clear();
    systems::journal::write(
        &mut s,
        &mut j,
        Tone::Quiet,
        "You wake in the lock house. The town has been silent for three days. On the kitchen table, the note Mr Orme left that night.",
    );
    systems::codex::find(&mut s, &mut j, "note_orme");
    let _ = j;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn config(seed: u32) -> GameConfig {
        GameConfig {
            seed,
            difficulty: Difficulty::Still,
            background: "clerk".into(),
            keepsake: None,
            daily: None,
            held_marks: vec![],
        }
    }

    #[test]
    fn new_game_basics() {
        let s = new_game(&config(1)).unwrap();
        assert_eq!(s.day(), 1);
        assert_eq!(s.time_of_day(), START_MINUTE);
        assert_eq!(s.phase, Phase::Home);
        assert!(s.count("hammer") == 1);
        assert!(s.codex.contains("note_orme"));
        assert!(s.places["weir"].known && !s.places["wharf"].known);
        assert!((45..=47).contains(&s.freeze_day));
    }

    #[test]
    fn new_game_rejects_bad_config() {
        let mut c = config(1);
        c.background = "astronaut".into();
        assert!(new_game(&c).is_err());
        let mut c = config(1);
        c.keepsake = Some("nope".into());
        assert!(new_game(&c).is_err());
        let mut c = config(1);
        c.daily = Some("2026-13-01".into());
        assert!(new_game(&c).is_err());
        let mut c = config(1);
        c.held_marks = vec!["nope".into()];
        assert!(new_game(&c).is_err());
    }

    #[test]
    fn keepsake_applies() {
        let mut c = config(3);
        c.keepsake = Some("valley_map".into());
        let s = new_game(&c).unwrap();
        assert!(s.places["boatyard"].known);
        let mut c = config(3);
        c.keepsake = Some("coat".into());
        let s = new_game(&c).unwrap();
        assert_eq!(s.worn.get(&Slot::Coat).map(String::as_str), Some("wool_coat"));
    }

    #[test]
    fn codex_kind_counts() {
        assert!(crate::content::codex::count(crate::content::codex::CodexKind::FieldGuide) >= 8);
    }
}
