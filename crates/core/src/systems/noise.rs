//! Noise: a per-day resource. Spending it is fast; it draws them.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::{Skill, Weather};
use crate::state::GameState;

pub const MAX: u32 = 100;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Moving and searching: quiet step reduces it.
    Movement,
    /// Everything else: tools, fights, shots.
    Other,
}

/// The noise an action of base `n` will actually make.
pub fn effective(s: &GameState, n: u32, kind: Kind) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut pct = 100i64;
    if kind == Kind::Movement {
        pct -= 7 * i64::from(s.skill(Skill::QuietStep));
    }
    if s.weather == Weather::Rain && !s.at_home() {
        pct -= 35;
    }
    if s.weather == Weather::Rain && s.at_home() {
        pct -= 20;
    }
    ((i64::from(n) * pct.max(20) + 50) / 100).max(1) as u32
}

/// Make noise. Returns the amount actually added.
pub fn make(s: &mut GameState, j: &mut Journal, n: u32, kind: Kind) -> u32 {
    let n = effective(s, n, kind);
    if n == 0 {
        return 0;
    }
    let before = s.noise;
    s.noise = (s.noise + n).min(MAX);
    s.stats.noise_total = s.stats.noise_total.saturating_add(n);
    if let Some(t) = s.trip.as_mut() {
        t.site_noise = t.site_noise.saturating_add(n);
    }
    if n >= 12 || (before < 50 && s.noise >= 50) {
        j.events.push(GameEvent::Noise {
            amount: n,
            total: s.noise,
        });
    }
    n
}

pub fn label(noise: u32) -> &'static str {
    match noise {
        0..=9 => "Silent",
        10..=24 => "Quiet",
        25..=44 => "Heard",
        45..=69 => "Loud",
        _ => "Ringing",
    }
}

/// Overnight decay: a quarter carries into the next day.
pub fn decay(noise: u32) -> u32 {
    noise / 4
}
