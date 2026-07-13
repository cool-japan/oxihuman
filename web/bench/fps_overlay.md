# FPS: the live badge is the real number

There are two different frame-rate figures in this project, and they measure
different things. Keep them straight when quoting numbers.

## 1. Engine morph cost — `web/bench/fps_bench.mjs` (headless)

The bench drives `set_param(...) + refresh_geometry()` in Node over N frames and
reports the wall-clock of the **WASM morph pipeline only** (p50 / p95 / p99),
plus a *derived* "morph headroom" FPS = `1000 / ms`.

This is honest about what it is: the CPU cost of re-morphing the mesh in place
before anything is drawn. It **excludes** GPU rasterisation, buffer upload, and
compositing. It is an upper bound on achievable FPS from the CPU side, not the
rendered frame rate. Label it "engine morph cost", never "GPU FPS".

## 2. End-to-end FPS — the live demo badge (authoritative)

The badge in the bottom-left of the demo canvas (`hud-fps`) is a
`requestAnimationFrame` rolling average of the **whole** frame: OrbitControls
damping + idle breath + (when a slider moved) `refresh_geometry()` + three.js
draw + GPU raster + the browser compositor. On a WebGL2-capable machine this is
vsync-bound (typically 60 fps, or your display's refresh rate) because the
per-frame morph headroom (figure 1) is far below one frame budget.

That badge is computed on the viewer's real render loop
(`demo/src/viewer.js` → `Viewer.fps`) and shown by `demo/src/badges.js`. It is
never hardcoded. It is the number to trust for "how fast does the demo run".

## Why the split is honest

A headless Node process has no GPU surface, so it *cannot* report a real
rendered FPS — quoting `1000 / morph_ms` as "FPS" would overstate performance by
ignoring the GPU. So the reproducible bench reports only what it can measure
truthfully (morph cost), and the deployment metric (end-to-end FPS) is read live
in the browser where a GPU actually exists.
