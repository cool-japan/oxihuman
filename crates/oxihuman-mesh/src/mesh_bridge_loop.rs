// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bridge two edge loops by generating connecting faces.

/// Configuration for the bridge loop operation.
#[derive(Debug, Clone)]
pub struct BridgeLoopConfig {
    /// Number of intermediate cuts between the two loops.
    pub cuts: usize,
    /// If true, merge vertices that are closer than `merge_threshold`.
    pub merge: bool,
    pub merge_threshold: f32,
}

impl Default for BridgeLoopConfig {
    fn default() -> Self {
        Self { cuts: 0, merge: false, merge_threshold: 1e-4 }
    }
}

/// Result of bridging two loops.
#[derive(Debug, Clone, Default)]
pub struct BridgeLoopResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub bridge_face_count: usize,
}

/// Bridge two vertex loops with quads (returned as triangles).
///
/// `loop_a` and `loop_b` are vertex-index slices into `positions`.
pub fn bridge_vertex_loops(
    positions: &[[f32; 3]],
    indices: &[u32],
    loop_a: &[usize],
    loop_b: &[usize],
    cfg: &BridgeLoopConfig,
) -> BridgeLoopResult {
    let na = loop_a.len();
    let nb = loop_b.len();
    if na == 0 || nb == 0 {
        return BridgeLoopResult {
            positions: positions.to_vec(),
            indices: indices.to_vec(),
            bridge_face_count: 0,
        };
    }
    let n = na.max(nb);
    let mut out_pos = positions.to_vec();
    let mut out_idx = indices.to_vec();
    let mut face_count = 0usize;

    /* generate intermediate loops via lerp if cuts > 0 */
    let total_layers = cfg.cuts + 1;
    let mut layers: Vec<Vec<usize>> = Vec::with_capacity(total_layers + 1);
    layers.push(loop_a.to_vec());
    for c in 1..=cfg.cuts {
        let t = c as f32 / total_layers as f32;
        let mut layer_verts = Vec::with_capacity(n);
        for k in 0..n {
            let ia = loop_a[k % na];
            let ib = loop_b[k % nb];
            let pa = positions[ia];
            let pb = positions[ib];
            let p = lerp3(pa, pb, t);
            out_pos.push(p);
            layer_verts.push(out_pos.len() - 1);
        }
        layers.push(layer_verts);
    }
    layers.push(loop_b.to_vec());

    /* generate quads between consecutive layers */
    for layer in 0..layers.len().saturating_sub(1) {
        let la = &layers[layer];
        let lb = &layers[layer + 1];
        let na2 = la.len();
        let nb2 = lb.len();
        let n2 = na2.max(nb2);
        for k in 0..n2 {
            let a0 = la[k % na2] as u32;
            let a1 = la[(k + 1) % na2] as u32;
            let b0 = lb[k % nb2] as u32;
            let b1 = lb[(k + 1) % nb2] as u32;
            /* quad as two tris */
            out_idx.extend_from_slice(&[a0, b0, b1]);
            out_idx.extend_from_slice(&[a0, b1, a1]);
            face_count += 2;
        }
    }

    BridgeLoopResult { positions: out_pos, indices: out_idx, bridge_face_count: face_count }
}

/// Default bridge loop configuration.
pub fn default_bridge_loop_config() -> BridgeLoopConfig {
    BridgeLoopConfig::default()
}

/// Number of faces added by bridging.
pub fn bridge_face_count(r: &BridgeLoopResult) -> usize {
    r.bridge_face_count
}

/// Centroid of a vertex loop.
pub fn loop_centroid_bl(positions: &[[f32; 3]], loop_verts: &[usize]) -> [f32; 3] {
    if loop_verts.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = loop_verts.len() as f32;
    let mut c = [0.0f32; 3];
    for &vi in loop_verts {
        if vi < positions.len() {
            c[0] += positions[vi][0];
            c[1] += positions[vi][1];
            c[2] += positions[vi][2];
        }
    }
    [c[0] / n, c[1] / n, c[2] / n]
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_quads() -> (Vec<[f32; 3]>, Vec<u32>) {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ];
        let i = vec![];
        (p, i)
    }

    #[test]
    fn test_bridge_adds_faces() {
        /* bridging two loops of 4 verts creates 8 triangle faces */
        let (p, i) = two_quads();
        let la = vec![0, 1, 2, 3];
        let lb = vec![4, 5, 6, 7];
        let r = bridge_vertex_loops(&p, &i, &la, &lb, &default_bridge_loop_config());
        assert!(bridge_face_count(&r) > 0);
    }

    #[test]
    fn test_bridge_empty_loop_a() {
        /* empty loop_a returns unmodified geometry */
        let (p, i) = two_quads();
        let r = bridge_vertex_loops(&p, &i, &[], &[4, 5], &default_bridge_loop_config());
        assert_eq!(bridge_face_count(&r), 0);
    }

    #[test]
    fn test_bridge_empty_loop_b() {
        /* empty loop_b returns unmodified geometry */
        let (p, i) = two_quads();
        let r = bridge_vertex_loops(&p, &i, &[0, 1], &[], &default_bridge_loop_config());
        assert_eq!(bridge_face_count(&r), 0);
    }

    #[test]
    fn test_bridge_with_cuts() {
        /* more cuts produce more intermediate vertices */
        let (p, i) = two_quads();
        let la = vec![0, 1, 2, 3];
        let lb = vec![4, 5, 6, 7];
        let cfg = BridgeLoopConfig { cuts: 2, merge: false, merge_threshold: 1e-4 };
        let r = bridge_vertex_loops(&p, &i, &la, &lb, &cfg);
        assert!(r.positions.len() > p.len());
    }

    #[test]
    fn test_loop_centroid_bl() {
        /* centroid of unit square loop should be near center */
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let c = loop_centroid_bl(&p, &[0, 1, 2, 3]);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_loop_centroid_bl_empty() {
        /* empty loop returns origin */
        let p: Vec<[f32; 3]> = vec![];
        let c = loop_centroid_bl(&p, &[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_default_config_values() {
        /* default config has zero cuts and no merge */
        let cfg = default_bridge_loop_config();
        assert_eq!(cfg.cuts, 0);
        assert!(!cfg.merge);
    }

    #[test]
    fn test_bridge_positions_unchanged_for_orig() {
        /* original positions are preserved at beginning of output */
        let (p, i) = two_quads();
        let r = bridge_vertex_loops(&p, &i, &[0, 1], &[4, 5], &default_bridge_loop_config());
        assert_eq!(r.positions[0], p[0]);
    }

    #[test]
    fn test_bridge_face_count_zero_cuts_four_verts() {
        /* 0 cuts with 4-vert loops: 4 quads = 8 tris */
        let (p, i) = two_quads();
        let la = vec![0, 1, 2, 3];
        let lb = vec![4, 5, 6, 7];
        let r = bridge_vertex_loops(&p, &i, &la, &lb, &default_bridge_loop_config());
        assert_eq!(bridge_face_count(&r), 8);
    }

    #[test]
    fn test_lerp3_endpoints() {
        /* lerp at t=0 gives a, at t=1 gives b */
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let r0 = lerp3(a, b, 0.0);
        let r1 = lerp3(a, b, 1.0);
        assert!((r0[0] - a[0]).abs() < 1e-6);
        assert!((r1[0] - b[0]).abs() < 1e-6);
    }
}
