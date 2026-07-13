// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Native round-trip tests for the precise measurement + fit surface.
//!
//! Two asset guards keep the suite green on a lean checkout: pack-backed tests
//! skip when `assets/packs/oxihuman-core-v1.ohpk` is absent, and ground-truth
//! tests skip when the upstream MakeHuman assets are absent.
//!
//! # Measurement conventions under test
//!
//! The `CrossSectionMeasurer` reads tape girths as band extrema over torso
//! cross-sections: chest = fullest slice of the anatomical bust band
//! (≈ 0.66–0.76 of stature, arm sections rejected per slice), waist =
//! narrowest slice of the natural-waist band (≈ 0.58–0.70), hip = fullest
//! slice above the crotch (≈ 0.505–0.60). The ground-truth test below pins
//! that every MakeHuman `measure/` girth target, **applied with correct
//! vertex indices**, moves its measurement by several centimetres with the
//! right sign and does not move the other girths.
//!
//! # Pack indexing (v3 pack)
//!
//! `.target` files address the base mesh in raw MakeHuman v-line order, but
//! the pack's base mesh stores vertices in the OBJ loader's
//! face-first-occurrence order (UV-seam splits included: 21 833 verts vs
//! 19 158 v-lines). Since the v3 rebuild, `oxihuman-cli pack-core` re-indexes
//! every target through the loader's raw→packed mapping (duplicating each
//! delta across seam copies), so pack targets land on exactly the geometry
//! the upstream `.target` addressed — verified structurally by the
//! `pack_core` invariant tests and behaviourally by the pack-backed
//! girth-response test below, which asserts each `measure/` target moves its
//! own measured girth by the ground-truth magnitude (±30 % for band-
//! convention drift) with the right sign.
//!
//! Achieved end-to-end accuracy is documented in
//! `docs/bench/measurement-error.md` (reproduce with
//! `scripts/measure_roundtrip.sh`).

use std::path::PathBuf;

use oxihuman_morph::measurements::CrossSectionMeasurer;
use oxihuman_wasm::WasmEngine;

fn pack_bytes() -> Option<Vec<u8>> {
    let p =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/oxihuman-core-v1.ohpk");
    std::fs::read(&p).ok()
}

fn upstream_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/upstream/makehuman/makehuman/data")
}

/// Extract `(target, measured, delta)` for a measurement name from a fit report.
fn entry(report: &serde_json::Value, name: &str) -> Option<(f64, f64, f64)> {
    report
        .get("results")?
        .as_array()?
        .iter()
        .find(|r| r.get("name").and_then(|v| v.as_str()) == Some(name))
        .map(|r| {
            (
                r.get("target_cm")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::NAN),
                r.get("measured_cm")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::NAN),
                r.get("delta_cm")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::NAN),
            )
        })
}

/// Parse `base.obj` in raw v-line order, keeping only faces of the `body`
/// group (helpers dropped), fan-triangulated. Returns `(verts_cm, indices)`.
///
/// This deliberately bypasses the engine's OBJ loader: `.target` files index
/// the raw v-line order, so building the measurer directly over that order is
/// the ground-truth way to apply a MakeHuman morph target.
fn parse_base_obj_body_cm(src: &str) -> (Vec<[f64; 3]>, Vec<u32>) {
    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut in_body = false;
    for line in src.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("v ") {
            let mut it = rest.split_whitespace();
            let x: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let y: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let z: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            verts.push([x * 10.0, y * 10.0, z * 10.0]); // decimetres → cm
        } else if let Some(rest) = line.strip_prefix("g ") {
            in_body = rest.trim() == "body";
        } else if let Some(rest) = line.strip_prefix("f ") {
            if !in_body {
                continue;
            }
            let corners: Vec<u32> = rest
                .split_whitespace()
                .filter_map(|c| c.split('/').next())
                .filter_map(|s| s.parse::<u32>().ok())
                .map(|i| i - 1)
                .collect();
            for k in 1..corners.len().saturating_sub(1) {
                indices.push(corners[0]);
                indices.push(corners[k]);
                indices.push(corners[k + 1]);
            }
        }
    }
    (verts, indices)
}

