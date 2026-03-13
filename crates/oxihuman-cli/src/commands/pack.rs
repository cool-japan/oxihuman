// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pack/bundle subcommands: pack-build, zip-pack, quantize, morph-export,
//! asset-bundle, validate-pack, sign-pack, verify-sign.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_core::{
    read_signature_file, sign_pack_dir, verify_pack_signature, write_signature_file,
};
use oxihuman_export::asset_bundle::{bundle_from_dir, export_bundle, AssetBundle as OxbBundle};
use oxihuman_export::pack::{build_pack, PackBuilderConfig};
use oxihuman_export::{
    from_target_files, morph_delta_stats, pack_mesh_assets, quantize_mesh, quantize_stats,
    write_morph_delta_bin, write_quantized_bin,
};
use oxihuman_mesh::MeshBuffers;

// ── pack-build ────────────────────────────────────────────────────────────────

pub fn cmd_pack_build(args: &[String]) -> Result<()> {
    let mut targets_dir: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut max_targets: Option<usize> = None;
    let mut strict = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--targets" => {
                i += 1;
                targets_dir = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--max-targets" => {
                i += 1;
                max_targets = Some(args[i].parse()?);
            }
            "--strict" => {
                strict = true;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let targets_dir = targets_dir.context("--targets is required for pack-build")?;
    if !targets_dir.exists() {
        bail!("targets directory not found: {}", targets_dir.display());
    }

    let policy = if strict {
        Policy::new(PolicyProfile::Strict)
    } else {
        Policy::new(PolicyProfile::Standard)
    };

    eprintln!("OxiHuman: scanning {}...", targets_dir.display());
    let manifest = build_pack(PackBuilderConfig {
        targets_dir,
        policy,
        max_files: max_targets,
    })?;

    eprintln!(
        "OxiHuman: {} files ({} allowed, {} blocked), {} deltas, ~{} KB",
        manifest.stats.total_files,
        manifest.stats.allowed_files,
        manifest.stats.blocked_files,
        manifest.stats.total_deltas,
        manifest.stats.estimated_memory_bytes / 1024,
    );

    if let Some(out) = output {
        manifest.write_to(&out)?;
        eprintln!("OxiHuman: manifest -> {}", out.display());
    } else {
        println!("{}", manifest.to_toml()?);
    }

    Ok(())
}

// ── quantize ──────────────────────────────────────────────────────────────────

pub fn cmd_quantize(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut base: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut stats = false;

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
            "--stats" => {
                stats = true;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for quantize")?;
    let output = output.context("--output is required for quantize")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

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
    let mesh = MeshBuffers::from_morph(morph_buf);

    let qmesh = quantize_mesh(&mesh);

    if stats {
        let qs = quantize_stats(&mesh, &qmesh);
        println!("Quantize stats:");
        println!("  position_error_rms: {:.6}", qs.position_error_rms);
        println!("  normal_error_rms:   {:.6}", qs.normal_error_rms);
        println!("  uv_error_rms:       {:.6}", qs.uv_error_rms);
        println!("  compression_ratio:  {:.3}x", qs.compression_ratio);
    }

    let bytes = write_quantized_bin(&qmesh, &output)
        .with_context(|| format!("writing QMSH to {}", output.display()))?;

    println!(
        "Written QMSH: {} vertices, {} indices → {} bytes",
        qmesh.positions.len(),
        qmesh.indices.len(),
        bytes
    );

    Ok(())
}

// ── morph-export ───────────────────────────────────────────────────────────────

pub fn cmd_morph_export(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;
    use oxihuman_core::parser::target::parse_target;

    let mut base: Option<PathBuf> = None;
    let mut targets_dir: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut max_targets: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets_dir = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--max-targets" => {
                i += 1;
                max_targets = Some(args[i].parse()?);
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for morph-export")?;
    let targets_dir = targets_dir.context("--targets is required for morph-export")?;
    let output = output.context("--output is required for morph-export")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }
    if !targets_dir.exists() {
        bail!("targets directory not found: {}", targets_dir.display());
    }

    let src = std::fs::read_to_string(&base)
        .with_context(|| format!("reading OBJ: {}", base.display()))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let vertex_count = obj.positions.len() as u32;

    // Collect .target files from directory
    let mut entries = std::fs::read_dir(&targets_dir)
        .with_context(|| format!("reading targets dir: {}", targets_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());

    if let Some(max) = max_targets {
        entries.truncate(max);
    }

    let mut target_pairs: Vec<(String, oxihuman_core::parser::target::TargetFile)> = Vec::new();
    for entry in &entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading target: {}", path.display()))?;
        let tf = parse_target(&name, &text)
            .with_context(|| format!("parsing target: {}", path.display()))?;
        target_pairs.push((name, tf));
    }

    let ref_pairs: Vec<(String, &oxihuman_core::parser::target::TargetFile)> =
        target_pairs.iter().map(|(n, t)| (n.clone(), t)).collect();

    let bin = from_target_files(&ref_pairs, vertex_count);
    let stats = morph_delta_stats(&bin);

    write_morph_delta_bin(&bin, &output)
        .with_context(|| format!("writing OXMD to {}", output.display()))?;

    let file_size = std::fs::metadata(&output)?.len() as usize;

    println!(
        "Written OXMD: {} targets, {} total deltas → {} bytes",
        bin.targets.len(),
        stats.total_deltas,
        file_size
    );

    Ok(())
}

