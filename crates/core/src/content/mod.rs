//! Static game content: everything the rules read but never change.
//!
//! Ids are short snake_case strings. `tests::content_integrity` checks that
//! every cross-reference (recipe inputs, loot, events, codex reveals ...)
//! resolves, so a typo here fails `cargo test` rather than a player's run.

pub mod backgrounds;
pub mod codex;
pub mod creatures;
pub mod events;
pub mod items;
pub mod marks;
pub mod places;
pub mod prose;
pub mod recipes;
pub mod stations;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

macro_rules! simple_enum {
    ($(#[$m:meta])* $name:ident { $($variant:ident = $id:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[cfg_attr(feature = "ts", derive(TS))]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];
            pub fn id(self) -> &'static str {
                match self { $($name::$variant => $id),+ }
            }
        }
    };
}

simple_enum!(
    /// The six vitals, each 0..=100.
    VitalKind {
        Health = "health",
        Satiety = "satiety",
        Hydration = "hydration",
        Rest = "rest",
        Nerve = "nerve",
        Warmth = "warmth",
    }
);

simple_enum!(
    /// Skills improve by use, levels 0..=5.
    Skill {
        QuietStep = "quiet_step",
        Lockwork = "lockwork",
        Mending = "mending",
        Marksmanship = "marksmanship",
        Foraging = "foraging",
        FieldMedicine = "field_medicine",
    }
);

simple_enum!(
    /// Lock house stations, levels 0..=3.
    Station {
        Stove = "stove",
        Cot = "cot",
        Workbench = "workbench",
        Barrel = "barrel",
        Garden = "garden",
        Barricade = "barricade",
        Radio = "radio",
        Medicine = "medicine",
        Store = "store",
    }
);

simple_enum!(
    /// Clothing slots. Worn clothing has no carried weight.
    Slot {
        Coat = "coat",
        Shell = "shell",
        Head = "head",
        Neck = "neck",
        Hands = "hands",
        Legs = "legs",
    }
);

simple_enum!(
    Weather {
        Clear = "clear",
        Overcast = "overcast",
        Rain = "rain",
        Fog = "fog",
        Frost = "frost",
        Snow = "snow",
    }
);

simple_enum!(
    /// Kinds of the Stilled. Each behaves differently in an encounter.
    Creature {
        Stander = "stander",
        Listener = "listener",
        Runner = "runner",
        Drowned = "drowned",
        Hollow = "hollow",
        Small = "small",
    }
);

simple_enum!(
    /// Distance bands in an encounter.
    Band {
        Far = "far",
        Near = "near",
        Close = "close",
    }
);

simple_enum!(
    Difficulty {
        Quiet = "quiet",
        Still = "still",
        DeadOfWinter = "dead_of_winter",
    }
);

impl VitalKind {
    pub fn name(self) -> &'static str {
        match self {
            VitalKind::Health => "Health",
            VitalKind::Satiety => "Satiety",
            VitalKind::Hydration => "Hydration",
            VitalKind::Rest => "Rest",
            VitalKind::Nerve => "Nerve",
            VitalKind::Warmth => "Warmth",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            VitalKind::Health => "health",
            VitalKind::Satiety => "food",
            VitalKind::Hydration => "water",
            VitalKind::Rest => "rest",
            VitalKind::Nerve => "nerve",
            VitalKind::Warmth => "warmth",
        }
    }
}

impl Skill {
    pub fn name(self) -> &'static str {
        match self {
            Skill::QuietStep => "Quiet step",
            Skill::Lockwork => "Lockwork",
            Skill::Mending => "Mending",
            Skill::Marksmanship => "Marksmanship",
            Skill::Foraging => "Foraging",
            Skill::FieldMedicine => "Field medicine",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Skill::QuietStep => "skill_quiet",
            Skill::Lockwork => "lock",
            Skill::Mending => "workbench",
            Skill::Marksmanship => "skill_aim",
            Skill::Foraging => "skill_forage",
            Skill::FieldMedicine => "medicine",
        }
    }
    pub fn effect(self) -> &'static str {
        match self {
            Skill::QuietStep => "Less noise moving and searching. Better odds holding still.",
            Skill::Lockwork => "Faster, surer lock picking.",
            Skill::Mending => "Faster building, crafting and repair.",
            Skill::Marksmanship => "Better odds with a firearm.",
            Skill::Foraging => "More from searches, snares, lines and the garden.",
            Skill::FieldMedicine => "Bandages hold. Infections turn back sooner.",
        }
    }
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Weather::Clear => "Clear",
            Weather::Overcast => "Overcast",
            Weather::Rain => "Rain",
            Weather::Fog => "Fog",
            Weather::Frost => "Frost",
            Weather::Snow => "Snow",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Weather::Clear => "weather_clear",
            Weather::Overcast => "weather_overcast",
            Weather::Rain => "weather_rain",
            Weather::Fog => "weather_fog",
            Weather::Frost => "weather_frost",
            Weather::Snow => "weather_snow",
        }
    }
    pub fn effect(self) -> &'static str {
        match self {
            Weather::Clear => "Cold nights.",
            Weather::Overcast => "Nothing to help or hinder.",
            Weather::Rain => "Fills the barrel. Masks noise. Wet and cold outside.",
            Weather::Fog => "Hides you. Hides them.",
            Weather::Frost => "Bitter outside. Hard on the garden. They are slower.",
            Weather::Snow => "Slow going. Bitter outside.",
        }
    }
}

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Quiet => "Quiet",
            Difficulty::Still => "Still",
            Difficulty::DeadOfWinter => "Dead of Winter",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Difficulty::Quiet => "Forgiving. Slower hunger, fewer of them, a later freeze.",
            Difficulty::Still => "The game as meant.",
            Difficulty::DeadOfWinter => "Harsh. Early freeze, thin pickings. Scores more.",
        }
    }
    /// Percent multiplier on vital decay.
    pub fn decay_pct(self) -> i32 {
        match self {
            Difficulty::Quiet => 80,
            Difficulty::Still => 100,
            Difficulty::DeadOfWinter => 115,
        }
    }
    /// Percent multiplier on encounter and visit chances.
    pub fn threat_pct(self) -> i32 {
        match self {
            Difficulty::Quiet => 70,
            Difficulty::Still => 100,
            Difficulty::DeadOfWinter => 125,
        }
    }
    /// Percent multiplier on loot chances.
    pub fn loot_pct(self) -> i32 {
        match self {
            Difficulty::Quiet => 115,
            Difficulty::Still => 100,
            Difficulty::DeadOfWinter => 85,
        }
    }
    /// Base day the river freezes (the seed shifts it by up to two days).
    pub fn freeze_day(self) -> u32 {
        match self {
            Difficulty::Quiet => 52,
            Difficulty::Still => 45,
            Difficulty::DeadOfWinter => 39,
        }
    }
    /// Score multiplier, percent.
    pub fn score_pct(self) -> u32 {
        match self {
            Difficulty::Quiet => 75,
            Difficulty::Still => 100,
            Difficulty::DeadOfWinter => 160,
        }
    }
}

