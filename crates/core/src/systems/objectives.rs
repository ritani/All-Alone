//! Objectives: a short list of what matters next, generated from state, so a
//! new player is never lost and a veteran always has a next target.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::content::items::{self, FERRY_PARTS, ItemKind};
use crate::content::{Slot, Station, VitalKind, places, stations};
use crate::state::{DogState, GameState, inv_count};
use crate::systems::{barter, food, inventory};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct Objective {
    pub id: String,
    pub text: String,
    /// Shown in the danger accent.
    pub urgent: bool,
    pub icon: String,
}

fn o(id: &str, text: impl Into<String>, urgent: bool, icon: &str) -> Objective {
    Objective {
        id: id.to_string(),
        text: text.into(),
        urgent,
        icon: icon.to_string(),
    }
}

fn where_part(s: &GameState, part: &str) -> String {
    let (source, work) = match part {
        "spark_coil" => ("coil_cracked", "Rewind the cracked coil at the workbench."),
        "drive_belt" => ("belting", "Cut the belting down at the workbench."),
        _ => (part, ""),
    };
    if source != part && inv_count(&s.store, source) + inv_count(&s.pack, source) > 0 {
        return work.to_string();
    }
    let part = source;
    let at = places::PLACES
        .iter()
        .find(|p| p.rooms.iter().any(|r| r.fixed.iter().any(|(id, _)| *id == part)))
        .map(|p| p.id);
    let name = items::name(part).to_lowercase();
    match at {
        Some(pid) if s.places.get(pid).is_some_and(|p| p.known) => {
            format!("Find the {name}: try {}.", places::get(pid).map_or("", |p| p.name))
        }
        _ => format!("Find the {name}. Notes and the radio may say where."),
    }
}

pub const MAX: usize = 6;

pub fn list(s: &GameState) -> Vec<Objective> {
    let mut v = Vec::new();
    if s.ended() {
        return v;
    }
    let home = s.at_home();
    let p = |k| s.vitals.pts(k);

    if s.bleeding > 0 {
        v.push(o("bleeding", "Stop the bleeding.", true, "medicine"));
    }
    if s.infection > 0 {
        v.push(o(
            "infection",
            "The wound is turning. Clean it, or find antibiotics.",
            true,
            "medicine",
        ));
    }
    if p(VitalKind::Hydration) < 30 {
        v.push(o("drink", "Drink something.", true, "water"));
    }
    if p(VitalKind::Satiety) < 25 {
        v.push(o("eat", "Eat.", true, "food"));
    }
    if p(VitalKind::Warmth) < 30 {
        let t = if home {
            "Get warm. Feed the stove."
        } else {
            "Get indoors. Get warm."
        };
        v.push(o("warm", t, true, "warmth"));
    }
    if !home && s.is_dark() {
        v.push(o("dark", "Get home. They are awake.", true, "stilled"));
    }
    if p(VitalKind::Rest) < 20 {
        v.push(o("sleep", "Sleep.", true, "rest"));
    }
    if home && s.time_of_day() >= 21 * 60 {
        v.push(o("bed", "Bar the door and go to bed.", false, "cot"));
    }
    let max = stations::barricade_max(s.level(Station::Barricade));
    if s.barricade * 100 < max * 40 && (s.day() >= 2 || s.level(Station::Barricade) == 0) {
        v.push(o(
            "barricade",
            "The barricade will not last another night.",
            true,
            "barricade",
        ));
    }

    // The ferry: what to do next.
    let missing_installed: Vec<&str> = FERRY_PARTS
        .iter()
        .copied()
        .filter(|pt| !s.ferry.contains(*pt))
        .collect();
    if !s.frozen {
        if missing_installed.is_empty() {
            let t = if crate::systems::home::can_cast_off(s).is_ok() {
                "The Kittiwake is ready. Cast off.".to_string()
            } else {
                format!(
                    "Put by {} meals and {} clean water for the voyage.",
                    crate::systems::home::VOYAGE_MEALS,
                    crate::systems::home::VOYAGE_WATER
                )
            };
            v.push(o("cast_off", t, false, "ferry"));
        } else if let Some(pt) = missing_installed.iter().find(|pt| inv_count(&s.store, pt) > 0) {
            v.push(o(
                "fit_part",
                format!("Fit the {} to the ferry.", items::name(pt).to_lowercase()),
                false,
                "ferry",
            ));
        } else if let Some(pt) = missing_installed.iter().find(|pt| inv_count(&s.pack, pt) == 0) {
            v.push(o("find_part", where_part(s, pt), false, "ferry"));
        }
    }
    if s.frozen {
        v.push(o(
            "frozen",
            "The river is frozen. Survive, or walk out onto the ice.",
            false,
            "sigil_ice",
        ));
    } else if s.freeze_known {
        let left = s.freeze_day.saturating_sub(s.day());
        v.push(o(
            "freeze",
            format!("The river closes in about {left} days."),
            left <= 5,
            "sigil_ice",
        ));
    }

    if inventory::water_capacity(s) <= 4 {
        v.push(o("water_store", "Find something to hold water.", false, "water"));
    }
    if s.level(Station::Stove) == 0 && s.day() >= 2 {
        v.push(o(
            "stove",
            "Get the stove working before the cold comes.",
            false,
            "stove",
        ));
    }
    let any_dark = places::PLACES
        .iter()
        .any(|pl| s.places.get(pl.id).is_some_and(|ps| ps.known) && pl.rooms.iter().any(|r| r.dark));
    let has_light = s
        .store
        .keys()
        .chain(s.pack.keys())
        .any(|id| matches!(items::get(id).map(|d| d.kind), Some(ItemKind::Light { .. })));
    if any_dark && !has_light {
        v.push(o("light", "Find a light for the dark rooms.", false, "light"));
    }
    if s.day() >= 12 && !s.worn.contains_key(&Slot::Coat) {
        v.push(o("coat", "Find a warm coat. The nights are turning.", false, "coat"));
    }
    if s.dog.state == DogState::Companion && s.dog.fed <= 0 {
        v.push(o("dog", "The dog has not eaten.", true, "sigil_dog"));
    }
    if home && barter::present(s) {
        v.push(o("widow", "The widow is at the weir today.", false, "place_weir"));
    }
    if home && (s.traps.snared > 0 || s.traps.hooked > 0) {
        v.push(o(
            "traps",
            "Something is waiting on the trap line or the night line.",
            false,
            "trap",
        ));
    }
    if home && food::ready(s) > 0 {
        v.push(o("harvest", "The garden is ready to lift.", false, "garden"));
    }
    v.truncate(MAX);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameConfig, new_game};

    #[test]
    fn new_player_has_direction() {
        let s = new_game(&GameConfig {
            seed: 1,
            difficulty: crate::content::Difficulty::Still,
            background: "clerk".into(),
            keepsake: None,
            daily: None,
            held_marks: vec![],
        })
        .unwrap();
        let l = list(&s);
        assert!(!l.is_empty());
        assert!(l.iter().any(|x| x.id == "water_store"), "{l:?}");
        assert!(l.iter().any(|x| x.id == "find_part"), "{l:?}");
    }
}
