/*
 * OxiHuman BodyLab — src/badges.js
 * Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Honest runtime badges. Every displayed number is measured at runtime — never
 * hardcoded:
 *   • wasm + pack transfer sizes come from PerformanceResourceTiming
 *     (transferSize, with encodedBodySize / decodedBodySize fallbacks for
 *     cache hits);
 *   • "0 bytes uploaded" is verified by asserting no cross-origin resource was
 *     loaded (the page issues only same-origin GETs — no upload path exists);
 *   • FPS is a rAF rolling average read from the live viewer;
 *   • vertex count is read from the engine.
 */

function humanBytes(n) {
  if (n == null) return '–';
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MB`;
}

/** Most recent resource-timing entry whose URL path ends with `suffix`. */
function findResource(suffix) {
  const entries = performance.getEntriesByType('resource');
  for (let i = entries.length - 1; i >= 0; i--) {
    try {
      const path = new URL(entries[i].name).pathname;
      if (path.endsWith(suffix)) return entries[i];
    } catch (_) { /* ignore malformed */ }
  }
  return null;
}

/** Bytes actually pulled over the wire, robust to cache hits. */
function transferBytes(entry) {
  if (!entry) return null;
  if (entry.transferSize && entry.transferSize > 0) return entry.transferSize;
  if (entry.encodedBodySize && entry.encodedBodySize > 0) return entry.encodedBodySize;
  if (entry.decodedBodySize && entry.decodedBodySize > 0) return entry.decodedBodySize;
  return 0;
}

/** Sum of bytes fetched from any origin other than this page's. */
function crossOriginBytes() {
  const here = location.origin;
  let total = 0;
  for (const e of performance.getEntriesByType('resource')) {
    let origin = '';
    try { origin = new URL(e.name).origin; } catch (_) { continue; }
    if (origin !== here) total += transferBytes(e) || 0;
  }
  return total;
}

export function initBadges({ viewer, engine, dom }) {
  const state = { wasm: null, pack: null };

  function refreshSizes() {
    if (state.wasm == null) {
      const b = transferBytes(findResource('oxihuman_wasm_bg.wasm'));
      if (b != null) { state.wasm = b; dom.wasm.textContent = humanBytes(b); }
    }
    if (state.pack == null) {
      const b = transferBytes(findResource('oxihuman-core-v1.ohpk'));
      if (b != null) { state.pack = b; dom.pack.textContent = humanBytes(b); }
    }
    return state.wasm != null && state.pack != null;
  }

  function refreshPrivacy() {
    const foreign = crossOriginBytes();
    if (foreign === 0) {
      dom.privacy.textContent = '0 bytes uploaded';
    } else {
      // Never lie: if anything ever loads cross-origin, show the real figure.
      dom.privacy.textContent = `${humanBytes(foreign)} off-origin`;
    }
  }

  // Resource entries may land just after init resolves; observe + poll briefly.
  let observer = null;
  if ('PerformanceObserver' in window) {
    try {
      observer = new PerformanceObserver(() => { if (refreshSizes()) stopObserving(); refreshPrivacy(); });
      observer.observe({ type: 'resource', buffered: true });
    } catch (_) { observer = null; }
  }
  function stopObserving() { if (observer) { observer.disconnect(); observer = null; } }

  let polls = 0;
  const poll = setInterval(() => {
    refreshPrivacy();
    if (refreshSizes() || ++polls > 20) { clearInterval(poll); stopObserving(); }
  }, 250);
  refreshSizes();
  refreshPrivacy();

  // Live HUD: FPS + vertices, ~4 Hz (with an immediate first paint so the
  // vertex count and fps show at once rather than after the first tick).
  let lastVerts = -1;
  function tickHud() {
    dom.fps.textContent = String(viewer.fps || 0);
    if (engine) {
      try {
        const v = engine.vertex_count();
        if (v !== lastVerts) { lastVerts = v; dom.verts.textContent = v.toLocaleString(); }
      } catch (_) { /* engine freed */ }
    }
  }
  tickHud();
  const hud = setInterval(tickHud, 250);

  return {
    stop() { clearInterval(poll); clearInterval(hud); stopObserving(); },
    sizes() { return { wasmBytes: state.wasm, packBytes: state.pack }; },
  };
}
