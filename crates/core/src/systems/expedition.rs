//! Expeditions: packing, the walk, rooms, searching, locks, light, and
//! carrying things home.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::events::Ctx;
use crate::content::places::{self, PlaceDef, RoomDef};
use crate::content::{Band, Creature, Skill, VitalKind, Weather, creatures, items};
use crate::rng::Rng;
use crate::state::{DogState, Foe, GameState, Inv, MILLI, Phase, Resume, SearchState, Tone, Trip, inv_add, inv_take};
use crate::systems::clock::{self, Activity};
use crate::systems::{codex, dog, events, inventory, journal, noise, pct, skills};

pub fn plan(s: &mut GameState, j: &mut Journal, place: &str) {
    s.dest = Some(place.to_string());
    s.phase = Phase::Packing;
    s.flags.remove("take_dog");
    if s.dog.state == DogState::Companion {
        s.flags.insert("take_dog".into());
    }
    let name = places::get(place).map_or(place, |p| p.name);
    journal::plain(s, j, &format!("You lay out what to take to {name}."));
}

pub fn cancel(s: &mut GameState) {
    inventory::unpack_all(s);
    s.dest = None;
    s.phase = Phase::Home;
}

pub fn depart(s: &mut GameState, j: &mut Journal) {
    let Some(place) = s.dest.clone() else { return };
    let dog = s.has_flag("take_dog") && s.dog.state == DogState::Companion;
    s.trip = Some(Trip {
        place: place.clone(),
        room: None,
        dog,
        departed: s.minute,
        site_noise: 0,
    });
    s.homeward = false;
    s.phase = Phase::Travelling;
    s.stats.trips += 1;
    let name = places::get(&place).map_or("", |p| p.name);
    let with = if dog { " The dog goes ahead of you." } else { "" };
    journal::plain(s, j, &format!("You set out for {name}.{with}"));
}

/// Walking minutes for the current leg.
pub fn walk_minutes(s: &GameState, hurry: bool) -> u32 {
    let Some(p) = s.dest.as_deref().and_then(places::get) else {
        return 0;
    };
    let mut m = p.minutes;
    if s.weather == Weather::Snow {
        m = m * 13 / 10;
    }
    if hurry {
        m = m * 6 / 10;
    }
    clock::cost(s, m)
}

pub fn walk(s: &mut GameState, j: &mut Journal, hurry: bool) {
    let m = walk_minutes(s, hurry);
    let metres = s.dest.as_deref().and_then(places::get).map_or(0, |p| p.minutes * 80);
    clock::pass(s, j, m, if hurry { Activity::Exertion } else { Activity::Awake });
    if s.ended() {
        return;
    }
    s.stats.metres = s.stats.metres.saturating_add(metres);
    if hurry {
        noise::make(s, j, 6, noise::Kind::Movement);
    }
    let then = if s.homeward { Resume::ArriveHome } else { Resume::Arrive };
    if events::maybe(s, j, Ctx::Road, then) {
        return;
    }
    if s.homeward {
        arrive_home(s, j);
    } else {
        arrive(s, j);
    }
}

pub fn turn_back(s: &mut GameState, j: &mut Journal) {
    s.trip = None;
    inventory::unpack_all(s);
    s.dest = None;
    s.phase = Phase::Home;
    journal::plain(s, j, "You think better of it and go back inside.");
}

pub fn arrive(s: &mut GameState, j: &mut Journal) {
    let Some(place) = s.trip.as_ref().map(|t| t.place.clone()) else {
        s.phase = Phase::Home;
        return;
    };
    let Some(def) = places::get(&place) else { return };
    s.phase = Phase::Site;
    if let Some(t) = s.trip.as_mut() {
        t.room = None;
    }
    let first = s.places.get(&place).is_some_and(|p| !p.visited);
    if let Some(p) = s.places.get_mut(&place) {
        p.visited = true;
        p.known = true;
        for r in &mut p.rooms {
            r.lit = false;
        }
    }
    if first {
        journal::plain(s, j, &format!("{}. {}", def.name, def.desc));
    } else {
        journal::plain(s, j, &format!("You reach {}.", def.name));
    }
    if place == "weir" {
        codex::find(s, j, "fg_water");
        if crate::systems::barter::present(s) {
            journal::plain(s, j, "The widow is on the weir steps with her handcart.");
        }
    }
    events::maybe(s, j, Ctx::Arrive, Resume::Site);
}

