// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Animation subcommands: pc2, mdd, anim-bake, stream-export.
//!
//! `pc2` and `mdd` bake a mesh vertex-cache from a base OBJ. Passing
//! `--anim <json>` (and a `--targets <dir>` of `.target` morph deltas) makes
//! every output frame the result of evaluating time-varying morph parameters
//! at `t = start_time + frame_index / fps` (see [`crate::commands::anim_params`]);
//! without `--anim` all frames use the mesh's default (constant) parameters,
//! which is the correct — not stubbed — result of an unanimated input.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_export::{
    mesh_sequence_to_pc2, stream_mesh_positions, uniform_time_mdd, write_mdd, write_pc2,
    StreamFormat, StreamingExportConfig,
};
use oxihuman_morph::HumanEngine;

use super::anim_params::{load_anim_source, AnimSource};

// ── shared frame evaluation ─────────────────────────────────────────────────

/// Decide how many output frames to bake.
///
/// A dense (snapshot) anim source fixes the frame count outright. A curve
/// source uses an explicit `--frames` if the caller passed one, otherwise it
/// derives a sensible count from the curve's own duration and `fps`. With no
/// anim source at all, the plain `--frames` flag (or its default) applies.
fn resolve_frame_count(
    anim_source: Option<&AnimSource>,
    frames_flag: usize,
    frames_explicit: bool,
    fps: f32,
) -> usize {
    match anim_source {
        Some(src) => match src.fixed_frame_count() {
            Some(fixed) => fixed,
            None if frames_explicit => frames_flag,
            None => src.suggested_frame_count(fps),
        },
        None => frames_flag,
    }
}

/// Evaluate `frame_count` real mesh-position frames from a base OBJ.
///
/// When `targets_dir` is given, `.target` morph deltas are loaded into a
/// [`HumanEngine`] (auto-mapped to parameter names by filename, e.g.
/// `height.target` responds to the `height` param). For each output frame
/// `i`, if an `anim_source` is present its parameters at frame `i` are
/// applied via `engine.set_params` *before* `engine.build_mesh()` is called,
/// so frames genuinely differ whenever the sampled parameters differ and a
/// matching morph target is loaded. This replaces the former stub that
/// cloned one static frame `frame_count` times regardless of input.
fn build_frame_positions(
    input: &Path,
    targets_dir: Option<&Path>,
    anim_source: Option<&AnimSource>,
    frame_count: usize,
    fps: f32,
    start_time: f32,
) -> Result<Vec<Vec<[f32; 3]>>> {
    use oxihuman_core::parser::obj::parse_obj;

    if frame_count == 0 {
        bail!("frame count must be at least 1");
    }

    let src = std::fs::read_to_string(input)
        .with_context(|| format!("reading OBJ: {}", input.display()))?;
    let base = parse_obj(&src).context("parsing OBJ")?;

    let policy = Policy::new(PolicyProfile::Standard);
    let mut engine = HumanEngine::new(base, policy);

    if let Some(dir) = targets_dir {
        let loaded = engine
            .load_targets_from_dir_auto(dir)
            .with_context(|| format!("loading morph targets from {}", dir.display()))?;
        if loaded == 0 {
            bail!(
                "no .target files found in {} (nothing to animate)",
                dir.display()
            );
        }
    }

    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        if let Some(source) = anim_source {
            engine.set_params(source.params_at_frame(i, fps, start_time));
        }
        frames.push(engine.build_mesh().positions);
    }
    Ok(frames)
}

// ── pc2 ───────────────────────────────────────────────────────────────────────

