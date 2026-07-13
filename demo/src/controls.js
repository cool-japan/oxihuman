/*
 * OxiHuman BodyLab — src/controls.js
 * Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Wires the control panel to the engine + viewer:
 *   • live [0,1] parameter sliders (height / weight / muscle / age / gender)
 *     — every change is a set_param + viewer.requestMorph(), so the actual
 *     geometry refresh is throttled to the render loop's rAF (60 fps morph);
 *   • presets (engine.apply_preset + Reset);
 *   • measurement fit (engine.fit_to_measurements, ~1.3 s, behind an explicit
 *     button + spinner) with an honest re-measured Δ readout;
 *   • GLB / VRM / STL / OBJ exports, downloaded straight from WASM memory.
 */

const $ = (id) => document.getElementById(id);

function ageParamToYears(p) {
  return p <= 0.5 ? 1 + p * 48 : 25 + (p - 0.5) * 130;
}
function ageYearsToParam(y) {
  const v = y <= 25 ? (y - 1) / 48 : 0.5 + (y - 25) / 130;
  return Math.min(Math.max(v, 0), 1);
}

const PARAMS = [
  { key: 'height', label: 'Height', fmt: (p) => `${Math.round(p * 100)}%` },
  { key: 'weight', label: 'Weight', fmt: (p) => `${Math.round(p * 100)}%` },
  { key: 'muscle', label: 'Muscle', fmt: (p) => `${Math.round(p * 100)}%` },
  { key: 'age', label: 'Age', fmt: (p) => `${Math.round(ageParamToYears(p))} y` },
  { key: 'gender', label: 'Gender', fmt: (p) => (p <= 0.02 ? 'masc' : p >= 0.98 ? 'fem' : `${Math.round(p * 100)}% ♀`) },
];

function downloadBytes(data, filename, mime) {
  const blob = new Blob([data], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.rel = 'noopener';
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 15000);
  return blob.size;
}

function deltaClass(d) {
  const a = Math.abs(d);
  if (a <= 0.5) return 'delta-good';
  if (a <= 1.5) return 'delta-warn';
  return 'delta-bad';
}