/// Read an upstream `.target` file into sparse `(vid, [dx, dy, dz] cm)` deltas.
fn parse_target_cm(src: &str) -> Vec<(usize, [f64; 3])> {
    let mut out = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        let vid: usize = match it.next().and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let dx: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let dy: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let dz: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        out.push((vid, [dx * 10.0, dy * 10.0, dz * 10.0]));
    }
    out
}

/// Measure `(chest, waist, hip)` of the ground-truth base mesh with an
/// upstream measure target applied at the given weight.
fn ground_truth_girths(
    verts: &[[f64; 3]],
    indices: &[u32],
    deltas: &[(usize, [f64; 3])],
    weight: f64,
) -> Option<(f64, f64, f64)> {
    let mut morphed = verts.to_vec();
    for &(vid, d) in deltas {
        if let Some(p) = morphed.get_mut(vid) {
            p[0] += d[0] * weight;
            p[1] += d[1] * weight;
            p[2] += d[2] * weight;
        }
    }
    let m = CrossSectionMeasurer::new(morphed, indices);
    Some((
        m.chest_circumference()?,
        m.waist_circumference()?,
        m.hip_circumference()?,
    ))
}

/// The measurer must observe every MakeHuman `measure/` girth target when the
/// target is applied with correct vertex indices: several centimetres of
/// response on its own measurement (tape convention, right sign) and no
/// leakage into the other girths. This is the regression that pins the
/// arm-excluding bust band and the untrimmed band-extremum conventions.
#[test]
fn measure_targets_move_measured_girths_ground_truth() {
    let base_path = upstream_dir().join("3dobjs/base.obj");
    let Ok(src) = std::fs::read_to_string(&base_path) else {
        eprintln!("upstream base.obj absent — skipping ground-truth girth test");
        return;
    };
    let (verts, indices) = parse_base_obj_body_cm(&src);
    let base = ground_truth_girths(&verts, &indices, &[], 0.0).expect("base girths resolve");
    let (chest0, waist0, hip0) = base;
    // MakeHuman neutral base body: sanity-check the conventions land on
    // plausible adult girths before testing responses.
    assert!(
        (70.0..95.0).contains(&chest0),
        "base chest {chest0:.1} cm implausible"
    );
    assert!(
        (55.0..80.0).contains(&waist0),
        "base waist {waist0:.1} cm implausible"
    );
    assert!(
        (85.0..105.0).contains(&hip0),
        "base hip {hip0:.1} cm implausible"
    );
    assert!(
        waist0 < chest0 && chest0 < hip0 + 15.0,
        "girth ordering implausible: chest {chest0:.1} waist {waist0:.1} hip {hip0:.1}"
    );

    // (target file, measurement index 0=chest/1=waist/2=hip, minimum signed
    // response in cm at weight 1). Cross-talk on the other two girths must
    // stay within ±1.5 cm — bust targets legitimately brush the top of the
    // waist band, so underbust leakage is checked against a wider bound below.
    let cases: [(&str, usize, f64); 8] = [
        ("measure-bust-circ-incr", 0, 8.0),
        ("measure-bust-circ-decr", 0, -8.0),
        ("measure-underbust-circ-incr", 0, 4.0),
        ("measure-underbust-circ-decr", 0, -1.5),
        ("measure-waist-circ-incr", 1, 4.0),
        ("measure-waist-circ-decr", 1, -4.0),
        ("measure-hips-circ-incr", 2, 8.0),
        ("measure-hips-circ-decr", 2, -4.0),
    ];
    for (name, idx, min_response) in cases {
        let tp = upstream_dir().join(format!("targets/measure/{name}.target"));
        let Ok(tsrc) = std::fs::read_to_string(&tp) else {
            eprintln!("{name}: upstream target absent — skipping");
            continue;
        };
        let deltas = parse_target_cm(&tsrc);
        let g =
            ground_truth_girths(&verts, &indices, &deltas, 1.0).expect("morphed girths resolve");
        let girths = [g.0, g.1, g.2];
        let bases = [chest0, waist0, hip0];
        let response = girths[idx] - bases[idx];
        println!(
            "{name}: chest {:+.2} waist {:+.2} hip {:+.2} cm (own-girth response {response:+.2})",
            g.0 - chest0,
            g.1 - waist0,
            g.2 - hip0
        );
        if min_response >= 0.0 {
            assert!(
                response >= min_response,
                "{name}: response {response:+.2} cm below expected +{min_response:.1} cm"
            );
        } else {
            assert!(
                response <= min_response,
                "{name}: response {response:+.2} cm above expected {min_response:.1} cm"
            );
        }
        // Cross-talk guard: girths the target is not authored for must stay
        // put. The bust/underbust pair legitimately overlaps the waist band
        // top, so allow a looser bound there.
        for (j, (m, b)) in girths.iter().zip(bases.iter()).enumerate() {
            if j == idx {
                continue;
            }
            let leak = (m - b).abs();
            let bound = if name.contains("bust") && j == 1 {
                10.0
            } else {
                1.5
            };
            assert!(
                leak <= bound,
                "{name}: leaks {leak:.2} cm into girth #{j} (bound {bound:.1})"
            );
        }
    }
}