/// Bake a PC2 (Point Cache 2) vertex-cache from a base OBJ mesh.
///
/// Options:
///   --input `<obj>`        base mesh (required)
///   --output `<pc2>`       output path (required)
///   --targets `<dir>`      directory of `.target` morph deltas (optional)
///   --anim `<json>`        keyframed/snapshot params JSON — see
///                        [`crate::commands::anim_params`] for the accepted shapes
///                        (optional; requires `--targets` to have any visual
///                        effect on the output)
///   --frames `<n>`         frame count when no anim source fixes it (default 10)
///   --fps `<f32>`          sample rate stored in the header (default 24.0)
///   --start-time `<f32>`   start time stored in the header (default 0.0)
#[allow(dead_code)]
pub fn cmd_pc2(args: &[String]) -> Result<()> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut anim_path: Option<PathBuf> = None;
    let mut frames: usize = 10;
    let mut frames_explicit = false;
    let mut fps: f32 = 24.0;
    let mut start_time: f32 = 0.0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--anim" => {
                i += 1;
                anim_path = Some(PathBuf::from(&args[i]));
            }
            "--frames" => {
                i += 1;
                frames = args[i].parse().context("--frames must be a number")?;
                frames_explicit = true;
            }
            "--fps" => {
                i += 1;
                fps = args[i].parse().context("--fps must be a number")?;
            }
            "--start-time" => {
                i += 1;
                start_time = args[i].parse().context("--start-time must be a number")?;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let input = input.context("--input is required for pc2")?;
    let output = output.context("--output is required for pc2")?;

    if !input.exists() {
        bail!("input mesh not found: {}", input.display());
    }
    if let Some(ref dir) = targets {
        if !dir.exists() {
            bail!("targets directory not found: {}", dir.display());
        }
    }

    let anim_source = match &anim_path {
        Some(p) => {
            if !p.exists() {
                bail!("anim JSON not found: {}", p.display());
            }
            Some(load_anim_source(p).context("parsing --anim JSON")?)
        }
        None => None,
    };

    let frame_count = resolve_frame_count(anim_source.as_ref(), frames, frames_explicit, fps);
    let frame_data = build_frame_positions(
        &input,
        targets.as_deref(),
        anim_source.as_ref(),
        frame_count,
        fps,
        start_time,
    )?;

    let cache = mesh_sequence_to_pc2(&frame_data, start_time, fps);
    let bytes = write_pc2(&cache);
    std::fs::write(&output, &bytes)
        .with_context(|| format!("writing PC2: {}", output.display()))?;

    println!(
        "Written PC2: {} points × {} frames → {}",
        cache.header.point_count,
        cache.header.sample_count,
        output.display()
    );
    Ok(())
}

// ── mdd ───────────────────────────────────────────────────────────────────────

/// Bake an MDD (Motion Displacement Data) vertex-cache from a base OBJ mesh.
///
/// Options:
///   --input `<obj>`        base mesh (required)
///   --output `<mdd>`       output path (required)
///   --targets `<dir>`      directory of `.target` morph deltas (optional)
///   --anim `<json>`        keyframed/snapshot params JSON — see
///                        [`crate::commands::anim_params`] (optional; requires
///                        `--targets` to have any visual effect)
///   --frames `<n>`         frame count when no anim source fixes it (default 10)
///   --fps `<f32>`          playback rate; also drives per-frame timestamps
///                        (default 24.0)
#[allow(dead_code)]
pub fn cmd_mdd(args: &[String]) -> Result<()> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut anim_path: Option<PathBuf> = None;
    let mut frames: usize = 10;
    let mut frames_explicit = false;
    let mut fps: f32 = 24.0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--anim" => {
                i += 1;
                anim_path = Some(PathBuf::from(&args[i]));
            }
            "--frames" => {
                i += 1;
                frames = args[i].parse().context("--frames must be a number")?;
                frames_explicit = true;
            }
            "--fps" => {
                i += 1;
                fps = args[i].parse().context("--fps must be a number")?;
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let input = input.context("--input is required for mdd")?;
    let output = output.context("--output is required for mdd")?;

    if !input.exists() {
        bail!("input mesh not found: {}", input.display());
    }
    if let Some(ref dir) = targets {
        if !dir.exists() {
            bail!("targets directory not found: {}", dir.display());
        }
    }

    let anim_source = match &anim_path {
        Some(p) => {
            if !p.exists() {
                bail!("anim JSON not found: {}", p.display());
            }
            Some(load_anim_source(p).context("parsing --anim JSON")?)
        }
        None => None,
    };

    let frame_count = resolve_frame_count(anim_source.as_ref(), frames, frames_explicit, fps);
    let frame_data = build_frame_positions(
        &input,
        targets.as_deref(),
        anim_source.as_ref(),
        frame_count,
        fps,
        0.0,
    )?;

    let cache = uniform_time_mdd(&frame_data, fps);
    let bytes = write_mdd(&cache);
    std::fs::write(&output, &bytes)
        .with_context(|| format!("writing MDD: {}", output.display()))?;

    println!(
        "Written MDD: {} points × {} frames → {}",
        cache.point_count,
        cache.frames.len(),
        output.display()
    );
    Ok(())
}

// ── anim-bake ─────────────────────────────────────────────────────────────────