pub fn leave(s: &mut GameState, j: &mut Journal) {
    let depth = s.trip.as_ref().and_then(|t| t.room).map_or(0, |r| r as u32 + 1);
    if depth > 0 {
        clock::pass(s, j, depth * 4, Activity::Awake);
        if s.ended() {
            return;
        }
    }
    if let Some(t) = s.trip.as_mut() {
        t.room = None;
    }
    s.homeward = true;
    s.phase = Phase::Travelling;
    journal::plain(s, j, "You turn for home.");
}

pub fn arrive_home(s: &mut GameState, j: &mut Journal) {
    let had_dog = s.trip.as_ref().is_some_and(|t| t.dog);
    s.trip = None;
    s.dest = None;
    inventory::unpack_all(s);
    s.phase = Phase::Home;
    let t = s.time_of_day();
    if t < crate::state::DAWN {
        s.stats.night_return = true;
    }
    journal::plain(
        s,
        j,
        if had_dog {
            "You are home. The dog drinks from the lock and lies down."
        } else {
            "You are home."
        },
    );
    if s.house_breached {
        s.house_breached = false;
        journal::danger(
            s,
            j,
            "The door is broken in. They were here in the night, and things are missing.",
        );
    }
    if !s.is_dark() {
        events::maybe(s, j, Ctx::Home, Resume::Home);
    }
}

// ---------------------------------------------------------------- Rooms

pub fn place_def(s: &GameState) -> Option<&'static PlaceDef> {
    s.trip.as_ref().and_then(|t| places::get(&t.place))
}

pub fn current_room(s: &GameState) -> Option<(usize, &'static RoomDef)> {
    let t = s.trip.as_ref()?;
    let i = t.room?;
    Some((i, places::get(&t.place)?.rooms.get(i)?))
}

pub fn next_room(s: &GameState) -> Option<(usize, &'static RoomDef)> {
    let t = s.trip.as_ref()?;
    let i = t.room.map_or(0, |r| r + 1);
    Some((i, places::get(&t.place)?.rooms.get(i)?))
}

pub fn room_state(s: &GameState, i: usize) -> Option<&crate::state::RoomState> {
    let t = s.trip.as_ref()?;
    s.places.get(&t.place)?.rooms.get(i)
}

fn room_state_mut(s: &mut GameState, i: usize) -> Option<&mut crate::state::RoomState> {
    let place = s.trip.as_ref()?.place.clone();
    s.places.get_mut(&place)?.rooms.get_mut(i)
}

/// Is the next room's door shut to you?
pub fn next_locked(s: &GameState) -> bool {
    match next_room(s) {
        Some((i, r)) => r.lock.is_some() && !room_state(s, i).is_some_and(|rs| rs.unlocked),
        None => false,
    }
}

pub fn is_lit(s: &GameState) -> bool {
    match current_room(s) {
        Some((i, r)) => !r.dark || room_state(s, i).is_some_and(|rs| rs.lit),
        None => true,
    }
}

fn place_index(place: &str) -> u32 {
    places::index(place).unwrap_or(0) as u32
}

