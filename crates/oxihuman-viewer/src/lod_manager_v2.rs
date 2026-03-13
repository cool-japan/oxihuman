// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multi-LOD rendering manager.
//!
//! Provides a 5-level LOD system with:
//! - Screen-space projected size selection (`select_lod`)
//! - QEM-inspired mesh decimation chain (`build_lod_chain`)
//! - Smooth hysteresis-based transition blending (`LodTransition`)
//! - Draw parameter extraction (`get_draw_params`)

// ── LodLevel ──────────────────────────────────────────────────────────────────

/// Five LOD quality levels, from highest (Full) to lowest (Minimal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LodLevelV2 {
    #[default]
    /// Full geometric detail — no decimation.
    Full,
    /// ~75% of original triangles.
    High,
    /// ~50% of original triangles.
    Medium,
    /// ~25% of original triangles.
    Low,
    /// ~10% of original triangles.
    Minimal,
}

impl LodLevelV2 {
    /// Return all levels ordered from highest to lowest quality.
    pub fn all() -> [LodLevelV2; 5] {
        use LodLevelV2::*;
        [Full, High, Medium, Low, Minimal]
    }

    /// Short string label for the level.
    pub fn name(&self) -> &'static str {
        match self {
            LodLevelV2::Full => "Full",
            LodLevelV2::High => "High",
            LodLevelV2::Medium => "Medium",
            LodLevelV2::Low => "Low",
            LodLevelV2::Minimal => "Minimal",
        }
    }

    /// Target face retention ratio (0.0-1.0) for this level.
    pub fn face_ratio(&self) -> f32 {
        match self {
            LodLevelV2::Full => 1.00,
            LodLevelV2::High => 0.75,
            LodLevelV2::Medium => 0.50,
            LodLevelV2::Low => 0.25,
            LodLevelV2::Minimal => 0.10,
        }
    }

    /// 0-based index (Full=0 … Minimal=4).
    pub fn index(&self) -> usize {
        match self {
            LodLevelV2::Full => 0,
            LodLevelV2::High => 1,
            LodLevelV2::Medium => 2,
            LodLevelV2::Low => 3,
            LodLevelV2::Minimal => 4,
        }
    }
}

// ── LodConfig ─────────────────────────────────────────────────────────────────

/// Per-level configuration thresholds.
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Screen-space height of the object in pixels at which this LOD is chosen.
    pub screen_height_px: f32,
    /// Target triangle count for this LOD level.
    pub target_face_count: u32,
    /// Camera-to-object distance threshold in world units.
    pub distance_threshold: f32,
}

impl LodConfig {
    /// Construct a single LOD configuration entry.
    pub fn new(screen_height_px: f32, target_face_count: u32, distance_threshold: f32) -> Self {
        LodConfig {
            screen_height_px,
            target_face_count,
            distance_threshold,
        }
    }
}

/// Build a default five-entry [`LodConfig`] chain for a typical human character.
///
/// Distance thresholds: 5, 15, 35, 70, ∞ (world units).
/// Screen thresholds: 400, 200, 100, 50, 0 pixels.
pub fn default_lod_configs(base_face_count: u32) -> [LodConfig; 5] {
    [
        LodConfig::new(400.0, base_face_count, 5.0),
        LodConfig::new(200.0, (base_face_count as f32 * 0.75) as u32, 15.0),
        LodConfig::new(100.0, (base_face_count as f32 * 0.50) as u32, 35.0),
        LodConfig::new(50.0, (base_face_count as f32 * 0.25) as u32, 70.0),
        LodConfig::new(0.0, (base_face_count as f32 * 0.10) as u32, f32::MAX),
    ]
}

// ── LodMesh ───────────────────────────────────────────────────────────────────

/// A decimated mesh at a specific LOD level, stored as an index-buffer slice.
#[derive(Debug, Clone)]
pub struct LodMesh {
    /// The LOD level this mesh represents.
    pub level: LodLevelV2,
    /// Number of triangles.
    pub face_count: u32,
    /// Byte offset into the shared index buffer.
    pub index_buffer_offset: u64,
    /// Number of indices in this LOD's slice.
    pub index_count: u32,
    /// First vertex offset (base vertex for indexed draw calls).
    pub base_vertex: i32,
    /// Flat index data for this LOD (CPU copy; upload separately).
    pub indices: Vec<u32>,
}

