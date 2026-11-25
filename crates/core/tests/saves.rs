//! Saves are untrusted: every corrupted or hostile save must be rejected
//! cleanly, never panic, never half-load.

mod common;

use all_alone_core::content::Difficulty;
use all_alone_core::meta::Profile;
use all_alone_core::rng::Rng;
use all_alone_core::save::*;
use serde_json::Value;

fn mid_run() -> String {
    let r = common::play(&common::config(77, Difficulty::Still), common::heuristic, 300);
    assert!(!r.state.ended(), "the sample run should still be going");
    save_to_string(&r.state)
}

#[track_caller]
fn rejects(text: &str) {
    assert!(
        load_from_string(text).is_err(),
        "accepted: {}",
        &text[..text.len().min(200)]
    );
}

#[test]
fn round_trip_mid_run() {
    let text = mid_run();
    let s = load_from_string(&text).unwrap();
    assert_eq!(save_to_string(&s), text);
}

#[test]
fn truncated_saves_are_rejected() {
    let text = mid_run();
    let mut cut = text.len();
    while cut > 0 {
        cut = cut * 9 / 10;
        if text.is_char_boundary(cut) {
            rejects(&text[..cut]);
        }
    }
}

#[test]
fn huge_and_garbage_inputs_are_rejected() {
    rejects(&"x".repeat(MAX_RUN_BYTES + 1));
    rejects(&format!(
        "{{\"format\":\"all-alone/run\",\"version\":1,\"data\":{}}}",
        "[".repeat(5_000)
    ));
    for g in [
        "",
        "null",
        "[]",
        "{}",
        "\"save\"",
        "{\"format\":\"all-alone/run\"}",
        "\u{feff}{}",
        "{\"format\":\"all-alone/run\",\"version\":1,\"data\":{},\"extra\":1}",
    ] {
        rejects(g);
    }
}

/// Apply `f` to the parsed save and expect rejection.
#[track_caller]
fn mutate(text: &str, f: impl Fn(&mut Value)) {
    let mut v: Value = serde_json::from_str(text).unwrap();
    f(&mut v);
    rejects(&serde_json::to_string(&v).unwrap());
}

#[test]
fn hostile_values_are_rejected() {
    let t = mid_run();
    mutate(&t, |v| v["version"] = 99.into());
    mutate(&t, |v| v["format"] = "all-alone/profile".into());
    mutate(&t, |v| v["data"]["rng"] = serde_json::json!([0, 0, 0, 0]));
    mutate(&t, |v| v["data"]["rng"] = serde_json::json!([1, 2, 3]));
    mutate(&t, |v| v["data"]["vitals"]["health"] = 100_001.into());
    mutate(&t, |v| v["data"]["vitals"]["nerve"] = (-1).into());
    mutate(&t, |v| v["data"]["vitals"]["rest"] = "full".into());
    mutate(&t, |v| v["data"]["noise"] = 101.into());
    mutate(&t, |v| v["data"]["minute"] = 4_000_000_000u64.into());
    mutate(&t, |v| v["data"]["minute"] = 1.5.into());
    mutate(&t, |v| v["data"]["store"]["plutonium"] = 1.into());
    mutate(&t, |v| v["data"]["store"]["candle"] = 0.into());
    mutate(&t, |v| v["data"]["store"]["candle"] = 1_000_000.into());
    mutate(&t, |v| v["data"]["background"] = "astronaut".into());
    mutate(&t, |v| v["data"]["stations"]["stove"] = 4.into());
    mutate(&t, |v| v["data"]["stations"]["sauna"] = 1.into());
    mutate(&t, |v| v["data"]["codex"] = serde_json::json!(["note_nowhere"]));
    mutate(&t, |v| v["data"]["marks"] = serde_json::json!(["mark_nowhere"]));
    mutate(&t, |v| v["data"]["phase"] = "flying".into());
    mutate(&t, |v| v["data"]["phase"] = "encounter".into());
    mutate(&t, |v| v["data"]["phase"] = "ended".into());
    mutate(&t, |v| {
        v["data"]["places"]["atlantis"] = v["data"]["places"]["weir"].clone()
    });
    mutate(&t, |v| {
        v["data"]["places"].as_object_mut().unwrap().remove("weir");
    });
    mutate(&t, |v| v["data"]["places"]["weir"]["rooms"] = serde_json::json!([]));
    mutate(&t, |v| v["data"]["bleeding"] = 9.into());
    mutate(&t, |v| v["data"]["harm"] = "boredom".into());
    mutate(&t, |v| v["data"]["freeze_day"] = 3.into());
    mutate(&t, |v| v["data"]["flags"] = serde_json::json!(["Robert'); DROP TABLE"]));
    mutate(&t, |v| v["data"]["scheduled"] = serde_json::json!([["nope", 3]]));
    mutate(&t, |v| v["data"]["unknown_field"] = true.into());
    mutate(
        &t,
        |v| {
            v["data"]["journal"] =
                serde_json::json!([{"day": 1, "time": "07:00", "text": "x", "tone": "plain", "evil": 1}])
        },
    );
    mutate(
        &t,
        |v| {
            v["data"]["trip"] =
                serde_json::json!({"place": "weir", "room": 99, "dog": false, "departed": 0, "site_noise": 0})
        },
    );
    mutate(&t, |v| v["data"]["worn"] = serde_json::json!({"coat": "hammer"}));
}

