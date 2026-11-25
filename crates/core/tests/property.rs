//! Property-style test: random allowed actions for thousands of steps across
//! many seeds never panic and keep every invariant. The view's action list is
//! checked against `dispatch` along the way.

mod common;

use all_alone_core::content::{Difficulty, VitalKind};
use all_alone_core::rng::Rng;
use all_alone_core::state::{GameState, MILLI};
use all_alone_core::systems::inventory;
use all_alone_core::{Action, dispatch, save, view};

fn invariants(s: &GameState) {
    for k in VitalKind::ALL {
        let v = s.vitals.get(*k);
        assert!((0..=100 * MILLI).contains(&v), "{k:?} = {v}");
    }
    assert!(s.noise <= 100);
    assert!(
        inventory::carried(s) <= inventory::capacity(s),
        "overloaded: {} > {}",
        inventory::carried(s),
        inventory::capacity(s)
    );
    assert!(s.store.values().chain(s.pack.values()).all(|n| *n > 0));
    if let Err(e) = save::validate(s) {
        panic!("state failed validation in {:?}: {e}", s.phase);
    }
}

fn random_walk(seed: u32, difficulty: Difficulty, steps: u32) -> GameState {
    let mut rng = Rng::from_seed(seed.wrapping_mul(2_654_435_761));
    let mut s = all_alone_core::new_game(&common::config(seed, difficulty)).unwrap();
    for i in 0..steps {
        if s.ended() {
            assert!(s.ledger.is_some());
            assert!(dispatch(&mut s, &Action::Rise).is_err());
            break;
        }
        let mostly_sensible = seed % 2 == 0 && i % 3 != 0;
        let a = if mostly_sensible || i % 7 == 0 {
            // Mostly sensible play keeps runs long enough to reach late phases.
            common::heuristic(&s)
        } else {
            let v = common::enabled(&s);
            assert!(!v.is_empty(), "no action available in {:?}", s.phase);
            v[rng.below(v.len() as u32) as usize].clone()
        };
        if i % 25 == 0 {
            let vw = view(&s);
            let json = serde_json::to_string(&vw).unwrap();
            assert!(json.len() < 400_000);
            for av in vw.actions.iter().filter(|a| a.enabled).take(3) {
                let mut t = s.clone();
                assert!(
                    dispatch(&mut t, &av.action).is_ok(),
                    "view said {:?} was allowed",
                    av.action
                );
            }
            for av in vw.actions.iter().filter(|a| !a.enabled).take(3) {
                let mut t = s.clone();
                assert!(
                    dispatch(&mut t, &av.action).is_err(),
                    "view said {:?} was not allowed",
                    av.action
                );
                assert_eq!(t, s);
                assert!(av.reason.is_some());
            }
        }
        if let Err(e) = dispatch(&mut s, &a) {
            panic!("enabled action {a:?} failed: {e}");
        }
        invariants(&s);
        if i % 200 == 0 {
            let back = save::load_from_string(&save::save_to_string(&s)).unwrap();
            assert_eq!(back, s);
        }
    }
    s
}

#[test]
fn random_play_keeps_invariants() {
    let mut ended = 0;
    let mut longest = 0;
    for seed in 0..80u32 {
        let d = Difficulty::ALL[(seed % 3) as usize];
        let s = random_walk(seed, d, 5_000);
        longest = longest.max(s.day());
        if s.ended() {
            ended += 1;
        }
    }
    assert!(ended > 0, "some random runs should end");
    assert!(
        longest >= 10,
        "random play should sometimes last a while (longest {longest})"
    );
}

#[test]
fn hostile_actions_are_rejected_without_panic() {
    let mut s = all_alone_core::new_game(&common::config(5, Difficulty::Still)).unwrap();
    let before = s.clone();
    let nasty = [
        r#"{"type":"pack","item":"","qty":4294967295}"#,
        r#"{"type":"take","item":"fuel_drum","qty":1}"#,
        r#"{"type":"choose","option":4294967295}"#,
        r#"{"type":"craft","recipe":"../../etc"}"#,
        r#"{"type":"build","station":"radio"}"#,
        r#"{"type":"use","item":"🜁"}"#,
        r#"{"type":"install_part","part":"hammer"}"#,
        r#"{"type":"plan_trip","place":"wharf"}"#,
        r#"{"type":"buy","item":"antibiotics"}"#,
        r#"{"type":"sell","item":"hammer","qty":1}"#,
        r#"{"type":"throw","item":"glass_bottle"}"#,
        r#"{"type":"plant","seed":"hammer"}"#,
    ];
    for n in nasty {
        let a: Action = serde_json::from_str(n).unwrap();
        assert!(dispatch(&mut s, &a).is_err(), "{n}");
        assert_eq!(s, before);
    }
    for bad in [
        r#"{"type":"pack","item":"x","qty":-1}"#,
        r#"{"type":"pack"}"#,
        r#"[]"#,
        r#"{"type":1}"#,
        "null",
    ] {
        assert!(serde_json::from_str::<Action>(bad).is_err(), "{bad}");
    }
}
