# OxiHuman — Measurement & Fit Round-Trip Error

Honest per-measurement error for `WasmEngine::fit_to_measurements` against the shipped core pack. Values come from the precise cross-section measurer (`oxihuman_morph::measurements::CrossSectionMeasurer`): chest / waist / hip are convex-hull tape circumferences of the torso cross-section (helper geometry excluded, torso isolated per slice by central-axis containment plus a torso-width guard that rejects arm-merged hulls), height is stature, and mass is body-volume × human mean density.

## Measurement band conventions

Tape convention per girth — the band **extremum** slice, no trimming, so a genuine localised girth change (e.g. a `measure/` morph target) is observed, not cancelled:

* **chest** — fullest torso slice of the anatomical bust band (≈ 0.66–0.76 of stature, up to the armpit line). Arm cross-sections are laterally offset clusters that fail the central-axis test and are rejected per slice; slices where the A-pose arms merge into the chest hull are rejected by the torso width guard (> 0.27 × stature wide).
* **waist** — narrowest torso slice of the natural-waist band (≈ 0.58–0.70 of stature); collapsed slivers (< 0.08 × stature wide) at extreme pinch morphs are rejected.
* **hip** — fullest torso slice across the pelvis (≈ 0.505–0.60 of stature, above the crotch so the section is one central hull, not two legs).

These bands are verified against ground-truth application of the upstream MakeHuman `measure/` girth targets (see `tests/measurement_fit.rs`): correctly applied, `measure-bust-circ-incr` moves the measured chest by ≈ +16 cm at weight 1, `measure-waist-circ-incr/decr` move the waist by ≈ ±8 cm, `measure-hips-circ-incr` moves the hip by ≈ +19 cm, with no leakage into the other girths.

## Changelog

* **v3 pack — core-pack index corruption fixed.** Earlier packs encoded `.target` vertex ids in raw MakeHuman v-line order while the pack's base mesh stores vertices in the OBJ loader's face-first-occurrence order (21 833 packed verts after UV-seam splits vs 19 158 v-lines), so every pack target was applied to permuted vertices and the localised `measure/` girth targets deformed noise instead of their girth. `oxihuman-cli pack-core` now re-indexes every target through the loader's raw→packed mapping (duplicating each delta across seam copies; verified by the `pack_core` invariant tests), the lever-gated girth refinement engages end-to-end, and Grid B's former multi-centimetre pack-data ceiling is gone — every girth residual below is sub-1.5 cm and `brief-172` is sub-0.7 cm.

* Pack: `assets/packs/oxihuman-core-v1.ohpk` (v3 — 8 `measure/` girth targets, indices remapped to the packed base mesh)
* Optimiser: `height` solved directly on the monotone stature response (bisection), Nelder–Mead over `weight, muscle, gender` with height re-trimmed between passes, then a lever-gated coordinate-descent **refinement** over the `measure/` bust / underbust / waist / hips target weights
* `Δ = measured − target`, re-measured from the fully-refined mesh (never echoed)
* `*` after the iteration count = hit the iteration cap without simplex convergence

## Worst case (self-consistent grid)

| Measurement | Worst \|Δ\| (cm) | Case |
|---|---:|---|
| chest | 0.27 | heavy |
| height | 0.10 | petite |
| hip | 0.46 | heavy |
| waist | 0.42 | heavy |

Median fit time: **838 ms** (native release, indicative — wall-clock varies with machine load / thermal state). The browser figure is the deployment metric: `scripts/wasm_node_check.mjs` reports a full four-measurement fit at **~0.9 s** under Node.

## Grid A — self-consistent round trip

Each row fits to the measurements of a body the pack actually produces, so a small Δ confirms the measurer and optimiser recover it faithfully.

