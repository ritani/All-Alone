//! Backgrounds (the lodger's former trade) and starting keepsakes.

use super::Skill;

#[derive(Clone, Copy, Debug, Default)]
pub struct Mods {
    /// Added to carry capacity, tenths of a kilogram.
    pub carry: i32,
    /// Percent added to satiety decay.
    pub hunger: i32,
    /// Percent added to rest recovered by sleep.
    pub sleep: i32,
    /// Added to melee damage.
    pub melee: i32,
    /// Percent added to prices at the weir.
    pub barter: i32,
    /// Percent added to noise from building and crafting.
    pub work_noise: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct BackgroundDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub drawback: &'static str,
    pub skill: Skill,
    pub skill_level: u8,
    pub kit: &'static [(&'static str, u32)],
    pub mods: Mods,
    pub open_at_start: bool,
    /// Earning this mark unlocks the background for free.
    pub unlock_mark: Option<&'static str>,
    /// Or it can be bought with remnants.
    pub cost: u32,
    pub icon: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct KeepsakeDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub items: &'static [(&'static str, u32)],
    pub reveals: &'static [&'static str],
    pub cost: u32,
    pub icon: &'static str,
}

const M: Mods = Mods {
    carry: 0,
    hunger: 0,
    sleep: 0,
    melee: 0,
    barter: 0,
    work_noise: 0,
};

pub static BACKGROUNDS: &[BackgroundDef] = &[
    BackgroundDef {
        id: "clerk",
        name: "Clerk",
        desc: "You kept the books at the mill office, and the keys to the cupboards.",
        drawback: "Soft hands. Your blows do less harm.",
        skill: Skill::Lockwork,
        skill_level: 1,
        kit: &[("biscuits", 3), ("tinned_meat", 1), ("candle", 2)],
        mods: Mods { melee: -2, ..M },
        open_at_start: true,
        unlock_mark: None,
        cost: 0,
        icon: "bg_clerk",
    },
    BackgroundDef {
        id: "nurse",
        name: "Nurse",
        desc: "Twelve years on the wards at the county hospital.",
        drawback: "Light sleeper. Sleep restores less.",
        skill: Skill::FieldMedicine,
        skill_level: 2,
        kit: &[("bandage", 3), ("antiseptic", 1), ("painkillers", 1)],
        mods: Mods { sleep: -12, ..M },
        open_at_start: false,
        unlock_mark: Some("mended"),
        cost: 60,
        icon: "bg_nurse",
    },
    BackgroundDef {
        id: "poacher",
        name: "Poacher",
        desc: "Rabbits from the estate, trout from the private water, never caught.",
        drawback: "Known to the widow. She asks more of you.",
        skill: Skill::Foraging,
        skill_level: 2,
        kit: &[("snare", 2), ("fishing_line", 1), ("smoked_fish", 2)],
        mods: Mods { barter: 25, ..M },
        open_at_start: false,
        unlock_mark: Some("trapper"),
        cost: 60,
        icon: "bg_poacher",
    },
    BackgroundDef {
        id: "machinist",
        name: "Machinist",
        desc: "You kept the mill's looms running until the new engine came.",
        drawback: "Heavy-handed. Building and crafting make more noise.",
        skill: Skill::Mending,
        skill_level: 2,
        kit: &[("spanner", 1), ("scrap", 3), ("parts", 2)],
        mods: Mods { work_noise: 60, ..M },
        open_at_start: false,
        unlock_mark: Some("all_stations_1"),
        cost: 80,
        icon: "bg_machinist",
    },
    BackgroundDef {
        id: "warden",
        name: "Night warden",
        desc: "You walked the mill yard at night with a lamp, for years.",
        drawback: "Big appetite. Hunger comes on faster.",
        skill: Skill::QuietStep,
        skill_level: 2,
        kit: &[("oil_lamp", 1), ("lamp_oil", 1)],
        mods: Mods { hunger: 15, ..M },
        open_at_start: false,
        unlock_mark: Some("week"),
        cost: 100,
        icon: "bg_warden",
    },
];

pub static KEEPSAKES: &[KeepsakeDef] = &[
    KeepsakeDef {
        id: "flask",
        name: "Hip flask",
        desc: "The lock-keeper's, left on the dresser. Nearly full.",
        items: &[("spirits", 1)],
        reveals: &[],
        cost: 20,
        icon: "bottle",
    },
    KeepsakeDef {
        id: "candles",
        name: "Box of candles",
        desc: "Four tallow candles in a cigar box.",
        items: &[("candle", 4)],
        reveals: &[],
        cost: 15,
        icon: "light",
    },
    KeepsakeDef {
        id: "seed_tin",
        name: "Tin of seeds",
        desc: "Kale and turnip, labelled in your own hand.",
        items: &[("seed_kale", 2), ("seed_turnip", 1)],
        reveals: &[],
        cost: 30,
        icon: "herb",
    },
    KeepsakeDef {
        id: "valley_map",
        name: "Walker's map",
        desc: "The valley in green and brown. The boatyard and the surgery are marked.",
        items: &[],
        reveals: &["boatyard", "surgery"],
        cost: 40,
        icon: "map",
    },
    KeepsakeDef {
        id: "rattles",
        name: "Two tin rattles",
        desc: "Made on the first night, before you knew what they were for.",
        items: &[("tin_rattle", 2)],
        reveals: &[],
        cost: 15,
        icon: "noise",
    },
    KeepsakeDef {
        id: "tackle",
        name: "Fishing tin",
        desc: "A night line and a tin of worms.",
        items: &[("fishing_line", 1), ("bait", 2)],
        reveals: &[],
        cost: 25,
        icon: "trap",
    },
    KeepsakeDef {
        id: "coat",
        name: "Your own coat",
        desc: "Thick wool. You came to the lock house in it.",
        items: &[("wool_coat", 1)],
        reveals: &[],
        cost: 35,
        icon: "coat",
    },
];

pub fn get(id: &str) -> Option<&'static BackgroundDef> {
    BACKGROUNDS.iter().find(|b| b.id == id)
}

pub fn keepsake(id: &str) -> Option<&'static KeepsakeDef> {
    KEEPSAKES.iter().find(|k| k.id == id)
}

/// The items everyone starts with, before background and keepsake.
pub static START_KIT: &[(&str, u32)] = &[
    ("hammer", 1),
    ("kitchen_knife", 1),
    ("candle", 1),
    ("tinned_beans", 2),
    ("biscuits", 4),
    ("water_clean", 3),
    ("cloth", 2),
    ("rag", 2),
    ("plank", 3),
    ("nails", 8),
    ("firewood", 3),
    ("glass_bottle", 1),
    ("fishing_line", 1),
];
