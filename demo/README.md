# OxiHuman Demo

A single-page interactive demo of the OxiHuman privacy-first human body generator.

## What it does

- Body parameter sliders: height, weight, age, muscle, body fat
- Alpha preset buttons: Athletic, Slim, Heavy, Tall, Short
- Live 3-D wireframe preview with drag-to-rotate (2-D canvas projection)
- Export buttons: Download GLB, OBJ, STL
- FPS / vertex / memory stats
- Service worker for offline cache-first asset serving

The page works fully in a browser with no WASM module present (a JS fallback
procedural mesh is used).  When `wasm-pack` has been run the WASM engine is
loaded automatically.

## Running locally

```sh
# From the workspace root
cd demo
python3 -m http.server 8080
# then open http://localhost:8080
```

Any static file server works.  The service worker registration requires
a `localhost` or HTTPS origin.

## Building the WASM module

```sh
# From the workspace root — requires wasm-pack and the Rust toolchain
wasm-pack build crates/oxihuman-wasm \
  --target web \
  --out-dir ../../demo/pkg \
  --release
```

After building, the demo automatically loads
`demo/pkg/oxihuman_wasm.js` and `demo/pkg/oxihuman_wasm_bg.wasm`.

## Deploying to GitHub Pages

1. Build the WASM module as above (or skip for the JS-fallback-only demo).
2. Push the `demo/` directory to the `gh-pages` branch:

```sh
# One-shot publish of the demo/ directory
git subtree push --prefix demo origin gh-pages
```

Or configure GitHub Actions to run `wasm-pack build` and then deploy the
`demo/` directory with the `peaceiris/actions-gh-pages` action.

A minimal workflow (`.github/workflows/deploy-demo.yml`):

```yaml
name: Deploy OxiHuman Demo
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo install wasm-pack
      - run: |
          wasm-pack build crates/oxihuman-wasm \
            --target web \
            --out-dir ../../demo/pkg \
            --release
      - uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./demo
```

## License

Apache-2.0 — Copyright 2026 COOLJAPAN OU (Team Kitasan)
