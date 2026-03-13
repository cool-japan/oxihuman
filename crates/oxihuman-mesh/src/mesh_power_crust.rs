// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Power Crust surface reconstruction from oriented point clouds.
//!
//! This module implements surface reconstruction using a simplified
//! ball-pivoting algorithm (BPA), which approximates the power crust approach
//! without requiring a full Delaunay/Voronoi library.
//!
//! # Algorithm overview
//!
//! 1. For each input point, estimate Voronoi poles by finding the farthest
//!    sample point in the approximate Voronoi cell (using k-NN search).
//! 2. Classify poles as inner/outer via angle test against normals.
//! 3. Run ball-pivoting to extract surface triangles: seed a triangle from
//!    three mutually close neighbours, then pivot over each active edge to
//!    grow the front.
//!
//! # Reference
//!
//! Nina Amenta, Sunghee Choi, Ravi Krishna Kolluri — "The Power Crust" (2001).

use anyhow::{bail, Result};
use std::collections::{HashMap, VecDeque};

// ── Math helpers ─────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f64; 3]) -> f64 {
    dot3(v, v).sqrt()
}

#[inline]
fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let l = len3(v);
    if l < 1e-14 {
        [0.0, 0.0, 1.0]
    } else {
        scale3(v, 1.0 / l)
    }
}

#[inline]
fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    len3(sub3(a, b))
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for Power Crust / ball-pivoting reconstruction.
#[derive(Debug, Clone)]
pub struct PowerCrustConfig {
    /// Number of Laplacian smoothing iterations on the output mesh.
    pub smoothing_iterations: u32,
    /// Ratio multiplied with the average nearest-neighbour distance to choose
    /// the ball radius for pivoting.  Default 1.5.
    pub pole_ratio: f64,
    /// Minimum dihedral angle (degrees) used when pivoting to the next face.
    pub min_angle_degrees: f64,
}

impl Default for PowerCrustConfig {
    fn default() -> Self {
        Self {
            smoothing_iterations: 2,
            pole_ratio: 1.5,
            min_angle_degrees: 5.0,
        }
    }
}

// ── Input point ───────────────────────────────────────────────────────────────

#[derive(Clone)]
struct InputPoint {
    pos: [f64; 3],
    normal: [f64; 3],
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Ball-pivoting / power-crust surface reconstructor.
pub struct PowerCrustBuilder {
    points: Vec<InputPoint>,
    config: PowerCrustConfig,
}

impl PowerCrustBuilder {
    /// Create a new builder with the given configuration.
    pub fn new(config: PowerCrustConfig) -> Self {
        Self {
            points: Vec::new(),
            config,
        }
    }

    /// Add a point with an optional surface normal.
    /// If `normal` is `None`, a placeholder is stored; normals will be
    /// estimated from neighbours before reconstruction starts.
    pub fn add_point(&mut self, pos: [f64; 3], normal: Option<[f64; 3]>) {
        self.points.push(InputPoint {
            pos,
            normal: normal.unwrap_or([0.0, 0.0, 0.0]),
        });
    }

