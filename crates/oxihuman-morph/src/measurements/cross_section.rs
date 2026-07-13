// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Robust cross-section tailoring measurer — child module of
//! [`measurements`](super), re-exported there.

use std::collections::HashMap;

// The `BodyMeasurements` slicer (parent module) chains raw intersection
// segments, which is exact for clean closed meshes but fragile on the real
// MakeHuman base mesh:
// that mesh bundles helper geometry (a helper-tights shell and ~170 per-joint
// helper cubes), is split into many surface patches at UV seams, and is posed
// with the arms out. Naively summing every cross-section loop perimeter then
// yields metres-wide "circumferences".
//
// `CrossSectionMeasurer` is the production path used by the WASM measurement
// and fit APIs. It is robust to all three problems:
//   * Helper geometry is dropped by measuring only the *body* vertex prefix
//     (see [`body_vertex_boundary`]).
//   * Arms/legs/stray patches are rejected per slice by keeping only the
//     cross-section cluster whose convex hull encloses the body's central
//     vertical axis — the torso is the one section that wraps the axis.
//   * A tape measurement bridges surface concavities, so each torso section is
//     summarised by its convex-hull perimeter (Andrew's monotone chain).
// A per-slice Y-bucket index keeps each slice close to O(section triangles)
// instead of O(all triangles) so a full fit stays interactive.

/// Compact set of tailoring measurements (all lengths in centimetres, mass in
/// kilograms) produced by [`CrossSectionMeasurer::tailoring_summary`].
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TailoringSummary {
    /// Standing height (stature) in centimetres.
    pub height_cm: f64,
    /// Chest / bust circumference in centimetres.
    pub chest_cm: f64,
    /// Waist circumference in centimetres.
    pub waist_cm: f64,
    /// Hip circumference in centimetres.
    pub hip_cm: f64,
    /// Body mass in kilograms (mesh-volume × human mean density).
    pub weight_kg: f64,
}

/// Human mean whole-body density in kg per litre (≈ kg/dm³).
///
/// Whole-body density sits just above water (fat ≈ 0.9, lean tissue ≈ 1.1);
/// 1.01 kg/L is the standard reference used for volume→mass estimates.
pub const HUMAN_DENSITY_KG_PER_L: f64 = 1.01;

/// Derive the body/helper vertex boundary `K` for a MakeHuman-style base mesh:
/// the body skin occupies the contiguous vertex prefix `[0, K)` and any helper
/// geometry (tights shell, joint cubes) the disjoint suffix `[K, N)`.
///
/// The two blocks never share a triangle, so `K` is a *clean cut*: no triangle
/// straddles it. Several clean cuts exist (the body itself is many UV-seam
/// patches), so we pick the first clean cut whose vertex prefix already spans
/// nearly the full stature (`≥ 0.93 ·` full Y-extent) and at least half the
/// vertices — i.e. the point at which the whole standing body is present but
/// the interior helpers are not yet. Returns `positions.len()` (no split) when
/// no such cut exists, so a clean single-object mesh measures in full.
pub fn body_vertex_boundary(positions: &[[f64; 3]], triangles: &[[usize; 3]]) -> usize {
    let n = positions.len();
    if n == 0 {
        return 0;
    }
    // straddle[k] = (# triangles started but not closed at cut k); a cut k is
    // clean when straddle == 0. Built with a +1/-1 difference array.
    let mut diff = vec![0i32; n + 2];
    for t in triangles {
        let mn = t[0].min(t[1]).min(t[2]);
        let mx = t[0].max(t[1]).max(t[2]);
        diff[mn + 1] += 1;
        if mx < n {
            diff[mx + 1] -= 1;
        }
    }
    let full_ylo = positions.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
    let full_yhi = positions.iter().map(|p| p[1]).fold(f64::MIN, f64::max);
    let full_h = full_yhi - full_ylo;
    if full_h <= 0.0 {
        return n;
    }
    let mut running = 0i32;
    let mut ylo = f64::MAX;
    let mut yhi = f64::MIN;
    for k in 1..n {
        ylo = ylo.min(positions[k - 1][1]);
        yhi = yhi.max(positions[k - 1][1]);
        running += diff[k];
        if running == 0 && (yhi - ylo) >= 0.93 * full_h && (k as f64) >= 0.5 * n as f64 {
            return k;
        }
    }
    n
}

