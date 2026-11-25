//! Saves: a versioned envelope around the run state or the profile.
//!
//! Saves are untrusted input. Loading checks the size cap first, parses
//! strictly (`deny_unknown_fields` everywhere), migrates old versions, then
//! validates every id and range. Anything wrong is an error; nothing is ever
//! partially loaded and nothing panics.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::GameError;
use crate::content::items::{self, ItemKind};
use crate::content::{Skill, Station, backgrounds, codex, creatures, events, marks, places, stations};
use crate::meta::{self, Profile};
use crate::state::{GameState, Inv, JOURNAL_CAP, MAX_DAY, Phase, VITAL_MAX};
use crate::systems::encounter::MAX_FOES;

pub const RUN_FORMAT: &str = "all-alone/run";
pub const PROFILE_FORMAT: &str = "all-alone/profile";
pub const RUN_VERSION: u32 = 1;
pub const PROFILE_VERSION: u32 = 1;
pub const MAX_RUN_BYTES: usize = 768 * 1024;
pub const MAX_PROFILE_BYTES: usize = 256 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    data: Value,
}

#[derive(Serialize)]
struct EnvelopeOut<'a, T: Serialize> {
    format: &'a str,
    version: u32,
    data: &'a T,
}

fn err(m: impl Into<String>) -> GameError {
    GameError::new("bad_save", m)
}

fn wrap<T: Serialize>(format: &str, version: u32, data: &T) -> String {
    // Serialising plain data with string keys cannot fail.
    serde_json::to_string(&EnvelopeOut { format, version, data }).unwrap_or_default()
}

fn open(text: &str, format: &str, max: usize) -> Result<(u32, Value), GameError> {
    if text.len() > max {
        return Err(err("The save is too large."));
    }
    let env: Envelope = serde_json::from_str(text).map_err(|e| err(format!("The save is not readable: {e}")))?;
    if env.format != format {
        return Err(err("This is not the right kind of save."));
    }
    Ok((env.version, env.data))
}

// ---------------------------------------------------------------- Run save

pub fn save_to_string(s: &GameState) -> String {
    wrap(RUN_FORMAT, RUN_VERSION, s)
}

/// Bring an older run save's data up to the current version.
pub fn migrate_run(version: u32, data: Value) -> Result<Value, GameError> {
    match version {
        RUN_VERSION => Ok(data),
        0 => Err(err("Unknown save version.")),
        v if v > RUN_VERSION => Err(err("This save is from a newer version of the game.")),
        _ => Err(err("Unknown save version.")),
    }
}

pub fn load_from_string(text: &str) -> Result<GameState, GameError> {
    let (version, data) = open(text, RUN_FORMAT, MAX_RUN_BYTES)?;
    let data = migrate_run(version, data)?;
    let s: GameState = serde_json::from_value(data).map_err(|e| err(format!("The save does not fit: {e}")))?;
    validate(&s)?;
    Ok(s)
}

// ---------------------------------------------------------------- Profile

pub fn profile_to_string(p: &Profile) -> String {
    wrap(PROFILE_FORMAT, PROFILE_VERSION, p)
}

pub fn migrate_profile(version: u32, data: Value) -> Result<Value, GameError> {
    match version {
        PROFILE_VERSION => Ok(data),
        v if v > PROFILE_VERSION => Err(err("This profile is from a newer version of the game.")),
        _ => Err(err("Unknown profile version.")),
    }
}

pub fn profile_from_string(text: &str) -> Result<Profile, GameError> {
    let (version, data) = open(text, PROFILE_FORMAT, MAX_PROFILE_BYTES)?;
    let data = migrate_profile(version, data)?;
    let p: Profile = serde_json::from_value(data).map_err(|e| err(format!("The profile does not fit: {e}")))?;
    meta::validate_profile(&p).map_err(|e| err(e.message))?;
    Ok(p)
}

// ---------------------------------------------------------------- Validation

fn check(ok: bool, what: &str) -> Result<(), GameError> {
    if ok {
        Ok(())
    } else {
        Err(err(format!("Out of range: {what}.")))
    }
}

fn check_inv(inv: &Inv, what: &str) -> Result<(), GameError> {
    check(inv.len() <= items::ITEMS.len(), what)?;
    for (id, n) in inv {
        check(items::get(id).is_some(), what)?;
        check((1..=9_999).contains(n), what)?;
    }
    Ok(())
}

fn short_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 40
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

const HARMS: &[&str] = &[
    "bleeding",
    "infection",
    "cold",
    "hunger",
    "thirst",
    "exhaustion",
    "sickness",
    "fall",
    "stander",
    "listener",
    "runner",
    "drowned",
    "hollow",
    "small",
];

