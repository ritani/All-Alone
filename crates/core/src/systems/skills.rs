//! Skills improve by use. Levels 0..=5 from accumulated practice.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::Skill;
use crate::state::{GameState, Tone};
use crate::systems::journal;

/// Practice needed to reach each level.
pub const THRESHOLDS: [u32; 6] = [0, 3, 8, 16, 28, 45];
pub const MAX_XP: u32 = 999;

pub fn level(xp: u32) -> u8 {
    THRESHOLDS.iter().rposition(|&t| xp >= t).unwrap_or(0) as u8
}

pub fn xp_for(level: u8) -> u32 {
    THRESHOLDS[usize::from(level.min(5))]
}

/// Practice to reach the next level, if any.
pub fn next(xp: u32) -> Option<u32> {
    let l = usize::from(level(xp));
    THRESHOLDS.get(l + 1).copied()
}

pub fn practise(s: &mut GameState, j: &mut Journal, skill: Skill, n: u32) {
    let before = s.skill(skill);
    let xp = s.skills.entry(skill).or_insert(0);
    *xp = xp.saturating_add(n).min(MAX_XP);
    let after = s.skill(skill);
    if after > before {
        j.events.push(GameEvent::SkillUp { skill, level: after });
        journal::write(
            s,
            j,
            Tone::Quiet,
            &format!("{} comes easier now. Level {after}.", skill.name()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels() {
        assert_eq!(level(0), 0);
        assert_eq!(level(2), 0);
        assert_eq!(level(3), 1);
        assert_eq!(level(44), 4);
        assert_eq!(level(45), 5);
        assert_eq!(level(999), 5);
        assert_eq!(next(45), None);
        assert_eq!(next(4), Some(8));
        for l in 0..=5 {
            assert_eq!(level(xp_for(l)), l);
        }
    }
}
