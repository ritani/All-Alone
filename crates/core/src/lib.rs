//! ALL ALONE: the game core.
//!
//! Pure, deterministic rules for a survival-horror game in Calder's Reach.
//! The UI starts a run with [`new_game`], reads [`view`], and changes the
//! game only through [`dispatch`]. Saves and the profile are strings produced
//! and validated here ([`save`]); marks, ledger, remnants and the daily river
//! are in [`meta`].

#![forbid(unsafe_code)]

pub mod action;
pub mod content;
pub mod meta;
pub mod rng;
pub mod save;
pub mod state;
pub mod systems;
pub mod view;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(all(test, feature = "ts"))]
mod ts_export;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

pub use action::{Action, GameEvent, Outcome, dispatch};
pub use meta::{Ledger, Profile};
pub use state::{GameConfig, GameState, new_game};
pub use view::{View, view};

/// Every fallible API returns this. `code` is stable and machine-readable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
pub struct GameError {
    pub code: String,
    pub message: String,
}

impl GameError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        GameError {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for GameError {}

/// The accumulator used while an action resolves.
pub type Journal = Outcome;