export function initControls({ engine, viewer }) {
  const floorYears = engine.age_floor_years();
  const ageFloorParam = floorYears != null ? ageYearsToParam(floorYears) : 0;

  // ── Build live sliders ────────────────────────────────────────────────
  const slidersEl = $('sliders');
  const rows = {};
  for (const p of PARAMS) {
    let value = 0.5;
    try { const v = engine.get_param(p.key); if (Number.isFinite(v)) value = v; } catch (_) { /* default */ }

    const row = document.createElement('div');
    row.className = 'slider-row';
    const min = p.key === 'age' ? ageFloorParam : 0;

    const top = document.createElement('div');
    top.className = 'slider-top';
    const label = document.createElement('label');
    label.textContent = p.label;
    label.setAttribute('for', `sl-${p.key}`);
    const val = document.createElement('span');
    val.className = 'slider-val';
    val.id = `slval-${p.key}`;
    top.append(label, val);

    const input = document.createElement('input');
    input.type = 'range';
    input.id = `sl-${p.key}`;
    input.min = String(min);
    input.max = '1';
    input.step = '0.001';
    input.value = String(Math.max(value, min));
    input.setAttribute('aria-label', p.label);

    const sync = () => { val.textContent = p.fmt(Number(input.value)); };
    input.addEventListener('input', () => {
      const v = Number(input.value);
      try { engine.set_param(p.key, v); } catch (_) { /* ignore */ }
      sync();
      viewer.requestMorph();
      clearActivePreset();
    });

    sync();
    row.append(top, input);
    slidersEl.appendChild(row);
    rows[p.key] = { input, sync };
  }

  // Gender lives in the engine's `extra` map and is unset until first written
  // (get_param returns NaN, gender_slider defaults to 0.5). Commit the shown
  // default so the slider, the mesh and any fit share one baseline.
  try { engine.set_param('gender', Number(rows.gender.input.value)); } catch (_) { /* ignore */ }

  function syncSlidersFromEngine() {
    for (const p of PARAMS) {
      try {
        const v = engine.get_param(p.key);
        if (Number.isFinite(v)) {
          const min = p.key === 'age' ? ageFloorParam : 0;
          rows[p.key].input.value = String(Math.max(v, min));
        }
      } catch (_) { /* ignore */ }
      rows[p.key].sync();
    }
  }

  // ── Presets ───────────────────────────────────────────────────────────
  const presetBtns = Array.from(document.querySelectorAll('#presets [data-preset]'));
  function clearActivePreset() { presetBtns.forEach((b) => b.classList.remove('active')); }
  for (const btn of presetBtns) {
    btn.addEventListener('click', () => {
      const name = btn.dataset.preset;
      try {
        if (name === '__reset') engine.reset_params();
        else engine.apply_preset(name);
      } catch (_) { /* ignore */ }
      clearActivePreset();
      btn.classList.add('active');
      syncSlidersFromEngine();
      viewer.requestMorph();
    });
    btn.disabled = false;
  }

  // ── Measurement fit ───────────────────────────────────────────────────
  const fitBtn = $('btn-fit');
  const readout = $('fit-readout');
  const readoutBody = $('fit-readout-body');
  const fitMeta = $('fit-meta');
  const inputs = { height: $('m-height'), chest: $('m-chest'), waist: $('m-waist'), hip: $('m-hip') };

  function collectTargets() {
    const t = {};
    const map = { height: 'height_cm', chest: 'chest_cm', waist: 'waist_cm', hip: 'hip_cm' };
    for (const [k, el] of Object.entries(inputs)) {
      const v = Number(el.value);
      if (Number.isFinite(v) && v > 0) t[map[k]] = v;
    }
    return t;
  }

  async function runFit() {
    const targets = collectTargets();
    if (Object.keys(targets).length === 0) return;

    fitBtn.classList.add('busy');
    fitBtn.disabled = true;
    fitBtn.querySelector('.btn-label').textContent = 'Fitting…';
    // Yield one frame so the busy state paints before the blocking WASM call.
    await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));

    let report = null;
    let err = null;
    try {
      report = JSON.parse(engine.fit_to_measurements(JSON.stringify(targets)));
    } catch (e) {
      err = e;
    }

    fitBtn.classList.remove('busy');
    fitBtn.disabled = false;
    fitBtn.querySelector('.btn-label').textContent = 'Fit body';

    if (err) {
      fitMeta.hidden = false;
      fitMeta.textContent = `Fit failed: ${err && err.message ? err.message : err}`;
      return;
    }

    // Honest re-measured readout, straight from the returned JSON.
    readoutBody.replaceChildren();
    const labels = { height: 'Height', chest: 'Chest', waist: 'Waist', hip: 'Hip' };
    for (const r of report.results) {
      const tr = document.createElement('tr');
      const name = document.createElement('td'); name.textContent = labels[r.name] || r.name;
      const tgt = document.createElement('td'); tgt.textContent = `${r.target_cm.toFixed(1)}`;
      const mea = document.createElement('td'); mea.textContent = `${r.measured_cm.toFixed(1)}`;
      const del = document.createElement('td');
      del.className = deltaClass(r.delta_cm);
      del.textContent = `${r.delta_cm > 0 ? '+' : ''}${r.delta_cm.toFixed(2)}`;
      tr.append(name, tgt, mea, del);
      readoutBody.appendChild(tr);
    }
    readout.hidden = false;
    fitMeta.hidden = false;
    fitMeta.textContent = `${report.iterations} iterations · ${report.converged ? 'converged' : 'stopped at cap'} · cm re-measured from geometry`;

    syncSlidersFromEngine();
    clearActivePreset();
    viewer.requestMorph();
  }

  fitBtn.addEventListener('click', () => { runFit(); });
  fitBtn.disabled = false;
  for (const el of Object.values(inputs)) {
    el.addEventListener('keydown', (e) => { if (e.key === 'Enter') runFit(); });
  }

  // ── Exports ───────────────────────────────────────────────────────────
  const status = $('export-status');
  const exporters = [
    { id: 'ex-glb', name: 'oxihuman.glb', mime: 'model/gltf-binary', run: () => engine.export_glb() },
    { id: 'ex-vrm', name: 'oxihuman.vrm', mime: 'model/gltf-binary', run: () => engine.export_vrm() },
    { id: 'ex-stl', name: 'oxihuman.stl', mime: 'model/stl', run: () => engine.export_stl(true) },
    { id: 'ex-obj', name: 'oxihuman.obj', mime: 'text/plain;charset=utf-8', run: () => engine.export_obj() },
  ];
  for (const ex of exporters) {
    const btn = $(ex.id);
    btn.addEventListener('click', () => {
      try {
        const data = ex.run();
        const size = downloadBytes(data, ex.name, ex.mime);
        status.textContent = `Saved ${ex.name} — ${(size / 1024).toFixed(0)} KB, on-device.`;
      } catch (e) {
        status.textContent = `Export failed: ${e && e.message ? e.message : e}`;
      }
    });
    btn.disabled = false;
  }
}
