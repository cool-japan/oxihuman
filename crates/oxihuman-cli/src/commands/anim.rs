// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Animation subcommands: pc2, mdd, anim-bake, stream-export.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use oxihuman_export::{
    mesh_sequence_to_pc2, stream_mesh_positions, uniform_time_mdd, write_mdd, write_pc2,
    StreamFormat, StreamingExportConfig,
};

// ── pc2 ───────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cmd_pc2(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut frames: usize = 10;
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
            "--frames" => {
                i += 1;
                frames = args[i].parse().context("--frames must be a number")?;
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

    let src = std::fs::read_to_string(&input)
        .with_context(|| format!("reading OBJ: {}", input.display()))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let positions: Vec<[f32; 3]> = obj.positions.iter().map(|p| [p[0], p[1], p[2]]).collect();

    // Stub animation: all frames are identical (base pose).
    let frame_data: Vec<Vec<[f32; 3]>> = (0..frames).map(|_| positions.clone()).collect();
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

#[allow(dead_code)]
pub fn cmd_mdd(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut frames: usize = 10;
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
            "--frames" => {
                i += 1;
                frames = args[i].parse().context("--frames must be a number")?;
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

    let src = std::fs::read_to_string(&input)
        .with_context(|| format!("reading OBJ: {}", input.display()))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let positions: Vec<[f32; 3]> = obj.positions.iter().map(|p| [p[0], p[1], p[2]]).collect();

    // Stub animation: all frames are identical (base pose).
    let frame_data: Vec<Vec<[f32; 3]>> = (0..frames).map(|_| positions.clone()).collect();
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

/// Bake an animation cache from a params JSON array to PC2 or MDD format.
///
/// For each entry in the params JSON array the stub simply reuses the base mesh
/// positions (no morphing), then writes the resulting frame sequence to the
/// chosen animation cache format.
#[allow(dead_code)]
pub fn cmd_anim_bake(args: &[String]) -> Result<()> {
    use oxihuman_core::parser::obj::parse_obj;

    let mut input: Option<PathBuf> = None;
    let mut params_json: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut format = String::from("pc2");
    let mut fps: f32 = 30.0;

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
            "--format" => {
                i += 1;
                format = args[i].clone();
            }
            "--fps" => {
                i += 1;
                fps = args[i].parse().context("--fps must be a number")?;
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

    // Parse JSON array of param objects -> frame count = array length.
    let json_src = std::fs::read_to_string(&params_json)
        .with_context(|| format!("reading params-json: {}", params_json.display()))?;
    let param_array: serde_json::Value =
        serde_json::from_str(&json_src).context("parsing params-json")?;
    let frames = match &param_array {
        serde_json::Value::Array(arr) => arr.len(),
        _ => bail!("anim-bake: params-json must be a JSON array"),
    };
    if frames == 0 {
        bail!("anim-bake: params-json array is empty");
    }

    // Load base mesh.
    let src = std::fs::read_to_string(&input)
        .with_context(|| format!("reading OBJ: {}", input.display()))?;
    let obj = parse_obj(&src).context("parsing OBJ")?;
    let positions: Vec<[f32; 3]> = obj.positions.iter().map(|p| [p[0], p[1], p[2]]).collect();

    // Stub: every frame uses the base mesh positions unchanged.
    let frame_data: Vec<Vec<[f32; 3]>> = (0..frames).map(|_| positions.clone()).collect();

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
