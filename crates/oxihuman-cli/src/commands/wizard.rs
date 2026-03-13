// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Interactive 7-step wizard for building `.oxp` asset packs.
//!
//! All I/O is injected via generic `BufRead` / `Write` parameters so that
//! the full wizard flow can be exercised in unit tests using `std::io::Cursor`
//! and `Vec<u8>` without touching the real terminal.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::{ensure, Context, Result};

use oxihuman_core::asset_pack_builder::{AssetPackBuilder, AssetPackMeta};

// ── Public API ────────────────────────────────────────────────────────────────

/// Entry point for tests / programmatic callers: accepts any `BufRead` +
/// `Write` pair so the wizard can run without a real terminal.
pub fn cmd_pack_wizard_io<R: BufRead, W: Write>(
    _args: &[String],
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
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

/// Scan `targets_dir` for `.target` files and build the OXP bytes.
/// Prints a progress dot per file to `writer`.
#[allow(clippy::too_many_arguments)]
fn build_pack_from_wizard<W: Write>(
    pack_name: &str,
    author: &str,
    version: &str,
    license: &str,
    targets_dir: &std::path::Path,
    _texture_dir: Option<&std::path::Path>,
    _preset_csv: Option<&std::path::Path>,
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

    // Scan .target files in the targets directory.
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(targets_dir)
        .with_context(|| format!("reading targets dir: {}", targets_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.path());

    write!(writer, "  ").ok();
    for entry in &entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let data = std::fs::read(&path)
            .with_context(|| format!("reading target file: {}", path.display()))?;
        builder.add_target(oxihuman_core::asset_pack_builder::TargetDelta { name, data });
        write!(writer, ".").ok();
        writer.flush().ok();
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
}
