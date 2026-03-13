// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Info/inspection subcommands: info, session, stats, workspace, validate, target-info,
//! plugin-list, camera-info.

use anyhow::{bail, Context, Result};
use std::path::Path;

use oxihuman_core::default_builtin_plugins;
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_export::glb::verify_glb_header;
use oxihuman_mesh::MeshBuffers;
use oxihuman_morph::session::MorphSession;

// ── session ───────────────────────────────────────────────────────────────────

pub fn cmd_session(args: &[String]) -> Result<()> {
    let file = args
        .first()
        .context("session requires a file argument (path to session JSON)")?;
    let path = Path::new(file);

    if !path.exists() {
        bail!("session file not found: {}", path.display());
    }

    let session = MorphSession::load(path)
        .with_context(|| format!("loading session from {}", path.display()))?;

    println!("Session file:  {}", path.display());
    println!("Version:       {}", session.version);
    if let Some(ref label) = session.label {
        println!("Label:         {}", label);
    }
    println!("Params:");
    println!("  height: {:.4}", session.params.height);
    println!("  weight: {:.4}", session.params.weight);
    println!("  muscle: {:.4}", session.params.muscle);
    println!("  age:    {:.4}", session.params.age);
    if !session.params.extra.is_empty() {
        println!("  extra:");
        let mut extras: Vec<_> = session.params.extra.iter().collect();
        extras.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in extras {
            println!("    {}: {:.4}", k, v);
        }
    }
    if let Some(ref td) = session.targets_dir {
        println!("Targets dir:   {}", td.display());
    }
    if !session.loaded_target_names.is_empty() {
        println!("Loaded targets ({}):", session.loaded_target_names.len());
        for name in session.loaded_target_names.iter().take(10) {
            println!("  - {}", name);
        }
        if session.loaded_target_names.len() > 10 {
            println!("  ... and {} more", session.loaded_target_names.len() - 10);
        }
    }

    Ok(())
}

// ── validate ──────────────────────────────────────────────────────────────────