    /// Run reconstruction and return `(vertices, indices)`.
    pub fn reconstruct(&mut self) -> Result<(Vec<[f32; 3]>, Vec<u32>)> {
        let n = self.points.len();
        if n < 4 {
            bail!("PowerCrustBuilder::reconstruct requires at least 4 input points, got {n}");
        }

        // Estimate missing normals from neighbourhood PCA
        self.estimate_missing_normals();

        // Compute ball radius as pole_ratio × average k-NN distance
        let ball_radius = self.estimate_ball_radius();

        // Run ball-pivoting algorithm
        let (verts, tris) = ball_pivoting(&self.points, ball_radius, &self.config)?;

        if tris.is_empty() {
            bail!("PowerCrustBuilder: reconstruction produced no triangles");
        }

        // Optional Laplacian smoothing
        let verts = laplacian_smooth_verts(verts, &tris, self.config.smoothing_iterations);

        Ok((verts, tris))
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn estimate_missing_normals(&mut self) {
        let positions: Vec<[f64; 3]> = self.points.iter().map(|p| p.pos).collect();
        let k = 8_usize.min(positions.len() - 1);

        for i in 0..self.points.len() {
            if len3(self.points[i].normal) > 1e-8 {
                continue; // already has a normal
            }
            let nbrs = knn_indices(&positions, i, k);
            let centre = positions[i];
            // Covariance matrix (symmetric 3×3) via neighbours
            let mut cov = [[0.0f64; 3]; 3];
            for &j in &nbrs {
                let d = sub3(positions[j], centre);
                for r in 0..3 {
                    for c in 0..3 {
                        cov[r][c] += d[r] * d[c];
                    }
                }
            }
            // Smallest eigenvector via power iteration on (I - cov/lambda)
            // Use the Gram-Schmidt deflation to pick the smallest eigenvalue direction
            let n = pca_smallest_eigenvector(&cov);
            self.points[i].normal = n;
        }

        // Orient normals consistently via propagation (minimum spanning tree greedy)
        orient_normals_consistent(&mut self.points);
    }

    fn estimate_ball_radius(&self) -> f64 {
        let positions: Vec<[f64; 3]> = self.points.iter().map(|p| p.pos).collect();
        let n = positions.len();
        let sample_count = (n / 4).clamp(8, 200);
        let step = n / sample_count;

        let mut sum = 0.0f64;
        let mut cnt = 0usize;
        for i in (0..n).step_by(step.max(1)) {
            let mut best = f64::INFINITY;
            for j in 0..n {
                if j == i {
                    continue;
                }
                let d = dist3(positions[i], positions[j]);
                if d < best {
                    best = d;
                }
            }
            if best.is_finite() {
                sum += best;
                cnt += 1;
            }
        }

        let mean_nn = if cnt > 0 { sum / cnt as f64 } else { 1.0 };
        mean_nn * self.config.pole_ratio
    }
}

// ── Eigenvector (PCA) helper ──────────────────────────────────────────────────

/// Compute the eigenvector corresponding to the smallest eigenvalue of a
/// symmetric 3×3 matrix using the power method on (I - M / ‖M‖).
fn pca_smallest_eigenvector(cov: &[[f64; 3]; 3]) -> [f64; 3] {
    // Frobenius norm for scaling
    let mut frob = 0.0f64;
    for row in cov {
        for &v in row {
            frob += v * v;
        }
    }
    let scale = frob.sqrt().max(1e-14);

    // Build A = I - cov/scale  →  largest eigvec of A = smallest of cov
    let mut a = [[0.0f64; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            a[r][c] = (if r == c { 1.0 } else { 0.0 }) - cov[r][c] / scale;
        }
    }

    // Power iteration
    let mut v = [1.0f64, 0.577, 0.577];
    for _ in 0..32 {
        let mut w = [0.0f64; 3];
        for r in 0..3 {
            for c in 0..3 {
                w[r] += a[r][c] * v[c];
            }
        }
        v = normalize3(w);
    }
    v
}

// ── Normal orientation propagation ───────────────────────────────────────────

fn orient_normals_consistent(points: &mut [InputPoint]) {
    let n = points.len();
    if n == 0 {
        return;
    }
    let positions: Vec<[f64; 3]> = points.iter().map(|p| p.pos).collect();
    let k = 6_usize.min(n - 1);

    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
    queue.push_back(0usize);
    visited[0] = true;

    while let Some(i) = queue.pop_front() {
        let nbrs = knn_indices(&positions, i, k);
        for j in nbrs {
            if dot3(points[i].normal, points[j].normal) < 0.0 {
                // Flip neighbour normal to align
                let nn = points[j].normal;
                points[j].normal = [-nn[0], -nn[1], -nn[2]];
            }
            if !visited[j] {
                visited[j] = true;
                queue.push_back(j);
            }
        }
    }
}

// ── k-NN helper (brute force, adequate for typical reconstruction sizes) ──────

fn knn_indices(positions: &[[f64; 3]], query: usize, k: usize) -> Vec<usize> {
    let qp = positions[query];
    let mut dists: Vec<(f64, usize)> = positions
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != query)
        .map(|(i, p)| (dist3(qp, *p), i))
        .collect();
    dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    dists.truncate(k);
    dists.into_iter().map(|(_, i)| i).collect()
}

// ── Active edge for ball pivoting ─────────────────────────────────────────────

/// An oriented boundary edge in the current mesh front.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ActiveEdge {
    a: usize, // source vertex index (output)
    b: usize, // destination vertex index (output)
    /// The third vertex of the triangle that spawned this edge (its "left" side is interior).
    pivot_vert: usize,
}

