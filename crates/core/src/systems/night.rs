//! Night at the lock house: going to bed, the small-hours visit, waking, and
//! the processing every dawn brings (weather, spoilage, traps, the freeze).

use crate::Journal;
use crate::action::GameEvent;
use crate::content::events::Ctx;
use crate::content::{Band, Creature, Station, VitalKind, Weather, items, places, stations};
use crate::state::{
    DAWN, DAY, DawnSummary, DogState, Encounter, Foe, GameState, MILLI, NightLog, Phase, Resume, Tone, inv_take,
};
use crate::systems::clock::{self, Activity, VISIT_HOUR};
use crate::systems::{barter, codex, dog, events, food, inventory, journal, noise, pct, weather};

/// Percent chance something comes to the house tonight, from today's noise.
pub fn visit_chance(s: &GameState) -> i32 {
    let day = s.day() as i32;
    let mut c = 3 + (s.noise as i32) * 8 / 10 + day * 6 / 10;
    if s.weather == Weather::Rain {
        c = pct(c, 65);
    }
    if s.weather == Weather::Fog {
        c = pct(c, 90);
    }
    if s.weather == Weather::Frost {
        c = pct(c, 85);
    }
    c = pct(c, s.difficulty.threat_pct());
    c.clamp(2, 85)
}

fn visitor_kind(s: &mut GameState) -> Creature {
    let day = s.day();
    let w = [60, if day >= 8 { 22 } else { 0 }, 15, if day >= 24 { 14 } else { 0 }];
    match s.rng.weighted(&w) {
        Some(1) => Creature::Runner,
        Some(2) => Creature::Listener,
        Some(3) => Creature::Hollow,
        _ => Creature::Stander,
    }
}

struct Visit {
    kinds: Vec<Creature>,
    damage: u32,
}

fn roll_visit(s: &mut GameState) -> Option<Visit> {
    if !s.rng.chance(visit_chance(s)) {
        return None;
    }
    let n = (1 + s.rng.below(1 + s.day() / 12) + s.noise / 30).min(5);
    let mut kinds = Vec::new();
    let mut damage = 0;
    for _ in 0..n {
        let k = visitor_kind(s);
        damage += if k == Creature::Hollow {
            s.rng.range(22, 30)
        } else {
            s.rng.range(10, 18)
        } as u32;
        kinds.push(k);
    }
    Some(Visit { kinds, damage })
}

/// The house is visited while you are away from it.
pub fn away_visit(s: &mut GameState, j: &mut Journal) {
    s.visit_day = s.day();
    let Some(v) = roll_visit(s) else { return };
    if v.damage >= s.barricade {
        s.barricade = 0;
        s.house_breached = true;
        let n = 3 + s.rng.below(4);
        inventory::rob(s, j, n);
    } else {
        s.barricade -= v.damage;
    }
}

pub fn go_to_bed(s: &mut GameState, j: &mut Journal) {
    s.phase = Phase::Night;
    journal::plain(s, j, "You bar the door and bank the stove for the night.");
}

/// Start the night. `watch`: sit up by the door until the small hours.
pub fn sleep(s: &mut GameState, j: &mut Journal, watch: bool) {
    s.night = Some(NightLog {
        bed: s.minute,
        watch,
        stage: 0,
        noise: s.noise,
        visitors: 0,
        damage: 0,
        breached: false,
        rest_before: s.vitals.rest,
        warmth_low: false,
        event: None,
    });
    if watch {
        journal::plain(s, j, "You sit up in the chair by the door with the lamp turned low.");
    }
    if events::maybe(s, j, Ctx::Night, Resume::Night) {
        if let (Some(n), Some(e)) = (s.night.as_mut(), s.event.as_ref()) {
            n.event = Some(e.id.clone());
        }
        return;
    }
    continue_night(s, j);
}

/// Resume the night after an event or a fight at the door.
pub fn continue_night(s: &mut GameState, j: &mut Journal) {
    if s.ended() {
        return;
    }
    if s.night.is_none() {
        // A night without a log (a scheduled night event fired some other way).
        s.phase = Phase::Home;
        return;
    }
    // Scheduled night events (a knock answered) chain on before sleep.
    if events::maybe_scheduled(s, j, Ctx::Night, Resume::Night) {
        return;
    }
    s.phase = Phase::Night;
    let watch = s.night.as_ref().is_some_and(|n| n.watch) || s.has_flag("watching");
    let stage = s.night.as_ref().map_or(1, |n| n.stage);
    if stage == 0 {
        let t = s.minute % DAY;
        let before_visit = !(VISIT_HOUR..DAWN).contains(&t);
        if before_visit {
            let mins = if t < VISIT_HOUR {
                VISIT_HOUR - t
            } else {
                DAY - t + VISIT_HOUR
            };
            clock::pass(s, j, mins, if watch { Activity::Awake } else { Activity::Asleep });
            if s.ended() {
                return;
            }
        }
        if let Some(n) = s.night.as_mut() {
            n.stage = 1;
        }
        if s.visit_day != s.day() && home_visit(s, j, watch) {
            return;
        }
    }
    finish(s, j);
}