#[test]
fn fuzzed_saves_never_panic() {
    let t = mid_run();
    let bytes = t.as_bytes();
    let mut rng = Rng::from_seed(2024);
    let mut accepted = 0;
    for _ in 0..3_000 {
        let mut b = bytes.to_vec();
        for _ in 0..1 + rng.below(4) {
            let i = rng.below(b.len() as u32) as usize;
            b[i] = match rng.below(4) {
                0 => b'9',
                1 => b'"',
                2 => b'}',
                _ => rng.below(128) as u8,
            };
        }
        if let Ok(text) = String::from_utf8(b)
            && load_from_string(&text).is_ok()
        {
            accepted += 1;
        }
    }
    // Some single-digit edits are still valid saves; most are not.
    assert!(accepted < 1_500, "{accepted} fuzzed saves accepted");
}

#[test]
fn profiles_are_validated_too() {
    let text = profile_to_string(&Profile::new());
    assert_eq!(profile_from_string(&text).unwrap(), Profile::new());
    let bad = |f: &dyn Fn(&mut Value)| {
        let mut v: Value = serde_json::from_str(&text).unwrap();
        f(&mut v);
        assert!(profile_from_string(&serde_json::to_string(&v).unwrap()).is_err());
    };
    bad(&|v| v["data"]["marks"] = serde_json::json!({"nope": 1}));
    bad(&|v| v["data"]["backgrounds"] = serde_json::json!([]));
    bad(&|v| v["data"]["remnants"] = 5.into());
    bad(&|v| v["data"]["keepsakes"] = serde_json::json!(["crown"]));
    bad(&|v| v["data"]["daily"]["attempted"] = serde_json::json!(["yesterday"]));
    bad(&|v| v["data"]["extra"] = 1.into());
    bad(&|v| v["version"] = 7.into());
    assert!(profile_from_string(&"{".repeat(MAX_PROFILE_BYTES + 2)).is_err());
}

// ---------------------------------------------------------------- Older versions

const RUN_V1_LIVE: &str = include_str!("fixtures/run_v1_live.json");
const RUN_V1_ENDED: &str = include_str!("fixtures/run_v1_ended.json");
const PROFILE_V1: &str = include_str!("fixtures/profile_v1.json");

/// Saves written by version 1 (milestone M3) still load, and play on.
#[test]
fn version_1_saves_load() {
    assert!(RUN_V1_LIVE.contains("\"version\":1"));
    let mut s = load_from_string(RUN_V1_LIVE).expect("v1 run in progress loads");
    assert!(!s.ended());
    // It re-saves at the current version and loads again unchanged.
    let again = save_to_string(&s);
    assert!(again.contains(&format!("\"version\":{RUN_VERSION}")));
    assert_eq!(load_from_string(&again).unwrap(), s);
    // And it can be played on.
    for _ in 0..50 {
        if s.ended() {
            break;
        }
        let a = common::heuristic(&s);
        all_alone_core::dispatch(&mut s, &a).unwrap();
    }

    let e = load_from_string(RUN_V1_ENDED).expect("v1 ended run loads");
    let l = e.ledger.as_ref().unwrap();
    assert_eq!(l.places_all as usize, all_alone_core::content::places::PLACES.len());
    assert_eq!(
        l.places_visited as usize,
        e.places.values().filter(|p| p.visited).count(),
        "the migrated ledger counts the places the run reached"
    );
    assert!(l.places_visited > 0);

    let p = profile_from_string(PROFILE_V1).expect("v1 profile loads");
    assert_eq!(p.best.len(), 1);
    assert_eq!(p.best[0].places_all, 0, "unknown for old ledgers");
    let back = profile_to_string(&p);
    assert!(back.contains(&format!("\"version\":{PROFILE_VERSION}")));
    assert_eq!(profile_from_string(&back).unwrap(), p);
    // The old profile still takes a new run's ledger.
    let mut p = p;
    let mut fresh = load_from_string(RUN_V1_LIVE).unwrap();
    let mut j = all_alone_core::Outcome::default();
    all_alone_core::systems::forfeit(&mut fresh, &mut j);
    all_alone_core::meta::apply_run_end(&mut p, fresh.ledger.as_ref().unwrap()).unwrap();
    assert_eq!(p.totals.runs, 2);
}

/// A version 1 run that ended after its ledger was entered in a version 1
/// profile (the game closed before the run save was removed) is not counted
/// again once both are migrated: the run's fingerprint survives migration.
#[test]
fn migrated_ended_run_is_not_counted_twice() {
    let mut p = profile_from_string(PROFILE_V1).unwrap();
    let e = load_from_string(RUN_V1_ENDED).unwrap();
    let l = e.ledger.as_ref().unwrap();
    let before = p.totals.clone();
    let err = all_alone_core::meta::apply_run_end(&mut p, l).expect_err("already in the profile");
    assert_eq!(err.code, "already_applied");
    assert_eq!(p.totals, before);

    // The same holds once the profile has been re-saved at the current version.
    let mut p = profile_from_string(&profile_to_string(&p)).unwrap();
    assert_eq!(
        all_alone_core::meta::apply_run_end(&mut p, l).unwrap_err().code,
        "already_applied"
    );

    // And a run applied by this build is recognised after its save round-trips.
    let mut fresh = load_from_string(RUN_V1_LIVE).unwrap();
    let mut j = all_alone_core::Outcome::default();
    all_alone_core::systems::forfeit(&mut fresh, &mut j);
    let mut p2 = profile_from_string(PROFILE_V1).unwrap();
    all_alone_core::meta::apply_run_end(&mut p2, fresh.ledger.as_ref().unwrap()).unwrap();
    let reloaded = load_from_string(&save_to_string(&fresh)).unwrap();
    let mut p2 = profile_from_string(&profile_to_string(&p2)).unwrap();
    assert_eq!(
        all_alone_core::meta::apply_run_end(&mut p2, reloaded.ledger.as_ref().unwrap())
            .unwrap_err()
            .code,
        "already_applied"
    );
}
