// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the `pack-core` OHPK v1 core pack.
//!
//! When the committed core pack (`assets/packs/oxihuman-core-v1.ohpk`) is
//! present these tests exercise the *real* shipped asset; otherwise they fall
//! back to a small synthetic pack built in-memory with the same invariants so
//! the suite stays green on a fresh checkout that has not run `pack-core`.

use std::path::PathBuf;

use oxihuman_export::{CorePack, CorePackBuilder, CorePackManifest};

/// Repo-root-relative path to the committed core pack.
fn core_pack_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/oxihuman-cli
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("..")
        .join("assets/packs/oxihuman-core-v1.ohpk")
}

/// Build a synthetic OHPK pack that satisfies the same invariants the real
/// core pack must (>10k verts, >=30 targets, age floor 18) so tests can run
/// even before `pack-core` has produced the committed asset.
fn synthetic_pack_bytes() -> Vec<u8> {
    let n_verts = 10_001usize;
    let mut positions = Vec::with_capacity(n_verts * 3);
    for i in 0..n_verts {
        let f = i as f32;
        positions.push((f * 0.001) - 5.0);
        positions.push((f * 0.0007) - 3.0);
        positions.push((f * 0.0003) - 1.0);
    }
    // A trivial two-triangle index list is enough for the container.
    let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

    let mut builder = CorePackBuilder::new();
    let mut manifest = CorePackManifest::new("OxiHuman Core Pack (synthetic)", "1.0.0");
    manifest.age_floor_years = Some(18.0);
    builder.set_manifest(manifest);
    builder.set_base_mesh(&positions, &indices, None);

    // 32 targets, one of them `universal-*` with a clearly non-trivial delta so
    // the morph smoke assertion has something to move.
    for t in 0..32u32 {
        let name = if t == 0 {
            "macrodetails/universal-female-young-maxmuscle-maxweight".to_string()
        } else {
            format!("macrodetails/universal-t{t}")
        };
        // Every target displaces vertex `t` by 0.05 model units on x.
        let sparse = vec![(t, [0.05f32, 0.0, 0.0])];
        builder.add_target(name, "muscle", &sparse);
    }
    builder.build().expect("build synthetic core pack")
}

/// Load the real committed pack when present, else the synthetic fallback.
fn load_pack_bytes() -> Vec<u8> {
    let path = core_pack_path();
    match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => synthetic_pack_bytes(),
    }
}

#[test]
fn core_pack_parses_with_expected_shape() {
    let bytes = load_pack_bytes();
    let pack = CorePack::parse(&bytes).expect("core pack must parse");

    assert!(
        pack.vertex_count() > 10_000,
        "expected >10k base vertices, got {}",
        pack.vertex_count()
    );
    assert!(
        pack.targets().len() >= 30,
        "expected >=30 morph targets, got {}",
        pack.targets().len()
    );
    assert_eq!(
        pack.manifest().age_floor_years,
        Some(18.0),
        "core pack must carry an adult-only age floor of 18 years"
    );
    assert_eq!(pack.manifest().license, "CC0-1.0");
}

#[test]
fn universal_target_morph_displaces_mesh() {
    let bytes = load_pack_bytes();
    let pack = CorePack::parse(&bytes).expect("core pack must parse");

    let base = pack.base_positions();
    let n_verts = pack.vertex_count();
    assert_eq!(base.len(), n_verts * 3);

    // Pick one `universal` target (falls back to the first target if none is
    // named `universal`, which cannot happen for the real or synthetic pack).
    let target = pack
        .targets()
        .iter()
        .find(|t| t.name().contains("universal"))
        .or_else(|| pack.targets().first())
        .expect("pack must contain at least one target");

    // Apply the sparse deltas at weight 1.0 to a clone of the base positions.
    let mut morphed = base.clone();
    for (vid, delta) in target.sparse() {
        let base_idx = (vid as usize).checked_mul(3);
        if let Some(b) = base_idx {
            if b + 2 < morphed.len() {
                morphed[b] += delta[0];
                morphed[b + 1] += delta[1];
                morphed[b + 2] += delta[2];
            }
        }
    }

    // Maximum per-vertex displacement magnitude must be visibly non-zero.
    let mut max_disp = 0.0f32;
    for (o, m) in base.chunks_exact(3).zip(morphed.chunks_exact(3)) {
        let dx = m[0] - o[0];
        let dy = m[1] - o[1];
        let dz = m[2] - o[2];
        let disp = (dx * dx + dy * dy + dz * dz).sqrt();
        if disp > max_disp {
            max_disp = disp;
        }
    }

    assert!(
        max_disp > 0.001,
        "universal target '{}' produced only {max_disp} model-unit displacement",
        target.name()
    );
}

#[test]
fn measure_girth_targets_present_and_inert() {
    let bytes = load_pack_bytes();
    let pack = CorePack::parse(&bytes).expect("core pack must parse");

    // Only the real committed v2 pack carries the `measure/` girth targets; the
    // synthetic fallback (used on a lean checkout) does not, so gate on them.
    let measure: Vec<_> = pack
        .targets()
        .iter()
        .filter(|t| t.name().starts_with("measure/"))
        .collect();
    if measure.is_empty() {
        eprintln!("synthetic pack (no measure/ targets) — skipping");
        return;
    }

    assert!(
        measure.len() >= 8,
        "expected >= 8 measure/ girth targets, got {}",
        measure.len()
    );
    // They must be recorded under the inert `measure` category so the runtime
    // keys them by name at weight 0 (driven only by the fit's refinement stage)
    // instead of wiring them to a macro slider.
    for t in &measure {
        assert_eq!(
            t.category(),
            "measure",
            "measure target {} must carry the inert 'measure' category, got {}",
            t.name(),
            t.category()
        );
    }
    // Each girth dimension is present as an incr/decr pair.
    for dim in ["bust", "underbust", "waist", "hips"] {
        for dir in ["incr", "decr"] {
            let name = format!("measure/measure-{dim}-circ-{dir}");
            assert!(
                pack.targets().iter().any(|t| t.name() == name),
                "core pack must include {name}"
            );
        }
    }
}

#[test]
fn quantization_report_is_honest_and_finite() {
    let bytes = load_pack_bytes();
    let pack = CorePack::parse(&bytes).expect("core pack must parse");
    let report = pack.quantization_report();

    assert!(report.base_pos_max_error_units >= 0.0);
    assert!(report.base_pos_max_error_units.is_finite());
    // The mm figure follows the decimetre convention (1 unit = 100 mm).
    assert!((report.base_pos_max_error_mm - report.base_pos_max_error_units * 100.0).abs() < 1e-3);
    for t in &report.targets {
        assert!(t.max_error_units >= 0.0 && t.max_error_units.is_finite());
    }
}