/// One torso cross-section chosen from a horizontal slice: its convex-hull
/// perimeter and X width, plus whether the hull passed the central-axis
/// containment test (`false` means the largest-cluster fallback was used).
#[derive(Clone, Copy)]
struct TorsoSection {
    perimeter: f64,
    x_extent: f64,
    on_axis: bool,
}

/// The positions-independent part of a [`CrossSectionMeasurer`]: the
/// body/helper vertex boundary and the body-only triangle list.
///
/// Morphing moves vertices but never changes the triangle topology or the
/// body/helper split, so a fit loop that re-measures the same mesh dozens of
/// times per solve can derive this once ([`MeasurerTopology::derive`]) and
/// rebuild only the position-dependent state per evaluation
/// ([`CrossSectionMeasurer::with_topology`]). Cloning is cheap — the triangle
/// list is behind an [`Arc`](std::sync::Arc).
#[derive(Clone)]
pub struct MeasurerTopology {
    /// Body/helper vertex boundary (see [`body_vertex_boundary`]).
    boundary: usize,
    /// Body-only triangles (helper geometry excluded).
    body_tris: std::sync::Arc<Vec<[usize; 3]>>,
    /// Vertex count of the mesh the topology was derived from.
    n_verts: usize,
    /// Flat index count of the mesh the topology was derived from.
    n_indices: usize,
}

impl MeasurerTopology {
    /// Derive the topology (body boundary + body triangle list) from
    /// centimetre vertices and a flat triangle index list.
    pub fn derive(verts_cm: &[[f64; 3]], indices: &[u32]) -> Self {
        let tris: Vec<[usize; 3]> = indices
            .chunks_exact(3)
            .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
            .collect();
        let boundary = body_vertex_boundary(verts_cm, &tris);
        let body_tris: Vec<[usize; 3]> = tris
            .into_iter()
            .filter(|t| t[0] < boundary && t[1] < boundary && t[2] < boundary)
            .collect();
        Self {
            boundary,
            body_tris: std::sync::Arc::new(body_tris),
            n_verts: verts_cm.len(),
            n_indices: indices.len(),
        }
    }

    /// Whether this topology was derived from a mesh with the given vertex
    /// and flat-index counts (the cheap staleness guard a caching layer
    /// checks before reuse; callers must additionally invalidate on any
    /// base-mesh replacement).
    #[inline]
    pub fn matches(&self, n_verts: usize, n_indices: usize) -> bool {
        self.n_verts == n_verts && self.n_indices == n_indices
    }
}

/// Robust cross-section body measurer over a triangle mesh in centimetres
/// (Y-up, X left-right, Z front-back).
pub struct CrossSectionMeasurer {
    verts: Vec<[f64; 3]>,
    /// Body-only triangles (helper geometry excluded); shared with the
    /// [`MeasurerTopology`] it was built from.
    body_tris: std::sync::Arc<Vec<[usize; 3]>>,
    y_min: f64,
    y_max: f64,
    z_axis: f64,
    /// Y-bucket triangle index: `buckets[b]` lists body triangles whose Y-range
    /// overlaps bucket `b`.
    buckets: Vec<Vec<u32>>,
    bucket_h: f64,
}

impl CrossSectionMeasurer {
    /// Number of Y buckets used for the slice acceleration structure.
    const N_BUCKETS: usize = 160;

    /// Build a measurer from centimetre vertices and a flat triangle index list.
    ///
    /// Helper geometry is excluded via [`body_vertex_boundary`]; the remaining
    /// body triangles are bucketed by Y for fast horizontal slicing. Derives
    /// the topology from scratch — hot loops that re-measure the same mesh
    /// under different morphs should derive a [`MeasurerTopology`] once and
    /// call [`Self::with_topology`] instead.
    pub fn new(verts_cm: Vec<[f64; 3]>, indices: &[u32]) -> Self {
        let topo = MeasurerTopology::derive(&verts_cm, indices);
        Self::with_topology(verts_cm, &topo)
    }