| Case | Measurement | Target (cm) | Measured (cm) | Δ (cm) | Iters | ms |
|---|---|---:|---:|---:|---:|---:|
| neutral | height | 164.6 | 164.6 | -0.00 | 60* | 769 |
|  | chest | 83.4 | 83.4 | +0.00 |  |  |
|  | waist | 66.4 | 66.4 | -0.01 |  |  |
|  | hip | 94.5 | 94.5 | +0.00 |  |  |
| tall | height | 187.7 | 187.7 | +0.00 | 58* | 705 |
|  | chest | 91.8 | 91.8 | -0.00 |  |  |
|  | waist | 73.0 | 73.0 | +0.01 |  |  |
|  | hip | 101.5 | 101.5 | -0.00 |  |  |
| heavy | height | 171.8 | 171.8 | -0.04 | 67* | 890 |
|  | chest | 87.4 | 87.7 | +0.27 |  |  |
|  | waist | 69.7 | 70.1 | +0.42 |  |  |
|  | hip | 95.4 | 95.9 | +0.46 |  |  |
| lean | height | 176.2 | 176.2 | -0.01 | 67* | 934 |
|  | chest | 85.2 | 85.2 | +0.00 |  |  |
|  | waist | 68.6 | 68.6 | -0.01 |  |  |
|  | hip | 93.7 | 93.7 | +0.01 |  |  |
| muscular | height | 179.4 | 179.5 | +0.09 | 67* | 823 |
|  | chest | 88.8 | 88.8 | -0.03 |  |  |
|  | waist | 69.9 | 70.0 | +0.12 |  |  |
|  | hip | 97.2 | 97.1 | -0.11 |  |  |
| feminine | height | 170.3 | 170.3 | -0.04 | 64* | 816 |
|  | chest | 85.6 | 85.7 | +0.06 |  |  |
|  | waist | 68.4 | 68.1 | -0.27 |  |  |
|  | hip | 95.2 | 95.3 | +0.14 |  |  |
| masculine | height | 180.6 | 180.6 | +0.00 | 67* | 793 |
|  | chest | 88.6 | 88.6 | +0.01 |  |  |
|  | waist | 71.4 | 71.4 | -0.00 |  |  |
|  | hip | 101.0 | 101.0 | -0.00 |  |  |
| petite | height | 166.7 | 166.6 | -0.10 | 67* | 838 |
|  | chest | 81.9 | 81.8 | -0.13 |  |  |
|  | waist | 65.6 | 65.4 | -0.15 |  |  |
|  | hip | 91.2 | 91.5 | +0.33 |  |  |
| broad | height | 186.0 | 186.0 | +0.05 | 67* | 814 |
|  | chest | 92.8 | 92.9 | +0.10 |  |  |
|  | waist | 74.2 | 74.0 | -0.23 |  |  |
|  | hip | 102.0 | 102.1 | +0.07 |  |  |
| slim-tall | height | 193.3 | 193.3 | -0.03 | 67* | 817 |
|  | chest | 91.8 | 91.8 | +0.01 |  |  |
|  | waist | 73.5 | 73.4 | -0.06 |  |  |
|  | hip | 99.2 | 99.2 | +0.04 |  |  |

## Grid B — anthropometric targets

Realistic adult tape measurements. Height is solved directly and lands within a millimetre-scale error everywhere inside the pack's stature range; the lever-gated `measure/` refinement then closes each girth residual against the re-measured geometry (the v3 pack's re-indexed targets move exactly the bands the measurer reads — see the ground-truth and pack-response tests in `tests/measurement_fit.rs`). Residuals are honest re-measurements, never echoes of the request; the worst remaining case is `adult-XL`'s hip, at the edge of the pack's reachable girth envelope. `brief-172` is the brief's `{height 172, chest 96, waist 82, hip 98}` probe.

| Case | Measurement | Target (cm) | Measured (cm) | Δ (cm) | Iters | ms |
|---|---|---:|---:|---:|---:|---:|
| adult-S | height | 165.0 | 164.6 | -0.41 | 67* | 1238 |
|  | chest | 84.0 | 83.4 | -0.63 |  |  |
|  | waist | 70.0 | 69.8 | -0.16 |  |  |
|  | hip | 90.0 | 90.2 | +0.24 |  |  |
| adult-M | height | 172.0 | 172.0 | -0.00 | 67* | 1182 |
|  | chest | 90.0 | 89.6 | -0.40 |  |  |
|  | waist | 76.0 | 75.5 | -0.52 |  |  |
|  | hip | 96.0 | 95.4 | -0.56 |  |  |
| adult-L | height | 180.0 | 179.9 | -0.09 | 67* | 1133 |
|  | chest | 96.0 | 96.5 | +0.46 |  |  |
|  | waist | 82.0 | 81.5 | -0.47 |  |  |
|  | hip | 100.0 | 99.7 | -0.29 |  |  |
| adult-XL | height | 188.0 | 187.7 | -0.29 | 67* | 1290 |
|  | chest | 104.0 | 104.9 | +0.87 |  |  |
|  | waist | 90.0 | 89.7 | -0.27 |  |  |
|  | hip | 108.0 | 106.6 | -1.44 |  |  |
| brief-172 | height | 172.0 | 171.9 | -0.05 | 67* | 1204 |
|  | chest | 96.0 | 95.3 | -0.66 |  |  |
|  | waist | 82.0 | 81.7 | -0.28 |  |  |
|  | hip | 98.0 | 97.6 | -0.41 |  |  |

## Reproduce

```sh
scripts/measure_roundtrip.sh > docs/bench/measurement-error.md
```

The script runs `cargo run --release --example measure_roundtrip -p oxihuman-wasm` and requires only the checked-in core pack — no network, no external tools.
