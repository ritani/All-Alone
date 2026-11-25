//! The view model: everything the UI draws, already resolved. The UI never
//! re-implements a rule; it reads this and dispatches the actions listed here.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::action::{self, Action};
use crate::content::codex::{self, CodexKind};
use crate::content::events as ev;
use crate::content::items::{self, ItemKind, MedKind};
use crate::content::{
    Band, Creature, Skill, Station, VitalKind, Weather, backgrounds, creatures, places, recipes, stations,
};
use crate::meta::Ledger;
use crate::state::{DUSK, DawnSummary, DogState, EndKind, Entry, GameState, Inv, Phase, SearchState};
use crate::systems::objectives::{self, Objective};
use crate::systems::{
    barter, clock, dog, encounter, expedition, home, inventory, journal, night, noise, skills, weather,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct ActionView {
    /// Send this back to `dispatch` unchanged (quantity actions may change `qty` up to `max_qty`).
    pub action: Action,
    pub label: String,
    /// "survive", "house", "station", "craft", "trip", "food", "ferry", "pack", "move", "room", "lock", "loot", "fight", "choice", "trade", "night".
    pub group: String,
    pub icon: String,
    pub minutes: u32,
    pub noise: u32,
    pub odds: Option<i32>,
    pub enabled: bool,
    pub reason: Option<String>,
    /// Risky or irreversible: draw in the danger accent.
    pub danger: bool,
    pub max_qty: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct VitalView {
    pub id: VitalKind,
    pub name: String,
    pub value: i32,
    pub icon: String,
    /// "ok", "low" (< 35) or "critical" (< 15).
    pub level: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct ConditionView {
    pub id: String,
    pub name: String,
    pub detail: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct WeatherInfo {
    pub id: Weather,
    pub name: String,
    pub icon: String,
    pub effect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct WeatherView {
    pub today: WeatherInfo,
    /// The radio's guess at tomorrow (may be wrong).
    pub forecast: Option<WeatherInfo>,
    pub forecast_accuracy: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct SeasonView {
    pub temperature: i32,
    pub frozen: bool,
    /// Days until the freeze, once the radio has said.
    pub freeze_in: Option<u32>,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct NoiseView {
    pub value: u32,
    pub label: String,
    /// Percent chance something comes to the house tonight, at today's noise.
    pub visit_chance: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct ItemView {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub qty: u32,
    /// Tenths of a kilogram, per unit.
    pub weight: u32,
    /// Total weight, e.g. "1.2 kg".
    pub weight_label: String,
    pub value: u32,
    /// "food", "medicine", "material", "weapon", "ammo", "tool", "light", "fuel", "bag", "clothing", "part", "key", "seed", "bait", "comfort", "trap".
    pub kind: String,
    /// One line of numbers: "Satiety +34", "Damage 13-19".
    pub detail: String,
    pub uses_left: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct InventoryView {
    pub store: Vec<ItemView>,
    pub pack: Vec<ItemView>,
    pub worn: Vec<ItemView>,
    pub carried: u32,
    pub capacity: u32,
    pub carry_label: String,
    pub water: u32,
    pub water_capacity: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct CostView {
    pub item: String,
    pub name: String,
    pub icon: String,
    pub need: u32,
    pub have: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct NextLevelView {
    pub level: u8,
    pub name: String,
    pub effect: String,
    pub cost: Vec<CostView>,
    pub workbench: u8,
    pub minutes: u32,
    pub noise: u32,
    pub can_build: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct StationView {
    pub id: Station,
    pub name: String,
    pub icon: String,
    pub level: u8,
    pub level_name: String,
    pub effect: String,
    /// Live state: "Burning until 21:30", "Holds 40 of 50".
    pub status: Option<String>,
    pub next: Option<NextLevelView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct RecipeView {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub inputs: Vec<CostView>,
    pub outputs: Vec<CostView>,
    pub station: Option<Station>,
    pub station_level: u8,
    pub fire: bool,
    pub minutes: u32,
    pub can_craft: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct MapNode {
    /// "home" or a place id.
    pub id: String,
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub x: u32,
    pub y: u32,
    pub minutes: u32,
    pub visited: bool,
    pub rooms: u32,
    pub rooms_searched: u32,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct MapView {
    pub nodes: Vec<MapNode>,
    pub paths: Vec<(String, String)>,
    /// Known places / all places.
    pub known: u32,
    pub total: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct CodexEntryView {
    pub id: String,
    pub kind: CodexKind,
    pub title: String,
    pub text: String,
    pub icon: String,
}

impl CodexEntryView {
    pub fn of(c: &codex::CodexDef) -> Self {
        CodexEntryView {
            id: c.id.to_string(),
            kind: c.kind,
            title: c.title.to_string(),
            text: c.text.to_string(),
            icon: c.kind.icon().to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct CodexSection {
    pub kind: CodexKind,
    pub name: String,
    pub icon: String,
    pub found: u32,
    pub total: u32,
    pub entries: Vec<CodexEntryView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct CodexView {
    pub sections: Vec<CodexSection>,
    pub found: u32,
    pub total: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct SkillView {
    pub id: Skill,
    pub name: String,
    pub icon: String,
    pub level: u8,
    pub xp: u32,
    /// Practice needed for the next level; null at level 5.
    pub next: Option<u32>,
    pub effect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct DogView {
    pub state: DogState,
    pub health: i32,
    pub fed: i32,
    pub with_you: bool,
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct FerryPartView {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub installed: bool,
    pub in_store: bool,
    /// What it still needs, in one line; null once fitted.
    pub need: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct FerryView {
    pub parts: Vec<FerryPartView>,
    pub installed: u32,
    pub ready: bool,
    pub frozen: bool,
    /// Provisions for the voyage, put by in the store, and what it takes.
    pub meals: u32,
    pub meals_need: u32,
    pub water: u32,
    pub water_need: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct PlotView {
    pub crop: String,
    pub name: String,
    pub growth: u32,
    pub need: u32,
    pub ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct GroundsView {
    pub snares_set: u32,
    pub snared: u32,
    pub line_set: bool,
    pub line_baited: bool,
    pub hooked: u32,
    pub plots: Vec<PlotView>,
    pub plots_total: u32,
    pub tended_today: bool,
    pub fire_until: Option<String>,
    pub barricade: u32,
    pub barricade_max: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct FoeView {
    pub kind: Creature,
    pub name: String,
    pub band: Band,
    /// Percent of its health left.
    pub health: i32,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct RoomSummary {
    pub index: u32,
    /// Room name once seen (the next door is always seen); "Further in" otherwise.
    pub name: String,
    pub entered: bool,
    pub here: bool,
    pub dark: bool,
    pub flooded: bool,
    pub locked: bool,
    pub search: SearchState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct RoomView {
    pub index: u32,
    pub name: String,
    pub desc: String,
    pub dark: bool,
    pub lit: bool,
    pub flooded: bool,
    pub search: SearchState,
    pub pile: Vec<ItemView>,
    /// Still ones standing here, not yet woken.
    pub dormant: Vec<FoeView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct SiteView {
    pub place: String,
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub rooms: Vec<RoomSummary>,
    pub room: Option<RoomView>,
    pub site_noise: u32,
    pub widow_here: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct EncounterView {
    pub foes: Vec<FoeView>,
    pub turn: u32,
    pub gripped: bool,
    pub at_home: bool,
    pub dark: bool,
    /// What happened in the last exchange.
    pub log: Vec<String>,
    /// What you would strike with ("bare hands" without a weapon).
    pub weapon: String,
    /// A gun you carry, loaded or not, and its rounds.
    pub gun: Option<String>,
    pub ammo: u32,
    pub ammo_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct OptionView {
    pub index: u32,
    pub label: String,
    pub enabled: bool,
    pub reason: Option<String>,
    pub odds: Option<i32>,
    pub minutes: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct EventView {
    pub id: String,
    pub title: String,
    pub text: String,
    pub options: Vec<OptionView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct TravelView {
    pub place: String,
    pub name: String,
    pub homeward: bool,
    pub minutes: u32,
    pub hurry_minutes: u32,
    /// You will arrive after dusk if you walk.
    pub arrive_dark: bool,
    /// Clock time on arrival, walking and hurrying ("HH:MM").
    pub arrive_at: String,
    pub hurry_arrive_at: String,
    pub hurry_arrive_dark: bool,
}

/// Something the destination is known to call for, and whether the pack has it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct NeedView {
    pub id: String,
    pub text: String,
    pub icon: String,
    pub met: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct PackingView {
    pub place: String,
    pub name: String,
    pub minutes: u32,
    pub take_dog: bool,
    pub has_light: bool,
    pub has_weapon: bool,
    pub meals: u32,
    pub water: u32,
    /// What you know the place calls for (only what has been seen there).
    pub needs: Vec<NeedView>,
    /// You have not been there before.
    pub unvisited: bool,
    /// Rooms seen / rooms in all, and rooms searched through.
    pub rooms: u32,
    pub rooms_searched: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct OfferView {
    pub item: String,
    pub name: String,
    pub icon: String,
    pub qty: u32,
    pub price: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct SellView {
    pub item: String,
    pub name: String,
    pub icon: String,
    pub qty: u32,
    pub price: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct BarterView {
    /// Her side of it: she does not speak, she writes on a slate.
    pub line: String,
    pub credit: u32,
    pub offers: Vec<OfferView>,
    pub sell: Vec<SellView>,
    pub trades: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct EndingView {
    pub kind: EndKind,
    pub cause: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct RunInfo {
    pub seed: u32,
    pub difficulty: crate::content::Difficulty,
    pub difficulty_name: String,
    pub background: String,
    pub background_name: String,
    pub keepsake: Option<String>,
    pub daily: Option<String>,
    pub nights: u32,
    pub kills: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct View {
    pub phase: Phase,
    pub day: u32,
    pub clock: String,
    pub minute_of_day: u32,
    pub dark: bool,
    /// Minutes of daylight left, while it is light.
    pub daylight_left: Option<u32>,
    pub season: SeasonView,
    pub weather: WeatherView,
    pub vitals: Vec<VitalView>,
    pub conditions: Vec<ConditionView>,
    pub noise: NoiseView,
    /// Where you are, in words.
    pub location: String,
    pub actions: Vec<ActionView>,
    pub objectives: Vec<Objective>,
    pub inventory: InventoryView,
    pub stations: Vec<StationView>,
    pub recipes: Vec<RecipeView>,
    pub grounds: GroundsView,
    pub map: MapView,
    pub codex: CodexView,
    pub skills: Vec<SkillView>,
    pub dog: Option<DogView>,
    pub ferry: FerryView,
    pub packing: Option<PackingView>,
    pub travel: Option<TravelView>,
    pub site: Option<SiteView>,
    pub encounter: Option<EncounterView>,
    pub event: Option<EventView>,
    pub barter: Option<BarterView>,
    pub dawn: Option<DawnSummary>,
    pub ending: Option<EndingView>,
    pub ledger: Option<Ledger>,
    /// Most recent last; at most `JOURNAL_IN_VIEW` entries.
    pub journal: Vec<Entry>,
    /// Marks earned this run, named (hidden ones are revealed once earned).
    pub marks: Vec<MarkEarned>,
    pub run: RunInfo,
}

/// A mark earned in this run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct MarkEarned {
    pub id: String,
    pub name: String,
    pub text: String,
    pub icon: String,
    pub tier: u8,
    /// The profile did not hold it before this run.
    pub new: bool,
}

pub const JOURNAL_IN_VIEW: usize = 80;

fn kg(tenths: u32) -> String {
    format!("{}.{} kg", tenths / 10, tenths % 10)
}

fn kind_name(k: &ItemKind) -> &'static str {
    match k {
        ItemKind::Food { sat, .. } if *sat <= 2 => "drink",
        ItemKind::Food { .. } => "food",
        ItemKind::Medicine(_) => "medicine",
        ItemKind::Material => "material",
        ItemKind::Weapon(_) => "weapon",
        ItemKind::Ammo => "ammo",
        ItemKind::Tool => "tool",
        ItemKind::Light { .. } => "light",
        ItemKind::Fuel { .. } => "fuel",
        ItemKind::Bag { .. } => "bag",
        ItemKind::Clothing { .. } => "clothing",
        ItemKind::Part => "part",
        ItemKind::Key => "key",
        ItemKind::Seed { .. } => "seed",
        ItemKind::Bait => "bait",
        ItemKind::Comfort { .. } => "comfort",
        ItemKind::Trap => "trap",
    }
}

fn signed(label: &str, v: i32) -> Option<String> {
    if v == 0 {
        None
    } else {
        Some(format!("{label} {}{v}", if v > 0 { "+" } else { "" }))
    }
}

fn detail(d: &items::ItemDef) -> String {
    let parts: Vec<String> = match d.kind {
        ItemKind::Food {
            sat,
            hyd,
            nerve,
            warm,
            sick,
        } => [
            signed("Satiety", sat),
            signed("Hydration", hyd),
            signed("Nerve", nerve),
            signed("Warmth", warm),
            if sick > 0 {
                Some(format!("Sickness risk {sick}%"))
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        .collect(),
        ItemKind::Medicine(MedKind::Bandage(p)) => vec![format!("Stops bleeding, about {p}%")],
        ItemKind::Medicine(MedKind::Antiseptic(n)) => vec![format!("Infection -{n}, slowed a day")],
        ItemKind::Medicine(MedKind::Antibiotic(n)) => vec![format!("Infection -{n}, cures a mild one")],
        ItemKind::Medicine(MedKind::Painkiller { health, nerve }) => vec![format!("Health +{health}, nerve +{nerve}")],
        ItemKind::Weapon(w) => {
            let mut v = vec![
                format!("Damage {}-{}", w.dmg_min, w.dmg_max),
                format!("noise {}", w.noise),
            ];
            if w.reach {
                v.push("reach".into());
            }
            if let Some(a) = w.ammo {
                v.push(format!("uses {}", items::name(a).to_lowercase()));
            }
            v
        }
        ItemKind::Light { uses, fuel } => match fuel {
            Some(f) => vec![format!("Burns {}", items::name(f).to_lowercase())],
            None => vec![format!("Lights {uses} dark room{}", if uses == 1 { "" } else { "s" })],
        },
        ItemKind::Fuel { uses } => vec![journal::count(uses, "use", "uses")],
        ItemKind::Bag { capacity } => vec![format!("Carry +{}", kg(capacity))],
        ItemKind::Clothing {
            insulation,
            rainproof,
            wading,
            ..
        } => {
            let mut v = vec![format!("Warmth +{insulation} outdoors")];
            if rainproof {
                v.push("rainproof".into());
            }
            if wading {
                v.push("for flooded rooms".into());
            }
            v
        }
        ItemKind::Seed { days, .. } => vec![format!("Ready in about {}", journal::count(days, "day", "days"))],
        ItemKind::Comfort { nerve } => vec![format!("Nerve +{nerve}")],
        _ => vec![],
    };
    let mut out = parts.join(", ");
    if d.holds_water > 0 {
        if !out.is_empty() {
            out.push_str(", ");
        }
        out.push_str(&format!("holds {} water at home", d.holds_water));
    }
    if d.throw > 0 {
        if !out.is_empty() {
            out.push_str(", ");
        }
        out.push_str("can be thrown to draw them");
    }
    out
}

fn item_view(s: &GameState, id: &str, qty: u32) -> ItemView {
    let Some(d) = items::get(id) else {
        return ItemView {
            id: id.to_string(),
            name: id.to_string(),
            desc: String::new(),
            icon: "item".into(),
            qty,
            weight: 0,
            weight_label: kg(0),
            value: 0,
            kind: "material".into(),
            detail: String::new(),
            uses_left: None,
        };
    };
    let uses_left = match d.kind {
        ItemKind::Weapon(w) if w.ammo.is_none() => Some(inventory::uses_left(s, id, w.durability)),
        ItemKind::Fuel { uses } | ItemKind::Light { uses, fuel: None } => Some(inventory::uses_left(s, id, uses)),
        _ => None,
    };
    ItemView {
        id: id.to_string(),
        name: d.name.to_string(),
        desc: d.desc.to_string(),
        icon: d.icon.to_string(),
        qty,
        weight: d.weight,
        weight_label: kg(d.weight * qty),
        value: d.value,
        kind: kind_name(&d.kind).to_string(),
        detail: detail(d),
        uses_left,
    }
}

fn items_of(s: &GameState, inv: &Inv) -> Vec<ItemView> {
    inv.iter().map(|(id, n)| item_view(s, id, *n)).collect()
}

fn cost_view(s: &GameState, list: &[(&str, u32)], have_from_store: bool) -> Vec<CostView> {
    list.iter()
        .map(|(id, n)| CostView {
            item: id.to_string(),
            name: items::name(id).to_string(),
            icon: items::get(id).map_or("item", |d| d.icon).to_string(),
            need: *n,
            have: if have_from_store {
                crate::state::inv_count(&s.store, id)
            } else {
                0
            },
        })
        .collect()
}

fn foe_view(f: &crate::state::Foe) -> FoeView {
    let d = creatures::def(f.kind);
    FoeView {
        kind: f.kind,
        name: d.name.to_string(),
        band: f.band,
        health: (f.health.max(0) * 100 / d.health.max(1)).clamp(0, 100),
        icon: "stilled".into(),
    }
}

/// Label, group, icon and danger flag for an action.
fn describe(s: &GameState, a: &Action) -> (String, &'static str, String, bool) {
    use Action::*;
    let iname = |id: &str| items::name(id).to_string();
    let icon_of = |id: &str| items::get(id).map_or("item", |d| d.icon).to_string();
    match a {
        Use { item } => {
            let verb = match items::get(item).map(|d| d.kind) {
                Some(ItemKind::Food { sat, .. }) if sat > 2 => "Eat",
                Some(ItemKind::Food { .. }) => "Drink",
                Some(ItemKind::Medicine(MedKind::Bandage(_))) => "Bind the wound with",
                Some(ItemKind::Medicine(_)) => "Take",
                Some(ItemKind::Clothing { .. }) => "Put on",
                _ => "Use",
            };
            (
                format!("{verb} {}", iname(item).to_lowercase()),
                "survive",
                icon_of(item),
                false,
            )
        }
        Build { station } => {
            let d = stations::def(*station);
            let next = d.levels.get(usize::from(s.level(*station)) + 1).map_or("", |l| l.name);
            (
                format!("Build: {}", next.to_lowercase()),
                "station",
                d.icon.to_string(),
                false,
            )
        }
        Craft { recipe } => {
            let r = recipes::get(recipe);
            (
                r.map_or(recipe.clone(), |r| r.name.to_string()),
                "craft",
                r.map_or("workbench", |r| r.icon).to_string(),
                false,
            )
        }
        RepairBarricade => ("Mend the barricade".into(), "house", "barricade".into(), false),
        FeedStove => ("Feed the stove".into(), "house", "stove".into(), false),
        DrawWater => ("Draw water from the lock".into(), "house", "water".into(), false),
        Forage => ("Work along the bank".into(), "food", "herb".into(), false),
        Rest => ("Lie down for an hour".into(), "house", "rest".into(), false),
        ListenRadio => ("Listen to the wireless".into(), "house", "radio".into(), false),
        // Allowed from 18:00, an hour before dusk, so you can be in and barred
        // before they wake. Say so, rather than look like a mistake.
        GoToBed => (
            if !s.is_dark() && s.time_of_day() >= 18 * 60 {
                "Bar the door early, before dusk".into()
            } else {
                "Bar the door for the night".into()
            },
            "night",
            "cot".into(),
            false,
        ),
        PlanTrip { place } => {
            let p = places::get(place);
            (
                format!("Go to {}", p.map_or(place.as_str(), |p| p.name)),
                "trip",
                p.map_or("map", |p| p.icon).to_string(),
                s.is_dark(),
            )
        }
        SetSnares => ("Set snares along the bank".into(), "food", "trap".into(), false),
        CheckSnares => ("Walk the trap line".into(), "food", "trap".into(), false),
        SetLine => ("Set the night line".into(), "food", "trap".into(), false),
        CheckLine => ("Haul in the night line".into(), "food", "fish".into(), false),
        Plant { seed } => (
            format!("Plant {}", iname(seed).to_lowercase()),
            "food",
            "garden".into(),
            false,
        ),
        TendGarden => ("Water the garden".into(), "food", "garden".into(), false),
        Harvest => ("Lift the crop".into(), "food", "garden".into(), false),
        FeedDog => ("Feed the dog".into(), "house", "sigil_dog".into(), false),
        InstallPart { part } => (
            format!("Fit the {} to the ferry", iname(part).to_lowercase()),
            "ferry",
            "ferry".into(),
            false,
        ),
        CastOff => ("Cast off".into(), "ferry", "ferry".into(), true),
        WalkOntoIce => ("Walk out onto the ice".into(), "ferry", "sigil_ice".into(), true),
        Pack { item, .. } => (
            format!("Pack {}", iname(item).to_lowercase()),
            "pack",
            icon_of(item),
            false,
        ),
        Unpack { item, .. } => (
            format!("Leave {}", iname(item).to_lowercase()),
            "pack",
            icon_of(item),
            false,
        ),
        TakeDog { take } => (
            if *take {
                "Take the dog".into()
            } else {
                "Leave the dog home".into()
            },
            "pack",
            "sigil_dog".into(),
            false,
        ),
        Depart => ("Set out".into(), "pack", "map".into(), s.is_dark()),
        CancelTrip => ("Stay home".into(), "pack", "house".into(), false),
        Walk { hurry } => (
            if *hurry { "Hurry".into() } else { "Walk".into() },
            "move",
            "map".into(),
            *hurry,
        ),
        TurnBack => ("Turn back".into(), "move", "back".into(), false),
        Enter { quiet } => {
            let name = expedition::next_room(s).map_or("further in", |(_, r)| r.name);
            let how = if *quiet { "Go in quietly" } else { "Go in fast" };
            (
                format!("{how}: {}", name.to_lowercase()),
                "move",
                "house".into(),
                !*quiet,
            )
        }
        Back => ("Go back".into(), "move", "back".into(), false),
        Leave => ("Head home".into(), "move", "house".into(), false),
        Search { careful } => (
            if *careful {
                "Search carefully".into()
            } else {
                "Search fast".into()
            },
            "room",
            "pack".into(),
            !*careful,
        ),
        LightUp => {
            let l = inventory::light(s).map_or("light".to_string(), |(l, _)| items::name(l).to_lowercase());
            let ahead = expedition::current_room(s).is_none_or(|(i, _)| expedition::light_target(s) != Some(i));
            let label = if ahead {
                format!("Light the {l} for the dark ahead")
            } else {
                format!("Light the {l}")
            };
            (label, "room", "light".into(), false)
        }
        PickLock => ("Pick the lock".into(), "lock", "lock".into(), false),
        ForceLock => ("Force the door".into(), "lock", "lock".into(), true),
        UseKey => ("Use the key".into(), "lock", "key".into(), false),
        Take { item, .. } => (
            format!("Take {}", iname(item).to_lowercase()),
            "loot",
            icon_of(item),
            false,
        ),
        TakeAll => ("Take what fits".into(), "loot", "pack".into(), false),
        Drop { item, .. } => (
            format!("Leave {} here", iname(item).to_lowercase()),
            "loot",
            icon_of(item),
            false,
        ),
        Ambush => ("Strike before it turns".into(), "fight", "blade".into(), true),
        Trade => ("Trade with the widow".into(), "trade", "sigil_scale".into(), false),
        Strike => {
            let w = inventory::melee(s).map_or("bare hands".to_string(), |(id, _)| items::name(id).to_lowercase());
            (format!("Strike ({w})"), "fight", "blade".into(), false)
        }
        Shoot => {
            let g = inventory::gun(s).map_or("gun".to_string(), |(id, _)| items::name(id).to_lowercase());
            (format!("Shoot ({g})"), "fight", "gun".into(), true)
        }
        Shove => ("Shove it back".into(), "fight", "club".into(), false),
        BackAway => ("Back away".into(), "fight", "back".into(), false),
        HoldStill => ("Hold still".into(), "fight", "nerve".into(), false),
        Flee => ("Run".into(), "fight", "back".into(), true),
        Throw { item } => (
            format!("Throw the {}", iname(item).to_lowercase()),
            "fight",
            "noise".into(),
            false,
        ),
        Choose { option } => {
            let label = crate::systems::events::current(s)
                .and_then(|e| e.opts.get(*option as usize))
                .map_or("Choose", |o| o.label);
            (label.to_string(), "choice", "journal".into(), false)
        }
        Sell { item, .. } => (
            format!("Sell {}", iname(item).to_lowercase()),
            "trade",
            icon_of(item),
            false,
        ),
        Buy { item } => (
            format!("Buy {}", iname(item).to_lowercase()),
            "trade",
            icon_of(item),
            false,
        ),
        EndTrade => ("Done trading".into(), "trade", "back".into(), false),
        Sleep { watch } => (
            if *watch {
                "Sit up by the door, then sleep".into()
            } else {
                "Sleep".into()
            },
            "night",
            "rest".into(),
            false,
        ),
        GetUp => ("Get up again".into(), "night", "house".into(), false),
        Rise => ("Get up".into(), "night", "house".into(), false),
    }
}

pub fn actions(s: &GameState) -> Vec<ActionView> {
    action::candidates(s)
        .into_iter()
        .map(|a| {
            let (label, group, icon, danger) = describe(s, &a);
            let max_qty = action::max_qty(s, &a);
            match action::check(s, &a) {
                Ok(i) => ActionView {
                    action: a,
                    label,
                    group: group.to_string(),
                    icon,
                    minutes: i.minutes,
                    noise: i.noise,
                    odds: i.odds,
                    enabled: true,
                    reason: None,
                    danger,
                    max_qty,
                },
                Err(reason) => ActionView {
                    action: a,
                    label,
                    group: group.to_string(),
                    icon,
                    minutes: 0,
                    noise: 0,
                    odds: None,
                    enabled: false,
                    reason: Some(reason),
                    danger,
                    max_qty: 0,
                },
            }
        })
        .collect()
}

fn weather_info(w: Weather) -> WeatherInfo {
    WeatherInfo {
        id: w,
        name: w.name().to_string(),
        icon: w.icon().to_string(),
        effect: w.effect().to_string(),
    }
}

fn stations_view(s: &GameState) -> Vec<StationView> {
    Station::ALL
        .iter()
        .map(|st| {
            let d = stations::def(*st);
            let lvl = s.level(*st);
            let cur = &d.levels[usize::from(lvl)];
            let status = match st {
                Station::Stove if s.fire_until > s.minute => {
                    Some(format!("Burning until {}", journal::clock(s.fire_until)))
                }
                Station::Stove => Some("Cold".into()),
                Station::Barricade => Some(format!("Holds {} of {}", s.barricade, stations::barricade_max(lvl))),
                Station::Barrel => Some(format!(
                    "Water {} of {}",
                    inventory::water_stored(s),
                    inventory::water_capacity(s)
                )),
                Station::Garden if lvl > 0 => Some(format!(
                    "{} of {} plots planted",
                    s.garden.len(),
                    stations::garden_plots(lvl)
                )),
                Station::Radio if lvl > 0 => Some(format!(
                    "Forecasts right about {}% of the time",
                    stations::forecast_pct(lvl)
                )),
                _ => None,
            };
            let next = d.levels.get(usize::from(lvl) + 1).map(|n| {
                let ok = home::can_build(s, *st);
                NextLevelView {
                    level: lvl + 1,
                    name: n.name.to_string(),
                    effect: n.effect.to_string(),
                    cost: cost_view(s, n.cost, true),
                    workbench: n.workbench,
                    minutes: home::build_minutes(s, *st),
                    noise: home::build_noise(s, *st),
                    can_build: ok.is_ok(),
                    reason: ok.err(),
                }
            });
            StationView {
                id: *st,
                name: d.name.to_string(),
                icon: d.icon.to_string(),
                level: lvl,
                level_name: cur.name.to_string(),
                effect: cur.effect.to_string(),
                status,
                next,
            }
        })
        .collect()
}

fn recipes_view(s: &GameState) -> Vec<RecipeView> {
    recipes::RECIPES
        .iter()
        .map(|r| {
            let ok = home::can_craft(s, r);
            RecipeView {
                id: r.id.to_string(),
                name: r.name.to_string(),
                icon: r.icon.to_string(),
                inputs: cost_view(s, r.inputs, true),
                outputs: cost_view(s, r.outputs, true),
                station: r.station.map(|(st, _)| st),
                station_level: r.station.map_or(0, |(_, l)| l),
                fire: r.fire,
                minutes: home::craft_minutes(s, r),
                can_craft: ok.is_ok(),
                reason: ok.err(),
            }
        })
        .collect()
}

fn map_view(s: &GameState) -> MapView {
    let mut nodes = vec![MapNode {
        id: "home".into(),
        name: "The lock house".into(),
        desc: "Your house on the lock, below the weir. The ferry at the far slip.".into(),
        icon: "house".into(),
        x: places::HOME_X,
        y: places::HOME_Y,
        minutes: 0,
        visited: true,
        rooms: 0,
        rooms_searched: 0,
        notes: vec![],
    }];
    let mut known = 0;
    for p in places::PLACES {
        let Some(ps) = s.places.get(p.id) else { continue };
        if !ps.known {
            continue;
        }
        known += 1;
        let mut notes = Vec::new();
        if p.id == "weir" && s.widow.met {
            match barter::next_visit(s) {
                Some(0) => notes.push("The widow is here today.".to_string()),
                Some(n) => notes.push(format!(
                    "The widow comes again in {n} day{}.",
                    if n == 1 { "" } else { "s" }
                )),
                None => {}
            }
        }
        let searched = ps.rooms.iter().filter(|r| r.search == SearchState::Full).count() as u32;
        if ps.visited && searched as usize == p.rooms.len() {
            notes.push("Searched through.".into());
        }
        let left: u32 = ps.rooms.iter().map(|r| r.pile.values().sum::<u32>()).sum();
        if left > 0 {
            notes.push(format!(
                "You left {left} thing{} here.",
                if left == 1 { "" } else { "s" }
            ));
        }
        nodes.push(MapNode {
            id: p.id.to_string(),
            name: p.name.to_string(),
            desc: p.desc.to_string(),
            icon: p.icon.to_string(),
            x: p.x,
            y: p.y,
            minutes: p.minutes,
            visited: ps.visited,
            rooms: p.rooms.len() as u32,
            rooms_searched: searched,
            notes,
        });
    }
    let shown = |id: &str| nodes.iter().any(|n| n.id == id);
    let paths = places::PATHS
        .iter()
        .filter(|(a, b)| shown(a) && shown(b))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    MapView {
        nodes,
        paths,
        known,
        total: places::PLACES.len() as u32,
    }
}

fn codex_view(s: &GameState) -> CodexView {
    let sections: Vec<CodexSection> = CodexKind::ALL
        .iter()
        .map(|k| {
            let entries: Vec<CodexEntryView> = codex::CODEX
                .iter()
                .filter(|c| c.kind == *k && s.codex.contains(c.id))
                .map(CodexEntryView::of)
                .collect();
            CodexSection {
                kind: *k,
                name: k.name().to_string(),
                icon: k.icon().to_string(),
                found: entries.len() as u32,
                total: codex::count(*k) as u32,
                entries,
            }
        })
        .collect();
    CodexView {
        found: s.codex.len() as u32,
        total: codex::CODEX.len() as u32,
        sections,
    }
}

fn site_view(s: &GameState) -> Option<SiteView> {
    if !matches!(s.phase, Phase::Site | Phase::Encounter | Phase::Event | Phase::Barter) {
        return None;
    }
    let t = s.trip.as_ref()?;
    let p = places::get(&t.place)?;
    let ps = s.places.get(&t.place)?;
    let next = t.room.map_or(0, |r| r + 1);
    let rooms = p
        .rooms
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let rs = &ps.rooms[i.min(ps.rooms.len().saturating_sub(1))];
            let seen = rs.entered || i == next;
            RoomSummary {
                index: i as u32,
                name: if seen { r.name.to_string() } else { "Further in".into() },
                entered: rs.entered,
                here: t.room == Some(i),
                dark: seen && r.dark,
                flooded: seen && r.flooded,
                locked: seen && r.lock.is_some() && !rs.unlocked,
                search: rs.search,
            }
        })
        .collect();
    let room = expedition::current_room(s).map(|(i, r)| {
        let rs = &ps.rooms[i];
        RoomView {
            index: i as u32,
            name: r.name.to_string(),
            desc: r.desc.to_string(),
            dark: r.dark,
            lit: rs.lit,
            flooded: r.flooded,
            search: rs.search,
            pile: items_of(s, &rs.pile),
            dormant: rs.foes.iter().map(foe_view).collect(),
        }
    });
    Some(SiteView {
        place: p.id.to_string(),
        name: p.name.to_string(),
        desc: p.desc.to_string(),
        icon: p.icon.to_string(),
        rooms,
        room,
        site_noise: t.site_noise,
        widow_here: p.id == "weir" && barter::present(s),
    })
}

/// What the place is known to need, from rooms already seen there.
fn packing_needs(s: &GameState, p: &places::PlaceDef) -> Vec<NeedView> {
    let mut v = Vec::new();
    let Some(ps) = s.places.get(p.id) else { return v };
    // The first door is always seen; the rest once entered (or the door beyond).
    let seen = |i: usize| {
        i == 0 || ps.rooms.get(i).is_some_and(|r| r.entered) || ps.rooms.get(i - 1).is_some_and(|r| r.entered)
    };
    let pack_has =
        |pred: &dyn Fn(&ItemKind) -> bool| s.pack.keys().any(|id| items::get(id).is_some_and(|d| pred(&d.kind)));
    let mut need = |id: &str, text: &str, icon: &str, met: bool| {
        v.push(NeedView {
            id: id.into(),
            text: text.into(),
            icon: icon.into(),
            met,
        })
    };
    let rooms: Vec<(usize, &places::RoomDef)> = p.rooms.iter().enumerate().filter(|(i, _)| seen(*i)).collect();
    if rooms
        .iter()
        .any(|(i, r)| r.dark && ps.rooms[*i].search != SearchState::Full)
    {
        need(
            "light",
            "Dark rooms. A light.",
            "light",
            pack_has(&|k| matches!(k, ItemKind::Light { .. })),
        );
    }
    let locked: Vec<&places::Lock> = rooms
        .iter()
        .filter(|(i, _)| !ps.rooms[*i].unlocked)
        .filter_map(|(_, r)| r.lock.as_ref())
        .collect();
    if !locked.is_empty() {
        let key = locked
            .iter()
            .filter_map(|l| l.key)
            .any(|k| crate::state::inv_count(&s.pack, k) > 0);
        let picks = locked.iter().any(|l| l.pick.is_some()) && crate::state::inv_count(&s.pack, "lockpicks") > 0;
        let force = ["crowbar", "hatchet", "hammer"]
            .iter()
            .any(|t| crate::state::inv_count(&s.pack, t) > 0);
        need(
            "lock",
            "A locked door. The key, picks, or something to force it.",
            "lock",
            key || picks || force,
        );
    }
    if rooms
        .iter()
        .any(|(i, r)| r.flooded && ps.rooms[*i].search != SearchState::Full)
    {
        need("flooded", "Flooded rooms. Waders.", "water", inventory::waders(s));
    }
    if rooms.iter().any(|(_, r)| r.danger >= 2) || ps.rooms.iter().any(|r| !r.foes.is_empty()) {
        need(
            "weapon",
            "They are there. Something to strike with.",
            "blade",
            s.pack.keys().any(|id| items::weapon(id).is_some()),
        );
    }
    v
}

fn location(s: &GameState) -> String {
    match (&s.trip, s.phase) {
        (None, Phase::Packing) => "The lock house, packing.".into(),
        (None, _) => "The lock house.".into(),
        (Some(t), Phase::Travelling) => {
            let name = places::get(&t.place).map_or("", |p| p.name);
            if s.homeward {
                format!("On the road home from {name}.")
            } else {
                format!("On the road to {name}.")
            }
        }
        (Some(t), _) => {
            let p = places::get(&t.place);
            let name = p.map_or("", |p| p.name);
            match expedition::current_room(s) {
                Some((_, r)) => format!("{name}: {}.", r.name),
                None => format!("Outside {name}."),
            }
        }
    }
}

fn dark_at(minute: u32) -> bool {
    let t = minute % crate::state::DAY;
    !(crate::state::DAWN..DUSK).contains(&t)
}

pub fn view(s: &GameState) -> View {
    let t = s.time_of_day();
    let vitals = VitalKind::ALL
        .iter()
        .map(|k| {
            let v = s.vitals.pts(*k);
            VitalView {
                id: *k,
                name: k.name().to_string(),
                value: v,
                icon: k.icon().to_string(),
                level: if v < 15 {
                    "critical"
                } else if v < 35 {
                    "low"
                } else {
                    "ok"
                }
                .to_string(),
            }
        })
        .collect();
    let mut conditions = Vec::new();
    if s.bleeding > 0 {
        conditions.push(ConditionView {
            id: "bleeding".into(),
            name: "Bleeding".into(),
            detail: ["", "A steady bleed.", "Bleeding badly.", "Bleeding heavily."][usize::from(s.bleeding.min(3))]
                .into(),
            icon: "health".into(),
        });
    }
    if s.infection > 0 {
        let pts = s.infection / crate::state::MILLI;
        conditions.push(ConditionView {
            id: "infection".into(),
            name: "Infected".into(),
            detail: format!(
                "{} ({pts})",
                if pts < 30 {
                    "The wound is hot."
                } else if pts < 60 {
                    "Fever."
                } else {
                    "The fever is winning."
                }
            ),
            icon: "medicine".into(),
        });
    }
    if s.sick_until > s.minute {
        conditions.push(ConditionView {
            id: "sick".into(),
            name: "Sick".into(),
            detail: "Bad water or bad food. It will pass, if you drink.".into(),
            icon: "water".into(),
        });
    }

    let encounter = s
        .encounter
        .as_ref()
        .filter(|_| s.phase == Phase::Encounter)
        .map(|e| EncounterView {
            foes: e.foes.iter().map(foe_view).collect(),
            turn: e.turn,
            gripped: e.gripped,
            at_home: e.at_home,
            dark: s.is_dark() || !expedition::is_lit(s),
            log: e.log.clone(),
            weapon: inventory::melee(s).map_or("bare hands".to_string(), |(id, _)| items::name(id).to_string()),
            gun: s
                .inv()
                .keys()
                .find(|id| items::weapon(id).is_some_and(|w| w.ammo.is_some()))
                .map(|id| items::name(id).to_string()),
            ammo: s
                .inv()
                .keys()
                .filter_map(|id| items::weapon(id).and_then(|w| w.ammo))
                .next()
                .map_or(0, |a| crate::state::inv_count(s.inv(), a)),
            ammo_name: s
                .inv()
                .keys()
                .filter_map(|id| items::weapon(id).and_then(|w| w.ammo))
                .next()
                .map(|a| items::name(a).to_string()),
        });
    let event = if s.phase == Phase::Event {
        crate::systems::events::current(s).map(|def| EventView {
            id: def.id.to_string(),
            title: def.title.to_string(),
            text: def.text.to_string(),
            options: def
                .opts
                .iter()
                .enumerate()
                .map(|(i, o): (usize, &ev::Opt)| {
                    let ok = crate::systems::events::option_ok(s, o);
                    OptionView {
                        index: i as u32,
                        label: o.label.to_string(),
                        enabled: ok.is_ok(),
                        reason: ok.err(),
                        odds: crate::systems::events::odds(s, &o.odds),
                        minutes: o.minutes,
                    }
                })
                .collect(),
        })
    } else {
        None
    };
    let packing = if s.phase == Phase::Packing {
        s.dest.as_deref().and_then(places::get).map(|p| PackingView {
            place: p.id.to_string(),
            name: p.name.to_string(),
            minutes: clock::cost(s, p.minutes),
            take_dog: s.has_flag("take_dog") && s.dog.state == DogState::Companion,
            has_light: s
                .pack
                .keys()
                .any(|id| matches!(items::get(id).map(|d| d.kind), Some(ItemKind::Light { .. }))),
            has_weapon: s.pack.keys().any(|id| items::weapon(id).is_some()),
            meals: inventory::meal_count(&s.pack),
            water: inventory::water_count(&s.pack),
            needs: packing_needs(s, p),
            unvisited: s.places.get(p.id).is_none_or(|ps| !ps.visited),
            rooms: p.rooms.len() as u32,
            rooms_searched: s.places.get(p.id).map_or(0, |ps| {
                ps.rooms.iter().filter(|r| r.search == SearchState::Full).count() as u32
            }),
        })
    } else {
        None
    };
    let travel = if s.phase == Phase::Travelling {
        s.trip.as_ref().and_then(|t| places::get(&t.place)).map(|p| {
            let m = expedition::walk_minutes(s, false);
            TravelView {
                place: p.id.to_string(),
                name: p.name.to_string(),
                homeward: s.homeward,
                minutes: m,
                hurry_minutes: expedition::walk_minutes(s, true),
                arrive_dark: dark_at(s.minute + m),
                arrive_at: journal::clock(s.minute + m),
                hurry_arrive_at: journal::clock(s.minute + expedition::walk_minutes(s, true)),
                hurry_arrive_dark: dark_at(s.minute + expedition::walk_minutes(s, true)),
            }
        })
    } else {
        None
    };
    let barter_view = if s.phase == Phase::Barter {
        Some(BarterView {
            line: if s.widow.credit > 0 {
                format!(
                    "The slate reads {}. She waits. What is left on it when you go is lost: no change.",
                    s.widow.credit
                )
            } else if s.widow.trades == 0 {
                "She writes on her slate: TRADE. NO CHANGE. Then she waits.".to_string()
            } else {
                "She wipes her slate and waits. Sell first; she chalks what it is worth.".to_string()
            },
            credit: s.widow.credit,
            offers: s
                .widow
                .stock
                .iter()
                .map(|o| OfferView {
                    item: o.item.clone(),
                    name: items::name(&o.item).to_string(),
                    icon: items::get(&o.item).map_or("item", |d| d.icon).to_string(),
                    qty: o.qty,
                    price: o.price,
                })
                .collect(),
            sell: s
                .pack
                .iter()
                .filter(|(id, _)| barter::sell_price(id) > 0)
                .map(|(id, n)| SellView {
                    item: id.clone(),
                    name: items::name(id).to_string(),
                    icon: items::get(id).map_or("item", |d| d.icon).to_string(),
                    qty: *n,
                    price: barter::sell_price(id),
                })
                .collect(),
            trades: s.widow.trades,
        })
    } else {
        None
    };
    let dog_view = match s.dog.state {
        DogState::Unmet => None,
        st => Some(DogView {
            state: st,
            health: s.dog.health,
            fed: s.dog.fed.clamp(0, 100),
            with_you: dog::with_you(s) && !s.at_home(),
            line: match st {
                DogState::Following => "A rope-collared dog hangs about the woodpile.".into(),
                DogState::Companion if s.dog.fed <= 0 => "The dog is hungry.".into(),
                DogState::Companion if s.dog.health < 40 => "The dog is hurt.".into(),
                DogState::Companion => "The dog is with you.".into(),
                DogState::Lost => "The dog is gone.".into(),
                DogState::Dead => "The dog is buried by the lock.".into(),
                DogState::Unmet => String::new(),
            },
        }),
    };
    let lvl_barricade = s.level(Station::Barricade);
    let weather_v = WeatherView {
        today: weather_info(s.weather),
        forecast: s.forecast.map(weather_info),
        forecast_accuracy: stations::forecast_pct(s.level(Station::Radio)),
    };
    let freeze_in = if s.freeze_known && !s.frozen {
        Some(s.freeze_day.saturating_sub(s.day()))
    } else {
        None
    };
    let season_note = if s.frozen {
        "The river is frozen.".to_string()
    } else if let Some(n) = freeze_in {
        format!("The river closes in about {}.", journal::count(n, "day", "days"))
    } else {
        "The river will freeze. You do not know when.".to_string()
    };
    let bg = backgrounds::get(&s.background);
    View {
        phase: s.phase,
        day: s.day(),
        clock: journal::clock(s.minute),
        minute_of_day: t,
        dark: s.is_dark(),
        daylight_left: if s.is_dark() { None } else { Some(DUSK - t) },
        season: SeasonView {
            temperature: weather::temperature(s),
            frozen: s.frozen,
            freeze_in,
            note: season_note,
        },
        weather: weather_v,
        vitals,
        conditions,
        noise: NoiseView {
            value: s.noise,
            label: noise::label(s.noise).to_string(),
            visit_chance: night::visit_chance(s),
        },
        location: location(s),
        actions: actions(s),
        objectives: objectives::list(s),
        inventory: InventoryView {
            store: items_of(s, &s.store),
            pack: items_of(s, &s.pack),
            worn: s.worn.values().map(|id| item_view(s, id, 1)).collect(),
            carried: inventory::carried(s),
            capacity: inventory::capacity(s),
            carry_label: format!("{} of {}", kg(inventory::carried(s)), kg(inventory::capacity(s))),
            water: inventory::water_stored(s),
            water_capacity: inventory::water_capacity(s),
        },
        stations: stations_view(s),
        recipes: recipes_view(s),
        grounds: GroundsView {
            snares_set: s.traps.snares,
            snared: s.traps.snared,
            line_set: s.traps.line,
            line_baited: s.traps.baited,
            hooked: s.traps.hooked,
            plots: s
                .garden
                .iter()
                .map(|p| PlotView {
                    crop: p.crop.clone(),
                    name: items::name(&p.crop).to_string(),
                    growth: p.growth,
                    need: p.need,
                    ready: p.growth >= p.need,
                })
                .collect(),
            plots_total: stations::garden_plots(s.level(Station::Garden)) as u32,
            tended_today: s.tended,
            fire_until: if s.fire_until > s.minute {
                Some(journal::clock(s.fire_until))
            } else {
                None
            },
            barricade: s.barricade,
            barricade_max: stations::barricade_max(lvl_barricade),
        },
        map: map_view(s),
        codex: codex_view(s),
        skills: Skill::ALL
            .iter()
            .map(|k| {
                let xp = s.skills.get(k).copied().unwrap_or(0);
                SkillView {
                    id: *k,
                    name: k.name().to_string(),
                    icon: k.icon().to_string(),
                    level: skills::level(xp),
                    xp,
                    next: skills::next(xp),
                    effect: k.effect().to_string(),
                }
            })
            .collect(),
        dog: dog_view,
        ferry: FerryView {
            parts: items::FERRY_PARTS
                .iter()
                .map(|p| FerryPartView {
                    id: p.to_string(),
                    name: items::name(p).to_string(),
                    icon: items::get(p).map_or("part", |d| d.icon).to_string(),
                    installed: s.ferry.contains(*p),
                    in_store: crate::state::inv_count(&s.store, p) > 0,
                    need: if s.ferry.contains(*p) {
                        None
                    } else if crate::state::inv_count(&s.store, p) > 0 {
                        Some(match home::can_install(s, p) {
                            Ok(()) => "In the store. Fit it.".to_string(),
                            Err(why) => why,
                        })
                    } else {
                        Some(objectives::where_part(s, p))
                    },
                })
                .collect(),
            installed: s.ferry.len() as u32,
            ready: s.ferry.len() == 4 && !s.frozen,
            frozen: s.frozen,
            meals: inventory::meal_count(&s.store),
            meals_need: home::VOYAGE_MEALS,
            water: crate::state::inv_count(&s.store, "water_clean"),
            water_need: home::VOYAGE_WATER,
        },
        packing,
        travel,
        site: site_view(s),
        encounter,
        event,
        barter: barter_view,
        dawn: if s.phase == Phase::Dawn { s.dawn.clone() } else { None },
        ending: s.ending.as_ref().map(|e| EndingView {
            kind: e.kind,
            cause: e.cause.clone(),
            text: e.text.clone(),
        }),
        ledger: s.ledger.clone(),
        journal: s.journal.iter().rev().take(JOURNAL_IN_VIEW).rev().cloned().collect(),
        marks: s
            .marks
            .iter()
            .filter_map(|id| crate::content::marks::get(id))
            .map(|m| MarkEarned {
                id: m.id.to_string(),
                name: m.name.to_string(),
                text: m.text.to_string(),
                icon: m.icon.to_string(),
                tier: m.tier,
                new: !s.held_marks.contains(m.id),
            })
            .collect(),
        run: RunInfo {
            seed: s.seed,
            difficulty: s.difficulty,
            difficulty_name: s.difficulty.name().to_string(),
            background: s.background.clone(),
            background_name: bg.map_or("", |b| b.name).to_string(),
            keepsake: s.keepsake.clone(),
            daily: s.daily.clone(),
            nights: s.stats.nights,
            kills: s.stats.kills,
        },
    }
}

/// Encounters say which encounter odds apply; re-exported for the bot.
pub fn encounter_nearest(s: &GameState) -> Option<Band> {
    encounter::nearest(s)
}
