#![allow(dead_code)]
//! Per-vertex displacement vectors.

/// Per-vertex displacement data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexDisplacement {
    pub displacements: Vec<[f32; 3]>,
}

/// Create a new displacement set for N vertices.
#[allow(dead_code)]
pub fn new_displacement(count: usize) -> VertexDisplacement {
    VertexDisplacement {
        displacements: vec![[0.0; 3]; count],
    }
}

/// Set displacement at a vertex.
#[allow(dead_code)]
pub fn set_displacement(disp: &mut VertexDisplacement, index: usize, value: [f32; 3]) {
    if index < disp.displacements.len() {
        disp.displacements[index] = value;
    }
}

/// Get displacement at a vertex.
#[allow(dead_code)]
pub fn get_displacement(disp: &VertexDisplacement, index: usize) -> [f32; 3] {
    if index < disp.displacements.len() {
        disp.displacements[index]
    } else {
        [0.0; 3]
    }
}

/// Apply displacements to positions in-place.
#[allow(dead_code)]
pub fn apply_displacement(disp: &VertexDisplacement, positions: &mut [[f32; 3]]) {
    for (pos, d) in positions.iter_mut().zip(disp.displacements.iter()) {
        pos[0] += d[0];
        pos[1] += d[1];
        pos[2] += d[2];
    }
}

/// Compute the magnitude of displacement at a vertex.
#[allow(dead_code)]
pub fn displacement_magnitude(disp: &VertexDisplacement, index: usize) -> f32 {
    if index >= disp.displacements.len() {
        return 0.0;
    }
    let d = disp.displacements[index];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Return number of displacement entries.
#[allow(dead_code)]
pub fn displacement_count(disp: &VertexDisplacement) -> usize {
    disp.displacements.len()
}

/// Clear all displacements to zero.
#[allow(dead_code)]
pub fn clear_displacement(disp: &mut VertexDisplacement) {
    for d in &mut disp.displacements {
        *d = [0.0; 3];
    }
}

/// Serialize displacements to bytes.
#[allow(dead_code)]
pub fn displacement_to_bytes(disp: &VertexDisplacement) -> Vec<u8> {
    let mut buf = Vec::new();
    for d in &disp.displacements {
        for &f in d {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_displacement() {
        let d = new_displacement(5);
        assert_eq!(displacement_count(&d), 5);
    }

    #[test]
    fn test_set_get_displacement() {
        let mut d = new_displacement(2);
        set_displacement(&mut d, 0, [1.0, 2.0, 3.0]);
        assert_eq!(get_displacement(&d, 0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_get_oob() {
        let d = new_displacement(1);
        assert_eq!(get_displacement(&d, 10), [0.0; 3]);
    }

    #[test]
    fn test_apply_displacement() {
        let mut d = new_displacement(1);
        set_displacement(&mut d, 0, [1.0, 0.0, 0.0]);
        let mut pos = [[5.0, 5.0, 5.0]];
        apply_displacement(&d, &mut pos);
        assert_eq!(pos[0], [6.0, 5.0, 5.0]);
    }

    #[test]
    fn test_displacement_magnitude() {
        let mut d = new_displacement(1);
        set_displacement(&mut d, 0, [3.0, 4.0, 0.0]);
        assert!((displacement_magnitude(&d, 0) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_displacement_magnitude_oob() {
        let d = new_displacement(1);
        assert_eq!(displacement_magnitude(&d, 5), 0.0);
    }

    #[test]
    fn test_clear_displacement() {
        let mut d = new_displacement(2);
        set_displacement(&mut d, 0, [1.0, 1.0, 1.0]);
        clear_displacement(&mut d);
        assert_eq!(get_displacement(&d, 0), [0.0; 3]);
    }

    #[test]
    fn test_displacement_to_bytes() {
        let d = new_displacement(1);
        let bytes = displacement_to_bytes(&d);
        assert_eq!(bytes.len(), 12); // 3 floats * 4 bytes
    }

    #[test]
    fn test_empty_displacement() {
        let d = new_displacement(0);
        assert_eq!(displacement_count(&d), 0);
    }

    #[test]
    fn test_set_oob() {
        let mut d = new_displacement(1);
        set_displacement(&mut d, 100, [1.0; 3]); // should not panic
        assert_eq!(displacement_count(&d), 1);
    }
}
