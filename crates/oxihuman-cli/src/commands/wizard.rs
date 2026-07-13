// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Interactive 7-step wizard for building `.oxp` asset packs.
//!
//! All I/O is injected via generic `BufRead` / `Write` parameters so that
//! the full wizard flow can be exercised in unit tests using `std::io::Cursor`
//! and `Vec<u8>` without touching the real terminal.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::{ensure, Context, Result};

use oxihuman_core::asset_pack_builder::{
    AssetPackBuilder, AssetPackMeta, MorphPreset, TextureAsset, TextureFormat,
};
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_core::{csv_row_count, detect_format, parse_csv, ImageFormat};

use crate::commands::pack::partition_by_policy;

// ── Public API ────────────────────────────────────────────────────────────────

/// Entry point for tests / programmatic callers: accepts any `BufRead` +
/// `Write` pair so the wizard can run without a real terminal.
pub fn cmd_pack_wizard_io<R: BufRead, W: Write>(
    args: &[String],
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    let strict = args.iter().any(|a| a == "--strict");
    let policy = if strict {
        Policy::new(PolicyProfile::Strict)
    } else {
        Policy::new(PolicyProfile::Standard)
    };

    // ── Step 1: Pack metadata ────────────────────────────────────────────────
    writeln!(writer, "=== OxiHuman Asset Pack Wizard ===").ok();
    writeln!(writer).ok();
    writeln!(writer, "Step 1: Pack metadata").ok();

    let pack_name = prompt_with_default(reader, writer, "Pack name", "my_pack")?;
    let author = prompt_with_default(reader, writer, "Author", "COOLJAPAN OU")?;
    let version = prompt_with_default(reader, writer, "Version", "0.1.0")?;
    let license = prompt_with_default(reader, writer, "License", "Apache-2.0")?;

    // ── Step 2: Targets directory (required) ─────────────────────────────────
    writeln!(writer).ok();
    writeln!(writer, "Step 2: Targets directory (required)").ok();
    let targets_raw = prompt_with_default(reader, writer, "Targets directory", "")?;
    ensure!(!targets_raw.is_empty(), "targets directory is required");
    let targets_dir = PathBuf::from(&targets_raw);
    ensure!(
        targets_dir.exists(),
        "targets directory does not exist: {}",
        targets_dir.display()
    );

    // ── Step 3: Texture directory (optional) ─────────────────────────────────
    writeln!(writer).ok();
    writeln!(
        writer,
        "Step 3: Texture directory (optional, press Enter to skip)"
    )
    .ok();
    let texture_dir = prompt_optional_path(reader, writer, "Texture directory")?;

    // ── Step 4: Preset CSV file (optional) ───────────────────────────────────
    writeln!(writer).ok();
    writeln!(
        writer,
        "Step 4: Preset CSV file (optional, press Enter to skip)"
    )
    .ok();
    let preset_csv = prompt_optional_path(reader, writer, "Preset CSV file")?;

    // ── Step 5: Output path ───────────────────────────────────────────────────
    writeln!(writer).ok();
    writeln!(writer, "Step 5: Output path").ok();
    let output_raw = prompt_with_default(reader, writer, "Output path", "./output.oxp")?;
    let output_path = PathBuf::from(&output_raw);

    // ── Step 6: Build ─────────────────────────────────────────────────────────
    writeln!(writer).ok();
    writeln!(writer, "Step 6: Building pack...").ok();

    let pack_bytes = build_pack_from_wizard(
        &pack_name,
        &author,
        &version,
        &license,
        &targets_dir,
        texture_dir.as_deref(),
        preset_csv.as_deref(),
        &policy,
        writer,
    )?;

    // Write the OXP file.
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory: {}", parent.display()))?;
        }
    }
    std::fs::write(&output_path, &pack_bytes)
        .with_context(|| format!("writing pack to: {}", output_path.display()))?;
    writeln!(writer).ok();

    // Generate manifest JSON alongside the output file.
    let manifest_path = {
        let mut p = output_path.clone().into_os_string();
        p.push(".manifest.json");
        PathBuf::from(p)
    };
    let manifest_json = build_manifest_json(
        &pack_name,
        &author,
        &version,
        &license,
        &targets_dir,
        &output_path,
    );
    std::fs::write(&manifest_path, manifest_json.as_bytes())
        .with_context(|| format!("writing manifest to: {}", manifest_path.display()))?;

    // ── Step 7: Done ──────────────────────────────────────────────────────────
    writeln!(writer).ok();
    writeln!(writer, "Done: {}", output_path.display()).ok();

    Ok(())
}

