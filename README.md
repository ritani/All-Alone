# ALL ALONE

An original survival-horror game. Calder's Reach, a river town, stopped in the
night. You are alone in the lock house. Scavenge, keep warm, keep quiet, and
repair the ferry before the river freezes.

Text and line art, built for phones first. Web now; iOS and Android later via
Capacitor from the same build.

**Status:** milestone M3 (lock house UI). The whole game runs in the Rust
core (`crates/core`, compiled to WebAssembly) and is playable end to end in
the browser: new run setup, the daily river, the lock house, choice events,
night, dawn, death and the ledger, marks and remnants. Expedition phases
(packing, travel, rooms, encounters, barter) use a plain fallback screen until
M4 designs them.

The design, look, technology, security rules and milestones are in
[docs/PLAN.md](docs/PLAN.md). Read it before changing anything.

## Toolchain

- Node 22.12 or newer.
- Rust stable (1.85+) with the `wasm32-unknown-unknown` target, `wasm-pack`,
  and optionally `wasm-opt` (binaryen) for a smaller `.wasm`. `cargo-audit` for
  `npm run audit:rust`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack        # and: cargo install cargo-audit
```

`cargo`, `rustc` and `wasm-pack` must be on `PATH` (on this machine:
`/opt/homebrew/opt/rustup/bin` and `~/.cargo/bin`). The first `wasm-pack`
build downloads the matching `wasm-bindgen` CLI once.

## Run

```sh
npm ci            # install exactly what the lockfile pins
npm run dev       # build the core, then the dev server
npm run check     # Rust fmt/clippy/tests + types check, wasm build, typecheck, lint, Vitest, production build
npm run preview   # serve the production build from dist/
```

Other scripts:

- `build:core` builds the Rust core to `src/core/pkg/` (git-ignored) with
  `wasm-pack --target web`, then `wasm-opt -Oz` if installed.
- `types` regenerates `src/core/types.ts` from the Rust serde types (the
  check fails if it is stale).
- `check:rust` runs only the Rust side; `audit:rust` runs `cargo audit`.
- `build`, `typecheck`, `lint`, `test`.

`cargo test --test bots -- --nocapture` prints the balance report from the
headless bot playthroughs; `cargo run --example trace -- <seed>` prints one
bot run's journal.

Open the dev or preview URL with `?kit` appended to see every icon and UI
primitive on one page.

## Content-Security-Policy

`index.html` carries a strict CSP meta tag; `public/_headers` carries the same
policy plus `frame-ancestors` and the other security headers for static hosts.
`script-src` includes `'wasm-unsafe-eval'`, which WebAssembly compilation
needs; nothing else is loosened.

The dev server injects CSS as inline `<style>` tags for hot reload, so in dev
only (`npm run dev`) a small Vite plugin adds `'unsafe-inline'` to `style-src`.
The production build keeps the strict policy, and the build fails if the
emitted HTML contains inline scripts, inline styles, style attributes, event
handler attributes or remote resources. Check CSP behaviour with
`npm run build && npm run preview`, not with the dev server.

## Layout

```
crates/core/   the game core in Rust: content/, systems/, state, actions, saves, marks, wasm bridge
src/core/      typed TS wrapper over the wasm core, generated types.ts, generated pkg/ (ignored)
src/platform/  storage (async key-value; web = localStorage) and haptics adapters
src/ui/        state/ (the app store: run, view, profile, autosave), components, screens, icons, styles
```
