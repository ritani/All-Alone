// Regenerates every app icon and splash image from assets/mark.svg:
// PWA icons (public/icons), the iOS app icon and launch image, and the
// Android launcher icons (legacy, round, adaptive), adaptive vector
// foreground and splash images.
//
// Renders with a local headless Chrome (no npm dependency): set CHROME to
// its binary if it is not at the default macOS path. Run: npm run icons
import { spawn } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync, copyFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { deflateSync, inflateSync } from 'node:zlib';

/**
 * Run a command and wait for it to exit. stdio is ignored: Chrome can leave a
 * helper holding inherited pipes open, which would stall a wait for 'close'.
 */
function run(cmd, args) {
  return new Promise((resolve, reject) => {
    const p = spawn(cmd, args, { stdio: 'ignore' });
    p.on('error', reject);
    p.on('exit', (code) => (code === 0 ? resolve() : reject(new Error(`${cmd} exited with ${code}`))));
  });
}
const root = new URL('..', import.meta.url).pathname;
const FIELD = '#0b0c0d';
const BONE = '#d8d4c8';

const CHROME =
  process.env.CHROME ??
  ['/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', '/usr/bin/google-chrome', '/usr/bin/chromium'].find(
    (p) => existsSync(p),
  );
if (!CHROME) {
  console.error('No Chrome found. Set CHROME=/path/to/chrome and run again.');
  process.exit(1);
}

// ---------------------------------------------------------------- the mark

const source = readFileSync(join(root, 'assets/mark.svg'), 'utf8');
const paths = [...source.matchAll(/<path d="([^"]+)"\s*\/>/g)].map((m) => m[1]);
const strokeWidth = Number(/stroke-width="([\d.]+)"/.exec(source)?.[1] ?? '4');
if (paths.length === 0) throw new Error('assets/mark.svg: no <path d="..."/> found');
/** The mark's drawn extent on its 100 grid (centre used for placement). */
const BOX = { x: 10, y: 14, w: 80, h: 81 };

/**
 * An SVG of `w` x `h` px: the field (or nothing), and the mark scaled so its
 * box is `frac` of the shorter side, centred. `shape` clips the field.
 */
function compose({ w, h, frac, field = true, shape = 'square' }) {
  const side = Math.min(w, h) * frac;
  const k = side / Math.max(BOX.w, BOX.h);
  const tx = w / 2 - (BOX.x + BOX.w / 2) * k;
  const ty = h / 2 - (BOX.y + BOX.h / 2) * k;
  const bg = !field
    ? ''
    : shape === 'circle'
      ? `<circle cx="${w / 2}" cy="${h / 2}" r="${Math.min(w, h) / 2}" fill="${FIELD}"/>`
      : `<rect width="${w}" height="${h}" fill="${FIELD}"/>`;
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">`,
    bg,
    `<g transform="translate(${tx.toFixed(3)} ${ty.toFixed(3)}) scale(${k.toFixed(5)})" fill="none" stroke="${BONE}"`,
    ` stroke-width="${strokeWidth}" stroke-linecap="square" stroke-linejoin="miter">`,
    ...paths.map((d) => `<path d="${d}"/>`),
    '</g></svg>',
  ].join('');
}

// ---------------------------------------------------------------- PNG helpers

