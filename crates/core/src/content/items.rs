//! Items. Weights are in tenths of a kilogram; values are barter units.

use super::Slot;

#[derive(Clone, Copy, Debug)]
pub struct WeaponDef {
    pub dmg_min: i32,
    pub dmg_max: i32,
    /// Percent added to (or taken from) the base hit chance.
    pub accuracy: i32,
    /// Can strike at the near band, not only close.
    pub reach: bool,
    pub noise: u32,
    /// Strikes or shots before one breaks.
    pub durability: u32,
    pub ammo: Option<&'static str>,
}

#[derive(Clone, Copy, Debug)]
pub enum MedKind {
    /// Stops bleeding with the given base percent chance.
    Bandage(i32),
    /// Lowers infection by this much and slows it for a day.
    Antiseptic(i32),
    /// Lowers infection by this much; cures outright below the threshold.
    Antibiotic(i32),
    /// Restores health and nerve.
    Painkiller { health: i32, nerve: i32 },
}

#[derive(Clone, Copy, Debug)]
pub enum ItemKind {
    /// Eaten or drunk. `sick` is the percent chance of a stomach sickness.
    Food {
        sat: i32,
        hyd: i32,
        nerve: i32,
        warm: i32,
        sick: i32,
    },
    Medicine(MedKind),
    Material,
    Weapon(WeaponDef),
    Ammo,
    Tool,
    /// A light for a dark room. `uses` per unit for self-consuming lights;
    /// lamps instead burn one use of `fuel`.
    Light {
        uses: u32,
        fuel: Option<&'static str>,
    },
    /// Burnt by a light; `uses` per unit.
    Fuel {
        uses: u32,
    },
    Bag {
        capacity: u32,
    },
    Clothing {
        slot: Slot,
        insulation: i32,
        rainproof: bool,
        wading: bool,
    },
    Part,
    Key,
    Seed {
        crop: &'static str,
        days: u32,
    },
    Bait,
    Comfort {
        nerve: i32,
    },
    Trap,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub icon: &'static str,
    pub weight: u32,
    pub value: u32,
    pub kind: ItemKind,
    /// Percent of a stack lost each dawn before the store's protection.
    pub perish: u32,
    /// Extra water storage this item gives while kept at the lock house.
    pub holds_water: u32,
    /// Noise when thrown to draw them off; 0 if it cannot be thrown.
    pub throw: u32,
    /// Tool roles: "build", "chop", "force", "spanner", "pick".
    pub tags: &'static [&'static str],
}

const fn food(sat: i32, hyd: i32, nerve: i32, warm: i32, sick: i32) -> ItemKind {
    ItemKind::Food {
        sat,
        hyd,
        nerve,
        warm,
        sick,
    }
}

const BASE: ItemDef = ItemDef {
    id: "",
    name: "",
    desc: "",
    icon: "item",
    weight: 1,
    value: 1,
    kind: ItemKind::Material,
    perish: 0,
    holds_water: 0,
    throw: 0,
    tags: &[],
};

macro_rules! items {
    ($($id:literal => { $($field:ident : $val:expr),* $(,)? }),* $(,)?) => {
        pub static ITEMS: &[ItemDef] = &[
            $(ItemDef { id: $id, $($field: $val,)* ..BASE }),*
        ];
    };
}

