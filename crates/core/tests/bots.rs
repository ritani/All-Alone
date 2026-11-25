//! Headless bot playthroughs, used to tune pacing and balance.
//! `cargo test --test bots -- --nocapture` prints the report.
//!
//! Targets on Still: the heuristic bot wins about 30-50% of seeds, its median
//! winning day falls between 24 and 35, some runs are caught by the freeze;
//! Quiet is easier and Dead of Winter clearly harder. The careless bot dies.

mod common;

use all_alone_core::content::Difficulty;
use all_alone_core::rng::Rng;
use common::*;

const SEEDS: u32 = 60;
const MAX_STEPS: u32 = 60_000;

struct Report {
    rate: f64,
    median_days: u32,
    median_win_day: u32,
    frozen_out: u32,
}

fn report(d: Difficulty) -> Report {
    let mut wins = 0;
    let mut days = Vec::new();
    let mut win_days = Vec::new();
    let mut frozen_out = 0;
    for seed in 0..SEEDS {
        let r = play(&config(1000 + seed, d), heuristic, MAX_STEPS);
        assert!(r.state.ended() || r.steps == MAX_STEPS);
        if r.won {
            wins += 1;
            win_days.push(r.days);
        } else if r.state.stats.saw_freeze {
            frozen_out += 1;
        }
        days.push(r.days);
    }
    let rep = Report {
        rate: f64::from(wins) / f64::from(SEEDS),
        median_days: median(days),
        median_win_day: median(win_days),
        frozen_out,
    };
    println!(
        "{}: win rate {:.0}%, median winning day {}, median day reached {}, caught by the freeze {}/{SEEDS}",
        d.name(),
        rep.rate * 100.0,
        rep.median_win_day,
        rep.median_days,
        rep.frozen_out
    );
    rep
}

#[test]
fn pacing_and_difficulty() {
    let still = report(Difficulty::Still);
    let quiet = report(Difficulty::Quiet);
    let winter = report(Difficulty::DeadOfWinter);
    assert!((0.28..=0.55).contains(&still.rate), "Still win rate {}", still.rate);
    assert!(
        (24..=35).contains(&still.median_win_day),
        "Still median winning day {}",
        still.median_win_day
    );
    assert!(still.frozen_out > 0, "the freeze should catch some Still runs");
    assert!(
        quiet.rate > still.rate,
        "Quiet {} should be easier than Still {}",
        quiet.rate,
        still.rate
    );
    assert!(
        winter.rate < still.rate * 0.6,
        "Dead of Winter {} should be clearly harder than Still {}",
        winter.rate,
        still.rate
    );
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
