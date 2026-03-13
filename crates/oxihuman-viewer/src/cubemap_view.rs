// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cubemap view — sampling, face indexing, direction-to-UV mapping,
//! and spherical-to-cubemap conversion utilities.

use std::f32::consts::{FRAC_1_SQRT_2, PI};

/// Cubemap face identifier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

/// Result of direction-to-cubemap lookup.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CubemapLookup {
    pub face: CubeFace,
    pub u: f32,
    pub v: f32,
}

/// Convert a 3D direction to cubemap face + UV.
#[allow(dead_code)]
pub fn direction_to_cubemap(dir: [f32; 3]) -> CubemapLookup {
    let ax = dir[0].abs();
    let ay = dir[1].abs();
    let az = dir[2].abs();

    let (face, sc, tc, ma) = if ax >= ay && ax >= az {
        if dir[0] > 0.0 {
            (CubeFace::PositiveX, -dir[2], -dir[1], ax)
        } else {
            (CubeFace::NegativeX, dir[2], -dir[1], ax)
        }
    } else if ay >= ax && ay >= az {
        if dir[1] > 0.0 {
            (CubeFace::PositiveY, dir[0], dir[2], ay)
        } else {
            (CubeFace::NegativeY, dir[0], -dir[2], ay)
        }
    } else if dir[2] > 0.0 {
        (CubeFace::PositiveZ, dir[0], -dir[1], az)
    } else {
        (CubeFace::NegativeZ, -dir[0], -dir[1], az)
    };

    let u = if ma.abs() < 1e-9 { 0.5 } else { 0.5 * (sc / ma + 1.0) };
    let v = if ma.abs() < 1e-9 { 0.5 } else { 0.5 * (tc / ma + 1.0) };

    CubemapLookup {
        face,
        u: u.clamp(0.0, 1.0),
        v: v.clamp(0.0, 1.0),
    }
}

/// Convert cubemap face + UV back to a 3D direction (normalised).
#[allow(dead_code)]
pub fn cubemap_to_direction(face: CubeFace, u: f32, v: f32) -> [f32; 3] {
    let sc = 2.0 * u - 1.0;
    let tc = 2.0 * v - 1.0;

    let dir = match face {
        CubeFace::PositiveX => [1.0, -tc, -sc],
        CubeFace::NegativeX => [-1.0, -tc, sc],
        CubeFace::PositiveY => [sc, 1.0, tc],
        CubeFace::NegativeY => [sc, -1.0, -tc],
        CubeFace::PositiveZ => [sc, -tc, 1.0],
        CubeFace::NegativeZ => [-sc, -tc, -1.0],
    };
    normalize(dir)
}

/// Convert spherical coordinates (theta, phi) to direction.
///
/// `theta` is azimuth (0..2PI), `phi` is elevation (0..PI, 0=north pole).
#[allow(dead_code)]
pub fn spherical_to_direction(theta: f32, phi: f32) -> [f32; 3] {
    let sin_phi = phi.sin();
    [sin_phi * theta.cos(), phi.cos(), sin_phi * theta.sin()]
}

/// Convert direction to spherical (theta, phi).
#[allow(dead_code)]
pub fn direction_to_spherical(dir: [f32; 3]) -> (f32, f32) {
    let d = normalize(dir);
    let phi = d[1].clamp(-1.0, 1.0).acos();
    let theta = d[2].atan2(d[0]);
    let theta = if theta < 0.0 { theta + 2.0 * PI } else { theta };
    (theta, phi)
}

/// Compute the solid angle of a cubemap texel.
///
/// `face_size` is the resolution of each face.
#[allow(dead_code)]
pub fn texel_solid_angle(u: f32, v: f32, face_size: u32) -> f32 {
    if face_size == 0 {
        return 0.0;
    }
    let inv_res = 1.0 / face_size as f32;
    let s = 2.0 * u - 1.0;
    let t = 2.0 * v - 1.0;
    let x0 = s - inv_res;
    let x1 = s + inv_res;
    let y0 = t - inv_res;
    let y1 = t + inv_res;

    fn area_element(x: f32, y: f32) -> f32 {
        (x * y).atan2((x * x + y * y + 1.0).sqrt())
    }
    (area_element(x0, y0) - area_element(x0, y1) - area_element(x1, y0) + area_element(x1, y1)).abs()
}

/// Bilinear interpolation of 4 corner values.
#[allow(dead_code)]
pub fn bilinear(c00: f32, c10: f32, c01: f32, c11: f32, u: f32, v: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let a = c00 * (1.0 - u) + c10 * u;
    let b = c01 * (1.0 - u) + c11 * u;
    a * (1.0 - v) + b * v
}

/// All six faces in standard order.
#[allow(dead_code)]
pub fn all_faces() -> [CubeFace; 6] {
    [
        CubeFace::PositiveX,
        CubeFace::NegativeX,
        CubeFace::PositiveY,
        CubeFace::NegativeY,
        CubeFace::PositiveZ,
        CubeFace::NegativeZ,
    ]
}

/// Face name string.
#[allow(dead_code)]
pub fn face_name(face: CubeFace) -> &'static str {
    match face {
        CubeFace::PositiveX => "+X",
        CubeFace::NegativeX => "-X",
        CubeFace::PositiveY => "+Y",
        CubeFace::NegativeY => "-Y",
        CubeFace::PositiveZ => "+Z",
        CubeFace::NegativeZ => "-Z",
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 { return [0.0, 1.0, 0.0]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_positive_x_lookup() {
        let r = direction_to_cubemap([1.0, 0.0, 0.0]);
        assert_eq!(r.face, CubeFace::PositiveX);
    }

    #[test]
    fn test_negative_z_lookup() {
        let r = direction_to_cubemap([0.0, 0.0, -1.0]);
        assert_eq!(r.face, CubeFace::NegativeZ);
    }

    #[test]
    fn test_roundtrip_direction() {
        let dir = normalize([0.5, 0.3, 0.8]);
        let lookup = direction_to_cubemap(dir);
        let back = cubemap_to_direction(lookup.face, lookup.u, lookup.v);
        for i in 0..3 {
            assert!((dir[i] - back[i]).abs() < 0.05, "Component {i}: {} vs {}", dir[i], back[i]);
        }
    }

    #[test]
    fn test_spherical_roundtrip() {
        let dir = normalize([1.0, 0.5, -0.3]);
        let (theta, phi) = direction_to_spherical(dir);
        let back = spherical_to_direction(theta, phi);
        for i in 0..3 {
            assert!((dir[i] - back[i]).abs() < 1e-4);
        }
    }

    #[test]
    fn test_texel_solid_angle_positive() {
        let sa = texel_solid_angle(0.5, 0.5, 256);
        assert!(sa > 0.0);
    }

    #[test]
    fn test_texel_solid_angle_zero_res() {
        assert_eq!(texel_solid_angle(0.5, 0.5, 0), 0.0);
    }

    #[test]
    fn test_bilinear_corners() {
        assert!((bilinear(1.0, 0.0, 0.0, 0.0, 0.0, 0.0) - 1.0).abs() < 1e-6);
        assert!((bilinear(0.0, 1.0, 0.0, 0.0, 1.0, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bilinear_centre() {
        let v = bilinear(0.0, 1.0, 0.0, 1.0, 0.5, 0.5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_all_faces_count() {
        assert_eq!(all_faces().len(), 6);
    }

    #[test]
    fn test_face_names() {
        assert_eq!(face_name(CubeFace::PositiveX), "+X");
        assert_eq!(face_name(CubeFace::NegativeZ), "-Z");
    }
}
