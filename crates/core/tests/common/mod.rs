//! Shared helpers for integration tests: configs, a heuristic bot that tries
//! to win, and a careless bot that does whatever is loudest and nearest.

#![allow(dead_code)]

use all_alone_core::action::{self, Action, check};
use all_alone_core::content::items::{self, ItemKind};
use all_alone_core::content::{Band, Difficulty, Station, VitalKind, places};
use all_alone_core::rng::Rng;
use all_alone_core::state::{DogState, GameConfig, GameState, Phase, SearchState, inv_count};
use all_alone_core::systems::{encounter, expedition, inventory};

pub fn config(seed: u32, difficulty: Difficulty) -> GameConfig {
    GameConfig {
        seed,
        difficulty,
        background: "clerk".into(),
        keepsake: None,
        daily: None,
        held_marks: vec![],
    }
}

pub fn enabled(s: &GameState) -> Vec<Action> {
    action::candidates(s)
        .into_iter()
        .filter(|a| check(s, a).is_ok())
        .collect()
}

fn ok(s: &GameState, a: &Action) -> bool {
    check(s, a).is_ok()
}

fn first(s: &GameState, list: impl IntoIterator<Item = Action>) -> Option<Action> {
    list.into_iter().find(|a| ok(s, a))
}

fn pts(s: &GameState, k: VitalKind) -> i32 {
    s.vitals.pts(k)
}

// ---------------------------------------------------------------- Careless bot

/// Picks a random allowed action, preferring loud and fast ones, never minding its needs.
pub fn careless(s: &GameState, rng: &mut Rng) -> Action {
    let mut v = enabled(s);
    v.retain(|a| {
        !matches!(
            a,
            Action::Use { .. }
                | Action::CastOff
                | Action::Sleep { watch: true }
                | Action::HoldStill
                | Action::Flee
                | Action::BackAway
        )
    });
    let loud: Vec<Action> = v
        .iter()
        .filter(|a| {
            matches!(
                a,
                Action::Enter { quiet: false }
                    | Action::Search { careful: false }
                    | Action::ForceLock
                    | Action::Walk { hurry: true }
                    | Action::Strike
                    | Action::Depart
                    | Action::PlanTrip { .. }
            )
        })
        .cloned()
        .collect();
    if !loud.is_empty() && rng.chance(80) {
        return loud[rng.below(loud.len() as u32) as usize].clone();
    }
    if v.is_empty() {
        return enabled(s).into_iter().next().unwrap_or(Action::Rise);
    }
    v[rng.below(v.len() as u32) as usize].clone()
}

// ---------------------------------------------------------------- Heuristic bot

fn food_score(id: &str, s: &GameState) -> Option<i32> {
    match items::get(id)?.kind {
        ItemKind::Food { sat, sick, .. } if sat >= 6 => {
            if sick > 0 && pts(s, VitalKind::Satiety) > 15 {
                return None;
            }
            let perish = items::get(id).map_or(0, |d| d.perish) as i32;
            Some(sat + perish / 2 - sick)
        }
        _ => None,
    }
}

