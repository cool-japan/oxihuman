#!/usr/bin/env node
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// End-to-end Node.js check for the OxiHuman WASM surface.
//
// Usage:
//   1. Build the nodejs package:
//        wasm-pack build --release --target nodejs --out-dir pkg-node \
//            crates/oxihuman-wasm --no-default-features --features bindgen
//   2. Run:
//        node scripts/wasm_node_check.mjs [--pkg <dir>] [--pack <file.ohpk>]
//
//   --pkg   Path to the wasm-pack nodejs output dir
//           (default: crates/oxihuman-wasm/pkg-node, relative to repo root).
//   --pack  Optional real OHPK core pack to load in addition to the
//           synthetic pack (default: assets/packs/oxihuman-core-v1.ohpk,
//           skipped cleanly when absent).
//
// The synthetic OHPK v1 pack is assembled in pure JS (uncompressed body,
// flags = 0) — no Rust helper or fixture file needed.

import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// ─── CLI args ────────────────────────────────────────────────────────────────
const args = process.argv.slice(2);
function argValue(flag, dflt) {
  const i = args.indexOf(flag);
  return i >= 0 && args[i + 1] ? args[i + 1] : dflt;
}
const pkgDir = resolve(argValue("--pkg", join(repoRoot, "crates/oxihuman-wasm/pkg-node")));
const realPackPath = resolve(
  argValue("--pack", join(repoRoot, "assets/packs/oxihuman-core-v1.ohpk"))
);

if (!existsSync(join(pkgDir, "oxihuman_wasm.js"))) {
  console.error(`FATAL: wasm pkg not found at ${pkgDir}`);
  console.error(
    "Build it first: wasm-pack build --release --target nodejs --out-dir pkg-node " +
      "crates/oxihuman-wasm --no-default-features --features bindgen"
  );
  process.exit(2);
}

const require = createRequire(import.meta.url);
const wasm = require(join(pkgDir, "oxihuman_wasm.js"));
// The nodejs glue does not re-export the WebAssembly.Memory; the crate
// exposes it via the `wasm_memory()` binding instead.
const memory = wasm.wasm_memory();

// ─── Tiny test framework ─────────────────────────────────────────────────────
let passed = 0;
let failed = 0;
function check(name, cond, detail = "") {
  if (cond) {
    passed += 1;
    console.log(`  ok   ${name}`);
  } else {
    failed += 1;
    console.error(`  FAIL ${name} ${detail}`);
  }
}

// ─── Synthetic OHPK v1 builder (uncompressed body) ──────────────────────────
class ByteWriter {
  constructor() {
    this.chunks = [];
  }
  u8(v) {
    this.chunks.push(Uint8Array.of(v & 0xff));
  }
  u16(v) {
    const b = new Uint8Array(2);
    new DataView(b.buffer).setUint16(0, v, true);
    this.chunks.push(b);
  }
  i16(v) {
    const b = new Uint8Array(2);
    new DataView(b.buffer).setInt16(0, v, true);
    this.chunks.push(b);
  }
  u32(v) {
    const b = new Uint8Array(4);
    new DataView(b.buffer).setUint32(0, v >>> 0, true);
    this.chunks.push(b);
  }
  f32(v) {
    const b = new Uint8Array(4);
    new DataView(b.buffer).setFloat32(0, v, true);
    this.chunks.push(b);
  }
  bytes(arr) {
    this.chunks.push(arr instanceof Uint8Array ? arr : new Uint8Array(arr));
  }
  uvarint(v) {
    let x = v >>> 0;
    for (;;) {
      const byte = x & 0x7f;
      x >>>= 7;
      if (x === 0) {
        this.u8(byte);
        return;
      }
      this.u8(byte | 0x80);
    }
  }
  concat() {
    const total = this.chunks.reduce((a, c) => a + c.length, 0);
    const out = new Uint8Array(total);
    let off = 0;
    for (const c of this.chunks) {
      out.set(c, off);
      off += c.length;
    }
    return out;
  }
}

