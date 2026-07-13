// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Miscellaneous subcommands: batch-chars, proxies, remesh, physics-export.

use anyhow::{bail, ensure, Context, Result};
use std::path::PathBuf;

use oxihuman_export::{
    generate_param_grid, run_batch, specs_from_param_grid, BatchConfig, BatchOutputFormat,
};
use oxihuman_mesh::MeshBuffers;
use oxihuman_physics::generate_proxies;

// ── shared helpers ───────────────────────────────────────────────────────────

/// Load an OBJ file from `path` and convert it into an `oxihuman_mesh`
/// [`MeshBuffers`] via the morph-engine buffer bridge. Shared by `proxies`
/// and `physics-export` so both build their proxy/body geometry from the
/// same actual input mesh rather than duplicating the parse+convert dance.
fn load_mesh_from_obj_path(path: &std::path::Path) -> Result<MeshBuffers> {
    use oxihuman_core::parser::obj::parse_obj;

    let src = std::fs::read_to_string(path)
        .with_context(|| format!("reading OBJ: {}", path.display()))?;
    let obj = parse_obj(&src).with_context(|| format!("parsing OBJ: {}", path.display()))?;
    let morph_buf = oxihuman_morph::engine::MeshBuffers {
        positions: obj.positions,
        normals: obj.normals,
        uvs: obj.uvs,
        indices: obj.indices,
        has_suit: false,
    };
    Ok(MeshBuffers::from_morph(morph_buf))
}

// ── proxies ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_proxies(args: &[String]) -> Result<()> {
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
    let mesh = load_mesh_from_obj_path(std::path::Path::new(&base))?;

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

/// Remesh an OBJ mesh via voxelization + Marching Cubes reconstruction
/// (`oxihuman_mesh::mesh_voxel_remesh`).
///
/// `--voxel-size` controls the resolution of the intermediate occupancy
/// grid (smaller = higher fidelity, more triangles); `--iters` controls
/// the number of Laplacian smoothing passes applied to the reconstructed
/// surface. When `--output` is given, the remeshed geometry is written as
/// an OBJ file; otherwise only the JSON summary is printed.
pub fn cmd_remesh(args: &[String]) -> Result<()> {
    use oxihuman_mesh::mesh_voxel_remesh::{
        remesh_from_voxels, voxelize_mesh_remesh, VoxelRemeshConfig,
    };
    use oxihuman_mesh::normals::compute_normals;
    use oxihuman_mesh::suit::apply_suit_flag;

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut voxel_size: f32 = 0.05;
    let mut iters: u32 = 3;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--voxel-size" => {
                i += 1;
                let s = args.get(i).context("--voxel-size requires a value")?;
                voxel_size = s.parse::<f32>().context("--voxel-size must be a float")?;
            }
            "--iters" => {
                i += 1;
                let s = args.get(i).context("--iters requires a value")?;
                iters = s.parse::<u32>().context("--iters must be an integer")?;
            }
            "--output" => {
                i += 1;
                let s = args.get(i).context("--output requires a value")?;
                output = Some(PathBuf::from(s));
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
    ensure!(
        voxel_size > 0.0,
        "remesh: --voxel-size must be positive, got {}",
        voxel_size
    );

    let mesh = load_mesh_from_obj_path(&input)?;
    ensure!(
        !mesh.positions.is_empty() && !mesh.indices.is_empty(),
        "remesh: input mesh '{}' has no geometry",
        input.display()
    );
    let triangles: Vec<[u32; 3]> = mesh
        .indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    let cfg = VoxelRemeshConfig {
        voxel_size,
        smooth_iterations: iters,
        preserve_boundaries: true,
    };
    let grid = voxelize_mesh_remesh(&mesh.positions, &triangles, &cfg);
    let result = remesh_from_voxels(&grid, &cfg);

    if !result.success {
        bail!(
            "remesh: voxel remesh produced no geometry for '{}' (try a smaller --voxel-size)",
            input.display()
        );
    }

    let mut output_path_str: Option<String> = None;
    if let Some(out_path) = &output {
        let vertex_count = result.vertices.len();
        let morph_buf = oxihuman_morph::engine::MeshBuffers {
            positions: result.vertices.clone(),
            normals: vec![[0.0, 1.0, 0.0]; vertex_count],
            uvs: vec![[0.0, 0.0]; vertex_count],
            indices: result.triangles.clone(),
            has_suit: false,
        };
        let mut out_mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut out_mesh);
        apply_suit_flag(&mut out_mesh);
        oxihuman_export::export_obj(&out_mesh, out_path)
            .with_context(|| format!("writing remeshed OBJ to {}", out_path.display()))?;
        output_path_str = Some(out_path.display().to_string());
    }

    println!(
        "{}",
        serde_json::json!({
            "command": "remesh",
            "input": input.display().to_string(),
            "output": output_path_str,
            "voxel_size": voxel_size,
            "iters": iters,
            "vertex_count": result.vertex_count,
            "triangle_count": result.triangle_count,
            "status": "ok"
        })
    );
    Ok(())
}

