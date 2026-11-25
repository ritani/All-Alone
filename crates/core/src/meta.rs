//! Across runs: marks, the run ledger and score, remnants and unlocks, the
//! persistent profile, and the daily river.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::GameError;
use crate::Journal;
use crate::action::GameEvent;
use crate::content::codex::{self, CodexKind};
use crate::content::marks::{self, Earn, MarkDef};
use crate::content::{Difficulty, Skill, Station, backgrounds, items, places};
use crate::rng::{hash_str, mix};
use crate::state::{DogState, EndKind, GameConfig, GameState};

// ---------------------------------------------------------------- Marks

fn kind_full(s: &GameState, kind: CodexKind) -> bool {
    codex::CODEX
        .iter()
        .filter(|c| c.kind == kind)
        .all(|c| s.codex.contains(c.id))
}

fn won(s: &GameState) -> bool {
    s.ending.as_ref().is_some_and(|e| e.kind == EndKind::Departure)
}

/// Whether a run-judged mark is earned in this state.
pub fn earned_in_run(s: &GameState, m: &MarkDef) -> bool {
    match m.earn {
        Earn::Nights(n) => s.stats.nights >= n,
        Earn::Win => won(s),
        Earn::WinOn(d) => won(s) && s.difficulty == d,
        Earn::WinNoShot => won(s) && s.stats.shots == 0,
        Earn::WinWithDog => won(s) && s.dog.state == DogState::Companion,
        Earn::WinNoKill => won(s) && s.stats.kills == 0,
        Earn::WinBefore(d) => won(s) && s.day() < d,
        Earn::Freeze => s.stats.saw_freeze,
        Earn::QuietDay(_) => s.has_flag("quiet_day"),
        Earn::QuietStreak(n) => s.stats.best_quiet_streak >= n,
        Earn::FieldGuideFull => kind_full(s, CodexKind::FieldGuide),
        Earn::NotesFull => kind_full(s, CodexKind::Note),
        Earn::PhotosFull => kind_full(s, CodexKind::Photograph),
        Earn::RadioFull => kind_full(s, CodexKind::Radio),
        Earn::DieOnDay(d) => s.ending.as_ref().is_some_and(|e| e.kind == EndKind::Death) && s.day() == d,
        Earn::Kills(n) => s.stats.kills >= n,
        Earn::AllPlaces => s.places.values().all(|p| p.known),
        Earn::AllStations(l) => Station::ALL.iter().all(|st| s.level(*st) >= l),
        Earn::AnyStation(l) => Station::ALL.iter().any(|st| s.level(*st) >= l),
        Earn::DogJoined => s.stats.dog_joined,
        Earn::DogLost => s.stats.dog_lost,
        Earn::AnySkill(l) => Skill::ALL.iter().any(|k| s.skill(*k) >= l),
        Earn::AllSkills(l) => Skill::ALL.iter().all(|k| s.skill(*k) >= l),
        Earn::Trades(n) => s.stats.trades >= n,
        Earn::Fish(n) => s.stats.fish >= n,
        Earn::Snared(n) => s.stats.snared >= n,
        Earn::Harvests(n) => s.stats.harvests >= n,
        Earn::Treated(n) => s.stats.treated >= n,
        Earn::WalkIce => s.stats.walked_ice,
        Earn::NightReturn => s.stats.night_return,
        Earn::BreachSurvived => s.stats.breaches_survived > 0,
        Earn::Flag(f) => s.has_flag(f),
        Earn::DailyDone => s.ending.is_some() && s.daily.is_some(),
        Earn::LifetimeKills(_) | Earn::DailyStreak(_) | Earn::LifetimeRuns(_) | Earn::LifetimeCodex(_) => false,
    }
}

/// Award any marks this state has earned. Called after every action.
pub fn award_marks(s: &mut GameState, j: &mut Journal) {
    for m in marks::MARKS {
        if m.earn.is_lifetime() || s.marks.contains(m.id) {
            continue;
        }
        if earned_in_run(s, m) {
            s.marks.insert(m.id.to_string());
            j.events.push(GameEvent::Mark {
                id: m.id.to_string(),
                new: !s.held_marks.contains(m.id),
            });
        }
    }
}

