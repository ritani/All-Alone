//! Places in Calder's Reach. Each has an ordered set of rooms: you must pass
//! through one to reach the next. Loot is seeded per run and per room.

use super::Creature;

#[derive(Clone, Copy, Debug)]
pub struct Lock {
    /// A key that opens it quietly.
    pub key: Option<&'static str>,
    /// Pick difficulty, percent taken from the odds. `None`: cannot be picked.
    pub pick: Option<i32>,
}

#[derive(Clone, Copy, Debug)]
pub struct LootDef {
    pub item: &'static str,
    pub pct: i32,
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RoomDef {
    pub id: &'static str,
    pub name: &'static str,
    /// Written to the journal the first time you enter.
    pub desc: &'static str,
    pub dark: bool,
    pub flooded: bool,
    pub lock: Option<Lock>,
    /// 0..=3: how many of them may be here.
    pub danger: u8,
    /// Weighted kinds for this room; empty means the usual mix.
    pub kinds: &'static [(Creature, u32)],
    pub loot: &'static [LootDef],
    /// Always here, found by any search.
    pub fixed: &'static [(&'static str, u32)],
    /// Codex entries found by a careful search.
    pub codex: &'static [&'static str],
    /// Minutes for a careful search.
    pub search: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PlaceDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub icon: &'static str,
    /// Map position, 0..=100 on both axes. The lock house is at (50, 58).
    pub x: u32,
    pub y: u32,
    /// Walking minutes from the lock house, one way.
    pub minutes: u32,
    pub known_at_start: bool,
    pub rooms: &'static [RoomDef],
}

const fn l(item: &'static str, pct: i32, min: u32, max: u32) -> LootDef {
    LootDef { item, pct, min, max }
}

const fn key(k: &'static str, pick: Option<i32>) -> Option<Lock> {
    Some(Lock { key: Some(k), pick })
}

const fn pick(d: i32) -> Option<Lock> {
    Some(Lock {
        key: None,
        pick: Some(d),
    })
}

const ROOM: RoomDef = RoomDef {
    id: "",
    name: "",
    desc: "",
    dark: false,
    flooded: false,
    lock: None,
    danger: 0,
    kinds: &[],
    loot: &[],
    fixed: &[],
    codex: &[],
    search: 60,
};

pub const HOME_X: u32 = 50;
pub const HOME_Y: u32 = 58;

