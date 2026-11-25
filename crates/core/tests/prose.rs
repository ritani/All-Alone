//! The journal's prose: plurals agree, names are not repeated, causes read.

mod common;

use all_alone_core::content::{Difficulty, stations};
use all_alone_core::systems::home::build_line;
use all_alone_core::{dispatch, new_game, view};

/// "1 measures", "1 days": a one followed by a plural noun.
fn bad_plural(text: &str) -> Option<String> {
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .collect();
    for pair in words.windows(2) {
        let (n, w) = (pair[0], pair[1]);
        if n == "1" && w.ends_with('s') && !w.ends_with("ss") && w.len() > 3 {
            return Some(format!("{n} {w}"));
        }
    }
    None
}

#[test]
fn journal_plurals_agree_in_play() {
    for seed in 0..12 {
        let d = [Difficulty::Still, Difficulty::Quiet, Difficulty::DeadOfWinter][seed as usize % 3];
        let mut s = new_game(&common::config(2000 + seed, d)).unwrap();
        let mut steps = 0;
        while !s.ended() && steps < 20_000 {
            let a = common::heuristic(&s);
            let out = dispatch(&mut s, &a).unwrap();
            for e in &out.journal {
                assert!(
                    bad_plural(&e.text).is_none(),
                    "seed {seed}: {:?} in {:?}",
                    bad_plural(&e.text),
                    e.text
                );
            }
            if steps % 50 == 0 {
                let v = view(&s);
                for a in &v.actions {
                    for t in [Some(&a.label), a.reason.as_ref()].into_iter().flatten() {
                        assert!(bad_plural(t).is_none(), "action text {t:?}");
                    }
                }
                for o in &v.objectives {
                    assert!(bad_plural(&o.text).is_none(), "objective {:?}", o.text);
                }
                for i in v.inventory.store.iter().chain(&v.inventory.pack) {
                    assert!(bad_plural(&i.detail).is_none(), "item {:?}", i.detail);
                }
            }
            steps += 1;
        }
    }
}

#[test]
fn build_lines_do_not_repeat_the_name() {
    for st in stations::STATIONS.iter() {
        for lv in &st.levels[1..] {
            let line = build_line(st.name, lv.name, lv.effect);
            let lower = line.to_lowercase();
            let name = st.name.to_lowercase();
            assert!(lower.matches(&name).count() <= 1, "repeated name: {line}");
        }
    }
    assert_eq!(
        build_line("Workbench", "Workbench", "Snares."),
        "Workbench built. Snares."
    );
    assert_eq!(build_line("Stove", "Iron stove", "Warm."), "Iron stove built. Warm.");
    assert_eq!(build_line("Bed", "Cot", "Fair sleep."), "Bed: Cot. Fair sleep.");
}

#[test]
fn the_checker_catches_bad_plurals() {
    assert!(bad_plural("You fill 1 measures from the lock.").is_some());
    assert!(bad_plural("You fill 1 measure from the lock.").is_none());
    assert!(bad_plural("You fill 2 measures.").is_none());
}
