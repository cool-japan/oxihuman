// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! JSON file I/O for [`BodyPreset`].

#![allow(dead_code)]

use std::path::Path;

use crate::presets::BodyPreset;

/// Save a [`BodyPreset`] to a JSON file.
pub fn save_preset_json(preset: &BodyPreset, path: &Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(preset)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load a [`BodyPreset`] from a JSON file.
pub fn load_preset_json(path: &Path) -> anyhow::Result<BodyPreset> {
    let contents = std::fs::read_to_string(path)?;
    let preset = serde_json::from_str(&contents)?;
    Ok(preset)
}

/// Save multiple presets to a JSON array file.
pub fn save_preset_library_json(presets: &[BodyPreset], path: &Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(presets)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load multiple presets from a JSON array file.
pub fn load_preset_library_json(path: &Path) -> anyhow::Result<Vec<BodyPreset>> {
    let contents = std::fs::read_to_string(path)?;
    let presets = serde_json::from_str(&contents)?;
    Ok(presets)
}

/// Serialize a [`BodyPreset`] to a JSON string (no file I/O).
pub fn preset_to_json_string(preset: &BodyPreset) -> anyhow::Result<String> {
    Ok(serde_json::to_string(preset)?)
}

/// Deserialize a [`BodyPreset`] from a JSON string.
pub fn preset_from_json_string(s: &str) -> anyhow::Result<BodyPreset> {
    Ok(serde_json::from_str(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("/tmp/test_preset_io_{}.json", name))
    }

    #[test]
    fn save_and_load_roundtrip() {
        let preset = BodyPreset::Athletic;
        let path = tmp_path("roundtrip");
        save_preset_json(&preset, &path).expect("save failed");
        let loaded = load_preset_json(&path).expect("load failed");
        assert_eq!(preset, loaded);
    }

    #[test]
    fn preset_to_json_string_is_valid_json() {
        let preset = BodyPreset::Average;
        let s = preset_to_json_string(&preset).expect("serialize failed");
        let value: serde_json::Value = serde_json::from_str(&s).expect("not valid JSON");
        assert!(
            value.is_string(),
            "BodyPreset should serialize as a JSON string"
        );
    }

    #[test]
    fn preset_from_json_string_roundtrip() {
        for name in BodyPreset::all_names() {
            let preset = BodyPreset::from_name(name).unwrap();
            let s = preset_to_json_string(&preset).expect("serialize failed");
            let back = preset_from_json_string(&s).expect("deserialize failed");
            assert_eq!(preset, back, "roundtrip failed for {}", name);
        }
    }

    #[test]
    fn save_library_and_load_library_roundtrip() {
        let presets = vec![
            BodyPreset::Average,
            BodyPreset::Athletic,
            BodyPreset::Slender,
            BodyPreset::Heavy,
        ];
        let path = tmp_path("library_roundtrip");
        save_preset_library_json(&presets, &path).expect("save failed");
        let loaded = load_preset_library_json(&path).expect("load failed");
        assert_eq!(presets, loaded);
    }

    #[test]
    fn load_json_from_nonexistent_file_errors() {
        let path = tmp_path("nonexistent_xyz_abc_999");
        // make sure it doesn't exist
        let _ = std::fs::remove_file(&path);
        let result = load_preset_json(&path);
        assert!(result.is_err(), "expected Err for nonexistent file");
    }

    #[test]
    fn save_creates_file() {
        let preset = BodyPreset::Tall;
        let path = tmp_path("creates_file");
        // remove if it exists
        let _ = std::fs::remove_file(&path);
        save_preset_json(&preset, &path).expect("save failed");
        assert!(path.exists(), "file should exist after save");
    }

    #[test]
    fn load_string_invalid_json_errors() {
        let result = preset_from_json_string("not valid json {{{{");
        assert!(result.is_err(), "expected Err for invalid JSON");
    }

    #[test]
    fn library_preserves_count() {
        let presets: Vec<BodyPreset> = BodyPreset::all_names()
            .iter()
            .map(|n| BodyPreset::from_name(n).unwrap())
            .collect();
        let n = presets.len();
        let path = tmp_path("count");
        save_preset_library_json(&presets, &path).expect("save failed");
        let loaded = load_preset_library_json(&path).expect("load failed");
        assert_eq!(loaded.len(), n);
    }

    #[test]
    fn library_first_preset_fields_match() {
        let presets = vec![BodyPreset::Senior, BodyPreset::Child, BodyPreset::Petite];
        let path = tmp_path("first_fields");
        save_preset_library_json(&presets, &path).expect("save failed");
        let loaded = load_preset_library_json(&path).expect("load failed");
        assert_eq!(loaded[0], BodyPreset::Senior);
        // Also verify params match
        let p_orig = BodyPreset::Senior.params();
        let p_load = loaded[0].params();
        assert!((p_orig.height - p_load.height).abs() < 1e-6);
        assert!((p_orig.age - p_load.age).abs() < 1e-6);
    }

    #[test]
    fn preset_extra_params_preserved() {
        // BodyPreset is an enum variant — its params() returns ParamState which has
        // extra HashMap. Verify round-trip preserves the variant identity (which
        // implies params() output is identical).
        let preset = BodyPreset::Child;
        let s = preset_to_json_string(&preset).expect("serialize");
        let back = preset_from_json_string(&s).expect("deserialize");
        assert_eq!(preset, back);
        let orig_params = preset.params();
        let back_params = back.params();
        assert!((orig_params.height - back_params.height).abs() < 1e-6);
        assert!((orig_params.weight - back_params.weight).abs() < 1e-6);
        assert!((orig_params.muscle - back_params.muscle).abs() < 1e-6);
        assert!((orig_params.age - back_params.age).abs() < 1e-6);
    }

    #[test]
    fn load_library_wrong_schema_errors() {
        // A JSON that is a single string, not an array — should fail for load_library
        let path = tmp_path("wrong_schema");
        std::fs::write(&path, "\"Athletic\"").unwrap();
        let result = load_preset_library_json(&path);
        assert!(result.is_err(), "expected Err when JSON is not an array");
    }

    #[test]
    fn save_and_load_all_variants() {
        for name in BodyPreset::all_names() {
            let preset = BodyPreset::from_name(name).unwrap();
            let path = tmp_path(&format!("all_variants_{}", name));
            save_preset_json(&preset, &path).expect("save failed");
            let loaded = load_preset_json(&path).expect("load failed");
            assert_eq!(preset, loaded, "mismatch for variant {}", name);
        }
    }
}