// ── zip-pack ───────────────────────────────────────────────────────────────────

pub fn cmd_zip_pack(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut base: Option<PathBuf> = None;
    let mut targets_dir: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets_dir = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for zip-pack")?;
    let targets_dir = targets_dir.context("--targets is required for zip-pack")?;
    let output = output.context("--output is required for zip-pack")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }
    if !targets_dir.exists() {
        bail!("targets directory not found: {}", targets_dir.display());
    }

    // Build GLB from base OBJ
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

    // Apply suit flag and normals so GLB export accepts the mesh
    oxihuman_mesh::normals::compute_normals(&mut mesh);
    oxihuman_mesh::suit::apply_suit_flag(&mut mesh);

    // Export mesh to GLB bytes
    let tmp_glb = output.with_extension("_tmp.glb");
    oxihuman_export::glb::export_glb(&mesh, &tmp_glb)
        .with_context(|| format!("exporting GLB to {}", tmp_glb.display()))?;
    let glb_bytes = std::fs::read(&tmp_glb)?;
    let _ = std::fs::remove_file(&tmp_glb);

    // Build params JSON
    let params_json = serde_json::to_vec(&serde_json::json!({
        "base": base.display().to_string(),
        "targets": targets_dir.display().to_string(),
    }))?;

    // Build manifest JSON (list of .target file names)
    let target_names: Vec<String> = std::fs::read_dir(&targets_dir)
        .with_context(|| format!("reading targets dir: {}", targets_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let manifest_json = serde_json::to_vec(&serde_json::json!({ "targets": target_names }))?;

    let result = pack_mesh_assets(&glb_bytes, &params_json, &manifest_json, &output)
        .with_context(|| format!("writing ZIP to {}", output.display()))?;

    println!(
        "Written ZIP pack: {} entries → {}",
        result.entry_count,
        output.display()
    );

    Ok(())
}

// ── asset-bundle ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_asset_bundle(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut base: Option<PathBuf> = None;
    let mut targets_dir: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut manifest: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets_dir = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--manifest" => {
                i += 1;
                manifest = Some(PathBuf::from(&args[i]));
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for asset-bundle")?;
    let targets_dir = targets_dir.context("--targets is required for asset-bundle")?;
    let output = output.context("--output is required for asset-bundle")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }
    if !targets_dir.exists() {
        bail!("targets directory not found: {}", targets_dir.display());
    }

    // Load the OBJ file bytes.
    let obj_bytes =
        std::fs::read(&base).with_context(|| format!("reading OBJ: {}", base.display()))?;

    // Build a bundle starting with the base OBJ.
    let mut bundle = OxbBundle::new();
    let base_name = base
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("base.obj")
        .to_string();
    bundle
        .add_bytes(&base_name, obj_bytes)
        .with_context(|| format!("adding base OBJ '{}' to bundle", base_name))?;

    // Also scan the OBJ for a quick stat check.
    let obj_src = std::fs::read_to_string(&base)
        .with_context(|| format!("reading OBJ: {}", base.display()))?;
    let _obj = parse_obj(&obj_src).context("parsing OBJ")?;

    // Scan the targets directory and add each .target file.
    let target_bundle = bundle_from_dir(&targets_dir)
        .with_context(|| format!("scanning targets dir: {}", targets_dir.display()))?;
    let target_count = target_bundle.entry_count();
    for name in target_bundle.entry_names() {
        if let Some(entry) = target_bundle.get(name) {
            let entry_data = entry.data.clone();
            // Avoid name collisions with base OBJ by prefixing with "targets/".
            let bundle_name = format!("targets/{}", name);
            bundle.add_bytes(bundle_name, entry_data).ok(); // skip duplicates silently
        }
    }

    // Optionally include manifest bytes.
    if let Some(ref mp) = manifest {
        if !mp.exists() {
            bail!("manifest file not found: {}", mp.display());
        }
        let manifest_bytes =
            std::fs::read(mp).with_context(|| format!("reading manifest: {}", mp.display()))?;
        bundle.add_bytes("manifest.toml", manifest_bytes).ok();
    }

    let total_assets = bundle.entry_count();
    export_bundle(&bundle, &output)
        .with_context(|| format!("writing bundle: {}", output.display()))?;

    println!(
        "Written bundle: {} assets → {}",
        total_assets,
        output.display()
    );
    let _ = target_count; // suppress unused warning if targets dir was empty
    Ok(())
}

