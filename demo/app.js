/*
 * OxiHuman BodyLab — app.js  (module entry point)
 * Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Boot flow:
 *   1. dynamic-import the wasm-pack glue (./pkg/…) so a missing build shows a
 *      graceful, actionable error panel instead of a blank page;
 *   2. init() the module, install the panic hook;
 *   3. fetch the OHPK core pack with a real progress bar;
 *   4. OxiHumanEngine.from_core_pack_bytes → three.js viewer → controls → badges;
 *   5. start the single rAF render loop and register the offline service worker.
 */

import { Viewer } from './src/viewer.js';
import { initControls } from './src/controls.js';
import { initBadges } from './src/badges.js';

const WASM_JS = './pkg/oxihuman_wasm.js';
const PACK_URL = './pack/oxihuman-core-v1.ohpk';
const BUILD_CMD = 'scripts/build_demo.sh';

const $ = (id) => document.getElementById(id);

function setLoading(title, sub, pct) {
  const t = $('loading-title'); if (t && title != null) t.textContent = title;
  const s = $('loading-sub'); if (s && sub != null) s.textContent = sub;
  const bar = $('loading-bar'); if (bar && pct != null) bar.style.width = `${Math.round(pct * 100)}%`;
}

function hideLoading() {
  const el = $('loading');
  if (!el) return;
  el.classList.add('hide');
  setTimeout(() => { el.hidden = true; }, 480);
}

function showError(detail, err) {
  if (err) console.error('[OxiHuman BodyLab]', detail, err);
  const loading = $('loading'); if (loading) loading.hidden = true;
  const panel = $('error-panel');
  const d = $('error-detail');
  if (d) d.textContent = `${detail} Expected build output at demo/pkg/ and demo/pack/.`;
  if (panel) panel.hidden = false;
}

async function fetchWithProgress(url, onProgress) {
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`${resp.status} ${resp.statusText} for ${url}`);
  const total = Number(resp.headers.get('content-length')) || 0;
  if (!resp.body || !total) {
    const buf = new Uint8Array(await resp.arrayBuffer());
    onProgress(1);
    return buf;
  }
  const reader = resp.body.getReader();
  const chunks = [];
  let received = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    received += value.length;
    onProgress(total ? received / total : 0.5);
  }
  const out = new Uint8Array(received);
  let off = 0;
  for (const c of chunks) { out.set(c, off); off += c.length; }
  return out;
}

async function registerServiceWorker() {
  if (!('serviceWorker' in navigator)) return;
  if (location.protocol !== 'http:' && location.protocol !== 'https:') return;
  try {
    await navigator.serviceWorker.register('./sw.js');
  } catch (e) {
    console.warn('[OxiHuman BodyLab] service worker registration failed:', e);
  }
}

async function boot() {
  setLoading('Starting OxiHuman…', 'Loading the WebAssembly engine.', 0.05);

  // 1. WASM glue (dynamic import → graceful failure if the demo is unbuilt).
  let wasm;
  try {
    wasm = await import(WASM_JS);
  } catch (e) {
    return showError(`The WebAssembly package is not built (run ${BUILD_CMD}).`, e);
  }

  // 2. init + panic hook.
  try {
    await wasm.default();
  } catch (e) {
    return showError('The WebAssembly module failed to initialise.', e);
  }
  wasm.set_panic_hook();
  const version = wasm.get_version();
  const foot = $('foot-version'); if (foot) foot.textContent = `OxiHuman ${version}`;
  setLoading('Fetching the core pack…', 'A CC0 anthropometric model, downloaded once.', 0.35);

  // 3. Fetch the core pack with a progress bar.
  let packBytes;
  try {
    packBytes = await fetchWithProgress(PACK_URL, (p) => setLoading(null, null, 0.35 + p * 0.55));
  } catch (e) {
    return showError(`The core pack could not be fetched (run ${BUILD_CMD}).`, e);
  }

  // 4. Engine + viewer + controls + badges.
  setLoading('Reconstructing the body…', 'Decoding the pack and building the mesh.', 0.94);
  let engine;
  try {
    engine = wasm.OxiHumanEngine.from_core_pack_bytes(packBytes);
  } catch (e) {
    return showError('The core pack could not be decoded.', e);
  }

  const memory = wasm.wasm_memory();
  const canvas = $('stage-canvas');
  let viewer;
  try {
    viewer = new Viewer(canvas);
    viewer.setEngine(engine, memory);
  } catch (e) {
    return showError('WebGL2 is required and could not be initialised.', e);
  }

  initControls({ engine, viewer });
  initBadges({
    viewer,
    engine,
    dom: {
      fps: $('hud-fps'),
      verts: $('hud-verts'),
      wasm: $('hud-wasm'),
      pack: $('hud-pack'),
      privacy: $('privacy-badge'),
    },
  });

  // 5. Single render loop.
  function frame(now) {
    viewer.render(now);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  setLoading('Ready', null, 1);
  hideLoading();
  registerServiceWorker();
}

boot().catch((e) => showError('Unexpected startup error.', e));