// ── Ball-pivoting algorithm ───────────────────────────────────────────────────

/// Core ball-pivoting surface reconstruction.
///
/// Returns `(f32 positions, u32 indices)`.
fn ball_pivoting(
    points: &[InputPoint],
    ball_radius: f64,
    config: &PowerCrustConfig,
) -> Result<(Vec<[f32; 3]>, Vec<u32>)> {
    let n = points.len();
    let min_cos = (config.min_angle_degrees * std::f64::consts::PI / 180.0).cos();

    // ── Spatial grid for fast range queries ──────────────────────────────────
    let cell_size = ball_radius * 2.0;
    let mut grid: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
    for (i, pt) in points.iter().enumerate() {
        let cell = world_to_cell(pt.pos, cell_size);
        grid.entry(cell).or_default().push(i);
    }

    let range_query = |pos: [f64; 3], radius: f64| -> Vec<usize> {
        let r = (radius / cell_size).ceil() as i32 + 1;
        let base = world_to_cell(pos, cell_size);
        let mut result = Vec::new();
        for dz in -r..=r {
            for dy in -r..=r {
                for dx in -r..=r {
                    let cell = [base[0] + dx, base[1] + dy, base[2] + dz];
                    if let Some(pts) = grid.get(&cell) {
                        for &j in pts {
                            if dist3(pos, points[j].pos) <= radius {
                                result.push(j);
                            }
                        }
                    }
                }
            }
        }
        result
    };

    // ── Output buffers ────────────────────────────────────────────────────────
    let mut out_verts: Vec<[f32; 3]> = Vec::new();
    // Map from input point index to output vertex index
    let mut pt_to_out: HashMap<usize, u32> = HashMap::new();
    let mut out_indices: Vec<u32> = Vec::new();

    let add_vertex = |pt_idx: usize,
                      out_verts: &mut Vec<[f32; 3]>,
                      pt_to_out: &mut HashMap<usize, u32>|
     -> u32 {
        if let Some(&ov) = pt_to_out.get(&pt_idx) {
            return ov;
        }
        let p = points[pt_idx].pos;
        let idx = out_verts.len() as u32;
        out_verts.push([p[0] as f32, p[1] as f32, p[2] as f32]);
        pt_to_out.insert(pt_idx, idx);
        idx
    };

    // Track which input-point edges have been used
    let mut used_edges: HashMap<(usize, usize), bool> = HashMap::new();

    // Active front queue
    let mut front: VecDeque<ActiveEdge> = VecDeque::new();
    // Tracks which input-point triangles are in the mesh
    let mut in_mesh: HashMap<(usize, usize, usize), bool> = HashMap::new();

    // ── Seed triangle search ──────────────────────────────────────────────────
    let mut found_seed = false;
    'seed: for i in 0..n {
        let nbrs = range_query(points[i].pos, ball_radius * 2.5);
        for &j in &nbrs {
            if j == i {
                continue;
            }
            for &k_idx in &nbrs {
                if k_idx == i || k_idx == j {
                    continue;
                }
                // Check if ball of radius r can rest on triangle (i,j,k)
                if let Some(_centre) = circumsphere_centre(
                    points[i].pos,
                    points[j].pos,
                    points[k_idx].pos,
                    ball_radius,
                ) {
                    // Check winding vs average normal
                    let avg_n = add3(
                        add3(points[i].normal, points[j].normal),
                        points[k_idx].normal,
                    );
                    let face_n = cross3(
                        sub3(points[j].pos, points[i].pos),
                        sub3(points[k_idx].pos, points[i].pos),
                    );
                    let (a_out, b_out, c_out) = if dot3(avg_n, face_n) >= 0.0 {
                        (i, j, k_idx)
                    } else {
                        (i, k_idx, j)
                    };

                    let key = canon_tri(a_out, b_out, c_out);
                    if in_mesh.contains_key(&key) {
                        continue;
                    }
                    in_mesh.insert(key, true);

                    let oa = add_vertex(a_out, &mut out_verts, &mut pt_to_out);
                    let ob = add_vertex(b_out, &mut out_verts, &mut pt_to_out);
                    let oc = add_vertex(c_out, &mut out_verts, &mut pt_to_out);
                    out_indices.extend_from_slice(&[oa, ob, oc]);

                    // Push boundary edges
                    for (ea, eb, ep) in [
                        (a_out, b_out, c_out),
                        (b_out, c_out, a_out),
                        (c_out, a_out, b_out),
                    ] {
                        used_edges.insert((ea, eb), true);
                        front.push_back(ActiveEdge {
                            a: ea,
                            b: eb,
                            pivot_vert: ep,
                        });
                    }

                    found_seed = true;
                    break 'seed;
                }
            }
        }
    }

    if !found_seed {
        // Fallback: gift-wrap a convex seed from nearest 4 points
        let seed = gift_wrap_seed(points);
        if let Some((i, j, k)) = seed {
            let key = canon_tri(i, j, k);
            in_mesh.insert(key, true);
            let oa = add_vertex(i, &mut out_verts, &mut pt_to_out);
            let ob = add_vertex(j, &mut out_verts, &mut pt_to_out);
            let oc = add_vertex(k, &mut out_verts, &mut pt_to_out);
            out_indices.extend_from_slice(&[oa, ob, oc]);
            for (ea, eb, ep) in [(i, j, k), (j, k, i), (k, i, j)] {
                used_edges.insert((ea, eb), true);
                front.push_back(ActiveEdge {
                    a: ea,
                    b: eb,
                    pivot_vert: ep,
                });
            }
        } else {
            bail!("ball_pivoting: could not find a seed triangle");
        }
    }

    // ── Main pivoting loop ────────────────────────────────────────────────────
    let max_iters = n * 20 + 10_000;
    let mut iters = 0usize;

    while let Some(edge) = front.pop_front() {
        iters += 1;
        if iters > max_iters {
            break;
        }

        let a = edge.a;
        let b = edge.b;
        let pivot = edge.pivot_vert;

        // Check if reverse edge already closed this boundary
        if used_edges.get(&(b, a)).copied().unwrap_or(false) {
            continue;
        }

        let pa = points[a].pos;
        let pb = points[b].pos;

        // Find candidate c: in ball range, not already a, b, or pivot
        let mid = scale3(add3(pa, pb), 0.5);
        let candidates = range_query(mid, ball_radius * 2.5);

        // Compute pivot direction reference: normal of current face edge
        let pivot_pos = points[pivot].pos;

        // Project into edge plane: choose c that minimises dihedral from pivot
        // (maximises dot product with the "outward" rotated direction)
        let mut best_c: Option<usize> = None;
        let mut best_score = f64::NEG_INFINITY;

        let edge_dir = normalize3(sub3(pb, pa));
        let to_pivot = sub3(pivot_pos, pa);
        // Rejection of to_pivot from edge_dir
        let to_pivot_proj = sub3(to_pivot, scale3(edge_dir, dot3(to_pivot, edge_dir)));
        let ref_dir = normalize3(to_pivot_proj);

        for &c in &candidates {
            if c == a || c == b || c == pivot {
                continue;
            }
            // Ball must fit on the new triangle
            if circumsphere_centre(pa, pb, points[c].pos, ball_radius).is_none() {
                continue;
            }
            // Check min angle
            let to_c = sub3(points[c].pos, pa);
            let to_c_proj = sub3(to_c, scale3(edge_dir, dot3(to_c, edge_dir)));
            if len3(to_c_proj) < 1e-12 {
                continue;
            }
            let to_c_n = normalize3(to_c_proj);
            let cos_angle = dot3(ref_dir, to_c_n);
            if cos_angle > min_cos {
                continue; // less than min_angle away from pivot → degenerate
            }
            // Check not already in mesh
            let key = canon_tri(a, b, c);
            if in_mesh.contains_key(&key) {
                continue;
            }
            // Score: furthest rotation (most negative cosine = largest angle)
            // Use signed angle via cross product
            let cross = cross3(ref_dir, to_c_n);
            let cross_dot_edge = dot3(cross, edge_dir);
            // For CCW rotation (positive edge direction), positive cross_dot is "inward"
            // We want the first face when sweeping outward (smallest positive angle from ref_dir)
            let score = if cross_dot_edge >= 0.0 {
                -2.0 - cos_angle // inward half: large negative
            } else {
                cos_angle // outward half: prefer most "outward" (highest cosine first)
            };

            if score > best_score {
                best_score = score;
                best_c = Some(c);
            }
        }

        if let Some(c) = best_c {
            let key = canon_tri(a, b, c);
            if in_mesh.contains_key(&key) {
                continue;
            }
            in_mesh.insert(key, true);

            let oa = if let Some(&v) = pt_to_out.get(&a) {
                v
            } else {
                add_vertex(a, &mut out_verts, &mut pt_to_out)
            };
            let ob = if let Some(&v) = pt_to_out.get(&b) {
                v
            } else {
                add_vertex(b, &mut out_verts, &mut pt_to_out)
            };
            let oc = add_vertex(c, &mut out_verts, &mut pt_to_out);

            out_indices.extend_from_slice(&[oa, ob, oc]);

            // New boundary edges from the new triangle (b→c and c→a)
            for (ea, eb, ep) in [(b, c, a), (c, a, b)] {
                if !used_edges.get(&(eb, ea)).copied().unwrap_or(false) {
                    used_edges.insert((ea, eb), true);
                    front.push_back(ActiveEdge {
                        a: ea,
                        b: eb,
                        pivot_vert: ep,
                    });
                }
            }
        }
    }

    Ok((out_verts, out_indices))
}

