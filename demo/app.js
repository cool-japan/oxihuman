/**
 * OxiHuman Demo — app.js
 * Copyright (C) 2026 COOLJAPAN OU (Team Kitasan)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Loads the WASM module (from pkg/ when built with wasm-pack),
 * wires HTML sliders to engine parameters, renders a projected
 * wireframe on a 2-D canvas, and handles export downloads.
 *
 * The engine is accessed through a thin facade defined below.
 * When the WASM module is not yet built, a pure-JS fallback
 * procedural mesh is used so the page remains functional.
 */

// ── Constants ────────────────────────────────────────────────────────────────

const WASM_JS_PATH = './pkg/oxihuman_wasm.js';

// Fallback mesh colours
const WIREFRAME_COLOR   = '#5c8df6';
const WIREFRAME_OPACITY = 0.55;

// ── Slider descriptors ───────────────────────────────────────────────────────

/** @typedef {{ id: string, valId: string, paramKey: string, fmt: (v:number)=>string }} SliderDesc */

/** @type {SliderDesc[]} */
const SLIDERS = [
  {
    id:       'sl-height',
    valId:    'val-height',
    paramKey: 'height',
    fmt:      v => `${(v / 100).toFixed(2)} m`,
  },
  {
    id:       'sl-weight',
    valId:    'val-weight',
    paramKey: 'weight',
    fmt:      v => `${v} kg`,
  },
  {
    id:       'sl-age',
    valId:    'val-age',
    paramKey: 'age',
    fmt:      v => `${v}`,
  },
  {
    id:       'sl-muscle',
    valId:    'val-muscle',
    paramKey: 'muscle_mass',
    fmt:      v => `${v}%`,
  },
  {
    id:       'sl-fat',
    valId:    'val-fat',
    paramKey: 'body_fat',
    fmt:      v => `${v}%`,
  },
];

// ── Alpha preset definitions ──────────────────────────────────────────────────

/** @type {Record<string, Record<string, number>>} */
const ALPHA_PRESETS = {
  athletic: { height: 180, weight: 80,  age: 28, 'sl-muscle': 75, 'sl-fat': 12 },
  slim:     { height: 175, weight: 60,  age: 25, 'sl-muscle': 30, 'sl-fat': 10 },
  heavy:    { height: 175, weight: 110, age: 40, 'sl-muscle': 45, 'sl-fat': 38 },
  tall:     { height: 210, weight: 85,  age: 30, 'sl-muscle': 50, 'sl-fat': 18 },
  short:    { height: 155, weight: 58,  age: 30, 'sl-muscle': 50, 'sl-fat': 18 },
  reset:    { height: 175, weight: 70,  age: 30, 'sl-muscle': 50, 'sl-fat': 18 },
};

// ── Engine facade ─────────────────────────────────────────────────────────────

/**
 * Thin facade over the WASM engine (or the JS fallback).
 * The WASM module is expected to export:
 *   - OxiHumanEngine (class)
 *       .new()                    -> OxiHumanEngine
 *       .set_param(key, value)    -> void
 *       .build_mesh_bytes()       -> Uint8Array  (raw f32 vertex positions)
 *       .export_glb()             -> Uint8Array
 *       .export_obj()             -> Uint8Array
 *       .export_stl()             -> Uint8Array
 *       .vertex_count()           -> number
 *       .memory_bytes()           -> number
 */
class EngineProxy {
  /** @param {object|null} wasmEngine  null → use JS fallback */
  constructor(wasmEngine) {
    this._wasm   = wasmEngine;
    this._params = {
      height:      175,
      weight:      70,
      age:         30,
      muscle_mass: 50,
      body_fat:    18,
    };
    this._mesh = null; // cached mesh
    this._dirty = true;
  }

  setParam(key, value) {
    this._params[key] = value;
    this._dirty = true;
    if (this._wasm) {
      try { this._wasm.set_param(key, value); } catch (_) { /* ignore */ }
    }
  }

  /** Returns { positions: Float32Array, edges: Uint16Array } */
  getMesh() {
    if (!this._dirty && this._mesh) return this._mesh;
    this._dirty = false;

    if (this._wasm) {
      try {
        const raw = this._wasm.build_mesh_bytes();
        this._mesh = parseMeshBytes(raw);
        return this._mesh;
      } catch (_) { /* fall through to JS fallback */ }
    }

    this._mesh = buildFallbackMesh(this._params);
    return this._mesh;
  }

