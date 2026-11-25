//! The lock house: using things, building stations, crafting, the stove,
//! water, the radio, the ferry at the slip, and the endings.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::codex::{self, CodexKind};
use crate::content::items::{self, ItemKind, MedKind};
use crate::content::recipes::RecipeDef;
use crate::content::{Skill, Station, VitalKind, Weather, backgrounds, stations};
use crate::state::{DogState, EndKind, GameState, MILLI, Tone, inv_add, inv_count, inv_take};
use crate::systems::clock::{self, Activity};
use crate::systems::{end_run, inventory, journal, noise, pct_u, skills};

// ---------------------------------------------------------------- Using things

/// Minutes to use an item outside an encounter.
pub fn use_minutes(item: &str) -> u32 {
    match items::get(item).map(|d| d.kind) {
        Some(ItemKind::Food { sat, .. }) if sat <= 2 => 2,
        Some(ItemKind::Food { .. }) => 10,
        Some(ItemKind::Medicine(MedKind::Bandage(_))) => 10,
        Some(ItemKind::Medicine(_)) => 3,
        Some(ItemKind::Clothing { .. }) => 3,
        _ => 5,
    }
}

/// Why an item cannot be used now, or Ok.
pub fn can_use(s: &GameState, item: &str) -> Result<(), String> {
    let Some(d) = items::get(item) else {
        return Err("Unknown item.".into());
    };
    if s.count(item) == 0 {
        return Err("You have none.".into());
    }
    match d.kind {
        ItemKind::Food { sat, hyd, .. } => {
            let full_food = sat > 0 && s.vitals.pts(VitalKind::Satiety) >= 97;
            let full_drink = hyd > 0 && s.vitals.pts(VitalKind::Hydration) >= 97;
            if (sat <= 0 || full_food) && (hyd <= 0 || full_drink) && (sat > 0 || hyd > 0) {
                return Err("You could not take another mouthful.".into());
            }
            Ok(())
        }
        ItemKind::Medicine(MedKind::Bandage(_)) => {
            if s.bleeding == 0 {
                Err("You are not bleeding.".into())
            } else {
                Ok(())
            }
        }
        ItemKind::Medicine(MedKind::Antiseptic(_) | MedKind::Antibiotic(_)) => {
            if s.infection == 0 {
                Err("No wound is infected.".into())
            } else {
                Ok(())
            }
        }
        ItemKind::Medicine(MedKind::Painkiller { .. }) | ItemKind::Comfort { .. } => Ok(()),
        ItemKind::Clothing { .. } => Ok(()),
        _ => Err("You cannot use that.".into()),
    }
}

pub fn bandage_odds(s: &GameState, base: i32) -> i32 {
    let shelf = if s.at_home() && s.level(Station::Medicine) >= 1 {
        10
    } else {
        0
    };
    (base + 6 * i32::from(s.skill(Skill::FieldMedicine)) + shelf).clamp(5, 98)
}

