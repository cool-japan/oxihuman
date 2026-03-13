// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Batch export example: generate three body configurations and export each
//! to a separate GLB file in the system temp directory.
//!
//! Demonstrates parametric variation over height/weight and how to iterate
//! an export loop without heap-allocating random numbers.

use anyhow::Result;
use oxihuman::export::export_auto;
use oxihuman::mesh::MeshBuffers;
use oxihuman::morph::{HumanEngine, ParamState};
use oxihuman_core::parser::obj::parse_obj;
use oxihuman_core::policy::{Policy, PolicyProfile};

/// Minimal base mesh shared across all iterations.
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

/// Deterministic parameter sets: (height, weight, muscle, age, label).
const PARAM_SETS: &[(f32, f32, f32, f32, &str)] = &[
    (0.40, 0.30, 0.50, 0.25, "slim_young"),
    (0.60, 0.55, 0.65, 0.50, "average_adult"),
    (0.80, 0.75, 0.80, 0.70, "tall_heavy_senior"),
];

fn main() -> Result<()> {
    let base_obj = parse_obj(BASE_OBJ)?;
    let policy = Policy::new(PolicyProfile::Standard);
    let mut engine = HumanEngine::new(base_obj, policy);

    let tmp = std::env::temp_dir();

    for &(height, weight, muscle, age, label) in PARAM_SETS {
        // Update morph parameters for this configuration
        engine.set_params(ParamState::new(height, weight, muscle, age));

        // Build mesh and convert to export-ready buffers
        let morph_buffers = engine.build_mesh();
        let vertex_count = morph_buffers.positions.len();
        let mesh = MeshBuffers::from_morph(morph_buffers);

        // Export to <tmp>/batch_<label>.glb
        let out_path = tmp.join(format!("batch_{label}.glb"));
        export_auto(&mesh, &out_path)?;

        let byte_count = std::fs::metadata(&out_path)?.len();
        println!(
            "[{label}] h={height:.2} w={weight:.2} → {} vertices, {} GLB bytes → {}",
            vertex_count,
            byte_count,
            out_path.display()
        );
    }

    println!("Batch export complete ({} files).", PARAM_SETS.len());
    Ok(())
}