/// Eat, drink, treat wounds: whatever keeps you alive, from what is at hand.
fn upkeep(s: &GameState) -> Option<Action> {
    let inv = s.inv();
    let use_ = |id: &str| Action::Use { item: id.to_string() };
    if s.bleeding > 0 {
        for id in ["bandage", "rag"] {
            if ok(s, &use_(id)) {
                return Some(use_(id));
            }
        }
    }
    if s.infection > 0 {
        for id in ["antibiotics", "antiseptic", "poultice"] {
            if ok(s, &use_(id)) && (id != "antibiotics" || s.infection > 25_000) {
                return Some(use_(id));
            }
        }
    }
    if pts(s, VitalKind::Hydration) < 60 {
        if ok(s, &use_("water_clean")) {
            return Some(use_("water_clean"));
        }
        if ok(s, &use_("hot_tea")) {
            return Some(use_("hot_tea"));
        }
        if pts(s, VitalKind::Hydration) < 22 && ok(s, &use_("water_dirty")) {
            return Some(use_("water_dirty"));
        }
    }
    let hunger_line = if s.ferry.len() == 4 { 35 } else { 55 };
    if pts(s, VitalKind::Satiety) < hunger_line {
        let best = inv
            .keys()
            .filter_map(|id| food_score(id, s).map(|sc| (sc, id.clone())))
            .max();
        if let Some((_, id)) = best
            && ok(s, &use_(&id))
        {
            return Some(use_(&id));
        }
    }
    if pts(s, VitalKind::Nerve) < 25 {
        for id in ["tobacco", "spirits", "painkillers", "chocolate"] {
            if ok(s, &use_(id)) {
                return Some(use_(id));
            }
        }
    }
    if pts(s, VitalKind::Health) < 45 && ok(s, &use_("painkillers")) {
        return Some(use_("painkillers"));
    }
    // Put on clothing that is better than what is worn.
    for id in inv.keys() {
        if let Some(ItemKind::Clothing { slot, insulation, .. }) = items::get(id).map(|d| d.kind) {
            let cur = s
                .worn
                .get(&slot)
                .and_then(|w| items::get(w))
                .map_or(-1, |d| match d.kind {
                    ItemKind::Clothing { insulation, .. } => insulation,
                    _ => 0,
                });
            if insulation > cur && ok(s, &use_(id)) {
                return Some(use_(id));
            }
        }
    }
    None
}

const BUILD_ORDER: &[(Station, u8)] = &[
    (Station::Workbench, 1),
    (Station::Stove, 1),
    (Station::Barricade, 1),
    (Station::Workbench, 2),
    (Station::Workbench, 3),
    (Station::Cot, 1),
    (Station::Medicine, 1),
    (Station::Barrel, 1),
    (Station::Radio, 1),
    (Station::Barricade, 2),
    (Station::Store, 1),
    (Station::Barrel, 2),
    (Station::Cot, 2),
    (Station::Stove, 2),
    (Station::Garden, 1),
    (Station::Workbench, 3),
    (Station::Barricade, 3),
];

const CRAFT_ORDER: &[&str] = &[
    "boil_water",
    "torch",
    "nail_club",
    "oil_lamp",
    "cook_fish",
    "roast_rabbit",
    "rabbit_stew",
    "root_stew",
    "bake_potatoes",
    "porridge",
    "satchel",
    "rucksack",
    "frame_pack",
    "hull_patch",
    "drive_belt",
    "cut_belt",
    "rewind_coil",
    "lockpicks",
    "salvage_nails",
    "snare",
    "night_line",
    "bandages",
    "poultice",
];

fn part_place(part: &str) -> Option<&'static str> {
    places::PLACES
        .iter()
        .find(|p| p.rooms.iter().any(|r| r.fixed.iter().any(|(id, _)| *id == part)))
        .map(|p| p.id)
}

fn have_part(s: &GameState, part: &str) -> bool {
    s.ferry.contains(part) || inv_count(&s.store, part) > 0 || inv_count(&s.pack, part) > 0
}

/// How well armed the bot is: rooms above this danger are left alone.
pub fn max_danger(s: &GameState) -> u8 {
    let best = s
        .store
        .keys()
        .chain(s.pack.keys())
        .filter_map(|id| items::weapon(id))
        .map(|w| if w.ammo.is_some() { 40 } else { w.dmg_min + w.dmg_max })
        .max()
        .unwrap_or(0);
    let hp = pts(s, VitalKind::Health);
    if hp < 55 {
        1
    } else if best >= 20 && hp >= 70 {
        3
    } else {
        2
    }
}

pub fn best_melee(s: &GameState) -> i32 {
    s.store
        .keys()
        .chain(s.pack.keys())
        .filter_map(|id| items::weapon(id))
        .filter(|w| w.ammo.is_none())
        .map(|w| w.dmg_min + w.dmg_max)
        .max()
        .unwrap_or(0)
}