/// Something comes to the house while you are in it. True if a fight started.
fn home_visit(s: &mut GameState, j: &mut Journal, watch: bool) -> bool {
    s.visit_day = s.day();
    let Some(v) = roll_visit(s) else { return false };
    let dog_here = s.dog.state == DogState::Companion;
    let bonus = if watch { 20 } else { 0 } + if dog_here { 10 } else { 0 };
    let effective = v.damage.saturating_sub(bonus);
    let breached = effective >= s.barricade;
    let excess = effective.saturating_sub(s.barricade);
    s.barricade = s.barricade.saturating_sub(effective);
    codex::find(s, j, "fg_sound");
    if let Some(n) = s.night.as_mut() {
        n.visitors = v.kinds.len() as u32;
        n.damage = effective;
        n.breached = breached;
    }
    j.events.push(GameEvent::NightVisit {
        visitors: v.kinds.len() as u32,
        breached,
    });
    if dog_here {
        journal::write(s, j, Tone::Danger, "The dog wakes you, growling low at the door.");
    }
    s.vitals.add(VitalKind::Rest, -8 * MILLI);
    if !breached {
        s.vitals
            .add(VitalKind::Nerve, if dog_here || watch { -4 } else { -9 } * MILLI);
        let line = if v.kinds.len() == 1 {
            "In the small hours something leans on the door, and tries it, and goes."
        } else {
            "In the small hours they come, several, and push at the boards until the nails creak. The boards hold."
        };
        journal::write(s, j, Tone::Danger, line);
        return false;
    }
    s.vitals.add(VitalKind::Nerve, -12 * MILLI);
    let inside = (1 + excess / 20).min(v.kinds.len() as u32).clamp(1, 3) as usize;
    let band = if dog_here || watch { Band::Near } else { Band::Close };
    let foes: Vec<Foe> = v
        .kinds
        .iter()
        .take(inside)
        .map(|k| Foe {
            kind: *k,
            health: crate::content::creatures::def(*k).health,
            band,
            wait: 0,
        })
        .collect();
    journal::write(s, j, Tone::Danger, "The boards give. Something is in the house.");
    s.encounter = Some(Encounter {
        foes,
        turn: 0,
        at_home: true,
        room: None,
        gripped: false,
        then: Resume::Night,
        log: Vec::new(),
    });
    s.phase = Phase::Encounter;
    j.events.push(GameEvent::EncounterStarted);
    true
}

fn wake_minutes(s: &GameState) -> u32 {
    let t = s.minute % DAY;
    let to_dawn = if t < DAWN { DAWN - t } else { DAY - t + DAWN };
    if to_dawn >= 180 {
        to_dawn
    } else {
        to_dawn.max(300.min(to_dawn + 120))
    }
}

fn finish(s: &mut GameState, j: &mut Journal) {
    let mins = wake_minutes(s);
    clock::pass(s, j, mins, Activity::Asleep);
    if s.ended() {
        return;
    }
    s.flags.remove("watching");
    let Some(log) = s.night.take() else {
        s.phase = Phase::Home;
        return;
    };
    let mut lines = Vec::new();
    let rest_gain = (s.vitals.rest - log.rest_before) / MILLI;
    lines.push(if rest_gain >= 40 {
        "You slept well.".to_string()
    } else if rest_gain >= 15 {
        "You slept, a little.".to_string()
    } else {
        "You hardly slept.".to_string()
    });
    if s.vitals.pts(VitalKind::Warmth) < 30 {
        lines.push("You woke cold to the bone.".to_string());
    }
    if log.breached && s.barricade == 0 {
        lines.push("The door is broken. The barricade must be rebuilt.".into());
    } else if log.visitors > 0 {
        let max = stations::barricade_max(s.level(Station::Barricade));
        lines.push(format!("The barricade holds at {} of {max}.", s.barricade));
    }
    for e in s.journal.iter().rev().take_while(|e| e.day == s.day()) {
        if e.tone == Tone::Danger || e.tone == Tone::Find {
            if e.text.starts_with("Dusk") {
                continue;
            }
            lines.push(e.text.clone());
        }
    }
    lines.truncate(8);
    let visit = if log.breached {
        "They broke in."
    } else if log.visitors > 1 {
        "They came, and the boards held."
    } else if log.visitors == 1 {
        "Something tried the door."
    } else {
        "Nothing came."
    };
    let night = s.stats.nights;
    s.dawn = Some(DawnSummary {
        night,
        day: s.day(),
        headline: format!("Night {night}. Noise {}. {visit}", log.noise),
        lines,
        noise: log.noise,
        visitors: log.visitors,
        breached: log.breached,
        weather: s.weather,
        forecast: None,
    });
    s.phase = Phase::Dawn;
}