/// Bake an animation cache from a params JSON source to PC2 or MDD format.
///
/// `--params-json` accepts either shape documented in
/// [`crate::commands::anim_params`]: a dense array of per-frame parameter
/// snapshots (frame count = array length), or a `{"keyframes": [...]}`
/// object describing continuous animation curves sampled at `t = i / fps`.
/// Pass `--targets <dir>` (a directory of `.target` morph deltas) so the
/// sampled parameters actually displace vertices; without it every frame
/// correctly reduces to the base mesh, since there is nothing to morph.
#[allow(dead_code)]
pub fn cmd_anim_bake(args: &[String]) -> Result<()> {
    let mut input: Option<PathBuf> = None;
    let mut params_json: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut format = String::from("pc2");
    let mut fps: f32 = 30.0;
    let mut frames: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = Some(PathBuf::from(&args[i]));
            }
            "--params-json" => {
                i += 1;
                params_json = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--format" => {
                i += 1;
                format = args[i].clone();
            }
            "--fps" => {
                i += 1;
                fps = args[i].parse().context("--fps must be a number")?;
            }
            "--frames" => {
                i += 1;
                frames = Some(args[i].parse().context("--frames must be a number")?);
            }
            other => bail!("anim-bake: unknown option: {}", other),
        }
        i += 1;
    }

    let input = input.context("--input is required for anim-bake")?;
    let params_json = params_json.context("--params-json is required for anim-bake")?;
    let output = output.context("--output is required for anim-bake")?;

    if !input.exists() {
        bail!("anim-bake: input mesh not found: {}", input.display());
    }
    if !params_json.exists() {
        bail!(
            "anim-bake: params-json file not found: {}",
            params_json.display()
        );
    }
    if let Some(ref dir) = targets {
        if !dir.exists() {
            bail!("anim-bake: targets directory not found: {}", dir.display());
        }
    }

    let anim_source = load_anim_source(&params_json).context("anim-bake: parsing params-json")?;
    let frame_count = match anim_source.fixed_frame_count() {
        Some(fixed) => fixed,
        None => frames.unwrap_or_else(|| anim_source.suggested_frame_count(fps)),
    };

    let frame_data = build_frame_positions(
        &input,
        targets.as_deref(),
        Some(&anim_source),
        frame_count,
        fps,
        0.0,
    )?;

    match format.as_str() {
        "pc2" => {
            let cache = mesh_sequence_to_pc2(&frame_data, 0.0, fps);
            let bytes = write_pc2(&cache);
            std::fs::write(&output, &bytes)
                .with_context(|| format!("writing PC2: {}", output.display()))?;
            println!(
                "anim-bake: written PC2 {} frames × {} points → {}",
                cache.header.sample_count,
                cache.header.point_count,
                output.display()
            );
        }
        "mdd" => {
            let cache = uniform_time_mdd(&frame_data, fps);
            let bytes = write_mdd(&cache);
            std::fs::write(&output, &bytes)
                .with_context(|| format!("writing MDD: {}", output.display()))?;
            println!(
                "anim-bake: written MDD {} frames × {} points → {}",
                cache.frames.len(),
                cache.point_count,
                output.display()
            );
        }
        other => bail!("anim-bake: unknown format '{}'. Use 'pc2' or 'mdd'", other),
    }

    Ok(())
}

// ── stream-export ─────────────────────────────────────────────────────────────

/// Stream-export mesh vertex positions in encoded chunks.
#[allow(dead_code)]
pub fn cmd_stream_export(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut format_str = String::from("f32");
    let mut chunk_size: usize = 4096;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--format" => {
                i += 1;
                format_str = args[i].clone();
            }
            "--chunk-size" => {
                i += 1;
                chunk_size = args[i].parse().context("--chunk-size must be a number")?;
            }
            other => bail!("stream-export: unknown option: {}", other),
        }
        i += 1;
    }

    let input = input.context("--input is required for stream-export")?;
    let output = output.context("--output is required for stream-export")?;

    if !input.exists() {
        bail!("stream-export: input mesh not found: {}", input.display());
    }

    let stream_format = match format_str.as_str() {
        "f32" => StreamFormat::BinaryFloat32,
        "f16" => StreamFormat::BinaryFloat16,
        "csv" => StreamFormat::AsciiCsv,
        other => bail!(
            "stream-export: unknown format '{}'. Use 'f32', 'f16', or 'csv'",
            other
        ),
    };

    let src = std::fs::read_to_string(&input)
        .with_context(|| format!("reading OBJ: {}", input.display()))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let positions: Vec<[f32; 3]> = obj.positions.iter().map(|p| [p[0], p[1], p[2]]).collect();

    let cfg = StreamingExportConfig {
        chunk_size,
        format: stream_format,
        compress: false,
    };

    let chunks = stream_mesh_positions(&positions, &cfg);
    let total_bytes: usize = chunks.iter().map(|c| c.data.len()).sum();
    let mut all_bytes: Vec<u8> = Vec::with_capacity(total_bytes);
    for chunk in &chunks {
        all_bytes.extend_from_slice(&chunk.data);
    }

    std::fs::write(&output, &all_bytes)
        .with_context(|| format!("writing stream-export output: {}", output.display()))?;

    println!(
        "stream-export: {} vertices in {} chunks ({} bytes) → {}",
        positions.len(),
        chunks.len(),
        total_bytes,
        output.display()
    );
    Ok(())
}
