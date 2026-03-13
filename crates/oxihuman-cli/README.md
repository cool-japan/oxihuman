# oxihuman-cli

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 134 passing | **Commands:** 34 subcommands | **Version:** 0.1.1 | **Updated:** 2026-03-13

Command-line interface for OxiHuman body generation and export. Binary: `oxihuman`

---

## Installation

```bash
cargo install oxihuman-cli
```

---

## Quick Start

```bash
# Generate a morphed GLB from base mesh, targets, and parameters
oxihuman generate --base human_base.obj --targets ./targets/ --height 1.75 --output human.glb

# Build a verified target manifest from a directory
oxihuman pack-build ./targets/ --output manifest.json

# Generate a batch of character variants from a parameter grid
oxihuman batch-chars --params params.json --output-dir ./variants/
```

---

## Subcommands — 34 Total

### Mesh Generation

| Command | Description |
|---------|-------------|
| `generate` | Build a morphed GLB from base mesh, morph targets, and body parameters |
| `batch-chars` | Generate multiple character variants from a parameter grid JSON |
| `remesh` | Remesh an OBJ mesh using isotropic remeshing |

### Asset Management

| Command | Description |
|---------|-------------|
| `pack-build` | Scan a targets directory and build a verified manifest |
| `pack-wizard` | Interactive 7-step wizard for building an asset pack from scratch |
| `validate` | Validate a `.target` file or pack manifest |
| `validate-pack` | Validate a full pack manifest including all referenced targets |
| `sign-pack` | Sign a pack directory with a signature file |
| `verify-sign` | Verify a pack directory's signature file |
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
| `stream-export` | Stream-export mesh vertex positions to chunked files (f32, i16, or CSV) |
| `report` | Generate an HTML pipeline report for a build |
| `mdd` | Export motion data in MDD format |

### Advanced

| Command | Description |
|---------|-------------|
| `proxies` | Generate body collision proxies and print as JSON |
| `physics-export` | Export a physics scene (gltf-physics or openxr format) |
| `target-info` | Print metadata about a `.target` file |
| `anim-bake` | Bake animation sequences to PC2 or MDD format |

---

## Command Reference

### `generate`

Build a morphed, export-ready GLB from raw inputs.

```bash
oxihuman generate \
  --base human_base.obj \
  --targets ./targets/ \
  --height 1.75 \
  --weight 0.4 \
  --muscle 0.6 \
  --age 30 \
  --output human.glb
```

### `batch-chars`

Generate many character variants in one pass from a JSON parameter grid. Each entry in the grid becomes a separate output file.

```bash
oxihuman batch-chars \
  --base human_base.obj \
  --targets ./targets/ \
  --params params.json \
  --output-dir ./variants/
```

Example `params.json`:
```json
[
  { "name": "variant_a", "height": 1.80, "weight": 0.3, "muscle": 0.7 },
  { "name": "variant_b", "height": 1.60, "weight": 0.6, "muscle": 0.2 }
]
```

### `pack-build`

Scan a directory of `.target` files, verify checksums, and write a manifest JSON.

```bash
oxihuman pack-build ./targets/ --output manifest.json
```

### `pack-wizard`

Interactive 7-step wizard for building an asset pack. Prompts for pack name, output directory, base mesh path, targets directory, format options, signing key, and a final confirmation before writing output.

```bash
oxihuman pack-wizard
```

Equivalent non-interactive usage (all prompts answered via stdin):

```bash
echo -e "my_pack\n./out\nhuman_base.obj\n./targets\n\n\nyes" | oxihuman pack-wizard
```

### `validate` / `validate-pack`

```bash
oxihuman validate face_slim.target
oxihuman validate-pack manifest.json
```

### `sign-pack` / `verify-sign`

```bash
oxihuman sign-pack ./targets/ --key signing.key --output targets.sig
oxihuman verify-sign ./targets/ --sig targets.sig --key signing.pub
```

### `quantize`

Compact a mesh to QMSH for efficient storage and fast loading.

```bash
oxihuman quantize human_base.obj --output human.qmsh
```

### `lod-export`

Export multiple LOD levels in a single pass.

```bash
oxihuman lod-export human.glb \
  --levels 0.9,0.5,0.25,0.1 \
  --output-dir ./lods/
```

### `stream-export`

Stream vertex positions to chunked binary or text output.

```bash
# Float32 binary chunks
oxihuman stream-export human.glb --format f32 --chunk-size 1024 --output-dir ./stream/

# CSV (human-readable)
oxihuman stream-export human.glb --format csv --output positions.csv
```

### `anim-bake`

Bake a parameter-driven animation to a point cache format.

```bash
oxihuman anim-bake \
  --base human_base.obj \
  --targets ./targets/ \
  --anim anim.json \
  --format pc2 \
  --output anim.pc2
```

### `report`

Generate a self-contained HTML pipeline report.

```bash
oxihuman report \
  --base human_base.obj \
  --targets ./targets/ \
  --params params.json \
  --output pipeline_report.html
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
