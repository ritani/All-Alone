//! Crafting recipes. Made at the lock house from what is in the store.

use super::Station;

#[derive(Clone, Copy, Debug)]
pub struct RecipeDef {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub inputs: &'static [(&'static str, u32)],
    pub outputs: &'static [(&'static str, u32)],
    /// Station and minimum level, if any.
    pub station: Option<(Station, u8)>,
    /// Needs the stove lit.
    pub fire: bool,
    /// Tool tags that must be in the store (any item with the tag).
    pub tools: &'static [&'static str],
    pub minutes: u32,
    pub noise: u32,
}

const R: RecipeDef = RecipeDef {
    id: "",
    name: "",
    icon: "workbench",
    inputs: &[],
    outputs: &[],
    station: None,
    fire: false,
    tools: &[],
    minutes: 30,
    noise: 0,
};

pub static RECIPES: &[RecipeDef] = &[
    // By hand
    RecipeDef {
        id: "tear_rags",
        name: "Tear cloth into strips",
        icon: "bandage",
        inputs: &[("cloth", 1)],
        outputs: &[("rag", 2)],
        minutes: 10,
        ..R
    },
    RecipeDef {
        id: "torch",
        name: "Make a pitch torch",
        icon: "light",
        inputs: &[("cloth", 1), ("firewood", 1)],
        outputs: &[("torch", 1)],
        minutes: 15,
        ..R
    },
    RecipeDef {
        id: "snare",
        name: "Twist a wire snare",
        icon: "trap",
        inputs: &[("wire", 1)],
        outputs: &[("snare", 1)],
        minutes: 20,
        ..R
    },
    RecipeDef {
        id: "split_wood",
        name: "Cut planks for the stove",
        icon: "plank",
        inputs: &[("plank", 1)],
        outputs: &[("firewood", 2)],
        tools: &["chop"],
        minutes: 20,
        noise: 3,
        ..R
    },
    RecipeDef {
        id: "salvage_nails",
        name: "Draw nails from old boards",
        icon: "metal",
        inputs: &[("plank", 1)],
        outputs: &[("nails", 4)],
        tools: &["build"],
        minutes: 30,
        noise: 2,
        ..R
    },
    RecipeDef {
        id: "tin_rattle",
        name: "Make a tin rattle",
        icon: "noise",
        inputs: &[("tin_can", 1), ("nails", 2)],
        outputs: &[("tin_rattle", 1)],
        minutes: 10,
        ..R
    },
    RecipeDef {
        id: "scarf",
        name: "Knot a scarf",
        icon: "coat",
        inputs: &[("cloth", 3)],
        outputs: &[("scarf", 1)],
        minutes: 60,
        ..R
    },
    // Workbench
    RecipeDef {
        id: "night_line",
        name: "Make a night line",
        icon: "trap",
        inputs: &[("rope", 1), ("scrap", 1)],
        outputs: &[("fishing_line", 1)],
        station: Some((Station::Workbench, 1)),
        minutes: 40,
        ..R
    },
    RecipeDef {
        id: "lockpicks",
        name: "File a set of lockpicks",
        icon: "lock",
        inputs: &[("wire", 2), ("scrap", 1)],
        outputs: &[("lockpicks", 1)],
        station: Some((Station::Workbench, 1)),
        minutes: 45,
        ..R
    },
    RecipeDef {
        id: "nail_club",
        name: "Make a nailed club",
        icon: "club",
        inputs: &[("plank", 1), ("nails", 4)],
        outputs: &[("nail_club", 1)],
        station: Some((Station::Workbench, 1)),
        tools: &["build"],
        minutes: 30,
        noise: 4,
        ..R
    },
    RecipeDef {
        id: "satchel",
        name: "Sew a satchel",
        icon: "pack",
        inputs: &[("cloth", 4), ("rope", 1)],
        outputs: &[("satchel", 1)],
        station: Some((Station::Workbench, 1)),
        minutes: 90,
        ..R
    },
    RecipeDef {
        id: "gloves",
        name: "Cut and sew gloves",
        icon: "coat",
        inputs: &[("leather", 1), ("cloth", 1)],
        outputs: &[("gloves", 1)],
        station: Some((Station::Workbench, 1)),
        minutes: 60,
        ..R
    },
    RecipeDef {
        id: "rucksack",
        name: "Sew a rucksack",
        icon: "pack",
        inputs: &[("leather", 3), ("cloth", 2), ("rope", 1)],
        outputs: &[("rucksack", 1)],
        station: Some((Station::Workbench, 2)),
        minutes: 180,
        ..R
    },
    RecipeDef {
        id: "oil_lamp",
        name: "Make an oil lamp",
        icon: "light",
        inputs: &[("tin_can", 1), ("glass_bottle", 1), ("cloth", 1)],
        outputs: &[("oil_lamp", 1)],
        station: Some((Station::Workbench, 2)),
        minutes: 60,
        ..R
    },
    RecipeDef {
        id: "waders",
        name: "Seal a pair of waders",
        icon: "coat",
        inputs: &[("leather", 2), ("pitch", 1), ("cloth", 2)],
        outputs: &[("waders", 1)],
        station: Some((Station::Workbench, 2)),
        fire: true,
        minutes: 120,
        ..R
    },
    RecipeDef {
        id: "hull_patch",
        name: "Make a hull patch",
        icon: "part",
        inputs: &[("plank", 2), ("pitch", 2), ("scrap", 2), ("nails", 6)],
        outputs: &[("hull_patch", 1)],
        station: Some((Station::Workbench, 2)),
        fire: true,
        tools: &["build"],
        minutes: 240,
        noise: 8,
    },
    RecipeDef {
        id: "frame_pack",
        name: "Build a frame pack",
        icon: "pack",
        inputs: &[("plank", 2), ("leather", 2), ("rope", 2), ("nails", 4)],
        outputs: &[("frame_pack", 1)],
        station: Some((Station::Workbench, 3)),
        tools: &["build"],
        minutes: 240,
        noise: 4,
        ..R
    },
    RecipeDef {
        id: "drive_belt",
        name: "Stitch a drive belt",
        icon: "part",
        inputs: &[("leather", 4), ("rope", 1)],
        outputs: &[("drive_belt", 1)],
        station: Some((Station::Workbench, 3)),
        minutes: 300,
        ..R
    },
    RecipeDef {
        id: "cut_belt",
        name: "Cut the belting to size",
        icon: "part",
        inputs: &[("belting", 1)],
        outputs: &[("drive_belt", 1)],
        station: Some((Station::Workbench, 1)),
        minutes: 90,
        ..R
    },
    RecipeDef {
        id: "rewind_coil",
        name: "Rewind the spark coil",
        icon: "part",
        inputs: &[("coil_cracked", 1), ("wire", 4), ("battery", 2), ("parts", 1)],
        outputs: &[("spark_coil", 1)],
        station: Some((Station::Workbench, 3)),
        minutes: 240,
        ..R
    },
    // Stove
    RecipeDef {
        id: "boil_water",
        name: "Boil river water",
        icon: "water",
        inputs: &[("water_dirty", 3)],
        outputs: &[("water_clean", 3)],
        station: Some((Station::Stove, 0)),
        fire: true,
        minutes: 25,
        ..R
    },
    RecipeDef {
        id: "cook_fish",
        name: "Cook fish",
        icon: "fish",
        inputs: &[("raw_fish", 1)],
        outputs: &[("cooked_fish", 1)],
        station: Some((Station::Stove, 0)),
        fire: true,
        minutes: 20,
        ..R
    },
    RecipeDef {
        id: "roast_rabbit",
        name: "Roast a rabbit",
        icon: "meat",
        inputs: &[("raw_meat", 1)],
        outputs: &[("cooked_meat", 1)],
        station: Some((Station::Stove, 0)),
        fire: true,
        minutes: 35,
        ..R
    },
    RecipeDef {
        id: "bake_potatoes",
        name: "Bake potatoes",
        icon: "crop",
        inputs: &[("potato", 2)],
        outputs: &[("baked_potato", 2)],
        station: Some((Station::Stove, 0)),
        fire: true,
        minutes: 40,
        ..R
    },
    RecipeDef {
        id: "porridge",
        name: "Make porridge",
        icon: "pot",
        inputs: &[("oats", 1), ("water_clean", 1)],
        outputs: &[("porridge", 2)],
        station: Some((Station::Stove, 0)),
        fire: true,
        minutes: 20,
        ..R
    },
    RecipeDef {
        id: "hot_tea",
        name: "Brew tea",
        icon: "pot",
        inputs: &[("tea", 1), ("water_clean", 1)],
        outputs: &[("hot_tea", 2)],
        station: Some((Station::Stove, 0)),
        fire: true,
        minutes: 10,
        ..R
    },
    RecipeDef {
        id: "rabbit_stew",
        name: "Make rabbit stew",
        icon: "pot",
        inputs: &[("raw_meat", 1), ("potato", 1), ("water_clean", 1)],
        outputs: &[("stew", 2)],
        station: Some((Station::Stove, 2)),
        fire: true,
        minutes: 60,
        ..R
    },
    RecipeDef {
        id: "root_stew",
        name: "Make root stew",
        icon: "pot",
        inputs: &[("turnip", 1), ("kale", 1), ("potato", 1), ("water_clean", 1)],
        outputs: &[("stew", 2)],
        station: Some((Station::Stove, 2)),
        fire: true,
        minutes: 60,
        ..R
    },
    RecipeDef {
        id: "smoke_fish",
        name: "Smoke fish",
        icon: "fish",
        inputs: &[("raw_fish", 2), ("firewood", 1)],
        outputs: &[("smoked_fish", 2)],
        station: Some((Station::Stove, 2)),
        fire: true,
        minutes: 120,
        ..R
    },
    RecipeDef {
        id: "distil",
        name: "Distil spirits",
        icon: "bottle",
        inputs: &[("potato", 3), ("water_clean", 2), ("firewood", 1)],
        outputs: &[("spirits", 1)],
        station: Some((Station::Stove, 3)),
        fire: true,
        minutes: 240,
        ..R
    },
    // Medicine shelf
    RecipeDef {
        id: "bandages",
        name: "Boil cloth into bandages",
        icon: "bandage",
        inputs: &[("cloth", 1), ("water_clean", 1)],
        outputs: &[("bandage", 2)],
        station: Some((Station::Medicine, 1)),
        fire: true,
        minutes: 30,
        ..R
    },
    RecipeDef {
        id: "poultice",
        name: "Make a poultice",
        icon: "bandage",
        inputs: &[("herbs", 2), ("cloth", 1)],
        outputs: &[("poultice", 1)],
        station: Some((Station::Medicine, 1)),
        minutes: 20,
        ..R
    },
    RecipeDef {
        id: "antiseptic",
        name: "Make antiseptic",
        icon: "pills",
        inputs: &[("spirits", 1), ("herbs", 1)],
        outputs: &[("antiseptic", 2)],
        station: Some((Station::Medicine, 2)),
        minutes: 40,
        ..R
    },
    RecipeDef {
        id: "tincture",
        name: "Steep willow tincture",
        icon: "pills",
        inputs: &[("herbs", 2), ("water_clean", 1)],
        outputs: &[("tincture", 1)],
        station: Some((Station::Medicine, 2)),
        fire: true,
        minutes: 60,
        ..R
    },
];

pub fn get(id: &str) -> Option<&'static RecipeDef> {
    RECIPES.iter().find(|r| r.id == id)
}