/// Decide who is in a room the first time you go in.
fn roll_foes(s: &mut GameState, i: usize, r: &RoomDef) {
    let place = s.trip.as_ref().map(|t| t.place.clone()).unwrap_or_default();
    let mut rr = Rng::derived(s.seed, &format!("foes:{place}"), i as u32);
    let mut n = match r.danger {
        0 => 0,
        1 => u32::from(rr.chance(55)),
        2 => 1 + u32::from(rr.chance(50)),
        _ => 2 + u32::from(rr.chance(40)),
    };
    let day = s.day();
    if r.danger > 0 && day >= 18 && s.rng.chance(45) {
        n += 1;
    }
    if r.danger > 0 && day >= 32 && s.rng.chance(45) {
        n += 1;
    }
    if r.danger > 0 && s.difficulty == crate::content::Difficulty::DeadOfWinter && s.rng.chance(30) {
        n += 1;
    }
    if s.difficulty == crate::content::Difficulty::Quiet && n > 0 && s.rng.chance(35) {
        n -= 1;
    }
    // The first days, most of the town is still deep in it.
    if day <= 4 && r.danger < 3 && n > 0 && s.rng.chance(50) {
        n -= 1;
    }
    let mut foes = Vec::new();
    for _ in 0..n.min(4) {
        let kind = if r.kinds.is_empty() {
            let w = [
                70,
                15,
                if day >= 6 { 15 } else { 5 },
                0,
                if day >= 26 { 8 } else { 0 },
                0,
            ];
            Creature::ALL[rr.weighted(&w).unwrap_or(0)]
        } else {
            let w: Vec<u32> = r
                .kinds
                .iter()
                .map(|(k, w)| if *k == Creature::Hollow && day < 20 { 0 } else { *w })
                .collect();
            match rr.weighted(&w) {
                Some(x) => r.kinds[x].0,
                None => Creature::Stander,
            }
        };
        foes.push(Foe {
            kind,
            health: creatures::def(kind).health,
            band: Band::Near,
            wait: 0,
        });
    }
    if let Some(rs) = room_state_mut(s, i) {
        rs.rolled = true;
        rs.foes = foes;
    }
}

/// Percent chance dormant ones wake when you do something of this loudness.
pub fn wake_chance(s: &GameState, base: i32) -> i32 {
    if s.is_dark() {
        return 100;
    }
    let mut p = base + s.noise as i32 / 6 + s.trip.as_ref().map_or(0, |t| t.site_noise as i32 / 4);
    if s.day() <= 5 {
        p -= 3;
    }
    if s.weather == Weather::Fog {
        p -= 5;
    }
    if s.weather == Weather::Frost {
        p -= 5;
    }
    pct(p, s.difficulty.threat_pct()).clamp(2, 95)
}

fn further(b: Band) -> Band {
    match b {
        Band::Close => Band::Near,
        _ => Band::Far,
    }
}

/// Wake whoever is in room `i` and start a fight at `band`.
fn wake_room(s: &mut GameState, j: &mut Journal, i: usize, band: Band) {
    let Some(rs) = room_state_mut(s, i) else { return };
    let mut foes = std::mem::take(&mut rs.foes);
    if foes.is_empty() {
        return;
    }
    let band = if dog::with_you(s) { further(band) } else { band };
    for f in &mut foes {
        f.band = band;
        f.wait = 0;
    }
    let first = foes[0].kind;
    if dog::with_you(s) {
        journal::write(
            s,
            j,
            Tone::Danger,
            "The dog's hackles go up a moment before you see them.",
        );
    }
    journal::write(s, j, Tone::Danger, creatures::def(first).sight);
    s.vitals.add(VitalKind::Nerve, -6 * MILLI);
    events::begin_fight(s, j, foes, Resume::Site);
    if let Some(e) = s.encounter.as_mut() {
        e.room = Some(i);
    }
}

pub fn enter_minutes(s: &GameState, quiet: bool) -> u32 {
    let flooded = next_room(s).is_some_and(|(_, r)| r.flooded);
    let base = if quiet { 15 } else { 5 } + if flooded { 10 } else { 0 };
    clock::cost(s, base)
}

