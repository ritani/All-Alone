//! Food you work for: the trap line, the night line at the lock gate, the
//! garden bed, and foraging along the bank.

use crate::Journal;
use crate::content::items::{self, ItemKind};
use crate::content::{Skill, Station, Weather, stations};
use crate::state::{GameState, Plot, inv_add, inv_count, inv_take};
use crate::systems::{inventory, journal, skills};

/// Dawn: snares and the line may have caught something; the garden grows.
pub fn dawn(s: &mut GameState, j: &mut Journal) {
    let forage = i32::from(s.skill(Skill::Foraging));
    // Snares.
    let mut broken = 0;
    for _ in 0..s.traps.snares {
        if s.rng.chance(4) {
            broken += 1;
            continue;
        }
        if s.traps.snared < s.traps.snares {
            let mut p = 25 + forage * 4;
            if s.weather == Weather::Rain {
                p += 5;
            }
            if s.weather == Weather::Snow {
                p -= 8;
            }
            if s.rng.chance(p) {
                s.traps.snared += 1;
            }
        }
    }
    if broken > 0 {
        s.traps.snares -= broken;
        s.traps.snared = s.traps.snared.min(s.traps.snares);
    }
    // Night line.
    if s.traps.line {
        if s.frozen {
            s.traps.line = false;
            journal::plain(
                s,
                j,
                "The night line is frozen into the ice. You cut it free and bring it in.",
            );
            inv_add(&mut s.store, "fishing_line", 1);
        } else if s.rng.chance(1) {
            s.traps.line = false;
            s.traps.baited = false;
            journal::plain(s, j, "The night line has gone, snapped at the gate.");
        } else if s.traps.hooked < 3 {
            let mut p = 40 + forage * 5;
            if s.traps.baited {
                p += 25;
            }
            if s.weather == Weather::Rain || s.weather == Weather::Overcast {
                p += 8;
            }
            if s.rng.chance(p) {
                s.traps.hooked = (s.traps.hooked + if s.rng.chance(40) { 2 } else { 1 }).min(3);
            }
            s.traps.baited = false;
        }
    }
    // Garden.
    let lvl = s.level(Station::Garden);
    let cold_ok = lvl >= 2 || !matches!(s.weather, Weather::Frost | Weather::Snow);
    let grow = s.tended && cold_ok;
    for p in &mut s.garden {
        if grow && p.growth < p.need {
            p.growth += 1;
            if lvl >= 3 && s.rng.chance(25) {
                p.growth = (p.growth + 1).min(p.need);
            }
        }
    }
    if !cold_ok && !s.garden.is_empty() {
        journal::plain(s, j, "Frost on the garden bed. Nothing grew.");
    }
    s.tended = s.weather == Weather::Rain;
}

pub fn forage(s: &mut GameState, j: &mut Journal) {
    let f = s.skill(Skill::Foraging);
    let mut wood = s.rng.range(1, 2) as u32 + u32::from(f >= 3);
    if s.weather == Weather::Snow {
        wood = wood.saturating_sub(1).max(1);
    }
    let mut got = vec![journal::qty("firewood", wood)];
    inventory::gain(s, j, "firewood", wood);
    let bait_p = if s.weather == Weather::Rain { 75 } else { 45 } - if s.frozen { 35 } else { 0 };
    if s.rng.chance(bait_p) {
        let n = s.rng.range(1, 2) as u32;
        inventory::gain(s, j, "bait", n);
        got.push(journal::qty("worms", n));
    }
    if s.day() < 34 && s.rng.chance(35 + 6 * i32::from(f)) {
        inventory::gain(s, j, "herbs", 1);
        got.push("yarrow and willow bark".into());
    }
    // Windfalls and blackberries while the season lasts.
    if s.day() < 24 && s.rng.chance(45 + 5 * i32::from(f)) {
        let n = s.rng.range(1, 2) as u32;
        inventory::gain(s, j, "apple", n);
        got.push(journal::qty("windfall apples", n));
    }
    journal::find(
        s,
        j,
        &format!("You work along the bank and come back with {}.", journal::list(&got)),
    );
    skills::practise(s, j, Skill::Foraging, 1);
}