/// Standard entry point that uses the real stdin/stdout.
pub fn cmd_pack_wizard(args: &[String]) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    cmd_pack_wizard_io(args, &mut reader, &mut writer)
}

// ── Helper: prompt with default ───────────────────────────────────────────────

/// Print `"<prompt> [<default>]: "`, read a line, and return the trimmed input.
/// If the input is empty the default is returned instead.
pub fn prompt_with_default<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
    default: &str,
) -> Result<String> {
    if default.is_empty() {
        write!(writer, "{}: ", prompt).ok();
    } else {
        write!(writer, "{} [{}]: ", prompt, default).ok();
    }
    writer.flush().ok();

    let mut line = String::new();
    reader.read_line(&mut line).context("reading input line")?;

    let trimmed = line.trim().to_string();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed)
    }
}

/// Print an optional-path prompt.  Returns `None` if the user enters nothing.
pub fn prompt_optional_path<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prompt: &str,
) -> Result<Option<PathBuf>> {
    write!(writer, "{} (optional): ", prompt).ok();
    writer.flush().ok();

    let mut line = String::new();
    reader.read_line(&mut line).context("reading input line")?;

    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(trimmed)))
    }
}

// ── Internal build logic ──────────────────────────────────────────────────────

/// Scan `targets_dir` for `.target` files (filtered by `policy`), optionally
/// ingest a `texture_dir` of raster images and a `preset_csv` of morph
/// presets, and build the OXP bytes. Prints progress markers to `writer`.
#[allow(clippy::too_many_arguments)]
fn build_pack_from_wizard<W: Write>(
    pack_name: &str,
    author: &str,
    version: &str,
    license: &str,
    targets_dir: &std::path::Path,
    texture_dir: Option<&std::path::Path>,
    preset_csv: Option<&std::path::Path>,
    policy: &Policy,
    writer: &mut W,
) -> Result<Vec<u8>> {
    let mut builder = AssetPackBuilder::new(pack_name);
    let meta = AssetPackMeta {
        version: version.to_string(),
        author: author.to_string(),
        license: license.to_string(),
        description: format!("Asset pack: {}", pack_name),
        created_at: 0,
    };
    builder.set_meta(meta);

    // ── Targets: scan .target files, filtered by policy ──────────────────────
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(targets_dir)
        .with_context(|| format!("reading targets dir: {}", targets_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.path());

    let stems: Vec<String> = entries
        .iter()
        .map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        })
        .collect();
    let (_allowed, rejected) = partition_by_policy(&stems, policy);
    if !rejected.is_empty() {
        writeln!(
            writer,
            "  {} target(s) rejected by policy: {}",
            rejected.len(),
            rejected.join(", ")
        )
        .ok();
    }

    write!(writer, "  ").ok();
    for (entry, name) in entries.iter().zip(stems.iter()) {
        if !policy.is_target_allowed(name, &[]) {
            continue;
        }
        let path = entry.path();
        let data = std::fs::read(&path)
            .with_context(|| format!("reading target file: {}", path.display()))?;
        builder.add_target(oxihuman_core::asset_pack_builder::TargetDelta {
            name: name.clone(),
            data,
        });
        write!(writer, ".").ok();
        writer.flush().ok();
    }
    writeln!(writer).ok();

    // ── Textures: decode every recognised raster image in texture_dir ────────
    if let Some(td) = texture_dir {
        writeln!(writer, "  scanning textures in {}...", td.display()).ok();
        let mut tex_entries: Vec<std::fs::DirEntry> = std::fs::read_dir(td)
            .with_context(|| format!("reading texture dir: {}", td.display()))?
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();
        tex_entries.sort_by_key(|e| e.path());

        for entry in &tex_entries {
            let path = entry.path();
            let bytes = std::fs::read(&path)
                .with_context(|| format!("reading texture file: {}", path.display()))?;
            let format = detect_format(&bytes);
            let decoded = match format {
                ImageFormat::Png => Some((oxihuman_core::png_decode(&bytes), TextureFormat::Png)),
                ImageFormat::Jpeg => Some((
                    oxihuman_core::jpeg_decode(&bytes)
                        .map_err(|e| oxihuman_core::ImageError::DecodeError(e.to_string())),
                    TextureFormat::Jpeg,
                )),
                ImageFormat::Gif => Some((
                    oxihuman_core::gif_decode(&bytes)
                        .map_err(|e| oxihuman_core::ImageError::DecodeError(e.to_string())),
                    TextureFormat::Png,
                )),
                ImageFormat::Tiff => Some((
                    oxihuman_core::tiff_decode(&bytes)
                        .map_err(|e| oxihuman_core::ImageError::DecodeError(e.to_string())),
                    TextureFormat::Png,
                )),
                ImageFormat::Webp => Some((
                    oxihuman_core::webp_decode(&bytes)
                        .map_err(|e| oxihuman_core::ImageError::DecodeError(e.to_string())),
                    TextureFormat::Png,
                )),
                _ => None,
            };

            let Some((decode_result, tex_format)) = decoded else {
                writeln!(
                    writer,
                    "    skip (unrecognised image format): {}",
                    path.display()
                )
                .ok();
                continue;
            };

            let raw = decode_result
                .with_context(|| format!("decoding texture image: {}", path.display()))?;
            let pixel_count = raw.width * raw.height;
            if pixel_count == 0 || raw.pixels.len() % pixel_count != 0 {
                writeln!(
                    writer,
                    "    skip (inconsistent pixel data): {}",
                    path.display()
                )
                .ok();
                continue;
            }
            let channels = (raw.pixels.len() / pixel_count) as u8;
            if !(1..=4).contains(&channels) {
                writeln!(
                    writer,
                    "    skip (unsupported channel count {}): {}",
                    channels,
                    path.display()
                )
                .ok();
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("texture")
                .to_string();
            let texture = TextureAsset {
                name: name.clone(),
                width: raw.width as u32,
                height: raw.height as u32,
                channels,
                data: raw.pixels,
                format: tex_format,
            };
            builder
                .add_texture(texture)
                .with_context(|| format!("adding texture '{}'", name))?;
            write!(writer, ".").ok();
            writer.flush().ok();
        }
        writeln!(writer).ok();
    }

    // ── Presets: parse preset_csv into MorphPreset entries ────────────────────
    if let Some(csv_path) = preset_csv {
        writeln!(writer, "  parsing presets from {}...", csv_path.display()).ok();
        let csv_text = std::fs::read_to_string(csv_path)
            .with_context(|| format!("reading preset CSV: {}", csv_path.display()))?;
        let table = parse_csv(&csv_text);
        ensure!(
            table.headers.iter().any(|h| h == "name"),
            "preset CSV must have a 'name' column: {}",
            csv_path.display()
        );

        let param_cols: Vec<&str> = table
            .headers
            .iter()
            .map(|h| h.as_str())
            .filter(|h| *h != "name" && *h != "description" && *h != "tags")
            .collect();

        for row in 0..csv_row_count(&table) {
            let name = oxihuman_core::csv_field(&table, row, "name")
                .unwrap_or_default()
                .to_string();
            if name.is_empty() {
                continue;
            }
            let description = oxihuman_core::csv_field(&table, row, "description")
                .unwrap_or_default()
                .to_string();
            let tags: Vec<String> = oxihuman_core::csv_field(&table, row, "tags")
                .unwrap_or_default()
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let mut params: HashMap<String, f64> = HashMap::new();
            for col in &param_cols {
                if let Some(raw) = oxihuman_core::csv_field(&table, row, col) {
                    if let Ok(v) = raw.trim().parse::<f64>() {
                        params.insert((*col).to_string(), v);
                    }
                }
            }

            let preset = MorphPreset {
                name: name.clone(),
                description,
                params,
                tags,
            };
            builder
                .add_preset(preset)
                .with_context(|| format!("adding preset '{}'", name))?;
            write!(writer, ".").ok();
            writer.flush().ok();
        }
        writeln!(writer).ok();
    }

    builder.build()
}

