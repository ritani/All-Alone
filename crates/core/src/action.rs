//! Actions: the only way the UI changes the game.
//!
//! `check` decides whether an action is allowed right now (and what it will
//! cost); `dispatch` runs it. The view's action list is built from the same
//! `candidates` + `check`, so the UI never re-implements a rule.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::content::codex::CodexKind;
use crate::content::items::{self, ItemKind};
use crate::content::{Creature, Skill, Station, VitalKind, places, recipes, stations};
use crate::state::{DAWN, DogState, EndKind, Entry, GameState, Phase, SearchState, inv_count};
use crate::systems::encounter::{self, Move};
use crate::systems::{barter, clock, dog, events, expedition, food, home, inventory, night, noise};
use crate::{GameError, Journal};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(tag = "type", rename_all = "snake_case")]
// ts-rs cannot read this attribute on enums; the `ts` feature only generates types.
#[cfg_attr(not(feature = "ts"), serde(deny_unknown_fields))]
pub enum Action {
    // Anywhere it makes sense
    Use { item: String },
    // The lock house
    Build { station: Station },
    Craft { recipe: String },
    RepairBarricade,
    FeedStove,
    DrawWater,
    Forage,
    Rest,
    ListenRadio,
    GoToBed,
    PlanTrip { place: String },
    SetSnares,
    CheckSnares,
    SetLine,
    CheckLine,
    Plant { seed: String },
    TendGarden,
    Harvest,
    FeedDog,
    InstallPart { part: String },
    CastOff,
    WalkOntoIce,
    // Packing
    Pack { item: String, qty: u32 },
    Unpack { item: String, qty: u32 },
    TakeDog { take: bool },
    Depart,
    CancelTrip,
    // The road
    Walk { hurry: bool },
    TurnBack,
    // At a place
    Enter { quiet: bool },
    Back,
    Leave,
    Search { careful: bool },
    LightUp,
    PickLock,
    ForceLock,
    UseKey,
    Take { item: String, qty: u32 },
    TakeAll,
    Drop { item: String, qty: u32 },
    Ambush,
    Trade,
    // Encounters
    Strike,
    Shoot,
    Shove,
    BackAway,
    HoldStill,
    Flee,
    Throw { item: String },
    // Events
    Choose { option: u32 },
    // Barter
    Sell { item: String, qty: u32 },
    Buy { item: String },
    EndTrade,
    // Night and dawn
    Sleep { watch: bool },
    GetUp,
    Rise,
}

/// Typed events for haptics, toasts and marks. The journal carries the prose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameEvent {
    Found {
        item: String,
        qty: u32,
    },
    Lost {
        item: String,
        qty: u32,
    },
    Hurt {
        amount: u32,
    },
    Wounded {
        bleeding: bool,
        infected: bool,
    },
    Sick,
    VitalLow {
        vital: VitalKind,
    },
    Kill {
        kind: Option<Creature>,
    },
    Shot,
    Noise {
        amount: u32,
        total: u32,
    },
    EncounterStarted,
    EncounterOver,
    EventStarted {
        id: String,
    },
    Codex {
        id: String,
        kind: CodexKind,
    },
    PlaceFound {
        place: String,
    },
    Built {
        station: Station,
        level: u8,
    },
    Crafted {
        recipe: String,
    },
    SkillUp {
        skill: Skill,
        level: u8,
    },
    NightVisit {
        visitors: u32,
        breached: bool,
    },
    Dawn {
        day: u32,
    },
    Dusk,
    Freeze,
    DogJoined,
    DogLost {
        dead: bool,
    },
    PartInstalled {
        part: String,
    },
    /// A mark earned in this run; `new` is false if the profile already held it.
    Mark {
        id: String,
        new: bool,
        /// The mark's name and line, revealed now even if it was hidden.
        name: String,
        text: String,
        icon: String,
        tier: u8,
    },
    RunEnded {
        kind: EndKind,
    },
}

/// What one dispatched action produced.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct Outcome {
    pub journal: Vec<Entry>,
    pub events: Vec<GameEvent>,
}

impl Outcome {
    pub fn clear(&mut self) {
        self.journal.clear();
        self.events.clear();
    }
}

/// What an allowed action will cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Info {
    pub minutes: u32,
    pub noise: u32,
    /// Percent chance of success, where the player should see it.
    pub odds: Option<i32>,
}

fn info(minutes: u32, noise: u32) -> Info {
    Info {
        minutes,
        noise,
        odds: None,
    }
}

fn odds(minutes: u32, noise: u32, odds: i32) -> Info {
    Info {
        minutes,
        noise,
        odds: Some(odds),
    }
}

type Check = Result<Info, String>;

fn need(ok: bool, reason: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(reason.to_string()) }
}

