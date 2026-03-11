// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export subcommands: stl, collada, gltf-sep, svg, lod-export, variant-pack, report.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_export::collada::{export_collada, ColladaExportOptions};
use oxihuman_export::gltf_sep::export_gltf_sep;
use oxihuman_export::lod_export::export_default_lod_pack;
use oxihuman_export::params_json::import_params;
use oxihuman_export::report_html::{
    export_html_report, mesh_report_from_buffers, PipelineReportData,
};
use oxihuman_export::stl::{export_stl_ascii, export_stl_binary};
use oxihuman_export::svg::{export_svg, export_uv_svg, SvgExportOptions, SvgProjection};
use oxihuman_export::variant_pack::{variant_entry, write_variant_pack};
use oxihuman_mesh::MeshBuffers;
use oxihuman_morph::params::ParamState;
use oxihuman_morph::presets::BodyPreset;

use crate::utils::{build_mesh_from_base, load_params};

// ── stl ───────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn cmd_stl(args: &[String]) -> Result<()> {
    let mut base: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut params_src: Option<String> = None;
    let mut preset_name: Option<String> = None;
    let mut binary = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--params" => {
                i += 1;
                params_src = Some(args[i].clone());
            }
            "--preset" => {
                i += 1;
                preset_name = Some(args[i].clone());
            }
            "--binary" => {
                binary = true;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for stl")?;
    let output = output.context("--output is required for stl")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    let params = if let Some(name) = &preset_name {
        BodyPreset::from_name(name)
            .ok_or_else(|| anyhow::anyhow!("unknown preset '{}'", name))?
            .params()
    } else if let Some(src) = &params_src {
        load_params(src)?
    } else {
        ParamState::default()
    };

    let policy = Policy::new(PolicyProfile::Standard);
    let mesh = build_mesh_from_base(&base, targets.as_deref(), params, policy)?;

    let tri_count = mesh.indices.len() / 3;
    if binary {
        export_stl_binary(&mesh, &output)
            .with_context(|| format!("writing binary STL to {}", output.display()))?;
        println!(
            "Written STL (binary): {} triangles → {}",
            tri_count,
            output.display()
        );
    } else {
        let stem = output
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        export_stl_ascii(&mesh, &output, stem)
            .with_context(|| format!("writing ASCII STL to {}", output.display()))?;
        println!(
            "Written STL (ascii): {} triangles → {}",
            tri_count,
            output.display()
        );
    }

    Ok(())
}

// ── collada ───────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn cmd_collada(args: &[String]) -> Result<()> {
    let mut base: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut params_src: Option<String> = None;
    let mut preset_name: Option<String> = None;
    let mut author: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--params" => {
                i += 1;
                params_src = Some(args[i].clone());
            }
            "--preset" => {
                i += 1;
                preset_name = Some(args[i].clone());
            }
            "--author" => {
                i += 1;
                author = Some(args[i].clone());
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for collada")?;
    let output = output.context("--output is required for collada")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    let params = if let Some(name) = &preset_name {
        BodyPreset::from_name(name)
            .ok_or_else(|| anyhow::anyhow!("unknown preset '{}'", name))?
            .params()
    } else if let Some(src) = &params_src {
        load_params(src)?
    } else {
        ParamState::default()
    };

    let policy = Policy::new(PolicyProfile::Standard);
    let mesh = build_mesh_from_base(&base, targets.as_deref(), params, policy)?;

    let mut opts = ColladaExportOptions::default();
    if let Some(a) = author {
        opts.author = a;
    }

    let stats = export_collada(&mesh, &output, &opts)
        .with_context(|| format!("writing COLLADA to {}", output.display()))?;

    println!(
        "Written COLLADA: {} vertices, {} triangles → {}",
        stats.vertex_count,
        stats.face_count,
        output.display()
    );

    Ok(())
}

// ── gltf-sep ──────────────────────────────────────────────────────────────────

pub fn cmd_gltf_sep(args: &[String]) -> Result<()> {
    let mut base: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut bin_path: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut params_src: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--bin" => {
                i += 1;
                bin_path = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--params" => {
                i += 1;
                params_src = Some(args[i].clone());
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for gltf-sep")?;
    let output = output.context("--output is required for gltf-sep")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    let params = if let Some(src) = &params_src {
        load_params(src)?
    } else {
        ParamState::default()
    };

    let policy = Policy::new(PolicyProfile::Standard);
    let mesh = build_mesh_from_base(&base, targets.as_deref(), params, policy)?;

    // Derive .bin path from .gltf stem if not provided
    let bin = bin_path.unwrap_or_else(|| output.with_extension("bin"));

    export_gltf_sep(&mesh, &output, &bin)
        .with_context(|| format!("writing glTF+BIN to {}", output.display()))?;

    println!("Written glTF+BIN: {} + {}", output.display(), bin.display());

    Ok(())
}

