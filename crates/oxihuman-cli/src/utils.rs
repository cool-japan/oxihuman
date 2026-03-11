// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared utility functions used by multiple CLI commands.

use anyhow::{Context, Result};
use std::path::Path;

use oxihuman_core::policy::Policy;
use oxihuman_export::params_json::import_params;
use oxihuman_mesh::MeshBuffers;
use oxihuman_morph::params::ParamState;
use oxihuman_morph::{apply_expression_to_engine, ExpressionPreset};

/// Load parameters from either an inline JSON string or a file path.
pub fn load_params(src: &str) -> Result<ParamState> {
    let json_str = if Path::new(src).exists() {
        std::fs::read_to_string(src)?
    } else {
        src.to_string()
    };
    let val: serde_json::Value = serde_json::from_str(&json_str)?;
    import_params(&val)
}

/// Build mesh using a manual engine pipeline (used when --expression is specified).
pub fn build_mesh_with_expression(
    base_path: &Path,
    targets_dir: Option<&Path>,
    params: ParamState,
    policy: Policy,
    output: &Path,
    expression_preset: &ExpressionPreset,
) -> Result<MeshBuffers> {
    use oxihuman_core::parser::obj::parse_obj;
    use oxihuman_mesh::normals::compute_normals;
    use oxihuman_mesh::suit::apply_suit_flag;
    use oxihuman_morph::HumanEngine;

    let base_src = std::fs::read_to_string(base_path)
        .with_context(|| format!("reading base OBJ: {}", base_path.display()))?;
    let base_mesh = parse_obj(&base_src).context("parsing base OBJ")?;
    let mut engine = HumanEngine::new(base_mesh, policy);

    // Load body morph targets if a directory was provided
    if let Some(td) = targets_dir {
        if td.exists() {
            if let Ok(n) = engine.load_targets_from_dir_auto(td) {
                eprintln!("OxiHuman: loaded {} body targets", n);
            }
        }
    }

    // Apply expression targets from {targets_dir}/expression/units/caucasian/
    let expr_dir = targets_dir.map(|td| td.join("expression").join("units").join("caucasian"));

    if let Some(ref ed) = expr_dir {
        let expr_count = apply_expression_to_engine(&mut engine, expression_preset, ed);
        if expr_count > 0 {
            eprintln!(
                "OxiHuman: applied {} expression target(s) for '{}'",
                expr_count, expression_preset.name
            );
        } else {
            eprintln!(
                "OxiHuman: expression preset '{}' (apply manually with --targets)",
                expression_preset.name
            );
        }
    } else {
        eprintln!(
            "OxiHuman: expression preset '{}' (apply manually with --targets)",
            expression_preset.name
        );
    }

    engine.set_params(params);
    let morph_buffers = engine.build_mesh();

    let mut mesh = MeshBuffers::from_morph(morph_buffers);
    compute_normals(&mut mesh);
    apply_suit_flag(&mut mesh);

    oxihuman_export::glb::export_glb(&mesh, output)
        .with_context(|| format!("exporting GLB to {}", output.display()))?;

    Ok(mesh)
}

/// Build a MeshBuffers from a base OBJ, optionally applying morph targets.
pub fn build_mesh_from_base(
    base: &Path,
    targets: Option<&Path>,
    params: ParamState,
    policy: Policy,
) -> Result<MeshBuffers> {
    use oxihuman_core::parser::obj::parse_obj;
    use oxihuman_mesh::normals::compute_normals;
    use oxihuman_mesh::suit::apply_suit_flag;
    use oxihuman_morph::HumanEngine;

    let src = std::fs::read_to_string(base)
        .with_context(|| format!("reading base OBJ: {}", base.display()))?;
    let obj = parse_obj(&src).context("parsing base OBJ")?;
    let mut engine = HumanEngine::new(obj, policy);

    if let Some(td) = targets {
        if td.exists() {
            if let Ok(n) = engine.load_targets_from_dir_auto(td) {
                eprintln!("OxiHuman: loaded {} targets", n);
            }
        }
    }

    engine.set_params(params);
    let morph_buf = engine.build_mesh();
    let mut mesh = MeshBuffers::from_morph(morph_buf);
    compute_normals(&mut mesh);
    apply_suit_flag(&mut mesh);
    Ok(mesh)
}
