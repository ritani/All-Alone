# ALL ALONE — remake plan

An original survival-horror game. It keeps the genre mechanics the POC explored
(shelter, vitals, scavenging, crafting) and replaces everything else: setting,
content, rules, look, and technology. It is not a recreation of any existing game.

## 1. Where the POC stands vs. where we are going

| Area | POC today | Target |
|---|---|---|
| Identity | "Buried City", described as inspired by another game | **ALL ALONE**, own setting, story, names, items, map |
| Tone | Cartoonish: bright fills, rounded shapes, emoji-like visuals | Serious, near-horror: near-black field, hairline bone-white line art, one danger accent, terse prose |
| Rendering | Phaser 3.60 canvas at fixed 800x600, scaled to fit (blurry text on phones) | DOM + inline SVG: text and lines stay crisp at every pixel density; portrait-first responsive layout |
| Time | Real-time ticker (3 s = 1 h) | Action-driven clock: nothing happens unless the player acts; every action costs minutes |
| Threat | Zombies with HP that scales per day | The Stilled: dormant by day, drawn to **noise**; noise is a resource the player spends |
| Vitals | HP, energy, mood, hunger, thirst | Health, satiety, hydration, rest, nerve, warmth (season turns toward winter) |
| Goal | None (survive indefinitely) | Repair the ferry and leave; death is permanent |
| Code | 24 global-scope JS files, script tags, Phaser loaded from a CDN | Game core in Rust (WebAssembly), thin strict-TypeScript view, bundled, no globals |
| Tests | None | `cargo test` on the core, including seeded headless playthroughs |
| Saves | Unvalidated | Versioned, schema-validated, size-capped, migrated |
| Platforms | Web page only | Web (PWA), iOS and Android via Capacitor from one build |
| Security | Third-party CDN script, no CSP, no lockfile | No remote code, strict CSP, pinned + audited deps, hardened native config |

## 2. Game design

### Premise
Calder's Reach, a mill town in a river valley. One night in early autumn, almost
everyone simply stopped — standing, sitting, mid-step. That was the Stillness.
Some of them move again after dark. They do not hunt by sight. They come to sound.

The player is the lock-keeper's lodger, alone in the lock house on the river.
The ferry on the far slip could carry one person downriver, if it ran.
Winter is coming and the river will freeze.

### Core loop
1. **Day, at the lock house** — eat, drink, mend, craft, build stations, read the radio.
2. **Expedition** — pick a place on the map, pack within a weight limit, walk there
   (time + vitals), move through its rooms, search, deal with what is inside, walk back.
3. **Night** — noise made that day, plus barricade strength, decides whether
   anything comes to the house. Sleep restores rest; cold and fear erode it.
4. Repeat while the season worsens. Find the four ferry parts, repair, leave.

### Rules
- **Clock**: minutes. Day 1 starts 07:00. Dusk 19:00, dawn 06:00. Every action has a minute cost.
- **Vitals** (0–100): health, satiety, hydration, rest, nerve, warmth. Each decays per
  hour at its own rate; any at 0 drains health. Low nerve reduces hit chance and can
  refuse actions; low rest slows everything; low warmth accelerates the others.
- **Season**: outside temperature falls day by day. Around day 45 the river freezes — the ferry
  can no longer leave (game does not end, but the win is gone; the journal says so).
- **Noise** (0–100, per day): forcing locks, breaking, firearms, running and fighting add
  noise. Quiet options cost more time. Noise raises encounter chance on site and
  night-visit chance at home. It decays overnight.
- **The Stilled**: dormant in daylight unless noise wakes them; active in dark rooms and
  at night. A few kinds with distinct behaviour (e.g. slow standers, a listener that
  calls others, a runner). Other survivors exist as rare, tense, text-driven events
  — not always hostile.
- **Encounters**: turn-based, terse. Distance bands (far / near / close). Actions:
  strike, shoot, shove, back away, hold still, flee. Weapons have reach, damage,
  noise, durability; firearms need ammunition. Wounds can bleed or infect and need treatment.
- **Carrying**: weight limit, raised by bags. What does not fit stays behind.
- **Places**: about ten, on a node map with walking times, unlocked by discovery
  (notes, radio, keys). Each has an ordered set of rooms with fixed character and
  seeded loot; searched rooms stay searched. Some rooms are dark (need light),
  locked (key, quiet pick, or loud force), or flooded.
- **Lock house stations** (levels 0–3): stove, cot, workbench, rain barrel, garden
  bed, barricade, radio, medicine shelf, store. Levels unlock recipes and passive effects.
