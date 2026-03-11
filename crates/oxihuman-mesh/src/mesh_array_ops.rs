#![allow(dead_code)]
//! Array-level vertex ops (vec3 operations).

/// Add two vec3 values.
#[allow(dead_code)]
pub fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Subtract two vec3 values.
#[allow(dead_code)]
pub fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Scale a vec3 by a scalar.
#[allow(dead_code)]
pub fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Dot product of two vec3 values.
#[allow(dead_code)]
pub fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product of two vec3 values.
#[allow(dead_code)]
pub fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Length of a vec3.
#[allow(dead_code)]
pub fn vec3_length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Normalize a vec3.
#[allow(dead_code)]
pub fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_length(v);
    if len < 1e-12 {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Linear interpolation between two vec3 values.
#[allow(dead_code)]
pub fn vec3_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Distance between two vec3 values.
#[allow(dead_code)]
pub fn vec3_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    vec3_length(vec3_sub(a, b))
}

/// Return a zero vec3.
#[allow(dead_code)]
pub fn vec3_zero() -> [f32; 3] {
    [0.0; 3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_add() {
        assert_eq!(vec3_add([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), [5.0, 7.0, 9.0]);
    }

    #[test]
    fn test_vec3_sub() {
        assert_eq!(vec3_sub([5.0, 7.0, 9.0], [4.0, 5.0, 6.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_vec3_scale() {
        assert_eq!(vec3_scale([1.0, 2.0, 3.0], 2.0), [2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_vec3_dot() {
        let d = vec3_dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_vec3_cross() {
        let c = vec3_cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_length() {
        let l = vec3_length([3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_normalize() {
        let n = vec3_normalize([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_normalize_zero() {
        assert_eq!(vec3_normalize([0.0; 3]), [0.0; 3]);
    }

    #[test]
    fn test_vec3_lerp() {
        let l = vec3_lerp([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], 0.5);
        assert!((l[0] - 1.0).abs() < 1e-6);
        assert!((l[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_distance() {
        let d = vec3_distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_zero() {
        assert_eq!(vec3_zero(), [0.0, 0.0, 0.0]);
    }
}