// ── physics-export ────────────────────────────────────────────────────────────

/// Estimate a plausible rigid-body mass (kg) from a primitive's volume
/// (cubic mesh units) using a nominal soft-tissue density. Purely a
/// heuristic for the physics rig — real games/engines re-tune per body
/// part — but it scales with the *actual* proxy size instead of using a
/// fixed constant regardless of input.
fn estimate_mass_kg(volume: f32) -> f32 {
    const NOMINAL_DENSITY_KG_PER_UNIT3: f32 = 985.0;
    const MIN_MASS_KG: f32 = 0.05;
    (volume * NOMINAL_DENSITY_KG_PER_UNIT3).max(MIN_MASS_KG)
}

/// Build a `GltfPhysicsScene` from actual [`oxihuman_physics::BodyProxies`]
/// collision geometry, instead of a hardcoded fixed biped layout.
fn physics_scene_from_proxies(
    proxies: &oxihuman_physics::BodyProxies,
) -> oxihuman_export::GltfPhysicsScene {
    use oxihuman_export::gltf_physics::{build_box_shape, build_capsule_shape, build_sphere_shape};
    use oxihuman_export::{GltfPhysicsScene, RigidBodyDescriptor};
    use std::f32::consts::PI;

    let mut rigid_bodies = Vec::new();
    let mut node_idx = 0usize;

    for c in &proxies.capsules {
        let dx = c.center_b[0] - c.center_a[0];
        let dy = c.center_b[1] - c.center_a[1];
        let dz = c.center_b[2] - c.center_a[2];
        let height = (dx * dx + dy * dy + dz * dz).sqrt();
        let r = c.radius.max(1e-4);
        let volume = PI * r * r * height + (4.0 / 3.0) * PI * r * r * r;
        rigid_bodies.push((
            node_idx,
            RigidBodyDescriptor {
                mass: estimate_mass_kg(volume),
                linear_damping: 0.05,
                angular_damping: 0.08,
                is_trigger: false,
                shape: build_capsule_shape(r, height),
            },
        ));
        node_idx += 1;
    }

    for s in &proxies.spheres {
        let r = s.radius.max(1e-4);
        let volume = (4.0 / 3.0) * PI * r * r * r;
        rigid_bodies.push((
            node_idx,
            RigidBodyDescriptor {
                mass: estimate_mass_kg(volume),
                linear_damping: 0.05,
                angular_damping: 0.08,
                is_trigger: false,
                shape: build_sphere_shape(r),
            },
        ));
        node_idx += 1;
    }

    for b in &proxies.boxes {
        let [hx, hy, hz] = b.half_extents;
        let volume = 8.0 * hx.max(1e-4) * hy.max(1e-4) * hz.max(1e-4);
        rigid_bodies.push((
            node_idx,
            RigidBodyDescriptor {
                mass: estimate_mass_kg(volume),
                linear_damping: 0.05,
                angular_damping: 0.08,
                is_trigger: false,
                shape: build_box_shape(b.half_extents),
            },
        ));
        node_idx += 1;
    }

    // Joint topology is not inferred from proxy geometry alone (proxies do
    // not carry a skeletal hierarchy); callers that need an articulated
    // ragdoll should post-process `rigid_bodies` with their own joint graph.
    GltfPhysicsScene {
        rigid_bodies,
        joints: Vec::new(),
    }
}

pub fn cmd_physics_export(args: &[String]) -> Result<()> {
    use oxihuman_export::build_physics_extension_json;
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
            let mesh = load_mesh_from_obj_path(&input)?;
            let proxies = generate_proxies(&mesh).with_context(|| {
                format!(
                    "physics-export: could not generate proxies from '{}' — mesh may be empty",
                    input.display()
                )
            })?;
            let scene = physics_scene_from_proxies(&proxies);
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
