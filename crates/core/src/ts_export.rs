//! Generates `src/core/types.ts` from the Rust serde types, so the TypeScript
//! view can never drift from the core. Run with the `ts` feature:
//!
//! - `cargo test --features ts ts_types` fails if the file is out of date;
//! - `UPDATE_TYPES=1 cargo test --features ts ts_types` rewrites it.

use ts_rs::{Config, TS};

macro_rules! decls {
    ($cfg:expr; $($t:ty),* $(,)?) => {
        vec![$(format!("export {}", <$t as TS>::decl($cfg))),*]
    };
}

fn generate() -> String {
    use crate::GameError;
    use crate::action::*;
    use crate::content::codex::CodexKind;
    use crate::content::*;
    use crate::meta::*;
    use crate::state::*;
    use crate::systems::objectives::Objective;
    use crate::view::*;
    let cfg = &Config::new();
    let mut out = String::from(
        "// Generated from the Rust core by `UPDATE_TYPES=1 cargo test --features ts ts_types`.\n// Do not edit by hand.\n/* eslint-disable */\n\n",
    );
    let all = decls!(cfg;
        GameConfig, GameError, Action, GameEvent, Outcome, Entry, Tone, Phase, EndKind, DogState, SearchState,
        VitalKind, Skill, Station, Weather, Creature, Band, Difficulty, CodexKind, DawnSummary, Objective,
        View, ActionView, VitalView, ConditionView, WeatherInfo, WeatherView, SeasonView, NoiseView, ItemView,
        InventoryView, CostView, NextLevelView, StationView, RecipeView, MapNode, MapView, CodexEntryView,
        CodexSection, CodexView, SkillView, DogView, FerryPartView, FerryView, PlotView, GroundsView, FoeView,
        RoomSummary, RoomView, SiteView, EncounterView, OptionView, EventView, TravelView, PackingView, OfferView,
        SellView, BarterView, EndingView, RunInfo,
        Ledger, ScoreLine, Totals, DailyResult, DailyRecord, Profile, RunEndReport, Unlock, DailySetup,
        MarkView, BackgroundView, KeepsakeView, DifficultyView, ProfileView, DailyStatus, MarkEarned, NeedView,
    );
    for d in all {
        out.push_str(&d);
        out.push_str("\n\n");
    }
    out.trim_end().to_string() + "\n"
}

#[test]
fn ts_types() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/core/types.ts");
    let fresh = generate();
    if std::env::var_os("UPDATE_TYPES").is_some() {
        std::fs::write(&path, &fresh).expect("write types.ts");
        return;
    }
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == fresh,
        "src/core/types.ts is out of date: run `UPDATE_TYPES=1 cargo test --features ts ts_types` (npm run types)"
    );
}
