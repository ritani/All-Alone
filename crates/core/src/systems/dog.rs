//! The dog. Found at the gate, kept by feeding, lost by neglect or bad luck.

use crate::Journal;
use crate::action::GameEvent;
use crate::content::events::DogFx;
use crate::state::{DogState, GameState, Tone};
use crate::systems::{inventory, journal};

pub fn apply(s: &mut GameState, j: &mut Journal, fx: DogFx) {
    match fx {
        DogFx::Follow => {
            if s.dog.state == DogState::Unmet {
                s.dog.state = DogState::Following;
                s.dog.fed = 100;
            }
        }
        DogFx::Join => {
            if matches!(s.dog.state, DogState::Unmet | DogState::Following) {
                s.dog.state = DogState::Companion;
                s.dog.fed = 100;
                s.dog.days_hungry = 0;
                s.stats.dog_joined = true;
                j.events.push(GameEvent::DogJoined);
            }
        }
        DogFx::Lose => lose(s, j, false),
        DogFx::Hurt(n) => hurt(s, j, n),
        DogFx::Heal(n) => s.dog.health = (s.dog.health + n).min(100),
    }
}

pub fn lose(s: &mut GameState, j: &mut Journal, dead: bool) {
    if matches!(s.dog.state, DogState::Lost | DogState::Dead | DogState::Unmet) {
        return;
    }
    let was_companion = s.dog.state == DogState::Companion;
    s.dog.state = if dead { DogState::Dead } else { DogState::Lost };
    if was_companion {
        s.stats.dog_lost = true;
        j.events.push(GameEvent::DogLost { dead });
    }
    if let Some(t) = s.trip.as_mut() {
        t.dog = false;
    }
}

pub fn hurt(s: &mut GameState, j: &mut Journal, n: i32) {
    if s.dog.state != DogState::Companion {
        return;
    }
    s.dog.health -= n;
    if s.dog.health <= 0 {
        s.dog.health = 0;
        journal::write(s, j, Tone::Danger, "The dog lies still and does not get up again.");
        lose(s, j, true);
    }
}

/// Is the dog with you right now?
pub fn with_you(s: &GameState) -> bool {
    s.dog.state == DogState::Companion && s.trip.as_ref().is_none_or(|t| t.dog)
}

/// Daily: hunger, healing, and leaving if starved.
pub fn dawn(s: &mut GameState, j: &mut Journal) {
    match s.dog.state {
        DogState::Companion => {
            s.dog.fed -= 40;
            if s.dog.fed <= 0 {
                s.dog.fed = 0;
                s.dog.days_hungry += 1;
            } else {
                s.dog.days_hungry = 0;
            }
            s.dog.health = (s.dog.health + 8).min(100);
            if s.dog.days_hungry >= 2 {
                journal::write(
                    s,
                    j,
                    Tone::Danger,
                    "The dog has gone. It waited for food as long as it could.",
                );
                lose(s, j, false);
            } else if s.dog.days_hungry == 1 {
                journal::write(s, j, Tone::Danger, "The dog is hungry. It watches you and the store.");
            }
        }
        DogState::Following => {
            // A fed stray waits a day or two for the next meal.
            s.dog.fed -= 50;
            if s.dog.fed <= -50 {
                lose(s, j, false);
            }
        }
        _ => {}
    }
}

pub fn feed(s: &mut GameState, j: &mut Journal) {
    if inventory::take_meals(s, j, 1) == 1 {
        s.dog.fed = 100;
        s.dog.days_hungry = 0;
        journal::plain(s, j, "You feed the dog. It eats fast, then leans against your leg.");
    }
}