pub fn enter(s: &mut GameState, j: &mut Journal, quiet: bool) {
    let Some((i, r)) = next_room(s) else { return };
    let m = enter_minutes(s, quiet);
    clock::pass(s, j, m, if r.flooded { Activity::Exertion } else { Activity::Awake });
    if s.ended() {
        return;
    }
    noise::make(s, j, if quiet { 0 } else { 4 }, noise::Kind::Movement);
    if dog::with_you(s) && s.rng.chance(6) {
        noise::make(s, j, 10, noise::Kind::Other);
        journal::danger(s, j, "The dog barks once, sharp, at nothing you can see.");
    }
    if let Some(t) = s.trip.as_mut() {
        t.room = Some(i);
    }
    let first = room_state(s, i).is_some_and(|rs| !rs.entered);
    if let Some(rs) = room_state_mut(s, i) {
        rs.entered = true;
    }
    if r.flooded {
        if inventory::waders(s) {
            journal::plain(
                s,
                j,
                &format!("{}. You wade in. The waders keep the water out.", r.name),
            );
        } else {
            s.vitals.add(VitalKind::Warmth, -15 * MILLI);
            journal::plain(
                s,
                j,
                &format!("{}. The water is to your waist and it takes your breath.", r.name),
            );
        }
    }
    if first {
        journal::plain(s, j, r.desc);
    } else if !r.flooded {
        journal::plain(s, j, &format!("You go back into the {}.", r.name.to_lowercase()));
    }
    if r.dark && !is_lit(s) {
        s.vitals.add(VitalKind::Nerve, -3 * MILLI);
    }
    if !room_state(s, i).is_some_and(|rs| rs.rolled) {
        roll_foes(s, i, r);
    }
    let count = room_state(s, i).map_or(0, |rs| rs.foes.len());
    if count > 0 {
        let dark_here = r.dark && !is_lit(s);
        let awake = dark_here || s.rng.chance(wake_chance(s, if quiet { 4 } else { 22 }));
        if awake {
            let band = if quiet {
                Band::Far
            } else if dark_here {
                Band::Close
            } else {
                Band::Near
            };
            wake_room(s, j, i, band);
            return;
        }
        codex::find(s, j, "fg_still");
        let kinds: Vec<Creature> = room_state(s, i)
            .map(|rs| rs.foes.iter().map(|f| f.kind).collect())
            .unwrap_or_default();
        let line = if kinds.len() == 1 {
            format!(
                "A {} stands here, still. It has not turned.",
                creatures::def(kinds[0]).name
            )
        } else {
            format!("{} of them stand here, still. None has turned.", kinds.len())
        };
        journal::write(s, j, Tone::Danger, &line);
        s.vitals.add(VitalKind::Nerve, -3 * MILLI);
        return;
    }
    if first {
        events::maybe(s, j, Ctx::Room, Resume::Site);
    }
}

pub fn back(s: &mut GameState, j: &mut Journal) {
    clock::pass(s, j, clock::cost(s, 5), Activity::Awake);
    if let Some(t) = s.trip.as_mut() {
        t.room = match t.room {
            Some(0) | None => None,
            Some(i) => Some(i - 1),
        };
    }
    match current_room(s) {
        Some((_, r)) => journal::plain(s, j, &format!("You go back to the {}.", r.name.to_lowercase())),
        None => journal::plain(s, j, "You go back outside."),
    }
}

fn roll_loot(s: &mut GameState, i: usize, r: &RoomDef) -> Inv {
    let place = s.trip.as_ref().map(|t| t.place.clone()).unwrap_or_default();
    let mut rr = Rng::derived(s.seed, &format!("loot:{place}"), i as u32 + 100 * place_index(&place));
    let lp = s.difficulty.loot_pct();
    let mut out = Inv::new();
    for l in r.loot {
        if rr.chance(pct(l.pct, lp)) {
            let n = rr.range(l.min as i32, l.max as i32).max(1) as u32;
            inv_add(&mut out, l.item, n);
        }
    }
    // After dusk the better finds: what others left for the dark.
    if s.is_dark() {
        for l in r.loot {
            if s.rng.chance(l.pct / 2) {
                inv_add(&mut out, l.item, 1);
            }
        }
    }
    for (id, n) in r.fixed {
        inv_add(&mut out, id, *n);
    }
    out
}

pub fn search_minutes(s: &GameState, careful: bool) -> u32 {
    let Some((_, r)) = current_room(s) else { return 0 };
    let mut m = if careful { r.search } else { r.search / 3 };
    if !is_lit(s) {
        m = m * 3 / 2;
    }
    clock::cost(s, m)
}