/// Everything that happens at 06:00, wherever you are.
pub fn new_day(s: &mut GameState, j: &mut Journal) {
    let day = s.day();
    // Judge the day that has ended.
    let noise_was = s.noise;
    s.stats.quietest_day = s.stats.quietest_day.min(noise_was);
    if noise_was < 20 {
        s.stats.quiet_streak += 1;
        s.stats.best_quiet_streak = s.stats.best_quiet_streak.max(s.stats.quiet_streak);
    } else {
        s.stats.quiet_streak = 0;
    }
    if noise_was < 10 {
        s.flags.insert("quiet_day".into());
    }
    s.noise = noise::decay(noise_was);
    s.stats.nights += 1;

    // Weather.
    s.weather = s.tomorrow;
    s.tomorrow = weather::roll(&mut s.rng, day + 1, s.weather);
    s.forecast = None;
    s.temp_offset = s.rng.range(-2, 2);
    j.events.push(GameEvent::Dawn { day });
    journal::write(
        s,
        j,
        Tone::Quiet,
        &format!("Day {day}. {}", weather::dawn_line(s.weather)),
    );
    if s.weather == Weather::Frost {
        codex::find(s, j, "fg_cold");
    }

    // Rain fills the barrel and waters the garden.
    let barrel = s.level(Station::Barrel);
    if s.weather == Weather::Rain && barrel >= 1 {
        let room = inventory::water_capacity(s).saturating_sub(inventory::water_stored(s));
        let n = stations::barrel_rain(barrel).min(room);
        if n > 0 {
            let id = if barrel >= 2 { "water_clean" } else { "water_dirty" };
            crate::state::inv_add(&mut s.store, id, n);
            journal::plain(s, j, &format!("The barrel has caught {n} measures of rain."));
        }
    }
    food::dawn(s, j);
    spoil(s, j);
    dog::dawn(s, j);
    barter::dawn(s, j);

    // The freeze.
    if !s.frozen && day + 7 == s.freeze_day {
        journal::write(
            s,
            j,
            Tone::Danger,
            "Ice along both banks this morning, thin as glass. The river is closing.",
        );
    }
    if !s.frozen && day >= s.freeze_day {
        s.frozen = true;
        s.stats.saw_freeze = true;
        j.events.push(GameEvent::Freeze);
        journal::write(
            s,
            j,
            Tone::Danger,
            "The river is white from bank to bank. Nothing will move on it until spring. The Kittiwake is held fast at the slip.",
        );
    }

    restock(s);
}

/// Wanderers drift back into rooms you have emptied.
fn restock(s: &mut GameState) {
    let day = s.day();
    if day < 5 {
        return;
    }
    let pct_chance = if day >= 25 { 10 } else { 6 };
    let ids: Vec<String> = s.places.keys().cloned().collect();
    for id in ids {
        let Some(def) = places::get(&id) else { continue };
        for (i, r) in def.rooms.iter().enumerate() {
            if r.danger == 0 {
                continue;
            }
            let roll = s.rng.chance(pct_chance);
            let Some(room) = s.places.get_mut(&id).and_then(|p| p.rooms.get_mut(i)) else {
                continue;
            };
            if room.rolled && room.foes.is_empty() && roll && room.restocked + 3 <= day {
                let kind = if r.flooded {
                    Creature::Drowned
                } else {
                    Creature::Stander
                };
                room.foes.push(Foe {
                    kind,
                    health: crate::content::creatures::def(kind).health,
                    band: Band::Near,
                    wait: 0,
                });
                room.restocked = day;
            }
        }
    }
}

fn spoil(s: &mut GameState, j: &mut Journal) {
    let day = s.day();
    let cold: u32 = if day >= 30 {
        40
    } else if day >= 20 {
        65
    } else {
        100
    };
    let store_pct = stations::spoil_pct(s.level(Station::Store));
    let mut lost = Vec::new();
    for at_home in [true, false] {
        let ids: Vec<String> = if at_home {
            s.store.keys().cloned().collect()
        } else {
            s.pack.keys().cloned().collect()
        };
        for id in ids {
            let Some(d) = items::get(&id) else { continue };
            if d.perish == 0 {
                continue;
            }
            let have = if at_home {
                crate::state::inv_count(&s.store, &id)
            } else {
                crate::state::inv_count(&s.pack, &id)
            };
            let p = u64::from(d.perish) * u64::from(if at_home { store_pct } else { 100 }) * u64::from(cold);
            // p is in millionths of the stack.
            let exact = u64::from(have) * p;
            let mut n = (exact / 1_000_000) as u32;
            let frac = ((exact % 1_000_000) / 10_000) as i32;
            if s.rng.chance(frac) {
                n += 1;
            }
            let n = n.min(have);
            if n > 0 {
                let inv = if at_home { &mut s.store } else { &mut s.pack };
                inv_take(inv, &id, n);
                j.events.push(GameEvent::Lost {
                    item: id.clone(),
                    qty: n,
                });
                lost.push(journal::qty(d.name, n).to_lowercase());
            }
        }
    }
    if !lost.is_empty() {
        journal::plain(s, j, &format!("Spoiled overnight: {}.", journal::list(&lost)));
    }
}