fn phase_allows(p: Phase, a: &Action) -> bool {
    use Action::*;
    match p {
        Phase::Home => matches!(
            a,
            Use { .. }
                | Build { .. }
                | Craft { .. }
                | RepairBarricade
                | FeedStove
                | DrawWater
                | Forage
                | Rest
                | ListenRadio
                | GoToBed
                | PlanTrip { .. }
                | SetSnares
                | CheckSnares
                | SetLine
                | CheckLine
                | Plant { .. }
                | TendGarden
                | Harvest
                | FeedDog
                | InstallPart { .. }
                | CastOff
                | WalkOntoIce
        ),
        Phase::Packing => matches!(a, Pack { .. } | Unpack { .. } | TakeDog { .. } | Depart | CancelTrip),
        Phase::Travelling => matches!(a, Walk { .. } | TurnBack | Use { .. }),
        Phase::Site => matches!(
            a,
            Enter { .. }
                | Back
                | Leave
                | Search { .. }
                | LightUp
                | PickLock
                | ForceLock
                | UseKey
                | Take { .. }
                | TakeAll
                | Drop { .. }
                | Ambush
                | Trade
                | Use { .. }
        ),
        Phase::Encounter => matches!(
            a,
            Strike | Shoot | Shove | BackAway | HoldStill | Flee | Throw { .. } | Use { .. }
        ),
        Phase::Event => matches!(a, Choose { .. }),
        Phase::Barter => matches!(a, Sell { .. } | Buy { .. } | EndTrade),
        Phase::Night => matches!(a, Sleep { .. } | GetUp | Use { .. }),
        Phase::Dawn => matches!(a, Rise),
        Phase::Ended => false,
    }
}

/// Late at home: only bed, the stove and eating are left.
fn too_late(s: &GameState, minutes: u32) -> bool {
    let t = s.time_of_day();
    t < DAWN || t + minutes > 23 * 60 + 30
}

const LATE: &str = "It is too late for that. Bar the door and sleep.";

fn home_time(s: &GameState, minutes: u32) -> Result<(), String> {
    need(!too_late(s, minutes), LATE)
}

fn daylight(s: &GameState) -> Result<(), String> {
    need(!s.is_dark(), "Not in the dark.")
}

fn recipe(id: &str) -> Result<&'static recipes::RecipeDef, String> {
    recipes::get(id).ok_or_else(|| "Unknown recipe.".to_string())
}

fn known_item(id: &str) -> Result<&'static items::ItemDef, String> {
    items::get(id).ok_or_else(|| "Unknown item.".to_string())
}

/// Giving up a bag must not leave you carrying more than you can.
fn keeps_load(s: &GameState, item: &str, qty: u32) -> Result<(), String> {
    if !matches!(items::get(item).map(|d| d.kind), Some(ItemKind::Bag { .. })) {
        return Ok(());
    }
    let w = items::get(item).map_or(0, |d| d.weight) * qty;
    let after = inventory::carried(s).saturating_sub(w);
    need(
        after <= inventory::capacity_without(s, item),
        "Then you could not carry the rest.",
    )
}

