/// <reference types="node" />
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { Run, initCoreSync, profile, type View } from '../../core';
import { ICON_NAMES, iconPath, ICON_PATHS, isIconName } from './paths';

beforeAll(() => {
  initCoreSync(readFileSync(new URL('../../core/pkg/all_alone_core_bg.wasm', import.meta.url)));
});

/** Every `icon` field anywhere in a JSON value. */
function collectIcons(value: unknown, out: Set<string>): void {
  if (Array.isArray(value)) {
    for (const v of value) collectIcons(v, out);
  } else if (value && typeof value === 'object') {
    for (const [k, v] of Object.entries(value)) {
      if (k === 'icon' && typeof v === 'string') out.add(v);
      else collectIcons(v, out);
    }
  }
}

function rustFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    return statSync(p).isDirectory() ? rustFiles(p) : p.endsWith('.rs') ? [p] : [];
  });
}

/** Icon keys written in the core's source: `icon: "x"`, prefixed keys, and action tuples. */
function iconsInSource(): Set<string> {
  const root = fileURLToPath(new URL('../../../crates/core/src', import.meta.url));
  const out = new Set<string>();
  for (const file of rustFiles(root)) {
    if (file.endsWith('ts_export.rs')) continue;
    const text = readFileSync(file, 'utf8');
    for (const m of text.matchAll(/\bicon:\s*"([a-z_]+)"/g)) out.add(m[1]!);
    for (const m of text.matchAll(/"((?:sigil|place|bg|weather|skill)_[a-z_]+)"/g)) out.add(m[1]!);
    // describe(): ("Label".into(), "group", "icon".into(), danger)
    for (const m of text.matchAll(/\.into\(\),\s*"[a-z]+",\s*"([a-z_]+)"\.into\(\)/g)) out.add(m[1]!);
    // fn icon(self) -> &'static str { match ... => "x" }
    for (const block of text.matchAll(/fn icon\(self\)[^{]*\{([\s\S]*?)\n {4}\}/g)) {
      for (const m of block[1]!.matchAll(/=>\s*"([a-z_]+)"/g)) out.add(m[1]!);
    }
  }
  return out;
}

/** Play a seeded run with a simple picker, collecting icons from every view. */
function iconsFromPlay(seed: number, out: Set<string>): void {
  const p = profile.create();
  const run = Run.start(profile.runConfig(p, seed, 'still', 'clerk', null));
  let x = seed;
  for (let i = 0; i < 4000 && !run.ended; i++) {
    const view: View = run.view();
    collectIcons(view, out);
    const enabled = view.actions.filter((a) => a.enabled);
    if (enabled.length === 0) break;
    x = (Math.imul(x, 1103515245) + 12345) >>> 0;
    run.dispatch(enabled[x % enabled.length]!.action);
  }
  collectIcons(run.view(), out);
  run.free();
}

describe('icons', () => {
  it('has a path for every icon key the core can emit', () => {
    const keys = iconsInSource();
    const p = profile.create();
    collectIcons(profile.view(p), keys);
    for (const seed of [1, 2, 3, 5, 8, 13]) iconsFromPlay(seed, keys);
    expect(keys.size).toBeGreaterThan(60);
    const missing = [...keys].filter((k) => !isIconName(k)).sort();
    expect(missing).toEqual([]);
  });

  it('falls back to the generic item for unknown keys', () => {
    expect(iconPath('no_such_icon')).toBe(ICON_PATHS.item);
    expect(iconPath('house')).toBe(ICON_PATHS.house);
  });

  it('draws within the 24 grid with plain path commands', () => {
    for (const name of ICON_NAMES) {
      const d = ICON_PATHS[name];
      expect(d).toMatch(/^M[\d.\sMLHVAZQ-]+$/);
      const nums = d.match(/-?\d+(\.\d+)?/g)!.map(Number);
      for (const n of nums) expect(Math.abs(n)).toBeLessThanOrEqual(24);
    }
  });
});