pub fn light_uses(s: &GameState) -> u32 {
    let c = |id: &str| inv_count(&s.store, id) + inv_count(&s.pack, id);
    let lamp = if c("oil_lamp") > 0 { c("lamp_oil") * 3 } else { 0 };
    let torch = if c("flashlight") > 0 { c("battery") * 4 } else { 0 };
    c("candle") + c("torch") * 2 + lamp + torch
}

fn has_light(s: &GameState) -> bool {
    s.store
        .keys()
        .chain(s.pack.keys())
        .any(|id| matches!(items::get(id).map(|d| d.kind), Some(ItemKind::Light { .. })))
}

pub fn place_value(s: &GameState, id: &str) -> i32 {
    let Some(p) = places::get(id) else { return -1 };
    let Some(ps) = s.places.get(id) else { return -1 };
    if !ps.known {
        return -1;
    }
    let cap = max_danger(s);
    let mut v = 0;
    for (i, r) in p.rooms.iter().enumerate() {
        let rs = &ps.rooms[i];
        let live = if rs.rolled { rs.foes.len() as u8 } else { r.danger };
        if live > cap || (r.dark && !has_light(s)) {
            break;
        }
        if let Some(l) = r.lock
            && !rs.unlocked
        {
            let owns = |id: &str| inv_count(&s.store, id) + inv_count(&s.pack, id) > 0;
            let key = l.key.is_some_and(&owns);
            let pick = l.pick.is_some_and(|d| {
                owns("lockpicks") && 60 + 10 * i32::from(s.skill(all_alone_core::content::Skill::Lockwork)) - d >= 30
            });
            let force = owns("crowbar") || owns("hatchet") || owns("hammer");
            if !(key || pick || force) {
                break;
            }
        }
        if rs.search != SearchState::Full {
            v += 10 + r.loot.len() as i32 * 2;
            v += r
                .fixed
                .iter()
                .filter(|(id, _)| partish(id) && !have_part(s, part_of(id)) && hinted(s, id))
                .count() as i32
                * 80;
            v += r.codex.len() as i32 * 3;
            v += r
                .loot
                .iter()
                .filter(|l| needed(s, l.item))
                .map(|l| l.pct / 4)
                .sum::<i32>();
        }
        v += rs
            .pile
            .iter()
            .filter(|(id, _)| worth_carrying(s, id))
            .map(|(_, n)| *n)
            .sum::<u32>() as i32
            * 2;
        v -= i32::from(r.danger) * 4;
    }
    v - p.minutes as i32 / 5
}

fn choose_place(s: &GameState) -> Option<String> {
    let t = s.time_of_day();
    places::PLACES
        .iter()
        .filter(|p| {
            // Back well before dusk: two walks and a few hours inside.
            t + 2 * p.minutes + 150 <= 18 * 60 + 30
        })
        .map(|p| (place_value(s, p.id), p.id))
        .filter(|(v, _)| *v > 5)
        .max()
        .map(|(_, id)| id.to_string())
}

