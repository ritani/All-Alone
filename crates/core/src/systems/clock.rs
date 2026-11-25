//! Time passing: the clock, the six vitals, wounds, sickness and the hour
//! boundaries (dusk, the small hours, dawn).

use crate::Journal;
use crate::action::GameEvent;
use crate::content::{Station, VitalKind, Weather, backgrounds};
use crate::state::{DAWN, DAY, DUSK, DogState, GameState, MILLI, Tone};
use crate::systems::{check_death, inventory, journal, night, weather};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Awake,
    /// Walking hard, fighting, wading.
    Exertion,
    Asleep,
}

/// Above this the body cannot fight an infection off by itself.
pub const INFECTION_SERIOUS: i32 = 30 * MILLI;

/// The small hours, when things come to the house.
pub const VISIT_HOUR: u32 = 2 * 60;

/// Minutes an action really takes: tiredness slows everything.
pub fn cost(s: &GameState, base: u32) -> u32 {
    if base == 0 {
        return 0;
    }
    let rest = s.vitals.pts(VitalKind::Rest);
    let mut pct = 100;
    if rest < 15 {
        pct += 35;
    } else if rest < 30 {
        pct += 15;
    }
    if s.vitals.pts(VitalKind::Warmth) < 20 {
        pct += 10;
    }
    crate::systems::pct_u(base, pct).max(1)
}

fn minutes_to(now: u32, target_tod: u32) -> u32 {
    let t = now % DAY;
    if t < target_tod {
        target_tod - t
    } else {
        DAY - t + target_tod
    }
}

/// Let `minutes` pass. Stops early if the run ends.
pub fn pass(s: &mut GameState, j: &mut Journal, minutes: u32, act: Activity) {
    let mut left = minutes;
    while left > 0 && !s.ended() {
        let to_b = minutes_to(s.minute, DUSK)
            .min(minutes_to(s.minute, VISIT_HOUR))
            .min(minutes_to(s.minute, DAWN));
        let step = left.min(60).min(to_b);
        tick(s, j, step, act);
        s.minute = s.minute.saturating_add(step);
        left -= step;
        if check_death(s, j) {
            return;
        }
        if step == to_b {
            let t = s.minute % DAY;
            if t == DUSK {
                j.events.push(GameEvent::Dusk);
                let line = if s.at_home() {
                    "Dusk. The light goes off the river."
                } else {
                    "Dusk. They will be waking now."
                };
                journal::write(s, j, Tone::Danger, line);
            } else if t == VISIT_HOUR {
                if !s.at_home() && s.visit_day != s.day() {
                    night::away_visit(s, j);
                }
            } else if t == DAWN {
                night::new_day(s, j);
            }
        }
    }
}

/// Target warmth where you are now.
pub fn warmth_target(s: &GameState, act: Activity) -> i32 {
    let temp = weather::temperature(s);
    if s.at_home() {
        let mut t = 55 + temp * 3;
        if s.fire_until > s.minute {
            t += 28 + 10 * i32::from(s.level(Station::Stove));
        }
        if act == Activity::Asleep && s.level(Station::Cot) >= 2 {
            t += 8;
        }
        t.clamp(0, 100)
    } else {
        let mut t = 38 + temp * 4 + inventory::insulation(s);
        if s.weather == Weather::Rain && !inventory::rainproof(s) {
            t -= 12;
        }
        if s.weather == Weather::Snow {
            t -= 6;
        }
        if act == Activity::Exertion {
            t += 6;
        }
        if s.trip.as_ref().is_some_and(|t| t.room.is_some()) {
            t += 10;
        }
        t.clamp(0, 100)
    }
}

pub fn nerve_baseline(s: &GameState, act: Activity) -> i32 {
    let mut b = if s.at_home() { 58 } else { 45 };
    let dog = s.dog.state == DogState::Companion;
    if s.at_home() {
        if dog {
            b += 12;
        }
        if s.fire_until > s.minute {
            b += 5;
        }
        if act == Activity::Asleep && s.level(Station::Cot) >= 3 {
            b += 8;
        }
    } else {
        if dog && s.trip.as_ref().is_some_and(|t| t.dog) {
            b += 8;
        }
        if s.is_dark() {
            b -= 12;
        }
    }
    if s.frozen {
        b -= 5;
    }
    b
}

