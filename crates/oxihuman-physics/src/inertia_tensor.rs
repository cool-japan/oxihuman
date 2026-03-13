// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Inertia tensor computation for primitive shapes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InertiaTensorConfig {
    pub density: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InertiaTensor {
    pub tensor: [[f32; 3]; 3],
}

#[allow(dead_code)]
pub fn default_inertia_tensor_config() -> InertiaTensorConfig {
    InertiaTensorConfig { density: 1.0 }
}

#[allow(dead_code)]
pub fn inertia_sphere(mass: f32, radius: f32) -> InertiaTensor {
    let i = 2.0 / 5.0 * mass * radius * radius;
    InertiaTensor {
        tensor: [
            [i, 0.0, 0.0],
            [0.0, i, 0.0],
            [0.0, 0.0, i],
        ],
    }
}

#[allow(dead_code)]
pub fn inertia_box(mass: f32, half_extents: [f32; 3]) -> InertiaTensor {
    let [hx, hy, hz] = half_extents;
    let ix = mass / 3.0 * (hy * hy + hz * hz);
    let iy = mass / 3.0 * (hx * hx + hz * hz);
    let iz = mass / 3.0 * (hx * hx + hy * hy);
    InertiaTensor {
        tensor: [
            [ix, 0.0, 0.0],
            [0.0, iy, 0.0],
            [0.0, 0.0, iz],
        ],
    }
}

#[allow(dead_code)]
pub fn inertia_cylinder(mass: f32, radius: f32, height: f32) -> InertiaTensor {
    let iy = 0.5 * mass * radius * radius;
    let ixz = mass / 12.0 * (3.0 * radius * radius + height * height);
    InertiaTensor {
        tensor: [
            [ixz, 0.0, 0.0],
            [0.0, iy, 0.0],
            [0.0, 0.0, ixz],
        ],
    }
}

#[allow(dead_code)]
pub fn inertia_trace(t: &InertiaTensor) -> f32 {
    t.tensor[0][0] + t.tensor[1][1] + t.tensor[2][2]
}

#[allow(dead_code)]
pub fn inertia_is_diagonal(t: &InertiaTensor) -> bool {
    #[allow(clippy::needless_range_loop)]
    for i in 0..3 {
        for j in 0..3 {
            if i != j && t.tensor[i][j].abs() > 1e-6 {
                return false;
            }
        }
    }
    true
}

#[allow(dead_code)]
pub fn inertia_scale(t: &InertiaTensor, factor: f32) -> InertiaTensor {
    let mut out = t.clone();
    #[allow(clippy::needless_range_loop)]
    for i in 0..3 {
        for j in 0..3 {
            out.tensor[i][j] *= factor;
        }
    }
    out
}

#[allow(dead_code)]
pub fn inertia_to_json(t: &InertiaTensor) -> String {
    format!(
        "{{\"trace\":{},\"diagonal\":[{},{},{}]}}",
        inertia_trace(t),
        t.tensor[0][0],
        t.tensor[1][1],
        t.tensor[2][2]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_inertia_tensor_config();
        assert!((cfg.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sphere_diagonal() {
        let t = inertia_sphere(1.0, 1.0);
        assert!(inertia_is_diagonal(&t));
    }

    #[test]
    fn test_sphere_trace() {
        let t = inertia_sphere(1.0, 1.0);
        let i = 2.0 / 5.0;
        assert!((inertia_trace(&t) - 3.0 * i).abs() < 1e-5);
    }

    #[test]
    fn test_box_diagonal() {
        let t = inertia_box(1.0, [0.5, 0.5, 0.5]);
        assert!(inertia_is_diagonal(&t));
    }

    #[test]
    fn test_cylinder_diagonal() {
        let t = inertia_cylinder(1.0, 0.5, 1.0);
        assert!(inertia_is_diagonal(&t));
    }

    #[test]
    fn test_scale() {
        let t = inertia_sphere(1.0, 1.0);
        let scaled = inertia_scale(&t, 2.0);
        assert!((scaled.tensor[0][0] - t.tensor[0][0] * 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let t = inertia_sphere(1.0, 1.0);
        let json = inertia_to_json(&t);
        assert!(json.contains("\"trace\""));
    }

    #[test]
    fn test_box_symmetric() {
        let t = inertia_box(6.0, [1.0, 1.0, 1.0]);
        assert!((t.tensor[0][0] - t.tensor[1][1]).abs() < 1e-5);
        assert!((t.tensor[1][1] - t.tensor[2][2]).abs() < 1e-5);
    }

    #[test]
    fn test_sphere_isotropic() {
        let t = inertia_sphere(5.0, 2.0);
        assert!((t.tensor[0][0] - t.tensor[1][1]).abs() < 1e-5);
        assert!((t.tensor[1][1] - t.tensor[2][2]).abs() < 1e-5);
    }
}
