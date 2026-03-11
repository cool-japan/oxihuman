#![allow(dead_code)]
//! Deformation cage for mesh deformation.

/// A deformation cage wrapping a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DeformCage {
    pub vertices: Vec<[f32; 3]>,
    pub rest_vertices: Vec<[f32; 3]>,
    pub bound: bool,
}

/// Create a new deform cage from vertices.
#[allow(dead_code)]
pub fn new_deform_cage(vertices: Vec<[f32; 3]>) -> DeformCage {
    let rest = vertices.clone();
    DeformCage {
        vertices,
        rest_vertices: rest,
        bound: false,
    }
}

/// Return the number of cage vertices.
#[allow(dead_code)]
pub fn cage_vertex_count_dc(cage: &DeformCage) -> usize {
    cage.vertices.len()
}

/// Bind the cage to a mesh (marks as bound).
#[allow(dead_code)]
pub fn cage_bind(cage: &mut DeformCage) {
    cage.bound = true;
}

/// Deform mesh positions using the cage displacement.
#[allow(dead_code)]
pub fn cage_deform(cage: &DeformCage, positions: &mut [[f32; 3]]) {
    if !cage.bound {
        return;
    }
    for (pos, (cv, rv)) in positions.iter_mut().zip(cage.vertices.iter().zip(cage.rest_vertices.iter())) {
        pos[0] += cv[0] - rv[0];
        pos[1] += cv[1] - rv[1];
        pos[2] += cv[2] - rv[2];
    }
}

/// Reset the cage to rest state.
#[allow(dead_code)]
pub fn cage_reset(cage: &mut DeformCage) {
    cage.vertices = cage.rest_vertices.clone();
    cage.bound = false;
}

/// Serialize cage to JSON string.
#[allow(dead_code)]
pub fn cage_to_json(cage: &DeformCage) -> String {
    format!(
        "{{\"vertex_count\":{},\"bound\":{}}}",
        cage.vertices.len(),
        cage.bound
    )
}

/// Compute AABB bounds of the cage.
#[allow(dead_code)]
pub fn cage_bounds(cage: &DeformCage) -> ([f32; 3], [f32; 3]) {
    if cage.vertices.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = cage.vertices[0];
    let mut mx = cage.vertices[0];
    for v in &cage.vertices {
        for j in 0..3 {
            if v[j] < mn[j] { mn[j] = v[j]; }
            if v[j] > mx[j] { mx[j] = v[j]; }
        }
    }
    (mn, mx)
}

/// Check if the cage is currently bound.
#[allow(dead_code)]
pub fn cage_is_bound(cage: &DeformCage) -> bool {
    cage.bound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deform_cage() {
        let c = new_deform_cage(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert_eq!(cage_vertex_count_dc(&c), 2);
    }

    #[test]
    fn test_cage_bind() {
        let mut c = new_deform_cage(vec![[0.0; 3]]);
        assert!(!cage_is_bound(&c));
        cage_bind(&mut c);
        assert!(cage_is_bound(&c));
    }

    #[test]
    fn test_cage_deform_not_bound() {
        let c = new_deform_cage(vec![[1.0, 0.0, 0.0]]);
        let mut pos = [[0.0; 3]];
        cage_deform(&c, &mut pos);
        assert_eq!(pos[0], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_cage_deform_bound() {
        let mut c = new_deform_cage(vec![[0.0, 0.0, 0.0]]);
        cage_bind(&mut c);
        c.vertices[0] = [1.0, 2.0, 3.0];
        let mut pos = [[10.0, 20.0, 30.0]];
        cage_deform(&c, &mut pos);
        assert_eq!(pos[0], [11.0, 22.0, 33.0]);
    }

    #[test]
    fn test_cage_reset() {
        let mut c = new_deform_cage(vec![[0.0; 3]]);
        cage_bind(&mut c);
        c.vertices[0] = [5.0, 5.0, 5.0];
        cage_reset(&mut c);
        assert!(!cage_is_bound(&c));
        assert_eq!(c.vertices[0], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_cage_to_json() {
        let c = new_deform_cage(vec![[0.0; 3]]);
        let j = cage_to_json(&c);
        assert!(j.contains("\"vertex_count\":1"));
    }

    #[test]
    fn test_cage_bounds_empty() {
        let c = new_deform_cage(vec![]);
        let (mn, mx) = cage_bounds(&c);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_cage_bounds() {
        let c = new_deform_cage(vec![[-1.0, 0.0, 1.0], [2.0, 3.0, -1.0]]);
        let (mn, mx) = cage_bounds(&c);
        assert_eq!(mn, [-1.0, 0.0, -1.0]);
        assert_eq!(mx, [2.0, 3.0, 1.0]);
    }

    #[test]
    fn test_cage_vertex_count_empty() {
        let c = new_deform_cage(vec![]);
        assert_eq!(cage_vertex_count_dc(&c), 0);
    }

    #[test]
    fn test_cage_is_bound_initial() {
        let c = new_deform_cage(vec![[1.0; 3]]);
        assert!(!cage_is_bound(&c));
    }
}