impl Station {
    pub fn name(self) -> &'static str {
        stations::def(self).name
    }
}

impl Creature {
    pub fn name(self) -> &'static str {
        creatures::def(self).name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn unique<'a>(ids: impl Iterator<Item = &'a str>) {
        let mut seen = BTreeSet::new();
        for id in ids {
            assert!(seen.insert(id), "duplicate id {id}");
        }
    }

    #[test]
    fn content_integrity() {
        unique(items::ITEMS.iter().map(|i| i.id));
        unique(recipes::RECIPES.iter().map(|r| r.id));
        unique(places::PLACES.iter().map(|p| p.id));
        unique(events::EVENTS.iter().map(|e| e.id));
        unique(codex::CODEX.iter().map(|c| c.id));
        unique(marks::MARKS.iter().map(|m| m.id));
        unique(backgrounds::BACKGROUNDS.iter().map(|b| b.id));
        unique(backgrounds::KEEPSAKES.iter().map(|k| k.id));
        for p in places::PLACES {
            unique(p.rooms.iter().map(|r| r.id));
        }

        let item = |id: &str| assert!(items::get(id).is_some(), "unknown item {id}");
        for i in items::ITEMS {
            assert!(!i.name.is_empty() && !i.desc.is_empty());
            assert!(!i.desc.contains('!'), "exclamation in {}", i.id);
            if let items::ItemKind::Light { fuel: Some(f), .. } = i.kind {
                item(f);
            }
            if let items::ItemKind::Weapon(w) = i.kind
                && let Some(a) = w.ammo
            {
                item(a);
            }
            if let items::ItemKind::Seed { crop, .. } = i.kind {
                item(crop);
            }
        }
        for r in recipes::RECIPES {
            for (id, n) in r.inputs.iter().chain(r.outputs) {
                item(id);
                assert!(*n > 0);
            }
            for t in r.tools {
                assert!(
                    items::ITEMS.iter().any(|i| i.tags.contains(t)),
                    "no item has tool tag {t}"
                );
            }
        }
        for s in Station::ALL {
            let d = stations::def(*s);
            for lvl in d.levels {
                for (id, _) in lvl.cost {
                    item(id);
                }
            }
        }
        for p in places::PLACES {
            assert!(!p.rooms.is_empty());
            for r in p.rooms {
                for l in r.loot {
                    item(l.item);
                    assert!(l.min <= l.max && l.max > 0);
                }
                for (id, _) in r.fixed {
                    item(id);
                }
                for c in r.codex {
                    assert!(codex::get(c).is_some(), "unknown codex {c}");
                }
                if let Some(places::Lock { key: Some(k), .. }) = r.lock {
                    item(k);
                }
            }
        }
        for c in codex::CODEX {
            if let Some(p) = c.reveals {
                assert!(places::get(p).is_some(), "codex {} reveals unknown {p}", c.id);
            }
            assert!(!c.text.contains('!'), "exclamation in {}", c.id);
        }
        events::check_integrity();
        marks::check_integrity();
        for b in backgrounds::BACKGROUNDS {
            for (id, _) in b.kit {
                item(id);
            }
        }
        for k in backgrounds::KEEPSAKES {
            for (id, _) in k.items {
                item(id);
            }
        }
        // Every ferry part can be found somewhere, or made from something that can.
        let fixed = |id: &str| {
            places::PLACES
                .iter()
                .flat_map(|p| p.rooms.iter())
                .any(|r| r.fixed.iter().any(|(f, _)| *f == id))
        };
        for part in items::FERRY_PARTS {
            let made = recipes::RECIPES
                .iter()
                .any(|r| r.outputs.iter().any(|(o, _)| *o == part) && r.inputs.iter().any(|(i, _)| fixed(i)));
            assert!(fixed(part) || made, "{part} is nowhere");
        }
        assert!(fixed("spanner"), "the spanner must always be findable");
        // Every place other than the starting ones can be revealed somehow.
        for p in places::PLACES.iter().filter(|p| !p.known_at_start) {
            let by_codex = codex::CODEX.iter().any(|c| c.reveals == Some(p.id));
            assert!(by_codex, "{} cannot be discovered", p.id);
        }
    }
}