pub fn search(s: &mut GameState, j: &mut Journal, careful: bool) {
    let Some((i, r)) = current_room(s) else { return };
    let m = search_minutes(s, careful);
    clock::pass(s, j, m, Activity::Awake);
    if s.ended() {
        return;
    }
    noise::make(s, j, if careful { 1 } else { 6 }, noise::Kind::Movement);
    let lit = is_lit(s);
    let first = room_state(s, i).is_some_and(|rs| rs.search == SearchState::None);
    if first {
        let loot = roll_loot(s, i, r);
        if let Some(rs) = room_state_mut(s, i) {
            rs.hidden = loot;
        }
        s.stats.rooms_searched += 1;
    }
    let forage = i32::from(s.skill(Skill::Foraging));
    let find_pct = match (careful, lit) {
        (true, true) => 100,
        (true, false) => 60 + forage * 5,
        (false, true) => 65 + forage * 5,
        (false, false) => 40 + forage * 5,
    };
    let hidden: Vec<(String, u32)> = room_state(s, i)
        .map(|rs| rs.hidden.iter().map(|(k, v)| (k.clone(), *v)).collect())
        .unwrap_or_default();
    let mut found = Inv::new();
    for (id, n) in hidden {
        let fixed = r.fixed.iter().any(|(f, _)| *f == id);
        let mut got = 0;
        for _ in 0..n {
            if fixed || find_pct >= 100 || s.rng.chance(find_pct) {
                got += 1;
            }
        }
        if got > 0 {
            inv_add(&mut found, &id, got);
        }
    }
    let mut names = Vec::new();
    let papers_done = (careful && lit) || r.codex.iter().all(|c| s.codex.contains(*c));
    if let Some(rs) = room_state_mut(s, i) {
        for (id, n) in &found {
            inv_take(&mut rs.hidden, id, *n);
            inv_add(&mut rs.pile, id, *n);
        }
        rs.search = if rs.hidden.is_empty() && papers_done {
            SearchState::Full
        } else {
            SearchState::Partial
        };
    }
    for (id, n) in &found {
        names.push(journal::qty(items::name(id), *n).to_lowercase());
    }
    if careful {
        skills::practise(s, j, Skill::Foraging, 1);
    }
    let how = if careful {
        "You go through the room slowly"
    } else {
        "You turn the room over fast"
    };
    if names.is_empty() {
        journal::plain(s, j, &format!("{how}. Nothing worth taking."));
    } else {
        journal::find(s, j, &format!("{how}. You find {}.", journal::list(&names)));
    }
    if careful && lit {
        for c in r.codex {
            codex::find(s, j, c);
        }
    } else if !r.codex.iter().all(|c| s.codex.contains(*c)) && !careful {
        journal::plain(s, j, "There are papers here you did not stop to read.");
    }
    // Dormant ones may stir.
    if room_state(s, i).is_some_and(|rs| !rs.foes.is_empty()) {
        let base = if careful { 6 } else { 30 };
        if s.rng.chance(wake_chance(s, base)) {
            journal::danger(s, j, "Behind you, a slow turn of the head.");
            wake_room(s, j, i, if careful { Band::Near } else { Band::Close });
        }
    }
}

/// The dark room a light would be lit for: this one, or the one ahead.
pub fn light_target(s: &GameState) -> Option<usize> {
    if let Some((i, r)) = current_room(s)
        && r.dark
        && !room_state(s, i).is_some_and(|rs| rs.lit)
    {
        return Some(i);
    }
    match next_room(s) {
        Some((i, r)) if r.dark && !room_state(s, i).is_some_and(|rs| rs.lit) => Some(i),
        _ => None,
    }
}

pub fn light_up(s: &mut GameState, j: &mut Journal) {
    let Some(i) = light_target(s) else { return };
    let here = current_room(s).is_some_and(|(c, _)| c == i);
    if let Some(lamp) = inventory::burn_light(s, j) {
        if let Some(rs) = room_state_mut(s, i) {
            rs.lit = true;
        }
        let line = if here {
            format!(
                "You light the {}. The room comes up out of the dark.",
                items::name(lamp).to_lowercase()
            )
        } else {
            format!("You light the {} before you go in.", items::name(lamp).to_lowercase())
        };
        journal::plain(s, j, &line);
    }
}

