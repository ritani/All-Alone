//! Running choice events: picking one, checking options, applying outcomes.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::events::{self as ev, Cond, Ctx, DogIs, EventDef, Fx, Odds, Opt};
use crate::content::{Band, Creature, Station, VitalKind, creatures, items, stations};
use crate::state::{DogState, Encounter, Foe, GameState, MILLI, PendingEvent, Phase, Resume, Tone};
use crate::systems::clock::{self, Activity};
use crate::systems::{codex, dog, inventory, journal, noise, resume, skills};

fn ctx_chance(ctx: Ctx) -> i32 {
    match ctx {
        Ctx::Dawn => 32,
        Ctx::Home => 25,
        Ctx::Night => 24,
        Ctx::Road => 22,
        Ctx::Arrive => 30,
        Ctx::Radio => 40,
        Ctx::Room => 100,
    }
}

/// Why a condition is not met, or Ok.
pub fn cond(s: &GameState, c: &Cond) -> Result<(), String> {
    let ok = match c {
        Cond::MinDay(d) => s.day() >= *d,
        Cond::Flag(f) => s.has_flag(f),
        Cond::NoFlag(f) => !s.has_flag(f),
        Cond::Item(id, n) => {
            if s.count(id) < *n {
                return Err(if *n == 1 {
                    format!("Needs {}.", items::name(id).to_lowercase())
                } else {
                    format!("Needs {} x{n}.", items::name(id).to_lowercase())
                });
            }
            true
        }
        Cond::Meals(n) => {
            if inventory::meal_count(s.inv()) < *n {
                return Err(if *n == 1 {
                    "Needs food to give.".into()
                } else {
                    format!("Needs {n} meals to give.")
                });
            }
            true
        }
        Cond::Water(n) => {
            if inventory::water_count(s.inv()) < *n {
                return Err(format!("Needs {n} water."));
            }
            true
        }
        Cond::Weapon => {
            if !inventory::has_weapon(s) {
                return Err("Needs a weapon.".into());
            }
            true
        }
        Cond::Gun => {
            if inventory::gun(s).is_none() {
                return Err("Needs a loaded gun.".into());
            }
            true
        }
        Cond::Light => {
            if inventory::light(s).is_none() {
                return Err("Needs a light.".into());
            }
            true
        }
        Cond::Dog(d) => matches!(
            (d, s.dog.state),
            (DogIs::NeverMet, DogState::Unmet)
                | (DogIs::Following, DogState::Following)
                | (DogIs::Companion, DogState::Companion)
        ),
        Cond::Weather(w) => s.weather == *w,
        Cond::Dark => s.is_dark(),
        Cond::Station(st, l) => {
            if s.level(*st) < *l {
                return Err(format!(
                    "Needs the {} at level {l}.",
                    stations::def(*st).name.to_lowercase()
                ));
            }
            true
        }
        Cond::StationBelow(st, l) => s.level(*st) < *l,
        Cond::Codex(id) => s.codex.contains(*id),
        Cond::VitalAtLeast(k, v) => s.vitals.pts(*k) >= *v,
        Cond::Trades(n) => s.widow.trades >= *n,
        Cond::NotFrozen => !s.frozen,
    };
    if ok { Ok(()) } else { Err("Not now.".into()) }
}

fn eligible(s: &GameState, e: &EventDef, ctx: Ctx, place: Option<&str>, room: Option<&str>) -> bool {
    if e.ctx != ctx {
        return false;
    }
    if e.once && s.seen_events.contains(e.id) {
        return false;
    }
    if let Some(p) = e.place
        && Some(p) != place
    {
        return false;
    }
    if let Some(r) = e.room
        && Some(r) != room
    {
        return false;
    }
    e.req.iter().all(|c| cond(s, c).is_ok())
}

fn here(s: &GameState) -> (Option<String>, Option<String>) {
    let Some(t) = &s.trip else { return (None, None) };
    let room = t.room.and_then(|i| {
        crate::content::places::get(&t.place)
            .and_then(|p| p.rooms.get(i))
            .map(|r| r.id.to_string())
    });
    (Some(t.place.clone()), room)
}

fn start(s: &mut GameState, j: &mut Journal, id: &str, then: Resume) {
    s.event = Some(PendingEvent {
        id: id.to_string(),
        then,
    });
    s.phase = Phase::Event;
    s.seen_events.insert(id.to_string());
    if let Some(def) = ev::get(id) {
        journal::write(s, j, Tone::Plain, def.text);
    }
    j.events.push(GameEvent::EventStarted { id: id.to_string() });
}

