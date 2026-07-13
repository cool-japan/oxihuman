# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - Unreleased

## [0.2.1] - 2026-07-13

The **BodyLab production release**: a fully client-side, browser-ready human
body generator. Ships the first real CC0 asset pack (OHPK v1), in-memory
browser exports, a live three.js demo, a measurement-fit solver, and the legal
/ safety cleanroom that makes the whole thing shippable.

### Added
- **M0 — Legal & safety cleanroom.** Added the clean-room verification record
  [`docs/CLEANROOM_AUDIT.md`](docs/CLEANROOM_AUDIT.md), the CC0 provenance chain
  [`PROVENANCE.md`](PROVENANCE.md) with a machine-checkable
  [`scripts/check_provenance.sh`](scripts/check_provenance.sh) (verifies the
  pack SHA-256, the sidecar `.provenance.json`, all 38 bundled targets + the
  base mesh against the upstream CC0 manifest, and the `alpha_pack` manifest
  digest), [`CONTRIBUTING.md`](CONTRIBUTING.md) (forbidden-sources policy) and
  [`NOTICE`](NOTICE). Safety by construction: a named regression test
  `invariant_no_nude_mesh_stage`, an export gate enforced on every export entry
  point (GLB / VRM / OBJ / STL / COLLADA / USD / 3MF / X3D), a client-side age
  floor clamp (18 y for the shipped core pack), and [`SAFETY.md`](SAFETY.md).
- **M1 — Asset pack + OHPK v1 format.** New `core_pack` container in
  `oxihuman-export` and the `oxihuman-cli pack-core` command
  (`crates/oxihuman-cli/src/commands/pack_core.rs`, with `--report`) that builds
  the shipped [`assets/packs/oxihuman-core-v1.ohpk`](assets/packs) — 2,093,260 B,
  38 CC0 morph targets (30 macrodetails corners + 8 `measure/` girth targets),
  21,833 base vertices, sparse `i16` max-abs quantised deltas (worst-case
  reconstruction error 0.011 mm, see
  [`docs/bench/pack-reconstruction-error.md`](docs/bench/pack-reconstruction-error.md)).
  Added `scripts/fetch_upstream_assets.sh`, a precise cross-section body
  measurer (`oxihuman-morph` `measurements/cross_section.rs`, `body_measurement`,
  `units.rs`) that reads convex-hull tape circumferences of the isolated torso.
- **M2 — Browser exports + measurement fit.** `WasmEngine` gains in-memory,
  filesystem-free exporters `export_glb` / `export_vrm` / `export_stl` /
  `export_obj` (feature-gated behind `bindgen`), the
  `from_core_pack_bytes(bytes)` constructor that loads an OHPK pack directly in
  the browser, and `fit_to_measurements` (`engine_fit.rs`): height solved
  directly on the monotone stature response (bisection), Nelder–Mead over
  `weight / muscle / gender`, then a lever-gated coordinate-descent refinement
  over the `measure/` bust / underbust / waist / hips weights. The `brief-172`
  probe fits to |Δ| ≤ 0.66 cm in ~0.9 s under Node (see
  [`docs/bench/measurement-error.md`](docs/bench/measurement-error.md)). Also
  adds hand-written TypeScript `interface` definitions (`ts_types.rs`)
  injected into the generated `.d.ts` for richer IDE type hints on the WASM
  API.
- **M3 — BodyLab demo.** A fully static [`demo/`](demo/) app (parameter sliders,
  live preview, browser GLB/OBJ/STL export) rendering with **three.js r160**
  vendored under `demo/vendor/` (`three.module.min.js`, `OrbitControls.js`; see
  `demo/vendor/VENDOR.md`), split into `demo/src/{viewer,controls,badges}.js`.
  Added `scripts/build_demo.sh` (one-shot WASM + pack bundling) and
  `scripts/demo_serve_check.sh` (serves the static site and asserts every
  referenced asset returns 200). Verified end-to-end in headless Chrome.
- **M4 — Zero-copy geometry, benchmarks, CI.** Zero-copy geometry hand-off for
  three.js/WebGL: `wasm_memory()`, `refresh_geometry()`, `positions_ptr()` /
  `positions_len()` expose the engine's vertex buffer as a `Float32Array` view
  with no JS-side copy. Reproducible bench harness: `web/bench/fps_bench.mjs`
  (engine morph cost, p50 ≈ 0.52 ms/frame at 21,833 verts), `web/bench/sizes.sh`,
  `scripts/wasm_node_check.mjs` (60 checks), `scripts/validate_exports.mjs`
  (25 checks), `scripts/measure_roundtrip.sh`, and the `docs/bench/` reports.
  Hardened `.github/workflows/npm-publish.yml` with a gzip size gate that reads
  the single-source-of-truth budget from `crates/oxihuman-wasm/wasm-opt.toml`.

### Changed
- Workspace version bumped from `0.2.0` to `0.2.1`.
- Branding reworded across the workspace from "pure Rust MakeHuman port" to
  "MakeHuman-compatible independent implementation (reads `.target` / `.mhclo`
  formats)" — root `Cargo.toml` and the `oxihuman` facade crate description.
- Routine dependency maintenance: `oxiarc-deflate` `0.3.3` → `0.3.5`, `anyhow`
  `1.0.102` → `1.0.103`, `wasm-bindgen` `0.2.125` → `0.2.126`, `wasm-bindgen-test`
  `0.3.75` → `0.3.76`.
- Removed the unused `js-sys` dependency from `oxihuman-wasm` (zero references
  anywhere in the workspace; caught by `cargo-udeps` on the `bindgen` feature).

### Fixed
- **`export_glb` (and every wasm exporter) panicked on wasm32 (P0).** The
  exporters reached `std::fs` / `std::env::temp_dir()`, which *panics* on
  wasm32 (no filesystem); the trap poisoned the engine object's wasm-bindgen
  `WasmRefCell` borrow flag, so every subsequent call failed with "recursive use
  of an object detected". All exporters now use the in-memory byte builders from
  `oxihuman-export`, so a single engine instance can export repeatedly in the
  browser.