/// Is this action allowed now, and what will it cost?
pub fn check(s: &GameState, a: &Action) -> Check {
    if s.ended() {
        return Err("The run is over.".into());
    }
    if !phase_allows(s.phase, a) {
        return Err("Not now.".into());
    }
    use Action::*;
    match a {
        Use { item } => {
            known_item(item)?;
            home::can_use(s, item)?;
            if s.phase == Phase::Encounter {
                need(
                    matches!(items::get(item).map(|d| d.kind), Some(ItemKind::Medicine(_))),
                    "Not while they are this close.",
                )?;
                return Ok(info(1, 0));
            }
            Ok(info(home::use_minutes(item), 0))
        }
        Build { station } => {
            home::can_build(s, *station)?;
            let m = home::build_minutes(s, *station);
            home_time(s, m)?;
            Ok(info(
                m,
                noise::effective(s, home::build_noise(s, *station), noise::Kind::Other),
            ))
        }
        Craft { recipe: id } => {
            let r = recipe(id)?;
            home::can_craft(s, r)?;
            let m = home::craft_minutes(s, r);
            home_time(s, m)?;
            Ok(info(
                m,
                noise::effective(s, home::craft_noise(s, r), noise::Kind::Other),
            ))
        }
        RepairBarricade => {
            home::can_repair_barricade(s)?;
            home_time(s, 40)?;
            Ok(info(clock::cost(s, 40), noise::effective(s, 3, noise::Kind::Other)))
        }
        FeedStove => {
            need(inv_count(&s.store, "firewood") > 0, "No firewood.")?;
            need(s.fire_until < s.minute + 12 * 60, "The stove is full.")?;
            Ok(info(5, 0))
        }
        DrawWater => {
            need(home::draw_amount(s) > 0, "Every container is full.")?;
            let m = home::draw_minutes(s);
            home_time(s, m)?;
            Ok(info(
                m,
                noise::effective(s, if s.frozen { 5 } else { 1 }, noise::Kind::Other),
            ))
        }
        Forage => {
            daylight(s)?;
            home_time(s, 60)?;
            Ok(info(clock::cost(s, 60), noise::effective(s, 2, noise::Kind::Movement)))
        }
        Rest => {
            need(s.vitals.pts(VitalKind::Rest) < 95, "You are not tired.")?;
            home_time(s, 60)?;
            Ok(info(60, 0))
        }
        ListenRadio => {
            need(s.level(Station::Radio) >= 1, "The wireless is dead. Build it up first.")?;
            need(
                s.radio_day != s.day(),
                "You have listened today. Nothing new until tomorrow.",
            )?;
            home_time(s, 30)?;
            Ok(info(30, 0))
        }
        GoToBed => {
            let t = s.time_of_day();
            need(!(DAWN..18 * 60).contains(&t), "It is still day.")?;
            Ok(info(0, 0))
        }
        PlanTrip { place } => {
            let p = s.places.get(place).ok_or("Unknown place.")?;
            need(p.known, "You do not know the way.")?;
            home_time(s, 0)?;
            Ok(info(0, 0))
        }
        SetSnares => {
            need(inv_count(&s.store, "snare") > 0, "No snares.")?;
            daylight(s)?;
            home_time(s, 30)?;
            Ok(info(clock::cost(s, 30), noise::effective(s, 1, noise::Kind::Movement)))
        }
        CheckSnares => {
            need(s.traps.snares > 0, "No snares set.")?;
            daylight(s)?;
            home_time(s, 30)?;
            Ok(info(clock::cost(s, 30), noise::effective(s, 1, noise::Kind::Movement)))
        }
        SetLine => {
            need(!s.traps.line, "The line is already set.")?;
            need(inv_count(&s.store, "fishing_line") > 0, "No night line.")?;
            need(!s.frozen, "The river is frozen.")?;
            home_time(s, 15)?;
            Ok(info(15, 0))
        }
        CheckLine => {
            need(s.traps.line, "No line set.")?;
            home_time(s, 15)?;
            Ok(info(15, 0))
        }
        Plant { seed } => {
            let d = known_item(seed)?;
            need(matches!(d.kind, ItemKind::Seed { .. }), "That will not grow.")?;
            need(s.level(Station::Garden) >= 1, "Build the garden bed first.")?;
            need(inv_count(&s.store, seed) > 0, "You have none.")?;
            need(food::plots_free(s) > 0, "Every plot is planted.")?;
            daylight(s)?;
            home_time(s, 20)?;
            Ok(info(20, 0))
        }
        TendGarden => {
            need(!s.garden.is_empty(), "Nothing is planted.")?;
            need(!s.tended, "Already watered today.")?;
            need(inventory::water_count(&s.store) > 0, "No water to spare.")?;
            daylight(s)?;
            home_time(s, 20)?;
            Ok(info(20, 0))
        }
        Harvest => {
            need(food::ready(s) > 0, "Nothing is ready.")?;
            daylight(s)?;
            home_time(s, 30)?;
            Ok(info(30, 0))
        }
        FeedDog => {
            need(s.dog.state == DogState::Companion, "There is no dog.")?;
            need(s.dog.fed < 80, "The dog has eaten.")?;
            need(inventory::meal_count(&s.store) > 0, "Nothing to give it.")?;
            Ok(info(5, 0))
        }
        InstallPart { part } => {
            need(items::is_part(part), "That is not a ferry part.")?;
            home::can_install(s, part)?;
            let m = home::install_minutes(s);
            home_time(s, m)?;
            Ok(info(
                m,
                noise::effective(s, if part == "hull_patch" { 10 } else { 3 }, noise::Kind::Other),
            ))
        }
        CastOff => {
            home::can_cast_off(s)?;
            Ok(info(0, 0))
        }
        WalkOntoIce => {
            need(s.frozen, "The river is still open.")?;
            daylight(s)?;
            Ok(info(0, 0))
        }
        Pack { item, qty } => {
            known_item(item)?;
            need(*qty > 0, "Nothing to pack.")?;
            need(inv_count(&s.store, item) >= *qty, "You do not have that many.")?;
            let w = items::get(item).map_or(0, |d| d.weight) * qty;
            let left = inventory::capacity(s).saturating_sub(inventory::carried(s));
            need(w <= left, "It will not fit.")?;
            Ok(info(0, 0))
        }
        Unpack { item, qty } => {
            need(*qty > 0, "Nothing to unpack.")?;
            need(inv_count(&s.pack, item) >= *qty, "You are not carrying that many.")?;
            Ok(info(0, 0))
        }
        TakeDog { take } => {
            need(s.dog.state == DogState::Companion, "There is no dog.")?;
            need(*take != s.has_flag("take_dog"), "Already so.")?;
            Ok(info(0, 0))
        }
        Depart => Ok(info(0, 0)),
        CancelTrip => Ok(info(0, 0)),
        Walk { hurry } => {
            let m = expedition::walk_minutes(s, *hurry);
            Ok(info(
                m,
                if *hurry {
                    noise::effective(s, 6, noise::Kind::Movement)
                } else {
                    0
                },
            ))
        }
        TurnBack => {
            need(!s.homeward, "You are already going home.")?;
            Ok(info(0, 0))
        }
        Enter { quiet } => {
            let (i, r) = expedition::next_room(s).ok_or("There is nowhere further in.")?;
            need(!expedition::next_locked(s), "The door is locked.")?;
            if r.dark && s.vitals.pts(VitalKind::Nerve) < 10 {
                return Err("Your hands will not open the door.".into());
            }
            let _ = i;
            Ok(info(
                expedition::enter_minutes(s, *quiet),
                noise::effective(s, if *quiet { 0 } else { 4 }, noise::Kind::Movement),
            ))
        }
        Back => {
            need(s.trip.as_ref().is_some_and(|t| t.room.is_some()), "You are outside.")?;
            Ok(info(clock::cost(s, 5), 0))
        }
        Leave => {
            let depth = s.trip.as_ref().and_then(|t| t.room).map_or(0, |r| r as u32 + 1);
            Ok(info(depth * 4, 0))
        }
        Search { careful } => {
            let (i, _) = expedition::current_room(s).ok_or("You are outside.")?;
            let rs = expedition::room_state(s, i).ok_or("No room.")?;
            need(rs.search != SearchState::Full, "You have been through everything here.")?;
            if !careful && rs.search == SearchState::Partial && rs.hidden.is_empty() {
                return Err("Only papers left. Search carefully to read them.".into());
            }
            Ok(info(
                expedition::search_minutes(s, *careful),
                noise::effective(s, if *careful { 1 } else { 6 }, noise::Kind::Movement),
            ))
        }
        LightUp => {
            need(expedition::light_target(s).is_some(), "There is light enough.")?;
            need(inventory::light(s).is_some(), "You have no light.")?;
            Ok(info(1, 0))
        }
        PickLock => {
            need(expedition::next_locked(s), "Nothing is locked.")?;
            let lock = expedition::next_room(s)
                .and_then(|(_, r)| r.lock)
                .ok_or("Nothing is locked.")?;
            need(lock.pick.is_some(), "This lock cannot be picked.")?;
            need(s.count("lockpicks") > 0, "Needs lockpicks.")?;
            Ok(odds(
                expedition::pick_minutes(s),
                noise::effective(s, 1, noise::Kind::Movement),
                expedition::pick_odds(s),
            ))
        }
        ForceLock => {
            need(expedition::next_locked(s), "Nothing is locked.")?;
            need(
                expedition::force_tool(s).is_some(),
                "Needs a crowbar, a hatchet or a hammer.",
            )?;
            Ok(info(clock::cost(s, 5), noise::effective(s, 18, noise::Kind::Other)))
        }
        UseKey => {
            need(expedition::next_locked(s), "Nothing is locked.")?;
            let lock = expedition::next_room(s)
                .and_then(|(_, r)| r.lock)
                .ok_or("Nothing is locked.")?;
            let k = lock.key.ok_or("There is no key for this.")?;
            need(s.count(k) > 0, &format!("Needs the {}.", items::name(k).to_lowercase()))?;
            Ok(info(2, 0))
        }
        Take { item, qty } => {
            known_item(item)?;
            need(*qty > 0, "Nothing to take.")?;
            let (i, _) = expedition::current_room(s).ok_or("You are outside.")?;
            let have = expedition::room_state(s, i).map_or(0, |rs| inv_count(&rs.pile, item));
            need(have >= *qty, "There are not that many.")?;
            need(inventory::room_for(s, item) >= *qty, "No room. Leave something behind.")?;
            Ok(info(0, 0))
        }
        TakeAll => {
            let (i, _) = expedition::current_room(s).ok_or("You are outside.")?;
            let pile = expedition::room_state(s, i)
                .map(|rs| rs.pile.clone())
                .unwrap_or_default();
            need(!pile.is_empty(), "Nothing here to take.")?;
            need(
                pile.keys().any(|id| inventory::room_for(s, id) > 0),
                "No room. Leave something behind.",
            )?;
            Ok(info(0, 0))
        }
        Drop { item, qty } => {
            need(
                expedition::current_room(s).is_some(),
                "Leave things inside, not on the road.",
            )?;
            need(*qty > 0, "Nothing to leave.")?;
            need(inv_count(&s.pack, item) >= *qty, "You are not carrying that many.")?;
            keeps_load(s, item, *qty)?;
            Ok(info(0, 0))
        }
        Ambush => {
            let (i, _) = expedition::current_room(s).ok_or("You are outside.")?;
            let foes = expedition::room_state(s, i)
                .map(|rs| rs.foes.clone())
                .unwrap_or_default();
            let weakest = foes.iter().map(|f| f.health).min().ok_or("There is nothing here.")?;
            need(s.vitals.pts(VitalKind::Nerve) >= 10, "Your hands will not do it.")?;
            let (lo, hi) = expedition::ambush_damage(s);
            let span = (hi - lo + 1).max(1);
            let kills = (hi - weakest.max(lo) + 1).clamp(0, span);
            let p = kills * 100 / span;
            let w = inventory::melee(s).map_or(2, |(_, w)| w.noise) + 2;
            Ok(odds(2, noise::effective(s, w, noise::Kind::Other), p.clamp(0, 100)))
        }
        Trade => {
            need(
                s.trip.as_ref().is_some_and(|t| t.place == "weir" && t.room.is_none()),
                "Nobody to trade with here.",
            )?;
            need(barter::present(s), "The widow is not here today.")?;
            need(!s.is_dark(), "She has gone home for the night.")?;
            Ok(info(0, 0))
        }
        Strike => {
            let o = encounter::strike_odds(s).ok_or("Nothing in reach.")?;
            let n = inventory::melee(s).map_or(inventory::FISTS.noise, |(_, w)| w.noise);
            Ok(odds(1, noise::effective(s, n, noise::Kind::Other), o))
        }
        Shoot => {
            let (_, w) = inventory::gun(s).ok_or("No loaded gun.")?;
            let o = encounter::shoot_odds(s).ok_or("Nothing to aim at.")?;
            Ok(odds(1, noise::effective(s, w.noise, noise::Kind::Other), o))
        }
        Shove => {
            let o = encounter::shove_odds(s).ok_or("Nothing close enough.")?;
            Ok(odds(1, noise::effective(s, 2, noise::Kind::Other), o))
        }
        BackAway => {
            need(!encounter::gripped(s), "It has hold of you.")?;
            Ok(odds(1, 0, encounter::back_odds(s)))
        }
        HoldStill => Ok(odds(1, 0, encounter::hold_odds(s))),
        Flee => {
            let home_fight = s.encounter.as_ref().is_some_and(|e| e.at_home);
            need(!home_fight, "There is nowhere to run to.")?;
            need(!encounter::gripped(s), "It has hold of you.")?;
            Ok(odds(
                2,
                noise::effective(s, 4, noise::Kind::Movement),
                encounter::flee_odds(s),
            ))
        }
        Throw { item } => {
            let d = known_item(item)?;
            need(d.throw > 0, "That will not draw them.")?;
            need(s.count(item) > 0, "You have none.")?;
            Ok(odds(
                1,
                noise::effective(s, d.throw, noise::Kind::Other),
                encounter::throw_odds(s),
            ))
        }
        Choose { option } => {
            let def = events::current(s).ok_or("No choice is waiting.")?;
            let o = def.opts.get(*option as usize).ok_or("No such option.")?;
            events::option_ok(s, o)?;
            Ok(Info {
                minutes: o.minutes,
                noise: 0,
                odds: events::odds(s, &o.odds),
            })
        }
        Sell { item, qty } => {
            known_item(item)?;
            need(*qty > 0, "Nothing to sell.")?;
            need(inv_count(&s.pack, item) >= *qty, "You are not carrying that many.")?;
            need(barter::sell_price(item) > 0, "She will not give anything for that.")?;
            keeps_load(s, item, *qty)?;
            Ok(info(0, 0))
        }
        Buy { item } => {
            let offer = s
                .widow
                .stock
                .iter()
                .find(|o| &o.item == item && o.qty > 0)
                .ok_or("She has none.")?;
            need(s.widow.credit >= offer.price, "Not enough on the slate.")?;
            need(inventory::room_for(s, item) >= 1, "No room. Sell or leave something.")?;
            Ok(info(0, 0))
        }
        EndTrade => Ok(info(0, 0)),
        Sleep { .. } => Ok(info(0, 0)),
        GetUp => {
            need(!too_late(s, 0), LATE)?;
            Ok(info(0, 0))
        }
        Rise => Ok(info(0, 0)),
    }
}