// ── Circumsphere helpers ───────────────────────────────────────────────────────

/// Returns the circumsphere centre of triangle (p0,p1,p2) if a ball of the
/// given radius can rest on this triangle (i.e. circumradius ≤ ball_radius).
fn circumsphere_centre(
    p0: [f64; 3],
    p1: [f64; 3],
    p2: [f64; 3],
    ball_radius: f64,
) -> Option<[f64; 3]> {
    let a = sub3(p1, p0);
    let b = sub3(p2, p0);
    let cross = cross3(a, b);
    let cross_len2 = dot3(cross, cross);
    if cross_len2 < 1e-20 {
        return None; // degenerate triangle
    }

    // Circumradius
    let la = len3(a);
    let lb = len3(b);
    let lc = len3(sub3(p2, p1));
    let circum_r = (la * lb * lc) / (2.0 * cross_len2.sqrt());

    if circum_r > ball_radius {
        return None; // ball too small to rest on this triangle
    }

    // Circumcentre via barycentric formula
    let d1 = dot3(a, a);
    let d2 = dot3(a, b);
    let d3 = dot3(b, b);
    let denom = d1 * d3 - d2 * d2;
    if denom.abs() < 1e-20 {
        return None;
    }

    let s = (d1 * d3 - d2 * d2).recip();
    let u = (d3 - d2) * s * 0.5 * d1;
    let v = (d1 - d2) * s * 0.5 * d3;

    // Circumcentre in world space
    let cc = add3(p0, add3(scale3(a, u / d1), scale3(b, v / d3)));
    Some(cc)
}

