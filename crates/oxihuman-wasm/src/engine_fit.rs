// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Precise body measurements and measurement-driven parameter fitting for
//! [`WasmEngine`].
//!
//! Two capabilities back the BodyLab error-readout demo:
//!
//! * [`WasmEngine::tailoring_summary_cm`] — the precise cross-section
//!   measurement of the *current* morphed body (height / chest / waist / hip in
//!   centimetres plus a mesh-volume mass in kilograms), replacing the old
//!   `bbox-width × π` heuristic.
//! * [`WasmEngine::fit_to_measurements`] — a measurement-driven search over
//!   the engine's macro parameters (`height` solved directly, then Nelder–Mead
//!   over `weight`, `muscle`, `gender`) whose objective is the precise
//!   re-measurement of the resulting mesh, so the returned deltas come from
//!   real geometry rather than echoing the requested targets. The engine is
//!   the correct forward model because a macro parameter composes its corner
//!   targets through weight curves; fitting in parameter space therefore
//!   always yields a self-consistent body.
//!
//! # Fit structure: height solve + girth Nelder–Mead + measure-target refinement
//!
//! Stature is monotone in the `height` slider and only weakly coupled to the
//! other macro params, so the fit solves it *directly* (a deterministic
//! bracketing scan plus step-halving refinement) instead of letting a joint
//! optimiser trade centimetres of height for centimetres of girth. The
//! remaining `[weight, muscle, gender]` sliders are then searched by
//! Nelder–Mead with height held fixed, and height is re-trimmed after each
//! girth pass (block-coordinate descent; two girth passes suffice because the
//! height↔girth cross-coupling of the corner targets is small).
//!
//! After the macro fit, a **refinement** stage (see `refine_measure_weights`)
//! locally drives the MakeHuman `targets/measure/` girth targets — bust /
//! underbust / waist / hips — by name (via a bounded coordinate descent over
//! their weights) minimising the *same* re-measured objective. These are the
//! tape-measure targets authored for exactly this local-adjustment job. The
//! refinement is **lever-gated**: for each girth dimension it first probes
//! whether the associated measure target actually moves the re-measured
//! circumference, and skips the dimension when it does not, leaving that
//! weight at `0`. The [`CrossSectionMeasurer`] observes all of these targets
//! (chest is read at the anatomical bust band with arm sections excluded per
//! slice, waist/hip at their natural bands — verified against ground-truth
//! application of the upstream targets in `tests/measurement_fit.rs`, and
//! against the shipped pack itself, whose targets are re-indexed into the
//! packed base-mesh vertex order at pack-build time).
//!
//! Per-evaluation measurement cost is kept low by caching the measurer's
//! positions-independent topology (body/helper boundary + body triangle
//! list) on the engine across cost evaluations — see
//! [`WasmEngine::build_measurer_cm`].

use anyhow::Result;
use oxihuman_morph::calibration::nelder_mead_minimize;
use oxihuman_morph::measurements::{CrossSectionMeasurer, MeasurerTopology, TailoringSummary};

use crate::engine_core::{WasmEngine, MODEL_UNIT_CM};

/// One target measurement requested by a fit.
#[derive(Clone, Copy)]
struct TargetMeasure {
    /// JSON/result key (`"height"`, `"chest"`, `"waist"`, `"hip"`).
    name: &'static str,
    /// Requested value in centimetres.
    target_cm: f64,
}

/// A girth-refinement modifier: one measured dimension driven by an
/// `incr`/`decr` pair (or pairs) of MakeHuman `measure/` targets.
///
/// The modifier value `w ∈ [-1, 1]` composes the pair by MakeHuman semantics:
/// `w ≥ 0` drives the `incr` target(s) at weight `w` (and the `decr` at `0`),
/// `w < 0` drives the `decr` target(s) at weight `-w` (and the `incr` at `0`).
/// The two are never both active. `chest` additionally recruits the
/// `underbust` pair so the whole bust region swells together.
struct MeasureModifier {
    /// Requested-measurement key this modifier serves.
    measurement: &'static str,
    /// Target names driven when the modifier is positive.
    incr: &'static [&'static str],
    /// Target names driven when the modifier is negative.
    decr: &'static [&'static str],
}

