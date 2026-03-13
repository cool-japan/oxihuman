// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Miscellaneous subcommands: batch-chars, proxies, remesh, physics-export.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use oxihuman_export::{
    generate_param_grid, run_batch, specs_from_param_grid, BatchConfig, BatchOutputFormat,
};
use oxihuman_mesh::MeshBuffers;
use oxihuman_physics::generate_proxies;

// ── proxies ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_proxies(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut base_path: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut i = 0usize;

    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base_path = Some(args.get(i).cloned().context("--base requires a path")?);
            }
            "--output" => {
                i += 1;
                output_path = Some(args.get(i).cloned().context("--output requires a path")?);
            }
            "--json" => {} // JSON is the only supported output format
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base_path.context("--base <PATH> is required")?;
    let src = std::fs::read_to_string(&base).with_context(|| format!("reading OBJ: {}", base))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let morph_buf = oxihuman_morph::engine::MeshBuffers {
        positions: obj.positions,
        normals: obj.normals,
        uvs: obj.uvs,
        indices: obj.indices,
        has_suit: false,
    };
    let mesh = MeshBuffers::from_morph(morph_buf);

    let proxies =
        generate_proxies(&mesh).context("could not generate proxies — mesh may be empty")?;

    // Serialize to JSON manually (no serde derive on BodyProxies)
    let mut capsule_arr = Vec::new();
    for c in &proxies.capsules {
        capsule_arr.push(serde_json::json!({
            "label":    c.label,
            "center_a": c.center_a,
            "center_b": c.center_b,
            "radius":   c.radius,
        }));
    }
    let mut sphere_arr = Vec::new();
    for s in &proxies.spheres {
        sphere_arr.push(serde_json::json!({
            "label":  s.label,
            "center": s.center,
            "radius": s.radius,
        }));
    }
    let mut box_arr = Vec::new();
    for b in &proxies.boxes {
        box_arr.push(serde_json::json!({
            "label":        b.label,
            "center":       b.center,
            "half_extents": b.half_extents,
        }));
    }
    let json = serde_json::json!({
        "capsules": capsule_arr,
        "spheres":  sphere_arr,
        "boxes":    box_arr,
        "total":    proxies.total_count(),
    });

    let output = serde_json::to_string_pretty(&json)?;

    match output_path {
        Some(p) => {
            std::fs::write(&p, &output).with_context(|| format!("writing output to {}", p))?
        }
        None => println!("{}", output),
    }

    Ok(())
}

// ── batch-chars ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_batch_chars(args: &[String]) -> Result<()> {
    let mut out_dir: Option<PathBuf> = None;
    let mut format_str = String::from("glb");
    let mut height_steps: usize = 3;
    let mut weight_steps: usize = 3;
    let mut age_steps: usize = 2;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => {
                i += 1;
                out_dir = Some(PathBuf::from(&args[i]));
            }
            "--format" => {
                i += 1;
                format_str = args[i].clone();
            }
            "--height-steps" => {
                i += 1;
                height_steps = args[i].parse().with_context(|| "parsing --height-steps")?;
            }
            "--weight-steps" => {
                i += 1;
                weight_steps = args[i].parse().with_context(|| "parsing --weight-steps")?;
            }
            "--age-steps" => {
                i += 1;
                age_steps = args[i].parse().with_context(|| "parsing --age-steps")?;
            }
            other => bail!("batch-chars: unknown option: {}", other),
        }
        i += 1;
    }

    let out_dir = out_dir.context("--out-dir is required for batch-chars")?;
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating output dir: {}", out_dir.display()))?;

    let fmt = match format_str.as_str() {
        "glb" => BatchOutputFormat::Glb,
        "obj" => BatchOutputFormat::Obj,
        "stl" => BatchOutputFormat::Stl,
        "json" => BatchOutputFormat::Json,
        "csv" => BatchOutputFormat::Csv,
        other => bail!("unknown format: {}. Use: glb|obj|stl|json|csv", other),
    };

    let mut ranges = std::collections::HashMap::new();
    ranges.insert("height".to_string(), (0.0f32, 1.0, height_steps));
    ranges.insert("weight".to_string(), (0.0f32, 1.0, weight_steps));
    ranges.insert("age".to_string(), (0.0f32, 1.0, age_steps));

    let grid = generate_param_grid(&ranges);
    let specs = specs_from_param_grid(&grid, fmt, &out_dir);
    let cfg = BatchConfig::default();
    let result = run_batch(&specs, &cfg);

    println!("{}", oxihuman_export::batch_result_summary(&result));
    if !result.errors.is_empty() {
        for (id, err) in &result.errors {
            eprintln!("  FAILED {}: {}", id, err);
        }
    }
    Ok(())
}

// ── remesh ────────────────────────────────────────────────────────────────────

pub fn cmd_remesh(args: &[String]) -> Result<()> {
    let mut input: Option<PathBuf> = None;
    let mut target_edge_len: f32 = 0.05;
    let mut iters: u32 = 3;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target-edge-len" => {
                i += 1;
                let s = args.get(i).context("--target-edge-len requires a value")?;
                target_edge_len = s
                    .parse::<f32>()
                    .context("--target-edge-len must be a float")?;
            }
            "--iters" => {
                i += 1;
                let s = args.get(i).context("--iters requires a value")?;
                iters = s.parse::<u32>().context("--iters must be an integer")?;
            }
            other if !other.starts_with("--") => {
                input = Some(PathBuf::from(other));
            }
            other => bail!("remesh: unknown option: {}", other),
        }
        i += 1;
    }
    let input = input.context("remesh: input file is required")?;
    if !input.exists() {
        bail!("remesh: input file not found: {}", input.display());
    }
    // Stub: report parameters, no actual remesh performed yet
    println!(
        "{}",
        serde_json::json!({
            "command": "remesh",
            "input": input.display().to_string(),
            "target_edge_len": target_edge_len,
            "iters": iters,
            "status": "stub"
        })
    );
    Ok(())
}

// ── physics-export ────────────────────────────────────────────────────────────

pub fn cmd_physics_export(args: &[String]) -> Result<()> {
    use oxihuman_export::{biped_physics_scene, build_physics_extension_json};
    use oxihuman_export::{build_xr_scene_json, default_xr_scene};

    let mut input: Option<PathBuf> = None;
    let mut format = "gltf-physics".to_string();
    let mut output: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                i += 1;
                format = args.get(i).context("--format requires a value")?.clone();
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(
                    args.get(i).context("--output requires a value")?,
                ));
            }
            other if !other.starts_with("--") => {
                input = Some(PathBuf::from(other));
            }
            other => bail!("physics-export: unknown option: {}", other),
        }
        i += 1;
    }
    let input = input.context("physics-export: input file is required")?;
    if !input.exists() {
        bail!("physics-export: input file not found: {}", input.display());
    }

    let json_out = match format.as_str() {
        "gltf-physics" => {
            let scene = biped_physics_scene(14);
            build_physics_extension_json(&scene)
        }
        "openxr" => {
            let scene = default_xr_scene("OxiHuman");
            build_xr_scene_json(&scene)
        }
        other => bail!(
            "physics-export: unknown format '{}'. Use 'gltf-physics' or 'openxr'",
            other
        ),
    };

    if let Some(out_path) = output {
        std::fs::write(&out_path, &json_out)
            .with_context(|| format!("writing physics-export output: {}", out_path.display()))?;
        println!(
            "physics-export: written {} bytes → {}",
            json_out.len(),
            out_path.display()
        );
    } else {
        println!("{}", json_out);
    }
    Ok(())
}
