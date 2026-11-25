//! Turn-based encounters with the Stilled, in three distance bands.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::{Band, Creature, Skill, VitalKind, Weather, backgrounds, creatures, items};
use crate::state::{DogState, Foe, GameState, MILLI, Resume, Tone, inv_take};
use crate::systems::clock::{self, Activity};
use crate::systems::{codex, dog, expedition, inventory, journal, noise, resume, skills};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move<'a> {
    Strike,
    Shoot,
    Shove,
    BackAway,
    HoldStill,
    Flee,
    Throw(&'a str),
    Use(&'a str),
}

pub const MAX_FOES: usize = 5;

fn foes(s: &GameState) -> &[Foe] {
    s.encounter.as_ref().map_or(&[], |e| e.foes.as_slice())
}

pub fn nearest(s: &GameState) -> Option<Band> {
    foes(s).iter().map(|f| f.band).max()
}

/// Index of the foe you would strike or shoot: closest, then weakest.
fn target(s: &GameState, reach: Band) -> Option<usize> {
    foes(s)
        .iter()
        .enumerate()
        .filter(|(_, f)| f.band >= reach)
        .max_by_key(|(i, f)| (f.band, -f.health, std::cmp::Reverse(*i)))
        .map(|(i, _)| i)
}

fn penalties(s: &GameState) -> i32 {
    let mut p = 0;
    if s.vitals.pts(VitalKind::Nerve) < 25 {
        p += 15;
    }
    if s.vitals.pts(VitalKind::Rest) < 20 {
        p += 10;
    }
    if !s.at_home() && s.trip.as_ref().is_some_and(|t| t.room.is_some()) && !expedition::is_lit(s) {
        p += 12;
    }
    p
}

fn in_dark(s: &GameState) -> bool {
    (!s.at_home() && s.trip.as_ref().is_some_and(|t| t.room.is_some()) && !expedition::is_lit(s))
        || (s.at_home() && s.is_dark())
}

pub fn strike_reach(s: &GameState) -> Band {
    match inventory::melee(s) {
        Some((_, w)) if w.reach => Band::Near,
        _ => Band::Close,
    }
}

pub fn strike_odds(s: &GameState) -> Option<i32> {
    target(s, strike_reach(s))?;
    let acc = inventory::melee(s).map_or(inventory::FISTS.accuracy, |(_, w)| w.accuracy);
    Some((75 + acc - penalties(s)).clamp(10, 95))
}

pub fn shoot_odds(s: &GameState) -> Option<i32> {
    let (_, w) = inventory::gun(s)?;
    let t = target(s, Band::Far)?;
    let band = foes(s)[t].band;
    let b = match band {
        Band::Close => 20,
        Band::Near => 5,
        Band::Far => -15,
    };
    Some((50 + w.accuracy + 7 * i32::from(s.skill(Skill::Marksmanship)) + b - penalties(s)).clamp(10, 95))
}

pub fn shove_odds(s: &GameState) -> Option<i32> {
    let t = target(s, Band::Close)?;
    let k = foes(s)[t].kind;
    Some(match k {
        Creature::Hollow => 45,
        Creature::Drowned => 50,
        _ => 80,
    })
}

fn has_kind(s: &GameState, k: Creature) -> bool {
    foes(s).iter().any(|f| f.kind == k)
}

pub fn back_odds(s: &GameState) -> i32 {
    let mut p = 65 + 3 * i32::from(s.skill(Skill::QuietStep));
    if has_kind(s, Creature::Runner) {
        p -= 25;
    }
    if nearest(s) == Some(Band::Close) {
        p -= 10;
    }
    p.clamp(5, 95)
}

pub fn hold_odds(s: &GameState) -> i32 {
    let hearing = foes(s)
        .iter()
        .map(|f| creatures::def(f.kind).hearing)
        .max()
        .unwrap_or(0);
    let mut p = 32 + 8 * i32::from(s.skill(Skill::QuietStep)) - hearing - s.noise as i32 / 5;
    if s.weather == Weather::Fog {
        p += 15;
    }
    if in_dark(s) {
        p += 10;
    }
    if nearest(s) == Some(Band::Close) {
        p -= 25;
    }
    if dog::with_you(s) {
        p -= 10;
    }
    p.clamp(5, 90)
}

pub fn flee_odds(s: &GameState) -> i32 {
    let mut p = match nearest(s) {
        Some(Band::Close) => 40,
        Some(Band::Near) => 60,
        _ => 85,
    } + 3 * i32::from(s.skill(Skill::QuietStep));
    if has_kind(s, Creature::Runner) {
        p -= 20;
    }
    p.clamp(5, 95)
}

pub fn throw_odds(s: &GameState) -> i32 {
    let mut p = 75;
    if has_kind(s, Creature::Listener) {
        p -= 30;
    }
    if nearest(s) == Some(Band::Close) {
        p -= 20;
    }
    p.clamp(5, 95)
}

pub fn gripped(s: &GameState) -> bool {
    s.encounter.as_ref().is_some_and(|e| e.gripped)
}

fn say(s: &mut GameState, j: &mut Journal, tone: Tone, line: &str) {
    if let Some(e) = s.encounter.as_mut() {
        e.log.push(line.to_string());
    }
    journal::write(s, j, tone, line);
}

fn kill(s: &mut GameState, j: &mut Journal, idx: usize) {
    let Some(e) = s.encounter.as_mut() else { return };
    if idx >= e.foes.len() {
        return;
    }
    let f = e.foes.remove(idx);
    s.stats.kills += 1;
    j.events.push(GameEvent::Kill { kind: Some(f.kind) });
    let line = format!("The {} goes down and stays down.", creatures::def(f.kind).name);
    say(s, j, Tone::Plain, &line);
}

fn hit_foe(s: &mut GameState, j: &mut Journal, idx: usize, dmg: i32) {
    let dead = match s.encounter.as_mut().and_then(|e| e.foes.get_mut(idx)) {
        Some(f) => {
            f.health -= dmg;
            f.health <= 0
        }
        None => return,
    };
    if dead {
        kill(s, j, idx);
    }
}

/// End the encounter: survivors go back to the room, then play resumes.
fn finish(s: &mut GameState, j: &mut Journal) {
    let Some(e) = s.encounter.take() else { return };
    if let (Some(i), false) = (e.room, e.foes.is_empty())
        && let Some(t) = s.trip.as_ref()
    {
        let place = t.place.clone();
        if let Some(rs) = s.places.get_mut(&place).and_then(|p| p.rooms.get_mut(i)) {
            rs.foes.extend(e.foes.iter().cloned().map(|mut f| {
                f.band = Band::Near;
                f.wait = 0;
                f
            }));
        }
    }
    j.events.push(GameEvent::EncounterOver);
    if e.at_home && !e.foes.is_empty() {
        // Whatever is left goes back out into the dark.
        let _ = e.foes;
    }
    if e.at_home && e.then == Resume::Night {
        s.stats.breaches_survived += u32::from(s.night.as_ref().is_some_and(|n| n.breached));
    }
    resume(s, j, e.then);
}

pub fn act(s: &mut GameState, j: &mut Journal, mv: Move) {
    if let Some(e) = s.encounter.as_mut() {
        e.log.clear();
        e.turn += 1;
    }
    let mut stumble = false;
    let melee_bonus = backgrounds::get(&s.background).map_or(0, |b| b.mods.melee);
    match mv {
        Move::Strike => {
            let reach = strike_reach(s);
            let odds = strike_odds(s).unwrap_or(0);
            let weapon = inventory::melee(s);
            let (id, w) = weapon.unwrap_or(("fists", inventory::FISTS));
            noise::make(s, j, w.noise, noise::Kind::Other);
            if let Some(t) = target(s, reach) {
                let name = creatures::def(foes(s)[t].kind).name;
                if s.rng.chance(odds) {
                    let dmg = (s.rng.range(w.dmg_min, w.dmg_max) + melee_bonus).max(1);
                    say(s, j, Tone::Plain, &format!("You hit the {name}."));
                    hit_foe(s, j, t, dmg);
                } else {
                    say(s, j, Tone::Plain, &format!("You swing at the {name} and miss."));
                }
                if weapon.is_some() {
                    inventory::wear_out(s, j, id, w.durability);
                }
            }
        }
        Move::Shoot => {
            let odds = shoot_odds(s).unwrap_or(0);
            if let Some((id, w)) = inventory::gun(s) {
                if let Some(a) = w.ammo {
                    inv_take(s.inv_mut(), a, 1);
                }
                s.stats.shots += 1;
                j.events.push(GameEvent::Shot);
                noise::make(s, j, w.noise, noise::Kind::Other);
                skills::practise(s, j, Skill::Marksmanship, 1);
                if let Some(t) = target(s, Band::Far) {
                    let band = foes(s)[t].band;
                    let name = creatures::def(foes(s)[t].kind).name;
                    if s.rng.chance(odds) {
                        let mut dmg = s.rng.range(w.dmg_min, w.dmg_max);
                        if id == "shotgun" && band == Band::Far {
                            dmg /= 2;
                        }
                        say(
                            s,
                            j,
                            Tone::Danger,
                            &format!("The shot is enormous in the closed air. It takes the {name}."),
                        );
                        hit_foe(s, j, t, dmg);
                    } else {
                        say(s, j, Tone::Danger, "The shot is enormous in the closed air. You miss.");
                    }
                }
                inventory::wear_out(s, j, id, w.durability);
                if !s.at_home() && foes(s).len() < MAX_FOES && s.rng.chance(35) {
                    push_foe(s, Creature::Stander, Band::Far);
                    say(s, j, Tone::Danger, "Somewhere close, something heard it and is coming.");
                }
            }
        }
        Move::Shove => {
            let odds = shove_odds(s).unwrap_or(0);
            noise::make(s, j, 2, noise::Kind::Other);
            s.vitals.add(VitalKind::Rest, -3 * MILLI);
            if let Some(t) = target(s, Band::Close) {
                if s.rng.chance(odds) {
                    if let Some(e) = s.encounter.as_mut() {
                        e.foes[t].band = Band::Near;
                        e.foes[t].wait = 0;
                        e.gripped = false;
                    }
                    say(s, j, Tone::Plain, "You shove it back and it staggers.");
                } else {
                    say(s, j, Tone::Plain, "You shove and it does not give.");
                }
            }
        }
        Move::BackAway => {
            let all_far = foes(s).iter().all(|f| f.band == Band::Far);
            if s.rng.chance(back_odds(s)) {
                if all_far {
                    say(s, j, Tone::Plain, "You back out of the reach of their hearing.");
                    finish(s, j);
                    return;
                }
                if let Some(e) = s.encounter.as_mut() {
                    for f in &mut e.foes {
                        f.band = match f.band {
                            Band::Close => Band::Near,
                            _ => Band::Far,
                        };
                        f.wait = 0;
                    }
                }
                say(s, j, Tone::Plain, "You back away, slow, and put distance between you.");
            } else {
                say(s, j, Tone::Plain, "You back into something and lose your footing.");
                stumble = true;
            }
        }
        Move::HoldStill => {
            if s.rng.chance(hold_odds(s)) {
                skills::practise(s, j, Skill::QuietStep, 2);
                say(
                    s,
                    j,
                    Tone::Plain,
                    "You do not breathe. They lose the sound of you, and stand still again.",
                );
                finish(s, j);
                return;
            }
            skills::practise(s, j, Skill::QuietStep, 1);
            say(s, j, Tone::Plain, "You hold still. They keep coming.");
        }
        Move::Flee => {
            noise::make(s, j, 4, noise::Kind::Movement);
            s.vitals.add(VitalKind::Rest, -5 * MILLI);
            if s.rng.chance(flee_odds(s)) {
                let close = nearest(s) == Some(Band::Close);
                say(s, j, Tone::Plain, "You run.");
                if dog::with_you(s) && close && s.rng.chance(25) {
                    journal::danger(s, j, "The dog does not follow you.");
                    dog::lose(s, j, false);
                }
                let in_room = s
                    .encounter
                    .as_ref()
                    .is_some_and(|e| e.then == Resume::Site && e.room.is_some());
                finish(s, j);
                if in_room && !s.ended() {
                    if let Some(t) = s.trip.as_mut() {
                        t.room = match t.room {
                            Some(0) | None => None,
                            Some(i) => Some(i - 1),
                        };
                    }
                    clock::pass(s, j, 2, Activity::Exertion);
                }
                return;
            }
            say(s, j, Tone::Danger, "You turn to run and they are on you.");
            stumble = true;
        }
        Move::Throw(item) => {
            let n = items::get(item).map_or(0, |d| d.throw);
            if inv_take(s.inv_mut(), item, 1) == 1 {
                noise::make(s, j, n, noise::Kind::Other);
                if s.rng.chance(throw_odds(s)) {
                    let far_off: Vec<usize> = foes(s)
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| f.band != Band::Close)
                        .map(|(i, _)| i)
                        .collect();
                    let lured = far_off.len();
                    let leftover = foes(s).len() - lured;
                    if leftover == 0 {
                        say(
                            s,
                            j,
                            Tone::Plain,
                            "It breaks somewhere off to the side. They turn toward it, all of them, and go.",
                        );
                        // The lured ones wander off: they are gone from this room.
                        if let Some(e) = s.encounter.as_mut() {
                            e.foes.clear();
                        }
                        finish(s, j);
                        return;
                    }
                    if let Some(e) = s.encounter.as_mut() {
                        let mut i = 0;
                        e.foes.retain(|_| {
                            let keep = !far_off.contains(&i);
                            i += 1;
                            keep
                        });
                    }
                    say(
                        s,
                        j,
                        Tone::Plain,
                        "It breaks off to the side and the far ones turn toward it. The close one does not.",
                    );
                } else {
                    say(s, j, Tone::Plain, "It breaks off to the side. They do not turn.");
                }
            }
        }
        Move::Use(item) => {
            crate::systems::home::use_item(s, j, item);
        }
    }
    if s.ended() {
        return;
    }
    if foes(s).is_empty() {
        say(s, j, Tone::Plain, "It is over. The quiet comes back.");
        finish(s, j);
        return;
    }
    foes_turn(s, j, stumble);
    if s.ended() {
        return;
    }
    clock::pass(s, j, 1, Activity::Exertion);
    if !s.ended() && foes(s).is_empty() {
        finish(s, j);
    }
}

