#!/usr/bin/env node
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Structural validator for the four BodyLab exporters (GLB / VRM / STL / OBJ).
// Loads the REAL core pack, morphs the body, exercises every exporter, and
// checks the resulting bytes are well-formed AND internally consistent with the
// engine's own vertex / index counts. Exits non-zero on any failure.
//
// Usage:
//   node scripts/validate_exports.mjs [--pkg <nodejs-pkg-dir>] [--pack <file.ohpk>]
//
//   --pkg   wasm-pack --target nodejs output dir
//           (default: crates/oxihuman-wasm/pkg-node). Built automatically when
//           absent.
//   --pack  OHPK core pack (default: assets/packs/oxihuman-core-v1.ohpk).

import { createRequire } from 'node:module';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const argValue = (flag, dflt) => {
  const i = args.indexOf(flag);
  return i >= 0 && args[i + 1] ? args[i + 1] : dflt;
};

const pkgDir = resolve(argValue('--pkg', join(repoRoot, 'crates/oxihuman-wasm/pkg-node')));
const packPath = resolve(argValue('--pack', join(repoRoot, 'assets/packs/oxihuman-core-v1.ohpk')));

// ── Ensure a nodejs pkg exists (build it if needed) ──────────────────────────
if (!existsSync(join(pkgDir, 'oxihuman_wasm.js'))) {
  console.log(`[validate_exports] nodejs pkg missing at ${pkgDir} — building…`);
  execFileSync(
    'wasm-pack',
    [
      'build', '--release', '--target', 'nodejs',
      '--out-dir', pkgDir,
      join(repoRoot, 'crates/oxihuman-wasm'),
      '--no-default-features', '--features', 'bindgen',
    ],
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

// ── Tiny assertion harness ───────────────────────────────────────────────────
let passed = 0;
let failed = 0;
function check(name, cond, detail = '') {
  if (cond) { passed += 1; console.log(`  ok   ${name}`); }
  else { failed += 1; console.error(`  FAIL ${name} ${detail}`); }
}

// ── GLB parser (magic / version / chunk layout + JSON) ───────────────────────
function parseGlb(bytes) {
  if (bytes.length < 20) throw new Error('GLB too short');
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const magic = view.getUint32(0, true);
  const version = view.getUint32(4, true);
  const total = view.getUint32(8, true);
  const jsonLen = view.getUint32(12, true);
  const jsonType = view.getUint32(16, true);
  const jsonText = new TextDecoder().decode(bytes.subarray(20, 20 + jsonLen)).replace(/\0+$/, '').trimEnd();
  const binHeaderOff = 20 + jsonLen;
  const binLen = view.getUint32(binHeaderOff, true);
  const binType = view.getUint32(binHeaderOff + 4, true);
  return {
    magic, version, total,
    magicOk: magic === 0x46546c67, // 'glTF'
    versionOk: version === 2,
    totalOk: total === bytes.length,
    jsonTypeOk: jsonType === 0x4e4f534a, // 'JSON'
    binTypeOk: binType === 0x004e4942, // 'BIN\0'
    binLenOk: binHeaderOff + 8 + binLen === bytes.length,
    json: JSON.parse(jsonText),
  };
}

// Find the POSITION and indices accessor counts for the first mesh primitive.
function glbPrimitiveCounts(json) {
  const prim = json.meshes?.[0]?.primitives?.[0];
  if (!prim) return { posCount: null, idxCount: null };
  const acc = json.accessors || [];
  const posCount = prim.attributes?.POSITION != null ? acc[prim.attributes.POSITION]?.count ?? null : null;
  const idxCount = prim.indices != null ? acc[prim.indices]?.count ?? null : null;
  return { posCount, idxCount };
}

// ── Load engine + a non-degenerate morph ─────────────────────────────────────
console.log(`pkg:  ${pkgDir}`);
console.log(`pack: ${packPath}`);
console.log(`wasm: ${wasm.get_version()}`);

const packBytes = readFileSync(packPath);
const engine = wasm.OxiHumanEngine.from_core_pack_bytes(packBytes);
engine.set_param('height', 0.62);
engine.set_param('weight', 0.55);
engine.set_param('muscle', 0.6);
engine.refresh_geometry();

const vertexCount = engine.vertex_count();
const indexCount = engine.indices_len();
const triCount = indexCount / 3;
console.log(`\nengine: vertices=${vertexCount} indices=${indexCount} triangles=${triCount}`);
check('engine vertex count > 0', vertexCount > 0, `got ${vertexCount}`);
check('engine index count divisible by 3', indexCount % 3 === 0, `got ${indexCount}`);

// ── GLB ──────────────────────────────────────────────────────────────────────
console.log('\n[GLB]');
{
  const glb = engine.export_glb();
  const g = parseGlb(glb);
  check('GLB magic == glTF', g.magicOk);
  check('GLB version == 2', g.versionOk, `got ${g.version}`);
  check('GLB header total == byte length', g.totalOk, `total=${g.total} len=${glb.length}`);
  check('GLB JSON chunk tag', g.jsonTypeOk);
  check('GLB BIN chunk tag', g.binTypeOk);
  check('GLB BIN chunk length invariant', g.binLenOk);
  check('GLB asset.version == 2.0', g.json.asset?.version === '2.0', JSON.stringify(g.json.asset));
  const { posCount, idxCount } = glbPrimitiveCounts(g.json);
  check('GLB POSITION accessor count == engine vertex count', posCount === vertexCount, `glb=${posCount} engine=${vertexCount}`);
  check('GLB indices accessor count == engine index count', idxCount === indexCount, `glb=${idxCount} engine=${indexCount}`);
}

// ── VRM ────────────────────────────────────────────────────────────────────────
console.log('\n[VRM]');
{
  const vrm = engine.export_vrm();
  const g = parseGlb(vrm);
  check('VRM is a valid GLB', g.magicOk && g.versionOk && g.totalOk && g.binLenOk);
  const ext = g.json.extensions?.VRMC_vrm;
  check('VRM has VRMC_vrm extension', !!ext);
  check('VRM lists VRMC_vrm in extensionsUsed', (g.json.extensionsUsed || []).includes('VRMC_vrm'));
  const humanBones = ext?.humanoid?.humanBones;
  check('VRM humanoid.humanBones present', !!humanBones && typeof humanBones === 'object');
  const boneNames = humanBones ? Object.keys(humanBones) : [];
  check('VRM declares core humanoid bones (incl. hips)', boneNames.includes('hips') && boneNames.length >= 5, `bones=${boneNames.length}`);
  const { posCount } = glbPrimitiveCounts(g.json);
  check('VRM POSITION accessor count == engine vertex count', posCount === vertexCount, `vrm=${posCount} engine=${vertexCount}`);
}

// ── STL (binary) ─────────────────────────────────────────────────────────────
console.log('\n[STL binary]');
{
  const stl = engine.export_stl(true);
  check('STL has 80-byte header + count field', stl.length >= 84, `len=${stl.length}`);
  const dv = new DataView(stl.buffer, stl.byteOffset, stl.byteLength);
  const declaredTris = dv.getUint32(80, true);
  check('STL triangle-count field == index_count/3', declaredTris === triCount, `stl=${declaredTris} engine=${triCount}`);
  // Binary STL byte invariant: 80 header + 4 count + 50 bytes per triangle.
  const expectedLen = 84 + declaredTris * 50;
  check('STL byte length == 84 + 50 * triangles', stl.length === expectedLen, `len=${stl.length} expected=${expectedLen}`);
}

// ── OBJ ──────────────────────────────────────────────────────────────────────
console.log('\n[OBJ]');
{
  const obj = engine.export_obj();
  check('OBJ is a non-empty string', typeof obj === 'string' && obj.length > 0);
  const lines = obj.split('\n');
  let vCount = 0;
  let fCount = 0;
  let badFace = 0;
  let badLine = 0;
  const known = new Set(['v', 'vn', 'vt', 'vp', 'f', 'l', 'p', 'o', 'g', 's', 'mtllib', 'usemtl', '#']);
  for (const raw of lines) {
    const line = raw.trim();
    if (line === '') continue;
    const tok = line.split(/\s+/)[0];
    if (!known.has(tok)) { badLine += 1; continue; }
    if (tok === 'v') vCount += 1;
    else if (tok === 'f') {
      fCount += 1;
      const refs = line.split(/\s+/).length - 1;
      if (refs !== 3) badFace += 1;
    }
  }
  check('OBJ v-line count == engine vertex count', vCount === vertexCount, `obj=${vCount} engine=${vertexCount}`);
  check('OBJ f-line count == engine triangle count', fCount === triCount, `obj=${fCount} engine=${triCount}`);
  check('OBJ every face is a triangle', badFace === 0, `${badFace} non-triangle faces`);
  check('OBJ has no unrecognised tokens (line-clean)', badLine === 0, `${badLine} bad lines`);
}

engine.free();

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