/// Full structural and range validation of a run state.
pub fn validate(s: &GameState) -> Result<(), GameError> {
    check(s.rng.is_valid(), "rng")?;
    check(backgrounds::get(&s.background).is_some(), "background")?;
    if let Some(k) = &s.keepsake {
        check(backgrounds::keepsake(k).is_some(), "keepsake")?;
    }
    if let Some(d) = &s.daily {
        check(meta::parse_date(d).is_some(), "daily date")?;
    }
    check(s.held_marks.iter().all(|m| marks::get(m).is_some()), "held marks")?;
    check(s.marks.iter().all(|m| marks::get(m).is_some()), "marks")?;
    let day = s.day();
    check(s.minute >= crate::state::START_MINUTE && day <= MAX_DAY, "clock")?;
    for k in crate::content::VitalKind::ALL {
        check((0..=VITAL_MAX).contains(&s.vitals.get(*k)), "vitals")?;
    }
    check(s.bleeding <= 3, "bleeding")?;
    check((0..=VITAL_MAX).contains(&s.infection), "infection")?;
    let horizon = s.minute.saturating_add(3 * crate::state::DAY);
    check(
        s.antiseptic_until <= horizon && s.sick_until <= horizon && s.fire_until <= horizon,
        "timers",
    )?;
    check(HARMS.contains(&s.harm.as_str()), "harm")?;
    check(s.noise <= 100, "noise")?;
    check((-2..=2).contains(&s.temp_offset), "temperature")?;
    let fd = s.difficulty.freeze_day();
    check((fd..=fd + 2).contains(&s.freeze_day), "freeze day")?;
    check(!s.frozen || day >= s.freeze_day, "frozen")?;

    check_inv(&s.store, "store")?;
    check_inv(&s.pack, "pack")?;
    for (slot, id) in &s.worn {
        match items::get(id).map(|d| d.kind) {
            Some(ItemKind::Clothing { slot: sl, .. }) => check(sl == *slot, "worn slot")?,
            _ => return Err(err("Out of range: worn.")),
        }
    }
    for (id, n) in &s.wear {
        check(items::get(id).is_some() && *n < 10_000, "wear")?;
    }
    check(s.stations.len() == Station::ALL.len(), "stations")?;
    for st in Station::ALL {
        check(s.stations.get(st).is_some_and(|l| *l <= 3), "station level")?;
    }
    check(
        s.barricade <= stations::barricade_max(s.level(Station::Barricade)),
        "barricade",
    )?;
    check(s.skills.len() == Skill::ALL.len(), "skills")?;
    check(
        s.skills.values().all(|x| *x <= crate::systems::skills::MAX_XP),
        "skill practice",
    )?;

    // Places and rooms.
    check(s.places.len() == places::PLACES.len(), "places")?;
    for p in places::PLACES {
        let ps = s.places.get(p.id).ok_or_else(|| err("Out of range: places."))?;
        check(ps.rooms.len() == p.rooms.len(), "rooms")?;
        for rs in &ps.rooms {
            check_inv(&rs.hidden, "room loot")?;
            check_inv(&rs.pile, "room pile")?;
            check(rs.foes.len() <= MAX_FOES, "room occupants")?;
            check(rs.restocked <= day, "restock")?;
            for f in &rs.foes {
                check(
                    f.health > 0 && f.health <= creatures::def(f.kind).health && f.wait < 10,
                    "occupant",
                )?;
            }
        }
    }
    if let Some(t) = &s.trip {
        let p = places::get(&t.place).ok_or_else(|| err("Out of range: trip."))?;
        check(t.room.is_none_or(|r| r < p.rooms.len()), "trip room")?;
        check(t.departed <= s.minute, "trip time")?;
    }
    if let Some(d) = &s.dest {
        check(places::get(d).is_some(), "destination")?;
    }
    if let Some(e) = &s.encounter {
        check(!e.foes.is_empty() && e.foes.len() <= MAX_FOES, "encounter")?;
        check(e.log.len() <= 60 && e.turn < 10_000, "encounter log")?;
        for f in &e.foes {
            check(
                f.health > 0 && f.health <= creatures::def(f.kind).health && f.wait < 10,
                "foe",
            )?;
        }
        if let Some(i) = e.room {
            check(
                s.trip
                    .as_ref()
                    .and_then(|t| places::get(&t.place))
                    .is_some_and(|p| i < p.rooms.len()),
                "encounter room",
            )?;
        }
    }
    if let Some(e) = &s.event {
        check(events::get(&e.id).is_some(), "event")?;
    }
    check(s.scheduled.len() <= 50, "scheduled")?;
    for (id, due) in &s.scheduled {
        check(events::get(id).is_some() && *due <= day + 30, "scheduled event")?;
    }
    check(s.seen_events.iter().all(|e| events::get(e).is_some()), "seen events")?;
    check(s.codex.iter().all(|c| codex::get(c).is_some()), "codex")?;
    check(s.flags.len() <= 100 && s.flags.iter().all(|f| short_id(f)), "flags")?;
    check(
        s.ferry.iter().all(|p| items::FERRY_PARTS.contains(&p.as_str())),
        "ferry",
    )?;
    check(s.journal.len() <= JOURNAL_CAP, "journal")?;
    for e in &s.journal {
        check(
            e.text.len() <= 2_000 && e.time.len() == 5 && e.day <= MAX_DAY,
            "journal entry",
        )?;
    }
    check(
        (0..=100).contains(&s.dog.health) && (-100..=100).contains(&s.dog.fed) && s.dog.days_hungry < 100,
        "dog",
    )?;
    check(
        s.traps.snares <= 99 && s.traps.snared <= s.traps.snares && s.traps.hooked <= 3,
        "traps",
    )?;
    check(s.garden.len() <= stations::garden_plots(3), "garden")?;
    for p in &s.garden {
        check(
            items::get(&p.crop).is_some() && p.need <= 30 && p.growth <= p.need,
            "plot",
        )?;
    }
    let w = &s.widow;
    check(
        w.offset < 3 && w.stock.len() <= 12 && w.credit <= 100_000 && w.trades <= 100_000,
        "widow",
    )?;
    for o in &w.stock {
        check(
            items::get(&o.item).is_some() && o.qty <= 9 && o.price <= 100_000,
            "offer",
        )?;
    }
    check(
        w.extra.len() <= 12 && w.extra.iter().all(|x| items::get(x).is_some()),
        "widow extra",
    )?;
    check(w.sold.len() <= 12 && w.sold.keys().all(|k| short_id(k)), "widow memory")?;
    check(s.radio_day <= day && s.visit_day <= day, "days")?;
    if let Some(n) = &s.night {
        check(
            n.stage <= 1 && n.bed <= s.minute && n.noise <= 100 && n.visitors <= 10,
            "night",
        )?;
        if let Some(e) = &n.event {
            check(events::get(e).is_some(), "night event")?;
        }
    }
    if let Some(d) = &s.dawn {
        check(d.lines.len() <= 20 && d.headline.len() <= 200, "dawn")?;
    }
    if let Some(l) = &s.ledger {
        meta::validate_ledger(l).map_err(|e| err(e.message))?;
    }

    // The phase must agree with the rest of the state.
    let ok = match s.phase {
        Phase::Home | Phase::Night => s.trip.is_none(),
        Phase::Dawn => s.trip.is_none() && s.dawn.is_some(),
        Phase::Packing => s.trip.is_none() && s.dest.is_some(),
        Phase::Travelling => s.trip.is_some() && s.dest.is_some(),
        Phase::Site => s.trip.is_some(),
        Phase::Barter => s.trip.as_ref().is_some_and(|t| t.place == "weir" && t.room.is_none()),
        Phase::Encounter => s.encounter.is_some(),
        Phase::Event => s.event.is_some(),
        Phase::Ended => s.ending.is_some() && s.ledger.is_some(),
    };
    check(ok, "phase")?;
    check(s.encounter.is_none() || s.phase == Phase::Encounter, "stray encounter")?;
    check(s.event.is_none() || s.phase == Phase::Event, "stray event")?;
    check(s.ending.is_none() || s.phase == Phase::Ended, "stray ending")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameConfig, new_game};

    fn game() -> GameState {
        new_game(&GameConfig {
            seed: 77,
            difficulty: crate::content::Difficulty::Still,
            background: "clerk".into(),
            keepsake: None,
            daily: None,
            held_marks: vec![],
        })
        .unwrap()
    }

    #[test]
    fn round_trip() {
        let s = game();
        let text = save_to_string(&s);
        let back = load_from_string(&text).unwrap();
        assert_eq!(s, back);
        assert_eq!(save_to_string(&back), text);
    }

    #[test]
    fn profile_round_trip() {
        let p = Profile::new();
        let text = profile_to_string(&p);
        assert_eq!(profile_from_string(&text).unwrap(), p);
        assert!(load_from_string(&text).is_err(), "a profile is not a run save");
        assert!(profile_from_string(&save_to_string(&game())).is_err());
    }

    #[test]
    fn newer_version_rejected() {
        let s = game();
        let text = save_to_string(&s).replacen("\"version\":1", "\"version\":2", 1);
        assert!(load_from_string(&text).is_err());
    }
}
