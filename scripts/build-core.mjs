// Builds the Rust game core to WebAssembly in src/core/pkg (git-ignored).
// Needs cargo, the wasm32-unknown-unknown target and wasm-pack on PATH.
// Runs a local wasm-opt -Oz afterwards when one is installed (optional).
import { execFileSync } from 'node:child_process';
import { existsSync, statSync } from 'node:fs';
import { delimiter, join } from 'node:path';

const root = new URL('..', import.meta.url).pathname;
const out = join(root, 'src/core/pkg');
const wasm = join(out, 'all_alone_core_bg.wasm');

function onPath(cmd) {
  return (process.env.PATH ?? '').split(delimiter).some((dir) => existsSync(join(dir, cmd)));
}

const run = (cmd, args) => execFileSync(cmd, args, { cwd: root, stdio: 'inherit' });

if (!onPath('wasm-pack')) {
  console.error('wasm-pack not found. Put ~/.cargo/bin (and the rustup bin) on PATH; see README.');
  process.exit(1);
}

run('wasm-pack', [
  'build',
  'crates/core',
  '--target',
  'web',
  '--release',
  '--no-pack',
  '--out-dir',
  '../../src/core/pkg',
  '--out-name',
  'all_alone_core',
  '--',
  '--features',
  'wasm',
]);

if (onPath('wasm-opt')) {
  run('wasm-opt', ['-Oz', '--strip-debug', '--strip-producers', '--enable-bulk-memory', '--enable-nontrapping-float-to-int', '--enable-sign-ext', '--enable-mutable-globals', wasm, '-o', wasm]);
} else {
  console.warn('wasm-opt not found: skipping the extra size pass.');
}

console.log(`core: ${(statSync(wasm).size / 1024).toFixed(1)} KiB wasm`);
