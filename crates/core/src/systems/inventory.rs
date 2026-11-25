//! Carrying, wearing, water storage, wear and tear.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::backgrounds;
use crate::content::items::{self, ItemKind, WeaponDef};
use crate::content::{Slot, Station, stations};
use crate::state::{GameState, Inv, Tone, inv_add, inv_count, inv_take};
use crate::systems::journal;

/// Base carry capacity, tenths of a kilogram.
pub const BASE_CARRY: u32 = 120;
/// Water the house holds without any containers (jug, pans, kettle).
pub const BASE_WATER: u32 = 3;

pub fn weight_of(inv: &Inv) -> u32 {
    inv.iter()
        .map(|(id, n)| items::get(id).map_or(0, |d| d.weight).saturating_mul(*n))
        .fold(0u32, u32::saturating_add)
}

fn best_bag(inv: &Inv) -> u32 {
    inv.keys()
        .filter_map(|id| match items::get(id)?.kind {
            ItemKind::Bag { capacity } => Some(capacity),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// Carry capacity: base, the best bag you own, and your background.
pub fn capacity(s: &GameState) -> u32 {
    let bag = best_bag(&s.store).max(best_bag(&s.pack));
    let bg = backgrounds::get(&s.background).map_or(0, |b| b.mods.carry);
    (BASE_CARRY as i32 + bag as i32 + bg).max(40) as u32
}

/// Capacity if one unit of `item` left the pack.
pub fn capacity_without(s: &GameState, item: &str) -> u32 {
    let mut pack = s.pack.clone();
    crate::state::inv_take(&mut pack, item, 1);
    let bag = best_bag(&s.store).max(best_bag(&pack));
    let bg = backgrounds::get(&s.background).map_or(0, |b| b.mods.carry);
    (BASE_CARRY as i32 + bag as i32 + bg).max(40) as u32
}

pub fn carried(s: &GameState) -> u32 {
    weight_of(&s.pack)
}

pub fn room_for(s: &GameState, item: &str) -> u32 {
    let w = items::get(item).map_or(0, |d| d.weight);
    if w == 0 {
        return 9_999;
    }
    capacity(s).saturating_sub(carried(s)) / w
}

pub fn water_capacity(s: &GameState) -> u32 {
    let containers: u32 = s
        .store
        .iter()
        .map(|(id, n)| items::get(id).map_or(0, |d| d.holds_water).saturating_mul(*n))
        .sum();
    BASE_WATER + containers + stations::barrel_capacity(s.level(Station::Barrel))
}

pub fn water_stored(s: &GameState) -> u32 {
    inv_count(&s.store, "water_dirty") + inv_count(&s.store, "water_clean")
}

/// Insulation from what you wear.
pub fn insulation(s: &GameState) -> i32 {
    s.worn
        .values()
        .filter_map(|id| match items::get(id)?.kind {
            ItemKind::Clothing { insulation, .. } => Some(insulation),
            _ => None,
        })
        .sum()
}

pub fn wearing(s: &GameState, pred: fn(bool, bool) -> bool) -> bool {
    s.worn.values().any(|id| match items::get(id).map(|d| d.kind) {
        Some(ItemKind::Clothing { rainproof, wading, .. }) => pred(rainproof, wading),
        _ => false,
    })
}

pub fn rainproof(s: &GameState) -> bool {
    wearing(s, |r, _| r)
}

pub fn waders(s: &GameState) -> bool {
    wearing(s, |_, w| w)
}

/// Put on a piece of clothing from the current inventory; the old one goes back.
pub fn wear(s: &mut GameState, item: &str, j: &mut Journal) -> bool {
    let Some(ItemKind::Clothing { slot, .. }) = items::get(item).map(|d| d.kind) else {
        return false;
    };
    if inv_take(s.inv_mut(), item, 1) == 0 {
        return false;
    }
    if let Some(old) = s.worn.insert(slot, item.to_string()) {
        inv_add(s.inv_mut(), &old, 1);
    }
    journal::plain(s, j, &format!("You put on the {}.", items::name(item).to_lowercase()));
    true
}

pub fn worn_in(s: &GameState, slot: Slot) -> Option<&str> {
    s.worn.get(&slot).map(String::as_str)
}

/// Spend one use of a wearing item; it breaks when worn out. Returns true if it broke.
pub fn wear_out(s: &mut GameState, j: &mut Journal, item: &str, durability: u32) -> bool {
    let used = s.wear.entry(item.to_string()).or_insert(0);
    *used += 1;
    if *used >= durability.max(1) {
        s.wear.remove(item);
        let gone = inv_take(s.inv_mut(), item, 1);
        if gone > 0 {
            j.events.push(GameEvent::Lost {
                item: item.to_string(),
                qty: 1,
            });
            journal::write(
                s,
                j,
                Tone::Danger,
                &format!("The {} breaks.", items::name(item).to_lowercase()),
            );
            return true;
        }
    }
    false
}

/// Remaining uses on the current unit, for display.
pub fn uses_left(s: &GameState, item: &str, durability: u32) -> u32 {
    durability.saturating_sub(s.wear.get(item).copied().unwrap_or(0))
}

/// Best melee weapon in the current inventory (by average damage), or fists.
pub fn melee(s: &GameState) -> Option<(&'static str, WeaponDef)> {
    s.inv()
        .keys()
        .filter_map(|id| {
            let d = items::get(id)?;
            match d.kind {
                ItemKind::Weapon(w) if w.ammo.is_none() => Some((d.id, w)),
                _ => None,
            }
        })
        .max_by_key(|(id, w)| (w.dmg_min + w.dmg_max, std::cmp::Reverse(*id)))
}

/// Fists, when there is nothing else.
pub const FISTS: WeaponDef = WeaponDef {
    dmg_min: 3,
    dmg_max: 6,
    accuracy: 5,
    reach: false,
    noise: 1,
    durability: u32::MAX,
    ammo: None,
};

/// Best firearm with ammunition in the current inventory.
pub fn gun(s: &GameState) -> Option<(&'static str, WeaponDef)> {
    s.inv()
        .keys()
        .filter_map(|id| {
            let d = items::get(id)?;
            match d.kind {
                ItemKind::Weapon(w) if w.ammo.is_some_and(|a| s.count(a) > 0) => Some((d.id, w)),
                _ => None,
            }
        })
        .max_by_key(|(id, w)| (w.dmg_min + w.dmg_max, std::cmp::Reverse(*id)))
}

pub fn has_weapon(s: &GameState) -> bool {
    melee(s).is_some() || gun(s).is_some()
}

/// A usable light: (light item, item that is consumed).
pub fn light(s: &GameState) -> Option<(&'static str, &'static str)> {
    let mut best: Option<(u8, &'static str, &'static str)> = None;
    for id in s.inv().keys() {
        let Some(d) = items::get(id) else { continue };
        let cand = match d.kind {
            ItemKind::Light { fuel: Some(f), .. } if s.count(f) > 0 => Some((0u8, d.id, f)),
            ItemKind::Light { fuel: None, .. } => Some((if d.id == "torch" { 1 } else { 2 }, d.id, d.id)),
            _ => None,
        };
        if let Some(c) = cand
            && best.is_none_or(|b| c.0 < b.0)
        {
            best = Some(c);
        }
    }
    best.map(|(_, l, f)| (l, f))
}

/// Burn one use of light.
pub fn burn_light(s: &mut GameState, j: &mut Journal) -> Option<&'static str> {
    let (lamp, consumed) = light(s)?;
    let uses = match items::get(consumed).map(|d| d.kind) {
        Some(ItemKind::Fuel { uses }) => uses,
        Some(ItemKind::Light { uses, .. }) => uses,
        _ => 1,
    };
    let used = s.wear.entry(consumed.to_string()).or_insert(0);
    *used += 1;
    if *used >= uses.max(1) {
        s.wear.remove(consumed);
        inv_take(s.inv_mut(), consumed, 1);
        if consumed != lamp {
            journal::plain(
                s,
                j,
                &format!("That is the last of the {}.", items::name(consumed).to_lowercase()),
            );
        }
    }
    Some(lamp)
}

/// Meals in an inventory, cheapest first (what you give away first).
pub fn meals_by_value(inv: &Inv) -> Vec<String> {
    let mut v: Vec<(&String, u32)> = inv
        .keys()
        .filter(|id| items::is_meal(id))
        .map(|id| (id, items::get(id).map_or(0, |d| d.value)))
        .collect();
    v.sort_by_key(|(id, val)| (*val, (*id).clone()));
    v.into_iter().map(|(id, _)| id.clone()).collect()
}

pub fn meal_count(inv: &Inv) -> u32 {
    inv.iter().filter(|(id, _)| items::is_meal(id)).map(|(_, n)| *n).sum()
}

pub fn water_count(inv: &Inv) -> u32 {
    inv_count(inv, "water_clean") + inv_count(inv, "water_dirty")
}

/// Remove `n` meals from the current inventory, cheapest first.
pub fn take_meals(s: &mut GameState, j: &mut Journal, n: u32) -> u32 {
    let mut left = n;
    for id in meals_by_value(s.inv()) {
        if left == 0 {
            break;
        }
        let took = inv_take(s.inv_mut(), &id, left);
        if took > 0 {
            j.events.push(GameEvent::Lost {
                item: id.clone(),
                qty: took,
            });
        }
        left -= took;
    }
    n - left
}

pub fn take_water(s: &mut GameState, j: &mut Journal, n: u32) -> u32 {
    let mut left = n;
    for id in ["water_dirty", "water_clean"] {
        let took = inv_take(s.inv_mut(), id, left);
        if took > 0 {
            j.events.push(GameEvent::Lost {
                item: id.to_string(),
                qty: took,
            });
        }
        left -= took;
    }
    n - left
}

/// Lose up to `n` units of the most valuable things in the store (a robbery).
pub fn rob(s: &mut GameState, j: &mut Journal, n: u32) -> Vec<String> {
    let mut v: Vec<(String, u32)> = s
        .store
        .keys()
        .filter(|id| {
            !matches!(
                items::get(id).map(|d| d.kind),
                Some(ItemKind::Part | ItemKind::Key | ItemKind::Tool | ItemKind::Bag { .. })
            ) && !matches!(id.as_str(), "coil_cracked" | "belting")
        })
        .map(|id| (id.clone(), items::get(id).map_or(0, |d| d.value)))
        .collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut left = n;
    let mut lost = Vec::new();
    for (id, _) in v {
        if left == 0 {
            break;
        }
        let took = inv_take(&mut s.store, &id, left.min(2));
        if took > 0 {
            left -= took;
            j.events.push(GameEvent::Lost {
                item: id.clone(),
                qty: took,
            });
            lost.push(journal::qty(items::name(&id), took).to_lowercase());
        }
    }
    lost
}

/// Add items to the current inventory, with an event per item.
pub fn gain(s: &mut GameState, j: &mut Journal, item: &str, n: u32) {
    if n == 0 || items::get(item).is_none() {
        return;
    }
    inv_add(s.inv_mut(), item, n);
    j.events.push(GameEvent::Found {
        item: item.to_string(),
        qty: n,
    });
}

/// Move everything in the pack into the store.
pub fn unpack_all(s: &mut GameState) {
    let pack = std::mem::take(&mut s.pack);
    for (id, n) in pack {
        inv_add(&mut s.store, &id, n);
    }
}