/// At the end of a run (the ending is already set).
pub fn award_run_marks(s: &mut GameState, j: &mut Journal) {
    award_marks(s, j);
}

// ---------------------------------------------------------------- Ledger and score

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct ScoreLine {
    pub label: String,
    pub points: u32,
}

/// The record of one ended run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub seed: u32,
    pub difficulty: Difficulty,
    pub background: String,
    pub daily: Option<String>,
    pub end: EndKind,
    pub cause: String,
    pub text: String,
    pub days: u32,
    pub nights: u32,
    pub places_found: u32,
    pub places_total: u32,
    pub noise_total: u32,
    pub kills: u32,
    pub shots: u32,
    pub metres: u32,
    pub parts: u32,
    pub stations: u32,
    pub dog: bool,
    pub codex: Vec<String>,
    pub marks: Vec<String>,
    pub lines: Vec<ScoreLine>,
    pub score: u32,
    pub remnants: u32,
}

pub const REMNANT_RATE: u32 = 20;

pub fn ledger(s: &GameState) -> Ledger {
    let (end, cause, text) = match &s.ending {
        Some(e) => (e.kind, e.cause.clone(), e.text.clone()),
        None => (EndKind::Death, "Unfinished".to_string(), String::new()),
    };
    let places_found = places::PLACES
        .iter()
        .filter(|p| !p.known_at_start && s.places.get(p.id).is_some_and(|x| x.known))
        .count() as u32;
    let places_total = places::PLACES.iter().filter(|p| !p.known_at_start).count() as u32;
    let stations: u32 = s.stations.values().map(|l| u32::from(*l)).sum();
    let dog = s.dog.state == DogState::Companion;
    let mut lines = Vec::new();
    let mut add = |label: &str, points: u32| {
        if points > 0 {
            lines.push(ScoreLine {
                label: label.to_string(),
                points,
            });
        }
    };
    add("Nights survived", s.stats.nights * 15);
    add("Places found", places_found * 10);
    add("Codex", s.codex.len() as u32 * 4);
    add("Stations built", stations * 5);
    add("The Stilled put down", s.stats.kills * 2);
    add("Ferry parts fitted", s.ferry.len() as u32 * 40);
    add("Quietest stretch", s.stats.best_quiet_streak * 5);
    match end {
        EndKind::Departure => {
            add("Cast off", 400);
            add("Days to spare", s.freeze_day.saturating_sub(s.day()) * 15);
            if dog {
                add("The dog aboard", 60);
            }
        }
        EndKind::FreezeOut => add("Walked out", 60),
        EndKind::Death => {}
    }
    if s.stats.saw_freeze {
        add("Saw the freeze", 120);
    }
    let base: u32 = lines.iter().map(|l| l.points).sum();
    let score = crate::systems::pct_u(base, s.difficulty.score_pct());
    Ledger {
        seed: s.seed,
        difficulty: s.difficulty,
        background: s.background.clone(),
        daily: s.daily.clone(),
        end,
        cause,
        text,
        days: s.day(),
        nights: s.stats.nights,
        places_found,
        places_total,
        noise_total: s.stats.noise_total,
        kills: s.stats.kills,
        shots: s.stats.shots,
        metres: s.stats.metres,
        parts: s.ferry.len() as u32,
        stations,
        dog,
        codex: s.codex.iter().cloned().collect(),
        marks: s.marks.iter().cloned().collect(),
        lines,
        score,
        remnants: score / REMNANT_RATE,
    }
}

// ---------------------------------------------------------------- Dates and the daily river