// ── validate-pack ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_validate_pack(args: &[String]) -> Result<()> {
    use oxihuman_core::{scan_pack, verify_manifest_present, verify_pack};

    let mut pack_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pack-dir" => {
                i += 1;
                pack_dir = Some(PathBuf::from(&args[i]));
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let pack_dir = pack_dir.context("--pack-dir is required for validate-pack")?;

    if !pack_dir.exists() {
        bail!("pack directory not found: {}", pack_dir.display());
    }

    // Check for manifest file.
    match verify_manifest_present(&pack_dir) {
        Ok(()) => println!("Manifest: present"),
        Err(e) => println!("Manifest: MISSING — {}", e),
    }

    // Scan and verify all files.
    let records =
        scan_pack(&pack_dir).with_context(|| format!("scanning pack: {}", pack_dir.display()))?;
    println!("Scanned {} file(s)", records.len());

    let report = verify_pack(&pack_dir, &records);
    println!("{}", report.summary());

    if !report.is_valid {
        if !report.failed_files.is_empty() {
            println!("Failed files:");
            for f in &report.failed_files {
                println!("  FAIL: {}", f);
            }
        }
        if !report.missing_files.is_empty() {
            println!("Missing files:");
            for f in &report.missing_files {
                println!("  MISS: {}", f);
            }
        }
        bail!("pack validation failed");
    }
    Ok(())
}

// ── sign-pack ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_sign_pack(args: &[String]) -> Result<()> {
    let mut pack_dir: Option<PathBuf> = None;
    let mut key_str: Option<String> = None;
    let mut signer_id = String::from("oxihuman-cli");
    let mut output: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pack-dir" => {
                i += 1;
                pack_dir = Some(PathBuf::from(&args[i]));
            }
            "--key" => {
                i += 1;
                key_str = Some(args[i].clone());
            }
            "--signer-id" => {
                i += 1;
                signer_id = args[i].clone();
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            other => bail!("sign-pack: unknown option: {}", other),
        }
        i += 1;
    }

    let pack_dir = pack_dir.context("--pack-dir is required for sign-pack")?;
    let key_str = key_str.context("--key is required for sign-pack")?;
    let output = output.context("--output is required for sign-pack")?;

    if !pack_dir.exists() {
        bail!("pack-dir not found: {}", pack_dir.display());
    }

    let signed = sign_pack_dir(&pack_dir, key_str.as_bytes(), &signer_id)
        .with_context(|| format!("signing pack dir: {}", pack_dir.display()))?;
    write_signature_file(&signed, &output)
        .with_context(|| format!("writing signature file: {}", output.display()))?;
    println!("Pack signed. Signature written to: {}", output.display());
    Ok(())
}

