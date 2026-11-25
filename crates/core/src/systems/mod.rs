//! The rules, one system per file. Systems mutate `GameState` and write to
//! the `Journal` (the outcome being built); `action.rs` decides which system
//! an action goes to and whether it is allowed.

pub mod barter;
pub mod clock;
pub mod codex;
pub mod dog;
pub mod encounter;
pub mod events;
pub mod expedition;
pub mod food;
pub mod home;
pub mod inventory;
pub mod journal;
pub mod night;
pub mod noise;
pub mod objectives;
pub mod skills;
pub mod weather;

use crate::Journal;
use crate::action::GameEvent;
use crate::state::{EndKind, Ending, GameState, Phase, Resume, Tone};

/// Percent of `base`, rounded down.
pub fn pct(base: i32, p: i32) -> i32 {
    ((i64::from(base) * i64::from(p)) / 100) as i32
}

pub fn pct_u(base: u32, p: u32) -> u32 {
    ((u64::from(base) * u64::from(p)) / 100).min(u64::from(u32::MAX)) as u32
}

/// End the run. Idempotent: the first ending stands.
pub fn end_run(s: &mut GameState, j: &mut Journal, kind: EndKind, cause: &str, text: &str) {
    if s.ending.is_some() {
        return;
    }
    s.ending = Some(Ending {
        kind,
        cause: cause.to_string(),
        text: text.to_string(),
    });
    s.phase = Phase::Ended;
    s.encounter = None;
    s.event = None;
    journal::write(
        s,
        j,
        if kind == EndKind::Death {
            Tone::Danger
        } else {
            Tone::Quiet
        },
        text,
    );
    j.events.push(GameEvent::RunEnded { kind });
    crate::meta::award_run_marks(s, j);
    let ledger = crate::meta::ledger(s);
    s.ledger = Some(ledger);
}

/// If health has run out, end the run with the latest cause of harm.
pub fn check_death(s: &mut GameState, j: &mut Journal) -> bool {
    if s.ending.is_some() {
        return true;
    }
    if s.vitals.health > 0 {
        return false;
    }
    let place = match &s.trip {
        Some(t) => crate::content::places::get(&t.place).map_or("on the road", |p| p.name),
        None => "at the lock house",
    };
    let (cause, text) = journal::death_line(&s.harm, place, s.day());
    end_run(s, j, EndKind::Death, &cause, &text);
    true
}

/// Continue after an event or encounter.
pub fn resume(s: &mut GameState, j: &mut Journal, then: Resume) {
    if s.ended() {
        return;
    }
    match then {
        Resume::Home => s.phase = Phase::Home,
        Resume::Site => s.phase = Phase::Site,
        Resume::Arrive => expedition::arrive(s, j),
        Resume::ArriveHome => expedition::arrive_home(s, j),
        Resume::Night => night::continue_night(s, j),
        Resume::Rise => {
            s.phase = Phase::Home;
            events::maybe(s, j, crate::content::events::Ctx::Dawn, Resume::Home);
        }
    }
}
