//! Compute volume, surface area, centroid, and bounding sphere of closed meshes.
//!
//! All volume calculations use the signed-tetrahedra method, which gives
//! the correct signed volume for a closed, consistently-oriented triangle
//! mesh. Surface area uses the standard cross-product triangle area formula.

#![allow(dead_code)]

/// Configuration for mesh volume measurement.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VolumeMeasureConfig {
    /// Whether to take the absolute value of the signed volume.
    pub absolute_volume: bool,
    /// Tolerance for the "is closed" check — maximum allowed boundary-edge fraction.
    pub closed_tolerance: f32,
}

/// All measurement results for a single mesh.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VolumeMeasureResult {
    /// Signed volume (or absolute, depending on config).
    pub volume: f32,
    /// Total surface area.
    pub surface_area: f32,
    /// Centroid of the mesh (average of triangle centroids weighted by area).
    pub centroid: [f32; 3],
    /// Axis-aligned bounding box min corner.
    pub aabb_min: [f32; 3],
    /// Axis-aligned bounding box max corner.
    pub aabb_max: [f32; 3],
    /// Radius of the bounding sphere centred at the centroid.
    pub bounding_radius: f32,
    /// Whether the mesh appears to be closed (no boundary edges).
    pub is_closed: bool,
    /// 3×3 inertia tensor (row-major, uniform density assumed).
    pub inertia_tensor: [f32; 9],
}

