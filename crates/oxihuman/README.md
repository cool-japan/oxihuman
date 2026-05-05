# oxihuman

> Privacy-first, client-side human body generator — pure Rust MakeHuman port.

**Version:** 0.1.2 | **Updated:** 2026-05-05

This is the **facade crate** that re-exports all OxiHuman sub-crates under a single, ergonomic namespace.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
oxihuman = "0.1.2"
```

Then in your code:

```rust
use oxihuman::prelude::*;
```

## Features

| Feature  | Default | Description                |
|----------|---------|----------------------------|
| `viewer` | **yes** | Real-time rendering viewer |
| `wasm`   | no      | WebAssembly bindings       |
| `full`   | no      | All optional features      |

## Sub-crates

| Module    | Description                                      |
|-----------|--------------------------------------------------|
| `core`    | Foundation: parsers, policies, asset management   |
| `morph`   | Morphology engine: parameters, blendshapes        |
| `mesh`    | Geometry processing: decimation, subdivision, UV  |
| `export`  | Export pipeline: glTF, OBJ, STL, USD, 50+ formats|
| `physics` | Physics: collision proxies, cloth, soft body      |
| `viewer`  | Real-time rendering (feature-gated)               |
| `wasm`    | WebAssembly bindings (feature-gated)              |

## License

Apache-2.0 — Copyright (c) COOLJAPAN OU (Team Kitasan)
