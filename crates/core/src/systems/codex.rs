//! Finding codex entries and discovering places.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::{codex, places};
use crate::state::{GameState, Tone};
use crate::systems::journal;

/// Add an entry to the codex. Returns false if it was already there.
pub fn find(s: &mut GameState, j: &mut Journal, id: &str) -> bool {
    let Some(def) = codex::get(id) else {
        return false;
    };
    if !s.codex.insert(id.to_string()) {
        return false;
    }
    j.events.push(GameEvent::Codex {
        id: id.to_string(),
        kind: def.kind,
    });
    let line = match def.kind {
        codex::CodexKind::Note => format!("You find something written: {}.", def.title),
        codex::CodexKind::Photograph => format!("You find a photograph: {}.", def.title),
        codex::CodexKind::Radio => format!("On the radio: {}.", def.title),
        codex::CodexKind::FieldGuide => format!("Field guide: {}.", def.title),
    };
    journal::write(s, j, Tone::Quiet, &line);
    if let Some(p) = def.reveals {
        reveal(s, j, p);
    }
    true
}

/// Mark a place as known. Returns false if it already was.
pub fn reveal(s: &mut GameState, j: &mut Journal, id: &str) -> bool {
    let Some(def) = places::get(id) else {
        return false;
    };
    let Some(ps) = s.places.get_mut(id) else {
        return false;
    };
    if ps.known {
        return false;
    }
    ps.known = true;
    j.events.push(GameEvent::PlaceFound { place: id.to_string() });
    journal::write(s, j, Tone::Find, &format!("Now you know where to find {}.", def.name));
    true
}