  vertexCount() {
    const m = this.getMesh();
    return m ? m.positions.length / 3 : 0;
  }

  memoryBytes() {
    if (this._wasm) {
      try { return this._wasm.memory_bytes(); } catch (_) { /* ignore */ }
    }
    const m = this.getMesh();
    return m ? m.positions.byteLength + m.edges.byteLength : 0;
  }

  /** @returns {Uint8Array} */
  exportGlb() {
    if (this._wasm) {
      try { return this._wasm.export_glb(); } catch (_) { /* ignore */ }
    }
    return exportFallbackGlb(this._params);
  }

  /** @returns {Uint8Array} */
  exportObj() {
    if (this._wasm) {
      try { return this._wasm.export_obj(); } catch (_) { /* ignore */ }
    }
    return exportFallbackObj(this.getMesh());
  }

  /** @returns {Uint8Array} */
  exportStl() {
    if (this._wasm) {
      try { return this._wasm.export_stl(); } catch (_) { /* ignore */ }
    }
    return exportFallbackStl(this.getMesh());
  }
}

// ── Mesh binary format (from WASM) ────────────────────────────────────────────

/**
 * Parse the raw mesh bytes produced by build_mesh_bytes().
 * Format: [u32 vertex_count][u32 edge_count][f32... positions][u16... edge_indices]
 *
 * @param {Uint8Array} raw
 * @returns {{ positions: Float32Array, edges: Uint16Array }}
 */
function parseMeshBytes(raw) {
  const dv = new DataView(raw.buffer, raw.byteOffset, raw.byteLength);
  const vertCount = dv.getUint32(0, true);
  const edgeCount = dv.getUint32(4, true);

  const posOffset  = 8;
  const edgeOffset = posOffset + vertCount * 3 * 4;

  const positions = new Float32Array(
    raw.buffer, raw.byteOffset + posOffset, vertCount * 3
  );
  const edges = new Uint16Array(
    raw.buffer, raw.byteOffset + edgeOffset, edgeCount * 2
  );
  return { positions, edges };
}

// ── JS fallback mesh ──────────────────────────────────────────────────────────

/**
 * Build a very simple procedural stick-figure-like wireframe body
 * as a fallback when the WASM module is absent.
 *
 * The body is represented as a set of 3-D points + edges.
 * Units: metres, centred at origin, y = up.
 *
 * @param {Record<string, number>} params
 * @returns {{ positions: Float32Array, edges: Uint16Array }}
 */