- **Crafting**: recipes need materials, a station level, and time.
- **Journal**: every outcome is written as a short second-person line. The journal
  is the main feedback channel. Radio fragments and found notes carry the story.
- **Ending**: with hull patch, fuel, spark coil and drive belt installed before the
  freeze, the player can cast off. Death is permanent; one save slot, autosaved.
- **Determinism**: one seeded RNG in the state; same seed + same actions = same game.

### What makes it worth playing again
These are first-class features, not extras. All of them stay in the game's sober
voice: no confetti, no cartoon badges.

- **Choice events**: short situations with two to four options and real consequences —
  a knock at night, a voice on the radio asking where you are, a Stilled child on the
  stairs, a stranger at the weir. Outcomes depend on vitals, items, skills and the seed.
  At least 30, some chained across days.
- **The dog**: a stray can be found and, if fed, stays. It warns before night visits,
  steadies nerve, can come on expeditions (hears things first, but can make noise),
  and can be lost. It needs food and water.
- **Skills by use**: quiet step, lockwork, mending, marksmanship, foraging, field
  medicine. Each improves with practice (levels 0–5) and visibly changes odds/time costs.
- **Backgrounds**: the lodger's former trade, chosen at the start — e.g. nurse,
  poacher, machinist, clerk. Each changes starting kit, one skill, and one drawback.
  Only one is open at first; the rest are unlocked through play.
- **Food you work for**: snares on a trap line and fishing from the lock gate
  (set now, check later; results depend on bait, weather, skill), plus the garden bed.
- **Barter**: a ferryman's widow trades at the weir on some days; prices move with the
  season. What you sell her changes what she offers later.
- **Weather**: rain fills the barrel and masks noise; fog hides you and them; frost
  hurts warmth. Weather is forecast, imperfectly, by the radio.
- **Risk you opt into**: every room offers a quiet/slow and a loud/fast approach;
  staying out after dusk gives better finds and much worse odds.
- **Discoveries (codex)**: found notes, photographs, radio fragments and a field
  guide to the Stilled fill in as they are met. The story of what the Stillness was
  is only ever assembled from these.
- **Objectives**: a short, always-visible list of what matters next ("Find something
  to hold water", "The barricade will not last another night") generated from state,
  so a new player is never lost and a veteran always has a next target.
- **Difficulty**: Quiet (forgiving), Still (standard), Dead of Winter (harsh, scores more).

### Achievements and gamification
- **Marks** (achievements): about 40, in tiers, some hidden until earned. Examples:
  survive 1/7/21/40 nights; a full day under 10 noise; win without firing a shot;
  win with the dog alive; reach the freeze; fill the field guide; die on day one.
  Each has a line-art sigil and one line of text. Unlock shows a quiet toast and a haptic tick.
- **Profile**, stored separately from the run save, persists across deaths: marks
  earned, backgrounds unlocked, codex entries ever found, lifetime totals, best runs.
- **Run ledger**: every run ends (death, departure, or freeze-out) with a ledger —
  days survived, places found, noise made, kills, distance walked, cause of the
  end — and a score with a difficulty multiplier. The ten best ledgers are kept.
- **Remnants**: each ended run converts its score into remnants, spent to unlock
  backgrounds and optional starting keepsakes. Unlocks add variety, not raw power.
- **Daily river**: one fixed seed per calendar date (derived locally from the date,
  no network) with a fixed background; one attempt counts for the day's ledger.
  Tracks a streak of days attempted.
- **In-run progress**: skill levels, station levels, codex completion and ferry
  repair shown as hairline progress meters; a day-end summary page each dawn
  ("Night 6. Noise 14. Nothing came.").
- Marks, ledger and remnants are computed in the Rust core from state and an event
  stream, so they are testable and identical on every platform.

### Look and feel
- Palette: field `#0b0c0d`, panel `#111315`, line/text `#d8d4c8`, dim `#7c7a72`,
  faint `#3a3a37`, danger `#a3271f`. Danger red appears only for real danger.
- Line art: inline SVG, stroke only, 1–1.25 px, square caps, mitre joins, no fills,
  no gradients, no rounded corners, no drop shadows, no emoji.
- Type: system fonts only (no remote fonts). A serif stack for journal prose, a
  monospace stack for numbers, labels in small tracked capitals.
- Layout: portrait-first single column (max ~480 px wide, centred on desktop),
  safe-area aware, touch targets ≥ 44 px, works with keyboard, respects reduced motion.
- Motion: slow opacity fades only. Night dims the whole interface.
- Voice: plain, short, unsentimental. No jokes. No exclamation marks.

## 3. Technology

- **Game core: Rust**, compiled to WebAssembly with `wasm-pack` / `wasm-bindgen`.
  All rules, content, saves, marks, ledger and profile logic live here. `serde`
  types are the single source of truth; state crosses to the UI as JSON.
  Native `cargo test` runs the whole simulation headless.
- **View: TypeScript (strict) + Preact + plain CSS**, built with Vite. It only draws
  the state the core returns and sends actions back. No canvas engine: the game is
  text and line art, and the DOM keeps both sharp on every screen.
- **Native**: Capacitor (iOS + Android) wrapping the same `dist/` build.
  Plugins: Preferences (saves), Haptics, App (back button / pause), Status Bar.
- **Web**: static site + web manifest + offline service worker.
- **Tests/quality**: `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`,
  Vitest for the view, ESLint, `tsc --noEmit`.
- Toolchain on PATH: `/opt/homebrew/opt/rustup/bin` (cargo, rustc) and `~/.cargo/bin` (wasm-pack).

```
crates/core/   Rust game core (lib + cdylib), no platform code
  src/
    content/   items, recipes, stations, places, creatures, events, prose
    systems/   clock, vitals, noise, night, crafting, expedition, encounter, ...
    state.rs   GameState + new_game(config)
    action.rs  Action enum + dispatch(&mut state, action) -> Vec<Entry/Event>
    rng.rs     seeded PRNG held in the state
    save.rs    serialise / validate / migrate (run save and profile)
    meta.rs    marks, ledger, remnants, profile, daily seed
    wasm.rs    thin wasm-bindgen API (feature-gated)
src/
  core/        generated wasm package + a typed TS wrapper (types mirror the Rust serde types)
  platform/    storage, haptics, lifecycle (Capacitor with web fallback)
  ui/          components, screens, icons, styles
  main.tsx
```
The UI may only change the game through the core's `dispatch`. The Rust crate has no
knowledge of the UI or platform; persistence is the view's job (it stores opaque
strings the core produced and hands them back for validation on load).

