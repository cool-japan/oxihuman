# Vendored third-party runtime dependencies

The BodyLab demo is a **static, build-step-free** page: every runtime
dependency is committed here so the demo works offline after first load with
**no CDN references at runtime** (Content-Security-Policy friendly, no external
origins). Do not add `<script src="https://...">` or `import` from a remote URL
anywhere in the demo.

## three.js

| | |
|---|---|
| **Package** | [`three`](https://www.npmjs.com/package/three) |
| **Version** | r160 (`0.160.0`) |
| **License** | MIT — Copyright 2010-2024 three.js Authors |
| **Renderer** | WebGL2 (`WebGLRenderer`) — **not** WebGPU |

### `three.module.min.js`

* **Source URL:** <https://unpkg.com/three@0.160.0/build/three.module.min.js>
* **SHA-256:** `3e690ac7d180b0aadf0891bea39eec643e29e2d3e75c99b18689518665f69ba6`

### `OrbitControls.js`

* **Source URL:** <https://unpkg.com/three@0.160.0/examples/jsm/controls/OrbitControls.js>
* **SHA-256:** `5a44a9e86a2a0fb11933eed69bc2cd33c76a496854c1aed6ed776efa87d7b064`
* Imports `three` as a **bare specifier**; the page resolves it via the
  `<script type="importmap">` in `index.html` that maps `"three"` to
  `./vendor/three.module.min.js`. No bundler required.

## Verifying integrity

```bash
cd demo/vendor
shasum -a 256 -c <<'EOF'
3e690ac7d180b0aadf0891bea39eec643e29e2d3e75c99b18689518665f69ba6  three.module.min.js
5a44a9e86a2a0fb11933eed69bc2cd33c76a496854c1aed6ed776efa87d7b064  OrbitControls.js
EOF
```

## Re-vendoring (maintainers only)

```bash
V=0.160.0
curl -fsSL "https://unpkg.com/three@${V}/build/three.module.min.js"                 -o demo/vendor/three.module.min.js
curl -fsSL "https://unpkg.com/three@${V}/examples/jsm/controls/OrbitControls.js"     -o demo/vendor/OrbitControls.js
shasum -a 256 demo/vendor/three.module.min.js demo/vendor/OrbitControls.js   # update the hashes above
```
