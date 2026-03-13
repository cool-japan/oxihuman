// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Minimal example: generate a mesh with MorphEngine and export to a GLB file.
//!
//! Demonstrates the simplest end-to-end pipeline:
//!   ParamState → HumanEngine::build_mesh → MeshBuffers → export_auto → .glb

use anyhow::Result;
use oxihuman::export::export_auto;
use oxihuman::mesh::MeshBuffers;
use oxihuman::morph::{HumanEngine, ParamState};
use oxihuman_core::parser::obj::parse_obj;
use oxihuman_core::policy::{Policy, PolicyProfile};

/// Minimal valid OBJ — two triangles forming a quad.
const BASE_OBJ: &str = "\
v 0.0 0.0 0.0\n\
v 1.0 0.0 0.0\n\
v 1.0 1.0 0.0\n\
v 0.0 1.0 0.0\n\
vn 0.0 0.0 1.0\n\
vt 0.0 0.0\n\
vt 1.0 0.0\n\
vt 1.0 1.0\n\
vt 0.0 1.0\n\
f 1/1/1 2/2/1 3/3/1\n\
f 1/1/1 3/3/1 4/4/1\n\
";

fn main() -> Result<()> {
    // 1. Parse base mesh
    let base_obj = parse_obj(BASE_OBJ)?;

    // 2. Build morphology engine with standard content policy
    let policy = Policy::new(PolicyProfile::Standard);
    let mut engine = HumanEngine::new(base_obj, policy);

    // 3. Set morph parameters (all normalised to [0, 1])
    //    height=0.75, weight=0.45, muscle=0.55, age=0.35
    engine.set_params(ParamState::new(0.75, 0.45, 0.55, 0.35));

    // 4. Build mesh via the morph engine
    let morph_buffers = engine.build_mesh();
    let vertex_count = morph_buffers.positions.len();

    // 5. Convert to export-ready MeshBuffers
    let mesh = MeshBuffers::from_morph(morph_buffers);

    // 6. Export to a GLB file in the system temp directory
    let out_path = std::env::temp_dir().join("basic_generation.glb");
    export_auto(&mesh, &out_path)?;

    let byte_count = std::fs::metadata(&out_path)?.len();
    println!(
        "Generated mesh: {} vertices | GLB file: {} bytes → {}",
        vertex_count,
        byte_count,
        out_path.display()
    );

    Ok(())
}