/** Decode an 8-bit, non-interlaced RGB/RGBA PNG to raw RGBA. */
function decodePng(buf) {
  let pos = 8;
  let width = 0;
  let height = 0;
  let type = 0;
  const idat = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos);
    const kind = buf.toString('latin1', pos + 4, pos + 8);
    const data = buf.subarray(pos + 8, pos + 8 + len);
    if (kind === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      if (data[8] !== 8 || data[12] !== 0) throw new Error('unsupported PNG');
      type = data[9];
    } else if (kind === 'IDAT') idat.push(data);
    pos += 12 + len;
  }
  const ch = type === 6 ? 4 : type === 2 ? 3 : 0;
  if (!ch) throw new Error(`unsupported PNG colour type ${type}`);
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * ch;
  const out = Buffer.alloc(width * height * 4);
  let prev = Buffer.alloc(stride);
  for (let y = 0; y < height; y++) {
    const f = raw[y * (stride + 1)];
    const line = Buffer.from(raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1)));
    for (let i = 0; i < stride; i++) {
      const a = i >= ch ? line[i - ch] : 0;
      const b = prev[i];
      const c = i >= ch ? prev[i - ch] : 0;
      let v = line[i];
      if (f === 1) v += a;
      else if (f === 2) v += b;
      else if (f === 3) v += (a + b) >> 1;
      else if (f === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a);
        const pb = Math.abs(p - b);
        const pc = Math.abs(p - c);
        v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      line[i] = v & 255;
    }
    for (let x = 0; x < width; x++) {
      const o = (y * width + x) * 4;
      out[o] = line[x * ch];
      out[o + 1] = line[x * ch + 1];
      out[o + 2] = line[x * ch + 2];
      out[o + 3] = ch === 4 ? line[x * ch + 3] : 255;
    }
    prev = line;
  }
  return { width, height, rgba: out };
}

const CRC = new Int32Array(256).map((_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c;
});
function crc32(buf) {
  let c = -1;
  for (const b of buf) c = CRC[(c ^ b) & 255] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}