    /// Build a measurer from centimetre vertices and a pre-derived
    /// [`MeasurerTopology`], recomputing only the position-dependent state
    /// (Y extents, central axis, slice buckets).
    pub fn with_topology(verts_cm: Vec<[f64; 3]>, topo: &MeasurerTopology) -> Self {
        let boundary = topo.boundary.min(verts_cm.len());
        let body_tris = std::sync::Arc::clone(&topo.body_tris);

        let (mut y_min, mut y_max) = (f64::MAX, f64::MIN);
        let mut z_sum = 0.0;
        let mut z_cnt = 0usize;
        for v in verts_cm.iter().take(boundary) {
            y_min = y_min.min(v[1]);
            y_max = y_max.max(v[1]);
            z_sum += v[2];
            z_cnt += 1;
        }
        if z_cnt == 0 {
            y_min = 0.0;
            y_max = 0.0;
        }
        let z_axis = if z_cnt > 0 { z_sum / z_cnt as f64 } else { 0.0 };

        let span = (y_max - y_min).max(1e-9);
        let bucket_h = span / Self::N_BUCKETS as f64;
        let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); Self::N_BUCKETS];
        for (ti, t) in body_tris.iter().enumerate() {
            let ty_lo = verts_cm[t[0]][1]
                .min(verts_cm[t[1]][1])
                .min(verts_cm[t[2]][1]);
            let ty_hi = verts_cm[t[0]][1]
                .max(verts_cm[t[1]][1])
                .max(verts_cm[t[2]][1]);
            let b0 = (((ty_lo - y_min) / bucket_h).floor() as isize)
                .clamp(0, Self::N_BUCKETS as isize - 1) as usize;
            let b1 = (((ty_hi - y_min) / bucket_h).floor() as isize)
                .clamp(0, Self::N_BUCKETS as isize - 1) as usize;
            for b in buckets.iter_mut().take(b1 + 1).skip(b0) {
                b.push(ti as u32);
            }
        }

        Self {
            verts: verts_cm,
            body_tris,
            y_min,
            y_max,
            z_axis,
            buckets,
            bucket_h,
        }
    }

    /// Standing height (stature) in centimetres = body Y extent.
    pub fn stature_cm(&self) -> f64 {
        self.y_max - self.y_min
    }

    /// Collect the (x, z) intersection points of the horizontal plane at height
    /// `y` with the body triangles overlapping `y`'s bucket.
    fn slice_points(&self, y: f64) -> Vec<[f64; 2]> {
        if !(self.y_min..=self.y_max).contains(&y) {
            return Vec::new();
        }
        let b = (((y - self.y_min) / self.bucket_h).floor() as isize)
            .clamp(0, Self::N_BUCKETS as isize - 1) as usize;
        let bucket = &self.buckets[b];
        let mut out = Vec::with_capacity(bucket.len());
        for &ti in bucket {
            let t = self.body_tris[ti as usize];
            let p = [self.verts[t[0]], self.verts[t[1]], self.verts[t[2]]];
            let d = [p[0][1] - y, p[1][1] - y, p[2][1] - y];
            for e in 0..3 {
                let a = p[e];
                let bb = p[(e + 1) % 3];
                let (da, db) = (d[e], d[(e + 1) % 3]);
                if (da > 0.0 && db <= 0.0) || (da < 0.0 && db >= 0.0) {
                    let tt = da / (da - db);
                    out.push([a[0] + (bb[0] - a[0]) * tt, a[2] + (bb[2] - a[2]) * tt]);
                }
            }
        }
        out
    }

    /// Torso circumference (cm) at height `y`: convex-hull perimeter of the
    /// cross-section cluster whose hull encloses the central axis `(0, z_axis)`.
    /// Returns `None` when the slice has no substantial section.
    pub fn torso_circumference_at(&self, y: f64) -> Option<f64> {
        self.torso_section_at(y).map(|s| s.perimeter)
    }

    /// Torso cross-section summary at height `y`: the cluster whose convex
    /// hull encloses the central axis `(0, z_axis)` (falling back to the
    /// biggest substantial cluster when none does), with the hull's perimeter
    /// and X width plus whether the axis test succeeded.
    fn torso_section_at(&self, y: f64) -> Option<TorsoSection> {
        let pts = self.slice_points(y);
        if pts.len() < 8 {
            return None;
        }
        let clusters = cluster_points_2d(&pts, 2.5);
        // A torso section must be a *dominant* slice component, not a stray
        // sliver: at least 15% of the slice's points (and ≥ 24 absolute). This
        // rejects the tiny spurious central clusters that a bare min/max search
        // would otherwise latch onto at extreme morphs.
        let min_pts = (pts.len() / 6).max(24);
        let mut axis_best: Option<(usize, TorsoSection)> = None;
        let mut fallback: Option<(usize, TorsoSection)> = None;
        for c in &clusters {
            if c.len() < min_pts {
                continue;
            }
            let hull = convex_hull_2d_pts(c);
            if hull.len() < 3 {
                continue;
            }
            let on_axis = point_in_polygon_2d(&hull, [0.0, self.z_axis]);
            let sec = TorsoSection {
                perimeter: polygon_perimeter_2d(&hull),
                x_extent: hull_x_extent(&hull),
                on_axis,
            };
            if fallback.as_ref().map(|(n, _)| c.len() > *n).unwrap_or(true) {
                fallback = Some((c.len(), sec));
            }
            if on_axis
                && axis_best
                    .as_ref()
                    .map(|(n, _)| c.len() > *n)
                    .unwrap_or(true)
            {
                axis_best = Some((c.len(), sec));
            }
        }
        axis_best.or(fallback).map(|(_, s)| s)
    }

    /// Body Y extent `(y_min, y_max)` in centimetres (helper geometry
    /// excluded). Instrumentation for tests / benchmarks that need absolute
    /// slice heights from stature fractions.
    #[doc(hidden)]
    pub fn y_range(&self) -> (f64, f64) {
        (self.y_min, self.y_max)
    }

    /// All mesh vertices (centimetres, helpers included). Instrumentation for
    /// tests / benchmarks that need per-vertex displacement diagnostics.
    #[doc(hidden)]
    pub fn verts(&self) -> &[[f64; 3]] {
        &self.verts
    }

    /// Per-cluster diagnostics for the horizontal slice at height `y`:
    /// `(point_count, hull_perimeter_cm, contains_central_axis, x_extent_cm)`
    /// per cluster, largest cluster first. Instrumentation for verifying that
    /// arm sections stay separate from the torso cluster (used by tests /
    /// benchmarks).
    #[doc(hidden)]
    pub fn slice_cluster_info(&self, y: f64) -> Vec<(usize, f64, bool, f64)> {
        let pts = self.slice_points(y);
        let clusters = cluster_points_2d(&pts, 2.5);
        let mut out: Vec<(usize, f64, bool, f64)> = clusters
            .iter()
            .map(|c| {
                let hull = convex_hull_2d_pts(c);
                let pm = if hull.len() >= 3 {
                    polygon_perimeter_2d(&hull)
                } else {
                    0.0
                };
                let inside = hull.len() >= 3 && point_in_polygon_2d(&hull, [0.0, self.z_axis]);
                (c.len(), pm, inside, hull_x_extent(&hull))
            })
            .collect();
        out.sort_by_key(|c| std::cmp::Reverse(c.0));
        out
    }

    /// Narrowest plausible torso X width, as a fraction of stature. Sections
    /// thinner than this are collapsed slivers (extreme pinch morphs can split
    /// a cross-section into fragments), not a tape path.
    const TORSO_XW_MIN_FRAC: f64 = 0.08;
    /// Widest plausible torso X width, as a fraction of stature. Sections
    /// wider than this have merged with the arms (the T/A-pose arm line
    /// crosses the top of the bust band), so their hull is not a body tape
    /// path. Empirically the torso stays ≲ 0.22 × stature wide even on heavy
    /// bodies while arm-merged hulls jump to ≳ 0.30 × stature.
    const TORSO_XW_MAX_FRAC: f64 = 0.27;

    /// Band extremum of the torso girth over the stature-fraction band
    /// `[lo, hi]`, sampled at `steps + 1` uniformly spaced slices.
    ///
    /// Tape convention: the returned value is the *extremum slice* (max for
    /// bust/hip, min for waist) — no median trimming, so a genuine localised
    /// girth change (e.g. a `measure/measure-bust-circ-incr` morph) is
    /// observed instead of cancelled.
    ///
    /// Robustness comes from per-slice acceptance instead of trimming: a slice
    /// counts only when its torso section (a) wraps the central axis (rejects
    /// arms, legs and stray patches, which are laterally offset clusters) and
    /// (b) has an anatomically plausible X width for a torso (the guards
    /// [`Self::TORSO_XW_MIN_FRAC`] / [`Self::TORSO_XW_MAX_FRAC`], which reject
    /// arm-merged hulls and collapsed slivers). When no slice passes the full
    /// guard the method degrades gracefully: first to axis-verified slices,
    /// then to any substantial section, and only then to `None`.
    fn band_extremum(&self, lo: f64, hi: f64, steps: usize, want_max: bool) -> Option<f64> {
        let h = self.stature_cm();
        if h <= 0.0 || steps == 0 {
            return None;
        }
        let mut guarded: Vec<f64> = Vec::new();
        let mut on_axis: Vec<f64> = Vec::new();
        let mut any: Vec<f64> = Vec::new();
        for i in 0..=steps {
            let f = lo + (hi - lo) * (i as f64 / steps as f64);
            let y = self.y_min + h * f;
            let Some(s) = self.torso_section_at(y) else {
                continue;
            };
            any.push(s.perimeter);
            if s.on_axis {
                on_axis.push(s.perimeter);
                if s.x_extent >= Self::TORSO_XW_MIN_FRAC * h
                    && s.x_extent <= Self::TORSO_XW_MAX_FRAC * h
                {
                    guarded.push(s.perimeter);
                }
            }
        }
        let pick = |v: &[f64]| {
            if want_max {
                v.iter().copied().reduce(f64::max)
            } else {
                v.iter().copied().reduce(f64::min)
            }
        };
        pick(&guarded)
            .or_else(|| pick(&on_axis))
            .or_else(|| pick(&any))
    }

    /// Chest / bust circumference (cm): the fullest torso section over the
    /// anatomical bust band (≈ 0.66–0.76 of stature, spanning underbust to the
    /// armpit line — verified against the MakeHuman `measure/measure-bust-*`
    /// targets, whose girth response peaks at ≈ 0.71–0.75 of stature). Arm
    /// cross-sections are rejected per slice (separate clusters that do not
    /// wrap the central axis); slices where the arms merge into the chest hull
    /// are rejected by the torso width guard, so the band reads the true bust
    /// right up to the arm line.
    pub fn chest_circumference(&self) -> Option<f64> {
        self.band_extremum(0.66, 0.76, 10, true)
    }

    /// Waist circumference (cm): the narrowest torso section over the natural
    /// waist band (≈ 0.58–0.70 of stature, between hip flare and ribcage; the
    /// `measure/measure-waist-*` girth response peaks at ≈ 0.62).
    pub fn waist_circumference(&self) -> Option<f64> {
        self.band_extremum(0.58, 0.70, 12, false)
    }

    /// Hip circumference (cm): the fullest torso section across the pelvis /
    /// buttocks (≈ 0.505–0.60 of stature — above the crotch so the section is
    /// a single central hull rather than two legs; the
    /// `measure/measure-hips-*` girth response peaks at ≈ 0.52–0.57).
    pub fn hip_circumference(&self) -> Option<f64> {
        self.band_extremum(0.505, 0.60, 10, true)
    }

    /// Signed body volume (cm³) of the body mesh via the divergence theorem.
    ///
    /// Helper geometry is already excluded (only body triangles are stored), so
    /// this is the skin-enclosed volume. The MakeHuman body has small aperture
    /// holes (mouth, eye sockets) but the divergence-theorem sum is stable
    /// against apertures that small, so no watertight repair is required.
    pub fn body_volume_cm3(&self) -> f64 {
        let mut v = 0.0;
        for t in self.body_tris.iter() {
            let a = self.verts[t[0]];
            let b = self.verts[t[1]];
            let c = self.verts[t[2]];
            v += a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
        (v / 6.0).abs()
    }

    /// Estimated body mass (kg) = body volume × human mean density.
    pub fn mass_kg(&self) -> f64 {
        // cm³ → litres (÷1000), × density (kg/L).
        self.body_volume_cm3() / 1000.0 * HUMAN_DENSITY_KG_PER_L
    }

    /// Produce the compact tailoring summary. Circumferences that cannot be
    /// resolved fall back to a stature-proportional estimate so the result is
    /// always populated (keeping the fit objective continuous).
    pub fn tailoring_summary(&self) -> TailoringSummary {
        let h = self.stature_cm();
        TailoringSummary {
            height_cm: h,
            chest_cm: self.chest_circumference().unwrap_or(h * 0.55),
            waist_cm: self.waist_circumference().unwrap_or(h * 0.48),
            hip_cm: self.hip_circumference().unwrap_or(h * 0.58),
            weight_kg: self.mass_kg(),
        }
    }
}