function buildFallbackMesh(params) {
  const h      = (params.height      ?? 175) / 175;   // normalised
  const w      = (params.weight      ??  70) / 70;    // normalised
  const muscle = (params.muscle_mass ??  50) / 100;   // 0-1
  const fat    = (params.body_fat    ??  18) / 100;   // 0-1

  // Shoulder width influenced by muscle / fat
  const sw = 0.22 * (1 + muscle * 0.4 + fat * 0.3) * w;
  // Hip width
  const hw = 0.20 * (1 + fat * 0.5) * w;
  // Torso depth
  const td = 0.12 * (1 + fat * 0.6) * w;

  // Key skeleton points (x, y, z) — y = up
  const pts = [
    // 0: head top
    [0,      0.90 * h,  0],
    // 1: head bottom / neck
    [0,      0.78 * h,  0],
    // 2: left shoulder
    [-sw,    0.72 * h,  0],
    // 3: right shoulder
    [ sw,    0.72 * h,  0],
    // 4: centre chest
    [0,      0.72 * h,  0],
    // 5: left elbow
    [-sw * 1.1, 0.52 * h, 0],
    // 6: right elbow
    [ sw * 1.1, 0.52 * h, 0],
    // 7: left wrist
    [-sw * 1.15, 0.33 * h, 0],
    // 8: right wrist
    [ sw * 1.15, 0.33 * h, 0],
    // 9: left hip
    [-hw,    0.48 * h,  0],
    // 10: right hip
    [ hw,    0.48 * h,  0],
    // 11: centre waist
    [0,      0.50 * h,  0],
    // 12: left knee
    [-hw * 0.9, 0.26 * h, 0],
    // 13: right knee
    [ hw * 0.9, 0.26 * h, 0],
    // 14: left ankle
    [-hw * 0.8, 0.03 * h, 0],
    // 15: right ankle
    [ hw * 0.8, 0.03 * h, 0],
    // 16: centre groin
    [0,      0.48 * h,  0],
  ];

  // Head circle (8 points) — indices 17..24
  const HEAD_CIRCLE_RADIUS = 0.07 * h;
  const HEAD_CY = 0.845 * h;
  const HEAD_N  = 8;
  for (let i = 0; i < HEAD_N; i++) {
    const angle = (i / HEAD_N) * Math.PI * 2;
    pts.push([
      HEAD_CIRCLE_RADIUS * Math.cos(angle),
      HEAD_CY + HEAD_CIRCLE_RADIUS * Math.sin(angle),
      0,
    ]);
  }

  // Torso outline (4 points) — indices 25..28
  pts.push([-sw,    0.72 * h,  td]);   // 25 left shoulder back
  pts.push([ sw,    0.72 * h,  td]);   // 26 right shoulder back
  pts.push([-hw,    0.48 * h,  td]);   // 27 left hip back
  pts.push([ hw,    0.48 * h,  td]);   // 28 right hip back

  // Flatten positions
  const positions = new Float32Array(pts.length * 3);
  for (let i = 0; i < pts.length; i++) {
    positions[i * 3 + 0] = pts[i][0];
    positions[i * 3 + 1] = pts[i][1];
    positions[i * 3 + 2] = pts[i][2];
  }

  // Skeleton edges (pairs of point indices)
  const rawEdges = [
    // Spine
    [1, 4], [4, 11], [11, 16],
    // Head neck
    [0, 1],
    // Shoulders to neck
    [1, 2], [1, 3], [2, 4], [3, 4],
    // Left arm
    [2, 5], [5, 7],
    // Right arm
    [3, 6], [6, 8],
    // Hips
    [11, 9], [11, 10], [9, 16], [10, 16],
    // Left leg
    [9, 12], [12, 14],
    // Right leg
    [10, 13], [13, 15],
    // Torso depth lines
    [2, 25], [3, 26], [9, 27], [10, 28],
    [25, 26], [26, 28], [28, 27], [27, 25],
  ];

  // Head circle edges
  for (let i = 0; i < HEAD_N; i++) {
    rawEdges.push([17 + i, 17 + (i + 1) % HEAD_N]);
  }

  const edges = new Uint16Array(rawEdges.length * 2);
  for (let i = 0; i < rawEdges.length; i++) {
    edges[i * 2 + 0] = rawEdges[i][0];
    edges[i * 2 + 1] = rawEdges[i][1];
  }

  return { positions, edges };
}

// ── Fallback exporters ────────────────────────────────────────────────────────

/** @param {Record<string, number>} params @returns {Uint8Array} */
function exportFallbackGlb(params) {
  // Minimal valid GLB: JSON chunk with an empty scene, binary chunk empty.
  const json = JSON.stringify({
    asset: { version: '2.0', generator: 'OxiHuman-alpha-demo' },
    scene: 0,
    scenes: [{ name: 'OxiHuman', nodes: [] }],
    extensionsUsed: [],
    extras: { params },
  });
  const jsonBytes = new TextEncoder().encode(json);
  const paddedLen = (jsonBytes.length + 3) & ~3;
  const jsonChunk = new Uint8Array(paddedLen);
  jsonChunk.set(jsonBytes);
  for (let i = jsonBytes.length; i < paddedLen; i++) jsonChunk[i] = 0x20; // pad with spaces

  // Header: magic(4) + version(4) + length(4)
  // JSON chunk: chunkLength(4) + chunkType(4) + chunkData
  // BIN chunk:  chunkLength(4) + chunkType(4) (length 0)
  const totalLen = 12 + 8 + paddedLen + 8;
  const buf = new ArrayBuffer(totalLen);
  const dv  = new DataView(buf);
  let off = 0;

  // Header
  dv.setUint32(off, 0x46546C67, false); off += 4; // 'glTF'
  dv.setUint32(off, 2, true);            off += 4; // version 2
  dv.setUint32(off, totalLen, true);     off += 4;

  // JSON chunk
  dv.setUint32(off, paddedLen, true);    off += 4;
  dv.setUint32(off, 0x4E4F534A, false); off += 4; // 'JSON'
  new Uint8Array(buf, off, paddedLen).set(jsonChunk); off += paddedLen;

  // BIN chunk (empty)
  dv.setUint32(off, 0, true);            off += 4;
  dv.setUint32(off, 0x004E4942, false); off += 4; // 'BIN\0'

  return new Uint8Array(buf);
}