pub fn use_item(s: &mut GameState, j: &mut Journal, item: &str) {
    let Some(d) = items::get(item) else { return };
    if can_use(s, item).is_err() {
        return;
    }
    if let ItemKind::Clothing { .. } = d.kind {
        inventory::wear(s, item, j);
        return;
    }
    inv_take(s.inv_mut(), item, 1);
    let fm = i32::from(s.skill(Skill::FieldMedicine));
    match d.kind {
        ItemKind::Food {
            sat,
            hyd,
            nerve,
            warm,
            sick,
        } => {
            s.vitals.add(VitalKind::Satiety, sat * MILLI);
            s.vitals.add(VitalKind::Hydration, hyd * MILLI);
            s.vitals.add(VitalKind::Nerve, nerve * MILLI);
            s.vitals.add(VitalKind::Warmth, warm * MILLI);
            let verb = if sat > 2 { "eat" } else { "drink" };
            journal::plain(s, j, &format!("You {verb} the {}.", d.name.to_lowercase()));
            if sick > 0 && s.rng.chance(sick) {
                s.sick_until = s.minute + 18 * 60;
                s.harm = "sickness".into();
                j.events.push(GameEvent::Sick);
                journal::danger(s, j, "An hour later your stomach turns over. You are sick.");
            }
        }
        ItemKind::Medicine(MedKind::Bandage(base)) => {
            s.stats.treated += 1;
            if s.rng.chance(bandage_odds(s, base)) {
                s.bleeding = 0;
                skills::practise(s, j, Skill::FieldMedicine, 2);
                journal::plain(
                    s,
                    j,
                    &format!("You bind the wound with the {}. It holds.", d.name.to_lowercase()),
                );
            } else {
                s.bleeding = s.bleeding.saturating_sub(1).max(1);
                skills::practise(s, j, Skill::FieldMedicine, 1);
                journal::danger(s, j, "You bind it, and it soaks through.");
            }
        }
        ItemKind::Medicine(MedKind::Antiseptic(n)) => {
            s.stats.treated += 1;
            s.infection = (s.infection - (n + 2 * fm) * MILLI).max(0);
            s.antiseptic_until = s.minute + 24 * 60;
            skills::practise(s, j, Skill::FieldMedicine, 1);
            journal::plain(s, j, "You clean the wound. It burns, and then it is better.");
        }
        ItemKind::Medicine(MedKind::Antibiotic(n)) => {
            s.stats.treated += 1;
            s.infection = (s.infection - (n + 3 * fm) * MILLI).max(0);
            if s.infection < 20 * MILLI {
                s.infection = 0;
            }
            skills::practise(s, j, Skill::FieldMedicine, 1);
            journal::plain(
                s,
                j,
                "You take the tablets. By evening the heat has gone out of the wound.",
            );
        }
        ItemKind::Medicine(MedKind::Painkiller { health, nerve }) => {
            s.vitals.add(VitalKind::Health, health * MILLI);
            s.vitals.add(VitalKind::Nerve, nerve * MILLI);
            journal::plain(
                s,
                j,
                &format!("You take the {}. The edges soften.", d.name.to_lowercase()),
            );
        }
        ItemKind::Comfort { nerve } => {
            s.vitals.add(VitalKind::Nerve, nerve * MILLI);
            journal::plain(s, j, &format!("You take a moment with the {}.", d.name.to_lowercase()));
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- Building and crafting

fn has_tool(s: &GameState, tag: &str) -> bool {
    s.store.keys().any(|id| items::has_tag(id, tag))
}

fn tool_name(tag: &str) -> &'static str {
    match tag {
        "build" => "a hammer",
        "chop" => "a hatchet or saw",
        "spanner" => "a spanner",
        "force" => "a crowbar",
        "pick" => "lockpicks",
        _ => "a tool",
    }
}

fn missing(s: &GameState, cost: &[(&str, u32)]) -> Vec<String> {
    cost.iter()
        .filter(|(id, n)| inv_count(&s.store, id) < *n)
        .map(|(id, n)| format!("{} {}/{n}", items::name(id).to_lowercase(), inv_count(&s.store, id)))
        .collect()
}

fn work_minutes(s: &GameState, base: u32) -> u32 {
    let mut p = 100 - 7 * u32::from(s.skill(Skill::Mending));
    if s.level(Station::Workbench) >= 3 {
        p -= 10;
    }
    clock::cost(s, pct_u(base, p).max(5))
}

fn work_noise(s: &GameState, base: u32) -> u32 {
    let extra = backgrounds::get(&s.background).map_or(0, |b| b.mods.work_noise);
    pct_u(base, (100 + extra).max(0) as u32)
}

pub fn build_minutes(s: &GameState, st: Station) -> u32 {
    let lvl = s.level(st);
    if lvl >= 3 {
        return 0;
    }
    work_minutes(s, stations::def(st).levels[usize::from(lvl) + 1].minutes)
}

pub fn build_noise(s: &GameState, st: Station) -> u32 {
    let lvl = s.level(st);
    if lvl >= 3 {
        return 0;
    }
    work_noise(s, stations::def(st).levels[usize::from(lvl) + 1].noise)
}

pub fn can_build(s: &GameState, st: Station) -> Result<(), String> {
    let lvl = s.level(st);
    if lvl >= 3 {
        return Err("Built as far as it goes.".into());
    }
    let next = &stations::def(st).levels[usize::from(lvl) + 1];
    if s.level(Station::Workbench) < next.workbench && st != Station::Workbench {
        return Err(format!("Needs the workbench at level {}.", next.workbench));
    }
    if st == Station::Workbench && lvl >= 1 && s.level(Station::Workbench) < next.workbench {
        return Err(format!("Needs the workbench at level {}.", next.workbench));
    }
    if !has_tool(s, "build") {
        return Err("Needs a hammer.".into());
    }
    let m = missing(s, next.cost);
    if !m.is_empty() {
        return Err(format!("Needs {}.", journal::list(&m)));
    }
    Ok(())
}

pub fn build(s: &mut GameState, j: &mut Journal, st: Station) {
    let lvl = s.level(st);
    let Some(next) = stations::def(st).levels.get(usize::from(lvl) + 1) else {
        return;
    };
    let m = build_minutes(s, st);
    let n = build_noise(s, st);
    for (id, c) in next.cost {
        inv_take(&mut s.store, id, *c);
    }
    clock::pass(s, j, m, Activity::Exertion);
    if s.ended() {
        return;
    }
    noise::make(s, j, n, noise::Kind::Other);
    s.stations.insert(st, lvl + 1);
    if st == Station::Barricade {
        s.barricade = stations::barricade_max(lvl + 1);
    }
    s.stats.built += 1;
    skills::practise(s, j, Skill::Mending, 2);
    j.events.push(GameEvent::Built {
        station: st,
        level: lvl + 1,
    });
    journal::find(s, j, &build_line(stations::def(st).name, next.name, next.effect));
}

/// "Stove: Iron stove. ..." but "Workbench built. ..." when the level's name
/// already says which station it is.
pub fn build_line(station: &str, level: &str, effect: &str) -> String {
    if level.to_lowercase().contains(&station.to_lowercase()) {
        format!("{level} built. {effect}")
    } else {
        format!("{station}: {level}. {effect}")
    }
}

pub fn can_craft(s: &GameState, r: &RecipeDef) -> Result<(), String> {
    if let Some((st, lvl)) = r.station
        && s.level(st) < lvl
    {
        return Err(format!(
            "Needs the {} at level {lvl}.",
            stations::def(st).name.to_lowercase()
        ));
    }
    for t in r.tools {
        if !has_tool(s, t) {
            return Err(format!("Needs {}.", tool_name(t)));
        }
    }
    let m = missing(s, r.inputs);
    if !m.is_empty() {
        return Err(format!("Needs {}.", journal::list(&m)));
    }
    if r.fire && s.fire_until <= s.minute {
        return Err("Needs the stove lit.".into());
    }
    Ok(())
}

pub fn craft_minutes(s: &GameState, r: &RecipeDef) -> u32 {
    let cooking = matches!(r.station, Some((Station::Stove, _)));
    if cooking {
        clock::cost(s, r.minutes)
    } else {
        work_minutes(s, r.minutes)
    }
}

pub fn craft_noise(s: &GameState, r: &RecipeDef) -> u32 {
    work_noise(s, r.noise)
}

pub fn craft(s: &mut GameState, j: &mut Journal, r: &RecipeDef) {
    let m = craft_minutes(s, r);
    let n = craft_noise(s, r);
    for (id, c) in r.inputs {
        inv_take(&mut s.store, id, *c);
    }
    clock::pass(s, j, m, Activity::Awake);
    if s.ended() {
        return;
    }
    noise::make(s, j, n, noise::Kind::Other);
    let mut got = Vec::new();
    for (id, c) in r.outputs {
        inv_add(&mut s.store, id, *c);
        j.events.push(GameEvent::Found {
            item: id.to_string(),
            qty: *c,
        });
        got.push(journal::qty(items::name(id), *c).to_lowercase());
    }
    s.stats.crafted += 1;
    match r.station {
        Some((Station::Medicine, _)) => skills::practise(s, j, Skill::FieldMedicine, 1),
        Some((Station::Stove, _)) => {}
        _ => skills::practise(s, j, Skill::Mending, 1),
    }
    j.events.push(GameEvent::Crafted {
        recipe: r.id.to_string(),
    });
    journal::plain(s, j, &format!("{}: {}.", r.name, journal::list(&got)));
}

// ---------------------------------------------------------------- House work

pub const BARRICADE_REPAIR: u32 = 25;

pub fn can_repair_barricade(s: &GameState) -> Result<(), String> {
    let max = stations::barricade_max(s.level(Station::Barricade));
    if s.barricade >= max {
        return Err("The barricade is sound.".into());
    }
    if !has_tool(s, "build") {
        return Err("Needs a hammer.".into());
    }
    let m = missing(s, &[("plank", 1), ("nails", 2)]);
    if !m.is_empty() {
        return Err(format!("Needs {}.", journal::list(&m)));
    }
    Ok(())
}

pub fn repair_barricade(s: &mut GameState, j: &mut Journal) {
    inv_take(&mut s.store, "plank", 1);
    inv_take(&mut s.store, "nails", 2);
    clock::pass(s, j, work_minutes(s, 40), Activity::Exertion);
    if s.ended() {
        return;
    }
    noise::make(s, j, work_noise(s, 3), noise::Kind::Other);
    let max = stations::barricade_max(s.level(Station::Barricade));
    s.barricade = (s.barricade + BARRICADE_REPAIR).min(max);
    skills::practise(s, j, Skill::Mending, 1);
    journal::plain(
        s,
        j,
        &format!(
            "You nail fresh boards over the weak places. The barricade holds at {} of {max}.",
            s.barricade
        ),
    );
}

pub fn fire_hours(s: &GameState) -> u32 {
    stations::fire_hours(s.level(Station::Stove))
}

pub fn feed_stove(s: &mut GameState, j: &mut Journal) {
    inv_take(&mut s.store, "firewood", 1);
    let hours = fire_hours(s);
    let from = s.fire_until.max(s.minute);
    s.fire_until = (from + hours * 60).min(s.minute + 18 * 60);
    clock::pass(s, j, 5, Activity::Awake);
    journal::plain(
        s,
        j,
        &format!(
            "You feed the stove. It will burn until {}.",
            journal::clock(s.fire_until)
        ),
    );
}

pub fn draw_amount(s: &GameState) -> u32 {
    inventory::water_capacity(s)
        .saturating_sub(inventory::water_stored(s))
        .min(6)
}

pub fn draw_minutes(s: &GameState) -> u32 {
    clock::cost(s, if s.frozen { 35 } else { 15 })
}

pub fn draw_water(s: &mut GameState, j: &mut Journal) {
    let n = draw_amount(s);
    clock::pass(s, j, draw_minutes(s), Activity::Awake);
    if s.ended() {
        return;
    }
    noise::make(s, j, if s.frozen { 5 } else { 1 }, noise::Kind::Other);
    inv_add(&mut s.store, "water_dirty", n);
    j.events.push(GameEvent::Found {
        item: "water_dirty".into(),
        qty: n,
    });
    let line = if s.frozen {
        format!(
            "You break the ice at the lock edge and fill {}, brown and cold.",
            journal::count(n, "measure", "measures")
        )
    } else {
        format!(
            "You fill {} from the lock. Brown water. Boil it.",
            journal::count(n, "measure", "measures")
        )
    };
    journal::plain(s, j, &line);
}

pub fn rest(s: &mut GameState, j: &mut Journal) {
    journal::plain(s, j, "You lie down for an hour.");
    clock::pass(s, j, 60, Activity::Asleep);
}

// ---------------------------------------------------------------- Radio

fn forecast(s: &mut GameState) {
    let acc = stations::forecast_pct(s.level(Station::Radio));
    if s.rng.chance(acc) {
        s.forecast = Some(s.tomorrow);
    } else {
        let others: Vec<Weather> = Weather::ALL.iter().copied().filter(|w| *w != s.tomorrow).collect();
        let i = s.rng.below(others.len() as u32) as usize;
        s.forecast = others.get(i).copied();
    }
}

pub fn next_fragment(s: &GameState) -> Option<&'static str> {
    let lvl = s.level(Station::Radio);
    let day = s.day();
    codex::CODEX
        .iter()
        .filter(|c| c.kind == CodexKind::Radio && c.day <= day && c.radio <= lvl && !s.codex.contains(c.id))
        .min_by_key(|c| c.day)
        .map(|c| c.id)
}

pub fn listen(s: &mut GameState, j: &mut Journal) {
    clock::pass(s, j, 30, Activity::Awake);
    if s.ended() {
        return;
    }
    s.radio_day = s.day();
    // Reception comes and goes; a better set catches more.
    let clear = [0, 55, 70, 85][usize::from(s.level(Station::Radio).min(3))];
    let heard = if s.rng.chance(clear) { next_fragment(s) } else { None };
    match heard {
        Some(id) => {
            crate::systems::codex::find(s, j, id);
            if let Some(c) = codex::get(id) {
                journal::write(s, j, Tone::Quiet, c.text);
            }
            s.vitals.add(VitalKind::Nerve, 4 * MILLI);
        }
        None => journal::plain(s, j, "Static, and under the static, nothing."),
    }
    forecast(s);
    if let Some(f) = s.forecast {
        journal::write(s, j, Tone::Quiet, crate::systems::weather::forecast_line(f));
    }
    if !s.freeze_known && (s.codex.contains("radio_ice") || s.day() + 14 >= s.freeze_day) {
        s.freeze_known = true;
        let left = s.freeze_day.saturating_sub(s.day());
        journal::write(
            s,
            j,
            Tone::Danger,
            &format!(
                "From what they say at Saltmouth, the river here will close in about {}.",
                journal::count(left, "day", "days")
            ),
        );
    }
    crate::systems::events::maybe(s, j, crate::content::events::Ctx::Radio, crate::state::Resume::Home);
}

// ---------------------------------------------------------------- The ferry

pub fn part_tool(part: &str) -> Option<&'static str> {
    match part {
        "hull_patch" => Some("build"),
        "spark_coil" | "drive_belt" => Some("spanner"),
        _ => None,
    }
}