// ---- 2-D geometry helpers for the cross-section measurer ----

/// Union-find cluster of 2-D points within radius `r` using a uniform grid.
fn cluster_points_2d(points: &[[f64; 2]], r: f64) -> Vec<Vec<[f64; 2]>> {
    let n = points.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let cell = |p: &[f64; 2]| ((p[0] / r).floor() as i64, (p[1] / r).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, p) in points.iter().enumerate() {
        grid.entry(cell(p)).or_default().push(i);
    }
    let r2 = r * r;
    for (i, p) in points.iter().enumerate() {
        let (cx, cy) = cell(p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(js) = grid.get(&(cx + dx, cy + dy)) {
                    for &j in js {
                        if j <= i {
                            continue;
                        }
                        let ddx = points[j][0] - p[0];
                        let ddy = points[j][1] - p[1];
                        if ddx * ddx + ddy * ddy <= r2 {
                            let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                            if a != b {
                                parent[a] = b;
                            }
                        }
                    }
                }
            }
        }
    }
    let mut groups: HashMap<usize, Vec<[f64; 2]>> = HashMap::new();
    for (i, pt) in points.iter().enumerate() {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(*pt);
    }
    groups.into_values().collect()
}

/// Convex hull (CCW) of 2-D points via Andrew's monotone chain.
fn convex_hull_2d_pts(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut p: Vec<[f64; 2]> = points.to_vec();
    p.sort_by(|a, b| {
        a[0].partial_cmp(&b[0])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal))
    });
    p.dedup_by(|a, b| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9);
    if p.len() < 3 {
        return p;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut hull: Vec<[f64; 2]> = Vec::with_capacity(2 * p.len());
    for &pt in &p {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], pt) <= 0.0 {
            hull.pop();
        }
        hull.push(pt);
    }
    let lower = hull.len() + 1;
    for &pt in p.iter().rev() {
        while hull.len() >= lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], pt) <= 0.0 {
            hull.pop();
        }
        hull.push(pt);
    }
    hull.pop();
    hull
}

/// X extent (width) of a 2-D point set.
fn hull_x_extent(poly: &[[f64; 2]]) -> f64 {
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for p in poly {
        lo = lo.min(p[0]);
        hi = hi.max(p[0]);
    }
    if lo > hi {
        0.0
    } else {
        hi - lo
    }
}

/// Perimeter of a closed 2-D polygon.
fn polygon_perimeter_2d(poly: &[[f64; 2]]) -> f64 {
    if poly.len() < 2 {
        return 0.0;
    }
    let mut s = 0.0;
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        let dx = poly[j][0] - poly[i][0];
        let dy = poly[j][1] - poly[i][1];
        s += (dx * dx + dy * dy).sqrt();
    }
    s
}

/// Even-odd point-in-polygon test in 2-D.
fn point_in_polygon_2d(poly: &[[f64; 2]], pt: [f64; 2]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (poly[i][0], poly[i][1]);
        let (xj, yj) = (poly[j][0], poly[j][1]);
        if ((yi > pt[1]) != (yj > pt[1])) && (pt[0] < (xj - xi) * (pt[1] - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}