fn apply(s: &mut GameState, j: &mut Journal, a: &Action, inf: Info) {
    use Action::*;
    match a {
        Use { item } => {
            if s.phase == Phase::Encounter {
                encounter::act(s, j, Move::Use(item));
            } else {
                home::use_item(s, j, item);
                clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            }
        }
        Build { station } => home::build(s, j, *station),
        Craft { recipe: id } => {
            if let Some(r) = recipes::get(id) {
                home::craft(s, j, r);
            }
        }
        RepairBarricade => home::repair_barricade(s, j),
        FeedStove => home::feed_stove(s, j),
        DrawWater => home::draw_water(s, j),
        Forage => {
            clock::pass(s, j, inf.minutes, clock::Activity::Exertion);
            if !s.ended() {
                noise::make(s, j, 2, noise::Kind::Movement);
                food::forage(s, j);
            }
        }
        Rest => home::rest(s, j),
        ListenRadio => home::listen(s, j),
        GoToBed => night::go_to_bed(s, j),
        PlanTrip { place } => expedition::plan(s, j, place),
        SetSnares => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::set_snares(s, j);
        }
        CheckSnares => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::check_snares(s, j);
        }
        SetLine => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::set_line(s, j);
        }
        CheckLine => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::check_line(s, j);
        }
        Plant { seed } => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::plant(s, j, seed);
        }
        TendGarden => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::tend(s, j);
        }
        Harvest => {
            clock::pass(s, j, inf.minutes, clock::Activity::Awake);
            food::harvest(s, j);
        }
        FeedDog => {
            clock::pass(s, j, 5, clock::Activity::Awake);
            dog::feed(s, j);
        }
        InstallPart { part } => home::install(s, j, part),
        CastOff => home::cast_off(s, j),
        WalkOntoIce => home::walk_onto_ice(s, j),
        Pack { item, qty } => {
            let n = crate::state::inv_take(&mut s.store, item, *qty);
            crate::state::inv_add(&mut s.pack, item, n);
        }
        Unpack { item, qty } => {
            let n = crate::state::inv_take(&mut s.pack, item, *qty);
            crate::state::inv_add(&mut s.store, item, n);
        }
        TakeDog { take } => {
            if *take {
                s.flags.insert("take_dog".into());
            } else {
                s.flags.remove("take_dog");
            }
        }
        Depart => expedition::depart(s, j),
        CancelTrip => expedition::cancel(s),
        Walk { hurry } => expedition::walk(s, j, *hurry),
        TurnBack => expedition::turn_back(s, j),
        Enter { quiet } => expedition::enter(s, j, *quiet),
        Back => expedition::back(s, j),
        Leave => expedition::leave(s, j),
        Search { careful } => expedition::search(s, j, *careful),
        LightUp => expedition::light_up(s, j),
        PickLock => expedition::pick_lock(s, j),
        ForceLock => expedition::force_lock(s, j),
        UseKey => expedition::use_key(s, j),
        Take { item, qty } => expedition::take(s, j, item, *qty),
        TakeAll => expedition::take_all(s, j),
        Drop { item, qty } => expedition::drop(s, j, item, *qty),
        Ambush => expedition::ambush(s, j),
        Trade => {
            barter::open(s, j);
            s.phase = Phase::Barter;
        }
        Strike => encounter::act(s, j, Move::Strike),
        Shoot => encounter::act(s, j, Move::Shoot),
        Shove => encounter::act(s, j, Move::Shove),
        BackAway => encounter::act(s, j, Move::BackAway),
        HoldStill => encounter::act(s, j, Move::HoldStill),
        Flee => encounter::act(s, j, Move::Flee),
        Throw { item } => encounter::act(s, j, Move::Throw(item)),
        Choose { option } => events::choose(s, j, *option as usize),
        Sell { item, qty } => barter::sell(s, j, item, *qty),
        Buy { item } => barter::buy(s, j, item),
        EndTrade => {
            barter::close(s, j);
            s.phase = Phase::Site;
        }
        Sleep { watch } => night::sleep(s, j, *watch),
        GetUp => s.phase = Phase::Home,
        Rise => {
            s.dawn = None;
            crate::systems::resume(s, j, crate::state::Resume::Rise);
        }
    }
}

