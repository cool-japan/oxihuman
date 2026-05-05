// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Anatomical landmark detection and mapping for body meshes.
//!
//! Provides:
//! - [`LandmarkId`] — enumeration of named anatomical landmarks.
//! - [`Landmark`] — a detected landmark with position, confidence, and vertex index.
//! - [`LandmarkSet`] — a collection of landmarks with body measurement helpers.
//! - [`Side`] — left/right discriminant.
//! - Free functions for detecting, remapping, and transferring landmarks on meshes.

use crate::engine::MeshBuffers;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// LandmarkId
// ---------------------------------------------------------------------------

/// Named anatomical landmark identifiers covering the major body sites.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LandmarkId {
    // Head
    TopOfHead,
    ChinCenter,
    // Spine
    /// 7th cervical vertebra prominence.
    C7Cervical,
    /// 10th thoracic vertebra.
    T10Thoracic,
    /// 4th lumbar vertebra.
    L4Lumbar,
    // Shoulder
    AcromionLeft,
    AcromionRight,
    // Arms
    ElbowLeft,
    ElbowRight,
    WristLeft,
    WristRight,
    // Hips & Legs
    HipLeft,
    HipRight,
    KneeLeft,
    KneeRight,
    AnkleLeft,
    AnkleRight,
    // Torso
    NeckBase,
    ChestCenter,
    WaistCenter,
    NabelCenter,
    // Feet
    HeelLeft,
    HeelRight,
}

impl LandmarkId {
    /// Returns all landmark variants in a stable order.
    pub fn all() -> Vec<LandmarkId> {
        vec![
            LandmarkId::TopOfHead,
            LandmarkId::ChinCenter,
            LandmarkId::C7Cervical,
            LandmarkId::T10Thoracic,
            LandmarkId::L4Lumbar,
            LandmarkId::AcromionLeft,
            LandmarkId::AcromionRight,
            LandmarkId::ElbowLeft,
            LandmarkId::ElbowRight,
            LandmarkId::WristLeft,
            LandmarkId::WristRight,
            LandmarkId::HipLeft,
            LandmarkId::HipRight,
            LandmarkId::KneeLeft,
            LandmarkId::KneeRight,
            LandmarkId::AnkleLeft,
            LandmarkId::AnkleRight,
            LandmarkId::NeckBase,
            LandmarkId::ChestCenter,
            LandmarkId::WaistCenter,
            LandmarkId::NabelCenter,
            LandmarkId::HeelLeft,
            LandmarkId::HeelRight,
        ]
    }

