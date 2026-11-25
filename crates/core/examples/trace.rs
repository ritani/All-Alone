//! Print one heuristic-bot run's journal: `cargo run --example trace -- <seed> [still|quiet|dead_of_winter]`.

#[path = "../tests/common/mod.rs"]
mod common;

use all_alone_core::content::Difficulty;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1000);
    let d = match args.next().as_deref() {
        Some("quiet") => Difficulty::Quiet,
        Some("dead_of_winter") => Difficulty::DeadOfWinter,
        _ => Difficulty::Still,
    };
    let verbose = args.next().is_some();
    let cfg = common::config(seed, d);
    let mut s = all_alone_core::new_game(&cfg).unwrap();
    let mut steps = 0;
    while !s.ended() && steps < 40_000 {
        let a = common::heuristic(&s);
        if matches!(a, all_alone_core::Action::Rise) {
            let vals: Vec<String> = all_alone_core::content::places::PLACES
                .iter()
                .filter(|p| s.places[p.id].known)
                .map(|p| format!("{}={}", p.id, common::place_value(&s, p.id)))
                .collect();
            println!(
                "   [cap {} light {} melee {} meals {} | {}]",
                common::max_danger(&s),
                common::light_uses(&s),
                common::best_melee(&s),
                all_alone_core::systems::inventory::meal_count(&s.store),
                vals.join(" ")
            );
        }
        if verbose {
            println!(
                "    > {a:?} [{:?} {} hp{} sat{} hyd{} rest{} warm{} nerve{} noise{}]",
                s.phase,
                all_alone_core::systems::journal::clock(s.minute),
                s.vitals.pts(all_alone_core::content::VitalKind::Health),
                s.vitals.pts(all_alone_core::content::VitalKind::Satiety),
                s.vitals.pts(all_alone_core::content::VitalKind::Hydration),
                s.vitals.pts(all_alone_core::content::VitalKind::Rest),
                s.vitals.pts(all_alone_core::content::VitalKind::Warmth),
                s.vitals.pts(all_alone_core::content::VitalKind::Nerve),
                s.noise
            );
        }
        let out = all_alone_core::dispatch(&mut s, &a).unwrap();
        for e in out.journal {
            println!("D{} {} {}", e.day, e.time, e.text);
        }
        steps += 1;
    }
    println!(
        "ended: {:?} day {} parts {:?} stations {:?}",
        s.ending.as_ref().map(|e| &e.cause),
        s.day(),
        s.ferry,
        s.stations.values().collect::<Vec<_>>()
    );
    let keep: Vec<String> = s
        .store
        .iter()
        .filter(|(k, _)| {
            [
                "coil_cracked",
                "belting",
                "wire",
                "battery",
                "pitch",
                "nails",
                "plank",
                "scrap",
                "parts",
                "spanner",
                "key_wharf",
                "lockpicks",
                "fuel_drum",
                "hull_patch",
                "spark_coil",
                "drive_belt",
            ]
            .contains(&k.as_str())
        })
        .map(|(k, v)| format!("{k}:{v}"))
        .collect();
    println!("store: {}", keep.join(" "));
    let known: Vec<&str> = s
        .places
        .iter()
        .filter(|(_, p)| p.known)
        .map(|(k, _)| k.as_str())
        .collect();
    println!("known: {known:?}");
    for p in all_alone_core::content::items::FERRY_PARTS {
        println!("install {p}: {:?}", all_alone_core::systems::home::can_install(&s, p));
    }
    println!(
        "hammer {} wb {}",
        s.store.contains_key("hammer"),
        s.level(all_alone_core::content::Station::Workbench)
    );
}
