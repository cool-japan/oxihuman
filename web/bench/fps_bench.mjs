#!/usr/bin/env node
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Headless ENGINE MORPH COST benchmark.
//
// This measures the per-frame cost of the WASM morph pipeline only — the work
// the live demo does every frame BEFORE three.js uploads to the GPU:
//   set_param(...) → refresh_geometry()  (incremental, in-place).
// It is NOT a GPU rendering FPS number. The live demo's on-canvas FPS badge is
// the true end-to-end (CPU morph + GPU raster + compositor) figure; see
// web/bench/fps_overlay.md.
//
// Usage:
//   node web/bench/fps_bench.mjs [--frames N] [--pkg <dir>] [--pack <file>]

import { createRequire } from 'node:module';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import os from 'node:os';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const args = process.argv.slice(2);
const argValue = (flag, dflt) => {
  const i = args.indexOf(flag);
  return i >= 0 && args[i + 1] ? args[i + 1] : dflt;
};

const frames = Math.max(30, Number(argValue('--frames', '400')) | 0);
const pkgDir = resolve(argValue('--pkg', join(repoRoot, 'crates/oxihuman-wasm/pkg-node')));
const packPath = resolve(argValue('--pack', join(repoRoot, 'assets/packs/oxihuman-core-v1.ohpk')));

if (!existsSync(join(pkgDir, 'oxihuman_wasm.js'))) {
  console.log(`[fps_bench] nodejs pkg missing at ${pkgDir} — building…`);
  execFileSync(
    'wasm-pack',
    ['build', '--release', '--target', 'nodejs', '--out-dir', pkgDir,
      join(repoRoot, 'crates/oxihuman-wasm'), '--no-default-features', '--features', 'bindgen'],
    { stdio: 'inherit' },
  );
}
if (!existsSync(packPath)) {
  console.error(`FATAL: core pack not found at ${packPath}. Run scripts/build_demo.sh first.`);
  process.exit(2);
}

const require = createRequire(import.meta.url);
const wasm = require(join(pkgDir, 'oxihuman_wasm.js'));
wasm.set_panic_hook();

const engine = wasm.OxiHumanEngine.from_core_pack_bytes(readFileSync(packPath));
const verts = engine.vertex_count();

// Warm up (first build allocates the persistent buffers).
for (let i = 0; i < 20; i++) {
  engine.set_param('weight', 0.5 + 0.4 * Math.sin(i * 0.3));
  engine.refresh_geometry();
}

// Measure: vary parameters each frame so refresh_geometry does real work
// (a no-op refresh would measure nothing).
const durMs = new Array(frames);
for (let i = 0; i < frames; i++) {
  const t = i / frames;
  engine.set_param('weight', 0.5 + 0.45 * Math.sin(t * Math.PI * 8));
  engine.set_param('muscle', 0.5 + 0.35 * Math.sin(t * Math.PI * 5 + 1));
  engine.set_param('gender', 0.5 + 0.45 * Math.sin(t * Math.PI * 3 + 2));
  const t0 = process.hrtime.bigint();
  engine.refresh_geometry();
  const t1 = process.hrtime.bigint();
  durMs[i] = Number(t1 - t0) / 1e6;
}
engine.free();

durMs.sort((a, b) => a - b);
const pct = (p) => durMs[Math.min(durMs.length - 1, Math.floor((p / 100) * durMs.length))];
const mean = durMs.reduce((a, b) => a + b, 0) / durMs.length;
const p50 = pct(50);
const p95 = pct(95);
const p99 = pct(99);

const fmt = (x) => x.toFixed(3);
const fps = (ms) => (ms > 0 ? (1000 / ms).toFixed(0) : '∞');

console.log('OxiHuman — engine morph cost (headless, WASM only)');
console.log(`  machine     : ${os.type()} ${os.release()} · ${os.cpus()[0]?.model?.trim() || 'unknown CPU'} · node ${process.version}`);
console.log(`  wasm        : ${wasm.get_version()}`);
console.log(`  vertices    : ${verts.toLocaleString()}`);
console.log(`  frames      : ${frames}`);
console.log('');
console.log(`  set_param + refresh_geometry per frame (ms):`);
console.log(`    min  ${fmt(durMs[0])}   mean ${fmt(mean)}`);
console.log(`    p50  ${fmt(p50)}   p95  ${fmt(p95)}   p99  ${fmt(p99)}`);
console.log('');
console.log(`  derived engine-morph headroom (ignores GPU/compositor):`);
console.log(`    p50 → ${fps(p50)} fps    p95 → ${fps(p95)} fps`);
console.log('');
console.log('  NOTE: engine morph cost only. True end-to-end FPS is the live');
console.log('        demo badge (web/bench/fps_overlay.md).');