/// The girth modifiers, in optimiser order (`[chest, waist, hip]`). Names match
/// the pack target names produced by `pack-core` (category `"measure"`), so the
/// weights land on the pack-loaded targets keyed by name.
const MEASURE_MODIFIERS: [MeasureModifier; 3] = [
    MeasureModifier {
        measurement: "chest",
        incr: &[
            "measure/measure-bust-circ-incr",
            "measure/measure-underbust-circ-incr",
        ],
        decr: &[
            "measure/measure-bust-circ-decr",
            "measure/measure-underbust-circ-decr",
        ],
    },
    MeasureModifier {
        measurement: "waist",
        incr: &["measure/measure-waist-circ-incr"],
        decr: &["measure/measure-waist-circ-decr"],
    },
    MeasureModifier {
        measurement: "hip",
        incr: &["measure/measure-hips-circ-incr"],
        decr: &["measure/measure-hips-circ-decr"],
    },
];

/// Inclusive bound on a girth modifier value (MakeHuman measure sliders span
/// `[-1, 1]`; the per-target weight is further clamped to `[0, 1]` by the
/// engine when the incr/decr split is applied).
const MEASURE_BOUND: f64 = 1.0;

// The fit drives four normalised `[0, 1]` sliders, in optimiser-vector order:
// `[height, weight, muscle, gender]`. `gender` is the extra param `"gender"`
// (`0` = masculine, `1` = feminine); the others are first-class engine params.
// `age` is intentionally excluded and left at its current (age-floored) value.

impl WasmEngine {
    /// Build the current morphed geometry and return a precise cross-section
    /// measurer over it (vertices scaled from model decimetres to centimetres).
    ///
    /// This deliberately bypasses [`Self::build_mesh_prepared`]: the measurer
    /// needs only morphed positions and triangle indices, so it skips normal
    /// recomputation and the suit-flag pass, which roughly halves the per-fit
    /// evaluation cost. JSON-loaded targets are still applied so the measured
    /// body matches what the exporters produce.
    ///
    /// The positions-independent measurer topology (body/helper boundary +
    /// body triangle list) is derived once per base mesh and cached on the
    /// engine — morphs move vertices but never change topology, so the dozens
    /// of re-measurements a fit performs share a single derivation instead of
    /// re-deriving it per cost evaluation (see `measurer_topology` on
    /// [`WasmEngine`]; the cache is dropped whenever the base mesh is
    /// replaced).
    ///
    /// Returns `None` only for a degenerate (empty) mesh.
    pub fn build_measurer_cm(&mut self) -> Option<CrossSectionMeasurer> {
        let morph = self.engine.build_mesh_incremental();
        let mut positions = morph.positions;
        self.apply_json_targets(&mut positions);
        if positions.is_empty() || morph.indices.len() < 3 {
            return None;
        }
        let u = MODEL_UNIT_CM as f64;
        let verts: Vec<[f64; 3]> = positions
            .iter()
            .map(|p| [p[0] as f64 * u, p[1] as f64 * u, p[2] as f64 * u])
            .collect();
        let topo = match &self.measurer_topology {
            Some(t) if t.matches(verts.len(), morph.indices.len()) => t.clone(),
            _ => {
                let t = MeasurerTopology::derive(&verts, &morph.indices);
                self.measurer_topology = Some(t.clone());
                t
            }
        };
        Some(CrossSectionMeasurer::with_topology(verts, &topo))
    }

    /// Build the current morphed mesh and measure it precisely.
    ///
    /// Returns `None` only for a degenerate (empty) mesh. Every field is in
    /// centimetres / kilograms.
    pub fn tailoring_summary_cm(&mut self) -> Option<TailoringSummary> {
        self.build_measurer_cm().map(|m| m.tailoring_summary())
    }

