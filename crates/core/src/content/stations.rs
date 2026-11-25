//! Lock house stations, levels 0..=3. Level 0 is what the house has to begin with.

use super::Station;

#[derive(Clone, Copy, Debug)]
pub struct LevelDef {
    /// What the station is at this level.
    pub name: &'static str,
    /// What this level does, in one line.
    pub effect: &'static str,
    pub cost: &'static [(&'static str, u32)],
    /// Workbench level needed to build this level.
    pub workbench: u8,
    pub minutes: u32,
    pub noise: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct StationDef {
    pub station: Station,
    pub name: &'static str,
    pub icon: &'static str,
    /// Index 0 is level 0 (free); 1..=3 are the upgrades.
    pub levels: [LevelDef; 4],
}

const fn lvl(
    name: &'static str,
    effect: &'static str,
    cost: &'static [(&'static str, u32)],
    workbench: u8,
    minutes: u32,
    noise: u32,
) -> LevelDef {
    LevelDef {
        name,
        effect,
        cost,
        workbench,
        minutes,
        noise,
    }
}

pub static STATIONS: [StationDef; 9] = [
    StationDef {
        station: Station::Stove,
        name: "Stove",
        icon: "stove",
        levels: [
            lvl(
                "Open hearth",
                "A wood fire burns four hours. Boils water, cooks simply.",
                &[],
                0,
                0,
                0,
            ),
            lvl(
                "Iron stove",
                "A fire burns six hours and warms the house well.",
                &[("scrap", 3), ("parts", 1)],
                0,
                90,
                4,
            ),
            lvl(
                "Stove with oven",
                "Eight hours a load. Stews and smoking.",
                &[("scrap", 4), ("parts", 2), ("nails", 4)],
                1,
                150,
                4,
            ),
            lvl(
                "Range with boiler",
                "Ten hours a load. Distilling.",
                &[("scrap", 5), ("parts", 3), ("pitch", 1)],
                2,
                240,
                5,
            ),
        ],
    },
    StationDef {
        station: Station::Cot,
        name: "Bed",
        icon: "cot",
        levels: [
            lvl("Blanket on the floor", "Poor sleep.", &[], 0, 0, 0),
            lvl(
                "Cot",
                "Fair sleep.",
                &[("plank", 2), ("nails", 4), ("cloth", 2)],
                0,
                90,
                2,
            ),
            lvl(
                "Cot with mattress",
                "Good sleep. Cold bites less.",
                &[("cloth", 4), ("rope", 1)],
                1,
                120,
                0,
            ),
            lvl(
                "Box bed",
                "Deep sleep. Nerve mends overnight.",
                &[("plank", 3), ("cloth", 3), ("nails", 6)],
                2,
                180,
                3,
            ),
        ],
    },
    StationDef {
        station: Station::Workbench,
        name: "Workbench",
        icon: "workbench",
        levels: [
            lvl("Kitchen table", "Simple work by hand.", &[], 0, 0, 0),
            lvl(
                "Workbench",
                "Snares, picks, a satchel.",
                &[("plank", 2), ("nails", 4)],
                0,
                120,
                4,
            ),
            lvl(
                "Bench with vice",
                "Rucksacks, lamps, a hull patch.",
                &[("plank", 1), ("scrap", 3), ("parts", 1), ("nails", 4)],
                1,
                150,
                4,
            ),
            lvl(
                "Full workshop",
                "Coil winding, a frame pack. Mending is faster.",
                &[("scrap", 6), ("parts", 4), ("nails", 6), ("plank", 2)],
                2,
                240,
                5,
            ),
        ],
    },
    StationDef {
        station: Station::Barrel,
        name: "Rain barrel",
        icon: "barrel",
        levels: [
            lvl("No barrel", "Water comes from the river, brown.", &[], 0, 0, 0),
            lvl(
                "Rain barrel",
                "Holds 6 more. Rain fills it with river-grade water.",
                &[("scrap", 2), ("pitch", 1), ("plank", 2)],
                0,
                90,
                2,
            ),
            lvl(
                "Barrel with filter",
                "Rain comes through clean.",
                &[("cloth", 2), ("scrap", 2)],
                1,
                60,
                0,
            ),
            lvl(
                "Two barrels",
                "Holds 6 more again. Rain fills them faster.",
                &[("plank", 3), ("pitch", 2), ("nails", 4)],
                2,
                120,
                2,
            ),
        ],
    },
    StationDef {
        station: Station::Garden,
        name: "Garden bed",
        icon: "garden",
        levels: [
            lvl("Bare ground", "Nothing grows untended.", &[], 0, 0, 0),
            lvl(
                "Raised bed",
                "Two plots. Frost stops growth.",
                &[("plank", 3), ("nails", 4)],
                0,
                120,
                2,
            ),
            lvl(
                "Cold frame",
                "Three plots under glass. Frost no longer stops growth.",
                &[("glass_bottle", 4), ("plank", 2), ("nails", 4)],
                1,
                120,
                2,
            ),
            lvl(
                "Lean-to glasshouse",
                "Four plots. Faster growth.",
                &[("plank", 4), ("cloth", 3), ("nails", 6)],
                2,
                180,
                3,
            ),
        ],
    },
    StationDef {
        station: Station::Barricade,
        name: "Barricade",
        icon: "barricade",
        levels: [
            lvl("Door bar", "Holds very little.", &[], 0, 0, 0),
            lvl(
                "Boarded doors",
                "Holds a night or two of them.",
                &[("plank", 3), ("nails", 8)],
                0,
                120,
                6,
            ),
            lvl(
                "Shuttered house",
                "Holds a good deal.",
                &[("plank", 4), ("scrap", 3), ("nails", 8)],
                1,
                150,
                7,
            ),
            lvl(
                "Iron-bound house",
                "Holds almost anything.",
                &[("scrap", 6), ("plank", 4), ("nails", 10), ("rope", 2)],
                2,
                180,
                8,
            ),
        ],
    },
    StationDef {
        station: Station::Radio,
        name: "Radio",
        icon: "radio",
        levels: [
            lvl("Dead wireless", "Silent.", &[], 0, 0, 0),
            lvl(
                "Working wireless",
                "Voices, sometimes. A guess at tomorrow's weather.",
                &[("battery", 2), ("wire", 2), ("parts", 1)],
                1,
                120,
                0,
            ),
            lvl(
                "Wireless with aerial",
                "Further voices. A better forecast. You can answer.",
                &[("wire", 2), ("scrap", 2), ("rope", 1)],
                1,
                90,
                3,
            ),
            lvl(
                "Tuned set",
                "Clear reception. A forecast you can trust.",
                &[("parts", 3), ("battery", 2), ("wire", 2)],
                2,
                180,
                0,
            ),
        ],
    },
    StationDef {
        station: Station::Medicine,
        name: "Medicine shelf",
        icon: "medicine",
        levels: [
            lvl("Kitchen drawer", "Somewhere to keep things.", &[], 0, 0, 0),
            lvl(
                "Medicine shelf",
                "Boiled bandages. Poultices. Treatment at home holds better.",
                &[("plank", 1), ("nails", 2)],
                0,
                60,
                1,
            ),
            lvl(
                "Stocked shelf",
                "Antiseptic and tinctures.",
                &[("glass_bottle", 2), ("plank", 1), ("cloth", 1)],
                1,
                60,
                0,
            ),
            lvl(
                "Sickroom",
                "Rest at home heals faster. Infection slows.",
                &[("glass_bottle", 2), ("cloth", 3), ("plank", 2)],
                2,
                90,
                1,
            ),
        ],
    },
    StationDef {
        station: Station::Store,
        name: "Store",
        icon: "store",
        levels: [
            lvl("Corner of the kitchen", "Food spoils.", &[], 0, 0, 0),
            lvl(
                "Shelved pantry",
                "Food spoils more slowly.",
                &[("plank", 3), ("nails", 4)],
                0,
                60,
                2,
            ),
            lvl(
                "Cold cellar",
                "Food spoils at half the rate. Vermin stay out.",
                &[("plank", 3), ("nails", 4), ("pitch", 1)],
                1,
                120,
                2,
            ),
            lvl(
                "Sealed larder",
                "Food barely spoils.",
                &[("scrap", 3), ("plank", 3), ("nails", 6)],
                2,
                150,
                3,
            ),
        ],
    },
];