/// Pack-backed girth responses for the 8 `measure/` targets.
///
/// The v3 pack carries correctly re-indexed targets, so driving any
/// `measure/` girth target at weight 1 must move its own measured girth by
/// the ground-truth magnitude established in
/// `measure_targets_move_measured_girths_ground_truth` — the same target
/// applied with raw indices to the raw v-line mesh. A ±30 % band absorbs the
/// small band-convention differences between the raw body-group mesh and the
/// packed full mesh (plus quantisation). Cross-talk into unrelated girths
/// stays bounded, and resetting the weights restores the baseline readout.
#[test]
fn pack_measure_targets_girth_response_report() {
    let Some(bytes) = pack_bytes() else {
        eprintln!("core pack absent — skipping");
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    let base = eng.tailoring_summary_cm().expect("measure default body");
    println!(
        "pack default: chest {:.2} waist {:.2} hip {:.2} cm",
        base.chest_cm, base.waist_cm, base.hip_cm
    );
    // (target, girth index 0=chest/1=waist/2=hip, ground-truth response cm).
    // Ground truth: the upstream target applied with correct raw indices
    // (see the ground-truth test above). Tolerance ±30 %.
    let cases: [(&str, usize, f64); 8] = [
        ("measure/measure-bust-circ-incr", 0, 15.66),
        ("measure/measure-bust-circ-decr", 0, -17.86),
        ("measure/measure-underbust-circ-incr", 0, 8.35),
        ("measure/measure-underbust-circ-decr", 0, -2.3),
        ("measure/measure-waist-circ-incr", 1, 7.83),
        ("measure/measure-waist-circ-decr", 1, -8.75),
        ("measure/measure-hips-circ-incr", 2, 18.62),
        ("measure/measure-hips-circ-decr", 2, -9.49),
    ];
    const TOL: f64 = 0.30;
    for (t, idx, truth) in cases {
        eng.set_param(t, 1.0);
        let s = eng.tailoring_summary_cm().expect("measure morphed body");
        let deltas = [
            s.chest_cm - base.chest_cm,
            s.waist_cm - base.waist_cm,
            s.hip_cm - base.hip_cm,
        ];
        println!(
            "{t}: chest {:+.2} waist {:+.2} hip {:+.2} cm (ground truth {truth:+.2})",
            deltas[0], deltas[1], deltas[2]
        );
        for (n, v) in [
            ("chest", s.chest_cm),
            ("waist", s.waist_cm),
            ("hip", s.hip_cm),
        ] {
            assert!(
                v.is_finite() && v > 30.0 && v < 160.0,
                "{t}: {n} {v:.1} cm not plausible/finite"
            );
        }
        // Own-girth response: ground-truth magnitude, right sign.
        let (lo, hi) = if truth >= 0.0 {
            (truth * (1.0 - TOL), truth * (1.0 + TOL))
        } else {
            (truth * (1.0 + TOL), truth * (1.0 - TOL))
        };
        assert!(
            (lo..=hi).contains(&deltas[idx]),
            "{t}: own-girth response {:+.2} cm outside ground-truth band [{lo:+.2}, {hi:+.2}]",
            deltas[idx]
        );
        // Cross-talk guard: the waist and hip pairs must leave the other
        // girths in place. The bust/underbust pair legitimately overlaps the
        // top of the waist band (the underbust line *is* the waist band top),
        // so waist leakage is exempted for those targets only.
        for (j, d) in deltas.iter().enumerate() {
            if j == idx {
                continue;
            }
            if t.contains("bust") && j == 1 {
                continue;
            }
            assert!(
                d.abs() <= 1.5,
                "{t}: leaks {d:+.2} cm into girth #{j} (bound 1.5)"
            );
        }
        eng.set_param(t, 0.0);
    }
    // Weights back at zero: the readout must return to the baseline.
    let back = eng.tailoring_summary_cm().expect("measure reset body");
    assert!((back.chest_cm - base.chest_cm).abs() < 1e-6);
    assert!((back.waist_cm - base.waist_cm).abs() < 1e-6);
    assert!((back.hip_cm - base.hip_cm).abs() < 1e-6);
}

#[test]
#[ignore = "diagnostic: reachable girth envelope of the shipped pack under the new bands"]
fn probe_pack_girth_envelope() {
    let Some(bytes) = pack_bytes() else {
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    for w in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        for g in [0.0f32, 0.5, 1.0] {
            for m in [0.0f32, 0.5, 1.0] {
                eng.set_height(0.55);
                eng.set_weight(w);
                eng.set_muscle(m);
                eng.set_param("gender", g);
                let s = eng.tailoring_summary_cm().expect("measure");
                println!(
                    "w={w:.2} m={m:.2} g={g:.2}: h={:.1} chest={:.1} waist={:.1} hip={:.1}",
                    s.height_cm, s.chest_cm, s.waist_cm, s.hip_cm
                );
            }
        }
    }
}

#[test]
fn default_measurements_are_plausible_adult() {
    let Some(bytes) = pack_bytes() else {
        eprintln!("core pack absent — skipping");
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    let s = eng.tailoring_summary_cm().expect("measure default body");
    assert!(
        s.height_cm > 150.0 && s.height_cm < 185.0,
        "default stature {} cm out of plausible adult range",
        s.height_cm
    );
    assert!(
        s.weight_kg > 40.0 && s.weight_kg < 90.0,
        "default mass {} kg implausible (helper geometry not excluded?)",
        s.weight_kg
    );
    for (n, v) in [
        ("chest", s.chest_cm),
        ("waist", s.waist_cm),
        ("hip", s.hip_cm),
    ] {
        assert!(
            v > 50.0 && v < 130.0,
            "{n} circumference {v} cm implausible"
        );
    }
    // Tape ordering on the default androgynous body: waist is the narrowest
    // of the three girths.
    assert!(
        s.waist_cm < s.chest_cm && s.waist_cm < s.hip_cm,
        "waist {:.1} should be narrower than chest {:.1} and hip {:.1}",
        s.waist_cm,
        s.chest_cm,
        s.hip_cm
    );
}

#[test]
fn measurement_is_deterministic() {
    let Some(bytes) = pack_bytes() else {
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    eng.set_height(0.6);
    eng.set_weight(0.55);
    let a = eng.tailoring_summary_cm().expect("measure a");
    let b = eng.tailoring_summary_cm().expect("measure b");
    assert!((a.height_cm - b.height_cm).abs() < 1e-6);
    assert!((a.chest_cm - b.chest_cm).abs() < 1e-6);
    assert!((a.waist_cm - b.waist_cm).abs() < 1e-6);
    assert!((a.hip_cm - b.hip_cm).abs() < 1e-6);
}

#[test]
fn weight_scales_with_stature() {
    let Some(bytes) = pack_bytes() else {
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    eng.set_height(0.5);
    let short = eng.tailoring_summary_cm().expect("measure").weight_kg;
    eng.set_height(0.9);
    let tall = eng.tailoring_summary_cm().expect("measure").weight_kg;
    assert!(
        tall > short + 5.0,
        "mass should grow with a much taller body: {short} -> {tall} kg"
    );
}

#[test]
fn self_consistent_round_trip_recovers_measurements() {
    let Some(bytes) = pack_bytes() else {
        return;
    };
    // Measure a moderate body the pack can produce, then fit to those exact
    // measurements: the optimiser must recover a matching body.
    let mut src = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    src.set_height(0.6);
    src.set_weight(0.55);
    src.set_muscle(0.5);
    src.set_param("gender", 0.5);
    let s = src.tailoring_summary_cm().expect("measure source");

    let target = format!(
        r#"{{"height_cm":{:.1},"chest_cm":{:.1},"waist_cm":{:.1},"hip_cm":{:.1},"max_iterations":35}}"#,
        s.height_cm, s.chest_cm, s.waist_cm, s.hip_cm
    );
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    let report: serde_json::Value =
        serde_json::from_str(&eng.fit_to_measurements(&target).expect("fit")).expect("json");

    // Self-consistent targets are reachable by construction, so every
    // measurement recovers to within the benchmarked tolerance.
    for name in ["height", "chest", "waist", "hip"] {
        let (_t, _m, delta) = entry(&report, name).expect("measurement present");
        assert!(
            delta.abs() <= 2.5,
            "self-consistent {name} delta {delta:.2} cm exceeds 2.5 cm"
        );
    }
}

#[test]
fn brief_172_fit_reports_honest_deltas() {
    let Some(bytes) = pack_bytes() else {
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    let report: serde_json::Value = serde_json::from_str(
        &eng.fit_to_measurements(
            r#"{"height_cm":172,"chest_cm":96,"waist_cm":82,"hip_cm":98,"max_iterations":35}"#,
        )
        .expect("fit"),
    )
    .expect("json");
    println!("brief report: {report}");

    // Honest bounds from docs/bench/measurement-error.md against the v3
    // (index-remapped) pack: height is solved directly (millimetre-scale) and
    // the lever-gated `measure/` refinement closes every girth residual to
    // well under 1 cm on this brief (achieved: chest −0.26, waist +0.03,
    // hip −0.10 at 35 iterations). The bounds below are upper bounds on |Δ|
    // with headroom for cross-platform float drift.
    for (name, tol) in [
        ("height", 0.5),
        ("chest", 1.5),
        ("waist", 1.5),
        ("hip", 1.5),
    ] {
        let (_t, _m, delta) = entry(&report, name).expect("measurement present");
        assert!(
            delta.abs() <= tol,
            "brief-172 {name} delta {delta:.2} cm exceeds {tol} cm (see docs/bench/measurement-error.md)"
        );
    }

    // The fit report carries the refinement-stage measure-target weights.
    let mw = report
        .get("measure_weights")
        .expect("measure_weights present");
    assert!(mw.is_object(), "measure_weights must be a JSON object");
    for key in [
        "measure/measure-bust-circ-incr",
        "measure/measure-waist-circ-incr",
        "measure/measure-hips-circ-incr",
    ] {
        let w = mw.get(key).and_then(|v| v.as_f64());
        assert!(
            w.map(|w| (0.0..=1.0).contains(&w)).unwrap_or(false),
            "measure weight {key} = {w:?} must be a bounded [0,1] value"
        );
    }

    // `delta == measured - target` up to the report's display rounding — the
    // readout is re-measured geometry, never an echo of the request.
    let (target, measured, delta) = entry(&report, "chest").expect("chest present");
    assert!((target - 96.0).abs() < 1e-6);
    assert!(
        (delta - (measured - target)).abs() < 0.06,
        "delta {delta:.2} must equal measured {measured:.1} - target {target:.1} (re-measured, not echoed)"
    );

    // The engine is left set to the fitted body (re-measuring it reproduces
    // the reported chest, up to the report's 0.1 cm display rounding).
    let after = eng.tailoring_summary_cm().expect("measure fitted");
    assert!((after.chest_cm - measured).abs() < 0.1);
}

#[test]
fn partial_target_and_bad_input() {
    let Some(bytes) = pack_bytes() else {
        return;
    };
    let mut eng = WasmEngine::new_from_core_pack_bytes(&bytes).expect("load pack");
    // Height-only fit nails stature.
    let report: serde_json::Value = serde_json::from_str(
        &eng.fit_to_measurements(r#"{"height_cm":178}"#)
            .expect("fit"),
    )
    .expect("json");
    let (_t, _m, delta) = entry(&report, "height").expect("height present");
    assert!(delta.abs() <= 1.5, "height-only delta {delta:.2} cm");
    assert_eq!(report["results"].as_array().map(|a| a.len()), Some(1));

    // No usable target is an error, not a panic.
    assert!(eng.fit_to_measurements("{}").is_err());
    assert!(eng.fit_to_measurements("not json").is_err());
}