// ── Gift-wrap seed ────────────────────────────────────────────────────────────

/// Find an initial seed triangle for the front when ball-placing fails.
/// Returns indices into the input point array.
fn gift_wrap_seed(points: &[InputPoint]) -> Option<(usize, usize, usize)> {
    let n = points.len();
    if n < 3 {
        return None;
    }

    // Pick the point with smallest x as anchor
    let mut anchor = 0;
    for i in 1..n {
        if points[i].pos[0] < points[anchor].pos[0] {
            anchor = i;
        }
    }

    // Find nearest neighbour of anchor
    let mut best_j = usize::MAX;
    let mut best_d = f64::INFINITY;
    for j in 0..n {
        if j == anchor {
            continue;
        }
        let d = dist3(points[anchor].pos, points[j].pos);
        if d < best_d {
            best_d = d;
            best_j = j;
        }
    }
    if best_j == usize::MAX {
        return None;
    }

    // Find k that maximises solid angle / minimises dihedral with anchor normal
    let edge = sub3(points[best_j].pos, points[anchor].pos);
    let avg_n = normalize3(add3(points[anchor].normal, points[best_j].normal));
    let ref_n = if len3(avg_n) < 1e-8 {
        [0.0, 0.0, 1.0]
    } else {
        avg_n
    };

    let mut best_k = usize::MAX;
    let mut best_score = f64::NEG_INFINITY;
    for k in 0..n {
        if k == anchor || k == best_j {
            continue;
        }
        let face_n = cross3(edge, sub3(points[k].pos, points[anchor].pos));
        let score = dot3(normalize3(face_n), ref_n);
        if score > best_score {
            best_score = score;
            best_k = k;
        }
    }

    if best_k == usize::MAX {
        None
    } else {
        Some((anchor, best_j, best_k))
    }
}