pub fn def(s: Station) -> &'static StationDef {
    let idx = Station::ALL.iter().position(|x| *x == s).unwrap_or(0);
    &STATIONS[idx]
}

/// Hours of fire per load of firewood at a stove level.
pub fn fire_hours(level: u8) -> u32 {
    [4, 6, 8, 10][usize::from(level.min(3))]
}

/// Barricade integrity cap per level.
pub fn barricade_max(level: u8) -> u32 {
    [15, 50, 90, 140][usize::from(level.min(3))]
}

/// Percent of normal sleep recovery per bed level.
pub fn sleep_pct(level: u8) -> i32 {
    [60, 85, 100, 115][usize::from(level.min(3))]
}

/// Percent of base spoilage per store level.
pub fn spoil_pct(level: u8) -> u32 {
    [100, 70, 45, 25][usize::from(level.min(3))]
}

/// Garden plots per level.
pub fn garden_plots(level: u8) -> usize {
    [0, 2, 3, 4][usize::from(level.min(3))]
}

/// Extra water storage from the barrel.
pub fn barrel_capacity(level: u8) -> u32 {
    [0, 6, 6, 12][usize::from(level.min(3))]
}

/// Rain collected per rainy day.
pub fn barrel_rain(level: u8) -> u32 {
    [0, 3, 4, 6][usize::from(level.min(3))]
}

/// Forecast accuracy percent by radio level.
pub fn forecast_pct(level: u8) -> i32 {
    [0, 65, 80, 95][usize::from(level.min(3))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stations_line_up_with_enum() {
        for (i, s) in Station::ALL.iter().enumerate() {
            assert_eq!(STATIONS[i].station, *s);
            assert!(STATIONS[i].levels[0].cost.is_empty());
            for l in &STATIONS[i].levels[1..] {
                assert!(!l.cost.is_empty() && l.minutes > 0);
            }
        }
    }
}