- **Macro corner targets summed into a giant — partition-of-unity composition
  (P0).** The ~30 macrodetails corner targets were being *summed*, so at the
  neutral slider centre the base mesh ballooned. `oxihuman-wasm` now composes
  them as a partition of unity (`gender · age · muscle · weight` product blend,
  with a monotone stature handling for the height corners and a 1/3 ethnicity
  share), so neutral sliders yield the base body and each slider drives a
  realistically-scaled result.
- **Pack targets applied to permuted vertices — index remap (P0).** Earlier
  packs stored `.target` deltas in raw MakeHuman v-line order while the pack's
  base mesh stores vertices in the OBJ loader's face-first-occurrence order
  (21,833 packed verts after UV-seam splits vs 19,158 v-lines), so every target
  deformed the wrong vertices and the localised `measure/` girth targets moved
  noise. `oxihuman-cli pack-core` now re-indexes every target through the
  loader's raw→packed mapping (duplicating each delta across seam copies,
  verified by the `pack_core` invariant tests); the lever-gated girth refinement
  now engages end-to-end and every girth residual is sub-1.5 cm.
- **CLI de-stubs.** Real animation export (`pc2` / `mdd` / `anim-bake` via
  `commands/anim_params.rs`) and pack / pipeline command fixes replace prior
  placeholders.

### Removed
- **SMPL / SMPL-X export modules deleted** (`smpl_export.rs`, `smplx_export.rs`)
  per the forbidden-sources policy — OxiHuman ships no SMPL-family shape basis.
- Six dead / phantom export modules that were never reachable from the public
  API or were superseded duplicates: `glb_export.rs`, `obj_export.rs`,
  `obj_export_v2.rs`, `stl_export.rs`, `stl_export_v2.rs`, `three_mf_export.rs`.

### Testing
- **33,569 tests** (up from 33,410 in 0.2.0) — 0 failures across the full
  `--workspace --all-features` suite, including the new OHPK round-trip, pack
  index-remap invariants, measurement/fit round-trip, export well-formedness,
  and the `invariant_no_nude_mesh_stage` safety regression.
- 0 clippy warnings (workspace, all targets, all features) · 0 `unwrap` /
  `expect` in production code · `cargo fmt` clean.

---

## [0.2.0] - 2026-06-19

### Added
- **Real CRC-32 combine** (`crc_table.rs`): `CrcTable::combine` now implements the zlib
  GF(2) matrix algorithm — it builds the zeros-operator matrices (1/2/4-bit) and advances
  `crc1` over `len2` zero bytes via repeated GF(2) matrix squaring, then folds in `crc2`.
  `combine(crc(A), crc(B), B.len())` now equals `crc(A‖B)` exactly. Previously returned 0.
  Passes: concatenation-equivalence, empty-second identity, three-way associativity.
- **Real expression weights parser** (`expression_io.rs`): `expression_from_json` now parses
  the full `"weights":[…]` array via `extract_json_f32_array` instead of discarding it as a
  single `0.0`. Also wired the module into the crate (it was an orphan file, never compiled).
  Passes: weights round-trip (4 values within 1e-4), empty-array.
- **Real procedural cubemap environment** (`background_renderer.rs`): the `CubemapStub` variant
  is renamed `Cubemap` and now samples a genuine direction-dependent sky — equirectangular
  `uv → direction`, OpenGL cube-face selection (`direction_to_face_uv`), smoothstep elevation
  gradient (nadir→zenith), horizon haze, and a directional sun highlight. Previously returned a
  flat colour. Passes: zenith≈top/nadir≈bottom, direction-dependence, axis face selection.
- **Real GLB skinning export** (`glb.rs`): `export_glb_with_skeleton` now emits `JOINTS_0`
  (u16×4), `WEIGHTS_0` (f32×4) vertex attributes and per-joint `inverseBindMatrices`. Joint
  world bind transforms come from memoised forward kinematics (`compose_trs`/`mat4_mul`,
  any parent ordering); inverse-bind matrices via a general 4×4 `mat4_inverse`; per-vertex
  weights via top-4 nearest joints, inverse-distance weighted and normalised. Previously the
  mesh used the skeleton for the node hierarchy only. Passes: attributes present, 7 accessors,
  per-vertex weights sum to 1 (read back from the BIN chunk).
- **Real fracture cell volume** (`fracture.rs`): `cell_volume_approx` now computes the
  star-shaped solid volume from the seed apex via the divergence theorem
  `V = (1/6)|Σ (a−s)·((b−s)×(c−s))|` over a fan triangulation, replacing the `area × 0.1`
  thickness heuristic. Exact for closed, consistently wound cells. Passes: unit corner
  tetrahedron = 1/6 within 1e-4, plus all merge/non-negativity tests.
- **Real articulated-body forward dynamics** (`rigid_body_tree.rs`): `forward_dynamics_step`
  replaces the scalar `τ/mass·dt` integrator with multibody dynamics — it assembles the
  joint-space inertia matrix `H(q)` by propagating each link's spatial inertia through the
  tree (Composite-Rigid-Body method via world-frame geometric Jacobians), computes the
  Coriolis/centrifugal bias `C(q,q̇)` with the Recursive Newton–Euler algorithm, solves
  `H q̈ = τ − C` (Gauss–Jordan with partial pivoting), and integrates semi-implicitly.
  `total_kinetic_energy` now returns `½ q̇ᵀH q̇`. Passes: single-link parallel-axis
  (`H = Izz + m·d²`), kinetic-energy match, base-torque sign.