/** Per-axis affine quantiser matching core_pack.rs AxisQuant. */
function fitAxis(values) {
  let lo = Infinity;
  let hi = -Infinity;
  for (const v of values) {
    if (v < lo) lo = v;
    if (v > hi) hi = v;
  }
  const span = hi - lo;
  if (span > 0 && Number.isFinite(span)) {
    const scale = span / 65535.0;
    return { scale, bias: lo + 32768.0 * scale };
  }
  return { scale: 0.0, bias: Number.isFinite(lo) ? lo : 0.0 };
}
function quantise(axis, v) {
  if (axis.scale > 0) {
    const t = Math.round((v - axis.bias) / axis.scale);
    return Math.max(-32768, Math.min(32767, t));
  }
  return 0;
}

function buildSyntheticPack({ ageFloorYears = null } = {}) {
  // 8-vertex box, 12 triangles.
  const positions = [
    [-1, 0, -1], [1, 0, -1], [1, 0, 1], [-1, 0, 1],
    [-1, 8, -1], [1, 8, -1], [1, 8, 1], [-1, 8, 1],
  ];
  const indices = [
    0, 1, 2, 0, 2, 3,
    4, 6, 5, 4, 7, 6,
    0, 4, 5, 0, 5, 1,
    1, 5, 6, 1, 6, 2,
    2, 6, 7, 2, 7, 3,
    3, 7, 4, 3, 4, 0,
  ];
  const uvs = positions.map((_, i) => [i / 8, 0.5]);
  const targets = [
    {
      name: "tall",
      category: "height",
      sparse: [
        [4, [0, 2, 0]], [5, [0, 2, 0]], [6, [0, 2, 0]], [7, [0, 2, 0]],
      ],
    },
    { name: "wide", category: "weight", sparse: [[0, [-0.5, 0, 0]], [1, [0.5, 0, 0]]] },
  ];

  const manifest = {
    name: "wasm-node-check",
    version: "1",
    license: "CC0-1.0",
    provenance: { upstream_repo: "", upstream_commit: "", files: [] },
    categories: [],
    target_names: targets.map((t) => t.name),
  };
  if (ageFloorYears !== null) manifest.age_floor_years = ageFloorYears;

  const body = new ByteWriter();
  const manifestBytes = new TextEncoder().encode(JSON.stringify(manifest));
  body.u32(manifestBytes.length);
  body.bytes(manifestBytes);

  // Base mesh section.
  const nVerts = positions.length;
  body.u32(nVerts);
  body.u32(indices.length);
  const axes = [0, 1, 2].map((k) => fitAxis(positions.map((p) => p[k])));
  for (const a of axes) body.f32(a.scale);
  for (const a of axes) body.f32(a.bias);
  let posMaxErr = 0;
  const posQ = [];
  for (const p of positions) {
    for (let k = 0; k < 3; k++) {
      const q = quantise(axes[k], p[k]);
      const deq = q * axes[k].scale + axes[k].bias;
      posMaxErr = Math.max(posMaxErr, Math.abs(p[k] - deq));
      posQ.push(q);
    }
  }
  body.f32(posMaxErr);
  for (const q of posQ) body.i16(q);
  for (const i of indices) body.u32(i);

  // UV section.
  body.u8(1);
  const uvAxes = [0, 1].map((k) => fitAxis(uvs.map((uv) => uv[k])));
  for (const a of uvAxes) body.f32(a.scale);
  for (const a of uvAxes) body.f32(a.bias);
  let uvMaxErr = 0;
  const uvQ = [];
  for (const uv of uvs) {
    for (let k = 0; k < 2; k++) {
      const q = quantise(uvAxes[k], uv[k]);
      const deq = q * uvAxes[k].scale + uvAxes[k].bias;
      uvMaxErr = Math.max(uvMaxErr, Math.abs(uv[k] - deq));
      uvQ.push(q);
    }
  }
  body.f32(uvMaxErr);
  for (const q of uvQ) body.i16(q);

  // No helper metadata.
  body.u8(0);

  // Targets.
  body.u32(targets.length);
  const enc = new TextEncoder();
  for (const t of targets) {
    const sparse = [...t.sparse].sort((a, b) => a[0] - b[0]);
    let maxAbs = 0;
    for (const [, d] of sparse) for (const c of d) maxAbs = Math.max(maxAbs, Math.abs(c));
    const scale = maxAbs > 0 ? maxAbs / 32767.0 : 0.0;
    let maxErr = 0;
    const quantised = sparse.map(([, d]) =>
      d.map((c) => {
        const q = scale > 0 ? Math.max(-32767, Math.min(32767, Math.round(c / scale))) : 0;
        maxErr = Math.max(maxErr, Math.abs(c - q * scale));
        return q;
      })
    );
    const nameB = enc.encode(t.name);
    const catB = enc.encode(t.category);
    body.u16(nameB.length);
    body.bytes(nameB);
    body.u16(catB.length);
    body.bytes(catB);
    body.f32(scale);
    body.f32(maxErr);
    body.u32(sparse.length);
    let prev = 0;
    for (const [idx] of sparse) {
      body.uvarint((idx - prev) >>> 0);
      prev = idx;
    }
    for (const q of quantised) for (const c of q) body.i16(c);
  }

  const bodyBytes = body.concat();
  const out = new ByteWriter();
  out.bytes(new TextEncoder().encode("OHPK"));
  out.u8(1); // version
  out.u8(0); // flags: uncompressed body
  out.u32(bodyBytes.length);
  out.bytes(bodyBytes);
  return out.concat();
}