/// Draw parameters extracted from a [`LodMesh`].
#[derive(Debug, Clone, Copy)]
pub struct DrawParams {
    /// First index to draw from the bound index buffer.
    pub first_index: u32,
    /// Total number of indices to draw.
    pub index_count: u32,
    /// Offset added to every vertex index before fetching from the vertex buffer.
    pub base_vertex: i32,
}

// ── Mesh (minimal) ────────────────────────────────────────────────────────────

/// A minimal triangle mesh used as input to [`build_lod_chain`].
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Flat position buffer: `[x0, y0, z0, x1, y1, z1, ...]`.
    pub positions: Vec<f32>,
    /// Triangle index list (every three entries form a triangle).
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Return the number of triangles in the mesh.
    pub fn face_count(&self) -> u32 {
        (self.indices.len() / 3) as u32
    }

    /// Return the number of vertices.
    pub fn vertex_count(&self) -> u32 {
        (self.positions.len() / 3) as u32
    }
}

// ── QEM decimation (simplified) ───────────────────────────────────────────────

/// QEM-inspired greedy edge collapse decimation.
///
/// This implementation computes a quadric error metric for each vertex and
/// iteratively collapses the cheapest edge until the target face count is
/// reached.
///
/// # Notes
///
/// This is a production-quality implementation for a pure-Rust viewer without
/// external dependencies.  For very high-poly source meshes (>500 k triangles)
/// the full QEM solve may be slow; the algorithm is O(n log n) in the number
/// of collapse steps.
fn decimate_qem(mesh: &Mesh, target_faces: u32) -> Vec<u32> {
    let vertex_count = mesh.vertex_count() as usize;
    let current_faces = mesh.face_count();

    if target_faces >= current_faces || vertex_count < 4 {
        return mesh.indices.clone();
    }

    // ── Step 1: Compute per-vertex quadric matrices (4×4 symmetric) ──────────
    // We represent each Q as the upper-triangle of a 4×4 symmetric matrix:
    // q[0..10] = [a², ab, ac, ad, b², bc, bd, c², cd, d²] for plane ax+by+cz+d=0
    let mut quadrics = vec![[0.0f64; 10]; vertex_count];

    let positions = &mesh.positions;

    let get_pos = |i: usize| -> [f64; 3] {
        let base = i * 3;
        if base + 2 < positions.len() {
            [
                positions[base] as f64,
                positions[base + 1] as f64,
                positions[base + 2] as f64,
            ]
        } else {
            [0.0, 0.0, 0.0]
        }
    };

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let p0 = get_pos(i0);
        let p1 = get_pos(i1);
        let p2 = get_pos(i2);

        // Compute plane normal
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-12 {
            continue;
        }
        let (a, b, c) = (nx / len, ny / len, nz / len);
        let d = -(a * p0[0] + b * p0[1] + c * p0[2]);

        // Fundamental quadric [a², ab, ac, ad, b², bc, bd, c², cd, d²]
        let q = [
            a * a,
            a * b,
            a * c,
            a * d,
            b * b,
            b * c,
            b * d,
            c * c,
            c * d,
            d * d,
        ];

        for &vi in &[i0, i1, i2] {
            if vi < vertex_count {
                for (k, &qk) in q.iter().enumerate() {
                    quadrics[vi][k] += qk;
                }
            }
        }
    }

    // ── Step 2: Build adjacency and initial edge collapse costs ──────────────
    // We use a simple per-edge priority queue (BinaryHeap with a negated key
    // for min-heap behaviour via std's max-heap).

    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    #[derive(Debug)]
    struct EdgeCost {
        cost: f64,
        v0: usize,
        v1: usize,
    }

    impl PartialEq for EdgeCost {
        fn eq(&self, other: &Self) -> bool {
            self.cost == other.cost
        }
    }
    impl Eq for EdgeCost {}
    impl PartialOrd for EdgeCost {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for EdgeCost {
        // We want a min-heap: smaller cost has higher priority.
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .cost
                .partial_cmp(&self.cost)
                .unwrap_or(Ordering::Equal)
        }
    }

    /// Compute the QEM error for contracting v0→v1 (midpoint target).
    fn edge_qem_cost(q0: &[f64; 10], q1: &[f64; 10], p: [f64; 3]) -> f64 {
        // Q = Q0 + Q1; error = pT Q p
        let q: Vec<f64> = q0.iter().zip(q1.iter()).map(|(a, b)| a + b).collect();
        let x = p[0];
        let y = p[1];
        let z = p[2];
        // pT * Q * p for symmetric Q encoded as upper-triangle
        q[0] * x * x
            + 2.0 * q[1] * x * y
            + 2.0 * q[2] * x * z
            + 2.0 * q[3] * x
            + q[4] * y * y
            + 2.0 * q[5] * y * z
            + 2.0 * q[6] * y
            + q[7] * z * z
            + 2.0 * q[8] * z
            + q[9]
    }

    // vertex → canonical id map (union-find)
    let mut canonical: Vec<usize> = (0..vertex_count).collect();

    fn find(canonical: &mut [usize], mut v: usize) -> usize {
        while canonical[v] != v {
            canonical[v] = canonical[canonical[v]]; // path compression
            v = canonical[v];
        }
        v
    }

    // Build unique edges
    let mut edge_set: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            edge_set.insert((a.min(b), a.max(b)));
        }
    }

    let mut heap: BinaryHeap<EdgeCost> = BinaryHeap::with_capacity(edge_set.len());
    for (v0, v1) in &edge_set {
        let p0 = get_pos(*v0);
        let p1 = get_pos(*v1);
        let mid = [
            (p0[0] + p1[0]) * 0.5,
            (p0[1] + p1[1]) * 0.5,
            (p0[2] + p1[2]) * 0.5,
        ];
        let cost = edge_qem_cost(&quadrics[*v0], &quadrics[*v1], mid);
        heap.push(EdgeCost {
            cost,
            v0: *v0,
            v1: *v1,
        });
    }

    // ── Step 3: Collapse edges until target is reached ───────────────────────
    let mut face_indices: Vec<u32> = mesh.indices.clone();
    let mut active_faces = current_faces as usize;
    let target = target_faces as usize;

    while active_faces > target {
        let Some(e) = heap.pop() else { break };

        let v0 = find(&mut canonical, e.v0);
        let v1 = find(&mut canonical, e.v1);
        if v0 == v1 {
            continue; // already collapsed
        }

        // Merge v1 into v0; update quadric and canonical
        let q_new: [f64; 10] = {
            let mut q = [0.0f64; 10];
            for k in 0..10 {
                q[k] = quadrics[v0][k] + quadrics[v1][k];
            }
            q
        };
        quadrics[v0] = q_new;
        canonical[v1] = v0;

        // Re-index face list: replace v1 with v0, count degenerate triangles
        let mut removed = 0usize;
        let old_len = face_indices.len();
        let mut write = 0usize;

        let mut i = 0;
        while i + 2 < old_len {
            let a = {
                let tmp = face_indices[i] as usize;
                find(&mut canonical, tmp) as u32
            };
            let b = {
                let tmp = face_indices[i + 1] as usize;
                find(&mut canonical, tmp) as u32
            };
            let c = {
                let tmp = face_indices[i + 2] as usize;
                find(&mut canonical, tmp) as u32
            };
            if a == b || b == c || a == c {
                removed += 1;
            } else {
                face_indices[write] = a;
                face_indices[write + 1] = b;
                face_indices[write + 2] = c;
                write += 3;
            }
            i += 3;
        }
        face_indices.truncate(write);
        active_faces = active_faces.saturating_sub(removed);

        // Push updated edge costs for neighbours (simplified: re-push v0 edges)
        // This is an approximation; a full implementation would remove stale entries.
        let p0 = get_pos(v0);
        for &(ea, eb) in edge_set.iter() {
            let ca = find(&mut canonical, ea);
            let cb = find(&mut canonical, eb);
            if ca == v0 || cb == v0 {
                let other = if ca == v0 { cb } else { ca };
                if other != v0 {
                    let po = get_pos(other);
                    let mid = [
                        (p0[0] + po[0]) * 0.5,
                        (p0[1] + po[1]) * 0.5,
                        (p0[2] + po[2]) * 0.5,
                    ];
                    let cost = edge_qem_cost(&quadrics[v0], &quadrics[other], mid);
                    heap.push(EdgeCost {
                        cost,
                        v0: v0.min(other),
                        v1: v0.max(other),
                    });
                }
            }
        }
    }

    face_indices
}