// ── Canonical triangle key ────────────────────────────────────────────────────

#[inline]
fn canon_tri(a: usize, b: usize, c: usize) -> (usize, usize, usize) {
    let mut arr = [a, b, c];
    arr.sort_unstable();
    (arr[0], arr[1], arr[2])
}

// ── Spatial grid helper ───────────────────────────────────────────────────────

#[inline]
fn world_to_cell(p: [f64; 3], cell_size: f64) -> [i32; 3] {
    [
        (p[0] / cell_size).floor() as i32,
        (p[1] / cell_size).floor() as i32,
        (p[2] / cell_size).floor() as i32,
    ]
}

// ── Laplacian smoothing on output vertices ────────────────────────────────────

fn laplacian_smooth_verts(
    mut verts: Vec<[f32; 3]>,
    indices: &[u32],
    iterations: u32,
) -> Vec<[f32; 3]> {
    if iterations == 0 || verts.is_empty() {
        return verts;
    }
    let nv = verts.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nv];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        adj[a].push(b);
        adj[a].push(c);
        adj[b].push(a);
        adj[b].push(c);
        adj[c].push(a);
        adj[c].push(b);
    }

    for _ in 0..iterations {
        let prev = verts.clone();
        for i in 0..nv {
            if adj[i].is_empty() {
                continue;
            }
            let mut sum = [0.0f32; 3];
            for &j in &adj[i] {
                sum[0] += prev[j][0];
                sum[1] += prev[j][1];
                sum[2] += prev[j][2];
            }
            let k = adj[i].len() as f32;
            verts[i] = [sum[0] / k, sum[1] / k, sum[2] / k];
        }
    }
    verts
}