// ─── GLB / VRM / STL / OBJ validators ────────────────────────────────────────
function parseGlb(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const magic = view.getUint32(0, true);
  const version = view.getUint32(4, true);
  const total = view.getUint32(8, true);
  const jsonLen = view.getUint32(12, true);
  const jsonType = view.getUint32(16, true);
  const jsonText = new TextDecoder().decode(bytes.subarray(20, 20 + jsonLen)).trimEnd();
  const binHeaderOff = 20 + jsonLen;
  const binLen = view.getUint32(binHeaderOff, true);
  const binType = view.getUint32(binHeaderOff + 4, true);
  return {
    magicOk: magic === 0x46546c67,
    versionOk: version === 2,
    totalOk: total === bytes.length,
    jsonTypeOk: jsonType === 0x4e4f534a,
    binTypeOk: binType === 0x004e4942,
    binLenOk: binHeaderOff + 8 + binLen === bytes.length,
    json: JSON.parse(jsonText),
  };
}

// ─── Main checks ─────────────────────────────────────────────────────────────
console.log(`pkg:  ${pkgDir}`);
console.log(`wasm version: ${wasm.get_version()}`);
wasm.set_panic_hook();

console.log("\n[1] synthetic core pack");
const packBytes = buildSyntheticPack({ ageFloorYears: 18 });
const engine = wasm.OxiHumanEngine.from_core_pack_bytes(packBytes);
check("vertex_count matches pack", engine.vertex_count() === 8, `got ${engine.vertex_count()}`);
check("target_count matches pack", engine.target_count() === 2, `got ${engine.target_count()}`);

console.log("\n[2] zero-copy geometry views");
engine.set_param("height", 0.0);
const gen0 = engine.refresh_geometry();
const ptr0 = engine.positions_ptr();
const len0 = engine.positions_len();
check("positions_len == 3 * n_verts", len0 === 24, `got ${len0}`);
let posView = new Float32Array(memory.buffer, ptr0, len0);
const yTopBefore = posView[4 * 3 + 1];
engine.set_param("height", 1.0);
const gen1 = engine.refresh_geometry();
check("generation stable across set_param", gen0 === gen1, `${gen0} -> ${gen1}`);
check("positions_ptr stable across set_param", engine.positions_ptr() === ptr0);
posView = new Float32Array(memory.buffer, engine.positions_ptr(), engine.positions_len());
const yTopAfter = posView[4 * 3 + 1];
check(
  "set_param('height',1.0) changes positions view",
  yTopAfter - yTopBefore > 1.0,
  `dy=${yTopAfter - yTopBefore}`
);
check("normals_len == 3 * n_verts", engine.normals_len() === 24);
check("indices_len == 36", engine.indices_len() === 36);
check("uvs_len == 2 * n_verts", engine.uvs_len() === 16);

