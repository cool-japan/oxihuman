#![allow(dead_code)]
//! Vertex projection utilities.

use std::f32::consts::PI;

/// Vertex projection result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexProject {
    pub projected: Vec<[f32; 3]>,
    pub distances: Vec<f32>,
}

/// Project vertices onto a plane defined by normal and distance.
#[allow(dead_code)]
pub fn project_to_plane(positions: &[[f32; 3]], normal: [f32; 3], dist: f32) -> VertexProject {
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-12 {
        return VertexProject {
            projected: positions.to_vec(),
            distances: vec![0.0; positions.len()],
        };
    }
    let n = [normal[0] / len, normal[1] / len, normal[2] / len];
    let mut projected = Vec::with_capacity(positions.len());
    let mut distances = Vec::with_capacity(positions.len());
    for p in positions {
        let d = p[0] * n[0] + p[1] * n[1] + p[2] * n[2] - dist;
        projected.push([p[0] - d * n[0], p[1] - d * n[1], p[2] - d * n[2]]);
        distances.push(d.abs());
    }
    VertexProject {
        projected,
        distances,
    }
}

/// Project vertices onto a sphere.
#[allow(dead_code)]
pub fn project_to_sphere(positions: &[[f32; 3]], center: [f32; 3], radius: f32) -> VertexProject {
    let mut projected = Vec::with_capacity(positions.len());
    let mut distances = Vec::with_capacity(positions.len());
    for p in positions {
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        if d < 1e-12 {
            projected.push([center[0] + radius, center[1], center[2]]);
            distances.push(radius);
        } else {
            let scale = radius / d;
            projected.push([
                center[0] + dx * scale,
                center[1] + dy * scale,
                center[2] + dz * scale,
            ]);
            distances.push((d - radius).abs());
        }
    }
    VertexProject {
        projected,
        distances,
    }
}

/// Project vertices onto a cylinder (along Y axis).
#[allow(dead_code)]
pub fn project_to_cylinder(positions: &[[f32; 3]], center: [f32; 2], radius: f32) -> VertexProject {
    let mut projected = Vec::with_capacity(positions.len());
    let mut distances = Vec::with_capacity(positions.len());
    for p in positions {
        let dx = p[0] - center[0];
        let dz = p[2] - center[1];
        let d = (dx * dx + dz * dz).sqrt();
        if d < 1e-12 {
            projected.push([center[0] + radius, p[1], center[1]]);
            distances.push(radius);
        } else {
            let scale = radius / d;
            projected.push([center[0] + dx * scale, p[1], center[1] + dz * scale]);
            distances.push((d - radius).abs());
        }
    }
    VertexProject {
        projected,
        distances,
    }
}

/// Project vertices onto a surface (simplified: nearest axis-aligned plane).
#[allow(dead_code)]
pub fn project_to_surface(positions: &[[f32; 3]], axis: u8) -> VertexProject {
    let idx = (axis % 3) as usize;
    let mut projected = Vec::with_capacity(positions.len());
    let mut distances = Vec::with_capacity(positions.len());
    for p in positions {
        let mut q = *p;
        distances.push(q[idx].abs());
        q[idx] = 0.0;
        projected.push(q);
    }
    VertexProject {
        projected,
        distances,
    }
}

/// Get the projected distance for a vertex.
#[allow(dead_code)]
pub fn projected_distance(vp: &VertexProject, index: usize) -> f32 {
    if index < vp.distances.len() {
        vp.distances[index]
    } else {
        0.0
    }
}

/// Project along vertex normals.
#[allow(dead_code)]
pub fn project_along_normal(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    distance: f32,
) -> Vec<[f32; 3]> {
    let _ = PI; // use the import
    positions
        .iter()
        .zip(normals.iter())
        .map(|(p, n)| {
            [
                p[0] + n[0] * distance,
                p[1] + n[1] * distance,
                p[2] + n[2] * distance,
            ]
        })
        .collect()
}

/// Count projected vertices.
#[allow(dead_code)]
pub fn project_count(vp: &VertexProject) -> usize {
    vp.projected.len()
}

/// Serialize projection to JSON.
#[allow(dead_code)]
pub fn project_to_json(vp: &VertexProject) -> String {
    let avg_dist = if vp.distances.is_empty() {
        0.0
    } else {
        vp.distances.iter().sum::<f32>() / vp.distances.len() as f32
    };
    format!(
        "{{\"count\":{},\"avg_distance\":{:.4}}}",
        vp.projected.len(),
        avg_dist
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_to_plane() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let vp = project_to_plane(&pts, [0.0, 1.0, 0.0], 0.0);
        assert!((vp.projected[0][1]).abs() < 1e-6);
    }

    #[test]
    fn test_project_to_sphere() {
        let pts = vec![[2.0, 0.0, 0.0]];
        let vp = project_to_sphere(&pts, [0.0, 0.0, 0.0], 1.0);
        assert!((vp.projected[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_project_to_cylinder() {
        let pts = vec![[3.0, 5.0, 0.0]];
        let vp = project_to_cylinder(&pts, [0.0, 0.0], 1.0);
        assert!((vp.projected[0][0] - 1.0).abs() < 1e-6);
        assert!((vp.projected[0][1] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_project_to_surface() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let vp = project_to_surface(&pts, 1);
        assert!((vp.projected[0][1]).abs() < 1e-6);
    }

    #[test]
    fn test_projected_distance() {
        let vp = VertexProject {
            projected: vec![],
            distances: vec![1.5],
        };
        assert!((projected_distance(&vp, 0) - 1.5).abs() < 1e-6);
        assert!((projected_distance(&vp, 5)).abs() < 1e-6);
    }

    #[test]
    fn test_project_along_normal() {
        let pts = vec![[0.0, 0.0, 0.0]];
        let nrm = vec![[0.0, 1.0, 0.0]];
        let r = project_along_normal(&pts, &nrm, 2.0);
        assert!((r[0][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_project_count() {
        let vp = VertexProject {
            projected: vec![[0.0; 3]; 5],
            distances: vec![0.0; 5],
        };
        assert_eq!(project_count(&vp), 5);
    }

    #[test]
    fn test_project_to_json() {
        let vp = VertexProject {
            projected: vec![[0.0; 3]],
            distances: vec![1.0],
        };
        let j = project_to_json(&vp);
        assert!(j.contains("count"));
    }

    #[test]
    fn test_project_to_plane_zero_normal() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let vp = project_to_plane(&pts, [0.0, 0.0, 0.0], 0.0);
        assert_eq!(vp.projected[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_project_to_sphere_at_center() {
        let pts = vec![[0.0, 0.0, 0.0]];
        let vp = project_to_sphere(&pts, [0.0, 0.0, 0.0], 1.0);
        assert!((vp.projected[0][0] - 1.0).abs() < 1e-6);
    }
}
