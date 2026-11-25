//! Barter with the ferryman's widow at the weir. She comes every third day,
//! writes on a slate, gives no change, and remembers what you sold her.

use crate::Journal;
use crate::content::{backgrounds, items};
use crate::state::{GameState, Offer, inv_take};
use crate::systems::{inventory, journal, pct_u};

/// What she always might offer.
const POOL: &[&str] = &[
    "tinned_meat",
    "smoked_fish",
    "bandage",
    "candle",
    "lamp_oil",
    "rounds",
    "wool_hat",
    "seed_kale",
    "tea",
    "rope",
    "nails",
    "antiseptic",
    "battery",
    "glass_bottle",
    "oats",
];

/// What selling her enough of a category (3 units) adds to her offers.
const LEARNED: &[(&str, &[&str])] = &[
    ("food", &["seed_potato", "jam", "seed_turnip"]),
    ("medicine", &["antibiotics", "painkillers"]),
    ("arms", &["shells", "rounds"]),
    ("cloth", &["wool_coat", "gloves", "oilskin"]),
    ("light", &["flashlight", "lamp_oil"]),
    ("tools", &["snare", "fishing_line", "lockpicks"]),
    ("goods", &["pitch", "leather", "parts"]),
    ("drink", &["spirits", "water_clean"]),
];

pub fn present_on(s: &GameState, day: u32) -> bool {
    day >= 2 && (day + s.widow.offset) % 3 == 0 && !s.frozen
}

pub fn present(s: &GameState) -> bool {
    present_on(s, s.day())
}

/// Days until she is next at the weir (0 = today).
pub fn next_visit(s: &GameState) -> Option<u32> {
    (0..4).find(|d| present_on(s, s.day() + d))
}

/// Her price for one unit, which rises with the season.
pub fn price(s: &GameState, item: &str) -> u32 {
    let base = items::get(item).map_or(1, |d| d.value.max(1));
    let mut p = 100 + s.day() * 2;
    if items::category(item) == "food" && s.day() > 25 {
        p += 30;
    }
    if items::is_part(item) {
        p = 100;
    }
    let bg = backgrounds::get(&s.background).map_or(0, |b| b.mods.barter);
    pct_u(base, (p as i32 + bg).max(50) as u32).max(1)
}

/// What she gives for one unit.
pub fn sell_price(item: &str) -> u32 {
    let v = items::get(item).map_or(0, |d| d.value);
    pct_u(v, 60)
}

pub fn dawn(s: &mut GameState, _j: &mut Journal) {
    s.flags.remove("widow_here");
    s.widow.credit = 0;
    s.widow.stock.clear();
    if !present(s) {
        return;
    }
    s.flags.insert("widow_here".into());
    let mut pool: Vec<&str> = POOL.to_vec();
    for (cat, extra) in LEARNED {
        if s.widow.sold.get(*cat).copied().unwrap_or(0) >= 3 {
            pool.extend_from_slice(extra);
        }
    }
    let mut stock: Vec<Offer> = Vec::new();
    for _ in 0..5 {
        if pool.is_empty() {
            break;
        }
        let i = s.rng.below(pool.len() as u32) as usize;
        let item = pool.swap_remove(i);
        if stock.iter().any(|o| o.item == item) {
            continue;
        }
        let qty = s.rng.range(1, 3) as u32;
        stock.push(Offer {
            item: item.to_string(),
            qty,
            price: price(s, item),
        });
    }
    for item in s.widow.extra.clone() {
        if !stock.iter().any(|o| o.item == item) {
            stock.push(Offer {
                price: price(s, &item),
                item,
                qty: 1,
            });
        }
    }
    s.widow.stock = stock;
}

pub fn open(s: &mut GameState, j: &mut Journal) {
    s.widow.credit = 0;
    if !s.widow.met {
        s.widow.met = true;
        journal::plain(
            s,
            j,
            "A woman in black on the weir steps with a handcart and a child's slate. She does not speak. She writes: TRADE. NO CHANGE. Every third day.",
        );
    } else {
        journal::plain(s, j, "The widow wipes her slate and waits.");
    }
}

pub fn sell(s: &mut GameState, j: &mut Journal, item: &str, qty: u32) {
    let n = inv_take(&mut s.pack, item, qty);
    if n == 0 {
        return;
    }
    let value = sell_price(item) * n;
    s.widow.credit += value;
    *s.widow.sold.entry(items::category(item).to_string()).or_insert(0) += n;
    j.events.push(crate::action::GameEvent::Lost {
        item: item.to_string(),
        qty: n,
    });
    journal::plain(
        s,
        j,
        &format!(
            "She takes {} and chalks {value} on the slate.",
            journal::qty(items::name(item), n).to_lowercase()
        ),
    );
}

pub fn buy(s: &mut GameState, j: &mut Journal, item: &str) {
    let Some(pos) = s.widow.stock.iter().position(|o| o.item == item && o.qty > 0) else {
        return;
    };
    let price = s.widow.stock[pos].price;
    s.widow.credit -= price;
    s.widow.stock[pos].qty -= 1;
    if s.widow.stock[pos].qty == 0 {
        s.widow.stock.remove(pos);
    }
    s.widow.extra.retain(|x| x != item);
    inventory::gain(s, j, item, 1);
    s.widow.trades += 1;
    s.stats.trades += 1;
    journal::find(
        s,
        j,
        &format!(
            "She rubs out {price} and hands you {}.",
            items::name(item).to_lowercase()
        ),
    );
}

pub fn close(s: &mut GameState, j: &mut Journal) {
    if s.widow.credit > 0 {
        journal::plain(s, j, "She wipes the slate clean of what you did not spend. No change.");
    }
    s.widow.credit = 0;
}
