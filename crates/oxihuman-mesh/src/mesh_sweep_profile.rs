// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generalized profile sweep along an arbitrary 3D path.
//!
//! A 2D profile (polygon) is swept along a 3D path using a
//! Frenet-Serret-inspired frame. The resulting mesh has the profile
//! extruded at every path point, connected with quads.
//!
//! Note: distinct from `mesh_sweep` which provides simpler circle/rect
//! profiles. This module supports arbitrary polygon profiles with
//! optional end caps and path-proportional scaling.

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Configuration for generalized profile sweep.
pub struct SweepProfileConfig {
    /// If true, add triangulated caps at both ends of the sweep.
    pub close_caps: bool,
    /// If true, scale the profile uniformly along the path (linear ramp).
    pub scale_along_path: bool,
    /// Additional twist (radians) applied to the profile frame over the full path.
    pub twist_radians: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
/// A 2D vertex in the profile polygon.
pub struct ProfileVertex {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// A 3D sweep path with precomputed tangents.
pub struct SweepProfilePath {
    /// World-space positions along the path.
    pub points: Vec<[f32; 3]>,
    /// Normalised tangent at each path point.
    pub tangents: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Generated sweep mesh.
pub struct SweepProfileMesh {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// UV coordinates per vertex.
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

// ─── internal vector math ───────────────────────────────────────────────────

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub3(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ─── public helpers ──────────────────────────────────────────────────────────

/// Return default sweep profile configuration.
#[allow(dead_code)]
pub fn default_sweep_config() -> SweepProfileConfig {
    SweepProfileConfig {
        close_caps: false,
        scale_along_path: false,
        twist_radians: 0.0,
    }
}

/// Build a sweep path from raw points, computing finite-difference tangents.
#[allow(dead_code)]
pub fn build_sweep_path(pts: &[[f32; 3]]) -> SweepProfilePath {
    let n = pts.len();
    let mut tangents = Vec::with_capacity(n);

    for i in 0..n {
        let t = if i == 0 && n >= 2 {
            normalize3(sub3(pts[1], pts[0]))
        } else if i + 1 == n && n >= 2 {
            normalize3(sub3(pts[n - 1], pts[n - 2]))
        } else if n < 2 {
            [0.0, 0.0, 1.0]
        } else {
            normalize3(sub3(pts[i + 1], pts[i - 1]))
        };
        tangents.push(t);
    }

    SweepProfilePath {
        points: pts.to_vec(),
        tangents,
    }
}

/// Compute the perimeter of a 2D profile polygon.
#[allow(dead_code)]
pub fn sweep_profile_perimeter(profile: &[ProfileVertex]) -> f32 {
    let n = profile.len();
    if n < 2 {
        return 0.0;
    }
    profile
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let next = &profile[(i + 1) % n];
            let dx = next.x - v.x;
            let dy = next.y - v.y;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

/// Compute a stable normal-plane frame (normal, binormal) from a tangent,
/// propagated from the previous frame for continuity.
fn build_frame(tangent: [f32; 3], prev_normal: Option<[f32; 3]>) -> ([f32; 3], [f32; 3]) {
    let n_raw = match prev_normal {
        None => {
            // Bootstrap: pick a world vector not parallel to tangent
            let world_up = [0.0, 1.0, 0.0];
            let world_x = [1.0, 0.0, 0.0];
            let v = if dot3(tangent, world_up).abs() < 0.9 {
                world_up
            } else {
                world_x
            };
            normalize3(cross3(tangent, v))
        }
        Some(prev_n) => {
            // Project prev normal onto the plane perpendicular to tangent
            let proj = sub3(prev_n, scale3(tangent, dot3(prev_n, tangent)));
            normalize3(proj)
        }
    };
    let binormal = normalize3(cross3(tangent, n_raw));
    let normal = normalize3(cross3(binormal, tangent));
    (normal, binormal)
}

/// Sweep a 2D profile along a 3D path and return the resulting mesh.
#[allow(dead_code)]
pub fn sweep_profile(
    profile: &[ProfileVertex],
    path: &SweepProfilePath,
    cfg: &SweepProfileConfig,
) -> SweepProfileMesh {
    let np = profile.len();
    let ns = path.points.len();

    if np < 2 || ns < 2 {
        return SweepProfileMesh {
            positions: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        };
    }

    // Compute arc lengths for V coordinate
    let mut arc_lens = vec![0.0_f32; ns];
    for i in 1..ns {
        arc_lens[i] = arc_lens[i - 1] + dist3(path.points[i], path.points[i - 1]);
    }
    let total_len = arc_lens[ns - 1].max(1e-9);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Build profile U coordinates
    let profile_len = sweep_profile_perimeter(profile).max(1e-9);
    let mut u_coords = Vec::with_capacity(np);
    {
        let mut acc = 0.0_f32;
        for i in 0..np {
            u_coords.push(acc / profile_len);
            let next = (i + 1) % np;
            let dx = profile[next].x - profile[i].x;
            let dy = profile[next].y - profile[i].y;
            acc += (dx * dx + dy * dy).sqrt();
        }
    }

    // Propagate frame along path
    let mut prev_normal: Option<[f32; 3]> = None;

    for (s, (&tangent, &arc_len_s)) in path.tangents[..ns]
        .iter()
        .zip(arc_lens[..ns].iter())
        .enumerate()
    {
        let (normal, binormal) = build_frame(tangent, prev_normal);
        prev_normal = Some(normal);

        let v_coord = arc_len_s / total_len;

        // Optional scale ramp
        let scale = if cfg.scale_along_path {
            let t = v_coord * 2.0;
            if t < 1.0 { t } else { 2.0 - t }
        } else {
            1.0
        };

        // Twist
        let twist_angle = cfg.twist_radians * v_coord;
        let cos_t = twist_angle.cos();
        let sin_t = twist_angle.sin();

        for (pi, pv) in profile.iter().enumerate() {
            // Apply twist to profile vertex
            let px = pv.x * cos_t - pv.y * sin_t;
            let py = pv.x * sin_t + pv.y * cos_t;

            let world_pos = add3(
                path.points[s],
                add3(scale3(normal, px * scale), scale3(binormal, py * scale)),
            );
            positions.push(world_pos);
            uvs.push([u_coords[pi], v_coord]);
        }

        // Emit quads between ring s-1 and ring s
        if s > 0 {
            let base_prev = ((s - 1) * np) as u32;
            let base_cur = (s * np) as u32;
            for pi in 0..np as u32 {
                let pi_next = (pi + 1) % np as u32;
                // quad: prev[pi], prev[pi+1], cur[pi+1], cur[pi]
                indices.push(base_prev + pi);
                indices.push(base_prev + pi_next);
                indices.push(base_cur + pi_next);

                indices.push(base_prev + pi);
                indices.push(base_cur + pi_next);
                indices.push(base_cur + pi);
            }
        }
    }

    // Optional end caps (fan triangulation)
    if cfg.close_caps {
        // Front cap (first ring) — fan around ring centre
        let ring0_base = 0u32;
        for pi in 0..np as u32 {
            let pi_next = (pi + 1) % np as u32;
            // Centre approximation: use first profile vertex 0 as hub
            if pi > 0 && pi_next != 0 {
                indices.push(ring0_base);
                indices.push(ring0_base + pi_next);
                indices.push(ring0_base + pi);
            }
        }
        // Back cap (last ring)
        let ring_n_base = ((ns - 1) * np) as u32;
        for pi in 0..np as u32 {
            let pi_next = (pi + 1) % np as u32;
            if pi > 0 && pi_next != 0 {
                indices.push(ring_n_base);
                indices.push(ring_n_base + pi);
                indices.push(ring_n_base + pi_next);
            }
        }
    }

    SweepProfileMesh {
        positions,
        uvs,
        indices,
    }
}

/// Return total vertex count in a sweep profile mesh.
#[allow(dead_code)]
pub fn sweep_vertex_count(mesh: &SweepProfileMesh) -> usize {
    mesh.positions.len()
}

/// Serialize sweep profile mesh to compact JSON.
#[allow(dead_code)]
pub fn sweep_mesh_to_json(mesh: &SweepProfileMesh) -> String {
    format!(
        "{{\"vertex_count\":{},\"index_count\":{}}}",
        mesh.positions.len(),
        mesh.indices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_profile() -> Vec<ProfileVertex> {
        vec![
            ProfileVertex { x: -0.5, y: -0.5 },
            ProfileVertex { x: 0.5, y: -0.5 },
            ProfileVertex { x: 0.5, y: 0.5 },
            ProfileVertex { x: -0.5, y: 0.5 },
        ]
    }

    fn straight_path(n: usize) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32, 0.0, 0.0]).collect()
    }

    #[test]
    fn test_default_config() {
        let cfg = default_sweep_config();
        assert!(!cfg.close_caps);
        assert!(!cfg.scale_along_path);
        assert!((cfg.twist_radians).abs() < 1e-6);
    }

    #[test]
    fn test_build_sweep_path_tangents() {
        let pts = straight_path(4);
        let path = build_sweep_path(&pts);
        assert_eq!(path.tangents.len(), 4);
        // All tangents should point along X
        for t in &path.tangents {
            assert!((t[0] - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_build_sweep_path_single() {
        let pts = vec![[0.0, 0.0, 0.0]];
        let path = build_sweep_path(&pts);
        assert_eq!(path.tangents.len(), 1);
    }

    #[test]
    fn test_sweep_profile_perimeter_square() {
        let prof = square_profile();
        let perim = sweep_profile_perimeter(&prof);
        // 4 sides of length 1.0
        assert!((perim - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_sweep_profile_perimeter_empty() {
        assert!((sweep_profile_perimeter(&[])).abs() < 1e-6);
    }

    #[test]
    fn test_sweep_profile_basic_vertex_count() {
        let prof = square_profile(); // 4 vertices
        let pts = straight_path(5);
        let path = build_sweep_path(&pts);
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&prof, &path, &cfg);
        // 5 rings × 4 vertices = 20
        assert_eq!(mesh.positions.len(), 20);
    }

    #[test]
    fn test_sweep_profile_basic_index_count() {
        let prof = square_profile(); // 4 verts
        let pts = straight_path(5);
        let path = build_sweep_path(&pts);
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&prof, &path, &cfg);
        // (5-1) segments × 4 quads × 6 indices = 96
        assert_eq!(mesh.indices.len(), 96);
    }

    #[test]
    fn test_sweep_profile_empty_profile() {
        let path = build_sweep_path(&straight_path(3));
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&[], &path, &cfg);
        assert_eq!(mesh.positions.len(), 0);
    }

    #[test]
    fn test_sweep_profile_empty_path() {
        let prof = square_profile();
        let path = build_sweep_path(&[]);
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&prof, &path, &cfg);
        assert_eq!(mesh.positions.len(), 0);
    }

    #[test]
    fn test_sweep_profile_caps() {
        let prof = square_profile();
        let pts = straight_path(4);
        let path = build_sweep_path(&pts);
        let cfg = SweepProfileConfig {
            close_caps: true,
            scale_along_path: false,
            twist_radians: 0.0,
        };
        let mesh = sweep_profile(&prof, &path, &cfg);
        // vertex count stays 4×4=16
        assert_eq!(mesh.positions.len(), 16);
        // indices should be more than without caps
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn test_sweep_profile_scale_along_path() {
        let prof = square_profile();
        let pts = straight_path(6);
        let path = build_sweep_path(&pts);
        let cfg = SweepProfileConfig {
            close_caps: false,
            scale_along_path: true,
            twist_radians: 0.0,
        };
        let mesh = sweep_profile(&prof, &path, &cfg);
        assert_eq!(mesh.positions.len(), 24);
    }

    #[test]
    fn test_sweep_profile_twist() {
        let prof = square_profile();
        let pts = straight_path(4);
        let path = build_sweep_path(&pts);
        let cfg = SweepProfileConfig {
            close_caps: false,
            scale_along_path: false,
            twist_radians: std::f32::consts::PI,
        };
        let mesh = sweep_profile(&prof, &path, &cfg);
        assert_eq!(mesh.positions.len(), 16);
    }

    #[test]
    fn test_sweep_vertex_count() {
        let prof = square_profile();
        let path = build_sweep_path(&straight_path(3));
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&prof, &path, &cfg);
        assert_eq!(sweep_vertex_count(&mesh), mesh.positions.len());
    }

    #[test]
    fn test_sweep_mesh_to_json() {
        let prof = square_profile();
        let path = build_sweep_path(&straight_path(3));
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&prof, &path, &cfg);
        let json = sweep_mesh_to_json(&mesh);
        assert!(json.contains("vertex_count"));
        assert!(json.contains("index_count"));
    }

    #[test]
    fn test_indices_in_range() {
        let prof = square_profile();
        let pts = straight_path(5);
        let path = build_sweep_path(&pts);
        let cfg = default_sweep_config();
        let mesh = sweep_profile(&prof, &path, &cfg);
        let vcount = mesh.positions.len() as u32;
        for &idx in &mesh.indices {
            assert!(idx < vcount, "index {} out of range {}", idx, vcount);
        }
    }
}
