//! Same seed + same actions = the same game, byte for byte.

mod common;

use all_alone_core::content::Difficulty;
use all_alone_core::rng::Rng;
use all_alone_core::save::save_to_string;
use common::*;

#[test]
fn same_seed_same_actions_same_state() {
    for seed in [1u32, 42, 9_999] {
        let a = play(&config(seed, Difficulty::Still), heuristic, 3_000);
        let b = play(&config(seed, Difficulty::Still), heuristic, 3_000);
        assert_eq!(save_to_string(&a.state), save_to_string(&b.state));
    }
}

#[test]
fn random_play_is_reproducible() {
    let run = |seed: u32| {
        let mut rng = Rng::from_seed(seed ^ 0xabcd);
        play(
            &config(seed, Difficulty::DeadOfWinter),
            |s| careless(s, &mut rng),
            2_000,
        )
        .state
    };
    assert_eq!(run(7), run(7));
    assert_ne!(save_to_string(&run(7)), save_to_string(&run(8)));
}

#[test]
fn replaying_the_action_log_reproduces_the_state() {
    let cfg = config(123, Difficulty::Quiet);
    let mut s = all_alone_core::new_game(&cfg).unwrap();
    let mut log = Vec::new();
    while !s.ended() && log.len() < 2_500 {
        let a = heuristic(&s);
        all_alone_core::dispatch(&mut s, &a).unwrap();
        log.push(serde_json::to_string(&a).unwrap());
    }
    let mut t = all_alone_core::new_game(&cfg).unwrap();
    for json in &log {
        let a: all_alone_core::Action = serde_json::from_str(json).unwrap();
        all_alone_core::dispatch(&mut t, &a).unwrap();
    }
    assert_eq!(s, t);
}

#[test]
fn saving_mid_run_and_continuing_matches_an_unbroken_run() {
    let cfg = config(31, Difficulty::Still);
    let mut s = all_alone_core::new_game(&cfg).unwrap();
    let mut t = s.clone();
    for i in 0..1_500 {
        if s.ended() {
            break;
        }
        let a = heuristic(&s);
        all_alone_core::dispatch(&mut s, &a).unwrap();
        all_alone_core::dispatch(&mut t, &a).unwrap();
        if i % 97 == 0 {
            t = all_alone_core::save::load_from_string(&save_to_string(&t)).unwrap();
        }
    }
    assert_eq!(s, t);
}