/// Fire a due scheduled event for this context. True if one started.
pub fn maybe_scheduled(s: &mut GameState, j: &mut Journal, ctx: Ctx, then: Resume) -> bool {
    let day = s.day();
    let (place, room) = here(s);
    // Drop chains that can no longer happen.
    s.scheduled.retain(|(_, due)| due + 6 >= day);
    let mut chosen: Option<usize> = None;
    for (i, (id, due)) in s.scheduled.iter().enumerate() {
        if *due > day {
            continue;
        }
        let Some(def) = ev::get(id) else { continue };
        let ctx_ok = def.ctx == ctx || (ctx == Ctx::Dawn && def.ctx == Ctx::Home && *due < day);
        if !ctx_ok {
            continue;
        }
        let mut probe = *def;
        probe.ctx = ctx;
        probe.once = false;
        if eligible(s, &probe, ctx, place.as_deref(), room.as_deref()) {
            chosen = Some(i);
            break;
        }
    }
    if let Some(i) = chosen {
        let (id, _) = s.scheduled.remove(i);
        start(s, j, &id, then);
        return true;
    }
    false
}

/// Maybe start an event in this context. True if one started.
pub fn maybe(s: &mut GameState, j: &mut Journal, ctx: Ctx, then: Resume) -> bool {
    if s.ended() {
        return false;
    }
    if maybe_scheduled(s, j, ctx, then) {
        return true;
    }
    let (place, room) = here(s);
    let pool: Vec<&EventDef> = ev::EVENTS
        .iter()
        .filter(|e| e.weight > 0 && eligible(s, e, ctx, place.as_deref(), room.as_deref()))
        .collect();
    if pool.is_empty() {
        return false;
    }
    // Place- and room-bound events are rarer than the world, so they are not
    // diluted by the general chance.
    let bound = pool.iter().any(|e| e.place.is_some());
    if !bound && !s.rng.chance(ctx_chance(ctx)) {
        return false;
    }
    if bound && ctx != Ctx::Room && !s.rng.chance(60) {
        return false;
    }
    let weights: Vec<u32> = pool.iter().map(|e| e.weight).collect();
    let Some(i) = s.rng.weighted(&weights) else {
        return false;
    };
    let id = pool[i].id;
    start(s, j, id, then);
    true
}

pub fn odds(s: &GameState, o: &Odds) -> Option<i32> {
    let p = match o {
        Odds::Always => return None,
        Odds::Pct(p) => *p,
        Odds::Skill(sk, base, per) => base + per * i32::from(s.skill(*sk)),
        Odds::Nerve(base) => base + (s.vitals.pts(VitalKind::Nerve) - 50) / 2,
    };
    Some(p.clamp(5, 95))
}

/// Whether an option can be chosen now.
pub fn option_ok(s: &GameState, o: &Opt) -> Result<(), String> {
    for c in o.req {
        cond(s, c)?;
    }
    Ok(())
}

pub fn current(s: &GameState) -> Option<&'static EventDef> {
    s.event.as_ref().and_then(|e| ev::get(&e.id))
}

/// Resolve the chosen option of the pending event.
pub fn choose(s: &mut GameState, j: &mut Journal, index: usize) {
    let Some(pending) = s.event.take() else { return };
    let Some(def) = ev::get(&pending.id) else {
        resume(s, j, pending.then);
        return;
    };
    let Some(o) = def.opts.get(index) else {
        s.event = Some(pending);
        return;
    };
    s.stats.events += 1;
    if o.minutes > 0 {
        clock::pass(s, j, clock::cost(s, o.minutes), Activity::Awake);
        if s.ended() {
            return;
        }
    }
    let success = match odds(s, &o.odds) {
        None => true,
        Some(p) => s.rng.chance(p),
    };
    let res = if success { &o.ok } else { &o.fail };
    let tone = if success { Tone::Plain } else { Tone::Danger };
    journal::write(s, j, tone, res.text);
    let mut fight: Vec<Foe> = Vec::new();
    for fx in res.fx {
        apply(s, j, fx, &mut fight);
        if s.ended() {
            return;
        }
    }
    if crate::systems::check_death(s, j) {
        return;
    }
    if s.phase == Phase::Event {
        s.phase = Phase::Home;
    }
    if !fight.is_empty() {
        begin_fight(s, j, fight, pending.then);
        return;
    }
    resume(s, j, pending.then);
}

pub fn begin_fight(s: &mut GameState, j: &mut Journal, foes: Vec<Foe>, then: Resume) {
    for f in &foes {
        let d = creatures::def(f.kind);
        codex::find(s, j, d.guide);
    }
    let room = s.trip.as_ref().and_then(|t| t.room);
    s.encounter = Some(Encounter {
        foes,
        turn: 0,
        at_home: s.at_home(),
        room: if then == Resume::Site { room } else { None },
        gripped: false,
        then,
        log: Vec::new(),
    });
    s.phase = Phase::Encounter;
    j.events.push(GameEvent::EncounterStarted);
}

