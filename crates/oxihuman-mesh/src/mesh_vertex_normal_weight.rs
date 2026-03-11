#![allow(dead_code)]

/// Mode for computing weighted vertex normals.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalWeightMode {
    Area,
    Angle,
    Uniform,
}

/// Compute area-weighted vertex normal from surrounding face normals and areas.
#[allow(dead_code)]
pub fn area_weighted_normal(face_normals: &[[f32; 3]], face_areas: &[f32]) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for (normal, &area) in face_normals.iter().zip(face_areas.iter()) {
        sum[0] += normal[0] * area;
        sum[1] += normal[1] * area;
        sum[2] += normal[2] * area;
    }
    normalize(sum)
}

/// Compute angle-weighted vertex normal from surrounding face normals and angles.
#[allow(dead_code)]
pub fn angle_weighted_normal(face_normals: &[[f32; 3]], angles: &[f32]) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for (normal, &angle) in face_normals.iter().zip(angles.iter()) {
        sum[0] += normal[0] * angle;
        sum[1] += normal[1] * angle;
        sum[2] += normal[2] * angle;
    }
    normalize(sum)
}

/// Compute uniform-weighted vertex normal (simple average).
#[allow(dead_code)]
pub fn uniform_weighted_normal(face_normals: &[[f32; 3]]) -> [f32; 3] {
    if face_normals.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let mut sum = [0.0f32; 3];
    for normal in face_normals {
        sum[0] += normal[0];
        sum[1] += normal[1];
        sum[2] += normal[2];
    }
    normalize(sum)
}

/// Compute weighted normals for all vertices given faces and positions.
#[allow(dead_code)]
pub fn compute_weighted_normals(
    vertices: &[[f32; 3]],
    faces: &[[usize; 3]],
    mode: NormalWeightMode,
) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; vertices.len()];

    for face in faces {
        let a = vertices[face[0]];
        let b = vertices[face[1]];
        let c = vertices[face[2]];
        let fn_normal = face_normal(a, b, c);
        let area = face_area(a, b, c);

        for (i, &vi) in face.iter().enumerate() {
            let weight = match mode {
                NormalWeightMode::Area => area,
                NormalWeightMode::Angle => {
                    let prev = face[(i + 2) % 3];
                    let next = face[(i + 1) % 3];
                    angle_at_vertex(vertices[vi], vertices[prev], vertices[next])
                }
                NormalWeightMode::Uniform => 1.0,
            };
            normals[vi][0] += fn_normal[0] * weight;
            normals[vi][1] += fn_normal[1] * weight;
            normals[vi][2] += fn_normal[2] * weight;
        }
    }

    for n in normals.iter_mut() {
        *n = normalize(*n);
    }
    normals
}

/// Compare two normal weight modes by computing the angle between their results.
#[allow(dead_code)]
pub fn normal_weight_compare(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    d.clamp(-1.0, 1.0).acos()
}

/// Assign faces to smoothing groups based on angle threshold.
#[allow(dead_code)]
pub fn normal_smoothing_group(
    face_normals: &[[f32; 3]],
    faces: &[[usize; 3]],
    angle_threshold: f32,
) -> Vec<usize> {
    let mut groups = vec![0usize; faces.len()];
    let mut current_group = 0usize;

    for i in 0..faces.len() {
        if groups[i] != 0 {
            continue;
        }
        current_group += 1;
        groups[i] = current_group;
        for j in (i + 1)..faces.len() {
            if groups[j] != 0 {
                continue;
            }
            let dot = face_normals[i][0] * face_normals[j][0]
                + face_normals[i][1] * face_normals[j][1]
                + face_normals[i][2] * face_normals[j][2];
            let angle = dot.clamp(-1.0, 1.0).acos();
            if angle < angle_threshold {
                groups[j] = current_group;
            }
        }
    }
    groups
}

/// Convert a NormalWeightMode to a string.
#[allow(dead_code)]
pub fn normal_weight_to_string(mode: NormalWeightMode) -> &'static str {
    match mode {
        NormalWeightMode::Area => "area",
        NormalWeightMode::Angle => "angle",
        NormalWeightMode::Uniform => "uniform",
    }
}

/// Return the default normal weight mode.
#[allow(dead_code)]
pub fn normal_weight_default() -> NormalWeightMode {
    NormalWeightMode::Area
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    normalize([
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ])
}

fn face_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

fn angle_at_vertex(vertex: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let va = normalize([a[0] - vertex[0], a[1] - vertex[1], a[2] - vertex[2]]);
    let vb = normalize([b[0] - vertex[0], b[1] - vertex[1], b[2] - vertex[2]]);
    let d = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    d.clamp(-1.0, 1.0).acos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_area_weighted_normal() {
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let areas = vec![1.0, 2.0];
        let n = area_weighted_normal(&normals, &areas);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_angle_weighted_normal() {
        let normals = vec![[0.0, 0.0, 1.0]];
        let angles = vec![PI / 3.0];
        let n = angle_weighted_normal(&normals, &angles);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_uniform_weighted_normal() {
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let n = uniform_weighted_normal(&normals);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_uniform_weighted_empty() {
        let n = uniform_weighted_normal(&[]);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_weighted_normals() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2]];
        let normals = compute_weighted_normals(&verts, &faces, NormalWeightMode::Area);
        assert_eq!(normals.len(), 3);
        assert!((normals[0][2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_normal_weight_compare_same() {
        let angle = normal_weight_compare([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert!(angle.abs() < 1e-4);
    }

    #[test]
    fn test_normal_weight_compare_perpendicular() {
        let angle = normal_weight_compare([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_normal_weight_to_string() {
        assert_eq!(normal_weight_to_string(NormalWeightMode::Area), "area");
        assert_eq!(normal_weight_to_string(NormalWeightMode::Angle), "angle");
    }

    #[test]
    fn test_normal_weight_default() {
        assert_eq!(normal_weight_default(), NormalWeightMode::Area);
    }

    #[test]
    fn test_smoothing_groups() {
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]];
        let faces = vec![[0, 1, 2], [0, 1, 2], [0, 1, 2]];
        let groups = normal_smoothing_group(&normals, &faces, 0.5);
        assert_eq!(groups[0], groups[1]); // same normal => same group
    }
}