- **Real discrete curl** (`mesh_edge_flow_field.rs`): `edge_flow_field_curl_magnitude` now
  computes the curl via Stokes' theorem (circulation ÷ enclosed area) instead of returning 0.
  The ring argument changed from positions-only `&[[f32;3]]` to `&[(usize,[f32;3])]` so each
  ring vertex maps to its flow vector; area via Newell's method, circulation via the
  trapezoidal rule. Passes: rotational field `F=(−y,x,0)` ⇒ curl 2.0.
- **Real Laplacian AO smoothing** (`mesh_ambient_occlusion_mesh.rs`): `smooth_ao` is now a
  geometric Laplacian over an adaptive proximity graph (was a global-mean blend); added
  `smooth_ao_laplacian` (topological 1-ring umbrella operator over a triangle index buffer)
  and `AoMesh::smooth`. `AoMesh` gains an `indices: Vec<u32>` field. Passes: topological
  diffusion to neighbours, range preservation.

### Changed
- `ArtBody` gains `com_offset: [f32; 3]` and `link_length: f32` (link geometry for dynamics).
- `AoMesh` gains `indices: Vec<u32>` (triangle topology for AO smoothing).
- `BackgroundType::CubemapStub` renamed to `BackgroundType::Cubemap`.
- `edge_flow_field_curl_magnitude` ring parameter is now `&[(usize, [f32; 3])]`.

### Testing
- **33,410 tests** (up from 33,364) — 0 failures. New coverage across core/morph/viewer/
  export/mesh/physics for all eight de-fakes, including analytic checks (parallel-axis
  inertia, divergence-theorem tetrahedron volume, Stokes curl, CRC concatenation).
- 0 clippy warnings (workspace, all targets, all features) · 0 `unwrap` in production code.

### Removed
- Thirteen dead-code scaffolding modules that were never part of the public API (all files carried
  `#![allow(dead_code)]` and were unreachable from any public entry point):
  - `oxihuman-core`: `byte_order.rs` (`ByteOrderType` + byte-conversion helpers),
    `callback_registry.rs` (`CallbackEntry` / `CallbackRegistry`),
    `dep_resolver_simple.rs` (`DepGraph` topological-sort stub),
    `deque_ring.rs` (`DequeRing<T>` fixed-capacity ring buffer),
    `handle_map.rs`, `property_bag.rs`, `quaternion_utils.rs`.
  - `oxihuman-mesh`: `mesh_spin_duplicate.rs`.
  - `oxihuman-physics`: `angular_limit.rs`, `body_dynamics.rs`,
    `gyroscopic_torque.rs`, `ragdoll_config.rs`.
  - `oxihuman-viewer`: `tooltip.rs` (`Tooltip` display widget stub).

## [0.1.9] - 2026-06-10

### Added
- **Real adaptive AV1 arithmetic coding** (`ec.rs`, `coeffs.rs`, `cdf_tables.rs`): The AV1 coefficient
  coder now uses genuine CDF-adaptive symbol coding instead of 50/50 flat bits. New `encode_symbol_adapt`
  / `decode_symbol_adapt` wrappers apply correct descending-CDF adaptation (`adapt_cdf`) with
  two-pass strict monotone repair (`repair_cdf_monotone`) after every symbol, keeping encoder and
  decoder in strict lockstep. New `CoeffCdfContext` carries per-frame adaptive CDFs for skip, nonzero
  (DC/AC), sign (DC/AC), and magnitude-class (12-ary) coding; magnitudes ≤ 2^k−1 emit a k-bit bypass
  suffix; larger values fall back to exp-Golomb. Lossless round-trip is preserved bit-exactly.
- **Real dodecahedron** (`mesh_platonic_solid.rs`): `build_dodecahedron` now places 20 canonical
  golden-ratio vertices — 8 cube corners `(±1,±1,±1)` and 12 cyclic permutations of `(0,±1/φ,±φ)`,
  all normalized to the unit sphere. 12 pentagonal faces fan-triangulated to **36 triangles** with
  consistent outward CCW winding. Passes: 20 vertices, 36 tris, no degenerate triangles, equal
  pentagon-edge lengths (±1e-4), Euler V−E+F=2.
- **Real Tikhonov-regularized QEF solver** (`mesh_dual_contouring.rs`): `qef_dc_solve` solves
  `(AᵀA + λI)x = (Aᵀb + λc)` where λ≈1e-3·trace(AᵀA)/3 and c=cell centre, via `gaussian_solve`;
  falls back to cell centre on singular system or zero plane count. Verified by planar-SDF exact
  placement and three-orthogonal-planes corner test.