console.log("\n[3] age floor");
check("age_floor_years() == 18", engine.age_floor_years() === 18);
engine.set_param("age", 0.0);
const clampedAge = engine.get_param("age");
// years(p) = 1 + 48p for p <= 0.5  →  floor(18y) ≈ 0.3542
check(
  "age param clamped to >= floor",
  clampedAge >= (18 - 1) / 48 - 1e-6,
  `age=${clampedAge}`
);
engine.set_param("age", 0.9);
check("age above floor untouched", Math.abs(engine.get_param("age") - 0.9) < 1e-6);

console.log("\n[4] exports (all in-memory)");
const glb = engine.export_glb();
const glbParsed = parseGlb(glb);
check("GLB magic", glbParsed.magicOk);
check("GLB version 2", glbParsed.versionOk);
check("GLB total length", glbParsed.totalOk);
check("GLB JSON chunk tag", glbParsed.jsonTypeOk);
check("GLB BIN chunk tag", glbParsed.binTypeOk);
check("GLB BIN chunk length", glbParsed.binLenOk);
check("GLB asset 2.0", glbParsed.json.asset?.version === "2.0");

const vrm = engine.export_vrm();
const vrmParsed = parseGlb(vrm);
check("VRM is a valid GLB", vrmParsed.magicOk && vrmParsed.totalOk);
check("VRM has VRMC_vrm extension", !!vrmParsed.json.extensions?.VRMC_vrm);
check(
  "VRM lists VRMC_vrm in extensionsUsed",
  (vrmParsed.json.extensionsUsed || []).includes("VRMC_vrm")
);
const vrm2 = engine.export_vrm_with_options(JSON.stringify({ name: "NodeCheck", authors: ["CI"] }));
const vrm2Parsed = parseGlb(vrm2);
check(
  "VRM options override meta name",
  vrm2Parsed.json.extensions?.VRMC_vrm?.meta?.name === "NodeCheck"
);

const stlBin = engine.export_stl(true);
const triCount = new DataView(stlBin.buffer, stlBin.byteOffset).getUint32(80, true);
check(
  "binary STL triangle count == indices/3",
  triCount === engine.indices_len() / 3,
  `tri=${triCount}`
);
const stlAscii = new TextDecoder().decode(engine.export_stl(false));
check("ASCII STL starts with 'solid'", stlAscii.startsWith("solid "));
check(
  "ASCII STL facet count == indices/3",
  (stlAscii.match(/facet normal/g) || []).length === engine.indices_len() / 3
);

const obj = engine.export_obj();
check("OBJ has v lines", obj.split("\n").some((l) => l.startsWith("v ")));
check("OBJ has f lines", obj.split("\n").some((l) => l.startsWith("f ")));

console.log("\n[5] regression: no 'recursive use of an object' poisoning");
let recursiveUse = false;
const calls = [
  () => engine.build_mesh_bytes(),
  () => engine.export_obj(),
  () => engine.export_glb(),
  () => engine.export_obj(),
  () => engine.export_vrm(),
  () => engine.export_stl(true),
  () => engine.export_stl(false),
  () => engine.get_measurements_json(),
  () => engine.export_obj(),
  () => engine.build_mesh_bytes(),
];
for (const call of calls) {
  try {
    call();
  } catch (e) {
    if (String(e).includes("recursive use of an object")) recursiveUse = true;
  }
}
check("no 'recursive use of an object' across interleaved calls", !recursiveUse);