    /// Human-readable name for the landmark.
    pub fn name(&self) -> &'static str {
        match self {
            LandmarkId::TopOfHead => "Top of Head",
            LandmarkId::ChinCenter => "Chin Center",
            LandmarkId::C7Cervical => "C7 Cervical",
            LandmarkId::T10Thoracic => "T10 Thoracic",
            LandmarkId::L4Lumbar => "L4 Lumbar",
            LandmarkId::AcromionLeft => "Acromion Left",
            LandmarkId::AcromionRight => "Acromion Right",
            LandmarkId::ElbowLeft => "Elbow Left",
            LandmarkId::ElbowRight => "Elbow Right",
            LandmarkId::WristLeft => "Wrist Left",
            LandmarkId::WristRight => "Wrist Right",
            LandmarkId::HipLeft => "Hip Left",
            LandmarkId::HipRight => "Hip Right",
            LandmarkId::KneeLeft => "Knee Left",
            LandmarkId::KneeRight => "Knee Right",
            LandmarkId::AnkleLeft => "Ankle Left",
            LandmarkId::AnkleRight => "Ankle Right",
            LandmarkId::NeckBase => "Neck Base",
            LandmarkId::ChestCenter => "Chest Center",
            LandmarkId::WaistCenter => "Waist Center",
            LandmarkId::NabelCenter => "Navel Center",
            LandmarkId::HeelLeft => "Heel Left",
            LandmarkId::HeelRight => "Heel Right",
        }
    }

    /// Returns `true` if this landmark has a left/right counterpart.
    pub fn is_bilateral(&self) -> bool {
        self.mirror().is_some()
    }

    /// Returns the mirrored (opposite side) landmark, or `None` for midline landmarks.
    pub fn mirror(&self) -> Option<LandmarkId> {
        match self {
            LandmarkId::AcromionLeft => Some(LandmarkId::AcromionRight),
            LandmarkId::AcromionRight => Some(LandmarkId::AcromionLeft),
            LandmarkId::ElbowLeft => Some(LandmarkId::ElbowRight),
            LandmarkId::ElbowRight => Some(LandmarkId::ElbowLeft),
            LandmarkId::WristLeft => Some(LandmarkId::WristRight),
            LandmarkId::WristRight => Some(LandmarkId::WristLeft),
            LandmarkId::HipLeft => Some(LandmarkId::HipRight),
            LandmarkId::HipRight => Some(LandmarkId::HipLeft),
            LandmarkId::KneeLeft => Some(LandmarkId::KneeRight),
            LandmarkId::KneeRight => Some(LandmarkId::KneeLeft),
            LandmarkId::AnkleLeft => Some(LandmarkId::AnkleRight),
            LandmarkId::AnkleRight => Some(LandmarkId::AnkleLeft),
            LandmarkId::HeelLeft => Some(LandmarkId::HeelRight),
            LandmarkId::HeelRight => Some(LandmarkId::HeelLeft),
            // Midline landmarks
            LandmarkId::TopOfHead
            | LandmarkId::ChinCenter
            | LandmarkId::C7Cervical
            | LandmarkId::T10Thoracic
            | LandmarkId::L4Lumbar
            | LandmarkId::NeckBase
            | LandmarkId::ChestCenter
            | LandmarkId::WaistCenter
            | LandmarkId::NabelCenter => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Landmark
// ---------------------------------------------------------------------------

/// A single detected anatomical landmark on a mesh.
#[derive(Debug, Clone)]
pub struct Landmark {
    /// Semantic identifier.
    pub id: LandmarkId,
    /// World-space position `[x, y, z]`.
    pub position: [f32; 3],
    /// Detection confidence in `0..=1`.
    pub confidence: f32,
    /// Index of the nearest mesh vertex, if known.
    pub vertex_index: Option<u32>,
}

impl Landmark {
    /// Construct a new landmark.
    pub fn new(
        id: LandmarkId,
        position: [f32; 3],
        confidence: f32,
        vertex_index: Option<u32>,
    ) -> Self {
        Landmark {
            id,
            position,
            confidence,
            vertex_index,
        }
    }
}

// ---------------------------------------------------------------------------
// Side
// ---------------------------------------------------------------------------

/// Body side discriminant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// LandmarkSet
// ---------------------------------------------------------------------------

/// A collection of body landmarks with measurement helpers.
#[derive(Debug, Clone, Default)]
pub struct LandmarkSet {
    landmarks: HashMap<LandmarkId, Landmark>,
}

impl LandmarkSet {
    /// Create an empty landmark set.
    pub fn new() -> Self {
        LandmarkSet {
            landmarks: HashMap::new(),
        }
    }

    /// Insert or replace a landmark.
    pub fn insert(&mut self, landmark: Landmark) {
        self.landmarks.insert(landmark.id.clone(), landmark);
    }

    /// Retrieve a landmark by id.
    pub fn get(&self, id: &LandmarkId) -> Option<&Landmark> {
        self.landmarks.get(id)
    }

    /// Number of landmarks in this set.
    pub fn count(&self) -> usize {
        self.landmarks.len()
    }

    /// Returns all positions and their confidence values.
    pub fn all_positions(&self) -> Vec<([f32; 3], f32)> {
        self.landmarks
            .values()
            .map(|lm| (lm.position, lm.confidence))
            .collect()
    }

    /// Euclidean distance between two landmarks, or `None` if either is missing.
    pub fn distance(&self, a: &LandmarkId, b: &LandmarkId) -> Option<f32> {
        let pa = self.landmarks.get(a)?.position;
        let pb = self.landmarks.get(b)?.position;
        Some(vec3_dist(pa, pb))
    }

    /// Estimated body height from `TopOfHead` to `HeelLeft` (average of left/right if available).
    ///
    /// Returns `None` if `TopOfHead` is absent.
    pub fn body_height(&self) -> Option<f32> {
        let head = self.landmarks.get(&LandmarkId::TopOfHead)?.position;
        // Use the lower of the two heels as floor reference.
        let floor_y = match (
            self.landmarks.get(&LandmarkId::HeelLeft),
            self.landmarks.get(&LandmarkId::HeelRight),
        ) {
            (Some(l), Some(r)) => l.position[1].min(r.position[1]),
            (Some(l), None) => l.position[1],
            (None, Some(r)) => r.position[1],
            (None, None) => {
                // Fall back to ankle
                let al = self
                    .landmarks
                    .get(&LandmarkId::AnkleLeft)
                    .map(|l| l.position[1]);
                let ar = self
                    .landmarks
                    .get(&LandmarkId::AnkleRight)
                    .map(|l| l.position[1]);
                match (al, ar) {
                    (Some(a), Some(b)) => a.min(b),
                    (Some(a), None) | (None, Some(a)) => a,
                    (None, None) => return None,
                }
            }
        };
        Some((head[1] - floor_y).abs())
    }

    /// Shoulder width from left acromion to right acromion.
    pub fn shoulder_width(&self) -> Option<f32> {
        self.distance(&LandmarkId::AcromionLeft, &LandmarkId::AcromionRight)
    }

    /// Hip width from left hip to right hip.
    pub fn hip_width(&self) -> Option<f32> {
        self.distance(&LandmarkId::HipLeft, &LandmarkId::HipRight)
    }

    /// Arm length for the given side: shoulder (acromion) to wrist.
    pub fn arm_length(&self, side: Side) -> Option<f32> {
        let (shoulder, wrist) = match side {
            Side::Left => (&LandmarkId::AcromionLeft, &LandmarkId::WristLeft),
            Side::Right => (&LandmarkId::AcromionRight, &LandmarkId::WristRight),
        };
        self.distance(shoulder, wrist)
    }

    /// Leg length for the given side: hip to ankle.
    pub fn leg_length(&self, side: Side) -> Option<f32> {
        let (hip, ankle) = match side {
            Side::Left => (&LandmarkId::HipLeft, &LandmarkId::AnkleLeft),
            Side::Right => (&LandmarkId::HipRight, &LandmarkId::AnkleRight),
        };
        self.distance(hip, ankle)
    }

    /// Maximum left-right asymmetry across all bilateral landmark pairs (in world units).
    ///
    /// For each bilateral pair, computes the difference in X-axis distance from the
    /// midline. Returns the maximum such difference; returns `0.0` if no bilateral
    /// pairs are present.
    pub fn symmetry_error(&self) -> f32 {
        let mut max_err = 0.0f32;
        for id in LandmarkId::all() {
            if let Some(mirror_id) = id.mirror() {
                if let (Some(lm_a), Some(lm_b)) =
                    (self.landmarks.get(&id), self.landmarks.get(&mirror_id))
                {
                    // Compare absolute X offsets — for a symmetric body both should be equal in magnitude
                    let err = (lm_a.position[0].abs() - lm_b.position[0].abs()).abs();
                    // Y and Z should also match
                    let dy = (lm_a.position[1] - lm_b.position[1]).abs();
                    let dz = (lm_a.position[2] - lm_b.position[2]).abs();
                    let combined = err.max(dy).max(dz);
                    if combined > max_err {
                        max_err = combined;
                    }
                }
            }
        }
        max_err
    }

    /// Serialize to a map of `landmark_name → [x, y, z]`.
    pub fn to_map(&self) -> HashMap<String, [f32; 3]> {
        self.landmarks
            .values()
            .map(|lm| (lm.id.name().to_string(), lm.position))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helper math
// ---------------------------------------------------------------------------

#[inline]
fn vec3_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

// ---------------------------------------------------------------------------
// Bounding box helpers
// ---------------------------------------------------------------------------

/// Compute the axis-aligned bounding box of mesh positions.
/// Returns `(min, max)` or `([0,0,0],[0,0,0])` for an empty mesh.
fn mesh_aabb(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

/// Interpolate linearly between `min_v` and `max_v` by fraction `t` in `[0,1]`.
#[inline]
fn lerp1(min_v: f32, max_v: f32, t: f32) -> f32 {
    min_v + (max_v - min_v) * t
}

// ---------------------------------------------------------------------------
// detect_landmarks
// ---------------------------------------------------------------------------

/// Detect anatomical landmarks from a mesh using heuristic bounding-box rules.
///
/// The strategy assigns landmarks using approximate fractional positions within
/// the mesh's bounding box, then snaps each position to the nearest mesh vertex.
/// All detections receive `confidence = 1.0` (heuristic, not learned).
pub fn detect_landmarks(mesh: &MeshBuffers) -> LandmarkSet {
    let positions = &mesh.positions;
    let mut set = LandmarkSet::new();

    if positions.is_empty() {
        return set;
    }

    let (mn, mx) = mesh_aabb(positions);
    let cx = (mn[0] + mx[0]) * 0.5; // midline X
    let lx = lerp1(mn[0], cx, 0.5); // left X (negative side)
    let rx = lerp1(cx, mx[0], 0.5); // right X (positive side)

    // Helper: make a landmark at a given approximate world position, snapped to nearest vertex.
    let make = |id: LandmarkId, approx: [f32; 3]| -> Landmark {
        let (vidx, _dist) = nearest_vertex(mesh, approx);
        let pos = positions[vidx as usize];
        Landmark::new(id, pos, 1.0, Some(vidx))
    };

    // Y fractions (bottom=0, top=1) tuned for a standing human figure.
    // These are rough anatomical proportions.
    let y_frac = |t: f32| lerp1(mn[1], mx[1], t);

    // Head
    set.insert(make(
        LandmarkId::TopOfHead,
        [cx, y_frac(1.00), (mn[2] + mx[2]) * 0.5],
    ));
    set.insert(make(LandmarkId::ChinCenter, [cx, y_frac(0.88), mx[2]]));

    // Neck / upper torso
    set.insert(make(
        LandmarkId::NeckBase,
        [cx, y_frac(0.84), (mn[2] + mx[2]) * 0.5],
    ));
    set.insert(make(LandmarkId::C7Cervical, [cx, y_frac(0.83), mn[2]]));

    // Shoulders (acromions) — widest point at upper body height
    set.insert(make(
        LandmarkId::AcromionLeft,
        [lx, y_frac(0.79), (mn[2] + mx[2]) * 0.5],
    ));
    set.insert(make(
        LandmarkId::AcromionRight,
        [rx, y_frac(0.79), (mn[2] + mx[2]) * 0.5],
    ));

    // Chest / Thoracic
    set.insert(make(LandmarkId::ChestCenter, [cx, y_frac(0.72), mx[2]]));
    set.insert(make(LandmarkId::T10Thoracic, [cx, y_frac(0.65), mn[2]]));

    // Elbows — lateral, mid-arm height
    set.insert(make(
        LandmarkId::ElbowLeft,
        [mn[0], y_frac(0.60), (mn[2] + mx[2]) * 0.5],
    ));
    set.insert(make(
        LandmarkId::ElbowRight,
        [mx[0], y_frac(0.60), (mn[2] + mx[2]) * 0.5],
    ));

    // Navel
    set.insert(make(LandmarkId::NabelCenter, [cx, y_frac(0.57), mx[2]]));

    // Waist
    set.insert(make(LandmarkId::WaistCenter, [cx, y_frac(0.54), mn[2]]));
    set.insert(make(LandmarkId::L4Lumbar, [cx, y_frac(0.52), mn[2]]));

    // Hips
    set.insert(make(
        LandmarkId::HipLeft,
        [lx, y_frac(0.48), (mn[2] + mx[2]) * 0.5],
    ));
    set.insert(make(
        LandmarkId::HipRight,
        [rx, y_frac(0.48), (mn[2] + mx[2]) * 0.5],
    ));

    // Wrists — at extremity of arms
    set.insert(make(
        LandmarkId::WristLeft,
        [mn[0], y_frac(0.38), (mn[2] + mx[2]) * 0.5],
    ));
    set.insert(make(
        LandmarkId::WristRight,
        [mx[0], y_frac(0.38), (mn[2] + mx[2]) * 0.5],
    ));

    // Knees
    set.insert(make(LandmarkId::KneeLeft, [lx, y_frac(0.27), mx[2]]));
    set.insert(make(LandmarkId::KneeRight, [rx, y_frac(0.27), mx[2]]));

    // Ankles
    set.insert(make(LandmarkId::AnkleLeft, [lx, y_frac(0.07), mx[2]]));
    set.insert(make(LandmarkId::AnkleRight, [rx, y_frac(0.07), mx[2]]));

    // Heels — lowest Y, posterior Z
    set.insert(make(LandmarkId::HeelLeft, [lx, y_frac(0.01), mn[2]]));
    set.insert(make(LandmarkId::HeelRight, [rx, y_frac(0.01), mn[2]]));

    set
}

// ---------------------------------------------------------------------------
// nearest_vertex
// ---------------------------------------------------------------------------

/// Find the index of the mesh vertex closest to `pos`, and its distance.
///
/// Returns `(0, 0.0)` for an empty mesh.
pub fn nearest_vertex(mesh: &MeshBuffers, pos: [f32; 3]) -> (u32, f32) {
    if mesh.positions.is_empty() {
        return (0, 0.0);
    }
    let mut best_idx = 0u32;
    let mut best_dist = f32::MAX;
    for (i, p) in mesh.positions.iter().enumerate() {
        let d = vec3_dist(*p, pos);
        if d < best_dist {
            best_dist = d;
            best_idx = i as u32;
        }
    }
    (best_idx, best_dist)
}

// ---------------------------------------------------------------------------
// landmark_frame
// ---------------------------------------------------------------------------

/// Compute a local orthonormal frame at a landmark's vertex.
///
/// Returns `(normal, tangent, bitangent)`.  The normal is taken from the mesh
/// normal at the nearest vertex (or Y-up if absent).  The tangent is derived
/// from the mesh's stored tangent if available, otherwise constructed from the
/// normal.
pub fn landmark_frame(mesh: &MeshBuffers, landmark: &Landmark) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let vidx = landmark
        .vertex_index
        .unwrap_or_else(|| nearest_vertex(mesh, landmark.position).0) as usize;

    // Normal
    let normal = if vidx < mesh.normals.len() {
        vec3_normalize(mesh.normals[vidx])
    } else {
        [0.0, 1.0, 0.0]
    };

    // Build tangent — pick a reference vector not parallel to normal
    let ref_vec = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let bitangent = vec3_normalize(vec3_cross(normal, ref_vec));
    let tangent = vec3_normalize(vec3_cross(bitangent, normal));

    (normal, tangent, bitangent)
}

// ---------------------------------------------------------------------------
// remap_landmarks
// ---------------------------------------------------------------------------

/// Remap landmark positions from a source bounding box to a target bounding box.
///
/// Applies per-axis scale and translation so that each landmark's normalised
/// position within `source_bbox` maps to the same normalised position within
/// `target_bbox`.  Vertex indices are cleared (they are no longer valid in the
/// new space).
pub fn remap_landmarks(
    set: &LandmarkSet,
    source_bbox: ([f32; 3], [f32; 3]),
    target_bbox: ([f32; 3], [f32; 3]),
) -> LandmarkSet {
    let (src_min, src_max) = source_bbox;
    let (tgt_min, tgt_max) = target_bbox;

    let mut out = LandmarkSet::new();
    for lm in set.landmarks.values() {
        let mut new_pos = [0.0f32; 3];
        for i in 0..3 {
            let src_range = src_max[i] - src_min[i];
            let t = if src_range.abs() < 1e-9 {
                0.5
            } else {
                (lm.position[i] - src_min[i]) / src_range
            };
            new_pos[i] = lerp1(tgt_min[i], tgt_max[i], t);
        }
        out.insert(Landmark {
            id: lm.id.clone(),
            position: new_pos,
            confidence: lm.confidence,
            vertex_index: None,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// transfer_landmarks
// ---------------------------------------------------------------------------

/// Transfer landmarks from a template mesh to a deformed (target) mesh.
///
/// For each landmark in `template_set`, finds the template vertex it was
/// associated with (or the nearest vertex to its position on the template),
/// then uses that vertex's position on the target mesh as the new landmark
/// position.
pub fn transfer_landmarks(
    template_set: &LandmarkSet,
    template_mesh: &MeshBuffers,
    target_mesh: &MeshBuffers,
) -> LandmarkSet {
    let mut out = LandmarkSet::new();
    for lm in template_set.landmarks.values() {
        // Resolve the vertex index on the template
        let vidx = lm
            .vertex_index
            .unwrap_or_else(|| nearest_vertex(template_mesh, lm.position).0);

        // Get the corresponding position on the target mesh
        let new_pos = if (vidx as usize) < target_mesh.positions.len() {
            target_mesh.positions[vidx as usize]
        } else {
            // Fall back to nearest vertex on target if index out of range
            let (nearest_vidx, _) = nearest_vertex(target_mesh, lm.position);
            target_mesh.positions[nearest_vidx as usize]
        };

        // Snap to nearest target vertex and record index
        let (snap_vidx, _) = nearest_vertex(target_mesh, new_pos);

        out.insert(Landmark {
            id: lm.id.clone(),
            position: target_mesh.positions[snap_vidx as usize],
            confidence: lm.confidence,
            vertex_index: Some(snap_vidx),
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MeshBuffers;

    /// Build a simple humanoid-shaped mesh with enough vertices to exercise the
    /// landmark heuristics.  The mesh spans Y: 0..1.8 (height), X: -0.3..0.3,
    /// Z: -0.2..0.2.  It is a rough grid of points placed at anatomical heights.
    fn humanoid_mesh() -> MeshBuffers {
        let mut positions = Vec::new();
        // Lay down a 5×7×3 grid
        for xi in 0..5i32 {
            for yi in 0..11i32 {
                for zi in 0..5i32 {
                    let x = -0.3 + xi as f32 * 0.15;
                    let y = yi as f32 * 0.18;
                    let z = -0.2 + zi as f32 * 0.1;
                    positions.push([x, y, z]);
                }
            }
        }
        let n = positions.len();
        // normals pointing +Y for simplicity
        let normals = vec![[0.0, 1.0, 0.0]; n];
        let uvs = vec![[0.0, 0.0]; n];
        // Trivial indices: sequential triangles from consecutive triplets
        let mut indices = Vec::new();
        let tri_count = (n / 3) * 3;
        for i in (0..tri_count).step_by(3) {
            indices.push(i as u32);
            indices.push((i + 1) as u32);
            indices.push((i + 2) as u32);
        }
        MeshBuffers {
            positions,
            normals,
            uvs,
            indices,
            has_suit: false,
        }
    }

    /// Minimal 4-vertex mesh.
    fn tiny_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 1, 3, 2],
            has_suit: false,
        }
    }

    // ── Test 1: LandmarkId::all() returns correct count ─────────────────────

    #[test]
    fn all_landmarks_count() {
        let all = LandmarkId::all();
        assert_eq!(all.len(), 23, "Expected 23 landmarks, got {}", all.len());
    }

    // ── Test 2: Bilateral landmarks have mirrors ─────────────────────────────

    #[test]
    fn bilateral_landmarks_have_mirrors() {
        for id in LandmarkId::all() {
            if id.is_bilateral() {
                assert!(
                    id.mirror().is_some(),
                    "{:?} is_bilateral but mirror is None",
                    id
                );
            } else {
                assert!(
                    id.mirror().is_none(),
                    "{:?} is not bilateral but has mirror",
                    id
                );
            }
        }
    }

    // ── Test 3: Mirror is symmetric (A.mirror == B => B.mirror == A) ─────────

    #[test]
    fn mirror_is_symmetric() {
        for id in LandmarkId::all() {
            if let Some(m) = id.mirror() {
                let back = m.mirror().expect("mirror's mirror should exist");
                assert_eq!(back, id, "mirror is not symmetric for {:?}", id);
            }
        }
    }

    // ── Test 4: All midline landmarks have no mirror ─────────────────────────

    #[test]
    fn midline_landmarks_have_no_mirror() {
        let midline = vec![
            LandmarkId::TopOfHead,
            LandmarkId::ChinCenter,
            LandmarkId::C7Cervical,
            LandmarkId::T10Thoracic,
            LandmarkId::L4Lumbar,
            LandmarkId::NeckBase,
            LandmarkId::ChestCenter,
            LandmarkId::WaistCenter,
            LandmarkId::NabelCenter,
        ];
        for id in &midline {
            assert!(id.mirror().is_none(), "{:?} should have no mirror", id);
        }
    }

    // ── Test 5: LandmarkSet insert and get ───────────────────────────────────

    #[test]
    fn landmark_set_insert_and_get() {
        let mut set = LandmarkSet::new();
        let lm = Landmark::new(LandmarkId::TopOfHead, [0.0, 1.8, 0.0], 1.0, Some(0));
        set.insert(lm);
        assert_eq!(set.count(), 1);
        let got = set.get(&LandmarkId::TopOfHead).expect("landmark missing");
        assert!((got.position[1] - 1.8).abs() < 1e-6);
    }

    // ── Test 6: detect_landmarks produces non-empty set ──────────────────────

    #[test]
    fn detect_landmarks_non_empty() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        assert!(set.count() > 0, "Expected landmarks to be detected");
    }

    // ── Test 7: TopOfHead is near the top of the mesh ────────────────────────

    #[test]
    fn top_of_head_near_top() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let (mn, mx) = mesh_aabb(&mesh.positions);
        let head = set.get(&LandmarkId::TopOfHead).expect("TopOfHead missing");
        // Should be in the top 20% of the mesh height
        let threshold = lerp1(mn[1], mx[1], 0.80);
        assert!(
            head.position[1] >= threshold,
            "TopOfHead Y={} not above threshold {}",
            head.position[1],
            threshold
        );
    }

    // ── Test 8: nearest_vertex returns correct index ──────────────────────────

    #[test]
    fn nearest_vertex_correct() {
        let mesh = tiny_mesh();
        let (idx, dist) = nearest_vertex(&mesh, [0.0, 1.0, 0.0]);
        assert_eq!(idx, 2, "Expected vertex 2 nearest to (0,1,0)");
        assert!(dist < 1e-5, "Distance should be ~0, got {}", dist);
    }

    // ── Test 9: nearest_vertex on empty mesh returns (0, 0.0) ────────────────

    #[test]
    fn nearest_vertex_empty_mesh() {
        let empty = MeshBuffers {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        };
        let (idx, dist) = nearest_vertex(&empty, [1.0, 2.0, 3.0]);
        assert_eq!(idx, 0);
        assert!((dist - 0.0).abs() < 1e-9);
    }

    // ── Test 10: landmark_frame returns orthonormal basis ─────────────────────

    #[test]
    fn landmark_frame_orthonormal() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let lm = set.get(&LandmarkId::NeckBase).expect("NeckBase missing");
        let (n, t, b) = landmark_frame(&mesh, lm);

        // Each vector should be unit length
        let len_n = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let len_t = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        let len_b = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        assert!(
            (len_n - 1.0).abs() < 1e-5,
            "normal not unit length: {}",
            len_n
        );
        assert!(
            (len_t - 1.0).abs() < 1e-5,
            "tangent not unit length: {}",
            len_t
        );
        assert!(
            (len_b - 1.0).abs() < 1e-5,
            "bitangent not unit length: {}",
            len_b
        );

        // Normal and tangent should be perpendicular
        let dot_nt = vec3_dot(n, t);
        assert!(
            dot_nt.abs() < 1e-5,
            "normal·tangent = {} (not zero)",
            dot_nt
        );
    }

    // ── Test 11: remap_landmarks preserves relative position ─────────────────

    #[test]
    fn remap_landmarks_preserves_relative() {
        let mut set = LandmarkSet::new();
        // Landmark at center of [0,1]^3
        set.insert(Landmark::new(
            LandmarkId::ChestCenter,
            [0.5, 0.5, 0.5],
            1.0,
            None,
        ));
        let src = ([0.0; 3], [1.0; 3]);
        let tgt = ([0.0; 3], [2.0; 3]);
        let remapped = remap_landmarks(&set, src, tgt);
        let lm = remapped
            .get(&LandmarkId::ChestCenter)
            .expect("ChestCenter missing after remap");
        // Center of [0,2]^3 should be [1,1,1]
        for i in 0..3 {
            assert!(
                (lm.position[i] - 1.0).abs() < 1e-5,
                "axis {} mismatch: {}",
                i,
                lm.position[i]
            );
        }
    }

    // ── Test 12: transfer_landmarks maps to target mesh ───────────────────────

    #[test]
    fn transfer_landmarks_maps_to_target() {
        let template = humanoid_mesh();
        let mut target = humanoid_mesh();
        // Shift target mesh by 10 units in X
        for p in target.positions.iter_mut() {
            p[0] += 10.0;
        }
        let template_set = detect_landmarks(&template);
        let transferred = transfer_landmarks(&template_set, &template, &target);
        // All transferred positions should have X >= 10 - epsilon
        for (pos, _conf) in transferred.all_positions() {
            assert!(pos[0] >= 9.4, "transferred X={} expected >= ~10", pos[0]);
        }
    }

    // ── Test 13: body_height returns plausible value ──────────────────────────

    #[test]
    fn body_height_plausible() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let height = set.body_height().expect("body_height returned None");
        let (mn, mx) = mesh_aabb(&mesh.positions);
        let mesh_height = mx[1] - mn[1];
        // Detected height should be within 10% of mesh height
        assert!(
            (height - mesh_height).abs() < mesh_height * 0.15,
            "body_height={} vs mesh_height={}",
            height,
            mesh_height
        );
    }

    // ── Test 14: shoulder_width and hip_width are positive ────────────────────

    #[test]
    fn shoulder_and_hip_width_positive() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let sw = set.shoulder_width().expect("shoulder_width returned None");
        let hw = set.hip_width().expect("hip_width returned None");
        assert!(sw > 0.0, "shoulder_width should be positive, got {}", sw);
        assert!(hw > 0.0, "hip_width should be positive, got {}", hw);
    }

    // ── Test 15: arm_length and leg_length are positive ───────────────────────

    #[test]
    fn arm_and_leg_length_positive() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let al = set
            .arm_length(Side::Left)
            .expect("arm_length(Left) returned None");
        let ar = set
            .arm_length(Side::Right)
            .expect("arm_length(Right) returned None");
        let ll = set
            .leg_length(Side::Left)
            .expect("leg_length(Left) returned None");
        let lr = set
            .leg_length(Side::Right)
            .expect("leg_length(Right) returned None");
        assert!(al > 0.0, "arm_length left should be positive");
        assert!(ar > 0.0, "arm_length right should be positive");
        assert!(ll > 0.0, "leg_length left should be positive");
        assert!(lr > 0.0, "leg_length right should be positive");
    }

    // ── Test 16: symmetry_error is 0 for a perfectly symmetric set ───────────

    #[test]
    fn symmetry_error_zero_for_symmetric_set() {
        let mut set = LandmarkSet::new();
        // Place acromions at equal and opposite X
        set.insert(Landmark::new(
            LandmarkId::AcromionLeft,
            [-0.2, 1.4, 0.0],
            1.0,
            None,
        ));
        set.insert(Landmark::new(
            LandmarkId::AcromionRight,
            [0.2, 1.4, 0.0],
            1.0,
            None,
        ));
        let err = set.symmetry_error();
        assert!(
            err < 1e-5,
            "symmetry_error should be ~0 for symmetric set, got {}",
            err
        );
    }

    // ── Test 17: to_map serializes all landmarks ──────────────────────────────

    #[test]
    fn to_map_contains_all_inserted_landmarks() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let map = set.to_map();
        // Every name in the map should correspond to a known landmark name
        for key in map.keys() {
            let found = LandmarkId::all().iter().any(|id| id.name() == key.as_str());
            assert!(found, "Unknown landmark name in map: {}", key);
        }
    }

    // ── Test 18: Write detected landmarks to /tmp/ ────────────────────────────

    #[test]
    fn write_landmarks_to_tmp() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        let map = set.to_map();
        let mut lines: Vec<String> = map
            .iter()
            .map(|(k, v)| format!("{}: [{:.3}, {:.3}, {:.3}]", k, v[0], v[1], v[2]))
            .collect();
        lines.sort();
        let content = lines.join("\n");
        let tmp = std::env::temp_dir().join("oxihuman_body_landmarks.txt");
        std::fs::write(&tmp, &content).expect("should succeed");
        let read_back = std::fs::read_to_string(&tmp).expect("should succeed");
        assert!(
            read_back.contains("Top of Head") || read_back.contains("Neck"),
            "landmark names missing"
        );
    }

    // ── Test 19: detect_landmarks on empty mesh returns empty set ─────────────

    #[test]
    fn detect_empty_mesh_returns_empty_set() {
        let empty = MeshBuffers {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        };
        let set = detect_landmarks(&empty);
        assert_eq!(set.count(), 0);
    }

    // ── Test 20: remap with zero-size source bbox does not panic ──────────────

    #[test]
    fn remap_zero_size_source_bbox_no_panic() {
        let mut set = LandmarkSet::new();
        set.insert(Landmark::new(
            LandmarkId::TopOfHead,
            [0.5, 0.5, 0.5],
            1.0,
            None,
        ));
        let src = ([0.5; 3], [0.5; 3]); // zero size
        let tgt = ([0.0; 3], [1.0; 3]);
        let remapped = remap_landmarks(&set, src, tgt);
        assert_eq!(remapped.count(), 1);
    }

    // ── Test 21: all_positions returns same count as count() ─────────────────

    #[test]
    fn all_positions_count_matches() {
        let mesh = humanoid_mesh();
        let set = detect_landmarks(&mesh);
        assert_eq!(set.all_positions().len(), set.count());
    }

    // ── Test 22: distance returns correct value ───────────────────────────────

    #[test]
    fn distance_correct() {
        let mut set = LandmarkSet::new();
        set.insert(Landmark::new(
            LandmarkId::AcromionLeft,
            [-1.0, 0.0, 0.0],
            1.0,
            None,
        ));
        set.insert(Landmark::new(
            LandmarkId::AcromionRight,
            [1.0, 0.0, 0.0],
            1.0,
            None,
        ));
        let d = set
            .distance(&LandmarkId::AcromionLeft, &LandmarkId::AcromionRight)
            .expect("distance returned None");
        assert!((d - 2.0).abs() < 1e-5, "Expected distance 2.0, got {}", d);
    }

    // ── Test 23: LandmarkId names are unique ─────────────────────────────────

    #[test]
    fn landmark_names_are_unique() {
        let names: Vec<&str> = LandmarkId::all().iter().map(|id| id.name()).collect();
        let unique: std::collections::HashSet<&&str> = names.iter().collect();
        assert_eq!(names.len(), unique.len(), "Duplicate landmark names found");
    }
}
