# ALL ALONE

An original survival-horror game. Calder's Reach, a mill town in a river
valley, stopped in the night. Some of it moves again after dark, and it comes
to sound. You are alone in the lock house. Scavenge, keep warm, keep quiet,
and repair the ferry before the river freezes.

Text and line art, built for phones first. One build runs on the web (as an
installable, offline PWA) and as native iOS and Android apps via Capacitor.

The design, look, technology, security rules and milestones are in
[docs/PLAN.md](docs/PLAN.md). Read it before changing anything.

## How to play

- **Time only moves when you act.** Every action costs minutes; the clock,
  light, weather and the season all follow from what you do.
- **Six vitals**: health, satiety, hydration, rest, nerve and warmth. Any at
  zero drains health. Eat, drink, sleep, keep the stove going.
- **Noise is the danger.** Forcing locks, breaking things, running and
  shooting make noise; noise draws the Stilled, on site and to the house at
  night. Quiet options cost more time.
- **Expeditions**: pick a place on the map, pack within your weight limit,
  walk (or hurry) there, move room by room, search, and come home before dark.
- **The lock house** has stations to build and raise (stove, cot, workbench,
  rain barrel, garden bed, barricade, wireless, medicine shelf, pantry) that
  unlock recipes and help you through the nights.
- **The goal**: find and fit the ferry's four parts (hull patch, fuel, spark
  coil, drive belt), lay in food and water, and cast off before the freeze.
  Death is permanent; there is one run slot, saved after every action.
- Between runs, marks, the ledger of your best runs and remnants (spent on
  new backgrounds and keepsakes) persist. The daily river is one fixed seed
  per calendar day.

## Prerequisites

