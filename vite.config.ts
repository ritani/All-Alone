/// <reference types="node" />
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Plugin } from 'vite';
import { defineConfig } from 'vitest/config';

// The config is bundled to a temp file; import.meta.url keeps the real location.
const root = fileURLToPath(new URL('.', import.meta.url));

/**
 * The Content-Security-Policy, in three places that must agree:
 * - index.html (meta tag; `frame-ancestors` is ignored there, so it is left out),
 * - public/_headers (Netlify, Cloudflare Pages),
 * - vercel.json (Vercel).
 * The build fails if they differ or lose a required directive.
 */
const REQUIRED_DIRECTIVES = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self'",
  "connect-src 'self'",
  "worker-src 'self'",
  "manifest-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
];

function directives(policy: string): string[] {
  return policy
    .split(';')
    .map((d) => d.trim().replace(/\s+/g, ' '))
    .filter(Boolean)
    .sort();
}

function hostPolicies(): Record<string, string> {
  const headers = readFileSync(join(root, 'public/_headers'), 'utf8');
  const fromHeaders = /^\s*Content-Security-Policy:\s*(.+)$/m.exec(headers)?.[1] ?? '';
  const vercel = JSON.parse(readFileSync(join(root, 'vercel.json'), 'utf8')) as {
    headers?: { headers?: { key: string; value: string }[] }[];
  };
  const fromVercel =
    vercel.headers?.flatMap((h) => h.headers ?? []).find((h) => h.key === 'Content-Security-Policy')?.value ?? '';
  return { 'public/_headers': fromHeaders, 'vercel.json': fromVercel };
}

/**
 * index.html carries the strict CSP meta tag (see docs/PLAN.md section 4).
 *
 * - Dev server: Vite injects imported CSS as <style> tags for HMR, which
 *   `style-src 'self'` blocks, so the policy is loosened in dev only.
 * - Build: the policy is kept as written, and the build fails if the emitted
 *   HTML contains anything the policy would block (inline script/style), or
 *   if the meta policy and the static-host header policies disagree.
 */
function csp(): Plugin[] {
  return [
    {
      name: 'all-alone:csp-dev',
      apply: 'serve',
      transformIndexHtml: (html) => html.replace("style-src 'self'", "style-src 'self' 'unsafe-inline'"),
    },
    {
      name: 'all-alone:csp-verify',
      apply: 'build',
      transformIndexHtml: {
        order: 'post',
        handler(html) {
          const problems: string[] = [];
          const meta = /http-equiv="Content-Security-Policy"\s+content="([^"]+)"/.exec(html)?.[1];
          if (!meta) problems.push('CSP meta tag missing');
          if (/<script(?![^>]*\bsrc=)[^>]*>/i.test(html)) problems.push('inline <script>');
          if (/<style[\s>]/i.test(html)) problems.push('inline <style>');
          if (/\sstyle\s*=/i.test(html)) problems.push('style attribute');
          if (/\son[a-z]+\s*=/i.test(html)) problems.push('inline event handler');
          if (/(src|href)\s*=\s*"(https?:)?\/\//i.test(html)) problems.push('remote resource');
          if (meta) {
            const mine = directives(meta);
            for (const d of REQUIRED_DIRECTIVES) if (!mine.includes(d)) problems.push(`meta CSP lacks "${d}"`);
            if (mine.some((d) => /'unsafe-inline'|'unsafe-eval'|(^|\s)\*|https?:/.test(d))) {
              problems.push('meta CSP is loosened');
            }
            for (const [file, policy] of Object.entries(hostPolicies())) {
              const theirs = directives(policy);
              if (!theirs.includes("frame-ancestors 'none'")) problems.push(`${file}: no frame-ancestors 'none'`);
              const same = theirs.filter((d) => !d.startsWith('frame-ancestors')).join('; ') === mine.join('; ');
              if (!same) problems.push(`${file}: CSP differs from index.html`);
            }
          }
          if (problems.length > 0) {
            throw new Error(`index.html violates the CSP: ${problems.join(', ')}`);
          }
          return html;
        },
      },
    },
  ];
}

/** Every file under `dir`, as forward-slash paths relative to it. */
function walk(dir: string, top = dir): string[] {
  return readdirSync(dir).flatMap((name) => {
    const full = join(dir, name);
    return statSync(full).isDirectory() ? walk(full, top) : [relative(top, full).split(sep).join('/')];
  });
}

/** Files that are only for the host, never fetched by the page. */
const NOT_PRECACHED = /(^|\/)(_headers|_redirects|\.DS_Store|sw\.js)$/;

/**
 * Emits dist/sw.js from src/sw/service-worker.js with the build's file list
 * and a version derived from their contents, so every deploy gets a fresh
 * cache and old caches are dropped.
 */
function serviceWorker(): Plugin {
  return {
    name: 'all-alone:sw',
    apply: 'build',
    generateBundle(_options, bundle) {
      const hash = createHash('sha256');
      const files: string[] = [];
      for (const [name, out] of Object.entries(bundle)) {
        if (NOT_PRECACHED.test(name)) continue;
        files.push(name);
        hash.update(name);
        hash.update(out.type === 'chunk' ? out.code : out.source);
      }
      const pub = join(root, 'public');
      for (const name of walk(pub)) {
        if (NOT_PRECACHED.test(name)) continue;
        files.push(name);
        hash.update(name);
        hash.update(readFileSync(join(pub, name)));
      }
      // index.html is written after this hook runs; the page itself must be cached too.
      if (!files.includes('index.html')) files.push('index.html');
      files.sort();
      const template = readFileSync(join(root, 'src/sw/service-worker.js'), 'utf8');
      hash.update(template);
      hash.update(files.join('\n'));
      const version = hash.digest('hex').slice(0, 12);
      const source = template
        .replace("'__ALL_ALONE_VERSION__'", JSON.stringify(version))
        .replace("/** @type {string[]} */ ('__ALL_ALONE_ASSETS__')", JSON.stringify(files));
      if (source.includes('__ALL_ALONE_')) this.error('service worker placeholders were not replaced');
      this.emitFile({ type: 'asset', fileName: 'sw.js', source });
    },
  };
}

export default defineConfig({
  // Relative asset URLs: works from any sub-path and inside the Capacitor webview.
  base: './',
  plugins: [csp(), serviceWorker()],
  oxc: {
    jsx: { runtime: 'automatic', importSource: 'preact' },
  },
  build: {
    target: 'es2022',
    // The module preload polyfill is an inline script; modern targets do not need it.
    modulePreload: { polyfill: false },
    // Never inline assets as data: URIs into JS/CSS.
    assetsInlineLimit: 0,
    sourcemap: false,
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