pub fn pick_odds(s: &GameState) -> i32 {
    let diff = next_room(s)
        .and_then(|(_, r)| r.lock)
        .and_then(|l| l.pick)
        .unwrap_or(100);
    (60 + 10 * i32::from(s.skill(Skill::Lockwork)) - diff).clamp(5, 95)
}

pub fn pick_minutes(s: &GameState) -> u32 {
    let lw = u32::from(s.skill(Skill::Lockwork));
    clock::cost(s, (30u32.saturating_sub(lw * 4)).max(10))
}

fn unlock_next(s: &mut GameState) {
    if let Some((i, _)) = next_room(s)
        && let Some(rs) = room_state_mut(s, i)
    {
        rs.unlocked = true;
    }
}

pub fn pick_lock(s: &mut GameState, j: &mut Journal) {
    let m = pick_minutes(s);
    let odds = pick_odds(s);
    clock::pass(s, j, m, Activity::Awake);
    if s.ended() {
        return;
    }
    noise::make(s, j, 1, noise::Kind::Movement);
    if s.rng.chance(odds) {
        unlock_next(s);
        skills::practise(s, j, Skill::Lockwork, 2);
        journal::plain(s, j, "The last pin sets and the lock turns.");
    } else {
        skills::practise(s, j, Skill::Lockwork, 1);
        journal::plain(s, j, "The pick slips. The lock holds.");
        if s.rng.chance(12) {
            inv_take(s.inv_mut(), "lockpicks", 1);
            journal::danger(s, j, "The tension bar snaps in the lock.");
        }
    }
}

/// The tool you would force a door with.
pub fn force_tool(s: &GameState) -> Option<&'static str> {
    ["crowbar", "hatchet", "hammer"].into_iter().find(|t| s.count(t) > 0)
}

pub fn force_lock(s: &mut GameState, j: &mut Journal) {
    let Some(tool) = force_tool(s) else { return };
    clock::pass(s, j, clock::cost(s, 5), Activity::Exertion);
    if s.ended() {
        return;
    }
    noise::make(s, j, 18, noise::Kind::Other);
    unlock_next(s);
    journal::danger(
        s,
        j,
        &format!(
            "You put the {} to the door. It goes with a crack like a shot.",
            items::name(tool).to_lowercase()
        ),
    );
    if let Some(w) = items::weapon(tool) {
        inventory::wear_out(s, j, tool, w.durability / 3);
    }
    // Whoever is beyond the door may already be coming.
    if let Some((i, r)) = next_room(s) {
        if !room_state(s, i).is_some_and(|rs| rs.rolled) {
            roll_foes(s, i, r);
        }
    }
}

pub fn use_key(s: &mut GameState, j: &mut Journal) {
    clock::pass(s, j, 2, Activity::Awake);
    unlock_next(s);
    journal::plain(s, j, "The key turns.");
}

pub fn key_for_next(s: &GameState) -> Option<&'static str> {
    next_room(s)
        .and_then(|(_, r)| r.lock)
        .and_then(|l| l.key)
        .filter(|k| s.count(k) > 0)
}

pub fn take(s: &mut GameState, j: &mut Journal, item: &str, qty: u32) {
    let Some((i, _)) = current_room(s) else { return };
    let n = qty.min(inventory::room_for(s, item));
    let took = room_state_mut(s, i).map_or(0, |rs| inv_take(&mut rs.pile, item, n));
    if took > 0 {
        inv_add(&mut s.pack, item, took);
        j.events.push(GameEvent::Found {
            item: item.to_string(),
            qty: took,
        });
        journal::plain(
            s,
            j,
            &format!("You take {}.", journal::qty(items::name(item), took).to_lowercase()),
        );
    }
}

