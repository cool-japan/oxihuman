// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CastParams {
    pub shape: u8,
    pub factor: f32,
    pub radius: f32,
    pub use_x: bool,
    pub use_y: bool,
    pub use_z: bool,
}

pub fn new_cast_sphere(radius: f32, factor: f32) -> CastParams {
    CastParams {
        shape: 0,
        factor,
        radius,
        use_x: true,
        use_y: true,
        use_z: true,
    }
}

pub fn cast_target_sphere(p: [f32; 3], radius: f32) -> [f32; 3] {
    let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if len < 1e-10 {
        return [radius, 0.0, 0.0];
    }
    [
        p[0] / len * radius,
        p[1] / len * radius,
        p[2] / len * radius,
    ]
}

pub fn cast_target_cylinder(p: [f32; 3], radius: f32) -> [f32; 3] {
    let len = (p[0] * p[0] + p[1] * p[1]).sqrt();
    if len < 1e-10 {
        return [radius, 0.0, p[2]];
    }
    [p[0] / len * radius, p[1] / len * radius, p[2]]
}

pub fn cast_blend(original: [f32; 3], target: [f32; 3], factor: f32) -> [f32; 3] {
    let f = factor.clamp(0.0, 1.0);
    [
        original[0] * (1.0 - f) + target[0] * f,
        original[1] * (1.0 - f) + target[1] * f,
        original[2] * (1.0 - f) + target[2] * f,
    ]
}

pub fn cast_vertex(p: [f32; 3], params: &CastParams) -> [f32; 3] {
    let target = match params.shape {
        0 => cast_target_sphere(p, params.radius),
        1 => cast_target_cylinder(p, params.radius),
        _ => {
            let x = if params.use_x {
                p[0].clamp(-params.radius, params.radius)
            } else {
                p[0]
            };
            let y = if params.use_y {
                p[1].clamp(-params.radius, params.radius)
            } else {
                p[1]
            };
            let z = if params.use_z {
                p[2].clamp(-params.radius, params.radius)
            } else {
                p[2]
            };
            [x, y, z]
        }
    };
    cast_blend(p, target, params.factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cast_sphere() {
        /* shape=0 for sphere */
        let p = new_cast_sphere(1.0, 0.5);
        assert_eq!(p.shape, 0);
        assert!((p.radius - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cast_target_sphere_on_sphere() {
        /* point already on sphere stays there */
        let out = cast_target_sphere([1.0, 0.0, 0.0], 1.0);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cast_target_cylinder_preserves_z() {
        /* cylinder cast preserves z coordinate */
        let out = cast_target_cylinder([2.0, 0.0, 5.0], 1.0);
        assert!((out[2] - 5.0).abs() < 1e-5);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cast_blend_factor_zero() {
        /* factor=0 returns original */
        let out = cast_blend([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], 0.0);
        assert!((out[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cast_blend_factor_one() {
        /* factor=1 returns target */
        let out = cast_blend([0.0, 0.0, 0.0], [5.0, 6.0, 7.0], 1.0);
        assert!((out[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_cast_vertex_sphere() {
        /* cast to sphere pushes point toward surface */
        let params = new_cast_sphere(1.0, 1.0);
        let v = [2.0f32, 0.0, 0.0];
        let out = cast_vertex(v, &params);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cast_vertex_no_effect() {
        /* factor=0 leaves vertex unchanged */
        let params = new_cast_sphere(1.0, 0.0);
        let v = [3.0f32, 4.0, 5.0];
        let out = cast_vertex(v, &params);
        assert!((out[0] - 3.0).abs() < 1e-5);
    }
}
