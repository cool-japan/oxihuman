// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! WebAssembly (and native) bindings for the OxiHuman morph engine.
//!
//! The main type is [`WasmEngine`], a thin wrapper around
//! [`oxihuman_morph::engine::HumanEngine`] that exposes a flat, JavaScript-
//! friendly API. In native builds it works exactly the same way — useful for
//! server-side rendering or CLI tools that embed the WASM surface.
//!
//! # Buffer protocol
//!
//! [`WasmEngine::build_mesh_bytes`] returns a compact binary buffer. The
//! layout is described by [`BUFFER_FORMAT_VERSION`]:
//!
//! ```text
//! [version: u32 LE]
//! [n_verts: u32 LE]
//! [n_idx:   u32 LE]
//! [positions: f32 * 3 * n_verts]
//! [normals:   f32 * 3 * n_verts]
//! [uvs:       f32 * 2 * n_verts]
//! [indices:   u32 * n_idx]
//! ```
//!
//! # JavaScript usage example
//!
//! ```js
//! import init, { WasmEngine } from './oxihuman_wasm.js';
//!
//! await init();
//! const engine = WasmEngine.new_from_obj_bytes(objBytes);
//! engine.set_height(0.8);
//! engine.set_weight(0.4);
//! const buf = engine.build_mesh_bytes();          // Uint8Array
//! const view = new DataView(buf.buffer);
//! const nVerts = view.getUint32(4, true);         // little-endian
//! // upload positions starting at byte offset 12 to WebGL / WebGPU
//! ```

pub mod buffer;
pub mod buffer_transfer;
pub mod compressed_target;
pub mod engine;
pub mod error;
pub mod memory_profile;
pub mod pack;
pub mod service_worker;

// Engine implementation sub-modules (private; re-exported via `engine`).
mod engine_anim;
mod engine_core;
mod engine_io;
mod engine_targets;

/// Full wasm-bindgen JS/TS API surface.  Enabled with `--features bindgen`.
#[cfg(feature = "bindgen")]
pub mod wasm_api;

/// TypeScript `.d.ts` supplemental type declarations.  Enabled with `--features bindgen`.
#[cfg(feature = "bindgen")]
pub mod ts_types;

// Re-exports for public API surface.
pub use buffer::parse_mesh_bytes_header;
pub use engine::{Particle, ParticleSystem, WasmEngine};

/// Buffer format tag for the raw mesh bytes returned by `build_mesh_bytes()`.
/// Layout: [n_verts: u32 LE][n_idx: u32 LE][positions: f32*3*n][normals: f32*3*n][uvs: f32*2*n][indices: u32*m]
pub const BUFFER_FORMAT_VERSION: u32 = 1;

#[cfg(test)]
mod wasm_tests;
