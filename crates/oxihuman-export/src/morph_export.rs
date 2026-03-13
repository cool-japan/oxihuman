// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Morph target export: pack blend-shape deltas into GLB-compatible format.

// ── Structs / Enums ──────────────────────────────────────────────────────────

/// A single morph target with delta positions and optional delta normals.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTargetExport {
    /// Human-readable name of this morph target.
    pub name: String,
    /// Per-vertex position deltas (dx, dy, dz).
    pub delta_positions: Vec<[f32; 3]>,
    /// Per-vertex normal deltas (dx, dy, dz). May be empty if unused.
    pub delta_normals: Vec<[f32; 3]>,
    /// Default weight in [0, 1].
    pub default_weight: f32,
}

/// Configuration for morph target export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphExportConfig {
    /// Minimum delta magnitude to keep (prune smaller deltas to zero).
    pub threshold: f32,
    /// Whether to include delta normals in the export.
    pub include_normals: bool,
    /// Whether to normalize deltas before export.
    pub normalize: bool,
}

/// A bundle of morph targets ready for export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphExportBundle {
    /// All morph targets in export order.
    pub targets: Vec<MorphTargetExport>,
    /// Configuration used when building this bundle.
    pub config: MorphExportConfig,
}

// ── Constructor functions ────────────────────────────────────────────────────

/// Create a default morph export configuration.
#[allow(dead_code)]
pub fn default_morph_export_config() -> MorphExportConfig {
    MorphExportConfig {
        threshold: 1e-6,
        include_normals: true,
        normalize: false,
    }
}

/// Create a new morph target export with the given name and vertex count.
///
/// All deltas are initialised to zero.
#[allow(dead_code)]
pub fn new_morph_target_export(name: &str, vertex_count: usize) -> MorphTargetExport {
    MorphTargetExport {
        name: name.to_string(),
        delta_positions: vec![[0.0; 3]; vertex_count],
        delta_normals: vec![[0.0; 3]; vertex_count],
        default_weight: 0.0,
    }
}

// ── Accessors ────────────────────────────────────────────────────────────────

/// Return a reference to the delta positions of a morph target.
#[allow(dead_code)]
pub fn morph_delta_positions(target: &MorphTargetExport) -> &[[f32; 3]] {
    &target.delta_positions
}

/// Return a reference to the delta normals of a morph target.
#[allow(dead_code)]
pub fn morph_delta_normals(target: &MorphTargetExport) -> &[[f32; 3]] {
    &target.delta_normals
}

/// Return the number of morph targets in a bundle.
#[allow(dead_code)]
pub fn morph_target_count(bundle: &MorphExportBundle) -> usize {
    bundle.targets.len()
}

/// Return the name of a morph target, or an empty string if the index is out of range.
#[allow(dead_code)]
pub fn morph_target_name(bundle: &MorphExportBundle, index: usize) -> &str {
    bundle
        .targets
        .get(index)
        .map(|t| t.name.as_str())
        .unwrap_or("")
}

// ── Bundle building ──────────────────────────────────────────────────────────

/// Pack a slice of morph targets into an export bundle using the given config.
#[allow(dead_code)]
pub fn pack_morph_bundle(
    targets: &[MorphTargetExport],
    config: &MorphExportConfig,
) -> MorphExportBundle {
    let mut out_targets: Vec<MorphTargetExport> = Vec::with_capacity(targets.len());
    for t in targets {
        let mut target = t.clone();
        if config.normalize {
            normalize_morph_deltas_internal(&mut target.delta_positions);
        }
        if !config.include_normals {
            target.delta_normals.clear();
        }
        if config.threshold > 0.0 {
            filter_deltas_by_threshold(&mut target.delta_positions, config.threshold);
            if !target.delta_normals.is_empty() {
                filter_deltas_by_threshold(&mut target.delta_normals, config.threshold);
            }
        }
        out_targets.push(target);
    }
    MorphExportBundle {
        targets: out_targets,
        config: config.clone(),
    }
}