/// Days since 1970-01-01 for a "YYYY-MM-DD" date, or None if invalid.
pub fn parse_date(date: &str) -> Option<i64> {
    let b = date.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| -> Option<i64> {
        let s = date.get(r)?;
        if !s.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        s.parse().ok()
    };
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    if !(1970..=9999).contains(&y) || !(1..=12).contains(&m) {
        return None;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][(m - 1) as usize];
    if d < 1 || d > dim {
        return None;
    }
    // Days from civil (Howard Hinnant's algorithm).
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct DailySetup {
    pub date: String,
    pub seed: u32,
    pub difficulty: Difficulty,
    pub background: String,
}

/// The daily river for a date: one fixed seed and background, Still difficulty.
pub fn daily_setup(date: &str) -> Result<DailySetup, GameError> {
    let days = parse_date(date).ok_or_else(|| GameError::new("bad_date", "The date must be YYYY-MM-DD."))?;
    let seed = mix(hash_str("all-alone/daily-river"), days as u32);
    let bg = &backgrounds::BACKGROUNDS[(mix(seed, 7) as usize) % backgrounds::BACKGROUNDS.len()];
    Ok(DailySetup {
        date: date.to_string(),
        seed,
        difficulty: Difficulty::Still,
        background: bg.id.to_string(),
    })
}

// ---------------------------------------------------------------- Profile

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct Totals {
    pub runs: u32,
    pub deaths: u32,
    pub departures: u32,
    pub freeze_outs: u32,
    pub nights: u32,
    pub kills: u32,
    pub shots: u32,
    pub noise: u32,
    pub metres: u32,
    pub best_score: u32,
    pub longest: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct DailyResult {
    pub date: String,
    pub score: u32,
    pub end: EndKind,
    pub days: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct DailyRecord {
    /// Dates attempted, oldest first (capped).
    pub attempted: Vec<String>,
    /// The counted result per date (the first finished attempt).
    pub results: Vec<DailyResult>,
    pub best_streak: u32,
}

/// Persists across deaths. Stored separately from the run save.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Mark id -> the run number it was first earned in.
    pub marks: BTreeMap<String, u32>,
    pub backgrounds: BTreeSet<String>,
    pub keepsakes: BTreeSet<String>,
    pub codex: BTreeSet<String>,
    pub remnants: u32,
    pub remnants_earned: u32,
    pub totals: Totals,
    /// The ten best ledgers, best first.
    pub best: Vec<Ledger>,
    pub daily: DailyRecord,
    /// Fingerprint of the last ledger applied, so one run is not counted twice.
    pub last_ledger: u32,
}

pub const BEST_KEPT: usize = 10;
pub const DAILY_KEPT: usize = 120;

impl Default for Profile {
    fn default() -> Self {
        Self::new()
    }
}

impl Profile {
    pub fn new() -> Self {
        Profile {
            marks: BTreeMap::new(),
            backgrounds: backgrounds::BACKGROUNDS
                .iter()
                .filter(|b| b.open_at_start)
                .map(|b| b.id.to_string())
                .collect(),
            keepsakes: BTreeSet::new(),
            codex: BTreeSet::new(),
            remnants: 0,
            remnants_earned: 0,
            totals: Totals::default(),
            best: Vec::new(),
            daily: DailyRecord::default(),
            last_ledger: 0,
        }
    }

    /// Current run of consecutive daily rivers attempted, ending at the latest.
    pub fn daily_streak(&self) -> u32 {
        streak_of(&self.daily.attempted)
    }
}

fn streak_of(dates: &[String]) -> u32 {
    let mut days: Vec<i64> = dates.iter().filter_map(|d| parse_date(d)).collect();
    days.sort_unstable();
    days.dedup();
    let Some(&last) = days.last() else { return 0 };
    let mut n = 1;
    let mut expect = last - 1;
    for d in days.iter().rev().skip(1) {
        if *d == expect {
            n += 1;
            expect -= 1;
        } else {
            break;
        }
    }
    n
}

/// What applying a ledger to the profile changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct RunEndReport {
    pub new_marks: Vec<String>,
    pub remnants: u32,
    pub unlocked_backgrounds: Vec<String>,
    pub daily_counted: bool,
    /// Position in the best ten, 1-based, if it made it.
    pub rank: Option<u32>,
}

/// Validate a ledger before it touches the profile (it crosses the UI boundary).
pub fn validate_ledger(l: &Ledger) -> Result<(), GameError> {
    let bad = |m: &str| Err(GameError::new("bad_ledger", m));
    if backgrounds::get(&l.background).is_none() {
        return bad("Unknown background.");
    }
    if let Some(d) = &l.daily
        && parse_date(d).is_none()
    {
        return bad("Bad daily date.");
    }
    if l.days == 0 || l.days > crate::state::MAX_DAY || l.nights > l.days {
        return bad("Days out of range.");
    }
    if l.parts > 4
        || l.stations > 27
        || l.places_found > l.places_total
        || l.places_total as usize > places::PLACES.len()
    {
        return bad("Counts out of range.");
    }
    if l.codex.iter().any(|c| codex::get(c).is_none()) || l.codex.len() > codex::CODEX.len() {
        return bad("Unknown codex entry.");
    }
    if l.marks.iter().any(|m| marks::get(m).is_none()) || l.marks.len() > marks::MARKS.len() {
        return bad("Unknown mark.");
    }
    if l.score > 1_000_000 || l.remnants != l.score / REMNANT_RATE {
        return bad("Score out of range.");
    }
    if l.cause.len() > 200 || l.text.len() > 1000 || l.lines.len() > 20 || l.lines.iter().any(|x| x.label.len() > 60) {
        return bad("Text too long.");
    }
    Ok(())
}

fn fingerprint(l: &Ledger) -> u32 {
    hash_str(&serde_json::to_string(l).unwrap_or_default()) | 1
}

fn lifetime_earned(p: &Profile, m: &MarkDef) -> bool {
    match m.earn {
        Earn::LifetimeKills(n) => p.totals.kills >= n,
        Earn::LifetimeRuns(n) => p.totals.runs >= n,
        Earn::DailyStreak(n) => p.daily.best_streak >= n,
        Earn::LifetimeCodex(pct) => (p.codex.len() as u32) * 100 >= pct * codex::CODEX.len() as u32,
        _ => false,
    }
}

/// Fold an ended run into the profile.
pub fn apply_run_end(p: &mut Profile, l: &Ledger) -> Result<RunEndReport, GameError> {
    validate_ledger(l)?;
    let fp = fingerprint(l);
    if fp == p.last_ledger {
        return Err(GameError::new("already_applied", "This run is already in the profile."));
    }
    p.last_ledger = fp;
    let t = &mut p.totals;
    t.runs += 1;
    match l.end {
        EndKind::Death => t.deaths += 1,
        EndKind::Departure => t.departures += 1,
        EndKind::FreezeOut => t.freeze_outs += 1,
    }
    t.nights = t.nights.saturating_add(l.nights);
    t.kills = t.kills.saturating_add(l.kills);
    t.shots = t.shots.saturating_add(l.shots);
    t.noise = t.noise.saturating_add(l.noise_total);
    t.metres = t.metres.saturating_add(l.metres);
    t.best_score = t.best_score.max(l.score);
    t.longest = t.longest.max(l.days);
    let run = t.runs;
    for c in &l.codex {
        p.codex.insert(c.clone());
    }
    p.remnants = p.remnants.saturating_add(l.remnants);
    p.remnants_earned = p.remnants_earned.saturating_add(l.remnants);

    let mut daily_counted = false;
    if let Some(d) = &l.daily {
        if !p.daily.attempted.contains(d) {
            p.daily.attempted.push(d.clone());
        }
        if !p.daily.results.iter().any(|r| &r.date == d) {
            p.daily.results.push(DailyResult {
                date: d.clone(),
                score: l.score,
                end: l.end,
                days: l.days,
            });
            daily_counted = true;
        }
        trim_daily(p);
    }

    let mut new_marks = Vec::new();
    for m in &l.marks {
        if !p.marks.contains_key(m) {
            p.marks.insert(m.clone(), run);
            new_marks.push(m.clone());
        }
    }
    for m in marks::MARKS.iter().filter(|m| m.earn.is_lifetime()) {
        if !p.marks.contains_key(m.id) && lifetime_earned(p, m) {
            p.marks.insert(m.id.to_string(), run);
            new_marks.push(m.id.to_string());
        }
    }
    let mut unlocked = Vec::new();
    for b in backgrounds::BACKGROUNDS {
        if let Some(mid) = b.unlock_mark
            && p.marks.contains_key(mid)
            && p.backgrounds.insert(b.id.to_string())
        {
            unlocked.push(b.id.to_string());
        }
    }

    p.best.push(l.clone());
    p.best.sort_by(|a, b| b.score.cmp(&a.score).then(b.days.cmp(&a.days)));
    p.best.truncate(BEST_KEPT);
    let rank = p.best.iter().position(|x| x == l).map(|i| i as u32 + 1);

    Ok(RunEndReport {
        new_marks,
        remnants: l.remnants,
        unlocked_backgrounds: unlocked,
        daily_counted,
        rank,
    })
}

fn trim_daily(p: &mut Profile) {
    p.daily.best_streak = p.daily.best_streak.max(streak_of(&p.daily.attempted));
    if p.daily.attempted.len() > DAILY_KEPT {
        let drop = p.daily.attempted.len() - DAILY_KEPT;
        p.daily.attempted.drain(0..drop);
    }
    if p.daily.results.len() > DAILY_KEPT {
        let drop = p.daily.results.len() - DAILY_KEPT;
        p.daily.results.drain(0..drop);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(not(feature = "ts"), serde(deny_unknown_fields))]
pub enum Unlock {
    Background { id: String },
    Keepsake { id: String },
}

/// Spend remnants on an unlock.
pub fn unlock(p: &mut Profile, u: &Unlock) -> Result<(), GameError> {
    let (cost, set, id) = match u {
        Unlock::Background { id } => {
            let b = backgrounds::get(id).ok_or_else(|| GameError::new("unknown", "Unknown background."))?;
            (b.cost, &mut p.backgrounds, id)
        }
        Unlock::Keepsake { id } => {
            let k = backgrounds::keepsake(id).ok_or_else(|| GameError::new("unknown", "Unknown keepsake."))?;
            (k.cost, &mut p.keepsakes, id)
        }
    };
    if set.contains(id) {
        return Err(GameError::new("owned", "Already unlocked."));
    }
    if p.remnants < cost {
        return Err(GameError::new("cost", "Not enough remnants."));
    }
    p.remnants -= cost;
    set.insert(id.clone());
    Ok(())
}

/// A checked config for a normal run from this profile.
pub fn run_config(
    p: &Profile,
    seed: u32,
    difficulty: Difficulty,
    background: &str,
    keepsake: Option<&str>,
) -> Result<GameConfig, GameError> {
    if !p.backgrounds.contains(background) {
        return Err(GameError::new("locked", "That background is not unlocked."));
    }
    if let Some(k) = keepsake
        && !p.keepsakes.contains(k)
    {
        return Err(GameError::new("locked", "That keepsake is not unlocked."));
    }
    Ok(GameConfig {
        seed,
        difficulty,
        background: background.to_string(),
        keepsake: keepsake.map(str::to_string),
        daily: None,
        held_marks: p.marks.keys().cloned().collect(),
    })
}

/// Start the daily river for `date`: records the attempt (and the streak).
/// Returns the config and whether this attempt counts for the day's ledger.
pub fn daily_config(p: &mut Profile, date: &str) -> Result<(GameConfig, bool), GameError> {
    let d = daily_setup(date)?;
    let counts = !p.daily.attempted.iter().any(|x| x == date);
    if counts {
        p.daily.attempted.push(date.to_string());
        trim_daily(p);
    }
    Ok((
        GameConfig {
            seed: d.seed,
            difficulty: d.difficulty,
            background: d.background,
            keepsake: None,
            daily: Some(date.to_string()),
            held_marks: p.marks.keys().cloned().collect(),
        },
        counts,
    ))
}

/// Range and id checks for a loaded profile.
pub fn validate_profile(p: &Profile) -> Result<(), GameError> {
    let bad = |m: &str| Err(GameError::new("bad_profile", m));
    if p.marks.keys().any(|m| marks::get(m).is_none()) {
        return bad("Unknown mark.");
    }
    if p.backgrounds.iter().any(|b| backgrounds::get(b).is_none()) {
        return bad("Unknown background.");
    }
    if p.keepsakes.iter().any(|k| backgrounds::keepsake(k).is_none()) {
        return bad("Unknown keepsake.");
    }
    if p.codex.iter().any(|c| codex::get(c).is_none()) {
        return bad("Unknown codex entry.");
    }
    if !backgrounds::BACKGROUNDS
        .iter()
        .filter(|b| b.open_at_start)
        .all(|b| p.backgrounds.contains(b.id))
    {
        return bad("Missing the starting background.");
    }
    if p.best.len() > BEST_KEPT || p.daily.attempted.len() > DAILY_KEPT || p.daily.results.len() > DAILY_KEPT {
        return bad("Too many records.");
    }
    for l in &p.best {
        validate_ledger(l)?;
    }
    if p.daily.attempted.iter().any(|d| parse_date(d).is_none())
        || p.daily.results.iter().any(|r| parse_date(&r.date).is_none())
    {
        return bad("Bad daily date.");
    }
    if p.remnants > p.remnants_earned {
        return bad("Remnants out of range.");
    }
    if p.marks.values().any(|r| *r == 0 || *r > p.totals.runs.max(1)) {
        return bad("Mark run out of range.");
    }
    Ok(())
}

// ---------------------------------------------------------------- Profile view

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct MarkView {
    pub id: String,
    /// "???" while hidden and unearned.
    pub name: String,
    pub text: String,
    pub tier: u8,
    pub hidden: bool,
    pub earned: bool,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct BackgroundView {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub drawback: String,
    pub skill: Skill,
    pub skill_level: u8,
    pub kit: Vec<String>,
    pub unlocked: bool,
    pub cost: u32,
    /// The mark that unlocks it for free, by name.
    pub unlock_mark: Option<String>,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct KeepsakeView {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub unlocked: bool,
    pub cost: u32,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct DifficultyView {
    pub id: Difficulty,
    pub name: String,
    pub blurb: String,
    pub score_pct: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct ProfileView {
    pub remnants: u32,
    pub marks: Vec<MarkView>,
    pub marks_earned: u32,
    pub marks_total: u32,
    pub backgrounds: Vec<BackgroundView>,
    pub keepsakes: Vec<KeepsakeView>,
    pub difficulties: Vec<DifficultyView>,
    pub codex_found: u32,
    pub codex_total: u32,
    pub totals: Totals,
    pub best: Vec<Ledger>,
    pub daily_streak: u32,
    pub daily_best_streak: u32,
    pub daily_results: Vec<DailyResult>,
}

pub fn profile_view(p: &Profile) -> ProfileView {
    let marks: Vec<MarkView> = marks::MARKS
        .iter()
        .map(|m| {
            let earned = p.marks.contains_key(m.id);
            let veiled = m.hidden && !earned;
            MarkView {
                id: m.id.to_string(),
                name: if veiled { "???".into() } else { m.name.to_string() },
                text: if veiled {
                    "A hidden mark.".into()
                } else {
                    m.text.to_string()
                },
                tier: m.tier,
                hidden: m.hidden,
                earned,
                icon: m.icon.to_string(),
            }
        })
        .collect();
    let backgrounds = backgrounds::BACKGROUNDS
        .iter()
        .map(|b| BackgroundView {
            id: b.id.to_string(),
            name: b.name.to_string(),
            desc: b.desc.to_string(),
            drawback: b.drawback.to_string(),
            skill: b.skill,
            skill_level: b.skill_level,
            kit: b
                .kit
                .iter()
                .map(|(id, n)| crate::systems::journal::qty(items::name(id), *n))
                .collect(),
            unlocked: p.backgrounds.contains(b.id),
            cost: b.cost,
            unlock_mark: b.unlock_mark.and_then(marks::get).map(|m| m.name.to_string()),
            icon: b.icon.to_string(),
        })
        .collect();
    let keepsakes = backgrounds::KEEPSAKES
        .iter()
        .map(|k| KeepsakeView {
            id: k.id.to_string(),
            name: k.name.to_string(),
            desc: k.desc.to_string(),
            unlocked: p.keepsakes.contains(k.id),
            cost: k.cost,
            icon: k.icon.to_string(),
        })
        .collect();
    let difficulties = Difficulty::ALL
        .iter()
        .map(|d| DifficultyView {
            id: *d,
            name: d.name().to_string(),
            blurb: d.blurb().to_string(),
            score_pct: d.score_pct(),
        })
        .collect();
    ProfileView {
        remnants: p.remnants,
        marks_earned: p.marks.len() as u32,
        marks_total: marks::MARKS.len() as u32,
        marks,
        backgrounds,
        keepsakes,
        difficulties,
        codex_found: p.codex.len() as u32,
        codex_total: codex::CODEX.len() as u32,
        totals: p.totals.clone(),
        best: p.best.clone(),
        daily_streak: p.daily_streak(),
        daily_best_streak: p.daily.best_streak,
        daily_results: p.daily.results.iter().rev().take(30).cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(parse_date("1970-01-01"), Some(0));
        assert_eq!(parse_date("2000-03-01"), Some(11_017));
        assert!(parse_date("2024-02-29").is_some());
        assert_eq!(parse_date("2023-02-29"), None);
        assert_eq!(parse_date("2026-13-01"), None);
        assert_eq!(parse_date("2026-1-01"), None);
        assert_eq!(parse_date("+026-01-01"), None);
        assert_eq!(parse_date("２０２６-01-01"), None);
        assert_eq!(parse_date(""), None);
    }

    #[test]
    fn daily_seed_is_stable() {
        let a = daily_setup("2026-11-25").unwrap();
        let b = daily_setup("2026-11-25").unwrap();
        let c = daily_setup("2026-11-26").unwrap();
        assert_eq!(a, b);
        assert_ne!(a.seed, c.seed);
        // Pinned so a refactor cannot silently change every player's daily river.
        assert_eq!(
            a.seed,
            mix(
                hash_str("all-alone/daily-river"),
                parse_date("2026-11-25").unwrap() as u32
            )
        );
        assert!(daily_setup("tomorrow").is_err());
    }

    #[test]
    fn streaks() {
        let d = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(streak_of(&d(&[])), 0);
        assert_eq!(streak_of(&d(&["2026-01-01"])), 1);
        assert_eq!(streak_of(&d(&["2026-01-01", "2026-01-02", "2026-01-03"])), 3);
        assert_eq!(streak_of(&d(&["2026-01-01", "2026-01-03"])), 1);
        assert_eq!(streak_of(&d(&["2025-12-31", "2026-01-01"])), 2);
    }

    #[test]
    fn daily_attempt_counts_once() {
        let mut p = Profile::new();
        let (c1, counts1) = daily_config(&mut p, "2026-11-25").unwrap();
        let (c2, counts2) = daily_config(&mut p, "2026-11-25").unwrap();
        assert!(counts1 && !counts2);
        assert_eq!(c1, c2);
        assert_eq!(p.daily_streak(), 1);
        daily_config(&mut p, "2026-11-26").unwrap();
        assert_eq!(p.daily_streak(), 2);
    }

    #[test]
    fn unlocks_cost_remnants() {
        let mut p = Profile::new();
        let u = Unlock::Background { id: "nurse".into() };
        assert_eq!(unlock(&mut p, &u).unwrap_err().code, "cost");
        p.remnants = 100;
        p.remnants_earned = 100;
        unlock(&mut p, &u).unwrap();
        assert_eq!(p.remnants, 40);
        assert_eq!(unlock(&mut p, &u).unwrap_err().code, "owned");
        assert!(unlock(&mut p, &Unlock::Keepsake { id: "nope".into() }).is_err());
        assert!(run_config(&p, 1, Difficulty::Still, "nurse", None).is_ok());
        assert!(run_config(&p, 1, Difficulty::Still, "warden", None).is_err());
        assert!(run_config(&p, 1, Difficulty::Still, "clerk", Some("coat")).is_err());
    }
}
