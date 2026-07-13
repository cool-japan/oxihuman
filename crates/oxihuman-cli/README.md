# oxihuman-cli

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 193 passing | **Commands:** 30 subcommands | **Version:** 0.2.1 | **Updated:** 2026-07-13

Command-line interface for OxiHuman body generation and export. Binary: `oxihuman`

---

## Installation

```bash
cargo install oxihuman-cli
```

---

## Quick Start

```bash
# Generate a morphed GLB from base mesh, targets, and a body preset
oxihuman generate --base human_base.obj --targets ./targets/ --preset athletic --output human.glb

# Build a verified target manifest from a directory
oxihuman pack-build --targets ./targets/ --output manifest.toml

# Curate the upstream MakeHuman CC0 assets into the shipped OHPK v1 core pack
oxihuman pack-core --upstream assets/upstream/makehuman --out assets/packs/oxihuman-core-v1.ohpk
```

---

## Subcommands — 30 Total

### Mesh Generation

| Command | Description |
|---------|-------------|
| `generate` | Build a morphed GLB from base mesh, morph targets, and body parameters |
| `remesh` | Remesh an OBJ mesh using isotropic remeshing |

### Asset Management

| Command | Description |
|---------|-------------|
| `pack-build` | Scan a targets directory and build a verified manifest |
| `pack-wizard` | Interactive wizard for building an `.oxp` asset pack from scratch |
| `pack-core` | Curate upstream MakeHuman CC0 assets into an OHPK v1 core/full pack |
| `validate` | Validate a `.target` file, or a whole pack manifest with `--pack` |
| `pack-dist-manifest` | Generate a distribution manifest for a signed pack |
| `pack-verify-dist` | Verify a pack against a distribution manifest |

### Information & Reporting

| Command | Description |
|---------|-------------|
| `info` | Print information about a mesh, GLB, or manifest file |
| `session` | Print saved session JSON information |
| `stats` | Parse an OBJ and print detailed mesh statistics |
| `workspace` | Print workspace and build info (version, crate list, enabled features) |
| `plugin-list` | List built-in plugin descriptors |
| `camera-info` | Print the default camera rig as JSON |

### Export Formats

| Command | Description |
|---------|-------------|
| `quantize` | Quantize an OBJ mesh to compact QMSH binary format |
| `morph-export` | Export morph targets to OXMD binary format |
| `zip-pack` | Pack base mesh and targets into a self-contained ZIP |
| `stl` | Export mesh to STL (ASCII or binary mode) |
| `collada` | Export mesh to COLLADA (.dae) |
| `gltf-sep` | Export mesh to glTF JSON with a separate `.bin` buffer file |
| `svg` | Export mesh wireframe or UV layout to SVG |
| `lod-export` | Export a LOD pack with multiple decimation levels |
| `variant-pack` | Export multiple character variants as a named pack |
| `asset-bundle` | Pack base mesh and targets into an OXB asset bundle |
| `pc2` | Bake a base mesh (optionally animated) to a PC2 point cache |
| `mdd` | Bake a base mesh (optionally animated) to an MDD point cache |
| `stream-export` | Stream-export mesh vertex positions to chunked files (f32, f16, or CSV) |
| `report` | Generate an HTML pipeline report for a build |

### Advanced

| Command | Description |
|---------|-------------|
| `physics-export` | Export a physics scene (gltf-physics or openxr format) |
| `anim-bake` | Bake an animated params source to PC2/MDD |

---

## Command Reference

### `generate`

Build a morphed, export-ready GLB from raw inputs. Body parameters are normalized `[0.0, 1.0]` sliders passed via `--params` (inline JSON or a path to a JSON file), or picked from a named `--preset`.

```bash
oxihuman generate \
  --base human_base.obj \
  --targets ./targets/ \
  --params '{"height":0.8,"weight":0.4,"muscle":0.6,"age":0.5}' \
  --output human.glb
```

### `pack-build`

Scan a directory of `.target` files, verify checksums, and write a TOML manifest.

```bash
oxihuman pack-build --targets ./targets/ --output manifest.toml
```

### `pack-wizard`

Interactive wizard for building an `.oxp` asset pack. Prompts (in order, each with a sensible default) for pack name, author, version, and license, a required targets directory, an optional texture directory, an optional preset CSV file, and an output path — then builds the pack and writes a `.manifest.json` sidecar next to it.

```bash
oxihuman pack-wizard
```

Equivalent non-interactive usage (all prompts answered via stdin; blank lines accept the optional texture-directory / preset-CSV defaults):

```bash
echo -e "my_pack\nCOOLJAPAN OU\n0.1.0\nApache-2.0\n./targets\n\n\n./output.oxp" | oxihuman pack-wizard
```

### `pack-core`

Curate the upstream MakeHuman CC0 data tree into the shipped OHPK v1 asset pack. The `core` tier (default) is a small, curated, adult-only target set kept inside a hard byte budget (default 2 MiB); `full` packs every policy-allowed target with no budget. Every target's sparse deltas are re-indexed from raw MakeHuman `v`-line order into the pack's OBJ-loader vertex order before encoding, so morph deltas land on the correct (UV-seam-split) vertices.

```bash
oxihuman pack-core \
  --upstream assets/upstream/makehuman \
  --tier core \
  --out assets/packs/oxihuman-core-v1.ohpk \
  --report docs/bench/pack-reconstruction-error.md
```

Requires the upstream MakeHuman data tree fetched by `scripts/fetch_upstream_assets.sh` (base mesh + `.target` files). Writes the packed `.ohpk` file, a `.provenance.json` sidecar, and (core tier only) a reconstruction-error report.

### `validate`

Validate a single `.target` file, or an entire pack manifest with `--pack`.

```bash
oxihuman validate face_slim.target
oxihuman validate --pack manifest.toml
```

### `quantize`

Compact a mesh to QMSH for efficient storage and fast loading.

```bash
oxihuman quantize --base human_base.obj --output human.qmsh
```

### `lod-export`

Export multiple LOD levels in a single pass.

```bash
oxihuman lod-export \
  --base human_base.obj \
  --output-dir ./lods/ \
  --levels 4
```

### `stream-export`

Stream vertex positions to chunked binary or text output.

```bash
# Float32 binary chunks
oxihuman stream-export --input human_base.obj --format f32 --chunk-size 1024 --output positions.f32

# CSV (human-readable)
oxihuman stream-export --input human_base.obj --format csv --output positions.csv
```

### `anim-bake`

Bake a parameter-driven animation to a point cache format.

```bash
oxihuman anim-bake \
  --input human_base.obj \
  --targets ./targets/ \
  --params-json anim.json \
  --format pc2 \
  --output anim.pc2
```

### `report`

Generate a self-contained HTML pipeline report.

```bash
oxihuman report \
  --base human_base.obj \
  --targets ./targets/ \
  --output pipeline_report.html \
  --title "OxiHuman Report"
```

---

## Dependencies

`oxihuman-cli` is a binary-only crate (no public library API). Runtime dependencies:

- `anyhow` — error handling
- `serde_json` — JSON parsing and serialization
- All `oxihuman-*` workspace crates — core pipeline, mesh, morph, physics, export

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
