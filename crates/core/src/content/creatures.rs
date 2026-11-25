//! The Stilled. Several kinds, each with its own behaviour in an encounter.

use super::Creature;

#[derive(Clone, Copy, Debug)]
pub struct CreatureDef {
    pub kind: Creature,
    pub name: &'static str,
    pub plural: &'static str,
    /// One line used when it is first seen in an encounter.
    pub sight: &'static str,
    pub health: i32,
    pub dmg_min: i32,
    pub dmg_max: i32,
    /// Base percent to hit when close.
    pub hit: i32,
    /// Closes one band every `every` turns ...
    pub every: u32,
    /// ... by this many bands.
    pub stride: u8,
    /// Percent per hit that it opens a bleeding wound.
    pub bleed: i32,
    /// Percent per hit that the wound is dirty.
    pub infect: i32,
    /// Percent per turn, while not close, that it calls another.
    pub calls: i32,
    /// A hit holds you: you cannot back away or flee next turn.
    pub grips: bool,
    /// Percent taken from your chance to hold still unnoticed.
    pub hearing: i32,
    /// Field guide entry unlocked on first sight.
    pub guide: &'static str,
}

pub static CREATURES: [CreatureDef; 6] = [
    CreatureDef {
        kind: Creature::Stander,
        name: "stander",
        plural: "standers",
        sight: "A stander turns its head toward you, slowly, the way a plant turns.",
        health: 30,
        dmg_min: 7,
        dmg_max: 12,
        hit: 45,
        every: 2,
        stride: 1,
        bleed: 20,
        infect: 15,
        calls: 0,
        grips: false,
        hearing: 0,
        guide: "fg_stander",
    },
    CreatureDef {
        kind: Creature::Listener,
        name: "listener",
        plural: "listeners",
        sight: "A listener stands with its head tipped, mouth open. It has heard you.",
        health: 16,
        dmg_min: 4,
        dmg_max: 7,
        hit: 45,
        every: 2,
        stride: 1,
        bleed: 10,
        infect: 20,
        calls: 35,
        grips: false,
        hearing: 25,
        guide: "fg_listener",
    },
    CreatureDef {
        kind: Creature::Runner,
        name: "runner",
        plural: "runners",
        sight: "A runner. It comes at you all at once, bent low.",
        health: 18,
        dmg_min: 7,
        dmg_max: 12,
        hit: 55,
        every: 1,
        stride: 2,
        bleed: 30,
        infect: 15,
        calls: 0,
        grips: false,
        hearing: 10,
        guide: "fg_runner",
    },
    CreatureDef {
        kind: Creature::Drowned,
        name: "drowned one",
        plural: "drowned",
        sight: "Something rises out of the black water, streaming.",
        health: 42,
        dmg_min: 9,
        dmg_max: 14,
        hit: 50,
        every: 2,
        stride: 1,
        bleed: 15,
        infect: 35,
        calls: 0,
        grips: true,
        hearing: 0,
        guide: "fg_drowned",
    },
    CreatureDef {
        kind: Creature::Hollow,
        name: "hollow",
        plural: "hollows",
        sight: "A hollow. Too tall, too thin, its chest caved in like a drum.",
        health: 65,
        dmg_min: 12,
        dmg_max: 20,
        hit: 50,
        every: 3,
        stride: 1,
        bleed: 35,
        infect: 20,
        calls: 0,
        grips: true,
        hearing: 15,
        guide: "fg_hollow",
    },
    CreatureDef {
        kind: Creature::Small,
        name: "small one",
        plural: "small ones",
        sight: "A small one. A child's coat, a child's shoes.",
        health: 12,
        dmg_min: 3,
        dmg_max: 6,
        hit: 50,
        every: 1,
        stride: 1,
        bleed: 10,
        infect: 25,
        calls: 15,
        grips: false,
        hearing: 15,
        guide: "fg_small",
    },
];

pub fn def(kind: Creature) -> &'static CreatureDef {
    let idx = Creature::ALL.iter().position(|x| *x == kind).unwrap_or(0);
    &CREATURES[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligned() {
        for (i, k) in Creature::ALL.iter().enumerate() {
            assert_eq!(CREATURES[i].kind, *k);
            assert!(super::super::codex::get(CREATURES[i].guide).is_some());
        }
    }
}