fn push_foe(s: &mut GameState, kind: Creature, band: Band) {
    if let Some(e) = s.encounter.as_mut()
        && e.foes.len() < MAX_FOES
    {
        e.foes.push(Foe {
            kind,
            health: creatures::def(kind).health,
            band,
            wait: 0,
        });
    }
}

fn closer(b: Band, stride: u8) -> Band {
    let v = match b {
        Band::Far => 0,
        Band::Near => 1,
        Band::Close => 2,
    } + stride;
    match v {
        0 => Band::Far,
        1 => Band::Near,
        _ => Band::Close,
    }
}

fn foes_turn(s: &mut GameState, j: &mut Journal, stumble: bool) {
    let frost = s.weather == Weather::Frost;
    let n = foes(s).len();
    if let Some(e) = s.encounter.as_mut() {
        e.gripped = false;
    }
    // The dog goes for the closest one.
    let dog_here = s.dog.state == DogState::Companion && (s.at_home() || dog::with_you(s));
    if dog_here
        && let Some(t) = target(s, Band::Close)
        && s.rng.chance(35)
    {
        let dmg = s.rng.range(5, 9);
        say(s, j, Tone::Plain, "The dog goes in low and hangs on.");
        hit_foe(s, j, t, dmg);
    }
    for i in 0..n {
        let Some(f) = s.encounter.as_ref().and_then(|e| e.foes.get(i)).cloned() else {
            break;
        };
        let d = creatures::def(f.kind);
        if f.band == Band::Close {
            attack(s, j, i, stumble);
            if s.ended() {
                return;
            }
        } else {
            let every = d.every + u32::from(frost);
            let moved = if let Some(e) = s.encounter.as_mut() {
                let fm = &mut e.foes[i];
                fm.wait += 1;
                if fm.wait >= every {
                    fm.wait = 0;
                    fm.band = closer(fm.band, d.stride);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if moved {
                let band = s.encounter.as_ref().map_or(Band::Far, |e| e.foes[i].band);
                let line = match band {
                    Band::Close => format!("The {} is on you.", d.name),
                    _ => format!("The {} comes closer.", d.name),
                };
                say(s, j, Tone::Danger, &line);
            }
            if d.calls > 0 && s.rng.chance(d.calls) && foes(s).len() < MAX_FOES {
                push_foe(s, Creature::Stander, Band::Far);
                noise::make(s, j, 4, noise::Kind::Other);
                say(
                    s,
                    j,
                    Tone::Danger,
                    &format!("The {} makes a sound, low, and something answers it.", d.name),
                );
            }
        }
    }
}

fn attack(s: &mut GameState, j: &mut Journal, i: usize, stumble: bool) {
    let Some(f) = s.encounter.as_ref().and_then(|e| e.foes.get(i)).cloned() else {
        return;
    };
    let d = creatures::def(f.kind);
    let mut hit = d.hit;
    if s.vitals.pts(VitalKind::Rest) < 20 {
        hit += 10;
    }
    if stumble {
        hit += 10;
    }
    if dog::with_you(s) && s.rng.chance(15) {
        let dmg = s.rng.range(d.dmg_min, d.dmg_max) / 2;
        say(s, j, Tone::Danger, &format!("The {} turns on the dog.", d.name));
        dog::hurt(s, j, dmg);
        return;
    }
    if !s.rng.chance(hit) {
        say(
            s,
            j,
            Tone::Plain,
            &format!("The {} reaches for you and misses.", d.name),
        );
        return;
    }
    let dmg = s.rng.range(d.dmg_min, d.dmg_max);
    s.harm = f.kind.id().to_string();
    s.vitals.add(VitalKind::Health, -dmg * MILLI);
    s.vitals.add(VitalKind::Nerve, -4 * MILLI);
    j.events.push(GameEvent::Hurt { amount: dmg as u32 });
    let mut line = format!("The {} gets hold of you.", d.name);
    let mut bled = false;
    let mut infected = false;
    if s.rng.chance(d.bleed) {
        s.bleeding = (s.bleeding + 1).min(3);
        bled = true;
        line.push_str(" You are bleeding.");
    }
    if s.rng.chance(d.infect) {
        infected = s.infection == 0;
        s.infection = (s.infection + 10 * MILLI).min(100 * MILLI);
        line.push_str(" Its teeth.");
    }
    if bled || infected {
        j.events.push(GameEvent::Wounded {
            bleeding: bled,
            infected,
        });
    }
    if d.grips {
        if let Some(e) = s.encounter.as_mut() {
            e.gripped = true;
        }
        line.push_str(" It does not let go.");
    }
    say(s, j, Tone::Danger, &line);
    if !s.at_home() {
        codex::find(s, j, d.guide);
    }
    crate::systems::check_death(s, j);
}