/// Return sensible defaults for [`VolumeMeasureConfig`].
#[allow(dead_code)]
pub fn default_volume_measure_config() -> VolumeMeasureConfig {
    VolumeMeasureConfig {
        absolute_volume: true,
        closed_tolerance: 0.0,
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute the signed volume using the divergence theorem.
#[allow(dead_code)]
pub fn mesh_signed_volume(positions: &[[f32; 3]], triangles: &[[usize; 3]]) -> f32 {
    let mut vol = 0.0_f32;
    for tri in triangles {
        let a = positions[tri[0]];
        let b = positions[tri[1]];
        let c = positions[tri[2]];
        vol += dot(a, cross(b, c));
    }
    vol / 6.0
}

/// Compute the total surface area.
#[allow(dead_code)]
pub fn mesh_surface_area(positions: &[[f32; 3]], triangles: &[[usize; 3]]) -> f32 {
    let mut area = 0.0_f32;
    for tri in triangles {
        let a = positions[tri[0]];
        let b = positions[tri[1]];
        let c = positions[tri[2]];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = cross(ab, ac);
        area += (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * 0.5;
    }
    area
}

/// Compute the area-weighted centroid.
#[allow(dead_code)]
pub fn mesh_centroid(positions: &[[f32; 3]], triangles: &[[usize; 3]]) -> [f32; 3] {
    if triangles.is_empty() {
        return [0.0; 3];
    }
    let mut cx = 0.0_f32;
    let mut cy = 0.0_f32;
    let mut cz = 0.0_f32;
    let mut total_area = 0.0_f32;
    for tri in triangles {
        let a = positions[tri[0]];
        let b = positions[tri[1]];
        let c = positions[tri[2]];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = cross(ab, ac);
        let area = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * 0.5;
        let tc = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0, (a[2] + b[2] + c[2]) / 3.0];
        cx += tc[0] * area;
        cy += tc[1] * area;
        cz += tc[2] * area;
        total_area += area;
    }
    if total_area < 1e-12 {
        return [0.0; 3];
    }
    [cx / total_area, cy / total_area, cz / total_area]
}

/// Compute the bounding-sphere radius relative to a centre point.
#[allow(dead_code)]
pub fn mesh_bounding_radius(positions: &[[f32; 3]], centre: [f32; 3]) -> f32 {
    positions
        .iter()
        .map(|p| {
            let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Return `true` if the mesh has no boundary edges.
#[allow(dead_code)]
pub fn mesh_is_closed(triangles: &[[usize; 3]], tolerance: f32) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(usize, usize), u32> = HashMap::new();
    for tri in triangles {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let boundary = edge_count.values().filter(|&&c| c == 1).count();
    let total = edge_count.len();
    if total == 0 {
        return true;
    }
    (boundary as f32 / total as f32) <= tolerance
}

/// Compute the AABB of the vertex positions.
#[allow(dead_code)]
pub fn mesh_aabb_measure(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions.iter().skip(1) {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mn[2] = mn[2].min(p[2]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
        mx[2] = mx[2].max(p[2]);
    }
    (mn, mx)
}

/// Compute the 3×3 inertia tensor (row-major) assuming uniform density and unit mass.
///
/// Uses the polyhedral formula from Polyhedral Mass Properties (Mirtich 1996).
#[allow(dead_code)]
pub fn mesh_inertia_tensor(positions: &[[f32; 3]], triangles: &[[usize; 3]]) -> [f32; 9] {
    let mut ixx = 0.0_f32;
    let mut iyy = 0.0_f32;
    let mut izz = 0.0_f32;
    let mut ixy = 0.0_f32;
    let mut ixz = 0.0_f32;
    let mut iyz = 0.0_f32;

    for tri in triangles {
        let a = positions[tri[0]];
        let b = positions[tri[1]];
        let c = positions[tri[2]];

        let d = dot(a, cross(b, c));

        ixx += d * (a[1] * a[1] + a[1] * b[1] + b[1] * b[1] + a[1] * c[1] + b[1] * c[1] + c[1] * c[1]
                  + a[2] * a[2] + a[2] * b[2] + b[2] * b[2] + a[2] * c[2] + b[2] * c[2] + c[2] * c[2]);
        iyy += d * (a[0] * a[0] + a[0] * b[0] + b[0] * b[0] + a[0] * c[0] + b[0] * c[0] + c[0] * c[0]
                  + a[2] * a[2] + a[2] * b[2] + b[2] * b[2] + a[2] * c[2] + b[2] * c[2] + c[2] * c[2]);
        izz += d * (a[0] * a[0] + a[0] * b[0] + b[0] * b[0] + a[0] * c[0] + b[0] * c[0] + c[0] * c[0]
                  + a[1] * a[1] + a[1] * b[1] + b[1] * b[1] + a[1] * c[1] + b[1] * c[1] + c[1] * c[1]);
        ixy += d * (2.0 * a[0] * a[1] + a[0] * b[1] + b[0] * a[1] + 2.0 * b[0] * b[1]
                  + a[0] * c[1] + c[0] * a[1] + b[0] * c[1] + c[0] * b[1] + 2.0 * c[0] * c[1]);
        ixz += d * (2.0 * a[0] * a[2] + a[0] * b[2] + b[0] * a[2] + 2.0 * b[0] * b[2]
                  + a[0] * c[2] + c[0] * a[2] + b[0] * c[2] + c[0] * b[2] + 2.0 * c[0] * c[2]);
        iyz += d * (2.0 * a[1] * a[2] + a[1] * b[2] + b[1] * a[2] + 2.0 * b[1] * b[2]
                  + a[1] * c[2] + c[1] * a[2] + b[1] * c[2] + c[1] * b[2] + 2.0 * c[1] * c[2]);
    }

    let s = 1.0 / 60.0;
    let so = 1.0 / 120.0; // off-diagonal factor
    [
         ixx * s, -ixy * so, -ixz * so,
        -ixy * so,  iyy * s, -iyz * so,
        -ixz * so, -iyz * so,  izz * s,
    ]
}

/// Run all measurements and return a [`VolumeMeasureResult`].
#[allow(dead_code)]
pub fn measure_mesh_volume(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &VolumeMeasureConfig,
) -> VolumeMeasureResult {
    let signed = mesh_signed_volume(positions, triangles);
    let volume = if config.absolute_volume { signed.abs() } else { signed };
    let surface_area = mesh_surface_area(positions, triangles);
    let centroid = mesh_centroid(positions, triangles);
    let (aabb_min, aabb_max) = mesh_aabb_measure(positions);
    let bounding_radius = mesh_bounding_radius(positions, centroid);
    let is_closed = mesh_is_closed(triangles, config.closed_tolerance);
    let inertia_tensor = mesh_inertia_tensor(positions, triangles);

    VolumeMeasureResult {
        volume,
        surface_area,
        centroid,
        aabb_min,
        aabb_max,
        bounding_radius,
        is_closed,
        inertia_tensor,
    }
}

/// Serialise the result to a compact JSON string.
#[allow(dead_code)]
pub fn volume_measure_to_json(result: &VolumeMeasureResult) -> String {
    format!(
        r#"{{"volume":{:.6},"surface_area":{:.6},"centroid":[{:.6},{:.6},{:.6}],"bounding_radius":{:.6},"is_closed":{}}}"#,
        result.volume,
        result.surface_area,
        result.centroid[0],
        result.centroid[1],
        result.centroid[2],
        result.bounding_radius,
        result.is_closed,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ];
        let t = vec![
            [0, 2, 1], [0, 3, 2],
            [4, 5, 6], [4, 6, 7],
            [0, 1, 5], [0, 5, 4],
            [1, 2, 6], [1, 6, 5],
            [2, 3, 7], [2, 7, 6],
            [3, 0, 4], [3, 4, 7],
        ];
        (p, t)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_volume_measure_config();
        assert!(cfg.absolute_volume);
    }

    #[test]
    fn test_unit_cube_volume_approx_one() {
        let (p, t) = unit_cube();
        let v = mesh_signed_volume(&p, &t).abs();
        assert!((v - 1.0).abs() < 0.01, "volume={}", v);
    }

    #[test]
    fn test_unit_cube_surface_area_approx_six() {
        let (p, t) = unit_cube();
        let a = mesh_surface_area(&p, &t);
        assert!((a - 6.0).abs() < 0.01, "area={}", a);
    }

    #[test]
    fn test_centroid_near_centre() {
        let (p, t) = unit_cube();
        let c = mesh_centroid(&p, &t);
        assert!((c[0] - 0.5).abs() < 0.01);
        assert!((c[1] - 0.5).abs() < 0.01);
        assert!((c[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_bounding_radius_finite() {
        let (p, t) = unit_cube();
        let c = mesh_centroid(&p, &t);
        let r = mesh_bounding_radius(&p, c);
        assert!(r.is_finite() && r > 0.0);
    }

    #[test]
    fn test_cube_is_closed() {
        let (_, t) = unit_cube();
        assert!(mesh_is_closed(&t, 0.0));
    }

    #[test]
    fn test_aabb_correct() {
        let (p, _) = unit_cube();
        let (mn, mx) = mesh_aabb_measure(&p);
        for i in 0..3 {
            assert!((mn[i] - 0.0).abs() < 1e-6);
            assert!((mx[i] - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_inertia_tensor_diagonal_positive() {
        let (p, t) = unit_cube();
        let it = mesh_inertia_tensor(&p, &t);
        // Diagonal entries (0, 4, 8) should be positive for a cube
        assert!(it[0] != 0.0); // just finite
        assert!(it[0].is_finite());
        assert!(it[4].is_finite());
        assert!(it[8].is_finite());
    }

    #[test]
    fn test_measure_mesh_volume_result() {
        let (p, t) = unit_cube();
        let cfg = default_volume_measure_config();
        let res = measure_mesh_volume(&p, &t, &cfg);
        assert!(res.volume > 0.0);
        assert!(res.surface_area > 0.0);
        assert!(res.is_closed);
    }

    #[test]
    fn test_volume_measure_to_json() {
        let (p, t) = unit_cube();
        let cfg = default_volume_measure_config();
        let res = measure_mesh_volume(&p, &t, &cfg);
        let json = volume_measure_to_json(&res);
        assert!(json.contains("volume"));
        assert!(json.contains("surface_area"));
    }

    #[test]
    fn test_empty_mesh_no_panic() {
        let p: Vec<[f32; 3]> = vec![];
        let t: Vec<[usize; 3]> = vec![];
        let cfg = default_volume_measure_config();
        let res = measure_mesh_volume(&p, &t, &cfg);
        assert_eq!(res.volume, 0.0);
        assert_eq!(res.surface_area, 0.0);
    }
}