// ── svg ───────────────────────────────────────────────────────────────────────

pub fn cmd_svg(args: &[String]) -> Result<()> {
    let mut base: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut projection_str: Option<String> = None;
    let mut uv_mode = false;
    let mut width: u32 = 800;
    let mut height: u32 = 600;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--projection" => {
                i += 1;
                projection_str = Some(args[i].clone());
            }
            "--uv" => {
                uv_mode = true;
            }
            "--width" => {
                i += 1;
                width = args[i].parse().context("--width must be an integer")?;
            }
            "--height" => {
                i += 1;
                height = args[i].parse().context("--height must be an integer")?;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for svg")?;
    let output = output.context("--output is required for svg")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    let policy = Policy::new(PolicyProfile::Standard);
    let mesh = build_mesh_from_base(&base, None, ParamState::default(), policy)?;

    if uv_mode {
        export_uv_svg(&mesh, &output)
            .with_context(|| format!("writing UV SVG to {}", output.display()))?;
    } else {
        let projection = match projection_str.as_deref().unwrap_or("front") {
            "front" => SvgProjection::Front,
            "side" => SvgProjection::Side,
            "top" => SvgProjection::Top,
            other => bail!("unknown projection '{}'; choose front, side, or top", other),
        };
        let opts = SvgExportOptions {
            projection,
            width,
            height,
            ..Default::default()
        };
        export_svg(&mesh, &output, &opts)
            .with_context(|| format!("writing SVG to {}", output.display()))?;
    }

    println!("Written SVG: {}", output.display());

    Ok(())
}

// ── lod-export ────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn cmd_lod_export(args: &[String]) -> Result<()> {
    let mut base: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut params_src: Option<String> = None;
    let mut preset_name: Option<String> = None;
    let mut _levels: usize = 3;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--output-dir" => {
                i += 1;
                output_dir = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--params" => {
                i += 1;
                params_src = Some(args[i].clone());
            }
            "--preset" => {
                i += 1;
                preset_name = Some(args[i].clone());
            }
            "--levels" => {
                i += 1;
                _levels = args[i].parse().context("--levels must be an integer")?;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for lod-export")?;
    let output_dir = output_dir.context("--output-dir is required for lod-export")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    let params = if let Some(name) = &preset_name {
        BodyPreset::from_name(name)
            .ok_or_else(|| anyhow::anyhow!("unknown preset '{}'", name))?
            .params()
    } else if let Some(src) = &params_src {
        load_params(src)?
    } else {
        ParamState::default()
    };

    let policy = Policy::new(PolicyProfile::Standard);
    let mesh = build_mesh_from_base(&base, targets.as_deref(), params, policy)?;

    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    let paths = export_default_lod_pack(&mesh, stem, &output_dir)
        .with_context(|| format!("writing LOD pack to {}", output_dir.display()))?;

    println!(
        "Written LOD pack: {} levels → {}",
        paths.len(),
        output_dir.display()
    );

    Ok(())
}