pub fn set_snares(s: &mut GameState, j: &mut Journal) {
    let n = inv_take(&mut s.store, "snare", 99);
    s.traps.snares += n;
    journal::plain(
        s,
        j,
        &format!(
            "You set {} along the bank above the lock. {} set in all.",
            journal::qty("snare", n).to_lowercase(),
            s.traps.snares
        ),
    );
}

pub fn check_snares(s: &mut GameState, j: &mut Journal) {
    let n = s.traps.snared;
    s.traps.snared = 0;
    if n == 0 {
        journal::plain(s, j, "You walk the trap line. Nothing.");
    } else {
        inventory::gain(s, j, "raw_meat", n);
        s.stats.snared += n;
        journal::find(
            s,
            j,
            &format!(
                "You walk the trap line. {}.",
                if n == 1 {
                    "One rabbit".to_string()
                } else {
                    format!("{n} rabbits")
                }
            ),
        );
    }
    skills::practise(s, j, Skill::Foraging, 1);
}

pub fn set_line(s: &mut GameState, j: &mut Journal) {
    inv_take(&mut s.store, "fishing_line", 1);
    s.traps.line = true;
    s.traps.baited = inv_take(&mut s.store, "bait", 1) == 1;
    let line = if s.traps.baited {
        "You bait the night line and drop it from the lock gate."
    } else {
        "You drop the night line from the lock gate, unbaited."
    };
    journal::plain(s, j, line);
}

pub fn check_line(s: &mut GameState, j: &mut Journal) {
    let n = s.traps.hooked;
    s.traps.hooked = 0;
    if n == 0 {
        journal::plain(s, j, "You haul in the night line. Weed and an empty hook.");
    } else {
        inventory::gain(s, j, "raw_fish", n);
        s.stats.fish += n;
        journal::find(
            s,
            j,
            &format!(
                "You haul in the night line. {}.",
                if n == 1 {
                    "A fish".to_string()
                } else {
                    format!("{n} fish")
                }
            ),
        );
        skills::practise(s, j, Skill::Foraging, 1);
    }
    if !s.traps.baited && inv_take(&mut s.store, "bait", 1) == 1 {
        s.traps.baited = true;
        journal::plain(s, j, "You re-bait it and let it down again.");
    }
}

pub fn plots_free(s: &GameState) -> usize {
    stations::garden_plots(s.level(Station::Garden)).saturating_sub(s.garden.len())
}

pub fn plant(s: &mut GameState, j: &mut Journal, seed: &str) {
    let Some(ItemKind::Seed { crop, days }) = items::get(seed).map(|d| d.kind) else {
        return;
    };
    inv_take(&mut s.store, seed, 1);
    s.garden.push(Plot {
        crop: crop.to_string(),
        growth: 0,
        need: days,
    });
    journal::plain(
        s,
        j,
        &format!("You plant {} in the bed.", items::name(seed).to_lowercase()),
    );
}

pub fn tend(s: &mut GameState, j: &mut Journal) {
    let id = if inv_count(&s.store, "water_dirty") > 0 {
        "water_dirty"
    } else {
        "water_clean"
    };
    inv_take(&mut s.store, id, 1);
    s.tended = true;
    journal::plain(s, j, "You water the bed and pull what weeds there are.");
}

pub fn ready(s: &GameState) -> usize {
    s.garden.iter().filter(|p| p.growth >= p.need).count()
}

pub fn harvest(s: &mut GameState, j: &mut Journal) {
    let f = u32::from(s.skill(Skill::Foraging));
    let (done, left): (Vec<Plot>, Vec<Plot>) = s.garden.drain(..).partition(|p| p.growth >= p.need);
    s.garden = left;
    let mut got = Vec::new();
    for p in done {
        let n = 3 + f / 2 + u32::from(p.crop == "potato");
        inventory::gain(s, j, &p.crop, n);
        got.push(journal::qty(items::name(&p.crop), n).to_lowercase());
        if s.rng.chance(30) {
            let seed = match p.crop.as_str() {
                "kale" => "seed_kale",
                "turnip" => "seed_turnip",
                _ => "seed_potato",
            };
            inventory::gain(s, j, seed, 1);
        }
        s.stats.harvests += 1;
    }
    journal::find(s, j, &format!("You lift {}.", journal::list(&got)));
    skills::practise(s, j, Skill::Foraging, 1);
}