fn home(s: &GameState) -> Action {
    use Action::*;
    if ok(s, &CastOff) {
        return CastOff;
    }
    if let Some(a) = upkeep(s) {
        return a;
    }
    let t = s.time_of_day();
    let late = !(6 * 60..19 * 60).contains(&t);
    let fire = s.fire_until > s.minute;
    // Keep warm and keep the fire for cooking and boiling.
    let need_fire = pts(s, VitalKind::Warmth) < 55 || inv_count(&s.store, "water_dirty") >= 3 || late;
    if !fire && need_fire && ok(s, &FeedStove) {
        return FeedStove;
    }
    if late {
        if s.fire_until < s.minute + 6 * 60 && pts(s, VitalKind::Warmth) < 80 && ok(s, &FeedStove) {
            return FeedStove;
        }
        if s.barricade < 30 && ok(s, &RepairBarricade) {
            return RepairBarricade;
        }
        if ok(s, &GoToBed) {
            return GoToBed;
        }
    }
    for part in items::FERRY_PARTS {
        let a = InstallPart { part: part.to_string() };
        if ok(s, &a) {
            return a;
        }
    }
    if s.ferry.len() == 4 {
        // Put by food and water for the voyage, then go.
        if inv_count(&s.store, "water_clean") < 6 {
            if let Some(a) = first(
                s,
                [
                    Craft {
                        recipe: "boil_water".into(),
                    },
                    FeedStove,
                    DrawWater,
                ],
            ) {
                return a;
            }
        }
        for r in ["cook_fish", "roast_rabbit", "smoke_fish", "bake_potatoes", "porridge"] {
            let a = Craft { recipe: r.into() };
            if ok(s, &a) {
                return a;
            }
        }
        if s.traps.hooked > 0 && ok(s, &CheckLine) {
            return CheckLine;
        }
        if inventory::meal_count(&s.store) < 8 && ok(s, &Forage) {
            return Forage;
        }
    }
    if s.traps.snared > 0 && ok(s, &CheckSnares) {
        return CheckSnares;
    }
    if s.traps.hooked > 0 && ok(s, &CheckLine) {
        return CheckLine;
    }
    if let Some(a) = first(s, [Harvest, SetSnares, SetLine]) {
        return a;
    }
    if ok(s, &FeedDog) && s.dog.fed < 40 {
        return FeedDog;
    }
    let max = all_alone_core::content::stations::barricade_max(s.level(Station::Barricade));
    if s.barricade * 100 < max * 60 && ok(s, &RepairBarricade) && !spends_reserve(s, &[("plank", 1), ("nails", 2)]) {
        return RepairBarricade;
    }
    for r in CRAFT_ORDER {
        let a = Craft { recipe: r.to_string() };
        let wanted = match *r {
            "hull_patch" => !have_part(s, "hull_patch"),
            "drive_belt" => !have_part(s, "drive_belt"),
            "satchel" => {
                inv_count(&s.store, "satchel") + inv_count(&s.store, "rucksack") + inv_count(&s.store, "frame_pack")
                    == 0
            }
            "rucksack" => inv_count(&s.store, "rucksack") + inv_count(&s.store, "frame_pack") == 0,
            "frame_pack" => inv_count(&s.store, "frame_pack") == 0,
            "lockpicks" => inv_count(&s.store, "lockpicks") == 0,
            "salvage_nails" => inv_count(&s.store, "nails") < 6 && inv_count(&s.store, "plank") > 1,
            "snare" => s.traps.snares + inv_count(&s.store, "snare") < 4,
            "night_line" => !s.traps.line && inv_count(&s.store, "fishing_line") == 0,
            "bandages" => inv_count(&s.store, "bandage") < 3,
            "torch" => light_uses(s) < 4,
            "poultice" => s.infection > 0 || inv_count(&s.store, "poultice") == 0,
            "oil_lamp" => inv_count(&s.store, "oil_lamp") == 0,
            "nail_club" => best_melee(s) < 25,
            "porridge" | "bake_potatoes" => pts(s, VitalKind::Satiety) < 70 || inventory::meal_count(&s.store) < 4,
            _ => true,
        };
        let critical = matches!(
            *r,
            "cut_belt" | "rewind_coil" | "boil_water" | "cook_fish" | "roast_rabbit"
        );
        let cost = all_alone_core::content::recipes::get(r).map_or(&[][..], |x| x.inputs);
        if wanted && ok(s, &a) && (critical || !spends_reserve(s, cost)) {
            return a;
        }
    }
    for (st, lvl) in BUILD_ORDER {
        if s.level(*st) < *lvl {
            let a = Build { station: *st };
            let cost = all_alone_core::content::stations::def(*st).levels[usize::from(s.level(*st)) + 1].cost;
            let critical = *st == Station::Workbench && (*lvl <= 2 || !have_part(s, "spark_coil"));
            if ok(s, &a) && (critical || !spends_reserve(s, cost)) {
                return a;
            }
        }
    }
    if inventory::water_count(&s.store) < 5 && ok(s, &DrawWater) {
        return DrawWater;
    }
    if ok(s, &ListenRadio) {
        return ListenRadio;
    }
    if pts(s, VitalKind::Health) > 45 && pts(s, VitalKind::Rest) > 35 && s.vitals.pts(VitalKind::Hydration) > 40 {
        if let Some(p) = choose_place(s) {
            return PlanTrip { place: p };
        }
    }
    if inv_count(&s.store, "firewood") < 8 && ok(s, &Forage) {
        return Forage;
    }
    if ok(s, &TendGarden) {
        return TendGarden;
    }
    if ok(s, &Rest) && pts(s, VitalKind::Rest) < 70 {
        return Rest;
    }
    if ok(s, &Forage) {
        return Forage;
    }
    if ok(s, &GoToBed) {
        return GoToBed;
    }
    if ok(s, &Rest) {
        return Rest;
    }
    if ok(s, &DrawWater) {
        return DrawWater;
    }
    enabled(s)
        .into_iter()
        .find(|a| !matches!(a, Action::PlanTrip { .. } | Action::WalkOntoIce))
        .unwrap_or(GoToBed)
}