- **Node** 22.12 or newer (npm 10+).
- **Rust** stable (1.85+) with the `wasm32-unknown-unknown` target,
  [`wasm-pack`](https://rustwasm.github.io/wasm-pack/), optionally `wasm-opt`
  (binaryen) for a smaller `.wasm`, and `cargo-audit` for the audit.
- **iOS**: macOS with Xcode 16+ (built and tested with Xcode 26). The iOS
  project uses Swift Package Manager; CocoaPods is not needed.
- **Android**: Android SDK (platform 36) and a **JDK 21** (Gradle 8.14 does
  not run on JDK 25, which is what current Android Studio bundles; point
  `JAVA_HOME` at a JDK 21, or let Android Studio use its Gradle JDK setting).

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked        # and: cargo install cargo-audit --locked
```

`cargo`, `rustc` and `wasm-pack` must be on `PATH` (on the original machine:
`/opt/homebrew/opt/rustup/bin` and `~/.cargo/bin`). The first `wasm-pack`
build downloads the matching `wasm-bindgen` CLI once.

## Commands

```sh
npm ci            # install exactly what package-lock.json pins (no install scripts run)
npm run dev       # build the core, then the dev server
npm run check     # Rust fmt/clippy/tests + types check, wasm build, typecheck, lint, Vitest, production build
npm run audit     # npm audit + cargo audit
```

| Script | What it does |
|---|---|
| `build` | Rust core to `src/core/pkg/` (git-ignored), then the production web build in `dist/` |
| `preview` | Serve `dist/` (check CSP and offline behaviour here, not in `dev`) |
| `build:mobile` | `build`, then `cap sync` (copies `dist/` into both native projects) |
| `ios` / `android` | Open the native project in Xcode / Android Studio |
| `icons` | Regenerate every icon and splash image from `assets/mark.svg` |
| `types` | Regenerate `src/core/types.ts` from the Rust serde types (the check fails if stale) |
| `check:rust`, `audit:rust` | The Rust half of `check`; `cargo audit` alone |
| `typecheck`, `lint`, `test` | The TypeScript halves of `check` |

`cargo test --test bots -- --nocapture` prints the balance report from the
headless bot playthroughs; `cargo run --example trace -- <seed>` prints one
bot run's journal. In the dev server only, `?kit` shows every icon and UI
primitive on one page (the route is compiled out of production builds).

### Web

The project deploys to Cloudflare Pages from GitHub Actions: `main` is production
and `MVP` is staging. Setup is in [docs/DEPLOY.md](docs/DEPLOY.md).

To host it elsewhere: `npm run build` and deploy `dist/` to any static host over HTTPS. Send the
security headers: `public/_headers` (Netlify, Cloudflare Pages) is copied into
`dist/`; `vercel.json` carries the same for Vercel; for nginx, see
[SECURITY.md](SECURITY.md). The service worker (`dist/sw.js`, generated at
build time) precaches every built file, so after one visit the game runs
offline and can be installed from the browser.

### iOS

```sh
npm run build:mobile
npm run ios            # opens Xcode: pick a simulator or device and run
```

Command line, simulator, no signing:

```sh
cd ios/App
xcodebuild -project App.xcodeproj -scheme App -configuration Debug \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro' CODE_SIGNING_ALLOWED=NO build
```

### Android

```sh
npm run build:mobile
npm run android        # opens Android Studio
# or, with JAVA_HOME at a JDK 21 and ANDROID_HOME at the SDK:
cd android && ./gradlew assembleDebug          # app/build/outputs/apk/debug/
cd android && ./gradlew bundleRelease          # R8-shrunk, unsigned AAB until signing is set up
```

To inspect the WebView while debugging, sync with
`CAP_WEBVIEW_DEBUG=1 npx cap sync`, and sync again without it before any
release build (web inspection is off by default, on both platforms).

## Before publishing (the owner's steps)

- **App identity**: `appId` in `capacitor.config.ts` is the placeholder
  `com.allalone.game`. Change it there, in `ios/App/App.xcodeproj`
  (`PRODUCT_BUNDLE_IDENTIFIER`), in `android/app/build.gradle`
  (`namespace`, `applicationId`), `android/app/src/main/res/values/strings.xml`
  and the Java package path of `MainActivity`, then `npx cap sync`.
- **Versions**: `MARKETING_VERSION`/`CURRENT_PROJECT_VERSION` (Xcode) and
  `versionName`/`versionCode` (`android/app/build.gradle`).
- **Signing**: an Apple Developer team in Xcode; an Android upload keystore
  kept outside the repository, wired through a `signingConfig` that reads
  `android/keystore.properties` (git-ignored) or environment variables. Never
  commit keys (`*.jks`, `*.keystore`, `*.p12`, provisioning profiles are
  ignored).
- **Store listings**: App Store Connect and Google Play Console accounts,
  privacy declarations (the game collects nothing, uses no network and no
  tracking), screenshots and age rating.

## Architecture

```
crates/core/        the game in Rust: content/, systems/, state, actions, saves, marks, wasm bridge
src/core/           typed TS wrapper over the wasm core; generated types.ts; generated pkg/ (ignored)
src/platform/       the device: storage (localStorage / Capacitor Preferences), haptics,
                    lifecycle (save on pause, Android back), PWA registration, native.ts (Capacitor, loaded only in the apps)
src/ui/             Preact view: state/ (the app store), components, screens, icons, styles
src/sw/             the hand-written service worker template
assets/mark.svg     the app mark; scripts/icons.mjs renders every icon and splash from it
ios/, android/      Capacitor native shells (generated, then hardened; see SECURITY.md)
```

- The **Rust core** holds every rule, all content, saves, marks, ledger and
  profile logic. It is deterministic (one seeded RNG in the state) and tested
  headless with `cargo test`, including bot playthroughs that win and die.
- The **view** only draws the `View` the core returns and sends `Action`s back
  through one store method (`act`). It never re-implements a rule.
- The **platform** layer is picked once at start: on the web, localStorage and
  `navigator.vibrate`; in the apps (detected from Capacitor's bridge, without
  importing it), Preferences and the Haptics plugin, loaded on demand. `src/ui`
  may not import Capacitor (lint rule).
- **Lifecycle**: every action autosaves; pending writes are flushed when the
  page is hidden (`visibilitychange`, `pagehide`) or the app is paused or
  backgrounded. The Android back button closes an open sheet, then goes back
  a screen (a run in progress goes to the title and is kept, never ended);
  only from the title does it leave the app.

## Saves: format and migration policy

- Two keys hold strings the core wrote: the run (`all-alone.run`) and the
  profile (`all-alone.profile`). Each is a versioned envelope
  `{"format": "all-alone/run", "version": N, "data": {...}}`; run and profile
  saves are at **version 2**.
- Storage is untrusted. On load the core checks the size cap (768 KiB run,
  256 KiB profile), parses strictly (`deny_unknown_fields`), migrates older
  versions, then validates every value and cross-reference. An invalid save is
  rejected whole, never half-loaded; the view keeps the rejected string aside
  (`*.rejected`, up to 1 MiB) and says so.
- The one UI-only key (`all-alone.codex-unread`, codex entries not yet opened)
  is parsed in TypeScript with a size cap, an id pattern and a count cap.
- **Policy**: any change to a saved type bumps the version and adds a
  migration in `crates/core/src/save.rs`, with a fixture of the old format in
  `crates/core/tests/fixtures` and a test that it loads and plays on. A newer
  save than the build knows is refused, not guessed at. A run's ledger is
  entered in the profile exactly once, even across a migration (the "already
  applied" fingerprint uses only fields every version has).
- Web and native saves are separate (localStorage vs Preferences); nothing is
  migrated between them.

## Regenerating

- **Types**: `npm run types` after changing a serde type in the core.
- **Icons and splash**: edit `assets/mark.svg` (stroke-only paths on a
  100 x 100 grid), then `npm run icons`. It renders with a local headless
  Chrome (set `CHROME=/path/to/chrome` if it is elsewhere) and writes the PWA
  icons, the iOS app icon and launch image, and the Android launcher, adaptive,
  themed and splash resources.

## Security

Summary (details and the threat model in [SECURITY.md](SECURITY.md)): no
network use and no remote code; a strict CSP identical in substance in
`index.html`, `_headers` and `vercel.json` (checked by the build); saves
validated in Rust; exact dependency versions, lockfiles, `npm audit` and
`cargo audit` clean, no install scripts; native apps with no backup, no
cleartext, no INTERNET permission on Android, no web inspection in release,
R8-shrunk non-debuggable Android release; CI with pinned action SHAs and
read-only permissions.

### Content-Security-Policy in development

The dev server injects CSS as inline `<style>` tags for hot reload, so in dev
only a small Vite plugin adds `'unsafe-inline'` to `style-src`. The production
build keeps the strict policy, and the build fails if the emitted HTML has
inline scripts or styles, style attributes, event handler attributes, remote
resources, or a policy that differs from the static-host headers.