    /// Apply an optimiser vector to the engine (clamped to `[0, 1]`); the age
    /// floor is enforced by the shared `commit_params` chokepoint.
    fn apply_fit_params(&mut self, x: &[f64]) {
        let c = |v: f64| v.clamp(0.0, 1.0) as f32;
        self.set_height(c(x[0]));
        self.set_weight(c(x[1]));
        self.set_muscle(c(x[2]));
        self.set_param("gender", c(x[3]));
    }

    /// Deterministically solve the `height` slider so the re-measured stature
    /// matches `target_cm`, holding the other params (`x[1..]`) fixed.
    ///
    /// Stature is monotone (non-decreasing) in the slider, so a plain
    /// bisection converges in ~12 cheap evaluations (stature only, no
    /// cross-section slicing); a flat region simply collapses the bracket to
    /// its boundary. When `hint` carries a previous solution only a local
    /// step-halving refinement runs (the girth params shift stature by well
    /// under a bracket cell between passes).
    fn solve_height_slider(&mut self, target_cm: f64, x: &[f64; 4], hint: Option<f64>) -> f64 {
        let mut stature_at = |h: f64| -> f64 {
            self.apply_fit_params(&[h, x[1], x[2], x[3]]);
            match self.build_measurer_cm() {
                Some(m) => m.stature_cm(),
                None => f64::NAN,
            }
        };
        if let Some(h0) = hint {
            // Local refinement around the previous solution.
            let mut best_h = h0.clamp(0.0, 1.0);
            let mut best_e = (stature_at(best_h) - target_cm).abs();
            let mut step = 0.05;
            while step > 0.0015 {
                for cand in [best_h - step, best_h + step] {
                    let h = cand.clamp(0.0, 1.0);
                    let e = (stature_at(h) - target_cm).abs();
                    if e < best_e {
                        best_e = e;
                        best_h = h;
                    }
                }
                step *= 0.5;
            }
            return best_h;
        }
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        let s_lo = stature_at(lo);
        let s_hi = stature_at(hi);
        if !s_lo.is_finite() || !s_hi.is_finite() {
            return 0.5;
        }
        if target_cm <= s_lo {
            return lo;
        }
        if target_cm >= s_hi {
            return hi;
        }
        for _ in 0..12 {
            let mid = 0.5 * (lo + hi);
            let s = stature_at(mid);
            if !s.is_finite() {
                break;
            }
            if s < target_cm {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// Select the measured value for a target name from a summary.
    fn summary_value(summary: &TailoringSummary, name: &str) -> f64 {
        match name {
            "height" => summary.height_cm,
            "chest" => summary.chest_cm,
            "waist" => summary.waist_cm,
            "hip" => summary.hip_cm,
            _ => f64::NAN,
        }
    }

    /// Objective for [`Self::fit_to_measurements`]: apply `x` to the engine,
    /// re-measure, and return the sum of squared centimetre residuals over the
    /// requested measurements plus a mild `[0, 1]^4` box penalty. Only the
    /// circumferences flagged in `need = (chest, waist, hip)` are computed.
    fn fit_cost(&mut self, x: &[f64], targets: &[TargetMeasure], need: (bool, bool, bool)) -> f64 {
        let mut penalty = 0.0;
        for &v in x.iter() {
            if v < 0.0 {
                penalty += (v * 100.0).powi(2);
            } else if v > 1.0 {
                penalty += ((v - 1.0) * 100.0).powi(2);
            }
        }
        self.apply_fit_params(x);
        let measurer = match self.build_measurer_cm() {
            Some(m) => m,
            None => return 1e12 + penalty,
        };
        let height = measurer.stature_cm();
        let (need_chest, need_waist, need_hip) = need;
        let chest = if need_chest {
            measurer.chest_circumference().unwrap_or(height * 0.55)
        } else {
            f64::NAN
        };
        let waist = if need_waist {
            measurer.waist_circumference().unwrap_or(height * 0.48)
        } else {
            f64::NAN
        };
        let hip = if need_hip {
            measurer.hip_circumference().unwrap_or(height * 0.58)
        } else {
            f64::NAN
        };
        let mut cost = penalty;
        for t in targets {
            let m = match t.name {
                "height" => height,
                "chest" => chest,
                "waist" => waist,
                "hip" => hip,
                _ => f64::NAN,
            };
            if m.is_finite() {
                let d = m - t.target_cm;
                cost += d * d;
            }
        }
        cost
    }

    /// Apply a girth modifier value to its measure-target weights (leaving the
    /// macro params untouched). `w ≥ 0` drives the `incr` targets; `w < 0`
    /// drives the `decr` targets; the counterpart is forced to `0`.
    fn set_measure_modifier(&mut self, m: &MeasureModifier, w: f64) {
        let (w_incr, w_decr) = if w >= 0.0 {
            (w as f32, 0.0f32)
        } else {
            (0.0f32, (-w) as f32)
        };
        for t in m.incr {
            self.set_param(t, w_incr);
        }
        for t in m.decr {
            self.set_param(t, w_decr);
        }
    }

    /// Sum-of-squared-residual objective for the refinement stage: apply the
    /// three girth modifiers `mods` (macro params stay at the committed macro
    /// fit), re-measure, and score against the requested targets. Only the
    /// circumferences in `need` are computed. Shares the measurement policy of
    /// `Self::fit_cost` so the two stages optimise a consistent objective.
    fn measure_refine_cost(
        &mut self,
        mods: &[f64; 3],
        targets: &[TargetMeasure],
        need: (bool, bool, bool),
    ) -> f64 {
        for (i, m) in MEASURE_MODIFIERS.iter().enumerate() {
            self.set_measure_modifier(m, mods[i]);
        }
        let measurer = match self.build_measurer_cm() {
            Some(m) => m,
            None => return 1e12,
        };
        let height = measurer.stature_cm();
        let (need_chest, need_waist, need_hip) = need;
        let chest = if need_chest {
            measurer.chest_circumference().unwrap_or(height * 0.55)
        } else {
            f64::NAN
        };
        let waist = if need_waist {
            measurer.waist_circumference().unwrap_or(height * 0.48)
        } else {
            f64::NAN
        };
        let hip = if need_hip {
            measurer.hip_circumference().unwrap_or(height * 0.58)
        } else {
            f64::NAN
        };
        let mut cost = 0.0;
        for t in targets {
            let m = match t.name {
                "height" => height,
                "chest" => chest,
                "waist" => waist,
                "hip" => hip,
                _ => f64::NAN,
            };
            if m.is_finite() {
                let d = m - t.target_cm;
                cost += d * d;
            }
        }
        cost
    }

    /// Refinement stage: after the macro fit, locally drive the `measure/`
    /// girth targets by name to shrink any remaining circumference residual,
    /// returning the fitted modifier values `[chest, waist, hip]`.
    ///
    /// Deterministic bounded coordinate descent. Each dimension is first
    /// **lever-gated**: it is refined only when its measurement was requested,
    /// its post-macro residual exceeds `REFINE_RESIDUAL_CM`, *and* a
    /// ±probe shows the measure target actually moves the re-measured objective
    /// by more than `LEVER_EPS`. Dimensions with no observable lever (the
    /// current measurer cannot see the local girth change — see the module
    /// docs) are left at `0`, so on the shipped body this is a cheap, honest
    /// no-op that neither improves nor corrupts the fit. On exit the engine is
    /// left with the winning modifier weights committed.
    fn refine_measure_weights(
        &mut self,
        targets: &[TargetMeasure],
        need: (bool, bool, bool),
    ) -> [f64; 3] {
        /// Post-macro residual (cm) below which a dimension is already solved.
        const REFINE_RESIDUAL_CM: f64 = 0.5;
        /// Objective change (cm²) under which a modifier has no measurable
        /// lever on the readout and is skipped.
        const LEVER_EPS: f64 = 0.05;
        /// Coarse probe / initial line-search step in modifier units.
        const STEP0: f64 = 0.5;
        /// Hard cap on objective evaluations (keeps the stage well under the
        /// interactive time budget regardless of convergence; per-evaluation
        /// cost is low because the measurer topology is cached).
        const MAX_EVALS: u32 = 72;
        /// Line-search termination step (modifier units). The strongest
        /// lever (bust ≈ 16 cm at weight 1) then resolves ≈ 0.25 cm.
        const STEP_MIN: f64 = 0.015;

        let mut mods = [0.0f64; 3];
        let Some(base) = self.build_measurer_cm().map(|m| m.tailoring_summary()) else {
            return mods;
        };
        let mut evals: u32 = 0;
        let mut summary = base;

        for pass in 0..3 {
            // Re-measure the incumbent at the start of every later pass so
            // the residual gate below sees post-refinement girths (crosstalk
            // from the previous pass included), not the stale macro-fit ones.
            if pass > 0 {
                for (i, m) in MEASURE_MODIFIERS.iter().enumerate() {
                    self.set_measure_modifier(m, mods[i]);
                }
                let Some(s) = self.build_measurer_cm().map(|m| m.tailoring_summary()) else {
                    break;
                };
                evals += 1;
                summary = s;
            }
            let mut improved = false;
            for i in 0..MEASURE_MODIFIERS.len() {
                let name = MEASURE_MODIFIERS[i].measurement;
                let Some(t) = targets.iter().find(|t| t.name == name) else {
                    continue;
                };
                // Residual gate: a near-solved dimension is not worth a line
                // walk (the readout is displayed at 0.1 cm resolution).
                let cur = Self::summary_value(&summary, name);
                if (cur - t.target_cm).abs() < REFINE_RESIDUAL_CM {
                    continue;
                }
                if evals + 3 > MAX_EVALS {
                    break;
                }
                let c0 = self.measure_refine_cost(&mods, targets, need);
                let vp = (mods[i] + STEP0).min(MEASURE_BOUND);
                let cp = {
                    let mut m = mods;
                    m[i] = vp;
                    self.measure_refine_cost(&m, targets, need)
                };
                let vn = (mods[i] - STEP0).max(-MEASURE_BOUND);
                let cn = {
                    let mut m = mods;
                    m[i] = vn;
                    self.measure_refine_cost(&m, targets, need)
                };
                evals += 3;
                // No observable lever in either direction: leave at 0.
                if (cp - c0).abs() < LEVER_EPS && (cn - c0).abs() < LEVER_EPS {
                    continue;
                }
                // Descend in the better direction, shrinking the step on
                // overshoot; bounded and deterministic. Seed the incumbent
                // with the *best of the three probes* — value and cost
                // together — so a probe that already beats the centre is
                // accepted before the walk continues past it.
                let dir = if cp <= cn { 1.0 } else { -1.0 };
                let (mut best_v, mut best_c) = if c0 <= cp && c0 <= cn {
                    (mods[i], c0)
                } else if cp <= cn {
                    (vp, cp)
                } else {
                    (vn, cn)
                };
                let mut v = best_v;
                let mut step = STEP0;
                while evals < MAX_EVALS {
                    let nv = (v + dir * step).clamp(-MEASURE_BOUND, MEASURE_BOUND);
                    if (nv - v).abs() < 1e-6 {
                        break;
                    }
                    let c = {
                        let mut m = mods;
                        m[i] = nv;
                        self.measure_refine_cost(&m, targets, need)
                    };
                    evals += 1;
                    if c + 1e-9 < best_c {
                        best_c = c;
                        best_v = nv;
                        v = nv;
                    } else {
                        step *= 0.5;
                        if step < STEP_MIN {
                            break;
                        }
                    }
                }
                if (best_v - mods[i]).abs() > 1e-9 {
                    mods[i] = best_v;
                    improved = true;
                }
            }
            if !improved {
                break;
            }
        }

        // Commit the winning modifiers so the final re-measurement + engine
        // state reflect them.
        for (i, m) in MEASURE_MODIFIERS.iter().enumerate() {
            self.set_measure_modifier(m, mods[i]);
        }
        mods
    }

    /// Serialise the applied measure-target weights (only the driven targets)
    /// implied by the modifier values `mods` into a JSON object literal.
    fn measure_weights_json(mods: &[f64; 3]) -> String {
        let mut entries: Vec<String> = Vec::new();
        for (i, m) in MEASURE_MODIFIERS.iter().enumerate() {
            let w = mods[i];
            let (w_incr, w_decr) = if w >= 0.0 { (w, 0.0) } else { (0.0, -w) };
            for t in m.incr {
                entries.push(format!("\"{t}\":{w_incr:.4}"));
            }
            for t in m.decr {
                entries.push(format!("\"{t}\":{w_decr:.4}"));
            }
        }
        format!("{{{}}}", entries.join(","))
    }

    /// Fit the engine's macro parameters so the re-measured mesh matches the
    /// requested measurements, returning a JSON report.
    ///
    /// Input JSON accepts any subset of
    /// `{"height_cm":…, "chest_cm":…, "waist_cm":…, "hip_cm":…}` (extra keys
    /// are ignored). Optional `{"max_iterations":n}` overrides the iteration
    /// cap.
    ///
    /// Output JSON:
    /// ```json
    /// {
    ///   "params": {"height":0.55,"weight":0.5,"muscle":0.5,"gender":0.5,"age":0.5},
    ///   "results": [
    ///     {"name":"height","target_cm":172.0,"measured_cm":171.8,"delta_cm":0.2},
    ///     ...
    ///   ],
    ///   "measure_weights": {"measure/measure-bust-circ-incr":0.0, ...},
    ///   "iterations": 47,
    ///   "converged": true
    /// }
    /// ```
    ///
    /// `measure_weights` reports the weight the refinement stage settled on for
    /// each driven `measure/` girth target (see
    /// `Self::refine_measure_weights`); all-zero means the refinement found no
    /// observable lever on the current measurer and left the macro fit as the
    /// final answer.
    ///
    /// The optimiser is deterministic (fixed simplex origin and step, and a
    /// deterministic refinement descent), so the same input always yields the
    /// same fit. On success the engine is left set to the fitted parameters
    /// *and* refined measure weights. `delta_cm = measured_cm − target_cm` is
    /// computed from a final precise re-measurement of the fully-refined mesh.
    pub fn fit_to_measurements(&mut self, options_json: &str) -> Result<String> {
        let opts: serde_json::Value = serde_json::from_str(options_json)
            .map_err(|e| anyhow::anyhow!("invalid fit options JSON: {e}"))?;

        // Collect requested targets in a stable order.
        let candidates: [(&str, &str); 4] = [
            ("height", "height_cm"),
            ("chest", "chest_cm"),
            ("waist", "waist_cm"),
            ("hip", "hip_cm"),
        ];
        let mut targets: Vec<TargetMeasure> = Vec::new();
        for (name, key) in candidates {
            if let Some(v) = opts.get(key).and_then(|v| v.as_f64()) {
                if v.is_finite() && v > 0.0 {
                    targets.push(TargetMeasure { name, target_cm: v });
                }
            }
        }
        if targets.is_empty() {
            return Err(anyhow::anyhow!(
                "no fit targets supplied; expected any of height_cm / chest_cm / waist_cm / hip_cm"
            ));
        }

        let max_iterations = opts
            .get("max_iterations")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(45)
            .clamp(10, 400);

        // Only measure the circumferences that are actually requested — a
        // height-only fit then never pays for cross-section slicing.
        let need_chest = targets.iter().any(|t| t.name == "chest");
        let need_waist = targets.iter().any(|t| t.name == "waist");
        let need_hip = targets.iter().any(|t| t.name == "hip");
        let height_target = targets
            .iter()
            .find(|t| t.name == "height")
            .map(|t| t.target_cm);

        // Stage 1 — solve height directly. Stature is monotone in the height
        // slider, so a joint optimiser could only *lose* height accuracy by
        // trading it against girth residuals; solving it out of band removes
        // that failure mode entirely.
        let need = (need_chest, need_waist, need_hip);
        let any_girth = need_chest || need_waist || need_hip;
        let mut best = [self.params.height as f64, 0.5, 0.5, 0.5];
        if let Some(h_target) = height_target {
            best[0] = self.solve_height_slider(h_target, &best, None);
        }

        // Stage 2 — Nelder–Mead over [weight, muscle, gender] with height held
        // fixed, in two phases (coarse basin search + small-simplex polish past
        // the mild non-smoothness of the band-extremum measurement), re-trimming
        // height between phases because the corner targets couple girth params
        // to stature slightly. The objective closure is re-created per phase
        // because it uniquely borrows `self`.
        let mut iterations = 0usize;
        let mut converged = true;
        if any_girth {
            let (coarse, it1, _c1) = {
                let h = best[0];
                let obj = |g: &[f64]| self.fit_cost(&[h, g[0], g[1], g[2]], &targets, need);
                nelder_mead_minimize(
                    obj,
                    &[best[1], best[2], best[3]],
                    0.14,
                    max_iterations,
                    1e-4,
                )
            };
            best[1] = coarse[0];
            best[2] = coarse[1];
            best[3] = coarse[2];
            if let Some(h_target) = height_target {
                best[0] = self.solve_height_slider(h_target, &best, Some(best[0]));
            }
            let (polish, it2, c2) = {
                let h = best[0];
                let obj = |g: &[f64]| self.fit_cost(&[h, g[0], g[1], g[2]], &targets, need);
                nelder_mead_minimize(obj, &coarse, 0.05, max_iterations / 2, 1e-5)
            };
            best[1] = polish[0];
            best[2] = polish[1];
            best[3] = polish[2];
            if let Some(h_target) = height_target {
                // Final height re-trim, guarded by the *total* cost: on rare
                // pathological bodies the band measurement bifurcates between
                // adjacent height-slider values, so a blind re-trim could trade
                // a millimetre of stature for tens of centimetres of girth.
                // Keep the re-trimmed height only when it does not lose overall.
                let h_re = self.solve_height_slider(h_target, &best, Some(best[0]));
                let c_old = self.fit_cost(&[best[0], best[1], best[2], best[3]], &targets, need);
                let c_new = self.fit_cost(&[h_re, best[1], best[2], best[3]], &targets, need);
                if c_new <= c_old {
                    best[0] = h_re;
                }
            }
            iterations = it1 + it2;
            converged = c2;
        }

        // Commit the fitted macro parameters, then run the girth-refinement
        // stage (drives the `measure/` targets by name; a no-op on dimensions
        // with no observable lever — see the module docs).
        self.apply_fit_params(&best);
        let measure_mods = self.refine_measure_weights(&targets, need);
        let measure_weights_json = Self::measure_weights_json(&measure_mods);

        // Final honest re-measurement of the fully-refined body.
        let summary = self
            .tailoring_summary_cm()
            .ok_or_else(|| anyhow::anyhow!("fitted mesh could not be measured"))?;

        let clamp01 = |v: f64| v.clamp(0.0, 1.0);
        let params_json = format!(
            concat!(
                "{{",
                "\"height\":{:.4},",
                "\"weight\":{:.4},",
                "\"muscle\":{:.4},",
                "\"gender\":{:.4},",
                "\"age\":{:.4}",
                "}}"
            ),
            clamp01(best[0]),
            clamp01(best[1]),
            clamp01(best[2]),
            clamp01(best[3]),
            self.params.age as f64,
        );

        let mut results = Vec::with_capacity(targets.len());
        for t in &targets {
            let measured = Self::summary_value(&summary, t.name);
            results.push(format!(
                "{{\"name\":\"{}\",\"target_cm\":{:.1},\"measured_cm\":{:.1},\"delta_cm\":{:.2}}}",
                t.name,
                t.target_cm,
                measured,
                measured - t.target_cm,
            ));
        }

        Ok(format!(
            "{{\"params\":{},\"results\":[{}],\"measure_weights\":{},\"iterations\":{},\"converged\":{}}}",
            params_json,
            results.join(","),
            measure_weights_json,
            iterations,
            converged,
        ))
    }
}