console.log("\n[6] measurements are centimetres");
const meas = JSON.parse(engine.get_measurements_json());
check("measurements report units=cm", meas.units === "cm");
for (const key of ["total_height", "chest", "waist", "hip", "weight_kg"]) {
  check(`measurements JSON has finite ${key}`, Number.isFinite(meas[key]) && meas[key] >= 0, `${key}=${meas[key]}`);
}
const m = engine.get_measurements();
check(
  "get_measurements() height_cm matches JSON total_height",
  Math.abs(m.height_cm() - meas.total_height) < 1e-3,
  `${m.height_cm()} vs ${meas.total_height}`
);
for (const [key, val] of [
  ["chest_cm", m.chest_cm()],
  ["waist_cm", m.waist_cm()],
  ["hip_cm", m.hip_cm()],
  ["weight_kg", m.weight_kg()],
]) {
  check(`get_measurements() ${key} finite & >= 0`, Number.isFinite(val) && val >= 0, `${key}=${val}`);
}

console.log("\n[7] JSON targets drive the mesh");
engine.load_target_from_json("probe", JSON.stringify({ deltas: [[0, 0, 0, 3]] }));
engine.set_target_weight("probe", 1.0);
engine.refresh_geometry();
const pv = new Float32Array(memory.buffer, engine.positions_ptr(), engine.positions_len());
check("JSON target visible in positions view", Math.abs(pv[2] - (packZ0() + 3)) < 0.05, `z0=${pv[2]}`);
function packZ0() {
  return -1; // vertex 0 z in the synthetic pack
}
engine.unload_target("probe");

console.log("\n[8] sliders / anim player survive engine.free()");
{
  const tmp = wasm.OxiHumanEngine.from_core_pack_bytes(packBytes);
  const slider = wasm.OxiHumanMorphSlider.for_param(tmp, "height");
  const player = tmp.make_anim_player();
  player.record_frame();
  tmp.free(); // must NOT leave slider/player dangling
  let uafSafe = true;
  try {
    slider.set_value(0.7);
    if (Math.abs(slider.value() - 0.7) > 1e-6) uafSafe = false;
    if (player.frame_count() !== 1) uafSafe = false;
  } catch (e) {
    uafSafe = false;
  }
  check("slider+player usable after engine.free() (no UAF)", uafSafe);
}

console.log("\n[9] real upstream base.obj + .target (rayon/apply on wasm32)");
const baseObjPath = join(
  repoRoot,
  "assets/upstream/makehuman/makehuman/data/3dobjs/base.obj"
);
const targetPath = join(
  repoRoot,
  "assets/upstream/makehuman/makehuman/data/targets/macrodetails/universal-female-young-maxmuscle-averageweight.target"
);
if (existsSync(baseObjPath) && existsSync(targetPath)) {
  const eng2 = wasm.OxiHumanEngine.from_obj_bytes(readFileSync(baseObjPath));
  eng2.load_target_bytes("universal-female-young-maxmuscle-averageweight", readFileSync(targetPath));
  eng2.set_param("muscle", 1.0);
  const bytes = eng2.build_mesh_bytes();
  check("real target applies without panic", bytes.length > 0);
  const obj2 = eng2.export_obj();
  check("real mesh OBJ export non-empty", obj2.length > 1000);
  eng2.free();
} else {
  console.log("  skip (assets/upstream not fetched)");
}