## 4. Security

- No remote code or assets: everything is bundled; no CDN, analytics, ads, or network calls.
- Content-Security-Policy (meta tag + `_headers` for static hosts):
  `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:;
  connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`.
  No inline scripts, no inline style attributes that need `unsafe-inline`.
- Lint bans `eval`, `new Function`, `innerHTML`, `dangerouslySetInnerHTML`.
- Saves are untrusted input, validated in Rust: size cap, strict `serde` parsing
  (`deny_unknown_fields`), full structural
  validation with range clamping, version migration; an invalid save is rejected, never partially loaded.
- Dependencies: minimal, exact versions, `package-lock.json` and `Cargo.lock` committed,
  `npm audit` and `cargo audit` clean, `npm ci` in docs. No `unsafe` in the core (`#![forbid(unsafe_code)]`).
- Capacitor: no `allowNavigation`, no cleartext traffic, web debugging off in
  release, Android `allowBackup=false`, no unused permissions.
- Extra web headers: `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`,
  restrictive `Permissions-Policy`.

## 5. Milestones

Each milestone ends with `npm run check` (typecheck + lint + tests + build) passing.

| # | Milestone | Done when |
|---|---|---|
| M1 | **Foundation** — remove POC, scaffold TS/Vite/Preact/Vitest/ESLint (view layer), design tokens, icon primitives, title screen, CSP | App builds, title screen renders in the new visual language |
| M2 | **Core simulation** — all of `crates/core` (Rust) with content and tests, including events, dog, skills, backgrounds, barter, weather, objectives, marks, ledger, remnants, profile, daily seed | Seeded headless playthroughs can both die and win; marks unlock in tests; save and profile round-trip; hostile saves rejected |
| M3 | **Lock house UI** — new game (background, difficulty, daily river), vitals, journal, objectives, stations, store, crafting, choice events, sleep/night, dawn summary, autosave, death + ledger, marks and profile screens | A full day/night cycle is playable at home; marks unlock visibly |
| M4 | **Expeditions UI** — map, packing, travel, rooms, searching, encounters, dog, barter, traps/fishing, codex, return, ferry ending | The whole game is playable start to end in a browser |
| M5 | **Platforms and hardening** — Capacitor iOS/Android, storage adapter, safe areas, icons, PWA, headers, audit, docs | Web build passes audit; iOS and Android projects sync and build |

Out of scope for this pass: audio, localisation, cloud saves, store submission.