pub fn cmd_validate(args: &[String]) -> Result<()> {
    // If first arg is --pack, validate a whole manifest
    if args.first().map(|s| s.as_str()) == Some("--pack") {
        let manifest_path = args.get(1).context("--pack requires a manifest path")?;
        let path = std::path::Path::new(manifest_path);
        if !path.exists() {
            bail!("manifest not found: {}", path.display());
        }

        use oxihuman_export::pack::{validate_manifest, EntryStatus, PackManifest};

        let manifest = PackManifest::load(path)?;
        let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
        let policy = Policy::new(PolicyProfile::Standard);
        let report = validate_manifest(&manifest, base_dir, &policy);

        println!("{}", report.summary());
        for r in &report.results {
            let status = match &r.status {
                EntryStatus::Ok => "OK".to_string(),
                EntryStatus::Missing => "MISSING".to_string(),
                EntryStatus::HashMismatch { actual } => {
                    format!("HASH_MISMATCH (got {}...)", &actual[..8])
                }
                EntryStatus::PolicyViolation => "POLICY_VIOLATION".to_string(),
            };
            println!("  [{}] {}", status, r.name);
        }
        return Ok(());
    }

    // Single file validation
    let file = args.first().context("validate requires a file argument")?;
    let path = Path::new(file);

    if !path.exists() {
        bail!("file not found: {}", path.display());
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    match ext {
        "target" => {
            use oxihuman_core::parser::target::parse_target;
            let src = std::fs::read_to_string(path)?;
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let t = parse_target(name, &src)?;
            println!("OK: {} deltas in '{}'", t.deltas.len(), t.name);
            if let Some(first) = t.deltas.first() {
                println!(
                    "  First delta: vid={} dx={:.4} dy={:.4} dz={:.4}",
                    first.vid, first.dx, first.dy, first.dz
                );
            }
            if let Some(last) = t.deltas.last() {
                println!(
                    "  Last delta:  vid={} dx={:.4} dy={:.4} dz={:.4}",
                    last.vid, last.dx, last.dy, last.dz
                );
            }
        }
        "glb" => {
            verify_glb_header(path)?;
            let size = std::fs::metadata(path)?.len();
            println!("OK: valid GLB ({} bytes)", size);
        }
        other => {
            bail!("unsupported file type: .{}", other);
        }
    }

    Ok(())
}

// ── info (file) ───────────────────────────────────────────────────────────────

pub fn cmd_info(args: &[String]) -> Result<()> {
    let file = args.first().context("info requires a file argument")?;
    let path = Path::new(file);
    if !path.exists() {
        bail!("file not found: {}", path.display());
    }

    let size = std::fs::metadata(path)?.len();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    println!("File:      {}", path.display());
    println!("Size:      {} bytes ({:.1} KB)", size, size as f64 / 1024.0);
    println!("Extension: .{}", ext);

    match ext {
        "glb" => {
            verify_glb_header(path)?;
            println!("Format:    GLB 2.0 (valid)");
        }
        "target" => {
            use oxihuman_core::parser::target::parse_target;
            let src = std::fs::read_to_string(path)?;
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let t = parse_target(name, &src)?;
            println!("Format:    MakeHuman .target");
            println!("Deltas:    {}", t.deltas.len());
        }
        "obj" => {
            use oxihuman_core::parser::obj::parse_obj;
            let src = std::fs::read_to_string(path)?;
            let mesh = parse_obj(&src)?;
            println!("Format:    Wavefront OBJ");
            println!("Vertices:  {}", mesh.positions.len());
            println!("Faces:     {}", mesh.indices.len() / 3);
        }
        _ => {}
    }

    Ok(())
}

// ── stats ─────────────────────────────────────────────────────────────────────

pub fn cmd_stats(path: &str, full: bool, json: bool) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let src =
        std::fs::read_to_string(path).with_context(|| format!("reading OBJ file: {}", path))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;

    let morph_buf = oxihuman_morph::engine::MeshBuffers {
        positions: obj.positions,
        normals: obj.normals,
        uvs: obj.uvs,
        indices: obj.indices,
        has_suit: false,
    };
    let mesh = MeshBuffers::from_morph(morph_buf);
    let stats = oxihuman_mesh::compute_stats(&mesh);

    if json {
        let v = serde_json::json!({
            "vertex_count":      stats.vertex_count,
            "face_count":        stats.face_count,
            "edge_count":        stats.edge_count,
            "surface_area":      stats.surface_area,
            "volume_estimate":   stats.volume_estimate,
            "euler_characteristic": stats.euler_characteristic,
            "avg_edge_length":   stats.avg_edge_length,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!("{}", stats.summary());
        if full {
            println!("  avg_aspect_ratio: {:.4}", stats.avg_aspect_ratio);
            println!("  min_face_area:    {:.6}", stats.min_face_area);
            println!("  max_face_area:    {:.6}", stats.max_face_area);
        }
    }
    Ok(())
}

pub fn parse_stats_args(args: &[String]) -> Result<()> {
    let mut full = false;
    let mut json = false;
    let mut path: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--full" => {
                full = true;
            }
            "--json" => {
                json = true;
            }
            other if other.starts_with("--") => bail!("unknown option: {}", other),
            other => {
                if path.is_some() {
                    bail!("stats: unexpected extra argument: {}", other);
                }
                path = Some(other.to_string());
            }
        }
    }

    let path = path.context("stats requires a path to an OBJ file")?;
    cmd_stats(&path, full, json)
}

// ── workspace info ────────────────────────────────────────────────────────────