pub fn can_install(s: &GameState, part: &str) -> Result<(), String> {
    if s.ferry.contains(part) {
        return Err("Already fitted.".into());
    }
    if s.frozen {
        return Err("The ferry is held in the ice.".into());
    }
    if inv_count(&s.store, part) == 0 {
        return Err(format!("You do not have the {}.", items::name(part).to_lowercase()));
    }
    if part != "hull_patch" && !s.ferry.contains("hull_patch") {
        return Err("Patch the hull first, or it will sink at the slip.".into());
    }
    if let Some(t) = part_tool(part)
        && !has_tool(s, t)
    {
        return Err(format!("Needs {}.", tool_name(t)));
    }
    let m = missing(s, part_cost(part));
    if !m.is_empty() {
        return Err(format!("Needs {}.", journal::list(&m)));
    }
    if s.is_dark() {
        return Err("Too dark to work on the slip.".into());
    }
    Ok(())
}

/// What fitting a part uses up besides the part.
pub fn part_cost(part: &str) -> &'static [(&'static str, u32)] {
    match part {
        "hull_patch" => &[("nails", 4), ("pitch", 2), ("plank", 2)],
        _ => &[],
    }
}

pub fn install_minutes(s: &GameState) -> u32 {
    work_minutes(s, 180)
}