fn packing(s: &GameState) -> Action {
    use Action::*;
    let pack = |id: &str, want: u32| -> Option<Action> {
        let have = inv_count(&s.pack, id);
        if have < want && inv_count(&s.store, id) > 0 {
            let a = Pack {
                item: id.to_string(),
                qty: 1,
            };
            if ok(s, &a) {
                return Some(a);
            }
        }
        None
    };
    // A weapon, light, tools, a little food and water, a bandage.
    if let Some((w, _)) = {
        let mut best: Option<(&str, i32)> = None;
        for id in s.store.keys() {
            if let Some(wd) = items::weapon(id)
                && wd.ammo.is_none()
            {
                let sc = wd.dmg_min + wd.dmg_max;
                if best.is_none_or(|b| sc > b.1) {
                    best = Some((items::get(id).map_or("", |d| d.id), sc));
                }
            }
        }
        best
    } && let Some(a) = pack(w, 1)
    {
        return a;
    }
    let plan: &[(&str, u32)] = &[
        ("crowbar", 1),
        ("hammer", 1),
        ("lockpicks", 1),
        ("oil_lamp", 1),
        ("lamp_oil", 1),
        ("flashlight", 1),
        ("battery", 1),
        ("candle", 2),
        ("torch", 1),
        ("bandage", 1),
        ("rag", 1),
        ("water_clean", 2),
        ("revolver", 1),
        ("rounds", 6),
        ("tin_rattle", 1),
    ];
    for (id, n) in plan {
        if let Some(a) = pack(id, *n) {
            return a;
        }
    }
    if inventory::meal_count(&s.pack) < 2 {
        let best = s
            .store
            .keys()
            .filter(|id| items::is_meal(id) && food_score(id, s).is_some())
            .max_by_key(|id| items::get(id).map_or(0, |d| d.perish));
        if let Some(id) = best
            && let Some(a) = pack(id, 99)
        {
            return a;
        }
    }
    if s.dog.state == DogState::Companion && ok(s, &TakeDog { take: true }) {
        return TakeDog { take: true };
    }
    Depart
}

/// The ferry part an item becomes.
fn part_of(id: &str) -> &str {
    match id {
        "coil_cracked" => "spark_coil",
        "belting" => "drive_belt",
        x => x,
    }
}

/// The bot only heads for a part once something it has read points there,
/// as a player would.
fn hinted(s: &GameState, id: &str) -> bool {
    let hints: &[&str] = match id {
        "hull_patch" => &["note_lowe", "note_sluice", "radio_boatyard"],
        "coil_cracked" => &["note_lowe", "note_pell"],
        "belting" => &["note_lowe", "note_engineer"],
        "fuel_drum" => &[
            "note_mill_ledger",
            "note_pell",
            "note_lowe",
            "note_wharfinger",
            "note_stranger",
            "radio_coal",
        ],
        _ => &[],
    };
    hints.iter().any(|h| s.codex.contains(*h))
}

fn partish(id: &str) -> bool {
    items::is_part(id) || id == "coil_cracked" || id == "belting"
}

