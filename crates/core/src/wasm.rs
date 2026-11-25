//! The WebAssembly bridge: a small JSON-string API over the core.
//!
//! Every function takes and returns strings. Errors are thrown as JS `Error`s
//! whose message is a JSON `GameError` (`{"code": "...", "message": "..."}`).
//! The run save and profile are opaque strings the view stores and hands back.

use wasm_bindgen::prelude::*;

use crate::meta::{self, Unlock};
use crate::state::{GameConfig, GameState};
use crate::{Action, GameError, save};

const MAX_ACTION_BYTES: usize = 4 * 1024;
const MAX_CONFIG_BYTES: usize = 16 * 1024;
const MAX_LEDGER_BYTES: usize = 64 * 1024;

fn js(e: GameError) -> JsError {
    JsError::new(&serde_json::to_string(&e).unwrap_or_else(|_| "{\"code\":\"error\",\"message\":\"\"}".into()))
}

fn parse<T: serde::de::DeserializeOwned>(text: &str, max: usize, code: &str) -> Result<T, JsError> {
    if text.len() > max {
        return Err(js(GameError::new(code, "Input too large.")));
    }
    serde_json::from_str(text).map_err(|e| js(GameError::new(code, e.to_string())))
}

fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// One run.
#[wasm_bindgen]
pub struct Game {
    state: GameState,
}

#[wasm_bindgen]
impl Game {
    /// Start a new run from a `GameConfig` JSON.
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<Game, JsError> {
        let config: GameConfig = parse(config_json, MAX_CONFIG_BYTES, "bad_config")?;
        let state = crate::new_game(&config).map_err(js)?;
        Ok(Game { state })
    }

    /// Resume a run from a save string produced by `save()`.
    pub fn load(save_text: &str) -> Result<Game, JsError> {
        let state = save::load_from_string(save_text).map_err(js)?;
        Ok(Game { state })
    }

    /// Run one `Action` (JSON). Returns the `Outcome` (JSON).
    pub fn dispatch(&mut self, action_json: &str) -> Result<String, JsError> {
        let action: Action = parse(action_json, MAX_ACTION_BYTES, "bad_action")?;
        let out = crate::dispatch(&mut self.state, &action).map_err(js)?;
        Ok(to_json(&out))
    }

    /// The whole `View` (JSON).
    pub fn view(&self) -> String {
        to_json(&crate::view(&self.state))
    }

    /// The save string for this run.
    pub fn save(&self) -> String {
        save::save_to_string(&self.state)
    }

    /// The run's `Ledger` (JSON) once it has ended, else an empty string.
    pub fn ledger(&self) -> String {
        self.state.ledger.as_ref().map(to_json).unwrap_or_default()
    }

    pub fn ended(&self) -> bool {
        self.state.ended()
    }
}

/// A fresh profile's save string.
#[wasm_bindgen]
pub fn profile_new() -> String {
    save::profile_to_string(&meta::Profile::new())
}

/// Validate a profile save; returns it normalised.
#[wasm_bindgen]
pub fn profile_check(profile: &str) -> Result<String, JsError> {
    let p = save::profile_from_string(profile).map_err(js)?;
    Ok(save::profile_to_string(&p))
}

/// The `ProfileView` (JSON) for a profile save.
#[wasm_bindgen]
pub fn profile_view(profile: &str) -> Result<String, JsError> {
    let p = save::profile_from_string(profile).map_err(js)?;
    Ok(to_json(&meta::profile_view(&p)))
}

/// Fold a ledger into the profile. Returns `{"profile": <save>, "report": RunEndReport}`.
#[wasm_bindgen]
pub fn profile_apply_run_end(profile: &str, ledger_json: &str) -> Result<String, JsError> {
    let mut p = save::profile_from_string(profile).map_err(js)?;
    let l: meta::Ledger = parse(ledger_json, MAX_LEDGER_BYTES, "bad_ledger")?;
    let report = meta::apply_run_end(&mut p, &l).map_err(js)?;
    Ok(to_json(
        &serde_json::json!({ "profile": save::profile_to_string(&p), "report": report }),
    ))
}

/// Spend remnants on an `Unlock` (JSON). Returns the new profile save.
#[wasm_bindgen]
pub fn profile_unlock(profile: &str, unlock_json: &str) -> Result<String, JsError> {
    let mut p = save::profile_from_string(profile).map_err(js)?;
    let u: Unlock = parse(unlock_json, MAX_ACTION_BYTES, "bad_unlock")?;
    meta::unlock(&mut p, &u).map_err(js)?;
    Ok(save::profile_to_string(&p))
}

/// A checked `GameConfig` (JSON) for a normal run. `difficulty` is "quiet", "still" or "dead_of_winter".
#[wasm_bindgen]
pub fn profile_run_config(
    profile: &str,
    seed: u32,
    difficulty: &str,
    background: &str,
    keepsake: Option<String>,
) -> Result<String, JsError> {
    let p = save::profile_from_string(profile).map_err(js)?;
    let d = parse(&format!("\"{}\"", difficulty.escape_default()), 64, "bad_config")?;
    let c = meta::run_config(&p, seed, d, background, keepsake.as_deref()).map_err(js)?;
    Ok(to_json(&c))
}

/// Start the daily river. Returns `{"profile": <save>, "config": GameConfig, "counts": bool}`.
#[wasm_bindgen]
pub fn profile_daily(profile: &str, date: &str) -> Result<String, JsError> {
    let mut p = save::profile_from_string(profile).map_err(js)?;
    let (config, counts) = meta::daily_config(&mut p, date).map_err(js)?;
    Ok(to_json(
        &serde_json::json!({ "profile": save::profile_to_string(&p), "config": config, "counts": counts }),
    ))
}

/// The daily river's `DailySetup` for a date (JSON), without touching a profile.
#[wasm_bindgen]
pub fn daily_setup(date: &str) -> Result<String, JsError> {
    Ok(to_json(&meta::daily_setup(date).map_err(js)?))
}

/// The opening lines for a new run.
#[wasm_bindgen]
pub fn intro() -> String {
    to_json(&crate::content::prose::INTRO)
}

#[wasm_bindgen]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
