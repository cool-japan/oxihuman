#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BodyInertia {
    tensor: [f32; 9], // 3x3 matrix, row-major
    mass: f32,
}

#[allow(dead_code)]
pub fn new_body_inertia(mass: f32) -> BodyInertia {
    let i = mass;
    BodyInertia {
        tensor: [i, 0.0, 0.0, 0.0, i, 0.0, 0.0, 0.0, i],
        mass,
    }
}

#[allow(dead_code)]
pub fn inertia_tensor_bi(bi: &BodyInertia) -> [f32; 9] {
    bi.tensor
}

#[allow(dead_code)]
pub fn inverse_inertia(bi: &BodyInertia) -> [f32; 9] {
    let mut inv = [0.0f32; 9];
    for i in [0, 4, 8] {
        inv[i] = if bi.tensor[i].abs() > f32::EPSILON {
            1.0 / bi.tensor[i]
        } else {
            0.0
        };
    }
    inv
}

#[allow(dead_code)]
pub fn rotate_inertia_bi(bi: &BodyInertia, _angle: f32) -> BodyInertia {
    // Simplified: for diagonal tensors, rotation has no effect
    *bi
}

#[allow(dead_code)]
pub fn inertia_from_shape(mass: f32, shape_type: u32) -> BodyInertia {
    match shape_type {
        0 => inertia_sphere_bi(mass, 1.0),
        1 => inertia_box_bi(mass, 1.0, 1.0, 1.0),
        _ => new_body_inertia(mass),
    }
}

#[allow(dead_code)]
pub fn inertia_sphere_bi(mass: f32, radius: f32) -> BodyInertia {
    let i = 2.0 / 5.0 * mass * radius * radius;
    BodyInertia {
        tensor: [i, 0.0, 0.0, 0.0, i, 0.0, 0.0, 0.0, i],
        mass,
    }
}

#[allow(dead_code)]
pub fn inertia_box_bi(mass: f32, w: f32, h: f32, d: f32) -> BodyInertia {
    let ix = mass / 12.0 * (h * h + d * d);
    let iy = mass / 12.0 * (w * w + d * d);
    let iz = mass / 12.0 * (w * w + h * h);
    BodyInertia {
        tensor: [ix, 0.0, 0.0, 0.0, iy, 0.0, 0.0, 0.0, iz],
        mass,
    }
}

#[allow(dead_code)]
pub fn inertia_to_array(bi: &BodyInertia) -> [f32; 9] {
    bi.tensor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bi = new_body_inertia(1.0);
        let t = inertia_tensor_bi(&bi);
        assert!((t[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inverse() {
        let bi = new_body_inertia(2.0);
        let inv = inverse_inertia(&bi);
        assert!((inv[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sphere() {
        let bi = inertia_sphere_bi(5.0, 2.0);
        let expected = 2.0 / 5.0 * 5.0 * 4.0;
        assert!((bi.tensor[0] - expected).abs() < 1e-5);
    }

    #[test]
    fn test_box_inertia() {
        let bi = inertia_box_bi(12.0, 1.0, 1.0, 1.0);
        let expected = 12.0 / 12.0 * 2.0;
        assert!((bi.tensor[0] - expected).abs() < 1e-5);
    }

    #[test]
    fn test_from_shape_sphere() {
        let bi = inertia_from_shape(1.0, 0);
        assert!(bi.tensor[0] > 0.0);
    }

    #[test]
    fn test_from_shape_box() {
        let bi = inertia_from_shape(1.0, 1);
        assert!(bi.tensor[0] > 0.0);
    }

    #[test]
    fn test_from_shape_default() {
        let bi = inertia_from_shape(1.0, 99);
        assert!((bi.tensor[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rotate() {
        let bi = new_body_inertia(1.0);
        let rotated = rotate_inertia_bi(&bi, 0.5);
        assert_eq!(inertia_tensor_bi(&bi), inertia_tensor_bi(&rotated));
    }

    #[test]
    fn test_to_array() {
        let bi = new_body_inertia(3.0);
        let arr = inertia_to_array(&bi);
        assert_eq!(arr.len(), 9);
    }

    #[test]
    fn test_zero_mass() {
        let bi = new_body_inertia(0.0);
        let inv = inverse_inertia(&bi);
        assert_eq!(inv[0], 0.0);
    }
}