- **Real Bond stiffness rotation** (`anisotropic_material.rs`): `rotate_stiffness_z` now applies
  the full `C' = M C Mᵀ` Bond rotation where M is the 6×6 Voigt Bond matrix for rotation θ about Z.
  Passes: 0°=identity, 90°-permutation (C'[0][0]≈C[1][1]), isotropy-invariance, symmetry-preserved,
  in-plane-trace-invariant.
- **Real rational NURBS tessellation** (`mesh_nurbs_surface.rs`): `tessellate_nurbs` evaluates
  `S(u,v) = ΣΣ N_i(u) N_j(v) w_{ij} P_{ij} / ΣΣ N_i(u) N_j(v) w_{ij}` over clamped knot vectors
  using the Cox–de Boor `de_boor_basis` helper. Fixed clamped-endpoint bug: backward search finds
  the last non-degenerate knot span for t=1.0. Passes: partition-of-unity, corner interpolation
  (S(0,0)=P_0), planarity for flat control net, weight-sensitivity (bumped interior weight bulges
  surface).
- **Real face-conserving fracture cell merge** (`fracture.rs`): `merge_small_cells` now appends
  sub-threshold cells' face lists into the nearest surviving neighbor and updates the neighbor's seed
  to the union centroid, then drops merged indices. Previously the face lists were silently discarded
  (`let _ = j`). Passes: face-conservation (total faces in = total faces out), merged-target-grew,
  idempotent-on-large-cells.
- **Real XPBD tetrahedral volume constraint** (`xpbd_volume.rs`): `XpbdVolume::substep` now computes
  analytic volume-constraint gradients `∇₁C = (1/6)(p2−p0)×(p3−p0)` etc., updates
  `Δλ = (−C − α̃λ)/(Σ w_i|∇_iC|² + α̃)`, and applies `Δp_i = w_i ∇_iC Δλ` per XPBD. Pinned
  vertices (inv_mass=0) are unchanged. Passes: volume-restoration (monotone convergence over
  substeps), rest-state-stillness (C=0 ⇒ no motion), momentum-conservation (∇₀+∇₁+∇₂+∇₃=0).

### Changed
- Workspace version bumped from `0.1.8` to `0.1.9`.

### Testing
- **33,364 tests** (up from 33,341 in 0.1.8) — 23 new tests across mesh/physics/export/av1.
- 0 clippy warnings · 0 `unwrap` in production code.

## [0.1.8] - 2026-06-10

### Added
- **Full AV1 intra still-image codec** (pure Rust, no libaom/dav1d/rav1e): Profile 0,
  `reduced_still_picture_header=1`, single KEY_FRAME, single tile, 4:4:4, MC_IDENTITY (GBR
  ordering). Lossless backbone via WHT4×4 exact-integer butterfly (`base_q_idx=0`); optional lossy
  DCT4×4 path behind `AvifPreset`. New subtree `crates/oxihuman-export/src/av1/` (22 modules):
  `bitio`, `leb128`, `obu`, `transform`, `predict`, `scan`, `quant`, `color`, `levels`,
  `cdf_tables`, `ec`, `frame_header`, `seq_header`, `tile`, `block`, `coeffs`, `partition`,
  `encoder`, `decoder`, `isobmff`, `avif_writer`.
- **Real AVIF binary encoder** (`avif_export.rs`): `to_avif_bytes` builds a valid ISOBMFF/AVIF
  container (`ftyp`/`mdat`/`meta`/`iloc`/`iinf`/`iprp`/`av1C`/`colr`) wrapping the new AV1 intra
  OBU stream. Round-trip verified: encode→self-decode produces bit-exact RGBA reconstruction.
- **Real HDF5 binary encoder** (`hdf5_weights_export.rs`): `to_hdf5_bytes` writes a valid HDF5
  superblock v0 file — signature, object headers v1, Dataspace/Datatype/DataLayout messages, local
  heap (HEAP), v1 B-tree (TREE), symbol-table node (SNOD), and little-endian raw data blocks.
  `Hdf5Payload` enum supports Float32/Float64/Int32/Int64/Uint8. Added `export_mesh_hdf5_bytes`.
- **Thrift Compact protocol encoder** (`thrift_export.rs`): `ThriftCompactEncoder` with
  field-delta encoding, zigzag varints, compact type ids, nested struct begin/end, and list headers.
- **Real Parquet binary encoder** (`parquet_stub_export.rs`): `to_parquet_bytes` produces a valid
  Parquet file — PAR1 magic, PLAIN-encoded data pages, compact-Thrift page headers, compact-Thrift
  `FileMetaData` footer, footer-length u32 LE, trailing PAR1. Supports Int32/Int64/Float/Double/
  ByteArray/Boolean column types.
- **Real Voronoi cell area** (`mesh_voronoi.rs`): fan-triangulation of member vertices around the
  centroid gives exact geometric area; `cell.area` now reflects real surface area.
- **Real fluid surface extraction** (`mesh_fluid_surface.rs`): wired to `ScalarField` +
  `marching_cubes`; Laplacian smoothing and normal recomputation applied.
- **Multi-axis twist/bend/taper modifiers** (`mesh_twist/bend/taper_modifier.rs`): `match
  params.axis` correctly applies the transformation along X, Y, or Z axes.
- **QEF-based dual contouring** (`mesh_dual_contour.rs`): `qef_solve_lstsq` via Gaussian
  elimination places vertices at the least-squares minimiser of the Quadratic Error Function.
- **Gordon boolean-sum surfaces** (`mesh_gordon_surface.rs`): `tessellate_gordon` implements
  `S(u,v) = L_u + L_v − B` with polyline interpolation and bilinear blend.
- **Draco compression-level wiring** (`draco_compress.rs`): `compress_mesh` now gates Huffman
  entropy encoding on `compression_level`; higher levels produce smaller output.
- **Network RST race fix** (`oxihuman-core/network.rs`): `accept_one_in_background` parks the
  accepted connection for 500 ms, eliminating the test-time RST race in `send_increments_count`.

### Changed
- Workspace version bumped from `0.1.7` to `0.1.8`.

## [0.1.7] - 2026-06-10

### Added
- **Real ONNX protobuf binary encoder** (`onnx_export.rs`): `to_onnx_bytes` generates a valid
  protobuf-encoded ONNX `ModelProto` binary (ir_version, model_version, opset_import, graph with
  nodes/inputs/outputs/ValueInfoProto/TypeProto). Uses `ProtoEncoder` from `protobuf_export.rs`.
- **Real PSD binary encoder** (`psd_export.rs`): `to_psd_bytes` produces a valid Adobe Photoshop
  PSD file — all 5 sections (file header, color mode, image resources with 72-DPI resolution block,
  layer-and-mask info with real layer records, merged image data); supports multi-layer documents.
- **Real FlatBuffers binary encoder** (`flatbuf_stub_export.rs`): `to_flatbuf_bytes` and
  `flatbuf_encode_mesh` produce valid FlatBuffers binary (vtable-based layout, string vectors, u32
  and f32 vectors, correct soffset/vtable-size/object-size fields) using `FbBinaryBuilder`.
- **Real Cap'n Proto binary export** (`capnp_stub_export.rs`): `to_capnp_bytes` builds a single-
  segment Cap'n Proto message encoding `file_id` and struct count using `CapnSegment`/`CapnMessage`/
  `serialize_message` from core; `export_mesh_capnp_binary` encodes vertex/index lists with proper
  `ListPointer` fields.
- **FACS coactivation synergy rules** (`facs_synergy.rs`): `FacsCoactivationRules` with 12 rules
  covering synergistic pairs (AU6+AU12 Duchenne smile, AU1+AU2 surprise, AU23+AU24 anger, etc.),
  antagonistic pairs (AU1+AU4, AU12+AU15, AU6+AU7, AU25+AU28), and gate rules (AU25→AU26,
  AU5→AU7). `apply_coactivation_rules` applies all rules non-cascade (reads original state).
- **Position-Based Fluids solver** (`pbf_solver.rs`): Full Macklin & Müller 2013 algorithm —
  Poly6 density kernel, Spiky gradient kernel, λ computation with CFM relaxation (ε_spiky=0.01h²),
  iterative position correction, domain clamping, XSPH viscosity. O(n²) neighbour search.

### Testing
- 33,124 tests · 0 warnings · 0 clippy warnings (48 new tests vs 0.1.6)

## [0.1.6] - 2026-06-10

### Added
- **Real DLB dual-quaternion skinning** (`dual_quaternion_skin.rs`): `dqs_transform_vertex` and
  `dqs_transform_all` — weighted blend of `bone_dqs`, antipodal-flip correction, normalize, then
  position transform via real quaternion sandwich + dual-part translation extraction.
- **Real AO hemisphere sampler** (`ao_renderer.rs`): `compute_ao_at_vertex` now samples 16
  Hammersley low-discrepancy hemisphere directions (Van der Corput base-2, cosine-weighted),
  accumulates visibility via solid-angle integral, no longer returns the no-op `PI/PI` constant.
- **Real APNG/PNG encoder** (`apng_export.rs`): PNG signature + `IHDR`/`acTL`/`fcTL`/`IDAT`/
  `fdAT`/`IEND` chunks with CRC32 (core `crc32`), zlib-deflated scanlines (`oxiarc-deflate`),
  valid single-frame PNG and multi-frame APNG binary output.
- **Real `csg_to_mesh`** (`mesh_boolean_csg.rs`): converts SDF CSG (sphere+box ops) to a triangle
  mesh via `marching_cubes` on a sampled `ScalarField`.

### Changed
- **Real morph evaluators** — 12 previously zero-returning evaluators now compute their documented
  algorithms using data already present on each struct:
  - `nbs_forward` (`neural_blend_shape.rs`): single-layer MLP `Σ w·x + b` with ReLU/Tanh/Sigmoid.
  - `lc_evaluate` (`learned_corrective.rs`): accumulates `Σ activation_weight·delta` per entry.
  - `ddr_evaluate` (`data_driven_rig.rs`): inverse-distance-weighted regression over training samples.
  - `ebt_evaluate` (`emotion_blend_tree.rs`): recursive DFS tree walk with Add/Multiply/Override blend ops.
  - `apm_evaluate` (`age_progression_morph.rs`): piecewise-linear interpolation between age stages.
  - `ebmrph_evaluate` (`ethnic_blend_morph.rs`): normalised weighted blend of feature sets.
  - `ebm_evaluate` (`example_based_morph.rs`): rest-pose + `Σ weight·(example−rest)` blend shapes.
  - `psd_evaluate` (`pose_space_deform.rs`): full normalised `Σ w_e·delta_e / Σ w_e` (was argmax copy).
  - `randomize_gaussian_stub` (`param_randomizer.rs`): Box–Muller transform (was midpoint bias).
  - `vdm_process` (`voice_driven_morph.rs`): Goertzel DFT band-energy to morph weight mapping.
  - `gds_evaluate` (`gaze_driven_shape.rs`): yaw/pitch gain mapping to directional morph channels.
  - `pw_evaluate_with_positions` (`procedural_wrinkle.rs`): per-region distance/falloff displacement.
- **Real mesh boolean operations** — union/difference/intersection now use shared volumetric SDF path
  (`compute_sdf_on_bounds` on combined AABB + `sdf_union/subtraction/intersection` + `sdf_to_mesh`)
  instead of concatenation/identity stubs. `mesh_boolean_ops.rs` routes to per-op implementations.
- **Real `slice_mesh_with_plane`** (`mesh_slice_plane.rs`): per-triangle Sutherland–Hodgman clip
  with real `edge_plane_intersect` edge splitting; was centroid-side partition.
- **Real `remesh_from_voxels`** (`mesh_voxel_remesh.rs`): occupancy grid → `ScalarField` → full
  marching cubes with optional Laplacian smoothing; was heuristic count.
- **Real `poisson_reconstruct_stub`** (`mesh_poisson_recon.rs`): now delegates to the real
  `PoissonReconstructor::reconstruct`; was returning input points unchanged.
- **Real topological invariants** (`topological_insulator.rs`): `z2_invariant`, `chern_number`,
  `anomalous_hall_conductance_stub` now derive from `exchange_gap`/`bulk_gap` band-inversion
  condition (Fu–Kane Dirac surface model); were always-constant.
- **Real EPA solver** (`epa_solver.rs`): `epa_stub` runs full iterative polytope expansion using
  `minkowski_support` from `gjk_epa.rs`, horizon edge extraction, face stitching; was
  consecutive-vertex-triple scan.
- **Shared SDF primitive** (`mesh_sdf.rs`): extracted `compute_sdf_on_bounds(mesh, mn, mx, res,
  sign) -> SdfGrid` and `combined_aabb(a, b, padding)` as building blocks for mesh booleans.

### Testing
- **33,076 tests** (up from 32,976 in 0.1.5) — 100 new tests across morph/mesh/physics/viewer/export.
- 0 clippy warnings · 0 `unwrap` in production code.

## [0.1.5] - 2026-06-10

### Added
- **Real ZIP archive reader** (`archive_reader.rs`): `open_archive(path)` parses ZIP files from
  disk — EOCD scan, central directory walk, local file header dispatch; method 0 (Stored) and
  method 8 (DEFLATE via `oxiarc-deflate`) supported. `open_archive_stub` now delegates to the real
  parser, returning an empty reader on error for backward compatibility.
- **Real async signal with blocking wait** (`async_signal.rs`): `AsyncSignal` now wraps
  `Arc<(Mutex<SignalInner>, Condvar)>` — multiple threads share the same signal handle via
  `Clone`. `signal_wait` blocks until set (Mesa-style condvar loop, spurious-wakeup safe);
  `signal_wait_timeout(sig, ms)` timed wait returning `false` on expiry. `signal_wait_stub`
  deprecated alias kept.
- **Real baseline JPEG codec** (`image_jpeg.rs`): encoder — RGB→YCbCr, 8×8 forward DCT (reference
  formula, separable), standard Annex K quantization tables (quality-scaled), zigzag ordering,
  standard Huffman tables, DPCM/RLE, JFIF framing with byte-stuffing. Decoder — marker-driven
  state machine, IDCT, YCbCr→RGB. Wired into `encode_stub`/`decode_stub`.
- **Real GIF89a codec** (`image_gif.rs`): encoder — median-cut 256-colour quantisation, LZW
  variable-width (9–12 bit) LSB-first compressor, GIF89a framing with sub-block output. Decoder —
  header/LSD/GCT/extension skipping, LZW decompressor, palette mapping. Wired into `encode_stub`/`decode_stub`.
- **Real WebP VP8L (lossless) codec** (`image_webp.rs`): encoder — RIFF/WEBP container, VP8L
  bitstream with canonical Huffman coding (bit-reversed for LSB-first), Simple1/Simple2/Standard
  tree types, 5-channel coding (G, R, B, A, distance), literal-only mode. Decoder — full mirror of
  encoder. Wired into `encode_stub`/`decode_stub`.
- **Real baseline TIFF codec** (`image_tiff.rs`): encoder — little-endian, 12 IFD tags
  (ImageWidth/Length, BitsPerSample, Compression=1/uncompressed, PhotometricInterpretation=2/RGB,
  StripOffsets/ByteCounts, SamplesPerPixel, RowsPerStrip, XResolution/YResolution 72dpi,
  ResolutionUnit), uncompressed RGB24. Decoder — both LE (`II`) and BE (`MM`) byte-order,
  validates Compression=1. Wired into `encode_stub`/`decode_stub`.
- **Real WASM MVP bytecode interpreter** (`wasm_bridge.rs`): `wasm_call(bridge, name, args) ->
  Result<Vec<WasmValue>, WasmError>` executes WASM bytecode against shared linear memory. Stack
  machine interpreter with 100+ opcodes: all control flow (block/loop/if/else/br/br_if/return),
  parametric, local get/set/tee, all memory load/store variants, i32/i64/f32/f64 arithmetic and
  conversions. LEB128 decoding, block scanner for structured control flow. `wasm_call_stub` is now
  a deprecated alias calling `wasm_call`.

### Fixed
- **wgpu 29 API compatibility** (`oxihuman-viewer`): Updated `PipelineLayoutDescriptor.bind_group_layouts`
  to `&[Option<&BindGroupLayout>]` and `DepthStencilState.depth_write_enabled`/`depth_compare` to
  `Option<bool>`/`Option<CompareFunction>` per wgpu 29 API changes.

## [0.1.4] - 2026-05-29

### Added
- **Real SHA-256 (FIPS 180-4, pure Rust)** (`hashing_sha256.rs`): full 64-round compression
  function, correct message schedule (σ₀/σ₁), HMAC-SHA256 per RFC 2104. Validated against NIST
  known-answer tests. `sha256_hash` / `hmac_sha256_stub` now produce authentic digests; `hmac_sha256_stub`
  renamed `hmac_sha256` (deprecated alias kept).
- **Real BLAKE3 (pure Rust, spec-faithful)** (`hashing_blake3.rs`): G mixing function (7 rounds,
  BLAKE2s-derived), chunk state machine (64-byte blocks, 1024-byte chunks), parent-node compression,
  keyed mode. Validated against the BLAKE3 specification known-answer test vector for empty input.
- **Real xxHash64/32 (pure Rust, spec-faithful)** (`hashing_xxhash.rs`): 4-accumulator stripe
  processing (standard prime constants), correct finalization with avalanche mixing. `xxhash64` /
  `xxhash32` now produce values matching the xxHash reference implementation.
- **Real URL percent-encoding / decoding** (`encoding_utils.rs`): full RFC 3986 percent-encoding of
  all reserved/unsafe characters (not just spaces). `url_encode_stub` / `url_decode_stub` replaced
  with complete implementations (deprecated names kept).
- **Real Myers O(ND) diff** (`text_diff_myers.rs`): classic O(ND) algorithm with backtracking
  produces the shortest edit script, correctly handling inserts/deletes interleaved with equals.
  `edit_distance` now returns the true Levenshtein distance.
- **Real patience diff** (`patience_diff.rs`): LCS-of-unique-lines anchor discovery + recursive
  Myers diff on segments between anchors; produces cleaner, more readable diffs for code.
- **Real ear-clip polygon triangulation** (`mesh_triangulate.rs`): correct O(n²) ear-clip
  algorithm for simple polygons using 2D cross-product convexity and point-in-triangle tests;
  exposed via `triangulate_earclip_2d(polygon, positions2d)` and hooked into the
  `TriangulateMethod::EarClip` dispatcher when positions are supplied.
- `checksum_verifier.rs`: `sha256_stub` and inline xxHash64 now delegate to the real hash functions.

### Changed
- `hmac_sha256_stub` — deprecated alias for `hmac_sha256` (the name was always a misnomer).

## [0.1.3] - 2026-05-29

### Added
- **Real ChaCha20-Poly1305 AEAD** (`encryption_chacha.rs`): complete RFC 8439 implementation —
  ChaCha20 quarter-round / block function, Poly1305 authenticator, full AEAD construction with AAD.
  Validated against RFC 8439 §2.8.2 known-answer tests. Functions `chacha_encrypt` /
  `chacha_decrypt` now produce authentic ChaCha20-Poly1305 ciphertext; added `chacha_encrypt_aad` /
  `chacha_decrypt_aad` for explicit AAD.
- **Real AES-128/256-GCM AEAD** (`encryption_aes.rs`): FIPS-197 AES block cipher (S-box, key
  expansion, MixColumns / ShiftRows / AddRoundKey) + GCM mode (CTR counter + GHASH over GF(2¹²⁸)).
  `aes_gcm_encrypt` / `aes_gcm_decrypt` signatures unchanged; output framing is `nonce‖tag‖ct`.
  Validated against NIST SP800-38D known-answer tests for AES-128 and AES-256.
- **Real LZ4 block compression** (`compression_lz4.rs`): hash-chain match finder, LZ4 sequence
  token encoding (literal-length + match-length tokens, 4-byte minimum match, 16-bit back-reference
  offsets). Wire-compatible with the LZ4 block format specification.
- **Real Snappy compression** (`compression_snappy.rs`): varint uncompressed-length preamble +
  literal / copy-with-1-byte-offset / copy-with-2-byte-offset / copy-with-4-byte-offset tags.
  Wire-compatible with the Snappy framing-free block format.
- **Real LZ77/LZSS compression** (`compression_lz.rs`): sliding-window match search with
  length/distance token encoding; genuine compression ratio on repetitive data.
- **Real zstd compression** (`compression_zstd.rs`): spec-valid frames (magic `0xFD2FB528`, frame
  header, single raw/Huffman compressed block). Genuine Huffman + LZ77 entropy coding; real
  compression ratios. Self-consistent; round-trip verified.
- **Real brotli compression** (`compression_brotli.rs`): single meta-block with Huffman-coded
  literals and backward references. Genuine compression; round-trip verified.
- **Real compression pipeline** (`compression_pipeline.rs`): `CompressionStage` now routes through
  the real codecs above; `estimate_compressed_size` measures actual output instead of a fictional
  10%-shrink.
- **Real OAuth2 RFC-6749 protocol** (`oauth2.rs`): RFC-6749 form-urlencoded request-body builders
  for the authorization-code (§4.1.3 + PKCE verifier) and token-refresh (§6) grants; real
  `serde_json`-based token-response parsing (§5.1 / error §5.2); URL-encoded `build_authorization_url`;
  `OAuth2Transport` trait for caller-supplied HTTP transport (mock in tests).
- **Real linear blend skinning** (`fast_lbs.rs`): `fast_lbs_transform` now computes
  `Σ wᵢ · (Mᵢ · [x,y,z,1])` over the ≤4 bone records — identity matrix returns original position,
  rigid transforms produce correct blended output.
- **Real mean curvature map** (`engine_io.rs`, `wasm_api.rs`): `get_curvature_map` wired to
  `oxihuman_mesh::compute_mean_curvature` (cotangent-weight discrete Laplacian); WASM API now
  returns per-vertex curvature values instead of all-zeros.
- **Real consistent winding propagation** (`mesh_face_flip.rs`): BFS across shared edges, flipping
  faces whose orientation disagrees with the seed face.
- **Real iterative edge-flow smoothing** (`mesh_edge_flow.rs`): Laplacian relaxation of the flow
  field that honours the `iterations` parameter.
- **Real partition rebalancing** (`graph_partition.rs`): Kernighan–Lin-style boundary moves that
  equalize partition sizes while reducing edge cut.
- **Real constraint island splitting** (`constraint_island.rs`): union-find / BFS connected-component
  decomposition of the constraint graph.
- **Real render-graph topological sort** (`render_graph_builder.rs`): Kahn's algorithm over node
  dependency edges replaces the faked `0..n` order.
- **Real boundary-edge loop extraction** (`mesh_edge_boundary_detect.rs`): chains valence-1 edges
  into ordered boundary loops.
- **`oxihuman-test-utils` crate** (`crates/oxihuman-test-utils/`, `publish = false`): canonical
  `makehuman_data_dir()` / `targets_dir()` / `base_obj()` / `assets_dir()` test helpers. Eliminates
  6 duplicate definitions and 13 inline env-var blocks across 7 crates.

### Fixed
- Removed stale "stub: no-op" comment in `sparse_blend_shape.rs` — the implementation has been
  correct since 0.1.1.
- `mesh_multiresolution.rs`: replaced `.expect("at least one level")` with a fallible `Option`
  return so zero-level `MultiResolution` values are handled without panicking.

### Changed
- `gc_stub.rs` renamed to `gc.rs`; `GcStub` → `Gc`; `new_gc_stub()` → `new_gc()`. The type is
  re-exported from `oxihuman_core` as before; only the name changed.

## [0.1.2] - 2026-05-05

### Changed
- Test fixtures that depend on the MakeHuman dataset now resolve the data root via the
  `MAKEHUMAN_DATA_DIR` environment variable instead of hard-coded absolute paths.
  Set it to the MakeHuman `data/` directory (the one containing `3dobjs/base.obj` and `targets/`).
  Tests that require this data skip gracefully when the variable is unset.
- Asset pack tests resolve the asset root via `OXIHUMAN_ASSETS_DIR` (the directory containing
  `alpha_pack/oxihuman_assets.toml`). Tests skip gracefully when the variable is unset.

## [0.1.1] - 2026-03-13

### Added
- FABRIK IK solver (`fabrik_ik.rs`) with constrained variant (pole vector, cone angle limit)
- XPBD secondary motion system with Pin/Length/Volume constraints and self-collision detection
- BVH animation retargeting bridge: `parse_bvh_text`, `BvhData`, `BvhJointFrame`, `SkeletonMapping`
- USD time-sampled blend shape animation: `BlendShapeTimeSamples` and `UsdaWriter::write_blend_shape_animation`
- Interactive asset pack builder CLI wizard (`pack-wizard` command)
- Cargo fuzz targets for parser, OXP reader, and USD writer

### Fixed
- Clippy `div_ceil` warning in `fbx_binary.rs:476`

### Refactored
- `oxihuman-wasm` engine split into focused modules: `engine_core`, `engine_anim`, `engine_targets`, `engine_io`
- WASM tests extracted to `wasm_tests.rs` (lib.rs reduced from 1208L to <80L)

### Refactored (Policy compliance — 2 000-line limit)
- **oxihuman-physics** `lib.rs` 3 858 → 117 lines; extracted `proxy_types.rs`, `proxy_gen.rs`, `proxy_tests.rs`, and `modules_a.rs`–`modules_g.rs`
- **oxihuman-morph** `lib.rs` 3 438 → 37 lines; split into `_morph_part1.rs`–`_morph_part6.rs`
- **oxihuman-mesh** `lib.rs` 3 323 → 72 lines; extracted `color_utils.rs` and `mods_a.rs`–`mods_e.rs`
- **oxihuman-export** `lib.rs` 3 137 → 60 lines; split into `_export_part1.rs`–`_export_part6.rs`
- **oxihuman-core** `lib.rs` 2 609 → 43 lines; split into `_core_part1.rs`–`_core_part3.rs`

### Added
- **FBX binary**: zlib-compressed arrays for `I32Array`/`F32Array`/`F64Array` variants (>512 elements, encoding=1) via `miniz_oxide`; new `export_mesh_fbx_binary()` convenience function; ASCII FBX path deprecated since 0.1.1
- **Alembic Ogawa**: `AlembicWriter::from_mesh_buffers()`, `from_mesh_sequence()`, `to_bytes()`, `write_to_file()`, `frame_count()` convenience API; "stub" label removed from docs
- **oxihuman-viewer**: 8 criterion benchmarks (`lod_select`, `lod_chain_build`, `morph_updater_dirty/clean`, `camera_orbit`, `render_stats_snapshot`, `lod_transition_hysteresis`)
- **oxihuman-core**: 17 criterion benchmarks (bloom filter, kd-tree, octree, radix sort, skip list, string pool, asset cache, dependency resolver, spatial hash 2D)
- **Asset pack distribution**: `generate_distribution_manifest()` and `verify_distribution_manifest()` in `asset_pack_builder`; CLI subcommands `pack-dist-manifest` and `pack-verify-dist`; `docs/asset-pack-distribution.md`

## [0.1.0] - 2026-03-11

### Added

#### oxihuman-core
- Core data structures: arena allocator, splay tree, AVL tree, B-tree, bloom filters
- Graph algorithms: BFS shortest path, Bellman-Ford, Floyd-Warshall, Edmonds-Karp max flow, bipartite matching
- Asset management: registry, cache, hash (SHA-256), manifest, versioning
- Codec utilities: base64, base58, arithmetic coding, Huffman coding, CBOR, Avro stubs
- Spatial structures: AABB tree, BVH, spatial hash 2D, HyperLogLog, Count-Min sketch
- String/text: Aho-Corasick, Z-algorithm, indent detector, sentence splitter
- Math primitives: matrix3, color utilities, angle utils, bezier curves

#### oxihuman-morph
- MorphEngine: target-based parametric morphing over thousands of vertices
- Age model, body composition, anthropometry, muscle simulation
- FACS facial action units, pose graph, skin deformation
- Blendshape interpolation, diversity parameters, body symmetry
- Animation curves, param animation, anim retarget
- Constraint system, regions, nasal/hip controls

#### oxihuman-mesh
- Mesh data model: vertex groups, halfedge topology
- Dual contouring (DC) surface extraction
- Sharp feature detection and preservation
- UV mapping, UV packing, UV stitching, UV quality analysis
- LOD generation, mesh decimation
- Parametric surfaces, Gordon surfaces, Coons patches
- Normal map baking, bent normals, cage lattice deform
- Cloth pins, shape key mixing, paint mask

#### oxihuman-export
- GLB/glTF binary export with bytemuck zero-copy packing
- COLLADA export, STL export, SVG projection export
- Vertex animation export, animation layer export
- Texture packing, diffuse color export
- Asset signing and pack verification
- Streaming export, LOD export, morph quantization export

#### oxihuman-physics
- Capsule pair collision detection
- Porous flow (Darcy pressure solver)
- Lattice Boltzmann fluid simulation stub
- Runge-Kutta 4th-order integrator
- Hyperelastic, anisotropic, foam, auxetic material models
- Phase-field fracture, plate bending, creep/fatigue models
- Particle filter, tissue deformation

#### oxihuman-viewer
- wgpu/WebGPU rendering adapter (optional `webgpu` feature)
- LOD manager, depth linearization, instance batching
- Debug views: bent normals, thermal, false-color, histology

#### oxihuman-wasm
- `WasmEngine` — full browser-ready API via `wasm-bindgen`
- Target loading from bytes, ZIP pack loading
- Parameter get/set, mesh export to bytes
- Physics step, animation seek, preset application

#### oxihuman-cli
- `oxihuman generate` — generate mesh from parameter JSON
- `oxihuman export-gltf` / `export-collada` / `export-stl`
- `oxihuman morph-export`, `lod-export`, `proxies`
- `oxihuman asset-bundle`, `zip-pack`, `sign-pack`, `verify-sign`
- `oxihuman stats`, `validate`, `report`

[0.2.0]: https://github.com/cool-japan/oxihuman/releases/tag/v0.2.0