pub fn install(s: &mut GameState, j: &mut Journal, part: &str) {
    inv_take(&mut s.store, part, 1);
    for (id, n) in part_cost(part) {
        inv_take(&mut s.store, id, *n);
    }
    clock::pass(s, j, install_minutes(s), Activity::Exertion);
    if s.ended() {
        return;
    }
    noise::make(s, j, if part == "hull_patch" { 10 } else { 3 }, noise::Kind::Other);
    s.ferry.insert(part.to_string());
    skills::practise(s, j, Skill::Mending, 2);
    j.events.push(GameEvent::PartInstalled { part: part.to_string() });
    let line = match part {
        "hull_patch" => {
            "You cross the lock gates to the slip and rivet the patch over the split plank. It is not pretty. It will hold."
        }
        "fuel_drum" => "You fill the Kittiwake's tank from the drum. The smell of paraffin over the river.",
        "spark_coil" => {
            "You bolt the coil into its bracket and run the leads. The engine has a voice again, somewhere in it."
        }
        _ => "You fit the belt over the pulleys and take up the slack. It turns true.",
    };
    journal::find(s, j, line);
    if s.ferry.len() == 4 {
        journal::write(
            s,
            j,
            Tone::Quiet,
            "The Kittiwake is whole. She sits at the slip and waits for you.",
        );
    }
}

