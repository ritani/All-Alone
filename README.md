# ALL ALONE

An original survival-horror game. Calder's Reach, a river town, stopped in the
night. You are alone in the lock house. Scavenge, keep warm, keep quiet, and
repair the ferry before the river freezes.

Text and line art, built for phones first. Web now; iOS and Android later via
Capacitor from the same build.

**Status:** milestone M1 (foundation). There is a title screen and a review
sheet of visual primitives; the game itself is not playable yet.

The design, look, technology, security rules and milestones are in
[docs/PLAN.md](docs/PLAN.md). Read it before changing anything.

## Run

Requires Node 22.12 or newer.

```sh
npm ci            # install exactly what the lockfile pins
npm run dev       # dev server
npm run check     # typecheck + lint + tests + production build
npm run preview   # serve the production build from dist/
```

Other scripts: `build`, `typecheck`, `lint`, `test`.

Open the dev or preview URL with `?kit` appended to see every icon and UI
primitive on one page.

## Content-Security-Policy

`index.html` carries a strict CSP meta tag; `public/_headers` carries the same
policy plus `frame-ancestors` and the other security headers for static hosts.

The dev server injects CSS as inline `<style>` tags for hot reload, so in dev
only (`npm run dev`) a small Vite plugin adds `'unsafe-inline'` to `style-src`.
The production build keeps the strict policy, and the build fails if the
emitted HTML contains inline scripts, inline styles, style attributes, event
handler attributes or remote resources. Check CSP behaviour with
`npm run build && npm run preview`, not with the dev server.

## Layout

```
src/core/      pure game logic (no DOM, no UI, no platform imports; lint-enforced)
src/platform/  storage, haptics, lifecycle adapters
src/ui/        components, screens, icons, styles
```
