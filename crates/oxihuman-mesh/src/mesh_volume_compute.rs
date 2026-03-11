#![allow(dead_code)]
//! Compute mesh volume via divergence theorem.

/// Result of a volume computation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeResult {
    pub volume: f32,
    pub is_watertight: bool,
}

/// Compute signed volume of a mesh using the divergence theorem.
#[allow(dead_code)]
pub fn mesh_volume_signed(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let mut vol = 0.0f32;
    for tri in indices {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        vol += a[0] * (b[1] * c[2] - b[2] * c[1])
             + b[0] * (c[1] * a[2] - c[2] * a[1])
             + c[0] * (a[1] * b[2] - a[2] * b[1]);
    }
    vol / 6.0
}

/// Compute unsigned (absolute) volume.
#[allow(dead_code)]
pub fn mesh_volume_unsigned(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    mesh_volume_signed(positions, indices).abs()
}

/// Simple watertight check: each edge should appear exactly twice.
#[allow(dead_code)]
pub fn is_watertight(indices: &[[u32; 3]]) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count.values().all(|&c| c == 2)
}

/// Approximate center of mass assuming uniform density.
#[allow(dead_code)]
pub fn volume_center_of_mass(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> [f32; 3] {
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let mut total_vol = 0.0f32;
    for tri in indices {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let v = (a[0] * (b[1] * c[2] - b[2] * c[1])
               + b[0] * (c[1] * a[2] - c[2] * a[1])
               + c[0] * (a[1] * b[2] - a[2] * b[1])) / 6.0;
        let centroid = [
            (a[0] + b[0] + c[0]) / 4.0,
            (a[1] + b[1] + c[1]) / 4.0,
            (a[2] + b[2] + c[2]) / 4.0,
        ];
        cx += v * centroid[0];
        cy += v * centroid[1];
        cz += v * centroid[2];
        total_vol += v;
    }
    if total_vol.abs() < 1e-12 {
        return [0.0; 3];
    }
    [cx / total_vol, cy / total_vol, cz / total_vol]
}

/// Approximate inertia (scalar) = volume * (extent^2).
#[allow(dead_code)]
pub fn volume_inertia_approx(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let vol = mesh_volume_unsigned(positions, indices);
    let com = volume_center_of_mass(positions, indices);
    let mut max_r2 = 0.0f32;
    for p in positions {
        let dx = p[0] - com[0];
        let dy = p[1] - com[1];
        let dz = p[2] - com[2];
        let r2 = dx * dx + dy * dy + dz * dz;
        if r2 > max_r2 {
            max_r2 = r2;
        }
    }
    vol * max_r2
}

/// Ratio of mesh volume to bounding box volume.
#[allow(dead_code)]
pub fn volume_bbox_ratio(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            if p[i] < min[i] { min[i] = p[i]; }
            if p[i] > max[i] { max[i] = p[i]; }
        }
    }
    let bbox_vol = (max[0] - min[0]) * (max[1] - min[1]) * (max[2] - min[2]);
    if bbox_vol.abs() < 1e-12 {
        return 0.0;
    }
    mesh_volume_unsigned(positions, indices) / bbox_vol
}

/// Serialize volume result to a JSON-like string.
#[allow(dead_code)]
pub fn volume_to_json(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> String {
    let vol = mesh_volume_signed(positions, indices);
    let wt = is_watertight(indices);
    format!("{{\"volume\":{:.6},\"watertight\":{}}}", vol, wt)
}

/// Check if a volume value is valid (finite and non-NaN).
#[allow(dead_code)]
pub fn volume_is_valid(vol: f32) -> bool {
    vol.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ];
        let i = vec![
            [0, 2, 1], [0, 3, 2],
            [4, 5, 6], [4, 6, 7],
            [0, 1, 5], [0, 5, 4],
            [2, 3, 7], [2, 7, 6],
            [1, 2, 6], [1, 6, 5],
            [0, 4, 7], [0, 7, 3],
        ];
        (p, i)
    }

    #[test]
    fn test_signed_volume_cube() {
        let (p, i) = unit_cube();
        let vol = mesh_volume_signed(&p, &i);
        assert!((vol - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_unsigned_volume() {
        let (p, i) = unit_cube();
        assert!((mesh_volume_unsigned(&p, &i) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_is_watertight() {
        let (_, i) = unit_cube();
        assert!(is_watertight(&i));
    }

    #[test]
    fn test_not_watertight() {
        let i = vec![[0u32, 1, 2]];
        assert!(!is_watertight(&i));
    }

    #[test]
    fn test_center_of_mass() {
        let (p, i) = unit_cube();
        let com = volume_center_of_mass(&p, &i);
        assert!((com[0] - 0.5).abs() < 0.1);
        assert!((com[1] - 0.5).abs() < 0.1);
        assert!((com[2] - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_volume_inertia_approx() {
        let (p, i) = unit_cube();
        let inertia = volume_inertia_approx(&p, &i);
        assert!(inertia > 0.0);
    }

    #[test]
    fn test_volume_bbox_ratio() {
        let (p, i) = unit_cube();
        let ratio = volume_bbox_ratio(&p, &i);
        assert!((ratio - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_volume_to_json() {
        let (p, i) = unit_cube();
        let json = volume_to_json(&p, &i);
        assert!(json.contains("volume"));
        assert!(json.contains("watertight"));
    }

    #[test]
    fn test_volume_is_valid() {
        assert!(volume_is_valid(1.0));
        assert!(!volume_is_valid(f32::NAN));
        assert!(!volume_is_valid(f32::INFINITY));
    }

    #[test]
    fn test_empty_mesh_volume() {
        assert!((mesh_volume_signed(&[], &[])).abs() < 1e-6);
    }
}