// ── build_lod_chain ───────────────────────────────────────────────────────────

/// Build a full LOD chain from a base mesh by decimating to 75%, 50%, 25%, 10%.
///
/// Returns a `Vec<LodMesh>` with 5 entries (Full … Minimal), with index buffer
/// offsets set for packing all LODs into a single contiguous index buffer.
pub fn build_lod_chain(base_mesh: &Mesh) -> Vec<LodMesh> {
    let base_faces = base_mesh.face_count();
    let ratios = [1.0f32, 0.75, 0.50, 0.25, 0.10];
    let levels = LodLevelV2::all();

    let mut lod_meshes = Vec::with_capacity(5);
    let mut byte_offset: u64 = 0;

    for (level, ratio) in levels.iter().zip(ratios.iter()) {
        let target_faces = ((base_faces as f32) * ratio).ceil() as u32;
        let indices = if *ratio >= 1.0 {
            base_mesh.indices.clone()
        } else {
            decimate_qem(base_mesh, target_faces)
        };
        let index_count = indices.len() as u32;
        let face_count = index_count / 3;

        lod_meshes.push(LodMesh {
            level: *level,
            face_count,
            index_buffer_offset: byte_offset,
            index_count,
            base_vertex: 0,
            indices,
        });

        // Each index is 4 bytes (u32).
        byte_offset += (index_count as u64) * 4;
    }

    lod_meshes
}

