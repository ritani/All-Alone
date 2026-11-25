//! Standing prose the UI may show outside the journal.

/// Shown when a new run begins, before the first journal line.
pub const INTRO: &[&str] = &[
    "Calder's Reach. A mill town in a river valley.",
    "Three nights ago, at ten past two, almost everyone in the town stopped where they stood.",
    "Some of them move again after dark. They do not see. They come to sound.",
    "You are the lock-keeper's lodger, alone in the lock house below the weir. The ferry on the far slip could carry one person downriver, if it ran.",
    "Winter is coming, and the river will freeze.",
];

/// One line per phase, for screen headers.
pub fn phase_title(p: crate::state::Phase) -> &'static str {
    use crate::state::Phase::*;
    match p {
        Home => "The lock house",
        Packing => "Packing",
        Travelling => "The road",
        Site => "Searching",
        Encounter => "The Stilled",
        Event => "A choice",
        Barter => "The weir steps",
        Night => "Night",
        Dawn => "Dawn",
        Ended => "The end",
    }
}
