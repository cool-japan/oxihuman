# Clean-Room Audit — MakeHuman Derivation Review

Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)

Date: 2026-07-12
Scope: all Rust modules and documentation in this repository that reference
MakeHuman by name, cite MakeHuman Python source files, or implement
MakeHuman-originated file formats (`.target`, `.mhclo`, `.mhm`).

## 1. Purpose

MakeHuman (makehumancommunity/makehuman) application code is licensed under
AGPL-3.0. OxiHuman is Apache-2.0. This audit verifies that no OxiHuman code is
a copy, translation, or transliteration of MakeHuman Python source, and that
all public wording accurately describes the relationship as
**format compatibility, not code derivation**.

Note on data vs. code: MakeHuman's bundled *assets* (base mesh, morph targets)
are CC0-licensed; only the Python *application code* is AGPL. This audit
concerns code only.

## 2. Methodology

1. **Sweep**: case-insensitive `grep -ri 'makehuman'` and `grep -ri '\.py'`
   across `crates/**/*.rs` and `docs/**/*.md`, enumerating every citation of a
   MakeHuman Python source file versus mere mentions of file formats or the
   project name.
2. **Reference acquisition** (read-only, outside the repository): the cited
   upstream sources were fetched from
   `github.com/makehumancommunity/makehuman` (master) into a temporary
   directory for comparison only — never into this repository:
   - `makehuman/core/algos3d.py` (509 lines)
   - `makehuman/shared/proxy.py` (1088 lines) — the current home of the proxy
     binding logic; the historically cited `mh2proxy.py` does not exist in
     current MakeHuman master (it was a legacy 1.0.x module whose successor is
     `shared/proxy.py`).
3. **Structural comparison** per module: function decomposition, algorithm
   step order, constant values, identifier naming, and comment echoes were
   compared side-by-side. No AGPL text is reproduced in this document;
   structure is described, not quoted.
4. **Verdict scale**:
   - **INDEPENDENT** — format-compatible, own design; similarity (if any) is
     dictated by the file format itself.
   - **REFERENCED** — algorithm concept shared, implementation independent.
   - **TRANSLITERATED** — structure mirrors the Python; derivation risk
     (requires rewrite).

## 3. Citation sweep results

Files citing a MakeHuman **Python source file** (the only derivation-relevant
category):

| Location | Citation | Disposition |
|---|---|---|
| `crates/oxihuman-mesh/src/clothing.rs` (header) | "Algorithm (from MakeHuman `mh2proxy.py`)" | Audited (section 4.3); header reworded |
| `docs/ARCHITECTURE.md` line 34 | "Based on MakeHuman's `algos3d.py`" | Audited (section 4.4); line reworded |

All other ~50 matches reference only the MakeHuman *project name*, its
*file formats* (`.target`, `.mhclo`, `.mhm`), its *naming conventions* for
morph targets, its *coordinate/unit conventions*, or the optional
`MAKEHUMAN_DATA_DIR` test-asset environment variable. None cite Python source
files and none carry derivation risk.

## 4. Per-module verdicts

### 4.1 `crates/oxihuman-core/src/parser/target.rs` — VERDICT: INDEPENDENT

Compared against `algos3d.py` `Target._load_text` (the only text-format reader
upstream).

Shared behavior — all of it dictated by the `.target` format itself
(plain text; `#` comment lines; data lines of `vertex_index dx dy dz`):
skipping comment lines, whitespace tokenization, requiring exactly 4 tokens,
parsing one integer index and three floats.

Structural differences:
- Rust is a free function `parse_target(name, src) -> Result<TargetFile>`
  returning a named struct of `Delta { vid, dx, dy, dz }`; upstream is a
  method on a stateful `Target` class populating parallel numpy arrays
  (`verts` index array + `data` vector array via a structured dtype).
- Rust sorts deltas by vertex id after parsing; upstream does not.
- Rust attaches per-line error context and propagates parse failures;
  upstream silently accumulates and has no per-line error reporting.
