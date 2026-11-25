// ALL ALONE offline service worker. Hand-written, no dependencies.
//
// The build (vite.config.ts, `all-alone:sw`) replaces the two placeholders
// below with the build's version and the list of every emitted file, then
// writes this as dist/sw.js.
//
// - install: precache every built asset (the .wasm core included) into a
//   versioned cache, all or nothing.
// - activate: delete caches from older versions.
// - fetch: cache-first for same-origin GETs; navigations fall back to the
//   cached index.html. Cross-origin requests are never touched or cached
//   (the game makes none).
// It is registered only by production web builds, never in the native apps.

const VERSION = '__ALL_ALONE_VERSION__';
const ASSETS = /** @type {string[]} */ ('__ALL_ALONE_ASSETS__');
const PREFIX = 'all-alone-';
const CACHE = `${PREFIX}${VERSION}`;
const scope = new URL(self.registration.scope);
const INDEX = new URL('index.html', scope).href;

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll(['./', ...ASSETS].map((a) => new Request(new URL(a, scope), { cache: 'reload' }))))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k.startsWith(PREFIX) && k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener('fetch', (event) => {
  const req = event.request;
  if (req.method !== 'GET') return;
  const url = new URL(req.url);
  if (url.origin !== scope.origin) return;
  event.respondWith(
    caches.open(CACHE).then(async (cache) => {
      // ignoreVary: static files; a module script's Origin header must not miss the precache.
      const hit = await cache.match(req, { ignoreVary: true, ignoreSearch: req.mode === 'navigate' });
      if (hit) return hit;
      if (req.mode === 'navigate') {
        const index = await cache.match(INDEX, { ignoreVary: true });
        if (index) return index;
      }
      // Not precached (should not happen in a complete build): go to the network, do not store.
      return fetch(req);
    }),
  );
});