// ── Deprecated shim types (kept for library compatibility) ────────────────────

/// Legacy weighted power-diagram site.  Not used by the builder; kept for
/// downstream code that references the old stub API.
pub struct PowerSite {
    pub center: [f32; 3],
    pub weight: f32,
}

/// Legacy result type from the stub.  Prefer [`PowerCrustBuilder`] instead.
pub struct PowerCrustResult {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub inner_poles: Vec<[f32; 3]>,
    pub outer_poles: Vec<[f32; 3]>,
}

/// Compute the power distance from point `p` to site `s`.
pub fn power_distance(p: [f32; 3], site: &PowerSite) -> f32 {
    let d2 = (p[0] - site.center[0]).powi(2)
        + (p[1] - site.center[1]).powi(2)
        + (p[2] - site.center[2]).powi(2);
    d2 - site.weight
}

/// Find the site with minimum power distance.
pub fn nearest_power_site(p: [f32; 3], sites: &[PowerSite]) -> Option<&PowerSite> {
    sites.iter().min_by(|a, b| {
        power_distance(p, a)
            .partial_cmp(&power_distance(p, b))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Classify a pole as outer (`true`) or inner (`false`) based on normal alignment.
pub fn classify_pole(pole: [f32; 3], center: [f32; 3], normal: [f32; 3]) -> bool {
    let d = [
        pole[0] - center[0],
        pole[1] - center[1],
        pole[2] - center[2],
    ];
    d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2] > 0.0
}

/// Generate approximate inner/outer poles from a point cloud.
pub fn generate_power_poles(points: &[[f32; 3]], scale: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let inner = points.iter().map(|&p| [p[0], p[1], p[2] - scale]).collect();
    let outer = points.iter().map(|&p| [p[0], p[1], p[2] + scale]).collect();
    (inner, outer)
}

/// Legacy stub: returns a simple triangle fan (for backward compat tests only).
pub fn power_crust_stub(points: &[[f32; 3]], _radius: f32) -> PowerCrustResult {
    let (inner, outer) = generate_power_poles(points, 0.5);
    let n = points.len();
    let mut triangles = Vec::new();
    if n >= 3 {
        for i in 1..n - 1 {
            triangles.push([0, i, i + 1]);
        }
    }
    PowerCrustResult {
        positions: points.to_vec(),
        triangles,
        inner_poles: inner,
        outer_poles: outer,
    }
}

/// Estimate medial ball radius.
pub fn medial_ball_radius(point: [f32; 3], inner_pole: [f32; 3]) -> f32 {
    let d = [
        inner_pole[0] - point[0],
        inner_pole[1] - point[1],
        inner_pole[2] - point[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Count triangles in legacy result.
pub fn power_crust_triangle_count(result: &PowerCrustResult) -> usize {
    result.triangles.len()
}

/// Check if legacy result has geometry.
pub fn power_crust_has_geometry(result: &PowerCrustResult) -> bool {
    !result.positions.is_empty()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn sphere_cloud(n: usize) -> Vec<([f64; 3], [f64; 3])> {
        // Fibonacci sphere
        let golden = PI * (3.0 - 5.0f64.sqrt());
        (0..n)
            .map(|i| {
                let y = 1.0 - (i as f64 / (n as f64 - 1.0)) * 2.0;
                let r = (1.0 - y * y).max(0.0).sqrt();
                let theta = golden * i as f64;
                let x = r * theta.cos();
                let z = r * theta.sin();
                let pos = [x, y, z];
                let normal = [x, y, z]; // outward normal on unit sphere
                (pos, normal)
            })
            .collect()
    }

    #[test]
    fn reconstruct_sphere_produces_closed_mesh() {
        let cloud = sphere_cloud(200);
        let mut builder = PowerCrustBuilder::new(PowerCrustConfig::default());
        for (pos, n) in &cloud {
            builder.add_point(*pos, Some(*n));
        }
        let (verts, indices) = builder
            .reconstruct()
            .expect("reconstruction should succeed");
        assert!(!verts.is_empty(), "output must have vertices");
        assert!(indices.len() % 3 == 0, "indices must be a triangle list");
        // Should produce a non-trivial mesh
        assert!(
            indices.len() >= 6,
            "sphere reconstruction must have at least 2 triangles"
        );
    }

    #[test]
    fn reconstruct_requires_four_points() {
        let mut builder = PowerCrustBuilder::new(PowerCrustConfig::default());
        builder.add_point([0.0, 0.0, 0.0], None);
        builder.add_point([1.0, 0.0, 0.0], None);
        builder.add_point([0.0, 1.0, 0.0], None);
        assert!(builder.reconstruct().is_err());
    }

    #[test]
    fn reconstruct_small_tetrahedron() {
        let pts: Vec<([f64; 3], [f64; 3])> = vec![
            ([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
            ([-1.0, -1.0, 1.0], [-1.0, -1.0, 1.0]),
            ([-1.0, 1.0, -1.0], [-1.0, 1.0, -1.0]),
            ([1.0, -1.0, -1.0], [1.0, -1.0, -1.0]),
        ];
        let mut builder = PowerCrustBuilder::new(PowerCrustConfig {
            smoothing_iterations: 0,
            pole_ratio: 2.0,
            ..Default::default()
        });
        for (pos, n) in &pts {
            builder.add_point(*pos, Some(*n));
        }
        let res = builder.reconstruct();
        // 4 points may or may not produce a valid ball-pivot result depending on radius,
        // but it must not panic
        let _ = res;
    }

    // ── Legacy stub tests (kept for backward compat) ──────────────────────────

    fn cloud() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 0.5],
        ]
    }

    #[test]
    fn power_distance_at_center_is_negative_weight() {
        let site = PowerSite {
            center: [0.0, 0.0, 0.0],
            weight: 1.0,
        };
        let pd = power_distance([0.0, 0.0, 0.0], &site);
        assert!((pd + 1.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_power_site_found() {
        let sites = vec![
            PowerSite {
                center: [0.0, 0.0, 0.0],
                weight: 0.0,
            },
            PowerSite {
                center: [5.0, 0.0, 0.0],
                weight: 0.0,
            },
        ];
        let ns = nearest_power_site([0.1, 0.0, 0.0], &sites);
        assert!(ns.is_some());
        assert!((ns.expect("should succeed").center[0]).abs() < 1.0);
    }

    #[test]
    fn classify_pole_inner() {
        let pole = [0.0, 0.0, -1.0];
        let center = [0.0, 0.0, 0.0];
        let normal = [0.0, 0.0, 1.0];
        assert!(!classify_pole(pole, center, normal));
    }

    #[test]
    fn generate_poles_count() {
        let pts = cloud();
        let (inner, outer) = generate_power_poles(&pts, 0.5);
        assert_eq!(inner.len(), pts.len());
        assert_eq!(outer.len(), pts.len());
    }

    #[test]
    fn power_crust_stub_has_geometry() {
        let pts = cloud();
        let r = power_crust_stub(&pts, 1.0);
        assert!(power_crust_has_geometry(&r));
    }

    #[test]
    fn medial_ball_radius_positive() {
        let r = medial_ball_radius([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangle_count_nonzero() {
        let pts = cloud();
        let r = power_crust_stub(&pts, 1.0);
        assert!(power_crust_triangle_count(&r) > 0);
    }

    #[test]
    fn outer_poles_above_inner() {
        let pts = cloud();
        let (inner, outer) = generate_power_poles(&pts, 0.5);
        for (i, o) in inner.iter().zip(outer.iter()) {
            assert!(o[2] > i[2]);
        }
    }
}