pub fn cmd_workspace_info() {
    println!("OxiHuman v{}", env!("CARGO_PKG_VERSION"));
    println!("  Repository: https://github.com/cool-japan/oxihuman");
    println!("  License:    Apache-2.0");
    println!();
    println!("Workspace crates:");
    println!("  oxihuman-core    — parsers (.target, .obj, .mhclo), manifest, policy, integrity");
    println!("  oxihuman-morph   — morph engine, params, expressions, fitting, symmetry, history");
    println!("  oxihuman-mesh    — mesh utilities, normals, tangents, LOD, smoothing, bounds");
    println!("  oxihuman-export  — GLB, GLTF, OBJ, STL, USDA, JSON exporters + pipeline");
    println!("  oxihuman-physics — collision proxies, capsule generation, surface sampling");
    println!("  oxihuman-wasm    — WebAssembly bindings (stub)");
    println!("  oxihuman-viewer  — WebGPU viewer (stub)");
    println!("  oxihuman-cli     — command-line interface (this binary)");
}

// ── target-info ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_target_info(args: &[String]) -> Result<()> {
    use oxihuman_core::TargetIndex;
    use std::path::PathBuf;

    let mut target_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target-dir" => {
                i += 1;
                target_dir = Some(PathBuf::from(&args[i]));
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let target_dir = target_dir.context("--target-dir is required for target-info")?;

    if !target_dir.exists() {
        bail!("target directory not found: {}", target_dir.display());
    }

    let idx = TargetIndex::from_dir(&target_dir)
        .with_context(|| format!("loading target index from: {}", target_dir.display()))?;

    println!("Total targets: {}", idx.len());

    // Unique categories.
    let all = idx.all();
    let mut cats: Vec<String> = all
        .iter()
        .map(|e| format!("{:?}", e.category))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    cats.sort();
    println!("Categories: {}", cats.join(", "));

    // Top 10 by name (alphabetical).
    let mut names: Vec<&str> = all.iter().map(|e| e.name.as_str()).collect();
    names.sort();
    let top10: Vec<&str> = names.into_iter().take(10).collect();
    println!("Top 10 targets:");
    for name in top10 {
        println!("  {}", name);
    }

    Ok(())
}

// ── plugin-list ───────────────────────────────────────────────────────────────

/// Print the default built-in plugin descriptors as formatted text.
pub fn cmd_plugin_list() {
    let plugins = default_builtin_plugins();
    println!("OxiHuman built-in plugins ({} total):", plugins.len());
    println!();
    for p in &plugins {
        let kind_str = match p.kind {
            oxihuman_core::PluginKind::AssetLoader => "AssetLoader",
            oxihuman_core::PluginKind::TargetProvider => "TargetProvider",
            oxihuman_core::PluginKind::Exporter => "Exporter",
            oxihuman_core::PluginKind::Validator => "Validator",
        };
        let exts = if p.supported_extensions.is_empty() {
            String::from("(none)")
        } else {
            p.supported_extensions.join(", ")
        };
        println!("  id:          {}", p.id);
        println!("  name:        {}", p.name);
        println!("  version:     {}", p.version);
        println!("  kind:        {}", kind_str);
        println!("  extensions:  {}", exts);
        println!("  description: {}", p.description);
        println!();
    }
}

// ── camera-info ───────────────────────────────────────────────────────────────

pub fn cmd_camera_info() {
    use oxihuman_viewer::{default_orbit_camera, CameraMode};

    let rig = default_orbit_camera();
    let mode_str = match rig.mode {
        CameraMode::Orbit => "Orbit",
        CameraMode::Fly => "Fly",
        CameraMode::Fixed => "Fixed",
    };
    let pos = rig.orbit_position();
    println!(
        "{}",
        serde_json::json!({
            "mode": mode_str,
            "orbit": {
                "target": rig.orbit.target,
                "distance": rig.orbit.distance,
                "azimuth_rad": rig.orbit.azimuth,
                "elevation_rad": rig.orbit.elevation,
            },
            "position": pos,
            "fov_deg": rig.fov_deg,
            "near": rig.near,
            "far": rig.far,
        })
    );
}