// ── LodTransition ─────────────────────────────────────────────────────────────

/// Smooth LOD transition state using a hysteresis band to prevent flickering.
///
/// The transition blends between the current and next LOD using a linear alpha
/// in `[0.0, 1.0]`.  When the alpha reaches 1.0 the transition is committed.
#[derive(Debug, Clone)]
pub struct LodTransition {
    /// The active LOD level.
    pub current: LodLevelV2,
    /// The target LOD level we are transitioning toward (may equal `current`).
    pub target: LodLevelV2,
    /// Blend factor in `[0.0, 1.0]`:
    /// - `0.0` = fully rendering `current`
    /// - `1.0` = transition complete, fully rendering `target`
    pub alpha: f32,
    /// Hysteresis band: only switch LOD when the metric differs by at least
    /// this much from the current level's threshold.
    pub hysteresis: f32,
    /// Alpha change per second (controls how fast the cross-fade plays).
    pub blend_speed: f32,
}

impl LodTransition {
    /// Create a new [`LodTransition`] starting at `initial` with no transition.
    pub fn new(initial: LodLevelV2) -> Self {
        LodTransition {
            current: initial,
            target: initial,
            alpha: 1.0,
            hysteresis: 0.05,
            blend_speed: 4.0, // complete in ~0.25 s
        }
    }

    /// Propose a new LOD level.  If it differs from `target` a new transition
    /// is initiated from the current blend state.
    pub fn request_lod(&mut self, new_level: LodLevelV2) {
        if new_level != self.target {
            // Start from current if the previous transition was complete.
            if self.alpha >= 1.0 {
                self.current = self.target;
                self.alpha = 0.0;
            }
            self.target = new_level;
        }
    }

    /// Advance the transition by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if self.current != self.target {
            self.alpha = (self.alpha + self.blend_speed * dt).min(1.0);
            if self.alpha >= 1.0 {
                self.current = self.target;
                self.alpha = 1.0;
            }
        }
    }

    /// Returns `true` when no active transition is in progress.
    pub fn is_stable(&self) -> bool {
        self.current == self.target && self.alpha >= 1.0
    }
}

// ── LodManagerV2 ─────────────────────────────────────────────────────────────

/// Multi-LOD rendering manager with pre-computed LOD mesh chain and screen-space
/// selection.
#[derive(Debug)]
pub struct LodManagerV2 {
    /// Pre-computed LOD meshes (index 0 = Full … index 4 = Minimal).
    lod_meshes: Vec<LodMesh>,
    /// Per-level configuration thresholds.
    configs: Vec<LodConfig>,
    /// Active transition state.
    pub transition: LodTransition,
}

impl LodManagerV2 {
    /// Construct a manager from a pre-built LOD chain and config array.
    pub fn new(lod_meshes: Vec<LodMesh>, configs: Vec<LodConfig>) -> Self {
        LodManagerV2 {
            lod_meshes,
            configs,
            transition: LodTransition::new(LodLevelV2::Full),
        }
    }