/// Materials the ferry's critical path still needs (workbench 1-2, the coil, the hull).
fn reserved(s: &GameState, id: &str) -> u32 {
    let mut r = 0;
    let wb = s.level(Station::Workbench);
    if wb < 3 && !have_part(s, "spark_coil") {
        r += match id {
            "scrap" => 4,
            "parts" => 3,
            "nails" => 4,
            _ => 0,
        };
    }
    if wb < 1 {
        r += match id {
            "plank" => 2,
            "nails" => 4,
            _ => 0,
        };
    }
    if wb < 2 {
        r += match id {
            "plank" => 1,
            "scrap" => 3,
            "parts" => 1,
            "nails" => 4,
            _ => 0,
        };
    }
    if !have_part(s, "spark_coil") {
        r += match id {
            "wire" => 4,
            "battery" => 2,
            "parts" => 1,
            _ => 0,
        };
    }
    if !s.ferry.contains("hull_patch") && id == "nails" {
        r += 4;
    }
    r
}

/// Would spending this cost eat into what the ferry needs?
fn spends_reserve(s: &GameState, cost: &[(&str, u32)]) -> bool {
    cost.iter()
        .any(|(id, n)| inv_count(&s.store, id).saturating_sub(*n) < reserved(s, id))
}

/// Materials the next few builds and the ferry need, short of what is stored.
fn needed(s: &GameState, id: &str) -> bool {
    let have = inv_count(&s.store, id) + inv_count(&s.pack, id);
    let mut want = 0;
    let mut goals = 0;
    for (st, lvl) in BUILD_ORDER {
        if s.level(*st) < *lvl && goals < 3 {
            goals += 1;
            let l = &all_alone_core::content::stations::def(*st).levels[usize::from(s.level(*st)) + 1];
            want += l.cost.iter().filter(|(c, _)| *c == id).map(|(_, n)| *n).sum::<u32>();
        }
    }
    want += reserved(s, id);
    have < want
}

fn worth_carrying(s: &GameState, id: &str) -> bool {
    let Some(d) = items::get(id) else { return false };
    if partish(id) || needed(s, id) {
        return true;
    }
    match d.kind {
        ItemKind::Material => {
            let roomy = inventory::carried(s) + d.weight <= inventory::capacity(s) * 7 / 10;
            roomy && (d.weight <= 8 || (id == "plank" && inv_count(&s.store, "plank") < 10))
        }
        _ => true,
    }
}

