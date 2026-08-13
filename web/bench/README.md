# OxiHuman BodyLab — reproducible numbers

Every performance / size number the demo (or its docs) publishes is reproducible
by a stranger with this repo, `node`, `wasm-pack`, and `python3` — no network,
no proprietary tools. This file is the index: for each number, the exact command
that produces it and the value we last measured.

All benches read the **checked-in core pack**
(`assets/packs/oxihuman-core-v1.ohpk`). Sizes are read from real files; the demo
badges read the same figures at runtime from `PerformanceResourceTiming` — the
values here are never hardcoded into the page.

> **Prerequisite:** run `scripts/build_demo.sh` once (compiles the WASM package
> into `demo/pkg/` and copies the pack into `demo/pack/`).

## Numbers & how to reproduce them

| # | Number | Command | Last measured |
|---|--------|---------|---------------|
| 1 | WASM transfer size (raw / gzip) | `web/bench/sizes.sh` | **498.43 KB raw · 208.42 KB gzip** (`oxihuman_wasm_bg.wasm`; Linux, rustc 1.97.1, `wasm-pack build --release`, workspace `[profile.release]` opt-level="z"/lto=true/codegen-units=1) |
| 2 | Core-pack size (raw / gzip) | `web/bench/sizes.sh` | **2.00 MB raw · 1.97 MB gzip** (`oxihuman-core-v1.ohpk`, v3 index-remapped) — the container is already entropy-coded, so gzip barely helps |
| 3 | Engine morph cost (p50 / p95, ms) | `node web/bench/fps_bench.mjs` | **p50 0.52 ms · p95 0.55 ms** → 1935 / 1818 fps CPU headroom |
| 4 | End-to-end demo FPS | live badge on the demo canvas (`hud-fps`); see `fps_overlay.md` | **vsync-bound (≈60 fps)** on WebGL2 hardware |
| 5 | Measurement / fit round-trip Δ | `scripts/measure_roundtrip.sh > docs/bench/measurement-error.md` | worst \|Δ\|: 0.46 cm (self-consistent grid), 1.44 cm (anthropometric grid, adult-XL hip); brief-172 ≤ 0.66 cm (see `docs/bench/measurement-error.md`) |
| 6 | Browser fit wall-clock | `node scripts/wasm_node_check.mjs` (section `[11]`) | **≈0.9 s** for a four-measurement fit (Node) |
| 7 | Pack quantisation error | `oxihuman pack-core --tier core --upstream assets/upstream/makehuman --out assets/packs/oxihuman-core-v1.ohpk --report docs/bench/pack-reconstruction-error.md` | base-mesh max 0.013 mm; worst target 0.011 mm (see `docs/bench/pack-reconstruction-error.md`) |
| 8 | Vertex count | `node scripts/validate_exports.mjs` (header) or the live `verts` badge | **21,833 vertices · 36,972 triangles** |
| 9 | Export well-formedness | `node scripts/validate_exports.mjs` | 25/25 structural checks pass (GLB / VRM / STL / OBJ) |

> Row 2's pack size **defers to the regenerated pack**: `sizes.sh` and the demo
> both read whatever `demo/pack/*.ohpk` currently is. When the pack is rebuilt
> (more measure/ targets, etc.), re-run `web/bench/sizes.sh` and update the
> "last measured" cell — do not hardcode the byte count anywhere in the page.

## Machine note (last-measured column)

The rows above were measured on:

```
Darwin 25.5.0 · Apple M3 · node v23.7.0 · wasm 0.2.1 · release build (wasm-opt)
```

FPS / morph-cost figures are hardware- and thermal-dependent; treat them as
"this class of machine" indicators. Re-run the commands on your machine for
authoritative local numbers. Sizes and error figures are deterministic (they
depend only on the committed pack and the release toolchain).

## The three bench entry points

| File | Measures |
|------|----------|
| `web/bench/sizes.sh` | raw + gzip sizes of `demo/pkg/*.wasm` and `demo/pack/*.ohpk` (#1, #2) |
| `web/bench/fps_bench.mjs` | headless engine morph cost, p50/p95/p99 + derived FPS headroom (#3) — **not** GPU FPS |
| `web/bench/fps_overlay.md` | why the live in-page badge (#4) is the authoritative end-to-end FPS |

Round-trip (#5/#6) and quantisation (#7) live under `docs/bench/` with their own
`scripts/measure_roundtrip.sh` and `pack-core --report` reproducers; they are
indexed here so every published number has one home.