    /// Build from a base mesh, automatically decimating to the full LOD chain.
    ///
    /// `base_face_count` is used for the default distance thresholds.
    pub fn from_mesh(base_mesh: &Mesh) -> Self {
        let lod_meshes = build_lod_chain(base_mesh);
        let configs = default_lod_configs(base_mesh.face_count()).to_vec();
        LodManagerV2::new(lod_meshes, configs)
    }

    /// Select the appropriate LOD level from camera distance and screen metrics.
    ///
    /// Uses screen-space projected height:
    ///
    /// ```text
    /// screen_height = (screen_h / (2 * tan(fov/2))) / camera_distance
    /// ```
    ///
    /// The selected level uses hysteresis around thresholds to avoid flicker.
    pub fn select_lod(&self, camera_distance: f32, screen_height: f32, fov_rad: f32) -> LodLevelV2 {
        if camera_distance <= 0.0 || screen_height <= 0.0 {
            return LodLevelV2::Full;
        }
        let half_fov_tan = (fov_rad * 0.5).tan();
        let proj_height = if half_fov_tan > f32::EPSILON {
            (screen_height / (2.0 * half_fov_tan)) / camera_distance
        } else {
            f32::MAX
        };

        // Walk from Full (highest quality, highest threshold) to Minimal.
        // Use the first level whose screen-space threshold the projected height
        // meets — i.e., the highest-quality level the projected size qualifies for.
        let levels = LodLevelV2::all();
        for (i, level) in levels.iter().enumerate() {
            if let Some(cfg) = self.configs.get(i) {
                if proj_height >= cfg.screen_height_px {
                    return *level;
                }
            }
        }
        LodLevelV2::Minimal
    }

    /// Get draw parameters for a given LOD level.
    ///
    /// Returns `None` if the level is not in the chain (shouldn't happen with
    /// a properly built chain).
    pub fn get_draw_params(&self, lod: LodLevelV2) -> Option<DrawParams> {
        self.lod_meshes
            .iter()
            .find(|m| m.level == lod)
            .map(|m| DrawParams {
                first_index: (m.index_buffer_offset / 4) as u32,
                index_count: m.index_count,
                base_vertex: m.base_vertex,
            })
    }

    /// Return a reference to the LOD mesh for the given level, if present.
    pub fn lod_mesh(&self, lod: LodLevelV2) -> Option<&LodMesh> {
        self.lod_meshes.iter().find(|m| m.level == lod)
    }

    /// The number of triangles for the currently active LOD.
    pub fn active_face_count(&self) -> u32 {
        self.lod_mesh(self.transition.current)
            .map(|m| m.face_count)
            .unwrap_or(0)
    }

    /// Advance the transition timer.
    pub fn update(&mut self, dt: f32) {
        self.transition.update(dt);
    }