/// Food and water for two days on the river, taken aboard when you cast off.
pub const VOYAGE_MEALS: u32 = 8;
pub const VOYAGE_WATER: u32 = 5;

pub fn can_cast_off(s: &GameState) -> Result<(), String> {
    if s.frozen {
        return Err("The river is frozen.".into());
    }
    if s.ferry.len() < 4 {
        return Err(format!(
            "The ferry needs {} more.",
            journal::count(4 - s.ferry.len() as u32, "part", "parts")
        ));
    }
    let meals = inventory::meal_count(&s.store);
    let water = inv_count(&s.store, "water_clean");
    if meals < VOYAGE_MEALS || water < VOYAGE_WATER {
        return Err(format!(
            "Two days to the estuary. Needs {VOYAGE_MEALS} meals and {VOYAGE_WATER} clean water put by (you have {meals} and {water})."
        ));
    }
    if s.is_dark() {
        return Err("Not in the dark. Wait for light.".into());
    }
    Ok(())
}

pub fn cast_off(s: &mut GameState, j: &mut Journal) {
    inventory::take_meals(s, j, VOYAGE_MEALS);
    inv_take(&mut s.store, "water_clean", VOYAGE_WATER);
    let dog = s.dog.state == DogState::Companion;
    let text = if dog {
        "You cast off from the slip. The dog stands in the bow with its nose to the wind. Behind you, Calder's Reach does not move. Ahead, the river goes on toward the sea."
    } else {
        "You cast off from the slip, alone. Behind you, Calder's Reach does not move. Ahead, the river goes on toward the sea."
    };
    end_run(s, j, EndKind::Departure, "Left on the Kittiwake", text);
}

pub fn walk_onto_ice(s: &mut GameState, j: &mut Journal) {
    s.stats.walked_ice = true;
    end_run(
        s,
        j,
        EndKind::FreezeOut,
        "Walked out onto the ice",
        "You walk out onto the frozen river and keep walking, downstream, toward wherever it goes. Nobody sees which way you went.",
    );
}