/**
 * @param {{ positions: Float32Array, edges: Uint16Array }} mesh
 * @returns {Uint8Array}
 */
function exportFallbackObj(mesh) {
  const lines = ['# OxiHuman alpha demo export'];
  const vCount = mesh.positions.length / 3;
  for (let i = 0; i < vCount; i++) {
    const x = mesh.positions[i * 3 + 0].toFixed(6);
    const y = mesh.positions[i * 3 + 1].toFixed(6);
    const z = mesh.positions[i * 3 + 2].toFixed(6);
    lines.push(`v ${x} ${y} ${z}`);
  }
  const eCount = mesh.edges.length / 2;
  for (let i = 0; i < eCount; i++) {
    const a = mesh.edges[i * 2 + 0] + 1; // OBJ is 1-indexed
    const b = mesh.edges[i * 2 + 1] + 1;
    lines.push(`l ${a} ${b}`);
  }
  return new TextEncoder().encode(lines.join('\n') + '\n');
}

/**
 * @param {{ positions: Float32Array, edges: Uint16Array }} mesh
 * @returns {Uint8Array}
 */
function exportFallbackStl(mesh) {
  // ASCII STL: each edge becomes a degenerate triangle (for compatibility).
  const lines = ['solid oxihuman_alpha'];
  const eCount = mesh.edges.length / 2;
  for (let i = 0; i < eCount; i++) {
    const a = mesh.edges[i * 2 + 0];
    const b = mesh.edges[i * 2 + 1];
    const ax = mesh.positions[a * 3], ay = mesh.positions[a * 3 + 1], az = mesh.positions[a * 3 + 2];
    const bx = mesh.positions[b * 3], by = mesh.positions[b * 3 + 1], bz = mesh.positions[b * 3 + 2];
    lines.push(
      'facet normal 0 0 1',
      '  outer loop',
      `    vertex ${ax.toFixed(6)} ${ay.toFixed(6)} ${az.toFixed(6)}`,
      `    vertex ${bx.toFixed(6)} ${by.toFixed(6)} ${bz.toFixed(6)}`,
      `    vertex ${ax.toFixed(6)} ${ay.toFixed(6)} ${az.toFixed(6)}`,
      '  endloop',
      'endfacet',
    );
  }
  lines.push('endsolid oxihuman_alpha');
  return new TextEncoder().encode(lines.join('\n') + '\n');
}

// ── 3-D → 2-D projection ──────────────────────────────────────────────────────

/**
 * Project a single 3-D point to 2-D canvas coordinates.
 * Simple perspective projection.
 *
 * @param {number} x @param {number} y @param {number} z
 * @param {number} cx canvas centre x @param {number} cy canvas centre y
 * @param {number} scale  @param {number} fov  camera z offset
 * @returns {{ px: number, py: number }}
 */
function project(x, y, z, cx, cy, scale, fov) {
  const zOff = fov;
  const denom = zOff + z;
  const px = cx + (x / denom) * scale * fov;
  const py = cy - (y / denom) * scale * fov;
  return { px, py };
}

// ── Renderer ─────────────────────────────────────────────────────────────────

/** Manages rotation state, animation loop, and canvas drawing. */
class Renderer {
  /**
   * @param {HTMLCanvasElement} canvas
   * @param {EngineProxy} engine
   */
  constructor(canvas, engine) {
    this._canvas  = canvas;
    this._ctx     = canvas.getContext('2d');
    this._engine  = engine;
    this._angleY  = 0.3;  // radians, current yaw
    this._dragging = false;
    this._lastX   = 0;
    this._raf     = null;
    this._fpsTs   = performance.now();
    this._fpsCount = 0;
    this._fpsValue = 0;

    this._attachPointerEvents();
  }