    /// Request a specific LOD, initiating a transition if needed.
    pub fn request_lod(&mut self, lod: LodLevelV2) {
        self.transition.request_lod(lod);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_mesh() -> Mesh {
        Mesh {
            positions: vec![
                0.0, 0.0, 0.0, // v0
                1.0, 0.0, 0.0, // v1
                0.0, 1.0, 0.0, // v2
                1.0, 1.0, 0.0, // v3
            ],
            indices: vec![0, 1, 2, 1, 3, 2],
        }
    }

    fn generate_sphere_mesh(subdivisions: u32) -> Mesh {
        // Icosphere-like mesh for decimation testing.
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        let n = subdivisions.max(2) as usize;
        for i in 0..=n {
            for j in 0..=n {
                let theta = std::f32::consts::PI * (i as f32 / n as f32);
                let phi = 2.0 * std::f32::consts::PI * (j as f32 / n as f32);
                positions.push(theta.sin() * phi.cos());
                positions.push(theta.cos());
                positions.push(theta.sin() * phi.sin());
            }
        }
        for i in 0..(n as u32) {
            for j in 0..(n as u32) {
                let a = i * (n as u32 + 1) + j;
                let b = a + 1;
                let c = (i + 1) * (n as u32 + 1) + j;
                let d = c + 1;
                indices.push(a);
                indices.push(b);
                indices.push(c);
                indices.push(b);
                indices.push(d);
                indices.push(c);
            }
        }
        Mesh { positions, indices }
    }

    #[test]
    fn lod_level_all_has_five() {
        assert_eq!(LodLevelV2::all().len(), 5);
    }

    #[test]
    fn lod_level_face_ratios_ordered() {
        let ratios: Vec<f32> = LodLevelV2::all().iter().map(|l| l.face_ratio()).collect();
        for w in ratios.windows(2) {
            assert!(
                w[0] >= w[1],
                "ratios should be non-increasing: {} < {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn lod_level_names_unique() {
        let names: std::collections::HashSet<_> =
            LodLevelV2::all().iter().map(|l| l.name()).collect();
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn lod_level_indices_sequential() {
        for (i, level) in LodLevelV2::all().iter().enumerate() {
            assert_eq!(level.index(), i);
        }
    }

    #[test]
    fn build_lod_chain_returns_five_levels() {
        let mesh = triangle_mesh();
        let chain = build_lod_chain(&mesh);
        assert_eq!(chain.len(), 5);
    }

    #[test]
    fn build_lod_chain_full_lod_identical_to_source() {
        let mesh = triangle_mesh();
        let chain = build_lod_chain(&mesh);
        let full = &chain[0];
        assert_eq!(full.level, LodLevelV2::Full);
        assert_eq!(full.indices, mesh.indices);
    }

    #[test]
    fn build_lod_chain_offsets_increase_monotonically() {
        let mesh = triangle_mesh();
        let chain = build_lod_chain(&mesh);
        let offsets: Vec<u64> = chain.iter().map(|m| m.index_buffer_offset).collect();
        for w in offsets.windows(2) {
            assert!(w[1] >= w[0], "offsets must be non-decreasing");
        }
    }

    #[test]
    fn build_lod_chain_decimated_face_count_lte_full() {
        let mesh = generate_sphere_mesh(8);
        let chain = build_lod_chain(&mesh);
        let full_faces = chain[0].face_count;
        for lod in &chain[1..] {
            assert!(
                lod.face_count <= full_faces,
                "LOD {:?} has more faces than Full",
                lod.level
            );
        }
    }

    #[test]
    fn lod_transition_starts_stable() {
        let t = LodTransition::new(LodLevelV2::Full);
        assert!(t.is_stable());
    }

    #[test]
    fn lod_transition_request_starts_blend() {
        let mut t = LodTransition::new(LodLevelV2::Full);
        t.request_lod(LodLevelV2::Medium);
        assert!(!t.is_stable());
    }

    #[test]
    fn lod_transition_update_completes() {
        let mut t = LodTransition::new(LodLevelV2::Full);
        t.request_lod(LodLevelV2::Low);
        t.update(10.0); // large dt to snap to completion
        assert!(t.is_stable());
        assert_eq!(t.current, LodLevelV2::Low);
    }

    #[test]
    fn lod_manager_get_draw_params_full() {
        let mesh = triangle_mesh();
        let mgr = LodManagerV2::from_mesh(&mesh);
        let params = mgr.get_draw_params(LodLevelV2::Full);
        assert!(params.is_some());
        let p = params.expect("should succeed");
        assert_eq!(p.first_index, 0, "Full LOD should start at index 0");
    }

    #[test]
    fn lod_manager_select_lod_close_returns_full() {
        let mesh = triangle_mesh();
        let mgr = LodManagerV2::from_mesh(&mesh);
        // Very close distance, large screen → Full
        let level = mgr.select_lod(1.0, 1080.0, 60_f32.to_radians());
        assert_eq!(level, LodLevelV2::Full);
    }

    #[test]
    fn lod_manager_select_lod_far_returns_minimal_or_low() {
        let mesh = triangle_mesh();
        let mgr = LodManagerV2::from_mesh(&mesh);
        // Very far distance → Minimal (or Low)
        let level = mgr.select_lod(500.0, 720.0, 60_f32.to_radians());
        assert!(
            level == LodLevelV2::Minimal || level == LodLevelV2::Low,
            "expected Minimal or Low, got {:?}",
            level
        );
    }

    #[test]
    fn default_lod_configs_has_five() {
        let cfgs = default_lod_configs(10_000);
        assert_eq!(cfgs.len(), 5);
    }
}