// ── verify-sign ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_verify_sign(args: &[String]) -> Result<()> {
    let mut pack_dir: Option<PathBuf> = None;
    let mut sig_file: Option<PathBuf> = None;
    let mut key_str: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pack-dir" => {
                i += 1;
                pack_dir = Some(PathBuf::from(&args[i]));
            }
            "--sig-file" => {
                i += 1;
                sig_file = Some(PathBuf::from(&args[i]));
            }
            "--key" => {
                i += 1;
                key_str = Some(args[i].clone());
            }
            other => bail!("verify-sign: unknown option: {}", other),
        }
        i += 1;
    }

    let pack_dir = pack_dir.context("--pack-dir is required for verify-sign")?;
    let sig_file = sig_file.context("--sig-file is required for verify-sign")?;
    let key_str = key_str.context("--key is required for verify-sign")?;

    if !pack_dir.exists() {
        bail!("pack-dir not found: {}", pack_dir.display());
    }
    if !sig_file.exists() {
        bail!("sig-file not found: {}", sig_file.display());
    }

    let signed = read_signature_file(&sig_file)
        .with_context(|| format!("reading sig file: {}", sig_file.display()))?;
    if verify_pack_signature(&pack_dir, &signed, key_str.as_bytes()) {
        println!("VALID");
    } else {
        println!("INVALID");
    }
    Ok(())
}

// ── pack-dist-manifest ────────────────────────────────────────────────────────

/// Generate a SHA-256 distribution manifest for all files in a pack directory.
///
/// Usage: `oxihuman pack-dist-manifest --pack-dir <DIR>`
///
/// Prints the manifest JSON to stdout.
#[allow(dead_code)]
pub fn cmd_pack_dist_manifest(args: &[String]) -> Result<()> {
    let mut pack_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pack-dir" => {
                i += 1;
                if i >= args.len() {
                    bail!("pack-dist-manifest: --pack-dir requires an argument");
                }
                pack_dir = Some(PathBuf::from(&args[i]));
            }
            other => bail!("pack-dist-manifest: unknown option: {}", other),
        }
        i += 1;
    }

    let pack_dir = pack_dir.context("--pack-dir is required for pack-dist-manifest")?;
    if !pack_dir.exists() {
        bail!("pack-dir not found: {}", pack_dir.display());
    }

    let manifest = oxihuman_core::asset_pack_builder::generate_distribution_manifest(&pack_dir)
        .with_context(|| {
            format!(
                "generating distribution manifest for: {}",
                pack_dir.display()
            )
        })?;
    println!("{manifest}");
    Ok(())
}

// ── pack-verify-dist ──────────────────────────────────────────────────────────

/// Verify all files in a pack directory against a distribution manifest.
///
/// Usage: `oxihuman pack-verify-dist --manifest <FILE> --pack-dir <DIR>`
///
/// Exits with code 0 on success, 1 on verification failure.
#[allow(dead_code)]
pub fn cmd_pack_verify_dist(args: &[String]) -> Result<()> {
    let mut manifest_path: Option<PathBuf> = None;
    let mut pack_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" => {
                i += 1;
                if i >= args.len() {
                    bail!("pack-verify-dist: --manifest requires an argument");
                }
                manifest_path = Some(PathBuf::from(&args[i]));
            }
            "--pack-dir" => {
                i += 1;
                if i >= args.len() {
                    bail!("pack-verify-dist: --pack-dir requires an argument");
                }
                pack_dir = Some(PathBuf::from(&args[i]));
            }
            other => bail!("pack-verify-dist: unknown option: {}", other),
        }
        i += 1;
    }

    let manifest_path = manifest_path.context("--manifest is required for pack-verify-dist")?;
    let pack_dir = pack_dir.context("--pack-dir is required for pack-verify-dist")?;

    if !manifest_path.exists() {
        bail!("manifest file not found: {}", manifest_path.display());
    }
    if !pack_dir.exists() {
        bail!("pack-dir not found: {}", pack_dir.display());
    }

    let json = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading manifest: {}", manifest_path.display()))?;

    let ok = oxihuman_core::asset_pack_builder::verify_distribution_manifest(&json, &pack_dir)
        .with_context(|| {
            format!(
                "verifying distribution manifest against: {}",
                pack_dir.display()
            )
        })?;

    if ok {
        println!("Manifest verification: OK");
    } else {
        eprintln!("Manifest verification: FAILED");
        std::process::exit(1);
    }
    Ok(())
}