/// Serialise a morph bundle to a JSON string.
#[allow(dead_code)]
pub fn morph_bundle_to_json(bundle: &MorphExportBundle) -> String {
    let mut s = String::from("{\n  \"morph_targets\": [\n");
    for (i, t) in bundle.targets.iter().enumerate() {
        s.push_str(&format!(
            "    {{\"name\":\"{}\",\"vertex_count\":{},\"has_normals\":{},\"default_weight\":{:.6}}}",
            t.name,
            t.delta_positions.len(),
            !t.delta_normals.is_empty(),
            t.default_weight,
        ));
        if i + 1 < bundle.targets.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push_str("  ],\n");
    s.push_str(&format!(
        "  \"config\":{{\"threshold\":{:.8},\"include_normals\":{},\"normalize\":{}}}\n",
        bundle.config.threshold, bundle.config.include_normals, bundle.config.normalize,
    ));
    s.push('}');
    s
}

// ── Computation utilities ────────────────────────────────────────────────────

/// Compute the magnitude of a single morph delta vector.
#[allow(dead_code)]
pub fn morph_delta_magnitude(delta: [f32; 3]) -> f32 {
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

/// Normalize all delta positions in a morph target in-place so the maximum
/// magnitude is 1.0.  Does nothing if all deltas are zero.
#[allow(dead_code)]
pub fn normalize_morph_deltas(target: &mut MorphTargetExport) {
    normalize_morph_deltas_internal(&mut target.delta_positions);
}

/// Filter delta positions in a morph target: set any delta with magnitude below
/// `threshold` to zero.
#[allow(dead_code)]
pub fn filter_morph_by_threshold(target: &mut MorphTargetExport, threshold: f32) {
    filter_deltas_by_threshold(&mut target.delta_positions, threshold);
    if !target.delta_normals.is_empty() {
        filter_deltas_by_threshold(&mut target.delta_normals, threshold);
    }
}

/// Return the valid weight range for a morph target (always `(0.0, 1.0)`).
#[allow(dead_code)]
pub fn morph_weight_range() -> (f32, f32) {
    (0.0, 1.0)
}

/// Estimate the size in bytes of a morph export bundle when serialised to
/// a binary GLB buffer.
///
/// Each target contributes `vertex_count * 3 * 4` bytes for positions,
/// plus the same for normals if present.
#[allow(dead_code)]
pub fn morph_export_size_bytes(bundle: &MorphExportBundle) -> usize {
    let mut total = 0usize;
    for t in &bundle.targets {
        total += t.delta_positions.len() * 3 * 4;
        if !t.delta_normals.is_empty() {
            total += t.delta_normals.len() * 3 * 4;
        }
    }
    total
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn normalize_morph_deltas_internal(deltas: &mut [[f32; 3]]) {
    let max_mag = deltas
        .iter()
        .map(|d| morph_delta_magnitude(*d))
        .fold(0.0_f32, f32::max);
    if max_mag < 1e-12 {
        return;
    }
    let inv = 1.0 / max_mag;
    for d in deltas.iter_mut() {
        d[0] *= inv;
        d[1] *= inv;
        d[2] *= inv;
    }
}

fn filter_deltas_by_threshold(deltas: &mut [[f32; 3]], threshold: f32) {
    for d in deltas.iter_mut() {
        if morph_delta_magnitude(*d) < threshold {
            *d = [0.0; 3];
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_morph_export_config();
        assert!(cfg.include_normals);
        assert!(!cfg.normalize);
        assert!(cfg.threshold > 0.0);
    }

    #[test]
    fn new_morph_target_has_correct_size() {
        let t = new_morph_target_export("smile", 100);
        assert_eq!(t.name, "smile");
        assert_eq!(t.delta_positions.len(), 100);
        assert_eq!(t.delta_normals.len(), 100);
        assert!((t.default_weight - 0.0).abs() < 1e-6);
    }

    #[test]
    fn morph_delta_positions_accessor() {
        let t = new_morph_target_export("test", 5);
        let dp = morph_delta_positions(&t);
        assert_eq!(dp.len(), 5);
    }

    #[test]
    fn morph_delta_normals_accessor() {
        let t = new_morph_target_export("test", 5);
        let dn = morph_delta_normals(&t);
        assert_eq!(dn.len(), 5);
    }

    #[test]
    fn morph_delta_magnitude_basic() {
        assert!((morph_delta_magnitude([3.0, 4.0, 0.0]) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn morph_delta_magnitude_zero() {
        assert!((morph_delta_magnitude([0.0, 0.0, 0.0])).abs() < 1e-6);
    }

    #[test]
    fn pack_morph_bundle_preserves_count() {
        let targets = vec![
            new_morph_target_export("a", 10),
            new_morph_target_export("b", 10),
        ];
        let cfg = default_morph_export_config();
        let bundle = pack_morph_bundle(&targets, &cfg);
        assert_eq!(morph_target_count(&bundle), 2);
    }

    #[test]
    fn pack_morph_bundle_strips_normals_when_disabled() {
        let targets = vec![new_morph_target_export("a", 10)];
        let mut cfg = default_morph_export_config();
        cfg.include_normals = false;
        let bundle = pack_morph_bundle(&targets, &cfg);
        assert!(bundle.targets[0].delta_normals.is_empty());
    }

    #[test]
    fn morph_target_name_valid_index() {
        let targets = vec![new_morph_target_export("blink", 5)];
        let cfg = default_morph_export_config();
        let bundle = pack_morph_bundle(&targets, &cfg);
        assert_eq!(morph_target_name(&bundle, 0), "blink");
    }

    #[test]
    fn morph_target_name_invalid_index() {
        let targets = vec![new_morph_target_export("blink", 5)];
        let cfg = default_morph_export_config();
        let bundle = pack_morph_bundle(&targets, &cfg);
        assert_eq!(morph_target_name(&bundle, 99), "");
    }

    #[test]
    fn normalize_morph_deltas_max_is_one() {
        let mut t = new_morph_target_export("test", 3);
        t.delta_positions[0] = [3.0, 0.0, 0.0];
        t.delta_positions[1] = [0.0, 4.0, 0.0];
        t.delta_positions[2] = [0.0, 0.0, 5.0];
        normalize_morph_deltas(&mut t);
        let max_mag = t
            .delta_positions
            .iter()
            .map(|d| morph_delta_magnitude(*d))
            .fold(0.0_f32, f32::max);
        assert!((max_mag - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalize_morph_deltas_all_zero_is_noop() {
        let mut t = new_morph_target_export("test", 3);
        normalize_morph_deltas(&mut t);
        for d in &t.delta_positions {
            assert_eq!(*d, [0.0; 3]);
        }
    }

    #[test]
    fn filter_morph_by_threshold_zeroes_small() {
        let mut t = new_morph_target_export("test", 3);
        t.delta_positions[0] = [0.001, 0.0, 0.0];
        t.delta_positions[1] = [1.0, 0.0, 0.0];
        t.delta_positions[2] = [0.0005, 0.0, 0.0];
        filter_morph_by_threshold(&mut t, 0.01);
        assert_eq!(t.delta_positions[0], [0.0; 3]);
        assert!((t.delta_positions[1][0] - 1.0).abs() < 1e-6);
        assert_eq!(t.delta_positions[2], [0.0; 3]);
    }

    #[test]
    fn morph_weight_range_is_zero_to_one() {
        let (lo, hi) = morph_weight_range();
        assert!((lo - 0.0).abs() < 1e-6);
        assert!((hi - 1.0).abs() < 1e-6);
    }

    #[test]
    fn morph_export_size_bytes_calculation() {
        let targets = vec![new_morph_target_export("a", 10)];
        let cfg = default_morph_export_config();
        let bundle = pack_morph_bundle(&targets, &cfg);
        // 10 verts * 3 floats * 4 bytes = 120 for positions + 120 for normals
        assert_eq!(morph_export_size_bytes(&bundle), 240);
    }

    #[test]
    fn morph_export_size_bytes_no_normals() {
        let targets = vec![new_morph_target_export("a", 10)];
        let mut cfg = default_morph_export_config();
        cfg.include_normals = false;
        let bundle = pack_morph_bundle(&targets, &cfg);
        assert_eq!(morph_export_size_bytes(&bundle), 120);
    }

    #[test]
    fn morph_bundle_to_json_contains_name() {
        let targets = vec![new_morph_target_export("jaw_open", 5)];
        let cfg = default_morph_export_config();
        let bundle = pack_morph_bundle(&targets, &cfg);
        let json = morph_bundle_to_json(&bundle);
        assert!(json.contains("jaw_open"));
        assert!(json.contains("morph_targets"));
    }

    #[test]
    fn morph_bundle_to_json_multiple_targets() {
        let targets = vec![
            new_morph_target_export("a", 5),
            new_morph_target_export("b", 5),
        ];
        let cfg = default_morph_export_config();
        let bundle = pack_morph_bundle(&targets, &cfg);
        let json = morph_bundle_to_json(&bundle);
        assert!(json.contains("\"a\""));
        assert!(json.contains("\"b\""));
    }
}