fn site(s: &GameState) -> Action {
    use Action::*;
    if let Some(a) = upkeep(s) {
        return a;
    }
    let place = s.trip.as_ref().map(|t| t.place.clone()).unwrap_or_default();
    let p = places::get(&place);
    let t = s.time_of_day();
    let walk = p.map_or(60, |p| p.minutes);
    let must_go = t + walk + 20 >= 18 * 60 + 40 || t < 6 * 60;
    let weak = pts(s, VitalKind::Health) < 40
        || (s.bleeding > 0 && inv_count(&s.pack, "bandage") + inv_count(&s.pack, "rag") == 0);
    if must_go || weak {
        return Leave;
    }
    if let Some((i, r)) = expedition::current_room(s) {
        let rs = expedition::room_state(s, i);
        if r.dark && !expedition::is_lit(s) && ok(s, &LightUp) {
            return LightUp;
        }
        if rs.is_some_and(|rs| !rs.foes.is_empty()) && check(s, &Ambush).ok().and_then(|i| i.odds).unwrap_or(0) >= 60 {
            return Ambush;
        }
        if ok(s, &Search { careful: true }) {
            let m = check(s, &Search { careful: true }).map(|i| i.minutes).unwrap_or(0);
            if t + m + walk + 20 < 18 * 60 + 30 {
                return Search { careful: true };
            }
        }
        if ok(s, &Search { careful: false }) && rs.is_some_and(|rs| rs.search == SearchState::None) {
            return Search { careful: false };
        }
        // Make room for a ferry part.
        if let Some(rs) = rs {
            let part = rs.pile.keys().find(|id| partish(id)).cloned();
            if let Some(part) = part {
                let a = Take {
                    item: part.clone(),
                    qty: 1,
                };
                if ok(s, &a) {
                    return a;
                }
                let junk = s
                    .pack
                    .keys()
                    .filter(|id| {
                        !partish(id) && !matches!(items::get(id).map(|d| d.kind), Some(ItemKind::Tool | ItemKind::Key))
                    })
                    .max_by_key(|id| items::get(id).map_or(0, |d| d.weight))
                    .cloned();
                if let Some(j) = junk {
                    let n = inv_count(&s.pack, &j);
                    return Drop { item: j, qty: n };
                }
            }
            let mut pile: Vec<&String> = rs.pile.keys().filter(|id| worth_carrying(s, id)).collect();
            pile.sort_by_key(|id| std::cmp::Reverse(items::get(id).map_or(0, |d| d.value * 10 / d.weight.max(1))));
            for id in pile {
                let n = inv_count(&rs.pile, id).min(inventory::room_for(s, id));
                let a = Take {
                    item: id.clone(),
                    qty: n,
                };
                if n > 0 && ok(s, &a) {
                    return a;
                }
            }
        }
    }
    // Locks: key, pick, or force if what is beyond matters.
    if expedition::next_locked(s) {
        if ok(s, &UseKey) {
            return UseKey;
        }
        if check(s, &PickLock).ok().and_then(|i| i.odds).unwrap_or(0) >= 30 {
            return PickLock;
        }
        let worth = expedition::next_room(s).is_some_and(|(_, r)| {
            r.fixed.iter().any(|(id, _)| items::is_part(id) && !have_part(s, id)) || r.loot.len() >= 4
        });
        if worth && ok(s, &ForceLock) {
            return ForceLock;
        }
        return Leave;
    }
    if let Some((i, r)) = expedition::next_room(s) {
        let rs = expedition::room_state(s, i);
        let live = if rs.is_some_and(|x| x.rolled) {
            rs.map_or(0, |x| x.foes.len() as u8)
        } else {
            r.danger
        };
        if live > max_danger(s) {
            return Leave;
        }
        if r.dark && !rs.is_some_and(|x| x.lit) {
            if ok(s, &LightUp) {
                return LightUp;
            }
            return Leave;
        }
    }
    if ok(s, &Enter { quiet: true }) && inventory::carried(s) * 100 < inventory::capacity(s) * 90 {
        return Enter { quiet: true };
    }
    Leave
}

fn fight(s: &GameState) -> Action {
    use Action::*;
    let hp = pts(s, VitalKind::Health);
    if s.bleeding >= 2 && ok(s, &Use { item: "bandage".into() }) {
        return Use { item: "bandage".into() };
    }
    let foes = s.encounter.as_ref().map(|e| e.foes.clone()).unwrap_or_default();
    let n = foes.len();
    let heavy = foes.iter().any(|f| f.kind == all_alone_core::content::Creature::Hollow);
    let nearest = encounter::nearest(s);
    if hp < 35 && ok(s, &Flee) {
        return Flee;
    }
    if (n >= 2 || heavy) && ok(s, &Shoot) {
        return Shoot;
    }
    let strong = best_melee(s) >= 25 || inventory::gun(s).is_some();
    if (n >= 3 || heavy || (n >= 2 && !strong)) && nearest != Some(Band::Close) && ok(s, &Flee) {
        return Flee;
    }
    if nearest == Some(Band::Near) && n >= 2 {
        if let Some(a) = s
            .inv()
            .keys()
            .find(|id| items::get(id).is_some_and(|d| d.throw > 0))
            .map(|id| Throw { item: id.clone() })
        {
            if ok(s, &a) {
                return a;
            }
        }
    }
    if let Ok(i) = check(s, &Strike)
        && i.odds.unwrap_or(0) >= 35
    {
        return Strike;
    }
    if nearest == Some(Band::Close) && ok(s, &Shove) {
        return Shove;
    }
    if nearest != Some(Band::Close) {
        return HoldStill;
    }
    if ok(s, &Strike) {
        return Strike;
    }
    HoldStill
}