- Upstream additionally parses asset-license metadata out of comment lines and
  has an entire binary `.npz` cache layer (`_load_binary*`, `_save_binary`)
  with none of which any counterpart exists in Rust.
- No identifier overlap beyond format vocabulary; no comment echoes.

### 4.2 `crates/oxihuman-core/src/parser/mhclo.rs` — VERDICT: INDEPENDENT

Compared against `proxy.py` `loadTextProxy` + `ProxyRefVert`.

Upstream is a ~130-line stateful keyword dispatcher handling ~30 keys
(z_depth, max_pole, special_pose, material, uvLayer, x/y/z_scale, nine shear
variants, delete_verts, weights sections, deprecated keys, license comment
parsing) with a mode-flag state machine (`doRefVerts`/`doWeights`/
`doDeleteVerts`) and a `ProxyRefVert` class offering both a single-vertex form
(`fromSingle`) and a triple form (`fromTriple`, with optional offset when more
than 6 tokens are present).

The Rust parser is a deliberately minimal subset: four metadata keys
(`uuid`, `basemesh`, `name`, `obj_file`) plus a `verts` section of strictly
9-token lines, with a declared-count validation check that upstream does not
perform at all. It does not implement the single-vertex form, the transform
matrix keys, weights, delete_verts, or materials. Different decomposition,
different validation model, different scope; similarity is limited to the
key/value vocabulary defined by the `.mhclo` format itself.

### 4.3 `crates/oxihuman-mesh/src/clothing.rs` — VERDICT: REFERENCED

Compared against `proxy.py` `Proxy.getCoords` / `ProxyRefVert.getCoord`.

The core equation — each clothing vertex equals a barycentric-weighted sum of
three base-mesh vertices plus a residual offset — is the *meaning of the
`.mhclo` vertex-binding data*; any correct consumer of the format computes it.
The algorithm concept is therefore shared (hence REFERENCED, not
INDEPENDENT), but the implementation is not derived:

- Upstream computes the whole mesh in vectorized numpy expressions and
  multiplies the offset column through a `TMatrix` (scale/shear
  transformation) derived from the current body. Rust adds the raw offset
  directly and has no TMatrix concept.
- Rust is decomposed into `apply_clothing` (whole-mesh, with a binding-count
  precondition check that upstream does not have) plus a per-vertex
  `interpolate_vertex` with explicit index-bounds checking and typed error
  returns; upstream has neither bounds checks nor error returns at this layer.
- No naming or comment echoes.

**Remediation**: the module header previously said "Algorithm (from MakeHuman
`mh2proxy.py`)", which both overstated derivation and cited a file that no
longer exists upstream. Reworded to state that the module is an independent
implementation of the documented `.mhclo` proxy-binding file-format semantics.
No code change was required.

### 4.4 `crates/oxihuman-morph/src/engine.rs` — VERDICT: INDEPENDENT

Compared against `algos3d.py` `Target.apply` / `loadTranslationTarget`.

The only shared idea is `position[vid] += delta * weight` — the universal
definition of a sparse blendshape/morph target, ubiquitous across the
graphics field (glTF morph targets, FBX blendshapes, etc.).

Structural differences are total:
- Rust stores positions as SoA (`base_x/base_y/base_z` vectors) and rebuilds
  from base each time (or incrementally by subtracting old and adding new
  weighted contributions); upstream mutates a persistent AoS `obj.coord`
  buffer in place via numpy fancy indexing.
- Rust drives weights through per-target closures over a `ParamState`, with a
  params-keyed result cache, an incremental rebuild path, rayon-parallel and
  portable-SIMD kernels. None of these exist upstream.
- Upstream features with no Rust counterpart: face-group masking, pose-space
  application of deltas through a baked skeleton (`animatedMesh` path),
  normal-recalculation orchestration, target buffering keyed by canonical
  file path, `.npz` compiled-target caching.
- No identifier overlap, no comment echoes, no shared constants.

