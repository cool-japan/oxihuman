# **OxiHuman Blueprint & Roadmap (Pure Rust MakeHuman Port \+ WASM Alpha)**

**Vision:** A privacy‑first, client‑side human body (digital twin) generator that runs in the browser (WASM/WebGPU), does **not** send body data to servers, and outputs a safe, clothing‑constrained mesh by design.

---

## **0\. Core Principles**

### **0.1 Security & Privacy**

* **No network by default:** the core crates compile without any HTTP stack.  
* **Client‑side compute:** all morphing/mesh synthesis is local.  
* **Parametric privacy:** persisted state is only normalized parameters (e.g., slider values), not raw scans or geometry.  
* **Deterministic builds:** reproducible pipelines for WASM and native.

### **0.2 Safety by Construction**

* **The Base is The Suit:** the default base mesh is a bodysuit/unified garment.  
* **No naked moment:** the pipeline never emits a “nude mesh stage” in memory/exports.  
* **Target filtering:** undesirable morph targets are blocked at load time and at composition time.  
* **Asset policy:** official asset packs exclude explicit content. Third‑party pack loading is sandboxed and policy‑checked.

### **0.3 Performance**

* **SIMD‑friendly math:** morph application optimized for tens of thousands of vertices.  
* **WASM constraints:** memory layout and streaming designed for browsers.  
* **Incremental updates:** slider changes update only affected deltas.

### **0.4 Modularity**

* Compute core is independent from rendering and UI.  
* Asset ingestion is separated from morph math.  
* Rendering uses a thin adapter over wgpu/WebGPU.

---

## **1\. Product Definition (What It Does)**

### **1.1 User Experience (Alpha)**

* User loads the app in a browser.  
* App downloads a compact asset bundle (base suit mesh \+ safe targets \+ textures).  
* User adjusts sliders: height, weight, proportions, age, muscle, etc.  
* Viewer updates in real time with WebGPU.  
* User can export:  
  * **Safe mesh** (GLB/GLTF) with suit geometry integrated.  
  * **Physics proxy** mesh (coarse collision hull / tetrahedral / voxel proxy).  
  * **Parameter profile** JSON (small, shareable).

### **1.2 Non‑Goals (for Alpha)**

* Photogrammetry / body scan import.  
* Server‑side fitting.  
* Explicit anatomy.  
* High‑end cloth simulation in browser (deferred to later phases).

---

## **2\. Architecture Overview**

### **2.1 Crates & Modules**

**Phase 1 (Rust core):**

* `oxihuman-core`  
  * MakeHuman file parsers: `.target`, `.obj`, `.mhclo`, metadata.  
  * Asset manifest \+ integrity checks.  
  * Policy filters (hard rules \+ allowlist).  
* `oxihuman-morph`  
  * Morph math engine: apply weighted targets to base mesh.  
  * SIMD / SoA vertex buffers.  
  * Constraint system (clamp, smooth, avoid self‑intersection heuristics).  
* `oxihuman-mesh`  
  * Mesh utilities: normals, tangents, topology ops.  
  * LOD generation.  
  * Suit integration pipeline.  
* `oxihuman-export`  
  * GLB/GLTF exporter.  
  * Physics proxy export formats.

**Phase 2 (WASM \+ WebGPU):**

* `oxihuman-wasm`  
  * `wasm-bindgen` API: instantiate engine, set params, retrieve buffers.  
  * Zero‑copy buffer transfer design.  
* `oxihuman-viewer`  
  * WebGPU renderer (wgpu).  
  * Camera controls, lighting presets.  
  * UI hooks (minimal).

**Phase 3 (OxiRS integration):**

* `oxihuman-physics`  
  * Convert mesh → collision primitives / signed distance fields.  
  * Adapter layer to OxiRS simulation.

### **2.2 Data Flow**

1. Load asset pack → parse targets & base mesh → policy filter → cache.  
2. Compute morph deltas per parameter change.  
3. Compose final vertex positions → generate normals/tangents.  
4. Integrate suit geometry (always on) → output render mesh.  
5. Optionally generate physics proxy.

---

## **3\. File Formats & Parsing Blueprint**

