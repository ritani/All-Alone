//! The journal: every outcome as a short second-person line.

use crate::Journal;
use crate::state::{DAY, Entry, GameState, JOURNAL_CAP, Tone};

pub fn clock(minute: u32) -> String {
    let t = minute % DAY;
    format!("{:02}:{:02}", t / 60, t % 60)
}

/// Add a line to the run's journal and to this action's outcome.
pub fn write(s: &mut GameState, j: &mut Journal, tone: Tone, text: &str) {
    if text.is_empty() {
        return;
    }
    let e = Entry {
        day: s.day(),
        time: clock(s.minute),
        text: text.to_string(),
        tone,
    };
    s.journal.push(e.clone());
    if s.journal.len() > JOURNAL_CAP {
        let drop = s.journal.len() - JOURNAL_CAP;
        s.journal.drain(0..drop);
    }
    j.journal.push(e);
}

pub fn plain(s: &mut GameState, j: &mut Journal, text: &str) {
    write(s, j, Tone::Plain, text);
}

pub fn danger(s: &mut GameState, j: &mut Journal, text: &str) {
    write(s, j, Tone::Danger, text);
}

pub fn find(s: &mut GameState, j: &mut Journal, text: &str) {
    write(s, j, Tone::Find, text);
}

/// "a tin", "3 tins": quantity phrasing for item names.
pub fn qty(name: &str, n: u32) -> String {
    if n == 1 {
        name.to_string()
    } else {
        format!("{name} x{n}")
    }
}

/// "1 measure", "3 measures": a number with the right form of its noun.
pub fn count(n: u32, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// Where a death happened, as a phrase that follows the verb:
/// "at the lock house", "at Ford Chapel", "on the road to The Weir".
pub fn death_place(s: &crate::state::GameState) -> String {
    let Some(t) = &s.trip else {
        return "at the lock house".into();
    };
    let name = crate::content::places::get(&t.place).map_or("", |p| p.name);
    if name.is_empty() {
        return "on the road".into();
    }
    if s.phase == crate::state::Phase::Travelling {
        if s.homeward {
            format!("on the road home from {}", lower_the(name))
        } else {
            format!("on the road to {}", lower_the(name))
        }
    } else {
        format!("at {}", lower_the(name))
    }
}

/// "The Weir" reads as "the Weir" mid-sentence.
fn lower_the(name: &str) -> String {
    match name.strip_prefix("The ") {
        Some(rest) => format!("the {rest}"),
        None => name.to_string(),
    }
}

/// Join a list in plain English: "a, b and c". When a name has its own "and"
/// ("yarrow and willow bark"), the last join takes a comma too, so the list
/// still reads as a list: "rope, yarrow and willow bark, and a tin".
pub fn list(parts: &[String]) -> String {
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        _ => {
            let (last, rest) = parts.split_last().unwrap_or((&parts[0], &[]));
            let sep = if parts.iter().any(|p| p.contains(" and ")) {
                ", and "
            } else {
                " and "
            };
            format!("{}{sep}{last}", rest.join(", "))
        }
    }
}

/// The cause of death (short, for the ledger) and its journal line.
pub fn death_line(harm: &str, place: &str, day: u32) -> (String, String) {
    let (cause, how) = match harm {
        "bleeding" => ("Bled out", "You bled out"),
        "infection" => ("Infection", "The wound went bad, and the fever took you"),
        "cold" => ("Cold", "The cold got into you and you lay down in it"),
        "hunger" => ("Starvation", "You starved"),
        "thirst" => ("Thirst", "Thirst finished you"),
        "exhaustion" => ("Exhaustion", "You were too tired to go on, and you did not"),
        "sickness" => ("Sickness", "The sickness from the water took you"),
        "fall" => ("A fall", "You fell and did not get up"),
        "stander" => ("A stander", "A stander took you"),
        "listener" => ("Listeners", "The listener called them, and they came"),
        "runner" => ("A runner", "A runner took you before you could choose"),
        "drowned" => ("The drowned", "The drowned one held you under"),
        "hollow" => ("A hollow", "A hollow took you"),
        "small" => ("A small one", "A small one took you"),
        "gave_up" => ("Gave up", "You gave it up"),
        _ => ("Unknown", "You died"),
    };
    (cause.to_string(), format!("{how} {place}, on day {day}."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GameConfig, Phase, Trip, new_game};

    #[test]
    fn counts_agree() {
        assert_eq!(count(1, "measure", "measures"), "1 measure");
        assert_eq!(count(0, "measure", "measures"), "0 measures");
        assert_eq!(count(3, "day", "days"), "3 days");
    }

    #[test]
    fn lists_read_as_lists() {
        let l = |xs: &[&str]| list(&xs.iter().map(|x| x.to_string()).collect::<Vec<_>>());
        assert_eq!(l(&[]), "");
        assert_eq!(l(&["rope"]), "rope");
        assert_eq!(l(&["rope", "a tin"]), "rope and a tin");
        assert_eq!(l(&["rope", "a tin", "nails"]), "rope, a tin and nails");
        assert_eq!(
            l(&["rope", "yarrow and willow bark", "a tin"]),
            "rope, yarrow and willow bark, and a tin"
        );
        assert_eq!(
            l(&["rope", "yarrow and willow bark"]),
            "rope, and yarrow and willow bark"
        );
    }

    /// Every harm at every kind of place reads as a sentence.
    #[test]
    fn death_lines_read_for_every_place_and_cause() {
        let mut s = new_game(&GameConfig {
            seed: 4,
            difficulty: crate::content::Difficulty::Still,
            background: "clerk".into(),
            keepsake: None,
            daily: None,
            held_marks: vec![],
        })
        .unwrap();
        let harms = [
            "bleeding",
            "infection",
            "cold",
            "hunger",
            "thirst",
            "exhaustion",
            "sickness",
            "fall",
            "stander",
            "listener",
            "runner",
            "drowned",
            "hollow",
            "small",
            "gave_up",
        ];
        let mut places = vec![death_place(&s)];
        for p in crate::content::places::PLACES {
            s.trip = Some(Trip {
                place: p.id.into(),
                room: None,
                dog: false,
                departed: s.minute,
                site_noise: 0,
            });
            for (phase, home) in [
                (Phase::Site, false),
                (Phase::Travelling, false),
                (Phase::Travelling, true),
            ] {
                s.phase = phase;
                s.homeward = home;
                places.push(death_place(&s));
            }
        }
        for place in &places {
            assert!(
                place.starts_with("at ") || place.starts_with("on the road "),
                "bad place phrase: {place}"
            );
            assert!(!place.contains(" The "), "capital article mid-sentence: {place}");
            for h in harms {
                let (cause, text) = death_line(h, place, 3);
                assert_ne!(cause, "Unknown", "{h}");
                assert!(text.ends_with(", on day 3."), "{text}");
                assert!(text.contains(&format!(" {place},")), "{text}");
                assert!(!text.contains("  "), "{text}");
            }
        }
        assert!(places.contains(&"at Ford Chapel".to_string()));
        assert!(places.contains(&"at the Weir".to_string()));
        assert!(places.contains(&"on the road home from the Weir".to_string()));
    }
}