pub static PLACES: &[PlaceDef] = &[
    PlaceDef {
        id: "weir",
        name: "The Weir",
        desc: "Upstream, where the river falls over the stones. The widow trades on the steps.",
        icon: "place_weir",
        x: 36,
        y: 50,
        minutes: 15,
        known_at_start: true,
        rooms: &[
            RoomDef {
                id: "weir_steps",
                name: "Weir steps",
                desc: "Stone steps down to the water. The roar fills your head and leaves no room for anything else.",
                loot: &[
                    l("raw_fish", 40, 1, 3),
                    l("glass_bottle", 70, 1, 4),
                    l("rope", 50, 1, 2),
                    l("raw_fish", 35, 1, 2),
                    l("herbs", 35, 1, 2),
                ],
                search: 40,
                ..ROOM
            },
            RoomDef {
                id: "weir_hut",
                name: "Keeper's hut",
                desc: "A tarred hut on the sluice wall. The door hangs on one hinge.",
                danger: 1,
                loot: &[
                    l("smoked_fish", 40, 1, 2),
                    l("tinned_beans", 75, 1, 4),
                    l("fishing_line", 35, 1, 1),
                    l("wire", 40, 1, 4),
                    l("tobacco", 25, 1, 1),
                    l("candle", 40, 1, 2),
                ],
                codex: &["note_sluice"],
                ..ROOM
            },
            RoomDef {
                id: "weir_sluice",
                name: "Sluice walk",
                desc: "A narrow iron walk over the race. Something is caught in the grating below.",
                danger: 1,
                flooded: true,
                kinds: &[(Creature::Drowned, 3), (Creature::Stander, 1)],
                loot: &[
                    l("scrap", 60, 1, 6),
                    l("parts", 55, 1, 2),
                    l("boathook", 35, 1, 1),
                    l("jerrycan", 25, 1, 1),
                ],
                search: 50,
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "cottages",
        name: "Towpath Cottages",
        desc: "A row of six on the towpath. Most doors are open.",
        icon: "place_house",
        x: 62,
        y: 47,
        minutes: 35,
        known_at_start: true,
        rooms: &[
            RoomDef {
                id: "cot_front",
                name: "No. 1, front room",
                desc: "Two chairs turned to a cold grate. The clock on the mantel has stopped at ten past two.",
                loot: &[
                    l("scrap", 20, 1, 2),
                    l("nails", 50, 2, 8),
                    l("cloth", 90, 1, 6),
                    l("candle", 50, 1, 2),
                    l("tinned_beans", 70, 1, 3),
                    l("chocolate", 40, 1, 2),
                    l("plank", 65, 1, 4),
                ],
                codex: &["photo_wedding"],
                ..ROOM
            },
            RoomDef {
                id: "cot_kitchen",
                name: "No. 1, kitchen",
                desc: "A pan on the range, a crust of something in it. Plates set for two.",
                danger: 1,
                loot: &[
                    l("plank", 50, 1, 2),
                    l("biscuits", 60, 1, 4),
                    l("tinned_meat", 80, 1, 4),
                    l("oats", 75, 1, 3),
                    l("water_clean", 60, 1, 2),
                    l("kitchen_knife", 40, 1, 1),
                    l("tea", 40, 1, 1),
                    l("tin_can", 70, 1, 4),
                    l("jam", 55, 1, 3),
                ],
                codex: &["note_doctor_card"],
                ..ROOM
            },
            RoomDef {
                id: "cot_upstairs",
                name: "No. 3, upstairs",
                desc: "The stairs creak under you. Curtains drawn. Someone is in the bed.",
                dark: true,
                danger: 2,
                loot: &[
                    l("wire", 40, 1, 4),
                    l("cloth", 80, 2, 6),
                    l("wool_hat", 35, 1, 1),
                    l("painkillers", 35, 1, 1),
                    l("glass_bottle", 70, 1, 4),
                    l("tobacco", 30, 1, 1),
                    l("scarf", 30, 1, 1),
                ],
                codex: &["note_school_letter"],
                ..ROOM
            },
            RoomDef {
                id: "cot_constable",
                name: "No. 6, the constable's house",
                desc: "A blue lamp over the door, unlit. The door is locked.",
                lock: pick(15),
                danger: 1,
                loot: &[
                    l("nails", 40, 2, 8),
                    l("biscuits", 55, 1, 3),
                    l("tinned_meat", 75, 1, 4),
                    l("work_jacket", 45, 1, 1),
                    l("rounds", 30, 2, 4),
                    l("battery", 65, 1, 4),
                    l("satchel", 30, 1, 1),
                ],
                fixed: &[("key_station", 1)],
                codex: &["photo_constable"],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "chapel",
        name: "Ford Chapel",
        desc: "A grey chapel by the old ford. The bell has not rung since.",
        icon: "place_chapel",
        x: 30,
        y: 38,
        minutes: 50,
        known_at_start: true,
        rooms: &[
            RoomDef {
                id: "chapel_porch",
                name: "Porch",
                desc: "Notices for a harvest supper. Coats still on the pegs.",
                loot: &[
                    l("plank", 55, 1, 2),
                    l("nails", 40, 2, 6),
                    l("wool_coat", 30, 1, 1),
                    l("work_jacket", 30, 1, 1),
                    l("candle", 60, 1, 3),
                    l("scarf", 30, 1, 1),
                ],
                search: 30,
                ..ROOM
            },
            RoomDef {
                id: "chapel_nave",
                name: "Nave",
                desc: "The pews are full. Every head is bowed. None of them are praying.",
                danger: 3,
                kinds: &[(Creature::Stander, 5), (Creature::Listener, 2)],
                loot: &[
                    l("tinned_beans", 40, 1, 2),
                    l("biscuits", 75, 1, 5),
                    l("tinned_peaches", 55, 1, 3),
                    l("cloth", 70, 1, 4),
                    l("glass_bottle", 60, 1, 4),
                ],
                codex: &["photo_choir"],
                search: 65,
                ..ROOM
            },
            RoomDef {
                id: "chapel_vestry",
                name: "Vestry",
                desc: "A small room behind the altar. Locked from this side.",
                lock: pick(10),
                loot: &[
                    l("biscuits", 60, 1, 3),
                    l("tinned_meat", 45, 1, 2),
                    l("spirits", 70, 1, 2),
                    l("candle", 70, 2, 4),
                    l("bandage", 40, 1, 2),
                    l("chocolate", 40, 1, 2),
                ],
                fixed: &[("key_mill", 1)],
                codex: &["note_ringer"],
                ..ROOM
            },
            RoomDef {
                id: "chapel_tower",
                name: "Bell tower",
                desc: "Ladders up into the dark. The bell rope moves, a little, with no wind.",
                dark: true,
                danger: 1,
                loot: &[
                    l("rope", 90, 2, 6),
                    l("wire", 50, 1, 4),
                    l("plank", 75, 1, 4),
                    l("battery", 55, 1, 2),
                ],
                codex: &["note_tower"],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "mill",
        name: "Calder Mill",
        desc: "Five storeys of black brick over the river. The town worked here. It is still here.",
        icon: "place_mill",
        x: 18,
        y: 20,
        minutes: 90,
        known_at_start: true,
        rooms: &[
            RoomDef {
                id: "mill_gate",
                name: "Gatehouse",
                desc: "The clocking-in machine. A rack of cards, every one punched for the night shift.",
                danger: 1,
                loot: &[
                    l("tinned_beans", 50, 1, 2),
                    l("tinned_meat", 65, 1, 3),
                    l("biscuits", 75, 1, 4),
                    l("candle", 40, 1, 2),
                    l("work_jacket", 30, 1, 1),
                ],
                codex: &["note_shift_card"],
                search: 40,
                ..ROOM
            },
            RoomDef {
                id: "mill_carding",
                name: "Carding room",
                desc: "Machines like iron animals. The hands stand among them, white with lint.",
                danger: 3,
                loot: &[
                    l("wire", 40, 1, 4),
                    l("nails", 40, 2, 8),
                    l("cloth", 90, 2, 8),
                    l("parts", 75, 1, 4),
                    l("scrap", 60, 1, 6),
                    l("leather", 70, 1, 4),
                ],
                ..ROOM
            },
            RoomDef {
                id: "mill_office",
                name: "Manager's office",
                desc: "Glass on three sides, looking down on the floor. A heavy door.",
                lock: key("key_mill", Some(35)),
                loot: &[
                    l("biscuits", 50, 1, 2),
                    l("spirits", 50, 1, 1),
                    l("revolver", 25, 1, 1),
                    l("rounds", 40, 2, 6),
                    l("tobacco", 40, 1, 1),
                    l("lamp_oil", 40, 1, 1),
                ],
                codex: &["note_mill_ledger", "photo_engine"],
                fixed: &[("key_engine", 1)],
                ..ROOM
            },
            RoomDef {
                id: "mill_weaving",
                name: "Weaving shed",
                desc: "Rows of looms under a saw-tooth roof. The skylights are black with soot. You cannot see the far wall.",
                dark: true,
                danger: 3,
                kinds: &[(Creature::Stander, 4), (Creature::Listener, 2), (Creature::Runner, 2)],
                loot: &[
                    l("cloth", 90, 2, 8),
                    l("leather", 70, 1, 4),
                    l("parts", 65, 1, 4),
                    l("oil_lamp", 25, 1, 1),
                ],
                search: 80,
                ..ROOM
            },
            RoomDef {
                id: "mill_engine",
                name: "Engine house",
                desc: "The great engine fills the room, its flywheel taller than a house. The air hums, very low.",
                lock: key("key_engine", Some(45)),
                dark: true,
                danger: 3,
                kinds: &[(Creature::Stander, 3), (Creature::Hollow, 2), (Creature::Listener, 1)],
                loot: &[
                    l("parts", 90, 2, 6),
                    l("leather", 70, 1, 4),
                    l("scrap", 60, 2, 6),
                    l("lamp_oil", 50, 1, 2),
                    l("spanner", 30, 1, 1),
                ],
                fixed: &[("belting", 1)],
                codex: &["note_engineer"],
                search: 80,
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "school",
        name: "Board School",
        desc: "Two classrooms and a yard behind railings. Children's coats on hooks.",
        icon: "place_school",
        x: 70,
        y: 28,
        minutes: 60,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "school_yard",
                name: "Yard",
                desc: "Hopscotch in chalk, washed half away. A ball against the wall.",
                danger: 1,
                kinds: &[(Creature::Small, 2), (Creature::Stander, 1)],
                loot: &[
                    l("nails", 50, 2, 8),
                    l("plank", 75, 1, 4),
                    l("scrap", 40, 1, 4),
                    l("glass_bottle", 60, 1, 2),
                ],
                search: 30,
                ..ROOM
            },
            RoomDef {
                id: "school_class",
                name: "Classroom",
                desc: "Rows of small desks. On the blackboard, one line of music, a single note held for six bars.",
                danger: 2,
                kinds: &[(Creature::Small, 3), (Creature::Stander, 1)],
                loot: &[
                    l("plank", 55, 1, 3),
                    l("tinned_beans", 40, 1, 2),
                    l("biscuits", 75, 1, 4),
                    l("cloth", 70, 1, 4),
                    l("chocolate", 50, 1, 2),
                    l("wool_hat", 30, 1, 1),
                ],
                codex: &["photo_class"],
                ..ROOM
            },
            RoomDef {
                id: "school_stairs",
                name: "Stairs",
                desc: "A narrow stair to the head's rooms.",
                loot: &[l("candle", 40, 1, 1)],
                search: 20,
                ..ROOM
            },
            RoomDef {
                id: "school_office",
                name: "Headmistress's office",
                desc: "Registers in a neat stack. A first-aid tin on the wall.",
                lock: pick(20),
                loot: &[
                    l("wire", 40, 1, 4),
                    l("bandage", 70, 1, 3),
                    l("antiseptic", 45, 1, 1),
                    l("battery", 65, 1, 4),
                    l("tea", 40, 1, 1),
                    l("seed_kale", 40, 1, 2),
                ],
                codex: &["note_register"],
                ..ROOM
            },
            RoomDef {
                id: "school_kitchen",
                name: "School kitchen",
                desc: "Big pans, a dinner never served.",
                danger: 1,
                loot: &[
                    l("plank", 50, 1, 2),
                    l("oats", 95, 1, 5),
                    l("tinned_beans", 85, 1, 5),
                    l("tinned_peaches", 65, 1, 4),
                    l("water_clean", 50, 1, 3),
                    l("tin_can", 70, 1, 6),
                ],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "surgery",
        name: "The Surgery",
        desc: "Doctor Hale's house and surgery on Quarry Lane.",
        icon: "place_surgery",
        x: 82,
        y: 52,
        minutes: 65,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "surg_waiting",
                name: "Waiting room",
                desc: "Every chair is taken. They sit with their hands in their laps, waiting.",
                danger: 2,
                kinds: &[(Creature::Stander, 3), (Creature::Listener, 1)],
                loot: &[
                    l("biscuits", 40, 1, 2),
                    l("cloth", 60, 1, 4),
                    l("glass_bottle", 70, 1, 4),
                    l("tobacco", 25, 1, 1),
                ],
                search: 40,
                ..ROOM
            },
            RoomDef {
                id: "surg_consult",
                name: "Consulting room",
                desc: "A desk, a couch with straps, a smell of carbolic.",
                loot: &[
                    l("plank", 45, 1, 2),
                    l("bandage", 70, 1, 3),
                    l("painkillers", 60, 1, 2),
                    l("antiseptic", 50, 1, 1),
                    l("spirits", 30, 1, 1),
                ],
                codex: &["note_hale_cases"],
                ..ROOM
            },
            RoomDef {
                id: "surg_dispensary",
                name: "Dispensary",
                desc: "Shelves of brown glass behind a steel door.",
                lock: key("key_dispensary", Some(40)),
                loot: &[
                    l("antibiotics", 80, 1, 3),
                    l("antiseptic", 70, 1, 2),
                    l("painkillers", 60, 1, 2),
                    l("bandage", 60, 2, 3),
                ],
                ..ROOM
            },
            RoomDef {
                id: "surg_flat",
                name: "Doctor's flat",
                desc: "Upstairs, curtains drawn against the day. A wireless, a chair, a cold cup.",
                dark: true,
                danger: 1,
                loot: &[
                    l("wire", 30, 1, 4),
                    l("tinned_peaches", 50, 1, 3),
                    l("tea", 60, 1, 2),
                    l("wool_coat", 40, 1, 1),
                    l("tinned_meat", 75, 1, 4),
                    l("gloves", 35, 1, 1),
                    l("battery", 65, 1, 4),
                ],
                fixed: &[("key_dispensary", 1)],
                codex: &["note_hale_last", "photo_hale"],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "boatyard",
        name: "Lowe's Boatyard",
        desc: "Sheds and slipways below the town bridge. Hulls on trestles.",
        icon: "place_boatyard",
        x: 64,
        y: 76,
        minutes: 55,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "boat_slip",
                name: "Slipway",
                desc: "A half-planked hull on the slip, its ribs open to the sky.",
                danger: 1,
                loot: &[
                    l("tinned_beans", 45, 1, 3),
                    l("plank", 90, 2, 6),
                    l("pitch", 80, 1, 4),
                    l("nails", 70, 3, 12),
                    l("rope", 60, 1, 2),
                ],
                search: 45,
                ..ROOM
            },
            RoomDef {
                id: "boat_loft",
                name: "Sail loft",
                desc: "Canvas hung from the beams like washing. It moves when you do.",
                danger: 1,
                loot: &[
                    l("tinned_meat", 40, 1, 2),
                    l("cloth", 90, 2, 8),
                    l("rope", 90, 1, 6),
                    l("oilskin", 45, 1, 1),
                    l("leather", 60, 1, 4),
                ],
                codex: &["note_lowe"],
                ..ROOM
            },
            RoomDef {
                id: "boat_shop",
                name: "Workshop",
                desc: "Benches, a cold forge, tools in their outlines on the wall.",
                dark: true,
                danger: 2,
                loot: &[
                    l("wire", 50, 1, 4),
                    l("saw", 50, 1, 1),
                    l("nails", 70, 4, 16),
                    l("scrap", 70, 2, 6),
                    l("parts", 65, 1, 4),
                    l("pitch", 70, 1, 4),
                    l("lamp_oil", 40, 1, 1),
                ],
                fixed: &[("key_wharf", 1), ("hammer", 1)],
                ..ROOM
            },
            RoomDef {
                id: "boat_dock",
                name: "Dry dock",
                desc: "The dock gates leak. Waist-deep water, black and still, and the ferry's patch on a trestle at the far end.",
                flooded: true,
                danger: 2,
                kinds: &[(Creature::Drowned, 3), (Creature::Stander, 1)],
                loot: &[l("pitch", 80, 1, 4), l("scrap", 50, 1, 4), l("waders", 30, 1, 1)],
                fixed: &[("hull_patch", 1)],
                codex: &["photo_launch"],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "garage",
        name: "Pell's Garage",
        desc: "A motor garage at the edge of town. A lorry nose-down in the ditch outside.",
        icon: "place_garage",
        x: 88,
        y: 70,
        minutes: 80,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "gar_fore",
                name: "Forecourt",
                desc: "Two pumps, dry. A car with its doors open and its driver at the wheel.",
                danger: 1,
                loot: &[
                    l("nails", 50, 2, 10),
                    l("scrap", 70, 1, 6),
                    l("battery", 75, 1, 4),
                    l("tin_can", 60, 1, 4),
                    l("rope", 50, 1, 2),
                ],
                search: 40,
                ..ROOM
            },
            RoomDef {
                id: "gar_shop",
                name: "Workshop",
                desc: "Oil on the floor, black and soft. Engines hang from chains.",
                dark: true,
                danger: 2,
                kinds: &[(Creature::Stander, 2), (Creature::Runner, 2)],
                loot: &[
                    l("biscuits", 40, 1, 3),
                    l("parts", 90, 1, 6),
                    l("wire", 70, 1, 6),
                    l("crowbar", 40, 1, 1),
                    l("lamp_oil", 50, 1, 2),
                    l("leather", 60, 1, 2),
                ],
                fixed: &[("spanner", 1)],
                ..ROOM
            },
            RoomDef {
                id: "gar_office",
                name: "Office",
                desc: "Invoices on a spike. A calendar of a lake somewhere.",
                lock: pick(15),
                loot: &[
                    l("plank", 50, 1, 2),
                    l("tinned_beans", 50, 1, 3),
                    l("tobacco", 40, 1, 1),
                    l("biscuits", 75, 1, 4),
                    l("flashlight", 40, 1, 1),
                    l("battery", 75, 1, 4),
                    l("spirits", 30, 1, 1),
                ],
                codex: &["note_pell"],
                ..ROOM
            },
            RoomDef {
                id: "gar_pit",
                name: "Inspection pit",
                desc: "A long pit under a van, flooded to the brim with oily water.",
                dark: true,
                flooded: true,
                danger: 2,
                kinds: &[(Creature::Drowned, 2), (Creature::Stander, 1)],
                loot: &[l("parts", 85, 1, 4), l("scrap", 50, 1, 4)],
                fixed: &[("coil_cracked", 1)],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "station",
        name: "Police Station",
        desc: "Red brick, a blue lamp, bars on the low windows.",
        icon: "place_station",
        x: 54,
        y: 24,
        minutes: 60,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "pol_desk",
                name: "Front desk",
                desc: "The day book lies open on the desk. A constable stands behind it, at attention.",
                danger: 1,
                loot: &[
                    l("plank", 50, 1, 2),
                    l("tinned_meat", 45, 1, 3),
                    l("biscuits", 75, 1, 4),
                    l("tea", 40, 1, 1),
                    l("candle", 40, 1, 2),
                    l("battery", 65, 1, 2),
                ],
                codex: &["note_daybook"],
                search: 40,
                ..ROOM
            },
            RoomDef {
                id: "pol_cells",
                name: "Cells",
                desc: "Four cells along a corridor. Something knocks, slow and steady, behind one door.",
                dark: true,
                danger: 2,
                kinds: &[(Creature::Stander, 2), (Creature::Runner, 1), (Creature::Listener, 1)],
                loot: &[
                    l("scrap", 30, 1, 3),
                    l("cloth", 70, 1, 4),
                    l("tinned_meat", 65, 1, 4),
                    l("water_clean", 40, 1, 2),
                ],
                ..ROOM
            },
            RoomDef {
                id: "pol_office",
                name: "Sergeant's office",
                desc: "A gun cabinet on the wall. A desk drawer full of confiscated things.",
                lock: key("key_station", Some(45)),
                loot: &[
                    l("revolver", 60, 1, 1),
                    l("rounds", 80, 4, 8),
                    l("spirits", 40, 1, 1),
                    l("tobacco", 50, 1, 2),
                    l("lockpicks", 40, 1, 1),
                ],
                codex: &["photo_crowd"],
                ..ROOM
            },
            RoomDef {
                id: "pol_store",
                name: "Store room",
                desc: "Lost property, riot shields, a shotgun in a rack.",
                lock: key("key_station", None),
                danger: 1,
                loot: &[
                    l("wire", 50, 1, 6),
                    l("shotgun", 55, 1, 1),
                    l("shells", 80, 3, 6),
                    l("flashlight", 50, 1, 1),
                    l("battery", 85, 1, 6),
                    l("rucksack", 35, 1, 1),
                    l("wool_coat", 30, 1, 1),
                ],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "wharf",
        name: "Coal Wharf",
        desc: "Downstream, where barges load. Black heaps under a crane.",
        icon: "place_wharf",
        x: 86,
        y: 90,
        minutes: 100,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "wharf_yard",
                name: "Coal yard",
                desc: "Heaps of coal slack and a line of empty carts.",
                danger: 1,
                loot: &[
                    l("nails", 40, 2, 8),
                    l("firewood", 90, 2, 4),
                    l("scrap", 60, 1, 6),
                    l("rope", 60, 1, 4),
                    l("tin_can", 60, 1, 4),
                ],
                search: 40,
                ..ROOM
            },
            RoomDef {
                id: "wharf_hut",
                name: "Weighbridge hut",
                desc: "A stove, two mugs, a ledger of tonnages. The kettle is dry.",
                danger: 1,
                loot: &[
                    l("plank", 50, 1, 2),
                    l("parts", 45, 1, 2),
                    l("tinned_meat", 85, 1, 4),
                    l("tea", 50, 1, 1),
                    l("biscuits", 75, 1, 4),
                    l("lamp_oil", 50, 1, 2),
                    l("gloves", 40, 1, 1),
                ],
                codex: &["note_wharfinger"],
                ..ROOM
            },
            RoomDef {
                id: "wharf_barge",
                name: "Barge",
                desc: "A coal barge half sunk at its mooring. The hold is full of water.",
                flooded: true,
                danger: 2,
                kinds: &[(Creature::Drowned, 3)],
                loot: &[
                    l("rope", 90, 1, 6),
                    l("lamp_oil", 50, 1, 2),
                    l("smoked_fish", 65, 1, 5),
                    l("jerrycan", 40, 1, 1),
                ],
                ..ROOM
            },
            RoomDef {
                id: "wharf_store",
                name: "Fuel store",
                desc: "A brick store with a heavy padlock. Inside, drums on their sides.",
                lock: key("key_wharf", None),
                danger: 1,
                loot: &[l("lamp_oil", 80, 1, 3), l("jerrycan", 40, 1, 1), l("pitch", 70, 1, 4)],
                fixed: &[("fuel_drum", 1)],
                ..ROOM
            },
        ],
    },
    PlaceDef {
        id: "farm",
        name: "Hollin Farm",
        desc: "Up the valley side. A farmhouse, a barn, an orchard gone wild.",
        icon: "place_farm",
        x: 10,
        y: 68,
        minutes: 105,
        known_at_start: false,
        rooms: &[
            RoomDef {
                id: "farm_orchard",
                name: "Orchard",
                desc: "Windfalls in the long grass. Wasps, slow with cold.",
                loot: &[l("apple", 95, 3, 8), l("herbs", 60, 1, 3), l("seed_potato", 30, 1, 1)],
                search: 50,
                ..ROOM
            },
            RoomDef {
                id: "farm_kitchen",
                name: "Farmhouse kitchen",
                desc: "Flagstones, a range, hams hanging from the beam.",
                danger: 1,
                loot: &[
                    l("plank", 45, 1, 2),
                    l("jam", 95, 1, 5),
                    l("oats", 95, 1, 4),
                    l("smoked_fish", 65, 1, 4),
                    l("tinned_meat", 75, 1, 4),
                    l("spirits", 40, 1, 1),
                    l("tea", 40, 1, 1),
                ],
                codex: &["note_hollin"],
                ..ROOM
            },
            RoomDef {
                id: "farm_barn",
                name: "Barn",
                desc: "Hay to the rafters. Something has made a nest in it.",
                dark: true,
                danger: 2,
                kinds: &[(Creature::Stander, 2), (Creature::Runner, 1)],
                loot: &[
                    l("wire", 60, 2, 6),
                    l("nails", 50, 3, 12),
                    l("rope", 90, 1, 6),
                    l("leather", 90, 1, 6),
                    l("plank", 85, 2, 6),
                    l("hatchet", 40, 1, 1),
                    l("seed_turnip", 50, 1, 2),
                    l("snare", 40, 1, 2),
                ],
                ..ROOM
            },
            RoomDef {
                id: "farm_cellar",
                name: "Root cellar",
                desc: "Steps down into cold earth. Sacks along the walls.",
                dark: true,
                danger: 1,
                loot: &[
                    l("plank", 50, 1, 2),
                    l("potato", 95, 3, 8),
                    l("turnip", 95, 2, 6),
                    l("seed_potato", 50, 1, 2),
                    l("seed_kale", 40, 1, 1),
                    l("apple", 75, 2, 5),
                ],
                ..ROOM
            },
        ],
    },
];

pub fn get(id: &str) -> Option<&'static PlaceDef> {
    PLACES.iter().find(|p| p.id == id)
}

pub fn index(id: &str) -> Option<usize> {
    PLACES.iter().position(|p| p.id == id)
}

/// Map edges for drawing: pairs of node ids ("home" is the lock house).
pub static PATHS: &[(&str, &str)] = &[
    ("home", "weir"),
    ("home", "cottages"),
    ("home", "boatyard"),
    ("weir", "chapel"),
    ("weir", "farm"),
    ("chapel", "mill"),
    ("cottages", "station"),
    ("cottages", "school"),
    ("cottages", "surgery"),
    ("surgery", "garage"),
    ("boatyard", "garage"),
    ("garage", "wharf"),
    ("boatyard", "wharf"),
];