/// Run one action. Invalid actions return an error and change nothing.
pub fn dispatch(s: &mut GameState, a: &Action) -> Result<Outcome, GameError> {
    if s.ended() {
        return Err(GameError::new("run_over", "The run is over."));
    }
    let inf = check(s, a).map_err(|reason| GameError::new("not_available", reason))?;
    let mut out = Outcome::default();
    apply(s, &mut out, a, inf);
    crate::systems::check_death(s, &mut out);
    crate::meta::award_marks(s, &mut out);
    Ok(out)
}

/// Give the run up. It ends as a death ("Gave up"), with its ledger, so it is
/// scored and can be entered in the profile once like any other run.
pub fn forfeit(s: &mut GameState) -> Result<Outcome, GameError> {
    if s.ended() {
        return Err(GameError::new("run_over", "The run is over."));
    }
    let mut out = Outcome::default();
    crate::systems::forfeit(s, &mut out);
    Ok(out)
}

/// Every action worth showing in the current phase (allowed or not).
pub fn candidates(s: &GameState) -> Vec<Action> {
    use Action::*;
    let mut v = Vec::new();
    let usable = |inv: &crate::state::Inv, v: &mut Vec<Action>, medicine_only: bool| {
        for id in inv.keys() {
            let Some(d) = items::get(id) else { continue };
            let ok = match d.kind {
                ItemKind::Medicine(_) => true,
                ItemKind::Food { .. } | ItemKind::Comfort { .. } | ItemKind::Clothing { .. } => !medicine_only,
                _ => false,
            };
            if ok {
                v.push(Use { item: id.clone() });
            }
        }
    };
    match s.phase {
        Phase::Home => {
            if s.ferry.len() == 4 && !s.frozen {
                v.push(CastOff);
            }
            if s.frozen {
                v.push(WalkOntoIce);
            }
            v.push(GoToBed);
            v.push(FeedStove);
            v.push(DrawWater);
            v.push(Forage);
            v.push(Rest);
            if s.level(Station::Radio) >= 1 {
                v.push(ListenRadio);
            }
            if s.level(Station::Barricade) > 0 || s.barricade < stations::barricade_max(0) {
                v.push(RepairBarricade);
            }
            for p in places::PLACES {
                if s.places.get(p.id).is_some_and(|ps| ps.known) {
                    v.push(PlanTrip {
                        place: p.id.to_string(),
                    });
                }
            }
            for part in items::FERRY_PARTS {
                if inv_count(&s.store, part) > 0 && !s.ferry.contains(part) {
                    v.push(InstallPart { part: part.to_string() });
                }
            }
            if inv_count(&s.store, "snare") > 0 {
                v.push(SetSnares);
            }
            if s.traps.snares > 0 {
                v.push(CheckSnares);
            }
            if !s.traps.line && inv_count(&s.store, "fishing_line") > 0 {
                v.push(SetLine);
            }
            if s.traps.line {
                v.push(CheckLine);
            }
            if s.level(Station::Garden) >= 1 {
                for id in s.store.keys() {
                    if matches!(items::get(id).map(|d| d.kind), Some(ItemKind::Seed { .. })) {
                        v.push(Plant { seed: id.clone() });
                    }
                }
                if !s.garden.is_empty() {
                    v.push(TendGarden);
                    v.push(Harvest);
                }
            }
            if s.dog.state == DogState::Companion {
                v.push(FeedDog);
            }
            usable(&s.store, &mut v, false);
            for st in Station::ALL {
                if s.level(*st) < 3 {
                    v.push(Build { station: *st });
                }
            }
            for r in recipes::RECIPES {
                v.push(Craft {
                    recipe: r.id.to_string(),
                });
            }
        }
        Phase::Packing => {
            v.push(Depart);
            v.push(CancelTrip);
            if s.dog.state == DogState::Companion {
                v.push(TakeDog {
                    take: !s.has_flag("take_dog"),
                });
            }
            for id in s.pack.keys() {
                v.push(Unpack {
                    item: id.clone(),
                    qty: 1,
                });
            }
            for id in s.store.keys() {
                v.push(Pack {
                    item: id.clone(),
                    qty: 1,
                });
            }
        }
        Phase::Travelling => {
            v.push(Walk { hurry: false });
            v.push(Walk { hurry: true });
            if !s.homeward {
                v.push(TurnBack);
            }
            usable(&s.pack, &mut v, false);
        }
        Phase::Site => {
            if expedition::next_room(s).is_some() {
                if expedition::next_locked(s) {
                    if let Some(l) = expedition::next_room(s).and_then(|(_, r)| r.lock) {
                        if l.key.is_some() {
                            v.push(UseKey);
                        }
                        if l.pick.is_some() {
                            v.push(PickLock);
                        }
                    }
                    v.push(ForceLock);
                }
                v.push(Enter { quiet: true });
                v.push(Enter { quiet: false });
            }
            if expedition::light_target(s).is_some() {
                v.push(LightUp);
            }
            if let Some((i, _)) = expedition::current_room(s) {
                let rs = expedition::room_state(s, i);
                if rs.is_some_and(|rs| rs.search != SearchState::Full) {
                    v.push(Search { careful: true });
                    v.push(Search { careful: false });
                }
                if rs.is_some_and(|rs| !rs.foes.is_empty()) {
                    v.push(Ambush);
                }
                if let Some(rs) = rs {
                    if !rs.pile.is_empty() {
                        v.push(TakeAll);
                    }
                    for id in rs.pile.keys() {
                        v.push(Take {
                            item: id.clone(),
                            qty: 1,
                        });
                    }
                }
                v.push(Back);
            }
            if s.trip.as_ref().is_some_and(|t| t.place == "weir" && t.room.is_none()) && barter::present(s) {
                v.push(Trade);
            }
            v.push(Leave);
            usable(&s.pack, &mut v, false);
            if expedition::current_room(s).is_some() {
                for id in s.pack.keys() {
                    v.push(Drop {
                        item: id.clone(),
                        qty: 1,
                    });
                }
            }
        }
        Phase::Encounter => {
            v.push(Strike);
            if s.inv()
                .keys()
                .any(|id| items::weapon(id).is_some_and(|w| w.ammo.is_some()))
            {
                v.push(Shoot);
            }
            if encounter::nearest(s) == Some(crate::content::Band::Close) {
                v.push(Shove);
            }
            v.push(BackAway);
            v.push(HoldStill);
            if !s.encounter.as_ref().is_some_and(|e| e.at_home) {
                v.push(Flee);
            }
            for id in s.inv().keys() {
                if items::get(id).is_some_and(|d| d.throw > 0) {
                    v.push(Throw { item: id.clone() });
                }
            }
            usable(s.inv(), &mut v, true);
        }
        Phase::Event => {
            if let Some(def) = events::current(s) {
                for i in 0..def.opts.len() {
                    v.push(Choose { option: i as u32 });
                }
            }
        }
        Phase::Barter => {
            for o in &s.widow.stock {
                v.push(Buy { item: o.item.clone() });
            }
            for id in s.pack.keys() {
                if barter::sell_price(id) > 0 {
                    v.push(Sell {
                        item: id.clone(),
                        qty: 1,
                    });
                }
            }
            v.push(EndTrade);
        }
        Phase::Night => {
            v.push(Sleep { watch: false });
            v.push(Sleep { watch: true });
            v.push(GetUp);
            usable(&s.store, &mut v, false);
        }
        Phase::Dawn => v.push(Rise),
        Phase::Ended => {}
    }
    v
}