### **3.1 MakeHuman `.target`**

* Text file describing vertex deltas.  
* Parse with `nom` into a compact delta buffer:  
  * `vertex_id: u32`  
  * `dx, dy, dz: f32`  
* Store deltas sorted by vertex index for cache‑friendly application.

### **3.2 `.obj` \+ `.mhclo`**

* `.obj`: base mesh (suit) and optional clothing items.  
* `.mhclo`: clothing metadata \+ vertex mapping rules.  
* Normalize everything into:  
  * `Mesh { positions, normals, uvs, indices, groups }`  
  * `ClothingBinding { base_vertex_map, weights, seams }`

### **3.3 Asset Pack Manifest**

* `oxihuman_assets.toml` (or JSON) includes:  
  * Pack version, hash tree.  
  * Base mesh path.  
  * Allowed target list.  
  * Category tags (height/weight/age etc.).  
  * Policy profile (strict / standard).

---

## **4\. Morph Engine Blueprint**

### **4.1 Core Types**

// conceptual API sketch  
pub struct HumanEngine {  
    base: MeshBuffers,  
    targets: TargetLibrary,  
    policy: Policy,  
    params: ParamState,  
}

pub struct ParamState {  
    // normalized 0..1 (or \-1..1) values  
    pub height: f32,  
    pub weight: f32,  
    pub muscle: f32,  
    pub age: f32,  
    // ... extensible  
}

pub struct Target {  
    pub name: String,  
    pub category: TargetCategory,  
    pub deltas: Vec\<Delta\>,  
    pub bounds: ParamBounds,  
}

pub struct Delta {  
    pub vid: u32,  
    pub dx: f32,  
    pub dy: f32,  
    pub dz: f32,  
}

### **4.2 Fast Application Strategy**

* Convert base positions into **SoA** buffers:  
  * `x[]`, `y[]`, `z[]`  
* For each active target, apply deltas with SIMD when possible.  
* Maintain a precomputed list of *active targets* per parameter.

### **4.3 Constraints & Stabilization**

* Parameter clamping and non‑linear response curves.  
* Smoothing across correlated parameters.  
* Optional quick self‑intersection heuristic:  
  * approximate capsule checks for limbs or coarse SDF checks.

---

## **5\. Safety & Ethics Engineering (Technical)**

### **5.1 Policy Layer**

**Policy rules are enforced twice:**

1. **Load‑time**: reject targets/assets that violate rules.  
2. **Compose‑time**: even if a target slips in, composition refuses to output unsafe geometry.

### **5.2 “The Base is The Suit” Implementation**

* Default base mesh is a suit mesh (unitard). This is the canonical topology.  
* Any additional clothing is layered on top.  
* Exporters refuse to export if suit mesh is absent.

### **5.3 Target Filtering**

* Maintain a strict allowlist for official alpha.  
* Heuristic \+ metadata tags:  
  * Category tags must be present.  
  * “explicit” / “sexual” tags are rejected.  
* Hash‑based signature for official packs.

### **5.4 Parametric Privacy**

* Persisted profile is:

{ "height": 0.5, "muscle": 0.2, "age": 0.1 }

* No raw mesh stored unless user explicitly exports.  
* Export UI includes a “safe only” notice.

---

## **6\. WASM \+ WebGPU Alpha Plan**

### **6.1 WASM Binding API**

// conceptual  
const engine \= await OxiHuman.Engine.new(assetPackBytes);  
engine.setParams({ height: 0.52, weight: 0.44, muscle: 0.2, age: 0.1 });  
const { positions, normals, uvs, indices } \= engine.buildMesh();  
viewer.uploadMesh(positions, normals, uvs, indices);

### **6.2 Zero‑Copy Design**

* Use `wasm-bindgen` \+ `Uint8Array` / `Float32Array` views.  
* Keep buffers owned in WASM memory, export raw pointers/length.

### **6.3 Offline‑Friendly Asset Loading (Laos spec)**

* One‑time asset pack download, cached via Service Worker.  
* Compressed targets (zstd) inside pack.  
* Optional “lite pack” with fewer targets and LOD.

---

