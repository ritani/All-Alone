# Security

ALL ALONE is a single-player, offline game. It has no accounts, no server, no
analytics, no ads and makes no network requests. The security work is about
keeping it that way, and about the few places untrusted data can enter.

## Reporting

Please report a vulnerability privately to the maintainer (GitHub security
advisories on this repository) rather than in a public issue.

## Threat model

| Asset / entry point | Threat | Controls |
|---|---|---|
| Code the app runs | Injected or remote script (XSS, a compromised CDN) | Everything bundled, no CDN or remote fonts; strict CSP (`script-src 'self' 'wasm-unsafe-eval'`, no `unsafe-inline`/`unsafe-eval`); lint bans `eval`, `new Function`, `innerHTML`, `dangerouslySetInnerHTML`, `document.write`; the build fails on inline script/style, style attributes, inline handlers or remote URLs in `index.html` |
| Saves in localStorage / Preferences | Edited or corrupted save (cheating is accepted; crashes, hangs or code paths are not) | The core treats saves as untrusted: size caps, strict `serde` with `deny_unknown_fields`, structural validation and range checks, versioned migration, reject-whole; fuzz and hostile-value tests in `crates/core/tests/saves.rs`; the UI-only unread list is parsed with a size cap, id pattern and count cap; a rejected save is kept aside only up to 1 MiB |
| Framing / embedding | Clickjacking | `frame-ancestors 'none'`, `X-Frame-Options: DENY` |
| Third-party code (supply chain) | Malicious or vulnerable dependency, install-time scripts | Few dependencies (Preact, Capacitor and four official plugins at runtime); exact versions; `package-lock.json` and `Cargo.lock` committed; `npm ci`; `ignore-scripts=true` in `.npmrc` (nothing needs an install script); `npm audit` and `cargo audit` clean and run in CI; iOS Swift packages pinned `exact` with `Package.resolved` committed; Android Gradle dependencies pinned by Capacitor's `variables.gradle` and the Gradle distribution pinned by `distributionSha256Sum`; Google Services plugin removed |
| CI | Token abuse, mutable action tags | Actions pinned to commit SHAs; `permissions: contents: read`; `persist-credentials: false` |
| Native shell | Loading remote content, cleartext, data exfiltration via backups, debugging a release | No `server.url`, no `allowNavigation`; `cleartext: false`; Android: `usesCleartextTraffic="false"`, a network security config denying cleartext, `allowBackup="false"`, `fullBackupContent="false"`, data extraction rules excluding everything (cloud and device transfer), no `INTERNET` permission (local files are served in-process), FileProvider removed, release not debuggable and R8-shrunk; iOS: no ATS exceptions, no usage-description keys, `ITSAppUsesNonExemptEncryption` false; WebView inspection off unless `CAP_WEBVIEW_DEBUG=1` at sync time |
| Service worker (web) | Cache poisoning, stale or cross-origin caching | Hand-written, no dependency; same-origin GET only, never caches cross-origin or non-GET; precache list and version generated at build time; old caches deleted on activate; `worker-src 'self'`; never registered inside the native apps or the dev server |

Out of scope: a player editing their own save to cheat (the game is offline
and single-player; the core only guarantees it cannot be crashed or confused).

## Content-Security-Policy

```
default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:;
connect-src 'self'; worker-src 'self'; manifest-src 'self'; object-src 'none'; base-uri 'none';
form-action 'none'; frame-ancestors 'none'
```

`'wasm-unsafe-eval'` is what WebAssembly compilation needs; nothing else is
loosened. The meta tag in `index.html` carries the same policy minus
`frame-ancestors` (ignored in a meta tag). `vite.config.ts` fails the build if
`index.html`, `public/_headers` and `vercel.json` disagree, or if a required
directive is missing or loosened.

## Web headers

`public/_headers` (Netlify, Cloudflare Pages) and `vercel.json` (Vercel) send:

- `Content-Security-Policy` (above)
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer`
- `Permissions-Policy` denying every powerful feature (camera, microphone,
  geolocation, sensors, payment, USB, ...), `fullscreen=(self)`
- `X-Frame-Options: DENY`
- `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin`,
  `Cross-Origin-Embedder-Policy: require-corp`, `Origin-Agent-Cluster: ?1`
- `Cache-Control: no-cache` for `/sw.js`; immutable for hashed `/assets/*`
- **HSTS**: serve over HTTPS only. `vercel.json` sends
  `Strict-Transport-Security: max-age=63072000; includeSubDomains`. Netlify and
  Cloudflare Pages add HSTS on their own domains; on a custom domain add it
  once every subdomain is on HTTPS (add `preload` only deliberately).

nginx equivalent:

```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; connect-src 'self'; worker-src 'self'; manifest-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'" always;
add_header X-Content-Type-Options "nosniff" always;
add_header Referrer-Policy "no-referrer" always;
add_header Permissions-Policy "accelerometer=(), autoplay=(), bluetooth=(), browsing-topics=(), camera=(), clipboard-read=(), clipboard-write=(), display-capture=(), encrypted-media=(), fullscreen=(self), geolocation=(), gyroscope=(), hid=(), idle-detection=(), magnetometer=(), microphone=(), midi=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), serial=(), usb=(), web-share=(), xr-spatial-tracking=()" always;
add_header X-Frame-Options "DENY" always;
add_header Cross-Origin-Opener-Policy "same-origin" always;
add_header Cross-Origin-Resource-Policy "same-origin" always;
add_header Cross-Origin-Embedder-Policy "require-corp" always;
add_header Strict-Transport-Security "max-age=63072000; includeSubDomains" always;
types { application/wasm wasm; application/manifest+json webmanifest; }
location = /sw.js { add_header Cache-Control "no-cache"; }
```

(nginx does not inherit `add_header` into a `location` that sets its own; repeat
the security headers there, or use an include.)

## Supply chain notes

- `npm ci` installs exactly the lockfile. `.npmrc` sets `ignore-scripts=true`:
  no dependency needs an install script (the only one with one, `fsevents`, is
  an optional macOS file watcher for Vite that works without it). Explicit
  `npm run` scripts still run.
- Rust: `Cargo.lock` committed; `#![forbid(unsafe_code)]` in the core;
  `cargo audit --deny warnings` in CI. CI installs `wasm-pack` and
  `cargo-audit` at exact versions with `--locked`.
- The icon script uses a local Chrome; it adds no dependency.

## Things only the owner can do

Signing identities, keystores, store accounts and the final bundle id are not
in the repository and must never be committed (see README, "Before
publishing").
