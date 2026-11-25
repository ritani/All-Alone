//! Headless bot playthroughs. The heuristic bot must be able to win on Still
//! for a fair share of seeds; the careless bot must die. Run with
//! `cargo test --test bots -- --nocapture` to see the balance report.

mod common;

use all_alone_core::content::Difficulty;
use all_alone_core::rng::Rng;
use common::*;

const SEEDS: u32 = 60;
const MAX_STEPS: u32 = 40_000;

fn report(d: Difficulty) -> (f64, u32, Vec<String>) {
    let mut wins = 0;
    let mut days = Vec::new();
    let mut causes = Vec::new();
    for seed in 0..SEEDS {
        let r = play(&config(1000 + seed, d), heuristic, MAX_STEPS);
        assert!(r.state.ended() || r.steps == MAX_STEPS);
        if r.won {
            wins += 1;
        }
        days.push(r.days);
        causes.push(format!("{} (day {}, parts {})", r.cause, r.days, r.parts));
    }
    (f64::from(wins) / f64::from(SEEDS), median(days), causes)
}

#[test]
fn heuristic_bot_can_win_on_still() {
    let (rate, med, causes) = report(Difficulty::Still);
    println!("Still: heuristic win rate {:.0}%, median day {med}", rate * 100.0);
    for c in causes.iter().take(SEEDS as usize) {
        println!("  {c}");
    }
    assert!(rate >= 0.25, "win rate too low: {rate}");
    assert!(rate <= 0.9, "win rate too high: {rate}");
}

#[test]
fn difficulty_orders_the_win_rate() {
    let (q, qm, _) = report(Difficulty::Quiet);
    let (w, wm, _) = report(Difficulty::DeadOfWinter);
    println!("Quiet: heuristic win rate {:.0}%, median day {qm}", q * 100.0);
    println!("Dead of Winter: heuristic win rate {:.0}%, median day {wm}", w * 100.0);
    assert!(q >= w, "Quiet {q} should not be harder than Dead of Winter {w}");
}

#[test]
fn careless_bot_dies() {
    let mut deaths = 0;
    let mut days = Vec::new();
    for seed in 0..SEEDS {
        let mut rng = Rng::from_seed(seed);
        let r = play(
            &config(5000 + seed, Difficulty::Still),
            |s| careless(s, &mut rng),
            MAX_STEPS,
        );
        if r.died {
            deaths += 1;
        }
        assert!(!r.won, "the careless bot should never win");
        days.push(r.days);
    }
    println!("Careless: {deaths}/{SEEDS} died, median day {}", median(days));
    assert!(deaths * 10 >= SEEDS * 9, "only {deaths} careless deaths");
}