// ── variant-pack ──────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn cmd_variant_pack(args: &[String]) -> Result<()> {
    let mut params_list_path: Option<PathBuf> = None;
    let mut base: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut pack_name = "oxihuman_variants".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--params-list" => {
                i += 1;
                params_list_path = Some(PathBuf::from(&args[i]));
            }
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--output-dir" => {
                i += 1;
                output_dir = Some(PathBuf::from(&args[i]));
            }
            "--pack-name" => {
                i += 1;
                pack_name = args[i].clone();
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let params_list_path =
        params_list_path.context("--params-list is required for variant-pack")?;
    let base = base.context("--base is required for variant-pack")?;
    let output_dir = output_dir.context("--output-dir is required for variant-pack")?;

    if !params_list_path.exists() {
        bail!("params-list file not found: {}", params_list_path.display());
    }
    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    // Parse the JSON array of param sets
    let list_src = std::fs::read_to_string(&params_list_path)
        .with_context(|| format!("reading params-list: {}", params_list_path.display()))?;
    let param_values: Vec<serde_json::Value> =
        serde_json::from_str(&list_src).context("params-list must be a JSON array")?;

    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("creating output dir: {}", output_dir.display()))?;

    let policy = Policy::new(PolicyProfile::Standard);

    // Build each variant
    let mut entries_and_meshes: Vec<(oxihuman_export::variant_pack::VariantEntry, MeshBuffers)> =
        Vec::new();

    for (idx, val) in param_values.iter().enumerate() {
        let params = import_params(val)?;

        // Build a params HashMap for the VariantEntry
        let mut params_map = std::collections::HashMap::new();
        params_map.insert("height".to_string(), params.height);
        params_map.insert("weight".to_string(), params.weight);
        params_map.insert("muscle".to_string(), params.muscle);
        params_map.insert("age".to_string(), params.age);

        let id = format!("variant_{:03}", idx);
        let glb_filename = format!("{}.glb", id);
        let entry = variant_entry(&id, &format!("Variant {}", idx), &glb_filename, params_map);

        let mesh = build_mesh_from_base(&base, targets.as_deref(), params, policy.clone())?;
        entries_and_meshes.push((entry, mesh));
    }

    // Build slices for write_variant_pack
    let pairs: Vec<(oxihuman_export::variant_pack::VariantEntry, &MeshBuffers)> =
        entries_and_meshes
            .iter()
            .map(|(e, m)| (e.clone(), m))
            .collect();

    let result = write_variant_pack(&pairs, &output_dir, &pack_name)
        .with_context(|| format!("writing variant pack to {}", output_dir.display()))?;

    println!(
        "Written variant pack: {} variants → {}",
        result.glb_paths.len(),
        output_dir.display()
    );

    Ok(())
}

// ── report ────────────────────────────────────────────────────────────────────

pub fn cmd_report(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut base: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut targets_dir: Option<PathBuf> = None;
    let mut pack: Option<PathBuf> = None;
    let mut title = String::from("OxiHuman Report");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets_dir = Some(PathBuf::from(&args[i]));
            }
            "--pack" => {
                i += 1;
                pack = Some(PathBuf::from(&args[i]));
            }
            "--title" => {
                i += 1;
                title = args[i].clone();
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for report")?;
    let output = output.context("--output is required for report")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    // Parse the OBJ and compute mesh stats.
    let src = std::fs::read_to_string(&base)
        .with_context(|| format!("reading OBJ: {}", base.display()))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let morph_buf = oxihuman_morph::engine::MeshBuffers {
        positions: obj.positions,
        normals: obj.normals,
        uvs: obj.uvs,
        indices: obj.indices,
        has_suit: false,
    };
    let mut mesh = MeshBuffers::from_morph(morph_buf);
    oxihuman_mesh::normals::compute_normals(&mut mesh);
    oxihuman_mesh::suit::apply_suit_flag(&mut mesh);

    let base_name = base
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("base.obj")
        .to_string();
    let file_size = std::fs::metadata(&base).map(|m| m.len()).ok();
    let mesh_data = mesh_report_from_buffers(&mesh, &base_name, "obj");

    let mut report = PipelineReportData::new(&title);
    report.add_mesh({
        use oxihuman_export::report_html::MeshReportData;
        MeshReportData {
            name: mesh_data.name,
            vertex_count: mesh_data.vertex_count,
            face_count: mesh_data.face_count,
            has_normals: mesh_data.has_normals,
            has_uvs: mesh_data.has_uvs,
            has_colors: mesh_data.has_colors,
            bounding_box_min: mesh_data.bounding_box_min,
            bounding_box_max: mesh_data.bounding_box_max,
            file_size_bytes: file_size,
            format: mesh_data.format,
        }
    });

    // Count targets if directory provided.
    if let Some(ref td) = targets_dir {
        if !td.exists() {
            bail!("targets directory not found: {}", td.display());
        }
        let count = std::fs::read_dir(td)
            .with_context(|| format!("reading targets dir: {}", td.display()))?
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
            .count();
        report.add_param("target_count", count as f32);
    }

    // Load manifest if provided.
    if let Some(ref mp) = pack {
        if !mp.exists() {
            bail!("manifest file not found: {}", mp.display());
        }
        report.add_export_path(mp.to_string_lossy().into_owned());
    }

    export_html_report(&report, &output)
        .with_context(|| format!("writing HTML report: {}", output.display()))?;

    println!("Written report: {}", output.display());
    Ok(())
}