items! {
    // Food
    "tinned_meat" => { name: "Tinned meat", desc: "Salt beef in a dented tin. It keeps.", icon: "tin", weight: 4, value: 6, kind: food(34, 0, 1, 0, 0) },
    "tinned_beans" => { name: "Tinned beans", desc: "Beans in sauce. Cold, but food.", icon: "tin", weight: 4, value: 5, kind: food(27, 4, 0, 0, 0) },
    "tinned_peaches" => { name: "Tinned peaches", desc: "Syrup and soft fruit. Something like comfort.", icon: "tin", weight: 4, value: 6, kind: food(14, 12, 5, 0, 0) },
    "biscuits" => { name: "Ship's biscuit", desc: "Hard as slate. Dry in the mouth.", icon: "food", weight: 1, value: 2, kind: food(13, -3, 0, 0, 0) },
    "chocolate" => { name: "Chocolate", desc: "A bar, half its paper gone.", icon: "food", weight: 1, value: 5, kind: food(9, 0, 6, 0, 0) },
    "jam" => { name: "Jar of jam", desc: "Blackberry. Someone's label, someone's hand.", icon: "food", weight: 4, value: 5, kind: food(12, 2, 5, 0, 0) },
    "oats" => { name: "Oats", desc: "A sack's worth. Better cooked.", icon: "food", weight: 3, value: 3, kind: food(6, -2, 0, 0, 0) },
    "potato" => { name: "Potatoes", desc: "Earth still on them. Better cooked.", icon: "crop", weight: 3, value: 2, kind: food(6, 1, 0, 0, 0), perish: 4 },
    "turnip" => { name: "Turnips", desc: "Hard and white. They keep a while.", icon: "crop", weight: 3, value: 2, kind: food(7, 2, 0, 0, 0), perish: 4 },
    "kale" => { name: "Kale", desc: "Dark leaves. Grows in the cold.", icon: "crop", weight: 1, value: 2, kind: food(6, 3, 0, 0, 0), perish: 15 },
    "apple" => { name: "Apples", desc: "Windfalls. Some bruised.", icon: "crop", weight: 2, value: 2, kind: food(8, 5, 1, 0, 0), perish: 8 },
    "raw_fish" => { name: "Raw fish", desc: "Roach and perch from the lock. Cook it.", icon: "fish", weight: 4, value: 3, kind: food(6, 2, 0, 0, 30), perish: 45 },
    "cooked_fish" => { name: "Cooked fish", desc: "Hot through. Eat it soon.", icon: "fish", weight: 3, value: 4, kind: food(24, 2, 2, 4, 0), perish: 30 },
    "smoked_fish" => { name: "Smoked fish", desc: "Dark and dry. It will keep for weeks.", icon: "fish", weight: 2, value: 6, kind: food(21, -2, 1, 0, 0), perish: 2 },
    "raw_meat" => { name: "Rabbit, skinned", desc: "From the snares. Cook it.", icon: "meat", weight: 6, value: 3, kind: food(6, 0, 0, 0, 35), perish: 45 },
    "cooked_meat" => { name: "Roast rabbit", desc: "Lean, and it fills you.", icon: "meat", weight: 4, value: 5, kind: food(31, 0, 3, 4, 0), perish: 30 },
    "stew" => { name: "Stew", desc: "Thick and hot. The best thing in the valley.", icon: "pot", weight: 6, value: 7, kind: food(48, 10, 5, 10, 0), perish: 35 },
    "porridge" => { name: "Porridge", desc: "Grey and filling.", icon: "pot", weight: 4, value: 3, kind: food(23, 6, 2, 8, 0), perish: 50 },
    "baked_potato" => { name: "Baked potatoes", desc: "Skins black from the stove.", icon: "crop", weight: 3, value: 3, kind: food(17, 2, 1, 5, 0), perish: 20 },
    "hot_tea" => { name: "Hot tea", desc: "In a tin mug. It goes cold by night.", icon: "pot", weight: 3, value: 1, kind: food(2, 26, 6, 14, 0), perish: 100 },
    // Drink
    "water_dirty" => { name: "River water", desc: "Brown from the river. Boil it, or risk it.", icon: "water", weight: 10, value: 0, kind: food(0, 24, 0, 0, 22) },
    "water_clean" => { name: "Clean water", desc: "Boiled or bottled.", icon: "water", weight: 10, value: 2, kind: food(0, 30, 0, 0, 0) },
    "spirits" => { name: "Spirits", desc: "Clear and harsh. For nerve, or for wounds.", icon: "bottle", weight: 8, value: 8, kind: food(0, -6, 16, 6, 0) },
    "tea" => { name: "Tea leaves", desc: "A caddy, a quarter full.", icon: "food", weight: 1, value: 4, kind: ItemKind::Material },
    // Medicine
    "rag" => { name: "Rag strips", desc: "Torn cloth. It may hold a bleed.", icon: "bandage", weight: 1, value: 1, kind: ItemKind::Medicine(MedKind::Bandage(42)) },
    "bandage" => { name: "Bandage", desc: "Clean, rolled tight.", icon: "bandage", weight: 1, value: 4, kind: ItemKind::Medicine(MedKind::Bandage(72)) },
    "antiseptic" => { name: "Antiseptic", desc: "A brown bottle that stings.", icon: "pills", weight: 2, value: 6, kind: ItemKind::Medicine(MedKind::Antiseptic(22)) },
    "poultice" => { name: "Herb poultice", desc: "Yarrow and willow in cloth.", icon: "bandage", weight: 1, value: 3, kind: ItemKind::Medicine(MedKind::Antiseptic(14)) },
    "antibiotics" => { name: "Antibiotics", desc: "A paper fold of tablets. Worth more than anything.", icon: "pills", weight: 1, value: 16, kind: ItemKind::Medicine(MedKind::Antibiotic(70)) },
    "painkillers" => { name: "Painkillers", desc: "Aspirin, a strip of six.", icon: "pills", weight: 1, value: 6, kind: ItemKind::Medicine(MedKind::Painkiller { health: 6, nerve: 8 }) },
    "tincture" => { name: "Willow tincture", desc: "Bitter. It takes the edge off.", icon: "pills", weight: 2, value: 4, kind: ItemKind::Medicine(MedKind::Painkiller { health: 5, nerve: 6 }) },
    "herbs" => { name: "Yarrow and willow bark", desc: "Gathered on the bank. For poultices.", icon: "herb", weight: 1, value: 2, perish: 6 },
    // Materials
    "plank" => { name: "Planks", desc: "Floorboards and fence rails.", icon: "plank", weight: 22, value: 2 },
    "nails" => { name: "Nails", desc: "A fistful, some bent.", icon: "metal", weight: 1, value: 1 },
    "scrap" => { name: "Scrap metal", desc: "Pipe, hinges, a stove door.", icon: "metal", weight: 8, value: 1 },
    "cloth" => { name: "Cloth", desc: "Curtains, sheets, a shirt.", icon: "cloth", weight: 2, value: 1 },
    "rope" => { name: "Rope", desc: "Hemp, a few yards.", icon: "cloth", weight: 6, value: 2 },
    "wire" => { name: "Wire", desc: "Fence wire and fuse wire.", icon: "metal", weight: 2, value: 1 },
    "parts" => { name: "Machine parts", desc: "Springs, gears, a valve.", icon: "metal", weight: 6, value: 4 },
    "leather" => { name: "Leather", desc: "Belting, harness, an old bag.", icon: "cloth", weight: 4, value: 3 },
    "pitch" => { name: "Pitch", desc: "Black and stiff in a tin. Seals a hull.", icon: "tin", weight: 8, value: 3 },
    "glass_bottle" => { name: "Empty bottle", desc: "Holds water. Breaks loudly when thrown.", icon: "bottle", weight: 3, value: 0, holds_water: 1, throw: 10 },
    "tin_can" => { name: "Empty tin", desc: "Rinsed out.", icon: "tin", weight: 1, value: 0 },
    "jerrycan" => { name: "Jerrycan", desc: "Twenty litres, if it is full.", icon: "fuel", weight: 15, value: 6, holds_water: 5 },
    "firewood" => { name: "Firewood", desc: "Split and mostly dry.", icon: "plank", weight: 18, value: 1 },
    "tobacco" => { name: "Tobacco", desc: "A tin of rolling tobacco.", icon: "tin", weight: 1, value: 5, kind: ItemKind::Comfort { nerve: 9 } },
    // Light
    "candle" => { name: "Candle", desc: "Tallow. Enough for one dark room.", icon: "light", weight: 1, value: 2, kind: ItemKind::Light { uses: 1, fuel: None } },
    "torch" => { name: "Pitch torch", desc: "Rag on a stick. Two rooms, if you are quick.", icon: "light", weight: 4, value: 2, kind: ItemKind::Light { uses: 2, fuel: None } },
    "oil_lamp" => { name: "Oil lamp", desc: "A hurricane lamp. Burns lamp oil.", icon: "light", weight: 6, value: 7, kind: ItemKind::Light { uses: 0, fuel: Some("lamp_oil") } },
    "flashlight" => { name: "Flashlight", desc: "A steel torch. Takes batteries.", icon: "light", weight: 4, value: 10, kind: ItemKind::Light { uses: 0, fuel: Some("battery") } },
    "lamp_oil" => { name: "Lamp oil", desc: "Paraffin in a small can. Three rooms of light.", icon: "fuel", weight: 5, value: 4, kind: ItemKind::Fuel { uses: 3 } },
    "battery" => { name: "Battery", desc: "Square, heavy for its size. Four rooms of light, or the radio.", icon: "fuel", weight: 1, value: 4, kind: ItemKind::Fuel { uses: 4 } },
    // Weapons and ammunition
    "kitchen_knife" => { name: "Kitchen knife", desc: "Short. You have to be very close.", icon: "blade", weight: 2, value: 3, kind: ItemKind::Weapon(WeaponDef { dmg_min: 8, dmg_max: 12, accuracy: 5, reach: false, noise: 1, durability: 30, ammo: None }) },
    "hatchet" => { name: "Hatchet", desc: "For kindling, and not only kindling.", icon: "blade", weight: 9, value: 7, kind: ItemKind::Weapon(WeaponDef { dmg_min: 13, dmg_max: 19, accuracy: 0, reach: false, noise: 2, durability: 45, ammo: None }), tags: &["chop"] },
    "crowbar" => { name: "Crowbar", desc: "Opens doors. Opens other things.", icon: "club", weight: 15, value: 6, kind: ItemKind::Weapon(WeaponDef { dmg_min: 11, dmg_max: 16, accuracy: 0, reach: false, noise: 2, durability: 90, ammo: None }), tags: &["force"] },
    "boathook" => { name: "Boathook", desc: "Long. Keeps them at a distance.", icon: "club", weight: 18, value: 5, kind: ItemKind::Weapon(WeaponDef { dmg_min: 9, dmg_max: 13, accuracy: 0, reach: true, noise: 1, durability: 40, ammo: None }) },
    "nail_club" => { name: "Nailed club", desc: "A table leg and a handful of nails.", icon: "club", weight: 12, value: 3, kind: ItemKind::Weapon(WeaponDef { dmg_min: 10, dmg_max: 15, accuracy: -5, reach: false, noise: 2, durability: 22, ammo: None }) },
    "revolver" => { name: "Revolver", desc: "Six chambers. The sound will carry for miles.", icon: "gun", weight: 9, value: 26, kind: ItemKind::Weapon(WeaponDef { dmg_min: 26, dmg_max: 36, accuracy: 0, reach: true, noise: 30, durability: 250, ammo: Some("rounds") }) },
    "shotgun" => { name: "Shotgun", desc: "Double barrel. Nothing close survives it.", icon: "gun", weight: 30, value: 30, kind: ItemKind::Weapon(WeaponDef { dmg_min: 36, dmg_max: 52, accuracy: 10, reach: true, noise: 42, durability: 200, ammo: Some("shells") }) },
    "rounds" => { name: "Revolver rounds", desc: "Brass, loose in a paper box.", icon: "rounds", weight: 0, value: 3, kind: ItemKind::Ammo },
    "shells" => { name: "Shotgun shells", desc: "Red paper cases.", icon: "rounds", weight: 1, value: 3, kind: ItemKind::Ammo },
    // Tools and traps
    "hammer" => { name: "Claw hammer", desc: "The lock-keeper's. You need it to build, and it will break a lock, loudly.", icon: "tool", weight: 6, value: 4, kind: ItemKind::Tool, tags: &["build", "force"] },
    "saw" => { name: "Bow saw", desc: "Cuts planks to firewood.", icon: "tool", weight: 8, value: 4, kind: ItemKind::Tool, tags: &["chop"] },
    "spanner" => { name: "Spanner roll", desc: "A full set in oiled canvas.", icon: "tool", weight: 8, value: 6, kind: ItemKind::Tool, tags: &["spanner"] },
    "lockpicks" => { name: "Lockpicks", desc: "Bent wire and a tension bar.", icon: "lock", weight: 0, value: 6, kind: ItemKind::Tool, tags: &["pick"] },
    "snare" => { name: "Wire snare", desc: "Set along the bank for rabbits.", icon: "trap", weight: 1, value: 2, kind: ItemKind::Trap },
    "fishing_line" => { name: "Night line", desc: "Hooks on a weighted line, set from the lock gate.", icon: "trap", weight: 1, value: 3, kind: ItemKind::Trap },
    "bait" => { name: "Worms", desc: "Dug from the bank, in a tin of soil.", icon: "trap", weight: 1, value: 0, kind: ItemKind::Bait, perish: 20 },
    "tin_rattle" => { name: "Tin rattle", desc: "Nails in a tin. Thrown, it calls them away.", icon: "noise", weight: 2, value: 1, throw: 14 },
    // Bags
    "satchel" => { name: "Satchel", desc: "Canvas over the shoulder. Room for 4 kg more.", icon: "pack", weight: 0, value: 5, kind: ItemKind::Bag { capacity: 40 } },
    "rucksack" => { name: "Rucksack", desc: "A walker's pack. Room for 9 kg more.", icon: "pack", weight: 0, value: 10, kind: ItemKind::Bag { capacity: 90 } },
    "frame_pack" => { name: "Frame pack", desc: "Timber frame, leather straps. Room for 15 kg more.", icon: "pack", weight: 0, value: 14, kind: ItemKind::Bag { capacity: 150 } },
    // Clothing
    "wool_coat" => { name: "Wool coat", desc: "Heavy, long, someone else's.", icon: "coat", weight: 18, value: 10, kind: ItemKind::Clothing { slot: Slot::Coat, insulation: 16, rainproof: false, wading: false } },
    "work_jacket" => { name: "Work jacket", desc: "Canvas, lined. Better than nothing.", icon: "coat", weight: 10, value: 5, kind: ItemKind::Clothing { slot: Slot::Coat, insulation: 8, rainproof: false, wading: false } },
    "oilskin" => { name: "Oilskin", desc: "Stiff, yellowed. Keeps the rain out.", icon: "coat", weight: 8, value: 7, kind: ItemKind::Clothing { slot: Slot::Shell, insulation: 3, rainproof: true, wading: false } },
    "wool_hat" => { name: "Wool hat", desc: "Darned at the crown.", icon: "coat", weight: 1, value: 3, kind: ItemKind::Clothing { slot: Slot::Head, insulation: 5, rainproof: false, wading: false } },
    "scarf" => { name: "Scarf", desc: "Long enough to wind twice.", icon: "coat", weight: 1, value: 3, kind: ItemKind::Clothing { slot: Slot::Neck, insulation: 5, rainproof: false, wading: false } },
    "gloves" => { name: "Gloves", desc: "Leather, stiff at the knuckles.", icon: "coat", weight: 1, value: 4, kind: ItemKind::Clothing { slot: Slot::Hands, insulation: 6, rainproof: false, wading: false } },
    "waders" => { name: "Waders", desc: "Rubber to the chest. Flooded rooms stop costing you warmth.", icon: "coat", weight: 14, value: 8, kind: ItemKind::Clothing { slot: Slot::Legs, insulation: 2, rainproof: false, wading: true } },
    // Ferry parts
    "hull_patch" => { name: "Hull patch", desc: "Copper sheet, pitch and rivets. For the ferry's split plank.", icon: "part", weight: 30, value: 20, kind: ItemKind::Part },
    "fuel_drum" => { name: "Fuel drum", desc: "Forty litres of paraffin. Too heavy to carry without a proper bag.", icon: "part", weight: 140, value: 24, kind: ItemKind::Part },
    "spark_coil" => { name: "Spark coil", desc: "Wound copper in a black case. The ferry's engine will not fire without it.", icon: "part", weight: 8, value: 20, kind: ItemKind::Part },
    "drive_belt" => { name: "Drive belt", desc: "Stitched leather belting, oiled.", icon: "part", weight: 14, value: 18, kind: ItemKind::Part },
    "coil_cracked" => { name: "Cracked spark coil", desc: "The Kittiwake's coil. The windings have split. It needs rewinding at a bench.", icon: "part", weight: 8, value: 8 },
    "belting" => { name: "Engine belting", desc: "A length of the mill's leather belting. Too long and too wide as it is.", icon: "part", weight: 22, value: 6 },
    // Keys
    "key_dispensary" => { name: "Dispensary key", desc: "Small, brass, tagged 'Disp.'", icon: "key", weight: 0, value: 0, kind: ItemKind::Key },
    "key_station" => { name: "Station keys", desc: "A ring of three. One is stamped 'Sgt.'", icon: "key", weight: 0, value: 0, kind: ItemKind::Key },
    "key_mill" => { name: "Mill office key", desc: "Long iron key on a loop of string.", icon: "key", weight: 0, value: 0, kind: ItemKind::Key },
    "key_engine" => { name: "Engine house key", desc: "Heavy, stamped 'E.H.', on the manager's ring.", icon: "key", weight: 0, value: 0, kind: ItemKind::Key },
    "key_wharf" => { name: "Wharf padlock key", desc: "Tagged 'Fuel store'.", icon: "key", weight: 0, value: 0, kind: ItemKind::Key },
    // Seeds
    "seed_kale" => { name: "Kale seed", desc: "A paper twist. Grows in the cold.", icon: "herb", weight: 0, value: 3, kind: ItemKind::Seed { crop: "kale", days: 6 } },
    "seed_turnip" => { name: "Turnip seed", desc: "A paper twist.", icon: "herb", weight: 0, value: 3, kind: ItemKind::Seed { crop: "turnip", days: 8 } },
    "seed_potato" => { name: "Seed potatoes", desc: "Sprouting in the dark.", icon: "herb", weight: 3, value: 4, kind: ItemKind::Seed { crop: "potato", days: 10 } },
}

