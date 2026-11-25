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

/// Join a list in plain English: "a, b and c".
pub fn list(parts: &[String]) -> String {
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        _ => {
            let (last, rest) = parts.split_last().unwrap_or((&parts[0], &[]));
            format!("{} and {}", rest.join(", "), last)
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
        _ => ("Unknown", "You died"),
    };
    (cause.to_string(), format!("{how} {place}, on day {day}."))
}