/// Upper bound for quantity actions (pack, take, drop, sell); 1 otherwise.
pub fn max_qty(s: &GameState, a: &Action) -> u32 {
    use Action::*;
    match a {
        Pack { item, .. } => {
            let w = items::get(item).map_or(1, |d| d.weight);
            let fit = inventory::capacity(s)
                .saturating_sub(inventory::carried(s))
                .checked_div(w)
                .unwrap_or(u32::MAX);
            inv_count(&s.store, item).min(fit)
        }
        Unpack { item, .. } | Sell { item, .. } => inv_count(&s.pack, item),
        Drop { item, .. } => inv_count(&s.pack, item),
        Take { item, .. } => {
            let have = expedition::current_room(s)
                .and_then(|(i, _)| expedition::room_state(s, i))
                .map_or(0, |rs| inv_count(&rs.pile, item));
            have.min(inventory::room_for(s, item))
        }
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameConfig, new_game};

    fn game() -> GameState {
        new_game(&GameConfig {
            seed: 2,
            difficulty: crate::content::Difficulty::Still,
            background: "clerk".into(),
            keepsake: None,
            daily: None,
            held_marks: vec![],
        })
        .unwrap()
    }

    #[test]
    fn invalid_actions_are_errors_and_change_nothing() {
        let mut s = game();
        let before = s.clone();
        for a in [
            Action::Strike,
            Action::Rise,
            Action::Use {
                item: "no_such_thing".into(),
            },
            Action::Use {
                item: "antibiotics".into(),
            },
            Action::Craft { recipe: "nope".into() },
            Action::PlanTrip {
                place: "atlantis".into(),
            },
            Action::PlanTrip { place: "wharf".into() },
            Action::Choose { option: 9 },
            Action::Pack {
                item: "hammer".into(),
                qty: 0,
            },
            Action::Build {
                station: Station::Radio,
            },
        ] {
            let e = dispatch(&mut s, &a).unwrap_err();
            assert!(!e.message.is_empty());
            assert_eq!(s, before, "{a:?} changed the state");
        }
    }

    #[test]
    fn actions_round_trip_as_json() {
        let a = Action::Pack {
            item: "candle".into(),
            qty: 2,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, r#"{"type":"pack","item":"candle","qty":2}"#);
        assert_eq!(serde_json::from_str::<Action>(&json).unwrap(), a);
        #[cfg(not(feature = "ts"))]
        assert!(serde_json::from_str::<Action>(r#"{"type":"pack","item":"x","qty":1,"extra":1}"#).is_err());
        assert!(serde_json::from_str::<Action>(r#"{"type":"teleport"}"#).is_err());
        assert_eq!(
            serde_json::from_str::<Action>(r#"{"type":"go_to_bed"}"#).unwrap(),
            Action::GoToBed
        );
    }

    #[test]
    fn a_short_trip() {
        let mut s = game();
        dispatch(
            &mut s,
            &Action::PlanTrip {
                place: "cottages".into(),
            },
        )
        .unwrap();
        assert_eq!(s.phase, Phase::Packing);
        dispatch(
            &mut s,
            &Action::Pack {
                item: "candle".into(),
                qty: 1,
            },
        )
        .unwrap();
        dispatch(&mut s, &Action::Depart).unwrap();
        assert_eq!(s.phase, Phase::Travelling);
        let out = dispatch(&mut s, &Action::Walk { hurry: false }).unwrap();
        assert!(!out.journal.is_empty());
        if s.phase == Phase::Event {
            dispatch(&mut s, &Action::Choose { option: 0 }).ok();
        }
        assert!(matches!(
            s.phase,
            Phase::Site | Phase::Encounter | Phase::Event | Phase::Ended
        ));
    }
}
