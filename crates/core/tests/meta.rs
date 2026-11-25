//! Marks unlock, the ledger and score are computed, remnants and profile work.

mod common;

use all_alone_core::content::Difficulty;
use all_alone_core::meta::{self, Profile, Unlock};
use all_alone_core::state::EndKind;
use all_alone_core::{Action, GameEvent, dispatch};
use common::*;

#[test]
fn first_night_mark_is_announced_once() {
    let mut s = all_alone_core::new_game(&config(3, Difficulty::Quiet)).unwrap();
    let mut seen = 0;
    while s.stats.nights < 2 && !s.ended() {
        let a = heuristic(&s);
        let out = dispatch(&mut s, &a).unwrap();
        seen += out
            .events
            .iter()
            .filter(|e| matches!(e, GameEvent::Mark { id, new: true } if id == "first_night"))
            .count();
    }
    assert_eq!(seen, 1);
    assert!(s.marks.contains("first_night"));
}

#[test]
fn held_marks_are_not_new() {
    let mut cfg = config(3, Difficulty::Quiet);
    cfg.held_marks = vec!["first_night".into()];
    let mut s = all_alone_core::new_game(&cfg).unwrap();
    while s.stats.nights < 1 && !s.ended() {
        let a = heuristic(&s);
        let out = dispatch(&mut s, &a).unwrap();
        assert!(
            !out.events
                .iter()
                .any(|e| matches!(e, GameEvent::Mark { id, new: true } if id == "first_night"))
        );
    }
}

#[test]
fn a_win_earns_its_marks_and_a_good_ledger() {
    let r = (1000..1060)
        .map(|seed| play(&config(seed, Difficulty::Still), heuristic, 40_000))
        .find(|r| r.won)
        .expect("some seed wins");
    let s = &r.state;
    assert!(s.marks.contains("cast_off") && s.marks.contains("cast_off_still"));
    let l = s.ledger.clone().unwrap();
    assert_eq!(l.end, EndKind::Departure);
    assert_eq!(l.parts, 4);
    assert!(l.score >= 400, "score {}", l.score);
    assert_eq!(l.remnants, l.score / meta::REMNANT_RATE);
    assert_eq!(l.score, l.lines.iter().map(|x| x.points).sum::<u32>());
    assert!(dispatch(&mut s.clone(), &Action::Rise).is_err());
}

#[test]
fn dying_on_day_one_is_a_hidden_mark() {
    let mut rng = all_alone_core::rng::Rng::from_seed(1);
    let r = (0..200u32)
        .map(|seed| {
            play(
                &config(seed, Difficulty::DeadOfWinter),
                |s| careless(s, &mut rng),
                5_000,
            )
        })
        .find(|r| r.died && r.days == 1)
        .expect("a careless day-one death");
    assert!(r.state.marks.contains("day_one"));
    let l = r.state.ledger.unwrap();
    assert_eq!(l.end, EndKind::Death);
    assert!(!l.cause.is_empty());
}

#[test]
fn profile_folds_runs_and_unlocks() {
    let mut p = Profile::new();
    assert!(p.backgrounds.contains("clerk") && !p.backgrounds.contains("nurse"));
    let r = (1000..1060)
        .map(|seed| play(&config(seed, Difficulty::Still), heuristic, 40_000))
        .find(|r| r.won)
        .unwrap();
    let l = r.state.ledger.clone().unwrap();
    let rep = meta::apply_run_end(&mut p, &l).unwrap();
    assert!(rep.new_marks.contains(&"cast_off".to_string()));
    assert_eq!(rep.rank, Some(1));
    assert_eq!(p.remnants, l.remnants);
    assert_eq!(p.totals.runs, 1);
    assert_eq!(p.totals.departures, 1);
    assert!(p.marks.contains_key("week") == l.marks.contains(&"week".into()));
    // Applying the same ledger twice is refused.
    assert_eq!(meta::apply_run_end(&mut p, &l).unwrap_err().code, "already_applied");
    // Marks that unlock backgrounds unlock them.
    if p.marks.contains_key("week") {
        assert!(p.backgrounds.contains("warden"));
    }
    // Remnants buy keepsakes.
    p.remnants = 100;
    p.remnants_earned = p.remnants_earned.max(100);
    meta::unlock(
        &mut p,
        &Unlock::Keepsake {
            id: "valley_map".into(),
        },
    )
    .unwrap();
    assert_eq!(p.remnants, 60);
    let cfg = meta::run_config(&p, 9, Difficulty::Still, "clerk", Some("valley_map")).unwrap();
    assert!(cfg.held_marks.contains(&"cast_off".to_string()));
    let s = all_alone_core::new_game(&cfg).unwrap();
    assert!(s.places["boatyard"].known);
    // Profile survives a save round trip and validates.
    let text = all_alone_core::save::profile_to_string(&p);
    assert_eq!(all_alone_core::save::profile_from_string(&text).unwrap(), p);
    let v = meta::profile_view(&p);
    assert_eq!(v.marks_total as usize, all_alone_core::content::marks::MARKS.len());
    assert!(v.marks.iter().any(|m| m.hidden && !m.earned && m.name == "???"));
}

#[test]
fn ten_runs_and_lifetime_marks() {
    let mut p = Profile::new();
    let mut rng = all_alone_core::rng::Rng::from_seed(4);
    for seed in 0..10u32 {
        let r = play(&config(300 + seed, Difficulty::Still), |s| careless(s, &mut rng), 5_000);
        let l = r.state.ledger.unwrap();
        meta::apply_run_end(&mut p, &l).unwrap();
    }
    assert_eq!(p.totals.runs, 10);
    assert!(p.marks.contains_key("ten_runs"));
    assert!(p.best.len() <= meta::BEST_KEPT);
    assert!(p.best.windows(2).all(|w| w[0].score >= w[1].score));
}

#[test]
fn daily_river_is_fixed_and_counts_once() {
    let mut p = Profile::new();
    let (c, counts) = meta::daily_config(&mut p, "2026-11-25").unwrap();
    assert!(counts);
    assert_eq!(c.seed, meta::daily_setup("2026-11-25").unwrap().seed);
    let a = all_alone_core::new_game(&c).unwrap();
    let b = all_alone_core::new_game(&c).unwrap();
    assert_eq!(a, b);
    let mut rng = all_alone_core::rng::Rng::from_seed(9);
    let r = play(&c, |s| careless(s, &mut rng), 5_000);
    let l = r.state.ledger.unwrap();
    assert!(r.state.marks.contains("daily"));
    let rep = meta::apply_run_end(&mut p, &l).unwrap();
    assert!(rep.daily_counted);
    assert_eq!(p.daily.results.len(), 1);
    let (_, again) = meta::daily_config(&mut p, "2026-11-25").unwrap();
    assert!(!again);
}

#[test]
fn ledgers_from_outside_are_validated() {
    let mut p = Profile::new();
    let mut rng = all_alone_core::rng::Rng::from_seed(5);
    let r = play(&config(1, Difficulty::Still), |s| careless(s, &mut rng), 5_000);
    let good = r.state.ledger.unwrap();
    let mut l = good.clone();
    l.score += 1_000;
    assert!(meta::apply_run_end(&mut p, &l).is_err());
    let mut l = good.clone();
    l.marks.push("nope".into());
    assert!(meta::apply_run_end(&mut p, &l).is_err());
    let mut l = good;
    l.background = "astronaut".into();
    assert!(meta::apply_run_end(&mut p, &l).is_err());
    assert_eq!(p, Profile::new());
}
