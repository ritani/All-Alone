import type { Plugin } from 'vite';
import { defineConfig } from 'vitest/config';

/**
 * index.html carries the strict CSP meta tag (see docs/PLAN.md section 4).
 *
 * - Dev server: Vite injects imported CSS as <style> tags for HMR, which
 *   `style-src 'self'` blocks, so the policy is loosened in dev only.
 * - Build: the policy is kept as written, and the build fails if the emitted
 *   HTML contains anything the policy would block (inline script/style).
 */
function csp(): Plugin[] {
  return [
    {
      name: 'all-alone:csp-dev',
      apply: 'serve',
      transformIndexHtml: (html) =>
        html.replace("style-src 'self'", "style-src 'self' 'unsafe-inline'"),
    },
    {
      name: 'all-alone:csp-verify',
      apply: 'build',
      transformIndexHtml: {
        order: 'post',
        handler(html) {
          const problems: string[] = [];
          if (!html.includes('http-equiv="Content-Security-Policy"')) problems.push('CSP meta tag missing');
          if (/<script(?![^>]*\bsrc=)[^>]*>/i.test(html)) problems.push('inline <script>');
          if (/<style[\s>]/i.test(html)) problems.push('inline <style>');
          if (/\sstyle\s*=/i.test(html)) problems.push('style attribute');
          if (/\son[a-z]+\s*=/i.test(html)) problems.push('inline event handler');
          if (/(src|href)\s*=\s*"(https?:)?\/\//i.test(html)) problems.push('remote resource');
          if (problems.length > 0) {
            throw new Error(`index.html violates the CSP: ${problems.join(', ')}`);
          }
          return html;
        },
      },
    },
  ];
}

export default defineConfig({
  // Relative asset URLs: works from any sub-path and inside the Capacitor webview.
  base: './',
  plugins: [csp()],
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