/// Produce a manifest JSON string with pack metadata.
fn build_manifest_json(
    name: &str,
    author: &str,
    version: &str,
    license: &str,
    targets_dir: &std::path::Path,
    output_path: &std::path::Path,
) -> String {
    // Hand-build JSON to avoid adding a serde_json dependency (it's already
    // in the workspace transitively, but we stay within the allowed deps).
    format!(
        "{{\n  \"name\": {},\n  \"author\": {},\n  \"version\": {},\n  \"license\": {},\n  \"targets_dir\": {},\n  \"output_path\": {}\n}}\n",
        json_string(name),
        json_string(author),
        json_string(version),
        json_string(license),
        json_string(&targets_dir.display().to_string()),
        json_string(&output_path.display().to_string()),
    )
}

/// Minimal JSON string escaping for manifest values.
fn json_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Helper: build a simulated input stream from lines.
    fn make_input(lines: &[&str]) -> Cursor<Vec<u8>> {
        let joined = lines.join("\n") + "\n";
        Cursor::new(joined.into_bytes())
    }

    // ── Test 1: Wizard completes successfully ─────────────────────────────────

    #[test]
    fn wizard_completes_ok() -> Result<()> {
        let tmp = std::env::temp_dir().join("oxihuman_wizard_test_ok");
        std::fs::create_dir_all(&tmp)?;

        // Create a dummy .target file so the builder has something to pack.
        let target_file = tmp.join("test_target.target");
        std::fs::write(&target_file, b"1 0.1 0.0 0.0\n")?;

        let output_path = tmp.join("test_output.oxp");

        let input_lines = vec![
            "wizard_pack",                                     // pack name
            "Test Author",                                     // author
            "0.2.0",                                           // version
            "MIT",                                             // license
            tmp.to_str().unwrap_or("/tmp"),                    // targets dir
            "",                                                // texture dir (skip)
            "",                                                // preset CSV (skip)
            output_path.to_str().unwrap_or("/tmp/output.oxp"), // output path
        ];
        let mut reader = make_input(&input_lines);
        let mut writer: Vec<u8> = Vec::new();

        cmd_pack_wizard_io(&[], &mut reader, &mut writer)?;

        assert!(output_path.exists(), "output .oxp file must be created");

        let manifest_path = {
            let mut p = output_path.clone().into_os_string();
            p.push(".manifest.json");
            PathBuf::from(p)
        };
        assert!(manifest_path.exists(), "manifest JSON must be created");

        let output_text = String::from_utf8_lossy(&writer);
        assert!(
            output_text.contains("Done:"),
            "output must contain 'Done:' marker"
        );

        // Cleanup
        let _ = std::fs::remove_file(&target_file);
        let _ = std::fs::remove_file(&output_path);
        let _ = std::fs::remove_file(&manifest_path);

        Ok(())
    }

    // ── Test 2: Rejects nonexistent targets directory ─────────────────────────

    #[test]
    fn wizard_rejects_nonexistent_targets_dir() {
        let nonexistent = "/tmp/oxihuman_wizard_definitely_does_not_exist_12345";

        let input_lines = vec![
            "my_pack", // pack name
            "COOLJAPAN OU",
            "0.1.0",
            "Apache-2.0",
            nonexistent, // targets dir — does not exist
        ];
        let mut reader = make_input(&input_lines);
        let mut writer: Vec<u8> = Vec::new();

        let result = cmd_pack_wizard_io(&[], &mut reader, &mut writer);
        assert!(
            result.is_err(),
            "wizard must return Err for nonexistent targets dir"
        );
    }

    // ── Test 3: Uses defaults on all-empty input ──────────────────────────────

    #[test]
    fn wizard_uses_defaults_on_empty_input() -> Result<()> {
        let tmp = std::env::temp_dir().join("oxihuman_wizard_test_defaults");
        std::fs::create_dir_all(&tmp)?;

        // No .target files — empty directory is fine, builder will still build.
        let output_path = tmp.join("output.oxp");

        // All metadata fields are empty → defaults should kick in.
        // Targets dir must be provided (required), output is also provided.
        let input_lines = vec![
            "",                                                // pack name → "my_pack"
            "",                                                // author   → "COOLJAPAN OU"
            "",                                                // version  → "0.1.0"
            "",                                                // license  → "Apache-2.0"
            tmp.to_str().unwrap_or("/tmp"),                    // targets dir (required, must exist)
            "",                                                // texture dir → None
            "",                                                // preset CSV  → None
            output_path.to_str().unwrap_or("/tmp/output.oxp"), // output path
        ];
        let mut reader = make_input(&input_lines);
        let mut writer: Vec<u8> = Vec::new();

        cmd_pack_wizard_io(&[], &mut reader, &mut writer)?;

        // Verify the manifest JSON contains the default values.
        let manifest_path = {
            let mut p = output_path.clone().into_os_string();
            p.push(".manifest.json");
            PathBuf::from(p)
        };
        assert!(manifest_path.exists(), "manifest must be created");
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        assert!(
            manifest_content.contains("my_pack"),
            "manifest must contain default pack name 'my_pack'"
        );
        assert!(
            manifest_content.contains("COOLJAPAN OU"),
            "manifest must contain default author 'COOLJAPAN OU'"
        );
        assert!(
            manifest_content.contains("0.1.0"),
            "manifest must contain default version '0.1.0'"
        );
        assert!(
            manifest_content.contains("Apache-2.0"),
            "manifest must contain default license 'Apache-2.0'"
        );

        // Cleanup
        let _ = std::fs::remove_file(&output_path);
        let _ = std::fs::remove_file(&manifest_path);

        Ok(())
    }

    // ── Test 4: texture-dir and preset-CSV inputs are actually wired in ──────

    #[test]
    fn wizard_ingests_textures_and_presets() -> Result<()> {
        use oxihuman_core::asset_pack_builder::load_pack_from_bytes;
        use oxihuman_core::png_encode_rgb;

        let tmp = std::env::temp_dir().join(format!(
            "oxihuman_wizard_test_textures_presets_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp)?;

        let targets_dir = tmp.join("targets");
        std::fs::create_dir_all(&targets_dir)?;
        std::fs::write(targets_dir.join("height-up.target"), b"1 0.1 0.0 0.0\n")?;

        // A tiny 2x2 RGB PNG.
        let texture_dir = tmp.join("textures");
        std::fs::create_dir_all(&texture_dir)?;
        let pixels: Vec<u8> = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0];
        let png_bytes = png_encode_rgb(2, 2, &pixels).context("encoding test PNG")?;
        std::fs::write(texture_dir.join("skin_albedo.png"), &png_bytes)?;

        // A 2-row preset CSV with one numeric param column.
        let preset_csv = tmp.join("presets.csv");
        std::fs::write(
            &preset_csv,
            "name,description,tags,height\nTall,Above average,body;height,1.5\nShort,Below average,body,0.5\n",
        )?;

        let output_path = tmp.join("bundle.oxp");
        let input_lines = vec![
            "textured_pack",
            "COOLJAPAN OU",
            "0.1.0",
            "Apache-2.0",
            targets_dir.to_str().unwrap_or_default(),
            texture_dir.to_str().unwrap_or_default(),
            preset_csv.to_str().unwrap_or_default(),
            output_path.to_str().unwrap_or_default(),
        ];
        let mut reader = make_input(&input_lines);
        let mut writer: Vec<u8> = Vec::new();

        cmd_pack_wizard_io(&[], &mut reader, &mut writer)?;

        let pack_bytes = std::fs::read(&output_path)?;
        let index = load_pack_from_bytes(&pack_bytes).context("loading built pack")?;

        assert_eq!(index.textures.len(), 1, "one texture must be ingested");
        assert_eq!(index.textures[0].name, "skin_albedo");
        assert_eq!(index.textures[0].width, 2);
        assert_eq!(index.textures[0].height, 2);

        assert_eq!(
            index.presets.len(),
            2,
            "both preset CSV rows must be ingested"
        );
        let tall = index
            .presets
            .iter()
            .find(|p| p.name == "Tall")
            .expect("Tall preset must exist");
        assert!((tall.params.get("height").copied().unwrap_or(0.0) - 1.5).abs() < 1e-9);
        assert!(tall.tags.contains(&"height".to_string()));

        assert!(
            index.target_names.iter().any(|n| n == "height-up"),
            "target must still be ingested alongside textures/presets"
        );

        let _ = std::fs::remove_dir_all(&tmp);
        Ok(())
    }

    // ── Test 5: policy gate rejects blocked-tag target names ──────────────────

    #[test]
    fn wizard_filters_blocked_targets_by_policy() -> Result<()> {
        use oxihuman_core::asset_pack_builder::load_pack_from_bytes;

        let tmp = std::env::temp_dir().join(format!(
            "oxihuman_wizard_test_policy_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp)?;
        std::fs::write(tmp.join("height-up.target"), b"1 0.1 0.0 0.0\n")?;
        std::fs::write(tmp.join("explicit-pose.target"), b"1 0.1 0.0 0.0\n")?;

        let output_path = tmp.join("filtered.oxp");
        let input_lines = vec![
            "policy_pack",
            "COOLJAPAN OU",
            "0.1.0",
            "Apache-2.0",
            tmp.to_str().unwrap_or_default(),
            "",
            "",
            output_path.to_str().unwrap_or_default(),
        ];
        let mut reader = make_input(&input_lines);
        let mut writer: Vec<u8> = Vec::new();

        cmd_pack_wizard_io(&[], &mut reader, &mut writer)?;

        let pack_bytes = std::fs::read(&output_path)?;
        let index = load_pack_from_bytes(&pack_bytes).context("loading built pack")?;
        assert!(index.target_names.iter().any(|n| n == "height-up"));
        assert!(
            !index.target_names.iter().any(|n| n == "explicit-pose"),
            "blocked-tag target must be excluded from the pack"
        );

        let output_text = String::from_utf8_lossy(&writer);
        assert!(
            output_text.contains("rejected by policy"),
            "wizard output should note rejected targets"
        );

        let _ = std::fs::remove_dir_all(&tmp);
        Ok(())
    }
}