## **7\. OxiRS Integration Blueprint (Oxi‑Couture)**

### **7.1 Dual Mesh Model**

* **Render mesh:** high detail, suit enforced.  
* **Physics mesh:** simplified proxy.

### **7.2 Proxy Generation Options**

* Voxelization → marching cubes (coarse).  
* Tetrahedralization for soft body (later).  
* Primitive fitting (capsules for limbs) for fast collision.

### **7.3 APIs**

* `to_collision_object()` returns OxiRS collision representation.  
* `simulate_cloth()` (future) uses OxiRS cloth with garment patterns.

---

## **8\. Roadmap (Long)**

### **Phase 0 — Spec & Asset Baseline (2–4 weeks)**

* Define suit base mesh standard \+ topology constraints.  
* Create alpha asset pack (safe targets only).  
* Write policy spec and allowlist.

### **Phase 1 — Core Libraries (4–10 weeks)**

* `.target` parser \+ tests.  
* `.obj` loader \+ mesh normalization.  
* Morph engine v0 with SIMD baseline.  
* Exporter prototype (GLB).  
* Benchmarks: 10k/30k/60k vertices.

### **Phase 2 — WASM Alpha (4–8 weeks)**

* `oxihuman-wasm` bindings.  
* WebGPU viewer alpha.  
* Offline cache \+ asset pack loader.  
* Performance tuning: incremental morph updates.

### **Phase 3 — Physics Integration (6–12 weeks)**

* Proxy mesh generation.  
* OxiRS collision adapter.  
* Basic cloth/fit experiments.

### **Phase 4 — Creator Toolkit (8–16 weeks)**

* Target authoring tools (Rust CLI \+ optional GUI).  
* Pack signing \+ distribution.  
* Parameter schema evolution.

### **Phase 5 — Digital Twin Quality (ongoing)**

* Better constraints.  
* Calibration workflows.  
* Body measurement outputs.

---

## **9\. Testing & Verification**

### **9.1 Unit Tests**

* Parser golden tests for MakeHuman targets.  
* Mesh integrity tests (manifold checks, index bounds).  
* Policy tests: ensure blocked targets are rejected.

### **9.2 Property Tests (proptest)**

* Random parameter sweeps should never produce missing suit mesh.  
* Parameter extremes remain stable (no NaNs, no exploding vertices).

### **9.3 Benchmarks**

* `criterion` benches for:  
  * load \+ parse  
  * morph apply  
  * normal recompute  
  * export

---

## **10\. Risks & Mitigations**

| Risk | Impact | Mitigation |
| ----- | ----- | ----- |
| Asset licensing ambiguity | High | Only ship self‑made or clearly licensed packs. Pack signing. |
| WASM memory pressure | Medium | LOD, streaming, zstd, incremental morph. |
| Self‑intersection artifacts | Medium | Coarse SDF checks, constraints, smoothing. |
| Policy bypass via third‑party packs | High | Strict verification, default deny, sandbox parsing. |
| Performance on low‑end devices | Medium | Lite packs, reduced target set, LOD. |

---

## **11\. Deliverables Checklist (Alpha)**

* `oxihuman-core` parses targets & meshes.  
* `oxihuman-morph` applies targets fast.  
* `oxihuman-wasm` exposes engine to TS.  
* `oxihuman-viewer` renders via WebGPU.  
* Offline caching.  
* Export safe GLB \+ param profile JSON.

---

## **12\. Repo Blueprint (Suggested)**

oxihuman/  
  crates/  
    oxihuman-core/  
    oxihuman-morph/  
    oxihuman-mesh/  
    oxihuman-export/  
    oxihuman-wasm/  
    oxihuman-viewer/  
    oxihuman-physics/  
  assets/  
    alpha\_pack/  
  docs/  
    POLICY.md  
    ARCHITECTURE.md  
    ROADMAP.md

---

## **13\. Next Steps (Immediate)**

1. Freeze the **suit base mesh** and alpha target allowlist.  
2. Implement `.target` parser \+ tests.  
3. Implement morph apply with a minimal parameter set.  
4. Produce a WASM demo that morphs in real time in a browser.