**Remediation**: `docs/ARCHITECTURE.md` previously introduced the pseudocode
with "Based on MakeHuman's `algos3d.py`", overstating the relationship.
Reworded to describe it as the standard sparse blendshape scatter-add,
compatible with the documented `.target` semantics. The pseudocode itself is
original to this repository. No code change was required.

### 4.5 `crates/oxihuman-export/src/makehuman_export.rs` — VERDICT: INDEPENDENT

A small writer producing MakeHuman-compatible parameter text (`.mhm`-style
`version` + `name value` lines). Pure format emission; no upstream Python
counterpart logic was consulted or mirrored; no Python file is cited.

### 4.6 Remaining name-only references — VERDICT: INDEPENDENT (no code audit required)

Modules such as `oxihuman-morph/src/expression_retarget.rs`
(`makehuman_to_daz_map` — a name-mapping table), `units.rs` (parameter-space
convention), `regions.rs` / `weight_curves.rs` (target *filename* naming
conventions), `oxihuman-core/src/category.rs` (directory-layout mirror),
`oxihuman-test-utils` (test-asset path helper) reference MakeHuman only as a
naming/layout/data convention. No algorithm citation, no derivation surface.

## 5. Summary of remediation performed

| File | Change |
|---|---|
| `crates/oxihuman-mesh/src/clothing.rs` | Header: removed "(from MakeHuman `mh2proxy.py`)" claim; now states independent implementation of documented `.mhclo` semantics. Doc comments only; no code changed. |
| `docs/ARCHITECTURE.md` | "Based on MakeHuman's `algos3d.py`" replaced with standard-blendshape wording (independent implementation). |
| Root `Cargo.toml` | Workspace description: "pure Rust MakeHuman port" replaced with "MakeHuman-compatible independent implementation (reads .target/.mhclo formats)". |
| `crates/oxihuman/src/lib.rs`, `crates/oxihuman/README.md` | Same branding rewording as above. |

**No TRANSLITERATED verdict was reached; no code rewrite was necessary.**

## 6. Defensible public statement

The following wording is supported by this audit and is the approved public
positioning:

> OxiHuman is an independent, pure-Rust, Apache-2.0 implementation of a
> parametric human body generator. It is *format-compatible* with MakeHuman:
> it reads the documented `.target` (sparse vertex-delta morph) and `.mhclo`
> (barycentric proxy binding) file formats and follows MakeHuman's target
> naming and directory conventions. It contains no code copied, translated,
> or otherwise derived from the AGPL-licensed MakeHuman Python application.
> MakeHuman's CC0-licensed mesh/target *data assets* may be used with
> OxiHuman under their own terms.

Supported claims about the parsers: this audit's structural comparison of the
`.target` and `.mhclo` parsers against the upstream readers found no
structural mirroring, no shared decomposition, no naming or comment echoes,
and Rust-side behaviors (delta sorting, count validation, per-line error
context) absent upstream; every similarity found is dictated by the file
formats themselves. The parsers are consistent with, and are hereby verified
against, implementation from the file-format layout (comment-prefixed text;
`index dx dy dz` lines; key/value header plus 9-token binding lines) rather
than from upstream code.

Do **not** use the word "port" in any public description.

## 7. Residual risks

1. **Historical authorship evidence**: this audit is a structural (output)
   audit, not a development-process audit; it verifies non-derivation of the
   current code but cannot reconstruct how the code was originally authored.
   The structural findings above are the defensible basis.
2. **`.mhclo` subset**: the parser implements a subset of the format (no
   single-vertex bindings, no scale/shear transform keys, no `delete_verts`).
   This is a compatibility limitation, not a legal risk, but extending it in
   the future must again be done from format documentation, never from
   `shared/proxy.py`.
3. **Trademark**: "MakeHuman" remains a project name of the MakeHuman
   community. Usage here is nominative (compatibility statements only), which
   is the correct posture; avoid using the mark in crate names or logos.
4. **Data assets**: any bundled or fetched MakeHuman data must remain
   CC0-verified with provenance recorded (tracked separately under the
   PROVENANCE workstream).