fn apply(s: &mut GameState, j: &mut Journal, fx: &Fx, fight: &mut Vec<Foe>) {
    match *fx {
        Fx::Item(id, n) => {
            if n > 0 {
                inventory::gain(s, j, id, n as u32);
            } else if n < 0 {
                let took = crate::state::inv_take(s.inv_mut(), id, n.unsigned_abs());
                if took > 0 {
                    j.events.push(GameEvent::Lost {
                        item: id.to_string(),
                        qty: took,
                    });
                }
            }
        }
        Fx::Meals(n) => {
            inventory::take_meals(s, j, n);
        }
        Fx::Water(n) => {
            inventory::take_water(s, j, n);
        }
        Fx::Vital(k, d) => {
            if k == VitalKind::Health && d < 0 {
                s.harm = "fall".into();
                j.events.push(GameEvent::Hurt {
                    amount: d.unsigned_abs(),
                });
            }
            s.vitals.add(k, d * MILLI);
        }
        Fx::Noise(n) => {
            noise::make(s, j, n.max(0) as u32, noise::Kind::Other);
        }
        Fx::Flag(f) => {
            s.flags.insert(f.to_string());
        }
        Fx::Unflag(f) => {
            s.flags.remove(f);
        }
        Fx::Schedule(id, days) => {
            let due = s.day() + days;
            if !s.scheduled.iter().any(|(e, _)| e == id) {
                s.scheduled.push((id.to_string(), due));
            }
        }
        Fx::Reveal(p) => {
            codex::reveal(s, j, p);
        }
        Fx::Codex(c) => {
            codex::find(s, j, c);
        }
        Fx::Fight(kind, n, band) => {
            for _ in 0..n {
                fight.push(Foe {
                    kind,
                    health: creatures::def(kind).health,
                    band,
                    wait: 0,
                });
            }
        }
        Fx::Dog(d) => dog::apply(s, j, d),
        Fx::Bleed(n) => {
            s.bleeding = (s.bleeding + n).min(3);
            j.events.push(GameEvent::Wounded {
                bleeding: true,
                infected: false,
            });
        }
        Fx::Infect(n) => {
            s.infection = (s.infection + n * MILLI).clamp(0, 100 * MILLI);
            j.events.push(GameEvent::Wounded {
                bleeding: false,
                infected: true,
            });
        }
        Fx::Xp(sk, n) => skills::practise(s, j, sk, n),
        Fx::Barricade(n) => {
            let max = stations::barricade_max(s.level(Station::Barricade)) as i32;
            s.barricade = (s.barricade as i32 + n).clamp(0, max) as u32;
        }
        Fx::Minutes(m) => clock::pass(s, j, m, Activity::Awake),
        Fx::Loot(list) => {
            let mut got = Vec::new();
            for (id, p, lo, hi) in list {
                if s.rng.chance(*p) {
                    let n = s.rng.range(*lo as i32, *hi as i32).max(1) as u32;
                    inventory::gain(s, j, id, n);
                    got.push(journal::qty(items::name(id), n).to_lowercase());
                }
            }
            if !got.is_empty() {
                journal::find(s, j, &format!("You take {}.", journal::list(&got)));
            }
        }
        Fx::WidowStock(id) => {
            if !s.widow.extra.iter().any(|x| x == id) {
                s.widow.extra.push(id.to_string());
            }
        }
        Fx::Shot => {
            if let Some((_, w)) = inventory::gun(s)
                && let Some(a) = w.ammo
            {
                crate::state::inv_take(s.inv_mut(), a, 1);
                s.stats.shots += 1;
                j.events.push(GameEvent::Shot);
            }
        }
        Fx::Kill => {
            s.stats.kills += 1;
            j.events.push(GameEvent::Kill { kind: None });
        }
        Fx::Rob(n) => {
            let lost = inventory::rob(s, j, n);
            if !lost.is_empty() {
                journal::danger(s, j, &format!("Gone: {}.", journal::list(&lost)));
            }
        }
    }
}

/// Used by tests and the bot: the band a fight starts at.
pub fn default_band() -> Band {
    Band::Near
}

/// Field-guide kinds are found on sight; this is for completeness checks.
pub fn kinds() -> &'static [Creature] {
    Creature::ALL
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameConfig, new_game};

    #[test]
    fn every_event_can_start_and_resolve_every_option() {
        for e in ev::EVENTS {
            for (i, _) in e.opts.iter().enumerate() {
                for seed in 0..4 {
                    let mut s = new_game(&GameConfig {
                        seed,
                        difficulty: crate::content::Difficulty::Still,
                        background: "clerk".into(),
                        keepsake: None,
                        daily: None,
                        held_marks: vec![],
                    })
                    .unwrap();
                    let mut j = Journal::default();
                    for (id, n) in [
                        ("revolver", 1),
                        ("rounds", 6),
                        ("bandage", 2),
                        ("rag", 2),
                        ("snare", 1),
                        ("candle", 2),
                        ("firewood", 4),
                        ("plank", 2),
                        ("nails", 4),
                        ("tinned_meat", 6),
                        ("water_clean", 4),
                        ("kitchen_knife", 1),
                    ] {
                        crate::state::inv_add(&mut s.store, id, n);
                    }
                    start(&mut s, &mut j, e.id, Resume::Home);
                    assert_eq!(s.phase, Phase::Event);
                    choose(&mut s, &mut j, i);
                    assert_ne!(s.phase, Phase::Event, "{} option {i} left the event open", e.id);
                }
            }
        }
    }
}