console.log("\n[10] real core pack (guarded)");
if (existsSync(realPackPath)) {
  const realBytes = readFileSync(realPackPath);
  const engReal = wasm.OxiHumanEngine.from_core_pack_bytes(realBytes);
  console.log(
    `  real pack: vertex_count=${engReal.vertex_count()} target_count=${engReal.target_count()} age_floor=${engReal.age_floor_years()}`
  );
  check("real pack vertex_count > 0", engReal.vertex_count() > 0);
  check(
    "real pack v3 target_count >= 38 (macro corners + 8 measure/ girth targets)",
    engReal.target_count() >= 38,
    `got ${engReal.target_count()}`
  );
  engReal.set_param("height", 1.0);
  const g = engReal.export_glb();
  check("real pack GLB export", parseGlb(g).magicOk);
  engReal.free();

  // Precise measurement + measurement-driven fit on the real body.
  console.log("\n[11] precise measurements + fit (real pack)");
  const engFit = wasm.OxiHumanEngine.from_core_pack_bytes(realBytes);
  engFit.reset_params();
  const dm = engFit.get_measurements();
  check(
    "default stature is a plausible adult height (cm)",
    dm.height_cm() > 150 && dm.height_cm() < 185,
    `height=${dm.height_cm().toFixed(1)}`
  );
  check(
    "default mass is plausible (kg, helpers excluded)",
    dm.weight_kg() > 40 && dm.weight_kg() < 90,
    `kg=${dm.weight_kg().toFixed(1)}`
  );

  // Self-consistent round trip: measure a body the pack produces, fit to it.
  engFit.set_param("height", 0.6);
  engFit.set_param("weight", 0.55);
  const src = engFit.get_measurements();
  const tgt = JSON.stringify({
    height_cm: src.height_cm(),
    chest_cm: src.chest_cm(),
    waist_cm: src.waist_cm(),
    hip_cm: src.hip_cm(),
  });
  const engFit2 = wasm.OxiHumanEngine.from_core_pack_bytes(realBytes);
  const t0 = Date.now();
  const fit = JSON.parse(engFit2.fit_to_measurements(tgt));
  const fitMs = Date.now() - t0;
  console.log(`  fit took ${fitMs} ms (${fit.iterations} iters, converged=${fit.converged})`);
  check("fit report has params + results", !!fit.params && Array.isArray(fit.results));
  check("fit result count == 4", fit.results.length === 4);
  // v2: the refinement stage reports the driven measure-target weights, each a
  // bounded [0,1] value (0 when the measurer finds no lever — see engine_fit).
  check(
    "fit report has measure_weights object",
    !!fit.measure_weights && typeof fit.measure_weights === "object"
  );
  {
    const mw = fit.measure_weights || {};
    const vals = Object.values(mw);
    check(
      "measure_weights are bounded [0,1]",
      vals.length >= 6 && vals.every((w) => Number.isFinite(w) && w >= 0 && w <= 1),
      `weights=${JSON.stringify(mw)}`
    );
  }
  let selfConsistentOk = true;
  for (const r of fit.results) {
    // delta must be the re-measured geometry, not an echo of the target.
    if (Math.abs(r.delta_cm - (r.measured_cm - r.target_cm)) > 0.06) selfConsistentOk = false;
    if (Math.abs(r.delta_cm) > 2.5) selfConsistentOk = false;
  }
  check("self-consistent fit recovers all measurements <= 2.5 cm", selfConsistentOk);

  // Brief target {172/96/82/98}: height is solved directly (sub-cm) and the
  // v3 pack's correctly re-indexed measure/ targets let the lever-gated
  // refinement close every girth residual to well under 1 cm — see
  // docs/bench/measurement-error.md. The bounds are honest |Δ| ceilings with
  // headroom for cross-platform float drift.
  const brief = JSON.parse(
    engFit2.fit_to_measurements(
      JSON.stringify({ height_cm: 172, chest_cm: 96, waist_cm: 82, hip_cm: 98 })
    )
  );
  const briefBounds = { height: 0.5, chest: 1.5, waist: 1.5, hip: 1.5 };
  for (const [name, bound] of Object.entries(briefBounds)) {
    const r = brief.results.find((x) => x.name === name);
    check(
      `brief ${name} within honest bound (<= ${bound} cm)`,
      Math.abs(r.delta_cm) <= bound,
      `delta=${r.delta_cm}`
    );
  }
  // Non-echo: the reported chest must be the re-measured geometry of the
  // fitted engine, not the requested 96.
  const chest = brief.results.find((r) => r.name === "chest");
  check(
    "brief chest delta equals measured - target (re-measured, not echoed)",
    Math.abs(chest.delta_cm - (chest.measured_cm - chest.target_cm)) <= 0.06,
    `measured=${chest.measured_cm} delta=${chest.delta_cm}`
  );
  const afterFit = engFit2.get_measurements();
  check(
    "brief chest readout matches re-measuring the fitted engine",
    Math.abs(afterFit.chest_cm() - chest.measured_cm) < 0.1,
    `engine=${afterFit.chest_cm()} reported=${chest.measured_cm}`
  );
  engFit.free();
  engFit2.free();
} else {
  console.log(`  skip (${realPackPath} absent)`);
}

engine.free();

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