/// Take what fits, most valuable for its weight first.
pub fn take_all(s: &mut GameState, j: &mut Journal) {
    let Some((i, _)) = current_room(s) else { return };
    let mut pile: Vec<(String, u32)> = room_state(s, i)
        .map(|rs| rs.pile.iter().map(|(k, v)| (k.clone(), *v)).collect())
        .unwrap_or_default();
    pile.sort_by_key(|(id, _)| {
        let d = items::get(id);
        let v = d.map_or(0, |d| d.value) * 100 + if items::is_part(id) { 100_000 } else { 0 };
        let w = d.map_or(1, |d| d.weight.max(1));
        (std::cmp::Reverse(v / w), id.clone())
    });
    let mut took_any = Vec::new();
    for (id, n) in pile {
        let fit = n.min(inventory::room_for(s, &id));
        if fit == 0 {
            continue;
        }
        let took = room_state_mut(s, i).map_or(0, |rs| inv_take(&mut rs.pile, &id, fit));
        if took > 0 {
            inv_add(&mut s.pack, &id, took);
            j.events.push(GameEvent::Found {
                item: id.clone(),
                qty: took,
            });
            took_any.push(journal::qty(items::name(&id), took).to_lowercase());
        }
    }
    if took_any.is_empty() {
        journal::plain(s, j, "Nothing more will fit.");
    } else {
        journal::plain(s, j, &format!("You take {}.", journal::list(&took_any)));
    }
    if room_state(s, i).is_some_and(|rs| !rs.pile.is_empty()) {
        journal::plain(s, j, "What does not fit stays behind.");
    }
}

pub fn drop(s: &mut GameState, j: &mut Journal, item: &str, qty: u32) {
    let Some((i, _)) = current_room(s) else { return };
    let n = inv_take(&mut s.pack, item, qty);
    if n > 0 {
        if let Some(rs) = room_state_mut(s, i) {
            inv_add(&mut rs.pile, item, n);
        }
        journal::plain(
            s,
            j,
            &format!("You leave {} here.", journal::qty(items::name(item), n).to_lowercase()),
        );
    }
}

/// Damage a strike on a dormant one does.
pub fn ambush_damage(s: &GameState) -> (i32, i32) {
    let (_, w) = inventory::melee(s).unwrap_or(("", inventory::FISTS));
    let bonus = crate::content::backgrounds::get(&s.background).map_or(0, |b| b.mods.melee);
    ((w.dmg_min + bonus) * 2 + 4, (w.dmg_max + bonus) * 2 + 4)
}

pub fn ambush(s: &mut GameState, j: &mut Journal) {
    let Some((i, _)) = current_room(s) else { return };
    let weapon = inventory::melee(s);
    let (lo, hi) = ambush_damage(s);
    let dmg = s.rng.range(lo, hi);
    clock::pass(s, j, 2, Activity::Exertion);
    noise::make(s, j, weapon.map_or(2, |(_, w)| w.noise) + 2, noise::Kind::Other);
    if let Some((id, w)) = weapon {
        inventory::wear_out(s, j, id, w.durability);
    }
    let Some(rs) = room_state_mut(s, i) else { return };
    let Some(target) = rs.foes.iter_mut().min_by_key(|f| f.health) else {
        return;
    };
    target.health -= dmg;
    let kind = target.kind;
    if target.health <= 0 {
        rs.foes.retain(|f| f.health > 0);
        s.stats.kills += 1;
        j.events.push(GameEvent::Kill { kind: Some(kind) });
        codex::find(s, j, creatures::def(kind).guide);
        journal::write(
            s,
            j,
            Tone::Danger,
            &format!(
                "You come at the {} from behind. It goes down without a sound.",
                creatures::def(kind).name
            ),
        );
        if room_state(s, i).is_some_and(|rs| !rs.foes.is_empty()) && s.rng.chance(wake_chance(s, 20)) {
            journal::danger(s, j, "The others turn.");
            wake_room(s, j, i, Band::Near);
        }
    } else {
        journal::danger(
            s,
            j,
            &format!("The blow lands, but the {} turns to you.", creatures::def(kind).name),
        );
        wake_room(s, j, i, Band::Close);
    }
}