pub fn get(id: &str) -> Option<&'static ItemDef> {
    ITEMS.iter().find(|i| i.id == id)
}

/// Name or the id itself (never panics on unknown ids).
pub fn name(id: &str) -> &str {
    get(id).map_or(id, |d| d.name)
}

pub fn is_food(id: &str) -> bool {
    matches!(get(id).map(|d| d.kind), Some(ItemKind::Food { .. }))
}

/// Food that is not plain water: what a dog or a stranger can be given.
pub fn is_meal(id: &str) -> bool {
    matches!(get(id).map(|d| d.kind), Some(ItemKind::Food { sat, .. }) if sat >= 6)
}

pub fn is_part(id: &str) -> bool {
    matches!(get(id).map(|d| d.kind), Some(ItemKind::Part))
}

pub fn weapon(id: &str) -> Option<WeaponDef> {
    match get(id)?.kind {
        ItemKind::Weapon(w) => Some(w),
        _ => None,
    }
}

pub fn has_tag(id: &str, tag: &str) -> bool {
    get(id).is_some_and(|d| d.tags.contains(&tag))
}

pub const FERRY_PARTS: [&str; 4] = ["hull_patch", "fuel_drum", "spark_coil", "drive_belt"];

/// A short category used by the widow's memory of what you sold her.
pub fn category(id: &str) -> &'static str {
    match get(id).map(|d| d.kind) {
        Some(ItemKind::Food { sat, .. }) if sat >= 6 => "food",
        Some(ItemKind::Food { .. }) => "drink",
        Some(ItemKind::Medicine(_)) => "medicine",
        Some(ItemKind::Weapon(_) | ItemKind::Ammo) => "arms",
        Some(ItemKind::Clothing { .. } | ItemKind::Bag { .. }) => "cloth",
        Some(ItemKind::Light { .. } | ItemKind::Fuel { .. }) => "light",
        Some(ItemKind::Tool | ItemKind::Trap) => "tools",
        _ => "goods",
    }
}
