//! # OxiHuman
//!
//! Privacy-first, client-side human body generator — pure Rust MakeHuman port.
//!
//! This is the **facade crate** that re-exports all OxiHuman sub-crates
//! under a single, ergonomic namespace.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use oxihuman::prelude::*;
//! ```
//!
//! ## Module Overview
//!
//! | Module | Crate | Description |
//! |--------|-------|-------------|
//! | [`core`] | `oxihuman-core` | Foundation: parsers, policies, asset management, data structures |
//! | [`morph`] | `oxihuman-morph` | Morphology engine: parameters, blendshapes, synthesis |
//! | [`mesh`] | `oxihuman-mesh` | Geometry processing: decimation, subdivision, UV, topology |
//! | [`export`] | `oxihuman-export` | Export pipeline: glTF, OBJ, STL, COLLADA, USD, 50+ formats |
//! | [`physics`] | `oxihuman-physics` | Physics: collision proxies, cloth, soft body, ragdoll |
//! | [`viewer`] | `oxihuman-viewer` | Real-time rendering: scene graph, camera, materials (feature-gated) |
//! | [`wasm`] | `oxihuman-wasm` | WebAssembly bindings for browser deployment (feature-gated) |

// ── Core sub-crates (always available) ──────────────────────────────
pub use oxihuman_core as core;
pub use oxihuman_morph as morph;
pub use oxihuman_mesh as mesh;
pub use oxihuman_export as export;
pub use oxihuman_physics as physics;

// ── Optional sub-crates (feature-gated) ─────────────────────────────
#[cfg(feature = "viewer")]
pub use oxihuman_viewer as viewer;

#[cfg(feature = "wasm")]
pub use oxihuman_wasm as wasm;

/// Convenience prelude — imports the most commonly used types.
///
/// ```rust,no_run
/// use oxihuman::prelude::*;
/// ```
pub mod prelude {
    // Core infrastructure
    pub use oxihuman_core::{
        AssetManifest, EventBus, EventKind, Policy, PolicyProfile,
    };

    // Morphology engine
    pub use oxihuman_morph::{HumanEngine, ParamState};

    // Mesh processing
    pub use oxihuman_mesh::MeshBuffers;

    // Export pipeline
    pub use oxihuman_export::{export_auto, ExportFormat, ExportOptions};

    // Physics
    pub use oxihuman_physics::{BodyProxies, CapsuleProxy, SphereProxy};

    // Viewer (optional)
    #[cfg(feature = "viewer")]
    pub use oxihuman_viewer::{CameraState, Scene, Viewer, ViewerConfig};
}