  _attachPointerEvents() {
    const c = this._canvas;
    c.style.cursor = 'grab';

    c.addEventListener('pointerdown', e => {
      this._dragging = true;
      this._lastX    = e.clientX;
      c.style.cursor = 'grabbing';
      c.setPointerCapture(e.pointerId);
    });

    c.addEventListener('pointermove', e => {
      if (!this._dragging) return;
      const dx = e.clientX - this._lastX;
      this._lastX = e.clientX;
      this._angleY += dx * 0.008;
    });

    const endDrag = () => {
      this._dragging = false;
      c.style.cursor = 'grab';
    };
    c.addEventListener('pointerup',     endDrag);
    c.addEventListener('pointercancel', endDrag);
  }

  start() {
    const loop = () => {
      this._raf = requestAnimationFrame(loop);
      this._render();
    };
    this._raf = requestAnimationFrame(loop);
  }

  stop() {
    if (this._raf !== null) {
      cancelAnimationFrame(this._raf);
      this._raf = null;
    }
  }

  _render() {
    const canvas = this._canvas;
    const ctx    = this._ctx;
    const W      = canvas.width;
    const H      = canvas.height;

    ctx.clearRect(0, 0, W, H);

    const mesh = this._engine.getMesh();
    if (!mesh) return;

    const cx    = W / 2;
    const cy    = H / 2;
    const scale = Math.min(W, H) * 0.72;
    const fov   = 4.5;

    const ay = this._angleY;
    const cosY = Math.cos(ay);
    const sinY = Math.sin(ay);

    // Pre-project all vertices
    const vCount = mesh.positions.length / 3;
    const projected = new Array(vCount);
    for (let i = 0; i < vCount; i++) {
      const x0 = mesh.positions[i * 3 + 0];
      const y0 = mesh.positions[i * 3 + 1];
      const z0 = mesh.positions[i * 3 + 2];
      // Rotate around Y axis
      const rx =  x0 * cosY + z0 * sinY;
      const ry =  y0;
      const rz = -x0 * sinY + z0 * cosY;
      projected[i] = project(rx, ry, rz, cx, cy, scale, fov);
    }

    // Draw edges
    ctx.beginPath();
    ctx.strokeStyle = WIREFRAME_COLOR;
    ctx.globalAlpha = WIREFRAME_OPACITY;
    ctx.lineWidth   = 1.5;

    const eCount = mesh.edges.length / 2;
    for (let i = 0; i < eCount; i++) {
      const a = mesh.edges[i * 2 + 0];
      const b = mesh.edges[i * 2 + 1];
      if (a >= vCount || b >= vCount) continue;
      ctx.moveTo(projected[a].px, projected[a].py);
      ctx.lineTo(projected[b].px, projected[b].py);
    }
    ctx.stroke();

    // Draw vertices
    ctx.globalAlpha = 0.9;
    ctx.fillStyle = '#8f6bf8';
    for (let i = 0; i < vCount; i++) {
      const { px, py } = projected[i];
      ctx.beginPath();
      ctx.arc(px, py, 2.5, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.globalAlpha = 1;

    // Tick FPS
    this._fpsCount++;
    const now = performance.now();
    const elapsed = now - this._fpsTs;
    if (elapsed >= 500) {
      this._fpsValue = Math.round((this._fpsCount * 1000) / elapsed);
      this._fpsCount = 0;
      this._fpsTs    = now;
    }

    // Auto-rotate when not dragging
    if (!this._dragging) {
      this._angleY += 0.003;
    }
  }

  get fps() { return this._fpsValue; }
}

// ── Stats updater ─────────────────────────────────────────────────────────────

function formatBytes(n) {
  if (n < 1024)        return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MB`;
}

// ── Blob download helper ──────────────────────────────────────────────────────

/**
 * @param {Uint8Array} bytes
 * @param {string}     filename
 * @param {string}     mimeType
 */
function triggerDownload(bytes, filename, mimeType) {
  const blob = new Blob([bytes], { type: mimeType });
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href     = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

// ── Preset application ────────────────────────────────────────────────────────

/**
 * @param {string} presetName
 * @param {EngineProxy} engine
 */
function applyPreset(presetName, engine) {
  const preset = ALPHA_PRESETS[presetName];
  if (!preset) return;

  for (const [sliderKey, value] of Object.entries(preset)) {
    // sliderKey is either a paramKey ('height', 'weight', 'age')
    // or a slider element id ('sl-muscle', 'sl-fat')
    let elemId   = sliderKey;
    let paramKey = sliderKey;

    if (!sliderKey.startsWith('sl-')) {
      // Find matching slider descriptor
      const desc = SLIDERS.find(s => s.paramKey === sliderKey);
      if (desc) { elemId = desc.id; }
    } else {
      const desc = SLIDERS.find(s => s.id === sliderKey);
      if (desc) { paramKey = desc.paramKey; }
    }

    const el = document.getElementById(elemId);
    if (el) {
      el.value = value;
      el.dispatchEvent(new Event('input'));
    } else {
      engine.setParam(paramKey, value);
    }
  }
}

// ── WASM loader ───────────────────────────────────────────────────────────────

/**
 * Attempt to load the WASM module.  Returns the engine instance or null.
 * @returns {Promise<object|null>}
 */
async function tryLoadWasm() {
  try {
    const mod = await import(WASM_JS_PATH);
    await mod.default(); // wasm-pack init
    const engine = mod.OxiHumanEngine
      ? mod.OxiHumanEngine.new()
      : null;
    return engine;
  } catch (_) {
    return null;
  }
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
  const loadingScreen = document.getElementById('loading-screen');
  const loadingMsg    = document.getElementById('loading-msg');
  const statusDot     = document.getElementById('status-dot');
  const fpsDisplay    = document.getElementById('fps-display');
  const statVerts     = document.getElementById('stat-verts');
  const statFps       = document.getElementById('stat-fps');
  const statMem       = document.getElementById('stat-mem');

  // Attempt WASM load (non-fatal)
  loadingMsg.textContent = 'Loading WASM module…';
  const wasmEngine = await tryLoadWasm();

  if (!wasmEngine) {
    loadingMsg.textContent = 'WASM not built — using JS fallback renderer.';
  } else {
    loadingMsg.textContent = 'Initialising engine…';
  }

  const engine   = new EngineProxy(wasmEngine);
  const canvas   = document.getElementById('render-canvas');
  const renderer = new Renderer(canvas, engine);

  // Resize canvas to fill its container
  function resizeCanvas() {
    const parent = canvas.parentElement;
    const rect   = parent.getBoundingClientRect();
    canvas.width  = Math.floor(rect.width);
    canvas.height = Math.floor(rect.height);
  }
  resizeCanvas();
  new ResizeObserver(resizeCanvas).observe(canvas.parentElement);

  // Wire sliders
  for (const desc of SLIDERS) {
    const el = document.getElementById(desc.id);
    const vEl = document.getElementById(desc.valId);
    if (!el || !vEl) continue;

    const update = () => {
      const v = Number(el.value);
      vEl.textContent = desc.fmt(v);
      engine.setParam(desc.paramKey, v);
    };

    el.addEventListener('input', update);
    update(); // initial sync
  }

  // Wire preset buttons
  for (const btn of document.querySelectorAll('[data-preset]')) {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.preset-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      applyPreset(btn.dataset.preset, engine);
    });
  }

  // Wire export buttons
  document.getElementById('btn-glb').addEventListener('click', () => {
    triggerDownload(engine.exportGlb(), 'oxihuman.glb', 'model/gltf-binary');
  });
  document.getElementById('btn-obj').addEventListener('click', () => {
    triggerDownload(engine.exportObj(), 'oxihuman.obj', 'text/plain');
  });
  document.getElementById('btn-stl').addEventListener('click', () => {
    triggerDownload(engine.exportStl(), 'oxihuman.stl', 'model/stl');
  });

  // Start render loop
  renderer.start();

  // Stats ticker (every 500 ms)
  setInterval(() => {
    const fps   = renderer.fps;
    const verts = engine.vertexCount();
    const mem   = engine.memoryBytes();
    fpsDisplay.textContent  = fps;
    statVerts.textContent   = verts.toLocaleString();
    statFps.textContent     = fps;
    statMem.textContent     = formatBytes(mem);
  }, 500);

  // Hide loading screen
  loadingScreen.style.transition = 'opacity 0.4s ease';
  loadingScreen.style.opacity    = '0';
  setTimeout(() => {
    loadingScreen.style.display = 'none';
    statusDot.style.display     = 'block';
  }, 400);
}

main().catch(err => {
  console.error('[OxiHuman] Fatal error:', err);
  const msg = document.getElementById('loading-msg');
  if (msg) msg.textContent = 'Error: ' + err.message;
});