function chunk(kind, data) {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(data.length, 0);
  Buffer.from(kind, 'latin1').copy(head, 4);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([head.subarray(4), data])), 0);
  return Buffer.concat([head, data, crc]);
}
/** Encode RGBA as PNG; `alpha: false` writes opaque RGB (App Store icons). */
function encodePng({ width, height, rgba }, alpha) {
  const ch = alpha ? 4 : 3;
  const raw = Buffer.alloc(height * (width * ch + 1));
  for (let y = 0; y < height; y++) {
    raw[y * (width * ch + 1)] = 0;
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      const o = y * (width * ch + 1) + 1 + x * ch;
      raw[o] = rgba[i];
      raw[o + 1] = rgba[i + 1];
      raw[o + 2] = rgba[i + 2];
      if (alpha) raw[o + 3] = rgba[i + 3];
    }
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = alpha ? 6 : 2;
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

// ---------------------------------------------------------------- rendering

const tmp = mkdtempSync(join(tmpdir(), 'all-alone-icons-'));
let seq = 0;

async function render(svg, w, h, out, { alpha = false } = {}) {
  const n = ++seq;
  const svgFile = join(tmp, `in-${n}.svg`);
  const pngFile = join(tmp, `out-${n}.png`);
  writeFileSync(svgFile, svg);
  await run(CHROME, [
    '--headless=new',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-crash-reporter',
    '--disable-breakpad',
    '--disable-component-update',
    '--hide-scrollbars',
    `--user-data-dir=${join(tmp, `profile-${n}`)}`,
    '--force-device-scale-factor=1',
    '--default-background-color=00000000',
    `--window-size=${w},${h}`,
    `--screenshot=${pngFile}`,
    `file://${svgFile}`,
  ]);
  const img = decodePng(readFileSync(pngFile));
  if (img.width !== w || img.height !== h) throw new Error(`${out}: rendered ${img.width}x${img.height}, wanted ${w}x${h}`);
  const dest = join(root, out);
  mkdirSync(dirname(dest), { recursive: true });
  writeFileSync(dest, encodePng(img, alpha));
}

const jobs = [];
const job = (f) => jobs.push(f);

// PWA (public/icons). "any" icons fill the square; maskable keeps the mark
// inside the 80 % safe circle.
for (const s of [192, 512]) job(() => render(compose({ w: s, h: s, frac: 0.62 }), s, s, `public/icons/icon-${s}.png`));
job(() => render(compose({ w: 512, h: 512, frac: 0.5 }), 512, 512, 'public/icons/maskable-512.png'));
job(() => render(compose({ w: 180, h: 180, frac: 0.62 }), 180, 180, 'public/icons/apple-touch-icon.png'));

// iOS: one 1024 opaque icon (Xcode derives the rest) and the launch image.
job(() =>
  render(
    compose({ w: 1024, h: 1024, frac: 0.6 }),
    1024,
    1024,
    'ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png',
  ),
);
const iosSplash = 'ios/App/App/Assets.xcassets/Splash.imageset/splash-2732x2732.png';
job(() => render(compose({ w: 2732, h: 2732, frac: 0.16 }), 2732, 2732, iosSplash));

// Android launcher icons: legacy and round PNGs (API 24-25); API 26+ uses the
// adaptive icon (field colour + the vector foreground written below).
const res = 'android/app/src/main/res';
const densities = { mdpi: 1, hdpi: 1.5, xhdpi: 2, xxhdpi: 3, xxxhdpi: 4 };
for (const [d, k] of Object.entries(densities)) {
  const s = Math.round(48 * k);
  job(() => render(compose({ w: s, h: s, frac: 0.62 }), s, s, `${res}/mipmap-${d}/ic_launcher.png`));
  job(() =>
    render(compose({ w: s, h: s, frac: 0.56, shape: 'circle' }), s, s, `${res}/mipmap-${d}/ic_launcher_round.png`, {
      alpha: true,
    }),
  );
}
// Android splash (pre-12 window background), portrait and landscape.
const splash = {
  mdpi: [320, 480],
  hdpi: [480, 800],
  xhdpi: [720, 1280],
  xxhdpi: [960, 1600],
  xxxhdpi: [1280, 1920],
};
for (const [d, [w, h]] of Object.entries(splash)) {
  job(() => render(compose({ w, h, frac: 0.3 }), w, h, `${res}/drawable-port-${d}/splash.png`));
  job(() => render(compose({ w: h, h: w, frac: 0.3 }), h, w, `${res}/drawable-land-${d}/splash.png`));
}
job(() => render(compose({ w: 480, h: 320, frac: 0.3 }), 480, 320, `${res}/drawable/splash.png`));

// Four renders at a time.
const queue = [...jobs];
await Promise.all(
  Array.from({ length: 4 }, async () => {
    while (queue.length) await queue.shift()();
  }),
);

// The iOS launch image set lists one file per scale; all three are the same.
for (const n of ['splash-2732x2732-1.png', 'splash-2732x2732-2.png']) {
  copyFileSync(join(root, iosSplash), join(root, dirname(iosSplash), n));
}

// Vector forms: the favicon, and Android's adaptive (and themed) foreground
// and Android 12+ splash icon, crisp at any density. The mark sits inside the
// adaptive icon's 66 dp safe zone.
writeFileSync(join(root, 'public/favicon.svg'), compose({ w: 64, h: 64, frac: 0.78 }) + '\n');
const vectorPaths = (frac, size) => {
  const k = (size * frac) / Math.max(BOX.w, BOX.h);
  const tx = size / 2 - (BOX.x + BOX.w / 2) * k;
  const ty = size / 2 - (BOX.y + BOX.h / 2) * k;
  return [
    `    <group android:translateX="${tx.toFixed(3)}" android:translateY="${ty.toFixed(3)}" android:scaleX="${k.toFixed(5)}" android:scaleY="${k.toFixed(5)}">`,
    ...paths.map(
      (d) =>
        `        <path android:pathData="${d}" android:strokeColor="${BONE.toUpperCase()}" android:strokeWidth="${strokeWidth}" android:strokeLineCap="square" android:strokeLineJoin="miter" android:fillColor="#00000000"/>`,
    ),
    '    </group>',
  ].join('\n');
};
const vector = (frac) =>
  [
    '<?xml version="1.0" encoding="utf-8"?>',
    '<!-- Generated by scripts/icons.mjs from assets/mark.svg. Do not edit. -->',
    '<vector xmlns:android="http://schemas.android.com/apk/res/android"',
    '    android:width="108dp"',
    '    android:height="108dp"',
    '    android:viewportWidth="108"',
    '    android:viewportHeight="108">',
    vectorPaths(frac, 108),
    '</vector>',
    '',
  ].join('\n');
writeFileSync(join(root, `${res}/drawable-v24/ic_launcher_foreground.xml`), vector(0.42));
writeFileSync(join(root, `${res}/drawable/splash_mark.xml`), vector(0.42));

rmSync(tmp, { recursive: true, force: true });
console.log(`icons: ${jobs.length} images rendered`);