fn event(s: &GameState) -> Action {
    use all_alone_core::content::events::Fx;
    let def = all_alone_core::systems::events::current(s);
    let mut best: Option<(i32, u32)> = None;
    if let Some(def) = def {
        for (i, o) in def.opts.iter().enumerate() {
            let a = Action::Choose { option: i as u32 };
            if !ok(s, &a) {
                continue;
            }
            let risky =
                o.ok.fx
                    .iter()
                    .chain(o.fail.fx)
                    .any(|f| matches!(f, Fx::Fight(..) | Fx::Rob(_)));
            let costly = o.ok.fx.iter().any(|f| matches!(f, Fx::Meals(_) | Fx::Water(_)));
            let dog = o.ok.fx.iter().any(|f| {
                matches!(
                    f,
                    Fx::Dog(
                        all_alone_core::content::events::DogFx::Join | all_alone_core::content::events::DogFx::Follow
                    )
                )
            });
            let mut score = 0;
            if risky {
                score -= 20;
            }
            if costly && inventory::meal_count(s.inv()) < 6 {
                score -= 5;
            }
            if dog {
                score += 30;
            }
            score -= o.minutes as i32 / 30;
            if best.is_none_or(|b| score > b.0) {
                best = Some((score, i as u32));
            }
        }
    }
    Action::Choose {
        option: best.map_or(0, |b| b.1),
    }
}

/// The heuristic bot: tries to fit the four parts and leave before the freeze.
pub fn heuristic(s: &GameState) -> Action {
    use Action::*;
    let a = match s.phase {
        Phase::Home => home(s),
        Phase::Packing => packing(s),
        Phase::Travelling => {
            if let Some(a) = upkeep(s) {
                a
            } else {
                let hurry = s.homeward && s.time_of_day() + expedition::walk_minutes(s, false) > 18 * 60 + 50;
                Walk { hurry }
            }
        }
        Phase::Site => site(s),
        Phase::Encounter => fight(s),
        Phase::Event => event(s),
        Phase::Barter => EndTrade,
        Phase::Night => {
            if let Some(a) = upkeep(s) {
                a
            } else {
                Sleep { watch: false }
            }
        }
        Phase::Dawn => Rise,
        Phase::Ended => Rise,
    };
    if ok(s, &a) {
        return a;
    }
    // Never stall: fall back to anything allowed.
    let v = enabled(s);
    v.iter()
        .find(|x| {
            matches!(
                x,
                Leave
                    | Walk { .. }
                    | Rise
                    | Sleep { .. }
                    | GoToBed
                    | HoldStill
                    | Choose { .. }
                    | Depart
                    | EndTrade
                    | Strike
            )
        })
        .or_else(|| v.first())
        .cloned()
        .unwrap_or(Rise)
}

pub struct RunResult {
    pub won: bool,
    pub died: bool,
    pub days: u32,
    pub steps: u32,
    pub cause: String,
    pub parts: usize,
    pub state: GameState,
}

pub fn play(cfg: &GameConfig, mut pick: impl FnMut(&GameState) -> Action, max_steps: u32) -> RunResult {
    let mut s = all_alone_core::new_game(cfg).expect("config");
    let mut steps = 0;
    while !s.ended() && steps < max_steps {
        let a = pick(&s);
        if let Err(e) = all_alone_core::dispatch(&mut s, &a) {
            panic!("bot chose an invalid action {a:?} in {:?}: {e}", s.phase);
        }
        steps += 1;
    }
    let ending = s.ending.as_ref();
    RunResult {
        won: ending.is_some_and(|e| e.kind == all_alone_core::state::EndKind::Departure),
        died: ending.is_some_and(|e| e.kind == all_alone_core::state::EndKind::Death),
        days: s.day(),
        steps,
        cause: ending.map_or("unfinished".into(), |e| e.cause.clone()),
        parts: s.ferry.len(),
        state: s,
    }
}

pub fn median(mut v: Vec<u32>) -> u32 {
    v.sort_unstable();
    if v.is_empty() { 0 } else { v[v.len() / 2] }
}