/// Rates are per hour in thousandths; scaled to `m` minutes.
fn per(m: u32, rate: i32) -> i32 {
    ((i64::from(rate) * i64::from(m)) / 60) as i32
}

fn toward(cur: i32, target: i32, rate: i32) -> i32 {
    if cur < target {
        (cur + rate).min(target)
    } else {
        (cur - rate).max(target)
    }
}

fn tick(s: &mut GameState, j: &mut Journal, m: u32, act: Activity) {
    if m == 0 {
        return;
    }
    let bg = backgrounds::get(&s.background).map(|b| b.mods).unwrap_or_default();
    let diff = s.difficulty.decay_pct();
    let cold = s.vitals.pts(VitalKind::Warmth) < 25;
    let sick = s.sick_until > s.minute;
    let before: Vec<i32> = VitalKind::ALL.iter().map(|k| s.vitals.pts(*k)).collect();

    // Satiety
    let mut sat = match act {
        Activity::Awake => 2400,
        Activity::Exertion => 3400,
        Activity::Asleep => 1200,
    };
    sat = crate::systems::pct(sat, diff + bg.hunger + if cold { 40 } else { 0 });
    s.vitals.add(VitalKind::Satiety, -per(m, sat));

    // Hydration
    let mut hyd = match act {
        Activity::Awake => 4000,
        Activity::Exertion => 5600,
        Activity::Asleep => 2000,
    };
    if sick {
        hyd += 3000;
    }
    hyd = crate::systems::pct(hyd, diff);
    s.vitals.add(VitalKind::Hydration, -per(m, hyd));

    // Rest
    let rest = match act {
        Activity::Awake => -crate::systems::pct(4000, diff + if cold { 30 } else { 0 }),
        Activity::Exertion => -crate::systems::pct(6000, diff + if cold { 30 } else { 0 }),
        Activity::Asleep => {
            let mut r = 12_500;
            r = crate::systems::pct(r, crate::content::stations::sleep_pct(s.level(Station::Cot)) + bg.sleep);
            if s.vitals.pts(VitalKind::Warmth) < 30 {
                r = crate::systems::pct(r, 55);
            }
            if s.vitals.pts(VitalKind::Nerve) < 20 {
                r = crate::systems::pct(r, 75);
            }
            r
        }
    };
    s.vitals.add(VitalKind::Rest, per(m, rest));

    // Warmth drifts toward where you are.
    let target = warmth_target(s, act) * MILLI;
    let wrate = if s.at_home() { 12_000 } else { 15_000 };
    let w = toward(s.vitals.warmth, target, per(m, wrate));
    s.vitals.warmth = w.clamp(0, 100 * MILLI);

    // Nerve drifts toward a baseline; cold and sickness wear it down.
    let base = nerve_baseline(s, act) * MILLI;
    let nrate = if act == Activity::Asleep { 4_000 } else { 2_500 };
    let mut n = toward(s.vitals.nerve, base, per(m, nrate));
    if cold {
        n -= per(m, 1_000);
    }
    if sick {
        n -= per(m, 1_000);
    }
    s.vitals.nerve = n.clamp(0, 100 * MILLI);

    // Infection and bleeding.
    if s.infection > 0 {
        // A fed, rested, warm body throws off a small infection; a bad one grows.
        let fighting = s.infection < INFECTION_SERIOUS
            && s.vitals.pts(VitalKind::Satiety) > 40
            && s.vitals.pts(VitalKind::Hydration) > 40
            && s.vitals.pts(VitalKind::Rest) > 30
            && s.vitals.pts(VitalKind::Warmth) > 30;
        let rate = if s.antiseptic_until > s.minute {
            -900
        } else if fighting {
            -500
        } else {
            let shelf = if s.at_home() && s.level(Station::Medicine) >= 3 {
                50
            } else {
                100
            };
            crate::systems::pct(1_000, shelf)
        };
        s.infection = (s.infection + per(m, rate)).clamp(0, 100 * MILLI);
    }
    if s.bleeding == 1 && s.rng.chance((15 * m / 60) as i32) {
        s.bleeding = 0;
        journal::plain(s, j, "The bleeding has stopped on its own.");
    }

    // Health.
    let mut drains: Vec<(&str, i32)> = Vec::new();
    if s.vitals.satiety == 0 {
        drains.push(("hunger", 2_500));
    }
    if s.vitals.hydration == 0 {
        drains.push(("thirst", 4_500));
    }
    if s.vitals.rest == 0 {
        drains.push(("exhaustion", 2_000));
    }
    let warmth = s.vitals.pts(VitalKind::Warmth);
    if warmth == 0 {
        drains.push(("cold", 6_000));
    } else if warmth < 15 {
        drains.push(("cold", 1_500));
    }
    if s.bleeding > 0 {
        drains.push(("bleeding", 3_000 * i32::from(s.bleeding)));
    }
    if s.infection > 0 {
        drains.push(("infection", s.infection / 40));
    }
    if sick {
        drains.push(("sickness", 600));
    }
    if let Some((harm, _)) = drains.iter().max_by_key(|(_, r)| *r) {
        s.harm = (*harm).to_string();
    }
    let drain: i32 = drains.iter().map(|(_, r)| *r).sum();
    let well = s.vitals.pts(VitalKind::Satiety) > 30
        && s.vitals.pts(VitalKind::Hydration) > 30
        && s.vitals.pts(VitalKind::Rest) > 20
        && warmth > 30
        && s.bleeding == 0
        && s.infection < INFECTION_SERIOUS
        && !sick;
    let mut regen = 0;
    if well {
        regen = match act {
            Activity::Asleep => 3_000,
            _ => 1_000,
        };
        if s.at_home() && s.level(Station::Medicine) >= 3 {
            regen += 800;
        }
    }
    s.vitals.add(VitalKind::Health, per(m, regen - drain));

    // Announce vitals crossing into danger.
    for (i, k) in VitalKind::ALL.iter().enumerate() {
        let now = s.vitals.pts(*k);
        if before[i] >= 20 && now < 20 {
            j.events.push(GameEvent::VitalLow { vital: *k });
            let line = match k {
                VitalKind::Health => "You are badly hurt.",
                VitalKind::Satiety => "You are weak with hunger.",
                VitalKind::Hydration => "Your mouth is dry, your head aches. You need water.",
                VitalKind::Rest => "You are stumbling tired.",
                VitalKind::Nerve => "Your hands will not stop shaking.",
                VitalKind::Warmth => "The cold is in your bones.",
            };
            journal::danger(s, j, line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameConfig, new_game};

    fn game() -> GameState {
        new_game(&GameConfig {
            seed: 9,
            difficulty: crate::content::Difficulty::Still,
            background: "clerk".into(),
            keepsake: None,
            daily: None,
            held_marks: vec![],
        })
        .unwrap()
    }

    #[test]
    fn vitals_decay_and_stay_in_range() {
        let mut s = game();
        let mut j = Journal::default();
        let sat = s.vitals.satiety;
        pass(&mut s, &mut j, 120, Activity::Awake);
        assert!(s.vitals.satiety < sat);
        assert_eq!(s.minute, crate::state::START_MINUTE + 120);
        for _ in 0..200 {
            pass(&mut s, &mut j, 60, Activity::Exertion);
            for k in VitalKind::ALL {
                let v = s.vitals.get(*k);
                assert!((0..=100 * MILLI).contains(&v));
            }
        }
        assert!(s.ended(), "200 hours without food or water should kill you");
    }

    #[test]
    fn dusk_is_announced() {
        let mut s = game();
        let mut j = Journal::default();
        pass(&mut s, &mut j, 12 * 60 + 5, Activity::Awake);
        assert!(j.events.iter().any(|e| matches!(e, GameEvent::Dusk)));
    }

    #[test]
    fn cost_rises_when_tired() {
        let mut s = game();
        assert_eq!(cost(&s, 60), 60);
        s.vitals.rest = 5 * MILLI;
        assert!(cost(&s, 60) > 60);
    }
}
